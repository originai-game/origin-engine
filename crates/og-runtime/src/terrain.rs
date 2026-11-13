//! The IDL's `terrain` module (phase 2): cooked terrains loaded into the
//! renderer and into physics (a height field under a Geometry handle of
//! its own), drawn from og.Terrain components at extraction, sampled and
//! edited from game code. The registry is the render state's: terrains are
//! assets, kept across a hot reload.
//!
//! Phase 3: a terrain is not decoded whole at load. The renderer is handed
//! its tiles as the camera needs them (`super::terrain_stream`); physics
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

use super::engine::{Engine, engine};
use super::extract::{Hierarchy, RenderState};
use super::fx::{f, kept_rows};
use super::parse;
use super::wire::{
    self, Color, FromJson, TerrainBounds, TerrainBrush, TerrainEdit, TerrainInfo, TerrainLayerDesc,
    TerrainSample, TerrainSampleDesc, TerrainStampDesc, ToJson, record,
};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::World;
use og_render::{TerrainData, TerrainLayer, TerrainLod};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod stamp;

mod draw;
mod edit;
mod log;

pub(crate) use draw::*;
pub use edit::*;
pub(crate) use log::*;

/// The Geometry handles of terrains' height fields: a range of their own,
/// beside glTF geometries' (`assets::handle`).
const GEOMETRY_BASE: u64 = 0x7e << 32;

/// One loaded terrain.
pub(crate) struct LoadedTerrain {
    pub render: og_render::TerrainHandle,
    /// The cooked tiles, still compressed.
    pub cooked: og_bundle::runtime::CookedTerrain,
    /// The whole field in memory: physics' (a collider terrain) or an
    /// edited one's. `None`: only the tiles below are decoded.
    pub field: Option<TerrainData>,
    /// Edited: `field` holds the edits and the renderer draws it.
    pub edited: bool,
    /// Level-0 tiles' heights decoded for sampling, by tile index.
    pub tiles: std::cell::RefCell<IdMap<u32, Arc<[f32]>>>,
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
    pub loaded: Vec<LoadedTerrain>,
    pub by_id: BTreeMap<String, usize>,
    pub scatter_issues: BTreeMap<u64, String>,
    pub streamer: super::terrain_stream::Streamer,
    /// Every wanted tile decoded and shown each frame (no streaming).
    pub resident: bool,
    /// Bytes of tiles each terrain keeps on the GPU (0: its lod's).
    pub budget: u64,
    /// The next extraction is a capture's: settle the tiles first.
    pub settle: bool,
    /// The streamed terrains (load indices) a frame drew already: the first
    /// frame that draws one settles, as a capture does.
    pub drawn: BTreeSet<usize>,
}

impl LoadedTerrain {
    /// The field's extent, samples and current height range.
    fn size(&self) -> Vec2 {
        self.cooked.layout.size
    }

    pub(crate) fn range(&self) -> (f32, f32) {
        match (&self.field, self.edited) {
            (Some(f), true) => f.range(),
            _ => self.cooked.range,
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

    /// Whether the cell under `p` (the terrain's space) is a hole: the
    /// field's hole map, or the hole flag its level-0 tile carries.
    fn hole(&self, p: Vec2) -> bool {
        if let Some(d) = &self.field {
            return d.hole(p);
        }
        if !self.cooked.layout.holes {
            return false;
        }
        let sampler = self.tile_sampler();
        sampler
            .cell_of(p)
            .is_some_and(|(c, r)| og_render::hole_bit((sampler.sample)(c, r)))
    }

    /// The level-0 tiles as a sampler, decoded as they are read.
    fn tile_sampler(&self) -> og_render::TerrainSampler<impl Fn(u32, u32) -> f32 + '_> {
        let l = self.cooked.layout;
        let t = l.tile;
        let side = l.heights_side();
        og_render::TerrainSampler {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            sample: move |c: u32, r: u32| {
                let (tx, tz) = l.tiles(0);
                let (x, z) = ((c / t).min(tx - 1), (r / t).min(tz - 1));
                let index = l.index(0, x, z);
                let tile = {
                    let mut cache = self.tiles.borrow_mut();
                    match cache.get(&index) {
                        Some(h) => h.clone(),
                        None => {
                            let h: Arc<[f32]> = self
                                .cooked
                                .tile_heights(index)
                                .map(Arc::from)
                                .unwrap_or_else(|_| Arc::from(vec![0.0; (side * side) as usize]));
                            cache.insert(index, h.clone());
                            h
                        }
                    }
                };
                tile[((r - z * t + 1) * side + (c - x * t + 1)) as usize]
            },
        }
    }
}

fn handle(index: usize) -> u64 {
    super::assets::handle(index)
}

fn index_of(eng: &Engine, data: f64) -> Result<usize, String> {
    super::assets::index_of(data)
        .filter(|&i| i < eng.render.borrow().terrain.loaded.len())
        .ok_or_else(|| format!("{data} is not a terrain loaded with og.terrain.load"))
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
        columns: t.cooked.layout.columns,
        rows: t.cooked.layout.rows,
        min_height: lo,
        max_height: hi,
        layers: t.layers.len() as u32,
        collision: t.collision,
        cast_shadows: t.cast_shadows,
        sea_level: t.cooked.sea_level,
        holes: t
            .field
            .as_ref()
            .map_or(t.cooked.layout.holes, |f| f.holes.is_some()),
    }
}

/// `og.terrain.load`.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<TerrainInfo, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().terrain.by_id.get(&id) {
        return Ok(info(i, &eng.render.borrow().terrain.loaded[i]));
    }
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
                let i = super::assets::load_texture(eng, n)
                    .map_err(|e| format!("{id}: layer texture {e}"))?;
                *slot = eng.render.borrow().textures.get(i).copied();
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
                super::assets::load_texture_array_index(eng, n)
                    .map_err(|e| format!("{id}: layer array {e}"))?,
            );
        }
    }
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
    let t = LoadedTerrain {
        render,
        cooked: cooked.clone(),
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
    };
    let value = info(index, &t);
    rs.terrain.loaded.push(t);
    rs.terrain.by_id.insert(id, index);
    Ok(value)
}

pub fn og_terrain_load(world: u32, path: String) -> Result<crate::wire::TerrainInfo, String> {
    engine(world).and_then(|eng| load(&eng, &path))
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
            let i = loaded.iter().position(|l| l.render == t.terrain)?;
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
        Ok(TerrainSample {
            height: surface.y,
            normal: n,
            slope: og_core::math::acos(n.y.clamp(-1.0, 1.0)).to_degrees(),
            inside,
            hole: inside && t.hole(p),
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
