//! Volumetric fog (docs/design.md §2.6): fog lit as a participating medium.
//!
//! [`crate::RenderScene::volumetric_fog`] replaces the frame's plain fog
//! (`RenderScene::fog`) while it is set, for a perspective camera above
//! water. The medium (volumetric.wgsl): a density per metre plus height
//! fog, scattering its colour times `scattering` of the light it is lit by
//! with a Henyey-Greenstein phase, out to `distance` metres of view depth.
//!
//! - Full tier ([`VolumetricMethod::Froxel`]): the graph pass `volumetric`
//!   (two compute dispatches after the shadow passes) lights a 160 × 90 ×
//!   64 froxel grid over the view by the sun through its cascades and by
//!   the clustered point, spot and rect lights through their shadows and
//!   cookies, then sums it front to back. Each froxel's light is the mean
//!   of eight stratified samples over its slice's depth, jittered per
//!   froxel and per frame: a slice is metres deep far out, and a single
//!   sample there landed in or out of a shaft column by column (vertical
//!   streaks).
//! - WebGL2 ([`VolumetricMethod::Raymarch`]): no compute; the apply pass
//!   marches each pixel's view ray in 24 jittered steps, lit by the sun
//!   through its cascades and by the first [`RAYMARCH_LIGHTS`] point, spot
//!   and rect lights of each step's cluster through their shadows and
//!   cookies (a lantern lights its fog on WebGL2 too).
//!
//! Either way the last thing the `main` pass does is blend the result over
//! the lit image (colour × transmittance + in-scattered light) at each
//! pixel's prepass depth, so opaque surfaces, the sky (at `distance`) and
//! whatever was blended in front of them are all seen through the volume.

use crate::clusters::LightTables;
use glam::Vec3;

/// Fog lit as a participating medium ([`crate::RenderScene::volumetric_fog`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumetricFog {
    /// The medium's colour (its albedo's tint).
    pub color: Vec3,
    /// Extinction per metre everywhere.
    pub density: f32,
    /// The height below which the height fog is densest.
    pub height: f32,
    /// Extra extinction per metre at and below `height`.
    pub height_density: f32,
    /// How fast the height fog thins above `height`, per metre.
    pub falloff: f32,
    /// The share of the light a scattering event sends on (0 dims only).
    pub scattering: f32,
    /// Henyey-Greenstein g: towards 1 forward (shafts against the light).
    pub anisotropy: f32,
    /// How far the volume reaches, in view depth.
    pub distance: f32,
}

impl Default for VolumetricFog {
    fn default() -> Self {
        VolumetricFog {
            color: Vec3::ONE,
            density: 0.02,
            height: 0.0,
            height_density: 0.0,
            falloff: 0.5,
            scattering: 0.8,
            anisotropy: 0.6,
            distance: 48.0,
        }
    }
}

/// How a renderer draws volumetric fog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumetricMethod {
    Froxel,
    Raymarch,
}

/// The froxel grid: across, down, deep.
pub const FROXELS: [u32; 3] = [160, 90, 64];
/// The raymarch's steps per pixel.
pub const RAYMARCH_STEPS: u32 = 24;
/// The most point, spot and rect lights of a step's cluster that light the
/// raymarch's fog (the first of the cluster's, which keeps the brightest
/// at the camera first): each step evaluates them with their shadows and
/// cookies, so 24 steps × 8 lights is the WebGL2 fragment's bound.
pub const RAYMARCH_LIGHTS: u32 = 8;

/// The lights one sample of the fog is lit by, by method
/// ([`crate::RenderInfo::volumetric_lights`]).
pub(crate) fn lights(method: VolumetricMethod) -> u32 {
    match method {
        VolumetricMethod::Froxel => crate::MAX_CLUSTER_LIGHTS as u32,
        VolumetricMethod::Raymarch => RAYMARCH_LIGHTS,
    }
}
const FROXEL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct VolUniform {
    scatter: [f32; 4],
    medium: [f32; 4],
    extent: [f32; 4],
    grid: [f32; 4],
}

