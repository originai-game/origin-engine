//! The IDL's `terrain` module (phase 2): cooked terrains loaded into the
//! renderer and into physics (a height field under a Geometry handle of
//! its own), drawn from og.Terrain components at extraction, sampled and
//! edited from game code. The registry is the render state's: terrains are
//! assets, kept across a hot reload.
//!
//! Phase 3: a terrain is not decoded whole at load. The renderer is handed
//! its tiles as the camera needs them (`crate::bind::terrain_stream`); physics
//! decodes the whole field only for a terrain whose file asks for a
//! collider; `og.terrain.sample` decodes the level-0 tiles it reads. The
//! first edit decodes the whole field (heights and control map), and from
//! then on the renderer draws that field in memory.
//!
//! Edits are world state (phase 3): every edit joins an edit log kept in
//! the world's `og.terrain` state block, so a snapshot saves them, the
//! world hash covers them, and a restore puts every terrain back as cooked
//! and replays the restored log (before the next tick's systems, or the
//! next og.terrain call, whichever comes first).

use crate::engine::{Engine, engine};
use crate::host::wire::{
    self, Color, FromJson, TerrainBounds, TerrainBrush, TerrainCreateDesc, TerrainEdit,
    TerrainInfo, TerrainLayerDesc, TerrainSample, TerrainSampleDesc, TerrainStampDesc, ToJson,
    record,
};
use crate::parse;
use crate::present::extract::{Hierarchy, RenderState};
use crate::present::fx::{f, kept_rows};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::World;
use og_render::{TerrainControl, TerrainData, TerrainLayer, TerrainLayout, TerrainLod};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod stamp;

mod draw;
mod edit;
mod log;
mod release;

pub(crate) use draw::*;
pub use edit::*;
pub(crate) use log::*;
pub(crate) use release::free_released;

/// The Geometry handles of terrains' height fields: a range of their own,
/// beside glTF geometries' (`assets::handle`).
const GEOMETRY_BASE: u64 = 0x7e << 32;

/// One loaded terrain (cooked streamed or runtime-created).
pub(crate) struct LoadedTerrain {
    pub render: og_render::TerrainHandle,
    /// The cooked tiles, still compressed (`None` for runtime-created terrains).
    pub cooked: Option<og_bundle::runtime::CookedTerrain>,
    /// Initial heightfield for runtime-created terrains (restored on replay/unstamp).
    pub initial: Option<TerrainData>,
    /// Layout metadata (always populated for both cooked and runtime terrains).
    pub layout: TerrainLayout,
    /// Height range before edits.
    pub initial_range: (f32, f32),
    /// Sea level height for coastal cues.
    pub sea_level: Option<f32>,
    /// The whole field in memory: physics' (a collider terrain), a runtime
    /// terrain, or an edited one's. `None`: only the tiles below are decoded.
    pub field: Option<TerrainData>,
    /// Edited: `field` holds the edits and the renderer draws it.
    pub edited: bool,
    /// Level-0 tiles' heights and hole masks decoded for sampling, by
    /// tile index.
    pub tiles: std::cell::RefCell<IdMap<u32, SampledTile>>,
    pub layers: Vec<TerrainLayer>,
    /// The texture arrays its layers sample (`layers`, `layerNormals`), as
    /// load indices in RenderState::texture_arrays: resolved to the
    /// renderer's handles each frame (an array of a streaming group comes
    /// back under a new one).
    pub layer_arrays: [Option<usize>; 2],
    pub height_blend: f32,
    pub lod: TerrainLod,
    pub cast_shadows: bool,
    pub collision: bool,
    /// Its heights' version, new at its load, each edit and each restore's
    /// reset, unique across engines: what a seabed made from it is keyed by
    /// ([`seabed_grid`]).
    pub generation: u64,
    /// The runtime bundle it was loaded from (an asset entry index); `None`
    /// for the boot bundle's or runtime-created terrains.
    pub bundle: Option<usize>,
    /// Its layer textures and arrays with the bundle each came from
    /// (`None`: the boot bundle): what goes with it when it is freed.
    pub owned: Vec<(LayerAsset, Option<usize>)>,
}

