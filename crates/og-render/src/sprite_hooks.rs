//! Sprites drawn with a material shader ([`crate::SpriteShading`]): a
//! sprite given a material whose shader is of [`ShaderMode::Hooks`] or
//! [`ShaderMode::Sprite`] draws through pipelines built from the sprite
//! module with the shader's hooks (`sprite_hooks.wgsl`), the material's
//! parameters in a uniform of its own at group 3. The same shader asset,
//! material and parameters as a mesh's (og.load.shader,
//! og.render.shaderMaterial, og.render.setShaderParam): a dissolve written
//! for meshes dissolves a sprite. Sprites of one material, texture and
//! blend batch into one draw. Made on the first frame that draws one, so
//! a renderer that never does pays nothing, and every other sprite draws
//! through the sprite pipelines as before.

use crate::effects::SpriteBlend;
use crate::hooks::{MAX_SHADER_PARAMS, MaterialShader};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// The module's source as the renderer compiles it for `shader`.
pub(crate) fn source(shader: &MaterialShader) -> String {
    format!(
        "{}{}{}{}",
        crate::sprite_source(),
        crate::particles_gpu::NOISE_WGSL,
        crate::hooks::hook_functions(&shader.params, &shader.code),
        wgsl!("sprite_hooks.wgsl")
    )
}

/// A sprite shader using every hook it runs, two parameters and
/// `sprite_texel`: the module the gate validates (`shaders.rs`).
pub(crate) fn example() -> MaterialShader {
    use crate::hooks::{ParamKind, ShaderMode, ShaderParam};
    MaterialShader {
        name: "sprite example".into(),
        code: "fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  return vec3<f32>(0.0, sin(h.time + h.local.x * 6.0) * p.amount * 0.05, 0.0);
}
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let shifted = sprite_texel(h.uv + vec2<f32>(0.01 * p.amount, 0.0));
  return vec4<f32>(mix(c.rgb, shifted.rgb * p.tint.rgb, p.amount), c.a);
}
fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> {
  return e + p.tint.rgb * p.amount * h.color.a;
}
"
        .into(),
        params: vec![
            ShaderParam {
                name: "amount".into(),
                kind: ParamKind::Float,
                default: [0.5, 0.0, 0.0, 0.0],
            },
            ShaderParam {
                name: "tint".into(),
                kind: ParamKind::Vec4,
                default: [0.4, 0.8, 1.0, 1.0],
            },
        ],
        mode: ShaderMode::Sprite,
    }
}

/// `struct SpriteMaterial` in sprite_hooks.wgsl.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    custom: [[f32; 4]; MAX_SHADER_PARAMS],
}

/// The device's answer on a pipeline's error scope, while it has not
/// given it (the web answers on a later frame).
type Pending = std::pin::Pin<Box<dyn std::future::Future<Output = Option<wgpu::Error>>>>;

/// One shader's pipelines: alpha, additive and untonemapped, or why the
/// device refused them.
struct Pipelines {
    key: u64,
    sprite: Result<[wgpu::RenderPipeline; 3], String>,
    pending: Option<Pending>,
}

/// Polls `pending` once: `Err` when the device refused, `Ok(None)` when it
/// accepted, `Ok(Some)` while it has not answered.
fn poll(mut pending: Pending) -> Result<Option<Pending>, String> {
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match pending.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(Some(e)) => Err(e.to_string()),
        std::task::Poll::Ready(None) => Ok(None),
        std::task::Poll::Pending => Ok(Some(pending)),
    }
}

/// One material's parameters and group.
struct Group {
    params: [[f32; 4]; MAX_SHADER_PARAMS],
    buffer: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// The layouts, pipelines by shader and parameter groups by material.
pub(crate) struct SpriteHooks {
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    samples: u32,
    pipelines: HashMap<usize, Pipelines>,
    groups: HashMap<usize, Group>,
}

/// What a shader's pipelines are built from: its code, parameters and
/// mode (a reloaded shader at the same index builds again).
fn key_of(shader: &MaterialShader) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    shader.code.hash(&mut h);
    shader.mode.hash(&mut h);
    for p in &shader.params {
        p.name.hash(&mut h);
        p.kind.hash(&mut h);
    }
    h.finish()
}

