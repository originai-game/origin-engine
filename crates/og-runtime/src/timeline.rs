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

use super::codec::{Layout, RecordKind};
use super::engine::{Engine, engine};
use super::{ok, ok_void, parse, refuse};
use glam::{Mat3, Quat, Vec3};
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
    pub clips: BTreeMap<usize, (u64, BTreeMap<String, u64>)>,
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
    static REGISTRIES: RefCell<BTreeMap<u32, Arc<Mutex<Registry>>>> = const { RefCell::new(BTreeMap::new()) };
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
    let mut clips = BTreeMap::new();
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
    let clips = v["clips"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| Some((c["name"].as_str()?.to_string(), bits(&c["clip"]))))
        .collect();
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
        markers.sort_by(|a, b| a.0.total_cmp(&b.0));
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

// ── og.timeline: ops and the playback state ─────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Play,
    Pause,
    Seek(f32),
    Stop,
}

/// One og.Timeline's playback, kept by the og.timeline state block.
#[derive(Clone, Debug, Default, PartialEq)]
struct Run {
    asset: u64,
    /// The time in steps of 1 / (rate * STEPS) seconds.
    num: i64,
    /// The `time` field last written (another value is a seek).
    written: f32,
    /// `playing` as last written.
    playing: bool,
    /// The next tick advances the time (it was evaluated playing).
    advance: bool,
    /// Paused (a play resumes, without og.TimelineStarted).
    paused: bool,
    /// Evaluated and holding what it owns (camera, voices, caption).
    active: bool,
    /// Ended last tick: hands back what it owns this tick.
    release: bool,
    /// The camera it found: position, quaternion, fov.
    camera: Option<[f32; 8]>,
    /// Per animation track, the key last applied.
    anim: Vec<(u32, u32)>,
    /// Voices it started: (track, key, entity bits).
    voices: Vec<(u32, u32, u64)>,
}

#[derive(Default)]
struct Shared {
    runs: BTreeMap<u64, Run>,
    ops: Vec<(u64, Op)>,
}

thread_local! {
    static SHARED: RefCell<BTreeMap<u32, Arc<Mutex<Shared>>>> = const { RefCell::new(BTreeMap::new()) };
}

fn shared(engine: u32) -> Arc<Mutex<Shared>> {
    SHARED.with(|s| s.borrow_mut().entry(engine).or_default().clone())
}

fn has_timeline(eng: &Engine, e: Entity) -> Result<u64, String> {
    let w = eng.world()?;
    if !w.is_alive(e) {
        return Err(format!("entity {} is not alive", e.to_bits()));
    }
    let id = w
        .id_by_name("og.Timeline")
        .map_err(|e| e.reason().to_string())?;
    let bytes = w
        .get_bytes(e, id)
        .map_err(|_| format!("entity {} has no og.Timeline", e.to_bits()))?;
    let layout = eng
        .state
        .borrow()
        .layouts
        .get("og.Timeline")
        .cloned()
        .ok_or("og.Timeline is not registered")?;
    Ok(bits(&layout.decode(&bytes)["asset"]))
}

fn op(world: u32, entity: f64, op: Op) -> String {
    wire_void((|| {
        let eng = engine(world)?;
        let e = Entity::from_bits(entity as u64);
        let asset = has_timeline(&eng, e)?;
        if matches!(op, Op::Play | Op::Seek(_)) {
            let t = lock(&registry(world)).timeline(asset).ok_or_else(|| {
                format!("its og.Timeline.asset {asset} is not a loaded timeline (og.load.timeline)")
            })?;
            if let Op::Seek(s) = op {
                let d = lock(&t).timeline.duration;
                if !(s.is_finite() && (0.0..=d).contains(&s)) {
                    return Err(format!(
                        "time {s} is outside 0 to the timeline's duration {d} s"
                    ));
                }
            }
        }
        lock(&shared(world)).ops.push((e.to_bits(), op));
        Ok(())
    })())
}

pub fn og_timeline_play(world: u32, entity: f64) -> String {
    op(world, entity, Op::Play)
}

pub fn og_timeline_pause(world: u32, entity: f64) -> String {
    op(world, entity, Op::Pause)
}

pub fn og_timeline_seek(world: u32, entity: f64, time: f64) -> String {
    op(world, entity, Op::Seek(time as f32))
}

pub fn og_timeline_stop(world: u32, entity: f64) -> String {
    op(world, entity, Op::Stop)
}

// ── the state block ──────────────────────────────────────────────────────────

struct Bytes<'a>(&'a [u8]);