/// A texture or texture array a terrain's layers load, by its id.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LayerAsset {
    Texture(String),
    Array(String),
}

/// The loaded terrains by load index. A freed terrain's slot stays empty,
/// so a handle never comes to name another terrain.
#[derive(Default)]
pub(crate) struct Loaded(Vec<Option<Box<LoadedTerrain>>>);

impl Loaded {
    pub fn get(&self, i: usize) -> Option<&LoadedTerrain> {
        self.0.get(i)?.as_deref()
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut LoadedTerrain> {
        self.0.get_mut(i)?.as_deref_mut()
    }

    /// The slots, freed ones included: the next load's index.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The terrains loaded now, with their load indices.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &LoadedTerrain)> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, t)| Some((i, t.as_deref()?)))
    }

    fn push(&mut self, t: LoadedTerrain) {
        self.0.push(Some(Box::new(t)));
    }

    fn take(&mut self, i: usize) -> Option<Box<LoadedTerrain>> {
        self.0.get_mut(i)?.take()
    }
}

impl std::ops::Index<usize> for Loaded {
    type Output = LoadedTerrain;
    fn index(&self, i: usize) -> &LoadedTerrain {
        self.get(i).expect("a loaded terrain (index_of checks it)")
    }
}

impl std::ops::IndexMut<usize> for Loaded {
    fn index_mut(&mut self, i: usize) -> &mut LoadedTerrain {
        self.get_mut(i)
            .expect("a loaded terrain (index_of checks it)")
    }
}

/// A terrain heights' version no other has had ([`LoadedTerrain::generation`]).
fn next_generation() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Every terrain an engine loaded, by load index, and the og.Terrain
/// layout extraction reads.
#[derive(Default)]
pub(crate) struct TerrainState {
    pub loaded: Loaded,
    pub by_id: BTreeMap<String, usize>,
    pub scatter_issues: BTreeMap<u64, String>,
    pub streamer: crate::bind::terrain_stream::Streamer,
    /// Every wanted tile decoded and shown each frame (no streaming).
    pub resident: bool,
    /// Bytes of tiles each terrain keeps on the GPU (0: its lod's).
    pub budget: u64,
    /// The next extraction is a capture's: settle the tiles first.
    pub settle: bool,
    /// The streamed terrains (load indices) a frame drew already: the first
    /// frame that draws one settles, as a capture does.
    pub drawn: BTreeSet<usize>,
    /// The seabeds made from these terrains (og.Water.seabed), for the
    /// tick and the frame.
    pub shores: crate::bind::seabed::Shores,
}

fn band_weight(v: f32, lo: f32, hi: f32, fade: f32) -> f32 {
    let below = if fade > 0.0 {
        ((v - (lo - fade)) / fade).clamp(0.0, 1.0)
    } else if v >= lo {
        1.0
    } else {
        0.0
    };
    let above = if fade > 0.0 {
        (((hi + fade) - v) / fade).clamp(0.0, 1.0)
    } else if v <= hi {
        1.0
    } else {
        0.0
    };
    below * above
}

impl LoadedTerrain {
    /// The field's extent, samples and current height range.
    fn size(&self) -> Vec2 {
        self.layout.size
    }

    pub(crate) fn range(&self) -> (f32, f32) {
        match (&self.field, self.edited) {
            (Some(f), true) => f.range(),
            _ => self.initial_range,
        }
    }

    /// Calls `f` with a sampler of the heights as they are now: the field
    /// in memory, or the level-0 tiles decoded as they are read.
    fn with_sampler<R>(&self, f: impl FnOnce(&dyn Fn(Vec2) -> (f32, Vec3)) -> R) -> R {
        if let Some(d) = &self.field {
            return f(&|p| (d.height(p), d.normal(p)));
        }
        let sampler = self.tile_sampler();
        f(&|p| (sampler.height(p), sampler.normal(p)))
    }

