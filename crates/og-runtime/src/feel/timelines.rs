//! `og.tween.timeline` and the members of the `TweenTimeline` it answers:
//! a GSAP-shaped timeline over og-tween's [`og_tween::Timeline`].
//!
//! The object a game holds is a handle into this thread's table (as an
//! `RngStream` is): its world, its tween handle, its defaults and its last
//! refusal. The timeline itself is og.tween's state, so snapshots cover it.

use super::tweens::{component_schema, ease_of, has, lanes, required, token_name};
use super::*;
use crate::engine::HANDLES;
use crate::wire::{
    OneOf, StaggerDesc, TimelineDesc, TimelineTweenVars, TweenDefaults, TweenKeyframe,
};
use og_tween::{Callbacks, Child, Item, Position, Stagger, StaggerFrom, Timeline};
use std::rc::Rc;

/// A timeline object: its world, its tween, its defaults and why its last
/// refused call added nothing.
pub(crate) struct TimelineRef {
    world: u32,
    tween: u64,
    defaults: TweenDefaults,
    error: Option<String>,
}

fn with_ref<T>(h: u32, f: impl FnOnce(&mut TimelineRef) -> T) -> Option<T> {
    HANDLES.with(|t| t.borrow_mut().timelines.get_mut(h).map(f))
}

/// The timeline of object `h`, its engine and its defaults.
fn target(h: u32) -> Result<(Rc<Engine>, u64, TweenDefaults), String> {
    let (world, tween, defaults) =
        with_ref(h, |r| (r.world, r.tween, r.defaults.clone())).ok_or("not a timeline")?;
    Ok((engine(world)?, tween, defaults))
}

/// Runs `f` on object `h`'s timeline, recording a refusal as its `error`;
/// answers `h` (so calls chain), or 0 when it is not a timeline.
fn build(h: u32, f: impl FnOnce(&Engine, u64, &TweenDefaults) -> Result<(), String>) -> u32 {
    let r = target(h).and_then(|(eng, tween, defaults)| f(&eng, tween, &defaults));
    with_ref(h, |t| {
        t.error = r.err();
        h
    })
    .unwrap_or(0)
}

/// Runs `f` on the timeline itself.
fn on_timeline<T>(
    eng: &Engine,
    tween: u64,
    f: impl FnOnce(&mut Timeline) -> T,
) -> Result<T, String> {
    feel_of(eng)?
        .tweens
        .with_timeline(tween, f)
        .ok_or_else(|| "the timeline was killed".to_string())
}

fn position(text: Option<&str>) -> Result<Option<Position>, String> {
    Ok(
        crate::wire::arg::<Option<OneOf<f64, String>>>(text, "position")?.map(|p| match p {
            OneOf::A(t) => Position::At(t),
            OneOf::B(s) => Position::Text(s),
        }),
    )
}

/// A callback's name, at most 63 bytes (og.TweenEvent's `name`).
fn name_of(name: &Option<String>, key: &str) -> Result<Option<String>, String> {
    match name {
        Some(s) if s.len() > 63 || s.contains('\0') => {
            Err(format!("{key} \"{s}\" is longer than 63 bytes"))
        }
        s => Ok(s.clone()),
    }
}

fn callbacks(
    start: &Option<String>,
    update: &Option<String>,
    complete: &Option<String>,
    repeat: &Option<String>,
) -> Result<Callbacks, String> {
    Ok(Callbacks {
        start: name_of(start, "onStart")?,
        update: name_of(update, "onUpdate")?,
        complete: name_of(complete, "onComplete")?,
        repeat: name_of(repeat, "onRepeat")?,
    })
}

fn seconds(s: f32, key: &str) -> Result<f64, String> {
    if s.is_finite() && s >= 0.0 {
        Ok(f64::from(s))
    } else {
        Err(format!("{key} is seconds, 0 or more"))
    }
}

fn repeat_of(n: i32) -> Result<i32, String> {
    if n >= -1 {
        Ok(n)
    } else {
        Err("repeat is -1 (forever), 0 (once) or more".into())
    }
}