impl Bytes<'_> {
    fn take<const N: usize>(&mut self) -> og_core::Result<[u8; N]> {
        let (a, b) = self
            .0
            .split_first_chunk::<N>()
            .ok_or_else(|| invalid("the og.timeline state is cut short"))?;
        self.0 = b;
        Ok(*a)
    }
    fn u32(&mut self) -> og_core::Result<u32> {
        self.take::<4>().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> og_core::Result<u64> {
        self.take::<8>().map(u64::from_le_bytes)
    }
    fn f32(&mut self) -> og_core::Result<f32> {
        self.take::<4>().map(f32::from_le_bytes)
    }
}

fn save(sh: &Shared, out: &mut Vec<u8>) {
    let u32_ = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
    let u64_ = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
    u32_(out, sh.ops.len() as u32);
    for (e, op) in &sh.ops {
        u64_(out, *e);
        let (k, t) = match op {
            Op::Play => (0, 0.0f32),
            Op::Pause => (1, 0.0),
            Op::Seek(t) => (2, *t),
            Op::Stop => (3, 0.0),
        };
        u32_(out, k);
        out.extend_from_slice(&t.to_le_bytes());
    }
    u32_(out, sh.runs.len() as u32);
    for (e, r) in &sh.runs {
        u64_(out, *e);
        u64_(out, r.asset);
        u64_(out, r.num as u64);
        out.extend_from_slice(&r.written.to_le_bytes());
        let flags = u32::from(r.playing)
            | u32::from(r.advance) << 1
            | u32::from(r.paused) << 2
            | u32::from(r.active) << 3
            | u32::from(r.release) << 4
            | u32::from(r.camera.is_some()) << 5;
        u32_(out, flags);
        for v in r.camera.unwrap_or_default() {
            out.extend_from_slice(&v.to_le_bytes());
        }
        u32_(out, r.anim.len() as u32);
        for (t, k) in &r.anim {
            u32_(out, *t);
            u32_(out, *k);
        }
        u32_(out, r.voices.len() as u32);
        for (t, k, v) in &r.voices {
            u32_(out, *t);
            u32_(out, *k);
            u64_(out, *v);
        }
    }
}

fn load_state(bytes: &[u8]) -> og_core::Result<Shared> {
    let mut b = Bytes(bytes);
    let mut sh = Shared::default();
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let k = b.u32()?;
        let t = b.f32()?;
        sh.ops.push((
            e,
            match k {
                0 => Op::Play,
                1 => Op::Pause,
                2 => Op::Seek(t),
                3 => Op::Stop,
                _ => return Err(invalid("an unknown og.timeline op")),
            },
        ));
    }
    for _ in 0..b.u32()? {
        let e = b.u64()?;
        let mut r = Run {
            asset: b.u64()?,
            num: b.u64()? as i64,
            written: b.f32()?,
            ..Run::default()
        };
        let flags = b.u32()?;
        r.playing = flags & 1 != 0;
        r.advance = flags & 2 != 0;
        r.paused = flags & 4 != 0;
        r.active = flags & 8 != 0;
        r.release = flags & 16 != 0;
        let mut cam = [0.0f32; 8];
        for c in &mut cam {
            *c = b.f32()?;
        }
        r.camera = (flags & 32 != 0).then_some(cam);
        for _ in 0..b.u32()? {
            r.anim.push((b.u32()?, b.u32()?));
        }
        for _ in 0..b.u32()? {
            r.voices.push((b.u32()?, b.u32()?, b.u64()?));
        }
        sh.runs.insert(e, r);
    }
    if !b.0.is_empty() {
        return Err(invalid("the og.timeline state has bytes left over"));
    }
    Ok(sh)
}

struct TimelineBlock(Arc<Mutex<Shared>>);

impl StateBlock for TimelineBlock {
    fn save(&self, out: &mut Vec<u8>) {
        save(&lock(&self.0), out);
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Ok(Box::new(load_state(bytes)?))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let next = decoded
            .and_then(|d| d.downcast::<Shared>().ok())
            .map(|b| *b)
            .unwrap_or_default();
        *lock(&self.0) = next;
    }
}

// ── installing ───────────────────────────────────────────────────────────────

/// The records the systems read and write.
#[derive(Clone)]
struct Rec {
    transform: (SchemaId, Layout),
    follower: (SchemaId, Layout),
    timeline: (SchemaId, Layout),
    camera: (SchemaId, Layout),
    model: (SchemaId, Layout),
    skeleton: (SchemaId, Layout),
    player: (SchemaId, Layout),
    source: (SchemaId, Layout),
}

