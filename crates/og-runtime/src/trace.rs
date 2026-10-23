//! The run's trace (`og run --trace`, `docs/design.md` §3.4): spans the
//! engine records while tracing is on — each tick with its systems (from
//! the world's clock, `SystemInfo::ms`, laid end to end in run order from
//! the tick's start), each rendered frame with its render-graph passes
//! (og-render's `pass_timings`, likewise), and each cooked output read by a
//! load. Measurement only: nothing in the tick reads it. The host takes the
//! spans as JSON ([`take`]) and writes the Chrome trace format.

use og_ecs::World;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Default)]
struct Trace {
    tick_start: f64,
    events: Vec<Value>,
}

thread_local! {
    /// Per engine, while tracing is on.
    static TRACES: RefCell<BTreeMap<u32, Trace>> = const { RefCell::new(BTreeMap::new()) };
    /// Loads, for whichever engine takes them first (the assets layer reads
    /// without knowing the engine).
    static LOADS: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

/// The key that traces loads before an engine is made (engine ids start at 0).
pub const BEFORE_ENGINE: u32 = u32::MAX;

fn on() -> bool {
    TRACES.with(|t| !t.borrow().is_empty())
}

fn now() -> f64 {
    super::probe::now_ms()
}

/// Turns tracing on or off for an engine; off drops what was not taken.
/// [`BEFORE_ENGINE`] records loads before the engine exists (its boot
/// scene's), for the first engine that takes them.
pub fn set(engine: u32, on: bool) {
    TRACES.with(|t| {
        let mut t = t.borrow_mut();
        if on {
            t.entry(engine).or_default();
        } else {
            t.remove(&engine);
        }
    });
}

pub(crate) fn before_tick(engine: u32) {
    if !on() {
        return;
    }
    let at = now();
    TRACES.with(|t| {
        if let Some(tr) = t.borrow_mut().get_mut(&engine) {
            tr.tick_start = at;
        }
    });
}

pub(crate) fn after_tick(engine: u32, world: &World) {
    if !on() {
        return;
    }
    let end = now();
    let tick = world.time().tick;
    TRACES.with(|t| {
        let mut t = t.borrow_mut();
        let Some(tr) = t.get_mut(&engine) else {
            return;
        };
        let systems: Vec<Value> = world
            .system_info()
            .into_iter()
            .filter(|s| s.last_tick == Some(tick.saturating_sub(1)) || s.last_tick == Some(tick))
            .filter_map(|s| s.ms.map(|ms| json!({ "name": s.name, "ms": ms })))
            .collect();
        tr.events.push(json!({
            "kind": "tick",
            "name": format!("tick {tick}"),
            "tick": tick,
            "start": tr.tick_start,
            "ms": (end - tr.tick_start).max(0.0),
            "children": systems,
        }));
    });
}

/// A rendered frame: its start, length and passes in execution order.
pub(crate) fn frame(engine: u32, tick: u64, start: f64, ms: f64, passes: &[(String, f64)]) {
    if !on() {
        return;
    }
    TRACES.with(|t| {
        if let Some(tr) = t.borrow_mut().get_mut(&engine) {
            tr.events.push(json!({
                "kind": "render",
                "name": format!("frame {tick}"),
                "tick": tick,
                "start": start,
                "ms": ms,
                "children": passes.iter().map(|(n, ms)| json!({ "name": n, "ms": ms })).collect::<Vec<_>>(),
            }));
        }
    });
}

/// A cooked output read for a load, `bytes` long.
pub(crate) fn load(id: &str, start: f64, bytes: usize) {
    if !on() {
        return;
    }
    let ms = (now() - start).max(0.0);
    LOADS.with(|l| {
        l.borrow_mut().push(json!({
            "kind": "load", "name": id, "start": start, "ms": ms, "bytes": bytes,
        }))
    });
}

/// The start of a span when tracing (0 otherwise, never read).
pub(crate) fn start() -> f64 {
    if on() { now() } else { 0.0 }
}

/// The spans recorded since the last take, with the clock's reading now
/// (`now`, the same clock as every `start`, so a host can place them on
/// its own).
pub fn take(engine: u32) -> String {
    let mut events = TRACES.with(|t| {
        t.borrow_mut()
            .get_mut(&engine)
            .map(|tr| std::mem::take(&mut tr.events))
            .unwrap_or_default()
    });
    events.extend(LOADS.with(|l| std::mem::take(&mut *l.borrow_mut())));
    json!({ "now": now(), "events": events }).to_string()
}
