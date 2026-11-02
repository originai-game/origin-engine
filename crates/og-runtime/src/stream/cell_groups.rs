//! Cell asset groups and the hold gate (G3-3).
//!
//! A sub-scene that declares `stream "<name>" { cell; asset ... }` has its
//! own blob in that group's pack, beside the assets it names. Its handle
//! exists from the start (the placing scene's cell names it) as
//! [`Payload::SceneWaiting`], and becomes the decoded scene once the pack's
//! bytes are in ([`resolve_scenes`]); when the group unloads it waits again
//! ([`forget_scenes`]).
//!
//! When the pack is read is presentation: at every tick boundary a cell
//! group is wanted while one of its cells is loaded or loading, pinned, or
//! within its load distance of an anchor [`PREFETCH_SECONDS`] further along
//! at the anchor's `maxSpeed` ([`super::cells::wanted_groups`]).
//!
//! What a tick does is not: a tick that would load a cell from a sub-scene
//! whose pack has not landed is held by [`gate`] (chained before og.net's
//! in the probe), never skipped or reordered. The gate first reads the
//! group now (natively a read completes at once, so a native run or replay
//! never holds); a host that answers `Pending` (the web's fetch) holds the
//! tick, frames go on drawing, and the next frame's gate tries again. Each
//! hold is logged with how long it lasted, and `dump_stream`'s `hold` and
//! `holds` say what the tick waits for and what every hold took.

use super::super::assets::{Payload, Status, decode_scene};
use super::super::engine::{Engine, engine};
use super::{Streams, cells};
use og_ecs::World;
use serde_json::{Value, json};

/// How far ahead, in seconds of an anchor's `maxSpeed`, a cell group is
/// fetched: a pack of a few megabytes lands in that on a normal connection.
pub(crate) const PREFETCH_SECONDS: f32 = 2.0;

/// The holds `dump_stream` lists.
const MAX_HOLDS: usize = 64;

/// A tick held for a cell group's pack.
#[derive(Clone, Debug)]
pub(crate) struct Hold {
    tick: u64,
    cell: u64,
    group: String,
    /// The host's clock when it started (presentation, never a tick's).
    since_ms: f64,
}

/// The holds so far, for the probe.
#[derive(Default)]
pub(crate) struct Holds {
    /// The tick held now.
    now: Option<Hold>,
    /// The recent ones, each with how long it took.
    recent: Vec<Value>,
    count: u64,
    total_ms: f64,
    longest_ms: f64,
    /// The group the gate reads now, inside one call.
    reading: Option<String>,
}

impl Streams {
    /// Remembers a cell group's sub-scene handle (its load index).
    pub(crate) fn add_scene(&mut self, group: &str, index: usize) {
        let list = self.scenes.entry(group.to_string()).or_default();
        if !list.contains(&index) {
            list.push(index);
        }
    }

    /// The cell group whose pack holds the scene at load index `index`.
    pub(crate) fn group_of_scene(&self, index: usize) -> Option<String> {
        self.scenes
            .iter()
            .find(|(_, list)| list.contains(&index))
            .map(|(g, _)| g.clone())
    }

    /// The group a held tick waits for: wanted, and admitted first.
    pub(super) fn held_group(&self) -> Option<&str> {
        self.holds
            .reading
            .as_deref()
            .or(self.holds.now.as_ref().map(|h| h.group.as_str()))
    }
}

/// Decodes the waiting sub-scenes of `group` (every group: `None`) whose
/// pack has landed.
pub(crate) fn resolve_scenes(eng: &Engine, group: Option<&str>) {
    let waiting: Vec<(usize, String)> = {
        let a = eng.assets.borrow();
        a.stream
            .scenes
            .iter()
            .filter(|(g, _)| group.is_none_or(|x| x == g.as_str()))
            .filter(|(g, _)| {
                a.stream
                    .find(g)
                    .is_some_and(|i| !a.stream.groups[i].files.is_empty())
            })
            .flat_map(|(_, list)| list.iter().copied())
            .filter(|&i| matches!(a.entries[i].payload, Payload::SceneWaiting { .. }))
            .map(|i| (i, a.entries[i].path.clone()))
            .collect()
    };
    for (i, id) in waiting {
        let decoded = decode_scene(&eng.assets.borrow(), &id);
        let mut a = eng.assets.borrow_mut();
        match decoded {
            Ok(scene) => {
                a.entries[i].payload = Payload::Scene(Box::new(scene));
                a.entries[i].status = Status::Ready;
            }
            // Tried again once the group loads again; a cell of it warns
            // and stays unloaded meanwhile, never holding the tick.
            Err(e) => {
                a.entries[i].payload = Payload::None;
                a.entries[i].status = Status::Failed(e);
            }
        }
    }
}

