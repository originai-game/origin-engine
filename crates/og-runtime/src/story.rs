//! The IDL's `story` module over og-story (G2-8): dialogues and quests
//! loaded into the engine's library, the world's story (a state block,
//! `og.story`), the og.story system that runs timers and sends the story's
//! events before the game's systems, and the views a dialogue box and a
//! quest log read, resolved through og.i18n.
//!
//! og.Story and the story's events are lazy records (super::lazy): the
//! first og.story.load registers them, seeds the story from og.Rng's fork
//! "story" and adds the state block, so a world without a story keeps its
//! saves and hash. A restore of a save that has them registers them first
//! (lazy::before_restore), so the block is there to take its bytes.

use super::engine::engine;
use super::wire::{
    BarkView, CheckView, ChoiceView, ChosenView, DialogueView, MarkerView, ObjectiveView, OneOf,
    QuestView, RewardView, RollView, SpeakerView, StoryAsset, StoryLine, TextSpan, TimerView,
    ToJson,
};
use super::{ok_void, parse, refuse};
use og_core::{IdMap, Rng};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use og_story::{Dialogue, Event, Library, Quest, Story, Val, Wait};
use serde_json::{Map, Value, json};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The story's resource.
pub(crate) const STORY: &str = "og.Story";
pub(crate) const SHOWN: &str = "og.DialogueShown";
pub(crate) const ENDED: &str = "og.DialogueEnded";
pub(crate) const CHECK: &str = "og.DialogueCheck";
pub(crate) const QUEST_UPDATED: &str = "og.QuestUpdated";
pub(crate) const STORY_EVENT: &str = "og.StoryEvent";
pub(crate) const BARK_SAID: &str = "og.BarkSaid";
/// og.Story and the events, registered together.
pub(crate) const RECORDS: [&str; 7] = [
    STORY,
    SHOWN,
    ENDED,
    CHECK,
    QUEST_UPDATED,
    STORY_EVENT,
    BARK_SAID,
];
/// The og.story system and its state block.
pub(crate) const SYSTEM: &str = "og.story";

#[derive(Default)]
struct Rt {
    lib: Library,
    story: Option<Story>,
    /// Voice clips by asset path: their og.audio sound handles.
    voices: BTreeMap<String, u64>,
    /// What og.story's system has been granted on the world it runs on
    /// (records a bark's `when` or an objective's `where` reads, events
    /// objectives count).
    granted: std::collections::BTreeSet<String>,
    /// Shots warned about as naming no virtual camera (not saved).
    unresolved: std::collections::BTreeSet<String>,
}

/// The records a line's `shot` writes and reads (R10): the brain's solo,
/// the virtual cameras a cast role resolves to.
const BRAIN: &str = crate::feel::vcam::BRAIN;
const VCAM: &str = crate::feel::vcam::VCAM;

/// Whether a loaded dialogue shows any line in a shot.
fn uses_shots(lib: &Library) -> bool {
    lib.dialogues.values().any(|d| {
        d.nodes.iter().any(|n| {
            !n.shot.is_empty()
                || n.steps.iter().any(|s| match s {
                    og_story::Step::Say(l) => !l.shot.is_empty() && l.shot != "none",
                    _ => false,
                })
        })
    })
}

thread_local! {
    static STORIES: RefCell<IdMap<u32, Rt>> = const { RefCell::new(IdMap::new()) };
}

fn with<T>(engine: u32, f: impl FnOnce(&mut Rt) -> T) -> T {
    STORIES.with(|s| f(s.borrow_mut().entry(engine).or_default()))
}

/// The engine is gone.
pub(crate) fn retire(engine: u32) {
    STORIES.with(|s| s.borrow_mut().remove(&engine));
}

/// A story for a world that had none in its save (a fresh one on seed 0's
/// fork: deterministic on every host).
fn fresh() -> Story {
    Story::new(Rng::new(0).fork("story"))
}

struct StoryBlock(u32);

impl StateBlock for StoryBlock {
    fn save(&self, out: &mut Vec<u8>) {
        with(self.0, |rt| rt.story.get_or_insert_with(fresh).save(out));
    }
    fn decode(&self, bytes: &[u8]) -> og_core::Result<Box<dyn Any + Send>> {
        Story::load(bytes)
            .map(|s| Box::new(s) as Box<dyn Any + Send>)
            .map_err(|e| og_core::Error::new(og_core::ErrorKind::Parse, e))
    }
    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let story = decoded
            .and_then(|d| d.downcast::<Story>().ok())
            .map_or_else(fresh, |s| *s);
        with(self.0, |rt| rt.story = Some(story));
    }

    /// A story's state is its variables and choices: nothing in the world.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}

/// og.Story was registered with `world` (super::lazy::ensure, or assembly
/// for a world whose game names it): its resource at zero, the story
/// seeded from og.Rng's fork "story" if the engine has none, and the
/// state block.
pub(crate) fn attach(engine: u32, world: &mut World) -> Result<(), String> {
    let err = |e: og_core::Error| e.reason().to_string();
    let id = world.id_by_name(STORY).map_err(err)?;
    if !world.has_resource_id(id) {
        let size: usize = world
            .schema(id)
            .map_err(err)?
            .fields
            .iter()
            .map(|f| f.ty.size())
            .sum();
        world
            .insert_resource_bytes(id, &vec![0; size])
            .map_err(err)?;
    }
    let seed = world
        .resource::<Rng>()
        .map(|r| r.fork("story"))
        .unwrap_or_else(|_| Rng::new(0).fork("story"));
    with(engine, |rt| {
        if rt.story.is_none() {
            rt.story = Some(Story::new(seed));
        }
        // A new world: what the loaded stories read is granted afresh.
        rt.granted.clear();
    });
    match world.add_state_block(SYSTEM, Box::new(StoryBlock(engine))) {
        Err(e) if e.reason().contains("already added") => {}
        r => r.map_err(err)?,
    }
    grant(engine, Some(world))
}

