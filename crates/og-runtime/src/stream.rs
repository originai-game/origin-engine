//! Streaming groups (IDL `stream`, phase 3).
//!
//! A scene's `stream` blocks name assets that load and unload together; the
//! cook packs each group's files into `cooked/stream/<name>.ogb` and lists
//! the group in the manifest. Here, at every tick boundary, each group of
//! the game's bundle is wanted or not (the camera within its distance, kept
//! until past its unload distance; the camera inside its volume entity's
//! box; or og.stream.request), wanted groups are admitted by priority
//! within the memory budget (a lower-priority resident group is evicted for
//! a higher one; one that cannot fit waits, queued), and admitted groups
//! are read and put in place of their stand-ins:
//!
//! - a model draws its `#proxy` (the coarsest LOD, untextured) until its
//!   group is resident, then its full meshes, materials and textures under
//!   the same render handles; physics always has the proxy's meshes, so
//!   nothing the simulation reads depends on what is resident;
//! - a texture is a 1x1 placeholder until then;
//! - a sound is silence of its own rate, channels and length (the tick side
//!   times it exactly as the real one), then its samples behind the same
//!   handle.
//!
//! An unloaded group puts its stand-ins back and frees what its assets took
//! on the GPU and in the audio bank. The environment maps, texture arrays,
//! palettes, looks and colour LUTs loaded from its pack have no stand-in:
//! they are let go (removed from the renderer, their handles pending) and
//! come back when the group is resident again ([`resolve`]). One the frame
//! still uses (the map og.Environment lights it with, the palette Stylise
//! quantises to, an array a sprite, an overlay sprite or a particle
//! emitter draws from, the look last applied, the LUT PostEffects grades
//! with) is never freed under the frame: it is kept, with a warning naming
//! it, and let go at the first tick boundary the frame no longer uses it.
//! What is kept still takes its memory: its bytes count in the budget's
//! `used` (and `kept`) until it is let go, so a group that would not fit
//! beside it waits. Every change of a group's state sends og.StreamEvent at
//! the start of the next tick; the probe's `dump_stream` lists the budget,
//! what is used (kept assets included) and the peak, every group, and what
//! each keeps with its bytes.
//!
//! What streams is read through one queue ([`fetch`]): the chunks of the
//! sounds cooked in chunks first, the one a playhead needs soonest first
//! (`og_audio::chunk_distances`), then the loading groups' packs by
//! priority (then name). At most [`MAX_FETCHES`] reads are in flight: a
//! read the host answers `Pending` (the web host's fetch) holds a slot
//! until it lands, and what does not get a slot waits in order. Natively a
//! read completes at once, so nothing waits and only the order shows.
//! `dump_stream`'s `fetch` lists what is in flight and what waits.

mod loading;
mod residency;

use super::{assets, environment, extract, fx, geometry, look};
use loading::fetch_json;
pub(crate) use loading::{fetch, refresh, update};
pub(crate) use residency::{add_model, add_sound, add_texture, placeholder, resolve};
use residency::{downgrade, release, retry_retained, upgrade};

use super::engine::{Engine, engine, hooks};
use super::{ok, ok_void, refuse};
use og_bundle::runtime::{CookedSound, ManifestStream, StreamTrigger};
use og_core::IdMap;
use og_core::{Reflect, Str64};
use og_ecs::{SystemDesc, World};
use og_render::{
    MaterialHandle, MeshData, MeshHandle, StandardMaterial, TextureData, TextureHandle,
    TextureSource,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The system's name in the schedule (before the game's systems).
pub const STREAM_SYSTEM: &str = "og.stream";

/// The memory budget until og.stream.setBudget: 256 MiB.
pub const DEFAULT_BUDGET: u64 = 256 << 20;

/// The events `dump_stream` keeps.
const MAX_LOG: usize = 256;

/// The most streaming reads (sound chunks and packs) in flight at once: a
/// browser fetches six at a time from one origin over HTTP/1.1, and the
/// game's own requests need the rest.
pub const MAX_FETCHES: usize = 4;