pub fn og_tween_timeline(world: u32, desc: Option<String>) -> u32 {
    let made = (|| -> Result<(u32, Option<String>), String> {
        let eng = engine(world)?;
        let raw = parse(desc.as_deref())?;
        let (d, read) = match crate::wire::value::<TimelineDesc>(&raw, "desc") {
            Ok(d) => (d, Ok(())),
            Err(e) => (TimelineDesc::default(), Err(e)),
        };
        let mut tl = Timeline::default();
        // Refused settings leave their defaults and are the timeline's error.
        let settings = read.and_then(|()| {
            tl.repeat = repeat_of(d.repeat)?;
            tl.yoyo = d.yoyo;
            tl.repeat_delay = seconds(d.repeat_delay, "repeatDelay")? as f32;
            tl.time_scale = seconds(d.time_scale, "timeScale")? as f32;
            tl.paused = d.paused;
            tl.auto_kill = d.auto_kill;
            tl.on = callbacks(&d.on_start, &d.on_update, &d.on_complete, &d.on_repeat)?;
            ease_of(&d.defaults.ease, Ease::QuadOut)?;
            seconds(d.defaults.duration, "duration")?;
            Ok(())
        });
        let tween = feel_of(&eng)?
            .tweens
            .start(Node::Timeline(Box::new(tl)), d.unscaled, d.tag)
            .map_err(err)?;
        let defaults = if settings.is_ok() {
            d.defaults
        } else {
            TweenDefaults::default()
        };
        let h = HANDLES.with(|t| {
            t.borrow_mut().timelines.push(TimelineRef {
                world,
                tween,
                defaults,
                error: None,
            })
        });
        Ok((h, settings.err()))
    })();
    match made {
        Ok((h, error)) => {
            with_ref(h, |r| r.error = error);
            h
        }
        Err(_) => 0,
    }
}

pub fn og_tween_timeline_get_tween(h: u32) -> f64 {
    with_ref(h, |r| r.tween as f64).unwrap_or(0.0)
}

pub fn og_tween_timeline_get_error(h: u32) -> String {
    with_ref(h, |r| json!(r.error).to_string())
        .unwrap_or_else(|| json!("not a timeline").to_string())
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    To,
    From,
    FromTo,
    Set,
}

fn entities(text: &str) -> Result<Vec<u64>, String> {
    let targets = match crate::wire::arg::<Option<OneOf<u64, Vec<u64>>>>(Some(text), "targets")? {
        None => return Err("targets is an entity or a list of entities".into()),
        Some(OneOf::A(e)) => vec![e],
        Some(OneOf::B(list)) => list,
    };
    if let Some(e) = targets.iter().find(|e| **e == 0) {
        return Err(format!("{e} is not an entity"));
    }
    Ok(targets)
}

fn stagger_of(v: &Option<OneOf<f32, StaggerDesc>>) -> Result<Option<Stagger>, String> {
    let s = match v {
        None => return Ok(None),
        Some(OneOf::A(each)) if each.is_finite() => {
            return Ok(Some(Stagger::each(f64::from(*each))));
        }
        Some(OneOf::A(_)) => return Err("stagger is seconds".into()),
        Some(OneOf::B(s)) => s,
    };
    let num = |n: Option<f32>, k: &str| -> Result<Option<f64>, String> {
        match n {
            None => Ok(None),
            Some(x) if x.is_finite() => Ok(Some(f64::from(x))),
            Some(_) => Err(format!("stagger.{k} is seconds")),
        }
    };
    let from = match &s.from {
        None => StaggerFrom::Start,
        Some(OneOf::B(i)) => StaggerFrom::Index(*i),
        Some(OneOf::A(w)) => match w.as_str() {
            "start" => StaggerFrom::Start,
            "center" => StaggerFrom::Center,
            "end" => StaggerFrom::End,
            "edges" => StaggerFrom::Edges,
            "random" => StaggerFrom::Random,
            other => {
                return Err(format!(
                    "stagger.from \"{other}\" is start, center, end, edges, random or an index"
                ));
            }
        },
    };
    let grid = match s.grid.as_deref() {
        None => None,
        Some(&[rows, columns]) if rows >= 1 && columns >= 1 => Some([rows, columns]),
        Some(_) => return Err(
            "stagger.grid is [rows, columns] (\"auto\" needs a layout the engine does not have)"
                .into(),
        ),
    };
    Ok(Some(Stagger {
        each: num(s.each, "each")?,
        amount: num(s.amount, "amount")?,
        from,
        grid,
        ease: ease_of(&s.ease, Ease::Linear)?,
    }))
}

