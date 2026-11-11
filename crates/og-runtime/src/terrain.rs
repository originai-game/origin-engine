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
use super::fx::{color, f, kept_rows, on, u};
use super::wire::{
    self, Color, TerrainBounds, TerrainBrush, TerrainEdit, TerrainInfo, TerrainLayerDesc,
    TerrainSample, TerrainSampleDesc, TerrainStampDesc, ToJson,
};
use super::{parse, refuse};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::World;
use og_render::{TerrainData, TerrainLayer, TerrainLod};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod stamp;

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

/// The edits made since the world began, in order: (terrain asset id, the
/// brush as JSON). `replay`: a restore changed them, and the terrains must
/// be rebuilt from their cook.
#[derive(Default)]
pub(crate) struct EditLog {
    pub edits: Vec<(String, String)>,
    pub replay: bool,
}

pub(crate) type SharedLog = Arc<std::sync::Mutex<EditLog>>;

thread_local! {
    /// Each engine's edit log (its worlds' og.terrain blocks share it).
    static LOGS: std::cell::RefCell<IdMap<u32, SharedLog>> = const { std::cell::RefCell::new(IdMap::new()) };
}

/// Engine `id`'s edit log.
pub(crate) fn log_of(id: u32) -> SharedLog {
    LOGS.with(|l| l.borrow_mut().entry(id).or_default().clone())
}

/// The engine is gone.
pub(crate) fn retire(id: u32) {
    LOGS.with(|l| l.borrow_mut().remove(&id));
}

/// The probe's `terrain_stream:<stream|resident>[:<MiB>]`: `resident`
/// decodes and shows every tile the view wants each frame, `stream` (the
/// default) streams them; the MiB are each terrain's tile budget on the GPU
/// (0 or none: its lod's).
pub(crate) fn set_streaming(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write terrain_stream:<stream|resident>[:<MiB>], such as terrain_stream:resident or terrain_stream:stream:4";
    let resident = match args.first().map(String::as_str) {
        Some("resident") => true,
        Some("stream") => false,
        _ => return Err(USAGE.into()),
    };
    let budget = match args.get(1).filter(|a| !a.is_empty()) {
        None => 0,
        Some(m) => match m.parse::<f64>() {
            Ok(mib) if mib.is_finite() && mib >= 0.0 => (mib * 1048576.0) as u64,
            _ => {
                return Err(format!(
                    "terrain_stream: '{m}' is not a number of MiB; {USAGE}"
                ));
            }
        },
    };
    let eng = engine(engine_id)?;
    let mut rs = eng
        .render
        .try_borrow_mut()
        .map_err(|_| "terrain_stream: the renderer is busy".to_string())?;
    rs.terrain.resident = resident;
    rs.terrain.budget = budget;
    Ok(())
}

/// The terrain streamer for `dump_stream`: its mode, the tile budget asked
/// for (0: each terrain's own), and its last frame's tiles.
pub(crate) fn stream_json(engine_id: u32) -> Value {
    let Ok(eng) = engine(engine_id) else {
        return Value::Null;
    };
    let Ok(rs) = eng.render.try_borrow() else {
        return Value::Null;
    };
    let t = &rs.terrain;
    let s = t.streamer.stats;
    json!({
        "mode": if t.resident { "resident" } else { "stream" },
        "budget": t.budget as f64,
        "terrains": t.loaded.len(),
        "resident": s.resident,
        "waiting": s.waiting,
        "inFlight": s.in_flight,
        "cached": s.cached,
        "cachedBytes": s.cached_bytes as f64,
        "decoded": s.decoded as f64,
        "handed": s.handed,
        "worstFrameMs": (s.worst_frame_ms * 1000.0).round() / 1000.0,
    })
}

/// The edit log as world state.
struct TerrainBlock(SharedLog);

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn take_str(bytes: &[u8], at: &mut usize) -> Option<String> {
    let n = u32::from_le_bytes(bytes.get(*at..*at + 4)?.try_into().ok()?) as usize;
    let s = std::str::from_utf8(bytes.get(*at + 4..*at + 4 + n)?).ok()?;
    *at += 4 + n;
    Some(s.to_string())
}

impl og_ecs::StateBlock for TerrainBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let log = self.0.lock().expect("the edit log");
        out.extend_from_slice(&(log.edits.len() as u32).to_le_bytes());
        for (id, brush) in &log.edits {
            put_str(out, id);
            put_str(out, brush);
        }
    }

    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn std::any::Any + Send>> {
        let bad = || og_core::Error::new(og_core::ErrorKind::Parse, "a damaged terrain edit log");
        let n = u32::from_le_bytes(
            bytes
                .get(..4)
                .ok_or_else(bad)?
                .try_into()
                .map_err(|_| bad())?,
        );
        let mut at = 4;
        let mut edits = Vec::with_capacity(n.min(1 << 16) as usize);
        for _ in 0..n {
            let id = take_str(bytes, &mut at).ok_or_else(bad)?;
            let brush = take_str(bytes, &mut at).ok_or_else(bad)?;
            edits.push((id, brush));
        }
        if at != bytes.len() {
            return Err(bad());
        }
        Ok(Box::new(edits))
    }

    fn apply(&mut self, decoded: Option<Box<dyn std::any::Any + Send>>) {
        let edits = decoded
            .and_then(|d| d.downcast::<Vec<(String, String)>>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        let mut log = self.0.lock().expect("the edit log");
        log.edits = edits;
        log.replay = true;
    }

    /// Each logged brush carries its terrain's pose with its centre, as
    /// they were when it was applied: a replay applies it where it was in
    /// the terrain, whatever the world's zero is now. Left as logged, so a
    /// replay after a shift touches the same samples, bit for bit.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}

const TERRAIN_SYSTEM: &str = "og.terrain";

/// The og.terrain system (front: before physics steps) and the edit log's
/// state block.
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
    let log = log_of(engine_id);
    world
        .add_state_block(TERRAIN_SYSTEM, Box::new(TerrainBlock(log)))
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(og_ecs::SystemDesc::new(TERRAIN_SYSTEM), move |_ctx| {
            if let Ok(eng) = engine(engine_id) {
                replay_if_restored(&eng)
                    .map_err(|e| og_core::Error::new(og_core::ErrorKind::Invalid, e))?;
            }
            Ok(())
        })
        .map_err(|e| e.reason().to_string())
}