/// The records and events the loaded stories read, as og.story's system
/// must declare them: each world read's record (its longest name a layout
/// has) and each event an objective counts. og.Time is read as the clock.
fn needs(engine_id: u32) -> Result<Vec<String>, String> {
    let eng = engine(engine_id)?;
    let st = eng.state.borrow();
    let (reads, events) = with(engine_id, |rt| {
        let mut reads = Vec::new();
        let mut put = |x: &og_story::Expr| x.vars(&mut reads);
        for d in rt.lib.dialogues.values() {
            for b in &d.barks {
                b.when.iter().for_each(&mut put);
                b.lines
                    .iter()
                    .flat_map(|l| l.text.args.iter())
                    .for_each(|(_, x)| put(x));
            }
        }
        for o in rt
            .lib
            .quests
            .values()
            .flat_map(|q| q.stages.iter())
            .flat_map(|s| s.objectives.iter())
        {
            o.filter.iter().for_each(&mut put);
        }
        (reads, Story::counted_events(&rt.lib))
    });
    let mut out = std::collections::BTreeSet::new();
    if with(engine_id, |rt| uses_shots(&rt.lib)) {
        out.insert(BRAIN.to_string());
        out.insert(VCAM.to_string());
    }
    for v in reads.iter().filter(|v| v.contains('.')) {
        let path = v.strip_prefix("self.").unwrap_or(v);
        if path.starts_with("event.") || path.starts_with("og.Time.") {
            continue;
        }
        let rec = record_of(&st.layouts, path).ok_or_else(|| {
            format!(
                "'{v}' names no component, resource or event this game has (a world read is Record.field, self.Component.field or event.field)"
            )
        })?;
        out.insert(rec.0.to_string());
    }
    for e in events {
        if !st.layouts.contains_key(&e) {
            return Err(format!(
                "an objective counts '{e}', which is no event this game has (its name as declared, such as og.CollisionStarted)"
            ));
        }
        out.insert(e);
    }
    Ok(out.into_iter().collect())
}

/// Grants og.story's system what the loaded stories read; inside a system
/// (`world` None) only what it already has, else refused naming the fix.
fn grant(engine_id: u32, world: Option<&mut World>) -> Result<(), String> {
    let want = needs(engine_id)?;
    let missing: Vec<String> = with(engine_id, |rt| {
        want.iter()
            .filter(|n| !rt.granted.contains(*n))
            .cloned()
            .collect()
    });
    if missing.is_empty() {
        return Ok(());
    }
    let Some(world) = world else {
        return Err(format!(
            "this story reads {}, which og.story is not yet granted: load it in setup, outside a system",
            missing.join(", ")
        ));
    };
    // The virtual cameras' records are lazy: a dialogue with a shot
    // registers them as naming og.VirtualCamera would.
    if let Ok(eng) = engine(engine_id) {
        for n in &missing {
            super::lazy::ensure(&eng, world, n)?;
        }
    }
    let names: Vec<&str> = missing.iter().map(String::as_str).collect();
    world
        .grant(SYSTEM, &names)
        .map_err(|e| e.reason().to_string())?;
    with(engine_id, |rt| rt.granted.extend(missing));
    Ok(())
}

/// The layout a world read names (`Health.hp`, `og.Transform.position.x`)
/// and the field path after it: the longest record name that has a layout.
fn record_of<'a>(
    layouts: &'a BTreeMap<String, super::codec::Layout>,
    path: &'a str,
) -> Option<(&'a str, &'a str)> {
    path.rmatch_indices('.')
        .map(|(i, _)| (&path[..i], &path[i + 1..]))
        .find(|(rec, _)| layouts.contains_key(*rec))
}

fn walk(v: &Value, fields: &str) -> Option<Val> {
    let mut at = v;
    for f in fields.split('.') {
        at = at.get(f)?;
    }
    val_of(at)
}

/// The og.story system, mounted on every world before the game's systems.
/// It does nothing until og.Story is registered; then each tick it runs the
/// choice timers, counts the events objectives count, settles barks, starts
/// and stops voices, sends the events the story queued since the last run
/// and writes og.Story's counters.
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
    world
        .add_system(
            SystemDesc::new(SYSTEM).write::<og_audio::AudioSource>(),
            move |ctx| {
                let Ok(story_id) = ctx.id_by_name(STORY) else {
                    return Ok(());
                };
                let Ok(eng) = engine(engine_id) else {
                    return Ok(());
                };
                let time = ctx.time();
                let tick = time.tick as u32;
                let layout = |name: &str| eng.state.borrow().layouts.get(name).cloned();
                // The world as a story reads it.
                let read = |e: Option<u64>, path: &str| -> Option<Val> {
                    if let Some(f) = path.strip_prefix("og.Time.") {
                        return match f {
                            "tick" => Some(Val::Num(time.tick as f64)),
                            "dt" => Some(Val::Num(f64::from(time.dt))),
                            "elapsed" => Some(Val::Num(time.elapsed)),
                            _ => None,
                        };
                    }
                    let st = eng.state.borrow();
                    let (rec, fields) = record_of(&st.layouts, path)?;
                    let l = st.layouts.get(rec)?;
                    let id = ctx.id_by_name(rec).ok()?;
                    let bytes = match e {
                        Some(e) => ctx.get_bytes(Entity::from_bits(e), id).ok()?,
                        None => ctx.resource_bytes(id).ok()?,
                    };
                    walk(&l.decode(&bytes), fields)
                };
                // Events objectives count, as they were sent.
                let counted = with(engine_id, |rt| Story::counted_events(&rt.lib));
                let mut seen_events: Vec<(String, Value)> = Vec::new();
                for name in &counted {
                    let (Ok(id), Some(l)) = (ctx.id_by_name(name), layout(name)) else {
                        continue;
                    };
                    for b in ctx.events_bytes(id).unwrap_or_default() {
                        seen_events.push((name.clone(), l.decode(&b)));
                    }
                }
                let alive = |e: u64| ctx.is_alive(Entity::from_bits(e));
                let (starts, stops) = with(engine_id, |rt| {
                    let Rt {
                        lib, story, voices, ..
                    } = rt;
                    let s = story.get_or_insert_with(fresh);
                    s.tick(lib, tick);
                    for (name, ev) in &seen_events {
                        let with_event =
                            |e: Option<u64>, path: &str| match path.strip_prefix("event.") {
                                Some(f) => walk(ev, f),
                                None => read(e, path),
                            };
                        s.count_event(lib, name, &with_event);
                    }
                    s.run_barks(lib, tick, &read, &alive);
                    voice_plan(s, voices, tick)
                });
                shot(ctx, &eng, engine_id)?;
                // Voices: stop the ones whose line moved on or ended, start
                // the new lines' (each its own entity with an AudioSource).
                for e in stops {
                    let e = Entity::from_bits(e);
                    if ctx.is_alive(e) {
                        ctx.despawn(e)?;
                    }
                }
                let source = ctx.id_by_name("og.AudioSource")?;
                let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
                let mut spawned = Vec::new();
                if let Some(l) = layout("og.AudioSource") {
                    for (key, seq, handle) in starts {
                        let bytes = l
                            .encode(None, Some(&json!({ "sound": handle })))
                            .map_err(bad)?;
                        let e = ctx.spawn_bytes(vec![(source, bytes)])?;
                        spawned.push((key, seq, e.to_bits()));
                    }
                }
                let (events, counters, warnings) = with(engine_id, |rt| {
                    let s = rt.story.get_or_insert_with(fresh);
                    for (key, seq, e) in spawned {
                        started(s, &key, seq, e, tick);
                    }
                    let counters = json!({
                        "lines": s.seq,
                        "conversations": s.convs.values().filter(|c| c.wait != Wait::Ended).count(),
                        "activeQuests": s.quests.iter().filter(|q| q.status == "active").count(),
                    });
                    (
                        std::mem::take(&mut s.events),
                        counters,
                        std::mem::take(&mut s.warnings),
                    )
                });
                for e in &events {
                    let (name, v) = event_json(e);
                    if let Some(l) = layout(name) {
                        ctx.send_bytes(
                            ctx.id_by_name(name)?,
                            &l.encode(None, Some(&v)).map_err(bad)?,
                        )?;
                    }
                }
                if let Some(l) = layout(STORY) {
                    ctx.set_resource_bytes(
                        story_id,
                        &l.encode(None, Some(&counters)).map_err(bad)?,
                    )?;
                }
                log(&eng, warnings);
                Ok(())
            },
        )
        .map_err(|e| e.reason().to_string())
}

