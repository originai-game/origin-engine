//! Scatter culling on the GPU (G4, `shaders/scatter_cull.wgsl`): before the
//! main camera's prepass and lit pass, each chunk of copies the CPU kept
//! (its bounds in the view) is culled copy by copy: past the distance,
//! outside the view, behind the frame's depth pyramid (`cull_gpu.rs`, the
//! Hi-Z its occluders and the meshlets' first phase drew; none without
//! one) or thinned by the scatter's density map. The survivors are
//! compacted in the chunk's order into the frame's copy list, and the
//! chunk's two draws (its mesh, its impostor) become indirect, so the
//! prepass and the lit pass draw the same copies. Shadow passes, probe
//! faces and reflections keep the chunks' whole lists. Nothing is read
//! back: the counts live on the GPU (van Muijden, *GPU-Based Procedural
//! Placement in Horizon Zero Dawn*, GDC 2017: placement on the CPU here,
//! or on the GPU for a scatter that asks (`scatter_place.rs`, whose chunks
//! carry their count after their copies), culling on the GPU with Hi-Z).

use glam::{Mat4, Vec3, Vec4};

/// Copies a chunk holds at most (scatter.rs `CHUNK_CELLS` squared): one
/// workgroup's four per lane.
pub(crate) const MAX_CHUNK_COPIES: u32 = 1024;
/// Words per chunk in the draws buffer: an indexed draw, a plain one, a pad.
const DRAW_WORDS: u64 = 10;
/// Bytes a copy takes (instances.rs `GpuTransform`).
pub(crate) const COPY_BYTES: u64 = std::mem::size_of::<crate::instances::GpuTransform>() as u64;
/// A chunk's uniform slot (the dynamic offset's alignment).
const ITEM_SLOT: u64 = 256;

pub(crate) fn source() -> String {
    wgsl3d!("scatter_cull.wgsl").to_string()
}

/// `CullView` in scatter_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewGpu {
    view_proj: [[f32; 4]; 4],
    planes: [[f32; 4]; 6],
    camera: [f32; 4],
    sizes: [u32; 4],
}

/// `CullItem` in scatter_cull.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ItemGpu {
    to_terrain: [[f32; 4]; 4],
    item: [u32; 4],
    mesh: [u32; 4],
    shape: [f32; 4],
    extent: [f32; 4],
}

/// One chunk to cull this frame, as the scatter renderer plans it.
pub(crate) struct CullItem {
    /// The terrain's space from the frame's, and its size there.
    pub to_terrain: Mat4,
    pub extent: glam::Vec2,
    /// Its copies, the mesh's index count and radius about its origin,
    /// how far a copy moves at most and the scatter's distance (m).
    pub copies: u32,
    /// Placed on the GPU (`scatter_place.rs`): its count follows its
    /// copies in its buffer, and `copies` is ignored.
    pub placed: bool,
    pub index_count: u32,
    pub radius: f32,
    pub moves: f32,
    pub distance: f32,
    pub density_map: Option<crate::TextureHandle>,
}

/// Where a culled chunk draws from: its first place in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Culled {
    pub item: u32,
    pub first: u32,
}

impl Culled {
    /// The byte offsets of its copies in the list and of its indexed and
    /// plain draws in the draws buffer.
    pub fn list_offset(&self) -> u64 {
        u64::from(self.first) * COPY_BYTES
    }
    pub fn mesh_draw(&self) -> u64 {
        u64::from(self.item) * DRAW_WORDS * 4
    }
    pub fn card_draw(&self) -> u64 {
        self.mesh_draw() + 20
    }
}

/// What the main camera culls with this frame.
pub(crate) struct CullView<'a> {
    /// The jittered view the depth pyramid was drawn with.
    pub view_proj: Mat4,
    pub planes: &'a [Vec4],
    pub camera: Vec3,
    pub size: (u32, u32),
    pub pyramid: Option<(&'a wgpu::TextureView, u32)>,
}

