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
//!
//! **Layer arrays.** A terrain's layers may take their albedo and normal
//! maps from two texture arrays instead ([`Terrain::layer_arrays`], each
//! layer naming its layers with [`TerrainLayer::albedo_layer`] and
//! `normal_layer`): two sampled textures instead of up to eight, through
//! a variant of the terrain's group layout and pipelines made when a
//! terrain first uses it. A terrain of separate textures draws exactly as
//! before.

mod renderer;
pub(crate) use renderer::*;

use crate::deferred::{AsModule, Deferred};
use crate::scene::{TerrainHandle, TextureArrayHandle, TextureHandle};
use crate::terrain_tiles::{
    ControlPyramid, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES, TERRAIN_TILE, TerrainLayout,
    TerrainTile, TerrainView, terrain_tiles_wanted,
};
use glam::{Mat4, Vec2, Vec3};
use std::rc::Rc;

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
    /// With [`Terrain::layer_arrays`]: the layer of the albedo array this
    /// layer's colour (and height, in alpha) is (`None`: `color` alone).
    /// Past the array's last layer: the last.
    pub albedo_layer: Option<u32>,
    /// With [`Terrain::layer_arrays`]: the layer of the normal array this
    /// layer's normal map is (`None`: none).
    pub normal_layer: Option<u32>,
}

/// Two texture arrays a terrain's layers take their maps from
/// ([`Terrain::layer_arrays`]): the albedos (sRGB, the height in alpha)
/// and the normal maps (linear). With either set, each layer's
/// [`TerrainLayer::albedo_layer`] and `normal_layer` name its layers and
/// its `albedo` and `normal` textures are not read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerrainLayerArrays {
    pub albedo: Option<TextureArrayHandle>,
    pub normal: Option<TextureArrayHandle>,
}

impl TerrainLayerArrays {
    /// Whether the layers sample arrays.
    pub fn any(&self) -> bool {
        self.albedo.is_some() || self.normal.is_some()
    }
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
            albedo_layer: None,
            normal_layer: None,
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
    /// The texture arrays the layers' maps come from (none: each layer's
    /// own textures).
    pub layer_arrays: TerrainLayerArrays,
    /// How much the layers' heights (their albedo's alpha) sharpen their
    /// transitions (0: weights alone).
    pub height_blend: f32,
    pub cast_shadows: bool,
    pub lod: TerrainLod,
    /// Streamed tiles fade in over their parents as they arrive; `false`
    /// shows what has arrived at once (a capture after the tiles settled).
    pub fade_in: bool,
    /// Drawn (colour, prepass, shadows); `false` keeps it only as the
    /// ground its og.TerrainScatter copies stand on.
    pub visible: bool,
}