/// A line's shot (R10): the newest running conversation's current line's
/// shot is og.CameraBrain.solo while it shows. The solo the brain had when
/// the story took it is kept (Story.camera) and handed back when no line
/// has a shot, unless the game set another meanwhile, which is then the
/// one handed back (and a solo the game sets while the story holds it is
/// replaced at the next line's shot only).
fn shot(
    ctx: &mut SystemContext<'_>,
    eng: &super::engine::Engine,
    engine_id: u32,
) -> og_core::Result<()> {
    if !with(engine_id, |rt| rt.granted.contains(BRAIN)) {
        return Ok(());
    }
    let (Ok(bid), Ok(vid)) = (ctx.id_by_name(BRAIN), ctx.id_by_name(VCAM)) else {
        return Ok(());
    };
    let layouts = {
        let st = eng.state.borrow();
        (
            st.layouts.get(BRAIN).cloned(),
            st.layouts.get(VCAM).cloned(),
        )
    };
    let (Some(bl), Some(vl)) = layouts else {
        return Ok(());
    };
    if !ctx.has_resource_id(bid)? {
        return Ok(());
    }
    let (wanted, held) = with(engine_id, |rt| {
        let s = rt.story.as_ref();
        let wanted = s.and_then(|s| {
            let (id, shot) = s.shot()?;
            Some((shot.to_string(), s.convs.get(&id)?.cast.clone()))
        });
        (wanted, s.and_then(|s| s.camera))
    });
    let is_vcam = |e: Entity| ctx.is_alive(e) && ctx.get_bytes(e, vid).is_ok();
    // A name: that entity; a cast role: its entity's own virtual camera,
    // else the first (in query order) that looks at it.
    let want = match &wanted {
        None => None,
        Some((shot, cast)) => {
            let found = match cast.get(shot.as_str()) {
                Some(&bits) => {
                    let actor = Entity::from_bits(bits);
                    if is_vcam(actor) {
                        Some(actor)
                    } else {
                        let q = QueryDesc {
                            read: vec![VCAM.into()],
                            ..QueryDesc::default()
                        };
                        let mut hit = None;
                        'all: for a in ctx.query_with(&q)? {
                            for &e in a.entities() {
                                let v = vl.decode(&ctx.get_bytes(e, vid)?);
                                if v["lookAt"].as_f64().map_or(0, |x| x as u64) == bits {
                                    hit = Some(e);
                                    break 'all;
                                }
                            }
                        }
                        hit
                    }
                }
                None => ctx.find(shot).filter(|e| is_vcam(*e)),
            };
            if found.is_none() {
                let first = with(engine_id, |rt| rt.unresolved.insert(shot.clone()));
                if first {
                    log(
                        eng,
                        vec![format!(
                            "a line's shot '{shot}' names no virtual camera: no entity of that name with an og.VirtualCamera, and no cast role whose entity has one or is one's lookAt; the line shows without a shot"
                        )],
                    );
                }
            }
            found.map(Entity::to_bits)
        }
    };
    let bytes = ctx.resource_bytes(bid)?;
    let mut brain = bl.decode(&bytes);
    let cur = brain["solo"].as_f64().map_or(0, |x| x as u64);
    let (camera, write) = match (held, want) {
        (None, None) => return Ok(()),
        (None, Some(e)) => (Some((cur, e)), Some(e)),
        (Some((found, wrote)), Some(e)) => {
            let found = if cur == wrote { found } else { cur };
            (Some((found, e)), (cur != e).then_some(e))
        }
        (Some((found, wrote)), None) => (None, (cur == wrote).then_some(found)),
    };
    if let Some(solo) = write {
        brain["solo"] = json!(solo);
        brain["soloBlend"] = json!(-1.0);
        let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
        let next = bl.encode(Some(&bytes), Some(&brain)).map_err(bad)?;
        ctx.set_resource_bytes(bid, &next)?;
    }
    with(engine_id, |rt| {
        if let Some(s) = rt.story.as_mut() {
            s.camera = camera;
        }
    });
    Ok(())
}

/// The voice each speaking line or bark should have now: `c<conversation>`
/// for a conversation's current line, `b<entity>` for a bark, with its seq,
/// clip and length.
fn voice_wants(s: &Story) -> BTreeMap<String, (u32, String, u32, Option<u32>)> {
    let mut want = BTreeMap::new();
    for (id, c) in &s.convs {
        if c.wait == Wait::Ended {
            continue;
        }
        if let Some(l) = c.history.last().filter(|l| !l.voice.is_empty()) {
            want.insert(
                format!("c{id}"),
                (l.seq, l.voice.clone(), l.voice_ticks, l.voice_start),
            );
        }
    }
    for (e, b) in &s.barks {
        if !b.voice.is_empty() {
            want.insert(
                format!("b{e}"),
                (b.seq, b.voice.clone(), b.voice_ticks, b.voice_start),
            );
        }
    }
    want
}

