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

use super::engine::{Engine, engine, hooks};
use super::{ok, ok_void, refuse};
use og_cook::runtime::{CookedSound, ManifestStream, StreamTrigger};
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
    static EVENTS: RefCell<BTreeMap<u32, Vec<StreamEvent>>> = const { RefCell::new(BTreeMap::new()) };
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
    groups.sort_by(|a, b| a.desc.name.cmp(&b.desc.name));
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

/// At every tick boundary: what is wanted, what is admitted, what loads and
/// unloads.
pub(crate) fn update(eng: &Engine) {
    open(eng);
    if eng.assets.borrow().stream.groups.is_empty() {
        return;
    }
    let tick = tick_of(eng);
    retry_retained(eng);
    let cam = camera(eng);
    // Each volume group's box, in its entity's space (the camera inside it).
    let volumes: BTreeMap<String, bool> = {
        let a = eng.assets.borrow();
        let names: Vec<String> = a
            .stream
            .groups
            .iter()
            .filter_map(|g| match &g.desc.trigger {
                StreamTrigger::Volume(e) => Some(e.clone()),
                _ => None,
            })
            .collect();
        drop(a);
        let mut out = BTreeMap::new();
        if !names.is_empty()
            && let (Some(c), Ok(world)) = (cam, eng.world())
        {
            let mut h = super::extract::Hierarchy::of(world);
            for n in names {
                let bits = eng.state.borrow().names.get(&n).copied();
                let inside = bits.is_some_and(|b| {
                    let local = h.world_of(b).inverse().transform_point3(c);
                    local.abs().max_element() <= 0.5
                });
                out.insert(n, inside);
            }
        }
        out
    };
    let mut unload = Vec::new();
    {
        let mut a = eng.assets.borrow_mut();
        let s = &mut a.stream;
        for g in &mut s.groups {
            let trigger = match &g.desc.trigger {
                StreamTrigger::Manual => false,
                StreamTrigger::Distance {
                    anchor,
                    radius,
                    unload,
                } => {
                    let d = cam.map(|c| c.distance(glam::Vec3::from(*anchor)));
                    g.near = match d {
                        Some(d) if d <= *radius => true,
                        Some(d) if d > *unload => false,
                        Some(_) => g.near,
                        None => false,
                    };
                    g.near
                }
                StreamTrigger::Volume(e) => volumes.get(e).copied().unwrap_or(false),
            };
            g.wanted = g.requested || trigger;
        }
        // Unwanted groups go; a failed one is tried again once wanted again.
        for i in 0..s.groups.len() {
            if !s.groups[i].wanted && s.groups[i].state != State::Unloaded {
                if matches!(s.groups[i].state, State::Loading | State::Resident) {
                    unload.push(s.groups[i].desc.name.clone());
                }
                set_state(s, eng.id, i, State::Unloaded, tick, "");
            }
        }
    }
    // They let go before admission, so what they keep for the frame counts.
    for name in std::mem::take(&mut unload) {
        downgrade(eng, &name);
    }
    {
        let mut a = eng.assets.borrow_mut();
        let s = &mut a.stream;
        // Admission: the wanted groups by priority (then name) within the budget.
        let budget = s.budget();
        let mut order: Vec<usize> = (0..s.groups.len())
            .filter(|&i| s.groups[i].wanted)
            .collect();
        order.sort_by(|&x, &y| {
            let (gx, gy) = (&s.groups[x], &s.groups[y]);
            gy.desc
                .priority
                .cmp(&gx.desc.priority)
                .then(gx.desc.name.cmp(&gy.desc.name))
        });
        let admitted =
            |s: &Streams, i: usize| matches!(s.groups[i].state, State::Loading | State::Resident);
        // A lowered budget evicts the lowest admitted groups first.
        for &i in order.iter().rev() {
            if s.used() <= budget {
                break;
            }
            if admitted(s, i) {
                unload.push(s.groups[i].desc.name.clone());
                set_state(
                    s,
                    eng.id,
                    i,
                    State::Queued,
                    tick,
                    "evicted to keep within the budget",
                );
            }
        }
        for (k, &i) in order.iter().enumerate() {
            if admitted(s, i) {
                continue;
            }
            // A pack that could not be read is not read again while wanted.
            if s.groups[i].state == State::Failed && !s.groups[i].reason.starts_with("needs ") {
                continue;
            }
            let need = s.groups[i].desc.memory;
            if need > budget {
                let why = format!("needs {need} B, more than the budget of {budget} B");
                set_state(s, eng.id, i, State::Failed, tick, &why);
                continue;
            }
            // Room is made from the lowest-priority admitted groups below it.
            let mut free = budget - s.used().min(budget);
            let mut evict = Vec::new();
            for &j in order[k + 1..].iter().rev() {
                if free >= need {
                    break;
                }
                if admitted(s, j) && s.groups[j].desc.priority < s.groups[i].desc.priority {
                    free += s.groups[j].desc.memory;
                    evict.push(j);
                }
            }
            if free < need {
                set_state(
                    s,
                    eng.id,
                    i,
                    State::Queued,
                    tick,
                    "waiting for room in the budget",
                );
                continue;
            }
            for j in evict {
                unload.push(s.groups[j].desc.name.clone());
                let why = format!("evicted for '{}'", s.groups[i].desc.name);
                set_state(s, eng.id, j, State::Queued, tick, &why);
            }
            set_state(s, eng.id, i, State::Loading, tick, "");
            let used = s.used();
            s.peak = s.peak.max(used);
        }
    }
    for name in unload {
        downgrade(eng, &name);
    }
    // Loading groups read their packs (after the sound chunks a playhead
    // needs); one group is put in place per update.
    fetch(eng);
    let ready: Option<String> = {
        let a = eng.assets.borrow();
        a.stream
            .groups
            .iter()
            .find(|g| g.state == State::Loading && !g.files.is_empty())
            .map(|g| g.desc.name.clone())
    };
    if let Some(name) = ready {
        match upgrade(eng, &name) {
            Ok(true) => {
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                if let Some(i) = s.find(&name) {
                    s.groups[i].loads += 1;
                    set_state(s, eng.id, i, State::Resident, tick, "");
                }
            }
            // A texture still being transcoded: the next update finishes it.
            Ok(false) => {}
            Err(e) => {
                downgrade(eng, &name);
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                if let Some(i) = s.find(&name) {
                    s.groups[i].files.clear();
                    set_state(s, eng.id, i, State::Failed, tick, &e);
                }
            }
        }
    }
    refresh(eng);
}