/// After a restore: every terrain back as cooked, then the restored edits.
pub(crate) fn replay_if_restored(eng: &Engine) -> Result<(), String> {
    let edits = {
        let shared = log_of(eng.id);
        let mut log = shared.lock().expect("the edit log");
        if !log.replay {
            return Ok(());
        }
        log.replay = false;
        log.edits.clone()
    };
    let count = eng.render.borrow().terrain.loaded.len();
    let mut replayed = std::collections::BTreeMap::new();
    for i in 0..count {
        let (render, layout, range, collision, field) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            if !t.edited {
                continue;
            }
            t.edited = false;
            t.generation = next_generation();
            t.tiles.borrow_mut().clear();
            t.field = if t.collision {
                Some(t.cooked.field().map_err(|e| e.to_string())?)
            } else {
                None
            };
            (
                t.render,
                t.cooked.layout,
                t.cooked.range,
                t.collision,
                t.field.clone(),
            )
        };
        {
            let mut rs = eng.render.borrow_mut();
            rs.scene.set_terrain_stream(render, layout, range);
            rs.terrain.streamer.forget(i);
        }
        if collision {
            replayed.insert(i, field);
        }
    }
    for (id, brush) in edits {
        let Some(&i) = eng.render.borrow().terrain.by_id.get(&id) else {
            continue;
        };
        let b: Value = serde_json::from_str(&brush).map_err(|e| e.to_string())?;
        if stamp::removed(&b) {
            continue;
        }
        edit_loaded(eng, i, &b, false)?;
        if eng.render.borrow().terrain.loaded[i].collision {
            replayed.insert(i, eng.render.borrow().terrain.loaded[i].field.clone());
        }
    }
    // Physics gets each field's heights once, as the edits left them: the
    // snapshot's heights (a restore within the run that edited them) change
    // nothing, so the restored colliders keep their contacts and handles
    // and the world goes on as the uninterrupted run does.
    for (i, field) in replayed {
        if let Some(f) = field {
            set_heightfield(eng, GEOMETRY_BASE + i as u64, &f)?;
        }
    }
    Ok(())
}

/// The streamed terrains' presentation state, for a presentation
/// checkpoint: the streamer's and the tiles the renderer holds of each.
#[derive(Clone, Default)]
pub(crate) struct TerrainCheckpoint {
    stream: super::terrain_stream::StreamCheckpoint,
    held: Vec<(
        og_render::TerrainHandle,
        og_core::IdMap<u32, std::sync::Arc<og_render::TerrainTile>>,
    )>,
}

pub(crate) fn checkpoint(rs: &RenderState) -> TerrainCheckpoint {
    TerrainCheckpoint {
        stream: rs.terrain.streamer.checkpoint(),
        held: rs
            .terrain
            .loaded
            .iter()
            .filter_map(|t| Some((t.render, rs.scene.terrain_tiles(t.render)?.clone())))
            .collect(),
    }
}

