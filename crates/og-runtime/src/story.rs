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
use super::{ok, ok_void, parse, refuse};
use og_core::{IdMap, Rng};
use og_ecs::{StateBlock, SystemDesc, World};
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
/// og.Story and the events, registered together.
pub(crate) const RECORDS: [&str; 6] = [STORY, SHOWN, ENDED, CHECK, QUEST_UPDATED, STORY_EVENT];
/// The og.story system and its state block.
pub(crate) const SYSTEM: &str = "og.story";

#[derive(Default)]
struct Rt {
    lib: Library,
    story: Option<Story>,
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
    });
    match world.add_state_block(SYSTEM, Box::new(StoryBlock(engine))) {
        Err(e) if e.reason().contains("already added") => Ok(()),
        r => r.map_err(err),
    }
}

/// The og.story system, mounted on every world before the game's systems.
/// It does nothing until og.Story is registered; then each tick it runs the
/// choice timers, sends the events the story queued since the last run and
/// writes og.Story's counters.
pub(crate) fn mount(world: &mut World, engine_id: u32) -> Result<(), String> {
    world
        .add_system(SystemDesc::new(SYSTEM), move |ctx| {
            let Ok(story_id) = ctx.id_by_name(STORY) else {
                return Ok(());
            };
            let tick = ctx.time().tick;
            let (events, counters, warnings) = with(engine_id, |rt| {
                let s = rt.story.get_or_insert_with(fresh);
                s.tick(&rt.lib, tick as u32);
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
            let Ok(eng) = engine(engine_id) else {
                return Ok(());
            };
            let layouts: BTreeMap<&str, super::codec::Layout> = {
                let st = eng.state.borrow();
                RECORDS
                    .iter()
                    .filter_map(|n| st.layouts.get(*n).map(|l| (*n, l.clone())))
                    .collect()
            };
            let bad = |e: String| og_core::Error::new(og_core::ErrorKind::Invalid, e);
            for e in &events {
                let (name, v) = event_json(e);
                if let Some(l) = layouts.get(name) {
                    ctx.send_bytes(
                        ctx.id_by_name(name)?,
                        &l.encode(None, Some(&v)).map_err(bad)?,
                    )?;
                }
            }
            if let Some(l) = layouts.get(STORY) {
                ctx.set_resource_bytes(story_id, &l.encode(None, Some(&counters)).map_err(bad)?)?;
            }
            log(&eng, warnings);
            Ok(())
        })
        .map_err(|e| e.reason().to_string())
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
        let Rt { lib, story } = rt;
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

fn wire(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => ok(v),
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

// ── loading ─────────────────────────────────────────────────────────────

fn insert(engine_id: u32, text: &str) -> Result<Value, String> {
    let ty = og_core::ogt::Document::parse(text)
        .map_err(|e| e.reason().to_string())?
        .ty;
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
        let info = json!({
            "kind": "quest", "name": q.id,
            "nodes": q.stages.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
            "speakers": [],
            "variables": q.vars.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        });
        with(engine_id, |rt| {
            rt.lib.quests.insert(q.id.clone(), Arc::new(q))
        });
        Ok(info)
    } else if ty == og_story::DIALOGUE_TYPE {
        let d = Dialogue::parse(text)?;
        let info = json!({
            "kind": "dialogue", "name": d.name,
            "nodes": d.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>(),
            "speakers": d.speakers.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
            "variables": d.vars.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
        });
        with(engine_id, |rt| {
            rt.lib.dialogues.insert(d.name.clone(), Arc::new(d))
        });
        Ok(info)
    } else {
        Err(format!(
            "expected 'type dialogue' or 'type quest', found 'type {ty}'"
        ))
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
        let d = parse(desc.as_deref())?;
        let dialogue = d["dialogue"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("StartDesc.dialogue names the dialogue to start")?
            .to_string();
        let node = d["node"].as_str().filter(|s| !s.is_empty());
        let mut cast = BTreeMap::new();
        if let Some(o) = d["cast"].as_object() {
            for (k, v) in o {
                let e = v
                    .as_f64()
                    .filter(|e| e.is_finite() && *e >= 0.0)
                    .ok_or_else(|| format!("StartDesc.cast.{k} is an entity"))?;
                cast.insert(k.clone(), e as u64);
            }
        }
        let mut vars = BTreeMap::new();
        if let Some(o) = d["vars"].as_object() {
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

fn speaker_json(
    engine: u32,
    d: &Dialogue,
    c: &og_story::Conv,
    id: &str,
    expression: &str,
) -> Value {
    let sp = d.speaker(id);
    let name = sp.map_or(String::new(), |s| tr(engine, &s.name, &json!({})));
    let expression = if expression.is_empty() {
        sp.map_or("", |s| s.expression.as_str())
    } else {
        expression
    };
    json!({
        "id": id,
        "name": name,
        "entity": c.cast.get(id).copied().unwrap_or(0),
        "portrait": sp.map_or("", |s| s.portrait.as_str()),
        "expression": expression,
        "side": sp.map_or("left", |s| s.side.as_str()),
    })
}

fn line_json(engine: u32, d: &Dialogue, c: &og_story::Conv, l: &og_story::Shown) -> Value {
    let args = args_json(&l.args);
    let chosen = l.chosen.as_ref().and_then(|(i, _)| {
        let choice = d.node(&l.node)?.choices.get(*i as usize)?;
        Some(json!({ "index": i, "text": tr(engine, &choice.text.key, &json!({})) }))
    });
    json!({
        "seq": l.seq,
        "node": l.node,
        "speaker": speaker_json(engine, d, c, &l.speaker, &l.expression),
        "key": l.key,
        "text": tr(engine, &l.key, &args),
        "args": args,
        "voice": l.voice,
        "seen": l.seen,
        "chosen": chosen,
    })
}

pub fn og_story_view(world: u32, conversation: u32) -> String {
    story(world, |lib, s| -> Option<Value> {
        let d = s.resolve(lib, conversation).ok()?;
        let c = s.convs.get(&conversation)?;
        let last = c.history.last();
        let mut choices = Vec::new();
        let mut timer = Value::Null;
        if c.wait == Wait::Choices
            && let Some(node) = d.node(&c.node)
        {
            for (i, ch) in node.choices.iter().enumerate() {
                let (visible, enabled, chance) = s.choice_state(lib, &d, conversation, ch);
                if !visible {
                    continue;
                }
                let args = args_json(&s.args(lib, Some(&d), conversation, &ch.text.args));
                let (reason_key, reason_args) = match &ch.reason {
                    Some(r) => (
                        r.key.clone(),
                        args_json(&s.args(lib, Some(&d), conversation, &r.args)),
                    ),
                    None => (String::new(), json!({})),
                };
                let check = ch.check.as_ref().map(|k| {
                    let target = match s.eval(lib, Some(&d), conversation, &k.target) {
                        Some(Val::Num(n)) => n,
                        _ => 0.0,
                    };
                    json!({
                        "stat": k.stat, "target": target, "chance": chance,
                        "kind": if k.active { "active" } else { "passive" },
                    })
                });
                choices.push(json!({
                    "index": i,
                    "id": ch.id,
                    "text": tr(world, &ch.text.key, &args),
                    "key": ch.text.key,
                    "args": args,
                    "enabled": enabled,
                    "reason": if enabled { String::new() } else { tr(world, &reason_key, &reason_args) },
                    "reasonKey": reason_key,
                    "reasonArgs": reason_args,
                    "visited": c.visited.contains(&format!("{}/{}", node.id, ch.id)),
                    "kind": ch.kind,
                    "check": check,
                }));
            }
            if let (Some((left, total)), Some((_, t))) = (c.timer, &node.timer) {
                let default = node.choices.iter().position(|ch| &ch.id == t).unwrap_or(0);
                timer = json!({ "ticksLeft": left, "ticksTotal": total, "defaultChoice": default });
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
            seq: 0,
            seen: false,
            chosen: None,
        };
        let l = last.unwrap_or(&empty);
        let args = args_json(&l.args);
        Some(json!({
            "conversation": conversation,
            "node": c.node,
            "seq": l.seq,
            "seen": l.seen,
            "key": l.key,
            "args": args,
            "text": tr(world, &l.key, &args),
            "voice": l.voice,
            "speaker": speaker_json(world, &d, c, &l.speaker, &l.expression),
            "choices": choices,
            "timer": timer,
            "lastCheck": c.last_roll.as_ref().map(|r| json!({
                "stat": r.stat, "target": r.target, "roll": r.roll, "passed": r.passed,
            })),
            "ended": c.wait == Wait::Ended,
        }))
    })
    .ok()
    .flatten()
    .unwrap_or(Value::Null)
    .to_string()
}

pub fn og_story_history(world: u32, conversation: u32) -> String {
    story(world, |lib, s| -> Option<Value> {
        let c = s.convs.get(&conversation)?;
        let d = lib.dialogues.get(&c.dialogue)?;
        Some(Value::Array(
            c.history
                .iter()
                .map(|l| line_json(world, d, c, l))
                .collect(),
        ))
    })
    .ok()
    .flatten()
    .unwrap_or_else(|| json!([]))
    .to_string()
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

// ── variables ───────────────────────────────────────────────────────────

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

fn quest_json(engine: u32, lib: &Library, s: &Story, q: &og_story::QuestState) -> Value {
    let def = lib.quests.get(&q.id);
    let none = json!({});
    let t = |k: &str| tr(engine, k, &none);
    let stage = def.and_then(|d| d.stage(&q.stage));
    let mut objectives = Vec::new();
    let mut marker = q
        .marker
        .map(|(e, p)| json!({ "entity": e, "position": { "x": p[0], "y": p[1], "z": p[2] } }));
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
                marker =
                    Some(json!({ "entity": 0, "position": { "x": p[0], "y": p[1], "z": p[2] } }));
            }
            objectives.push(json!({
                "id": o.id,
                "text": tr(engine, &o.text.key, &args),
                "count": count,
                "target": o.target,
                "done": done,
                "failed": q.failed.contains(&o.id),
                "optional": o.optional,
                "hidden": o.hidden && count == 0 && !done,
            }));
        }
    }
    let summary = stage
        .map(|st| st.summary.as_str())
        .filter(|k| !k.is_empty())
        .or(def.map(|d| d.summary.as_str()))
        .unwrap_or("");
    json!({
        "id": q.id,
        "title": def.map_or(String::new(), |d| t(&d.title)),
        "summary": t(summary),
        "category": def.map_or("", |d| d.category.as_str()),
        "stage": q.stage,
        "status": q.status,
        "tracked": q.tracked,
        "isNew": q.is_new,
        "updated": q.updated,
        "updatedTick": q.updated_tick,
        "giver": def.map_or(String::new(), |d| t(&d.giver)),
        "region": def.map_or(String::new(), |d| t(&d.region)),
        "marker": marker,
        "rewards": def.map_or(Vec::new(), |d| d.rewards.iter().map(|r| json!({
            "name": t(&r.name), "amount": r.amount, "image": r.image, "rarity": r.rarity,
        })).collect()),
        "objectives": objectives,
    })
}

pub fn og_story_quest(world: u32, id: String) -> String {
    story(world, |lib, s| {
        s.quest(&id).map(|q| quest_json(world, lib, s, q))
    })
    .ok()
    .flatten()
    .unwrap_or(Value::Null)
    .to_string()
}

pub fn og_story_quests(world: u32) -> String {
    story(world, |lib, s| {
        Value::Array(
            s.quests
                .iter()
                .map(|q| quest_json(world, lib, s, q))
                .collect(),
        )
    })
    .unwrap_or_else(|_| json!([]))
    .to_string()
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
    let Ok(d) = parse(desc.as_deref()) else {
        return false;
    };
    let entity = d["entity"].as_f64().filter(|e| *e >= 0.0).unwrap_or(0.0) as u64;
    let p = &d["position"];
    let pos = ["x", "y", "z"].map(|k| p[k].as_f64().unwrap_or(0.0) as f32);
    with_quest(world, &id, |q| {
        q.marker = (entity != 0 || pos != [0.0; 3]).then_some((entity, pos));
    })
}