/// Which voice entities to despawn (their line moved on, ended or played
/// out) and which voices to start: (key, seq, sound handle).
fn voice_plan(
    s: &mut Story,
    handles: &BTreeMap<String, u64>,
    tick: u32,
) -> (Vec<(String, u32, u64)>, Vec<u64>) {
    let want = voice_wants(s);
    let mut stops = Vec::new();
    s.voices.retain(|k, (seq, e)| {
        let keep = want
            .get(k)
            .is_some_and(|w| w.0 == *seq && w.3.is_some_and(|t| t.saturating_add(w.2) > tick));
        if !keep {
            stops.push(*e);
        }
        keep
    });
    let starts = want
        .into_iter()
        .filter(|(k, w)| w.3.is_none() && w.2 > 0 && !s.voices.contains_key(k))
        .filter_map(|(k, (seq, clip, _, _))| handles.get(&clip).map(|h| (k, seq, *h)))
        .collect();
    (starts, stops)
}

/// A voice started at `tick` on entity `e`: the line or bark it is for
/// times its end from now.
fn started(s: &mut Story, key: &str, seq: u32, e: u64, tick: u32) {
    s.voices.insert(key.to_string(), (seq, e));
    let rest = &key[1..];
    if key.starts_with('c') {
        if let Some(l) = rest
            .parse::<u32>()
            .ok()
            .and_then(|id| s.convs.get_mut(&id))
            .and_then(|c| c.history.last_mut())
            .filter(|l| l.seq == seq)
        {
            l.voice_start = Some(tick);
        }
    } else if let Some(b) = rest
        .parse::<u64>()
        .ok()
        .and_then(|e| s.barks.get_mut(&e))
        .filter(|b| b.seq == seq)
    {
        b.voice_start = Some(tick);
    }
}

fn event_json(e: &Event) -> (&'static str, Value) {
    match e {
        Event::Line { conv, seq } => (SHOWN, json!({ "conversation": conv, "seq": seq })),
        Event::Ended { conv } => (ENDED, json!({ "conversation": conv })),
        Event::Check { conv, roll } => (
            CHECK,
            json!({
                "conversation": conv, "stat": roll.stat, "target": roll.target,
                "roll": roll.roll, "passed": roll.passed,
            }),
        ),
        Event::Quest { quest, status } => {
            (QUEST_UPDATED, json!({ "quest": quest, "status": status }))
        }
        Event::Emit { conv, name, arg } => (
            STORY_EVENT,
            json!({ "conversation": conv, "name": name, "arg": arg }),
        ),
        Event::Bark {
            entity,
            bark,
            key,
            seq,
        } => (
            BARK_SAID,
            json!({ "entity": entity, "bark": bark, "key": key, "seq": seq }),
        ),
    }
}

fn log(eng: &super::engine::Engine, warnings: Vec<String>) {
    if warnings.is_empty() {
        return;
    }
    let mut st = eng.state.borrow_mut();
    for w in warnings {
        st.log.push(json!({
            "tick": og_ecs::last_tick(), "warning": true, "text": format!("og.story: {w}"),
        }));
    }
}

/// Runs `f` on the engine's library and story, then logs the story's
/// warnings; refused before anything was loaded.
fn story<T>(engine_id: u32, f: impl FnOnce(&Library, &mut Story) -> T) -> Result<T, String> {
    let (r, warnings) = with(engine_id, |rt| {
        let Rt { lib, story, .. } = rt;
        let Some(s) = story.as_mut() else {
            return (None, Vec::new());
        };
        let r = f(lib, s);
        (Some(r), std::mem::take(&mut s.warnings))
    });
    if let Ok(eng) = engine(engine_id) {
        log(&eng, warnings);
    }
    r.ok_or_else(|| "no dialogue or quest is loaded: og.story.load one in setup".to_string())
}

fn wire<T: ToJson>(r: Result<T, String>) -> String {
    match r {
        Ok(v) => super::wire::ok(&v),
        Err(e) => refuse(e),
    }
}

