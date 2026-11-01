//! The depth S2 modules on the runtime (docs/design.md §2.7, "Timeline and
//! paths"): `og.path` (paths, og.PathFollower, the og.CameraRail path) and
//! `og.timeline` (timeline assets, og.Timeline, its events and the
//! command-channel actions `timeline:<play|seek|pause|resume|stop>`), over
//! og-timeline.
//!
//! Systems, in tick order after the game's systems and og.tween, before
//! og.anim and physics: `og.path` (followers), then `og.timeline` (every
//! og.Timeline: its time, then each track evaluated at that time). The
//! timelines' playback state (the exact time, the camera they found, the
//! voices and clips they started) is the `og.timeline` state block, so
//! snapshots, restore and the world hash carry it.
//!
//! **Exact time.** A timeline's time is kept as a whole number of steps of
//! a thousandth of a tick (`num / (rate * 1000)` seconds); a tick at speed
//! s adds `round(s * 1000)` steps and a seek to t sets `round(t * rate *
//! 1000)`. Playing 240 ticks from 0 at 60 Hz and seeking to 4 s land on the
//! same number, the same `time` field and the same evaluation: keys are
//! evaluated at that time, never accumulated.
//!
//! Paths and loaded timelines are assets, not world state (as og-anim's
//! clips): they live per engine here and a hot reload keeps them.

mod evaluate;
mod playback;

use super::{assets, feel, modules};
use evaluate::evaluate;
use playback::{Op, Rec, Run, shared};
pub(crate) use playback::{install, register_records};
pub use playback::{og_timeline_pause, og_timeline_play, og_timeline_seek, og_timeline_stop};

use super::codec::{Layout, RecordKind};
use super::engine::{Engine, engine};
use super::{ok, ok_void, parse, refuse};
use glam::{Mat3, Quat, Vec3};
use og_core::IdMap;
use og_core::{Error, ErrorKind, Reflect, Str64};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_timeline::eval::{self, crossed};
use og_timeline::{CameraMode, Path, PathDesc, PathKind, Timeline, Track, TransitionKind};
use serde_json::{Value, json};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const PATH_SYSTEM: &str = "og.path";
pub(crate) const TIMELINE_SYSTEM: &str = "og.timeline";

/// The timeline events, registered from their Rust types (the marker's
/// name is a str64), not from the IDL's declarations.
pub(crate) const REGISTERED_BY_TIMELINE: &[&str] = &[
    "og.TimelineStarted",
    "og.TimelineMarker",
    "og.TimelineEnded",
];

/// Path handles: this tag, then the index plus one.
const PATH_TAG: u64 = 2 << 32;
/// Timeline handles: this tag, then the index plus one.
const TIMELINE_TAG: u64 = 3 << 32;
/// Steps of a timeline's time per tick at speed 1.
const STEPS: i64 = 1000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TimelineStarted")]
pub struct TimelineStarted {
    pub entity: u64,
    pub asset: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TimelineMarker")]
pub struct TimelineMarker {
    pub entity: u64,
    pub asset: u64,
    pub name: Str64,
    pub time: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.TimelineEnded")]
pub struct TimelineEnded {
    pub entity: u64,
    pub asset: u64,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

fn v3(v: &Value) -> Vec3 {
    let n = |k: &str| v[k].as_f64().unwrap_or(0.0) as f32;
    Vec3::new(n("x"), n("y"), n("z"))
}

fn jv3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn jq(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
}

fn quat(v: &Value) -> Quat {
    let n = |k: &str, d: f64| v[k].as_f64().unwrap_or(d) as f32;
    Quat::from_xyzw(n("x", 0.0), n("y", 0.0), n("z", 0.0), n("w", 1.0))
}

fn bits(v: &Value) -> u64 {
    v.as_f64().map_or(0, |x| x as u64)
}

// ── the registry: paths and loaded timelines, per engine ────────────────────

/// A timeline loaded in an engine: the asset, its own paths, its sounds
/// (handle and seconds) and, per animation track, the clips it resolved.
pub(crate) struct Loaded {
    pub id: String,
    /// The cooked output's hash it was read from (a hot reload reads it
    /// again when the manifest's differs).
    pub hash: u64,
    pub timeline: Timeline,
    pub paths: BTreeMap<String, Arc<Path>>,
    pub sounds: BTreeMap<String, (u64, f32)>,
    /// Per animation track: (skeleton, clip handles by name).
    pub clips: IdMap<usize, (u64, BTreeMap<String, u64>)>,
}

#[derive(Default)]
pub(crate) struct Registry {
    paths: Vec<Arc<Path>>,
    /// Scene paths by name.
    named: BTreeMap<String, u64>,
    timelines: Vec<Arc<Mutex<Loaded>>>,
    by_id: BTreeMap<String, u64>,
    /// Where event keys' actions go (the probe's inbox).
    inbox: Option<og_probe::Inbox>,
    /// Warnings already logged (each once).
    warned: std::collections::BTreeSet<String>,
}

thread_local! {
    static REGISTRIES: RefCell<IdMap<u32, Arc<Mutex<Registry>>>> = const { RefCell::new(IdMap::new()) };
}

fn registry(engine: u32) -> Arc<Mutex<Registry>> {
    REGISTRIES.with(|r| r.borrow_mut().entry(engine).or_default().clone())
}

impl Registry {
    fn path(&self, h: u64) -> Option<Arc<Path>> {
        if h & !0xffff_ffff != PATH_TAG {
            return None;
        }
        let i = (h & 0xffff_ffff) as usize;
        self.paths.get(i.checked_sub(1)?).cloned()
    }

