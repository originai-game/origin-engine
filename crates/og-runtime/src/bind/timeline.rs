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
mod present;
pub(crate) use present::present;

use crate::{bind::modules, present::feel, world::assets};
use evaluate::{NESTED, NESTED_TRACKS, Step, drop_ended_voices, evaluate};
use playback::{Op, Rec, Run, shared};
pub(crate) use playback::{install, register_records};
pub use playback::{og_timeline_pause, og_timeline_play, og_timeline_seek, og_timeline_stop};

use crate::engine::{Engine, engine};
use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{Animation, FromJson, record};
use crate::parse;
use glam::{Mat3, Quat, Vec3};
use og_core::IdMap;
use og_core::{Error, ErrorKind, Reflect, Str64};
use og_ecs::{Entity, QueryDesc, SchemaId, StateBlock, SystemContext, SystemDesc, World};
use og_timeline::eval::{self, crossed};
use og_timeline::{CameraMode, Path, PathDesc, PathKind, Timeline, Track, TransitionKind};
use serde_json::{Value, json};
use std::any::Any;
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

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

fn jv3(v: Vec3) -> Value {
    json!({ "x": v.x, "y": v.y, "z": v.z })
}

fn jq(q: Quat) -> Value {
    json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w })
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
    /// The timelines its `timeline` tracks nest, by the id they name.
    pub nested: BTreeMap<String, Arc<Mutex<Loaded>>>,
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
    /// Records og.timeline was granted for the loaded timelines' tracks.
    granted: std::collections::BTreeSet<String>,
    /// Materials a material track drives, as they were before it: the
    /// standard material and the shader parameters it set.
    pub(crate) material_base: BTreeMap<u32, MaterialBase>,
}

/// A material as it was before a material track first drove it.
#[derive(Clone)]
pub(crate) struct MaterialBase {
    pub handle: og_render::MaterialHandle,
    pub standard: og_render::StandardMaterial,
    pub params: Vec<(String, [f32; 4])>,
}

/// og.path and og.timeline on one engine: the registry of its paths and
/// timelines (an asset of the engine, kept across a reload), and the runs
/// its world's block saves, shared with its systems and its calls.
#[derive(Default)]
pub(crate) struct ModuleState {
    registry: Arc<Mutex<Registry>>,
    playback: Arc<Mutex<playback::Shared>>,
}