    /// [`Self::with_sampler`]'s heights alone: a normal reads sixteen
    /// samples to a height's four, and the drop shadows' fit reads only
    /// heights (many of them a frame, along the sun's rays).
    fn with_heights<R>(&self, f: impl FnOnce(&dyn Fn(Vec2) -> f32) -> R) -> R {
        if let Some(d) = &self.field {
            return f(&|p| d.height(p));
        }
        let sampler = self.tile_sampler();
        f(&|p| sampler.height(p))
    }

    /// Dominant terrain layer index at terrain-local point `p`, world height
    /// `height`, and slope `slope_deg`: reads the control map when resident,
    /// otherwise evaluates layer altitude/slope rules.
    fn dominant_layer(&self, p: Vec2, height: f32, slope_deg: f32) -> u32 {
        if let Some(d) = &self.field
            && let Some(ctrl) = &d.control
            && ctrl.width > 0
            && ctrl.height > 0
        {
            let u = (p.x / d.size.x.max(1e-6) + 0.5).clamp(0.0, 1.0);
            let v = (p.y / d.size.y.max(1e-6) + 0.5).clamp(0.0, 1.0);
            let cx = ((u * ctrl.width.saturating_sub(1) as f32).round() as u32).min(ctrl.width - 1);
            let cy =
                ((v * ctrl.height.saturating_sub(1) as f32).round() as u32).min(ctrl.height - 1);
            let page_bytes = ctrl.width as usize * ctrl.height as usize * 4;
            let px = (cy as usize * ctrl.width as usize + cx as usize) * 4;
            let pages = ctrl.pages() as usize;
            let mut best = 0u32;
            let mut best_w = 0u8;
            for page in 0..pages {
                let base = page * page_bytes + px;
                if base + 4 > ctrl.rgba.len() {
                    break;
                }
                for ch in 0..4usize {
                    let idx = (page * 4 + ch) as u32;
                    let w = ctrl.rgba[base + ch];
                    if w > best_w {
                        best_w = w;
                        best = idx;
                    }
                }
            }
            return best;
        }
        if self.layers.len() <= 1 {
            return 0;
        }
        let mut best = 0u32;
        let mut best_w = -1.0f32;
        for (idx, l) in self.layers.iter().enumerate() {
            let w = band_weight(slope_deg, l.slope[0], l.slope[1], l.slope_fade)
                * band_weight(height, l.altitude[0], l.altitude[1], l.altitude_fade);
            if w > best_w {
                best_w = w;
                best = idx as u32;
            }
        }
        best
    }

    /// Whether the cell under `p` (the terrain's space) is a hole: the
    /// field's hole map, or its level-0 tile's.
    fn hole(&self, p: Vec2) -> bool {
        if let Some(d) = &self.field {
            return d.hole(p);
        }
        if !self.layout.holes || self.cooked.is_none() {
            return false;
        }
        self.tile_sampler()
            .cell_of(p)
            .is_some_and(|(c, r)| self.hole_at(c, r))
    }

    /// The level-0 tile holding field sample (`c`, `r`), decoded once, and
    /// the sample's place in its `(tile + 3)²` samples.
    fn sampled(&self, c: u32, r: u32) -> (SampledTile, usize) {
        let l = self.layout;
        let (t, side) = (l.tile, l.heights_side());
        let (tx, tz) = l.tiles(0);
        let (x, z) = ((c / t).min(tx - 1), (r / t).min(tz - 1));
        let index = l.index(0, x, z);
        let at = ((r - z * t + 1) * side + (c - x * t + 1)) as usize;
        let mut cache = self.tiles.borrow_mut();
        let tile = cache
            .entry(index)
            .or_insert_with(
                || match self.cooked.as_ref().map(|ck| ck.tile_heights(index)) {
                    Some(Ok((h, m))) => (Arc::from(h), m.map(Arc::from)),
                    _ => (Arc::from(vec![0.0; (side * side) as usize]), None),
                },
            )
            .clone();
        (tile, at)
    }

