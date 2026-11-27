//! A game's flow (`og api show flow`): a Yarn dialogue of
//! segments. `og.flow.load` prepares a loaded dialogue: each node with an
//! `og-flow` header names its segment, a timeline of footage (its own video
//! track, which plays on a deck), a live scene (a `scene` track), or both.
//! `og.flow.start` spawns the flow (og.Flow) and its decks (og.FlowDeck, each
//! with og.Timeline, og.VideoPlayer and og.AudioSource) and starts its
//! conversation holding at each flow node (og.story's StartDesc.hold).
//!
//! The og.flow system runs before og.timeline (which runs before og.video and
//! og.audio). Each tick it reads the current deck's og.Timeline time against
//! the segment's markers: the window opens at `flow.open` and locks at
//! `flow.lock`; optional `flow.loopStart`..`flow.loopEnd` loops an idle choice
//! beat until the player chooses; at the cut (`flow.cut`, else the end) — or
//! immediately for `#immediate` choices or held segments — the pending choice
//! (else `#default`) is taken, the conversation runs to its next hold, and
//! that node's segment starts on a deck.
//!
//! All simulation state lives in ECS components (`og.Flow`, `og.FlowDeck`,
//! decks' `og.Timeline`/`og.VideoPlayer`/`og.AudioSource`), `og.story`'s state
//! block, and `og.flow`'s own `FlowBlock` state block (ops, gestures,
//! inspected clues, transcript backlog, cross-segment BGM state, and the
//! project-declared flow handle), so snapshots, restore, and replay are exact.

mod chart;
mod state;

use crate::bind::story::{
    og_story_advance, og_story_choose, og_story_start, og_story_stop, og_story_view,
};
use crate::engine::{Engine, engine};
use crate::host::codec::Layout;
use crate::host::wire::{
    ChoiceView, FlowChart, FlowCheckpoint, FlowGestureDesc, FlowHotspot, FlowInfo, FlowJumpDesc,
    FlowMusicDesc, FlowStartDesc, FlowTranscriptLine, FlowView, QteView, WindowView, record,
};
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use og_timeline::segment::{MusicHeader, QTE_CHOICES, Qte, QteKind};
use og_timeline::{HotspotKey, HotspotKind};
use serde_json::{Value, json};
use state::{FlowBlock, Op, Saved};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const FLOW: &str = "og.Flow";
pub(crate) const DECK: &str = "og.FlowDeck";
const SEGMENT: &str = "og.FlowSegment";
const CHOICE: &str = "og.FlowChoice";
const CHAPTER: &str = "og.FlowChapter";
const ENDED: &str = "og.FlowEnded";
const INSPECTED: &str = "og.FlowInspected";
const QTE_RESULT: &str = "og.FlowQteResult";
/// og.flow's records, registered together.
pub(crate) const RECORDS: [&str; 8] = [
    FLOW, DECK, SEGMENT, CHOICE, CHAPTER, ENDED, INSPECTED, QTE_RESULT,
];
pub(crate) const SYSTEM: &str = "og.flow";
const TIMELINE: &str = "og.Timeline";
const PLAYER: &str = crate::bind::video::PLAYER;
const AUDIO: &str = "og.AudioSource";
/// The node header that names a segment (and holds the conversation).
const HEADER: &str = og_story::graph::FLOW_HEADER;
const MAX_DECKS: u32 = 3;

// QTE state bitmask fields in `Flow.qteState[slot]`.
const QTE_STARTED: u32 = 1 << 0;
const QTE_RESOLVED: u32 = 1 << 1;
const QTE_GRADE_SHIFT: u32 = 2;
const QTE_GRADE_MASK: u32 = 3 << QTE_GRADE_SHIFT;
const GRADE_MISS: u32 = 1;
const GRADE_GOOD: u32 = 2;
const GRADE_PERFECT: u32 = 3;

fn grade_name(state: u32) -> &'static str {
    match (state & QTE_GRADE_MASK) >> QTE_GRADE_SHIFT {
        GRADE_MISS => "miss",
        GRADE_GOOD => "good",
        GRADE_PERFECT => "perfect",
        _ => "",
    }
}

fn set_grade(state: &mut u32, grade: u32) {
    *state = (*state & !QTE_GRADE_MASK) | ((grade & 3) << QTE_GRADE_SHIFT);
}

/// Prepared per-choice metadata, indexed stably by `ChoiceView.id` so
/// multi-group Yarn option blocks never misindex.
#[derive(Clone, Debug)]
pub(super) struct ChoiceMeta {
    pub index: u32,
    pub key: String,
    pub slot: Option<usize>,
    pub qte: Option<Qte>,
    pub outcome: Option<String>,
    pub immediate: bool,
    pub conditional: bool,
    pub to: Vec<u32>,
}

/// A flow node of a prepared dialogue.
#[derive(Clone, Debug)]
pub(super) struct Node {
    pub title: String,
    pub display_title: String,
    pub segment: String,
    pub timeline: u64,
    pub first: Option<(u64, f32)>,
    pub poster: u64,
    pub video_width: u32,
    pub video_height: u32,
    pub open: f32,
    pub lock: f32,
    pub cut: f32,
    pub chapter: String,
    pub ending: String,
    /// Seconds its segment dissolves in over the one before (0: a cut).
    pub dissolve: f32,
    /// Optional `og-flow-music` header directive.
    pub music: Option<MusicHeader>,
    /// Optional idle choice loop `[loopStart, loopEnd)`.
    pub loop_region: Option<(f32, f32)>,
    /// Its segment's hotspot keys.
    pub hotspots: Vec<HotspotKey>,
    /// `flow.hold`: where its segment stops for og.flow.advance.
    pub hold: Option<f32>,
    /// Its footage plays under the scene (`layer under`): og.timeline puts
    /// it on the camera, and the view's picture is 0.
    pub under: bool,
    /// Whether this segment has a live `scene` track.
    pub live: bool,
    /// Its `#outcome:<name>` choices: (ChoiceView.index, name).
    pub outcomes: Vec<(u32, String)>,
    /// Choice metadata in declaration order.
    pub choices_list: Vec<ChoiceMeta>,
    /// Choice metadata index by `ChoiceView.id`.
    pub choice_by_id: BTreeMap<String, usize>,
    /// The flow nodes its own steps may lead to (a node without choices).
    pub next: Vec<u32>,
}

impl Node {
    fn choice_meta(&self, c: &ChoiceView) -> Option<&ChoiceMeta> {
        self.choice_by_id
            .get(&c.id)
            .and_then(|&i| self.choices_list.get(i))
            .or_else(|| self.choices_list.iter().find(|m| m.index == c.index))
    }

    fn choice_by_index(&self, index: u32) -> Option<&ChoiceMeta> {
        self.choices_list.iter().find(|m| m.index == index)
    }
}

/// A dialogue og.flow.load prepared: its flow nodes by id.
#[derive(Debug, Default)]
pub(super) struct Prepared {
    pub nodes: BTreeMap<u32, Node>,
    /// Its flow nodes in file order.
    pub order: Vec<u32>,
    /// The flow nodes the dialogue's start leads to first.
    pub roots: Vec<u32>,
}

/// What a flow showed at the end of the last tick (og.flow.view).
#[derive(Clone, Debug, Default)]
struct Shown {
    conversation: u32,
    deck: u64,
    previous_deck: u64,
    has_picture: bool,
    prev_has_picture: bool,
    live_handoff: bool,
    node: String,
    segment: String,
    texture: u64,
    time: f32,
    open: f32,
    lock: f32,
    cut: f32,
    speed: f32,
    video_width: u32,
    video_height: u32,
    dt: f32,
    window: u32,
    pending: i32,
    paused: bool,
    looping: bool,
    chapter: String,
    ending: String,
    ended: bool,
    held: bool,
    /// The node's `#outcome` names (og.flow.advance checks them).
    outcomes: Vec<String>,
    hotspots: Vec<FlowHotspot>,
    qtes: Vec<QteView>,
    previous: u64,
    mix: f32,
    /// The first frames of the segments the node in place may cut to,
    /// for the streaming queue ([`prefetch`]).
    ahead: Vec<Ahead>,
}

/// A segment's first frame a cut may need: its video (handle), the time
/// it starts at and the seconds until the cut.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Ahead {
    pub video: u64,
    pub from: f32,
    pub seconds: f64,
}

/// og.flow on one engine.
#[derive(Default)]
pub(crate) struct Rt {
    /// Prepared dialogues by name (asset-level, kept across reloads).
    flows: BTreeMap<String, Arc<Prepared>>,
    /// Deterministic world state saved and restored by `FlowBlock`.
    saved: Saved,
    /// Cached per-flow views rebuilt every tick and on restore.
    views: BTreeMap<u64, Shown>,
    warned: BTreeSet<String>,
}

pub(crate) type ModuleState = Arc<Mutex<Rt>>;

fn lock(m: &Mutex<Rt>) -> MutexGuard<'_, Rt> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn state(engine_id: u32) -> Result<ModuleState, String> {
    Ok(engine(engine_id)?.modules.flow.clone())
}

pub(super) fn bad(m: impl Into<String>) -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, m.into())
}

/// A flow node's id: FNV-1a of its title (never 0, which is none).
fn node_id(title: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in title.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    h.max(1)
}

fn warn(eng: &Engine, rt: &ModuleState, text: String) {
    if !lock(rt).warned.insert(text.clone()) {
        return;
    }
    if let Ok(mut st) = eng.state.try_borrow_mut() {
        st.log
            .push(json!({ "tick": og_ecs::last_tick(), "warning": true, "text": text }));
    }
}

/// Attaches the `og.flow` deterministic state block to `world` once `og.Flow`
/// is registered (`registry::ATTACH`).
pub(crate) fn attach(mount: crate::engine::Mount<'_>, world: &mut World) -> Result<(), String> {
    let block = FlowBlock(mount.modules.flow.clone());
    match world.add_state_block(SYSTEM, Box::new(block)) {
        Err(e) if e.reason().contains("already added") => Ok(()),
        r => r.map_err(|e| e.reason().to_string()),
    }
}

// ── og.flow.load ─────────────────────────────────────────────────────────────

