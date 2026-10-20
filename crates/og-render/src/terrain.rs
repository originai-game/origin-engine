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
use crate::terrain_tiles::{
    ControlPyramid, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES, TERRAIN_TILE, TerrainLayout,
    TerrainTile, TerrainView, terrain_tiles_wanted,
};
use glam::{Mat4, Vec2, Vec3};

pub use crate::scatter::{MAX_SCATTER_INSTANCES, TerrainScatter};
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

    /// The height at `local` (x and z relative to the terrain's centre) on
    /// the triangles the renderer draws at its finest level and physics
    /// collides with: each cell split along its (-x, -z) to (+x, +z)
    /// diagonal; clamped to the edge.
    pub fn height(&self, local: Vec2) -> f32 {
        TerrainSampler::of(self).height(local)
    }

    /// The surface normal at `local`: the samples' normals (central
    /// differences, as the renderer's normal map) interpolated bilinearly.
    pub fn normal(&self, local: Vec2) -> Vec3 {
        TerrainSampler::of(self).normal(local)
    }

    /// The lowest and highest height.
    pub fn range(&self) -> (f32, f32) {
        self.heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)))
    }
}

/// [`TerrainData::height`] and [`TerrainData::normal`] over a field read
/// sample by sample (`sample(column, row)`, both in range): the runtime
/// samples a streamed terrain's decoded tiles with it, and gets the same
/// answers as from the whole field.
pub struct TerrainSampler<F: Fn(u32, u32) -> f32> {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub sample: F,
}

impl TerrainSampler<fn(u32, u32) -> f32> {
    /// A sampler over a field in memory.
    pub fn of(d: &TerrainData) -> TerrainSampler<impl Fn(u32, u32) -> f32 + '_> {
        TerrainSampler {
            size: d.size,
            columns: d.columns,
            rows: d.rows,
            sample: move |c: u32, r: u32| d.heights[(r * d.columns + c) as usize],
        }
    }
}

impl<F: Fn(u32, u32) -> f32> TerrainSampler<F> {
    fn at(&self, c: u32, r: u32) -> f32 {
        (self.sample)(c.min(self.columns - 1), r.min(self.rows - 1))
    }

    fn cell(&self) -> Vec2 {
        self.size
            / Vec2::new(
                self.columns.max(2) as f32 - 1.0,
                self.rows.max(2) as f32 - 1.0,
            )
    }

    /// Sample coordinates of a point given relative to the terrain's centre.
    fn grid(&self, local: Vec2) -> Vec2 {
        let t = (local + self.size * 0.5) / self.cell();
        t.clamp(
            Vec2::ZERO,
            Vec2::new(self.columns as f32 - 1.0, self.rows as f32 - 1.0),
        )
    }

    /// As [`TerrainData::height`].
    pub fn height(&self, local: Vec2) -> f32 {
        let t = self.grid(local);
        let i = t
            .floor()
            .min(Vec2::new(self.columns as f32 - 2.0, self.rows as f32 - 2.0));
        let f = (t - i).clamp(Vec2::ZERO, Vec2::ONE);
        let (c, r) = (i.x as u32, i.y as u32);
        let h00 = self.at(c, r);
        let h10 = self.at(c + 1, r);
        let h01 = self.at(c, r + 1);
        let h11 = self.at(c + 1, r + 1);
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
        let hx = (self.at(c1, r) - self.at(c0, r)) / ((c1 - c0).max(1) as f32 * cell.x);
        let hz = (self.at(c, r1) - self.at(c, r0)) / ((r1 - r0).max(1) as f32 * cell.y);
        Vec3::new(-hx, 1.0, -hz).normalize()
    }

    /// As [`TerrainData::normal`].
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
    /// GPU bytes for its resident tiles (0: [`DEFAULT_TERRAIN_BUDGET`]):
    /// how many tiles it keeps, at least 8.
    pub budget: u64,
}