    /// Whether the cell at field sample (`c`, `r`) is a hole, from its
    /// level-0 tile's mask.
    pub(crate) fn hole_at(&self, c: u32, r: u32) -> bool {
        let ((_, mask), at) = self.sampled(c, r);
        mask.is_some_and(|m| m[at] != 0)
    }

    /// The level-0 tiles as a sampler, decoded as they are read.
    fn tile_sampler(&self) -> og_render::TerrainSampler<impl Fn(u32, u32) -> f32 + '_> {
        let l = self.layout;
        og_render::TerrainSampler {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            sample: move |c: u32, r: u32| {
                let ((heights, _), at) = self.sampled(c, r);
                heights[at]
            },
        }
    }
}

/// A level-0 tile decoded for sampling: its heights and hole mask.
pub(crate) type SampledTile = (Arc<[f32]>, Option<Arc<[u8]>>);

fn handle(index: usize) -> u64 {
    crate::world::assets::handle(index)
}

fn index_of(eng: &Engine, data: f64) -> Result<usize, String> {
    crate::world::assets::index_of(data)
        .filter(|&i| eng.render.borrow().terrain.loaded.get(i).is_some())
        .ok_or_else(|| {
            format!("{data} is not a terrain loaded with og.terrain.load (or it was freed with its released bundle)")
        })
}

fn v3(v: &Value) -> Vec3 {
    Vec3::new(f(v, "x", 0.0), f(v, "y", 0.0), f(v, "z", 0.0))
}

/// A terrain's placement from its entity's world position, rotation and
/// scale as game code passes them (a missing or zero quaternion is the
/// identity, a missing scale is 1).
pub(crate) fn placement(origin: &Value, quaternion: &Value, scale: &Value) -> Mat4 {
    let q = Quat::from_xyzw(
        f(quaternion, "x", 0.0),
        f(quaternion, "y", 0.0),
        f(quaternion, "z", 0.0),
        f(quaternion, "w", 0.0),
    );
    let s = if scale.is_object() {
        Vec3::new(f(scale, "x", 1.0), f(scale, "y", 1.0), f(scale, "z", 1.0))
    } else {
        Vec3::ONE
    };
    placed(v3(origin), q, s)
}

/// [`placement`] from read values: a zero quaternion is the identity.
fn placed(origin: Vec3, q: Quat, scale: Vec3) -> Mat4 {
    let q = if q.length_squared() > 1e-12 {
        q.normalize()
    } else {
        Quat::IDENTITY
    };
    Mat4::from_scale_rotation_translation(scale, q, origin)
}

/// Metres across the terrain's surface per metre of its own space (the mean
/// of its x and z scales) and up its y.
fn scales(m: Mat4) -> (f32, f32) {
    let along = (m.x_axis.truncate().length() + m.z_axis.truncate().length()) * 0.5;
    (along.max(1e-6), m.y_axis.truncate().length().max(1e-6))
}

fn heightfield(d: &TerrainData) -> og_physics::HeightfieldDesc {
    og_physics::HeightfieldDesc {
        columns: d.columns,
        rows: d.rows,
        size: d.size,
        heights: d.heights.clone(),
        holes: d.holes.clone(),
    }
}

/// The height field a SHAPE_HEIGHTFIELD collider's `mesh` names (a
/// collision terrain's TerrainInfo.geometry), as physics has it.
pub(crate) fn field_of_geometry(t: &TerrainState, geometry: u64) -> Option<TerrainData> {
    let index = geometry.checked_sub(GEOMETRY_BASE)?;
    t.loaded.get(index as usize)?.field.clone()
}

/// Registers a terrain's height field with physics and remembers it, so
/// a hot reload's new physics gets it too.
fn set_heightfield(eng: &Engine, geometry: u64, d: &TerrainData) -> Result<(), String> {
    let field = heightfield(d);
    let physics = eng.state.borrow().physics.clone();
    if let Some(p) = physics {
        p.set_heightfield(geometry, field.clone())
            .map_err(|e| e.reason().to_string())?;
    }
    eng.state
        .borrow_mut()
        .physics_heightfields
        .insert(geometry, field);
    Ok(())
}

