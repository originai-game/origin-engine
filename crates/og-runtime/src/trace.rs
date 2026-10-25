//! The run's trace (`og run --trace`, the web shell's trace,
//! `docs/design.md` §3.4): spans the engine records while tracing is on,
//! each where it ran, as Chrome trace events (the Trace Event Format's
//! `X` and `M` events). Measurement only: nothing in the tick reads it.
//!
//! Every span carries its own start, read from the clock it was timed
//! with: each tick (from before its first system to after its last) with
//! its systems inside it (the world's clock, `SystemInfo::start`/`ms`);
//! each rendered frame with its render-graph passes' CPU encoding
//! (og-render's `pass_timings`); each frame's passes on the GPU (timestamp
//! queries, og-render's `gpu_timings`), anchored at the frame's submit
//! since the GPU's clock is its own, and after the frame before it, since
//! the GPU runs them in order; each cooked output read by a load; and
//! every job og-lanes ran on a pool thread, on that thread's own track.
//!
//! Thread ids: 1 engine, 2 render, 3 load, 5 gpu, [`LANE_THREADS`] + n for
//! pool thread n (and [`LANE_THREADS`] − 1 for a job off the pool); og
//! adds 4 cook and its cook threads (og-cli's `trace.rs`). `ts` and `dur`
//! are whole microseconds on the engine's clock (`now_ms`), each span's
//! ends rounded alike so a span inside another stays inside it; the host
//! moves them onto its own origin with the clock's reading [`take`]
//! returns.

use og_core::IdMap;
use og_ecs::World;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;

/// The first thread id of og-lanes' pool threads.
pub const LANE_THREADS: u32 = 16;

const ENGINE: u32 = 1;
const RENDER: u32 = 2;
const LOAD: u32 = 3;
const GPU: u32 = 5;

#[derive(Default)]
struct Trace {
    tick_start: f64,
    events: Vec<Value>,
    /// Where the GPU finished the last frame it timed (engine clock).
    gpu_end: f64,
    /// Lane threads already named in this trace.
    named: BTreeSet<u32>,
}

thread_local! {
    /// Per engine, while tracing is on.
    static TRACES: RefCell<IdMap<u32, Trace>> = const { RefCell::new(IdMap::new()) };
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

/// A complete span (`ph: X`) from `start` for `ms` (engine clock, ms).
fn span(name: &str, cat: &str, tid: u32, start: f64, ms: f64, args: Value) -> Value {
    span_to(name, cat, tid, start, start + ms.max(0.0), args)
}

/// A complete span from `start` to `end` (engine clock, ms). Spans that
/// share an instant pass the same value here: one rounding, one
/// microsecond, so a child's end never lands past its parent's.
fn span_to(name: &str, cat: &str, tid: u32, start: f64, end: f64, args: Value) -> Value {
    let ts = (start * 1000.0).round() as i64;
    let end = (end.max(start) * 1000.0).round() as i64;
    json!({ "name": name, "cat": cat, "ph": "X", "pid": 1, "tid": tid, "ts": ts, "dur": end - ts, "args": args })
}

/// A thread's name and its place among the trace's threads.
fn thread_named(tid: u32, name: &str) -> [Value; 2] {
    [
        json!({ "name": "thread_name", "ph": "M", "pid": 1, "tid": tid, "args": { "name": name } }),
        json!({ "name": "thread_sort_index", "ph": "M", "pid": 1, "tid": tid, "args": { "sort_index": tid } }),
    ]
}

/// Turns tracing on or off for an engine; off drops what was not taken.
/// [`BEFORE_ENGINE`] records loads before the engine exists (its boot
/// scene's), for the first engine that takes them. og-lanes times its
/// jobs while any engine traces.
pub fn set(engine: u32, on: bool) {
    let any = TRACES.with(|t| {
        let mut t = t.borrow_mut();
        if on {
            t.entry(engine).or_default();
        } else {
            t.remove(&engine);
        }
        !t.is_empty()
    });
    og_lanes::set_trace(any);
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
        let ran = tick.saturating_sub(1);
        tr.events.push(span(
            &format!("tick {tick}"),
            "tick",
            ENGINE,
            tr.tick_start,
            end - tr.tick_start,
            json!({ "tick": tick }),
        ));
        // The systems of the tick that just ran (og.Time has advanced), as
        // they were timed: each from its own start.
        for s in world.system_info() {
            if s.last_tick != Some(ran) {
                continue;
            }
            if let (Some(start), Some(ms)) = (s.start, s.ms)
                && start >= tr.tick_start
            {
                tr.events.push(span(
                    &s.name,
                    "system",
                    ENGINE,
                    start,
                    ms,
                    json!({ "tick": tick }),
                ));
            }
        }
    });
}

/// A rendered frame: its start, length and passes in execution order,
/// each with its own start.
pub(crate) fn frame(engine: u32, tick: u64, start: f64, ms: f64, passes: &[og_render::PassTiming]) {
    if !on() {
        return;
    }
    TRACES.with(|t| {
        if let Some(tr) = t.borrow_mut().get_mut(&engine) {
            let args = json!({ "tick": tick });
            tr.events.push(span(
                &format!("frame {tick}"),
                "render",
                RENDER,
                start,
                ms,
                args.clone(),
            ));
            for p in passes {
                tr.events
                    .push(span(&p.name, "pass", RENDER, p.start, p.ms, args.clone()));
            }
        }
    });
}