fn wire_void(r: Result<(), String>) -> String {
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

fn val_json(v: &Val) -> Value {
    match v {
        Val::Num(n) => json!(n),
        Val::Bool(b) => json!(b),
        Val::Str(s) => json!(s),
    }
}

fn val_of(v: &Value) -> Option<Val> {
    match v {
        Value::Number(n) => Some(Val::Num(n.as_f64()?)),
        Value::Bool(b) => Some(Val::Bool(*b)),
        Value::String(s) => Some(Val::Str(s.clone())),
        _ => None,
    }
}

fn args_json(args: &[(String, Val)]) -> Value {
    Value::Object(
        args.iter()
            .map(|(k, v)| (k.clone(), val_json(v)))
            .collect::<Map<_, _>>(),
    )
}

/// A key in the active locale with its arguments; empty for no key.
fn tr(engine: u32, key: &str, args: &Value) -> String {
    if key.is_empty() {
        return String::new();
    }
    super::i18n::og_i18n_t(engine, key.to_string(), Some(args.to_string()))
}

/// Localize first: both the display text and its scalar ranges belong to the
/// active locale, never to the source-language string saved in a conversation.
fn rich_line(engine_id: u32, key: &str, args: &Value) -> (String, Vec<TextSpan>) {
    use og_story::yarn::markup::{Property, parse};
    let formatted_args = Value::Object(
        args.as_object()
            .into_iter()
            .flatten()
            .map(|(key, value)| {
                let value = val_of(value).map_or_else(
                    || value.clone(),
                    |value| Value::String(og_story::yarn::functions::string(&value)),
                );
                (key.clone(), value)
            })
            .collect(),
    );
    let text = tr(engine_id, key, &formatted_args);
    let markup = match parse(&text, &super::i18n::language(engine_id)) {
        Ok(markup) => markup,
        Err(error) => {
            if let Ok(eng) = engine(engine_id) {
                log(
                    &eng,
                    vec![format!("invalid localized markup in {key}: {error}")],
                );
            }
            return (text, Vec::new());
        }
    };
    let spans = markup
        .spans
        .into_iter()
        .map(|span| TextSpan {
            name: span.name,
            position: span.position,
            length: span.length,
            properties: span
                .properties
                .into_iter()
                .map(|(name, value)| {
                    let value = match value {
                        Property::String(value) => OneOf::A(value),
                        Property::Number(value) => OneOf::B(OneOf::A(value)),
                        Property::Bool(value) => OneOf::B(OneOf::B(value)),
                    };
                    (name, value)
                })
                .collect(),
        })
        .collect();
    (markup.text, spans)
}

// ── loading ─────────────────────────────────────────────────────────────

fn insert(engine_id: u32, text: &str) -> Result<StoryAsset, String> {
    let ty = if text
        .lines()
        .map(str::trim)
        .find(|s| !s.is_empty() && !s.starts_with("//") && !s.starts_with('#'))
        .is_some_and(|s| s.starts_with("type "))
    {
        og_core::ogt::Document::parse(text)
            .map_err(|e| e.reason().to_string())?
            .ty
    } else {
        og_story::DIALOGUE_TYPE.to_string()
    };
    // The first load registers og.Story and the events (outside a system).
    let eng = engine(engine_id)?;
    match eng.world() {
        Ok(world) => super::lazy::ensure(&eng, world, STORY)?,
        Err(e) => {
            if !with(engine_id, |rt| rt.story.is_some()) {
                return Err(format!(
                    "the first og.story.load registers og.Story, which a system cannot do: load dialogues and quests in setup ({e})"
                ));
            }
        }
    }
    if ty == og_story::QUEST_TYPE {
        let q = Quest::parse(text)?;
        let info = StoryAsset {
            kind: "quest".into(),
            name: q.id.clone(),
            nodes: q.stages.iter().map(|s| s.id.clone()).collect(),
            speakers: Vec::new(),
            variables: q.vars.iter().map(|v| v.name.clone()).collect(),
        };
        let id = q.id.clone();
        let old = with(engine_id, |rt| {
            rt.lib.quests.insert(id.clone(), Arc::new(q))
        });
        if let Err(e) = grant(engine_id, eng.world().ok()) {
            with(engine_id, |rt| match old {
                Some(o) => rt.lib.quests.insert(id, o),
                None => rt.lib.quests.remove(&id),
            });
            return Err(e);
        }
        Ok(info)
    } else if ty == og_story::DIALOGUE_TYPE {
        let d = if text.trim_start().starts_with('{') {
            Dialogue::decode(text)?
        } else {
            Dialogue::parse(text)?
        };
        super::i18n::base_strings(&eng, &d.locale, &d.strings)?;
        load_voices(&eng, engine_id, &d);
        let info = StoryAsset {
            kind: "dialogue".into(),
            name: d.name.clone(),
            nodes: d
                .nodes
                .iter()
                .filter(|n| n.id == n.title)
                .map(|n| n.id.clone())
                .collect(),
            speakers: d.speakers.iter().map(|s| s.id.clone()).collect(),
            variables: d.vars.iter().map(|v| v.name.clone()).collect(),
        };
        let name = d.name.clone();
        let old = with(engine_id, |rt| {
            rt.lib.dialogues.insert(name.clone(), Arc::new(d))
        });
        if let Err(e) = grant(engine_id, eng.world().ok()) {
            with(engine_id, |rt| match old {
                Some(o) => rt.lib.dialogues.insert(name, o),
                None => rt.lib.dialogues.remove(&name),
            });
            return Err(e);
        }
        Ok(info)
    } else {
        Err(format!(
            "expected Yarn dialogue or 'type quest', found 'type {ty}'"
        ))
    }
}

/// Loads each voice clip a dialogue names into the engine's sounds, and
/// records its length in ticks (a one-shot's, as og.audio ends it). A clip
/// that does not load is warned about; its line shows without a voice.
fn load_voices(eng: &super::engine::Engine, engine_id: u32, d: &Dialogue) {
    let clips: std::collections::BTreeSet<String> = d
        .nodes
        .iter()
        .flat_map(|n| n.steps.iter())
        .filter_map(|s| match s {
            og_story::Step::Say(l) => Some(l.voice.clone()),
            _ => None,
        })
        .chain(
            d.barks
                .iter()
                .flat_map(|b| b.lines.iter().map(|l| l.voice.clone())),
        )
        .filter(|v| !v.is_empty())
        .collect();
    let rate = f64::from(eng.state.borrow().rate.max(1));
    let mut warnings = Vec::new();
    for clip in clips {
        if with(engine_id, |rt| rt.voices.contains_key(&clip)) {
            continue;
        }
        match super::assets::load_sound(eng, &clip) {
            Ok(i) => {
                let (handle, ticks) = {
                    let a = eng.assets.borrow();
                    let m = &a.sounds[i].meta;
                    let ticks =
                        (f64::from(m.frames) * rate / f64::from(m.sample_rate.max(1))).ceil();
                    (a.sound_handles[i], ticks as u32)
                };
                with(engine_id, |rt| {
                    rt.voices.insert(clip.clone(), handle);
                    rt.lib.voice_ticks.insert(clip, ticks);
                });
            }
            Err(e) => warnings.push(format!(
                "og.story: dialogue '{}': its voice '{clip}' does not load ({e}); its lines show without it",
                d.name
            )),
        }
    }
    let mut st = eng.state.borrow_mut();
    for w in warnings {
        st.log
            .push(json!({ "tick": og_ecs::last_tick(), "warning": true, "text": w }));
    }
}

pub fn og_story_load(world: u32, path: String) -> String {
    wire((|| {
        let eng = engine(world)?;
        let id = og_bundle::runtime::asset_id(&path).to_string();
        let text = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .story_text(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        insert(world, &text).map_err(|e| format!("{id}: {e}"))
    })())
}

pub fn og_story_parse(world: u32, text: String) -> String {
    wire(insert(world, &text))
}

// ── conversations ───────────────────────────────────────────────────────

pub fn og_story_start(world: u32, desc: Option<String>) -> String {
    wire((|| {
        let d: super::wire::StartDesc = super::wire::arg(desc.as_deref(), "desc")?;
        if d.dialogue.is_empty() {
            return Err("StartDesc.dialogue names the dialogue to start".into());
        }
        let dialogue = d.dialogue;
        let node = Some(d.node.as_str()).filter(|s| !s.is_empty());
        let mut cast = BTreeMap::new();
        if let Some(o) = d.cast.as_object() {
            for (k, v) in o {
                let e = v
                    .as_f64()
                    .filter(|e| e.is_finite() && *e >= 0.0)
                    .ok_or_else(|| format!("StartDesc.cast.{k} is an entity"))?;
                cast.insert(k.clone(), e as u64);
            }
        }
        let mut vars = BTreeMap::new();
        if let Some(o) = d.vars.as_object() {
            for (k, v) in o {
                let v = val_of(v).ok_or_else(|| {
                    format!("StartDesc.vars.{k} is a number, a boolean or a string")
                })?;
                vars.insert(k.clone(), v);
            }
        }
        story(world, |lib, s| s.start(lib, &dialogue, cast, vars, node))?.map(|c| json!(c))
    })())
}

fn speaker_view(
    engine: u32,
    d: &Dialogue,
    c: &og_story::Conv,
    id: &str,
    expression: &str,
) -> SpeakerView {
    let sp = d.speaker(id);
    let name = sp.map_or(String::new(), |s| tr(engine, &s.name, &json!({})));
    let expression = if expression.is_empty() {
        sp.map_or("", |s| s.expression.as_str())
    } else {
        expression
    };
    SpeakerView {
        id: id.to_string(),
        name,
        entity: c.cast.get(id).copied().unwrap_or(0),
        portrait: sp.map_or("", |s| s.portrait.as_str()).to_string(),
        expression: expression.to_string(),
        side: sp.map_or("left", |s| s.side.as_str()).to_string(),
    }
}

fn story_line(engine: u32, d: &Dialogue, c: &og_story::Conv, l: &og_story::Shown) -> StoryLine {
    let args = args_json(&l.args);
    let (text, spans) = rich_line(engine, &l.key, &args);
    let chosen = l.chosen.as_ref().and_then(|(i, id)| {
        let choice = d
            .nodes
            .iter()
            .flat_map(|n| &n.choices)
            .find(|c| &c.id == id)?;
        let (text, spans) = rich_line(engine, &choice.text.key, &args_json(&l.chosen_args));
        Some(ChosenView {
            index: *i,
            text,
            spans,
        })
    });
    StoryLine {
        seq: l.seq,
        node: d
            .node(&l.node)
            .map_or(l.node.as_str(), |n| n.title.as_str())
            .to_string(),
        speaker: speaker_view(engine, d, c, &l.speaker, &l.expression),
        key: l.key.clone(),
        text,
        spans,
        args,
        voice: l.voice.clone(),
        seen: l.seen,
        chosen,
    }
}

pub fn og_story_view(world: u32, conversation: u32) -> String {
    let view = story(world, |lib, s| -> Option<DialogueView> {
        let d = s.resolve(lib, conversation).ok()?;
        let c = s.convs.get(&conversation)?;
        let last = c.history.last();
        let mut choices = Vec::new();
        let mut timer = None;
        if c.wait == Wait::Choices
            && let Some(node) = d.node(&c.node)
        {
            for (i, ch) in node.choices.iter().enumerate() {
                let (visible, enabled, chance) = s.choice_state(lib, &d, conversation, ch);
                if !visible {
                    continue;
                }
                let args = args_json(&s.choice_args(conversation, ch));
                let values = c.choice_values.get(&ch.id);
                let (reason_key, reason_args) = match &ch.reason {
                    Some(r) => (
                        r.key.clone(),
                        values.map_or_else(
                            || args_json(&s.args(lib, Some(&d), conversation, &r.args)),
                            |v| args_json(&v.reason_args),
                        ),
                    ),
                    None => (String::new(), json!({})),
                };
                let check = ch.check.as_ref().map(|k| {
                    let target = values.and_then(|v| v.target).unwrap_or_else(|| {
                        match s.eval(lib, Some(&d), conversation, &k.target) {
                            Some(Val::Num(n)) => n,
                            _ => 0.0,
                        }
                    });
                    CheckView {
                        stat: k.stat.clone(),
                        target: target as f32,
                        chance: chance as f32,
                        kind: if k.active { "active" } else { "passive" }.to_string(),
                    }
                });
                let (text, spans) = rich_line(world, &ch.text.key, &args);
                choices.push(ChoiceView {
                    index: i as u32,
                    id: ch.id.clone(),
                    text,
                    spans,
                    key: ch.text.key.clone(),
                    args,
                    enabled,
                    reason: if enabled {
                        String::new()
                    } else {
                        tr(world, &reason_key, &reason_args)
                    },
                    reason_key,
                    reason_args,
                    visited: c.visited.contains(&format!("{}/{}", node.id, ch.id)),
                    kind: ch.kind.clone(),
                    check,
                });
            }
            if let (Some((left, total)), Some((_, t))) = (c.timer, &node.timer) {
                let default = node.choices.iter().position(|ch| &ch.id == t).unwrap_or(0);
                timer = Some(TimerView {
                    ticks_left: left,
                    ticks_total: total,
                    default_choice: default as u32,
                });
            }
        }
        let empty = og_story::Shown {
            node: c.node.clone(),
            step: 0,
            speaker: og_story::NARRATOR.into(),
            key: String::new(),
            args: Vec::new(),
            voice: String::new(),
            expression: String::new(),
            shot: String::new(),
            seq: 0,
            seen: false,
            chosen: None,
            chosen_args: Vec::new(),
            voice_ticks: 0,
            voice_start: None,
        };
        let l = last.unwrap_or(&empty);
        let args = args_json(&l.args);
        let (text, spans) = rich_line(world, &l.key, &args);
        Some(DialogueView {
            conversation,
            node: d
                .node(&c.node)
                .map_or(c.node.as_str(), |n| n.title.as_str())
                .to_string(),
            seq: l.seq,
            seen: l.seen,
            key: l.key.clone(),
            args,
            text,
            spans,
            voice: l.voice.clone(),
            voice_ticks: if l.voice.is_empty() { 0 } else { l.voice_ticks },
            voice_left: if c.wait == Wait::Ended {
                0
            } else {
                l.voice_left(s.tick)
            },
            speaker: speaker_view(world, &d, c, &l.speaker, &l.expression),
            shot: l.shot.clone(),
            choices,
            timer,
            last_check: c.last_roll.as_ref().map(|r| RollView {
                stat: r.stat.clone(),
                target: r.target as f32,
                roll: r.roll,
                passed: r.passed,
            }),
            ended: c.wait == Wait::Ended,
        })
    })
    .ok()
    .flatten();
    super::wire::text_of(&view)
}

pub fn og_story_history(world: u32, conversation: u32) -> String {
    let lines = story(world, |lib, s| -> Option<Vec<StoryLine>> {
        let c = s.convs.get(&conversation)?;
        let d = lib.dialogues.get(&c.dialogue)?;
        Some(
            c.history
                .iter()
                .map(|l| story_line(world, d, c, l))
                .collect(),
        )
    })
    .ok()
    .flatten()
    .unwrap_or_default();
    super::wire::text_of(&lines)
}

pub fn og_story_conversations(world: u32) -> String {
    story(world, |_, s| json!(s.convs.keys().collect::<Vec<_>>()))
        .unwrap_or_else(|_| json!([]))
        .to_string()
}

pub fn og_story_advance(world: u32, conversation: u32) -> String {
    wire_void(story(world, |lib, s| s.advance(lib, conversation)).and_then(|r| r))
}

pub fn og_story_choose(world: u32, conversation: u32, index: u32) -> String {
    wire_void(story(world, |lib, s| s.choose(lib, conversation, index)).and_then(|r| r))
}

pub fn og_story_stop(world: u32, conversation: u32) -> bool {
    story(world, |_, s| s.stop(conversation)).unwrap_or(false)
}

// ── barks ───────────────────────────────────────────────────────────────

/// A bark's entity and topic, which it needs.
fn bark_target(entity: u64, topic: &str) -> Result<(), String> {
    if entity == 0 {
        return Err("the desc's entity is the entity that barks".into());
    }
    if topic.is_empty() {
        return Err("the desc's topic names the barks to choose from, such as idle".into());
    }
    Ok(())
}

pub fn og_story_bark(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let d: super::wire::BarkDesc = super::wire::arg(desc.as_deref(), "desc")?;
        bark_target(d.entity, &d.topic)?;
        story(world, |lib, s| s.bark(lib, d.entity, &d.topic, &d.dialogue))?
    })())
}

