//! Terrain (phase 2): a height field over a rectangle, drawn as a geometry
//! clipmap around the camera (`shaders/terrain.wgsl`), shaded with up to
//! four layered materials (`shaders/terrain_lit.wgsl`), casting and
//! receiving the directional light's cascaded shadows, under the fog.
//!
//! [`TerrainData`] is the geometry (heights, an optional control map),
//! added once with [`crate::RenderScene::add_terrain`] and replaced with
//! [`crate::RenderScene::set_terrain`] (a runtime edit); a [`Terrain`] in
//! [`crate::RenderScene::terrains`] draws it this frame at an origin with
//! its layers.
//!
//! **The clipmap.** Level `l` is a grid of `ring` × `ring` quads of
//! `spacing · 2^l` metres centred on the camera, its centre snapped to
//! twice its spacing so its vertices never move in the world; every level
//! but the finest leaves out the quads the finer level covers. The levels
//! stop when the next one's hole would cover the whole terrain or lie past
//! the camera's far plane. Near its outer edge a level's odd vertices
//! slide onto the coarser grid (CDLOD's morph), so at the edge both levels
//! have the same vertices with the same heights: no cracks, no T-junctions,
//! and no pop when a level re-centres. Positions are computed in whole grid
//! units, so the shared vertices are bit-identical. Heights are sampled
//! bilinearly from an `R32Float` texture in the vertex shader; the normal
//! map (built from the heights on the GPU, mipmapped) and the control map
//! are filtered per pixel.
//!
//! **Layers.** Each layer's weight is the control map's channel (1 without
//! a map) times its slope rule (degrees from flat) times its altitude rule
//! (metres above the origin), each a band with soft edges; with
//! `height_blend` > 0 the albedo's alpha (the layer's height map) sharpens
//! the transitions. Weights are normalised; where every weight is 0 the
//! first layer shows. A layer is sampled over world xz every `tile` metres
//! or triplanar (cliffs), with an optional normal map.

use crate::scene::{TerrainHandle, TextureHandle};
use glam::{Mat4, Vec2, Vec3};
use std::sync::Arc;

/// The most layers a [`Terrain`] blends.
pub const MAX_TERRAIN_LAYERS: usize = 4;
/// The most clipmap levels a terrain draws.
pub const MAX_TERRAIN_LEVELS: usize = 16;

/// A terrain's layer weights: RGBA8, one channel per layer (linear), rows
/// from -z to +z, columns from -x to +x, stretched over the terrain.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainControl {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

/// A terrain's geometry: `columns` × `rows` heights in metres over a
/// `size` rectangle centred on the terrain's origin, row-major from -z to
/// +z and -x to +x (sample `(c, r)` at `-size / 2 + (c, r) · cell`), and
/// an optional control map.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainData {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub heights: Arc<[f32]>,
    pub control: Option<TerrainControl>,
}

impl TerrainData {
    /// A flat terrain of `columns` × `rows` samples at height 0.
    pub fn flat(size: Vec2, columns: u32, rows: u32) -> TerrainData {
        TerrainData {
            size,
            columns,
            rows,
            heights: Arc::from(vec![0.0; (columns * rows) as usize]),
            control: None,
        }
    }

    /// The distance between two samples along x and z.
    pub fn cell(&self) -> Vec2 {
        self.size
            / Vec2::new(
                self.columns.max(2) as f32 - 1.0,
                self.rows.max(2) as f32 - 1.0,
            )
    }

    /// Why the data cannot be drawn, if it cannot.
    pub fn check(&self) -> Result<(), String> {
        if self.columns < 2 || self.rows < 2 {
            return Err(format!(
                "a terrain needs at least 2 × 2 samples, got {} × {}",
                self.columns, self.rows
            ));
        }
        if self.heights.len() != (self.columns * self.rows) as usize {
            return Err(format!(
                "a {} × {} terrain needs {} heights, got {}",
                self.columns,
                self.rows,
                self.columns * self.rows,
                self.heights.len()
            ));
        }
        if !(self.size.x > 0.0 && self.size.y > 0.0 && self.size.is_finite()) {
            return Err(format!(
                "a terrain's size must be positive, got {}",
                self.size
            ));
        }
        if self.heights.iter().any(|h| !h.is_finite()) {
            return Err("a terrain's heights must be finite".into());
        }
        if let Some(c) = &self.control
            && (c.width == 0 || c.height == 0 || c.rgba.len() != (c.width * c.height * 4) as usize)
        {
            return Err(format!(
                "a {} × {} control map needs {} bytes, got {}",
                c.width,
                c.height,
                c.width * c.height * 4,
                c.rgba.len()
            ));
        }
        Ok(())
    }

    fn sample(&self, c: u32, r: u32) -> f32 {
        let c = c.min(self.columns - 1);
        let r = r.min(self.rows - 1);
        self.heights[(r * self.columns + c) as usize]
    }

    /// Sample coordinates of a point given relative to the terrain's centre.
    fn grid(&self, local: Vec2) -> Vec2 {
        let t = (local + self.size * 0.5) / self.cell();
        t.clamp(
            Vec2::ZERO,
            Vec2::new(self.columns as f32 - 1.0, self.rows as f32 - 1.0),
        )
    }

