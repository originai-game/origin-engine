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
use super::fx::{color, f, on, rows, u};
use super::{ok, parse, refuse};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use og_core::IdMap;
use og_ecs::World;
use og_render::{TerrainData, TerrainLayer, TerrainLod};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The Geometry handles of terrains' height fields: a range of their own,
/// beside glTF geometries' (`assets::handle`).
const GEOMETRY_BASE: u64 = 0x7e << 32;

/// One loaded terrain.
pub(crate) struct LoadedTerrain {
    pub render: og_render::TerrainHandle,
    /// The cooked tiles, still compressed.
    pub cooked: og_cook::runtime::CookedTerrain,
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
    for i in 0..count {
        let (render, layout, range, collision, field) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            if !t.edited {
                continue;
            }
            t.edited = false;
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
        if let (true, Some(f)) = (collision, field) {
            set_heightfield(eng, GEOMETRY_BASE + i as u64, &f)?;
        }
    }
    for (id, brush) in edits {
        let Some(&i) = eng.render.borrow().terrain.by_id.get(&id) else {
            continue;
        };
        let b: Value = serde_json::from_str(&brush).map_err(|e| e.to_string())?;
        edit_loaded(eng, i, &b, false)?;
    }
    Ok(())
}

/// Every terrain an engine loaded, by load index, and the og.Terrain
/// layout extraction reads.
#[derive(Default)]
pub(crate) struct TerrainState {
    pub loaded: Vec<LoadedTerrain>,
    pub by_id: BTreeMap<String, usize>,
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

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
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

fn j3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
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
    let q = if q.length_squared() > 1e-12 {
        q.normalize()
    } else {
        Quat::IDENTITY
    };
    let s = if scale.is_object() {
        Vec3::new(f(scale, "x", 1.0), f(scale, "y", 1.0), f(scale, "z", 1.0))
    } else {
        Vec3::ONE
    };
    Mat4::from_scale_rotation_translation(s, q, v3(origin))
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

fn info(index: usize, t: &LoadedTerrain) -> Value {
    let (lo, hi) = t.range();
    let size = t.size();
    json!({
        "data": handle(index),
        "geometry": GEOMETRY_BASE + index as u64,
        "size": { "x": size.x, "y": size.y },
        "columns": t.cooked.layout.columns,
        "rows": t.cooked.layout.rows,
        "minHeight": lo,
        "maxHeight": hi,
        "layers": t.layers.len(),
        "collision": t.collision,
        "castShadows": t.cast_shadows,
        "seaLevel": t.cooked.sea_level,
    })
}

/// `og.terrain.load`.
pub(crate) fn load(eng: &Engine, path: &str) -> Result<Value, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
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
        let d = parse(desc.as_deref())?;
        let i = index_of(&eng, d["data"].as_f64().unwrap_or(0.0))?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        let m = placement(&d["origin"], &d["quaternion"], &d["scale"]);
        let (p, inside) = local(t.size(), m, f(&d, "x", 0.0), f(&d, "z", 0.0));
        let (h, n) = t.with_sampler(|s| s(p));
        let n = (Mat3::from_mat4(m).inverse().transpose() * n).normalize_or(Vec3::Y);
        let surface = m.transform_point3(Vec3::new(p.x, h, p.y));
        Ok(json!({
            "height": surface.y,
            "normal": j3(n),
            "slope": n.y.clamp(-1.0, 1.0).acos().to_degrees(),
            "inside": inside,
        }))
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
        let m = placement(
            &parse(origin.as_deref())?,
            &parse(quaternion.as_deref())?,
            &parse(scale.as_deref())?,
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
        Ok(json!({ "min": j3(min), "max": j3(max) }))
    })())
}

const MODES: [&str; 5] = ["raise", "lower", "flatten", "smooth", "paint"];

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