pub(crate) struct ScatterCull {
    pipeline: wgpu::ComputePipeline,
    shared_layout: wgpu::BindGroupLayout,
    pub chunk_layout: wgpu::BindGroupLayout,
    map_layout: wgpu::BindGroupLayout,
    view: wgpu::Buffer,
    items: wgpu::Buffer,
    item_cap: u64,
    pub list: wgpu::Buffer,
    list_cap: u64,
    pub draws: wgpu::Buffer,
    no_pyramid: wgpu::TextureView,
    white: wgpu::TextureView,
    sampler: wgpu::Sampler,
    /// This frame's chunks: their uniform rows written, their maps.
    frame: Vec<Option<crate::TextureHandle>>,
}

fn texture_1x1(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    data: &[u8],
) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("scatter cull"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        )
        .create_view(&Default::default())
}

impl ScatterCull {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> ScatterCull {
        let cs = wgpu::ShaderStages::COMPUTE;
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: cs,
            ty,
            count: None,
        };
        let buffer = |ty, dynamic| wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: dynamic,
            min_binding_size: None,
        };
        let rw = wgpu::BufferBindingType::Storage { read_only: false };
        let shared_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter cull"),
            entries: &[
                entry(0, buffer(wgpu::BufferBindingType::Uniform, false)),
                entry(
                    1,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ItemGpu>() as u64
                        ),
                    },
                ),
                entry(2, buffer(rw, false)),
                entry(3, buffer(rw, false)),
                entry(
                    4,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let chunk_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter cull chunk"),
            entries: &[entry(
                0,
                buffer(wgpu::BufferBindingType::Storage { read_only: true }, false),
            )],
        });
        let map_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter cull map"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter cull"),
            source: wgpu::ShaderSource::Wgsl(source().into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scatter cull"),
            bind_group_layouts: &[Some(&shared_layout), Some(&chunk_layout), Some(&map_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("scatter cull"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cull"),
            compilation_options: Default::default(),
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let make = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let uniform = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        ScatterCull {
            pipeline,
            view: make(
                "scatter cull view",
                std::mem::size_of::<ViewGpu>() as u64,
                uniform,
            ),
            items: make("scatter cull items", ITEM_SLOT * 64, uniform),
            item_cap: 64,
            list: make("scatter cull list", 0, Self::list_usage()),
            list_cap: 0,
            draws: make(
                "scatter cull draws",
                DRAW_WORDS * 4 * 64,
                Self::draws_usage(),
            ),
            shared_layout,
            chunk_layout,
            map_layout,
            no_pyramid: texture_1x1(device, queue, wgpu::TextureFormat::R32Float, &[0; 4]),
            white: texture_1x1(device, queue, wgpu::TextureFormat::Rgba8Unorm, &[255; 4]),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("scatter cull map"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }),
            frame: Vec::new(),
        }
    }

    fn list_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX
    }

    fn draws_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT
    }

    /// A chunk's copies as the cull reads them (its buffer has STORAGE).
    pub(crate) fn chunk_group(
        &self,
        device: &wgpu::Device,
        copies: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter cull chunk"),
            layout: &self.chunk_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: copies.as_entire_binding(),
            }],
        })
    }

    /// Plans this frame's chunks, in order: their uniform rows, the list
    /// and draws sized for them. Answers where each draws from.
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        items: &[CullItem],
    ) -> Vec<Culled> {
        self.frame.clear();
        let n = items.len() as u64;
        if n == 0 {
            return Vec::new();
        }
        let make = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        if self.item_cap < n {
            self.item_cap = n.next_power_of_two();
            self.items = make(
                "scatter cull items",
                ITEM_SLOT * self.item_cap,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            );
            self.draws = make(
                "scatter cull draws",
                DRAW_WORDS * 4 * self.item_cap,
                Self::draws_usage(),
            );
        }
        let copies = |i: &CullItem| {
            if i.placed { MAX_CHUNK_COPIES } else { i.copies }
        };
        let total: u64 = items.iter().map(|i| u64::from(copies(i))).sum();
        if self.list_cap < total.max(1) {
            self.list_cap = total.max(1).next_power_of_two();
            self.list = make(
                "scatter cull list",
                self.list_cap * COPY_BYTES,
                Self::list_usage(),
            );
        }
        let mut rows = vec![0u8; (ITEM_SLOT * n) as usize];
        let mut out = Vec::with_capacity(items.len());
        let mut first = 0u32;
        for (k, i) in items.iter().enumerate() {
            let gpu = ItemGpu {
                to_terrain: i.to_terrain.to_cols_array_2d(),
                item: [
                    first,
                    copies(i).min(MAX_CHUNK_COPIES),
                    k as u32,
                    u32::from(i.density_map.is_some()),
                ],
                mesh: [i.index_count, u32::from(i.placed), 0, 0],
                shape: [i.radius, i.moves, i.distance, 0.0],
                extent: [i.extent.x, i.extent.y, 0.0, 0.0],
            };
            let at = k * ITEM_SLOT as usize;
            rows[at..at + std::mem::size_of::<ItemGpu>()].copy_from_slice(bytemuck::bytes_of(&gpu));
            out.push(Culled {
                item: k as u32,
                first,
            });
            self.frame.push(i.density_map);
            first += copies(i);
        }
        queue.write_buffer(&self.items, 0, &rows);
        out
    }

    /// Culls this frame's chunks (`groups`: each one's copies, in the order
    /// [`Self::prepare`] was given them) into the list and the draws.
    pub(crate) fn encode<'t>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        view: &CullView,
        groups: &[&wgpu::BindGroup],
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) {
        if self.frame.is_empty() {
            return;
        }
        let gpu = ViewGpu {
            view_proj: view.view_proj.to_cols_array_2d(),
            planes: std::array::from_fn(|i| {
                view.planes.get(i).copied().unwrap_or(Vec4::ZERO).to_array()
            }),
            camera: view.camera.extend(0.0).to_array(),
            sizes: [
                view.size.0.max(1),
                view.size.1.max(1),
                view.pyramid.map_or(0, |p| p.1),
                0,
            ],
        };
        queue.write_buffer(&self.view, 0, bytemuck::bytes_of(&gpu));
        let shared = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter cull"),
            layout: &self.shared_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.view.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.items,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<ItemGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.list.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.draws.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        view.pyramid.map_or(&self.no_pyramid, |p| p.0),
                    ),
                },
            ],
        });
        let map_group = |m: Option<crate::TextureHandle>| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scatter cull map"),
                layout: &self.map_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            m.and_then(textures).unwrap_or(&self.white),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        };
        // One group per distinct map, in the order first met.
        let mut maps: Vec<(Option<crate::TextureHandle>, wgpu::BindGroup)> = Vec::new();
        for m in &self.frame {
            if !maps.iter().any(|(h, _)| h == m) {
                maps.push((*m, map_group(*m)));
            }
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scatter cull"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        for (k, (group, m)) in groups.iter().zip(&self.frame).enumerate() {
            let map = &maps.iter().find(|(h, _)| h == m).expect("made above").1;
            pass.set_bind_group(0, &shared, &[(k as u64 * ITEM_SLOT) as u32]);
            pass.set_bind_group(1, *group, &[]);
            pass.set_bind_group(2, map, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cull's uniforms have the layout its shader reads, and an item
    /// fits its dynamic slot.
    #[test]
    fn the_uniforms_reach_the_shader() {
        let module = naga::front::wgsl::parse_str(&source()).expect("parses");
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("lays out");
        let size = |name: &str| {
            let (h, _) = module
                .types
                .iter()
                .find(|(_, t)| t.name.as_deref() == Some(name))
                .expect("in the shader");
            layouter[h].size as usize
        };
        assert_eq!(size("CullView"), std::mem::size_of::<ViewGpu>());
        assert_eq!(size("CullItem"), std::mem::size_of::<ItemGpu>());
        assert!(std::mem::size_of::<ItemGpu>() as u64 <= ITEM_SLOT);
        assert_eq!(
            COPY_BYTES, 64,
            "four vectors per copy, as the shader reads them"
        );
        assert_eq!(MAX_CHUNK_COPIES, 256 * 4, "a lane's four copies");
    }
}