/// Dialogue `name` as og.flow plays it: every flow node's segment loaded
/// and checked, and where each choice may lead.
fn prepare(
    eng: &Engine,
    mut world: Option<&mut World>,
    name: &str,
) -> Result<(Prepared, FlowInfo), String> {
    let graph = crate::bind::story::graph_of(eng.id, name)
        .ok_or_else(|| format!("no dialogue '{name}' is loaded: og.story.load it first"))?;
    // The flow nodes reachable from `to` without passing another flow node.
    let reach = |to: &[String]| -> Vec<u32> {
        og_story::graph::flow_reach(&graph, to)
            .into_iter()
            .map(node_id)
            .collect()
    };
    let mut p = Prepared::default();
    let mut info = FlowInfo {
        dialogue: name.to_string(),
        ..FlowInfo::default()
    };
    for n in &graph {
        let header = |k: &str| n.headers.get(k).cloned().unwrap_or_default();
        if let Some(c) = n.headers.get("og-flow-chapter") {
            info.chapters.push(c.clone());
        }
        if let Some(e) = n.headers.get("og-flow-ending") {
            info.endings.push(e.clone());
        }
        let Some(segment) = n.headers.get(HEADER) else {
            continue;
        };
        let at = |e: String| format!("{name}: node '{}': {e}", n.title);
        let id = node_id(&n.title);
        if let Some(existing) = p.nodes.get(&id) {
            return Err(at(format!(
                "its 32-bit node id collides with node '{}': rename one of them",
                existing.title
            )));
        }
        let timeline = crate::bind::timeline::load_granted(eng, segment, world.as_deref_mut())
            .map_err(|e| at(format!("segment {segment}: {e}")))?;
        let (s, first) = crate::bind::timeline::segment(eng.id, timeline)
            .map_err(|e| at(format!("segment {segment}: {e}")))?;
        // Group-aware `#default` check: at most one `#default` per Yarn option group.
        let mut defaults_by_group: BTreeMap<u32, usize> = BTreeMap::new();
        for c in &n.choices {
            if c.tags.contains_key("default") {
                *defaults_by_group.entry(c.group).or_default() += 1;
            }
        }
        if let Some((_, count)) = defaults_by_group.into_iter().find(|&(_, n)| n > 1) {
            return Err(at(format!(
                "{count} choices in the same option group are #default; one at most"
            )));
        }
        let dissolve = match n.headers.get("og-flow-dissolve") {
            Some(h) => og_timeline::segment::dissolve(h).map_err(at)?,
            None => 0.0,
        };
        let music = match n.headers.get("og-flow-music") {
            Some(h) => {
                let m = og_timeline::segment::music_header(h).map_err(at)?;
                if !m.sound.is_empty() {
                    let _ = crate::world::assets::load_sound(eng, &m.sound);
                }
                Some(m)
            }
            None => None,
        };
        let parsed_outcomes = og_timeline::segment::outcomes(
            s.hold.is_some(),
            n.choices.iter().map(|c| (c.index, &c.tags)),
        )
        .map_err(at)?;
        let mut outcomes = Vec::new();
        let mut choices_list = Vec::with_capacity(n.choices.len());
        let mut choice_by_id = BTreeMap::new();
        let mut qte_slots = 0usize;
        for (c, out) in n.choices.iter().zip(parsed_outcomes) {
            let (slot, qte) = match c.tags.get("qte") {
                Some(t) => {
                    let q = Qte::parse(t).map_err(|e| at(format!("choice {}: {e}", c.index)))?;
                    if qte_slots >= QTE_CHOICES {
                        return Err(at(format!(
                            "choice {} is a #qte; a node may have at most {QTE_CHOICES} #qte choices",
                            c.index
                        )));
                    }
                    let sl = qte_slots;
                    qte_slots += 1;
                    (Some(sl), Some(q))
                }
                None => (None, None),
            };
            if let Some(ref o) = out {
                outcomes.push((c.index, o.clone()));
            }
            let idx = choices_list.len();
            choice_by_id.insert(c.id.clone(), idx);
            let to = reach(&c.to);
            choices_list.push(ChoiceMeta {
                index: c.index,
                key: c.key.clone(),
                slot,
                qte,
                outcome: out,
                immediate: c.tags.contains_key("immediate"),
                conditional: c.tags.contains_key("if") || to.len() > 1,
                to,
            });
        }
        let vinfo = first.and_then(|(vid, _)| crate::bind::video::info_of(eng, vid));
        let poster = vinfo.as_ref().map_or(0, |v| v.poster);
        let video_width = vinfo.as_ref().map_or(0, |v| v.width);
        let video_height = vinfo.as_ref().map_or(0, |v| v.height);
        p.nodes.insert(
            id,
            Node {
                title: n.title.clone(),
                display_title: header("og-flow-title"),
                segment: segment.clone(),
                timeline,
                first,
                poster,
                video_width,
                video_height,
                open: s.open,
                lock: s.lock,
                cut: s.cut,
                chapter: header("og-flow-chapter"),
                ending: header("og-flow-ending"),
                dissolve,
                music,
                loop_region: s.loop_region,
                hotspots: s.hotspots,
                hold: s.hold,
                under: s.under,
                live: s.live,
                outcomes,
                choices_list,
                choice_by_id,
                next: reach(&n.to),
            },
        );
        p.order.push(id);
        info.segments.push(n.title.clone());
    }
    let start = crate::bind::story::start_of(eng.id, name)
        .filter(|s| !s.is_empty())
        .or_else(|| graph.first().map(|n| n.title.clone()))
        .unwrap_or_default();
    p.roots = reach(&[start]);
    if p.nodes.is_empty() {
        return Err(format!(
            "{name} has no flow node: give a node an `og-flow: base/segments/<segment>` header"
        ));
    }
    Ok((p, info))
}

/// After a hot reload: re-prepares every loaded flow dialogue whose story or
/// segment timelines changed so edits take effect immediately.
pub(crate) fn refresh(eng: &Engine) -> Vec<String> {
    let names: Vec<String> = lock(&eng.modules.flow).flows.keys().cloned().collect();
    let mut failed = Vec::new();
    for name in names {
        match prepare(eng, eng.world().ok(), &name) {
            Ok((p, _)) => {
                lock(&eng.modules.flow).flows.insert(name, Arc::new(p));
            }
            Err(e) => failed.push(format!("flow {name}: {e}")),
        }
    }
    failed
}

pub fn og_flow_load(world: u32, dialogue: String) -> Result<FlowInfo, String> {
    let eng = engine(world)?;
    let w = eng.world().map_err(|e| {
        format!(
            "og.flow.load registers og.Flow, which a system cannot do: load flows in setup ({e})"
        )
    })?;
    crate::engine::lazy::ensure(&eng, w, FLOW)?;
    let (p, info) = prepare(&eng, Some(w), &dialogue)?;
    lock(&eng.modules.flow).flows.insert(dialogue, Arc::new(p));
    Ok(info)
}

// ── og.flow.start ────────────────────────────────────────────────────────────

fn insert(eng: &Engine, world: &mut World, e: Entity, name: &str, v: Value) -> Result<(), String> {
    let layout = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .ok_or_else(|| format!("{name} has no layout"))?;
    let id = world.id_by_name(name).map_err(|e| e.reason().to_string())?;
    let base = world.get_bytes(e, id).ok();
    let bytes = layout.encode(base.as_deref(), Some(&v))?;
    world
        .insert_bytes(e, id, &bytes)
        .map_err(|e| e.reason().to_string())
}

/// project.ogt's `flow` (its dialogue's asset id), before GameDesc.start:
/// loaded, prepared and started as a game's og.story.load, og.flow.load
/// and og.flow.start would.
pub(crate) fn start_project_flow(
    eng: &crate::engine::Engine,
    dialogue: String,
) -> Result<(), String> {
    let start = || {
        let story = crate::bind::story::og_story_load(eng.id, dialogue)?;
        og_flow_load(eng.id, story.name.clone())?;
        start_declared(eng.id, &story.name)
    };
    start().map_err(|e| format!("project.ogt's `flow`: {e}"))
}

/// Starts the flow project.ogt's `flow` declares (dialogue `name`, loaded
/// and prepared): og.flow.current's from now on.
fn start_declared(world: u32, name: &str) -> Result<(), String> {
    let flow = og_flow_start(world, Some(json!({ "dialogue": name }).to_string()))?;
    let st = state(world)?;
    lock(&st).saved.declared = flow.as_f64().map(|f| f as u64);
    Ok(())
}

pub fn og_flow_current(world: u32) -> Option<u64> {
    let st = state(world).ok()?;
    let rt = lock(&st);
    rt.saved.declared.filter(|f| rt.views.contains_key(f))
}

pub fn og_flow_start(world: u32, desc: Option<String>) -> Result<Value, String> {
    let d: FlowStartDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let eng = engine(world)?;
    let w = eng.world().map_err(|e| {
        format!("og.flow.start spawns a flow's decks: call it outside a system ({e})")
    })?;
    if !lock(&eng.modules.flow).flows.contains_key(&d.dialogue) {
        return Err(format!(
            "the dialogue '{}' is not prepared: og.flow.load it in setup",
            d.dialogue
        ));
    }
    let decks = if d.decks == 0 {
        MAX_DECKS
    } else {
        d.decks.min(MAX_DECKS)
    };
    let started = og_story_start(
        world,
        Some(json!({ "dialogue": d.dialogue, "node": d.node, "hold": HEADER }).to_string()),
    )?;
    let conversation = started
        .as_u64()
        .ok_or("og.story.start answered no conversation")? as u32;
    let spawn = |w: &mut World| w.spawn(()).map_err(|e| e.reason().to_string());
    let flow = spawn(w)?;
    let mut first = 0u64;
    for _ in 0..decks {
        let e = spawn(w)?;
        if first == 0 {
            first = e.to_bits();
        }
        insert(&eng, w, e, TIMELINE, json!({ "playing": false }))?;
        insert(
            &eng,
            w,
            e,
            PLAYER,
            json!({ "playing": false, "wait": true }),
        )?;
        insert(&eng, w, e, AUDIO, json!({}))?;
        insert(&eng, w, e, DECK, json!({ "flow": flow.to_bits() as f64 }))?;
    }
    insert(
        &eng,
        w,
        flow,
        FLOW,
        json!({
            "conversation": conversation,
            "decks": decks,
            "current": first as f64,
            "paused": d.paused,
            "speed": 1.0,
            "looping": false,
        }),
    )?;
    lock(&eng.modules.flow).views.insert(
        flow.to_bits(),
        Shown {
            conversation,
            deck: first,
            pending: -1,
            paused: d.paused,
            speed: 1.0,
            mix: 1.0,
            ..Shown::default()
        },
    );
    Ok(json!(flow.to_bits() as f64))
}

