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
//! bilinearly from an f32 storage buffer in the vertex shader; the normal
//! map (built from the heights on the GPU, mipmapped) and the control map
//! are filtered per pixel.
//!
//! **Layers.** Up to sixteen (past four, their maps are layers of the
//! layer arrays and the control map has a page of four channels per four
//! layers, the pages layers of the control array after their tile's
//! slot: no binding of their own). Each layer's weight is the control
//! map's channel (1 without a map, or past its pages) times its slope rule (degrees from flat) times its altitude rule
//! (metres above the origin), each a band with soft edges; with
//! `height_blend` > 0 the albedo's alpha (the layer's height map) sharpens
//! the transitions. Each pixel blends its four strongest layers (all of
//! them with four or fewer). Weights are normalised; where every weight
//! is 0 the first layer shows. A layer is sampled over world xz every `tile` metres,
//! triplanar, or past its `project_slope` by the two projections facing
//! the surface (biplanar: a steep face does not streak), with an optional
//! normal map; `anti_tiling` hex-tiles each projection (Mikkelsen 2022,
//! the lattice placed in f64 from the world's zero so it does not move
//! with the render origin), `macro_variation` drifts its brightness and
//! hue and blends in a coarser repeat far off, and `height_contrast`
//! narrows its height-blend window. The prepass reads only the layers'
//! weights and heights for its roughness, never their normal maps.
//!
//! **Holes.** [`TerrainData::holes`] cuts cells out (a cave mouth, a
//! pit): every tile height's lowest bit carries the hole flag of the cell
//! at its sample on the tile's level (`terrain_tiles::mark_holes`, a
//! coarser cell a hole where most of its field cells are), and the lit,
//! prepass and shadow passes discard a fragment over a hole of the finest
//! resident tile, so light falls through it too. No binding: the flag
//! rides in the height texture the passes already read. Physics removes
//! the same cells from the height field (og-physics `HeightfieldDesc`).
//!
//! **Layer arrays.** A terrain's layers may take their albedo and normal
//! maps from two texture arrays instead ([`Terrain::layer_arrays`], each
//! layer naming its layers with [`TerrainLayer::albedo_layer`] and
//! `normal_layer`): two sampled textures instead of up to eight, through
//! a variant of the terrain's group layout and pipelines made when a
//! terrain first uses it. A terrain of separate textures draws exactly as
//! before.

mod decals;
mod renderer;
pub(crate) use renderer::*;

use crate::deferred::{AsModule, Deferred};
use crate::pipeline_features::{Features, Variants};
use crate::scene::{TerrainHandle, TextureArrayHandle, TextureHandle};
use crate::terrain_tiles::{
    ControlPyramid, MAX_TERRAIN_PYRAMID, MAX_TERRAIN_TILES, TERRAIN_TILE, TerrainLayout,
    TerrainTile, TerrainView, terrain_tiles_wanted,
};
use glam::{Mat4, UVec2, Vec2, Vec3};
use std::rc::Rc;

pub use crate::scatter::{MAX_SCATTER_INSTANCES, TerrainScatter};
use std::sync::Arc;

/// The most layers a [`Terrain`] has: up to four with textures of their
/// own, up to sixteen whose maps are layers of [`Terrain::layer_arrays`].
pub const MAX_TERRAIN_LAYERS: usize = 16;
/// The most layers with textures of their own (four albedo and four
/// normal bindings), and the most a pixel blends: each pixel blends its
/// four strongest layers.
pub const TERRAIN_TEXTURED_LAYERS: usize = 4;
/// The most clipmap levels a terrain draws.
pub const MAX_TERRAIN_LEVELS: usize = 16;

/// A terrain's layer weights: RGBA8, one channel per layer (linear), rows
/// from -z to +z, columns from -x to +x, stretched over the terrain. Past
/// four layers it holds more pages of four channels, one after the other
/// (`rgba` is `width × height × 4` bytes per page: page `p` the layers
/// `4p` to `4p + 3`); a layer past its pages reads 1.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainControl {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

impl TerrainControl {
    /// Its pages of four channels (1 to 4).
    pub fn pages(&self) -> u32 {
        let page = (self.width as usize * self.height as usize * 4).max(1);
        (self.rgba.len() / page).max(1) as u32
    }

    /// Page `p` alone: a four-channel map.
    pub fn page(&self, p: u32) -> TerrainControl {
        let n = self.width as usize * self.height as usize * 4;
        let at = (p as usize * n).min(self.rgba.len());
        TerrainControl {
            width: self.width,
            height: self.height,
            rgba: Arc::from(&self.rgba[at..(at + n).min(self.rgba.len())]),
        }
    }
}

