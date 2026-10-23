//! Area lights and light cookies (docs/design.md §2.6).
//!
//! A [`crate::LightKind::Rect`] light is clustered with the point and spot
//! lights and shaded by area.wgsl with linearly transformed cosines (the
//! GGX fit of Heitz, Dupuy, Hill and Neubelt's `ltc_code`, BSD-licensed,
//! `third_party/ltc`, resampled from 64 × 64 to 32 × 32 half floats so the
//! table is one 16 KiB uniform: frame binding 11, no texture slot taken
//! from WebGL2's full skinned pipeline). Its record in the light table:
//! `color_kind.w` -1 one-sided, -2 two-sided; the half-width vector in
//! `direction_slot.xyz` (no shadow slot); the half-height in `cone.xyz`.
//! Shaders that do not know rect lights (terrain, scattered copies, 3D
//! fluids) light them as a point at their centre.
//!
//! Cookies ([`LightCookie`]) take layers of the local shadow array past the
//! shadow layers, each texel the cookie's linear colour packed 8:8:8 in an
//! exact integer (the array is R32Float, read with `textureLoad`, so no
//! texture slot either): one layer per spot cookie, six faces per point
//! cookie (the texture is six square faces stacked top to bottom, +X, -X,
//! +Y, -Y, +Z, -Z). A spot carries its layer in `color_kind.w` (2 + layer),
//! a point its first face in `cone.y` (-(layer + 1)). At most
//! [`MAX_COOKIE_LAYERS`] layers: cookies past them are not drawn.

use crate::frame::LightGpu;
use crate::scene::{LightKind, RenderScene, TextureHandle};

/// A light's cookie: the light by index into [`RenderScene::lights`] and
/// the texture projected from it (a point light's: six faces stacked).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightCookie {
    pub light: usize,
    pub texture: TextureHandle,
}

/// The most local shadow array layers cookies take.
pub const MAX_COOKIE_LAYERS: usize = 24;

const LTC: &[u8; 16384] = include_bytes!("../../../third_party/ltc/ltc32.bin");

/// The LTC table's uniform buffer (frame binding 11).
pub(crate) fn ltc_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("ltc table"),
            contents: LTC,
            usage: wgpu::BufferUsages::UNIFORM,
        },
    )
}

pub(crate) fn ltc_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 11,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// One cookie's place in the local shadow array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Placed {
    texture: TextureHandle,
    point: bool,
    /// Its first layer past the shadow layers.
    offset: u32,
    /// The scene's revision of the texture when it was written.
    revision: u64,
}

/// The cookies placed in the local shadow array, and the pass that writes
/// them: a fullscreen triangle per face sampling the texture as the
/// renderer uploaded it (RGBA8 or a compressed format alike), its colour
/// linear (an sRGB texture is sampled through its sRGB format) and packed.
#[derive(Default)]
pub(crate) struct Cookies {
    placed: Vec<Placed>,
    /// The array they were written to (its first layer) and its identity.
    written: Option<(u32, usize)>,
    pipeline: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Sampler)>,
}

/// Whether a cookie's light takes one (spot) or six (point) layers.
fn faces(scene: &RenderScene, c: &LightCookie) -> Option<u32> {
    match scene.lights.get(c.light)?.kind {
        LightKind::Point => Some(6),
        LightKind::Spot { .. } => Some(1),
        LightKind::Rect { .. } => None,
    }
}

/// The cookie pass's WGSL (validated with the others, shaders.rs).
pub(crate) fn cookie_source() -> String {
    format!("{}{}", crate::post::FULLSCREEN_WGSL, wgsl!("cookie.wgsl"))
}

impl Cookies {
    /// The layers this frame's cookies need.
    pub fn layers(scene: &RenderScene) -> u32 {
        let mut seen: Vec<(TextureHandle, bool)> = Vec::new();
        let mut layers = 0u32;
        for c in &scene.light_cookies {
            let Some(need) = faces(scene, c) else {
                continue;
            };
            let key = (c.texture, need == 6);
            if seen.contains(&key) || c.texture.index() >= scene.textures().len() {
                continue;
            }
            if layers + need > MAX_COOKIE_LAYERS as u32 {
                continue;
            }
            seen.push(key);
            layers += need;
        }
        layers
    }