/// `og.StreamState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Unloaded = 0,
    Queued = 1,
    Loading = 2,
    Resident = 3,
    Failed = 4,
}

impl State {
    fn name(self) -> &'static str {
        match self {
            State::Unloaded => "unloaded",
            State::Queued => "queued",
            State::Loading => "loading",
            State::Resident => "resident",
            State::Failed => "failed",
        }
    }
}

/// The event's schema name; the engine registers it from [`StreamEvent`]
/// (its group is a `str64`), not from the IDL's declaration.
pub(crate) const EVENT: &str = "og.StreamEvent";

/// `og.StreamEvent`: a group changed state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(name = "og.StreamEvent")]
pub struct StreamEvent {
    /// The group's name.
    pub group: Str64,
    /// Its new state (`og.StreamState`).
    pub state: u32,
}

pub(crate) struct Group {
    desc: ManifestStream,
    /// The bundle directory its pack is under.
    dir: String,
    state: State,
    /// og.stream.request asked for it (until og.stream.release).
    requested: bool,
    /// A distance trigger's camera is within reach (kept until past unload).
    near: bool,
    wanted: bool,
    /// The pack's files while resident: path under `cooked/` to bytes.
    files: BTreeMap<String, Vec<u8>>,
    reason: String,
    /// The tick of its last change, and how often it loaded.
    since: u64,
    loads: u32,
}

/// One loaded asset of a group: its stand-in's handles and what to put back.
enum Slot {
    Model {
        id: String,
        /// Per primitive, the proxy's mesh handle and geometry.
        meshes: Vec<(MeshHandle, MeshData)>,
        /// Per material, the proxy's handle and material (as drawn: a
        /// physical material has no glTF shape).
        materials: Vec<(MaterialHandle, StandardMaterial)>,
        /// The full model's textures, made each time it loads and removed
        /// when it unloads.
        textures: Vec<TextureHandle>,
    },
    Texture {
        id: String,
        handle: TextureHandle,
    },
    Sound {
        id: String,
        handle: u64,
    },
}

/// A loaded asset of a group with no stand-in, by its load index: let go
/// when the group unloads, added again when it is resident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Held {
    Environment(usize),
    TextureArray(usize),
    Palette(usize),
    Look(usize),
    Lut(usize),
}

impl Slot {
    fn id(&self) -> &str {
        match self {
            Slot::Model { id, .. } | Slot::Texture { id, .. } | Slot::Sound { id, .. } => id,
        }
    }
}

/// What a streaming read is for.
#[derive(Clone, Debug, PartialEq)]
enum Need {
    /// Chunk `n` of a sound cooked in chunks (`handle` in the bank), which
    /// a playhead reaches in `seconds` (0: it is playing).
    Chunk {
        handle: u64,
        id: String,
        n: u32,
        seconds: f64,
    },
    /// A loading group's pack.
    Pack { group: String, priority: i32 },
}

/// A read wanted now: the file under the bundle and what it is for.
#[derive(Clone, Debug)]
struct Wanted {
    path: String,
    need: Need,
}

impl Wanted {
    fn json(&self) -> Value {
        match &self.need {
            Need::Chunk { id, n, seconds, .. } => json!({
                "path": self.path, "kind": "chunk", "sound": id, "chunk": n, "seconds": seconds,
            }),
            Need::Pack { group, priority } => json!({
                "path": self.path, "kind": "pack", "group": group, "priority": priority,
            }),
        }
    }
}

/// The streaming reads ([`fetch`]).
#[derive(Default)]
struct Fetches {
    /// What the host is still fetching (it answered `Pending`), by path.
    in_flight: BTreeMap<String, Need>,
    /// What waited for a free slot at the last pass, in the order it is read.
    queued: Vec<Wanted>,
    /// Reads started and finished (landed or failed).
    started: u64,
    finished: u64,
    /// The most in flight at once.
    peak: usize,
}