/// Frames timed on the GPU (og-render's `gpu_timings`), oldest first.
pub(crate) fn gpu_frames(engine: u32, frames: &[og_render::GpuFrame]) {
    if !on() || frames.is_empty() {
        return;
    }
    TRACES.with(|t| {
        if let Some(tr) = t.borrow_mut().get_mut(&engine) {
            for f in frames {
                // The GPU starts a frame once it is submitted and the one
                // before it is done.
                let at = f.submitted.max(tr.gpu_end);
                let args = json!({ "frame": f.frame, "submitted": f.submitted });
                // The frame ends where its last pass does, and each pass
                // starts where the one before it ended: the same sums,
                // so the rounding agrees.
                let end = at + f.ms();
                tr.events.push(span_to(
                    &format!("gpu frame {}", f.frame),
                    "gpu",
                    GPU,
                    at,
                    end,
                    args.clone(),
                ));
                for p in &f.passes {
                    tr.events.push(span_to(
                        &p.name,
                        "gpu",
                        GPU,
                        at + p.start,
                        at + p.end,
                        args.clone(),
                    ));
                }
                tr.gpu_end = end;
            }
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
        l.borrow_mut()
            .push(span(id, "load", LOAD, start, ms, json!({ "bytes": bytes })))
    });
}

/// The start of a span when tracing (0 otherwise, never read).
pub(crate) fn start() -> f64 {
    if on() { now() } else { 0.0 }
}

/// og-lanes' jobs since the last take, moved onto the engine's clock.
fn lanes(tr: &mut Trace) -> Vec<Value> {
    let taken = og_lanes::take_trace();
    // The lanes' clock against the engine's, read together now.
    let shift = now() - taken.now;
    let mut out = Vec::new();
    for j in taken.jobs {
        let (tid, name) = match j.thread {
            u32::MAX => (LANE_THREADS - 1, "lane (caller)".to_string()),
            n => (LANE_THREADS + n, format!("lane {n}")),
        };
        if tr.named.insert(tid) {
            out.extend(thread_named(tid, &name));
        }
        let site = j.site.map_or("lane", og_lanes::Site::name);
        out.push(span(
            site,
            "lane",
            tid,
            j.start + shift,
            j.ms,
            json!({ "items": j.items }),
        ));
    }
    if taken.dropped > 0 {
        // A process-scoped instant event: the trace says what it lacks.
        out.push(json!({ "name": "lane jobs not kept", "ph": "i", "s": "p", "pid": 1, "tid": LANE_THREADS - 1,
            "ts": (now() * 1000.0).round() as i64, "args": { "dropped": taken.dropped } }));
    }
    out
}

/// The spans recorded since the last take as Chrome trace events (their
/// `ts` on the engine's clock, in µs), with the clock's reading now
/// (`now`, ms), so a host places them on its own origin: it adds
/// `(its time now − now) × 1000` to every `ts`. With `last`, the GPU's
/// frames still in flight are waited for first (natively), so the last
/// take misses none; otherwise they arrive in a later take.
pub fn take(engine: u32, last: bool) -> String {
    if last && let Ok(eng) = super::engine::engine(engine) {
        let frames = eng
            .render
            .borrow_mut()
            .backend
            .as_mut()
            .map(|b| b.gpu_timings(true))
            .unwrap_or_default();
        gpu_frames(engine, &frames);
    }
    let mut events = TRACES.with(|t| {
        let mut t = t.borrow_mut();
        let Some(tr) = t.get_mut(&engine) else {
            return Vec::new();
        };
        let mut events = std::mem::take(&mut tr.events);
        events.extend(lanes(tr));
        events
    });
    events.extend(LOADS.with(|l| std::mem::take(&mut *l.borrow_mut())));
    json!({ "now": now(), "events": events }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gpu_pass_ends_on_the_microsecond_its_frame_does() {
        // The frame's end is the last pass's end, in one sum: a pass end
        // reached by a second sum ((at + start) + ms) lands past the
        // frame's by a microsecond on these values, and the trace check
        // reports a child overlapping its parent.
        let at = 1948.0215173274164;
        let frame = og_render::GpuFrame {
            frame: 7,
            submitted: at,
            passes: vec![
                og_render::GpuPass {
                    name: "main".into(),
                    start: 0.0,
                    end: 1.5297082862458637,
                },
                og_render::GpuPass {
                    name: "tonemap".into(),
                    start: 1.5297082862458637,
                    end: 1.9769826725835173,
                },
            ],
        };
        const ENGINE_ID: u32 = 4242;
        TRACES.with(|t| {
            t.borrow_mut().insert(ENGINE_ID, Trace::default());
        });
        gpu_frames(ENGINE_ID, &[frame]);
        let events = TRACES
            .with(|t| std::mem::take(&mut t.borrow_mut().get_mut(&ENGINE_ID).unwrap().events));
        let ends: Vec<(String, i64, i64)> = events
            .iter()
            .map(|e| {
                let ts = e["ts"].as_i64().unwrap();
                (
                    e["name"].as_str().unwrap().to_string(),
                    ts,
                    ts + e["dur"].as_i64().unwrap(),
                )
            })
            .collect();
        assert_eq!(ends[0].0, "gpu frame 7");
        assert_eq!(ends[1].2, ends[2].1, "tonemap starts where main ends");
        assert_eq!(ends[2].2, ends[0].2, "tonemap ends where the frame does");
        assert_eq!(ends[0].2, 1949999);
    }
}