impl SpriteHooks {
    /// `layouts`: the sprite pipelines' frame, texture and light-map
    /// groups (0, 1, 2).
    pub fn new(
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        samples: u32,
    ) -> SpriteHooks {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite material"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite (material shader)"),
            bind_group_layouts: &[
                Some(layouts[0]),
                Some(layouts[1]),
                Some(layouts[2]),
                Some(&layout),
            ],
            immediate_size: 0,
        });
        SpriteHooks {
            layout,
            pipeline_layout,
            samples,
            pipelines: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Builds shader `index`'s pipelines unless they are current.
    pub fn prepare_shader(&mut self, device: &wgpu::Device, index: usize, shader: &MaterialShader) {
        let key = key_of(shader);
        if let Some(p) = self.pipelines.get_mut(&index)
            && p.key == key
        {
            if let Some(pending) = p.pending.take() {
                match poll(pending) {
                    Ok(pending) => p.pending = pending,
                    Err(e) => p.sprite = Err(e),
                }
            }
            return;
        }
        if let Err(e) = shader.check() {
            self.pipelines.insert(
                index,
                Pipelines {
                    key,
                    sprite: Err(e),
                    pending: None,
                },
            );
            return;
        }
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite (material shader)"),
            source: wgpu::ShaderSource::Wgsl(source(shader).into()),
        });
        let pipeline = |label, fs, blend| {
            crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label,
                    layout: &self.pipeline_layout,
                    module: &module,
                    vs: "vs_hooked",
                    fs,
                    buffers: crate::SPRITE_BUFFERS,
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples: self.samples,
                },
            )
        };
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let sprite = [
            pipeline(
                "sprite (material shader)",
                "fs_hooked",
                wgpu::BlendState::ALPHA_BLENDING,
            ),
            pipeline("sprite additive (material shader)", "fs_hooked", additive),
            pipeline(
                "sprite untonemapped (material shader)",
                "fs_hooked_untonemapped",
                crate::UNTONEMAPPED_BLEND,
            ),
        ];
        let (sprite, pending) = match poll(Box::pin(scope.pop())) {
            Ok(pending) => (Ok(sprite), pending),
            Err(e) => (Err(e), None),
        };
        self.pipelines.insert(
            index,
            Pipelines {
                key,
                sprite,
                pending,
            },
        );
    }

    /// The shaders whose sprite pipelines the device refused, by shader
    /// index, with its message.
    pub fn errors(&self) -> Vec<(usize, String)> {
        let mut out: Vec<_> = self
            .pipelines
            .iter()
            .filter_map(|(i, p)| p.sprite.as_ref().err().map(|e| (*i, e.clone())))
            .collect();
        out.sort();
        out
    }

    /// Material `index`'s group with `params`, written when they changed.
    pub fn prepare_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        index: usize,
        params: [[f32; 4]; MAX_SHADER_PARAMS],
    ) {
        // Sprites write no motion vectors: no previous parameters.
        let uniform = Uniform { custom: params };
        if let Some(g) = self.groups.get_mut(&index) {
            if g.params != params {
                queue.write_buffer(&g.buffer, 0, bytemuck::bytes_of(&uniform));
                g.params = params;
            }
            return;
        }
        let buffer = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("sprite material"),
                contents: bytemuck::bytes_of(&uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite material"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.groups.insert(
            index,
            Group {
                params,
                buffer,
                group,
            },
        );
    }

    /// Shader `shader`'s pipeline for `blend`, tone mapped or not: none
    /// while the device has not accepted it, or when it refused it.
    pub fn pipeline(
        &self,
        shader: usize,
        blend: SpriteBlend,
        tone_mapped: bool,
    ) -> Option<&wgpu::RenderPipeline> {
        let p = self
            .pipelines
            .get(&shader)
            .filter(|p| p.pending.is_none())?;
        let p = p.sprite.as_ref().ok()?;
        Some(match (blend, tone_mapped) {
            (SpriteBlend::Alpha, true) => &p[0],
            (SpriteBlend::Additive, _) => &p[1],
            (SpriteBlend::Alpha, false) => &p[2],
        })
    }

    /// Material `index`'s parameter group.
    pub fn group(&self, material: usize) -> Option<&wgpu::BindGroup> {
        self.groups.get(&material).map(|g| &g.group)
    }
}
