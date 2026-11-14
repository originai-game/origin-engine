//! The ground: a height field as what stands, grows or lies on it reads
//! it. The terrain system draws its surface; the grass, the scattered
//! copies and the id pass read the same field through here, so none of
//! them names another.
//!
//! - The field and how a frame places it: [`TerrainData`] and its layers
//!   (`og_render_data::terrain`), [`Terrain`], where its tiles come from
//!   ([`TerrainSource`]), its tiles and their pyramid ([`terrain_tiles`]).
//! - Its levels around a camera ([`TerrainClipmap`], `clipmap`): the grid
//!   its surface is drawn and picked on.
//! - Reading it at a point on the CPU ([`Ground`], `sample`): its height,
//!   normal, layer weights and holes, from a field in memory or a streamed
//!   terrain's resident tiles.
//! - Reading it on the GPU (`gpu`): the group a drawn ground binds
//!   ([`GroundDraw`], [`GroundGpu`]), the WGSL that declares it, and the
//!   far colour of what grows on it ([`FarGpu`]).

mod clipmap;
mod gpu;
mod sample;
mod source;
pub(crate) mod terrain_tiles;

pub use clipmap::*;
pub(crate) use gpu::*;
pub(crate) use sample::*;
pub(crate) use source::*;

use crate::scene::{TerrainHandle, TextureArrayHandle};
use crate::terrain_tiles::{TERRAIN_TILE, TerrainLayout, TerrainTile};
use glam::{Mat4, UVec2, Vec2, Vec3};
pub use og_render_data::terrain::*;
use std::sync::Arc;

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