/// Registers the timeline events (their layouts into `layouts`).
pub(crate) fn register_records(
    world: &mut World,
    layouts: &mut BTreeMap<String, Layout>,
) -> Result<(), String> {
    let err = |e: Error| e.reason().to_string();
    world.register::<TimelineStarted>().map_err(err)?;
    world.register::<TimelineMarker>().map_err(err)?;
    world.register::<TimelineEnded>().map_err(err)?;
    for n in REGISTERED_BY_TIMELINE {
        let id = world.id_by_name(n).map_err(err)?;
        let layout = Layout::from_schema(world.schema(id).map_err(err)?, RecordKind::Event);
        layouts.insert(layout.schema.clone(), layout);
    }
    Ok(())
}

/// Installs og.path and og.timeline (after the game's systems and og.tween,
/// before og.anim) and the og.timeline state block.
pub(crate) fn install(
    world: &mut World,
    engine_id: u32,
    layouts: &BTreeMap<String, Layout>,
    ui: &og_ui::SharedUi,
) -> Result<(), String> {
    let err = |e: Error| e.reason().to_string();
    let get = |n: &str| -> Result<(SchemaId, Layout), String> {
        let id = world.id_by_name(n).map_err(err)?;
        let layout = layouts.get(n).cloned().unwrap_or_else(|| {
            Layout::from_schema(
                world.schema(id).expect("a registered schema"),
                RecordKind::Component,
            )
        });
        Ok((id, layout))
    };
    let r = Rec {
        transform: get("og.Transform")?,
        follower: get("og.PathFollower")?,
        timeline: get("og.Timeline")?,
        camera: get("og.Camera")?,
        model: get("og.Model")?,
        skeleton: get("og.Skeleton")?,
        player: get("og.AnimationPlayer")?,
        source: get("og.AudioSource")?,
    };
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: PATH_SYSTEM.into(),
                reads: Vec::new(),
                writes: names(&[&r.follower, &r.transform]),
                queries: vec![QueryDesc {
                    write: names(&[&r.follower, &r.transform]),
                    ..QueryDesc::default()
                }],
            },
            move |ctx| path_system(ctx, engine_id, &rec),
        )
        .map_err(err)?;
    let sh = shared(engine_id);
    *lock(&sh) = Shared::default();
    let rec = r.clone();
    let ui = ui.clone();
    let state = sh.clone();
    let mut writes = names(&[
        &r.timeline,
        &r.transform,
        &r.camera,
        &r.skeleton,
        &r.player,
        &r.source,
    ]);
    writes.extend(REGISTERED_BY_TIMELINE.iter().map(|s| s.to_string()));
    world
        .add_system(
            SystemDesc {
                name: TIMELINE_SYSTEM.into(),
                reads: names(&[&r.model]),
                writes,
                queries: vec![QueryDesc {
                    write: names(&[&r.timeline]),
                    ..QueryDesc::default()
                }],
            },
            move |ctx| timeline_system(ctx, engine_id, &state, &rec, &ui),
        )
        .map_err(err)?;
    world
        .add_state_block(TIMELINE_SYSTEM, Box::new(TimelineBlock(sh)))
        .map_err(err)?;
    Ok(())
}

// ── og.path's system ─────────────────────────────────────────────────────────

fn path_system(ctx: &mut SystemContext<'_>, engine_id: u32, r: &Rec) -> og_core::Result<()> {
    let dt = ctx.scaled_dt();
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    for e in rows {
        let fb = ctx.get_bytes(e, r.follower.0)?;
        let mut f = r.follower.1.decode(&fb);
        let h = bits(&f["path"]);
        if h == 0 || !f["playing"].as_bool().unwrap_or(true) {
            continue;
        }
        let Some(p) = path_of(engine_id, h) else {
            note(
                engine_id,
                format!(
                    "og.PathFollower.path {h} is not a live path (og.path.create or og.path.find): it does not move"
                ),
            );
            continue;
        };
        let len = p.length();
        let speed = f["speed"].as_f64().unwrap_or(1.0) as f32;
        let mut d = f["distance"].as_f64().unwrap_or(0.0) as f32 + speed * dt;
        let mut finished = false;
        if len > 0.0 {
            if f["loop"].as_bool().unwrap_or(true) {
                d = d.rem_euclid(len);
            } else if d >= len || d <= 0.0 {
                d = d.clamp(0.0, len);
                finished = (speed > 0.0 && d >= len) || (speed < 0.0 && d <= 0.0);
            }
        }
        let s = p.sample(if len > 0.0 { d / len } else { 0.0 });
        let offset = v3(&f["offset"]);
        let align = f["align"].as_bool().unwrap_or(false);
        let tb = ctx.get_bytes(e, r.transform.0)?;
        let mut t = r.transform.1.decode(&tb);
        let pos = if align {
            let (m, q) = facing(&s);
            t["quaternion"] = jq(q);
            s.position + m * offset
        } else {
            s.position + offset
        };
        t["position"] = jv3(pos);
        f["distance"] = json!(d);
        f["finished"] = json!(finished);
        let tb = r.transform.1.encode(Some(&tb), Some(&t)).map_err(invalid)?;
        ctx.set_bytes(e, r.transform.0, &tb)?;
        let fb = r.follower.1.encode(Some(&fb), Some(&f)).map_err(invalid)?;
        ctx.set_bytes(e, r.follower.0, &fb)?;
    }
    Ok(())
}