/// A terrain's geometry: `columns` × `rows` heights in metres over a
/// `size` rectangle centred on the terrain's origin, row-major from -z to
/// +z and -x to +x (sample `(c, r)` at `-size / 2 + (c, r) · cell`), an
/// optional control map and optional holes.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainData {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub heights: Arc<[f32]>,
    pub control: Option<TerrainControl>,
    /// The cells cut out of the surface: `(columns - 1)` × `(rows - 1)`
    /// bytes, row-major like the heights, cell `(c, r)` the quad whose
    /// -x, -z corner is sample `(c, r)`; non-zero is a hole. Neither drawn
    /// nor collided with (`None`: no holes).
    pub holes: Option<Arc<[u8]>>,
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
            holes: None,
        }
    }

    /// Cells along x and z (the hole map's size).
    pub fn cells(&self) -> (u32, u32) {
        (self.columns.max(1) - 1, self.rows.max(1) - 1)
    }

    /// Whether the cell under `local` (x and z relative to the terrain's
    /// centre) is a hole; off the terrain, never.
    pub fn hole(&self, local: Vec2) -> bool {
        let Some(holes) = &self.holes else {
            return false;
        };
        TerrainSampler::of(self)
            .cell_of(local)
            .is_some_and(|(c, r)| holes[(r * self.cells().0 + c) as usize] != 0)
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
        if let Some(h) = &self.holes {
            let (cx, cz) = self.cells();
            if h.len() != (cx * cz) as usize {
                return Err(format!(
                    "a {} × {} terrain has {} cells, its hole map {}",
                    self.columns,
                    self.rows,
                    cx * cz,
                    h.len()
                ));
            }
        }
        if let Some(c) = &self.control {
            let page = (c.width * c.height * 4) as usize;
            let pages = MAX_TERRAIN_LAYERS / 4;
            if c.width == 0
                || c.height == 0
                || c.rgba.len() % page != 0
                || !(1..=pages).contains(&(c.rgba.len() / page))
            {
                return Err(format!(
                    "a {} × {} control map needs {page} bytes per page of four layers, 1 to {pages} pages, got {}",
                    c.width,
                    c.height,
                    c.rgba.len()
                ));
            }
        }
        Ok(())
    }

    /// The height at `local` (x and z relative to the terrain's centre) on
    /// the triangles the renderer draws at its finest level and physics
    /// collides with: each cell uses the diagonal with the smaller corner
    /// height difference, ties keeping (-x, -z) to (+x, +z); clamped to the edge.
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

    /// The cell (column, row) under `local`: the quad whose -x, -z corner is
    /// that sample; `None` off the terrain.
    pub fn cell_of(&self, local: Vec2) -> Option<(u32, u32)> {
        let t = (local + self.size * 0.5) / self.cell();
        let n = Vec2::new(self.columns as f32 - 1.0, self.rows as f32 - 1.0);
        if !(t.x >= 0.0 && t.y >= 0.0 && t.x <= n.x && t.y <= n.y) {
            return None;
        }
        let c = (t.x as u32).min(self.columns - 2);
        let r = (t.y as u32).min(self.rows - 2);
        Some((c, r))
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
        if !og_core::math::heightfield_zigzag(h00, h10, h01, h11) {
            if f.x + f.y <= 1.0 {
                h00 + (h10 - h00) * f.x + (h01 - h00) * f.y
            } else {
                h11 + (h01 - h11) * (1.0 - f.x) + (h10 - h11) * (1.0 - f.y)
            }
        } else if f.y >= f.x {
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
    /// Sample by three projections blended by the normal (cliffs), unless
    /// `project_slope` decides.
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
    /// Breaks the texture's repeat (0 off, 1 full): each sample blends
    /// three hexagonal tiles of the texture, each turned and moved by its
    /// own hash (Mikkelsen's hex-tiling, `terrain_lit.wgsl`), so no period
    /// shows across a field.
    pub anti_tiling: f32,
    /// How sharp the height blend is at this layer's edges, 0 to 1 (0: the
    /// terrain's window of 0.15, 1: a tenth of it).
    pub height_contrast: f32,
    /// The slope in degrees past which the layer is sampled by the two
    /// projections facing the surface most (biplanar) instead of from
    /// above, faded over 10° around it; 0 leaves `triplanar` to decide.
    pub project_slope: f32,
    /// Low-frequency brightness and hue drift over the ground, and the
    /// texture at a quarter of its frequency blended in with distance, 0 to 1.
    pub macro_variation: f32,
    /// Metres the drawn ground moves up and down by the layer's height
    /// (its albedo's alpha, about its middle) within
    /// [`Terrain::displacement_distance`] of the camera (0: flat).
    /// Only drawn: collision and og.terrain.sample keep the field.
    pub displacement: f32,
    /// How much each cell of the layer's pattern takes its own tone, 0
    /// to 1: its normal map's alpha names the cell (opaque: none), and
    /// each cell's brightness (up to ±25 %) and warmth (±20 %) move by it
    /// (`cell_tone`, terrain_layers.wgsl).
    pub cell_variation: f32,
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
            anti_tiling: 0.0,
            height_contrast: 0.0,
            project_slope: 0.0,
            macro_variation: 0.0,
            displacement: 0.0,
            cell_variation: 0.0,
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
    /// The height error, in pixels, a coarser level may show before the
    /// finer one reaches further out (0: 1; `terrain_error.rs`). Larger
    /// draws fewer vertices.
    pub error: f32,
}

/// A terrain's tile budget when its lod gives none: 16 MiB.
pub const DEFAULT_TERRAIN_BUDGET: u64 = 16 << 20;

impl TerrainLod {
    /// Quads per side of every level, as drawn.
    pub fn ring_quads(&self) -> u32 {
        let r = if self.ring == 0 { 64 } else { self.ring };
        (r.clamp(16, 256) / 4) * 4
    }

    /// The pixels of height error a level may show, as drawn.
    pub fn error_pixels(&self) -> f32 {
        if self.error > 0.0 { self.error } else { 1.0 }
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
    /// Up to [`MAX_TERRAIN_LAYERS`] layers (more are ignored); without
    /// `layer_arrays` only the first [`TERRAIN_TEXTURED_LAYERS`] read
    /// their textures. Each pixel blends its four strongest layers.
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
    /// Its horizon map shades it: the sun hidden behind its ridges past
    /// the shadow cascades, and the sky its valleys do not see.
    pub horizon_shadows: bool,
    /// Metres from the camera within which layers with a
    /// [`TerrainLayer::displacement`] displace the drawn ground, fading out
    /// over its last 40% (0: none).
    pub displacement_distance: f32,
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
            horizon_shadows: true,
            displacement_distance: DEFAULT_DISPLACEMENT_DISTANCE,
        }
    }

    /// Whether its drawn ground is displaced: a reach, and a layer that
    /// moves it.
    pub fn displaces(&self) -> bool {
        self.displacement_distance > 0.0
            && self
                .layers
                .iter()
                .take(MAX_TERRAIN_LAYERS)
                .any(|l| l.displacement != 0.0)
    }

    /// The clipmap drawn for `view`, also read by frame diagnostics and the
    /// id pass. Coordinates are grid units from the terrain's -x, -z
    /// corner. `reach` is how far out each decimation's height error shows
    /// ([`crate::RenderScene::terrain_clipmap`] measures it; all zeros: no
    /// error anywhere, the levels as the lod's ring alone places them).
    ///
    /// Each level hands over to the next coarser one in its morph band.
    /// The band starts where the base ring puts it, or further out where
    /// the coarser level's error would show past it (more than
    /// [`TerrainLod::error_pixels`]): a peak that is large on screen stays
    /// at the finer level. The band moves with the camera as the error
    /// does, so the surface changes by at most that error as it moves. The
    /// levels grow (the ring: the lod's, or [`AUTO_RING`] quads) only as
    /// far as the bands need; past its band a level draws the coarser
    /// one's surface, so the ring itself changes nothing drawn.
    pub fn clipmap(
        &self,
        layout: TerrainLayout,
        view: &crate::TerrainView,
        reach: &[f32],
    ) -> TerrainClipmap {
        let cell = layout.cell();
        let unit = if self.lod.spacing > 0.0 {
            self.lod.spacing
        } else if self.displaces() {
            let tile = self
                .layers
                .iter()
                .take(MAX_TERRAIN_LAYERS)
                .filter(|l| l.displacement != 0.0)
                .map(|l| l.tile)
                .fold(f32::MAX, f32::min);
            displaced_unit(cell.x.min(cell.y), tile / horizontal_scale(self.transform))
        } else {
            cell.x.min(cell.y)
        };
        let local = self.transform.inverse().transform_point3(view.camera);
        let camera = (Vec2::new(local.x, local.z) + layout.size * 0.5) / unit;
        // A displaced terrain's levels are denser unless its lod says
        // otherwise ([`DISPLACED_RING`]).
        let base = if self.lod.ring == 0 && self.displaces() {
            DISPLACED_RING
        } else {
            self.lod.ring_quads()
        };
        // A level's inner edge, where the finer level meets it, lies under
        // ring / 4 + 1 of its quads from the camera; the morph starts past
        // it, or that edge's odd vertices would slide off the finer level's
        // collapsed ones and open cracks (ring 16 did, by up to half a quad).
        let morph_width = (base / 8).max(2).min(base / 4 - 3) as f32;
        let preferred = base as f32 * 0.5 - 2.0 - morph_width;
        // Quads from the camera at which level `l` must still be unmorphed:
        // past the farthest place the next level's error shows. That
        // level's vertices are `2^(l+1) × unit / cell` samples apart.
        let samples = unit / cell.x.min(cell.y);
        let required = |l: u32| -> f32 {
            let apart = (1u64 << (l + 1)) as f32 * samples;
            if apart <= 1.0 + 1e-4 {
                return 0.0;
            }
            let j = (apart.log2() - 1e-4).ceil() as usize;
            let r = reach.get(j).copied().unwrap_or(0.0);
            r / (unit * (1u64 << l) as f32)
        };
        let fits = |n: u32| n as f32 * 0.5 - 2.0 - morph_width;
        // Grown to 128 quads at most: 256 costs a terrain-wide 3 ms on an
        // M4 Pro (island-mid: 4.8 ms at 64, 6.0 at 128 with its bands
        // out, 8.1 at 256), so only a lod asking for it draws it.
        let ring = [base, base.max(AUTO_RING)]
            .into_iter()
            .find(|&n| (0..MAX_TERRAIN_LEVELS as u32).all(|l| required(l) <= fits(n)))
            .unwrap_or(base.max(AUTO_RING));
        let mut levels = levels(
            camera,
            layout.size / unit,
            ring,
            (view.far / horizontal_scale(self.transform) / unit) as f64,
        );
        for (l, level) in levels.iter_mut().enumerate() {
            let inner = if level.hole.is_some() {
                ring as f32 * 0.25 + 1.0
            } else {
                0.0
            };
            level.morph_start = required(l as u32).max(preferred).clamp(inner, fits(ring));
        }
        TerrainClipmap {
            unit,
            camera,
            ring,
            morph_width,
            levels,
        }
    }
}

/// A mesh drawn as part of a terrain this frame (a terrain stamp: a
/// cliff, an arch, an overhang): shaded with the terrain's layers in world
/// space, its own normals choosing them by their slope rules (rock on its
/// faces, the ground's layers on its top), and within `blend` metres above
/// the ground under it blended towards the ground's normal and look, so it
/// meets the terrain without a seam (the runtime-virtual-texture technique
/// without the texture: the terrain's material function is called
/// directly, `terrain_lit.wgsl` `fs_stamp`). Drawn in every pass the
/// terrain is (the frame, the prepass, the shadows, reflections, probes
/// and views), with the terrain's visibility.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainStampDraw {
    /// Its terrain: an index into [`crate::RenderScene::terrains`].
    pub terrain: usize,
    pub mesh: crate::scene::MeshHandle,
    /// The mesh's model to world.
    pub transform: Mat4,
    /// Metres above the ground over which it takes the ground's look (0: none).
    pub blend: f32,
    pub cast_shadows: bool,
}

/// Quads a side the clipmap's levels grow to where a coarser level's
/// height error shows ([`Terrain::clipmap`]); a lod's own `ring` above it
/// is kept.
pub const AUTO_RING: u32 = 128;

/// Quads a side of a displaced terrain's levels when its lod gives no
/// ring: a clipmap's quad spans about 4 / ring of the view's focal length
/// in pixels wherever it is (each level twice as far and twice as
/// coarse), some 10 px at 720p for 256 against 39 for the default 64, at
/// which a sett's joints and edges are drawn by a vertex or two and its
/// silhouette steps in teeth a grid cell wide. The corner cache
/// (terrain_corners.wgsl) keeps the four times denser mesh cheap: the
/// displaced corners are evaluated once a frame, not per pass.
pub const DISPLACED_RING: u32 = 256;

/// A displaced terrain's reach when it gives none ([`Terrain::new`]).
pub const DEFAULT_DISPLACEMENT_DISTANCE: f32 = 40.0;

/// The finest clipmap spacing of a displaced terrain whose lod gives none,
/// for sample spacing `cell`: halved until it is at most a 128th of the
/// finest repeat (`tile`, the terrain's horizontal metres) of a layer that
/// displaces, at most seven times (a 128th of a cell). The relief is then
/// drawn with about 128 vertices across a repeat of its height map near
/// the eye (a metre of setts: 7.8 mm, four to a 3 cm joint; a 2 m pebble
/// texture: 1.6 cm), coarser with each level out, as a clipmap's
/// triangles keep about the same size on screen. (At a 32nd, 3 cm, a
/// sett's 2 cm joint fell between two vertices: the stones swelled over
/// it and their edges stepped from vertex to vertex.)
fn displaced_unit(cell: f32, tile: f32) -> f32 {
    let mut unit = cell;
    for _ in 0..7 {
        if unit <= tile / 128.0 {
            break;
        }
        unit *= 0.5;
    }
    unit
}

/// The same layout the renderer uploads, before frustum culling.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainClipmap {
    /// Terrain-space metres per finest grid unit.
    pub unit: f32,
    pub camera: Vec2,
    pub ring: u32,
    /// The morph band's width in each level's quads (where it starts:
    /// [`TerrainLevel::morph_start`]).
    pub morph_width: f32,
    pub levels: Vec<TerrainLevel>,
}

/// `TerrainLayerGpu` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LayerGpu {
    color: [f32; 4],
    mapping: [f32; 4],
    slope: [f32; 4],
    altitude: [f32; 4],
    /// xyz: the layer's uv at the scene's origin, whole repeats dropped
    /// (`origin.rs`): the layers' patterns stay put when it moves; w its
    /// displacement in world metres.
    offset: [f32; 4],
    /// Anti-tiling, height contrast, the biplanar slope, macro variation.
    look: [f32; 4],
    /// The hex-tiling lattice at the scene's origin per plane ([`hex_lattice`]).
    hex: [[f32; 4]; 3],
    hex_base: [[i32; 4]; 2],
    /// xyz: the uv at the scene's origin at a quarter of the frequency,
    /// w: the cell variation.
    coarse: [f32; 4],
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
    /// x: the displacement's reach (the terrain's horizontal metres, 0:
    /// none), y: world metres per unit of the terrain's y, z: the
    /// terrain's y in its horizontal metres, w: lying snow's lift (world
    /// metres, [`snow_lift`]).
    displace: [f32; 4],
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    pyramid: [[f32; 4]; MAX_TERRAIN_PYRAMID as usize],
    layers: [LayerGpu; MAX_TERRAIN_LAYERS],
    /// The far colour of the og.Grass field on it (`grass.rs`).
    grass: crate::grass::FarGpu,
}