/// The children one tween call adds (one per target, offset by its
/// stagger), at time 0 of the call.
fn tweens(
    eng: &Engine,
    world: u32,
    tween: u64,
    targets: &[u64],
    v: TimelineTweenVars,
    defaults: &TweenDefaults,
    mode: Mode,
) -> Result<Vec<Child>, String> {
    if targets.is_empty() {
        return Err("targets is empty".into());
    }
    let component = v
        .component
        .as_ref()
        .or(defaults.component.as_ref())
        .map_or(Some("og.Transform".to_string()), token_name)
        .ok_or("vars.component is a component token, such as og.Sprite")?;
    let (index, schema) = component_schema(eng, &component)?;
    if v.field.is_empty() {
        return Err("vars.field names the field, such as `position.y`".into());
    }
    let field = v.field.as_str();
    let duration = if mode == Mode::Set {
        0.0
    } else {
        let default = seconds(defaults.duration, "duration")?;
        v.duration.map_or(Ok(default), |s| seconds(s, "duration"))?
    };
    let ease = match &v.ease {
        None => ease_of(&defaults.ease, Ease::QuadOut)?,
        Some(e) => ease_of(e, Ease::QuadOut)?,
    };
    let delay = seconds(v.delay, "delay")?;
    let repeat = repeat_of(v.repeat)?;
    let yoyo = v.yoyo;
    let repeat_delay = seconds(v.repeat_delay, "repeatDelay")? as f32;
    let calls = callbacks(&v.on_start, &v.on_update, &v.on_complete, &v.on_repeat)?;
    let (to, from) = (lanes(v.to.clone()), lanes(v.from.clone()));
    let keyframes: Option<&[TweenKeyframe]> = v.keyframes.as_deref();
    match mode {
        Mode::To if keyframes.is_none() && to.is_none() => {
            return Err("to needs vars.to or vars.keyframes".into());
        }
        Mode::Set if to.is_none() => return Err("set needs vars.to".into()),
        Mode::From if from.is_none() => return Err("from needs vars.from".into()),
        Mode::FromTo if from.is_none() || to.is_none() => {
            return Err("fromTo needs vars.from and vars.to".into());
        }
        _ => {}
    }
    if keyframes.is_some() && mode != Mode::To {
        return Err("keyframes go with to".into());
    }
    let stagger = stagger_of(&v.stagger)?;
    let offsets = match &stagger {
        None => vec![0.0; targets.len()],
        Some(s) => {
            let label = format!("og.tween.stagger {tween} {}", targets.len());
            let mut shuffle = |n: usize| -> Vec<usize> {
                crate::api::read_rng(world, |r| {
                    let mut r = r.fork(&label);
                    og_core::procgen::permutation(&mut r, n as u32)
                })
                .map(|p| p.into_iter().map(|i| i as usize).collect())
                .unwrap_or_else(|| (0..n).collect())
            };
            s.offsets(targets.len(), &mut shuffle)
        }
    };
    let mut out = Vec::new();
    for (&entity, offset) in targets.iter().zip(offsets) {
        if !has(eng, entity, index)? {
            return Err(format!(
                "entity {entity} has no {component} (or is not alive)"
            ));
        }
        let target = Target::resolve(&schema, index, entity, field).map_err(err)?;
        let leaf = |to: Vec<f32>, duration: f64, ease: Ease| {
            Leaf::new(target, to)
                .duration(duration as f32)
                .ease(ease)
                .relative(v.relative)
        };
        let child = if let Some(frames) = keyframes {
            // Keyframes are a nested timeline: repeat and yoyo apply to
            // the whole, each step from where the one before ended.
            let mut nested = Timeline::default();
            nested.repeat = repeat;
            nested.yoyo = yoyo;
            nested.repeat_delay = repeat_delay;
            nested.on = calls.clone();
            let mut at = 0.0;
            for (i, k) in frames.iter().enumerate() {
                let to = lanes(k.to.clone())
                    .ok_or_else(|| required(&format!("vars.keyframes[{i}].to")))?;
                at += seconds(k.delay, "delay")?;
                let d = k
                    .duration
                    .map_or(Ok(duration), |s| seconds(s, "duration"))?;
                let e = match &k.ease {
                    None => ease,
                    Some(n) => ease_of(n, ease)?,
                };
                let l = leaf(to, d, e);
                l.check().map_err(err)?;
                nested.children.push(Child::new(at, Item::Tween(l)));
                at += d;
            }
            if nested.children.is_empty() {
                return Err("keyframes is empty".into());
            }
            Child::new(offset + delay, Item::Timeline(Box::new(nested)))
        } else {
            let mut l = leaf(to.clone().unwrap_or_default(), duration, ease)
                .loops(repeat, yoyo)
                .repeat_delay(repeat_delay);
            if mode == Mode::From {
                // The destination is the value now (GSAP reads it at the call).
                let e = Entity::from_bits(entity);
                let id = SchemaId::from_index(index);
                let bytes = with_world(eng, |c| c.get_bytes(e, id), |w| w.get_bytes(e, id))?;
                l.to = target
                    .read(&bytes)
                    .ok_or("cannot read the tween's target")?;
                l.relative = false;
            }
            if matches!(mode, Mode::From | Mode::FromTo) {
                l.from = from.clone();
            }
            l.check().map_err(err)?;
            let mut c = Child::new(offset + delay, Item::Tween(l));
            c.on = calls.clone();
            c.immediate = matches!(mode, Mode::From | Mode::FromTo);
            c
        };
        out.push(child);
    }
    Ok(out)
}

