//! Streaming groups (IDL `stream`, phase 3).
//!
//! A scene's `stream` blocks name assets that load and unload together; the
//! cook packs each group's files into `cooked/stream/<name>.ogb` and lists
//! the group in the manifest. Here, at every tick boundary, each group of
//! every open bundle is wanted or not (the camera within its distance, kept
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
//! sounds cooked in chunks and of the streamed videos first, the one a
//! playhead or a player needs soonest first (`og_audio::chunk_distances`,
//! og-video's `chunks_wanted`), then the loading groups' packs by
//! priority (then name). At most [`MAX_FETCHES`] reads are in flight: a
//! read the host answers `Pending` (the web host's fetch) holds a slot
//! until it lands, and what does not get a slot waits in order. Natively a
//! read completes at once, so nothing waits and only the order shows.
//! `dump_stream`'s `fetch` lists what is in flight and what waits.

pub(crate) mod cell_groups;
pub(crate) mod cell_store;
pub(crate) mod cells;
mod kept;
mod loading;
pub(crate) mod mips;
mod residency;

use crate::{
    present::environment, present::extract, present::fx, present::geometry, present::look,
    world::assets,
};
pub(crate) use cell_groups::resolve_scenes;
use loading::fetch_json;
pub(crate) use loading::{fetch, refresh, update};
pub(crate) use residency::{
    add_model, add_sound, add_texture, placeholder, replace_texture, resolve,
};
use residency::{downgrade, release, retry_retained, upgrade};