/// `TerrainLevel` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LevelGpu {
    center: [f32; 4],
    grid: [f32; 4],
    camera: [f32; 4],
    /// xy: the coarser level's centre (grid units), z: this level's first
    /// corner in the corner cache, w: the coarser level's.
    corners: [f32; 4],
}

const LEVEL_STRIDE: u64 = 256;

/// One clipmap level to draw: its uniform slot, its bounds and its
/// blocks' index ranges with their bounds (culled per pass: a block out
/// of the pass's view is not drawn; neighbours in view draw as one range).
#[derive(Clone, Debug)]
pub(crate) struct LevelDraw {
    pub offset: u32,
    pub center: Vec3,
    pub radius: f32,
    /// Each block's index range and its world box (min, max).
    pub blocks: Vec<(std::ops::Range<u32>, Vec3, Vec3)>,
    /// Its skirt's index range ([`RingRanges::skirt`]), when it morphs:
    /// drawn with any of its blocks.
    pub skirt: Option<std::ops::Range<u32>>,
}

/// The index ranges of `blocks` whose boxes `visible` admits, neighbours
/// in the buffer merged into one range.
pub(crate) fn visible_ranges(
    blocks: &[(std::ops::Range<u32>, Vec3, Vec3)],
    visible: impl Fn(Vec3, Vec3) -> bool,
) -> Vec<std::ops::Range<u32>> {
    let mut out: Vec<std::ops::Range<u32>> = Vec::new();
    for (r, lo, hi) in blocks {
        if r.is_empty() || !visible(*lo, *hi) {
            continue;
        }
        match out.last_mut() {
            Some(last) if last.end == r.start => last.end = r.end,
            _ => out.push(r.clone()),
        }
    }
    out
}

/// One terrain ready to draw in this frame's passes.
pub(crate) struct TerrainDraw {
    pub group: wgpu::BindGroup,
    /// Its corner cache (`terrain_vertex.wgsl`), read by every pass that
    /// draws its levels (group 2), and written by [`encode_corners`].
    pub corners: wgpu::BindGroup,
    pub corners_write: wgpu::BindGroup,
    /// Corners per level: (ring + 1)².
    pub corner_count: u32,
    /// Its pipelines (the layer-array variant's when its layers sample
    /// arrays).
    pub passes: TerrainPasses,
    /// The features its lit passes draw with this frame (pipeline_features.rs).
    pub features: Features,
    pub levels: Vec<LevelDraw>,
    pub cast_shadows: bool,
    /// Its index in [`crate::RenderScene::terrains`], and whether its group
    /// is the layer-array variant's (grass generation binds the group).
    pub terrain: usize,
    pub arrayed: bool,
    /// Its stamps this frame and their group (the stamp uniforms).
    pub stamps: Vec<StampDraw>,
    pub stamp_group: Option<wgpu::BindGroup>,
}

/// One terrain stamp to draw: its uniform slot, its mesh and its bounds.
#[derive(Clone, Debug)]
pub(crate) struct StampDraw {
    pub offset: u32,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub count: u32,
    pub center: Vec3,
    pub radius: f32,
    pub cast_shadows: bool,
}

/// `TerrainStampUniform` in terrain.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct StampGpu {
    model: [[f32; 4]; 4],
    normal: [[f32; 4]; 3],
    to_terrain: [[f32; 4]; 4],
    params: [f32; 4],
}

impl StampGpu {
    /// A stamp of `transform` on a terrain placed by `terrain`.
    pub(crate) fn new(transform: Mat4, terrain: Mat4, blend: f32) -> StampGpu {
        let n3 = glam::Mat3::from_mat4(transform).inverse().transpose();
        StampGpu {
            model: transform.to_cols_array_2d(),
            normal: [
                n3.x_axis.extend(0.0).to_array(),
                n3.y_axis.extend(0.0).to_array(),
                n3.z_axis.extend(0.0).to_array(),
            ],
            to_terrain: terrain.inverse().to_cols_array_2d(),
            params: [blend.max(0.0), terrain.y_axis.truncate().length(), 0.0, 0.0],
        }
    }
}

/// The stride of the stamps' uniform slots.
pub(crate) const STAMP_STRIDE: u64 = 256;

/// The levels of one terrain for a camera, in whole grid units.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainLevel {
    /// The centre (grid units from the terrain's -x, -z corner).
    pub center: [f64; 2],
    /// Grid units per quad (2^level).
    pub step: f64,
    /// Which ring variant leaves the finer level out (None: the finest level).
    pub hole: Option<(u32, u32)>,
    pub morph: bool,
    /// Where its morph band starts: its quads from the camera (Chebyshev).
    pub morph_start: f32,
}

/// The clipmap levels around `camera` (grid units, the terrain's -x, -z
/// corner at 0) over a terrain `extent` grid units wide, with `ring` quads
/// per side, out to `far` grid units.
pub(crate) fn levels(camera: Vec2, extent: Vec2, ring: u32, far: f64) -> Vec<TerrainLevel> {
    let cam = [camera.x as f64, camera.y as f64];
    let ext = [extent.x as f64, extent.y as f64];
    let quarter = ring as f64 / 4.0;
    let half = ring as f64 / 2.0;
    let mut out: Vec<TerrainLevel> = Vec::new();
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
            out.push(TerrainLevel {
                center: c,
                step,
                hole,
                morph: true,
                morph_start: 0.0,
            });
        }
        prev = Some(c);
    }
    if let Some(last) = out.last_mut() {
        last.morph = false;
    }
    out
}

/// Blocks per side of a level, each culled on its own (`BLOCKS`² per
/// level: Far Cry 5's per-patch culling on the CPU, with no readback).
pub(crate) const BLOCKS: u32 = 4;

/// One ring variant's index range and, within it, each block's (row by
/// row, `BLOCKS` to a row; a block the hole leaves empty is empty).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RingRange {
    pub all: std::ops::Range<u32>,
    pub blocks: [std::ops::Range<u32>; (BLOCKS * BLOCKS) as usize],
}

/// The index ranges of one ring size ([`ring_indices`]): the full grid and
/// the ring with its hole at each of the four offsets, and the skirt.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RingRanges {
    pub variants: [RingRange; 5],
    /// The zero-area triangles along the level's outer edge, each an odd
    /// corner between its two even neighbours: drawn with a level that
    /// morphs, they close the T-junctions where its odd corners meet the
    /// coarser level's edges (Losasso & Hoppe 2004's degenerate triangles).
    pub skirt: std::ops::Range<u32>,
}

/// Cell `(x, z)`'s four triangles in a level of `n` quads a side
/// (`terrain_vertex.wgsl` terrain_local_vertex's ids: the corners row by row,
/// then the centres): each from one edge to the cell's centre, wound as
/// the corners go round (-x -z, -x +z, +x +z, +x -z).
pub(crate) fn cell_fan(n: u32, x: u32, z: u32) -> [u32; 12] {
    let corner = |x: u32, z: u32| z * (n + 1) + x;
    let centre = (n + 1) * (n + 1) + z * n + x;
    let (a, b, c, d) = (
        corner(x, z),
        corner(x, z + 1),
        corner(x + 1, z + 1),
        corner(x + 1, z),
    );
    [a, b, centre, b, c, centre, c, d, centre, d, a, centre]
}

/// The skirt of a level of `n` quads a side ([`RingRanges::skirt`]).
pub(crate) fn skirt_indices(n: u32) -> Vec<u32> {
    let corner = |x: u32, z: u32| z * (n + 1) + x;
    let mut idx = Vec::with_capacity(2 * n as usize * 3);
    for i in (0..n).step_by(2) {
        idx.extend([corner(i, 0), corner(i + 1, 0), corner(i + 2, 0)]);
        idx.extend([corner(n, i), corner(n, i + 1), corner(n, i + 2)]);
        idx.extend([corner(i + 2, n), corner(i + 1, n), corner(i, n)]);
        idx.extend([corner(0, i + 2), corner(0, i + 1), corner(0, i)]);
    }
    idx
}

/// Whether ring variant `hole` leaves out cell `(x, z)` of a level of `n`
/// quads a side (the finer level's square).
pub(crate) fn in_hole(n: u32, hole: Option<(u32, u32)>, x: u32, z: u32) -> bool {
    hole.is_some_and(|(ox, oz)| {
        let (x0, z0) = (n / 4 + ox, n / 4 + oz);
        let (x1, z1) = (3 * n / 4 + ox, 3 * n / 4 + oz);
        x >= x0 && x < x1 && z >= z0 && z < z1
    })
}

impl RingRanges {
    /// These ranges `by` indices further on.
    fn shifted(&self, by: u32) -> RingRanges {
        let s = |r: &std::ops::Range<u32>| r.start + by..r.end + by;
        RingRanges {
            variants: self.variants.clone().map(|v| RingRange {
                all: s(&v.all),
                blocks: v.blocks.map(|b| s(&b)),
            }),
            skirt: s(&self.skirt),
        }
    }
}