/// Places `children` (at time 0 of what they add) at `position`.
fn place(
    eng: &Engine,
    tween: u64,
    mut children: Vec<Child>,
    position: Option<Position>,
) -> Result<(), String> {
    let inserting = children
        .iter()
        .map(|c| c.start + c.length())
        .fold(0.0, f64::max);
    on_timeline(eng, tween, |tl| {
        let at = tl.resolve(position.as_ref(), inserting)?;
        for c in &mut children {
            c.start += at;
        }
        tl.insert(children);
        Ok(())
    })?
}

fn add_tweens(
    h: u32,
    targets: String,
    vars: Option<String>,
    pos: Option<String>,
    mode: Mode,
) -> u32 {
    let world = with_ref(h, |r| r.world).unwrap_or(0);
    build(h, |eng, tween, defaults| {
        let v: TimelineTweenVars = crate::wire::arg(vars.as_deref(), "vars")?;
        let targets = entities(&targets)?;
        let position = position(pos.as_deref())?;
        let children = tweens(eng, world, tween, &targets, v, defaults, mode)?;
        place(eng, tween, children, position)
    })
}

pub fn og_tween_timeline_to(
    h: u32,
    targets: String,
    vars: Option<String>,
    position: Option<String>,
) -> u32 {
    add_tweens(h, targets, vars, position, Mode::To)
}

pub fn og_tween_timeline_from(
    h: u32,
    targets: String,
    vars: Option<String>,
    position: Option<String>,
) -> u32 {
    add_tweens(h, targets, vars, position, Mode::From)
}

pub fn og_tween_timeline_from_to(
    h: u32,
    targets: String,
    vars: Option<String>,
    position: Option<String>,
) -> u32 {
    add_tweens(h, targets, vars, position, Mode::FromTo)
}

pub fn og_tween_timeline_set(
    h: u32,
    targets: String,
    vars: Option<String>,
    position: Option<String>,
) -> u32 {
    add_tweens(h, targets, vars, position, Mode::Set)
}

pub fn og_tween_timeline_call(h: u32, name: String, pos: Option<String>) -> u32 {
    build(h, |eng, tween, _| {
        let name = name_of(&Some(name), "call")?.unwrap_or_default();
        place(
            eng,
            tween,
            vec![Child::new(0.0, Item::Call(name))],
            position(pos.as_deref())?,
        )
    })
}

pub fn og_tween_timeline_add_label(h: u32, label: String, pos: Option<String>) -> u32 {
    build(h, |eng, tween, _| {
        if label.is_empty() {
            return Err("a label has a name".into());
        }
        let position = position(pos.as_deref())?;
        on_timeline(eng, tween, |tl| {
            let at = tl.resolve(position.as_ref(), 0.0)?;
            tl.add_label(&label, at);
            Ok(())
        })?
    })
}

/// A running tween, group or timeline as a child of a timeline, at time
/// 0 of what it adds.
fn nest(node: Node) -> Child {
    let tween = |mut l: Leaf, at: f64| {
        let start = at + f64::from(l.delay);
        l.delay = 0.0;
        Child::new(start, Item::Tween(l))
    };
    match node {
        Node::Leaf(l) => tween(l, 0.0),
        Node::Group(g) => {
            let mut nested = Timeline::default();
            nested.repeat = g.loops;
            let mut at = 0.0;
            for l in g.children {
                let length = f64::from(l.length());
                nested.children.push(tween(l, at));
                if g.kind == GroupKind::Sequence {
                    at += length;
                }
            }
            Child::new(0.0, Item::Timeline(Box::new(nested)))
        }
        Node::Timeline(mut t) => {
            t.rewind();
            Child::new(0.0, Item::Timeline(t))
        }
    }
}

