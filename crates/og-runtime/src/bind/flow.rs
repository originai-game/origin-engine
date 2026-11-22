//! A game's flow (IDL `flow`, guides/flow.md): a Yarn dialogue of
//! segments. `og.flow.load` prepares a loaded dialogue: each node with an
//! `og-flow` header names its segment, a timeline of footage (its own video
//! track, which plays on a deck), a live scene (a `scene` track), or both. `og.flow.start` spawns the flow (og.Flow) and its
//! decks (og.FlowDeck, each with og.Timeline, og.VideoPlayer and
//! og.AudioSource) and starts its conversation holding at each flow node
//! (og.story's StartDesc.hold).
//!
//! The og.flow system runs before og.timeline. Each tick it reads the
//! current deck's og.Timeline time against the segment's markers: the
//! window opens at `flow.open` and locks at `flow.lock`; at the cut
//! (`flow.cut`, else the end) the pending choice, else the default, is
//! taken, the conversation runs to its next hold, and that node's segment
//! starts on a deck. The other decks hold the segments the current node
//! may lead to, paused at their first frame, so their decoders have it.
//! A playable segment stops at its `flow.hold` marker (og.Flow.held) until
//! the game calls og.flow.advance, which takes one of the node's hidden
//! `#outcome:<name>` choices; a choice the player takes while it holds cuts
//! at once. Each cut snaps the camera (og.render.snapCamera).
//!
//! Every value is world state: og.Flow and og.FlowDeck, the decks'
//! timelines and players, and the conversation (og.story's block). Only
//! the prepared dialogues (og.flow.load, in every process) and the ops a
//! call queued for this tick are the engine's; [`Rt::views`] caches what
//! the last tick showed for og.flow.view, which systems may call.
//!
//! og.Flow, og.FlowDeck and the flow events are lazy
//! (`crate::engine::lazy`): a world that never loads a flow keeps its type
//! table and hash, and has no og.flow system.

use crate::bind::story::{
    og_story_advance, og_story_choose, og_story_start, og_story_stop, og_story_view,
};
use crate::engine::{Engine, engine};
use crate::host::codec::Layout;
use crate::host::wire::{
    ChoiceView, FlowChart, FlowChartLink, FlowChartNode, FlowHotspot, FlowInfo, FlowStartDesc,
    FlowView, QteView, WindowView, record,
};
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use og_timeline::HotspotKey;
use og_timeline::segment::{QTE_CHOICES, Qte, QteKind};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const FLOW: &str = "og.Flow";
pub(crate) const DECK: &str = "og.FlowDeck";
const SEGMENT: &str = "og.FlowSegment";
const CHOICE: &str = "og.FlowChoice";
const CHAPTER: &str = "og.FlowChapter";
const ENDED: &str = "og.FlowEnded";
/// og.flow's records, registered together.
pub(crate) const RECORDS: [&str; 6] = [FLOW, DECK, SEGMENT, CHOICE, CHAPTER, ENDED];
pub(crate) const SYSTEM: &str = "og.flow";
const TIMELINE: &str = "og.Timeline";
const PLAYER: &str = crate::bind::video::PLAYER;
const AUDIO: &str = "og.AudioSource";
/// The node header that names a segment (and holds the conversation).
const HEADER: &str = og_story::graph::FLOW_HEADER;
const MAX_DECKS: u32 = 3;

/// A flow node of a prepared dialogue.
#[derive(Clone, Debug)]
struct Node {
    title: String,
    segment: String,
    timeline: u64,
    first: Option<(u64, f32)>,
    open: f32,
    lock: f32,
    cut: f32,
    chapter: String,
    ending: String,
    /// Seconds its segment dissolves in over the one before (0: a cut).
    dissolve: f32,
    /// Its segment's hotspot keys.
    hotspots: Vec<HotspotKey>,
    /// Per choice (by index): its quick-time event (`#qte`).
    qte: Vec<Option<Qte>>,
    /// `flow.hold`: where its segment stops for og.flow.advance.
    hold: Option<f32>,
    /// Its footage plays under the scene (`layer under`): og.timeline puts
    /// it on the camera, and the view's picture is 0.
    under: bool,
    /// Its `#outcome:<name>` choices: (ChoiceView.index, name).
    outcomes: Vec<(u32, String)>,
    /// Per choice (by index): the flow nodes it may lead to, through logic
    /// nodes.
    choices: Vec<Vec<u32>>,
    /// The flow nodes its own steps may lead to (a node without choices).
    next: Vec<u32>,
}