/// An engine's streaming groups (in its assets).
#[derive(Default)]
pub(crate) struct Streams {
    opened: bool,
    budget: Option<u64>,
    groups: Vec<Group>,
    /// Per group name, its loaded assets and whether each is upgraded.
    slots: BTreeMap<String, Vec<(Slot, bool)>>,
    /// Assets of unloaded groups the frame still used when their group
    /// unloaded, let go once unused.
    retained: Vec<Retained>,
    peak: u64,
    log: Vec<Value>,
    fetch: Fetches,
}

/// An asset kept for the frame after its group unloaded.
struct Retained {
    group: String,
    held: Held,
    id: String,
    /// What it takes on the GPU (counted in the budget while kept).
    bytes: u64,
}

thread_local! {
    static EVENTS: RefCell<IdMap<u32, Vec<StreamEvent>>> = const { RefCell::new(IdMap::new()) };
}

impl Streams {
    fn budget(&self) -> u64 {
        self.budget.unwrap_or(DEFAULT_BUDGET)
    }

    /// The memory the admitted groups take (loading ones count already),
    /// and what is kept for the frame of groups no longer admitted.
    fn used(&self) -> u64 {
        self.groups
            .iter()
            .filter(|g| matches!(g.state, State::Loading | State::Resident))
            .map(|g| g.desc.memory)
            .sum::<u64>()
            + self.kept()
    }

    /// The bytes of the assets kept for the frame whose group is not
    /// admitted (once it is again, the group's memory counts them).
    fn kept(&self) -> u64 {
        self.retained
            .iter()
            .filter(|r| {
                !self.find(&r.group).is_some_and(|i| {
                    matches!(self.groups[i].state, State::Loading | State::Resident)
                })
            })
            .map(|r| r.bytes)
            .sum()
    }

    /// The bytes `group` keeps for the frame.
    fn kept_by(&self, group: &str) -> u64 {
        self.retained
            .iter()
            .filter(|r| r.group == group)
            .map(|r| r.bytes)
            .sum()
    }

    fn find(&self, name: &str) -> Option<usize> {
        self.groups.iter().position(|g| g.desc.name == name)
    }

    /// A pack file of a resident group, by its path under `cooked/`.
    pub(crate) fn file(&self, group: &str, file: &str) -> Result<&[u8], String> {
        let g = self
            .find(group)
            .map(|i| &self.groups[i])
            .ok_or_else(|| format!("no streaming group '{group}'"))?;
        g.files.get(file).map(Vec::as_slice).ok_or_else(|| {
            format!(
                "{file} is in streaming group '{group}', which is {}",
                g.state.name()
            )
        })
    }

    fn resident(&self, group: &str) -> bool {
        self.find(group)
            .is_some_and(|i| self.groups[i].state == State::Resident)
    }
}

/// Reads the game bundle's groups once it is open.
fn open(eng: &Engine) {
    let mut a = eng.assets.borrow_mut();
    if a.stream.opened {
        return;
    }
    let Some(i) = a.default_bundle else { return };
    let super::assets::Payload::Bundle { bundle, dir } = &a.entries[i].payload else {
        return;
    };
    let mut groups: Vec<Group> = bundle
        .streams()
        .iter()
        .map(|d| Group {
            desc: d.clone(),
            dir: dir.clone(),
            state: State::Unloaded,
            requested: false,
            near: false,
            wanted: false,
            files: BTreeMap::new(),
            reason: String::new(),
            since: 0,
            loads: 0,
        })
        .collect();
    og_core::sort::sort_by(&mut groups, |a, b| a.desc.name.cmp(&b.desc.name));
    a.stream.groups = groups;
    a.stream.opened = true;
}

fn tick_of(eng: &Engine) -> u64 {
    eng.world().map(|w| w.time().tick).unwrap_or(0)
}

fn camera(eng: &Engine) -> Option<glam::Vec3> {
    let v = super::api::resource_value(eng, "og.Camera").ok()?;
    let p = &v["position"];
    Some(glam::Vec3::new(
        p["x"].as_f64()? as f32,
        p["y"].as_f64()? as f32,
        p["z"].as_f64()? as f32,
    ))
}

