//! Scatter culling on the GPU (G4, `shaders/scatter_cull.wgsl`): before the
//! main camera's prepass and lit pass, each chunk of copies the CPU kept
//! (its bounds in the view) is culled copy by copy: past the distance,
//! outside the view, behind the frame's depth pyramid (`cull_gpu.rs`, the
//! Hi-Z its occluders and the meshlets' first phase drew; none without
//! one) or thinned by the scatter's density map. The survivors are
//! compacted in the chunk's order into the frame's copy list, and the
//! chunk's draws become indirect, so the prepass and the lit pass draw the
//! same copies.
//!
//! A scatter whose mesh has levels of detail or an impostor lists its
//! copies by what they draw: a region of the list per level, chosen by the
//! copy's distance against the level's error ([`LOD_PIXELS`]), then one
//! for the copies that draw as the card, so a far copy costs no pass its
//! mesh's vertices. Such a scatter also lists a second set, culled by the
//! distance and the density map alone, which the shadow passes, the probe
//! faces and the reflections draw (coarser, [`LOD_PIXELS_OFF_CAMERA`]);
//! any other keeps its chunks' whole lists there. Nothing is read
//! back: the counts live on the GPU (van Muijden, *GPU-Based Procedural
//! Placement in Horizon Zero Dawn*, GDC 2017: placement on the CPU here,
//! or on the GPU for a scatter that asks (`scatter_place.rs`, whose chunks
//! carry their count after their copies), culling on the GPU with Hi-Z).

use glam::{Mat4, Vec3, Vec4};

/// Copies a chunk holds at most (scatter.rs `CHUNK_CELLS` squared): one
/// workgroup's four per lane.
pub(crate) const MAX_CHUNK_COPIES: u32 = 1024;
/// Words per set in the draws buffer: three indexed draws (a level
/// each), a plain one (the card), a pad.
const SET_WORDS: u64 = 20;
/// Words per chunk in the draws buffer: its two sets.
const DRAW_WORDS: u64 = 2 * SET_WORDS;
/// The levels of detail a scatter draws at most: the mesh and two coarser.
pub const SCATTER_LEVELS: usize = 3;
/// The error on screen, in pixels, a scattered copy's level of detail may
/// have in the main camera's passes. Coarser than a model's one pixel
/// (`lod.rs`): a level of thinned leaves keeps the canopy's cover, and its
/// error is how far a leaf's corner moved, not a hole in the silhouette.
pub const LOD_PIXELS: f32 = 2.0;
/// [`LOD_PIXELS`] in the passes off the main camera (shadow maps, probe
/// faces, reflections), whose texels are coarser than the frame's pixels.
pub const LOD_PIXELS_OFF_CAMERA: f32 = 8.0;
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
    lod: [f32; 4],
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
    bins: [u32; 4],
    bands: [u32; 4],
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
    /// The mesh's coarser levels, in order: each one's index count and
    /// its error in metres at a copy's scale of 1.
    pub levels: [Option<(u32, f32)>; SCATTER_LEVELS - 1],
    /// The distance past which a copy draws as the card (0: never), and
    /// whether this chunk draws that card (a model's first part does).
    pub card_from: f32,
    pub card: bool,
    /// Whether it lists the second set, for the passes off the camera.
    pub off_camera: bool,
    /// The nearest and the farthest one of its copies stands from the
    /// camera along x and z (m), and the least and the greatest scale of
    /// one: the levels it has places for ([`CullItem::band`]).
    pub reach: (f32, f32),
    pub scale: (f32, f32),
}

/// The regions a chunk lists in one set: the mesh's levels from `first`
/// (`count` of them; 0: none of its copies draws its mesh), then the
/// card's when it has one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Band {
    pub first: u32,
    pub count: u32,
    pub card: bool,
}

impl Band {
    fn regions(&self) -> u32 {
        self.count + u32::from(self.card)
    }
}