/// Rebuilds `Rt::views` from live ECS entities immediately after a world
/// restore or rollback rewind so `og.flow.view` and `og.flow.current` are
/// immediately up to date before the next tick runs.
// A guarded entry into the engine's `story` module (wasm-modules.json).
#[inline(never)]
pub(crate) fn after_restore(eng: &Engine, world: &World) {
    let layout = |name: &str| -> Option<(SchemaId, Layout)> {
        let id = world.id_by_name(name).ok()?;
        let l = eng.state.borrow().layouts.get(name).cloned()?;
        Some((id, l))
    };
    let (Some(fr), Some(dr), Some(tr), Some(pr)) =
        (layout(FLOW), layout(DECK), layout(TIMELINE), layout(PLAYER))
    else {
        lock(&eng.modules.flow).views.clear();
        return;
    };
    let dt = world.time().dt();
    let mut flows = Vec::new();
    for e in world.entities() {
        if let Ok(b) = world.get_bytes(e, fr.0)
            && let Ok(f) = fr.1.read::<record::Flow>(&b)
        {
            flows.push((e, f));
        }
    }
    let st = eng.modules.flow.clone();
    let mut views = BTreeMap::new();
    for (f, flow) in flows {
        let key = f.to_bits();
        let dialogue = crate::bind::story::dialogue_of(eng.id, flow.conversation);
        let prep = dialogue
            .as_ref()
            .and_then(|d| lock(&st).flows.get(d).cloned());
        let node = prep.as_ref().and_then(|p| p.nodes.get(&flow.node));
        let cur_deck = Entity::from_bits(flow.current);
        let time = world
            .get_bytes(cur_deck, tr.0)
            .ok()
            .and_then(|b| tr.1.read::<record::Timeline>(&b).ok())
            .map_or(0.0, |t| t.time);
        let deck_pic = |deck: Entity| -> (u64, bool) {
            let held = world
                .get_bytes(deck, dr.0)
                .ok()
                .and_then(|b| dr.1.read::<record::FlowDeck>(&b).ok());
            let has = held
                .and_then(|d| prep.as_ref()?.nodes.get(&d.node))
                .is_some_and(|n| !n.under && n.first.is_some());
            if !has {
                return (0, false);
            }
            let tex = world
                .get_bytes(deck, pr.0)
                .ok()
                .and_then(|b| pr.1.read::<record::VideoPlayer>(&b).ok())
                .map_or(0, |p| p.texture);
            (tex, true)
        };
        let (texture, has_picture) = deck_pic(cur_deck);
        let (previous_texture, prev_has_picture) = match previous(&flow) {
            Some(p) => deck_pic(p),
            None => (0, false),
        };
        let mix = if flow.dissolve_left > 0 && flow.dissolve > 0 {
            1.0 - flow.dissolve_left as f32 / flow.dissolve as f32
        } else {
            1.0
        };
        let inspected_set = lock(&st).saved.inspected.get(&key).cloned();
        let (hotspots, qtes) = match node {
            Some(n) if !flow.ended => {
                let choices = og_story_view(eng.id, flow.conversation)
                    .map(|v| seen(v.choices))
                    .unwrap_or_default();
                offers(n, &flow, &choices, inspected_set.as_ref(), time, dt)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let speed = if flow.speed > 0.0 { flow.speed } else { 1.0 };
        views.insert(
            key,
            Shown {
                conversation: flow.conversation,
                deck: flow.current,
                previous_deck: flow.previous,
                has_picture,
                prev_has_picture,
                live_handoff: false,
                node: node.map(|n| n.title.clone()).unwrap_or_default(),
                segment: node.map(|n| n.segment.clone()).unwrap_or_default(),
                texture,
                time,
                open: node.map_or(0.0, |n| n.open),
                lock: node.map_or(0.0, |n| n.lock),
                cut: node.map_or(0.0, |n| n.cut),
                speed,
                video_width: node.map_or(0, |n| n.video_width),
                video_height: node.map_or(0, |n| n.video_height),
                dt,
                window: flow.window,
                pending: flow.pending,
                paused: flow.paused,
                looping: flow.looping,
                chapter: prep
                    .as_ref()
                    .and_then(|p| p.nodes.get(&flow.chapter))
                    .map(|n| n.chapter.clone())
                    .unwrap_or_default(),
                ending: if flow.ended {
                    node.map(|n| n.ending.clone()).unwrap_or_default()
                } else {
                    String::new()
                },
                ended: flow.ended,
                held: flow.held,
                outcomes: node
                    .map(|n| n.outcomes.iter().map(|(_, o)| o.clone()).collect())
                    .unwrap_or_default(),
                hotspots,
                qtes,
                previous: previous_texture,
                mix,
                ahead: match (prep.as_ref(), node) {
                    (Some(p), Some(n)) if !flow.ended => ahead(p, n, time),
                    _ => Vec::new(),
                },
            },
        );
    }
    lock(&st).views = views;
}

// ── og.flow.view and the ops ─────────────────────────────────────────────────

/// Whether a choice is an `#outcome`, which the game takes, never the player.
fn outcome(c: &ChoiceView) -> bool {
    c.tags.get("outcome").is_some()
}

/// The choices the player sees: a node's without its `#outcome`s.
fn seen(choices: Vec<ChoiceView>) -> Vec<ChoiceView> {
    choices.into_iter().filter(|c| !outcome(c)).collect()
}

/// The choice taken when nothing was: `#default`, else the first enabled
/// one the player sees.
fn default_choice(choices: &[ChoiceView]) -> Option<u32> {
    let shown = || choices.iter().filter(|c| c.enabled && !outcome(c));
    shown()
        .find(|c| c.tags.get("default").is_some())
        .or_else(|| shown().next())
        .map(|c| c.index)
}

/// Reads `VideoPlayer.texture` live from the world when `og.flow` ran before
/// `og.video` assigned it on Tick 1.
fn live_deck_texture(eng: &Engine, deck_bits: u64) -> u64 {
    if deck_bits == 0 {
        return 0;
    }
    let Ok(w) = eng.world() else {
        return 0;
    };
    let Ok(id) = w.id_by_name(PLAYER) else {
        return 0;
    };
    let Some(l) = eng.state.borrow().layouts.get(PLAYER).cloned() else {
        return 0;
    };
    w.get_bytes(Entity::from_bits(deck_bits), id)
        .ok()
        .and_then(|b| l.read::<record::VideoPlayer>(&b).ok())
        .map_or(0, |p| p.texture)
}

pub fn og_flow_view(world: u32, flow: f64) -> Option<FlowView> {
    let eng = engine(world).ok()?;
    let st = eng.modules.flow.clone();
    let s = lock(&st).views.get(&(flow as u64)).cloned()?;
    let mut texture = s.texture;
    if texture == 0 && s.has_picture {
        texture = live_deck_texture(&eng, s.deck);
    }
    let mut previous = s.previous;
    if previous == 0 && s.previous_deck != 0 && s.prev_has_picture {
        previous = live_deck_texture(&eng, s.previous_deck);
    }
    let mut mix = s.mix;
    // Live-scene 1-frame handoff: keep previous footage visible while the new
    // live scene is queued for placement on the next `before_tick`.
    if s.live_handoff
        && texture == 0
        && previous != 0
        && crate::bind::timeline::scene_pending(world, s.deck)
    {
        texture = previous;
        previous = 0;
        mix = 1.0;
    }
    let (buffering, buffer_progress) = crate::bind::video::buffering_state(&eng);
    let choices = if s.window > 0 {
        og_story_view(world, s.conversation)
            .map(|v| seen(v.choices))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let ticks = |secs: f32| {
        if s.dt > 0.0 {
            (secs.max(0.0) / s.dt).ceil() as u32
        } else {
            0
        }
    };
    let window = (s.window > 0).then(|| WindowView {
        ticks_left: ticks(s.lock - s.time),
        ticks_total: ticks(s.lock - s.open),
        default_choice: default_choice(&choices).unwrap_or(0),
        pending: u32::try_from(s.pending).ok(),
        locked: s.window == 2,
        held: s.held || s.looping,
    });
    Some(FlowView {
        flow: flow as u64,
        node: s.node,
        segment: s.segment,
        texture,
        time: s.time,
        cut: s.cut,
        speed: s.speed,
        video_width: s.video_width,
        video_height: s.video_height,
        choices,
        window,
        hotspots: s.hotspots,
        qtes: s.qtes,
        previous,
        mix,
        paused: s.paused,
        held: s.held,
        looping: s.looping,
        buffering,
        buffer_progress,
        chapter: s.chapter,
        ending: s.ending,
        ended: s.ended,
    })
}

fn queue(world: u32, flow: f64, op: Op) -> Result<(), String> {
    let st = state(world)?;
    let key = flow as u64;
    let s = lock(&st)
        .views
        .get(&key)
        .cloned()
        .ok_or_else(|| format!("entity {key} is no flow (og.flow.start)"))?;
    match &op {
        Op::Choose(i) => {
            if s.window != 1 {
                return Err(if s.window == 2 {
                    "the window has locked: the cut takes what was chosen".into()
                } else {
                    "no choice window is open (flow.open to flow.lock)".into()
                });
            }
            let v =
                og_story_view(world, s.conversation).ok_or("the flow's conversation is gone")?;
            let c =
                v.choices.iter().find(|c| c.index == *i).ok_or_else(|| {
                    format!("choice {i} is not offered (hidden, or no such choice)")
                })?;
            if !c.enabled {
                return Err(format!("choice {i} is disabled: {}", c.reason));
            }
            if outcome(c) {
                return Err(format!(
                    "choice {i} is an #outcome: the game takes it with og.flow.advance(world, flow, \"{}\")",
                    c.tags
                        .get("outcome")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                ));
            }
        }
        Op::Advance(o) => {
            if s.ended {
                return Err("the flow has ended: nothing is left to advance".into());
            }
            match o {
                Some(o) if !s.outcomes.contains(o) => {
                    return Err(if s.outcomes.is_empty() {
                        format!(
                            "node '{}' has no #outcome choices: call og.flow.advance(world, flow) without one",
                            s.node
                        )
                    } else {
                        format!(
                            "node '{}' has no #outcome:{o}; its outcomes are {}",
                            s.node,
                            s.outcomes.join(", ")
                        )
                    });
                }
                None if !s.outcomes.is_empty()
                    && og_story_view(world, s.conversation)
                        .is_some_and(|v| seen(v.choices).is_empty()) =>
                {
                    return Err(format!(
                        "node '{}' ends only by its outcomes: name one ({})",
                        s.node,
                        s.outcomes.join(", ")
                    ));
                }
                _ => {}
            }
        }
        Op::Skip => {
            if s.ended {
                return Err("the flow has ended: nothing is left to skip".into());
            }
        }
        Op::SetSpeed(sp) => {
            if !(sp.is_finite() && (0.25..=4.0).contains(sp)) {
                return Err(format!("flow speed {sp} is outside 0.25 to 4.0"));
            }
        }
        Op::Seek(t) => {
            if s.ended {
                return Err("the flow has ended: nothing is left to seek".into());
            }
            if !(t.is_finite() && *t >= 0.0) {
                return Err(format!("seek time {t} must be finite and >= 0"));
            }
        }
        Op::Inspect(id) => {
            if !s
                .hotspots
                .iter()
                .any(|h| h.kind == "inspect" && h.id == *id)
            {
                return Err(format!(
                    "inspect hotspot '{id}' is not active on node '{}'",
                    s.node
                ));
            }
        }
        Op::Jump(j) => {
            if j.node.is_empty() {
                return Err("FlowJumpDesc.node names the flow node to jump to".into());
            }
            let dialogue = crate::bind::story::dialogue_of(world, s.conversation)
                .ok_or("the flow's conversation is gone")?;
            let exists = lock(&st)
                .flows
                .get(&dialogue)
                .is_some_and(|p| p.nodes.contains_key(&node_id(&j.node)));
            if !exists {
                return Err(format!(
                    "dialogue '{dialogue}' has no flow node '{}'",
                    j.node
                ));
            }
        }
        Op::Play | Op::Pause | Op::Stop | Op::SetMusic(_) => {}
    }
    lock(&st).saved.ops.push((key, op));
    Ok(())
}

pub fn og_flow_choose(world: u32, flow: f64, index: u32) -> Result<(), String> {
    queue(world, flow, Op::Choose(index))
}

pub fn og_flow_play(world: u32, flow: f64) -> Result<(), String> {
    queue(world, flow, Op::Play)
}

pub fn og_flow_pause(world: u32, flow: f64) -> Result<(), String> {
    queue(world, flow, Op::Pause)
}

pub fn og_flow_skip(world: u32, flow: f64) -> Result<(), String> {
    queue(world, flow, Op::Skip)
}

pub fn og_flow_advance(world: u32, flow: f64, outcome: Option<String>) -> Result<(), String> {
    queue(world, flow, Op::Advance(outcome.filter(|o| !o.is_empty())))
}

pub fn og_flow_stop(world: u32, flow: f64) -> Result<(), String> {
    queue(world, flow, Op::Stop)
}

pub fn og_flow_set_speed(world: u32, flow: f64, speed: f64) -> Result<(), String> {
    queue(world, flow, Op::SetSpeed(speed as f32))
}

pub fn og_flow_seek(world: u32, flow: f64, time: f64) -> Result<(), String> {
    queue(world, flow, Op::Seek(time as f32))
}

pub fn og_flow_inspect(world: u32, flow: f64, hotspot: String) -> Result<(), String> {
    queue(world, flow, Op::Inspect(hotspot))
}

pub fn og_flow_gesture(world: u32, flow: f64, desc: Option<String>) -> Result<(), String> {
    let d: FlowGestureDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    if !matches!(d.kind.as_str(), "swipe" | "drag") {
        return Err(format!(
            "FlowGestureDesc.kind '{}' must be 'swipe' or 'drag'",
            d.kind
        ));
    }
    if !matches!(d.direction.as_str(), "up" | "down" | "left" | "right") {
        return Err(format!(
            "FlowGestureDesc.direction '{}' must be 'up', 'down', 'left', or 'right'",
            d.direction
        ));
    }
    let st = state(world)?;
    let key = flow as u64;
    if !lock(&st).views.contains_key(&key) {
        return Err(format!("entity {key} is no flow (og.flow.start)"));
    }
    lock(&st).saved.gestures.push((key, d));
    Ok(())
}

pub fn og_flow_set_music(world: u32, flow: f64, desc: Option<String>) -> Result<(), String> {
    let d: FlowMusicDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    queue(world, flow, Op::SetMusic(d))
}

pub fn og_flow_history(world: u32, flow: f64) -> Vec<FlowTranscriptLine> {
    let Ok(st) = state(world) else {
        return Vec::new();
    };
    lock(&st)
        .saved
        .history
        .get(&(flow as u64))
        .map(|h| h.lines.clone())
        .unwrap_or_default()
}

pub fn og_flow_checkpoint(world: u32, flow: f64) -> Result<FlowCheckpoint, String> {
    let st = state(world)?;
    let key = flow as u64;
    let s = lock(&st)
        .views
        .get(&key)
        .cloned()
        .ok_or_else(|| format!("entity {key} is no flow (og.flow.start)"))?;
    if s.node.is_empty() {
        return Err("the flow has no active node yet".into());
    }
    let dialogue = crate::bind::story::dialogue_of(world, s.conversation)
        .ok_or("the flow's conversation is gone")?;
    let vars = crate::bind::story::all_vars(world);
    Ok(FlowCheckpoint {
        dialogue,
        node: s.node,
        vars,
    })
}

pub fn og_flow_jump(world: u32, flow: f64, desc: Option<String>) -> Result<(), String> {
    let d: FlowJumpDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    queue(world, flow, Op::Jump(d))
}

pub fn og_flow_chart(world: u32, dialogue: String) -> Result<FlowChart, String> {
    let st = state(world)?;
    let p = lock(&st).flows.get(&dialogue).cloned().ok_or_else(|| {
        format!("the dialogue '{dialogue}' is not prepared: og.flow.load it in setup")
    })?;
    Ok(chart::build(world, &p))
}

// ── the system ───────────────────────────────────────────────────────────────

/// Registers og.flow on a world that names its records
/// (`crate::engine::lazy`): its system, just before og.timeline.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|r| world.id_by_name(r).is_err()) {
        // The last of og.flow's records to register schedules it.
        return Ok(());
    }
    // og.VideoPlayer is registered first (crate::engine::lazy's NEEDS).
    let engine_id = mount.id;
    let st = mount.modules.flow.clone();
    let mut writes: Vec<String> = RECORDS.iter().map(|s| s.to_string()).collect();
    writes.extend([TIMELINE.to_string(), PLAYER.to_string(), AUDIO.to_string()]);
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            writes,
            queries: vec![
                QueryDesc {
                    write: vec![FLOW.into()],
                    ..QueryDesc::default()
                },
                QueryDesc {
                    write: vec![DECK.into()],
                    ..QueryDesc::default()
                },
            ],
            ..Default::default()
        },
        move |ctx| system(ctx, engine_id, &st),
    )?;
    world.move_system_before(SYSTEM, crate::bind::timeline::TIMELINE_SYSTEM)
}