// ── og.timeline's system ─────────────────────────────────────────────────────

fn timeline_system(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    state: &Mutex<Shared>,
    r: &Rec,
    ui: &og_ui::SharedUi,
) -> og_core::Result<()> {
    let time = ctx.time();
    let rate = (1.0 / f64::from(time.dt)).round().max(1.0) as i64;
    let den = rate * STEPS;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    let ops = std::mem::take(&mut lock(state).ops);
    // Runs of entities that lost their og.Timeline (or died): let go.
    let gone: Vec<u64> = lock(state)
        .runs
        .keys()
        .copied()
        .filter(|k| !rows.iter().any(|e| e.to_bits() == *k))
        .collect();
    for k in gone {
        let run = lock(state).runs.remove(&k);
        if let Some(mut run) = run {
            release(ctx, r, &mut run)?;
        }
    }
    let reg = registry(engine_id);
    for e in rows {
        let key = e.to_bits();
        let cb = ctx.get_bytes(e, r.timeline.0)?;
        let mut comp = r.timeline.1.decode(&cb);
        let asset = bits(&comp["asset"]);
        let mut run = lock(state).runs.remove(&key).unwrap_or_default();
        let loaded = lock(&reg).timeline(asset);
        // A new run starts where its `time` field is, stopped: nothing is
        // evaluated (the camera is not taken) until a play or a seek.
        let field_time = comp["time"].as_f64().unwrap_or(0.0) as f32;
        if run.asset != asset {
            release(ctx, r, &mut run)?;
            run = Run {
                asset,
                written: field_time,
                num: (f64::from(field_time.max(0.0)) * den as f64).round() as i64,
                ..Run::default()
            };
        }
        let Some(loaded) = loaded else {
            if asset != 0 {
                note(
                    engine_id,
                    format!(
                        "og.Timeline.asset {asset} is not a loaded timeline (og.load.timeline): it plays nothing"
                    ),
                );
            }
            lock(state).runs.insert(key, run);
            continue;
        };
        let mut playing = comp["playing"].as_bool().unwrap_or(false);
        let mut stop = false;
        let mut seek: Option<f32> = None;
        if field_time != run.written {
            seek = Some(field_time);
        }
        for (_, o) in ops.iter().filter(|(k, _)| *k == key) {
            match *o {
                Op::Play => playing = true,
                Op::Pause => playing = false,
                Op::Seek(t) => seek = Some(t),
                Op::Stop => {
                    stop = true;
                    playing = false;
                    seek = None;
                }
            }
        }
        let mut lt = lock(&loaded);
        let duration = lt.timeline.duration;
        let dur = (f64::from(duration) * den as f64).round() as i64;
        if stop {
            release(ctx, r, &mut run)?;
            run = Run {
                asset,
                ..Run::default()
            };
            comp["time"] = json!(0.0);
            comp["playing"] = json!(false);
            run.written = 0.0;
            // A stop resets; a play or seek queued after it (the
            // command `timeline:play` queues Stop then Play) still applies.
            if !playing && seek.is_none() {
                let out = r
                    .timeline
                    .1
                    .encode(Some(&cb), Some(&comp))
                    .map_err(invalid)?;
                ctx.set_bytes(e, r.timeline.0, &out)?;
                lock(state).runs.insert(key, run);
                continue;
            }
        }
        // A timeline that ended last tick hands back what it owns now
        // (its last frame showed the end).
        if run.release && !playing && seek.is_none() {
            release(ctx, r, &mut run)?;
            run.release = false;
            run.active = false;
        }
        let starting = playing && !run.playing;
        let before = run.num;
        let mut seeked = false;
        if let Some(s) = seek {
            run.num = ((f64::from(s.max(0.0)) * den as f64).round() as i64).clamp(0, dur);
            run.advance = false;
            seeked = true;
        }
        if starting && !seeked && run.num >= dur {
            // Played again after its end: from the start.
            run.num = 0;
        }
        let from = run.num;
        if playing {
            if run.advance {
                let speed = comp["speed"].as_f64().unwrap_or(1.0).clamp(0.0, 16.0);
                run.num += (speed * STEPS as f64).round() as i64;
            }
            run.advance = true;
        } else {
            run.advance = false;
        }
        let mut ended = false;
        let mut wrapped = false;
        if run.num >= dur {
            if playing && comp["loop"].as_bool().unwrap_or(false) && dur > 0 {
                run.num %= dur;
                wrapped = true;
            } else {
                run.num = dur;
                if playing {
                    ended = true;
                    playing = false;
                }
            }
        }
        if starting && !run.paused {
            ctx.send(TimelineStarted { entity: key, asset })?;
        }
        if playing || seeked || run.active || ended {
            run.active = true;
        }
        let secs = |n: i64| n as f64 / den as f64;
        let t = secs(run.num) as f32;
        // What playback passed this tick: (from, to] (a start includes its
        // first time), across the end when it wrapped.
        let mut spans: Vec<(f64, f64)> = Vec::new();
        if (playing || ended) && !seeked {
            let lo = if starting {
                secs(from) - 1e-9
            } else {
                secs(from)
            };
            if wrapped {
                spans.push((lo, secs(dur)));
                spans.push((-1e-9, secs(run.num)));
            } else {
                spans.push((lo, secs(run.num)));
            }
        }
        // A forward seek runs the retroactive events it jumps over.
        let retro = (seeked && run.num > before).then(|| (secs(before), secs(run.num)));
        if run.active {
            evaluate(
                ctx,
                engine_id,
                r,
                ui,
                &mut lt,
                &mut run,
                t,
                playing,
                &spans,
                retro,
                time.dt,
                comp["speed"].as_f64().unwrap_or(1.0) as f32,
            )?;
        }
        for (a, b) in &spans {
            for tr in &lt.timeline.tracks {
                if let Track::Marker(ks) = tr {
                    for k in ks.iter().filter(|k| crossed(k.time, *a, *b)) {
                        ctx.send(TimelineMarker {
                            entity: key,
                            asset,
                            name: Str64::new(&k.name).unwrap_or_default(),
                            time: k.time,
                        })?;
                    }
                }
            }
        }
        if ended {
            ctx.send(TimelineEnded { entity: key, asset })?;
            run.release = true;
            run.paused = false;
        } else if !playing && run.playing && !stop {
            run.paused = true;
        }
        if playing {
            run.paused = false;
        }
        drop(lt);
        comp["time"] = json!(t);
        comp["playing"] = json!(playing);
        run.written = t;
        run.playing = playing;
        let out = r
            .timeline
            .1
            .encode(Some(&cb), Some(&comp))
            .map_err(invalid)?;
        ctx.set_bytes(e, r.timeline.0, &out)?;
        lock(state).runs.insert(key, run);
    }
    Ok(())
}