/// `group` unloaded: its sub-scenes wait for its pack again.
pub(super) fn forget_scenes(eng: &Engine, group: &str) {
    let mut a = eng.assets.borrow_mut();
    let list = a.stream.scenes.get(group).cloned().unwrap_or_default();
    for i in list {
        a.entries[i].payload = Payload::SceneWaiting {
            group: group.to_string(),
        };
        a.entries[i].status = Status::Pending;
    }
}

/// The probe's tick gate, before og.net's: false holds the next tick
/// while it would load a cell whose sub-scene's pack has not landed.
pub(crate) fn gate(engine_id: u32, world: &mut World) -> bool {
    let Ok(eng) = engine(engine_id) else {
        return true;
    };
    if eng.assets.borrow().stream.scenes.is_empty() {
        return true;
    }
    let tick = world.time().tick;
    let mut tried: Vec<String> = Vec::new();
    loop {
        let Some((cell, group)) = cells::waiting(&eng, world) else {
            end_hold(&eng, tick);
            return true;
        };
        resolve_scenes(&eng, Some(&group));
        if !cells::waiting(&eng, world).is_some_and(|(c, g)| c == cell && g == group) {
            continue;
        }
        if tried.contains(&group) {
            start_hold(&eng, tick, cell.to_bits(), &group);
            return false;
        }
        // Wanted first, and read now: natively it lands at once.
        eng.assets.borrow_mut().stream.holds.reading = Some(group.clone());
        super::update(&eng);
        eng.assets.borrow_mut().stream.holds.reading = None;
        resolve_scenes(&eng, Some(&group));
        tried.push(group);
    }
}

/// The gate holds `tick` for `cell`'s `group` (the hold under way goes on
/// when it is the same tick's).
fn start_hold(eng: &Engine, tick: u64, cell: u64, group: &str) {
    let mut a = eng.assets.borrow_mut();
    let h = &mut a.stream.holds;
    let since_ms = h
        .now
        .as_ref()
        .filter(|n| n.tick == tick)
        .map_or_else(super::super::probe::now_ms, |n| n.since_ms);
    h.now = Some(Hold {
        tick,
        cell,
        group: group.to_string(),
        since_ms,
    });
}

/// The held tick goes on: its hold is logged with how long it took.
fn end_hold(eng: &Engine, tick: u64) {
    let hold = eng.assets.borrow_mut().stream.holds.now.take();
    let Some(hold) = hold else { return };
    let ms = (super::super::probe::now_ms() - hold.since_ms).max(0.0);
    let text = format!(
        "tick {} was held {ms:.0} ms for cell {}'s group '{}' (its sub-scene's pack)",
        hold.tick,
        og_ecs::Entity::from_bits(hold.cell),
        hold.group
    );
    {
        let mut a = eng.assets.borrow_mut();
        let h = &mut a.stream.holds;
        h.count += 1;
        h.total_ms += ms;
        h.longest_ms = h.longest_ms.max(ms);
        h.recent.push(json!({
            "tick": hold.tick as f64,
            "cell": hold.cell as f64,
            "group": hold.group,
            "ms": ms,
        }));
        if h.recent.len() > MAX_HOLDS {
            h.recent.remove(0);
        }
    }
    eng.state
        .borrow_mut()
        .log
        .push(json!({ "tick": tick, "warning": false, "text": text }));
}

/// `dump_stream`'s `hold` (the tick held now and what it waits for) and
/// `holds` (how many, their total and longest, and the recent ones).
pub(super) fn dump(s: &Streams, now_ms: f64) -> (Value, Value) {
    let h = &s.holds;
    let now = h.now.as_ref().map_or(Value::Null, |n| {
        let state = s
            .find(&n.group)
            .map_or("unknown", |i| s.groups[i].state.name());
        json!({
            "tick": n.tick as f64,
            "cell": n.cell as f64,
            "group": n.group,
            "state": state,
            "ms": (now_ms - n.since_ms).max(0.0),
        })
    });
    let holds = json!({
        "count": h.count as f64,
        "totalMs": h.total_ms,
        "longestMs": h.longest_ms,
        "recent": h.recent,
    });
    (now, holds)
}