/// Applies a brush to a terrain's data; answers the samples it touched.
pub(crate) fn apply(t: &mut LoadedTerrain, brush: &Value) -> Result<u32, String> {
    let mode = match &brush["mode"] {
        Value::String(s) => MODES
            .iter()
            .position(|m| m == s)
            .ok_or_else(|| format!("TerrainBrush.mode '{s}' is not one of {}", MODES.join(", ")))?,
        Value::Null => 0,
        other => return Err(format!("TerrainBrush.mode {other} is not a mode")),
    };
    let m = placement(&brush["origin"], &brush["quaternion"], &brush["scale"]);
    let (along, up) = scales(m);
    let world_center = v3(&brush["center"]);
    let center = m.inverse().transform_point3(world_center);
    let world_radius = f(brush, "radius", 8.0);
    let world_strength = f(brush, "strength", 1.0);
    // In the terrain's space: the radius across it, raise and lower up it.
    let radius = world_radius / along;
    let strength = if mode <= 1 {
        world_strength / up
    } else {
        world_strength
    };
    let falloff = f(brush, "falloff", 0.5);
    if !(world_radius > 0.0 && radius.is_finite() && world_strength >= 0.0 && strength.is_finite())
    {
        return Err("TerrainBrush.radius must be positive and strength at least 0".into());
    }
    if t.field.is_none() {
        t.field = Some(t.cooked.field().map_err(|e| e.to_string())?);
    }
    t.edited = true;
    let d = t.field.as_mut().expect("decoded above");
    let c = Vec2::new(center.x, center.z);
    let half = d.size * 0.5;
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
        let layer = u(brush, "layer", 0);
        if layer as usize >= t.layers.len().max(1) || layer > 3 {
            return Err(format!(
                "TerrainBrush.layer {layer}: the terrain has {} layer(s)",
                t.layers.len()
            ));
        }
        return Ok(paint(
            d,
            c,
            radius,
            falloff,
            strength.min(1.0),
            layer as usize,
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
    let old: Vec<f32> = d.heights.to_vec();
    let mut heights = old.clone();
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
    d.heights = Arc::from(heights);
    Ok(touched)
}

/// Paints `layer` into the control map (made all first-layer when the
/// terrain has none): the layer's channel raised towards 1 and the others
/// lowered, keeping their sum.
fn paint(
    d: &mut TerrainData,
    c: Vec2,
    radius: f32,
    falloff: f32,
    strength: f32,
    layer: usize,
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
    let px = d.size / Vec2::new(control.width as f32, control.height as f32);
    let half = d.size * 0.5;
    let mut touched = 0;
    for y in 0..control.height {
        for x in 0..control.width {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) * px - half;
            let w = weight((p - c).length(), radius, falloff) * strength;
            if w <= 0.0 {
                continue;
            }
            let k = ((y * control.width + x) * 4) as usize;
            for ch in 0..4 {
                let v = rgba[k + ch] as f32 / 255.0;
                let to = if ch == layer { 1.0 } else { 0.0 };
                rgba[k + ch] = ((v + (to - v) * w) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            touched += 1;
        }
    }
    control.rgba = rgba.into();
    touched
}

/// Applies a brush to loaded terrain `i` (and logs it, unless replaying).
fn edit_loaded(eng: &Engine, i: usize, b: &Value, log: bool) -> Result<Value, String> {
    {
        let (touched, snapshot, collision) = {
            let mut rs = eng.render.borrow_mut();
            let t = &mut rs.terrain.loaded[i];
            let n = apply(t, b)?;
            let snapshot = (t.render, t.field.clone().expect("edited"));
            (n, snapshot, t.collision)
        };
        if log {
            let rs = eng.render.borrow();
            let id = rs
                .terrain
                .by_id
                .iter()
                .find(|(_, k)| **k == i)
                .map(|(id, _)| id.clone())
                .unwrap_or_default();
            log_of(eng.id)
                .lock()
                .expect("the edit log")
                .edits
                .push((id, b.to_string()));
        }
        let (render, data) = snapshot;
        {
            let mut rs = eng.render.borrow_mut();
            rs.scene.set_terrain(render, data.clone());
            rs.terrain.streamer.forget(i);
        }
        if collision {
            set_heightfield(eng, GEOMETRY_BASE + i as u64, &data)?;
        }
        let (lo, hi) = data.range();
        Ok(json!({ "samples": touched, "minHeight": lo, "maxHeight": hi }))
    }
}

pub fn og_terrain_edit(world: u32, data: f64, brush: Option<String>) -> String {
    wire((|| {
        let eng = engine(world)?;
        replay_if_restored(&eng)?;
        let i = index_of(&eng, data)?;
        let b = parse(brush.as_deref())?;
        edit_loaded(&eng, i, &b, true)
    })())
}

fn layer_json(l: &TerrainLayer) -> Value {
    json!({
        "color": { "r": l.color.x, "g": l.color.y, "b": l.color.z },
        "roughness": l.roughness,
        "tile": l.tile,
        "triplanar": l.triplanar,
        "normalStrength": l.normal_strength,
        "slopeMin": l.slope[0],
        "slopeMax": l.slope[1],
        "slopeFade": l.slope_fade,
        "altitudeMin": l.altitude[0],
        "altitudeMax": l.altitude[1],
        "altitudeFade": l.altitude_fade,
    })
}

pub fn og_terrain_layer(world: u32, data: f64, index: u32) -> String {
    wire((|| {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let rs = eng.render.borrow();
        let t = &rs.terrain.loaded[i];
        t.layers
            .get(index as usize)
            .map(layer_json)
            .ok_or_else(|| format!("layer {index}: the terrain has {} layer(s)", t.layers.len()))
    })())
}

pub fn og_terrain_set_layer(world: u32, data: f64, index: u32, layer: Option<String>) -> String {
    match (|| -> Result<(), String> {
        let eng = engine(world)?;
        let i = index_of(&eng, data)?;
        let v = parse(layer.as_deref())?;
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
            color: color(&v["color"], Vec3::ONE),
            roughness: f(&v, "roughness", 0.9).clamp(0.04, 1.0),
            tile: f(&v, "tile", 4.0).max(0.01),
            triplanar: on(&v, "triplanar", false),
            normal_strength: f(&v, "normalStrength", 1.0),
            slope: [f(&v, "slopeMin", 0.0), f(&v, "slopeMax", 90.0)],
            slope_fade: f(&v, "slopeFade", 4.0).max(0.0),
            altitude: [f(&v, "altitudeMin", -1.0e6), f(&v, "altitudeMax", 1.0e6)],
            altitude_fade: f(&v, "altitudeFade", 4.0).max(0.0),
        };
        Ok(())
    })() {
        Ok(()) => super::ok_void(),
        Err(e) => refuse(e),
    }
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
    let layout = rs.draw.terrain.clone();
    let mut settle = std::mem::take(&mut rs.terrain.settle) || rs.terrain.resident;
    let budget = rs.terrain.budget;
    let mut streamed = Vec::new();
    for (e, v) in rows(world, layout.as_ref()) {
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
            },
            fade_in: !settle,
            visible: on(&v, "visible", true),
        });
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
    let layout = rs.draw.scatter.clone();
    for (e, v) in rows(world, layout.as_ref()) {
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
            },
            cast_shadows: on(&v, "castShadows", true),
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
