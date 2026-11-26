//! The decals' atlas: every texture the frame's decals name, copied into
//! one texture the lit passes sample (decals.wgsl `decal_atlas`), so each
//! decal has its own image and normal map while a fragment still walks its
//! cluster's decals with one binding. A scene without a textured decal has
//! no atlas, and a decal without a texture samples none (decals.wgsl skips
//! a record whose region starts below 0).
//!
//! A texture lands on whole cells ([`CELL`] texels a side, stretched to
//! fill them) and the atlas has [`LEVELS`] levels, so every level holds it
//! on whole texels and the inset decals.wgsl `decal_sample` keeps from a
//! region's edge is all that separates it from its neighbours. The atlas is
//! sRGB as an artist's decal sheet is: colours are filtered in linear light,
//! and a normal map's values are written as if they were linear light, so
//! the filter and the levels average the normals themselves.
//!
//! The atlas starts at the smallest of [`SIDES`] and is packed again, one
//! size up, when a texture finds no room; at the largest, every texture is
//! halved until they fit. A repack keeps only the textures the frame's
//! decals name.

use std::collections::HashMap;

/// A cell's side in texels.
const CELL: u32 = 64;
/// The atlas's levels: down to one texel a cell.
const LEVELS: u32 = CELL.trailing_zeros() + 1;
/// The atlas's sides, smallest first (the largest is 89 MB with its levels).
const SIDES: [u32; 3] = [1024, 2048, 4096];
/// The longest side a texture keeps in the atlas (a larger one is scaled
/// down to it).
const MAX_IMAGE: u32 = 2048;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// A texture in the atlas: the scene's texture slot, as a colour or as a
/// normal map (a texture used as both is held twice).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    pub index: usize,
    pub normal: bool,
}

/// How a texture is copied in (decal_atlas.wgsl's entry points).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Copy {
    Color,
    Normal,
    NormalSrgb,
    NormalRg,
}

/// What a frame's decal names: its key, its texture's size on the GPU, the
/// renderer's revision of it, and whether its texels change without one (a
/// render target, a video).
pub(crate) struct Wanted {
    pub key: Key,
    pub size: (u32, u32),
    pub revision: u64,
    pub live: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    /// x, y, width and height in cells.
    cells: [u32; 4],
    revision: u64,
}

/// Rows of cells filled left to right, the atlas top to bottom.
#[derive(Clone, Debug, Default, PartialEq)]
struct Shelves {
    side: u32,
    /// Each shelf's top, height and filled width, in cells.
    shelves: Vec<[u32; 3]>,
    next: u32,
}

impl Shelves {
    fn new(side: u32) -> Shelves {
        Shelves {
            side,
            shelves: Vec::new(),
            next: 0,
        }
    }

    /// Room for `w` by `h` cells: the lowest shelf that holds it (so a
    /// small image does not open a tall shelf), or a new one.
    fn place(&mut self, w: u32, h: u32) -> Option<[u32; 2]> {
        let fit = self
            .shelves
            .iter_mut()
            .filter(|s| s[1] >= h && s[2] + w <= self.side)
            .min_by_key(|s| s[1]);
        if let Some(s) = fit {
            let at = [s[2], s[0]];
            s[2] += w;
            return Some(at);
        }
        if w > self.side || self.next + h > self.side {
            return None;
        }
        self.shelves.push([self.next, h, w]);
        self.next += h;
        Some([0, self.next - h])
    }
}

/// The cells a `width` by `height` texture takes with every side halved
/// `shift` times.
fn cells_of((width, height): (u32, u32), shift: u32) -> [u32; 2] {
    let long = width.max(height).max(1);
    let scale = (MAX_IMAGE as f32 / long as f32).min(1.0) / (1u32 << shift) as f32;
    let cells = |texels: u32| ((texels as f32 * scale / CELL as f32).ceil() as u32).max(1);
    [cells(width), cells(height)]
}

/// Every one of `sizes` (in cells) placed in an atlas `side` cells a side,
/// the tallest first, or `None` when one finds no room.
fn pack(side: u32, sizes: &[[u32; 2]]) -> Option<(Shelves, Vec<[u32; 2]>)> {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| {
        (
            std::cmp::Reverse(sizes[i][1]),
            std::cmp::Reverse(sizes[i][0]),
        )
    });
    let mut shelves = Shelves::new(side);
    let mut at = vec![[0; 2]; sizes.len()];
    for i in order {
        at[i] = shelves.place(sizes[i][0], sizes[i][1])?;
    }
    Some((shelves, at))
}

