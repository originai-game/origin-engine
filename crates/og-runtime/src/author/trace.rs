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
//! queries, og-render's `gpu_timings`: each pass's own time), anchored at
//! the frame's submit since the GPU's clock is its own, and after the
//! frame before it, since the GPU runs them in order; each cooked output read by a load; and
//! every job og-lanes ran on a pool thread, on that thread's own track.
//!
//! Thread ids: 1 engine, 2 render, 3 load, 5 gpu (8 to 15 its passes that
//! overlap another, [`gpu_frames`]), [`LANE_THREADS`] + n for
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
const TEXTURES: u32 = 6;
/// The threads a GPU pass overlapping the gpu thread's goes on: from
/// [`GPU_LANE`], [`GPU_LANES`] of them (below og-lanes' pool threads).
const GPU_LANE: u32 = 8;
const GPU_LANES: u32 = 8;

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
    crate::author::probe::now_ms()
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
///
/// Each frame is a span on the gpu thread from its first timestamp to its
/// last, and each render-graph pass a span for every stretch the GPU ran
/// it: its own time, so passes the GPU ran at once overlap. A pass that
/// overlaps one already on the gpu thread goes on the next of the
/// [`GPU_LANES`] threads it fits on (a thread's spans never overlap). A
/// pass the GPU could not time is an instant at the frame's start with
/// why (`args.untimed`), never a span of 0. The frame's `args.passes`
/// holds every pass's own milliseconds (`null`: untimed).
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
                // A pass's end and the frame's are the same sum (at + the
                // same value): one rounding, so a pass never leaves it.
                let end = at + f.ms();
                let own: serde_json::Map<String, Value> = f
                    .passes
                    .iter()
                    .map(|p| (p.name.clone(), json!(p.ms())))
                    .collect();
                let mut frame_args = args.clone();
                frame_args["passes"] = Value::Object(own);
                tr.events.push(span_to(
                    &format!("gpu frame {}", f.frame),
                    "gpu",
                    GPU,
                    at,
                    end,
                    frame_args,
                ));
                let mut stretches: Vec<(&str, f64, f64)> = f
                    .passes
                    .iter()
                    .flat_map(|p| {
                        p.busy
                            .iter()
                            .map(|&(b, e)| (p.name.as_str(), at + b, at + e))
                    })
                    .collect();
                stretches.sort_by(|a, b| a.1.total_cmp(&b.1));
                // Each lane's last end (µs, as written).
                let mut lanes: Vec<i64> = Vec::new();
                for (name, b, e) in stretches {
                    let (from, to) = ((b * 1000.0).round() as i64, (e * 1000.0).round() as i64);
                    let lane = match lanes.iter().position(|&last| last <= from) {
                        Some(l) => l,
                        None if lanes.len() < 1 + GPU_LANES as usize => {
                            lanes.push(i64::MIN);
                            lanes.len() - 1
                        }
                        None => GPU_LANES as usize,
                    };
                    lanes[lane] = lanes[lane].max(to);
                    let tid = if lane == 0 {
                        GPU
                    } else {
                        GPU_LANE + lane as u32 - 1
                    };
                    if lane > 0 && tr.named.insert(tid) {
                        tr.events
                            .extend(thread_named(tid, &format!("gpu (overlapping) {lane}")));
                    }
                    tr.events
                        .push(span_to(name, "gpu", tid, b, e, args.clone()));
                }
                for p in &f.passes {
                    if let Some(why) = p.untimed {
                        let mut a = args.clone();
                        a["untimed"] = json!(why.reason());
                        tr.events
                            .push(json!({ "name": p.name, "cat": "gpu", "ph": "i", "s": "t",
                            "pid": 1, "tid": GPU, "ts": (at * 1000.0).round() as i64, "args": a }));
                    }
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

/// CPU work outside the ECS schedule, including polling/committing loads.
pub(crate) fn preparation(name: &str, start: f64) {
    if !on() || start == 0.0 {
        return;
    }
    LOADS.with(|l| {
        l.borrow_mut().push(span(
            name,
            "asset preparation",
            ENGINE,
            start,
            now() - start,
            json!({}),
        ))
    });
}

/// A completed native texture job, timed on its worker, then moved onto
/// the engine clock using the enqueue instant (no host call off-thread).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn texture_work(id: &str, start: f64, offset: f64, ms: f64) {
    if !on() || start == 0.0 {
        return;
    }
    LOADS.with(|l| {
        l.borrow_mut().push(span(
            id,
            "texture transcode",
            TEXTURES,
            start + offset,
            ms,
            json!({}),
        ))
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
    if last && let Ok(eng) = crate::engine::engine(engine) {
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
        if tr.named.insert(TEXTURES) {
            events.extend(thread_named(TEXTURES, "texture worker"));
        }
        events.extend(lanes(tr));
        events
    });
    events.extend(LOADS.with(|l| std::mem::take(&mut *l.borrow_mut())));
    json!({ "now": now(), "events": events }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The trace shows each GPU pass's own time: a cascade the GPU ran
    /// beside a longer cull keeps its 3 ms on a track of its own (laid
    /// back to back in queue order it read 0 and the cull took it), a pass
    /// that could not be timed says why instead of reading 0, and a pass
    /// ends on the very microsecond its frame does.
    #[test]
    fn gpu_passes_keep_their_own_time_where_they_overlap() {
        use og_render::{GpuFrame, GpuPass, GpuUntimed};
        let at = 1948.0215173274164;
        let pass = |name: &str, busy: Vec<(f64, f64)>| GpuPass {
            name: name.into(),
            busy,
            untimed: None,
        };
        let frame = GpuFrame {
            frame: 7,
            submitted: at,
            passes: vec![
                pass("cull", vec![(0.0, 8.0)]),
                pass("cascade", vec![(1.0, 4.0)]),
                pass("main", vec![(8.0, 9.9784)]),
                GpuPass {
                    name: "bloom".into(),
                    busy: Vec::new(),
                    untimed: Some(GpuUntimed::NoGpuPass),
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
        let span = |name: &str| {
            let e = events
                .iter()
                .find(|e| e["name"] == name && e["ph"] == "X")
                .unwrap_or_else(|| panic!("{name} has a span"));
            let ts = e["ts"].as_i64().unwrap();
            (
                e["tid"].as_u64().unwrap() as u32,
                ts,
                ts + e["dur"].as_i64().unwrap(),
            )
        };
        let frame = span("gpu frame 7");
        let (cull, cascade, main) = (span("cull"), span("cascade"), span("main"));
        assert_eq!(cull.2 - cull.1, 8000, "the cull's own 8 ms");
        assert_eq!(cascade.2 - cascade.1, 3000, "the cascade's own 3 ms, not 0");
        assert_eq!((cull.0, main.0), (GPU, GPU));
        assert_eq!(
            cascade.0, GPU_LANE,
            "beside the cull, on a track of its own"
        );
        assert!(
            events
                .iter()
                .any(|e| e["ph"] == "M" && e["tid"] == GPU_LANE)
        );
        assert_eq!(main.2, frame.2, "main ends where the frame does");
        assert_eq!(frame.2 - frame.1, 9978);
        let bloom = events.iter().find(|e| e["name"] == "bloom").expect("bloom");
        assert_eq!(bloom["ph"], "i", "an untimed pass is no span of 0");
        assert_eq!(bloom["args"]["untimed"], GpuUntimed::NoGpuPass.reason());
        let own = &events.iter().find(|e| e["name"] == "gpu frame 7").unwrap()["args"]["passes"];
        assert_eq!(own["cascade"], 3.0);
        assert!(own["bloom"].is_null());
    }
}