/// A terrain's tile budget when its lod gives none: 16 MiB.
pub const DEFAULT_TERRAIN_BUDGET: u64 = 16 << 20;

impl TerrainLod {
    /// Quads per side of every level, as drawn.
    pub fn ring_quads(&self) -> u32 {
        let r = if self.ring == 0 { 64 } else { self.ring };
        (r.clamp(16, 256) / 4) * 4
    }

    /// GPU bytes for resident tiles, as drawn.
    pub fn budget(&self) -> u64 {
        if self.budget == 0 {
            DEFAULT_TERRAIN_BUDGET
        } else {
            self.budget
        }
    }
}

/// A terrain drawn this frame: its data placed by `transform` (from the
/// terrain's own space, whose origin is the centre of the height field at
/// height 0 with x and z along its sides, to the world) with its layers.
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub terrain: TerrainHandle,
    /// Rotation, scale and translation of the height field (identity: centred
    /// on the world origin along world x and z).
    pub transform: Mat4,
    /// Up to [`MAX_TERRAIN_LAYERS`] layers (more are ignored).
    pub layers: Vec<TerrainLayer>,
    /// How much the layers' heights (their albedo's alpha) sharpen their
    /// transitions (0: weights alone).
    pub height_blend: f32,
    pub cast_shadows: bool,
    pub lod: TerrainLod,
    /// Streamed tiles fade in over their parents as they arrive; `false`
    /// shows what has arrived at once (a capture after the tiles settled).
    pub fade_in: bool,
}