/// [`set_heightfield`] after an edit that changed the heights only inside
/// `lo`..`hi` (the terrain's space).
fn edit_heightfield(
    eng: &Engine,
    geometry: u64,
    d: &TerrainData,
    lo: Vec2,
    hi: Vec2,
) -> Result<(), String> {
    let field = heightfield(d);
    let physics = eng.state.borrow().physics.clone();
    if let Some(p) = physics {
        p.edit_heightfield(geometry, field.clone(), lo, hi)
            .map_err(|e| e.reason().to_string())?;
    }
    eng.state
        .borrow_mut()
        .physics_heightfields
        .insert(geometry, field);
    Ok(())
}

fn info(index: usize, t: &LoadedTerrain) -> TerrainInfo {
    let (lo, hi) = t.range();
    TerrainInfo {
        data: handle(index),
        geometry: GEOMETRY_BASE + index as u64,
        size: t.size(),
        columns: t.layout.columns,
        rows: t.layout.rows,
        min_height: lo,
        max_height: hi,
        layers: t.layers.len() as u32,
        collision: t.collision,
        cast_shadows: t.cast_shadows,
        sea_level: t.sea_level,
        holes: t
            .field
            .as_ref()
            .map_or(t.layout.holes, |f| f.holes.is_some()),
    }
}

/// `og.terrain.load`.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<TerrainInfo, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    // The runtime bundle an asset comes from; None for the boot bundle's.
    let bundle_of = |asset: &str| {
        let a = eng.assets.borrow();
        a.source(asset, None)
            .ok()
            .filter(|&b| a.default_bundle != Some(b))
    };
    let known = eng.render.borrow().terrain.by_id.get(&id).copied();
    if let Some(i) = known {
        // Its bundle released and opened again: the open one owns it now.
        let bundle = bundle_of(&id);
        let mut rs = eng.render.borrow_mut();
        let t = &mut rs.terrain.loaded[i];
        if bundle.is_some() {
            t.bundle = bundle;
        }
        return Ok(info(i, t));
    }
    let mut owned = Vec::new();
    let cooked = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .terrain(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    // The layers' textures, through og.load.texture's cache.
    let mut layers = Vec::with_capacity(cooked.layers.len());
    for l in &cooked.layers {
        let mut layer = l.layer;
        for (name, slot) in [
            (&l.albedo, &mut layer.albedo),
            (&l.normal, &mut layer.normal),
        ] {
            if let Some(n) = name {
                let i = crate::world::assets::load_texture(eng, n)
                    .map_err(|e| format!("{id}: layer texture {e}"))?;
                *slot = eng.render.borrow().textures.get(i).copied();
                let asset = og_bundle::runtime::asset_id(n).to_string();
                owned.push((LayerAsset::Texture(asset.clone()), bundle_of(&asset)));
            }
        }
        layers.push(layer);
    }
    // The layer arrays, through og.load.textureArray's cache.
    let mut layer_arrays = [None; 2];
    for (slot, name) in layer_arrays
        .iter_mut()
        .zip([&cooked.albedo_array, &cooked.normal_array])
    {
        if let Some(n) = name {
            *slot = Some(
                crate::world::assets::load_texture_array_index(eng, n)
                    .map_err(|e| format!("{id}: layer array {e}"))?,
            );
            let asset = og_bundle::runtime::asset_id(n).to_string();
            owned.push((LayerAsset::Array(asset.clone()), bundle_of(&asset)));
        }
    }
    owned.sort();
    owned.dedup();
    let index = eng.render.borrow().terrain.loaded.len();
    // Physics needs the whole field (one height field); nothing else does.
    let field = if cooked.collision {
        let f = cooked.field().map_err(|e| e.to_string())?;
        set_heightfield(eng, GEOMETRY_BASE + index as u64, &f)?;
        Some(f)
    } else {
        None
    };
    let mut rs = eng.render.borrow_mut();
    let render = rs.scene.add_terrain_stream(cooked.layout, cooked.range);
    let layout = cooked.layout;
    let initial_range = cooked.range;
    let sea_level = cooked.sea_level;
    let t = LoadedTerrain {
        render,
        cooked: Some(cooked.clone()),
        initial: None,
        layout,
        initial_range,
        sea_level,
        field,
        edited: false,
        tiles: Default::default(),
        layers,
        layer_arrays,
        height_blend: cooked.height_blend,
        lod: cooked.lod,
        cast_shadows: cooked.cast_shadows,
        collision: cooked.collision,
        generation: next_generation(),
        bundle: bundle_of(&id),
        owned,
    };
    let value = info(index, &t);
    rs.terrain.loaded.push(t);
    rs.terrain.by_id.insert(id, index);
    Ok(value)
}