fn set_state(s: &mut Streams, engine: u32, i: usize, state: State, tick: u64, reason: &str) {
    let g = &mut s.groups[i];
    if g.state == state {
        // The same state for another reason: no change to send.
        if g.reason != reason {
            g.reason = reason.to_string();
        }
        return;
    }
    g.state = state;
    g.reason = reason.to_string();
    g.since = tick;
    let name = g.desc.name.clone();
    s.log
        .push(json!({ "tick": tick, "group": name, "state": state.name(), "reason": reason }));
    if s.log.len() > MAX_LOG {
        s.log.remove(0);
    }
    if let Ok(group) = Str64::new(&name) {
        EVENTS.with(|e| {
            e.borrow_mut().entry(engine).or_default().push(StreamEvent {
                group,
                state: state as u32,
            })
        });
    }
}

fn push(eng: &Engine, group: &str, slot: Slot) {
    open(eng);
    eng.assets
        .borrow_mut()
        .stream
        .slots
        .entry(group.to_string())
        .or_default()
        .push((slot, false));
    refresh(eng);
}

// ── the system, the probe and the IDL ───────────────────────────────────

/// Registers og.StreamEvent and the og.stream system (front).
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
    world
        .register::<StreamEvent>()
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            SystemDesc::new(STREAM_SYSTEM).write::<StreamEvent>(),
            move |ctx| {
                let events = EVENTS.with(|e| e.borrow_mut().remove(&engine_id).unwrap_or_default());
                for ev in events {
                    ctx.send(ev)?;
                }
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
}

pub(crate) fn retire(engine: u32) {
    EVENTS.with(|e| e.borrow_mut().remove(&engine));
}

fn trigger_json(t: &StreamTrigger) -> Value {
    match t {
        StreamTrigger::Manual => json!({ "kind": "manual" }),
        StreamTrigger::Distance {
            anchor,
            radius,
            unload,
        } => json!({ "kind": "distance", "anchor": anchor, "radius": radius, "unload": unload }),
        StreamTrigger::Volume(e) => json!({ "kind": "volume", "entity": e }),
    }
}

fn status_json(g: &Group) -> Value {
    json!({
        "name": g.desc.name,
        "scene": g.desc.scene,
        "state": g.state.name(),
        "wanted": g.wanted,
        "requested": g.requested,
        "priority": g.desc.priority,
        "trigger": trigger_json(&g.desc.trigger)["kind"].clone(),
        "memory": g.desc.memory as f64,
        "bytes": g.desc.bytes as f64,
        "assets": g.desc.assets,
        "reason": g.reason,
        "since": g.since as f64,
        "loads": g.loads,
    })
}

/// [`status_json`] with the bytes the group keeps for the frame.
fn group_json(s: &Streams, g: &Group) -> Value {
    let mut j = status_json(g);
    j["kept"] = json!(s.kept_by(&g.desc.name) as f64);
    j
}

fn budget_json(s: &Streams) -> Value {
    json!({
        "budget": s.budget() as f64,
        "used": s.used() as f64,
        "kept": s.kept() as f64,
        "peak": s.peak as f64,
        "groups": s.groups.iter().map(|g| group_json(s, g)).collect::<Vec<_>>(),
    })
}

/// The streaming budget for the probe's `memory`: the budget, what the
/// admitted groups take and its peak, in bytes.
pub(crate) fn memory(eng: &Engine) -> Value {
    let Ok(a) = eng.assets.try_borrow() else {
        return Value::Null;
    };
    let s = &a.stream;
    json!({ "budget": s.budget() as f64, "used": s.used() as f64, "peak": s.peak as f64 })
}