    /// The height at `local` (x and z relative to the terrain's centre) on
    /// the triangles the renderer draws at its finest level and physics
    /// collides with: each cell split along its (-x, -z) to (+x, +z)
    /// diagonal; clamped to the edge.
    pub fn height(&self, local: Vec2) -> f32 {
        let t = self.grid(local);
        let i = t
            .floor()
            .min(Vec2::new(self.columns as f32 - 2.0, self.rows as f32 - 2.0));
        let f = (t - i).clamp(Vec2::ZERO, Vec2::ONE);
        let (c, r) = (i.x as u32, i.y as u32);
        let h00 = self.sample(c, r);
        let h10 = self.sample(c + 1, r);
        let h01 = self.sample(c, r + 1);
        let h11 = self.sample(c + 1, r + 1);
        if f.y >= f.x {
            h00 + (h11 - h01) * f.x + (h01 - h00) * f.y
        } else {
            h00 + (h10 - h00) * f.x + (h11 - h10) * f.y
        }
    }

    fn sample_normal(&self, c: u32, r: u32) -> Vec3 {
        let cell = self.cell();
        let (c0, c1) = (c.saturating_sub(1), (c + 1).min(self.columns - 1));
        let (r0, r1) = (r.saturating_sub(1), (r + 1).min(self.rows - 1));
        let hx = (self.sample(c1, r) - self.sample(c0, r)) / ((c1 - c0).max(1) as f32 * cell.x);
        let hz = (self.sample(c, r1) - self.sample(c, r0)) / ((r1 - r0).max(1) as f32 * cell.y);
        Vec3::new(-hx, 1.0, -hz).normalize()
    }

    /// The surface normal at `local`: the samples' normals (central
    /// differences, as the renderer's normal map) interpolated bilinearly.
    pub fn normal(&self, local: Vec2) -> Vec3 {
        let t = self.grid(local);
        let i = t
            .floor()
            .min(Vec2::new(self.columns as f32 - 2.0, self.rows as f32 - 2.0));
        let f = (t - i).clamp(Vec2::ZERO, Vec2::ONE);
        let (c, r) = (i.x as u32, i.y as u32);
        let lo = self
            .sample_normal(c, r)
            .lerp(self.sample_normal(c + 1, r), f.x);
        let hi = self
            .sample_normal(c, r + 1)
            .lerp(self.sample_normal(c + 1, r + 1), f.x);
        lo.lerp(hi, f.y).normalize_or(Vec3::Y)
    }

    /// The lowest and highest height.
    pub fn range(&self) -> (f32, f32) {
        self.heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)))
    }
}

/// One material layer of a [`Terrain`]. The zero value is a plain grey
/// layer everywhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainLayer {
    /// The colour texture (sRGB; its alpha is the layer's height for
    /// `Terrain::height_blend`). `None`: `color` alone.
    pub albedo: Option<TextureHandle>,
    /// A tangent-space normal map (linear).
    pub normal: Option<TextureHandle>,
    /// Multiplies the albedo (linear).
    pub color: Vec3,
    pub roughness: f32,
    /// Metres per repeat of the textures.
    pub tile: f32,
    /// Sample by three projections blended by the normal (cliffs).
    pub triplanar: bool,
    pub normal_strength: f32,
    /// The slopes (degrees from flat) where the layer shows, and how far
    /// beyond them it fades out.
    pub slope: [f32; 2],
    pub slope_fade: f32,
    /// The heights above the terrain's origin where it shows, and the fade.
    pub altitude: [f32; 2],
    pub altitude_fade: f32,
}

impl Default for TerrainLayer {
    fn default() -> Self {
        TerrainLayer {
            albedo: None,
            normal: None,
            color: Vec3::splat(0.5),
            roughness: 0.9,
            tile: 4.0,
            triplanar: false,
            normal_strength: 1.0,
            slope: [0.0, 90.0],
            slope_fade: 4.0,
            altitude: [-1.0e6, 1.0e6],
            altitude_fade: 4.0,
        }
    }
}

/// How finely a terrain is drawn. The zero value is the default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainLod {
    /// The finest level's spacing in metres (0: the data's sample spacing).
    pub spacing: f32,
    /// Quads per side of every level (0: 64; rounded to a multiple of 4,
    /// 16 to 256). More quads push each level's detail further out.
    pub ring: u32,
}

impl TerrainLod {
    fn ring(&self) -> u32 {
        let r = if self.ring == 0 { 64 } else { self.ring };
        (r.clamp(16, 256) / 4) * 4
    }
}

/// A terrain drawn this frame: its data at `origin` (the centre of the
/// height field at height 0) with its layers.
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub terrain: TerrainHandle,
    pub origin: Vec3,
    /// Up to [`MAX_TERRAIN_LAYERS`] layers (more are ignored).
    pub layers: Vec<TerrainLayer>,
    /// How much the layers' heights (their albedo's alpha) sharpen their
    /// transitions (0: weights alone).
    pub height_blend: f32,
    pub cast_shadows: bool,
    pub lod: TerrainLod,
}

impl Terrain {
    pub fn new(terrain: TerrainHandle) -> Terrain {
        Terrain {
            terrain,
            origin: Vec3::ZERO,
            layers: vec![TerrainLayer::default()],
            height_blend: 0.0,
            cast_shadows: true,
            lod: TerrainLod::default(),
        }
    }
}

/// `TerrainLayerGpu` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerGpu {
    color: [f32; 4],
    mapping: [f32; 4],
    slope: [f32; 4],
    altitude: [f32; 4],
}

/// `TerrainUniform` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainGpu {
    origin: [f32; 4],
    extent: [f32; 4],
    params: [f32; 4],
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    layers: [LayerGpu; MAX_TERRAIN_LAYERS],
}

/// `TerrainLevel` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LevelGpu {
    center: [f32; 4],
    grid: [f32; 4],
    camera: [f32; 4],
}