impl CullItem {
    fn places(&self) -> u32 {
        if self.placed {
            MAX_CHUNK_COPIES
        } else {
            self.copies.min(MAX_CHUNK_COPIES)
        }
    }
    fn levels(&self) -> u32 {
        1 + self.levels.iter().flatten().count() as u32
    }
    fn sets(&self) -> u32 {
        1 + u32::from(self.off_camera)
    }
    /// The regions a set whose levels switch at `lod` metres per metre of
    /// error has places for: the levels a copy between its nearest and
    /// its farthest may draw (the shader holds a copy to them), a card
    /// when one may stand in its crossfade or past it.
    fn band(&self, lod: f32) -> Band {
        let (near, far) = (self.reach.0.max(0.0), self.reach.1.min(self.distance));
        if near > far {
            return Band::default();
        }
        let card_from = self.card_from.max(0.0);
        let card = self.card && card_from > 0.0 && far > 0.9 * card_from;
        if card_from > 0.0 && near >= card_from {
            return Band {
                card,
                ..Band::default()
            };
        }
        let level = |d: f32, scale: f32| {
            let coarser = self.levels.iter().map_while(|l| *l);
            coarser
                .enumerate()
                .filter(|(_, (_, error))| d > error * scale * lod)
                .map(|(l, _)| l as u32 + 1)
                .last()
                .unwrap_or(0)
        };
        let first = level(near, self.scale.1.max(self.scale.0));
        let mesh_far = if card_from > 0.0 {
            far.min(card_from)
        } else {
            far
        };
        let last = level(mesh_far, self.scale.0.min(self.scale.1)).max(first);
        Band {
            first,
            count: last - first + 1,
            card,
        }
    }
    /// Its two sets' regions; the second's are empty when it lists none.
    fn bands(&self, lod: [f32; 2]) -> [Band; 2] {
        match self.off_camera {
            true => [self.band(lod[0]), self.band(lod[1])],
            false => [self.band(lod[0]), Band::default()],
        }
    }
}

/// Which of a culled chunk's two sets a pass draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CullSet {
    /// The main camera's: what its view and the depth pyramid keep.
    Camera = 0,
    /// The passes off the camera: every copy the scatter draws.
    OffCamera = 1,
}

/// Where a culled chunk draws from: its first place in the list, and how
/// its regions are laid out from there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Culled {
    pub item: u32,
    pub first: u32,
    /// The places each region holds.
    places: u32,
    /// Each set's regions (the second's empty when it lists none).
    bands: [Band; 2],
    sets: u32,
}

impl Culled {
    /// Whether it lists `set` (the camera's always).
    pub fn has(&self, set: CullSet) -> bool {
        (set as u32) < self.sets
    }
    /// Whether `set` has a region for the mesh's `level`, and one for the
    /// card: a draw of any other would draw nothing.
    pub fn lists_level(&self, set: CullSet, level: u32) -> bool {
        let b = self.bands[set as usize];
        level >= b.first && level < b.first + b.count
    }
    pub fn lists_card(&self, set: CullSet) -> bool {
        self.bands[set as usize].card
    }
    fn region_offset(&self, set: CullSet, region: u32) -> u64 {
        let before = match set {
            CullSet::Camera => 0,
            CullSet::OffCamera => self.bands[0].regions(),
        };
        u64::from(self.first + (before + region) * self.places) * COPY_BYTES
    }
    /// The byte offset in the list of `set`'s copies that draw the mesh's
    /// `level` ([`Self::lists_level`]), and of those that draw the card.
    pub fn level_offset(&self, set: CullSet, level: u32) -> u64 {
        self.region_offset(set, level - self.bands[set as usize].first)
    }
    pub fn card_offset(&self, set: CullSet) -> u64 {
        self.region_offset(set, self.bands[set as usize].count)
    }
    /// The byte offsets in the draws buffer of `set`'s indexed draw of
    /// `level` and of its plain (card) draw.
    fn level_draw(&self, set: CullSet, level: u32) -> u64 {
        (u64::from(self.item) * DRAW_WORDS + set as u64 * SET_WORDS + u64::from(level) * 5) * 4
    }
    fn card_draw(&self, set: CullSet) -> u64 {
        self.level_draw(set, 0) + 60
    }
    /// Its draw's slot among the per-draw args (`per_draw`): four a set.
    fn slot(&self, set: CullSet, draw: u32) -> usize {
        self.item as usize * 8 + set as usize * 4 + draw as usize
    }
}

/// What the main camera culls with this frame.
pub(crate) struct CullView<'a> {
    /// The jittered view the depth pyramid was drawn with.
    pub view_proj: Mat4,
    pub planes: &'a [Vec4],
    pub camera: Vec3,
    /// The frame's pixels.
    pub size: (u32, u32),
    pub pyramid: Option<(&'a wgpu::TextureView, u32)>,
}