/// A dialogue og.flow.load prepared: its flow nodes by id.
#[derive(Debug, Default)]
struct Prepared {
    nodes: BTreeMap<u32, Node>,
    /// Its flow nodes in file order.
    order: Vec<u32>,
    /// The flow nodes the dialogue's start leads to first.
    roots: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
enum Op {
    Choose(u32),
    Play,
    Pause,
    Skip,
    Stop,
    /// og.flow.advance: cut now, by this outcome or the window's choice.
    Advance(Option<String>),
}

/// What a flow showed at the end of the last tick (og.flow.view).
#[derive(Clone, Debug, Default)]
struct Shown {
    conversation: u32,
    node: String,
    segment: String,
    texture: u64,
    time: f32,
    open: f32,
    lock: f32,
    cut: f32,
    dt: f32,
    window: u32,
    pending: i32,
    paused: bool,
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
    /// Prepared dialogues by name.
    flows: BTreeMap<String, Arc<Prepared>>,
    ops: Vec<(u64, Op)>,
    views: BTreeMap<u64, Shown>,
    warned: BTreeSet<String>,
    /// The flow project.ogt's `flow` started (og.flow.current), which every
    /// process starts the same way before GameDesc.start.
    declared: Option<u64>,
}

pub(crate) type ModuleState = Arc<Mutex<Rt>>;

fn lock(m: &Mutex<Rt>) -> MutexGuard<'_, Rt> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn state(engine_id: u32) -> Result<ModuleState, String> {
    Ok(engine(engine_id)?.modules.flow.clone())
}

fn bad(m: impl Into<String>) -> og_core::Error {
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

// ── og.flow.load ─────────────────────────────────────────────────────────────

/// Dialogue `name` as og.flow plays it: every flow node's segment loaded
/// and checked, and where each choice may lead.
fn prepare(eng: &Engine, world: &mut World, name: &str) -> Result<(Prepared, FlowInfo), String> {
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
        let timeline = crate::bind::timeline::load_granted(eng, segment, Some(&mut *world))
            .map_err(|e| at(format!("segment {segment}: {e}")))?;
        let (s, first) = crate::bind::timeline::segment(eng.id, timeline)
            .map_err(|e| at(format!("segment {segment}: {e}")))?;
        let defaults = n
            .choices
            .iter()
            .filter(|c| c.tags.contains_key("default"))
            .count();
        if defaults > 1 {
            return Err(at(format!("{defaults} choices are #default; one at most")));
        }
        let dissolve = match n.headers.get("og-flow-dissolve") {
            Some(h) => og_timeline::segment::dissolve(h).map_err(at)?,
            None => 0.0,
        };
        let mut qte = Vec::new();
        for c in &n.choices {
            let q = match c.tags.get("qte") {
                Some(t) => {
                    let q = Qte::parse(t).map_err(|e| at(format!("choice {}: {e}", c.index)))?;
                    if c.index as usize >= QTE_CHOICES {
                        return Err(at(format!(
                            "choice {} is a #qte; only a node's first {QTE_CHOICES} choices may be",
                            c.index
                        )));
                    }
                    Some(q)
                }
                None => None,
            };
            qte.push(q);
        }
        let outcomes = og_timeline::segment::outcomes(
            s.hold.is_some(),
            n.choices.iter().map(|c| (c.index, &c.tags)),
        )
        .map_err(at)?
        .into_iter()
        .zip(&n.choices)
        .filter_map(|(o, c)| Some((c.index, o?)))
        .collect();
        p.nodes.insert(
            node_id(&n.title),
            Node {
                title: n.title.clone(),
                segment: segment.clone(),
                timeline,
                first,
                open: s.open,
                lock: s.lock,
                cut: s.cut,
                chapter: header("og-flow-chapter"),
                ending: header("og-flow-ending"),
                dissolve,
                hotspots: s.hotspots,
                qte,
                hold: s.hold,
                under: s.under,
                outcomes,
                choices: n.choices.iter().map(|c| reach(&c.to)).collect(),
                next: reach(&n.to),
            },
        );
        p.order.push(node_id(&n.title));
        info.segments.push(n.title.clone());
    }
    let start = crate::bind::story::start_of(eng.id, name)
        .filter(|s| !s.is_empty())
        .or_else(|| graph.first().map(|n| n.title.clone()))
        .unwrap_or_default();
    p.roots = reach(&[start]);
    if p.nodes.is_empty() {
        return Err(format!(
            "{name} has no flow node: give a node an `og-flow: segments/<segment>.ogt` header"
        ));
    }
    Ok((p, info))
}