pub fn og_terrain_load(world: u32, path: String) -> Result<crate::host::wire::TerrainInfo, String> {
    engine(world).and_then(|eng| load(&eng, &path))
}

fn layer_from_desc(v: &TerrainLayerDesc) -> TerrainLayer {
    TerrainLayer {
        albedo: None,
        normal: None,
        albedo_layer: None,
        normal_layer: None,
        color: Vec3::new(v.color.r, v.color.g, v.color.b),
        roughness: v.roughness.clamp(0.04, 1.0),
        tile: v.tile.max(0.01),
        triplanar: v.triplanar,
        normal_strength: v.normal_strength,
        slope: [v.slope_min, v.slope_max],
        slope_fade: v.slope_fade.max(0.0),
        altitude: [v.altitude_min, v.altitude_max],
        altitude_fade: v.altitude_fade.max(0.0),
        project_slope: v.project_slope.clamp(0.0, 90.0),
        anti_tiling: v.anti_tiling.clamp(0.0, 1.0),
        height_contrast: v.height_contrast.clamp(0.0, 1.0),
        macro_variation: v.macro_variation.clamp(0.0, 1.0),
        displacement: v.displacement.clamp(0.0, 4.0),
        cell_variation: v.cell_variation.clamp(0.0, 1.0),
    }
}

