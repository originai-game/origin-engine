//! Interactive film (IDL `film`, guides/film.md): a Yarn dialogue paced by
//! its footage. `og.film.load` prepares a loaded dialogue: each node with
//! an `og-film` header names its segment, a timeline whose own video track
//! plays on a deck. `og.film.start` spawns the film (og.Film) and its
//! decks (og.FilmDeck, each with og.Timeline, og.VideoPlayer and
//! og.AudioSource) and starts its conversation holding at each film node
//! (og.story's StartDesc.hold).
//!
//! The og.film system runs before og.timeline. Each tick it reads the
//! current deck's og.Timeline time against the segment's markers: the
//! window opens at `film.open` and locks at `film.lock`; at the cut
//! (`film.cut`, else the end) the pending choice, else the default, is
//! taken, the conversation runs to its next hold, and that node's segment
//! starts on a deck. The other decks hold the segments the current node
//! may lead to, paused at their first frame, so their decoders have it.
//!
//! Every value is world state: og.Film and og.FilmDeck, the decks'
//! timelines and players, and the conversation (og.story's block). Only
//! the prepared dialogues (og.film.load, in every process) and the ops a
//! call queued for this tick are the engine's; [`Rt::views`] caches what
//! the last tick showed for og.film.view, which systems may call.
//!
//! og.Film, og.FilmDeck and the film events are lazy
//! (`crate::engine::lazy`): a world that never loads a film keeps its type
//! table and hash, and has no og.film system.

use crate::bind::story::{
    og_story_advance, og_story_choose, og_story_start, og_story_stop, og_story_view,
};
use crate::engine::{Engine, engine};
use crate::host::codec::Layout;
use crate::host::wire::{ChoiceView, FilmDesc, FilmInfo, FilmView, WindowView, record};
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) const FILM: &str = "og.Film";
pub(crate) const DECK: &str = "og.FilmDeck";
const SEGMENT: &str = "og.FilmSegment";
const CHOICE: &str = "og.FilmChoice";
const CHAPTER: &str = "og.FilmChapter";
const ENDED: &str = "og.FilmEnded";
/// og.film's records, registered together.
pub(crate) const RECORDS: [&str; 6] = [FILM, DECK, SEGMENT, CHOICE, CHAPTER, ENDED];
pub(crate) const SYSTEM: &str = "og.film";
const TIMELINE: &str = "og.Timeline";
const PLAYER: &str = crate::bind::video::PLAYER;
const AUDIO: &str = "og.AudioSource";
/// The node header that names a segment (and holds the conversation).
const HEADER: &str = "og-film";
const MAX_DECKS: u32 = 3;

/// A film node of a prepared dialogue.
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
    /// Per choice (by index): the film nodes it may lead to, through logic
    /// nodes.
    choices: Vec<Vec<u32>>,
    /// The film nodes its own steps may lead to (a node without choices).
    next: Vec<u32>,
}

/// A dialogue og.film.load prepared: its film nodes by id.
#[derive(Debug, Default)]
struct Prepared {
    nodes: BTreeMap<u32, Node>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Choose(u32),
    Play,
    Pause,
    Stop,
}

/// What a film showed at the end of the last tick (og.film.view).
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
}

/// og.film on one engine.
#[derive(Default)]
pub(crate) struct Rt {
    /// Prepared dialogues by name.
    films: BTreeMap<String, Arc<Prepared>>,
    ops: Vec<(u64, Op)>,
    views: BTreeMap<u64, Shown>,
    warned: BTreeSet<String>,
}

pub(crate) type ModuleState = Arc<Mutex<Rt>>;

fn lock(m: &Mutex<Rt>) -> MutexGuard<'_, Rt> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn state(engine_id: u32) -> Result<ModuleState, String> {
    Ok(engine(engine_id)?.modules.film.clone())
}

fn bad(m: impl Into<String>) -> og_core::Error {
    og_core::Error::new(og_core::ErrorKind::Invalid, m.into())
}

/// A film node's id: FNV-1a of its title (never 0, which is none).
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

// ── og.film.load ─────────────────────────────────────────────────────────────