pub(crate) fn restore(rs: &mut RenderState, c: &TerrainCheckpoint) {
    rs.terrain.streamer.restore(&c.stream);
    for (handle, tiles) in &c.held {
        let now: Vec<u32> = rs
            .scene
            .terrain_tiles(*handle)
            .map(|t| t.keys().copied().collect())
            .unwrap_or_default();
        for t in now {
            if !tiles.contains_key(&t) {
                rs.scene.set_terrain_tile(*handle, t, None);
            }
        }
        for (&t, tile) in tiles.iter() {
            rs.scene.set_terrain_tile(*handle, t, Some(tile.clone()));
        }
    }
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

fn wire<T: ToJson>(r: Result<T, String>) -> String {
    match r {
        Ok(v) => wire::ok(&v),
        Err(e) => refuse(e),
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

pub fn og_terrain_load(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load(&eng, &path)))
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
    if rs.scene.sprite_shadows.is_empty() || rs.scene.terrains.is_empty() {
        return;
    }
    let loaded = &rs.terrain.loaded;
    let scene = &mut rs.scene;
    let drawn: Vec<(usize, Mat4)> = scene
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
    let sun = scene.light.direction_to_light;
    for k in 0..scene.sprite_shadows.len() {
        let shadow = scene.sprite_shadows[k];
        let Some(sprite) = scene.sprites.get(shadow.sprite as usize) else {
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
        scene.sprite_shadows[k].ground = ground;
    }
}

pub fn og_terrain_sample(world: u32, desc: Option<String>) -> String {
    wire((|| {
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
    })())
}

pub fn og_terrain_bounds(
    world: u32,
    data: f64,
    origin: Option<String>,
    quaternion: Option<String>,
    scale: Option<String>,
) -> String {
    wire((|| {
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
    })())
}

/// The brush's weight at distance `d` from its centre: 1 inside the
/// unfaded part, smoothly to 0 at the radius.
fn weight(d: f32, radius: f32, falloff: f32) -> f32 {
    if d >= radius {
        return 0.0;
    }
    let inner = radius * (1.0 - falloff.clamp(0.0, 1.0));
    if d <= inner {
        return 1.0;
    }
    let t = (d - inner) / (radius - inner).max(1e-6);
    let s = 1.0 - t;
    s * s * (3.0 - 2.0 * s)
}

/// Applies a brush to a terrain's data; answers the samples it touched
/// and the rectangle of the terrain's space it can have changed.
pub(crate) fn apply(t: &mut LoadedTerrain, brush: &Value) -> Result<(u32, Vec2, Vec2), String> {
    let b: TerrainBrush = wire::value(brush, "brush")?;
    // raise, lower, flatten, smooth, paint, hole, fill.
    let mode = b.mode as usize;
    let m = placed(b.origin, b.quaternion, b.scale);
    let (along, up) = scales(m);
    let world_center = b.center;
    let center = m.inverse().transform_point3(world_center);
    let world_radius = b.radius;
    let world_strength = b.strength;
    // In the terrain's space: the radius across it, raise and lower up it.
    let radius = world_radius / along;
    let strength = if mode <= 1 {
        world_strength / up
    } else {
        world_strength
    };
    let falloff = b.falloff;
    if !(world_radius > 0.0 && radius.is_finite() && world_strength >= 0.0 && strength.is_finite())
    {
        return Err("TerrainBrush.radius must be positive and strength at least 0".into());
    }
    if t.field.is_none() {
        t.field = Some(t.cooked.field().map_err(|e| e.to_string())?);
    }
    t.edited = true;
    t.generation = next_generation();
    let d = t.field.as_mut().expect("decoded above");
    let c = Vec2::new(center.x, center.z);
    let half = d.size * 0.5;
    let (lo, hi) = (c - Vec2::splat(radius), c + Vec2::splat(radius));
    if c.x + radius < -half.x
        || c.x - radius > half.x
        || c.y + radius < -half.y
        || c.y - radius > half.y
    {
        return Err(format!(
            "the brush at ({}, {}) with radius {world_radius} misses the terrain ({} × {} m of its own space around ({}, {}))",
            world_center.x, world_center.z, d.size.x, d.size.y, m.w_axis.x, m.w_axis.z
        ));
    }
    if mode == 4 {
        let layer = b.layer;
        if layer as usize >= t.layers.len().max(1)
            || layer as usize >= og_render::MAX_TERRAIN_LAYERS
        {
            return Err(format!(
                "TerrainBrush.layer {layer}: the terrain has {} layer(s)",
                t.layers.len()
            ));
        }
        let n = paint(
            d,
            c,
            radius,
            falloff,
            strength.min(1.0),
            layer as usize,
            t.layers.len().div_ceil(4).max(1),
        );
        return Ok((n, lo, hi));
    }
    if mode >= 5 {
        return Ok((
            cut(d, c, radius, falloff, strength.min(1.0), mode == 5),
            lo,
            hi,
        ));
    }
    let cell = d.cell();
    let (cols, rows) = (d.columns as i64, d.rows as i64);
    let to_grid = |p: Vec2| ((p + half) / cell).to_array();
    let [gx0, gz0] = to_grid(c - Vec2::splat(radius));
    let [gx1, gz1] = to_grid(c + Vec2::splat(radius));
    let (c0, c1) = (
        (gx0.floor() as i64).max(0),
        (gx1.ceil() as i64).min(cols - 1),
    );
    let (r0, r1) = (
        (gz0.floor() as i64).max(0),
        (gz1.ceil() as i64).min(rows - 1),
    );
    // One copy of the field (the scene holds the other), read from the
    // unedited one.
    let old = d.heights.clone();
    let heights = Arc::make_mut(&mut d.heights);
    let mut touched = 0;
    let target = center.y;
    for r in r0..=r1 {
        for col in c0..=c1 {
            let p = Vec2::new(col as f32, r as f32) * cell - half;
            let w = weight((p - c).length(), radius, falloff);
            if w <= 0.0 {
                continue;
            }
            let k = (r * cols + col) as usize;
            let h = old[k];
            heights[k] = match mode {
                0 => h + strength * w,
                1 => h - strength * w,
                2 => h + (target - h) * (strength.min(1.0) * w),
                _ => {
                    let at = |c: i64, r: i64| {
                        old[(r.clamp(0, rows - 1) * cols + c.clamp(0, cols - 1)) as usize]
                    };
                    let mean =
                        (at(col - 1, r) + at(col + 1, r) + at(col, r - 1) + at(col, r + 1)) * 0.25;
                    h + (mean - h) * (strength.min(1.0) * w)
                }
            };
            touched += 1;
        }
    }
    Ok((touched, lo, hi))
}

/// Cuts holes (`hole`) or fills them in under the brush: every cell whose
/// centre the brush weighs at least half (its falloff times `strength`).
/// A terrain left with no hole has no hole map.
fn cut(d: &mut TerrainData, c: Vec2, radius: f32, falloff: f32, strength: f32, hole: bool) -> u32 {
    let (cx, cz) = d.cells();
    let mut holes = d
        .holes
        .as_ref()
        .map_or_else(|| vec![0u8; (cx * cz) as usize], |h| h.to_vec());
    let cell = d.cell();
    let half = d.size * 0.5;
    let mut touched = 0;
    let (c0, c1) = under(c.x - radius, c.x + radius, half.x, cell.x, cx);
    let (r0, r1) = under(c.y - radius, c.y + radius, half.y, cell.y, cz);
    for r in r0..r1 {
        for col in c0..c1 {
            let p = (Vec2::new(col as f32, r as f32) + 0.5) * cell - half;
            if weight((p - c).length(), radius, falloff) * strength < 0.5 {
                continue;
            }
            holes[(r * cx + col) as usize] = u8::from(hole);
            touched += 1;
        }
    }
    d.holes = holes.contains(&1).then(|| holes.into());
    touched
}

/// Paints `layer` into the control map (made all first-layer when the
/// terrain has none): the layer's channel raised towards 1 and the others
/// lowered, keeping their sum. The map first grows to `pages` pages of
/// four layers, a new page all 1 (what a layer past the pages read).
fn paint(
    d: &mut TerrainData,
    c: Vec2,
    radius: f32,
    falloff: f32,
    strength: f32,
    layer: usize,
    pages: usize,
) -> u32 {
    let control = d.control.get_or_insert_with(|| {
        let (w, h) = (d.columns.clamp(2, 1024), d.rows.clamp(2, 1024));
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..w * h {
            rgba.extend_from_slice(&[255, 0, 0, 0]);
        }
        og_render::TerrainControl {
            width: w,
            height: h,
            rgba: rgba.into(),
        }
    });
    let mut rgba = control.rgba.to_vec();
    let page = (control.width * control.height * 4) as usize;
    let pages = pages.max(control.pages() as usize);
    rgba.resize(page * pages, 255);
    let px = d.size / Vec2::new(control.width as f32, control.height as f32);
    let half = d.size * 0.5;
    let mut touched = 0;
    let (x0, x1) = under(c.x - radius, c.x + radius, half.x, px.x, control.width);
    let (y0, y1) = under(c.y - radius, c.y + radius, half.y, px.y, control.height);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) * px - half;
            let w = weight((p - c).length(), radius, falloff) * strength;
            if w <= 0.0 {
                continue;
            }
            let k = ((y * control.width + x) * 4) as usize;
            for ch in 0..4 * pages {
                let at = (ch / 4) * page + k + ch % 4;
                let v = rgba[at] as f32 / 255.0;
                let to = if ch == layer { 1.0 } else { 0.0 };
                rgba[at] = ((v + (to - v) * w) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            touched += 1;
        }
    }
    control.rgba = rgba.into();
    touched
}

/// The cells (of `size` from `-half`, `n` of them) whose centres can lie
/// in `a`..`b`: a range to visit instead of every cell.
fn under(a: f32, b: f32, half: f32, size: f32, n: u32) -> (u32, u32) {
    let at = |v: f32| (((v + half) / size).max(0.0) as u32).min(n);
    (at(a).saturating_sub(1), (at(b) + 2).min(n))
}

/// Loaded terrain `i`'s asset id, as the edit log names it.
fn terrain_id(t: &TerrainState, i: usize) -> String {
    t.by_id
        .iter()
        .find(|(_, k)| **k == i)
        .map(|(id, _)| id.clone())
        .unwrap_or_default()
}

/// Applies a brush or a stamp's patch to loaded terrain `i` (and logs it
/// and hands its heights to physics, unless replaying).
fn edit_loaded(eng: &Engine, i: usize, b: &Value, log: bool) -> Result<Value, String> {
    edit(eng, i, b, log).map(|e| e.to_json())
}

/// [`edit_loaded`]'s answer, typed.
fn edit(eng: &Engine, i: usize, b: &Value, log: bool) -> Result<TerrainEdit, String> {
    {
        let (touched, snapshot, collision) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            let (n, region) = if b.get("stamp").is_some() {
                if t.field.is_none() {
                    t.field = Some(t.cooked.field().map_err(|e| e.to_string())?);
                }
                t.edited = true;
                stamp::apply(t.field.as_mut().expect("decoded above"), b)?
            } else {
                let (n, lo, hi) = apply(t, b)?;
                (n, Some((lo, hi)))
            };
            let snapshot = (t.render, t.field.clone().expect("edited"), region);
            (n, snapshot, t.collision)
        };
        if log {
            let id = terrain_id(&eng.render.borrow().terrain, i);
            log_of(eng.id)
                .lock()
                .expect("the edit log")
                .edits
                .push((id, b.to_string()));
        }
        let (render, data, region) = snapshot;
        {
            let mut rs = eng.render.borrow_mut();
            // In place: the renderer builds again only the tiles the edit
            // reaches.
            match region {
                Some((lo, hi)) => rs.scene.edit_terrain(render, data.clone(), lo, hi),
                None => rs.scene.set_terrain(render, data.clone()),
            };
            rs.terrain.streamer.forget(i);
        }
        // A replay registers the final heights once (`replay_if_restored`).
        if collision && log {
            match region {
                // Physics rebuilds only the chunks of its colliders under it.
                Some((lo, hi)) => edit_heightfield(eng, GEOMETRY_BASE + i as u64, &data, lo, hi)?,
                None => set_heightfield(eng, GEOMETRY_BASE + i as u64, &data)?,
            }
        }
        let (lo, hi) = data.range();
        let stamp = b["stamp"].as_u64().unwrap_or(0) as u32;
        Ok(TerrainEdit {
            samples: touched,
            min_height: lo,
            max_height: hi,
            stamp,
        })
    }
}