const LEVEL_STRIDE: u64 = 256;

/// One clipmap level to draw: its uniform slot, which of the index
/// buffer's ranges, and its bounds (culling per pass).
#[derive(Clone, Debug)]
pub(crate) struct LevelDraw {
    pub offset: u32,
    pub indices: std::ops::Range<u32>,
    pub center: Vec3,
    pub radius: f32,
}

/// One terrain ready to draw in this frame's passes.
pub(crate) struct TerrainDraw {
    pub group: wgpu::BindGroup,
    pub levels: Vec<LevelDraw>,
    pub cast_shadows: bool,
}

/// The levels of one terrain for a camera, in whole grid units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Level {
    /// The centre (grid units from the terrain's -x, -z corner).
    pub center: [f64; 2],
    /// Grid units per quad (2^level).
    pub step: f64,
    /// Which ring variant leaves the finer level out (None: the finest level).
    pub hole: Option<(u32, u32)>,
    pub morph: bool,
}

/// The clipmap levels around `camera` (grid units, the terrain's -x, -z
/// corner at 0) over a terrain `extent` grid units wide, with `ring` quads
/// per side, out to `far` grid units.
pub(crate) fn levels(camera: Vec2, extent: Vec2, ring: u32, far: f64) -> Vec<Level> {
    let cam = [camera.x as f64, camera.y as f64];
    let ext = [extent.x as f64, extent.y as f64];
    let quarter = ring as f64 / 4.0;
    let half = ring as f64 / 2.0;
    let mut out: Vec<Level> = Vec::new();
    let mut prev: Option<[f64; 2]> = None;
    for l in 0..MAX_TERRAIN_LEVELS {
        let step = (1u64 << l) as f64;
        let c = cam.map(|x| (x / (2.0 * step)).floor() * 2.0 * step);
        let hole = prev.map(|p| {
            let o = |i: usize| ((p[i] - c[i]) / step).round().clamp(0.0, 1.0) as u32;
            (o(0), o(1))
        });
        if let Some(p) = prev {
            // The finer level's square: past it nothing is left to draw.
            let r = quarter * step;
            let covers = (0..2).all(|i| p[i] - r <= 0.0 && p[i] + r >= ext[i]);
            if covers || r > far {
                break;
            }
        }
        let r = half * step;
        let touches = (0..2).all(|i| c[i] + r >= 0.0 && c[i] - r <= ext[i]);
        if touches {
            out.push(Level {
                center: c,
                step,
                hole,
                morph: true,
            });
        }
        prev = Some(c);
    }
    if let Some(last) = out.last_mut() {
        last.morph = false;
    }
    out
}

/// The index buffer of one ring size: the full grid, then the ring with
/// the hole at each of its four offsets. Returns the indices and the
/// ranges.
fn ring_indices(n: u32) -> (Vec<u32>, [std::ops::Range<u32>; 5]) {
    let row = n + 1;
    let mut idx = Vec::new();
    let mut ranges: Vec<std::ops::Range<u32>> = Vec::new();
    let quad = |idx: &mut Vec<u32>, x: u32, z: u32| {
        let a = z * row + x;
        let b = (z + 1) * row + x;
        // Counter-clockwise seen from above (+y): (x, z), (x, z + 1), (x + 1, z + 1).
        idx.extend([a, b, b + 1, a, b + 1, a + 1]);
    };
    let start = idx.len() as u32;
    for z in 0..n {
        for x in 0..n {
            quad(&mut idx, x, z);
        }
    }
    ranges.push(start..idx.len() as u32);
    for (ox, oz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let start = idx.len() as u32;
        let (x0, z0) = (n / 4 + ox, n / 4 + oz);
        let (x1, z1) = (3 * n / 4 + ox, 3 * n / 4 + oz);
        for z in 0..n {
            for x in 0..n {
                if !(x >= x0 && x < x1 && z >= z0 && z < z1) {
                    quad(&mut idx, x, z);
                }
            }
        }
        ranges.push(start..idx.len() as u32);
    }
    let ranges: [std::ops::Range<u32>; 5] = ranges.try_into().expect("five ranges");
    (idx, ranges)
}

struct GpuData {
    revision: u64,
    heights: wgpu::Texture,
    heights_view: wgpu::TextureView,
    normals: wgpu::Texture,
    normals_view: wgpu::TextureView,
    normals_group: wgpu::BindGroup,
    normals_uniform: wgpu::Buffer,
    control: Option<(u32, u32, wgpu::Texture, wgpu::TextureView)>,
    columns: u32,
    rows: u32,
    size: Vec2,
    range: (f32, f32),
}

struct Slot {
    uniform: wgpu::Buffer,
    levels: wgpu::Buffer,
    /// The bind group and what it was made from: (data index, data
    /// revision, the layers' textures).
    group: Option<(GroupKey, wgpu::BindGroup)>,
}

type GroupKey = (usize, u64, [Option<TextureHandle>; 2 * MAX_TERRAIN_LAYERS]);

/// Draws [`Terrain`]s: the pipelines, each data's textures, a uniform and
/// level buffer per drawn terrain, kept across frames.
pub(crate) struct TerrainRenderer {
    layout: wgpu::BindGroupLayout,
    pub lit: wgpu::RenderPipeline,
    pub prepass: wgpu::RenderPipeline,
    pub shadow: wgpu::RenderPipeline,
    normals_layout: wgpu::BindGroupLayout,
    normals_pipeline: wgpu::RenderPipeline,
    layer_sampler: wgpu::Sampler,
    map_sampler: wgpu::Sampler,
    white: wgpu::TextureView,
    flat_normal: wgpu::TextureView,
    indices: Option<(u32, wgpu::Buffer, [std::ops::Range<u32>; 5])>,
    scene: u64,
    datas: Vec<Option<GpuData>>,
    slots: Vec<Slot>,
}