pub fn og_flow_load(world: u32, dialogue: String) -> Result<FlowInfo, String> {
    let eng = engine(world)?;
    let w = eng.world().map_err(|e| {
        format!(
            "og.flow.load registers og.Flow, which a system cannot do: load flows in setup ({e})"
        )
    })?;
    crate::engine::lazy::ensure(&eng, w, FLOW)?;
    let (p, info) = prepare(&eng, w, &dialogue)?;
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
    lock(&st).declared = flow.as_f64().map(|f| f as u64);
    Ok(())
}

pub fn og_flow_current(world: u32) -> Option<u64> {
    let st = state(world).ok()?;
    let rt = lock(&st);
    rt.declared.filter(|f| rt.views.contains_key(f))
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
        }),
    )?;
    lock(&eng.modules.flow).views.insert(
        flow.to_bits(),
        Shown {
            conversation,
            pending: -1,
            paused: d.paused,
            ..Shown::default()
        },
    );
    Ok(json!(flow.to_bits() as f64))
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

pub fn og_flow_view(world: u32, flow: f64) -> Option<FlowView> {
    let st = state(world).ok()?;
    let s = lock(&st).views.get(&(flow as u64)).cloned()?;
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
        held: s.held,
    });
    Some(FlowView {
        flow: flow as u64,
        node: s.node,
        segment: s.segment,
        texture: s.texture,
        time: s.time,
        cut: s.cut,
        choices,
        window,
        hotspots: s.hotspots,
        qtes: s.qtes,
        previous: s.previous,
        mix: s.mix,
        paused: s.paused,
        held: s.held,
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
    if let Op::Choose(i) = op {
        if s.window != 1 {
            return Err(if s.window == 2 {
                "the window has locked: the cut takes what was chosen".into()
            } else {
                "no choice window is open (flow.open to flow.lock)".into()
            });
        }
        let v = og_story_view(world, s.conversation).ok_or("the flow's conversation is gone")?;
        let c = v
            .choices
            .iter()
            .find(|c| c.index == i)
            .ok_or_else(|| format!("choice {i} is not offered (hidden, or no such choice)"))?;
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
    if let Op::Advance(o) = &op {
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
    if op == Op::Skip && s.ended {
        return Err("the flow has ended: nothing is left to skip".into());
    }
    lock(&st).ops.push((key, op));
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

// ── og.flow.chart ────────────────────────────────────────────────────────────

/// Graph units between columns and between rows, and around the chart.
const CHART_COLUMN: f32 = 200.0;
const CHART_ROW: f32 = 120.0;
const CHART_MARGIN: f32 = 60.0;

pub fn og_flow_chart(world: u32, dialogue: String) -> Result<FlowChart, String> {
    let st = state(world)?;
    let p = lock(&st).flows.get(&dialogue).cloned().ok_or_else(|| {
        format!("the dialogue '{dialogue}' is not prepared: og.flow.load it in setup")
    })?;
    Ok(chart(&p))
}

/// A link from (x, y) to (x2, y2) leaving and arriving level: the cubic
/// with both handles at the middle column, in 16 segments.
fn level_curve(x: f32, y: f32, x2: f32, y2: f32) -> Vec<f32> {
    let mid = (x + x2) * 0.5;
    (0..=16)
        .flat_map(|k| {
            let u = k as f32 / 16.0;
            let w = 1.0 - u;
            let (a, b, c, d) = (w * w * w, 3.0 * w * w * u, 3.0 * w * u * u, u * u * u);
            [a * x + (b + c) * mid + d * x2, (a + b) * y + (c + d) * y2]
        })
        .collect()
}

/// Columns by the fewest flow nodes from the start (breadth first, so a
/// loop back draws as a link to an earlier column); rows by the order each
/// column was reached in. Nodes the start never reaches follow the deepest.
fn chart(p: &Prepared) -> FlowChart {
    let leads = |n: &Node| -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for to in n.choices.iter().flatten().chain(&n.next) {
            if !out.contains(to) {
                out.push(*to);
            }
        }
        out
    };
    let mut depth: BTreeMap<u32, u32> = BTreeMap::new();
    let mut columns: Vec<Vec<u32>> = Vec::new();
    let mut frontier: Vec<u32> = Vec::new();
    for r in &p.roots {
        if p.nodes.contains_key(r) && !depth.contains_key(r) {
            depth.insert(*r, 0);
            frontier.push(*r);
        }
    }
    while !frontier.is_empty() {
        let d = columns.len() as u32;
        let mut next = Vec::new();
        for id in &frontier {
            for to in leads(&p.nodes[id]) {
                if p.nodes.contains_key(&to) && !depth.contains_key(&to) {
                    depth.insert(to, d + 1);
                    next.push(to);
                }
            }
        }
        columns.push(std::mem::replace(&mut frontier, next));
    }
    let unreached: Vec<u32> = p
        .order
        .iter()
        .copied()
        .filter(|id| !depth.contains_key(id))
        .collect();
    if !unreached.is_empty() {
        for id in &unreached {
            depth.insert(*id, columns.len() as u32);
        }
        columns.push(unreached);
    }
    let mut at: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
    for (c, col) in columns.iter().enumerate() {
        for (r, id) in col.iter().enumerate() {
            at.insert(
                *id,
                (
                    CHART_MARGIN + c as f32 * CHART_COLUMN,
                    CHART_MARGIN + r as f32 * CHART_ROW,
                ),
            );
        }
    }
    let rows = columns.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = FlowChart {
        width: 2.0 * CHART_MARGIN + columns.len().saturating_sub(1) as f32 * CHART_COLUMN,
        height: 2.0 * CHART_MARGIN + rows.saturating_sub(1) as f32 * CHART_ROW,
        ..FlowChart::default()
    };
    for id in &p.order {
        let n = &p.nodes[id];
        let (x, y) = at[id];
        out.nodes.push(FlowChartNode {
            node: n.title.clone(),
            segment: n.segment.clone(),
            chapter: n.chapter.clone(),
            ending: n.ending.clone(),
            depth: depth[id],
            x,
            y,
        });
        let mut link = |to: u32, choice: i32| {
            let Some(t) = p.nodes.get(&to) else { return };
            let (x2, y2) = at[&to];
            out.links.push(FlowChartLink {
                from: n.title.clone(),
                to: t.title.clone(),
                choice,
                points: level_curve(x, y, x2, y2),
            });
        };
        for (i, tos) in n.choices.iter().enumerate() {
            for to in tos {
                link(*to, i as i32);
            }
        }
        if n.choices.is_empty() {
            for to in &n.next {
                link(*to, -1);
            }
        }
    }
    out
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
    writes.extend([TIMELINE.to_string(), PLAYER.to_string()]);
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
    };
    let ops = std::mem::take(&mut lock(st).ops);
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
        if let Some(shown) = tick(ctx, &eng, st, &r, f, mine, &mine_ops)? {
            views.insert(key, shown);
        }
    }
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
fn load_deck(
    ctx: &mut SystemContext<'_>,
    r: &Recs,
    deck: Entity,
    id: u32,
    node: &Node,
    playing: bool,
) -> og_core::Result<()> {
    patch(
        ctx,
        &r.timeline,
        deck,
        json!({ "asset": node.timeline as f64, "time": 0.0, "playing": playing, "loop": false, "speed": 1.0, "preroll": !playing }),
    )?;
    patch(ctx, &r.deck, deck, json!({ "node": id }))
}

/// Lets go of a deck: no segment, its player stopped.
fn free_deck(ctx: &mut SystemContext<'_>, r: &Recs, deck: Entity) -> og_core::Result<()> {
    patch(
        ctx,
        &r.timeline,
        deck,
        json!({ "asset": 0.0, "time": 0.0, "playing": false }),
    )?;
    patch(ctx, &r.player, deck, json!({ "playing": false }))?;
    patch(ctx, &r.deck, deck, json!({ "node": 0 }))
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
) -> og_core::Result<Option<Shown>> {
    let fb = ctx.get_bytes(f, r.flow.0)?;
    let mut flow: record::Flow = r.flow.1.read(&fb).map_err(bad)?;
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
                "flow {}: its dialogue {} is not prepared (og.flow.load it in setup, in every process that restores a flow)",
                f.to_bits(),
                dialogue.as_deref().unwrap_or("(its conversation is gone)")
            ),
        );
        return Ok(None);
    };
    let dt = ctx.time().dt();
    let half = dt * 0.5;
    let current = |flow: &record::Flow| Entity::from_bits(flow.current);
    // og.timeline runs after this system and shows a playing timeline's
    // time plus dt, except on the tick it starts, resumes or is sought
    // (then its time as written): the markers are judged at the time this
    // tick shows. `advancing`: the current deck played last tick and still
    // does.
    let mut advancing = flow.node != 0 && deck_state(ctx, r, current(&flow))?.1;
    let shows =
        |ctx: &SystemContext<'_>, flow: &record::Flow, advancing: bool| -> og_core::Result<f32> {
            let t = deck_state(ctx, r, current(flow))?.0;
            Ok(if advancing { t + dt } else { t })
        };
    let mut advance: Option<Option<String>> = None;
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
                    let playing =
                        json!({ "playing": !flow.paused && !flow.held, "preroll": false });
                    patch(ctx, &r.timeline, current(&flow), playing.clone())?;
                    if let Some(p) = previous(&flow) {
                        patch(ctx, &r.timeline, p, playing)?;
                    }
                }
            }
            Op::Skip => {
                if let Some(node) = prep.nodes.get(&flow.node).filter(|_| !flow.ended) {
                    let t = shows(ctx, &flow, advancing)?;
                    let choosing =
                        og_story_view(eng.id, conv).is_some_and(|v| !seen(v.choices).is_empty());
                    let to = if choosing && t + half < node.open {
                        node.open
                    } else {
                        node.cut
                    };
                    patch(ctx, &r.timeline, current(&flow), json!({ "time": to }))?;
                    // A sought timeline shows the time written this tick.
                    advancing = false;
                }
            }
            Op::Stop => {
                og_story_stop(eng.id, conv);
                for (d, _) in &decks {
                    ctx.despawn(*d)?;
                }
                ctx.despawn(f)?;
                return Ok(None);
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
    }
    if flow.qte.len() != QTE_CHOICES {
        flow.qte.resize(QTE_CHOICES, 0);
    }

    // Begins node `title`'s segment on a deck (a warm one if it holds it),
    // then runs the node to its choices (or on to its next hold).
    let begin = |ctx: &mut SystemContext<'_>,
                 flow: &mut record::Flow,
                 decks: &mut Decks,
                 title: &str|
     -> og_core::Result<()> {
        let id = node_id(title);
        let Some(node) = prep.nodes.get(&id) else {
            warn(
                eng,
                st,
                format!(
                    "flow {}: node '{title}' holds without a segment",
                    f.to_bits()
                ),
            );
            return Ok(());
        };
        let cur = current(flow);
        let prev = previous(flow);
        let target = if flow.decks > 1 {
            decks
                .iter()
                .find(|(d, k)| *d != cur && k.node == id)
                .or_else(|| decks.iter().find(|(d, _)| *d != cur && Some(*d) != prev))
                .or_else(|| decks.iter().find(|(d, _)| *d != cur))
                .map_or(cur, |(d, _)| *d)
        } else {
            cur
        };
        // A dissolve still running ends at this cut.
        if let Some(p) = prev {
            if p != target {
                free_deck(ctx, r, p)?;
            }
            flow.previous = 0;
            flow.dissolve_left = 0;
        }
        if target != cur && decks.iter().any(|(d, _)| *d == cur) {
            let ticks = if node.dissolve > 0.0 && flow.node != 0 && dt > 0.0 {
                (node.dissolve / dt).ceil() as u32
            } else {
                0
            };
            if ticks > 0 {
                // It plays on under the new segment until the dissolve ends.
                patch(ctx, &r.deck, cur, json!({ "node": 0 }))?;
                flow.previous = cur.to_bits();
                flow.dissolve = ticks;
                flow.dissolve_left = ticks;
            } else {
                free_deck(ctx, r, cur)?;
            }
        }
        let warm = decks.iter().any(|(d, k)| *d == target && k.node == id);
        if warm {
            patch(
                ctx,
                &r.timeline,
                target,
                json!({ "playing": !flow.paused, "preroll": false }),
            )?;
        } else {
            load_deck(ctx, r, target, id, node, !flow.paused)?;
        }
        for (d, k) in decks.iter_mut() {
            if *d == target {
                k.node = id;
            } else if *d == cur && target != cur {
                k.node = 0;
            }
        }
        flow.current = target.to_bits();
        flow.node = id;
        flow.pending = -1;
        flow.window = 0;
        flow.held = false;
        // A cut: the camera the new segment places is drawn where it is,
        // never blended from the last one's.
        crate::host::api::og_render_snap_camera(eng.id);
        flow.qte = vec![0; QTE_CHOICES];
        send(
            ctx,
            eng,
            SEGMENT,
            json!({ "flow": f.to_bits() as f64, "node": node.title, "segment": node.segment }),
        )?;
        if !node.chapter.is_empty() {
            flow.chapter = id;
            send(
                ctx,
                eng,
                CHAPTER,
                json!({ "flow": f.to_bits() as f64, "node": node.title, "chapter": node.chapter }),
            )?;
        }
        if let Err(e) = og_story_advance(eng.id, conv) {
            warn(
                eng,
                st,
                format!("flow {}: node '{title}': {e}", f.to_bits()),
            );
        }
        skip_lines(eng, st, f, conv);
        Ok(())
    };

    if !flow.ended {
        let view = og_story_view(eng.id, conv);
        if flow.node == 0 {
            match &view {
                Some(v) if v.held => {
                    begin(ctx, &mut flow, &mut decks, &v.node)?;
                    advancing = false;
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
            flow.window = if !choosing || t + half < node.open {
                0
            } else if flow.held || t + half < node.lock {
                1
            } else {
                2
            };
            if flow.window == 1
                && !flow.paused
                && let Some(v) = &view
            {
                qte_tick(eng, st, f, &mut flow, node, v, dt);
            }
            let timed = !flow.paused && node.hold.is_none() && t + half >= node.cut;
            // A choice the player takes while it holds cuts at once.
            let chosen_held = flow.held && flow.pending >= 0;
            if timed || chosen_held || advance.is_some() {
                // The cut: the outcome og.flow.advance named, else the
                // pending choice, else the default; then on to the next hold.
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
                        send(
                            ctx,
                            eng,
                            CHOICE,
                            json!({
                                "flow": f.to_bits() as f64, "node": title, "index": i, "timedOut": taken.is_none() && advance.is_none(),
                            }),
                        )?;
                        if let Err(e) = og_story_choose(eng.id, conv, i) {
                            warn(
                                eng,
                                st,
                                format!("flow {}: node '{title}': {e}", f.to_bits()),
                            );
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
                                "flow {}: after '{title}' the conversation shows choices outside a flow node: give that node an `og-flow` segment",
                                f.to_bits()
                            ),
                        );
                        flow.ended = true;
                    }
                    _ => {
                        flow.ended = true;
                        flow.window = 0;
                        send(
                            ctx,
                            eng,
                            ENDED,
                            json!({ "flow": f.to_bits() as f64, "node": title, "ending": node.ending }),
                        )?;
                    }
                }
            }
        }
    }

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
                    for i in order {
                        want.extend(node.choices.get(i as usize).into_iter().flatten());
                    }
                }
            }
            _ => want.extend(&node.next),
        }
        let mut seen = BTreeSet::new();
        want.retain(|w| seen.insert(*w));
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
            load_deck(ctx, r, decks[i].0, w, n, false)?;
            decks[i].1.node = w;
        }
    }

    // A spare deck holding a segment the flow can no longer cut to (one
    // warmed for a choice not taken, or any once the flow ends) lets go:
    // its decoder and the scenes its segment placed are prepared for
    // nothing.
    let reach: BTreeSet<u32> = match prep.nodes.get(&flow.node) {
        Some(node) if !flow.ended => node
            .choices
            .iter()
            .flatten()
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
    let node = prep.nodes.get(&flow.node);
    let time = shows(ctx, &flow, advancing)?;
    // A segment whose footage plays under the scene shows no picture over
    // it, and one with no footage of its own (a live scene) none at all: its
    // deck's player still holds whatever that deck played last.
    let picture = |deck: Entity| -> og_core::Result<u64> {
        let held = r
            .deck
            .1
            .read::<record::FlowDeck>(&ctx.get_bytes(deck, r.deck.0)?)
            .map_err(bad)?;
        if prep
            .nodes
            .get(&held.node)
            .is_some_and(|n| n.under || n.first.is_none())
        {
            return Ok(0);
        }
        Ok(deck_state(ctx, r, deck)?.2)
    };
    let texture = picture(current(&flow))?;
    let (hotspots, qtes) = match node {
        Some(n) if flow.window > 0 && !flow.ended => {
            let choices = og_story_view(eng.id, conv)
                .map(|v| seen(v.choices))
                .unwrap_or_default();
            offers(n, &flow, &choices, time, dt)
        }
        _ => (Vec::new(), Vec::new()),
    };
    let previous_texture = match previous(&flow) {
        Some(p) => picture(p)?,
        None => 0,
    };
    let mix = if flow.dissolve_left > 0 && flow.dissolve > 0 {
        1.0 - flow.dissolve_left as f32 / flow.dissolve as f32
    } else {
        1.0
    };
    Ok(Some(Shown {
        conversation: conv,
        node: node.map(|n| n.title.clone()).unwrap_or_default(),
        segment: node.map(|n| n.segment.clone()).unwrap_or_default(),
        texture,
        time,
        open: node.map_or(0.0, |n| n.open),
        lock: node.map_or(0.0, |n| n.lock),
        cut: node.map_or(0.0, |n| n.cut),
        dt,
        window: flow.window,
        pending: flow.pending,
        paused: flow.paused,
        chapter: prep
            .nodes
            .get(&flow.chapter)
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
        ahead: match node {
            Some(n) if !flow.ended => ahead(&prep, n, time),
            _ => Vec::new(),
        },
    }))
}

