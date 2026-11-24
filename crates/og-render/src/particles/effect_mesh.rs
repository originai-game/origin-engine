//! Dedicated instanced [`crate::EffectMesh`] renderer: batches effect meshes
//! sharing `(mesh, material, blend, double_sided, hook, layers)` into single
//! instanced `draw_indexed` calls with full OIT support and zero per-mesh
//! particle uniform overhead.

use crate::RenderError;
use crate::deferred::Deferred;
use crate::particles::{ParticleBlend, hash_seed};
use crate::particles_gpu::{MESH_LAYOUT, ParticlePart};
use crate::scene::RenderScene;
use glam::Vec3;
use og_render_data::shader_source::FRAME_WGSL;
use std::collections::HashMap;

const EFFECT_MESH_WGSL: &str = wgsl!("effect_mesh.wgsl");
const COMMON_WGSL: &str = wgsl!("particle_common.wgsl");

/// Per-instance vertex data for [`EffectMeshRenderer`] (`EffectMeshInst` in
/// `effect_mesh.wgsl`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct EffectMeshInstance {
    pub col0: [f32; 4],
    pub col1: [f32; 4],
    pub col2: [f32; 4],
    pub col3: [f32; 4],
    pub color: [f32; 4],
    /// x: life (0..1), y: hue (rad), z: blend id, w: soft enabled (0 or 1).
    pub life_hue: [f32; 4],
    /// x: hashed seed, y: flags (bit 0: billboard), z/w: pad.
    pub seed_flags: [u32; 4],
}

const INSTANCE_STRIDE: u64 = std::mem::size_of::<EffectMeshInstance>() as u64;

pub(crate) const EFFECT_MESH_INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> =
    wgpu::VertexBufferLayout {
        array_stride: INSTANCE_STRIDE,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 80,
                shader_location: 9,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32x4,
                offset: 96,
                shader_location: 10,
            },
        ],
    };

pub(crate) const EFFECT_MESH_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(EFFECT_MESH_INSTANCE_LAYOUT), Some(MESH_LAYOUT)];

/// Leaves the HDR target's alpha (`tonemapped` mask) untouched.
const KEEP_ALPHA: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Zero,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Add,
};

/// One instanced draw batch of effect meshes sharing state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EffectMeshBatch {
    pub mesh: crate::MeshHandle,
    pub material: crate::EffectMaterialHandle,
    pub blend: ParticleBlend,
    pub double_sided: bool,
    pub hook: Option<usize>,
    pub layers: u32,
    pub depth: f32,
    pub first: u32,
    pub count: u32,
}

impl EffectMeshBatch {
    pub(crate) fn in_oit(&self) -> bool {
        self.blend == ParticleBlend::Alpha
    }

    pub(crate) fn oit_source(&self) -> crate::oit::Source {
        crate::oit::Source::EffectMesh(self.double_sided, self.hook)
    }
}

/// The instanced effect meshes prepared for this frame.
#[derive(Default)]
pub(crate) struct EffectMeshFrame {
    pub batches: Vec<EffectMeshBatch>,
}

/// WGSL source for the standalone instanced effect mesh module.
pub(crate) fn source(hook: Option<&crate::MaterialShader>) -> String {
    format!(
        "{FRAME_WGSL}{}{}{COMMON_WGSL}{}{}{EFFECT_MESH_WGSL}",
        crate::oit::OIT_WGSL,
        crate::oit::ARENA_WGSL,
        crate::effect::EFFECT_WGSL,
        crate::effect::hooks_source(hook),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BatchKey {
    layers: u32,
    blend: u8,
    double_sided: bool,
    hook: Option<usize>,
    material: usize,
    mesh: usize,
}

pub(crate) struct EffectMeshRenderer {
    pub(crate) module: crate::deferred::Module,
    hook_modules: HashMap<usize, wgpu::ShaderModule>,
    pipelines: HashMap<(ParticleBlend, u32, bool, Option<usize>), Deferred<wgpu::RenderPipeline>>,
    reachable: Vec<(ParticleBlend, bool)>,
    instances: wgpu::Buffer,
    scratch: Vec<EffectMeshInstance>,
}

fn instance_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some("effect mesh instances"),
            size: size.max(INSTANCE_STRIDE * 16),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        },
    )
}