pub(crate) const NORMALS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

/// What [`TerrainRenderer::new`] builds its pipelines from.
pub(crate) struct TerrainPipelines<'a> {
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub shadow_layout: &'a wgpu::BindGroupLayout,
    pub frame_wgsl: &'a str,
    pub lighting_wgsl: &'a str,
    /// The local lights' cluster tables (`clusters.rs`), after the frame.
    pub light_tables_wgsl: &'a str,
    pub samples: u32,
    pub white: wgpu::TextureView,
    pub flat_normal: wgpu::TextureView,
}

impl TerrainRenderer {
    pub(crate) fn new(device: &wgpu::Device, p: TerrainPipelines) -> TerrainRenderer {
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let uniform = |binding, dynamic| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: dynamic,
                min_binding_size: None,
            },
            count: None,
        };
        let sampler = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let mut entries = vec![uniform(0, false), uniform(1, true), tex(2, false)];
        entries.extend((3..13).map(|b| tex(b, true)));
        entries.extend([sampler(13), sampler(14)]);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain"),
            entries: &entries,
        });
        let common = include_str!("../shaders/terrain.wgsl");
        let lit_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}{}{}{common}{}",
                    p.frame_wgsl,
                    p.light_tables_wgsl,
                    p.lighting_wgsl,
                    include_str!("../shaders/terrain_lit.wgsl")
                )
                .into(),
            ),
        });
        let shadow_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain shadow"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{common}{}", include_str!("../shaders/terrain_shadow.wgsl")).into(),
            ),
        });
        let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let lit_layout = pl("terrain", &[p.frame_layout, &layout]);
        let shadow_layout = pl("terrain shadow", &[p.shadow_layout, &layout]);
        let lit = crate::hdr_pipeline(
            device,
            crate::HdrPipeline {
                label: "terrain",
                layout: &lit_layout,
                module: &lit_module,
                vs: "vs",
                fs: "fs",
                buffers: &[],
                cull: Some(wgpu::Face::Back),
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples: p.samples,
            },
        );
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let depth = |write| {
            Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(write),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            })
        };
        let prepass = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain prepass"),
            layout: Some(&lit_layout),
            vertex: wgpu::VertexState {
                module: &lit_module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: depth(true),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &lit_module,
                entry_point: Some("fs_prepass"),
                compilation_options: Default::default(),
                targets: &[
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                ],
            }),
            multiview_mask: None,
            cache: None,
        });
        // An open surface: both faces cast (the bias in the lit pass keeps
        // its own lit side from shadowing itself).
        let shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain shadow"),
            layout: Some(&shadow_layout),
            vertex: wgpu::VertexState {
                module: &shadow_module,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: depth(true),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shadow_module,
                entry_point: Some("fs_shadow"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::RED,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let normals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain normals"),
            entries: &[uniform(0, false), tex(1, false)],
        });
        let normals_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain normals"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/terrain_normals.wgsl").into(),
            ),
        });
        let normals_pl = pl("terrain normals", &[&normals_layout]);
        let normals_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain normals"),
            layout: Some(&normals_pl),
            vertex: wgpu::VertexState {
                module: &normals_module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &normals_module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[target(NORMALS_FORMAT)],
            }),
            multiview_mask: None,
            cache: None,
        });
        let layer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain layers"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain maps"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        TerrainRenderer {
            layout,
            lit,
            prepass,
            shadow,
            normals_layout,
            normals_pipeline,
            layer_sampler,
            map_sampler,
            white: p.white,
            flat_normal: p.flat_normal,
            indices: None,
            scene: 0,
            datas: Vec::new(),
            slots: Vec::new(),
        }
    }

    /// The index buffer (bound by the passes that draw terrain).
    pub(crate) fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref().map(|(_, b, _)| b)
    }

    /// Uploads new or edited terrain data (heights, then the normal map
    /// and the mips, recorded into `encoder`).
    fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mips: &mut crate::mips::MipGenerator,
        scene: &crate::RenderScene,
    ) -> Result<(), crate::RenderError> {
        if self.scene != scene.id() {
            self.scene = scene.id();
            self.datas.clear();
            for s in &mut self.slots {
                s.group = None;
            }
        }
        let (datas, revisions) = scene.terrain_datas();
        self.datas.resize_with(datas.len(), || None);
        // Only the data a terrain draws this frame is uploaded.
        let wanted: std::collections::BTreeSet<usize> =
            scene.terrains.iter().map(|t| t.terrain.index()).collect();
        for &i in &wanted {
            let Some(data) = datas.get(i) else { continue };
            if self.datas[i]
                .as_ref()
                .is_some_and(|g| g.revision == revisions[i])
            {
                continue;
            }
            data.check()
                .map_err(|e| crate::RenderError::InvalidData(format!("terrain {i}: {e}")))?;
            let reuse = self.datas[i]
                .take()
                .filter(|g| g.columns == data.columns && g.rows == data.rows);
            let gpu = self.upload(device, queue, encoder, mips, data, revisions[i], reuse);
            self.datas[i] = Some(gpu);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn upload(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mips: &mut crate::mips::MipGenerator,
        data: &TerrainData,
        revision: u64,
        reuse: Option<GpuData>,
    ) -> GpuData {
        let size = wgpu::Extent3d {
            width: data.columns,
            height: data.rows,
            depth_or_array_layers: 1,
        };
        let levels = crate::mips::mip_count(data.columns, data.rows);
        let mut gpu = match reuse {
            Some(g) => g,
            None => {
                let heights = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("terrain heights"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let normals = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("terrain normals"),
                    size,
                    mip_level_count: levels,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: NORMALS_FORMAT,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let heights_view = heights.create_view(&Default::default());
                let normals_view = normals.create_view(&Default::default());
                let normals_uniform = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain normals"),
                    size: 16,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let normals_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("terrain normals"),
                    layout: &self.normals_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: normals_uniform.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&heights_view),
                        },
                    ],
                });
                GpuData {
                    revision,
                    heights,
                    heights_view,
                    normals,
                    normals_view,
                    normals_group,
                    normals_uniform,
                    control: None,
                    columns: data.columns,
                    rows: data.rows,
                    size: data.size,
                    range: (0.0, 0.0),
                }
            }
        };
        gpu.revision = revision;
        gpu.size = data.size;
        gpu.range = data.range();
        queue.write_texture(
            gpu.heights.as_image_copy(),
            bytemuck::cast_slice(&data.heights),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(data.columns * 4),
                rows_per_image: None,
            },
            size,
        );
        queue.write_buffer(
            &gpu.normals_uniform,
            0,
            bytemuck::cast_slice(&[
                data.size.x,
                data.size.y,
                data.columns as f32,
                data.rows as f32,
            ]),
        );
        {
            let target = gpu.normals.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terrain normals"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.normals_pipeline);
            pass.set_bind_group(0, &gpu.normals_group, &[]);
            pass.draw(0..3, 0..1);
        }
        mips.generate(device, encoder, &gpu.normals);
        gpu.control = match &data.control {
            None => None,
            Some(c) => {
                let csize = wgpu::Extent3d {
                    width: c.width,
                    height: c.height,
                    depth_or_array_layers: 1,
                };
                let (tex, view) = match gpu.control.take() {
                    Some((w, h, t, v)) if (w, h) == (c.width, c.height) => (t, v),
                    _ => {
                        let t = device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("terrain control"),
                            size: csize,
                            mip_level_count: crate::mips::mip_count(c.width, c.height),
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST
                                | wgpu::TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        });
                        let v = t.create_view(&Default::default());
                        (t, v)
                    }
                };
                queue.write_texture(
                    tex.as_image_copy(),
                    &c.rgba,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(c.width * 4),
                        rows_per_image: None,
                    },
                    csize,
                );
                mips.generate(device, encoder, &tex);
                Some((c.width, c.height, tex, view))
            }
        };
        gpu
    }

    /// Uploads what changed and lays out this frame's levels for every
    /// terrain around `camera`. `textures` resolves a layer's texture
    /// handle (`None`: not a texture of this scene).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mips: &mut crate::mips::MipGenerator,
        scene: &crate::RenderScene,
        camera: Vec3,
        far: f32,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        textures: impl Fn(TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Result<Vec<TerrainDraw>, crate::RenderError> {
        if scene.terrains.is_empty() {
            return Ok(Vec::new());
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("terrain upload"),
        });
        self.sync(device, queue, &mut encoder, mips, scene)?;
        queue.submit([encoder.finish()]);
        let ring = scene
            .terrains
            .iter()
            .map(|t| t.lod.ring())
            .max()
            .unwrap_or(64);
        if self.indices.as_ref().is_none_or(|(n, ..)| *n != ring) {
            use wgpu::util::DeviceExt;
            let (idx, ranges) = ring_indices(ring);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("terrain indices"),
                contents: bytemuck::cast_slice(&idx),
                usage: wgpu::BufferUsages::INDEX,
            });
            self.indices = Some((ring, buffer, ranges));
        }
        let ranges = self.indices.as_ref().expect("made above").2.clone();
        while self.slots.len() < scene.terrains.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain"),
                    size: std::mem::size_of::<TerrainGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                levels: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain levels"),
                    size: LEVEL_STRIDE * MAX_TERRAIN_LEVELS as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                group: None,
            });
        }
        let mut draws = Vec::with_capacity(scene.terrains.len());
        for (ti, t) in scene.terrains.iter().enumerate() {
            let di = t.terrain.index();
            let Some(Some(data)) = self.datas.get(di) else {
                return Err(crate::RenderError::InvalidHandle(format!("terrain {di}")));
            };
            let mut layers = [LayerGpu::default(); MAX_TERRAIN_LAYERS];
            let mut handles = [None; 2 * MAX_TERRAIN_LAYERS];
            let count = t.layers.len().min(MAX_TERRAIN_LAYERS);
            for (i, l) in t.layers.iter().take(MAX_TERRAIN_LAYERS).enumerate() {
                for (k, h) in [(i, l.albedo), (MAX_TERRAIN_LAYERS + i, l.normal)] {
                    if let Some(h) = h {
                        if textures(h).is_none() {
                            return Err(crate::RenderError::InvalidHandle(format!(
                                "terrain layer {i} texture {}",
                                h.index()
                            )));
                        }
                        handles[k] = Some(h);
                    }
                }
                let bits = u32::from(l.albedo.is_some()) | (u32::from(l.normal.is_some()) << 1);
                layers[i] = LayerGpu {
                    color: l.color.extend(l.roughness.clamp(0.04, 1.0)).to_array(),
                    mapping: [
                        1.0 / l.tile.max(1e-3),
                        f32::from(l.triplanar),
                        l.normal_strength,
                        bits as f32,
                    ],
                    slope: [l.slope[0], l.slope[1], l.slope_fade.max(0.0), 0.0],
                    altitude: [l.altitude[0], l.altitude[1], l.altitude_fade.max(0.0), 0.0],
                };
            }
            let gpu = TerrainGpu {
                origin: t.origin.extend(count as f32).to_array(),
                extent: [
                    data.size.x,
                    data.size.y,
                    data.columns as f32,
                    data.rows as f32,
                ],
                params: [
                    t.height_blend.max(0.0),
                    f32::from(data.control.is_some()),
                    0.0,
                    0.0,
                ],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                layers,
            };
            let slot = &mut self.slots[ti];
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));

            // The levels around the camera, in grid units.
            let cell = data.size / Vec2::new(data.columns as f32 - 1.0, data.rows as f32 - 1.0);
            let unit = if t.lod.spacing > 0.0 {
                t.lod.spacing
            } else {
                cell.x.min(cell.y)
            };
            let lo = Vec2::new(t.origin.x, t.origin.z) - data.size * 0.5;
            let cam = (Vec2::new(camera.x, camera.z) - lo) / unit;
            let extent = data.size / unit;
            let n = t.lod.ring();
            let list = levels(cam, extent, n, (far / unit) as f64);
            let morph_width = (n / 8).max(2) as f32;
            let morph_start = n as f32 * 0.5 - 2.0 - morph_width;
            let mut level_bytes = vec![0u8; LEVEL_STRIDE as usize * list.len().max(1)];
            let mut out = Vec::with_capacity(list.len());
            let (hmin, hmax) = data.range;
            for (k, l) in list.iter().enumerate() {
                let g = LevelGpu {
                    center: [
                        l.center[0] as f32,
                        l.center[1] as f32,
                        l.step as f32,
                        k as f32,
                    ],
                    grid: [
                        n as f32,
                        morph_start,
                        if l.morph { morph_width } else { 0.0 },
                        0.0,
                    ],
                    camera: [cam.x, cam.y, unit, 0.0],
                };
                let at = k * LEVEL_STRIDE as usize;
                level_bytes[at..at + std::mem::size_of::<LevelGpu>()]
                    .copy_from_slice(bytemuck::bytes_of(&g));
                let indices = match l.hole {
                    None => ranges[0].clone(),
                    Some((ox, oz)) => ranges[1 + (ox + 2 * oz) as usize].clone(),
                };
                // Bounds: the level's square on the terrain, its height range.
                let half = n as f64 * 0.5 * l.step;
                let a = Vec2::new(
                    ((l.center[0] - half).max(0.0) * unit as f64) as f32,
                    ((l.center[1] - half).max(0.0) * unit as f64) as f32,
                );
                let b = Vec2::new(
                    ((l.center[0] + half) * unit as f64).min(data.size.x as f64) as f32,
                    ((l.center[1] + half) * unit as f64).min(data.size.y as f64) as f32,
                );
                let box_min = Vec3::new(lo.x + a.x, t.origin.y + hmin, lo.y + a.y);
                let box_max = Vec3::new(lo.x + b.x, t.origin.y + hmax, lo.y + b.y);
                out.push(LevelDraw {
                    offset: (k as u64 * LEVEL_STRIDE) as u32,
                    indices,
                    center: (box_min + box_max) * 0.5,
                    radius: (box_max - box_min).length() * 0.5,
                });
            }
            queue.write_buffer(&slot.levels, 0, &level_bytes);
            let key: GroupKey = (di, data.revision, handles);
            if slot.group.as_ref().is_none_or(|(k, _)| *k != key) {
                let white = &self.white;
                let flat = &self.flat_normal;
                let view = |h: Option<TextureHandle>, fallback| match h {
                    Some(h) => textures(h).unwrap_or(fallback),
                    None => fallback,
                };
                let control = data.control.as_ref().map_or(white, |c| &c.3);
                let mut resources = vec![
                    slot.uniform.as_entire_binding(),
                    wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &slot.levels,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<LevelGpu>() as u64),
                    }),
                    wgpu::BindingResource::TextureView(&data.heights_view),
                    wgpu::BindingResource::TextureView(&data.normals_view),
                    wgpu::BindingResource::TextureView(control),
                ];
                for h in &handles[..MAX_TERRAIN_LAYERS] {
                    resources.push(wgpu::BindingResource::TextureView(view(*h, white)));
                }
                for h in &handles[MAX_TERRAIN_LAYERS..] {
                    resources.push(wgpu::BindingResource::TextureView(view(*h, flat)));
                }
                resources.push(wgpu::BindingResource::Sampler(&self.layer_sampler));
                resources.push(wgpu::BindingResource::Sampler(&self.map_sampler));
                let entries: Vec<_> = resources
                    .into_iter()
                    .enumerate()
                    .map(|(i, resource)| wgpu::BindGroupEntry {
                        binding: i as u32,
                        resource,
                    })
                    .collect();
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("terrain"),
                    layout: &self.layout,
                    entries: &entries,
                });
                slot.group = Some((key, group));
            }
            draws.push(TerrainDraw {
                group: slot.group.as_ref().expect("made above").1.clone(),
                levels: out,
                cast_shadows: t.cast_shadows,
            });
        }
        Ok(draws)
    }

    /// The world bounds of the terrains drawn this frame (centre, radius),
    /// for the shadow fit.
    pub(crate) fn bounds(&self, scene: &crate::RenderScene) -> Vec<(Vec3, f32)> {
        scene
            .terrains
            .iter()
            .filter_map(|t| {
                let d = self.datas.get(t.terrain.index())?.as_ref()?;
                let lo = Vec3::new(
                    t.origin.x - d.size.x * 0.5,
                    t.origin.y + d.range.0,
                    t.origin.z - d.size.y * 0.5,
                );
                let hi = Vec3::new(
                    t.origin.x + d.size.x * 0.5,
                    t.origin.y + d.range.1,
                    t.origin.z + d.size.y * 0.5,
                );
                Some(((lo + hi) * 0.5, (hi - lo).length() * 0.5))
            })
            .collect()
    }
}