/// `dump_stream`'s JSON: the budget, the memory used and its peak, every
/// group (its trigger, state, why and since when), the recent events and
/// the terrain streamer's tiles.
pub(crate) fn dump(engine_id: u32) -> Value {
    let Ok(eng) = engine(engine_id) else {
        return json!({ "error": "no engine" });
    };
    open(&eng);
    let a = eng.assets.borrow();
    let s = &a.stream;
    let mut v = budget_json(s);
    v["groups"] = json!(
        s.groups
            .iter()
            .map(|g| {
                let mut j = group_json(s, g);
                j["trigger"] = trigger_json(&g.desc.trigger);
                j["file"] = json!(g.desc.file);
                j["slots"] = json!(s.slots.get(&g.desc.name).map_or(0, Vec::len));
                j["retained"] = json!(
                    s.retained
                        .iter()
                        .filter(|r| r.group == g.desc.name)
                        .map(|r| json!({ "id": r.id, "bytes": r.bytes as f64 }))
                        .collect::<Vec<_>>()
                );
                j
            })
            .collect::<Vec<_>>()
    );
    v["events"] = json!(s.log);
    v["fetch"] = fetch_json(s);
    drop(a);
    v["terrain"] = super::terrain::stream_json(engine_id);
    v
}

fn with_group<T>(
    world: u32,
    group: &str,
    f: impl FnOnce(&Engine, &mut Streams, usize) -> T,
) -> Result<T, String> {
    let eng = engine(world)?;
    open(&eng);
    let mut a = eng.assets.borrow_mut();
    let s = &mut a.stream;
    let i = s.find(group).ok_or_else(|| {
        let names: Vec<&str> = s.groups.iter().map(|g| g.desc.name.as_str()).collect();
        if names.is_empty() {
            format!("no streaming group '{group}': the game's scenes declare none (or its bundle is not open yet)")
        } else {
            format!("no streaming group '{group}'; the scenes declare {}", names.join(", "))
        }
    })?;
    Ok(f(&eng, s, i))
}

/// `og.stream.status`.
pub fn og_stream_status(world: u32, group: String) -> String {
    match with_group(world, &group, |_, s, i| group_json(s, &s.groups[i])) {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

/// `og.stream.request`: wanted until released (loaded at the next tick
/// boundary, within the budget).
pub fn og_stream_request(world: u32, group: String) -> String {
    match with_group(world, &group, |_, s, i| s.groups[i].requested = true) {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// `og.stream.release`: no longer asked for (its trigger may still want it).
pub fn og_stream_release(world: u32, group: String) -> String {
    match with_group(world, &group, |_, s, i| s.groups[i].requested = false) {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

/// `og.stream.budget`.
pub fn og_stream_budget(world: u32) -> String {
    match engine(world) {
        Ok(eng) => {
            open(&eng);
            ok(budget_json(&eng.assets.borrow().stream))
        }
        Err(e) => refuse(e),
    }
}

/// `og.stream.setBudget`: the memory the resident groups may take, from
/// the next tick boundary (a lower one evicts the lowest-priority groups).
pub fn og_stream_set_budget(world: u32, bytes: f64) -> String {
    if !(bytes.is_finite() && bytes >= 0.0) {
        return refuse(format!("budget {bytes}: give a size in bytes, 0 or more"));
    }
    match engine(world) {
        Ok(eng) => {
            let mut a = eng.assets.borrow_mut();
            a.stream.budget = Some(bytes as u64);
            ok_void()
        }
        Err(e) => refuse(e),
    }
}

/// The chunks of the sounds cooked in chunks: at a tick boundary and after
/// the ticks (before the host's audio pump), every chunk the playheads
/// want (`og_audio::chunks_wanted`: from half a second behind each to four
/// seconds ahead, and a loop's start) is read and put in the bank, and the
/// others are dropped, so a long track is never resident whole. Natively a
/// read completes at once; on the web it is fetched like a pack and put in
/// place at a later call (the four seconds ahead are for that fetch), the
/// chunk a playhead needs soonest first ([`fetch`]). Nothing here reaches
/// the world: the tick side times a sound by its length alone.
pub(crate) fn sound_chunks(eng: &Engine) {
    fetch(eng);
}