pub fn og_terrain_edit(world: u32, data: f64, brush: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        replay_if_restored(&eng)?;
        let i = index_of(&eng, data)?;
        let b = parse(brush.as_deref())?;
        edit(&eng, i, &b, true)
    })())
}

pub fn og_terrain_stamp(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        // Read through the IDL's type to refuse a wrong field; the stamp
        // and its log keep the desc as given.
        wire::value::<TerrainStampDesc>(&d, "desc")?;
        stamp::stamp(&eng, &d)
    })())
}

pub fn og_terrain_unstamp(world: u32, data: f64, stamp: u32) -> String {
    wire((|| stamp::unstamp(&*engine(world)?, data, stamp))())
}

fn layer_desc(l: &TerrainLayer) -> TerrainLayerDesc {
    TerrainLayerDesc {
        color: Color {
            r: l.color.x,
            g: l.color.y,
            b: l.color.z,
        },
        roughness: l.roughness,
        tile: l.tile,
        triplanar: l.triplanar,
        normal_strength: l.normal_strength,
        slope_min: l.slope[0],
        slope_max: l.slope[1],
        slope_fade: l.slope_fade,
        altitude_min: l.altitude[0],
        altitude_max: l.altitude[1],
        altitude_fade: l.altitude_fade,
        project_slope: l.project_slope,
        anti_tiling: l.anti_tiling,
        height_contrast: l.height_contrast,
        macro_variation: l.macro_variation,
        displacement: l.displacement,
        cell_variation: l.cell_variation,
    }
}