/// Upgrades the assets loaded after their group became resident.
pub(crate) fn refresh(eng: &Engine) {
    let names: Vec<String> = {
        let a = eng.assets.borrow();
        a.stream
            .slots
            .iter()
            .filter(|(g, slots)| a.stream.resident(g) && slots.iter().any(|(_, up)| !up))
            .map(|(g, _)| g.clone())
            .collect()
    };
    for name in names {
        let _ = upgrade(eng, &name);
    }
}

/// What one slot's upgrade puts in place.
enum Full {
    Model {
        meshes: Vec<MeshData>,
        model: Box<og_cook::runtime::CookedModel>,
        textures: Vec<TextureSource>,
    },
    Texture(TextureSource, bool),
    /// The cooked sound: whole, or the head of one cooked in chunks.
    Sound(Box<CookedSound>),
}

/// A texture over its stand-in, in whichever form the host decoded it to.
fn replace_texture(
    scene: &mut og_render::RenderScene,
    h: og_render::TextureHandle,
    src: TextureSource,
) {
    match src {
        TextureSource::Rgba(d) => scene.set_texture(h, d),
        TextureSource::Encoded(u) => scene.set_texture_upload(h, u),
    };
}

fn decode_texture(
    bundle: &og_cook::runtime::Bundle,
    files: &Streams,
    group: &str,
    id: &str,
    formats: Option<&og_render::TextureFormats>,
) -> Result<Option<(TextureSource, bool)>, String> {
    let out = bundle.output(id).map_err(|e| e.to_string())?;
    let bytes = super::assets::aligned(files.file(group, &out.file)?);
    let tex = bundle
        .texture(id, bytes.bytes())
        .map_err(|e| e.to_string())?;
    let pixel = tex.is_pixel_art();
    match hooks().map(|h| h.texture(&tex, formats)) {
        Some(super::assets::Fetch::Ready(d)) => Ok(Some((d, pixel))),
        Some(super::assets::Fetch::Pending) => Ok(None),
        Some(super::assets::Fetch::Failed(e)) => Err(format!("{id}: {e}")),
        None => Err("no host is installed to decode textures".into()),
    }
}