pub(crate) struct ScatterCull {
    /// Compiled on the compile workers (deferred.rs) from its creation.
    pipeline: crate::deferred::Deferred<wgpu::ComputePipeline>,
    shared_layout: wgpu::BindGroupLayout,
    pub chunk_layout: wgpu::BindGroupLayout,
    map_layout: wgpu::BindGroupLayout,
    view: wgpu::Buffer,
    items: wgpu::Buffer,
    item_cap: u64,
    pub list: wgpu::Buffer,
    list_cap: u64,
    /// This frame's level distances ([`lod_distances`]), as planned with.
    lod: [f32; 2],
    pub draws: wgpu::Buffer,
    no_pyramid: wgpu::TextureView,
    white: wgpu::TextureView,
    sampler: wgpu::Sampler,
    /// This frame's chunks: their uniform rows written, their maps, where
    /// they draw from.
    frame: Vec<(Option<crate::TextureHandle>, Culled)>,
    /// Each item's draws' args copied after the cull (`Culled::slot`),
    /// which draws read instead of `draws` on WebKit (`indirect_own.rs`);
    /// `None` elsewhere.
    per_draw: Option<crate::indirect_own::OwnArgs>,
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
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue, per_draw: bool) -> ScatterCull {
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
        let pipeline = crate::deferred::Deferred::anywhere({
            let device = device.clone();
            move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("scatter cull"),
                    layout: Some(&layout),
                    module: &module,
                    entry_point: Some("cull"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        crate::deferred::Start::start(&pipeline);
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
            lod: [0.0; 2],
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
            per_draw: per_draw.then(Default::default),
        }
    }

    fn list_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX
    }

    fn draws_usage() -> wgpu::BufferUsages {
        // COPY_SRC: copied to each draw's own args on WebKit (`per_draw`).
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC
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
        (view_proj, height, bias): (Mat4, u32, f32),
    ) -> Vec<Culled> {
        self.frame.clear();
        let n = items.len() as u64;
        if n == 0 {
            return Vec::new();
        }
        if let Some(own) = &mut self.per_draw {
            own.ensure(device, 8 * n as usize);
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
        self.lod = lod_distances(view_proj, height, bias);
        let bands: Vec<[Band; 2]> = items.iter().map(|i| i.bands(self.lod)).collect();
        let span = |k: usize| items[k].places() * (bands[k][0].regions() + bands[k][1].regions());
        let total: u64 = (0..items.len()).map(|k| u64::from(span(k))).sum();
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
                    i.places(),
                    k as u32,
                    u32::from(i.density_map.is_some()),
                ],
                mesh: [
                    i.index_count,
                    u32::from(i.placed),
                    i.levels[0].map_or(0, |l| l.0),
                    i.levels[1].map_or(0, |l| l.0),
                ],
                shape: [i.radius, i.moves, i.distance, i.card_from.max(0.0)],
                extent: [
                    i.extent.x,
                    i.extent.y,
                    i.levels[0].map_or(0.0, |l| l.1),
                    i.levels[1].map_or(0.0, |l| l.1),
                ],
                bins: [
                    i.places(),
                    i.levels(),
                    u32::from(bands[k][0].card),
                    i.sets(),
                ],
                bands: [
                    bands[k][0].first,
                    bands[k][0].count,
                    bands[k][1].first,
                    bands[k][1].count,
                ],
            };
            let at = k * ITEM_SLOT as usize;
            rows[at..at + std::mem::size_of::<ItemGpu>()].copy_from_slice(bytemuck::bytes_of(&gpu));
            let culled = Culled {
                item: k as u32,
                first,
                places: i.places(),
                bands: bands[k],
                sets: i.sets(),
            };
            out.push(culled);
            self.frame.push((i.density_map, culled));
            first += span(k);
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
            lod: [self.lod[0], self.lod[1], 0.0, 0.0],
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
        for (m, _) in &self.frame {
            if !maps.iter().any(|(h, _)| h == m) {
                maps.push((*m, map_group(*m)));
            }
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("scatter cull"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        for (k, (group, (m, _))) in groups.iter().zip(&self.frame).enumerate() {
            let map = &maps.iter().find(|(h, _)| h == m).expect("made above").1;
            pass.set_bind_group(0, &shared, &[(k as u64 * ITEM_SLOT) as u32]);
            pass.set_bind_group(1, *group, &[]);
            pass.set_bind_group(2, map, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        if let Some(own) = &self.per_draw {
            for (_, at) in &self.frame {
                for set in [CullSet::Camera, CullSet::OffCamera] {
                    if !at.has(set) {
                        continue;
                    }
                    for l in (0..SCATTER_LEVELS as u32).filter(|&l| at.lists_level(set, l)) {
                        let from = at.level_draw(set, l);
                        own.copy(encoder, &self.draws, from, at.slot(set, l), 20);
                    }
                    if at.lists_card(set) {
                        let from = at.card_draw(set);
                        own.copy(encoder, &self.draws, from, at.slot(set, 3), 16);
                    }
                }
            }
        }
    }

    /// The buffer and byte offset the indexed draw of `at`'s `level` in
    /// `set` reads its args from: `draws`, or its own copy on WebKit.
    pub fn level_args(&self, at: &Culled, set: CullSet, level: u32) -> (&wgpu::Buffer, u64) {
        match &self.per_draw {
            Some(own) => (own.get(at.slot(set, level)), 0),
            None => (&self.draws, at.level_draw(set, level)),
        }
    }

    /// [`Self::level_args`] of `at`'s plain (card) draw.
    pub fn card_args(&self, at: &Culled, set: CullSet) -> (&wgpu::Buffer, u64) {
        match &self.per_draw {
            Some(own) => (own.get(at.slot(set, 3)), 0),
            None => (&self.draws, at.card_draw(set)),
        }
    }
}

/// [`lod_distance`] over each set's pixels of error: the metres of
/// distance per metre of a level's error past which the set draws it.
/// `bias` is the scene's ([`crate::RenderScene::lod_bias`]): 2 keeps a
/// level twice as far, 0 draws the mesh alone.
fn lod_distances(view_proj: Mat4, height: u32, bias: f32) -> [f32; 2] {
    if bias <= 0.0 {
        return [1e30; 2];
    }
    let d = lod_distance(view_proj, height) * bias;
    [d / LOD_PIXELS, d / LOD_PIXELS_OFF_CAMERA]
}

/// The distance, in metres per metre of error, at which an error covers
/// one pixel of a frame `height` pixels tall drawn with `view_proj`: its
/// projection's vertical scale times half the height. An orthographic
/// view draws an error the same size at any distance: no level but the
/// mesh.
fn lod_distance(view_proj: Mat4, height: u32) -> f32 {
    let row = |r: usize| {
        Vec3::new(
            view_proj.x_axis[r],
            view_proj.y_axis[r],
            view_proj.z_axis[r],
        )
    };
    if row(3).length_squared() < 1e-12 {
        return 1e30;
    }
    row(1).length() * height.max(1) as f32 * 0.5
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

    /// A chunk's regions follow one another in the list, a set after the
    /// other, and its draws in its row: what the shader writes
    /// (`CULL_SET_WORDS`, a level's five words, the card's at 15).
    #[test]
    fn a_chunks_regions_and_draws_are_where_the_shader_writes_them() {
        // The camera's set lists levels 0 and 1 and a card, the second
        // levels 1 and 2 and a card: three regions each.
        let band = |first| Band {
            first,
            count: 2,
            card: true,
        };
        let at = Culled {
            item: 3,
            first: 100,
            places: 10,
            bands: [band(0), band(1)],
            sets: 2,
        };
        assert!(at.has(CullSet::Camera) && at.has(CullSet::OffCamera));
        assert!(at.lists_level(CullSet::Camera, 1) && !at.lists_level(CullSet::Camera, 2));
        assert!(!at.lists_level(CullSet::OffCamera, 0) && at.lists_card(CullSet::OffCamera));
        assert_eq!(at.level_offset(CullSet::Camera, 1), 110 * COPY_BYTES);
        assert_eq!(at.card_offset(CullSet::Camera), 120 * COPY_BYTES);
        assert_eq!(at.level_offset(CullSet::OffCamera, 1), 130 * COPY_BYTES);
        assert_eq!(at.level_offset(CullSet::OffCamera, 2), 140 * COPY_BYTES);
        assert_eq!(at.card_offset(CullSet::OffCamera), 150 * COPY_BYTES);
        assert_eq!(at.level_draw(CullSet::Camera, 2), (3 * 40 + 10) * 4);
        assert_eq!(at.card_draw(CullSet::OffCamera), (3 * 40 + 20 + 15) * 4);
        assert!(source().contains("const CULL_SET_WORDS: u32 = 20u;"));
        let plain = Culled { sets: 1, ..at };
        assert!(!plain.has(CullSet::OffCamera));
    }

    /// A chunk has places for the levels its copies can draw and no
    /// others: its nearest copy at the greatest scale gives the finest,
    /// its farthest at the least the coarsest, and the card starts in the
    /// crossfade before its distance.
    #[test]
    fn a_chunk_has_places_for_the_levels_its_copies_can_draw() {
        let item = |reach, card_from| CullItem {
            to_terrain: Mat4::IDENTITY,
            extent: glam::Vec2::ONE,
            copies: 8,
            placed: false,
            index_count: 300,
            radius: 1.0,
            moves: 0.0,
            distance: 500.0,
            density_map: None,
            levels: [Some((150, 0.2)), Some((75, 0.5))],
            card_from,
            card: true,
            off_camera: true,
            reach,
            scale: (0.5, 2.0),
        };
        let band = |first, count, card| Band { first, count, card };
        // 100 m per metre of error: level 1 past 20 m and level 2 past
        // 50 m at a scale of 1, half as far at 0.5, twice at 2.
        assert_eq!(item((0.0, 9.0), 0.0).band(100.0), band(0, 1, false));
        assert_eq!(item((0.0, 11.0), 0.0).band(100.0), band(0, 2, false));
        assert_eq!(item((30.0, 60.0), 0.0).band(100.0), band(0, 3, false));
        assert_eq!(item((41.0, 60.0), 0.0).band(100.0), band(1, 2, false));
        assert_eq!(item((101.0, 600.0), 0.0).band(100.0), band(2, 1, false));
        // A card from 80 m: in places from 72 m, alone past 80 m.
        assert_eq!(item((41.0, 71.0), 80.0).band(100.0), band(1, 2, false));
        assert_eq!(item((41.0, 73.0), 80.0).band(100.0), band(1, 2, true));
        assert_eq!(item((80.0, 90.0), 80.0).band(100.0), band(0, 0, true));
        // Past the scatter's distance: nothing.
        assert_eq!(item((501.0, 520.0), 80.0).band(100.0), band(0, 0, false));
        // The second set's levels switch four times nearer.
        let both = item((20.0, 60.0), 0.0).bands([100.0, 25.0]);
        assert_eq!(both, [band(0, 3, false), band(1, 2, false)]);
        let one = CullItem {
            off_camera: false,
            ..item((30.0, 60.0), 0.0)
        };
        assert_eq!(one.bands([100.0, 25.0])[1], Band::default());
    }

    /// A perspective view's level distance is its pixels per metre at a
    /// metre; an orthographic one never leaves the mesh.
    #[test]
    fn the_level_distance_is_the_views_pixels_per_metre() {
        let fov = 50f32.to_radians();
        let f = 1.0 / (fov * 0.5).tan();
        // A camera turned about y and moved: the scale is the lens's.
        let view = Mat4::from_rotation_y(0.7) * Mat4::from_translation(Vec3::new(3.0, 2.0, 5.0));
        let lens = |w: f32, rows: [f32; 2]| {
            Mat4::from_cols(
                Vec4::new(rows[0], 0.0, 0.0, 0.0),
                Vec4::new(0.0, rows[1], 0.0, 0.0),
                Vec4::new(0.0, 0.0, -1.0, -w),
                Vec4::new(0.0, 0.0, -0.1, 1.0 - w),
            )
        };
        let seen = lens(1.0, [f * 9.0 / 16.0, f]) * view;
        let d = lod_distance(seen, 720);
        assert!((d - 360.0 * f).abs() < 0.05, "{d}");
        // The scene's bias keeps a level as much farther; 0, the mesh alone.
        let [camera, off] = lod_distances(seen, 720, 2.0);
        assert!((camera - 2.0 * d / LOD_PIXELS).abs() < 0.05, "{camera}");
        assert!(
            (off - 2.0 * d / LOD_PIXELS_OFF_CAMERA).abs() < 0.05,
            "{off}"
        );
        assert!(lod_distances(seen, 720, 0.0)[0] > 1e20);
        assert!(lod_distance(lens(0.0, [1.0 / 8.0, 1.0 / 4.5]) * view, 720) > 1e20);
    }
}