/// Engine `engine`'s registry (a fresh one for an engine that is gone).
fn registry(engine: u32) -> Arc<Mutex<Registry>> {
    crate::engine::engine(engine)
        .map(|e| e.modules.timeline.registry.clone())
        .unwrap_or_default()
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
pub(crate) fn commit(eng: &Engine, inbox: og_probe::Inbox) {
    lock(&eng.modules.timeline.registry).inbox = Some(inbox);
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
    if let Ok(eng) = crate::engine::engine(engine)
        && let Ok(mut st) = eng.state.try_borrow_mut()
    {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
    }
}

// ── og.path ────────────────────────────────────────────────────────────────

fn curve_desc(v: &Value) -> Result<PathDesc, String> {
    let c: crate::host::wire::CurveDesc = crate::host::wire::value(v, "desc")?;
    if v["points"].is_null() {
        return Err("desc.points is the list of points ({x, y, z})".into());
    }
    let d = PathDesc {
        points: c.points,
        closed: c.closed,
        tension: c.tension,
        kind: match c.kind {
            crate::host::wire::PathKind::CatmullRom => PathKind::CatmullRom,
            crate::host::wire::PathKind::Bezier => PathKind::Bezier,
            crate::host::wire::PathKind::Linear => PathKind::Linear,
        },
    };
    Ok(d)
}

pub fn og_path_create(world: u32, desc: Option<String>) -> Result<Value, String> {
    (|| {
        engine(world)?;
        let v = parse(desc.as_deref())?;
        let p = Path::new(curve_desc(&v)?)?;
        Ok(json!(lock(&registry(world)).add_path(p) as f64))
    })()
}

pub fn og_path_find(world: u32, name: String) -> Result<serde_json::Value, String> {
    (|| {
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
    })()
}

fn with_path<T>(world: u32, path: f64, f: impl FnOnce(&Path) -> T) -> Result<T, String> {
    engine(world)?;
    let p = path_of(world, path as u64).ok_or_else(|| {
        format!("path {path} is not a live path (og.path.create or og.path.find)")
    })?;
    Ok(f(&p))
}

pub fn og_path_sample(
    world: u32,
    path: f64,
    t: f64,
) -> Result<crate::host::wire::PathSample, String> {
    with_path(world, path, |p| {
        let s = p.sample(t as f32);
        crate::host::wire::PathSample {
            position: s.position,
            tangent: s.tangent,
            up: s.up,
        }
    })
}

pub fn og_path_length(world: u32, path: f64) -> Result<serde_json::Value, String> {
    with_path(world, path, |p| json!(p.length()))
}

pub fn og_path_nearest(
    world: u32,
    path: f64,
    point: String,
) -> Result<crate::host::wire::PathNearest, String> {
    crate::host::wire::arg::<Vec3>(Some(&point), "point").and_then(|q| {
        with_path(world, path, |p| {
            let n = p.nearest(q);
            crate::host::wire::PathNearest {
                t: n.t,
                position: n.position,
                distance: n.distance,
            }
        })
    })
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
    load_at(eng, path, &mut Vec::new())
}

/// [`load`] from inside the timelines `chain` nests it in (a cycle or a
/// nesting past [`og_timeline::NEST_DEPTH`] is refused, naming the chain).
fn load_at(eng: &Engine, path: &str, chain: &mut Vec<String>) -> Result<u64, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let reg = registry(eng.id);
    if let Some(&h) = lock(&reg).by_id.get(&id) {
        return Ok(h);
    }
    if chain.contains(&id) || chain.len() > og_timeline::NEST_DEPTH {
        chain.push(id);
        return Err(format!(
            "the timelines nest in a cycle or deeper than {}: {}",
            og_timeline::NEST_DEPTH,
            chain.join(" → ")
        ));
    }
    chain.push(id.clone());
    let loaded = decode(eng, &id, chain);
    chain.pop();
    let loaded = loaded?;
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
fn decode(eng: &Engine, id: &str, chain: &mut Vec<String>) -> Result<Loaded, String> {
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
    let mut nested = BTreeMap::new();
    for tr in &timeline.tracks {
        if let Track::Timeline(keys) = tr {
            for k in keys {
                if nested.contains_key(&k.play) {
                    continue;
                }
                let h = load_at(eng, &k.play, chain).map_err(|e| format!("{id}: {e}"))?;
                let child = lock(&registry(eng.id))
                    .timeline(h)
                    .ok_or_else(|| format!("{id}: {} did not load", k.play))?;
                nested.insert(k.play.clone(), child);
            }
        }
    }
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
            let i = crate::world::assets::load_sound(eng, s)
                .map_err(|e| format!("{id}: sound {s}: {e}"))?;
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
        nested,
    })
}

/// The records a loaded timeline (and those it nests) writes beyond
/// og.timeline's own: a property track's record, the light, drawable and
/// blendshape records its tracks name, og.PostEffects for post and focus.
fn needs(eng: &Engine, lt: &Loaded, out: &mut std::collections::BTreeSet<String>, depth: usize) {
    use og_timeline::FieldsKind as K;
    let layouts = eng.state.borrow().layouts.clone();
    for tr in &lt.timeline.tracks {
        match tr {
            Track::Fields(f) => match f.kind {
                K::Property => {
                    for name in f.fields() {
                        if let Some((rec, _)) = evaluate::record_of(&layouts, name) {
                            out.insert(rec.to_string());
                        }
                    }
                }
                K::Material => {}
                k => out.extend(
                    evaluate::track_records(k)
                        .iter()
                        .filter(|r| layouts.contains_key(**r))
                        .map(|r| r.to_string()),
                ),
            },
            Track::Shot(_) => {
                out.insert(feel::vcam::BRAIN.to_string());
            }
            Track::Camera(c)
                if c.keys.iter().any(|k| {
                    k.focus.is_some() || k.focus_on.is_some() || k.focus_range.is_some()
                }) =>
            {
                out.insert("og.PostEffects".into());
            }
            _ => {}
        }
    }
    if depth < og_timeline::NEST_DEPTH {
        for child in lt.nested.values() {
            needs(eng, &lock(child), out, depth + 1);
        }
    }
}

/// Whether a timeline's camera track holds og.Camera (the brain of the
/// virtual cameras waits while one does).
pub(crate) fn owns_camera(engine: u32) -> bool {
    lock(&playback::shared(engine))
        .runs
        .values()
        .any(|r| r.camera.is_some())
}

/// Whether og.timeline was granted `record` (a field track writes only those).
pub(crate) fn granted(engine: u32, record: &str) -> bool {
    lock(&registry(engine)).granted.contains(record)
}