/// Puts a resident group's assets in place of their stand-ins. `Ok(false)`:
/// a texture is still being transcoded (nothing changed yet).
fn upgrade(eng: &Engine, group: &str) -> Result<bool, String> {
    let formats = super::assets::texture_formats(eng);
    let fulls: Vec<(usize, Full)> = {
        let a = eng.assets.borrow();
        let Some(i) = a.default_bundle else {
            return Ok(true);
        };
        let super::assets::Payload::Bundle { bundle, .. } = &a.entries[i].payload else {
            return Ok(true);
        };
        let s = &a.stream;
        let mut out = Vec::new();
        for (k, (slot, up)) in s.slots.get(group).into_iter().flatten().enumerate() {
            if *up {
                continue;
            }
            let id = slot.id();
            let out_file = &bundle.output(id).map_err(|e| e.to_string())?.file;
            let bytes = super::assets::aligned(s.file(group, out_file)?);
            let full = match slot {
                Slot::Model { .. } => {
                    let model = bundle.model(id, bytes.bytes()).map_err(|e| e.to_string())?;
                    let mut textures = Vec::with_capacity(model.textures.len());
                    for t in &model.textures {
                        match decode_texture(bundle, s, group, &t.id, formats.as_ref())? {
                            Some((d, _)) => textures.push(d),
                            None => return Ok(false),
                        }
                    }
                    Full::Model {
                        meshes: model.primitives.iter().map(|p| p.mesh(0)).collect(),
                        model: Box::new(model),
                        textures,
                    }
                }
                Slot::Texture { .. } => {
                    match decode_texture(bundle, s, group, id, formats.as_ref())? {
                        Some((d, pixel)) => Full::Texture(d, pixel),
                        None => return Ok(false),
                    }
                }
                Slot::Sound { .. } => {
                    let sound = bundle.sound(id, bytes.bytes()).map_err(|e| e.to_string())?;
                    Full::Sound(Box::new(sound))
                }
            };
            out.push((k, full));
        }
        out
    };
    let link = og_audio::link(eng.id);
    let mut a = eng.assets.borrow_mut();
    let mut rs = eng.render.borrow_mut();
    // Sounds cooked in chunks put in place: their stand-in's entry learns
    // its chunks (the playheads ask for them from now on), after the loop.
    let mut chunked: Vec<CookedSound> = Vec::new();
    let Some(slots) = a.stream.slots.get_mut(group) else {
        return Ok(true);
    };
    for (k, full) in fulls {
        let (slot, up) = &mut slots[k];
        match (slot, full) {
            (
                Slot::Model {
                    meshes,
                    materials,
                    textures,
                    ..
                },
                Full::Model {
                    meshes: full,
                    model,
                    textures: data,
                },
            ) => {
                for (i, d) in data.into_iter().enumerate() {
                    match textures.get(i) {
                        Some(&h) => {
                            replace_texture(&mut rs.scene, h, d);
                        }
                        None => textures.push(rs.scene.add_texture_source(d)),
                    }
                }
                for ((h, _), m) in meshes.iter().zip(full) {
                    rs.scene.set_mesh(*h, m);
                }
                for (i, (h, _)) in materials.iter().enumerate().take(model.materials.len()) {
                    if model.materials[i].physical.present == 0 {
                        rs.scene.set_material(*h, model.material(i, textures));
                    } else {
                        rs.scene
                            .set_standard_material(*h, model.standard_material(i, textures));
                    }
                }
            }
            (Slot::Texture { handle, .. }, Full::Texture(d, pixel)) => {
                replace_texture(&mut rs.scene, *handle, d);
                if pixel {
                    rs.scene
                        .set_texture_filter(*handle, og_render::TextureFilter::Nearest);
                }
            }
            (Slot::Sound { id, handle }, Full::Sound(sound)) => {
                if let Some(l) = &link {
                    if sound.chunked() {
                        // Its first chunk now; the others are read ahead of
                        // its playheads (sound_chunks), as in the first group.
                        l.bank()
                            .replace_chunked(
                                *handle,
                                id,
                                &sound.qoa,
                                sound.meta.chunk_frames,
                                sound.meta.chunk_count,
                            )
                            .map_err(|e| e.reason().to_string())?;
                        chunked.push(*sound);
                    } else {
                        l.bank()
                            .replace_qoa(*handle, id, &sound.qoa)
                            .map_err(|e| e.reason().to_string())?;
                    }
                }
            }
            _ => {}
        }
        *up = true;
    }
    for sound in chunked {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(l) = &link
            && let Some(i) = a.sounds.iter().position(|s| s.id == sound.id)
        {
            l.bank()
                .set_chunk_loader(a.sound_handles[i], super::assets::chunk_loader(&a, &sound));
        }
        if let Some(entry) = a.sounds.iter_mut().find(|s| s.id == sound.id) {
            *entry = sound;
        }
    }
    Ok(true)
}