pub fn og_story_ambient(world: u32, desc: Option<String>) -> String {
    wire_void((|| {
        let d: super::wire::AmbientDesc = super::wire::arg(desc.as_deref(), "desc")?;
        bark_target(d.entity, &d.topic)?;
        let (entity, topic, dialogue) = (d.entity, d.topic, d.dialogue);
        let every = d.every.min(1_000_000_000);
        let jitter = d.jitter.min(1_000_000_000);
        story(world, |lib, s| {
            if every > 0 {
                let known = lib
                    .dialogues
                    .values()
                    .filter(|x| dialogue.is_empty() || x.name == dialogue)
                    .any(|x| x.barks.iter().any(|b| b.topic == topic));
                if !known {
                    return Err(format!("no loaded dialogue has a bark of topic '{topic}'"));
                }
            }
            s.ambient(entity, &topic, &dialogue, every, jitter);
            Ok(())
        })?
    })())
}

fn bark_view(engine: u32, lib: &Library, s: &Story, e: u64, b: &og_story::Said) -> BarkView {
    let args = args_json(&b.args);
    let d = lib.dialogues.get(&b.dialogue);
    let sp = d.and_then(|d| d.speaker(&b.speaker));
    let (text, spans) = rich_line(engine, &b.key, &args);
    BarkView {
        entity: e,
        dialogue: b.dialogue.clone(),
        bark: b.bark.clone(),
        seq: b.seq,
        key: b.key.clone(),
        args,
        text,
        spans,
        speaker: SpeakerView {
            id: b.speaker.clone(),
            name: sp.map_or(String::new(), |x| tr(engine, &x.name, &json!({}))),
            entity: e,
            portrait: sp.map_or("", |x| x.portrait.as_str()).to_string(),
            expression: sp.map_or("", |x| x.expression.as_str()).to_string(),
            side: sp.map_or("left", |x| x.side.as_str()).to_string(),
        },
        voice: b.voice.clone(),
        voice_left: b.voice_left(s.tick),
        ticks_left: b.until.saturating_sub(s.tick),
        priority: b.priority,
    }
}

