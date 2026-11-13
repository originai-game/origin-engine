//! Crowd impostors' atlases (crowd.rs, **Impostors**): baked on the GPU
//! from a crowd's full mesh (`shaders/crowd_bake.wgsl`) the first frame a
//! crowd with an impostor is drawn, and again when what it shows changes:
//! its parts' meshes or colour maps, its clip's frames, its sphere or how
//! it is baked. Crowds whose parts share meshes and maps share an atlas
//! (their materials' colours are the cards', not the atlas's). An atlas
//! not drawn for [`KEEP_FRAMES`] frames is dropped.
//!
//! Each atlas is two textures of `views × size` by `frames × size` texels
//! with a mip chain down to 4 texels a tile: the colour detail with its
//! coverage (sRGB) and the normal, depth and part (half floats), both
//! premultiplied by coverage, so the 4× multisample resolve and the mips
//! (`mips.rs`, a box filter) average them as coverage does. The bake is
//! one instanced draw per part, an instance per tile.

use std::collections::HashMap;

/// Samples a texel of the bake.
const SAMPLES: u32 = 4;
/// A part's uniform slot (the dynamic offset's alignment).
const SLOT: u64 = 256;
/// Frames an atlas no crowd drew is kept.
const KEEP_FRAMES: u64 = 120;

pub(crate) const ALBEDO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub(crate) const DATA_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// `Bake` in crowd_bake.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct BakeGpu {
    pub clip: [u32; 4],
    pub tiles: [u32; 4],
    pub sphere: [f32; 4],
    pub view: [f32; 4],
}

/// What an atlas shows: its parts' meshes (with their revisions) and
/// colour maps, its clip and the clips' generation (skin.rs `BakeCache`),
/// its views, frames, size, elevation (bits) and sphere (bits).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AtlasKey {
    pub parts: Vec<(usize, u64, Option<usize>)>,
    pub clip: u32,
    pub generation: u64,
    pub tiles: [u32; 4],
    pub sphere: [u32; 4],
}

/// One part of a bake: its mesh's buffers and its material's group.
pub(crate) struct BakePart<'a> {
    pub vertices: &'a wgpu::Buffer,
    pub indices: &'a wgpu::Buffer,
    pub skin: &'a wgpu::Buffer,
    pub index_count: u32,
    pub material: &'a wgpu::BindGroup,
}

pub(crate) struct Atlas {
    pub albedo: wgpu::TextureView,
    pub data: wgpu::TextureView,
    used: u64,
    /// The cards' material groups over it, by their parts' colours (bits).
    pub groups: HashMap<Vec<u32>, wgpu::BindGroup>,
}

pub(crate) struct CrowdImpostors {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    slots: u64,
    group: wgpu::BindGroup,
    pub atlases: HashMap<AtlasKey, Atlas>,
    frame: u64,
    /// The multisampled targets of the last bake's size.
    msaa: Option<((u32, u32), [wgpu::TextureView; 3])>,
}