impl Terrain {
    pub fn new(terrain: TerrainHandle) -> Terrain {
        Terrain {
            terrain,
            transform: Mat4::IDENTITY,
            layers: vec![TerrainLayer::default()],
            height_blend: 0.0,
            cast_shadows: true,
            lod: TerrainLod::default(),
            fade_in: true,
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
    /// The terrain's space to the world.
    model: [[f32; 4]; 4],
    /// Normals from the terrain's space to the world (the inverse transpose
    /// of `model`'s upper 3×3, as three padded columns).
    normal: [[f32; 4]; 3],
    origin: [f32; 4],
    extent: [f32; 4],
    params: [f32; 4],
    tiles: [f32; 4],
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    pyramid: [[f32; 4]; MAX_TERRAIN_PYRAMID as usize],
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

/// Where a terrain's tiles come from: a whole field in memory (its tiles
/// made when drawn), or tiles handed over one by one (streaming).
#[derive(Clone, Debug)]
pub(crate) enum TerrainSource {
    Data(TerrainData),
    Stream {
        layout: TerrainLayout,
        range: (f32, f32),
        tiles: std::collections::BTreeMap<u32, Arc<TerrainTile>>,
    },
}

impl TerrainSource {
    /// The tile size a field in memory is drawn with: [`TERRAIN_TILE`], or
    /// larger for a field too big for the tile table.
    pub(crate) fn layout(&self) -> Result<TerrainLayout, String> {
        match self {
            TerrainSource::Stream { layout, .. } => Ok(*layout),
            TerrainSource::Data(d) => {
                d.check()?;
                let mut tile = TERRAIN_TILE;
                loop {
                    match d.layout(tile) {
                        Ok(l) => return Ok(l),
                        Err(e) if tile >= 1024 => return Err(e),
                        Err(_) => tile *= 2,
                    }
                }
            }
        }
    }

    pub(crate) fn range(&self) -> (f32, f32) {
        match self {
            TerrainSource::Stream { range, .. } => *range,
            TerrainSource::Data(d) => d.range(),
        }
    }
}

/// Frames a streamed tile takes to fade in over its parent.
const FADE_FRAMES: f32 = 12.0;

/// One resident tile: its index and data, and how far it has faded in.
struct SlotTile {
    tile: u32,
    data: Arc<TerrainTile>,
    fade: f32,
}

/// One terrain data's GPU state: the tile arrays (one layer per slot), the
/// tile table, and what each slot holds.
struct GpuData {
    revision: u64,
    layout: TerrainLayout,
    capacity: u32,
    /// Bumped whenever the arrays are made again (bind groups name it).
    generation: u64,
    heights: wgpu::Texture,
    heights_view: wgpu::TextureView,
    normals: wgpu::Texture,
    normals_view: wgpu::TextureView,
    control: Option<(wgpu::Texture, wgpu::TextureView)>,
    table: wgpu::Buffer,
    table_cpu: Vec<f32>,
    slots: Vec<Option<SlotTile>>,
    by_tile: std::collections::HashMap<u32, usize>,
    /// A field in memory's control map resampled for its tiles.
    pyramid: Option<ControlPyramid>,
    range: (f32, f32),
    streamed: bool,
    /// Drawn in an earlier frame: tiles arriving now fade in.
    drawn: bool,
}

struct Slot {
    uniform: wgpu::Buffer,
    levels: wgpu::Buffer,
    /// The bind group and what it was made from: (data index, data
    /// generation, the layers' textures).
    group: Option<(GroupKey, wgpu::BindGroup)>,
}

type GroupKey = (usize, u64, [Option<TextureHandle>; 2 * MAX_TERRAIN_LAYERS]);

/// Draws [`Terrain`]s: the pipelines, each data's tiles, a uniform and
/// level buffer per drawn terrain, kept across frames.
pub(crate) struct TerrainRenderer {
    layout: wgpu::BindGroupLayout,
    pub lit: wgpu::RenderPipeline,
    pub prepass: wgpu::RenderPipeline,
    pub shadow: wgpu::RenderPipeline,
    layer_sampler: wgpu::Sampler,
    map_sampler: wgpu::Sampler,
    white: wgpu::TextureView,
    flat_normal: wgpu::TextureView,
    /// A white array for terrains without a control map.
    white_array: wgpu::TextureView,
    indices: Option<(u32, wgpu::Buffer, [std::ops::Range<u32>; 5])>,
    scene: u64,
    datas: Vec<Option<GpuData>>,
    slots: Vec<Slot>,
    generations: u64,
    max_layers: u32,
    /// Tiles uploaded in the last prepare and resident after it.
    pub uploads: u32,
    pub resident: u32,
}

pub(crate) const NORMALS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

/// The tile table's size: one f32 per tile, four to a vector.
const TABLE_BYTES: u64 = MAX_TERRAIN_TILES as u64 * 4;

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
    /// Drops every terrain's bind group (a layer texture was replaced):
    /// the next prepare makes them again from the current views.
    pub(crate) fn forget_groups(&mut self) {
        for slot in &mut self.slots {
            slot.group = None;
        }
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: TerrainPipelines,
    ) -> TerrainRenderer {
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let tex = |binding, filterable, dim| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: vf,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: dim,
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
        let array = wgpu::TextureViewDimension::D2Array;
        let d2 = wgpu::TextureViewDimension::D2;
        let mut entries = vec![
            uniform(0, false),
            uniform(1, true),
            tex(2, false, array),
            tex(3, true, array),
            tex(4, true, array),
        ];
        entries.extend((5..13).map(|b| tex(b, true, d2)));
        entries.extend([sampler(13), sampler(14), uniform(15, false)]);
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
                format!(
                    "{common}{}{}",
                    crate::SHADOW_PASS_WGSL,
                    include_str!("../shaders/terrain_shadow.wgsl")
                )
                .into(),
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
        // Viewed as a 2D array, so it needs GL-safe layers, each white.
        let white_layers = crate::gl_safe_array_layers(1);
        let white_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("terrain no control"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: white_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            white_tex.as_image_copy(),
            &vec![255; 4 * white_layers as usize],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: white_layers,
            },
        );
        let white_array = white_tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        TerrainRenderer {
            layout,
            lit,
            prepass,
            shadow,
            layer_sampler,
            map_sampler,
            white: p.white,
            flat_normal: p.flat_normal,
            white_array,
            indices: None,
            scene: 0,
            datas: Vec::new(),
            slots: Vec::new(),
            generations: 0,
            max_layers: device.limits().max_texture_array_layers.min(256),
            uploads: 0,
            resident: 0,
        }
    }

    /// The index buffer (bound by the passes that draw terrain).
    pub(crate) fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.indices.as_ref().map(|(_, b, _)| b)
    }