/// The layouts and ids the system writes.
struct Recs {
    flow: (SchemaId, Layout),
    deck: (SchemaId, Layout),
    timeline: (SchemaId, Layout),
    player: (SchemaId, Layout),
    audio: (SchemaId, Layout),
}

fn rec(ctx: &SystemContext<'_>, eng: &Engine, name: &str) -> og_core::Result<(SchemaId, Layout)> {
    let l = eng
        .state
        .borrow()
        .layouts
        .get(name)
        .cloned()
        .ok_or_else(|| bad(format!("{name} has no layout")))?;
    Ok((ctx.id_by_name(name)?, l))
}

fn patch(
    ctx: &mut SystemContext<'_>,
    r: &(SchemaId, Layout),
    e: Entity,
    v: Value,
) -> og_core::Result<()> {
    let old = ctx.get_bytes(e, r.0)?;
    let out = r.1.encode(Some(&old), Some(&v)).map_err(bad)?;
    if out != old {
        ctx.set_bytes(e, r.0, &out)?;
    }
    Ok(())
}

fn send(ctx: &mut SystemContext<'_>, eng: &Engine, name: &str, v: Value) -> og_core::Result<()> {
    let (id, l) = rec(ctx, eng, name)?;
    ctx.send_bytes(id, &l.encode(None, Some(&v)).map_err(bad)?)
}

/// One flow's decks, by entity id (a stable order).
type Decks = Vec<(Entity, record::FlowDeck)>;

fn system(ctx: &mut SystemContext<'_>, engine_id: u32, st: &ModuleState) -> og_core::Result<()> {
    let Ok(eng) = engine(engine_id) else {
        return Ok(());
    };
    let r = Recs {
        flow: rec(ctx, &eng, FLOW)?,
        deck: rec(ctx, &eng, DECK)?,
        timeline: rec(ctx, &eng, TIMELINE)?,
        player: rec(ctx, &eng, PLAYER)?,
        audio: rec(ctx, &eng, AUDIO)?,
    };
    let (ops, gestures) = {
        let mut rt = lock(st);
        (
            std::mem::take(&mut rt.saved.ops),
            std::mem::take(&mut rt.saved.gestures),
        )
    };
    let mut flows: Vec<Entity> = Vec::new();
    for a in ctx.query_with(&QueryDesc {
        write: vec![FLOW.into()],
        ..QueryDesc::default()
    })? {
        flows.extend_from_slice(a.entities());
    }
    let mut decks: Decks = Vec::new();
    let mut ids: Vec<Entity> = Vec::new();
    for a in ctx.query_with(&QueryDesc {
        write: vec![DECK.into()],
        ..QueryDesc::default()
    })? {
        ids.extend_from_slice(a.entities());
    }
    for e in ids {
        let d: record::FlowDeck = r.deck.1.read(&ctx.get_bytes(e, r.deck.0)?).map_err(bad)?;
        decks.push((e, d));
    }
    decks.sort_by_key(|(e, _)| e.to_bits());
    flows.sort_by_key(|e| e.to_bits());
    let mut views = BTreeMap::new();
    let mut suppressed = BTreeSet::new();
    for f in flows {
        let key = f.to_bits();
        let mine: Decks = decks
            .iter()
            .filter(|(_, d)| d.flow == key)
            .cloned()
            .collect();
        let mine_ops: Vec<Op> = ops
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, o)| o.clone())
            .collect();
        let mine_gestures: Vec<FlowGestureDesc> = gestures
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, g)| g.clone())
            .collect();
        if let Some((shown, sup)) = tick(ctx, &eng, st, &r, f, mine, &mine_ops, &mine_gestures)? {
            if let Some(p) = sup {
                suppressed.insert(p);
            }
            views.insert(key, shown);
        }
    }
    crate::bind::timeline::set_suppressed(engine_id, suppressed);
    lock(st).views = views;
    Ok(())
}

/// What a deck's og.Timeline and og.VideoPlayer showed last tick: its
/// time, whether it played, and the player's texture.
fn deck_state(ctx: &SystemContext<'_>, r: &Recs, e: Entity) -> og_core::Result<(f32, bool, u64)> {
    let t: record::Timeline = r
        .timeline
        .1
        .read(&ctx.get_bytes(e, r.timeline.0)?)
        .map_err(bad)?;
    let p: record::VideoPlayer = r
        .player
        .1
        .read(&ctx.get_bytes(e, r.player.0)?)
        .map_err(bad)?;
    Ok((t.time, t.playing, p.texture))
}

/// Puts node `id`'s segment on `deck`: playing from 0, or in pre-roll
/// (og.Timeline.preroll), where og.timeline has the deck's player hold the
/// segment's first frame and shows nothing of it.
#[allow(clippy::too_many_arguments)]
fn load_deck(
    ctx: &mut SystemContext<'_>,
    r: &Recs,
    deck: Entity,
    id: u32,
    node: &Node,
    playing: bool,
    speed: f32,
    preroll: bool,
) -> og_core::Result<()> {
    patch(
        ctx,
        &r.timeline,
        deck,
        json!({
            "asset": node.timeline as f64,
            "time": 0.0,
            "playing": playing,
            "loop": false,
            "loopStart": 0.0,
            "loopEnd": 0.0,
            "speed": speed,
            "preroll": preroll,
        }),
    )?;
    patch(ctx, &r.audio, deck, json!({ "volume": 1.0 }))?;
    patch(ctx, &r.deck, deck, json!({ "node": id }))
}

/// Lets go of a deck: no segment, its player and audio stopped.
fn free_deck(ctx: &mut SystemContext<'_>, r: &Recs, deck: Entity) -> og_core::Result<()> {
    patch(
        ctx,
        &r.timeline,
        deck,
        json!({
            "asset": 0.0,
            "time": 0.0,
            "playing": false,
            "loop": false,
            "loopStart": 0.0,
            "loopEnd": 0.0,
            "preroll": false,
        }),
    )?;
    patch(
        ctx,
        &r.player,
        deck,
        json!({ "video": 0.0, "playing": false }),
    )?;
    patch(
        ctx,
        &r.audio,
        deck,
        json!({ "playing": false, "volume": 1.0 }),
    )?;
    patch(ctx, &r.deck, deck, json!({ "node": 0 }))
}

/// Starts, crossfades, or stops a flow's persistent cross-segment BGM.
#[allow(clippy::too_many_arguments)]
fn apply_music_cmd(
    ctx: &mut SystemContext<'_>,
    eng: &Engine,
    st: &ModuleState,
    r: &Recs,
    flow_key: u64,
    desc: &FlowMusicDesc,
    paused: bool,
    dt: f32,
) -> og_core::Result<()> {
    let mut m = lock(st)
        .saved
        .music
        .get(&flow_key)
        .cloned()
        .unwrap_or_default();
    let fade_ticks = if desc.fade > 0.0 && dt > 0.0 {
        (desc.fade / dt).ceil() as u32
    } else {
        0
    };
    let target_vol = desc.volume.clamp(0.0, 2.0);
    let target_duck = desc.duck.clamp(0.0, 1.0);
    // Same track already playing: just update volume/duck targets.
    if !desc.sound.is_empty() && desc.sound == m.sound && m.entity != 0 {
        let e = Entity::from_bits(m.entity);
        if ctx.is_alive(e) {
            m.volume = target_vol;
            m.duck = target_duck;
            lock(st).saved.music.insert(flow_key, m);
            return Ok(());
        }
    }
    // Retire any previous fading-out entity.
    if m.prev_entity != 0 {
        let pe = Entity::from_bits(m.prev_entity);
        if ctx.is_alive(pe) {
            ctx.despawn(pe)?;
        }
        m.prev_entity = 0;
    }
    if desc.sound.is_empty() {
        // Stop BGM (with optional fade-out).
        if m.entity != 0 {
            if fade_ticks == 0 {
                let e = Entity::from_bits(m.entity);
                if ctx.is_alive(e) {
                    ctx.despawn(e)?;
                }
                m.entity = 0;
            } else {
                m.prev_entity = m.entity;
                m.entity = 0;
            }
        }
        m.sound.clear();
        m.fade_ticks = fade_ticks;
        m.fade_left = fade_ticks;
        lock(st).saved.music.insert(flow_key, m);
        return Ok(());
    }
    // Load new BGM sound handle.
    let sound_handle = match crate::world::assets::load_sound(eng, &desc.sound) {
        Ok(i) => eng.assets.borrow().sound_handles[i],
        Err(e) => {
            warn(
                eng,
                st,
                format!("flow {flow_key}: music '{}' did not load: {e}", desc.sound),
            );
            return Ok(());
        }
    };
    if m.entity != 0 {
        if fade_ticks == 0 {
            let e = Entity::from_bits(m.entity);
            if ctx.is_alive(e) {
                ctx.despawn(e)?;
            }
        } else {
            m.prev_entity = m.entity;
        }
    }
    let init_vol = if fade_ticks > 0 {
        0.0
    } else {
        target_vol * m.current_duck
    };
    let v = json!({
        "sound": sound_handle as f64,
        "bus": "music",
        "volume": init_vol,
        "loop": true,
        "playing": !paused,
    });
    let bytes = r.audio.1.encode(None, Some(&v)).map_err(bad)?;
    let e = ctx.spawn_bytes(vec![(r.audio.0, bytes)])?;
    m.sound = desc.sound.clone();
    m.entity = e.to_bits();
    m.volume = target_vol;
    m.duck = target_duck;
    m.fade_ticks = fade_ticks;
    m.fade_left = fade_ticks;
    lock(st).saved.music.insert(flow_key, m);
    Ok(())
}

/// Steps persistent cross-segment BGM crossfade and automatic voice ducking.
fn step_music(
    ctx: &mut SystemContext<'_>,
    st: &ModuleState,
    r: &Recs,
    flow_key: u64,
    speaking: bool,
    paused: bool,
    dt: f32,
) -> og_core::Result<()> {
    let Some(mut m) = lock(st).saved.music.get(&flow_key).cloned() else {
        return Ok(());
    };
    if m.entity == 0 && m.prev_entity == 0 {
        return Ok(());
    }
    if !paused {
        if m.fade_left > 0 {
            m.fade_left -= 1;
        }
        let target_duck = if speaking { m.duck } else { 1.0 };
        let rate = if speaking { 4.0 } else { 2.0 };
        let step = (rate * dt).min(1.0);
        if m.current_duck < target_duck {
            m.current_duck = (m.current_duck + step).min(target_duck);
        } else {
            m.current_duck = (m.current_duck - step).max(target_duck);
        }
    }
    let fade_mix = if m.fade_ticks > 0 && m.fade_left > 0 {
        1.0 - m.fade_left as f32 / m.fade_ticks as f32
    } else {
        1.0
    };
    if m.entity != 0 {
        let e = Entity::from_bits(m.entity);
        if !ctx.is_alive(e) {
            m.entity = 0;
        } else if ctx.get_bytes(e, r.audio.0).is_ok() {
            // Entities spawned via `ctx.spawn_bytes` on this tick flush after
            // the system returns (their initial volume/playing are already set).
            let vol = m.volume * fade_mix * m.current_duck;
            patch(
                ctx,
                &r.audio,
                e,
                json!({ "volume": vol, "playing": !paused }),
            )?;
        }
    }
    if m.prev_entity != 0 {
        let pe = Entity::from_bits(m.prev_entity);
        if m.fade_left == 0 || !ctx.is_alive(pe) {
            if ctx.is_alive(pe) {
                ctx.despawn(pe)?;
            }
            m.prev_entity = 0;
        } else if ctx.get_bytes(pe, r.audio.0).is_ok() {
            let prev_vol = m.volume * (1.0 - fade_mix) * m.current_duck;
            patch(
                ctx,
                &r.audio,
                pe,
                json!({ "volume": prev_vol, "playing": !paused }),
            )?;
        }
    }
    lock(st).saved.music.insert(flow_key, m);
    Ok(())
}