/// Every segment the node in place may cut to (each choice's and its own
/// steps' flow nodes), by its first frame, needed at the cut. The decks
/// warm only as many as they have room for, from the window's open; this
/// asks the streaming queue for every target's first chunk as soon as the
/// segment plays, so a choice no deck warmed still starts from a chunk
/// that is here. Presentation: it only orders fetches.
fn ahead(prep: &Prepared, node: &Node, time: f32) -> Vec<Ahead> {
    let seconds = f64::from((node.cut - time).max(0.0));
    let mut seen = BTreeSet::new();
    node.choices
        .iter()
        .flatten()
        .chain(&node.next)
        .filter(|id| seen.insert(**id))
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

/// Ticks of input a quick-time event needs.
fn qte_need(q: &Qte, dt: f32) -> u32 {
    match q.kind {
        QteKind::Press => 1,
        QteKind::Hold(ms) if dt > 0.0 => ((ms / 1000.0) / dt).ceil().max(1.0) as u32,
        QteKind::Hold(_) => 1,
        QteKind::Mash(n) => n,
    }
}

/// Counts this tick's input toward the open window's quick-time events: a
/// press, a hold (let go: from 0 again) or presses. One that is reached
/// becomes the pending choice, replacing what was chosen before.
fn qte_tick(
    eng: &Engine,
    st: &ModuleState,
    f: Entity,
    flow: &mut record::Flow,
    node: &Node,
    view: &crate::host::wire::DialogueView,
    dt: f32,
) {
    let actions = eng.state.borrow().actions.clone();
    for c in view.choices.iter().filter(|c| c.enabled) {
        let i = c.index as usize;
        let Some(Some(q)) = node.qte.get(i) else {
            continue;
        };
        let need = qte_need(q, dt);
        if flow.qte[i] >= need {
            continue;
        }
        let state = actions
            .as_ref()
            .and_then(|a| a.lock().unwrap_or_else(|e| e.into_inner()).state(&q.action));
        let Some(s) = state else {
            warn(
                eng,
                st,
                format!(
                    "flow {}: node '{}': choice {i} is #qte:{}:{}, an action og.input.defineAction has not defined",
                    f.to_bits(),
                    node.title,
                    q.kind.name(),
                    q.action
                ),
            );
            continue;
        };
        flow.qte[i] = match q.kind {
            QteKind::Press => u32::from(s.pressed),
            QteKind::Hold(_) if s.down => flow.qte[i] + 1,
            QteKind::Hold(_) => 0,
            QteKind::Mash(_) => flow.qte[i] + u32::from(s.pressed),
        };
        if flow.qte[i] >= need {
            flow.pending = c.index as i32;
        }
    }
}

/// The hotspots and quick-time events a node's window offers now: those of
/// its enabled choices, the hotspots only while it is open.
fn offers(
    node: &Node,
    flow: &record::Flow,
    choices: &[ChoiceView],
    t: f32,
    dt: f32,
) -> (Vec<FlowHotspot>, Vec<QteView>) {
    let mut hotspots = Vec::new();
    let mut qtes = Vec::new();
    let here = og_timeline::eval::hotspots_at(&node.hotspots, t);
    for c in choices.iter().filter(|c| c.enabled) {
        if flow.window == 1
            && let Some(id) = c.tags.get("hotspot").and_then(Value::as_str)
            && let Some(h) = here.iter().find(|h| h.id == id)
        {
            let [x, y, width, height] = h.rect;
            hotspots.push(FlowHotspot {
                choice: c.index,
                id: id.to_string(),
                x,
                y,
                width,
                height,
            });
        }
        if let Some(Some(q)) = node.qte.get(c.index as usize) {
            let need = qte_need(q, dt);
            let have = flow.qte.get(c.index as usize).copied().unwrap_or(0);
            qtes.push(QteView {
                choice: c.index,
                kind: q.kind.name().into(),
                action: q.action.clone(),
                progress: (have as f32 / need as f32).min(1.0),
                done: have >= need,
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

/// `flow:choose:<i>`, `flow:play`, `flow:pause`: every flow this engine shows.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "flow",
        og_probe::CommandDesc {
            usage: "flow:choose:<index> | flow:play | flow:pause | flow:skip".into(),
            doc: "take a choice in every flow's open window (ChoiceView.index), play or pause every flow, or skip ahead in every flow's segment (og.flow.skip); it takes effect at this tick's og.flow".into(),
        },
        move |_world, args| command(engine_id, args),
    );
    let _ = probe.register_dump(
        "dump_flow",
        og_probe::CommandDesc {
            usage: "dump_flow[:path]".into(),
            doc: "og.flow: every flow (its dialogue, node, paused, held, ended) and its decks: the node each holds as current, previous, warm or free, its segment's time and pre-roll, whether each scene its scene track places is prepared (and why not), and the engine modules the segment needs".into(),
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
    let declared = lock(&st).declared;
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
            "paused": flow.paused,
            "held": flow.held,
            "ended": flow.ended,
            "decks": ds,
        }));
    }
    Ok(json!({ "flows": out }))
}

fn command(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str =
        "write flow:choose:<index>, flow:play, flow:pause, flow:skip or flow:advance[:<outcome>]";
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
        _ => return Err(USAGE.into()),
    };
    for f in flows {
        queue(engine_id, f as f64, op.clone())?;
    }
    Ok(())
}