pub fn og_terrain_layer(world: u32, data: f64, index: u32) -> String {
    wire((|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        t.layers
            .get(index as usize)
            .map(layer_desc)
            .ok_or_else(|| format!("layer {index}: the terrain has {} layer(s)", t.layers.len()))
    })())
}

pub fn og_terrain_set_layer(world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    match (|| -> Result<(), String> {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let v: TerrainLayerDesc = wire::arg(layer.as_deref(), "layer")?;
        let mut rs = eng.render.borrow_mut();
        let t = &mut rs.terrain.loaded[i];
        let count = t.layers.len();
        let l = t
            .layers
            .get_mut(index as usize)
            .ok_or_else(|| format!("layer {index}: the terrain has {count} layer(s)"))?;
        *l = TerrainLayer {
            albedo: l.albedo,
            normal: l.normal,
            albedo_layer: l.albedo_layer,
            normal_layer: l.normal_layer,
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
        };
        Ok(())
    })() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
}

/// Presentation-only diagnostics of the extracted terrain, from the same
/// clipmap layout the renderer uploads. Never stored in the ECS.
/// The heights the id pass draws this frame's streamed terrains with
/// (`og_render::IdHeights`): each one's level-0 samples, hole bits and
/// all, decoded from its cooked tiles as they are read (the tiles
/// og.terrain.sample reads). A terrain whose field is in memory is read
/// from the scene.
pub(crate) fn id_heights(rs: &RenderState) -> impl Fn(usize, u32, u32) -> Option<f32> + '_ {
    type Sample<'a> = Box<dyn Fn(u32, u32) -> f32 + 'a>;
    let samplers: Vec<Option<Sample>> = rs
        .scene
        .terrains
        .iter()
        .map(|d| {
            let t = rs.terrain.loaded.iter().find(|t| t.render == d.terrain)?;
            Some(Box::new(t.tile_sampler().sample) as Sample)
        })
        .collect();
    move |k, c, r| Some(samplers.get(k)?.as_ref()?(c, r))
}

pub(crate) fn dump_render(rs: &RenderState, entity: u64) -> Option<Value> {
    let i = rs.ids.terrains.iter().position(|&e| e == entity)?;
    let t = rs.scene.terrains.get(i)?;
    let layout = rs.scene.terrain_layout(t.terrain)?;
    let height = match rs.backend.as_ref().map(|b| b.size()) {
        Some((_, ht)) if ht > 0 => ht,
        _ => super::fx::viewport(rs).1 as u32,
    };
    let c = rs
        .scene
        .terrain_clipmap(i, &og_render::TerrainView::of(&rs.scene.camera, height))?;
    let regions: Vec<Value> = c
        .levels
        .iter()
        .enumerate()
        .map(|(level, l)| {
            let spacing = c.unit as f64 * l.step;
            let center = l.center.map(|v| v * c.unit as f64);
            let half = c.ring as f64 * spacing * 0.5;
            json!({
                "level": level, "ring": l.hole.is_some(), "holeOffset": l.hole,
                "localBoundsXZ": [center[0] - half - layout.size.x as f64 * 0.5,
                    center[1] - half - layout.size.y as f64 * 0.5,
                    center[0] + half - layout.size.x as f64 * 0.5,
                    center[1] + half - layout.size.y as f64 * 0.5],
                "triangleSpacingLocal": spacing,
                "triangleSpacingWorldXZ": [t.transform.x_axis.truncate().length() as f64 * spacing,
                    t.transform.z_axis.truncate().length() as f64 * spacing],
                "seam": {"morph": l.morph, "startQuads": l.morph_start,
                    "widthQuads": if l.morph { c.morph_width } else { 0.0 }},
            })
        })
        .collect();
    Some(json!({
        "visible": t.visible, "cell": layout.cell().to_array(), "ringQuads": c.ring,
        "regionsBeforeCulling": regions, "skirts": false,
        "heightBlend": t.height_blend,
        "layers": t.layers.iter().enumerate().map(|(i, l)| json!({
            "index": i, "triplanar": l.triplanar, "tile": l.tile,
            "slope": l.slope, "slopeFade": l.slope_fade,
            "altitude": l.altitude, "altitudeFade": l.altitude_fade,
        })).collect::<Vec<_>>(),
        "projectionWeights": {"view": "terrainProjection", "rgb": ["X", "Y", "Z"],
            "normal": "filtered height-field normal", "exponent": 4},
        "verticalTexelDensity": {"view": "terrainDensity", "rgb": ["X", "Y", "Z"],
            "layer": "dominant raw layer", "units": "base-level albedo texels per vertical screen pixel",
            "encoding": "linear RGB d/(1+d), before projection weighting"},
        "normalMip": {"view": "terrainMip", "measurement": "isotropic derivative estimate",
            "colors": ["green", "yellow", "red", "cyan", "purple", "orange"]},
    }))
}