/// `og.terrain.create`: creates a live procedural terrain from TypedArrays.
pub fn og_terrain_create(
    world: u32,
    desc: Option<Vec<u8>>,
) -> Result<crate::host::wire::TerrainInfo, String> {
    let eng = engine(world)?;
    let d: TerrainCreateDesc = crate::host::bin::arg(desc, "desc")?;
    if d.columns < 2 || d.rows < 2 || d.columns > 2049 || d.rows > 2049 {
        return Err(format!(
            "desc.columns x desc.rows is {} x {}: each must be 2 to 2049",
            d.columns, d.rows
        ));
    }
    if !(d.size.x.is_finite() && d.size.x > 0.0 && d.size.y.is_finite() && d.size.y > 0.0) {
        return Err("desc.size components must be finite and > 0".into());
    }
    let expected_heights = (d.columns as usize) * (d.rows as usize);
    if d.heights.len() != expected_heights {
        return Err(format!(
            "desc.heights has {} values; {} x {} requires {expected_heights}",
            d.heights.len(),
            d.columns,
            d.rows
        ));
    }
    if d.heights.iter().any(|h| !h.is_finite()) {
        return Err("desc.heights contains a non-finite value".into());
    }
    if d.layers.len() > og_render::MAX_TERRAIN_LAYERS {
        return Err(format!(
            "desc.layers has {} entries; maximum is {}",
            d.layers.len(),
            og_render::MAX_TERRAIN_LAYERS
        ));
    }
    let (control, ctrl_side, pages) = match d.control {
        None => (None, 0u32, 1u32),
        Some(bytes) if bytes.is_empty() => (None, 0u32, 1u32),
        Some(bytes) => {
            let cw = if d.control_width > 0 {
                d.control_width
            } else {
                d.columns
            };
            let ch = if d.control_height > 0 {
                d.control_height
            } else {
                d.rows
            };
            let page_bytes = (cw as usize)
                .checked_mul(ch as usize)
                .and_then(|n| n.checked_mul(4))
                .ok_or("desc.control dimensions overflow")?;
            if page_bytes == 0 || !bytes.len().is_multiple_of(page_bytes) {
                return Err(format!(
                    "desc.control has {} bytes; expected a multiple of {} ({} x {} x 4)",
                    bytes.len(),
                    page_bytes,
                    cw,
                    ch
                ));
            }
            let pages = (bytes.len() / page_bytes) as u32;
            if !(1..=4).contains(&pages) {
                return Err(format!("desc.control has {pages} pages; expected 1 to 4"));
            }
            (
                Some(TerrainControl {
                    width: cw,
                    height: ch,
                    rgba: Arc::from(bytes),
                }),
                cw.max(ch),
                pages,
            )
        }
    };
    let holes = match d.holes {
        None => None,
        Some(bytes) if bytes.is_empty() => None,
        Some(bytes) => {
            let expected_cells = (d.columns as usize - 1) * (d.rows as usize - 1);
            if bytes.len() != expected_cells {
                return Err(format!(
                    "desc.holes has {} bytes; ({} - 1) x ({} - 1) requires {expected_cells}",
                    bytes.len(),
                    d.columns,
                    d.rows
                ));
            }
            Some(Arc::from(bytes))
        }
    };
    let layers: Vec<TerrainLayer> = if d.layers.is_empty() {
        vec![TerrainLayer::default()]
    } else {
        d.layers.iter().map(layer_from_desc).collect()
    };
    let data = TerrainData {
        size: d.size,
        columns: d.columns,
        rows: d.rows,
        heights: Arc::from(d.heights),
        control,
        holes,
    };
    let initial_range = data.range();
    let index = eng.render.borrow().terrain.loaded.len();
    if d.collision {
        set_heightfield(&eng, GEOMETRY_BASE + index as u64, &data)?;
    }
    let layout = TerrainLayout {
        size: d.size,
        columns: d.columns,
        rows: d.rows,
        tile: 64,
        control: ctrl_side,
        levels: 1,
        holes: data.holes.is_some(),
        pages,
    };
    let mut rs = eng.render.borrow_mut();
    let render = rs.scene.add_terrain(data.clone());
    let id = format!("og.terrain.create#{index}");
    let t = LoadedTerrain {
        render,
        cooked: None,
        initial: Some(data.clone()),
        layout,
        initial_range,
        sea_level: d.sea_level.is_finite().then_some(d.sea_level),
        field: Some(data),
        edited: false,
        tiles: Default::default(),
        layers,
        layer_arrays: [None; 2],
        height_blend: if d.height_blend.is_finite() {
            d.height_blend.clamp(0.0, 1.0)
        } else {
            0.15
        },
        lod: TerrainLod::default(),
        cast_shadows: d.cast_shadows,
        collision: d.collision,
        generation: next_generation(),
        bundle: None,
        owned: Vec::new(),
    };
    let value = info(index, &t);
    rs.terrain.loaded.push(t);
    rs.terrain.by_id.insert(id, index);
    Ok(value)
}

/// The point in the terrain's space (relative to its centre) of world `x`,
/// `z` for a terrain of `size` placed by `m` (taken at the terrain's centre
/// height, so along its up axis), and whether it is over the terrain.
fn local(size: Vec2, m: Mat4, x: f32, z: f32) -> (Vec2, bool) {
    let l = m.inverse().transform_point3(Vec3::new(x, m.w_axis.y, z));
    let p = Vec2::new(l.x, l.z);
    let half = size * 0.5;
    (p, p.x.abs() <= half.x && p.y.abs() <= half.y)
}