/// Draws the levels of `draws` that touch `visible` (group 1: the terrain;
/// group 0 bound by the caller). Returns (draws, triangles).
pub(crate) fn draw_levels(
    pass: &mut wgpu::RenderPass,
    draws: &[TerrainDraw],
    indices: &wgpu::Buffer,
    shadows_only: bool,
    visible: impl Fn(Vec3, f32) -> bool,
) -> (u32, u64) {
    let mut calls = 0;
    let mut triangles = 0;
    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
    for d in draws {
        if shadows_only && !d.cast_shadows {
            continue;
        }
        for l in &d.levels {
            if !visible(l.center, l.radius) {
                continue;
            }
            pass.set_bind_group(1, &d.group, &[l.offset]);
            pass.draw_indexed(l.indices.clone(), 0, 0..1);
            calls += 1;
            triangles += (l.indices.len() / 3) as u64;
        }
    }
    (calls, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every level's hole (the ring variant its index range draws) is
    /// exactly the finer level's square, so the levels tile the plane: no
    /// quad is drawn twice and none is missing.
    #[test]
    fn levels_tile_the_plane_without_overlap_or_gaps() {
        let ring = 16u32;
        let n = ring as f64;
        for cam in [
            Vec2::new(500.3, 700.9),
            Vec2::new(3.0, 1.0),
            Vec2::new(1023.99, 0.01),
        ] {
            let list = levels(cam, Vec2::splat(1024.0), ring, 1e9);
            assert!(list.len() >= 4, "{list:?}");
            let outer = &list[list.len() - 1];
            let reach = n * 0.5 * outer.step;
            let (cx, cz) = (cam.x as i64, cam.y as i64);
            for z in (cz - 120)..(cz + 120) {
                for x in (cx - 120)..(cx + 120) {
                    let p = [x as f64 + 0.5, z as f64 + 0.5];
                    if !(0..2).all(|i| (p[i] - outer.center[i]).abs() < reach) {
                        continue;
                    }
                    let mut hits = 0;
                    for l in &list {
                        // The level's quads, less its ring variant's hole.
                        let lo = [0, 1].map(|i| l.center[i] - n * 0.5 * l.step);
                        let q = [0, 1].map(|i| ((p[i] - lo[i]) / l.step).floor());
                        if q.iter().any(|&q| q < 0.0 || q >= n) {
                            continue;
                        }
                        if let Some((ox, oz)) = l.hole {
                            let o = [ox as f64, oz as f64];
                            if (0..2).all(|i| q[i] >= n / 4.0 + o[i] && q[i] < 3.0 * n / 4.0 + o[i])
                            {
                                continue;
                            }
                        }
                        hits += 1;
                    }
                    assert_eq!(hits, 1, "cell {x},{z} drawn {hits} times for camera {cam}");
                }
            }
        }
    }
}

// ── scatter ─────────────────────────────────────────────────────────────────

/// The most instances one [`TerrainScatter`] places in a frame.
pub const MAX_SCATTER_INSTANCES: usize = 65_536;

/// Detail meshes (grass, flowers, stones) scattered over a terrain where
/// one of its layers shows, near the camera: [`Terrain::scatter`] places
/// them on a jittered grid fixed to the terrain, so an instance keeps its
/// place, size and turn as the camera moves, and each cell grows one with
/// the probability of the layer's weight there. Instances shrink away over
/// the outer `fade` of `distance`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainScatter {
    /// The layer whose weight (control map, slope and altitude rules,
    /// normalised over the layers) places the instances.
    pub layer: u32,
    /// Instances per square metre where the layer's weight is 1.
    pub density: f32,
    /// How far from the camera (along x and z) instances are placed.
    pub distance: f32,
    /// The outer part of `distance` over which they shrink to nothing (0..1).
    pub fade: f32,
    /// The range of their uniform scale.
    pub scale: [f32; 2],
    /// How far they lean with the ground: 0 upright, 1 along its normal.
    pub align: f32,
    pub seed: u32,
}