/// og.Terrain components into `rs.scene.terrains`: each terrain placed by
/// its entity's world transform with its layers; og.TerrainScatter
/// components into `rs.scene.scatters`; then the streamed terrains' tiles.
/// The first frame that draws a streamed terrain settles its tiles as a
/// capture does, so no frame draws a terrain before any of its tiles
/// arrived (web findings #5: Lantern Vale's first frame drew no ground, on
/// one host at tick 0 and on the other at tick 1, and auto exposure
/// started from that frame's metering).
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) -> Vec<(u64, u32)> {
    rs.scene.terrains.clear();
    rs.ids.terrains.clear();
    let layout = rs.draw.terrain.clone();
    let mut settle = std::mem::take(&mut rs.terrain.settle) || rs.terrain.resident;
    let budget = rs.terrain.budget;
    let mut streamed = Vec::new();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Some(i) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0)) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes
                .push(format!("og.Terrain on entity {e} names no loaded terrain"));
            continue;
        };
        let world = h.world_of(e);
        let spacing = f(&v, "lodSpacing", 0.0);
        let ring = u(&v, "lodRing", 0);
        let array = |i: Option<usize>| i.and_then(|i| rs.texture_arrays.get(i)?.handle);
        let layer_arrays = og_render::TerrainLayerArrays {
            albedo: array(t.layer_arrays[0]),
            normal: array(t.layer_arrays[1]),
        };
        rs.scene.terrains.push(og_render::Terrain {
            terrain: t.render,
            transform: world,
            layers: t.layers.clone(),
            layer_arrays,
            height_blend: t.height_blend,
            cast_shadows: on(&v, "castShadows", true) && t.cast_shadows,
            lod: TerrainLod {
                spacing: if spacing > 0.0 {
                    spacing
                } else {
                    t.lod.spacing
                },
                ring: if ring > 0 { ring } else { t.lod.ring },
                budget: if budget > 0 { budget } else { t.lod.budget },
                error: f(&v, "lodError", 0.0).max(0.0),
            },
            fade_in: !settle,
            visible: on(&v, "visible", true),
            horizon_shadows: on(&v, "horizonShadows", true),
            displacement_distance: f(&v, "displacementDistance", 40.0).max(0.0),
        });
        rs.ids.terrains.push(e);
        if !t.edited {
            streamed.push((i, rs.scene.terrains.len() - 1));
        }
    }
    if streamed.iter().any(|(i, _)| !rs.terrain.drawn.contains(i)) {
        settle = true;
        for &(i, k) in &streamed {
            rs.terrain.drawn.insert(i);
            rs.scene.terrains[k].fade_in = false;
        }
    }
    // og.TerrainScatter: copies over the drawn terrain of the same data,
    // placed and drawn by the renderer.
    rs.scene.scatters.clear();
    rs.terrain.scatter_issues.clear();
    let layout = rs.draw.scatter.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Some(i) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0)) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes.push(format!(
                "og.TerrainScatter on entity {e} names no loaded terrain"
            ));
            continue;
        };
        let Some(terrain) = rs.scene.terrains.iter().position(|d| d.terrain == t.render) else {
            continue;
        };
        let index = |key: &str| (v[key].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
        let (Some(&mesh), Some(&material)) = (
            rs.geometries.get(index("geometry")),
            rs.materials.get(index("material")),
        ) else {
            continue;
        };
        let model = v["impostor"].as_f64().unwrap_or(0.0) as u64;
        let lods = rs.model_lods.get(&model);
        let impostor = lods.and_then(|l| l.impostor.map(|i| i.0));
        let mesh = lods
            .and_then(|l| l.scatter_meshes.iter().find(|p| p.0 == mesh))
            .map_or(mesh, |p| p.1);
        if let Some(card) = impostor {
            let Some((min, max)) = rs.scene.mesh(mesh).and_then(|m| m.bounds()) else {
                continue;
            };
            let center = (min + max) * 0.5;
            let radius = ((max - min) * 0.5).length().max(1e-3);
            // Cooked positions are quantized to 16 bits; compare bounds in model space.
            let tolerance = card.radius * 1e-4;
            if center.distance(card.center) > tolerance || (radius - card.radius).abs() > tolerance
            {
                let issue = format!(
                    "geometry bounds (center {center:?}, radius {radius}) differ from impostor bounds (center {:?}, radius {}); use geometry from the same complete model",
                    card.center, card.radius
                );
                rs.notes
                    .push(format!("og.TerrainScatter on entity {e}: {issue}"));
                rs.terrain.scatter_issues.insert(e, issue);
                continue;
            }
        }
        rs.scene.scatters.push(og_render::ScatterDraw {
            terrain,
            mesh,
            material,
            scatter: og_render::TerrainScatter {
                layer: u(&v, "layer", 0),
                density: f(&v, "density", 2.0),
                distance: f(&v, "distance", 40.0),
                fade: f(&v, "fade", 0.3),
                scale: [f(&v, "minScale", 0.8), f(&v, "maxScale", 1.2)],
                align: f(&v, "align", 0.0),
                seed: u(&v, "seed", 1),
                sway: f(&v, "sway", 1.0),
                noise_scale: f(&v, "noiseScale", 8.0),
                noise_threshold: f(&v, "noiseThreshold", 0.0),
                footprint: f(&v, "footprint", 0.0),
                priority: u(&v, "priority", 0),
                avoid_water: f(&v, "avoidWater", 0.0),
                density_map: super::fx::handle_index(&v["densityMap"])
                    .and_then(|i| rs.textures.get(i).copied()),
                gpu: on(&v, "gpuPlacement", true),
            },
            cast_shadows: on(&v, "castShadows", true),
            impostor,
            impostor_distance: f(&v, "impostorDistance", 0.0),
        });
    }
    // og.TerrainStamp: meshes drawn with the drawn terrain of the same
    // data, in its material and blended into its ground.
    rs.scene.terrain_stamps.clear();
    rs.ids.stamps.clear();
    let layout = rs.draw.stamp.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let Some(t) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0))
            .and_then(|i| rs.terrain.loaded.get(i))
        else {
            rs.notes.push(format!(
                "og.TerrainStamp on entity {e} names no loaded terrain"
            ));
            continue;
        };
        let Some(terrain) = rs.scene.terrains.iter().position(|d| d.terrain == t.render) else {
            continue;
        };
        let geometry = (v["geometry"].as_f64().unwrap_or(0.0) as u64 & 0xffff_ffff) as usize;
        let Some(&mesh) = rs.geometries.get(geometry) else {
            continue;
        };
        rs.scene.terrain_stamps.push(og_render::TerrainStampDraw {
            terrain,
            mesh,
            transform: h.world_of(e),
            blend: f(&v, "blend", 1.0),
            cast_shadows: on(&v, "castShadows", true),
        });
        rs.ids.stamps.push(e);
    }
    // og.Grass: fields over the drawn terrain of the same data, generated
    // and drawn by the renderer.
    rs.scene.grass.clear();
    rs.ids.grass.clear();
    let layout = rs.draw.grass.clone();
    // og.GrassType: the kinds of blade fields list in `types`, by entity.
    let kinds: og_core::IdMap<u64, og_render::GrassType> =
        kept_rows(world, rs.draw.grass_type.as_ref())
            .into_iter()
            .map(|(e, v)| {
                let d = og_render::GrassType::default();
                let kind = og_render::GrassType {
                    height: f(&v, "height", d.height),
                    width: f(&v, "width", d.width),
                    curve: f(&v, "curve", d.curve),
                    base_color: color(&v["baseColor"], d.base_color),
                    tip_color: color(&v["tipColor"], d.tip_color),
                    share: f(&v, "share", d.share),
                };
                (e, kind)
            })
            .collect();
    let texture = |v: &Value| super::fx::handle_index(v).and_then(|i| rs.textures.get(i).copied());
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let Some(i) = super::assets::index_of(v["data"].as_f64().unwrap_or(0.0)) else {
            continue;
        };
        let Some(t) = rs.terrain.loaded.get(i) else {
            rs.notes
                .push(format!("og.Grass on entity {e} names no loaded terrain"));
            continue;
        };
        let Some(terrain) = rs.scene.terrains.iter().position(|d| d.terrain == t.render) else {
            continue;
        };
        let d = og_render::Grass::default();
        let listed = v["types"].as_array().map_or(&[][..], Vec::as_slice);
        let mut types = listed
            .iter()
            .filter_map(|t| kinds.get(&(t.as_f64().unwrap_or(0.0) as u64)).copied());
        let types = [types.next(), types.next(), types.next()];
        let (type_map, density_map, height_map) = (
            texture(&v["typeMap"]),
            texture(&v["densityMap"]),
            texture(&v["heightMap"]),
        );
        rs.scene.grass.push(og_render::GrassDraw {
            terrain,
            grass: og_render::Grass {
                layer: u(&v, "layer", d.layer),
                density: f(&v, "density", d.density),
                distance: f(&v, "distance", d.distance),
                far_tint: on(&v, "farTint", d.far_tint),
                height: f(&v, "height", d.height),
                height_variation: f(&v, "heightVariation", d.height_variation),
                width: f(&v, "width", d.width),
                curve: f(&v, "curve", d.curve),
                lean: f(&v, "lean", d.lean),
                clump_size: f(&v, "clumpSize", d.clump_size),
                clump_lean: f(&v, "clumpLean", d.clump_lean),
                tufts: f(&v, "tufts", d.tufts),
                base_color: color(&v["baseColor"], d.base_color),
                tip_color: color(&v["tipColor"], d.tip_color),
                dry_color: color(&v["dryColor"], d.dry_color),
                color_variation: f(&v, "colorVariation", d.color_variation),
                dryness: f(&v, "dryness", d.dryness),
                translucency: f(&v, "translucency", d.translucency),
                sheen: f(&v, "sheen", d.sheen),
                roughness: f(&v, "roughness", d.roughness),
                sway: f(&v, "sway", d.sway),
                stiffness: f(&v, "stiffness", d.stiffness),
                flutter: f(&v, "flutter", d.flutter),
                cast_shadows: on(&v, "castShadows", d.cast_shadows),
                seed: u(&v, "seed", d.seed),
                recovery: f(&v, "recovery", d.recovery),
                types,
                type_map,
                density_map,
                height_map,
            },
        });
        rs.ids.grass.push(e);
    }
    // og.GrassPusher: what presses the grass and scattered copies (the
    // renderer's trample map), placed by their entities' drawn transforms.
    rs.scene.grass_pushers.clear();
    let layout = rs.draw.grass_pusher.clone();
    for (e, v) in kept_rows(world, layout.as_ref()) {
        let m = h.world_of(e);
        rs.scene.grass_pushers.push(og_render::GrassPusher {
            position: m.transform_point3(glam::Vec3::ZERO),
            radius: f(&v, "radius", 0.5),
            height: f(&v, "height", 1.8),
            strength: f(&v, "strength", 1.0),
        });
    }
    // The unedited terrains' tiles, as the camera needs them.
    if !streamed.is_empty() {
        let height = match rs.backend.as_ref().map(|b| b.size()) {
            Some((_, ht)) if ht > 0 => ht,
            _ => super::fx::viewport(rs).1 as u32,
        };
        let view = og_render::TerrainView::of(&rs.scene.camera, height);
        let terrains = rs.scene.terrains.clone();
        let details: Vec<f32> = (0..terrains.len())
            .map(|k| rs.scene.scatter_detail(k))
            .collect();
        let st = &mut rs.terrain;
        let drawn: Vec<super::terrain_stream::Drawn> = streamed
            .iter()
            .map(|&(i, k)| super::terrain_stream::Drawn {
                index: i,
                render: st.loaded[i].render,
                cooked: &st.loaded[i].cooked,
                terrain: &terrains[k],
                detail: details[k],
            })
            .collect();
        let hooks = super::engine::hooks();
        st.streamer.update(
            &drawn,
            &mut rs.scene,
            view,
            settle,
            hooks.as_deref(),
            &mut rs.notes,
        );
    }
    Vec::new()
}