/// The copies' pipelines, layout and sampler, made with the first atlas.
pub(crate) struct DecalCopies {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    color: wgpu::RenderPipeline,
    normal: wgpu::RenderPipeline,
    normal_srgb: wgpu::RenderPipeline,
    normal_rg: wgpu::RenderPipeline,
}

/// decal_atlas.wgsl (`crate::shaders` validates it).
pub(crate) const DECAL_ATLAS_WGSL: &str = wgsl_library!("fullscreen.wgsl", "decal_atlas.wgsl");

impl DecalCopies {
    pub fn new(device: &wgpu::Device) -> DecalCopies {
        let shader = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("decal atlas"),
                source: wgpu::ShaderSource::Wgsl(DECAL_ATLAS_WGSL.into()),
            },
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("decal atlas"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("decal atlas"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        // A texture scaled down into its cells reads its own levels.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("decal atlas"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let pipeline = |entry: &str| {
            crate::traffic::create_render_pipeline(
                device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some("decal atlas"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                },
            )
        };
        DecalCopies {
            color: pipeline("fs_color"),
            normal: pipeline("fs_normal"),
            normal_srgb: pipeline("fs_normal_srgb"),
            normal_rg: pipeline("fs_normal_rg"),
            layout,
            sampler,
        }
    }
}

/// One texture to copy into its cells this frame.
pub(crate) struct Blit<'a> {
    pub key: Key,
    pub source: &'a wgpu::TextureView,
    pub copy: Copy,
}

pub(crate) struct DecalAtlas {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    level0: wgpu::TextureView,
    /// Its side in cells.
    side: u32,
    /// How many times every texture's sides are halved.
    shift: u32,
    shelves: Shelves,
    entries: HashMap<Key, Entry>,
    /// Moves when the texture is made again: what binds `view` binds the
    /// new one.
    pub generation: u32,
    /// Whether level 0 holds nothing its entries can keep (the texture is
    /// new or was packed again): the next copies clear it first.
    blank: bool,
}

fn texture(
    device: &wgpu::Device,
    side: u32,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("decal atlas"),
        size: wgpu::Extent3d {
            width: side * CELL,
            height: side * CELL,
            depth_or_array_layers: 1,
        },
        mip_level_count: LEVELS,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        texture_binding_view_dimension: None,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let level0 = crate::mips::level_view(&texture, "decal atlas", 0);
    (texture, view, level0)
}

impl DecalAtlas {
    /// The sides (in cells) this device can hold, smallest first.
    fn sides(device: &wgpu::Device) -> impl Iterator<Item = u32> {
        let max = device.limits().max_texture_dimension_2d;
        SIDES
            .into_iter()
            .filter(move |&s| s <= max || s == SIDES[0])
            .map(|s| s / CELL)
    }

    pub fn new(device: &wgpu::Device) -> DecalAtlas {
        let side = SIDES[0] / CELL;
        let (texture, view, level0) = texture(device, side);
        DecalAtlas {
            texture,
            view,
            level0,
            side,
            shift: 0,
            shelves: Shelves::new(side),
            entries: HashMap::new(),
            generation: 0,
            blank: true,
        }
    }

    /// Bytes allocated, every level.
    pub fn bytes(&self) -> u64 {
        let texels = u64::from(self.side * CELL).pow(2);
        (0..LEVELS).map(|l| (texels >> (2 * l)) * 4).sum()
    }

    /// `uv` (a region of the texture `key`, `[u0, v0, u1, v1]`) in the
    /// atlas; `None` for a texture the atlas does not hold.
    pub fn region(&self, key: Key, uv: [f32; 4]) -> Option<[f32; 4]> {
        let [x, y, w, h] = self
            .entries
            .get(&key)?
            .cells
            .map(|c| c as f32 / self.side as f32);
        Some([x + uv[0] * w, y + uv[1] * h, x + uv[2] * w, y + uv[3] * h])
    }