/// Dialogue `name` as og.film plays it: every film node's segment loaded
/// and checked, and where each choice may lead.
fn prepare(eng: &Engine, world: &mut World, name: &str) -> Result<(Prepared, FilmInfo), String> {
    let graph = crate::bind::story::graph_of(eng.id, name)
        .ok_or_else(|| format!("no dialogue '{name}' is loaded: og.story.load it first"))?;
    let by_title: BTreeMap<&str, &og_story::graph::GraphNode> =
        graph.iter().map(|n| (n.title.as_str(), n)).collect();
    // The film nodes reachable from `to` without passing another film node.
    let reach = |to: &[String]| -> Vec<u32> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        let mut stack: Vec<&str> = to.iter().rev().map(String::as_str).collect();
        while let Some(t) = stack.pop() {
            if !seen.insert(t) {
                continue;
            }
            let Some(n) = by_title.get(t) else { continue };
            if n.headers.contains_key(HEADER) {
                let id = node_id(t);
                if !out.contains(&id) {
                    out.push(id);
                }
                continue;
            }
            for c in n.choices.iter().rev() {
                stack.extend(c.to.iter().rev().map(String::as_str));
            }
            stack.extend(n.to.iter().rev().map(String::as_str));
        }
        out
    };
    let mut p = Prepared::default();
    let mut info = FilmInfo {
        dialogue: name.to_string(),
        ..FilmInfo::default()
    };
    for n in &graph {
        let header = |k: &str| n.headers.get(k).cloned().unwrap_or_default();
        if let Some(c) = n.headers.get("og-film-chapter") {
            info.chapters.push(c.clone());
        }
        if let Some(e) = n.headers.get("og-film-ending") {
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
                chapter: header("og-film-chapter"),
                ending: header("og-film-ending"),
                choices: n.choices.iter().map(|c| reach(&c.to)).collect(),
                next: reach(&n.to),
            },
        );
        info.segments.push(n.title.clone());
    }
    if p.nodes.is_empty() {
        return Err(format!(
            "{name} has no film node: give a node an `og-film: segments/<segment>.ogt` header"
        ));
    }
    Ok((p, info))
}

pub fn og_film_load(world: u32, dialogue: String) -> Result<FilmInfo, String> {
    let eng = engine(world)?;
    let w = eng.world().map_err(|e| {
        format!(
            "og.film.load registers og.Film, which a system cannot do: load films in setup ({e})"
        )
    })?;
    crate::engine::lazy::ensure(&eng, w, FILM)?;
    let (p, info) = prepare(&eng, w, &dialogue)?;
    lock(&eng.modules.film).films.insert(dialogue, Arc::new(p));
    Ok(info)
}

// ── og.film.start ────────────────────────────────────────────────────────────

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