fn despawn_music(
    ctx: &mut SystemContext<'_>,
    st: &ModuleState,
    flow_key: u64,
) -> og_core::Result<()> {
    if let Some(m) = lock(st).saved.music.remove(&flow_key) {
        for bits in [m.entity, m.prev_entity] {
            if bits != 0 {
                let e = Entity::from_bits(bits);
                if ctx.is_alive(e) {
                    ctx.despawn(e)?;
                }
            }
        }
    }
    Ok(())
}

/// Records captions newly shown on `deck` into the flow's transcript backlog
/// directly from `timeline` at time `t` (deterministic across save/restore).
fn record_captions(
    eng: &Engine,
    st: &ModuleState,
    flow_key: u64,
    deck_bits: u64,
    timeline: u64,
    node_title: &str,
    t: f32,
) -> bool {
    let caps = crate::bind::timeline::captions_at(eng.id, timeline, deck_bits, t);
    let speaking = !caps.is_empty();
    let mut rt = lock(st);
    let h = rt.saved.history.entry(flow_key).or_default();
    let mut cur_ids = BTreeSet::new();
    for cap in caps {
        cur_ids.insert(cap.id.clone());
        if !h.last_caption_ids.contains(&cap.id) {
            h.push(
                node_title.to_string(),
                cap.speaker,
                cap.text,
                cap.spans,
                false,
            );
        }
    }
    h.last_caption_ids = cur_ids;
    speaking
}