    /// The tile arrays for `layout` with `capacity` slots.
    fn make_data(
        &mut self,
        device: &wgpu::Device,
        layout: TerrainLayout,
        capacity: u32,
        revision: u64,
        range: (f32, f32),
        streamed: bool,
    ) -> GpuData {
        // `capacity` slots; the spare layers keep a one-tile (or six-tile)
        // terrain a 2D array on WebGL2 and are never used.
        let layers = crate::gl_safe_array_layers(capacity);
        let array = |label, side: u32, mips: u32, format| {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: layers,
                },
                mip_level_count: mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let v = t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
            (t, v)
        };
        let (heights, heights_view) = array(
            "terrain heights",
            layout.heights_side(),
            1,
            wgpu::TextureFormat::R32Float,
        );
        let ns = layout.normals_side();
        let (normals, normals_view) = array(
            "terrain normals",
            ns,
            crate::mips::mip_count(ns, ns),
            NORMALS_FORMAT,
        );
        let control = (layout.control > 0).then(|| {
            let cs = layout.control_side();
            array(
                "terrain control",
                cs,
                crate::mips::mip_count(cs, cs),
                wgpu::TextureFormat::Rgba8Unorm,
            )
        });
        let table = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terrain tiles"),
            size: TABLE_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.generations += 1;
        GpuData {
            revision,
            layout,
            capacity,
            generation: self.generations,
            heights,
            heights_view,
            normals,
            normals_view,
            control,
            table,
            table_cpu: vec![-1.0; MAX_TERRAIN_TILES as usize],
            slots: (0..capacity).map(|_| None).collect(),
            by_tile: Default::default(),
            pyramid: None,
            range,
            streamed,
            drawn: false,
        }
    }

    /// Writes one tile into slot `slot`.
    fn upload_tile(queue: &wgpu::Queue, gpu: &GpuData, slot: u32, tile: &TerrainTile) {
        fn layer(texture: &wgpu::Texture, mip: u32, slot: u32) -> wgpu::TexelCopyTextureInfo<'_> {
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: mip,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: slot,
                },
                aspect: wgpu::TextureAspect::All,
            }
        }
        let size = |side: u32| wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        };
        let row = |side: u32, bytes: u32| wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(side * bytes),
            rows_per_image: None,
        };
        let hs = gpu.layout.heights_side();
        queue.write_texture(
            layer(&gpu.heights, 0, slot),
            bytemuck::cast_slice(&tile.heights),
            row(hs, 4),
            size(hs),
        );
        let mut at = 0usize;
        for (mip, side) in crate::terrain_tiles::mip_sides(gpu.layout.normals_side())
            .into_iter()
            .enumerate()
        {
            let n = (side * side * 2) as usize;
            if let Some(texels) = tile.normals.get(at..at + n) {
                queue.write_texture(
                    layer(&gpu.normals, mip as u32, slot),
                    bytemuck::cast_slice(texels),
                    row(side, 4),
                    size(side),
                );
            }
            at += n;
        }
        if let (Some((texture, _)), Some(control)) = (&gpu.control, &tile.control) {
            let mut at = 0usize;
            for (mip, side) in crate::terrain_tiles::mip_sides(gpu.layout.control_side())
                .into_iter()
                .enumerate()
            {
                let n = (side * side * 4) as usize;
                if let Some(texels) = control.get(at..at + n) {
                    queue.write_texture(
                        layer(texture, mip as u32, slot),
                        texels,
                        row(side, 4),
                        size(side),
                    );
                }
                at += n;
            }
        }
    }

    /// Brings every drawn data's slots to this frame's tiles: the tiles
    /// each terrain wants around the camera (made from a field in memory,
    /// or taken from those handed over), uploaded into free slots, the
    /// others evicted; streamed tiles fade in over their parents.
    fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        view: TerrainView,
    ) -> Result<(), crate::RenderError> {
        if self.scene != scene.id() {
            self.scene = scene.id();
            self.datas.clear();
            for s in &mut self.slots {
                s.group = None;
            }
        }
        self.uploads = 0;
        self.resident = 0;
        let (sources, revisions) = scene.terrain_sources();
        self.datas.resize_with(sources.len(), || None);
        // Each data's wanted tiles, over every terrain drawing it, best first.
        let mut wanted: std::collections::BTreeMap<usize, (Vec<u32>, u32)> = Default::default();
        for (ti, t) in scene.terrains.iter().enumerate() {
            let i = t.terrain.index();
            let Some(source) = sources.get(i) else {
                continue;
            };
            let layout = source
                .layout()
                .map_err(|e| crate::RenderError::InvalidData(format!("terrain {i}: {e}")))?;
            // The atlases allocate at most one spare layer beyond the slots
            // (`gl_safe_array_layers`), which must stay within `max_layers`.
            // (`max`/`min`, not `clamp`: a device with eight layers or
            // fewer must not panic.)
            let capacity = (t.lod.budget() / layout.slot_bytes().max(1))
                .max(8)
                .min((self.max_layers as u64).saturating_sub(1).max(1))
                .min(layout.tile_count() as u64) as u32;
            let view = TerrainView {
                detail: scene.scatter_detail(ti),
                ..view
            };
            let list = terrain_tiles_wanted(&layout, t, source.range(), &view, capacity as usize);
            let entry = wanted.entry(i).or_insert((Vec::new(), capacity));
            entry.1 = entry.1.max(capacity);
            for tile in list {
                if !entry.0.contains(&tile) {
                    entry.0.push(tile);
                }
            }
        }
        for (i, (list, capacity)) in wanted {
            let source = &sources[i];
            let layout = source.layout().map_err(crate::RenderError::InvalidData)?;
            let streamed = matches!(source, TerrainSource::Stream { .. });
            let keep = self.datas[i]
                .take()
                .filter(|g| g.layout == layout && g.capacity == capacity);
            let mut gpu = match keep {
                Some(mut g) if g.revision == revisions[i] => {
                    g.streamed = streamed;
                    g
                }
                Some(mut g) => {
                    // Edited or replaced: every slot is stale.
                    g.revision = revisions[i];
                    g.slots.iter_mut().for_each(|s| *s = None);
                    g.by_tile.clear();
                    g.pyramid = None;
                    g.range = source.range();
                    g.streamed = streamed;
                    g
                }
                None => self.make_data(
                    device,
                    layout,
                    capacity,
                    revisions[i],
                    source.range(),
                    streamed,
                ),
            };
            gpu.range = source.range();
            // The tiles to hold, best first: every one after its parent.
            let mut target: Vec<(u32, Arc<TerrainTile>)> = Vec::new();
            let mut chosen = std::collections::HashSet::new();
            for &tile in list.iter().take(capacity as usize) {
                if let Some(p) = layout.parent(tile)
                    && !chosen.contains(&p)
                {
                    continue;
                }
                let data = match source {
                    TerrainSource::Stream { tiles, .. } => match tiles.get(&tile) {
                        Some(t) => t.clone(),
                        None => continue,
                    },
                    TerrainSource::Data(d) => match gpu.by_tile.get(&tile) {
                        Some(&s) => gpu.slots[s].as_ref().expect("held").data.clone(),
                        None => {
                            if gpu.pyramid.is_none()
                                && let Some(c) = &d.control
                            {
                                gpu.pyramid = ControlPyramid::new(&layout, c);
                            }
                            Arc::new(d.tile(&layout, tile, gpu.pyramid.as_ref()))
                        }
                    },
                };
                chosen.insert(tile);
                target.push((tile, data));
            }
            // Evict what is not held this frame, or was replaced.
            for s in 0..gpu.slots.len() {
                let stale = match &gpu.slots[s] {
                    Some(st) => !target
                        .iter()
                        .any(|(t, d)| *t == st.tile && Arc::ptr_eq(d, &st.data)),
                    None => false,
                };
                if stale {
                    let st = gpu.slots[s].take().expect("held");
                    gpu.by_tile.remove(&st.tile);
                }
            }
            let fade_in = scene
                .terrains
                .iter()
                .filter(|t| t.terrain.index() == i)
                .all(|t| t.fade_in);
            let fresh_fade = if streamed && gpu.drawn && fade_in {
                0.0
            } else {
                1.0
            };
            if !fade_in {
                for st in gpu.slots.iter_mut().flatten() {
                    st.fade = 1.0;
                }
            }
            for (tile, data) in &target {
                if gpu.by_tile.contains_key(tile) {
                    continue;
                }
                let Some(slot) = gpu.slots.iter().position(Option::is_none) else {
                    break;
                };
                Self::upload_tile(queue, &gpu, slot as u32, data);
                gpu.slots[slot] = Some(SlotTile {
                    tile: *tile,
                    data: data.clone(),
                    fade: fresh_fade,
                });
                gpu.by_tile.insert(*tile, slot);
                self.uploads += 1;
            }
            // Fades advance once their parent has fully faded in.
            let parents_in: Vec<bool> = gpu
                .slots
                .iter()
                .map(|s| match s {
                    Some(st) => layout.parent(st.tile).is_none_or(|p| {
                        gpu.by_tile
                            .get(&p)
                            .and_then(|&ps| gpu.slots[ps].as_ref())
                            .is_some_and(|ps| ps.fade >= 1.0)
                    }),
                    None => false,
                })
                .collect();
            for (s, parent_in) in gpu.slots.iter_mut().zip(parents_in) {
                if let Some(st) = s
                    && st.fade < 1.0
                    && parent_in
                {
                    st.fade = (st.fade + 1.0 / FADE_FRAMES).min(1.0);
                }
            }
            let mut table = vec![0.0f32; MAX_TERRAIN_TILES as usize];
            for (s, st) in gpu.slots.iter().enumerate() {
                if let Some(st) = st {
                    table[st.tile as usize] = (s as f32 + 1.0) * 256.0 + (st.fade * 255.0).round();
                }
            }
            if table != gpu.table_cpu {
                queue.write_buffer(&gpu.table, 0, bytemuck::cast_slice(&table));
                gpu.table_cpu = table;
            }
            gpu.drawn = true;
            self.resident += gpu.by_tile.len() as u32;
            self.datas[i] = Some(gpu);
        }
        Ok(())
    }

    /// Uploads what changed and lays out this frame's levels for every
    /// terrain around `camera`. `textures` resolves a layer's texture
    /// handle (`None`: not a texture of this scene).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        camera: Vec3,
        far: f32,
        target_height: u32,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        textures: impl Fn(TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Result<Vec<TerrainDraw>, crate::RenderError> {
        if scene.terrains.is_empty() {
            self.uploads = 0;
            self.resident = 0;
            return Ok(Vec::new());
        }
        let view = TerrainView {
            camera,
            far,
            ..TerrainView::of(&scene.camera, target_height)
        };
        self.sync(device, queue, scene, view)?;
        let ring = scene
            .terrains
            .iter()
            .map(|t| t.lod.ring_quads())
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
            let top = data.layout.levels - 1;
            let (ttx, ttz) = data.layout.tiles(top);
            let top_in = (0..ttz).all(|z| {
                (0..ttx).all(|x| data.by_tile.contains_key(&data.layout.index(top, x, z)))
            });
            if !top_in {
                // A streamed terrain whose top level has not arrived: nothing to draw yet.
                continue;
            }
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
            let lay = data.layout;
            let model = t.transform;
            let n3 = glam::Mat3::from_mat4(model).inverse().transpose();
            let mut pyramid = [[0.0f32; 4]; MAX_TERRAIN_PYRAMID as usize];
            for k in 0..lay.levels {
                let (x, z) = lay.tiles(k);
                pyramid[k as usize] = [lay.first(k) as f32, x as f32, z as f32, 0.0];
            }
            let gpu = TerrainGpu {
                model: model.to_cols_array_2d(),
                normal: [
                    n3.x_axis.extend(0.0).to_array(),
                    n3.y_axis.extend(0.0).to_array(),
                    n3.z_axis.extend(0.0).to_array(),
                ],
                origin: model.w_axis.truncate().extend(count as f32).to_array(),
                extent: [lay.size.x, lay.size.y, lay.columns as f32, lay.rows as f32],
                params: [
                    t.height_blend.max(0.0),
                    f32::from(lay.control > 0),
                    0.0,
                    0.0,
                ],
                tiles: [lay.tile as f32, lay.control as f32, lay.levels as f32, 0.0],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                pyramid,
                layers,
            };
            let slot = &mut self.slots[ti];
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));

            // The levels are laid out in the terrain's own space: the camera
            // brought into it, the far plane by the terrain's horizontal scale.
            let cell = lay.cell();
            let unit = if t.lod.spacing > 0.0 {
                t.lod.spacing
            } else {
                cell.x.min(cell.y)
            };
            let local_camera = model.inverse().transform_point3(camera);
            let lo = -lay.size * 0.5;
            let cam = (Vec2::new(local_camera.x, local_camera.z) - lo) / unit;
            let extent = lay.size / unit;
            let n = t.lod.ring_quads();
            let along = horizontal_scale(model);
            let list = levels(cam, extent, n, (far / along / unit) as f64);
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
                    ((l.center[0] + half) * unit as f64).min(lay.size.x as f64) as f32,
                    ((l.center[1] + half) * unit as f64).min(lay.size.y as f64) as f32,
                );
                let box_min = Vec3::new(lo.x + a.x, hmin, lo.y + a.y);
                let box_max = Vec3::new(lo.x + b.x, hmax, lo.y + b.y);
                let (center, radius) = world_sphere(model, box_min, box_max);
                out.push(LevelDraw {
                    offset: (k as u64 * LEVEL_STRIDE) as u32,
                    indices,
                    center,
                    radius,
                });
            }
            queue.write_buffer(&slot.levels, 0, &level_bytes);
            let key: GroupKey = (di, data.generation, handles);
            if slot.group.as_ref().is_none_or(|(k, _)| *k != key) {
                let white = &self.white;
                let flat = &self.flat_normal;
                let view = |h: Option<TextureHandle>, fallback| match h {
                    Some(h) => textures(h).unwrap_or(fallback),
                    None => fallback,
                };
                let control = data.control.as_ref().map_or(&self.white_array, |c| &c.1);
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
                resources.push(data.table.as_entire_binding());
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
                let s = d.layout.size;
                let lo = Vec3::new(-s.x * 0.5, d.range.0, -s.y * 0.5);
                let hi = Vec3::new(s.x * 0.5, d.range.1, s.y * 0.5);
                Some(world_sphere(t.transform, lo, hi))
            })
            .collect()
    }
}

/// How far one metre along the terrain's x and z reaches in the world (the
/// larger of the two): what the far plane is divided by in its own space.
pub(crate) fn horizontal_scale(model: Mat4) -> f32 {
    model
        .x_axis
        .truncate()
        .length()
        .max(model.z_axis.truncate().length())
        .max(1e-6)
}

/// The world bounding sphere (centre, radius) of a box in the terrain's
/// space.
pub(crate) fn world_sphere(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, f32) {
    let center = model.transform_point3((lo + hi) * 0.5);
    let mut radius: f32 = 0.0;
    for i in 0..8 {
        let corner = Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        );
        radius = radius.max(model.transform_point3(corner).distance(center));
    }
    (center, radius)
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