/// Grants og.timeline what timeline `h` writes; inside a system only what
/// it already has, else refused naming the fix.
fn grant(eng: &Engine, h: u64, world: Option<&mut World>) -> Result<(), String> {
    let Some(t) = lock(&registry(eng.id)).timeline(h) else {
        return Ok(());
    };
    let l = lock(&t);
    grant_loaded(eng, &l, world)
}

fn grant_loaded(eng: &Engine, lt: &Loaded, mut world: Option<&mut World>) -> Result<(), String> {
    let mut want = std::collections::BTreeSet::new();
    needs(eng, lt, &mut want, 0);
    // A shot track registers the virtual cameras' lazy records, as naming
    // og.VirtualCamera would.
    if want.contains(feel::vcam::BRAIN)
        && let Some(w) = world.as_deref_mut()
    {
        crate::engine::lazy::ensure(eng, w, feel::vcam::BRAIN)?;
    }
    // A lazy record no game has used yet is not registered: its tracks
    // write nothing until it is and the timeline loads again.
    if let Some(w) = world.as_deref() {
        want.retain(|n| w.id_by_name(n).is_ok());
    }
    let missing: Vec<String> = {
        let reg = registry(eng.id);
        let r = lock(&reg);
        want.into_iter()
            .filter(|n| !r.granted.contains(n))
            .collect()
    };
    if missing.is_empty() {
        return Ok(());
    }
    let Some(world) = world else {
        return Err(format!(
            "this timeline writes {}, which og.timeline is not yet granted: load it in setup, outside a system",
            missing.join(", ")
        ));
    };
    let names: Vec<&str> = missing.iter().map(String::as_str).collect();
    world
        .grant(TIMELINE_SYSTEM, &names)
        .map_err(|e| e.reason().to_string())?;
    lock(&registry(eng.id)).granted.extend(missing);
    Ok(())
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
        match decode(eng, &id, &mut vec![id.clone()]) {
            Ok(fresh) => {
                if let Err(e) = grant_loaded(eng, &fresh, eng.world().ok()) {
                    failed.push(format!("timeline {id}: {e}"));
                    continue;
                }
                *lock(&t) = fresh;
            }
            Err(e) => failed.push(format!("timeline {id}: {e}")),
        }
    }
    failed
}

/// A model's skeleton and clips by name (og.load.animation of it).
fn clip_set(eng: &Engine, model: &str) -> Result<(u64, BTreeMap<String, u64>), String> {
    let a = Animation::from_json(&crate::bind::anim::load_animation(eng, model)?)?;
    let clips = og_core::sort::map(a.clips.into_iter().map(|c| (c.name, c.clip)));
    Ok((a.skeleton, clips))
}

pub fn og_load_timeline(world: u32, path: String) -> Result<serde_json::Value, String> {
    engine(world).and_then(|eng| {
        let h = load(&eng, &path)?;
        grant(&eng, h, eng.world().ok())?;
        Ok(json!(h as f64))
    })
}

pub fn og_timeline_info(world: u32, asset: f64) -> Result<crate::host::wire::TimelineInfo, String> {
    (|| {
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
        Ok(crate::host::wire::TimelineInfo {
            asset: asset as u64,
            name: tl.name.clone(),
            duration: tl.duration,
            tracks: tl
                .tracks
                .iter()
                .map(|t| crate::host::wire::TimelineTrackInfo {
                    kind: t.kind().to_string(),
                    target: t.target().to_string(),
                    keys: t.key_count() as u32,
                })
                .collect(),
            paths: tl.paths.iter().map(|(n, _)| n.clone()).collect(),
            markers: markers.into_iter().map(|m| m.1).collect(),
        })
    })()
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
                    .get(og_bundle::runtime::asset_id(&asset_path(n)))
                    .ok_or_else(|| {
                        format!("timeline '{n}' is not loaded (timeline:play:{n} loads it)")
                    })?,
            ),
            None => None,
        };
        let mut out = Vec::new();
        for e in world.entities() {
            if let Ok(b) = world.get_bytes(e, tid)
                && want.is_none_or(|w| {
                    layout
                        .read::<record::Timeline>(&b)
                        .is_ok_and(|t| t.asset == w)
                })
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
            grant(&eng, h, Some(&mut *world))?;
            let entity_name = format!("timeline-{}", short(name));
            let e = match world.find(&entity_name) {
                Some(e) => e,
                None => {
                    let e = world.spawn(()).map_err(|e| e.reason().to_string())?;
                    world
                        .set_name(e, &entity_name)
                        .map_err(|e| e.reason().to_string())?;
                    // og.world.find reads the runtime's names.
                    crate::host::api::set_name(&eng, e, &entity_name);
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
