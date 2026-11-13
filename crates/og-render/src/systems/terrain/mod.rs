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
pub(crate) mod terrain_tiles;
pub(crate) use renderer::*;

mod levels;
mod passes;
mod shading;
mod source;
pub use levels::*;
use passes::*;
pub(crate) use shading::*;
pub(crate) use source::*;

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
    /// Its transform mirrors it: it draws with the pipelines that take
    /// clockwise as the front ([`TerrainPasses::lit`]).
    pub mirrored: bool,
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
    /// Its own transform mirrors its mesh ([`TerrainDraw::mirrored`]).
    pub mirrored: bool,
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
    /// The features the frame's terrain pipelines draw with (the
    /// renderer's `frame_features`: every one in a capture).
    pub features: crate::pipeline_features::Features,
    /// The lit features the renderer is made without (pipeline_features.rs):
    /// without the weather on surfaces, lying snow lifts no ground.
    off: crate::pipeline_features::Features,
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
    /// The lit features the renderer is made without (pipeline_features.rs):
    /// out of the lit source.
    pub off: crate::pipeline_features::Features,
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
    /// (pipeline_features.rs), each twice: `[0]` for a terrain (or stamp)
    /// as it is made, `[1]` for one its transform mirrors (a negative
    /// determinant, [`crate::mirrors`]), which take clockwise as the
    /// front. The shadows draw both faces and need no second.
    pub lit: [Rc<Variants<crate::deferred::Sampled<wgpu::RenderPipeline>>>; 2],
    pub prepass: [Rc<Variants<Deferred<wgpu::RenderPipeline>>>; 2],
    pub shadow: Rc<Deferred<crate::ShadowSet>>,
    pub stamp_lit: [Rc<Variants<crate::deferred::Sampled<wgpu::RenderPipeline>>>; 2],
    pub stamp_prepass: [Rc<Variants<Deferred<wgpu::RenderPipeline>>>; 2],
    pub stamp_shadow: Rc<Deferred<crate::ShadowSet>>,
    /// The corner cache's writer (`terrain_corners.wgsl`).
    pub corners: Rc<Deferred<wgpu::ComputePipeline>>,
}

impl TerrainPasses {
    /// Starts compiling what a frame draws a terrain with on worker
    /// threads (deferred.rs): the corners' writer and the lit pass, the
    /// prepass and the shadows when the frame draws them, and the same
    /// for its stamps. `mirrored`: the terrain's transform mirrors it
    /// (a mirrored stamp's pipelines are made when it is first drawn).
    pub(crate) fn start(
        &self,
        features: Features,
        mirrored: bool,
        prepass: bool,
        shadows: bool,
        stamps: bool,
    ) {
        use crate::deferred::Start;
        let m = usize::from(mirrored);
        self.lit[m].start(features);
        if prepass {
            self.prepass[m].start(features);
        }
        if stamps {
            self.stamp_lit[0].start(features);
            if prepass {
                self.stamp_prepass[0].start(features);
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

#[cfg(test)]
mod tests;