impl Default for TerrainScatter {
    fn default() -> Self {
        TerrainScatter {
            layer: 0,
            density: 2.0,
            distance: 40.0,
            fade: 0.3,
            scale: [0.8, 1.2],
            align: 0.0,
            seed: 1,
        }
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `band` in terrain_lit.wgsl.
fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
    let f = fade.max(1e-4);
    smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x))
}

/// A 32-bit integer hash (lowbias32).
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / 16_777_216.0
}

impl TerrainData {
    /// The control map's RGBA at `local`, bilinear between texel centres
    /// (as the renderer's linear sampler reads it), 0..1; all 1 without one.
    pub fn control_at(&self, local: Vec2) -> [f32; 4] {
        let Some(c) = &self.control else {
            return [1.0; 4];
        };
        let uv = ((local + self.size * 0.5) / self.size).clamp(Vec2::ZERO, Vec2::ONE);
        let t = uv * Vec2::new(c.width as f32, c.height as f32) - 0.5;
        let i = t.floor();
        let f = t - i;
        let texel = |x: f32, y: f32| {
            let x = (x as i64).clamp(0, c.width as i64 - 1) as usize;
            let y = (y as i64).clamp(0, c.height as i64 - 1) as usize;
            let at = (y * c.width as usize + x) * 4;
            [0, 1, 2, 3].map(|k| c.rgba[at + k] as f32 / 255.0)
        };
        let (a, b) = (texel(i.x, i.y), texel(i.x + 1.0, i.y));
        let (d, e) = (texel(i.x, i.y + 1.0), texel(i.x + 1.0, i.y + 1.0));
        [0, 1, 2, 3].map(|k| {
            let lo = a[k] + (b[k] - a[k]) * f.x;
            let hi = d[k] + (e[k] - d[k]) * f.x;
            lo + (hi - lo) * f.y
        })
    }
}

