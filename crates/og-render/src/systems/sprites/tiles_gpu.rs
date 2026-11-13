//! Tile chunks stay on the device for the lifetime of their shared geometry.
//! Animated UVs are selected in the vertex shader from tick time. Views and
//! repeats share geometry, with one small dynamic uniform per placement.
use crate::{RenderScene, TileChunk};
use glam::Vec2;
use std::sync::Arc;

pub(crate) fn source() -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::FRAME_WGSL,
        crate::oit::OIT_WGSL,
        crate::oit::ARENA_WGSL,
        crate::LIGHTING_WGSL,
        crate::SPRITE_WGSL,
        wgsl!("tiles.wgsl")
    )
}

pub(crate) struct TileDraw {
    pub layer: usize,
    pub chunk: usize,
    pub offset: Vec2,
    pub lit: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Quad {
    min_size: [f32; 4],
    uv: [f32; 4],
    frames: [u32; 2],
    total: f32,
    padding: f32,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Frame {
    uv: [f32; 4],
    duration: [f32; 4],
}

fn pack(chunk: &TileChunk) -> (Vec<Quad>, Vec<Frame>) {
    let mut quads: Vec<_> = chunk
        .quads
        .iter()
        .map(|q| Quad {
            min_size: [q.min.x, q.min.y, q.size.x, q.size.y],
            uv: q.uv,
            frames: [0, 0],
            total: 0.0,
            padding: 0.0,
        })
        .collect();
    let mut frames = Vec::new();
    for a in &chunk.animated {
        let total: f32 = a.durations.iter().sum();
        if total <= 0.0 || a.uvs.is_empty() {
            continue;
        }
        if let Some(q) = quads.get_mut(a.quad as usize) {
            q.frames = [frames.len() as u32, a.durations.len() as u32];
            q.total = total;
            frames.extend(a.durations.iter().enumerate().map(|(k, &d)| Frame {
                uv: a.uvs[k.min(a.uvs.len() - 1)],
                duration: [d, 0.0, 0.0, 0.0],
            }));
        }
    }
    (quads, frames)
}

struct Chunk {
    quads: wgpu::Buffer,
    frames: wgpu::Buffer,
    group: Option<wgpu::BindGroup>,
}
struct Layer {
    source: Arc<Vec<TileChunk>>,
    chunks: Vec<Chunk>,
}

pub(crate) struct Tiles {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    normal: Option<wgpu::RenderPipeline>,
    params: wgpu::Buffer,
    layers: Vec<Layer>,
    /// The cached layer/chunk of each view's draw.
    draws: Vec<(usize, usize)>,
    pub textures: Vec<crate::TextureHandle>,
    /// The lit features its module is made without (`RenderDesc::without`).
    off: crate::pipeline_features::Features,
}

impl Tiles {
    pub fn supported(flags: wgpu::DownlevelFlags) -> bool {
        flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS | wgpu::DownlevelFlags::VERTEX_STORAGE)
    }

    pub fn new(
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        samples: u32,
        off: crate::pipeline_features::Features,
    ) -> Self {
        let storage = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tile chunk"),
            entries: &[
                storage(0),
                storage(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: std::num::NonZeroU64::new(48),
                    },
                    count: None,
                },
            ],
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tile placements"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pipeline = pipeline(device, layouts, &layout, samples, false, off);
        Self {
            layout,
            pipeline,
            normal: None,
            params,
            layers: Vec::new(),
            draws: Vec::new(),
            textures: Vec::new(),
            off,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        draws: &[TileDraw],
        normal: Option<[&wgpu::BindGroupLayout; 3]>,
        samples: u32,
    ) {
        if self.normal.is_none()
            && let Some(layouts) = normal
        {
            self.normal = Some(pipeline(
                device,
                layouts,
                &self.layout,
                samples,
                true,
                self.off,
            ));
        }
        // Retain the Arc as the geometry identity: copy-on-write edits and
        // reloads cannot alias a cached allocation, even across scenes.
        self.layers.retain(|l| {
            scene
                .sprite
                .tile_layers
                .iter()
                .any(|s| Arc::ptr_eq(&l.source, &s.chunks))
        });
        self.draws.clear();
        self.textures.clear();
        let mut params = Vec::with_capacity(draws.len() * 256);
        for draw in draws {
            let layer = &scene.sprite.tile_layers[draw.layer];
            let index = match self
                .layers
                .iter()
                .position(|l| Arc::ptr_eq(&l.source, &layer.chunks))
            {
                Some(i) => i,
                None => {
                    let chunks = layer
                        .chunks
                        .iter()
                        .map(|chunk| {
                            let (q, f) = pack(chunk);
                            let buffer = |label, bytes: &[u8]| {
                                crate::buffer_init(
                                    device,
                                    queue,
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some(label),
                                        contents: if bytes.is_empty() { &[0; 48] } else { bytes },
                                        usage: wgpu::BufferUsages::STORAGE,
                                    },
                                )
                            };
                            Chunk {
                                quads: buffer("tile quads", bytemuck::cast_slice(&q)),
                                frames: buffer("tile frames", bytemuck::cast_slice(&f)),
                                group: None,
                            }
                        })
                        .collect();
                    self.layers.push(Layer {
                        source: layer.chunks.clone(),
                        chunks,
                    });
                    self.layers.len() - 1
                }
            };
            self.draws.push((index, draw.chunk));
            self.textures.push(layer.texture);
            let at = layer.origin.truncate() + draw.offset;
            let data = [
                [at.x, at.y, layer.origin.z, layer.time],
                layer.color,
                [draw.lit, 0.0, 0.0, 0.0],
            ];
            params.extend_from_slice(bytemuck::cast_slice(&data));
            params.resize(self.draws.len() * 256, 0);
        }
        let size = self.params.size();
        crate::renderer::grow_and_write(
            device,
            queue,
            &mut self.params,
            "tile placements",
            &params,
        );
        let grew = size != self.params.size();
        for layer in &mut self.layers {
            for chunk in &mut layer.chunks {
                if grew || chunk.group.is_none() {
                    chunk.group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("tile chunk"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: chunk.quads.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: chunk.frames.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.params,
                                    offset: 0,
                                    size: std::num::NonZeroU64::new(48),
                                }),
                            },
                        ],
                    }));
                }
            }
        }
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, index: u32, count: u32, normal: bool) {
        let (layer, chunk) = self.draws[index as usize];
        pass.set_pipeline(if normal {
            self.normal.as_ref().expect("prepared")
        } else {
            &self.pipeline
        });
        pass.set_bind_group(
            3,
            self.layers[layer].chunks[chunk]
                .group
                .as_ref()
                .expect("prepared"),
            &[index * 256],
        );
        pass.draw(0..6, 0..count);
    }
}

