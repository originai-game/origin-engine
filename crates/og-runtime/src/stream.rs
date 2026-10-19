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
//! on the GPU and in the audio bank. Every change of a group's state sends
//! og.StreamEvent at the start of the next tick; the probe's `dump_stream`
//! lists the budget, what is used and the peak, and every group.

use super::engine::{Engine, engine, hooks};
use super::{ok, ok_void, refuse};
use og_cook::runtime::{CookedSound, ManifestStream, StreamTrigger};
use og_core::{Reflect, Str64};
use og_ecs::{SystemDesc, World};
use og_render::{
    Material, MaterialHandle, MeshData, MeshHandle, TextureData, TextureHandle, TextureSource,
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
        /// Per material, the proxy's handle and material.
        materials: Vec<(MaterialHandle, Material)>,
        /// The full model's textures, made the first time it loads.
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

impl Slot {
    fn id(&self) -> &str {
        match self {
            Slot::Model { id, .. } | Slot::Texture { id, .. } | Slot::Sound { id, .. } => id,
        }
    }
}

/// An engine's streaming groups (in its assets).
#[derive(Default)]
pub(crate) struct Streams {
    opened: bool,
    budget: Option<u64>,
    groups: Vec<Group>,
    /// Per group name, its loaded assets and whether each is upgraded.
    slots: BTreeMap<String, Vec<(Slot, bool)>>,
    peak: u64,
    log: Vec<Value>,
}

thread_local! {
    static EVENTS: RefCell<BTreeMap<u32, Vec<StreamEvent>>> = const { RefCell::new(BTreeMap::new()) };
}

impl Streams {
    fn budget(&self) -> u64 {
        self.budget.unwrap_or(DEFAULT_BUDGET)
    }

    /// The memory the admitted groups take (loading ones count already).
    fn used(&self) -> u64 {
        self.groups
            .iter()
            .filter(|g| matches!(g.state, State::Loading | State::Resident))
            .map(|g| g.desc.memory)
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
    // Loading groups read their packs; one group is put in place per update.
    let loading: Vec<(usize, String)> = {
        let a = eng.assets.borrow();
        a.stream
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.state == State::Loading && g.files.is_empty())
            .map(|(i, g)| (i, format!("{}/{}", g.dir, g.desc.file)))
            .collect()
    };
    for (i, path) in loading {
        let read = match hooks() {
            Some(h) => h.read_asset(&path),
            None => super::assets::Fetch::Failed("no host is installed to read assets".into()),
        };
        let mut a = eng.assets.borrow_mut();
        let s = &mut a.stream;
        match read {
            super::assets::Fetch::Pending => {}
            super::assets::Fetch::Failed(e) => {
                set_state(s, eng.id, i, State::Failed, tick, &format!("{path}: {e}"))
            }
            super::assets::Fetch::Ready(bytes) => match og_cook::runtime::unpack(&bytes) {
                Ok(files) => s.groups[i].files = files.into_iter().collect(),
                Err(e) => set_state(s, eng.id, i, State::Failed, tick, &format!("{path}: {e}")),
            },
        }
    }
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
    Sound(Vec<u8>),
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
                    Full::Sound(sound.qoa)
                }
            };
            out.push((k, full));
        }
        out
    };
    let link = og_audio::link(eng.id);
    let mut a = eng.assets.borrow_mut();
    let mut rs = eng.render.borrow_mut();
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
                    rs.scene.set_material(*h, model.material(i, textures));
                }
            }
            (Slot::Texture { handle, .. }, Full::Texture(d, pixel)) => {
                replace_texture(&mut rs.scene, *handle, d);
                if pixel {
                    rs.scene
                        .set_texture_filter(*handle, og_render::TextureFilter::Nearest);
                }
            }
            (Slot::Sound { id, handle }, Full::Sound(qoa)) => {
                if let Some(l) = &link {
                    l.bank()
                        .replace_qoa(*handle, id, &qoa)
                        .map_err(|e| e.reason().to_string())?;
                }
            }
            _ => {}
        }
        *up = true;
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

/// Puts a group's stand-ins back and drops its pack.
fn downgrade(eng: &Engine, group: &str) {
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
                    rs.scene.set_material(*h, m.clone());
                }
                for h in textures.iter() {
                    rs.scene.set_texture(*h, placeholder());
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

// ── the loads' side ─────────────────────────────────────────────────────

/// A grouped model's proxy was added: its meshes and materials are the
/// handles the full model takes over.
pub(crate) fn add_model(
    eng: &Engine,
    group: &str,
    id: &str,
    meshes: Vec<(MeshHandle, MeshData)>,
    materials: Vec<(MaterialHandle, Material)>,
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

fn budget_json(s: &Streams) -> Value {
    json!({
        "budget": s.budget() as f64,
        "used": s.used() as f64,
        "peak": s.peak as f64,
        "groups": s.groups.iter().map(status_json).collect::<Vec<_>>(),
    })
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
                let mut j = status_json(g);
                j["trigger"] = trigger_json(&g.desc.trigger);
                j["file"] = json!(g.desc.file);
                j["slots"] = json!(s.slots.get(&g.desc.name).map_or(0, Vec::len));
                j
            })
            .collect::<Vec<_>>()
    );
    v["events"] = json!(s.log);
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
    match with_group(world, &group, |_, s, i| status_json(&s.groups[i])) {
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