#[allow(clippy::too_many_arguments)]
fn tick(
    ctx: &mut SystemContext<'_>,
    eng: &Engine,
    st: &ModuleState,
    r: &Recs,
    f: Entity,
    mut decks: Decks,
    ops: &[Op],
    gestures: &[FlowGestureDesc],
) -> og_core::Result<Option<(Shown, Option<u64>)>> {
    let key = f.to_bits();
    let fb = ctx.get_bytes(f, r.flow.0)?;
    let mut flow: record::Flow = r.flow.1.read(&fb).map_err(bad)?;
    if flow.speed <= 0.0 {
        flow.speed = 1.0;
    }
    if flow.qte.len() != QTE_CHOICES {
        flow.qte.resize(QTE_CHOICES, 0);
    }
    if flow.qte_state.len() != QTE_CHOICES {
        flow.qte_state.resize(QTE_CHOICES, 0);
    }
    let conv = flow.conversation;
    let dialogue = crate::bind::story::dialogue_of(eng.id, conv);
    let Some(prep) = dialogue
        .as_ref()
        .and_then(|d| lock(st).flows.get(d).cloned())
    else {
        warn(
            eng,
            st,
            format!(
                "flow {key}: its dialogue {} is not prepared (og.flow.load it in setup, in every process that restores a flow)",
                dialogue.as_deref().unwrap_or("(its conversation is gone)")
            ),
        );
        return Ok(None);
    };
    let dt = ctx.time().dt();
    let half = dt * 0.5;
    let current = |flow: &record::Flow| Entity::from_bits(flow.current);
    let mut advancing = flow.node != 0 && deck_state(ctx, r, current(&flow))?.1;
    let shows =
        |ctx: &SystemContext<'_>, flow: &record::Flow, advancing: bool| -> og_core::Result<f32> {
            let t = deck_state(ctx, r, current(flow))?.0;
            Ok(if advancing { t + dt * flow.speed } else { t })
        };
    let mut advance: Option<Option<String>> = None;
    let mut jump_to: Option<FlowJumpDesc> = None;
    let mut inspect_ids: Vec<String> = Vec::new();
    let mut music_cmds: Vec<FlowMusicDesc> = Vec::new();
    for op in ops {
        match op.clone() {
            Op::Advance(o) => advance = Some(o),
            Op::Choose(i) if flow.window == 1 => flow.pending = i as i32,
            Op::Choose(_) => {}
            Op::Play | Op::Pause => {
                flow.paused = *op == Op::Pause;
                advancing &= !flow.paused;
                if !flow.ended && flow.node != 0 {
                    // A held segment stays stopped at its hold.
                    let playing = json!({
                        "playing": !flow.paused && !flow.held,
                        "preroll": false,
                    });
                    patch(ctx, &r.timeline, current(&flow), playing.clone())?;
                    if let Some(p) = previous(&flow) {
                        patch(ctx, &r.timeline, p, playing)?;
                    }
                }
            }
            Op::SetSpeed(sp) => {
                flow.speed = sp.clamp(0.25, 4.0);
                if !flow.ended && flow.node != 0 {
                    patch(
                        ctx,
                        &r.timeline,
                        current(&flow),
                        json!({ "speed": flow.speed }),
                    )?;
                }
            }
            Op::Seek(target_time) => {
                if let Some(node) = prep.nodes.get(&flow.node).filter(|_| !flow.ended) {
                    let to = target_time.clamp(0.0, node.cut);
                    let was_held = flow.held;
                    flow.held = node.hold.is_some_and(|h| to + half >= h);
                    if flow.looping
                        && !node
                            .loop_region
                            .is_some_and(|(a, b)| to + half >= a && to < b)
                    {
                        flow.looping = false;
                        patch(
                            ctx,
                            &r.timeline,
                            current(&flow),
                            json!({ "loop": false, "loopStart": 0.0, "loopEnd": 0.0 }),
                        )?;
                    }
                    let mut p = json!({ "time": to });
                    if was_held && !flow.held && !flow.paused {
                        p["playing"] = json!(true);
                    } else if flow.held {
                        p["playing"] = json!(false);
                    }
                    patch(ctx, &r.timeline, current(&flow), p)?;
                    advancing = false;
                }
            }
            Op::Skip => {
                // Skip is ignored while a playable segment is held at flow.hold.
                if !flow.held
                    && let Some(node) = prep.nodes.get(&flow.node).filter(|_| !flow.ended)
                {
                    let t = shows(ctx, &flow, advancing)?;
                    let choosing =
                        og_story_view(eng.id, conv).is_some_and(|v| !seen(v.choices).is_empty());
                    let to = if choosing && t + half < node.open {
                        node.open
                    } else {
                        node.cut
                    };
                    if flow.looping {
                        flow.looping = false;
                        patch(
                            ctx,
                            &r.timeline,
                            current(&flow),
                            json!({ "loop": false, "loopStart": 0.0, "loopEnd": 0.0 }),
                        )?;
                    }
                    patch(ctx, &r.timeline, current(&flow), json!({ "time": to }))?;
                    advancing = false;
                }
            }
            Op::Inspect(id) => inspect_ids.push(id),
            Op::SetMusic(m) => music_cmds.push(m),
            Op::Jump(j) => jump_to = Some(j),
            Op::Stop => {
                og_story_stop(eng.id, conv);
                despawn_music(ctx, st, key)?;
                lock(st).saved.remove_flow(key);
                for (d, _) in &decks {
                    ctx.despawn(*d)?;
                }
                ctx.despawn(f)?;
                return Ok(None);
            }
        }
    }

    // Apply explicit music commands queued before segment cuts.
    for mc in music_cmds {
        apply_music_cmd(ctx, eng, st, r, key, &mc, flow.paused, dt)?;
    }

    // Process inspect hotspots without cutting.
    if !inspect_ids.is_empty()
        && !flow.ended
        && let Some(node) = prep.nodes.get(&flow.node)
    {
        let t = shows(ctx, &flow, advancing)?;
        let active = og_timeline::eval::hotspots_at(&node.hotspots, t);
        for id in inspect_ids {
            if active
                .iter()
                .any(|h| h.kind == HotspotKind::Inspect && h.id == id)
            {
                let mark = format!("{}:{id}", node.title);
                lock(st)
                    .saved
                    .inspected
                    .entry(key)
                    .or_default()
                    .insert(mark);
                send(
                    ctx,
                    eng,
                    INSPECTED,
                    json!({
                        "flow": key as f64,
                        "node": node.title,
                        "hotspot": id,
                    }),
                )?;
            }
        }
    }

    // The dissolve runs out: the deck it showed under lets go.
    if flow.dissolve_left > 0 && !flow.paused {
        flow.dissolve_left -= 1;
    }
    if flow.dissolve_left == 0
        && let Some(p) = previous(&flow)
    {
        free_deck(ctx, r, p)?;
        flow.previous = 0;
        flow.dissolve = 0;
        patch(ctx, &r.audio, current(&flow), json!({ "volume": 1.0 }))?;
    }

    let mut live_handoff = false;

    // Begins node `title`'s segment on a deck (a warm one if it holds it),
    // then runs the node to its choices (or on to its next hold).
    let mut begin = |ctx: &mut SystemContext<'_>,
                     flow: &mut record::Flow,
                     decks: &mut Decks,
                     title: &str|
     -> og_core::Result<()> {
        let id = node_id(title);
        let Some(node) = prep.nodes.get(&id) else {
            warn(
                eng,
                st,
                format!("flow {key}: node '{title}' holds without a segment"),
            );
            return Ok(());
        };
        let cur = current(flow);
        let prev = previous(flow);
        let prev_node = prep.nodes.get(&flow.node);
        let prev_was_footage = prev_node.is_some_and(|n| !n.under && n.first.is_some());
        let reuse_paused_open =
            flow.node == 0 && decks.iter().any(|(d, k)| *d == cur && k.node == id);
        let target = if reuse_paused_open || flow.decks <= 1 {
            cur
        } else {
            decks
                .iter()
                .find(|(d, k)| *d != cur && k.node == id)
                .or_else(|| decks.iter().find(|(d, _)| *d != cur && Some(*d) != prev))
                .or_else(|| decks.iter().find(|(d, _)| *d != cur))
                .map_or(cur, |(d, _)| *d)
        };
        // A dissolve still running ends at this cut.
        if let Some(p) = prev {
            if p != target {
                free_deck(ctx, r, p)?;
            }
            flow.previous = 0;
            flow.dissolve = 0;
            flow.dissolve_left = 0;
        }
        if target != cur && decks.iter().any(|(d, _)| *d == cur) {
            let ticks = if node.dissolve > 0.0 && flow.node != 0 && dt > 0.0 {
                (node.dissolve / dt).ceil() as u32
            } else {
                0
            };
            if ticks > 0 {
                // Keep `deck.node` intact on `cur` so `picture(previous)` and
                // `dump_flow` know what node is dissolving out.
                flow.previous = cur.to_bits();
                flow.dissolve = ticks;
                flow.dissolve_left = ticks;
            } else if node.live && node.first.is_none() && prev_was_footage && flow.node != 0 {
                // 1-tick footage -> live handoff: keep `cur` paused on its last
                // frame (muted) while `target`'s live scene queues for next
                // `before_tick`, eliminating the 1-frame black void.
                patch(ctx, &r.timeline, cur, json!({ "playing": false }))?;
                patch(
                    ctx,
                    &r.audio,
                    cur,
                    json!({ "playing": false, "volume": 0.0 }),
                )?;
                flow.previous = cur.to_bits();
                flow.dissolve = 1;
                flow.dissolve_left = 1;
                live_handoff = true;
            } else {
                free_deck(ctx, r, cur)?;
            }
        }
        // Treat a deck as warm when switching to a different pre-warmed deck, or
        // when unpausing an opening preview loaded at `flow.node == 0`. Self-loops
        // on `decks: 1` (`flow.node != 0 && target == cur`) always reload from t = 0.
        let warm = reuse_paused_open
            || (target != cur && decks.iter().any(|(d, k)| *d == target && k.node == id));
        if warm {
            patch(
                ctx,
                &r.timeline,
                target,
                json!({
                    "playing": !flow.paused,
                    "speed": flow.speed,
                    "loop": false,
                    "loopStart": 0.0,
                    "loopEnd": 0.0,
                    "preroll": false,
                }),
            )?;
            patch(ctx, &r.audio, target, json!({ "volume": 1.0 }))?;
        } else {
            load_deck(ctx, r, target, id, node, !flow.paused, flow.speed, false)?;
        }
        for (d, k) in decks.iter_mut() {
            if *d == target {
                k.node = id;
            } else if *d == cur && target != cur && flow.previous != cur.to_bits() {
                k.node = 0;
            }
        }
        flow.current = target.to_bits();
        flow.node = id;
        flow.pending = -1;
        flow.window = 0;
        flow.held = false;
        flow.looping = false;
        flow.ended = false;
        // A cut: the camera the new segment places is drawn where it is,
        // never blended from the last one's.
        crate::host::api::og_render_snap_camera(eng.id);
        flow.qte = vec![0; QTE_CHOICES];
        flow.qte_state = vec![0; QTE_CHOICES];
        send(
            ctx,
            eng,
            SEGMENT,
            json!({ "flow": key as f64, "node": node.title, "segment": node.segment }),
        )?;
        if !node.chapter.is_empty() {
            flow.chapter = id;
            send(
                ctx,
                eng,
                CHAPTER,
                json!({ "flow": key as f64, "node": node.title, "chapter": node.chapter }),
            )?;
        }
        if let Some(ref mh) = node.music {
            let desc = FlowMusicDesc {
                sound: mh.sound.clone(),
                volume: if mh.sound.is_empty() { 0.0 } else { mh.volume },
                fade: mh.fade,
                duck: mh.duck,
            };
            apply_music_cmd(ctx, eng, st, r, key, &desc, flow.paused, dt)?;
        }
        if let Err(e) = og_story_advance(eng.id, conv) {
            warn(eng, st, format!("flow {key}: node '{title}': {e}"));
        }
        skip_lines(eng, st, f, conv);
        Ok(())
    };

    // Handle explicit branch jump (`og.flow.jump`) before regular playback.
    if let Some(j) = jump_to {
        if let Err(e) = crate::bind::story::set_vars(eng.id, &j.vars) {
            warn(eng, st, format!("flow {key}: jump vars: {e}"));
        }
        match crate::bind::story::jump_conv(eng.id, conv, &j.node) {
            Ok(()) => {
                if let Some(p) = previous(&flow) {
                    free_deck(ctx, r, p)?;
                    flow.previous = 0;
                    flow.dissolve = 0;
                    flow.dissolve_left = 0;
                }
                let target_node = og_story_view(eng.id, conv)
                    .filter(|v| v.held)
                    .map(|v| v.node)
                    .unwrap_or_else(|| j.node.clone());
                begin(ctx, &mut flow, &mut decks, &target_node)?;
                advancing = false;
            }
            Err(e) => warn(eng, st, format!("flow {key}: jump '{}': {e}", j.node)),
        }
    } else if !flow.ended {
        let view = og_story_view(eng.id, conv);
        if flow.node == 0 {
            match &view {
                Some(v) if v.held => {
                    if flow.paused {
                        // Paused start: show the opening frame on the deck
                        // without firing FlowSegment/FlowChapter or advancing
                        // the conversation until `og.flow.play` unpauses it.
                        let id = node_id(&v.node);
                        if let Some(node) = prep.nodes.get(&id) {
                            let cur = current(&flow);
                            let loaded_already =
                                decks.iter().any(|(d, k)| *d == cur && k.node == id);
                            if !loaded_already {
                                load_deck(ctx, r, cur, id, node, false, flow.speed, false)?;
                                // A timeline loaded paused evaluates nothing
                                // until a play or a seek: seek it to its start
                                // so its video track hands the deck's player
                                // the footage and the opening frame shows.
                                crate::bind::timeline::queue_seek(eng.id, cur.to_bits(), 0.0);
                                for (d, k) in decks.iter_mut() {
                                    if *d == cur {
                                        k.node = id;
                                    }
                                }
                            }
                        }
                    } else {
                        begin(ctx, &mut flow, &mut decks, &v.node)?;
                        advancing = false;
                    }
                }
                _ => flow.ended = true,
            }
        } else if let Some(node) = prep.nodes.get(&flow.node) {
            let mut t = shows(ctx, &flow, advancing)?;
            // A playable segment stops at its hold, its time written there.
            if let Some(h) = node.hold
                && !flow.held
                && !flow.paused
                && t + half >= h
            {
                flow.held = true;
                patch(
                    ctx,
                    &r.timeline,
                    current(&flow),
                    json!({ "time": h, "playing": false }),
                )?;
                advancing = false;
                t = h;
            }
            let choosing = view
                .as_ref()
                .is_some_and(|v| v.choices.iter().any(|c| !outcome(c)));
            let prev_window = flow.window;
            flow.window = if !choosing || t + half < node.open {
                0
            } else if flow.held || flow.looping || t + half < node.lock {
                1
            } else {
                2
            };
            // Idle choice loop (`flow.loopStart`..`flow.loopEnd`): loop the
            // choice beat seamlessly while waiting for the player to decide.
            if let Some((ls, le)) = node.loop_region {
                if !flow.paused
                    && !flow.held
                    && flow.window == 1
                    && flow.pending < 0
                    && t + half >= ls
                {
                    if !flow.looping {
                        flow.looping = true;
                        patch(
                            ctx,
                            &r.timeline,
                            current(&flow),
                            json!({ "loop": true, "loopStart": ls, "loopEnd": le }),
                        )?;
                    }
                } else if flow.looping && (flow.pending >= 0 || flow.window != 1) {
                    flow.looping = false;
                    patch(
                        ctx,
                        &r.timeline,
                        current(&flow),
                        json!({ "loop": false, "loopStart": 0.0, "loopEnd": 0.0 }),
                    )?;
                }
            }
            if flow.window == 1
                && !flow.paused
                && let Some(v) = &view
            {
                qte_tick(ctx, eng, st, f, &mut flow, node, v, gestures, t, dt)?;
            } else if prev_window == 1
                && flow.window == 2
                && let Some(v) = &view
            {
                qte_timeout(ctx, eng, f, &mut flow, node, v)?;
            }
            // Release idle loop if a QTE completed and populated `flow.pending`.
            if flow.looping && flow.pending >= 0 {
                flow.looping = false;
                patch(
                    ctx,
                    &r.timeline,
                    current(&flow),
                    json!({ "loop": false, "loopStart": 0.0, "loopEnd": 0.0 }),
                )?;
            }
            // Immediate choice: selecting a `#immediate` choice in an open
            // window cuts on this tick without waiting for `flow.cut`.
            let immediate_chosen = !flow.paused
                && flow.window == 1
                && u32::try_from(flow.pending).ok().is_some_and(|idx| {
                    view.as_ref()
                        .and_then(|v| v.choices.iter().find(|c| c.index == idx))
                        .and_then(|c| node.choice_meta(c))
                        .is_some_and(|m| m.immediate)
                });
            let timed =
                !flow.paused && !flow.looping && node.hold.is_none() && t + half >= node.cut;
            // A choice the player takes while held cuts at once (unless paused).
            let chosen_held = !flow.paused && flow.held && flow.pending >= 0;
            if timed || chosen_held || immediate_chosen || advance.is_some() {
                let title = node.title.clone();
                if let Some(v) = view.as_ref().filter(|v| !v.choices.is_empty()) {
                    let by_outcome = advance
                        .clone()
                        .flatten()
                        .and_then(|o| node.outcomes.iter().find(|(_, n)| *n == o).map(|(i, _)| *i));
                    let pending = u32::try_from(flow.pending).ok().filter(|i| {
                        v.choices
                            .iter()
                            .any(|c| c.index == *i && c.enabled && !outcome(c))
                    });
                    let taken = by_outcome.or(pending);
                    if let Some(i) = taken.or_else(|| default_choice(&v.choices)) {
                        // Record player choice in transcript backlog (skip hidden #outcome).
                        if let Some(chosen_view) =
                            v.choices.iter().find(|c| c.index == i && !outcome(c))
                        {
                            lock(st).saved.history.entry(key).or_default().push(
                                title.clone(),
                                String::new(),
                                chosen_view.text.clone(),
                                chosen_view.spans.clone(),
                                true,
                            );
                        }
                        send(
                            ctx,
                            eng,
                            CHOICE,
                            json!({
                                "flow": key as f64,
                                "node": title,
                                "index": i,
                                "timedOut": taken.is_none() && advance.is_none(),
                            }),
                        )?;
                        if let Err(e) = og_story_choose(eng.id, conv, i) {
                            warn(eng, st, format!("flow {key}: node '{title}': {e}"));
                        }
                    }
                }
                skip_lines(eng, st, f, conv);
                match og_story_view(eng.id, conv) {
                    Some(v) if v.held => {
                        begin(ctx, &mut flow, &mut decks, &v.node)?;
                        advancing = false;
                    }
                    Some(v) if !v.ended => {
                        warn(
                            eng,
                            st,
                            format!(
                                "flow {key}: after '{title}' the conversation shows choices outside a flow node: give that node an `og-flow` segment"
                            ),
                        );
                        flow.ended = true;
                    }
                    _ => {
                        flow.ended = true;
                        flow.window = 0;
                        flow.looping = false;
                        send(
                            ctx,
                            eng,
                            ENDED,
                            json!({ "flow": key as f64, "node": title, "ending": node.ending }),
                        )?;
                    }
                }
            }
        }
    }

    // Audio crossfade during dissolve (and suppress previous deck's secondary tracks).
    let mix = if flow.dissolve_left > 0 && flow.dissolve > 0 {
        1.0 - flow.dissolve_left as f32 / flow.dissolve as f32
    } else {
        1.0
    };
    let suppressed_prev = if let Some(p) = previous(&flow) {
        if !live_handoff {
            patch(ctx, &r.audio, current(&flow), json!({ "volume": mix }))?;
            patch(ctx, &r.audio, p, json!({ "volume": (1.0 - mix).max(0.0) }))?;
        }
        Some(p.to_bits())
    } else {
        None
    };

    // Record active captions from the current deck and step persistent BGM ducking.
    let speaking = if flow.node != 0
        && !flow.ended
        && let Some(active_node) = prep.nodes.get(&flow.node)
    {
        let active_t = shows(ctx, &flow, advancing)?;
        record_captions(
            eng,
            st,
            key,
            flow.current,
            active_node.timeline,
            &active_node.title,
            active_t,
        )
    } else {
        false
    };
    step_music(ctx, st, r, key, speaking, flow.paused, dt)?;

    // Warm the segments the current node may lead to on the other decks.
    if flow.decks > 1
        && !flow.ended
        && let Some(node) = prep.nodes.get(&flow.node)
    {
        let t = shows(ctx, &flow, advancing)?;
        let view = og_story_view(eng.id, conv);
        let mut want: Vec<u32> = Vec::new();
        match &view {
            Some(v) if v.held => want.push(node_id(&v.node)),
            Some(v) if !v.choices.is_empty() => {
                // A playable segment's outcomes are known from its start.
                if t + half >= node.open || node.hold.is_some() {
                    let mut order: Vec<u32> =
                        u32::try_from(flow.pending).ok().into_iter().collect();
                    order.extend(default_choice(&v.choices));
                    order.extend(v.choices.iter().filter(|c| c.enabled).map(|c| c.index));
                    for idx in order {
                        if let Some(c) = v.choices.iter().find(|c| c.index == idx)
                            && let Some(meta) = node.choice_meta(c)
                        {
                            want.extend(&meta.to);
                        } else if let Some(meta) = node.choice_by_index(idx) {
                            want.extend(&meta.to);
                        }
                    }
                }
            }
            _ => want.extend(&node.next),
        }
        let mut seen_ids = BTreeSet::new();
        want.retain(|w| seen_ids.insert(*w));
        want.truncate((flow.decks - 1) as usize);
        let cur = current(&flow);
        let prev = previous(&flow);
        let spare = |d: &Entity| *d != cur && Some(*d) != prev;
        let wanted: BTreeSet<u32> = want.iter().copied().collect();
        for w in want {
            if decks.iter().any(|(d, k)| spare(d) && k.node == w) {
                continue;
            }
            // A deck holding nothing the node may lead to.
            let free = decks
                .iter()
                .position(|(d, k)| spare(d) && !wanted.contains(&k.node));
            let (Some(i), Some(n)) = (free, prep.nodes.get(&w)) else {
                break;
            };
            load_deck(ctx, r, decks[i].0, w, n, false, 1.0, true)?;
            decks[i].1.node = w;
        }
    }

    // A spare deck holding a segment the flow can no longer cut to lets go.
    let reach: BTreeSet<u32> = match prep.nodes.get(&flow.node) {
        Some(node) if !flow.ended => node
            .choices_list
            .iter()
            .flat_map(|c| c.to.iter())
            .chain(&node.next)
            .copied()
            .collect(),
        _ => BTreeSet::new(),
    };
    let cur = current(&flow);
    let prev = previous(&flow);
    for (d, k) in decks.iter_mut() {
        if *d != cur && Some(*d) != prev && k.node != 0 && !reach.contains(&k.node) {
            free_deck(ctx, r, *d)?;
            k.node = 0;
        }
    }

    let out = r
        .flow
        .1
        .encode(Some(&fb), Some(&flow_json(&flow)))
        .map_err(bad)?;
    if out != fb {
        ctx.set_bytes(f, r.flow.0, &out)?;
    }
    // Opening paused preview: if flow.node == 0 && paused, use deck's loaded node for picture.
    let preview_node_id = if flow.node == 0 && flow.paused {
        decks
            .iter()
            .find(|(d, _)| *d == cur)
            .map(|(_, k)| k.node)
            .unwrap_or(0)
    } else {
        flow.node
    };
    let node = prep.nodes.get(&preview_node_id);
    let time = shows(ctx, &flow, advancing)?;
    let picture = |deck: Entity| -> og_core::Result<(u64, bool)> {
        let held = r
            .deck
            .1
            .read::<record::FlowDeck>(&ctx.get_bytes(deck, r.deck.0)?)
            .map_err(bad)?;
        if prep
            .nodes
            .get(&held.node)
            .is_some_and(|n| !n.under && n.first.is_some())
        {
            Ok((deck_state(ctx, r, deck)?.2, true))
        } else {
            Ok((0, false))
        }
    };
    let (texture, has_picture) = picture(current(&flow))?;
    let (previous_texture, prev_has_picture) = match previous(&flow) {
        Some(p) => picture(p)?,
        None => (0, false),
    };
    let inspected_set = lock(st).saved.inspected.get(&key).cloned();
    let (hotspots, qtes) = match prep.nodes.get(&flow.node) {
        Some(n) if !flow.ended => {
            let choices = if flow.window > 0 {
                og_story_view(eng.id, conv)
                    .map(|v| seen(v.choices))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            offers(n, &flow, &choices, inspected_set.as_ref(), time, dt)
        }
        _ => (Vec::new(), Vec::new()),
    };
    let active_node = prep.nodes.get(&flow.node);
    Ok(Some((
        Shown {
            conversation: conv,
            deck: flow.current,
            previous_deck: flow.previous,
            has_picture,
            prev_has_picture,
            live_handoff,
            node: active_node.map(|n| n.title.clone()).unwrap_or_default(),
            segment: active_node.map(|n| n.segment.clone()).unwrap_or_default(),
            texture,
            time,
            open: active_node.map_or(0.0, |n| n.open),
            lock: active_node.map_or(0.0, |n| n.lock),
            cut: active_node.map_or(0.0, |n| n.cut),
            speed: flow.speed,
            video_width: node.map_or(0, |n| n.video_width),
            video_height: node.map_or(0, |n| n.video_height),
            dt,
            window: flow.window,
            pending: flow.pending,
            paused: flow.paused,
            looping: flow.looping,
            chapter: prep
                .nodes
                .get(&flow.chapter)
                .map(|n| n.chapter.clone())
                .unwrap_or_default(),
            ending: if flow.ended {
                active_node.map(|n| n.ending.clone()).unwrap_or_default()
            } else {
                String::new()
            },
            ended: flow.ended,
            held: flow.held,
            outcomes: active_node
                .map(|n| n.outcomes.iter().map(|(_, o)| o.clone()).collect())
                .unwrap_or_default(),
            hotspots,
            qtes,
            previous: previous_texture,
            mix,
            ahead: match active_node {
                Some(n) if !flow.ended => ahead(&prep, n, time),
                _ => Vec::new(),
            },
        },
        suppressed_prev,
    )))
}