struct Froxels {
    frame_layout: wgpu::BindGroupLayout,
    inject_layout: wgpu::BindGroupLayout,
    integrate_layout: wgpu::BindGroupLayout,
    inject: wgpu::ComputePipeline,
    integrate: wgpu::ComputePipeline,
    scatter: wgpu::TextureView,
    sum: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

/// The renderer's volumetric fog: made on the first frame that has it.
pub(crate) struct Volumetric {
    pub method: VolumetricMethod,
    tables: LightTables,
    uniform: wgpu::Buffer,
    froxels: Option<Froxels>,
    apply_layout: wgpu::BindGroupLayout,
    apply: Option<wgpu::RenderPipeline>,
}

/// A bind group of `(binding, resource)` pairs (layouts whose bindings
/// skip numbers).
pub(crate) fn bind_at(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    resources: &[(u32, wgpu::BindingResource)],
) -> wgpu::BindGroup {
    let entries: Vec<_> = resources
        .iter()
        .map(|(binding, r)| wgpu::BindGroupEntry {
            binding: *binding,
            resource: r.clone(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

fn storage_3d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: FROXEL_FORMAT,
            view_dimension: wgpu::TextureViewDimension::D3,
        },
        count: None,
    }
}

fn entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

fn uniform_ty() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn texture_ty(dim: wgpu::TextureViewDimension, filterable: bool) -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable },
        view_dimension: dim,
        multisampled: false,
    }
}

/// The module's source for a tier (`full`: the froxel entry points).
pub(crate) fn source(tables: LightTables, full: bool) -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::FRAME_WGSL,
        tables.wgsl(),
        crate::LIGHTING_WGSL,
        crate::AREA_WGSL,
        wgsl!("volumetric.wgsl"),
        if full {
            wgsl!("volumetric_froxel.wgsl").to_string()
        } else {
            format!(
                "const RAYMARCH_LIGHTS: u32 = {RAYMARCH_LIGHTS}u;\n{}",
                wgsl!("volumetric_raymarch.wgsl")
            )
        }
    )
}