/// The index buffer of one ring size: the full grid, then the ring with
/// the hole at each of its four offsets, each laid out block by block
/// (`n / BLOCKS` quads a side) so a block's quads are one range, then the
/// skirt. Returns the indices and the ranges.
fn ring_indices(n: u32) -> (Vec<u32>, RingRanges) {
    let mut idx = Vec::new();
    let mut ranges: Vec<RingRange> = Vec::new();
    let q = n / BLOCKS;
    for hole in [None, Some((0, 0)), Some((1, 0)), Some((0, 1)), Some((1, 1))] {
        let start = idx.len() as u32;
        let mut blocks: Vec<std::ops::Range<u32>> = Vec::new();
        for bz in 0..BLOCKS {
            for bx in 0..BLOCKS {
                let b = idx.len() as u32;
                for z in bz * q..(bz + 1) * q {
                    for x in bx * q..(bx + 1) * q {
                        if !in_hole(n, hole, x, z) {
                            idx.extend(cell_fan(n, x, z));
                        }
                    }
                }
                blocks.push(b..idx.len() as u32);
            }
        }
        ranges.push(RingRange {
            all: start..idx.len() as u32,
            blocks: blocks.try_into().expect("BLOCKS² blocks"),
        });
    }
    let start = idx.len() as u32;
    idx.extend(skirt_indices(n));
    let ranges = RingRanges {
        variants: ranges.try_into().expect("five ranges"),
        skirt: start..idx.len() as u32,
    };
    (idx, ranges)
}

/// The height over a cell of corner heights `h` (-x -z, +x -z, -x +z,
/// +x +z) at `f` (0 to 1 along x and z), on the triangle of its split
/// (og_core's heightfield_zigzag; `terrain_vertex.wgsl` split_height).
pub(crate) fn split_height(h: [f32; 4], f: Vec2) -> f32 {
    if (h[0] - h[3]).abs() > (h[1] - h[2]).abs() {
        if f.x + f.y <= 1.0 {
            h[0] + f.x * (h[1] - h[0]) + f.y * (h[2] - h[0])
        } else {
            h[3] + (1.0 - f.x) * (h[2] - h[3]) + (1.0 - f.y) * (h[1] - h[3])
        }
    } else if f.x >= f.y {
        h[0] + f.x * (h[1] - h[0]) + f.y * (h[3] - h[1])
    } else {
        h[0] + f.y * (h[2] - h[0]) + f.x * (h[3] - h[2])
    }
}

/// One clipmap level's mesh as `terrain_vertex.wgsl` terrain_local_vertex makes
/// it, on the CPU (the id pass draws what the frame drew): its (n + 1)²
/// corners then its n² cells' centres, in the terrain's space. `height`
/// answers the field's height at field coordinates (the finest level, as
/// every clipmap level reads it); `None` when it does.
pub(crate) fn level_vertices(
    clipmap: &TerrainClipmap,
    level: &TerrainLevel,
    layout: TerrainLayout,
    height: &dyn Fn(Vec2) -> Option<f32>,
) -> Option<Vec<Vec3>> {
    let n = clipmap.ring;
    let unit = clipmap.unit;
    let s = level.step as f32;
    let center = Vec2::new(level.center[0] as f32, level.center[1] as f32);
    let extent = layout.size / unit;
    let cell = layout.cell();
    let lo = -layout.size * 0.5;
    let width = if level.morph {
        clipmap.morph_width
    } else {
        0.0
    };
    let clamp = |c: Vec2| c.clamp(Vec2::ZERO, extent);
    let at = |m: Vec2| height(m * unit / cell);
    let alpha = |c: Vec2| {
        if width <= 0.0 {
            return 0.0;
        }
        let d = (c - clipmap.camera).abs().max_element() / s;
        ((d - level.morph_start) / width).clamp(0.0, 1.0)
    };
    let zigzag = |h: [f32; 4]| (h[0] - h[3]).abs() > (h[1] - h[2]).abs();
    let coarse_cell = |c: Vec2| -> Option<(Vec2, Vec2, [f32; 4])> {
        let p = (c / (2.0 * s)).floor() * (2.0 * s);
        let (a, b) = (clamp(p), clamp(p + 2.0 * s));
        Some((
            a,
            b,
            [
                at(a)?,
                at(Vec2::new(b.x, a.y))?,
                at(Vec2::new(a.x, b.y))?,
                at(b)?,
            ],
        ))
    };
    let coarse_height = |(a, b, h): (Vec2, Vec2, [f32; 4]), m: Vec2| {
        let span = b - a;
        let f = Vec2::select(
            span.cmpgt(Vec2::ZERO),
            (m - a) / span.max(Vec2::splat(1e-6)),
            Vec2::ZERO,
        );
        split_height(h, f)
    };
    let corner = |g: UVec2| center + (g.as_vec2() - n as f32 * 0.5) * s;
    let morphed = |c: Vec2, m: Vec2, h: f32, k: Option<(Vec2, Vec2, [f32; 4])>| {
        let a = alpha(c);
        match k {
            Some(k) if a > 0.0 => h + (coarse_height(k, m) - h) * a,
            _ => h,
        }
    };
    let local = |m: Vec2, h: f32| {
        let xz = lo + m * unit;
        Vec3::new(xz.x, h, xz.y)
    };
    let mut out = Vec::with_capacity(((n + 1) * (n + 1) + n * n) as usize);
    for gz in 0..=n {
        for gx in 0..=n {
            let g = UVec2::new(gx, gz);
            let c = corner(g);
            let m = clamp(c);
            let mut h = at(m)?;
            if alpha(c) > 0.0 {
                h = morphed(c, m, h, coarse_cell(c));
            }
            out.push(local(m, h));
        }
    }
    for gz in 0..n {
        for gx in 0..n {
            let g = UVec2::new(gx, gz);
            let c00 = corner(g);
            let k = coarse_cell(c00 + 0.5 * s)?;
            let gs = [g, g + UVec2::X, g + UVec2::Y, g + UVec2::ONE];
            let mut own = [0.0; 4];
            let mut mh = [0.0; 4];
            for i in 0..4 {
                let c = corner(gs[i]);
                let m = clamp(c);
                own[i] = at(m)?;
                mh[i] = morphed(c, m, own[i], Some(k));
            }
            let main = (mh[0] + mh[3]) * 0.5;
            let cross = (mh[1] + mh[2]) * 0.5;
            let mine = if zigzag(own) { cross } else { main };
            let theirs = if zigzag(k.2) { cross } else { main };
            let h = mine + (theirs - mine) * alpha(c00 + 0.5 * s);
            let m = (clamp(c00) + clamp(c00 + Vec2::splat(s))) * 0.5;
            out.push(local(m, h));
        }
    }
    Some(out)
}

/// Each block of a clipmap level of `n` quads a side ([`ring_indices`]'s
/// order): its box in the terrain's space, its xz the block's square on
/// the terrain (the morph moves vertices only up and down), clamped to
/// the terrain, its y the height range.
pub(crate) fn level_blocks(
    level: &TerrainLevel,
    n: u32,
    unit: f32,
    size: Vec2,
    (hmin, hmax): (f32, f32),
) -> Vec<(Vec3, Vec3)> {
    let q = (n / BLOCKS) as f64;
    let lo = -size * 0.5;
    let mut out = Vec::with_capacity((BLOCKS * BLOCKS) as usize);
    for bz in 0..BLOCKS {
        for bx in 0..BLOCKS {
            let corner = |i: usize, b: u32| {
                let g = b as f64 * q - n as f64 * 0.5;
                let c = level.center[i] + g * level.step;
                let ext = if i == 0 { size.x } else { size.y } as f64;
                ((c * unit as f64).clamp(0.0, ext)) as f32
            };
            let a = Vec2::new(corner(0, bx), corner(1, bz));
            let b = Vec2::new(corner(0, bx + 1), corner(1, bz + 1));
            out.push((
                Vec3::new(lo.x + a.x, hmin, lo.y + a.y),
                Vec3::new(lo.x + b.x, hmax, lo.y + b.y),
            ));
        }
    }
    out
}

/// Where a terrain's tiles come from: a whole field in memory (its tiles
/// made when drawn), or tiles handed over one by one (streaming).
#[derive(Clone, Debug)]
pub(crate) enum TerrainSource {
    /// The field, its height range (measured once, when it was set) and
    /// its height errors (`terrain_error.rs`; built when first asked for).
    Data(TerrainData, (f32, f32), FieldErrors),
    Stream {
        layout: TerrainLayout,
        range: (f32, f32),
        tiles: og_core::IdMap<u32, Arc<TerrainTile>>,
    },
}

/// A field's height errors, built when first asked for.
pub(crate) type FieldErrors = Arc<std::sync::OnceLock<Arc<crate::terrain_error::Decimation>>>;

impl TerrainSource {
    /// A field in memory, its range measured.
    pub(crate) fn data(d: TerrainData) -> TerrainSource {
        let range = d.range();
        TerrainSource::Data(d, range, FieldErrors::default())
    }

    /// `d`, a field of the same shape as this one's edited inside `lo`..`hi`
    /// (the terrain's space): its errors built again only there, when this
    /// one's were built.
    pub(crate) fn edited(&self, d: TerrainData, lo: Vec2, hi: Vec2) -> TerrainSource {
        let errors = FieldErrors::default();
        if let TerrainSource::Data(_, _, old) = self
            && let Some(old) = old.get()
        {
            let top = Vec2::new((d.columns - 1) as f32, (d.rows - 1) as f32);
            let sample = |v: Vec2| ((v + d.size * 0.5) / d.cell()).clamp(Vec2::ZERO, top);
            let (a, b) = (sample(lo).floor(), sample(hi).ceil());
            let lo = (a.x as u32, a.y as u32);
            let hi = (b.x as u32, b.y as u32);
            let mut e = (**old).clone();
            e.update(&|x, z| d.heights[(z * d.columns + x) as usize], lo, hi);
            let _ = errors.set(Arc::new(e));
        }
        let range = d.range();
        TerrainSource::Data(d, range, errors)
    }