/// Lets go of what a run owns: the camera back, its voices stopped.
fn release(ctx: &mut SystemContext<'_>, r: &Rec, run: &mut Run) -> og_core::Result<()> {
    if let Some(c) = run.camera.take() {
        let bytes = ctx.resource_bytes(r.camera.0)?;
        let mut cam = r.camera.1.decode(&bytes);
        cam["position"] = jv3(Vec3::new(c[0], c[1], c[2]));
        cam["quaternion"] = jq(Quat::from_xyzw(c[3], c[4], c[5], c[6]));
        cam["fov"] = json!(c[7]);
        let out = r
            .camera
            .1
            .encode(Some(&bytes), Some(&cam))
            .map_err(invalid)?;
        ctx.set_resource_bytes(r.camera.0, &out)?;
    }
    for (_, _, v) in std::mem::take(&mut run.voices) {
        let e = Entity::from_bits(v);
        if ctx.is_alive(e) {
            ctx.despawn(e)?;
        }
    }
    run.anim.clear();
    run.active = false;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn evaluate(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    ui: &og_ui::SharedUi,
    lt: &mut Loaded,
    run: &mut Run,
    t: f32,
    playing: bool,
    spans: &[(f64, f64)],
    retro: Option<(f64, f64)>,
    dt: f32,
    speed: f32,
) -> og_core::Result<()> {
    let duration = lt.timeline.duration;
    // One-shot voices that ended go; while paused every voice goes (a
    // resume starts the music again at its offset).
    let mut keep = Vec::new();
    for (tr, k, v) in std::mem::take(&mut run.voices) {
        let e = Entity::from_bits(v);
        let alive = ctx.is_alive(e)
            && ctx
                .get_bytes(e, r.source.0)
                .is_ok_and(|b| r.source.1.decode(&b)["playing"].as_bool().unwrap_or(false));
        if alive && playing {
            keep.push((tr, k, v));
        } else if ctx.is_alive(e) {
            ctx.despawn(e)?;
        }
    }
    run.voices = keep;
    let tracks = lt.timeline.tracks.clone();
    for (ti, tr) in tracks.iter().enumerate() {
        let ti32 = ti as u32;
        match tr {
            Track::Camera(c) => camera(ctx, engine_id, r, lt, run, c, t)?,
            Track::Transform(tt) => {
                let Some(e) = ctx.find(&tt.entity) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: no entity '{}' for its transform track",
                            lt.id, tt.entity
                        ),
                    );
                    continue;
                };
                let at = eval::transform_at(tt, t);
                let bytes = ctx.get_bytes(e, r.transform.0)?;
                let mut v = r.transform.1.decode(&bytes);
                if let Some(p) = at.position {
                    v["position"] = jv3(p);
                }
                if let Some(q) = at.quaternion {
                    v["quaternion"] = jq(q);
                }
                if let Some(s) = at.scale {
                    v["scale"] = jv3(s);
                }
                let out = r
                    .transform
                    .1
                    .encode(Some(&bytes), Some(&v))
                    .map_err(invalid)?;
                ctx.set_bytes(e, r.transform.0, &out)?;
            }
            Track::Animation(a) => {
                let Some((ki, ct)) = eval::clip_at(a, t) else {
                    continue;
                };
                let Some(e) = ctx.find(&a.entity) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: no entity '{}' for its animation track",
                            lt.id, a.entity
                        ),
                    );
                    continue;
                };
                if let std::collections::btree_map::Entry::Vacant(slot) = lt.clips.entry(ti) {
                    match model_clips(ctx, engine_id, r, e) {
                        Ok(c) => {
                            slot.insert(c);
                        }
                        Err(m) => {
                            note(
                                engine_id,
                                format!(
                                    "timeline {}: animation track of '{}': {m}",
                                    lt.id, a.entity
                                ),
                            );
                            continue;
                        }
                    }
                }
                let (skeleton, clips) = &lt.clips[&ti];
                let key = &a.keys[ki];
                let Some(&clip) = clips.get(&key.clip) else {
                    note(
                        engine_id,
                        format!(
                            "timeline {}: '{}' has no clip '{}' (its clips: {})",
                            lt.id,
                            a.entity,
                            key.clip,
                            clips.keys().cloned().collect::<Vec<_>>().join(", ")
                        ),
                    );
                    continue;
                };
                if ctx.get_bytes(e, r.skeleton.0).is_err() {
                    let s = r
                        .skeleton
                        .1
                        .encode(None, Some(&json!({ "asset": *skeleton as f64 })))
                        .map_err(invalid)?;
                    ctx.insert_bytes(e, r.skeleton.0, &s)?;
                }
                let changed = run.anim.iter().find(|x| x.0 == ti32).map(|x| x.1) != Some(ki as u32);
                run.anim.retain(|x| x.0 != ti32);
                run.anim.push((ti32, ki as u32));
                // og.anim advances the player by its speed after this
                // system: write the time it lands on `ct` from.
                let eff = if playing { key.speed * speed } else { 0.0 };
                let mut v = match ctx.get_bytes(e, r.player.0) {
                    Ok(b) => r.player.1.decode(&b),
                    Err(_) => json!({}),
                };
                v["clip"] = json!(clip as f64);
                v["speed"] = json!(eff);
                v["loop"] = json!(key.looping);
                v["time"] = json!((ct - dt * eff).max(0.0));
                if changed {
                    v["fade"] = json!(if playing
                        && spans.iter().any(|(a, b)| crossed(key.time, *a, *b))
                    {
                        key.blend_in
                    } else {
                        0.0
                    });
                }
                let base = ctx.get_bytes(e, r.player.0).ok();
                let out = r
                    .player
                    .1
                    .encode(base.as_deref(), Some(&v))
                    .map_err(invalid)?;
                if base.is_some() {
                    ctx.set_bytes(e, r.player.0, &out)?;
                } else {
                    ctx.insert_bytes(e, r.player.0, &out)?;
                }
            }
            Track::Audio(ks) if playing => {
                for (ki, k) in ks.iter().enumerate() {
                    let (handle, seconds) = lt.sounds.get(&k.sound).copied().unwrap_or((0, 0.0));
                    let started = run.voices.iter().any(|v| v.0 == ti32 && v.1 == ki as u32);
                    let fire = spans.iter().any(|(a, b)| crossed(k.time, *a, *b));
                    // Joined mid-sound (a start or a seek past its key): from its offset.
                    let mid = spans.is_empty() || spans.iter().all(|(a, _)| *a > f64::from(k.time));
                    let within = t >= k.time && t < k.time + seconds / k.pitch.max(1e-3);
                    if started || handle == 0 || !(fire || (mid && within)) {
                        continue;
                    }
                    let offset = if fire { 0.0 } else { (t - k.time) * k.pitch };
                    let v = json!({
                        "sound": handle as f64, "bus": k.bus, "volume": k.volume, "pitch": k.pitch,
                        "playing": true, "startOffset": offset,
                    });
                    let bytes = r.source.1.encode(None, Some(&v)).map_err(invalid)?;
                    let e = ctx.spawn_bytes(vec![(r.source.0, bytes)])?;
                    run.voices.push((ti32, ki as u32, e.to_bits()));
                }
            }
            Track::Music(ks) => {
                for (ki, k) in ks.iter().enumerate() {
                    let gain = if playing {
                        eval::music_gain(k, t, duration)
                    } else {
                        None
                    };
                    let at = run
                        .voices
                        .iter()
                        .position(|v| v.0 == ti32 && v.1 == ki as u32);
                    match (gain, at) {
                        (Some(g), Some(i)) => {
                            let e = Entity::from_bits(run.voices[i].2);
                            let b = ctx.get_bytes(e, r.source.0)?;
                            let mut v = r.source.1.decode(&b);
                            v["volume"] = json!(g);
                            let out = r.source.1.encode(Some(&b), Some(&v)).map_err(invalid)?;
                            ctx.set_bytes(e, r.source.0, &out)?;
                        }
                        (Some(g), None) => {
                            let Some(&(handle, _)) = lt.sounds.get(&k.sound) else {
                                continue;
                            };
                            let v = json!({
                                "sound": handle as f64, "bus": k.bus, "volume": g, "looping": k.looping,
                                "playing": true, "startOffset": t - k.time,
                            });
                            let bytes = r.source.1.encode(None, Some(&v)).map_err(invalid)?;
                            let e = ctx.spawn_bytes(vec![(r.source.0, bytes)])?;
                            run.voices.push((ti32, ki as u32, e.to_bits()));
                        }
                        (None, Some(i)) => {
                            let (_, _, v) = run.voices.remove(i);
                            let e = Entity::from_bits(v);
                            if ctx.is_alive(e) {
                                ctx.despawn(e)?;
                            }
                        }
                        (None, None) => {}
                    }
                }
            }
            Track::Caption(c) => {
                if let Some(k) = eval::caption_at(c, t) {
                    caption(ui, ti, k, c);
                }
            }
            Track::Event(ks) => {
                for k in ks {
                    let run_it = spans.iter().any(|(a, b)| crossed(k.time, *a, *b))
                        || (k.retroactive && retro.is_some_and(|(a, b)| crossed(k.time, a, b)));
                    if run_it {
                        let mut parts = k.action.split(':').map(str::to_string);
                        let action = parts.next().unwrap_or_default();
                        if let Some(inbox) = lock(&registry(engine_id)).inbox.clone() {
                            inbox.push(action, parts.collect());
                        }
                    }
                }
            }
            Track::Transition(ks) => {
                if let Some(c) = eval::cover_at(ks, t) {
                    let kind = match c.kind {
                        TransitionKind::Cut | TransitionKind::Fade => og_ui::ScreenWipe::Fade,
                        TransitionKind::Wipe => og_ui::ScreenWipe::Wipe,
                        TransitionKind::Dissolve => og_ui::ScreenWipe::Dissolve,
                        TransitionKind::Iris => og_ui::ScreenWipe::Iris,
                        TransitionKind::Slide => og_ui::ScreenWipe::Slide,
                    };
                    let side = [
                        og_ui::Side::Left,
                        og_ui::Side::Right,
                        og_ui::Side::Top,
                        og_ui::Side::Bottom,
                    ];
                    let byte = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
                    let desc = og_ui::ScreenTransitionDesc {
                        kind,
                        color: Some(og_ui::Srgba {
                            rgb: [byte(c.color.x), byte(c.color.y), byte(c.color.z)],
                            alpha: 1.0,
                        }),
                        from: side[(c.from as usize).min(3)],
                        ..Default::default()
                    };
                    ui.lock().cover(&desc, c.amount, c.revealing);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The skeleton and clips of the model an entity draws (its og.Model's glTF).
fn model_clips(
    ctx: &SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    e: Entity,
) -> Result<(u64, BTreeMap<String, u64>), String> {
    let b = ctx.get_bytes(e, r.model.0).map_err(|_| {
        "it has no og.Model; name the glTF with `model \"<path>\"` in the track".to_string()
    })?;
    let asset = r.model.1.decode(&b)["asset"].as_f64().unwrap_or(0.0);
    let eng = engine(engine_id)?;
    let path = {
        let a = eng.assets.borrow();
        super::assets::index_of(asset)
            .and_then(|i| a.entries.get(i))
            .map(|x| x.path.clone())
            .ok_or("its og.Model.asset is not a loaded glTF")?
    };
    clip_set(&eng, &path)
}

fn camera(
    ctx: &mut SystemContext<'_>,
    engine_id: u32,
    r: &Rec,
    lt: &Loaded,
    run: &mut Run,
    c: &og_timeline::CameraTrack,
    t: f32,
) -> og_core::Result<()> {
    let at = eval::camera_at(c, t);
    let bytes = ctx.resource_bytes(r.camera.0)?;
    let mut cam = r.camera.1.decode(&bytes);
    if run.camera.is_none() {
        let p = v3(&cam["position"]);
        let q = quat(&cam["quaternion"]);
        run.camera = Some([
            p.x,
            p.y,
            p.z,
            q.x,
            q.y,
            q.z,
            q.w,
            cam["fov"].as_f64().unwrap_or(50.0) as f32,
        ]);
    }
    let (pos, target) = match &c.mode {
        CameraMode::Keys => (at.position, at.target),
        CameraMode::Rail(name) => {
            let p = lt.paths.get(name).cloned().or_else(|| {
                let reg = registry(engine_id);
                let r = lock(&reg);
                r.named.get(name).and_then(|h| r.path(*h))
            });
            let Some(p) = p else {
                note(
                    engine_id,
                    format!(
                        "timeline {}: the camera's rail '{name}' is neither a path of the timeline nor of a spawned scene",
                        lt.id
                    ),
                );
                return Ok(());
            };
            let s = p.sample(at.at.unwrap_or(0.0));
            (
                Some(s.position),
                Some(at.target.unwrap_or(s.position + s.tangent)),
            )
        }
        CameraMode::Follow(name) => {
            let Some(e) = ctx.find(name) else {
                note(
                    engine_id,
                    format!(
                        "timeline {}: no entity '{name}' for the camera to follow",
                        lt.id
                    ),
                );
                return Ok(());
            };
            let b = ctx.get_bytes(e, r.transform.0)?;
            let p = v3(&r.transform.1.decode(&b)["position"]);
            (
                Some(p + at.position.unwrap_or(Vec3::new(0.0, 2.0, 6.0))),
                Some(p + at.target.unwrap_or(Vec3::new(0.0, 1.0, 0.0))),
            )
        }
    };
    if let Some(p) = pos {
        cam["position"] = jv3(p);
    }
    if let Some(target) = target {
        let eye = pos.unwrap_or_else(|| v3(&cam["position"]));
        cam["quaternion"] = jq(super::feel::look_rotation(eye, target));
    }
    if let Some(f) = at.fov {
        cam["fov"] = json!(f);
    }
    let out = r
        .camera
        .1
        .encode(Some(&bytes), Some(&cam))
        .map_err(invalid)?;
    ctx.set_resource_bytes(r.camera.0, &out)
}

fn caption(
    ui: &og_ui::SharedUi,
    ti: usize,
    k: &og_timeline::CaptionKey,
    c: &og_timeline::CaptionTrack,
) {
    let mut ui = ui.lock();
    let theme = *ui.theme();
    let style = k.style.as_deref().unwrap_or(&c.style);
    let anchor = k.anchor.as_deref().unwrap_or(&c.anchor);
    let id = format!("timeline-caption-{ti}");
    ui.open_anchor(
        id.as_str(),
        og_ui::AnchorDesc {
            anchor: super::modules::anchor_of(&json!(anchor)).unwrap_or(og_ui::Anchor::Bottom),
            padding: og_ui::Insets::xy(16.0, 8.0),
            background: Some(og_ui::Srgba {
                rgb: [0, 0, 0],
                alpha: 0.62,
            }),
            radius: 8.0,
            over: true,
            instant: true,
            ..Default::default()
        },
    );
    ui.label(
        format!("{id}-text").as_str(),
        &k.text,
        og_ui::LabelDesc {
            step: super::modules::step(&theme, style),
            color: Some(og_ui::Srgba {
                rgb: [255, 255, 255],
                alpha: 1.0,
            }),
            align: og_ui::TextAlign::Center,
            ..Default::default()
        },
    );
    ui.close();
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