pub fn og_story_barks(world: u32) -> String {
    let barks = story(world, |lib, s| {
        s.barks
            .iter()
            .map(|(e, b)| bark_view(world, lib, s, *e, b))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    super::wire::text_of(&barks)
}

/// `dump_story`: the whole story as the probe writes it.
pub(crate) fn dump(engine_id: u32) -> Value {
    with(engine_id, |rt| {
        let lib = &rt.lib;
        let Some(s) = rt.story.as_ref() else {
            return json!({ "loaded": false });
        };
        let vars: Map<String, Value> = s
            .globals
            .iter()
            .map(|(k, v)| (k.clone(), val_json(v)))
            .collect();
        let convs: Vec<Value> = s.convs.iter().map(|(id, c)| {
            let l = c.history.last();
            json!({
                "conversation": id, "dialogue": c.dialogue, "node": c.node, "step": c.pc,
                "wait": match c.wait { Wait::Line => "line", Wait::Choices => "choices", Wait::Ended => "ended" },
                "line": l.map(|l| json!({ "seq": l.seq, "key": l.key, "args": args_json(&l.args), "voice": l.voice,
                    "voiceLeft": l.voice_left(s.tick), "seen": l.seen })),
                "locals": c.locals.iter().map(|(k, v)| (k.clone(), val_json(v))).collect::<Map<_, _>>(),
                "cast": c.cast, "visited": c.visited, "lines": c.history.len(),
                "timer": c.timer.map(|(l, t)| json!({ "ticksLeft": l, "ticksTotal": t })),
                "lastCheck": c.last_roll.as_ref().map(|r| json!({ "stat": r.stat, "target": r.target, "roll": r.roll, "passed": r.passed })),
            })
        }).collect();
        let quests: Vec<Value> = s.quests.iter().map(|q| json!({
            "id": q.id, "stage": q.stage, "status": q.status, "counts": q.counts, "failed": q.failed,
            "tracked": q.tracked, "updatedTick": q.updated_tick,
            "countsEvents": lib.quests.get(&q.id).and_then(|d| d.stage(&q.stage)).map(|st| st.objectives.iter()
                .filter_map(|o| o.on.as_ref().map(|e| json!({ "objective": o.id, "on": e }))).collect::<Vec<_>>()),
        })).collect();
        let barks: Vec<Value> = s.barks.iter().map(|(e, b)| json!({
            "entity": e, "dialogue": b.dialogue, "bark": b.bark, "key": b.key, "seq": b.seq,
            "priority": b.priority, "ticksLeft": b.until.saturating_sub(s.tick), "voiceLeft": b.voice_left(s.tick),
        })).collect();
        let cooldowns: Vec<Value> = s
            .bark_last_by
            .iter()
            .map(|((b, e), t)| json!({ "bark": b, "entity": e, "saidTick": t }))
            .collect();
        json!({
            "loaded": true,
            "tick": s.tick,
            "lines": s.seq,
            "dialogues": lib.dialogues.keys().collect::<Vec<_>>(),
            "quests": quests,
            "variables": vars,
            "seen": s.seen,
            "read": s.read.len(),
            "conversations": convs,
            "barks": barks,
            "ambient": s.ambient.iter().map(|(e, a)| json!({ "entity": e, "topic": a.topic, "every": a.every, "jitter": a.jitter, "nextTick": a.next })).collect::<Vec<_>>(),
            "barkLast": s.bark_last,
            "barkCooldowns": cooldowns,
            "barkOnce": s.bark_once,
            "voices": s.voices.iter().map(|(k, (seq, e))| json!({ "for": k, "seq": seq, "entity": e })).collect::<Vec<_>>(),
            "voiceClips": rt.voices,
            "granted": rt.granted,
            "rng": { "seed": s.rng.seed(), "state": s.rng.save() },
        })
    })
}

// ── variables ───────────────────────────────────────────────────────────

pub fn og_story_set_saliency(world: u32, strategy: String) -> String {
    use og_story::yarn::saliency::Saliency;
    wire_void((|| {
        let strategy = match strategy.as_str() {
            "first" => Saliency::First,
            "best" => Saliency::Best,
            "bestLeastRecentlyViewed" => Saliency::BestLeastRecentlyViewed,
            "randomBestLeastRecentlyViewed" => Saliency::RandomBestLeastRecentlyViewed,
            "conditionalFirstWeighted" => Saliency::ConditionalFirstWeighted,
            _ => return Err(format!("unknown Yarn saliency strategy '{strategy}'")),
        };
        story(world, |_, s| s.saliency = strategy)
    })())
}

pub fn og_story_get(world: u32, name: String) -> String {
    story(world, |lib, s| s.get(lib, &name))
        .ok()
        .flatten()
        .map_or(Value::Null, |v| val_json(&v))
        .to_string()
}

pub fn og_story_set(world: u32, name: String, value: String) -> String {
    wire_void((|| {
        let v = parse(Some(&value))?;
        let v = val_of(&v).ok_or("a story variable is a number, a boolean or a string")?;
        story(world, |lib, s| s.set(lib, &name, v))
    })())
}

pub fn og_story_reset(world: u32) {
    let _ = story(world, |_, s| s.reset());
}

// ── quests ──────────────────────────────────────────────────────────────

pub fn og_story_start_quest(world: u32, id: String) -> String {
    wire(
        story(world, |lib, s| s.start_quest(lib, &id))
            .and_then(|r| r)
            .map(|b| json!(b)),
    )
}

pub fn og_story_progress(
    world: u32,
    quest: String,
    objective: String,
    amount: Option<u32>,
) -> bool {
    story(world, |lib, s| {
        s.progress(lib, &quest, &objective, amount.unwrap_or(1))
    })
    .unwrap_or(false)
}

fn quest_op(world: u32, op: og_story::QuestOp) -> String {
    wire(
        story(world, |lib, s| {
            let r = s.quest_op(lib, &op);
            s.settle(lib);
            r
        })
        .and_then(|r| r)
        .map(|b| json!(b)),
    )
}

pub fn og_story_set_stage(world: u32, quest: String, stage: String) -> String {
    quest_op(world, og_story::QuestOp::Stage(quest, stage))
}

pub fn og_story_complete_quest(world: u32, quest: String) -> String {
    quest_op(world, og_story::QuestOp::Complete(quest))
}

pub fn og_story_fail_quest(world: u32, quest: String) -> String {
    quest_op(world, og_story::QuestOp::Fail(quest))
}

fn quest_view(engine: u32, lib: &Library, s: &Story, q: &og_story::QuestState) -> QuestView {
    let def = lib.quests.get(&q.id);
    let none = json!({});
    let t = |k: &str| tr(engine, k, &none);
    let stage = def.and_then(|d| d.stage(&q.stage));
    let mut objectives = Vec::new();
    let mut marker = q.marker.map(|(e, p)| MarkerView {
        entity: e,
        position: glam::Vec3::from_array(p),
    });
    if let Some(st) = stage {
        for o in &st.objectives {
            let count = q.counts.get(&o.id).copied().unwrap_or(0);
            let done = count >= o.target;
            let mut args = args_json(&s.args(lib, None, 0, &o.text.args));
            args["count"] = json!(count);
            args["target"] = json!(o.target);
            if marker.is_none()
                && !done
                && let Some(p) = o.marker
            {
                marker = Some(MarkerView {
                    entity: 0,
                    position: glam::Vec3::from_array(p),
                });
            }
            objectives.push(ObjectiveView {
                id: o.id.clone(),
                text: tr(engine, &o.text.key, &args),
                count,
                target: o.target,
                done,
                failed: q.failed.contains(&o.id),
                optional: o.optional,
                hidden: o.hidden && count == 0 && !done,
            });
        }
    }
    let summary = stage
        .map(|st| st.summary.as_str())
        .filter(|k| !k.is_empty())
        .or(def.map(|d| d.summary.as_str()))
        .unwrap_or("");
    QuestView {
        id: q.id.clone(),
        title: def.map_or(String::new(), |d| t(&d.title)),
        summary: t(summary),
        category: def.map_or("", |d| d.category.as_str()).to_string(),
        stage: q.stage.clone(),
        status: q.status.to_string(),
        tracked: q.tracked,
        is_new: q.is_new,
        updated: q.updated,
        updated_tick: q.updated_tick,
        giver: def.map_or(String::new(), |d| t(&d.giver)),
        region: def.map_or(String::new(), |d| t(&d.region)),
        marker,
        rewards: def.map_or(Vec::new(), |d| {
            d.rewards
                .iter()
                .map(|r| RewardView {
                    name: t(&r.name),
                    amount: r.amount,
                    image: r.image.clone(),
                    rarity: r.rarity.clone(),
                })
                .collect()
        }),
        objectives,
    }
}

pub fn og_story_quest(world: u32, id: String) -> String {
    let quest = story(world, |lib, s| {
        s.quest(&id).map(|q| quest_view(world, lib, s, q))
    })
    .ok()
    .flatten();
    super::wire::text_of(&quest)
}

pub fn og_story_quests(world: u32) -> String {
    let quests = story(world, |lib, s| {
        s.quests
            .iter()
            .map(|q| quest_view(world, lib, s, q))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    super::wire::text_of(&quests)
}

fn with_quest(world: u32, id: &str, f: impl FnOnce(&mut og_story::QuestState)) -> bool {
    story(world, |_, s| {
        match s.quests.iter_mut().find(|q| q.id == id) {
            Some(q) => {
                f(q);
                true
            }
            None => false,
        }
    })
    .unwrap_or(false)
}

pub fn og_story_track(world: u32, id: String, on: bool) -> bool {
    with_quest(world, &id, |q| q.tracked = on)
}

pub fn og_story_mark_seen(world: u32, id: String) -> bool {
    with_quest(world, &id, |q| {
        q.is_new = false;
        q.updated = false;
    })
}

pub fn og_story_mark(world: u32, id: String, desc: Option<String>) -> bool {
    let Ok(d) = super::wire::arg::<super::wire::MarkerDesc>(desc.as_deref(), "desc") else {
        return false;
    };
    let (entity, pos) = (d.entity, d.position.to_array());
    with_quest(world, &id, |q| {
        q.marker = (entity != 0 || pos != [0.0; 3]).then_some((entity, pos));
    })
}