fn pipeline(
    device: &wgpu::Device,
    layouts: [&wgpu::BindGroupLayout; 3],
    tiles: &wgpu::BindGroupLayout,
    samples: u32,
    normal: bool,
    off: crate::pipeline_features::Features,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("tiles"),
        bind_group_layouts: &[
            Some(layouts[0]),
            Some(layouts[1]),
            Some(layouts[2]),
            Some(tiles),
        ],
        immediate_size: 0,
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("tiles"),
        source: wgpu::ShaderSource::Wgsl(off.leave_out(source()).into()),
    });
    crate::hdr_pipeline(
        device,
        crate::HdrPipeline {
            alpha_to_coverage: false,
            label: "tiles",
            layout: &layout,
            module: &module,
            vs: "vs_tile",
            fs: if normal { "fs_normal" } else { "fs" },
            buffers: &[],
            cull: None,
            depth_write: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            samples,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TileAnimation, TileQuad};

    #[test]
    fn packing_keeps_static_uvs_and_animation_ranges_separate() {
        let chunk = TileChunk {
            quads: vec![
                TileQuad {
                    min: Vec2::new(-3.0, 5.0),
                    size: Vec2::new(2.0, 1.0),
                    uv: [0.0, 0.0, 0.5, 1.0]
                };
                3
            ],
            animated: vec![TileAnimation {
                quad: 1,
                uvs: Arc::from([[0.0, 0.0, 0.25, 1.0], [0.25, 0.0, 1.0, 1.0]]),
                durations: Arc::from([0.125, 0.375]),
            }],
            ..Default::default()
        };
        let (quads, frames) = pack(&chunk);
        assert_eq!(size_of::<Quad>(), 48);
        assert_eq!(size_of::<Frame>(), 32);
        assert_eq!(quads[0].min_size, [-3.0, 5.0, 2.0, 1.0]);
        assert_eq!(quads[0].uv, [0.0, 0.0, 0.5, 1.0]);
        assert_eq!(quads[0].frames, [0, 0]);
        assert_eq!(quads[1].frames, [0, 2]);
        assert_eq!(quads[1].total, 0.5);
        assert_eq!(quads[2].frames, [0, 0]);
        assert_eq!(frames[1].uv, [0.25, 0.0, 1.0, 1.0]);
        assert_eq!(frames[0].duration[0], 0.125);
        assert_eq!(frames[1].duration[0], 0.375);
    }
}