/// Every segment the node in place may cut to (each choice's and its own
/// steps' flow nodes), by its first frame, needed at the cut.
fn ahead(prep: &Prepared, node: &Node, time: f32) -> Vec<Ahead> {
    let seconds = f64::from((node.cut - time).max(0.0));
    let mut seen_ids = BTreeSet::new();
    node.choices_list
        .iter()
        .flat_map(|c| c.to.iter())
        .chain(&node.next)
        .filter(|id| seen_ids.insert(**id))
        .filter_map(|id| prep.nodes.get(id)?.first)
        .map(|(video, from)| Ahead {
            video,
            from,
            seconds,
        })
        .collect()
}

/// The first frames the flows on `eng` may cut to (crate::bind::video's
/// `wanted_chunks` asks for their chunks).
pub(crate) fn prefetch(eng: &Engine) -> Vec<Ahead> {
    lock(&eng.modules.flow)
        .views
        .values()
        .flat_map(|s| s.ahead.iter().copied())
        .collect()
}

/// The deck dissolving out under the current one, if any.
fn previous(flow: &record::Flow) -> Option<Entity> {
    (flow.previous != 0).then(|| Entity::from_bits(flow.previous))
}

/// Ticks or steps of input a quick-time event needs.
fn qte_need(q: &Qte, dt: f32) -> u32 {
    match q.kind {
        QteKind::Press => 1,
        QteKind::Hold(ms) if dt > 0.0 => ((ms / 1000.0) / dt).ceil().max(1.0) as u32,
        QteKind::Hold(_) => 1,
        QteKind::Mash(n) => n.max(1),
        QteKind::Timing(_, _) => 1,
        QteKind::Sequence => q.sequence.len().max(1) as u32,
        QteKind::Swipe => 1,
        QteKind::Drag => 1000,
    }
}

fn emit_qte_result(
    ctx: &mut SystemContext<'_>,
    eng: &Engine,
    f: Entity,
    node: &Node,
    choice: u32,
    grade: &str,
) -> og_core::Result<()> {
    send(
        ctx,
        eng,
        QTE_RESULT,
        json!({
            "flow": f.to_bits() as f64,
            "node": node.title,
            "choice": choice,
            "grade": grade,
        }),
    )
}

/// Counts this tick's inputs and gestures toward the open window's QTEs.
#[allow(clippy::too_many_arguments)]
fn qte_tick(
    ctx: &mut SystemContext<'_>,
    eng: &Engine,
    st: &ModuleState,
    f: Entity,
    flow: &mut record::Flow,
    node: &Node,
    view: &crate::host::wire::DialogueView,
    gestures: &[FlowGestureDesc],
    t: f32,
    dt: f32,
) -> og_core::Result<()> {
    let actions = eng.state.borrow().actions.clone();
    let action_state = |name: &str| {
        actions
            .as_ref()
            .and_then(|a| a.lock().unwrap_or_else(|e| e.into_inner()).state(name))
    };
    for c in view.choices.iter().filter(|c| c.enabled && !outcome(c)) {
        let Some(meta) = node.choice_meta(c) else {
            continue;
        };
        let (Some(slot), Some(q)) = (meta.slot, meta.qte.as_ref()) else {
            continue;
        };
        if (flow.qte_state[slot] & QTE_RESOLVED) != 0 {
            continue;
        }
        let need = qte_need(q, dt);
        match &q.kind {
            QteKind::Press => {
                let Some(s) = action_state(&q.action) else {
                    warn_missing_action(eng, st, f, node, c.index, q, &q.action);
                    continue;
                };
                if s.pressed {
                    flow.qte[slot] = 1;
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                    flow.pending = c.index as i32;
                    emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                }
            }
            QteKind::Hold(_) => {
                let Some(s) = action_state(&q.action) else {
                    warn_missing_action(eng, st, f, node, c.index, q, &q.action);
                    continue;
                };
                if s.pressed {
                    flow.qte_state[slot] |= QTE_STARTED;
                }
                if (flow.qte_state[slot] & QTE_STARTED) != 0 && s.down {
                    flow.qte[slot] = (flow.qte[slot] + 1).min(need);
                } else if flow.qte[slot] > 0 {
                    // Decay smoothly on early release and allow re-press.
                    flow.qte[slot] = flow.qte[slot].saturating_sub(2);
                    if flow.qte[slot] == 0 {
                        flow.qte_state[slot] &= !QTE_STARTED;
                    }
                }
                if flow.qte[slot] >= need {
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                    flow.pending = c.index as i32;
                    emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                }
            }
            QteKind::Mash(_) => {
                let Some(s) = action_state(&q.action) else {
                    warn_missing_action(eng, st, f, node, c.index, q, &q.action);
                    continue;
                };
                if s.pressed {
                    flow.qte[slot] = (flow.qte[slot] + 1).min(need);
                }
                if flow.qte[slot] >= need {
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                    flow.pending = c.index as i32;
                    emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                }
            }
            QteKind::Timing(zone_start, zone_end) => {
                let Some(s) = action_state(&q.action) else {
                    warn_missing_action(eng, st, f, node, c.index, q, &q.action);
                    continue;
                };
                if s.pressed {
                    let span = (node.lock - node.open).max(1e-4);
                    let pos = ((t - node.open) / span).clamp(0.0, 1.0);
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    if pos >= *zone_start && pos <= *zone_end {
                        let mid = 0.5 * (zone_start + zone_end);
                        let half_w = 0.5 * (zone_end - zone_start).max(1e-4);
                        let (grade_bits, grade_str) = if (pos - mid).abs() <= half_w * 0.4 {
                            (GRADE_PERFECT, "perfect")
                        } else {
                            (GRADE_GOOD, "good")
                        };
                        flow.qte[slot] = 1;
                        set_grade(&mut flow.qte_state[slot], grade_bits);
                        flow.pending = c.index as i32;
                        emit_qte_result(ctx, eng, f, node, c.index, grade_str)?;
                    } else {
                        set_grade(&mut flow.qte_state[slot], GRADE_MISS);
                        emit_qte_result(ctx, eng, f, node, c.index, "miss")?;
                    }
                }
            }
            QteKind::Sequence => {
                let step = flow.qte[slot] as usize;
                let Some(act) = q.sequence.get(step) else {
                    continue;
                };
                let Some(s) = action_state(act) else {
                    warn_missing_action(eng, st, f, node, c.index, q, act);
                    continue;
                };
                if s.pressed {
                    flow.qte[slot] += 1;
                    if flow.qte[slot] >= need {
                        flow.qte_state[slot] |= QTE_RESOLVED;
                        set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                        flow.pending = c.index as i32;
                        emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                    }
                }
            }
            QteKind::Swipe => {
                let want = q.direction.as_str();
                if gestures
                    .iter()
                    .any(|g| g.kind == "swipe" && g.direction.eq_ignore_ascii_case(want))
                {
                    flow.qte[slot] = 1;
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                    flow.pending = c.index as i32;
                    emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                }
            }
            QteKind::Drag => {
                let want = q.direction.as_str();
                for g in gestures
                    .iter()
                    .filter(|g| g.kind == "drag" && g.direction.eq_ignore_ascii_case(want))
                {
                    let p = (g.progress.clamp(0.0, 1.0) * 1000.0).round() as u32;
                    flow.qte[slot] = flow.qte[slot].max(p).min(1000);
                }
                if flow.qte[slot] >= need {
                    flow.qte_state[slot] |= QTE_RESOLVED;
                    set_grade(&mut flow.qte_state[slot], GRADE_PERFECT);
                    flow.pending = c.index as i32;
                    emit_qte_result(ctx, eng, f, node, c.index, "perfect")?;
                }
            }
        }
    }
    Ok(())
}

/// Emits `"miss"` for any unresolved QTE when its choice window locks.
fn qte_timeout(
    ctx: &mut SystemContext<'_>,
    eng: &Engine,
    f: Entity,
    flow: &mut record::Flow,
    node: &Node,
    view: &crate::host::wire::DialogueView,
) -> og_core::Result<()> {
    for c in view.choices.iter().filter(|c| c.enabled && !outcome(c)) {
        let Some(meta) = node.choice_meta(c) else {
            continue;
        };
        let (Some(slot), Some(_)) = (meta.slot, meta.qte.as_ref()) else {
            continue;
        };
        if (flow.qte_state[slot] & QTE_RESOLVED) == 0 {
            flow.qte_state[slot] |= QTE_RESOLVED;
            set_grade(&mut flow.qte_state[slot], GRADE_MISS);
            emit_qte_result(ctx, eng, f, node, c.index, "miss")?;
        }
    }
    Ok(())
}

fn warn_missing_action(
    eng: &Engine,
    st: &ModuleState,
    f: Entity,
    node: &Node,
    choice: u32,
    q: &Qte,
    action: &str,
) {
    warn(
        eng,
        st,
        format!(
            "flow {}: node '{}': choice {choice} is #qte:{}:{action}, an action og.input.defineAction has not defined",
            f.to_bits(),
            node.title,
            q.kind.name(),
        ),
    );
}