impl EffectMeshRenderer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let module = crate::deferred::module(device, "effect meshes", source(None));
        EffectMeshRenderer {
            module,
            hook_modules: HashMap::new(),
            pipelines: HashMap::new(),
            reachable: Vec::new(),
            instances: instance_buffer(device, INSTANCE_STRIDE * 64),
            scratch: Vec::new(),
        }
    }

    /// Returns the shader module for `hook` (creating and caching hooked
    /// modules on first use), shared with OIT pipeline creation.
    pub(crate) fn shader_module(
        &mut self,
        device: &wgpu::Device,
        hook: Option<usize>,
        shaders: &[crate::MaterialShader],
    ) -> wgpu::ShaderModule {
        match hook {
            None => self.module.get().clone(),
            Some(h) => self
                .hook_modules
                .entry(h)
                .or_insert_with(|| {
                    crate::gpu::shader_module(
                        device,
                        wgpu::ShaderModuleDescriptor {
                            label: Some("effect meshes (hook)"),
                            source: wgpu::ShaderSource::Wgsl(source(shaders.get(h)).into()),
                        },
                    )
                })
                .clone(),
        }
    }

    /// Prepares instanced batches from `scene.effects.effect_meshes` and
    /// uploads all instance records in one write.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        oit_order_free: bool,
        eye: Vec3,
        forward: Vec3,
        meshes_out: &mut Vec<usize>,
        needs_depth: &mut bool,
        soft_out: &mut bool,
    ) -> Result<EffectMeshFrame, RenderError> {
        self.scratch.clear();
        let src = &scene.effects.effect_meshes;
        if src.is_empty() {
            return Ok(EffectMeshFrame::default());
        }
        let diagnostic = scene.frame.debug != crate::DebugView::None;
        let soft_quality = scene.frame.quality.soft_particles;
        let mut items: Vec<(BatchKey, bool, f32, EffectMeshInstance)> =
            Vec::with_capacity(src.len());

        for (i, m) in src.iter().enumerate() {
            let material = scene.effect_material(m.material).ok_or_else(|| {
                RenderError::InvalidHandle(format!(
                    "effect mesh {i}: effect material {}",
                    m.material.index()
                ))
            })?;
            if scene.mesh(m.mesh).is_none() {
                return Err(RenderError::InvalidHandle(format!(
                    "effect mesh {i}: mesh {}",
                    m.mesh.index()
                )));
            }
            meshes_out.push(m.mesh.index());
            let soft = soft_quality && material.soft_depth > 0.0;
            *needs_depth |= soft;
            *soft_out |= soft;
            let blend = if diagnostic {
                ParticleBlend::Alpha
            } else {
                material.blend.unwrap_or(m.blend)
            };
            let blend_id = match blend {
                ParticleBlend::Additive => 0.0,
                ParticleBlend::Alpha => 1.0,
                ParticleBlend::Subtractive => 2.0,
                ParticleBlend::Multiply => 3.0,
            };
            let cols = m.transform.to_cols_array_2d();
            let depth = (m.transform.w_axis.truncate() - eye).dot(forward);
            let order_sensitive = blend == ParticleBlend::Alpha && !oit_order_free;
            let key = BatchKey {
                layers: m.layers,
                blend: blend_order(blend),
                double_sided: material.double_sided,
                hook: material.shader.map(|h| h.index()),
                material: m.material.index(),
                mesh: m.mesh.index(),
            };
            let inst = EffectMeshInstance {
                col0: cols[0],
                col1: cols[1],
                col2: cols[2],
                col3: cols[3],
                color: m.color,
                life_hue: [
                    m.life.clamp(0.0, 1.0),
                    m.hue,
                    blend_id,
                    u32::from(soft) as f32,
                ],
                seed_flags: [hash_seed(m.seed), u32::from(m.billboard), 0, 0],
            };
            items.push((key, order_sensitive, depth, inst));
        }

        // Order-independent batches (Additive, Sub, Mul, and OIT Alpha) group
        // aggressively by state key so identical meshes collapse into 1 draw.
        // Non-OIT Alpha sorts back-to-front by depth and merges adjacent runs.
        og_core::sort::sort_by(&mut items, |a, b| match (a.1, b.1) {
            (true, true) => b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.0.cmp(&b.0).then(b.2.total_cmp(&a.2)),
        });

        let mut batches: Vec<EffectMeshBatch> = Vec::new();
        for (key, _, depth, inst) in items {
            let idx = self.scratch.len() as u32;
            self.scratch.push(inst);
            let blend = blend_from_order(key.blend);
            match batches.last_mut() {
                Some(b)
                    if b.layers == key.layers
                        && b.blend == blend
                        && b.double_sided == key.double_sided
                        && b.hook == key.hook
                        && b.material.index() == key.material
                        && b.mesh.index() == key.mesh =>
                {
                    b.count += 1;
                    b.depth = b.depth.max(depth);
                }
                _ => batches.push(EffectMeshBatch {
                    mesh: crate::MeshHandle::from_index(key.mesh),
                    material: crate::EffectMaterialHandle(key.material as u32),
                    blend,
                    double_sided: key.double_sided,
                    hook: key.hook,
                    layers: key.layers,
                    depth,
                    first: idx,
                    count: 1,
                }),
            }
        }

        let bytes: &[u8] = bytemuck::cast_slice(&self.scratch);
        if bytes.len() as u64 > self.instances.size() {
            self.instances = instance_buffer(device, (bytes.len() as u64).next_power_of_two());
        }
        crate::traffic::write_buffer(queue, &self.instances, 0, bytes);
        Ok(EffectMeshFrame { batches })
    }

    pub(crate) fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        frame: &EffectMeshFrame,
        samples: u32,
        shaders: &[crate::MaterialShader],
    ) {
        for b in &frame.batches {
            let key = (b.blend, samples, b.double_sided, b.hook);
            if !self.pipelines.contains_key(&key) {
                self.make_pipeline(device, layout, key, shaders);
            }
        }
    }

    pub(crate) fn note_loaded(&mut self, materials: &[crate::EffectMaterial]) {
        self.reachable.clear();
        for m in materials {
            if m.shader.is_some() {
                continue;
            }
            let blends: &[ParticleBlend] = match m.blend {
                Some(b) => std::slice::from_ref(match b {
                    ParticleBlend::Additive => &ParticleBlend::Additive,
                    ParticleBlend::Alpha => &ParticleBlend::Alpha,
                    ParticleBlend::Subtractive => &ParticleBlend::Subtractive,
                    ParticleBlend::Multiply => &ParticleBlend::Multiply,
                }),
                None => &[ParticleBlend::Additive, ParticleBlend::Alpha],
            };
            for &b in blends {
                let k = (b, m.double_sided);
                if !self.reachable.contains(&k) {
                    self.reachable.push(k);
                }
            }
        }
    }

    pub(crate) fn warm_loaded(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        samples: u32,
    ) -> bool {
        let next = self
            .reachable
            .iter()
            .copied()
            .find(|&(b, ds)| !self.pipelines.contains_key(&(b, samples, ds, None)));
        match next {
            Some((blend, double_sided)) => {
                self.make_pipeline(device, layout, (blend, samples, double_sided, None), &[]);
                true
            }
            None => false,
        }
    }

    fn make_pipeline(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        key: (ParticleBlend, u32, bool, Option<usize>),
        shaders: &[crate::MaterialShader],
    ) {
        let (blend_of, samples, double_sided, hook) = key;
        let module = self.shader_module(device, hook, shaders);
        let blend = match blend_of {
            ParticleBlend::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: KEEP_ALPHA,
            },
            ParticleBlend::Alpha => wgpu::BlendState {
                color: wgpu::BlendComponent::OVER,
                alpha: KEEP_ALPHA,
            },
            ParticleBlend::Subtractive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: KEEP_ALPHA,
            },
            ParticleBlend::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: KEEP_ALPHA,
            },
        };
        let (device, layout) = (device.clone(), layout.clone());
        let p = Deferred::anywhere(move || {
            crate::hdr_pipeline(
                &device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label: "effect meshes",
                    layout: &layout,
                    module: &module,
                    vs: "vs_effect_mesh",
                    fs: "fs_effect_mesh",
                    buffers: EFFECT_MESH_BUFFERS,
                    cull: (!double_sided).then_some(wgpu::Face::Back),
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        });
        crate::deferred::Start::start(&p);
        self.pipelines.insert(key, p);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &EffectMeshFrame,
        samples: u32,
        group: &wgpu::BindGroup,
        white_sprite: &'b wgpu::BindGroup,
        effects: &crate::effect::EffectRenderer,
        mask: u32,
        part: ParticlePart,
        oit_lookup: &impl Fn(crate::oit::Source) -> Option<&'b wgpu::RenderPipeline>,
        mesh: &impl Fn(crate::MeshHandle) -> Option<(wgpu::BufferSlice<'b>, &'b wgpu::Buffer, u32)>,
    ) -> u32 {
        if frame.batches.is_empty() || matches!(part, ParticlePart::Before | ParticlePart::Half) {
            return 0;
        }
        let mut draws = 0;
        let mut bound_shared = false;
        for b in &frame.batches {
            if b.layers & mask == 0 {
                continue;
            }
            let wants = match part {
                ParticlePart::All | ParticlePart::Full => true,
                ParticlePart::Before | ParticlePart::Half => false,
                ParticlePart::Oit => b.in_oit(),
                ParticlePart::After(_) => !b.in_oit(),
            };
            if !wants {
                continue;
            }
            let Some((vertices, indices, index_count)) = mesh(b.mesh) else {
                continue;
            };
            let pipeline = match part {
                ParticlePart::Oit => oit_lookup(b.oit_source()),
                _ => self
                    .pipelines
                    .get(&(b.blend, samples, b.double_sided, b.hook))
                    .and_then(|p| p.get()),
            };
            let Some(pipeline) = pipeline else {
                continue;
            };
            pass.set_pipeline(pipeline);
            if !bound_shared {
                pass.set_bind_group(1, white_sprite, &[]);
                pass.set_bind_group(2, group, &[0]);
                pass.set_vertex_buffer(0, self.instances.slice(..));
                bound_shared = true;
            }
            let (fx_group, offset) = effects.bind(Some(b.material));
            pass.set_bind_group(3, fx_group, &[offset]);
            pass.set_vertex_buffer(1, vertices);
            pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..index_count, 0, b.first..b.first + b.count);
            draws += 1;
        }
        draws
    }
}

fn blend_order(b: ParticleBlend) -> u8 {
    match b {
        ParticleBlend::Alpha => 0,
        ParticleBlend::Multiply => 1,
        ParticleBlend::Subtractive => 2,
        ParticleBlend::Additive => 3,
    }
}

fn blend_from_order(v: u8) -> ParticleBlend {
    match v {
        0 => ParticleBlend::Alpha,
        1 => ParticleBlend::Multiply,
        2 => ParticleBlend::Subtractive,
        _ => ParticleBlend::Additive,
    }
}