/// The heights of a terrain under a sea (og.Water.seabed, W1): the terrain
/// at load index `index`, placed by `m`, sampled as world heights on a grid
/// over the world xz rectangle `lo` to `hi` (its part over the terrain),
/// at the terrain's own spacing or coarser (at most 1025 samples a side;
/// [`SeabedGrid::heights`]). Off the terrain the seabed is unknown and taken
/// to be far below. With a key for what makes it (the terrain's heights'
/// version, its placement, the rectangle), so the tick and the frame share
/// one field and rebuild it only when one changes.
pub(crate) fn seabed_grid(
    t: &TerrainState,
    index: usize,
    m: Mat4,
    lo: Vec2,
    hi: Vec2,
) -> Option<SeabedGrid> {
    let terrain = t.loaded.get(index)?;
    let size = terrain.size();
    let half = size * 0.5;
    let (along, _) = scales(m);
    // The terrain's footprint in the world, cut to the rectangle.
    let (mut tlo, mut thi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for (x, z) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let w = m.transform_point3(Vec3::new(x * half.x, 0.0, z * half.y));
        tlo = tlo.min(Vec2::new(w.x, w.z));
        thi = thi.max(Vec2::new(w.x, w.z));
    }
    let (lo, hi) = (lo.max(tlo), hi.min(thi));
    if !(hi.x > lo.x && hi.y > lo.y) {
        return None;
    }
    let l = terrain.cooked.layout;
    let spacing = (size.x / (l.columns.max(2) - 1) as f32) * along;
    let extent = (hi - lo).max_element();
    let cell = spacing.max(extent / 1024.0).max(1e-3);
    let columns = (((hi.x - lo.x) / cell).ceil() as u32 + 1).max(2);
    let rows = (((hi.y - lo.y) / cell).ceil() as u32 + 1).max(2);
    let mut key = og_core::hash::StableHasher::new();
    key.write_u64(terrain.generation);
    for v in m.to_cols_array().iter().chain(&[lo.x, lo.y, hi.x, hi.y]) {
        key.write_u32(v.to_bits());
    }
    Some(SeabedGrid {
        origin: lo,
        cell,
        columns,
        rows,
        key: key.finish(),
        index,
        m,
    })
}