/// The hotspots and quick-time events a node offers now: inspect hotspots
/// throughout the segment, choice hotspots while the window is open, and QTEs
/// while the window is open or locked.
fn offers(
    node: &Node,
    flow: &record::Flow,
    choices: &[ChoiceView],
    inspected: Option<&BTreeSet<String>>,
    t: f32,
    dt: f32,
) -> (Vec<FlowHotspot>, Vec<QteView>) {
    let mut hotspots = Vec::new();
    let mut qtes = Vec::new();
    let here = og_timeline::eval::hotspots_at(&node.hotspots, t);
    // Non-cutting inspect hotspots are active whenever their timeline span covers `t`.
    for h in here.iter().filter(|h| h.kind == HotspotKind::Inspect) {
        let [x, y, width, height] = h.rect;
        let mark = format!("{}:{}", node.title, h.id);
        hotspots.push(FlowHotspot {
            choice: None,
            id: h.id.to_string(),
            kind: "inspect".to_string(),
            shape: h.shape.name().to_string(),
            x,
            y,
            width,
            height,
            label: h.label.to_string(),
            inspected: inspected.is_some_and(|s| s.contains(&mark)),
        });
    }
    for c in choices.iter().filter(|c| c.enabled) {
        if flow.window == 1
            && let Some(id) = c.tags.get("hotspot").and_then(Value::as_str)
            && let Some(h) = here
                .iter()
                .find(|h| h.kind == HotspotKind::Choice && h.id == id)
        {
            let [x, y, width, height] = h.rect;
            hotspots.push(FlowHotspot {
                choice: Some(c.index),
                id: id.to_string(),
                kind: "choice".to_string(),
                shape: h.shape.name().to_string(),
                x,
                y,
                width,
                height,
                label: if h.label.is_empty() {
                    c.text.clone()
                } else {
                    h.label.to_string()
                },
                inspected: false,
            });
        }
        if flow.window > 0
            && let Some(meta) = node.choice_meta(c)
            && let (Some(slot), Some(q)) = (meta.slot, meta.qte.as_ref())
        {
            let need = qte_need(q, dt);
            let have = flow.qte.get(slot).copied().unwrap_or(0);
            let st_bits = flow.qte_state.get(slot).copied().unwrap_or(0);
            let grade = grade_name(st_bits).to_string();
            let (action, direction, progress, zone_start, zone_end, done) = match q.kind {
                QteKind::Timing(zone_start, zone_end) => {
                    let span = (node.lock - node.open).max(1e-4);
                    let pos = ((t - node.open) / span).clamp(0.0, 1.0);
                    let passed = grade == "perfect" || grade == "good";
                    (
                        q.action.clone(),
                        String::new(),
                        pos,
                        zone_start,
                        zone_end,
                        passed,
                    )
                }
                QteKind::Sequence => {
                    let cur = q
                        .sequence
                        .get(have as usize)
                        .or_else(|| q.sequence.last())
                        .cloned()
                        .unwrap_or_default();
                    (
                        cur,
                        String::new(),
                        (have as f32 / need as f32).min(1.0),
                        0.0,
                        0.0,
                        have >= need,
                    )
                }
                QteKind::Swipe | QteKind::Drag => (
                    String::new(),
                    q.direction.clone(),
                    (have as f32 / need as f32).min(1.0),
                    0.0,
                    0.0,
                    have >= need,
                ),
                QteKind::Press | QteKind::Hold(_) | QteKind::Mash(_) => (
                    q.action.clone(),
                    String::new(),
                    (have as f32 / need as f32).min(1.0),
                    0.0,
                    0.0,
                    have >= need,
                ),
            };
            qtes.push(QteView {
                choice: c.index,
                kind: q.kind.name().into(),
                action,
                direction,
                progress,
                zone_start,
                zone_end,
                grade,
                done,
            });
        }
    }
    (hotspots, qtes)
}

fn flow_json(f: &record::Flow) -> Value {
    json!({
        "conversation": f.conversation,
        "decks": f.decks,
        "current": f.current as f64,
        "node": f.node,
        "pending": f.pending,
        "window": f.window,
        "paused": f.paused,
        "chapter": f.chapter,
        "ended": f.ended,
        "held": f.held,
        "previous": f.previous as f64,
        "dissolve": f.dissolve,
        "dissolveLeft": f.dissolve_left,
        "qte": f.qte,
        "qteState": f.qte_state,
        "speed": f.speed,
        "looping": f.looping,
    })
}

/// A flow node shows no Yarn lines (its captions are the segment's): a
/// line it reaches is warned of and passed.
fn skip_lines(eng: &Engine, st: &ModuleState, f: Entity, conv: u32) {
    for _ in 0..64 {
        let Some(v) = og_story_view(eng.id, conv) else {
            return;
        };
        if v.held || v.ended || !v.choices.is_empty() {
            return;
        }
        warn(
            eng,
            st,
            format!(
                "flow {}: node '{}' says '{}': a flow node shows no Yarn lines; put what is said in its segment's caption track",
                f.to_bits(),
                v.node,
                v.key
            ),
        );
        if og_story_advance(eng.id, conv).is_err() {
            return;
        }
    }
}

// ── the command channel ──────────────────────────────────────────────────────

/// `flow:choose:<i>`, `flow:play`, `flow:pause`, `flow:skip`, `flow:speed:<x>`,
/// `flow:seek:<t>`, `flow:inspect:<id>`, `flow:jump:<node>`.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "flow",
        og_probe::CommandDesc {
            usage: "flow:choose:<index> | flow:play | flow:pause | flow:skip | flow:advance[:<outcome>] | flow:speed:<mult> | flow:seek:<seconds> | flow:inspect:<id> | flow:jump:<node>".into(),
            doc: "control every playing flow from the command channel; takes effect at this tick's og.flow".into(),
        },
        move |_world, args| command(engine_id, args),
    );
    let _ = probe.register_dump(
        "dump_flow",
        og_probe::CommandDesc {
            usage: "dump_flow[:path]".into(),
            doc: "og.flow: every flow (its dialogue, node, speed, paused, held, looping, ended, history count, music) and its decks".into(),
        },
        0,
        move |world, _args| dump(engine_id, world),
    );
}

/// `dump_flow`: every flow's decks, each with the node it holds and how
/// ready its segment is to show (pre-roll, scenes, modules).
fn dump(engine_id: u32, world: &World) -> Result<Value, String> {
    let eng = engine(engine_id)?;
    let st = state(engine_id)?;
    let layout = |name: &str| -> Result<(SchemaId, Layout), String> {
        let l = eng
            .state
            .borrow()
            .layouts
            .get(name)
            .cloned()
            .ok_or_else(|| format!("{name} has no layout"))?;
        Ok((world.id_by_name(name).map_err(|e| e.to_string())?, l))
    };
    let (Ok(fr), Ok(dr), Ok(tr)) = (layout(FLOW), layout(DECK), layout(TIMELINE)) else {
        return Ok(json!({ "flows": [] }));
    };
    let mut flows = Vec::new();
    let mut decks = Vec::new();
    for e in world.entities() {
        if let Ok(b) = world.get_bytes(e, fr.0) {
            flows.push((e, fr.1.read::<record::Flow>(&b)?));
        }
        if let Ok(b) = world.get_bytes(e, dr.0) {
            decks.push((e, dr.1.read::<record::FlowDeck>(&b)?));
        }
    }
    flows.sort_by_key(|(e, _)| e.to_bits());
    decks.sort_by_key(|(e, _)| e.to_bits());
    let (declared, music_map, history_counts) = {
        let rt = lock(&st);
        let mus: BTreeMap<u64, String> = rt
            .saved
            .music
            .iter()
            .map(|(&k, m)| (k, m.sound.clone()))
            .collect();
        let hist: BTreeMap<u64, usize> = rt
            .saved
            .history
            .iter()
            .map(|(&k, h)| (k, h.lines.len()))
            .collect();
        (rt.saved.declared, mus, hist)
    };
    let mut out = Vec::new();
    for (f, flow) in &flows {
        let key = f.to_bits();
        let dialogue = crate::bind::story::dialogue_of(engine_id, flow.conversation);
        let prep = dialogue
            .as_ref()
            .and_then(|d| lock(&st).flows.get(d).cloned());
        let title = |id: u32| {
            prep.as_ref()
                .and_then(|p| p.nodes.get(&id))
                .map(|n| n.title.clone())
                .unwrap_or_default()
        };
        let mut ds = Vec::new();
        for (d, deck) in decks.iter().filter(|(_, d)| d.flow == key) {
            let t: record::Timeline = match world.get_bytes(*d, tr.0) {
                Ok(b) => tr.1.read(&b)?,
                Err(_) => record::Timeline::default(),
            };
            let node = prep.as_ref().and_then(|p| p.nodes.get(&deck.node));
            let role = if d.to_bits() == flow.current {
                "current"
            } else if d.to_bits() == flow.previous {
                "previous"
            } else if deck.node != 0 {
                "warm"
            } else {
                "free"
            };
            let scenes: Vec<Value> = node
                .map(|_| crate::bind::timeline::scenes_of(engine_id, t.asset))
                .unwrap_or_default()
                .into_iter()
                .map(|id| {
                    let (ready, why) = match crate::world::scenes::prepared(&eng, &id) {
                        crate::world::assets::Prepared::Yes => ("yes", String::new()),
                        crate::world::assets::Prepared::Busy => ("busy", String::new()),
                        crate::world::assets::Prepared::No(w) => ("no", w),
                    };
                    json!({ "scene": id, "prepared": ready, "why": why })
                })
                .collect();
            ds.push(json!({
                "deck": d.to_bits() as f64,
                "role": role,
                "node": node.map(|n| n.title.clone()).unwrap_or_default(),
                "segment": node.map(|n| n.segment.clone()).unwrap_or_default(),
                "time": t.time,
                "playing": t.playing,
                "preroll": t.preroll,
                "scenes": scenes,
                "modules": node
                    .map(|n| crate::world::needs::of(&eng, &n.segment, None))
                    .unwrap_or_default(),
            }));
        }
        out.push(json!({
            "flow": key as f64,
            "dialogue": dialogue.unwrap_or_default(),
            "declared": declared == Some(key),
            "prepared": prep.is_some(),
            "node": title(flow.node),
            "speed": flow.speed,
            "paused": flow.paused,
            "held": flow.held,
            "looping": flow.looping,
            "ended": flow.ended,
            "music": music_map.get(&key).cloned().unwrap_or_default(),
            "historyCount": history_counts.get(&key).copied().unwrap_or(0),
            "decks": ds,
        }));
    }
    Ok(json!({ "flows": out }))
}

fn command(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write flow:choose:<index>, flow:play, flow:pause, flow:skip, flow:advance[:<outcome>], flow:speed:<mult>, flow:seek:<seconds>, flow:inspect:<id> or flow:jump:<node>";
    let st = state(engine_id)?;
    let flows: Vec<u64> = lock(&st).views.keys().copied().collect();
    if flows.is_empty() {
        return Err("no flow is playing (og.flow.start)".into());
    }
    let op = match args.first().map(String::as_str) {
        Some("choose") => Op::Choose(
            args.get(1)
                .and_then(|a| a.trim().parse().ok())
                .ok_or("write flow:choose:<index>, such as flow:choose:0")?,
        ),
        Some("play") => Op::Play,
        Some("pause") => Op::Pause,
        Some("skip") => Op::Skip,
        Some("advance") => Op::Advance(args.get(1).filter(|o| !o.is_empty()).cloned()),
        Some("speed") => Op::SetSpeed(
            args.get(1)
                .and_then(|a| a.trim().parse().ok())
                .ok_or("write flow:speed:<multiplier>, such as flow:speed:2")?,
        ),
        Some("seek") => Op::Seek(
            args.get(1)
                .and_then(|a| a.trim().parse().ok())
                .ok_or("write flow:seek:<seconds>, such as flow:seek:1.5")?,
        ),
        Some("inspect") => Op::Inspect(
            args.get(1)
                .filter(|s| !s.is_empty())
                .cloned()
                .ok_or("write flow:inspect:<hotspot>")?,
        ),
        Some("jump") => Op::Jump(FlowJumpDesc {
            node: args
                .get(1)
                .filter(|s| !s.is_empty())
                .cloned()
                .ok_or("write flow:jump:<node>")?,
            vars: Value::Null,
        }),
        _ => return Err(USAGE.into()),
    };
    for f in flows {
        queue(engine_id, f as f64, op.clone())?;
    }
    Ok(())
}