    /// Its height errors' blocks for `layout` (`terrain_error.rs`): a
    /// field's, built the first time; a streamed terrain's, its tiles'
    /// (each tile's, at its own level; a finer one where it has come).
    pub(crate) fn error_blocks(
        &self,
        layout: &TerrainLayout,
    ) -> Vec<crate::terrain_error::ErrorBlock> {
        use crate::terrain_error::{Decimation, ERROR_BLOCK, ErrorBlock};
        let cell = layout.cell();
        let half = layout.size * 0.5;
        let metres = |x: u32, z: u32| Vec2::new(x as f32, z as f32) * cell - half;
        match self {
            TerrainSource::Data(d, _, errors) => {
                let e = errors.get_or_init(|| {
                    let n = (d.columns.max(d.rows) - 1).max(2);
                    let levels = 32 - (n - 1).leading_zeros();
                    Arc::new(Decimation::new(d.columns, d.rows, levels, &|x, z| {
                        d.heights[(z * d.columns + x) as usize]
                    }))
                });
                e.blocks()
                    .map(|(lo, hi, step, error)| ErrorBlock {
                        lo: metres(lo.0, lo.1),
                        hi: metres(hi.0, hi.1),
                        step,
                        error,
                    })
                    .collect()
            }
            TerrainSource::Stream { tiles, .. } => {
                let t = layout.tile;
                let mut out = Vec::new();
                for (&index, tile) in tiles.iter() {
                    let Some((k, tx, tz)) = layout.locate(index) else {
                        continue;
                    };
                    let step = layout.step(k);
                    let fine = |g: u32, n: u32| (g as u64 * step as u64).min(n as u64 - 1) as u32;
                    let mut at = 0usize;
                    for (j, b) in Decimation::tile_shape(t) {
                        let span = ERROR_BLOCK << j;
                        for bz in 0..b {
                            for bx in 0..b {
                                let error = tile.error.get(at).copied().unwrap_or(0.0);
                                at += 1;
                                let (x0, z0) = (tx * t + bx * span, tz * t + bz * span);
                                let (x1, z1) =
                                    ((x0 + span).min((tx + 1) * t), (z0 + span).min((tz + 1) * t));
                                out.push(ErrorBlock {
                                    lo: metres(fine(x0, layout.columns), fine(z0, layout.rows)),
                                    hi: metres(fine(x1, layout.columns), fine(z1, layout.rows)),
                                    step: k + j,
                                    error,
                                });
                            }
                        }
                    }
                }
                out
            }
        }
    }

    /// The tile size a field in memory is drawn with: [`TERRAIN_TILE`], or
    /// larger for a field too big for the tile table.
    pub(crate) fn layout(&self) -> Result<TerrainLayout, String> {
        match self {
            TerrainSource::Stream { layout, .. } => Ok(*layout),
            TerrainSource::Data(d, ..) => {
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
            TerrainSource::Data(_, range, _) => *range,
        }
    }
}

/// Seconds of tick time a streamed tile takes to fade in over its parent:
/// twelve ticks at 60 Hz, whatever the display's rate.
const FADE_SECONDS: f32 = 0.2;
/// The most ticks one frame steps the fades by; after a longer gap (ticks
/// stepped without frames, a capture-only run) every resident tile is in.
const FADE_CATCH_UP: u64 = 64;

/// [`TerrainRenderer::save`]: each resident tile's fade (data index, tile,
/// fade, the tile data's identity) and the tick the fades last stepped at.
#[derive(Clone, Default)]
pub(crate) struct TerrainSave {
    fades: Vec<(usize, u32, f32, usize)>,
    tick: Option<u64>,
}

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
    heights: wgpu::Buffer,
    normals: wgpu::Texture,
    normals_view: wgpu::TextureView,
    control: Option<(wgpu::Texture, wgpu::TextureView)>,
    table: wgpu::Buffer,
    table_cpu: Vec<f32>,
    slots: Vec<Option<SlotTile>>,
    by_tile: std::collections::HashMap<u32, usize>,
    /// A field in memory's control map resampled for its tiles.
    pyramid: Option<ControlPyramid>,
    /// A field in memory's hole counts (its tiles' hole flags).
    holes: Option<crate::terrain_tiles::HoleCounts>,
    /// The hole map `holes` counts (an in-place edit counts again only a
    /// changed one).
    holes_of: Option<Arc<[u8]>>,
    /// A field in memory's coarser levels (its tiles past level 0).
    height_pyramid: Option<crate::terrain_tiles::HeightPyramid>,
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
    /// Its corner cache: the buffer's corners, the buffer, and its groups
    /// to read and to write it (made again, larger, when it is outgrown).
    corners: Option<(u64, wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroup)>,
}

type GroupKey = (
    usize,
    u64,
    [Option<TextureHandle>; 2 * TERRAIN_TEXTURED_LAYERS],
    TerrainLayerArrays,
);

/// Draws [`Terrain`]s: the pipelines, each data's tiles, a uniform and
/// level buffer per drawn terrain, kept across frames.
pub(crate) struct TerrainRenderer {
    layout: wgpu::BindGroupLayout,
    passes: TerrainPasses,
    pub decals: decals::TerrainDecals,
    /// Draw every resident streamed tile faded in, without stepping the
    /// fades (a capture, `Renderer::render_capture`: tiles arrive by wall
    /// time, so a capture draws them settled).
    pub settled: bool,
    /// The layer-array variant (a terrain whose layers sample texture
    /// arrays, [`Terrain::layer_arrays`]): its group layout, with the
    /// albedo (5) and normal (9) maps as 2D arrays and no other layer
    /// texture, and its pipelines; made when a terrain first needs it.
    arrays: Option<(wgpu::BindGroupLayout, TerrainPasses)>,
    /// The passes with the hole test (a terrain with holes): the standard
    /// layout's, then the layer-array variant's; made when first needed.
    holed: [Option<TerrainPasses>; 2],
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
    /// One index buffer holding every ring size drawn so far
    /// ([`ring_indices`], one after another), and each one's ranges in it.
    indices: Option<(wgpu::Buffer, Vec<(u32, RingRanges)>)>,
    scene: u64,
    datas: Vec<Option<GpuData>>,
    /// The tick the streamed tiles' fades last stepped at.
    fade_tick: Option<u64>,
    /// The fades a restore brought back, for tiles uploaded at the next sync.
    restored: Option<TerrainSave>,
    slots: Vec<Slot>,
    generations: u64,
    max_layers: u32,
    /// Tiles uploaded in the last prepare and resident after it.
    pub uploads: u32,
    pub resident: u32,
    /// The corner cache's group layouts: read by the levels' passes, written
    /// by its writer.
    corners_layout: wgpu::BindGroupLayout,
    corners_write_layout: wgpu::BindGroupLayout,
    /// The stamps' group layout (one uniform slot, dynamic offset), their
    /// uniform slots and group, and their meshes uploaded (by mesh index:
    /// the scene, the mesh's revision, its vertices and indices).
    stamp_layout: wgpu::BindGroupLayout,
    stamp_slots: Option<(u64, wgpu::Buffer, wgpu::BindGroup)>,
    stamp_meshes: std::collections::HashMap<usize, StampMesh>,
}

/// A stamp's mesh on the GPU.
struct StampMesh {
    scene: u64,
    revision: u64,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
    bounds: (Vec3, Vec3),
}

pub(crate) const NORMALS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgb10a2Unorm;

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
/// standard one, or the layer-array variant's ([`TerrainRenderer`]); and
/// the same three for the terrain's stamps ([`TerrainStampDraw`]), which
/// draw a mesh with the terrain's group and their own (group 2).
#[derive(Clone)]
pub(crate) struct TerrainPasses {
    /// The lit and prepass pipelines for the frame's features
    /// (pipeline_features.rs).
    pub lit: Rc<Variants<crate::deferred::Sampled<wgpu::RenderPipeline>>>,
    pub prepass: Rc<Variants<Deferred<wgpu::RenderPipeline>>>,
    pub shadow: Rc<Deferred<crate::ShadowSet>>,
    pub stamp_lit: Rc<Variants<crate::deferred::Sampled<wgpu::RenderPipeline>>>,
    pub stamp_prepass: Rc<Variants<Deferred<wgpu::RenderPipeline>>>,
    pub stamp_shadow: Rc<Deferred<crate::ShadowSet>>,
    /// The corner cache's writer (`terrain_corners.wgsl`).
    pub corners: Rc<Deferred<wgpu::ComputePipeline>>,
}

impl TerrainPasses {
    /// Starts compiling what a frame draws a terrain with on worker
    /// threads (deferred.rs): the corners' writer and the lit pass, the
    /// prepass and the shadows when the frame draws them, and the same
    /// for its stamps.
    pub(crate) fn start(&self, features: Features, prepass: bool, shadows: bool, stamps: bool) {
        use crate::deferred::Start;
        self.lit.start(features);
        if prepass {
            self.prepass.start(features);
        }
        if stamps {
            self.stamp_lit.start(features);
            if prepass {
                self.stamp_prepass.start(features);
            }
        }
        let mut items: Vec<&dyn Start> = vec![&*self.corners];
        if shadows {
            items.push(&*self.shadow);
        }
        if stamps && shadows {
            items.push(&*self.stamp_shadow);
        }
        crate::deferred::start_all(&items);
    }
}

/// Which pass [`draw_levels`] draws: each terrain sets its own pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerrainPass {
    Lit,
    Prepass,
    /// Into a shadow layer of this kind (shadow_target.rs).
    Shadow(crate::ShadowTarget),
}

/// The pipeline layouts and modules [`terrain_passes`] builds from.
pub(crate) struct TerrainPassLayouts<'a> {
    pub frame: &'a wgpu::BindGroupLayout,
    pub shadow: &'a wgpu::BindGroupLayout,
    pub terrain: &'a wgpu::BindGroupLayout,
    pub stamp: &'a wgpu::BindGroupLayout,
    pub decals: &'a wgpu::BindGroupLayout,
    /// The corner cache read (the levels' passes) and written.
    pub corners: &'a wgpu::BindGroupLayout,
    pub corners_write: &'a wgpu::BindGroupLayout,
}