impl SeabedGrid {
    /// The world heights of its samples, row-major from `origin`.
    pub(crate) fn heights(&self, t: &TerrainState) -> Vec<f32> {
        let Some(terrain) = t.loaded.get(self.index) else {
            return vec![f32::MIN; (self.columns * self.rows) as usize];
        };
        let half = terrain.size() * 0.5;
        let inverse = self.m.inverse();
        let (deep, _) = terrain.range();
        let (m, lo, cell, columns, rows) =
            (self.m, self.origin, self.cell, self.columns, self.rows);
        terrain.with_heights(|h| {
            (0..rows)
                .flat_map(|r| (0..columns).map(move |c| (c, r)))
                .map(|(c, r)| {
                    let w = lo + Vec2::new(c as f32, r as f32) * cell;
                    let p = inverse.transform_point3(Vec3::new(w.x, 0.0, w.y));
                    let p = Vec2::new(p.x, p.z);
                    let inside = p.x.abs() <= half.x && p.y.abs() <= half.y;
                    let local = if inside { h(p) } else { deep - 1000.0 };
                    m.transform_point3(Vec3::new(p.x, local, p.y)).y
                })
                .collect()
        })
    }
}

/// [`seabed_grid`]'s grid: `columns` × `rows` samples from `origin` (world
/// xz) `cell` metres apart, and the key of what makes its heights.
pub(crate) struct SeabedGrid {
    pub origin: Vec2,
    pub cell: f32,
    pub columns: u32,
    pub rows: u32,
    pub key: u64,
    index: usize,
    m: Mat4,
}