/// Drop shadows over terrain (`SpriteDesc.dropShadowHeight`): a shadow
/// whose sprite stands where a drawn og.Terrain is higher than its level
/// ground lies on the terrain instead, on the plane fitted to the heights
/// under it (`ShadowGround::fit`), sampled as og.terrain.sample samples
/// them. Where terrains overlap the highest counts. Runs after the frame's
/// sprites and terrains are extracted.
// A guarded entry into the 3D world's engine module (wasm-modules.json).
#[inline(never)]
pub(crate) fn ground_drop_shadows(rs: &mut RenderState) {
    if rs.scene.sprite.sprite_shadows.is_empty() || rs.scene.world.terrains.is_empty() {
        return;
    }
    let loaded = &rs.terrain.loaded;
    let scene = &mut rs.scene;
    let drawn: Vec<(usize, Mat4)> = scene
        .world
        .terrains
        .iter()
        .filter_map(|t| {
            let (i, _) = loaded.iter().find(|(_, l)| l.render == t.terrain)?;
            Some((i, t.transform))
        })
        .collect();
    if drawn.is_empty() {
        return;
    }
    // The highest drawn terrain's height at world x, z; None off them all.
    let height_at = |x: f32, z: f32| -> Option<f32> {
        drawn
            .iter()
            .filter_map(|&(i, m)| {
                let t = &loaded[i];
                let (p, inside) = local(t.size(), m, x, z);
                inside.then(|| {
                    let h = t.with_heights(|s| s(p));
                    m.transform_point3(Vec3::new(p.x, h, p.y)).y
                })
            })
            .reduce(f32::max)
    };
    let sun = scene.lighting.light.direction_to_light;
    for k in 0..scene.sprite.sprite_shadows.len() {
        let shadow = scene.sprite.sprite_shadows[k];
        let Some(sprite) = scene.sprite.sprites.get(shadow.sprite as usize) else {
            continue;
        };
        let Some(h) = height_at(sprite.position.x, sprite.position.z) else {
            continue;
        };
        if h <= shadow.ground.height {
            continue;
        }
        // A billboard's shadow is cast by its sun-facing card.
        let facing = scene
            .sprite
            .sprite_facing
            .get(shadow.sprite as usize)
            .copied()
            .unwrap_or_default();
        let ground = og_render::ShadowGround::fit_facing(
            |x, z| height_at(x, z).unwrap_or(h),
            sprite,
            facing,
            sun,
        );
        scene.sprite.sprite_shadows[k].ground = ground;
    }
}

pub fn og_terrain_sample(world: u32, desc: Option<String>) -> Result<TerrainSample, String> {
    (|| {
        let eng = engine(world)?;
        replay_if_restored(&eng)?;
        let d: TerrainSampleDesc = wire::arg(desc.as_deref(), "desc")?;
        let i = index_of(&eng, d.data as f64)?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        let m = placed(d.origin, d.quaternion, d.scale);
        let (p, inside) = local(t.size(), m, d.x, d.z);
        let (h, n) = t.with_sampler(|s| s(p));
        let n = (Mat3::from_mat4(m).inverse().transpose() * n).normalize_or(Vec3::Y);
        let surface = m.transform_point3(Vec3::new(p.x, h, p.y));
        let slope = og_core::math::acos(n.y.clamp(-1.0, 1.0)).to_degrees();
        let layer = if inside {
            t.dominant_layer(p, h, slope)
        } else {
            0
        };
        Ok(TerrainSample {
            height: surface.y,
            normal: n,
            slope,
            inside,
            hole: inside && t.hole(p),
            layer,
        })
    })()
}

pub fn og_terrain_bounds(
    world: u32,
    data: f64,
    origin: Option<String>,
    quaternion: Option<String>,
    scale: Option<String>,
) -> Result<TerrainBounds, String> {
    (|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let m = placed(
            wire::arg(origin.as_deref(), "origin")?,
            wire::arg(quaternion.as_deref(), "quaternion")?,
            wire::arg::<Option<Vec3>>(scale.as_deref(), "scale")?.unwrap_or(Vec3::ONE),
        );
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        let (lo, hi) = t.range();
        let half = t.size() * 0.5;
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for k in 0..8 {
            let corner = m.transform_point3(Vec3::new(
                if k & 1 == 0 { -half.x } else { half.x },
                if k & 2 == 0 { lo } else { hi },
                if k & 4 == 0 { -half.y } else { half.y },
            ));
            min = min.min(corner);
            max = max.max(corner);
        }
        Ok(TerrainBounds { min, max })
    })()
}