    /// Gives every one of `wanted` its cells, growing or packing the atlas
    /// again when one finds no room, and answers the keys whose texels
    /// must be copied in this frame (new, moved, changed or live).
    pub fn place(&mut self, device: &wgpu::Device, wanted: &[Wanted]) -> Vec<Key> {
        let mut copies = Vec::new();
        let mut full = false;
        for w in wanted {
            let [cw, ch] = cells_of(w.size, self.shift);
            match self.entries.get_mut(&w.key) {
                Some(e) if e.cells[2..] == [cw, ch] => {
                    if e.revision != w.revision || w.live {
                        e.revision = w.revision;
                        copies.push(w.key);
                    }
                }
                // New, or its texture changed size: fresh cells (the old
                // ones are free again at the next repack).
                _ => match self.shelves.place(cw, ch) {
                    Some([x, y]) => {
                        self.entries.insert(
                            w.key,
                            Entry {
                                cells: [x, y, cw, ch],
                                revision: w.revision,
                            },
                        );
                        copies.push(w.key);
                    }
                    None => full = true,
                },
            }
        }
        if !full {
            return copies;
        }
        self.repack(device, wanted);
        wanted
            .iter()
            .map(|w| w.key)
            .filter(|k| self.entries.contains_key(k))
            .collect()
    }

    /// Packs only `wanted`, in the smallest atlas not smaller than this one
    /// that holds them whole, or in the largest with every texture halved
    /// as often as it takes.
    fn repack(&mut self, device: &wgpu::Device, wanted: &[Wanted]) {
        let largest = Self::sides(device)
            .last()
            .unwrap_or(self.side)
            .max(self.side);
        let tries = Self::sides(device)
            .filter(|&s| s >= self.side)
            .map(|s| (s, 0))
            .chain((1..LEVELS).map(|shift| (largest, shift)));
        let mut packed = None;
        for (side, shift) in tries {
            let sizes: Vec<[u32; 2]> = wanted.iter().map(|w| cells_of(w.size, shift)).collect();
            if let Some((shelves, at)) = pack(side, &sizes) {
                packed = Some((side, shift, shelves, at, sizes));
                break;
            }
        }
        // More textures than the largest atlas has cells: the first that
        // fit, a cell each; the rest draw untextured.
        let (side, shift, shelves, at, sizes) = packed.unwrap_or_else(|| {
            let room = (largest * largest) as usize;
            let sizes = vec![[1, 1]; wanted.len().min(room)];
            let (shelves, at) = pack(largest, &sizes).expect("a cell each fits");
            (largest, LEVELS - 1, shelves, at, sizes)
        });
        if side != self.side {
            let (texture, view, level0) = texture(device, side);
            self.texture = texture;
            self.view = view;
            self.level0 = level0;
            self.generation = self.generation.wrapping_add(1);
        }
        self.side = side;
        self.shift = shift;
        self.shelves = shelves;
        self.blank = true;
        self.entries = wanted
            .iter()
            .zip(at.iter().zip(&sizes))
            .map(|(w, (&[x, y], &[cw, ch]))| {
                (
                    w.key,
                    Entry {
                        cells: [x, y, cw, ch],
                        revision: w.revision,
                    },
                )
            })
            .collect();
    }

    /// Copies `blits` into their cells and fills the levels below from
    /// level 0.
    pub fn copy(
        &mut self,
        device: &wgpu::Device,
        copies: &DecalCopies,
        mips: &mut crate::mips::MipGenerator,
        blits: &[Blit],
        encoder: &mut crate::Encoder,
    ) {
        if blits.is_empty() && !self.blank {
            return;
        }
        let group = |source: &wgpu::TextureView| {
            crate::bind_group(
                device,
                "decal atlas",
                &copies.layout,
                &[
                    wgpu::BindingResource::TextureView(source),
                    wgpu::BindingResource::Sampler(&copies.sampler),
                ],
            )
        };
        let draws: Vec<([u32; 4], Copy, wgpu::BindGroup)> = blits
            .iter()
            .filter_map(|b| Some((self.entries.get(&b.key)?.cells, b.copy, group(b.source))))
            .collect();
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("decal atlas"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.level0,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if self.blank {
                            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                        } else {
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            for (cells, copy, group) in &draws {
                let [x, y, w, h] = cells.map(|c| c * CELL);
                pass.set_pipeline(match copy {
                    Copy::Color => &copies.color,
                    Copy::Normal => &copies.normal,
                    Copy::NormalSrgb => &copies.normal_srgb,
                    Copy::NormalRg => &copies.normal_rg,
                });
                pass.set_bind_group(0, group, &[]);
                pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
                pass.set_scissor_rect(x, y, w, h);
                pass.draw(0..3, 0..1);
            }
        }
        self.blank = false;
        mips.generate(device, encoder, &self.texture);
    }
}