use crate::engine::{Engine, engine, hooks};
use crate::host::wire::{FromJson, StreamBudget, StreamState, StreamStatus, ToJson, record};
use og_bundle::runtime::{CookedSound, ManifestStream, StreamTrigger};
use og_core::IdMap;
use og_core::{Reflect, Str64};
use og_ecs::{SystemDesc, World};
use og_render::{
    MaterialHandle, MeshData, MeshHandle, StandardMaterial, TextureData, TextureHandle,
    TextureSource,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

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
        self.wire().as_str()
    }

    /// The IDL's StreamState.
    fn wire(self) -> StreamState {
        match self {
            State::Unloaded => StreamState::Unloaded,
            State::Queued => StreamState::Queued,
            State::Loading => StreamState::Loading,
            State::Resident => StreamState::Resident,
            State::Failed => StreamState::Failed,
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
    /// The asset entry of the bundle that declared this group.
    bundle: usize,
    /// The bundle directory its pack is under.
    dir: String,
    state: State,
    /// Presentation only: worker completion never changes state or events.
    ready: bool,
    /// og.stream.request asked for it (until og.stream.release).
    requested: bool,
    /// A distance trigger's camera is within reach (kept until past unload).
    near: bool,
    wanted: bool,
    /// Device-priced reservation, learned from the pack before admission.
    /// Kept across unloads; reading a pack does not make its assets resident.
    memory: Option<u64>,
    /// Its measured assets have a reservation; metadata reads have none.
    reserved: bool,
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
        /// Per primitive, the proxy's mesh handle and (while upgraded)
        /// saved geometry. When unloaded the scene owns the only copy.
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
    /// A streamed texture's whole mip chain (`mips`), for the scene
    /// texture of that slot.
    Mips { id: String, handle: u32 },
    /// Chunk `n` (`stream`, its id) of the stream of video `video` (by
    /// index), which a player reaches in `seconds` (0: it shows it now).
    Video {
        video: usize,
        stream: String,
        n: u32,
        seconds: f64,
    },
}

impl Need {
    /// How soon a chunk is needed (sound and video chunks, read soonest
    /// first); `None` for what is read after them.
    fn seconds(&self) -> Option<f64> {
        match self {
            Need::Chunk { seconds, .. } | Need::Video { seconds, .. } => Some(*seconds),
            Need::Pack { .. } | Need::Mips { .. } => None,
        }
    }
}

/// A chunk of a streamed video a player wants (crate::bind::video's
/// `wanted_chunks`): its file, the video's index, the chunk's id and
/// index, and the seconds until a player reaches it.
pub(crate) struct VideoChunkWant {
    pub path: String,
    pub video: usize,
    pub stream: String,
    pub n: u32,
    pub seconds: f64,
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
            Need::Mips { id, .. } => json!({
                "path": self.path, "kind": "mips", "texture": id,
            }),
            Need::Video {
                stream, n, seconds, ..
            } => json!({
                "path": self.path, "kind": "video", "chunk": stream, "n": n, "seconds": seconds,
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
    /// The world cells' history by cell entity bits, for `dump_stream`
    /// (0: refusals of no one cell).
    pub(crate) cells: BTreeMap<u64, cells::CellLog>,
    /// Per cell group, the load indexes of the sub-scenes its pack holds.
    scenes: BTreeMap<String, Vec<usize>>,
    /// The ticks held for a cell group's pack ([`cell_groups::gate`]).
    holds: cell_groups::Holds,
    /// The textures streamed by their mips.
    mips: mips::Mips,
    /// The world tick whose boundary last put a group in place. Ordinary
    /// placement is limited to one per tick; a cell hold may admit its
    /// group too so it never waits for the tick it is holding to advance.
    placed: Option<u64>,
}

/// An asset kept for the frame after its group unloaded.
struct Retained {
    group: String,
    held: Held,
    id: String,
    /// What it takes on the GPU (counted in the budget while kept).
    bytes: u64,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// og.stream on one engine: the group transitions made since the last
/// tick, which its system sends as og.StreamEvent at the next one.
#[derive(Default)]
pub(crate) struct ModuleState(Arc<Mutex<Vec<StreamEvent>>>);

impl Streams {
    /// Register a ready bundle atomically: a name belongs to one open bundle.
    pub(crate) fn add_bundle(
        &mut self,
        index: usize,
        bundle: &og_bundle::runtime::Bundle,
        dir: &str,
    ) -> Result<(), String> {
        let mut added = Vec::new();
        for desc in bundle.streams() {
            if let Some(i) = self.find(&desc.name) {
                let owner = &self.groups[i];
                if owner.dir == dir {
                    continue;
                }
                return Err(format!(
                    "streaming group '{}' is declared by both '{}' and '{}'; group names must be unique among open bundles",
                    desc.name, owner.dir, dir
                ));
            }
            added.push(Group {
                desc: desc.clone(),
                bundle: index,
                dir: dir.to_string(),
                state: State::Unloaded,
                ready: false,
                requested: false,
                near: false,
                wanted: false,
                memory: None,
                reserved: false,
                files: BTreeMap::new(),
                reason: String::new(),
                since: 0,
                loads: 0,
            });
        }
        self.groups.extend(added);
        og_core::sort::sort_by(&mut self.groups, |a, b| a.desc.name.cmp(&b.desc.name));
        Ok(())
    }

    fn budget(&self) -> u64 {
        self.budget.unwrap_or(DEFAULT_BUDGET)
    }

    /// The memory the admitted groups take (loading ones count already),
    /// and what is kept for the frame of groups no longer admitted.
    fn used(&self) -> u64 {
        self.groups
            .iter()
            .filter(|g| g.reserved)
            .map(|g| g.memory.unwrap_or(g.desc.memory))
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

fn tick_of(eng: &Engine) -> u64 {
    eng.world().map(|w| w.time().tick).unwrap_or(0)
}

fn camera(eng: &Engine) -> Option<glam::Vec3> {
    let v = crate::host::api::resource_value(eng, "og.Camera").ok()?;
    // A NaN member (null in the row) is no camera position, as before.
    Some(record::Camera::from_json(&v).ok()?.position).filter(|p| !p.is_nan())
}

fn set_state(
    s: &mut Streams,
    events: &ModuleState,
    i: usize,
    state: State,
    tick: u64,
    reason: &str,
) {
    let g = &mut s.groups[i];
    if !matches!(state, State::Loading | State::Resident) {
        g.reserved = false;
    }
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
        lock(&events.0).push(StreamEvent {
            group,
            state: state as u32,
        });
    }
}

fn push(eng: &Engine, group: &str, slot: Slot) {
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
pub(crate) fn mount(world: &mut World, mount: crate::engine::Mount<'_>) -> Result<(), String> {
    let pending = mount.modules.stream.0.clone();
    world
        .register::<StreamEvent>()
        .map_err(|e| e.reason().to_string())?;
    world
        .add_system(
            SystemDesc::new(STREAM_SYSTEM).write::<StreamEvent>(),
            move |ctx| {
                let events = std::mem::take(&mut *lock(&pending));
                for ev in events {
                    ctx.send(ev)?;
                }
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
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
        StreamTrigger::Cell => json!({ "kind": "cell" }),
    }
}

/// The bytes of a group's pack received: all of it once it landed (or
/// the group is resident), what the host says while it fetches it, 0
/// before it is admitted.
pub(crate) fn received(g: &Group) -> f64 {
    let all = g.desc.bytes as f64;
    match g.state {
        State::Resident => all,
        State::Loading if !g.files.is_empty() => all,
        State::Loading => hooks()
            .and_then(|h| h.read_progress(&format!("{}/{}", g.dir, g.desc.file)))
            .map_or(0.0, |(r, _)| r.min(all)),
        _ => 0.0,
    }
}

/// A group's status, with the bytes it keeps for the frame.
fn group_status(eng: &Engine, a: &assets::Assets, g: &Group) -> StreamStatus {
    let s = &a.stream;
    StreamStatus {
        name: g.desc.name.clone(),
        scene: g.desc.scene.clone(),
        state: g.state.wire(),
        ready: g.state == State::Resident
            && g.ready
            && cell_groups::ready(a, &eng.render.borrow(), &g.desc.name),
        wanted: g.wanted,
        requested: g.requested,
        priority: g.desc.priority,
        trigger: trigger_json(&g.desc.trigger)["kind"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        memory: g.memory.unwrap_or(g.desc.memory) as f64,
        bytes: g.desc.bytes as f64,
        received: received(g),
        progress: if g.state == State::Resident || g.desc.bytes == 0 {
            if g.state == State::Resident { 1.0 } else { 0.0 }
        } else {
            (received(g) / g.desc.bytes as f64).clamp(0.0, 1.0) as f32
        },
        assets: g.desc.assets.clone(),
        reason: g.reason.clone(),
        since: g.since as f64,
        loads: g.loads,
        kept: s.kept_by(&g.desc.name) as f64,
    }
}

fn stream_budget(eng: &Engine, a: &assets::Assets) -> StreamBudget {
    let s = &a.stream;
    StreamBudget {
        budget: s.budget() as f64,
        used: s.used() as f64,
        kept: s.kept() as f64,
        peak: s.peak as f64,
        groups: s.groups.iter().map(|g| group_status(eng, a, g)).collect(),
    }
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
/// group (its trigger, state, why and since when), the recent events, the
/// terrain streamer's tiles and the streamed videos' chunks.
pub(crate) fn dump(engine_id: u32) -> Value {
    let Ok(eng) = engine(engine_id) else {
        return json!({ "error": "no engine" });
    };
    let a = eng.assets.borrow();
    let s = &a.stream;
    let mut v = stream_budget(&eng, &a).to_json();
    v["groups"] = json!(
        s.groups
            .iter()
            .map(|g| {
                let mut j = group_status(&eng, &a, g).to_json();
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
    v["mips"] = mips::dump(&s.mips);
    drop(a);
    v["terrain"] = crate::bind::terrain::stream_json(engine_id);
    v["videos"] = crate::bind::video::stream_json(&eng);
    v["cells"] = cells::dump(&eng);
    let (hold, holds) =
        cell_groups::dump(&eng.assets.borrow().stream, crate::author::probe::now_ms());
    v["hold"] = hold;
    v["holds"] = holds;
    v
}

fn with_group<T>(
    world: u32,
    group: &str,
    f: impl FnOnce(&Engine, &mut assets::Assets, usize) -> T,
) -> Result<T, String> {
    let eng = engine(world)?;
    let mut a = eng.assets.borrow_mut();
    let s = &mut a.stream;
    let i = s.find(group).ok_or_else(|| {
        let names: Vec<&str> = s.groups.iter().map(|g| g.desc.name.as_str()).collect();
        if names.is_empty() {
            format!("no streaming group '{group}': no ready open bundle declares one (or its bundle is not open yet)")
        } else {
            format!("no streaming group '{group}'; the open bundles declare {}", names.join(", "))
        }
    })?;
    Ok(f(&eng, &mut a, i))
}

/// The status of every group scene `scene` (an asset id) declares, by name.
pub(crate) fn scene_groups(eng: &Engine, scene: &str) -> Vec<StreamStatus> {
    let a = eng.assets.borrow();
    let s = &a.stream;
    s.groups
        .iter()
        .filter(|g| og_bundle::runtime::asset_id(&g.desc.scene) == scene)
        .map(|g| group_status(eng, &a, g))
        .collect()
}

/// `og.stream.status`.
pub fn og_stream_status(
    world: u32,
    group: String,
) -> Result<crate::host::wire::StreamStatus, String> {
    with_group(world, &group, |eng, a, i| {
        group_status(eng, a, &a.stream.groups[i])
    })
}

/// `og.stream.request`: wanted until released (loaded at the next tick
/// boundary, within the budget).
pub fn og_stream_request(world: u32, group: String) -> Result<(), String> {
    match with_group(world, &group, |_, a, i| a.stream.groups[i].requested = true) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

/// `og.stream.release`: no longer asked for (its trigger may still want it).
pub fn og_stream_release(world: u32, group: String) -> Result<(), String> {
    match with_group(world, &group, |_, a, i| {
        a.stream.groups[i].requested = false
    }) {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

/// `og.stream.budget`.
pub fn og_stream_budget(world: u32) -> Result<crate::host::wire::StreamBudget, String> {
    match engine(world) {
        Ok(eng) => Ok(stream_budget(&eng, &eng.assets.borrow())),
        Err(e) => Err(e),
    }
}

/// `og.stream.setBudget`: the memory the resident groups may take, from
/// the next tick boundary (a lower one evicts the lowest-priority groups).
pub fn og_stream_set_budget(world: u32, bytes: f64) -> Result<(), String> {
    if !(bytes.is_finite() && bytes >= 0.0) {
        return Err(format!("budget {bytes}: give a size in bytes, 0 or more"));
    }
    match engine(world) {
        Ok(eng) => {
            let mut a = eng.assets.borrow_mut();
            a.stream.budget = Some(bytes as u64);
            Ok(())
        }
        Err(e) => Err(e),
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