impl Terrain {
    pub fn new(terrain: TerrainHandle) -> Terrain {
        Terrain {
            terrain,
            transform: Mat4::IDENTITY,
            layers: vec![TerrainLayer::default()],
            layer_arrays: TerrainLayerArrays::default(),
            height_blend: 0.0,
            cast_shadows: true,
            lod: TerrainLod::default(),
            fade_in: true,
            visible: true,
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
    /// Its pipelines (the layer-array variant's when its layers sample
    /// arrays).
    pub passes: TerrainPasses,
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
        tiles: og_core::IdMap<u32, Arc<TerrainTile>>,
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

type GroupKey = (
    usize,
    u64,
    [Option<TextureHandle>; 2 * MAX_TERRAIN_LAYERS],
    TerrainLayerArrays,
);

/// Draws [`Terrain`]s: the pipelines, each data's tiles, a uniform and
/// level buffer per drawn terrain, kept across frames.
pub(crate) struct TerrainRenderer {
    layout: wgpu::BindGroupLayout,
    passes: TerrainPasses,
    /// The layer-array variant (a terrain whose layers sample texture
    /// arrays, [`Terrain::layer_arrays`]): its group layout, with the
    /// albedo (5) and normal (9) maps as 2D arrays and no other layer
    /// texture, and its pipelines; made when a terrain first needs it.
    arrays: Option<(wgpu::BindGroupLayout, TerrainPasses)>,
    /// The variant's stand-ins for a missing array: white and the flat normal.
    array_stand_ins: Option<(wgpu::TextureView, wgpu::TextureView)>,
    /// What the variant is made from.
    frame_layout: wgpu::BindGroupLayout,
    shadow_layout: wgpu::BindGroupLayout,
    lit_source: String,
    samples: u32,
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

/// The lit, prepass and shadow pipelines of one terrain group layout: the
/// standard one, or the layer-array variant's ([`TerrainRenderer`]).
#[derive(Clone)]
pub(crate) struct TerrainPasses {
    pub lit: Rc<crate::deferred::Sampled<wgpu::RenderPipeline>>,
    pub prepass: Rc<Deferred<wgpu::RenderPipeline>>,
    pub shadow: Rc<Deferred<wgpu::RenderPipeline>>,
}

/// Which pass [`draw_levels`] draws: each terrain sets its own pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerrainPass {
    Lit,
    Prepass,
    Shadow,
}

/// The pipelines in [`TerrainPasses`] over `layout`, from these modules.
fn terrain_passes(
    device: &wgpu::Device,
    frame_layout: &wgpu::BindGroupLayout,
    shadow_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    samples: u32,
    lit_source: String,
    shadow_source: String,
) -> TerrainPasses {
    // Made when a terrain is first drawn (deferred.rs).
    let lit_module = crate::deferred::module(device, "terrain", lit_source);
    let shadow_module = crate::deferred::module(device, "terrain shadow", shadow_source);
    let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
        let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        })
    };
    let lit_layout = pl("terrain", &[frame_layout, layout]);
    let shadow_layout = pl("terrain shadow", &[shadow_layout, layout]);
    let lit = crate::deferred_hdr(
        device,
        &lit_layout,
        lit_module.clone(),
        samples,
        move |layout, module| crate::HdrPipeline {
            alpha_to_coverage: false,
            label: "terrain",
            layout,
            module,
            vs: "vs",
            fs: "fs",
            buffers: &[],
            cull: Some(wgpu::Face::Back),
            depth_write: true,
            depth_compare: wgpu::CompareFunction::Less,
            blend: None,
            samples,
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
    let prepass = {
        let (device, lit_layout) = (device.clone(), lit_layout.clone());
        Deferred::new(move || {
            let lit_module = lit_module.module();
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("terrain prepass"),
                layout: Some(&lit_layout),
                vertex: wgpu::VertexState {
                    module: lit_module,
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
                    module: lit_module,
                    entry_point: Some("fs_prepass"),
                    compilation_options: Default::default(),
                    targets: &[
                        target(crate::post::GBUFFER_FORMAT),
                        target(crate::post::LINDEPTH_FORMAT),
                        target(crate::post::OUTLINE_FORMAT),
                    ],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(&device).as_ref(),
            })
        })
    };
    // An open surface: both faces cast (the bias in the lit pass keeps
    // its own lit side from shadowing itself).
    let shadow = {
        let (device, shadow_layout) = (device.clone(), shadow_layout.clone());
        Deferred::new(move || {
            let shadow_module = shadow_module.module();
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("terrain shadow"),
                layout: Some(&shadow_layout),
                vertex: wgpu::VertexState {
                    module: shadow_module,
                    entry_point: Some("vs_shadow"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: depth(true),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: shadow_module,
                    entry_point: Some("fs_shadow"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(&device).as_ref(),
            })
        })
    };
    TerrainPasses {
        lit: Rc::new(lit),
        prepass: Rc::new(prepass),
        shadow: Rc::new(shadow),
    }
}

const TERRAIN_WGSL: &str = wgsl3d!("terrain.wgsl");

/// The terrain's lit module after the frame group, the light tables and the lighting.
pub(crate) fn terrain_lit_source(frame: &str, tables: &str, lighting: &str) -> String {
    format!(
        "{frame}{tables}{lighting}{}{TERRAIN_WGSL}{}",
        crate::AREA_WGSL,
        wgsl3d!("terrain_lit.wgsl")
    )
}

/// The eight layer textures' declarations (terrain.wgsl), and the layer
/// arrays' that replace them in the layer-array variant.
const LAYER_TEXTURES: &str = "@group(1) @binding(5) var albedo0: texture_2d<f32>;
@group(1) @binding(6) var albedo1: texture_2d<f32>;
@group(1) @binding(7) var albedo2: texture_2d<f32>;
@group(1) @binding(8) var albedo3: texture_2d<f32>;
@group(1) @binding(9) var normal0: texture_2d<f32>;
@group(1) @binding(10) var normal1: texture_2d<f32>;
@group(1) @binding(11) var normal2: texture_2d<f32>;
@group(1) @binding(12) var normal3: texture_2d<f32>;
";
const LAYER_ARRAYS: &str = "@group(1) @binding(5) var albedo_array: texture_2d_array<f32>;
@group(1) @binding(9) var normal_array: texture_2d_array<f32>;
";

/// `source` (a terrain module) with the layer arrays declared in place of
/// the eight layer textures.
fn terrain_array_declarations(source: &str) -> String {
    assert!(
        source.contains(LAYER_TEXTURES),
        "the layer-array variant cannot find terrain.wgsl's layer textures"
    );
    source.replacen(LAYER_TEXTURES, LAYER_ARRAYS, 1)
}

/// The terrain lit module (`standard`, [`terrain_lit_source`]) as its
/// layer-array variant: the arrays declared, and `layer_albedo` and
/// `layer_normal_map` reading layer `i`'s array layer (slope.w and
/// altitude.w) instead of its texture. The rest is the same text, so the
/// shading is the standard path's.
pub(crate) fn terrain_lit_array_source(standard: &str) -> String {
    let s = terrain_array_declarations(standard);
    let start = s
        .find("fn layer_albedo(")
        .expect("terrain_lit.wgsl's layer_albedo");
    let end = s[start..]
        .find("fn band(")
        .map(|e| start + e)
        .expect("terrain_lit.wgsl's layer_normal_map ends before band");
    format!(
        "{}{}{}",
        &s[..start],
        "fn layer_albedo(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(albedo_array, layer_sampler, uv, i32(terrain.layers[i].slope.w), dx, dy);
}

fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec3<f32> {
  let t = textureSampleGrad(normal_array, layer_sampler, uv, i32(terrain.layers[i].altitude.w), dx, dy);
  return t.xyz * 2.0 - 1.0;
}

",
        &s[end..]
    )
}

/// The terrain's shadow module for the layer-array variant's layout (the
/// same entry points; the layer arrays declared).
pub(crate) fn terrain_shadow_array_source() -> String {
    terrain_array_declarations(&terrain_shadow_source())
}

/// The terrain's shadow module.
pub(crate) fn terrain_shadow_source() -> String {
    format!(
        "{TERRAIN_WGSL}{}{}",
        crate::SHADOW_PASS_WGSL,
        wgsl3d!("terrain_shadow.wgsl")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layer-array variant is the standard lit module with two arrays
    /// declared in place of the eight layer textures, and its two layer
    /// reads taking the layer from slope.w and altitude.w; nothing else.
    #[test]
    fn the_layer_array_variant_reads_two_arrays() {
        let standard = terrain_lit_source("", "", "");
        let array = terrain_lit_array_source(&standard);
        for gone in [
            "albedo0",
            "albedo3",
            "normal1",
            "normal3",
            "texture_2d<f32>",
        ] {
            assert!(standard.contains(gone));
            assert!(!array.contains(gone), "{gone}");
        }
        assert_eq!(array.matches("texture_2d_array<f32>;").count(), 5);
        assert!(array.contains("i32(terrain.layers[i].slope.w)"));
        assert!(array.contains("i32(terrain.layers[i].altitude.w)"));
        let tail = |s: &str| s[s.find("fn band(").unwrap()..].to_string();
        assert_eq!(tail(&array), tail(&standard));
        assert!(!terrain_shadow_array_source().contains("albedo0"));
    }

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