/// The pipelines in [`TerrainPasses`] over `layouts`, from these modules;
/// `holes`: with the hole test (terrain.wgsl `TERRAIN_HOLES`), whose
/// discard a terrain without holes leaves out (early depth and Apple's
/// hidden-surface removal keep working for it).
fn terrain_passes(
    device: &wgpu::Device,
    layouts: TerrainPassLayouts,
    samples: u32,
    lit_source: String,
    shadow_source: String,
    corners_source: String,
    holes: bool,
) -> TerrainPasses {
    // Made when a terrain is first drawn (deferred.rs).
    let lit_module = crate::deferred::module(device, "terrain", lit_source);
    let shadow_module = crate::deferred::module(device, "terrain shadow", shadow_source);
    let corners_module = crate::deferred::module(device, "terrain corners", corners_source);
    let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
        let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        })
    };
    let lit_layout = pl(
        "terrain",
        &[
            layouts.frame,
            layouts.terrain,
            layouts.corners,
            layouts.decals,
        ],
    );
    let prepass_layout = pl(
        "terrain prepass",
        &[layouts.frame, layouts.terrain, layouts.corners],
    );
    let shadow_layout = pl(
        "terrain shadow",
        &[layouts.shadow, layouts.terrain, layouts.corners],
    );
    let corners_layout = pl("terrain corners", &[layouts.corners_write, layouts.terrain]);
    let stamp_lit_layout = pl(
        "terrain stamp",
        &[
            layouts.frame,
            layouts.terrain,
            layouts.stamp,
            layouts.decals,
        ],
    );
    let stamp_prepass_layout = pl(
        "terrain stamp prepass",
        &[layouts.frame, layouts.terrain, layouts.stamp],
    );
    let stamp_shadow_layout = pl(
        "terrain stamp shadow",
        &[layouts.shadow, layouts.terrain, layouts.stamp],
    );
    let constants: &'static [(&'static str, f64)] = if holes {
        &[("TERRAIN_HOLES", 1.0)]
    } else {
        &[("TERRAIN_HOLES", 0.0)]
    };
    // The lit module's constants for a variant with `features`.
    let declared = lit_module.declared;
    let lit_constants = move |features: Features| {
        let mut c = constants.to_vec();
        c.extend(features.constants(declared));
        c
    };
    const STAMP_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
        &[Some(crate::VERTEX_LAYOUT)];
    let lit = |label: &'static str,
               layout: &wgpu::PipelineLayout,
               vs: &'static str,
               fs: &'static str,
               buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
        let (device, layout, module) = (device.clone(), layout.clone(), lit_module.clone());
        Variants::new(declared, move |features| {
            let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
            let constants = lit_constants(features);
            crate::deferred::Sampled::anywhere(samples, move |samples| {
                crate::hdr_pipeline_with(
                    &device,
                    &constants,
                    crate::HdrPipeline {
                        alpha_to_coverage: false,
                        label,
                        layout: &layout,
                        module: module.module(),
                        vs,
                        fs,
                        buffers,
                        cull: Some(wgpu::Face::Back),
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples,
                    },
                )
            })
        })
    };
    let options = move || wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    };
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
    let prepass =
        |label: &'static str,
         layout: &wgpu::PipelineLayout,
         vs: &'static str,
         fs: &'static str,
         buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
            let (device, layout, module) = (device.clone(), layout.clone(), lit_module.clone());
            Variants::new(declared, move |features| {
                let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
                let constants = lit_constants(features);
                Deferred::anywhere(move || {
                    let module = module.module();
                    let options = || wgpu::PipelineCompilationOptions {
                        constants: &constants,
                        ..Default::default()
                    };
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some(label),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module,
                            entry_point: Some(vs),
                            compilation_options: options(),
                            buffers,
                        },
                        primitive: wgpu::PrimitiveState {
                            cull_mode: Some(wgpu::Face::Back),
                            ..Default::default()
                        },
                        depth_stencil: depth(true),
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module,
                            entry_point: Some(fs),
                            compilation_options: options(),
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
            })
        };
    // An open surface: both faces cast (the bias in the lit pass keeps
    // its own lit side from shadowing itself); a stamp's back faces too.
    let shadow = |label: &'static str,
                  layout: &wgpu::PipelineLayout,
                  vs: &'static str,
                  fs: &'static str,
                  buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
        let (device, layout, module) = (device.clone(), layout.clone(), shadow_module.clone());
        // Both faces cast into a local layer's two channels too
        // (shadow_target.rs).
        let fragment_constants: &'static [(&'static str, f64)] = if holes {
            &[("TERRAIN_HOLES", 1.0), ("SHADOW_SIDE", 2.0)]
        } else {
            &[("TERRAIN_HOLES", 0.0), ("SHADOW_SIDE", 2.0)]
        };
        Deferred::anywhere(move || {
            let module = module.module();
            crate::ShadowSet::new(|target| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some(vs),
                        compilation_options: options(),
                        buffers,
                    },
                    primitive: wgpu::PrimitiveState {
                        cull_mode: target.cull(crate::Side::Double),
                        ..Default::default()
                    },
                    depth_stencil: Some(target.depth()),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some(fs),
                        compilation_options: wgpu::PipelineCompilationOptions {
                            constants: fragment_constants,
                            ..Default::default()
                        },
                        targets: &[Some(target.color())],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        })
    };
    TerrainPasses {
        lit: Rc::new(lit("terrain", &lit_layout, "vs", "fs", &[])),
        prepass: Rc::new(prepass(
            "terrain prepass",
            &prepass_layout,
            "vs",
            "fs_prepass",
            &[],
        )),
        shadow: Rc::new(shadow(
            "terrain shadow",
            &shadow_layout,
            "vs_shadow",
            "fs_shadow",
            &[],
        )),
        stamp_lit: Rc::new(lit(
            "terrain stamp",
            &stamp_lit_layout,
            "vs_stamp",
            "fs_stamp",
            STAMP_BUFFERS,
        )),
        stamp_prepass: Rc::new(prepass(
            "terrain stamp prepass",
            &stamp_prepass_layout,
            "vs_stamp",
            "fs_stamp_prepass",
            STAMP_BUFFERS,
        )),
        stamp_shadow: Rc::new(shadow(
            "terrain stamp shadow",
            &stamp_shadow_layout,
            "vs_stamp_shadow",
            "fs_stamp_shadow",
            STAMP_BUFFERS,
        )),
        corners: Rc::new({
            let device = device.clone();
            Deferred::anywhere(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("terrain corners"),
                    layout: Some(&corners_layout),
                    module: corners_module.module(),
                    entry_point: Some("write_corners"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        }),
    }
}

/// Hexagons per texture repeat along the lattice (terrain_lit.wgsl
/// `HEX_SCALE`, 2√3 as Mikkelsen's demo).
const HEX_SCALE: f64 = 3.464_101_615_137_754_6;

/// Where the hex-tiling lattice of a layer of `tile` metres stands at the
/// scene's `origin`, per projection plane (xz, zy, xy): the origin's place
/// in its skewed cell and that cell's centre in uv (whole repeats
/// dropped), and the cell's ids. Computed in f64 from the world's zero,
/// so a hexagon covers the same ground wherever the origin is: the
/// shader adds the point's uv from the origin to these.
fn hex_lattice(origin: Vec3, tile: f32) -> ([[f32; 4]; 3], [[i32; 4]; 2]) {
    let k = f64::from(1.0 / tile.max(1e-3));
    let o = [
        f64::from(origin.x),
        f64::from(origin.y),
        f64::from(origin.z),
    ];
    let mut hex = [[0.0; 4]; 3];
    let mut ids = [[0i32; 2]; 3];
    for (plane, (a, b)) in [(0, 2), (2, 1), (0, 1)].into_iter().enumerate() {
        let st = [o[a] * k * HEX_SCALE, o[b] * k * HEX_SCALE];
        let skewed = [st[0] - st[1] / 3f64.sqrt(), st[1] * 2.0 / 3f64.sqrt()];
        let base = skewed.map(f64::floor);
        let centre = [
            ((base[0] + 0.5 * base[1]) / HEX_SCALE).rem_euclid(1.0),
            (base[1] * 3f64.sqrt() * 0.5 / HEX_SCALE).rem_euclid(1.0),
        ];
        hex[plane] = [
            (skewed[0] - base[0]) as f32,
            (skewed[1] - base[1]) as f32,
            centre[0] as f32,
            centre[1] as f32,
        ];
        // Wrapping: the shader adds its cells in i32 the same way.
        ids[plane] = base.map(|b| b.rem_euclid(4_294_967_296.0) as u32 as i32);
    }
    (
        hex,
        [
            [ids[0][0], ids[0][1], ids[1][0], ids[1][1]],
            [ids[2][0], ids[2][1], 0, 0],
        ],
    )
}

/// A layer's anti-tiling, blend and macro fields at the scene's origin.
fn layer_look(l: &TerrainLayer, origin: Vec3) -> LayerGpu {
    let (hex, hex_base) = hex_lattice(origin, l.tile);
    LayerGpu {
        look: [
            l.anti_tiling.clamp(0.0, 1.0),
            l.height_contrast.clamp(0.0, 1.0),
            l.project_slope.clamp(0.0, 90.0),
            l.macro_variation.clamp(0.0, 1.0),
        ],
        hex,
        hex_base,
        coarse: {
            let mut c = uv_offset(origin, l.tile * 4.0);
            c[3] = l.cell_variation.clamp(0.0, 1.0);
            c
        },
        ..LayerGpu::default()
    }
}

/// How far lying snow lifts the drawn ground, in world metres
/// (terrain_vertex.wgsl terrain_snow_lift): the weather's lying depth
/// (`SurfaceWeather::snow_depth`) once its cover has built up, at most
/// 10 cm. Only the drawn ground moves (collision, `og.terrain.sample`,
/// picking and what stands on the ground keep the field), so it stays
/// small enough that nothing visibly floats or sinks.
pub(crate) fn snow_lift(scene: &crate::RenderScene) -> f32 {
    scene.surface_weather.as_ref().map_or(0.0, |w| {
        let t = (w.snow / 0.3).clamp(0.0, 1.0);
        w.snow_depth.clamp(0.0, 0.1) * t * t * (3.0 - 2.0 * t)
    })
}

pub(crate) const TERRAIN_WGSL: &str = wgsl3d!("terrain.wgsl");
/// The layers' sampling (after [`TERRAIN_WGSL`]): the lit pass's shading
/// and the displacement's heights.
const TERRAIN_LAYERS_WGSL: &str = wgsl3d!("terrain_layers.wgsl");
/// The clipmap's mesh with its displacement (after [`TERRAIN_LAYERS_WGSL`]):
/// the lit and shadow modules' `terrain_local_vertex`.
const TERRAIN_VERTEX_WGSL: &str = wgsl3d!("terrain_vertex.wgsl");
/// What terrain.wgsl's far grass needs declared (after wind_common.wgsl).
pub(crate) const GRASS_COMMON_WGSL: &str = wgsl3d!("grass_common.wgsl");

/// The terrain's lit module after the frame group, the light tables and the lighting.
pub(crate) fn terrain_lit_source(frame: &str, tables: &str, lighting: &str) -> String {
    format!(
        "{frame}{tables}{lighting}{}{}{}{GRASS_COMMON_WGSL}{TERRAIN_WGSL}{TERRAIN_LAYERS_WGSL}{TERRAIN_VERTEX_WGSL}{}{}",
        crate::WEATHER_SURFACE_WGSL,
        crate::AREA_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        wgsl3d!("terrain_lit.wgsl"),
        wgsl3d!("decals.wgsl")
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
pub(crate) fn terrain_array_declarations(source: &str) -> String {
    assert!(
        source.contains(LAYER_TEXTURES),
        "the layer-array variant cannot find terrain.wgsl's layer textures"
    );
    source.replacen(LAYER_TEXTURES, LAYER_ARRAYS, 1)
}

/// A terrain module (`standard`: [`terrain_lit_source`] or
/// [`terrain_shadow_source`]) as its layer-array variant: the arrays
/// declared, and `layer_albedo` and `layer_normal_map` reading layer
/// `i`'s array layer (slope.w and altitude.w) instead of its texture. The
/// rest is the same text, so the shading and the displacement are the
/// standard path's.
pub(crate) fn terrain_array_source(standard: &str) -> String {
    let s = terrain_array_declarations(standard);
    let start = s
        .find("fn layer_albedo(")
        .expect("terrain_layers.wgsl's layer_albedo");
    let end = s[start..]
        .find("fn band(")
        .map(|e| start + e)
        .expect("terrain_layers.wgsl's layer_normal_map ends before band");
    format!(
        "{}{}{}",
        &s[..start],
        "fn layer_albedo(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(albedo_array, layer_sampler, uv, i32(terrain.layers[i].slope.w), dx, dy);
}

fn layer_dimensions(i: u32) -> vec2<f32> {
  return vec2<f32>(textureDimensions(albedo_array));
}

fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  let t = textureSampleGrad(normal_array, layer_sampler, uv, i32(terrain.layers[i].altitude.w), dx, dy);
  return vec4<f32>(t.xyz * 2.0 - 1.0, t.w);
}

",
        &s[end..]
    )
}

/// The terrain's shadow module for the layer-array variant's layout (the
/// same entry points; the layer arrays declared).
pub(crate) fn terrain_shadow_array_source() -> String {
    terrain_array_source(&terrain_shadow_source())
}

/// The corner cache's writer module (its layer-array variant:
/// [`terrain_array_source`] of it).
pub(crate) fn terrain_corners_source() -> String {
    format!(
        "{}{GRASS_COMMON_WGSL}{TERRAIN_WGSL}{TERRAIN_LAYERS_WGSL}{TERRAIN_VERTEX_WGSL}{}",
        crate::wind::WIND_COMMON_WGSL,
        wgsl3d!("terrain_corners.wgsl")
    )
}

/// The terrain's shadow module.
pub(crate) fn terrain_shadow_source() -> String {
    format!(
        "{}{GRASS_COMMON_WGSL}{TERRAIN_WGSL}{TERRAIN_LAYERS_WGSL}{TERRAIN_VERTEX_WGSL}{}{}",
        crate::wind::WIND_COMMON_WGSL,
        crate::SHADOW_PASS_WGSL,
        wgsl3d!("terrain_shadow.wgsl")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diagonal_ridge_keeps_its_height_between_samples() {
        let mut field = TerrainData {
            size: Vec2::ONE,
            columns: 2,
            rows: 2,
            heights: [0.0, 300.0, 300.0, 100.0].into(),
            control: None,
            holes: None,
        };
        for (p, expected) in [
            (Vec2::ZERO, 300.0),
            (Vec2::splat(-0.25), 150.0),
            (Vec2::splat(0.25), 200.0),
        ] {
            assert_eq!(field.height(p), expected, "{p}");
        }
        field.heights = [300.0, 0.0, 100.0, 300.0].into();
        assert_eq!(field.height(Vec2::ZERO), 300.0);
        // Equal differences retain h00--h11, not the other diagonal's 250.
        field.heights = [0.0, 200.0, 300.0, 100.0].into();
        assert_eq!(field.height(Vec2::ZERO), 50.0);
    }

    /// The layer-array variant is the standard lit module with two arrays
    /// declared in place of the eight layer textures, and its two layer
    /// reads taking the layer from slope.w and altitude.w; nothing else.
    #[test]
    fn the_layer_array_variant_reads_two_arrays() {
        let standard = terrain_lit_source("", "", "");
        let array = terrain_array_source(&standard);
        for gone in ["albedo0", "albedo3", "normal1", "normal3"] {
            assert!(standard.contains(gone));
            assert!(!array.contains(gone), "{gone}");
        }
        // Normals, control, layer albedo and layer normals; the decal atlas
        // remains the one 2D texture shared with the standard variant.
        assert_eq!(array.matches("texture_2d_array<f32>;").count(), 4);
        assert_eq!(array.matches("texture_2d<f32>;").count(), 1);
        assert!(array.contains("i32(terrain.layers[i].slope.w)"));
        assert!(array.contains("i32(terrain.layers[i].altitude.w)"));
        let tail = |s: &str| s[s.find("fn band(").unwrap()..].to_string();
        assert_eq!(tail(&array), tail(&standard));
        assert!(!terrain_shadow_array_source().contains("albedo0"));
    }

    /// The lit terrain module as the renderer compiles it.
    fn lit_module() -> naga::Module {
        let src = crate::shaders::shader_sources()
            .into_iter()
            .find(|s| s.label == "terrain")
            .expect("the terrain module");
        naga::front::wgsl::parse_str(&src.wgsl).expect("parses")
    }

    /// Every function an entry point reaches through calls, by name.
    fn reached(module: &naga::Module, entry: &str) -> std::collections::BTreeSet<String> {
        fn walk(
            module: &naga::Module,
            body: &naga::Block,
            out: &mut std::collections::BTreeSet<String>,
        ) {
            for st in body.iter() {
                match st {
                    naga::Statement::Call { function, .. } => {
                        let f = &module.functions[*function];
                        if out.insert(f.name.clone().unwrap_or_default()) {
                            walk(module, &f.body, out);
                        }
                    }
                    naga::Statement::Block(b) => walk(module, b, out),
                    naga::Statement::If { accept, reject, .. } => {
                        walk(module, accept, out);
                        walk(module, reject, out);
                    }
                    naga::Statement::Switch { cases, .. } => {
                        for c in cases {
                            walk(module, &c.body, out);
                        }
                    }
                    naga::Statement::Loop {
                        body, continuing, ..
                    } => {
                        walk(module, body, out);
                        walk(module, continuing, out);
                    }
                    _ => {}
                }
            }
        }
        let e = module
            .entry_points
            .iter()
            .find(|e| e.name == entry)
            .unwrap_or_else(|| panic!("{entry}"));
        let mut out = Default::default();
        walk(module, &e.function.body, &mut out);
        out
    }

    /// The prepass writes the roughness the lit pass shades with (one
    /// function of the layers' weights, `blended_roughness`, in both)
    /// without reading a normal map or blending a colour: before, it ran
    /// the whole `terrain_look`, every layer's normal taps included.
    #[test]
    fn the_prepass_reads_roughness_alone() {
        let module = lit_module();
        let prepass = reached(&module, "fs_prepass");
        let lit = reached(&module, "fs");
        for f in [
            "blended_roughness",
            "blend_weights",
            "pick_layers",
            "toksvig_roughness",
        ] {
            assert!(prepass.contains(f) && lit.contains(f), "{f}");
        }
        for f in [
            "layer_normal_map",
            "taps_normal",
            "terrain_look",
            "macro_tint",
        ] {
            assert!(!prepass.contains(f), "the prepass reaches {f}");
            assert!(lit.contains(f), "the lit pass reaches {f}");
        }
    }

    /// The layer records and the terrain's uniform have the layout the
    /// shader reads (naga's offsets of the WGSL structs).
    #[test]
    fn the_terrain_uniform_matches_the_shader() {
        use std::mem::{offset_of, size_of};
        let module = lit_module();
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("lays out");
        let members = |name: &str| {
            let (_, ty) = module
                .types
                .iter()
                .find(|(_, t)| t.name.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("{name}"));
            let naga::TypeInner::Struct { members, span } = &ty.inner else {
                panic!("{name} is a struct");
            };
            (
                members
                    .iter()
                    .map(|m| (m.name.clone().unwrap_or_default(), m.offset as usize))
                    .collect::<Vec<_>>(),
                *span as usize,
            )
        };
        macro_rules! fields {
            ($t:ty: $($f:ident),*) => { vec![$((stringify!($f).to_string(), offset_of!($t, $f))),*] };
        }
        assert_eq!(
            members("TerrainLayerGpu"),
            (
                fields!(LayerGpu: color, mapping, slope, altitude, offset, look, hex, hex_base, coarse),
                size_of::<LayerGpu>()
            )
        );
        assert_eq!(
            members("TerrainStampUniform"),
            (
                fields!(StampGpu: model, normal, to_terrain, params),
                size_of::<StampGpu>()
            )
        );
        let (list, span) = members("TerrainUniform");
        assert_eq!(span, size_of::<TerrainGpu>());
        assert_eq!(
            list,
            fields!(TerrainGpu: model, normal, origin, extent, params, tiles, displace, view_proj,
                prev_view_proj, pyramid, layers, grass)
        );
    }

    /// The hex-tiling lattice, as `plane_taps` places it from the values
    /// [`hex_lattice`] gives, covers the same ground wherever the scene's
    /// origin is: a world point falls in the same cell (the same hash, so
    /// the same turn and shift) at the same uv, whole repeats aside. The
    /// shader's arithmetic in f32 from the point relative to the origin.
    #[test]
    fn hexagons_stay_on_the_ground_when_the_origin_moves() {
        const S: f32 = HEX_SCALE as f32;
        let centre = |v: Vec2| Vec2::new(v.x + 0.5 * v.y, 0.866_025_4 * v.y) / S;
        // Per tap: its cell's ids and the uv of the point in its copy
        // before the turn and hash shift (both of which follow the ids).
        let taps = |world: Vec3, origin: Vec3, tile: f32| {
            let (hex, base) = hex_lattice(origin, tile);
            let k = 1.0 / tile;
            let rel = world - origin;
            let u = Vec2::new(rel.x, rel.z) * k;
            let hf = hex[0];
            let st = u * S;
            let sk =
                Vec2::new(st.x - 0.577_350_27 * st.y, 1.154_700_5 * st.y) + Vec2::new(hf[0], hf[1]);
            let id = sk.floor();
            let f = sk - id;
            let s = if 1.0 - f.x - f.y < 0.0 { 1.0 } else { 0.0 };
            let mf = centre(Vec2::new(hf[0], hf[1]));
            [
                id + Vec2::new(s, s),
                id + Vec2::new(s, 1.0 - s),
                id + Vec2::new(1.0 - s, s),
            ]
            .map(|v| {
                let cell = (
                    (v.x as i32).wrapping_add(base[0][0]),
                    (v.y as i32).wrapping_add(base[0][1]),
                );
                let cen = centre(v);
                let local = u + mf - cen;
                let at = Vec2::new(hf[2], hf[3]) + cen;
                (cell, local, at - at.floor())
            })
        };
        let tile = 3.7;
        for world in [
            Vec3::new(10.3, 2.0, -4.6),
            Vec3::new(20_001.13, 0.0, 15_000.9),
        ] {
            let a = taps(world, Vec3::ZERO, tile);
            for origin in [
                Vec3::new(world.x - 3.3, 0.0, world.z + 7.1),
                Vec3::new(world.x - 100.0, 50.0, world.z - 64.0),
            ] {
                let b = taps(world, origin, tile);
                for ((ca, la, aa), (cb, lb, ab)) in a.iter().zip(&b) {
                    assert_eq!(ca, cb, "{world} from {origin}");
                    assert!((*la - *lb).length() < 2e-3, "{la} {lb}");
                    let d = (*aa - *ab).abs();
                    let d = d.min(Vec2::ONE - d);
                    assert!(d.length() < 2e-3, "{aa} {ab}");
                }
            }
        }
    }

    /// Each ring variant's blocks cover its range in order, every quad in
    /// the block whose square holds it, and the hole's quads in none.
    #[test]
    fn a_rings_blocks_partition_its_quads_by_square() {
        let n = 64u32;
        let (idx, ranges) = ring_indices(n);
        let q = n / BLOCKS;
        for (v, r) in ranges.variants.iter().enumerate() {
            assert_eq!(r.blocks[0].start, r.all.start);
            assert_eq!(r.blocks.last().unwrap().end, r.all.end);
            for w in r.blocks.windows(2) {
                assert_eq!(w[0].end, w[1].start);
            }
            let quads = r.all.len() as u32 / 12;
            assert_eq!(quads, if v == 0 { n * n } else { n * n - n * n / 4 });
            for (b, br) in r.blocks.iter().enumerate() {
                let (bx, bz) = (b as u32 % BLOCKS, b as u32 / BLOCKS);
                // Every third index is a triangle's cell centre.
                for &i in idx[br.start as usize..br.end as usize]
                    .iter()
                    .skip(2)
                    .step_by(3)
                {
                    let cell = i - (n + 1) * (n + 1);
                    let (x, z) = (cell % n, cell / n);
                    assert!(
                        x / q == bx && z / q == bz,
                        "variant {v}: quad ({x}, {z}) in block {b}"
                    );
                }
            }
        }
    }

    /// From the terrain's edge looking out over it from above, culling
    /// each level's blocks draws less than half the triangles culling
    /// whole levels does (every level holds the camera, so whole-level
    /// culling draws them all).
    #[test]
    fn blocks_out_of_view_are_not_drawn() {
        let size = Vec2::splat(2048.0);
        let layout = TerrainLayout::new(size, 1025, 1025, 128, None).unwrap();
        let t = Terrain::new(
            crate::RenderScene::new().add_terrain(TerrainData::flat(size, 1025, 1025)),
        );
        let eye = Vec3::new(1000.0, 40.0, 0.0);
        let camera = crate::Camera {
            position: eye,
            target: Vec3::new(1400.0, 0.0, 30.0),
            near: 0.3,
            far: 3000.0,
            ..Default::default()
        };
        let f = crate::frame::camera_frame(&camera, 16.0 / 9.0, Vec2::ZERO);
        let m = f.view_proj;
        let planes = [
            m.row(3) + m.row(0),
            m.row(3) - m.row(0),
            m.row(3) + m.row(1),
            m.row(3) - m.row(1),
            m.row(2),
            m.row(3) - m.row(2),
        ]
        .map(|p| p / p.truncate().length());
        let visible = |c: Vec3, r: f32| planes.iter().all(|p| p.truncate().dot(c) + p.w >= -r);
        let clipmap = t.clipmap(layout, &crate::TerrainView::of(&camera, 720), &[]);
        let (_, ranges) = ring_indices(clipmap.ring);
        let (mut levels, mut blocks) = (0usize, 0usize);
        for l in &clipmap.levels {
            let ring = match l.hole {
                None => &ranges.variants[0],
                Some((ox, oz)) => &ranges.variants[1 + (ox + 2 * oz) as usize],
            };
            let boxes = level_blocks(l, clipmap.ring, clipmap.unit, size, (0.0, 10.0));
            let lo = boxes.iter().fold(Vec3::MAX, |a, b| a.min(b.0));
            let hi = boxes.iter().fold(Vec3::MIN, |a, b| a.max(b.1));
            let (c, r) = renderer::world_sphere(t.transform, lo, hi);
            if !visible(c, r) {
                continue;
            }
            levels += ring.all.len() / 3;
            let world: Vec<_> = boxes
                .iter()
                .zip(&ring.blocks)
                .map(|((a, b), range)| {
                    let (lo, hi) = renderer::world_box(t.transform, *a, *b);
                    (range.clone(), lo, hi)
                })
                .collect();
            let inside = |lo: Vec3, hi: Vec3| {
                planes.iter().all(|p| {
                    let n = p.truncate();
                    n.dot(Vec3::select(n.cmpge(Vec3::ZERO), hi, lo)) + p.w >= 0.0
                })
            };
            blocks += visible_ranges(&world, inside)
                .iter()
                .map(|r| r.len() / 3)
                .sum::<usize>();
        }
        assert!(levels > 0 && blocks > 0, "{levels} {blocks}");
        assert!(
            blocks * 2 <= levels,
            "blocks draw {blocks} triangles, whole levels {levels}"
        );
    }

    /// Every level's hole (the ring variant its index range draws) is
    /// exactly the finer level's square, so the levels tile the plane: no
    /// quad is drawn twice and none is missing.
    /// A level's morph band moves out past where the next coarser level's
    /// height error shows, so a peak large on screen keeps its finer level;
    /// with no error the bands stay where the lod's ring puts them.
    #[test]
    fn a_visible_height_error_keeps_the_finer_level_out_to_it() {
        let t = Terrain::new(crate::RenderScene::new().add_terrain(TerrainData::flat(
            Vec2::splat(64.0),
            2,
            2,
        )));
        let layout = TerrainLayout::new(Vec2::splat(4096.0), 4097, 4097, 128, None).unwrap();
        let view = crate::TerrainView::perspective(Vec3::new(0.0, 50.0, 0.0), 5000.0, 60.0, 720);
        let flat = t.clipmap(layout, &view, &[]);
        assert_eq!(flat.ring, 64);
        assert!(flat.levels.iter().all(|l| l.morph_start == 22.0));
        // Level 1's coarser neighbour (4 samples apart) errs as far as 150 m
        // out: level 1 (2 m quads) must stay unmorphed to 75 of its quads,
        // which a ring of 64 cannot hold.
        let mut reach = [0.0f32; 32];
        reach[2] = 150.0;
        let c = t.clipmap(layout, &view, &reach);
        assert_eq!(c.ring, AUTO_RING);
        assert_eq!(c.levels[1].morph_start, 54.0, "{:?}", c.levels);
        // The others keep the base band, past their inner edge (ring / 4 + 1).
        assert!(
            c.levels
                .iter()
                .enumerate()
                .all(|(l, v)| l == 1 || v.morph_start <= 33.0)
        );
        // Within 44 m (22 quads), ring 64's own band covers it.
        reach[2] = 40.0;
        let c = t.clipmap(layout, &view, &reach);
        assert_eq!((c.ring, c.levels[1].morph_start), (64, 22.0));
        // At 100 m the band starts at 50 quads, in a ring of 128.
        reach[2] = 100.0;
        let c = t.clipmap(layout, &view, &reach);
        assert_eq!((c.ring, c.levels[1].morph_start), (AUTO_RING, 50.0));
    }

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