pub fn og_film_start(world: u32, desc: Option<String>) -> Result<Value, String> {
    let d: FilmDesc = crate::host::wire::arg(desc.as_deref(), "desc")?;
    let eng = engine(world)?;
    let w = eng.world().map_err(|e| {
        format!("og.film.start spawns a film's decks: call it outside a system ({e})")
    })?;
    if !lock(&eng.modules.film).films.contains_key(&d.dialogue) {
        return Err(format!(
            "the dialogue '{}' is not prepared: og.film.load it in setup",
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
    let film = spawn(w)?;
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
        insert(&eng, w, e, DECK, json!({ "film": film.to_bits() as f64 }))?;
    }
    insert(
        &eng,
        w,
        film,
        FILM,
        json!({
            "conversation": conversation,
            "decks": decks,
            "current": first as f64,
            "paused": d.paused,
        }),
    )?;
    lock(&eng.modules.film).views.insert(
        film.to_bits(),
        Shown {
            conversation,
            pending: -1,
            paused: d.paused,
            ..Shown::default()
        },
    );
    Ok(json!(film.to_bits() as f64))
}

// ── og.film.view and the ops ─────────────────────────────────────────────────

/// The choice taken when nothing was: `#default`, else the first enabled.
fn default_choice(choices: &[ChoiceView]) -> Option<u32> {
    choices
        .iter()
        .find(|c| c.enabled && c.tags.get("default").is_some())
        .or_else(|| choices.iter().find(|c| c.enabled))
        .map(|c| c.index)
}

pub fn og_film_view(world: u32, film: f64) -> Option<FilmView> {
    let st = state(world).ok()?;
    let s = lock(&st).views.get(&(film as u64)).cloned()?;
    let choices = if s.window > 0 {
        og_story_view(world, s.conversation)
            .map(|v| v.choices)
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
    });
    Some(FilmView {
        film: film as u64,
        node: s.node,
        segment: s.segment,
        texture: s.texture,
        time: s.time,
        cut: s.cut,
        choices,
        window,
        paused: s.paused,
        chapter: s.chapter,
        ending: s.ending,
        ended: s.ended,
    })
}

fn queue(world: u32, film: f64, op: Op) -> Result<(), String> {
    let st = state(world)?;
    let key = film as u64;
    let s = lock(&st)
        .views
        .get(&key)
        .cloned()
        .ok_or_else(|| format!("entity {key} is no film (og.film.start)"))?;
    if let Op::Choose(i) = op {
        if s.window != 1 {
            return Err(if s.window == 2 {
                "the window has locked: the cut takes what was chosen".into()
            } else {
                "no choice window is open (film.open to film.lock)".into()
            });
        }
        let v = og_story_view(world, s.conversation).ok_or("the film's conversation is gone")?;
        let c = v
            .choices
            .iter()
            .find(|c| c.index == i)
            .ok_or_else(|| format!("choice {i} is not offered (hidden, or no such choice)"))?;
        if !c.enabled {
            return Err(format!("choice {i} is disabled: {}", c.reason));
        }
    }
    lock(&st).ops.push((key, op));
    Ok(())
}

pub fn og_film_choose(world: u32, film: f64, index: u32) -> Result<(), String> {
    queue(world, film, Op::Choose(index))
}

pub fn og_film_play(world: u32, film: f64) -> Result<(), String> {
    queue(world, film, Op::Play)
}

pub fn og_film_pause(world: u32, film: f64) -> Result<(), String> {
    queue(world, film, Op::Pause)
}

pub fn og_film_stop(world: u32, film: f64) -> Result<(), String> {
    queue(world, film, Op::Stop)
}

// ── the system ───────────────────────────────────────────────────────────────

/// Registers og.film on a world that names its records
/// (`crate::engine::lazy`): its system, just before og.timeline.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    if world.systems().contains(&SYSTEM) || RECORDS.iter().any(|r| world.id_by_name(r).is_err()) {
        // The last of og.film's records to register schedules it.
        return Ok(());
    }
    // og.VideoPlayer is registered first (crate::engine::lazy's NEEDS).
    let engine_id = mount.id;
    let st = mount.modules.film.clone();
    let mut writes: Vec<String> = RECORDS.iter().map(|s| s.to_string()).collect();
    writes.extend([TIMELINE.to_string(), PLAYER.to_string()]);
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            writes,
            queries: vec![
                QueryDesc {
                    write: vec![FILM.into()],
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
    film: (SchemaId, Layout),
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

/// One film's decks, by entity id (a stable order).
type Decks = Vec<(Entity, record::FilmDeck)>;

fn system(ctx: &mut SystemContext<'_>, engine_id: u32, st: &ModuleState) -> og_core::Result<()> {
    let Ok(eng) = engine(engine_id) else {
        return Ok(());
    };
    let r = Recs {
        film: rec(ctx, &eng, FILM)?,
        deck: rec(ctx, &eng, DECK)?,
        timeline: rec(ctx, &eng, TIMELINE)?,
        player: rec(ctx, &eng, PLAYER)?,
    };
    let ops = std::mem::take(&mut lock(st).ops);
    let mut films: Vec<Entity> = Vec::new();
    for a in ctx.query_with(&QueryDesc {
        write: vec![FILM.into()],
        ..QueryDesc::default()
    })? {
        films.extend_from_slice(a.entities());
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
        let d: record::FilmDeck = r.deck.1.read(&ctx.get_bytes(e, r.deck.0)?).map_err(bad)?;
        decks.push((e, d));
    }
    decks.sort_by_key(|(e, _)| e.to_bits());
    films.sort_by_key(|e| e.to_bits());
    let mut views = BTreeMap::new();
    for f in films {
        let key = f.to_bits();
        let mine: Decks = decks
            .iter()
            .filter(|(_, d)| d.film == key)
            .cloned()
            .collect();
        let mine_ops: Vec<Op> = ops
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, o)| *o)
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

/// Puts node `id`'s segment on `deck`, paused at its first frame (or
/// playing from 0).
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
        json!({ "asset": node.timeline as f64, "time": 0.0, "playing": playing, "loop": false, "speed": 1.0 }),
    )?;
    if let Some((video, from)) = node.first {
        patch(
            ctx,
            &r.player,
            deck,
            json!({ "video": video as f64, "time": from, "playing": false, "wait": true }),
        )?;
    }
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
    let fb = ctx.get_bytes(f, r.film.0)?;
    let mut film: record::Film = r.film.1.read(&fb).map_err(bad)?;
    let conv = film.conversation;
    let dialogue = crate::bind::story::dialogue_of(eng.id, conv);
    let Some(prep) = dialogue
        .as_ref()
        .and_then(|d| lock(st).films.get(d).cloned())
    else {
        warn(
            eng,
            st,
            format!(
                "film {}: its dialogue {} is not prepared (og.film.load it in setup, in every process that restores a film)",
                f.to_bits(),
                dialogue.as_deref().unwrap_or("(its conversation is gone)")
            ),
        );
        return Ok(None);
    };
    let dt = ctx.time().dt;
    let half = dt * 0.5;
    let current = |film: &record::Film| Entity::from_bits(film.current);
    // og.timeline runs after this system and shows a playing timeline's
    // time plus dt, except on the tick it starts, resumes or is sought
    // (then its time as written): the markers are judged at the time this
    // tick shows. `advancing`: the current deck played last tick and still
    // does.
    let mut advancing = film.node != 0 && deck_state(ctx, r, current(&film))?.1;
    let shows =
        |ctx: &SystemContext<'_>, film: &record::Film, advancing: bool| -> og_core::Result<f32> {
            let t = deck_state(ctx, r, current(film))?.0;
            Ok(if advancing { t + dt } else { t })
        };
    for op in ops {
        match *op {
            Op::Choose(i) if film.window == 1 => film.pending = i as i32,
            Op::Choose(_) => {}
            Op::Play | Op::Pause => {
                film.paused = *op == Op::Pause;
                advancing &= !film.paused;
                if !film.ended && film.node != 0 {
                    patch(
                        ctx,
                        &r.timeline,
                        current(&film),
                        json!({ "playing": !film.paused }),
                    )?;
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

    // Begins node `title`'s segment on a deck (a warm one if it holds it),
    // then runs the node to its choices (or on to its next hold).
    let begin = |ctx: &mut SystemContext<'_>,
                 film: &mut record::Film,
                 decks: &mut Decks,
                 title: &str|
     -> og_core::Result<()> {
        let id = node_id(title);
        let Some(node) = prep.nodes.get(&id) else {
            warn(
                eng,
                st,
                format!(
                    "film {}: node '{title}' holds without a segment",
                    f.to_bits()
                ),
            );
            return Ok(());
        };
        let cur = current(film);
        let target = if film.decks > 1 {
            decks
                .iter()
                .find(|(d, k)| *d != cur && k.node == id)
                .or_else(|| decks.iter().find(|(d, _)| *d != cur))
                .map_or(cur, |(d, _)| *d)
        } else {
            cur
        };
        if target != cur && decks.iter().any(|(d, _)| *d == cur) {
            free_deck(ctx, r, cur)?;
        }
        let warm = decks.iter().any(|(d, k)| *d == target && k.node == id);
        if warm {
            patch(ctx, &r.timeline, target, json!({ "playing": !film.paused }))?;
        } else {
            load_deck(ctx, r, target, id, node, !film.paused)?;
        }
        for (d, k) in decks.iter_mut() {
            if *d == target {
                k.node = id;
            } else if *d == cur && target != cur {
                k.node = 0;
            }
        }
        film.current = target.to_bits();
        film.node = id;
        film.pending = -1;
        film.window = 0;
        send(
            ctx,
            eng,
            SEGMENT,
            json!({ "film": f.to_bits() as f64, "node": node.title, "segment": node.segment }),
        )?;
        if !node.chapter.is_empty() {
            film.chapter = id;
            send(
                ctx,
                eng,
                CHAPTER,
                json!({ "film": f.to_bits() as f64, "node": node.title, "chapter": node.chapter }),
            )?;
        }
        if let Err(e) = og_story_advance(eng.id, conv) {
            warn(
                eng,
                st,
                format!("film {}: node '{title}': {e}", f.to_bits()),
            );
        }
        skip_lines(eng, st, f, conv);
        Ok(())
    };

    if !film.ended {
        let view = og_story_view(eng.id, conv);
        if film.node == 0 {
            match &view {
                Some(v) if v.held => {
                    begin(ctx, &mut film, &mut decks, &v.node)?;
                    advancing = false;
                }
                _ => film.ended = true,
            }
        } else if let Some(node) = prep.nodes.get(&film.node) {
            let t = shows(ctx, &film, advancing)?;
            let choosing = view.as_ref().is_some_and(|v| !v.choices.is_empty());
            film.window = if !choosing || t + half < node.open {
                0
            } else if t + half < node.lock {
                1
            } else {
                2
            };
            if !film.paused && t + half >= node.cut {
                // The cut: the pending choice, else the default; then on to
                // the next hold.
                let title = node.title.clone();
                if let Some(v) = view.as_ref().filter(|v| !v.choices.is_empty()) {
                    let pending = u32::try_from(film.pending)
                        .ok()
                        .filter(|i| v.choices.iter().any(|c| c.index == *i && c.enabled));
                    if let Some(i) = pending.or_else(|| default_choice(&v.choices)) {
                        send(
                            ctx,
                            eng,
                            CHOICE,
                            json!({
                                "film": f.to_bits() as f64, "node": title, "index": i, "timedOut": pending.is_none(),
                            }),
                        )?;
                        if let Err(e) = og_story_choose(eng.id, conv, i) {
                            warn(
                                eng,
                                st,
                                format!("film {}: node '{title}': {e}", f.to_bits()),
                            );
                        }
                    }
                }
                skip_lines(eng, st, f, conv);
                match og_story_view(eng.id, conv) {
                    Some(v) if v.held => {
                        begin(ctx, &mut film, &mut decks, &v.node)?;
                        advancing = false;
                    }
                    Some(v) if !v.ended => {
                        warn(
                            eng,
                            st,
                            format!(
                                "film {}: after '{title}' the conversation shows choices outside a film node: give that node an `og-film` segment",
                                f.to_bits()
                            ),
                        );
                        film.ended = true;
                    }
                    _ => {
                        film.ended = true;
                        film.window = 0;
                        send(
                            ctx,
                            eng,
                            ENDED,
                            json!({ "film": f.to_bits() as f64, "node": title, "ending": node.ending }),
                        )?;
                    }
                }
            }
        }
    }

    // Warm the segments the current node may lead to on the other decks.
    if film.decks > 1
        && !film.ended
        && let Some(node) = prep.nodes.get(&film.node)
    {
        let t = shows(ctx, &film, advancing)?;
        let view = og_story_view(eng.id, conv);
        let mut want: Vec<u32> = Vec::new();
        match &view {
            Some(v) if v.held => want.push(node_id(&v.node)),
            Some(v) if !v.choices.is_empty() => {
                if t + half >= node.open {
                    let mut order: Vec<u32> =
                        u32::try_from(film.pending).ok().into_iter().collect();
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
        want.truncate((film.decks - 1) as usize);
        let cur = current(&film);
        let wanted: BTreeSet<u32> = want.iter().copied().collect();
        for w in want {
            if decks.iter().any(|(d, k)| *d != cur && k.node == w) {
                continue;
            }
            // A deck holding nothing the node may lead to.
            let free = decks
                .iter()
                .position(|(d, k)| *d != cur && !wanted.contains(&k.node));
            let (Some(i), Some(n)) = (free, prep.nodes.get(&w)) else {
                break;
            };
            load_deck(ctx, r, decks[i].0, w, n, false)?;
            decks[i].1.node = w;
        }
    }

    let out = r
        .film
        .1
        .encode(Some(&fb), Some(&film_json(&film)))
        .map_err(bad)?;
    if out != fb {
        ctx.set_bytes(f, r.film.0, &out)?;
    }
    let node = prep.nodes.get(&film.node);
    let time = shows(ctx, &film, advancing)?;
    let texture = deck_state(ctx, r, current(&film))?.2;
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
        window: film.window,
        pending: film.pending,
        paused: film.paused,
        chapter: prep
            .nodes
            .get(&film.chapter)
            .map(|n| n.chapter.clone())
            .unwrap_or_default(),
        ending: if film.ended {
            node.map(|n| n.ending.clone()).unwrap_or_default()
        } else {
            String::new()
        },
        ended: film.ended,
    }))
}

fn film_json(f: &record::Film) -> Value {
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
    })
}

/// A film node shows no Yarn lines (its captions are the segment's): a
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
                "film {}: node '{}' says '{}': a film node shows no Yarn lines; put what is said in its segment's caption track",
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

/// `film:choose:<i>`, `film:play`, `film:pause`: every film this engine shows.
pub(crate) fn register_commands(probe: &mut og_probe::Probe, engine_id: u32) {
    let _ = probe.register_command(
        "film",
        og_probe::CommandDesc {
            usage: "film:choose:<index> | film:play | film:pause".into(),
            doc: "take a choice in every film's open window (ChoiceView.index), or play or pause every film; it takes effect at this tick's og.film".into(),
        },
        move |_world, args| command(engine_id, args),
    );
}

fn command(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write film:choose:<index>, film:play or film:pause";
    let st = state(engine_id)?;
    let films: Vec<u64> = lock(&st).views.keys().copied().collect();
    if films.is_empty() {
        return Err("no film is playing (og.film.start)".into());
    }
    let op = match args.first().map(String::as_str) {
        Some("choose") => Op::Choose(
            args.get(1)
                .and_then(|a| a.trim().parse().ok())
                .ok_or("write film:choose:<index>, such as film:choose:0")?,
        ),
        Some("play") => Op::Play,
        Some("pause") => Op::Pause,
        _ => return Err(USAGE.into()),
    };
    for f in films {
        queue(engine_id, f as f64, op)?;
    }
    Ok(())
}