/// The 1x1 a streamed texture shows until its group is resident.
pub(crate) fn placeholder() -> TextureData {
    TextureData {
        width: 1,
        height: 1,
        rgba: vec![128, 128, 128, 255],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
}

/// Tells the host that `group` let go of its pack and textures.
fn release(eng: &Engine, group: &str) {
    let Some(h) = hooks() else {
        return;
    };
    let (pack, textures) = {
        let a = eng.assets.borrow();
        let Some(i) = a.stream.find(group) else {
            return;
        };
        let g = &a.stream.groups[i];
        let textures = a
            .default_bundle
            .and_then(|b| match &a.entries[b].payload {
                super::assets::Payload::Bundle { bundle, .. } => Some(
                    bundle
                        .manifest()
                        .outputs()
                        .filter(|o| o.group.as_deref() == Some(group) && o.ty == "og.Texture")
                        .map(|o| o.id.clone())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        (format!("{}/{}", g.dir, g.desc.file), textures)
    };
    // The host drops it when it lands; its slot is free now.
    eng.assets.borrow_mut().stream.fetch.in_flight.remove(&pack);
    h.release_stream(&pack, &textures);
}

/// Puts a group's stand-ins back and drops its pack (the host's too), then
/// lets go of its assets without a stand-in.
fn downgrade(eng: &Engine, group: &str) {
    put_stand_ins_back(eng, group);
    release_held(eng, group);
}

fn put_stand_ins_back(eng: &Engine, group: &str) {
    release(eng, group);
    let link = og_audio::link(eng.id);
    let mut a = eng.assets.borrow_mut();
    if let Some(i) = a.stream.find(group) {
        a.stream.groups[i].files.clear();
    }
    let mut rs = eng.render.borrow_mut();
    for (slot, up) in a.stream.slots.get_mut(group).into_iter().flatten() {
        if !*up {
            continue;
        }
        match slot {
            Slot::Model {
                meshes,
                materials,
                textures,
                ..
            } => {
                for (h, m) in meshes.iter() {
                    rs.scene.set_mesh(*h, m.clone());
                }
                for (h, m) in materials.iter() {
                    rs.scene.set_standard_material(*h, *m);
                }
                // The full model's own textures go (its materials are the
                // proxy's again, untextured): the renderer frees them and
                // the next load fills the same slots under new generations.
                for h in textures.drain(..) {
                    rs.scene.remove_texture(h);
                }
            }
            Slot::Texture { handle, .. } => {
                rs.scene.set_texture(*handle, placeholder());
            }
            Slot::Sound { handle, .. } => {
                if let Some(l) = &link {
                    l.bank().silence(*handle);
                }
            }
        }
        *up = false;
    }
}

// ── assets without a stand-in ───────────────────────────────────────────

/// Every environment map, texture array, palette and look loaded from
/// `group`'s pack that the renderer still holds.
fn held_of(eng: &Engine, group: &str) -> Vec<(Held, String)> {
    let loaded: Vec<(Held, String)> = {
        let rs = eng.render.borrow();
        let maps = rs.env.maps.iter().enumerate();
        let maps = maps.filter(|(_, (_, h))| h.is_some());
        let arrays = rs.texture_arrays.iter().enumerate();
        let arrays = arrays.filter(|(_, a)| a.handle.is_some());
        let palettes = rs.fx.look.palettes.iter().enumerate();
        let palettes = palettes.filter(|(_, (_, h))| h.is_some());
        let looks = rs.fx.look.looks.iter().enumerate();
        let looks = looks.filter(|(_, l)| l.resident);
        let luts = rs.fx.luts.iter().enumerate();
        let luts = luts.filter(|(_, l)| l.handle.is_some());
        maps.map(|(i, (id, _))| (Held::Environment(i), id.clone()))
            .chain(arrays.map(|(i, a)| (Held::TextureArray(i), a.id.clone())))
            .chain(palettes.map(|(i, (id, _))| (Held::Palette(i), id.clone())))
            .chain(looks.map(|(i, l)| (Held::Look(i), l.id.clone())))
            .chain(luts.map(|(i, l)| (Held::Lut(i), l.id.clone())))
            .collect()
    };
    let a = eng.assets.borrow();
    loaded
        .into_iter()
        .filter(|(_, id)| a.group_of(id).as_deref() == Some(group))
        .collect()
}

/// Why the frame still needs `held` at tick boundary `tick` (`None`: it
/// does not): what the last frame drew with (its sprites, overlay sprites
/// and particle emitters for an array), and the sprites queued for the
/// next one.
fn in_use(rs: &super::extract::RenderState, held: Held, tick: u64) -> Option<&'static str> {
    match held {
        Held::Environment(i) => {
            let h = rs.env.maps.get(i)?.1?;
            (rs.scene.environment_map.map(|m| m.map) == Some(h))
                .then_some("og.Environment.map lights the frame with it")
        }
        Held::TextureArray(i) => {
            let h = rs.texture_arrays.get(i)?.handle?;
            // The queue is cleared lazily: one older than the last tick is stale.
            let queued = rs.texts_tick + 1 >= tick;
            let overlay = rs.scene.overlay.iter().flat_map(|o| &o.sprite_arrays);
            let drawn = rs
                .scene
                .sprite_arrays
                .iter()
                .chain(overlay)
                .any(|s| s.array == h)
                || queued
                    && rs
                        .fx
                        .sprites
                        .iter()
                        .any(|q| q.array.map(|a| a.0) == Some(h));
            if drawn {
                return Some("a sprite draws from it");
            }
            let emitted = rs
                .scene
                .particles
                .iter()
                .any(|p| rs.scene.emitter(p.emitter).and_then(|e| e.array) == Some(h));
            if emitted {
                return Some("a particle emitter draws from it");
            }
            let slot = h.slot();
            let sampled = rs
                .materials
                .iter()
                .filter_map(|&m| rs.scene.standard_material(m))
                .any(|m| m.mapArray == slot || m.normalMapArray == slot);
            if sampled {
                return Some("a material samples it (mapArray, normalMapArray)");
            }
            let layered = rs
                .terrain
                .loaded
                .iter()
                .any(|t| t.layer_arrays.contains(&Some(i)));
            layered.then_some("a terrain's layers sample it (layers, layerNormals)")
        }
        Held::Palette(i) => {
            let h = rs.fx.look.palettes.get(i)?.1?;
            (rs.scene.stylise.quantize.and_then(|q| q.palette) == Some(h))
                .then_some("Stylise.palette quantises the frame to it")
        }
        Held::Look(i) => (rs.fx.look.applied == Some(i)).then_some("it is the look applied"),
        Held::Lut(i) => {
            let h = rs.fx.luts.get(i)?.handle?;
            (rs.scene.post.color_grading.map(|g| g.lut) == Some(h))
                .then_some("PostEffects.lut grades the frame with it")
        }
    }
}

/// What `held` takes on the GPU (a look's values take none).
fn bytes_of(rs: &super::extract::RenderState, held: Held) -> u64 {
    let scene = &rs.scene;
    match held {
        Held::Environment(i) => rs
            .env
            .maps
            .get(i)
            .and_then(|m| m.1)
            .and_then(|h| scene.environment_map_data(h))
            .map_or(0, |m| {
                4 * og_render::EnvironmentMapData::texel_count(m.size) as u64
            }),
        Held::TextureArray(i) => rs
            .texture_arrays
            .get(i)
            .and_then(|a| a.handle)
            .and_then(|h| scene.texture_array(h))
            .and_then(|d| d.info().ok())
            .map_or(0, |info| info.bytes),
        Held::Palette(_) => og_render::MAX_PALETTE_COLORS as u64 * 4,
        Held::Look(_) => 0,
        Held::Lut(i) => rs
            .fx
            .luts
            .get(i)
            .and_then(|l| l.handle)
            .and_then(|h| scene.lut(h))
            .map_or(0, |l| u64::from(l.size).pow(3) * 4),
    }
}

/// Lets `held` go: out of the renderer, its handle pending.
fn free(rs: &mut super::extract::RenderState, held: Held) {
    match held {
        Held::Environment(i) => {
            if let Some(h) = rs.env.maps.get_mut(i).and_then(|m| m.1.take()) {
                rs.scene.remove_environment_map(h);
            }
        }
        Held::TextureArray(i) => {
            if let Some(h) = rs.texture_arrays.get_mut(i).and_then(|a| a.handle.take()) {
                rs.scene.remove_texture_array(h);
            }
        }
        Held::Palette(i) => {
            if let Some(h) = rs.fx.look.palettes.get_mut(i).and_then(|p| p.1.take()) {
                rs.scene.remove_palette(h);
            }
        }
        Held::Look(i) => {
            if let Some(l) = rs.fx.look.looks.get_mut(i) {
                l.values = Vec::new();
                l.resident = false;
            }
        }
        Held::Lut(i) => {
            if let Some(h) = rs.fx.luts.get_mut(i).and_then(|l| l.handle.take()) {
                rs.scene.remove_lut(h);
            }
        }
    }
}

/// The unloaded `group`'s environment maps, texture arrays, palettes,
/// looks and LUTs: freed, or kept with a warning while the frame uses them.
fn release_held(eng: &Engine, group: &str) {
    let tick = tick_of(eng);
    for (held, id) in held_of(eng, group) {
        let (why, bytes) = {
            let mut rs = eng.render.borrow_mut();
            match in_use(&rs, held, tick) {
                Some(why) => (why, bytes_of(&rs, held)),
                None => {
                    free(&mut rs, held);
                    continue;
                }
            }
        };
        let mut a = eng.assets.borrow_mut();
        if a.stream.retained.iter().any(|r| r.held == held) {
            continue;
        }
        a.stream.retained.push(Retained {
            group: group.to_string(),
            held,
            id: id.clone(),
            bytes,
        });
        drop(a);
        eng.state.borrow_mut().log.push(json!({
            "tick": tick,
            "warning": true,
            "text": format!(
                "streaming group '{group}' unloaded, but {id} is kept: {why}; it is let go once the frame stops using it"
            ),
        }));
    }
}

/// At a tick boundary: the kept assets the frame no longer uses are let
/// go; those whose group is resident again stay as they are.
fn retry_retained(eng: &Engine) {
    let tick = tick_of(eng);
    let retained = std::mem::take(&mut eng.assets.borrow_mut().stream.retained);
    let mut keep = Vec::new();
    for r in retained {
        if eng.assets.borrow().stream.resident(&r.group) {
            continue;
        }
        let mut rs = eng.render.borrow_mut();
        if in_use(&rs, r.held, tick).is_some() {
            keep.push(r);
        } else {
            free(&mut rs, r.held);
        }
    }
    eng.assets.borrow_mut().stream.retained.extend(keep);
}

/// At a tick boundary: the environment maps, texture arrays, palettes,
/// looks and LUTs a resident group's pack holds that are not in the
/// renderer (their group unloaded them, or they were loaded before it was
/// resident) are added again under their handles.
pub(crate) fn resolve(eng: &Engine) {
    super::environment::resolve(eng);
    super::assets::resolve_texture_arrays(eng);
    super::look::resolve(eng);
    super::fx::resolve_luts(eng);
}

// ── the loads' side ─────────────────────────────────────────────────────

/// A grouped model's proxy was added: its meshes and materials are the
/// handles the full model takes over.
pub(crate) fn add_model(
    eng: &Engine,
    group: &str,
    id: &str,
    meshes: Vec<(MeshHandle, MeshData)>,
    materials: Vec<(MaterialHandle, StandardMaterial)>,
) {
    push(
        eng,
        group,
        Slot::Model {
            id: id.to_string(),
            meshes,
            materials,
            textures: Vec::new(),
        },
    );
}

pub(crate) fn add_texture(eng: &Engine, group: &str, id: &str, handle: TextureHandle) {
    push(
        eng,
        group,
        Slot::Texture {
            id: id.to_string(),
            handle,
        },
    );
}

/// A grouped sound: silence of its shape in the bank, and the stand-in the
/// tick side reads (its rate, channels and length; no samples).
pub(crate) fn add_sound(eng: &Engine, group: &str, id: &str) -> Result<(CookedSound, u64), String> {
    let shape = {
        let a = eng.assets.borrow();
        let i = a.stream.find(group);
        i.and_then(|i| {
            a.stream.groups[i]
                .desc
                .sounds
                .iter()
                .find(|s| s.id == id)
                .cloned()
        })
        .ok_or_else(|| {
            format!("{id}: its group '{group}' does not list its shape; run `og cook`")
        })?
    };
    let link = og_audio::link(eng.id).ok_or_else(|| format!("{id}: this engine has no audio"))?;
    let handle = link
        .bank()
        .insert_silent(id, shape.sample_rate, shape.channels as usize, shape.frames)
        .map_err(|e| e.reason().to_string())?;
    push(
        eng,
        group,
        Slot::Sound {
            id: id.to_string(),
            handle,
        },
    );
    let sound = CookedSound {
        id: id.to_string(),
        meta: og_cook::format::SoundMeta {
            sample_rate: shape.sample_rate,
            channels: shape.channels,
            frames: shape.frames.min(u32::MAX as u64) as u32,
            ..Default::default()
        },
        qoa: Vec::new(),
    };
    Ok((sound, handle))
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

/// The chunks the playheads want that are not in the bank (dropping the
/// others), each with how soon it is needed.
fn wanted_chunks(eng: &Engine) -> Vec<Wanted> {
    let Some(link) = og_audio::link(eng.id) else {
        return Vec::new();
    };
    if !eng.assets.borrow().sounds.iter().any(|s| s.chunked()) {
        return Vec::new();
    }
    let Ok(world) = eng.world() else {
        return Vec::new();
    };
    let bank = link.bank();
    let distances = og_audio::chunk_distances(world, bank);
    let sets = distances
        .iter()
        .map(|(h, chunks)| (*h, chunks.keys().copied().collect()))
        .collect();
    let a = eng.assets.borrow();
    let mut out = Vec::new();
    for (handle, id, n) in bank.keep_chunks(&sets) {
        let chunk = og_cook::runtime::chunk_id(&id, n);
        let Ok((bundle, dir)) = a.bundle_for(&chunk) else {
            continue;
        };
        let Ok(o) = bundle.output(&chunk) else {
            continue;
        };
        let seconds = distances
            .get(&handle)
            .and_then(|c| c.get(&n))
            .copied()
            .unwrap_or(f64::INFINITY);
        out.push(Wanted {
            path: format!("{dir}/{}", o.file),
            need: Need::Chunk {
                handle,
                id,
                n,
                seconds,
            },
        });
    }
    out
}

/// The streaming reads, in order: the sound chunks a playhead needs
/// soonest first, then the loading groups' packs by priority and name; at
/// most [`MAX_FETCHES`] in flight (a read the host answers `Pending` holds
/// its slot until it lands). A chunk no playhead wants any more is still
/// asked after until it lands (then dropped), so its slot comes back; a
/// pack whose group let go of it frees its slot at once (the host drops it
/// when it lands).
pub(crate) fn fetch(eng: &Engine) {
    let mut wants = wanted_chunks(eng);
    wants.sort_by(|a, b| match (&a.need, &b.need) {
        (
            Need::Chunk {
                id: ia,
                n: na,
                seconds: sa,
                ..
            },
            Need::Chunk {
                id: ib,
                n: nb,
                seconds: sb,
                ..
            },
        ) => sa.total_cmp(sb).then(ia.cmp(ib)).then(na.cmp(nb)),
        _ => std::cmp::Ordering::Equal,
    });
    {
        let a = eng.assets.borrow();
        let mut packs: Vec<&Group> = a
            .stream
            .groups
            .iter()
            .filter(|g| g.state == State::Loading && g.files.is_empty())
            .collect();
        packs.sort_by(|x, y| {
            y.desc
                .priority
                .cmp(&x.desc.priority)
                .then(x.desc.name.cmp(&y.desc.name))
        });
        wants.extend(packs.into_iter().map(|g| Wanted {
            path: format!("{}/{}", g.dir, g.desc.file),
            need: Need::Pack {
                group: g.desc.name.clone(),
                priority: g.desc.priority,
            },
        }));
    }
    let read = |path: &str| match hooks() {
        Some(h) => h.read_asset(path),
        None => super::assets::Fetch::Failed("no host is installed to read assets".into()),
    };
    // In flight, no longer wanted: asked after so the slot comes back.
    let stale: Vec<String> = {
        let a = eng.assets.borrow();
        a.stream
            .fetch
            .in_flight
            .keys()
            .filter(|p| !wants.iter().any(|w| &w.path == *p))
            .cloned()
            .collect()
    };
    for path in stale {
        if !matches!(read(&path), super::assets::Fetch::Pending) {
            let f = &mut eng.assets.borrow_mut().stream.fetch;
            f.in_flight.remove(&path);
            f.finished += 1;
        }
    }
    let mut queued = Vec::new();
    for w in wants {
        {
            let a = eng.assets.borrow();
            let f = &a.stream.fetch;
            if !f.in_flight.contains_key(&w.path) && f.in_flight.len() >= MAX_FETCHES {
                drop(a);
                queued.push(w);
                continue;
            }
        }
        let result = read(&w.path);
        {
            let mut a = eng.assets.borrow_mut();
            let f = &mut a.stream.fetch;
            let flying = f.in_flight.contains_key(&w.path);
            if !flying {
                f.started += 1;
            }
            if matches!(result, super::assets::Fetch::Pending) {
                f.in_flight.insert(w.path.clone(), w.need.clone());
                f.peak = f.peak.max(f.in_flight.len());
                continue;
            }
            f.in_flight.remove(&w.path);
            f.finished += 1;
        }
        let bytes = match result {
            super::assets::Fetch::Ready(b) => Ok(b),
            super::assets::Fetch::Failed(e) => Err(e),
            super::assets::Fetch::Pending => unreachable!("held in flight above"),
        };
        deliver(eng, &w, bytes);
    }
    eng.assets.borrow_mut().stream.fetch.queued = queued;
}

/// A read landed: a chunk into the bank, a pack into its loading group (a
/// failure fails the group and lets go of what the host put).
fn deliver(eng: &Engine, w: &Wanted, bytes: Result<Vec<u8>, String>) {
    let path = &w.path;
    match &w.need {
        Need::Chunk { handle, id, n, .. } => {
            let bytes = match bytes {
                Ok(b) => b,
                Err(e) => return warn_once(eng, format!("{path}: {e}")),
            };
            let chunk = og_cook::runtime::chunk_id(id, *n);
            let decoded = {
                let a = eng.assets.borrow();
                let Ok((bundle, _)) = a.bundle_for(&chunk) else {
                    return;
                };
                bundle.sound_chunk(&chunk, super::assets::aligned(&bytes).bytes())
            };
            match decoded {
                Ok((k, qoa)) if k == *n => {
                    if let Some(link) = og_audio::link(eng.id)
                        && let Err(e) = link.bank().put_chunk(*handle, *n, &qoa)
                    {
                        warn_once(eng, e.reason().to_string());
                    }
                }
                Ok((k, _)) => warn_once(eng, format!("{path} holds chunk {k}, not {n}")),
                Err(e) => warn_once(eng, e.to_string()),
            }
        }
        Need::Pack { group, .. } => {
            let tick = tick_of(eng);
            {
                let mut a = eng.assets.borrow_mut();
                let s = &mut a.stream;
                let Some(i) = s.find(group) else { return };
                // Let go of while it was fetched: the host dropped it.
                if s.groups[i].state != State::Loading {
                    return;
                }
                match bytes.and_then(|b| og_cook::runtime::unpack(&b).map_err(|e| e.to_string())) {
                    Ok(files) => {
                        s.groups[i].files = files.into_iter().collect();
                        return;
                    }
                    Err(e) => set_state(s, eng.id, i, State::Failed, tick, &format!("{path}: {e}")),
                }
            }
            // Textures the host put before the pack failed go with it.
            release(eng, group);
        }
    }
}

/// `dump_stream`'s `fetch`: the slots, what is in flight and what waits.
fn fetch_json(s: &Streams) -> Value {
    let f = &s.fetch;
    json!({
        "max": MAX_FETCHES,
        "inFlight": f.in_flight.iter().map(|(path, need)| Wanted { path: path.clone(), need: need.clone() }.json()).collect::<Vec<_>>(),
        "queued": f.queued.iter().map(Wanted::json).collect::<Vec<_>>(),
        "started": f.started as f64,
        "finished": f.finished as f64,
        "peak": f.peak,
    })
}

fn warn_once(eng: &Engine, text: String) {
    let mut st = eng.state.borrow_mut();
    if st.ui_warned.insert(text.clone()) {
        st.log
            .push(json!({ "tick": 0, "warning": true, "text": text }));
    }
}