pub fn og_tween_timeline_add(h: u32, child: String, pos: Option<String>) -> u32 {
    build(h, |eng, tween, _| {
        let position = position(pos.as_deref())?;
        match crate::wire::arg::<Option<OneOf<u64, String>>>(Some(&child), "child")? {
            None => Err("add takes a tween handle or a label".into()),
            Some(OneOf::B(label)) => on_timeline(eng, tween, |tl| {
                let at = tl.resolve(position.as_ref(), 0.0)?;
                tl.add_label(&label, at);
                Ok(())
            })?,
            Some(OneOf::A(handle)) => {
                if handle == tween {
                    return Err("a timeline cannot be added to itself".into());
                }
                let node = feel_of(eng)?
                    .tweens
                    .take(handle)
                    .ok_or_else(|| format!("{handle} is not a running tween"))?;
                place(eng, tween, vec![nest(node)], position)
            }
        }
    })
}

/// The time `pos` names for a seek: seconds, a label or `label+=n`.
fn seek_time(tl: &Timeline, pos: &Position) -> Result<f64, String> {
    match pos {
        Position::At(t) if t.is_finite() => Ok(*t),
        Position::At(t) => Err(format!("{t} is not a time")),
        Position::Text(s) => {
            let s = s.trim();
            if let Ok(t) = s.parse::<f64>() {
                return Ok(t);
            }
            let (name, rest) = match s.find("+=").or_else(|| s.find("-=")) {
                Some(i) => (s[..i].trim(), &s[i..]),
                None => (s, ""),
            };
            let base = tl
                .labels
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, t)| *t)
                .ok_or_else(|| format!("no label \"{name}\""))?;
            let off = if rest.is_empty() {
                0.0
            } else {
                rest[2..]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| format!("\"{s}\" is a label, label+=seconds or label-=seconds"))?
            };
            Ok(if rest.starts_with('-') {
                base - off
            } else {
                base + off
            })
        }
    }
}

/// A control: `f` on the timeline, after a seek to `pos` when given.
fn control(h: u32, pos: Option<String>, f: impl FnOnce(&mut Timeline)) -> u32 {
    build(h, |eng, tween, _| {
        let pos = position(pos.as_deref())?;
        on_timeline(eng, tween, |tl| {
            if let Some(p) = &pos {
                let t = seek_time(tl, p)?;
                tl.seek_total(t);
            }
            f(tl);
            Ok(())
        })?
    })
}

pub fn og_tween_timeline_play(h: u32, from: Option<String>) -> u32 {
    control(h, from, |tl| {
        tl.paused = false;
        tl.reversed = false;
    })
}

pub fn og_tween_timeline_pause(h: u32, at: Option<String>) -> u32 {
    control(h, at, |tl| tl.paused = true)
}

pub fn og_tween_timeline_resume(h: u32) -> u32 {
    control(h, None, |tl| tl.paused = false)
}

pub fn og_tween_timeline_reverse(h: u32, from: Option<String>) -> u32 {
    control(h, from, |tl| {
        tl.paused = false;
        tl.reversed = true;
    })
}

pub fn og_tween_timeline_restart(h: u32) -> u32 {
    // Back to the start without callbacks (what it wrote returns to its
    // start values), then forwards: its callbacks fire again.
    control(h, None, |tl| {
        tl.seek_total(0.0);
        tl.paused = false;
        tl.reversed = false;
    })
}

pub fn og_tween_timeline_seek(h: u32, position: String) -> u32 {
    control(h, Some(position), |_| {})
}

/// A reading of the timeline (`refused` when it is gone).
fn read<T>(h: u32, refused: T, f: impl FnOnce(&mut Timeline) -> T) -> T {
    target(h)
        .and_then(|(eng, tween, _)| on_timeline(&eng, tween, f))
        .unwrap_or(refused)
}

pub fn og_tween_timeline_progress(h: u32, value: Option<f64>) -> f64 {
    read(h, 0.0, |tl| {
        if let Some(p) = value.filter(|p| p.is_finite()) {
            tl.set_progress(p);
        }
        tl.progress()
    })
}

pub fn og_tween_timeline_time(h: u32) -> f64 {
    read(h, 0.0, |tl| tl.time())
}

pub fn og_tween_timeline_duration(h: u32) -> f64 {
    read(h, 0.0, |tl| tl.duration())
}

pub fn og_tween_timeline_time_scale(h: u32, value: Option<f64>) -> f64 {
    read(h, 0.0, |tl| {
        if let Some(s) = value.filter(|s| s.is_finite() && *s >= 0.0) {
            tl.time_scale = s as f32;
        }
        f64::from(tl.time_scale)
    })
}

pub fn og_tween_timeline_is_active(h: u32) -> bool {
    read(h, false, |tl| tl.is_active())
}

pub fn og_tween_timeline_kill(h: u32) -> bool {
    target(h).is_ok_and(|(eng, tween, _)| feel_of(&eng).is_ok_and(|f| f.tweens.cancel(tween)))
}