impl CrowdImpostors {
    pub fn new(
        device: &wgpu::Device,
        material_layout: &wgpu::BindGroupLayout,
        palette_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("crowd bake"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<BakeGpu>() as u64),
                },
                count: None,
            }],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crowd bake"),
            source: wgpu::ShaderSource::Wgsl(crate::crowd_bake_source().into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("crowd bake"),
            bind_group_layouts: &[Some(&layout), Some(material_layout), Some(palette_layout)],
            immediate_size: 0,
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("crowd bake"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_bake"),
                compilation_options: Default::default(),
                buffers: &[
                    Some(crate::VERTEX_LAYOUT),
                    Some(crate::skin::SKIN_VERTEX_LAYOUT),
                ],
            },
            // Both faces: a part's caps and thin pieces seen from any side.
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: SAMPLES,
                ..Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_bake"),
                compilation_options: Default::default(),
                targets: &[target(ALBEDO_FORMAT), target(DATA_FORMAT)],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let (uniform, group) = Self::slots(device, &layout, 4);
        CrowdImpostors {
            layout,
            pipeline,
            uniform,
            slots: 4,
            group,
            atlases: HashMap::new(),
            frame: 0,
            msaa: None,
        }
    }

    fn slots(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        slots: u64,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crowd bake"),
            size: slots * SLOT,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("crowd bake"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<BakeGpu>() as u64),
                }),
            }],
        });
        (uniform, group)
    }

    /// Starts a frame: atlases not drawn for [`KEEP_FRAMES`] are dropped.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
        let frame = self.frame;
        self.atlases.retain(|_, a| frame - a.used < KEEP_FRAMES);
    }

    /// The atlas `key` shows, baked into `encoder` now when it has none
    /// (`parts`: the full mesh of each part, in the crowd's order; `bake`:
    /// the uniform without its part, which each part's slot fills in).
    #[allow(clippy::too_many_arguments)]
    pub fn atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        mips: &mut crate::mips::MipGenerator,
        key: &AtlasKey,
        bake: BakeGpu,
        parts: &[BakePart<'_>],
        palette: &wgpu::BindGroup,
    ) -> &mut Atlas {
        let frame = self.frame;
        if !self.atlases.contains_key(key) {
            let atlas = self.bake(device, queue, encoder, mips, bake, parts, palette);
            self.atlases.insert(key.clone(), atlas);
        }
        let atlas = self.atlases.get_mut(key).expect("baked above");
        atlas.used = frame;
        atlas
    }

    #[allow(clippy::too_many_arguments)]
    fn bake(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        mips: &mut crate::mips::MipGenerator,
        bake: BakeGpu,
        parts: &[BakePart<'_>],
        palette: &wgpu::BindGroup,
    ) -> Atlas {
        let [views, frames, size, _] = bake.tiles;
        let extent = (views * size, frames * size);
        // Down to 4 texels a tile: each level's tiles stay whole.
        let levels = (size.max(4).ilog2() - 1).max(1);
        let texture = |label, format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: extent.0,
                    height: extent.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let albedo = texture("crowd impostor albedo", ALBEDO_FORMAT);
        let data = texture("crowd impostor data", DATA_FORMAT);
        if self.msaa.as_ref().is_none_or(|m| m.0 != extent) {
            let target = |label, format| {
                device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d {
                            width: extent.0,
                            height: extent.1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: SAMPLES,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            self.msaa = Some((
                extent,
                [
                    target("crowd bake albedo", ALBEDO_FORMAT),
                    target("crowd bake data", DATA_FORMAT),
                    target("crowd bake depth", DEPTH_FORMAT),
                ],
            ));
        }
        if self.slots < parts.len() as u64 {
            self.slots = (parts.len() as u64).next_power_of_two();
            (self.uniform, self.group) = Self::slots(device, &self.layout, self.slots);
        }
        for k in 0..parts.len() {
            let mut b = bake;
            b.clip[3] = k as u32;
            queue.write_buffer(&self.uniform, k as u64 * SLOT, bytemuck::bytes_of(&b));
        }
        let level0 = |t: &wgpu::Texture| crate::mips::level_view(t, "crowd impostor level", 0);
        let (albedo0, data0) = (level0(&albedo), level0(&data));
        let [msaa_albedo, msaa_data, msaa_depth] = &self.msaa.as_ref().expect("made above").1;
        {
            let clear = |view, resolve| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: Some(resolve),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Discard,
                    },
                })
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("crowd bake"),
                color_attachments: &[clear(msaa_albedo, &albedo0), clear(msaa_data, &data0)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: msaa_depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(2, palette, &[]);
            for (k, p) in parts.iter().enumerate() {
                pass.set_bind_group(0, &self.group, &[(k as u64 * SLOT) as u32]);
                pass.set_bind_group(1, p.material, &[]);
                pass.set_vertex_buffer(0, p.vertices.slice(..));
                pass.set_vertex_buffer(1, p.skin.slice(..));
                pass.set_index_buffer(p.indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..p.index_count, 0, 0..views * frames);
            }
        }
        for t in [&albedo, &data] {
            mips.generate(device, encoder, t);
        }
        Atlas {
            albedo: albedo.create_view(&Default::default()),
            data: data.create_view(&Default::default()),
            used: self.frame,
            groups: HashMap::new(),
        }
    }
}