impl Terrain {
    /// The layers' weights at `local` (summing to 1), as the renderer weighs
    /// them before the height blend: control map times the slope and
    /// altitude rules; the first layer where every weight is 0.
    pub fn layer_weights(&self, data: &TerrainData, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        let c = data.control_at(local);
        let n = data.normal(local);
        let angle = n.y.clamp(-1.0, 1.0).acos().to_degrees();
        let altitude = data.height(local);
        let mut w = [0.0f32; MAX_TERRAIN_LAYERS];
        for (i, l) in self.layers.iter().take(MAX_TERRAIN_LAYERS).enumerate() {
            w[i] = c[i]
                * band(angle, l.slope[0], l.slope[1], l.slope_fade)
                * band(altitude, l.altitude[0], l.altitude[1], l.altitude_fade);
        }
        let sum: f32 = w.iter().sum();
        if sum < 1e-4 {
            return [1.0, 0.0, 0.0, 0.0];
        }
        w.map(|x| x / sum)
    }

    /// Places `s`'s instances for a camera at `camera`: each a cell key
    /// (stable while the cell is in range: for per-object motion) and its
    /// local-to-world transform, at most [`MAX_SCATTER_INSTANCES`], nearest
    /// rows first. `data` is this terrain's data.
    pub fn scatter(
        &self,
        data: &TerrainData,
        s: &TerrainScatter,
        camera: Vec3,
    ) -> Vec<(u32, Mat4)> {
        let mut out = Vec::new();
        if (s.layer as usize) >= self.layers.len().min(MAX_TERRAIN_LAYERS)
            || s.density.is_nan()
            || s.density <= 0.0
            || s.distance.is_nan()
            || s.distance <= 0.0
        {
            return out;
        }
        let cell = 1.0 / s.density.clamp(1e-4, 400.0).sqrt();
        let half = data.size * 0.5;
        let eye = Vec2::new(camera.x - self.origin.x, camera.z - self.origin.z);
        let cells = (data.size / cell).ceil();
        let lo = ((eye - s.distance + half) / cell).floor().max(Vec2::ZERO);
        let hi = ((eye + s.distance + half) / cell).ceil().min(cells);
        if lo.x >= hi.x || lo.y >= hi.y {
            return out;
        }
        let (x0, x1, z0, z1) = (lo.x as u32, hi.x as u32, lo.y as u32, hi.y as u32);
        let fade_from = s.distance * (1.0 - s.fade.clamp(0.0, 1.0));
        let seed = hash(s.seed ^ 0x9e37_79b9);
        // Rows outwards from the camera's, so the cap drops the farthest.
        let centre =
            (((eye.y + half.y) / cell).floor() as i64).clamp(z0 as i64, z1 as i64 - 1) as u32;
        let rows = (0..(z1 - z0)).map(|k| {
            let step = (k as i64 + 1) / 2;
            if k % 2 == 0 {
                centre as i64 + step
            } else {
                centre as i64 - step
            }
        });
        for z in rows
            .filter(|&z| z >= z0 as i64 && z < z1 as i64)
            .map(|z| z as u32)
        {
            for x in x0..x1 {
                let key = z.wrapping_mul(cells.x as u32).wrapping_add(x);
                let h0 = hash(key ^ seed);
                let (h1, h2, h3, h4) = (hash(h0), hash(h0 ^ 1), hash(h0 ^ 2), hash(h0 ^ 3));
                let p = Vec2::new(x as f32 + unit(h1), z as f32 + unit(h2)) * cell - half;
                if p.x > half.x || p.y > half.y {
                    continue;
                }
                let d = (p - eye).length();
                if d > s.distance {
                    continue;
                }
                let shrink = if s.distance > fade_from {
                    1.0 - smoothstep(fade_from, s.distance, d)
                } else {
                    1.0
                };
                if shrink <= 0.01 || unit(h3) >= self.layer_weights(data, p)[s.layer as usize] {
                    continue;
                }
                let t = unit(h4);
                let size = (s.scale[0] + (s.scale[1] - s.scale[0]) * t) * shrink;
                let up = Vec3::Y
                    .lerp(data.normal(p), s.align.clamp(0.0, 1.0))
                    .normalize_or(Vec3::Y);
                let turn = glam::Quat::from_rotation_y(unit(hash(h0 ^ 4)) * std::f32::consts::TAU);
                let rotation = glam::Quat::from_rotation_arc(Vec3::Y, up) * turn;
                let at = self.origin + Vec3::new(p.x, data.height(p), p.y);
                out.push((
                    key,
                    Mat4::from_scale_rotation_translation(Vec3::splat(size), rotation, at),
                ));
                if out.len() >= MAX_SCATTER_INSTANCES {
                    return out;
                }
            }
        }
        out
    }
}