    fn add_path(&mut self, p: Path) -> u64 {
        self.paths.push(Arc::new(p));
        PATH_TAG | self.paths.len() as u64
    }

    fn timeline(&self, h: u64) -> Option<Arc<Mutex<Loaded>>> {
        if h & !0xffff_ffff != TIMELINE_TAG {
            return None;
        }
        let i = (h & 0xffff_ffff) as usize;
        self.timelines.get(i.checked_sub(1)?).cloned()
    }
}

/// The path of a handle in an engine (og.CameraRail.path, og.PathFollower).
pub(crate) fn path_of(engine: u32, h: u64) -> Option<Arc<Path>> {
    lock(&registry(engine)).path(h)
}

/// The engine's probe inbox, for event keys (the engine was assembled).
pub(crate) fn commit(engine: u32, inbox: og_probe::Inbox) {
    lock(&registry(engine)).inbox = Some(inbox);
}

pub(crate) fn retire(engine: u32) {
    REGISTRIES.with(|r| r.borrow_mut().remove(&engine));
}

/// A spawned scene's named paths (a later scene's path of the same name
/// replaces it).
pub(crate) fn register_scene_paths(engine: u32, paths: &[(String, PathDesc)]) {
    if paths.is_empty() {
        return;
    }
    let reg = registry(engine);
    let mut r = lock(&reg);
    for (name, desc) in paths {
        if let Ok(p) = Path::new(desc.clone()) {
            let h = r.add_path(p);
            r.named.insert(name.clone(), h);
        }
    }
}

fn note(engine: u32, text: String) {
    let reg = registry(engine);
    if !lock(&reg).warned.insert(text.clone()) {
        return;
    }
    if let Ok(eng) = super::engine::engine(engine)
        && let Ok(mut st) = eng.state.try_borrow_mut()
    {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
    }
}

// ── og.path ────────────────────────────────────────────────────────────────

fn curve_desc(v: &Value) -> Result<PathDesc, String> {
    let mut d = PathDesc::default();
    let pts = v["points"]
        .as_array()
        .ok_or("desc.points is the list of points ({x, y, z})")?;
    d.points = pts.iter().map(v3).collect();
    d.closed = v["closed"].as_bool().unwrap_or(false);
    if let Some(k) = v["kind"].as_str() {
        d.kind = PathKind::from_name(k)
            .ok_or_else(|| format!("desc.kind '{k}' is not catmullRom, bezier or linear"))?;
    } else if let Some(k) = v["kind"].as_u64() {
        d.kind = PathKind::from_code(k as u32).ok_or("desc.kind is 0 to 2")?;
    }
    if let Some(t) = v["tension"].as_f64() {
        d.tension = t as f32;
    }
    Ok(d)
}

pub fn og_path_create(world: u32, desc: Option<String>) -> String {
    wire((|| {
        engine(world)?;
        let v = parse(desc.as_deref())?;
        let p = Path::new(curve_desc(&v)?)?;
        Ok(json!(lock(&registry(world)).add_path(p) as f64))
    })())
}

pub fn og_path_find(world: u32, name: String) -> String {
    wire((|| {
        engine(world)?;
        let reg = registry(world);
        let r = lock(&reg);
        r.named.get(&name).map(|h| json!(*h as f64)).ok_or_else(|| {
            let names: Vec<&str> = r.named.keys().map(String::as_str).collect();
            if names.is_empty() {
                format!("no spawned scene declares path '{name}' (none declares any: add `path \"{name}\" {{ point x y z ... }}` to a scene)")
            } else {
                format!(
                    "no spawned scene declares path '{name}'; the scenes' paths are {}",
                    names.join(", ")
                )
            }
        })
    })())
}

fn with_path<T>(world: u32, path: f64, f: impl FnOnce(&Path) -> T) -> Result<T, String> {
    engine(world)?;
    let p = path_of(world, path as u64).ok_or_else(|| {
        format!("path {path} is not a live path (og.path.create or og.path.find)")
    })?;
    Ok(f(&p))
}

fn sample_json(s: og_timeline::PathSample) -> Value {
    json!({ "position": jv3(s.position), "tangent": jv3(s.tangent), "up": jv3(s.up) })
}

pub fn og_path_sample(world: u32, path: f64, t: f64) -> String {
    wire(with_path(world, path, |p| sample_json(p.sample(t as f32))))
}

pub fn og_path_length(world: u32, path: f64) -> String {
    wire(with_path(world, path, |p| json!(p.length())))
}

pub fn og_path_nearest(world: u32, path: f64, point: String) -> String {
    wire(parse(Some(&point)).and_then(|v| {
        let q = v3(&v);
        with_path(world, path, |p| {
            let n = p.nearest(q);
            json!({ "t": n.t, "position": jv3(n.position), "distance": n.distance })
        })
    }))
}

/// The frame of a path's point: -Z along the tangent, +Y its up.
fn facing(s: &og_timeline::PathSample) -> (Mat3, Quat) {
    let z = -s.tangent;
    let x = s.up.cross(z).normalize_or(Vec3::X);
    let y = z.cross(x);
    let m = Mat3::from_cols(x, y, z);
    (m, Quat::from_mat3(&m).normalize())
}

// ── og.load.timeline and og.timeline.info ───────────────────────────────────

fn load(eng: &Engine, path: &str) -> Result<u64, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    let reg = registry(eng.id);
    if let Some(&h) = lock(&reg).by_id.get(&id) {
        return Ok(h);
    }
    let loaded = decode(eng, &id)?;
    let mut r = lock(&reg);
    r.timelines.push(Arc::new(Mutex::new(loaded)));
    let h = TIMELINE_TAG | r.timelines.len() as u64;
    r.by_id.insert(id, h);
    Ok(h)
}

/// The cooked output hash of `id` in the open bundles (0: not found).
fn output_hash(eng: &Engine, id: &str) -> u64 {
    let a = eng.assets.borrow();
    a.bundle_for(id)
        .ok()
        .and_then(|(b, _)| b.output(id).ok().map(|o| o.hash))
        .unwrap_or(0)
}

/// Timeline `id` read from its cooked blob, with its paths, sounds and clips.
fn decode(eng: &Engine, id: &str) -> Result<Loaded, String> {
    let hash = output_hash(eng, id);
    let timeline = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle
            .timeline(id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut paths = BTreeMap::new();
    for (name, desc) in &timeline.paths {
        paths.insert(name.clone(), Arc::new(Path::new(desc.clone())?));
    }
    let mut sounds = BTreeMap::new();
    let mut clips = IdMap::new();
    for (k, tr) in timeline.tracks.iter().enumerate() {
        let names: Vec<&str> = match tr {
            Track::Audio(ks) => ks.iter().map(|k| k.sound.as_str()).collect(),
            Track::Music(ks) => ks.iter().map(|k| k.sound.as_str()).collect(),
            Track::Animation(a) if !a.model.is_empty() => {
                clips.insert(k, clip_set(eng, &a.model)?);
                Vec::new()
            }
            _ => Vec::new(),
        };
        for s in names {
            if sounds.contains_key(s) {
                continue;
            }
            let i =
                super::assets::load_sound(eng, s).map_err(|e| format!("{id}: sound {s}: {e}"))?;
            let a = eng.assets.borrow();
            let m = &a.sounds[i].meta;
            let seconds = (m.frames as f64 / f64::from(m.sample_rate.max(1))) as f32;
            sounds.insert(s.to_string(), (a.sound_handles[i], seconds));
        }
    }
    Ok(Loaded {
        id: id.to_string(),
        hash,
        timeline,
        paths,
        sounds,
        clips,
    })
}

/// After a hot reload (`og dev` cooks an edit before it reloads; the
/// bundles' manifests were read again): every loaded timeline whose cooked
/// output changed is read again into its own handle, so an og.Timeline
/// playing it goes on from its time (its state block untouched) with the
/// edited tracks. Answers why any failed (that one keeps its old tracks).
pub(crate) fn refresh(eng: &Engine) -> Vec<String> {
    let loaded: Vec<Arc<Mutex<Loaded>>> = lock(&registry(eng.id)).timelines.clone();
    let mut failed = Vec::new();
    for t in loaded {
        let (id, hash) = {
            let l = lock(&t);
            (l.id.clone(), l.hash)
        };
        let now = output_hash(eng, &id);
        if now == 0 || now == hash {
            continue;
        }
        match decode(eng, &id) {
            Ok(fresh) => *lock(&t) = fresh,
            Err(e) => failed.push(format!("timeline {id}: {e}")),
        }
    }
    failed
}

/// A model's skeleton and clips by name (og.load.animation of it).
fn clip_set(eng: &Engine, model: &str) -> Result<(u64, BTreeMap<String, u64>), String> {
    let v = super::anim::load_animation(eng, model)?;
    let clips = og_core::sort::map(
        v["clips"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| Some((c["name"].as_str()?.to_string(), bits(&c["clip"])))),
    );
    Ok((bits(&v["skeleton"]), clips))
}

pub fn og_load_timeline(world: u32, path: String) -> String {
    wire(engine(world).and_then(|eng| load(&eng, &path).map(|h| json!(h as f64))))
}

pub fn og_timeline_info(world: u32, asset: f64) -> String {
    wire((|| {
        engine(world)?;
        let t = lock(&registry(world))
            .timeline(asset as u64)
            .ok_or_else(|| {
                format!("{asset} is not a timeline loaded in this world (og.load.timeline)")
            })?;
        let l = lock(&t);
        let tl = &l.timeline;
        let mut markers: Vec<(f32, String)> = Vec::new();
        for tr in &tl.tracks {
            if let Track::Marker(ks) = tr {
                markers.extend(ks.iter().map(|k| (k.time, k.name.clone())));
            }
        }
        og_core::sort::sort_by(&mut markers, |a, b| a.0.total_cmp(&b.0));
        Ok(json!({
            "asset": asset,
            "name": tl.name,
            "duration": tl.duration,
            "tracks": tl.tracks.iter().map(|t| json!({ "kind": t.kind(), "target": t.target(), "keys": t.key_count() })).collect::<Vec<_>>(),
            "paths": tl.paths.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
            "markers": markers.into_iter().map(|m| m.1).collect::<Vec<_>>(),
        }))
    })())
}

// ── the command channel ──────────────────────────────────────────────────────

/// `timeline:<play|seek|pause|resume|stop>[...]`.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "timeline",
        og_probe::CommandDesc {
            usage: "timeline:play:<name> | timeline:seek:<seconds>[:<name>] | timeline:pause[:<name>] | timeline:resume[:<name>] | timeline:stop[:<name>]".into(),
            doc: "play a timeline (`timelines/<name>.ogt`, or an asset id) on an entity named `timeline-<name>` (made when missing), or seek, pause, resume or stop it (every og.Timeline without a name); it takes effect at this tick's og.timeline".into(),
        },
        move |world, args| command(engine_id, world, args),
    );
}

fn asset_path(name: &str) -> String {
    if name.contains('/') || name.ends_with(".ogt") {
        name.to_string()
    } else {
        format!("timelines/{name}.ogt")
    }
}

fn short(name: &str) -> String {
    let base = name.rsplit('/').next().unwrap_or(name);
    base.trim_end_matches(".ogt").to_string()
}

fn command(engine_id: u32, world: &mut World, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write timeline:play:<name>, timeline:seek:<seconds>[:<name>], timeline:pause[:<name>], timeline:resume[:<name>] or timeline:stop[:<name>]";
    let verb = args.first().map(String::as_str).unwrap_or("");
    let eng = engine(engine_id)?;
    let layout = eng
        .state
        .borrow()
        .layouts
        .get("og.Timeline")
        .cloned()
        .ok_or("og.Timeline is not registered")?;
    let tid = world
        .id_by_name("og.Timeline")
        .map_err(|e| e.reason().to_string())?;
    // Every entity with og.Timeline, or the one of a named timeline.
    let targets = |world: &World, name: Option<&str>| -> Result<Vec<Entity>, String> {
        let want = match name {
            Some(n) => Some(
                *lock(&registry(engine_id))
                    .by_id
                    .get(og_cook::runtime::asset_id(&asset_path(n)))
                    .ok_or_else(|| {
                        format!("timeline '{n}' is not loaded (timeline:play:{n} loads it)")
                    })?,
            ),
            None => None,
        };
        let mut out = Vec::new();
        for e in world.entities() {
            if let Ok(b) = world.get_bytes(e, tid)
                && want.is_none_or(|w| bits(&layout.decode(&b)["asset"]) == w)
            {
                out.push(e);
            }
        }
        if out.is_empty() {
            return Err(match name {
                Some(n) => format!("no og.Timeline plays '{n}'"),
                None => "no entity has an og.Timeline (timeline:play:<name> makes one)".into(),
            });
        }
        Ok(out)
    };
    let push = |e: Entity, o: Op| lock(&shared(engine_id)).ops.push((e.to_bits(), o));
    match verb {
        "play" => {
            let name = args.get(1).filter(|a| !a.is_empty()).ok_or(USAGE)?;
            let h = load(&eng, &asset_path(name))?;
            let entity_name = format!("timeline-{}", short(name));
            let e = match world.find(&entity_name) {
                Some(e) => e,
                None => {
                    let e = world.spawn(()).map_err(|e| e.reason().to_string())?;
                    world
                        .set_name(e, &entity_name)
                        .map_err(|e| e.reason().to_string())?;
                    // og.world.find reads the runtime's names.
                    super::api::set_name(&eng, e, &entity_name);
                    e
                }
            };
            let v = json!({ "asset": h as f64, "playing": false, "time": 0.0 });
            let base = world.get_bytes(e, tid).ok();
            let bytes = layout.encode(base.as_deref(), Some(&v))?;
            world
                .insert_bytes(e, tid, &bytes)
                .map_err(|e| e.reason().to_string())?;
            push(e, Op::Stop);
            push(e, Op::Play);
            Ok(())
        }
        "seek" => {
            let s: f32 = args
                .get(1)
                .and_then(|a| a.trim().parse().ok())
                .filter(|s: &f32| s.is_finite() && *s >= 0.0)
                .ok_or("write timeline:seek:<seconds>[:<name>], such as timeline:seek:4")?;
            for e in targets(world, args.get(2).map(String::as_str))? {
                push(e, Op::Seek(s));
            }
            Ok(())
        }
        "pause" | "resume" | "stop" => {
            let o = match verb {
                "pause" => Op::Pause,
                "resume" => Op::Play,
                _ => Op::Stop,
            };
            for e in targets(world, args.get(1).map(String::as_str))? {
                push(e, o);
            }
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}
