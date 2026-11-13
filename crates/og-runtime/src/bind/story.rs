//! The IDL's `story` module over og-story (G2-8): dialogues and quests
//! loaded into the engine's library, the world's story (a state block,
//! `og.story`), the og.story system that runs timers and sends the story's
//! events before the game's systems, and the views a dialogue box and a
//! quest log read, resolved through og.i18n.
//!
//! og.Story and the story's events are lazy records (crate::engine::lazy): the
//! first og.story.load registers them, seeds the story from og.Rng's fork
//! "story" and adds the state block, so a world without a story keeps its
//! saves and hash. A restore of a save that has them registers them first
//! (lazy::before_restore), so the block is there to take its bytes.

use crate::engine::engine;
use crate::host::wire::{
    BarkView, CheckView, ChoiceView, ChosenView, DialogueView, FromJson, MarkerView, ObjectiveView,
    OneOf, QuestView, RewardView, RollView, SpeakerView, StoryAsset, StoryLine, TextSpan,
    TimerView, record,
};
use crate::parse;
use og_core::Rng;
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use og_story::{Dialogue, Event, Library, Quest, Story, Val, Wait};
use serde_json::{Map, Value, json};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

mod barks;
mod conversations;
mod load;
mod quests;
mod system;
mod variables;

pub use barks::*;
pub use conversations::*;
pub use load::*;
pub use quests::*;
pub(crate) use system::*;
pub use variables::*;

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
pub(crate) struct Rt {
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
const BRAIN: &str = crate::present::feel::vcam::BRAIN;
const VCAM: &str = crate::present::feel::vcam::VCAM;

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

/// og.story on one engine: its stories, their runtime and the story state
/// its world's block saves, shared with the block.
pub(crate) type ModuleState = Arc<Mutex<Rt>>;

fn lock(m: &Mutex<Rt>) -> std::sync::MutexGuard<'_, Rt> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn with<T>(engine: u32, f: impl FnOnce(&mut Rt) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut lock(&eng.modules.story)),
        // An engine that is gone has none: a fresh one.
        Err(_) => f(&mut Rt::default()),
    }
}

/// A story for a world that had none in its save (a fresh one on seed 0's
/// fork: deterministic on every host).
fn fresh() -> Story {
    Story::new(Rng::new(0).fork("story"))
}

struct StoryBlock(ModuleState);

impl StateBlock for StoryBlock {
    fn save(&self, out: &mut Vec<u8>) {
        lock(&self.0).story.get_or_insert_with(fresh).save(out);
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
        lock(&self.0).story = Some(story);
    }

    /// A story's state is its variables and choices: nothing in the world.
    fn shift(&mut self, _: &og_ecs::Shift) {}
}

/// og.Story was registered with `world` (crate::engine::lazy::ensure, or assembly
/// for a world whose game names it): its resource at zero, the story
/// seeded from og.Rng's fork "story" if the engine has none, and the
/// state block.
pub(crate) fn attach(mount: crate::engine::Mount<'_>, world: &mut World) -> Result<(), String> {
    let engine = mount.id;
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
    {
        let mut rt = lock(&mount.modules.story);
        if rt.story.is_none() {
            rt.story = Some(Story::new(seed));
        }
        // A new world: what the loaded stories read is granted afresh.
        rt.granted.clear();
    }
    let block = StoryBlock(mount.modules.story.clone());
    match world.add_state_block(SYSTEM, Box::new(block)) {
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
            crate::engine::lazy::ensure(&eng, world, n)?;
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
    layouts: &'a BTreeMap<String, crate::host::codec::Layout>,
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

fn log(eng: &crate::engine::Engine, warnings: Vec<String>) {
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
    crate::bind::i18n::og_i18n_t(engine, key.to_string(), Some(args.to_string()))
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
    let markup = match parse(&text, &crate::bind::i18n::language(engine_id)) {
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

// ── barks ───────────────────────────────────────────────────────────────

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