impl Volumetric {
    pub fn new(device: &wgpu::Device, tables: LightTables, full: bool) -> Volumetric {
        let method = if full {
            VolumetricMethod::Froxel
        } else {
            VolumetricMethod::Raymarch
        };
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let mut entries = vec![
            entry(0, fragment | wgpu::ShaderStages::COMPUTE, uniform_ty()),
            entry(
                4,
                fragment,
                texture_ty(wgpu::TextureViewDimension::D2, false),
            ),
        ];
        if full {
            entries.push(entry(
                5,
                fragment,
                texture_ty(wgpu::TextureViewDimension::D3, true),
            ));
            entries.push(entry(
                6,
                fragment,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            ));
        }
        let apply_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric apply"),
            entries: &entries,
        });
        Volumetric {
            method,
            tables,
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("volumetric"),
                size: std::mem::size_of::<VolUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            froxels: None,
            apply_layout,
            apply: None,
        }
    }

    /// Makes the pipelines on first use; `frame_layout`: the lit frame group's.
    fn ensure(&mut self, device: &wgpu::Device, frame_layout: &wgpu::BindGroupLayout) {
        if self.apply.is_some() {
            return;
        }
        let full = self.method == VolumetricMethod::Froxel;
        let module = crate::shader(device, "volumetric", source(self.tables, full));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("volumetric apply"),
            bind_group_layouts: &[Some(frame_layout), Some(&self.apply_layout)],
            immediate_size: 0,
        });
        self.apply = Some(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("volumetric apply"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_volumetric"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(if full { "fs_froxel" } else { "fs_raymarch" }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: crate::HDR_FORMAT,
                        // colour × transmittance + in-scattered light.
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::SrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            }),
        );
        if !full {
            return;
        }
        let compute = wgpu::ShaderStages::COMPUTE;
        let table_ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let frame_layout_c = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric frame"),
            entries: &[
                entry(0, compute, uniform_ty()),
                entry(
                    1,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D2Array, false),
                ),
                entry(
                    2,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D2Array, false),
                ),
                entry(7, compute, table_ty),
                entry(8, compute, table_ty),
                entry(9, compute, table_ty),
            ],
        });
        let inject_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric inject"),
            entries: &[entry(0, compute, uniform_ty()), storage_3d(1)],
        });
        let integrate_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("volumetric integrate"),
            entries: &[
                entry(0, compute, uniform_ty()),
                entry(
                    2,
                    compute,
                    texture_ty(wgpu::TextureViewDimension::D3, false),
                ),
                storage_3d(3),
            ],
        });
        let pipeline = |label, group: &wgpu::BindGroupLayout, entry_point| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(&frame_layout_c), Some(group)],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let inject = pipeline("volumetric inject", &inject_layout, "inject");
        let integrate = pipeline("volumetric integrate", &integrate_layout, "integrate");
        let volume = |label| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: FROXELS[0],
                        height: FROXELS[1],
                        depth_or_array_layers: FROXELS[2],
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D3,
                    format: FROXEL_FORMAT,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        self.froxels = Some(Froxels {
            frame_layout: frame_layout_c,
            inject_layout,
            integrate_layout,
            inject,
            integrate,
            scatter: volume("froxel scatter"),
            sum: volume("froxel sum"),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("froxels"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
        });
    }

    /// Writes the frame's medium; `size`: the HDR target's.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_layout: &wgpu::BindGroupLayout,
        fog: &VolumetricFog,
        size: (u32, u32),
        tick: u64,
    ) {
        self.ensure(device, frame_layout);
        let u = VolUniform {
            scatter: (fog.color.max(Vec3::ZERO) * fog.scattering.max(0.0))
                .extend(fog.anisotropy.clamp(-0.95, 0.95))
                .to_array(),
            medium: [
                fog.density.max(0.0),
                fog.height,
                fog.height_density.max(0.0),
                fog.falloff.max(1e-4),
            ],
            extent: [
                fog.distance.max(1.0),
                RAYMARCH_STEPS as f32,
                size.0 as f32,
                size.1 as f32,
            ],
            grid: [
                FROXELS[0] as f32,
                FROXELS[1] as f32,
                FROXELS[2] as f32,
                (tick % 64) as f32,
            ],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
    }

    /// The froxel dispatches (full tier; nothing on WebGL2): after the
    /// shadow passes, whose maps they read.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_froxels(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &wgpu::Buffer,
        cascades: &wgpu::TextureView,
        locals: &wgpu::TextureView,
        tables: [&wgpu::Buffer; 3],
    ) {
        let Some(f) = &self.froxels else {
            return;
        };
        let frame_group = bind_at(
            device,
            "volumetric frame",
            &f.frame_layout,
            &[
                (0, frame.as_entire_binding()),
                (1, wgpu::BindingResource::TextureView(cascades)),
                (2, wgpu::BindingResource::TextureView(locals)),
                (7, tables[0].as_entire_binding()),
                (8, tables[1].as_entire_binding()),
                (9, tables[2].as_entire_binding()),
            ],
        );
        let inject = bind_at(
            device,
            "volumetric inject",
            &f.inject_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (1, wgpu::BindingResource::TextureView(&f.scatter)),
            ],
        );
        let integrate = bind_at(
            device,
            "volumetric integrate",
            &f.integrate_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (2, wgpu::BindingResource::TextureView(&f.scatter)),
                (3, wgpu::BindingResource::TextureView(&f.sum)),
            ],
        );
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("volumetric"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &frame_group, &[]);
        pass.set_pipeline(&f.inject);
        pass.set_bind_group(1, &inject, &[]);
        pass.dispatch_workgroups(FROXELS[0].div_ceil(8), FROXELS[1].div_ceil(8), FROXELS[2]);
        pass.set_pipeline(&f.integrate);
        pass.set_bind_group(1, &integrate, &[]);
        pass.dispatch_workgroups(FROXELS[0].div_ceil(8), FROXELS[1].div_ceil(8), 1);
    }

    /// Blends the volume over `hdr` at the depths of `depth` (the prepass's
    /// view depth).
    pub fn encode_apply(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame_group: &wgpu::BindGroup,
        hdr: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let Some(pipeline) = &self.apply else {
            return;
        };
        let mut resources = vec![
            (0, self.uniform.as_entire_binding()),
            (4, wgpu::BindingResource::TextureView(depth)),
        ];
        if let Some(f) = &self.froxels {
            resources.push((5, wgpu::BindingResource::TextureView(&f.sum)));
            resources.push((6, wgpu::BindingResource::Sampler(&f.sampler)));
        }
        let group = bind_at(device, "volumetric apply", &self.apply_layout, &resources);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("volumetric apply"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: hdr,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, frame_group, &[]);
        pass.set_bind_group(1, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}