    fn ensure(&mut self, device: &wgpu::Device) {
        if self.pipeline.is_some() {
            return;
        }
        let fragment = wgpu::ShaderStages::FRAGMENT;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cookie"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: fragment,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: fragment,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: fragment,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = crate::shader(device, "cookie", cookie_source());
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cookie"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cookie"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_cookie"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cookie"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        self.pipeline = Some((pipeline, layout, sampler));
    }

    /// Places this frame's cookies from layer `base` of the local shadow
    /// array (`texture`, its per-layer views `layers`) and writes those not
    /// already there; `view` is the uploaded view of a scene texture. `id`
    /// names the array: a new array is written afresh.
    pub fn prepare<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        (texture, layers): (&wgpu::Texture, &[wgpu::TextureView]),
        base: u32,
        view: impl Fn(TextureHandle) -> Option<&'a wgpu::TextureView>,
    ) {
        let id = texture as *const wgpu::Texture as usize;
        if self.written != Some((base, id)) {
            self.placed.clear();
            self.written = Some((base, id));
        }
        let mut placed: Vec<Placed> = Vec::new();
        let mut writes: Vec<(TextureHandle, u32, u32)> = Vec::new();
        let mut next = 0u32;
        for c in &scene.light_cookies {
            let Some(need) = faces(scene, c) else {
                continue;
            };
            let point = need == 6;
            if placed
                .iter()
                .any(|p| p.texture == c.texture && p.point == point)
            {
                continue;
            }
            let Some(&revision) = scene.texture_revisions().get(c.texture.index()) else {
                continue;
            };
            if next + need > MAX_COOKIE_LAYERS as u32 {
                continue;
            }
            let p = Placed {
                texture: c.texture,
                point,
                offset: next,
                revision,
            };
            next += need;
            if !self.placed.contains(&p) {
                writes.push((c.texture, base + p.offset, need));
            }
            placed.push(p);
        }
        self.placed = placed;
        if writes.is_empty() {
            return;
        }
        self.ensure(device);
        let (pipeline, layout, sampler) = self.pipeline.as_ref().expect("made above");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cookies"),
        });
        for (handle, first, faces) in writes {
            let Some(source) = view(handle) else {
                continue;
            };
            for face in 0..faces {
                let Some(target) = layers.get((first + face) as usize) else {
                    continue;
                };
                let face_uniform = wgpu::util::DeviceExt::create_buffer_init(
                    device,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("cookie face"),
                        contents: bytemuck::cast_slice(&[face as f32, faces as f32, 0.0, 0.0]),
                        usage: wgpu::BufferUsages::UNIFORM,
                    },
                );
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("cookie"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: face_uniform.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(source),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                });
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("cookie"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.draw(0..3, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
    }

    /// Marks the records of lights with a placed cookie (`kept`: the
    /// scene's light index of each record) with their layers.
    pub fn patch(&self, scene: &RenderScene, kept: &[usize], gpu: &mut [LightGpu], base: u32) {
        if self.placed.is_empty() {
            return;
        }
        for c in &scene.light_cookies {
            let Some(k) = kept.iter().position(|&i| i == c.light) else {
                continue;
            };
            let point = matches!(scene.lights[c.light].kind, LightKind::Point);
            let Some(p) = self
                .placed
                .iter()
                .find(|p| p.texture == c.texture && p.point == point)
            else {
                continue;
            };
            let layer = (base + p.offset) as f32;
            let g = &mut gpu[k];
            if point {
                g.cone[1] = -(layer + 1.0);
            } else {
                g.color_kind[3] = 2.0 + layer;
            }
        }
    }
}

/// A rect light's record (`frame::local_lights`).
pub(crate) fn rect_record(
    position: glam::Vec3,
    range: f32,
    radiance: glam::Vec3,
    half_width: glam::Vec3,
    half_height: glam::Vec3,
    two_sided: bool,
) -> LightGpu {
    LightGpu {
        position_range: position.extend(range).to_array(),
        color_kind: radiance
            .extend(if two_sided { -2.0 } else { -1.0 })
            .to_array(),
        direction_slot: half_width.extend(-1.0).to_array(),
        cone: half_height.extend(0.0).to_array(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ltc_table_starts_with_the_identity() {
        // The first texel of ltc_1 is (1, 0, 0, 2e-5): a, b = 1, 0.
        let a = u16::from_le_bytes([LTC[0], LTC[1]]);
        assert_eq!(a, 0x3c00);
    }
}
