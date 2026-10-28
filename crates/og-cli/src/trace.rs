//! `og run --trace out.json` (`docs/design.md` §3.4): the Chrome trace
//! format (the Trace Event Format's JSON object form, which Perfetto and
//! chrome://tracing open). og writes the cook's spans first, with the
//! trace's origin; the game process (`packages/engine/src/node/run.ts`,
//! `--trace-append`) adds the ticks with their systems, the frames with
//! their render passes and the loads, on the same clock (Unix ms). Lanes
//! are threads of process 1: 1 engine (ticks, systems), 2 render (frames,
//! passes, CPU encoding), 3 loads, 4 cook, 5 gpu (each frame's passes from
//! timestamp queries); og-lanes' pool threads are 16 + their index
//! ("lane N") and the cook's threads [`COOK_THREADS`] + theirs ("cook N"),
//! each named by a metadata event where it first appears. Every span lies
//! where it ran: its start is recorded, not derived.

use og_cook::Report;
use serde_json::{Value, json};
use std::path::Path;

/// The trace's thread ids and names.
pub const LANES: [(u32, &str); 5] = [
    (1, "engine"),
    (2, "render"),
    (3, "load"),
    (4, "cook"),
    (5, "gpu"),
];

/// The first thread id of the cook's threads (cook thread `n` is this + n).
pub const COOK_THREADS: u32 = 48;

/// Unix time in milliseconds.
pub fn epoch_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

/// A span from `ts_ms` for `dur_ms`, both rounded to whole microseconds
/// at their ends, so a span inside another stays inside it.
fn span(name: &str, tid: u32, ts_ms: f64, dur_ms: f64, args: Value) -> Value {
    let ts = (ts_ms * 1000.0).round() as i64;
    let end = ((ts_ms + dur_ms.max(0.0)) * 1000.0).round() as i64;
    json!({
        "name": name, "cat": "cook", "ph": "X", "pid": 1, "tid": tid,
        "ts": ts, "dur": end - ts, "args": args,
    })
}

/// The cook's spans: the whole cook on the cook lane, and each output
/// cooked in this run on the thread that cooked it, from when it began
/// (`OutputReport::start_ms`, `worker`), with each thread named.
pub fn cook_events(report: &Report, start_ms: f64, origin_ms: f64) -> Vec<Value> {
    let at = start_ms - origin_ms;
    let mut out = vec![span(
        "cook",
        4,
        at,
        report.elapsed_ms,
        json!({ "assets": report.assets.len() }),
    )];
    let mut named = std::collections::BTreeSet::new();
    for a in &report.assets {
        for o in a.outputs.iter().filter(|o| o.cooked) {
            let tid = COOK_THREADS + o.worker;
            if named.insert(tid) {
                out.extend(thread_named(tid, &format!("cook {}", o.worker)));
            }
            out.push(span(
                &o.id,
                tid,
                at + o.start_ms,
                o.ms,
                json!({ "source": a.source, "kind": a.kind, "bytes": o.bytes }),
            ));
        }
    }
    out
}

/// A thread's name and its place among the trace's threads.
fn thread_named(tid: u32, name: &str) -> [Value; 2] {
    [
        json!({ "name": "thread_name", "ph": "M", "pid": 1, "tid": tid, "args": { "name": name } }),
        json!({ "name": "thread_sort_index", "ph": "M", "pid": 1, "tid": tid, "args": { "sort_index": tid } }),
    ]
}

/// The trace's metadata events: the process and each lane's name.
pub fn metadata() -> Vec<Value> {
    let mut out = vec![json!({
        "name": "process_name", "ph": "M", "pid": 1, "tid": 0, "args": { "name": "og run" },
    })];
    for (tid, name) in LANES {
        out.extend(thread_named(tid, name));
    }
    out
}

/// Writes the trace's first part: metadata and the cook's spans, with the
/// origin every `ts` counts from.
pub fn write_prelude(path: &Path, cook: Vec<Value>, origin_ms: f64) -> Result<(), String> {
    let mut events = metadata();
    events.extend(cook);
    let doc = json!({
        "traceEvents": events,
        "displayTimeUnit": "ms",
        "otherData": { "originUnixMs": origin_ms, "producer": "og run --trace" },
    });
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("--trace {}: {e}", path.display()))?;
    }
    std::fs::write(path, doc.to_string()).map_err(|e| format!("--trace {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cook_spans_lie_where_they_ran_on_their_threads() {
        let report = Report {
            root: String::new(),
            cooker: String::new(),
            texture_encoder: String::new(),
            assets: vec![og_cook::cook::AssetReport {
                source: "models/a.glb".into(),
                kind: "gltf".into(),
                source_bytes: 10,
                outputs: ["a", "b", "c"]
                    .iter()
                    .enumerate()
                    .map(|(i, id)| og_cook::cook::OutputReport {
                        id: id.to_string(),
                        file: String::new(),
                        ty: String::new(),
                        bytes: 1,
                        cooked: i != 1,
                        written: true,
                        ms: 2.5,
                        // a on thread 0 from 0.5 ms, c on thread 1 from 0.25 ms:
                        // side by side, not one after the other.
                        start_ms: if i == 0 { 0.5 } else { 0.25 },
                        worker: if i == 0 { 0 } else { 1 },
                    })
                    .collect(),
                notes: vec![],
                error: None,
                cook_ms: 5.0,
            }],
            ignored: vec![],
            removed: vec![],
            warnings: vec![],
            conform: vec![],
            elapsed_ms: 6.0,
        };
        let e = cook_events(&report, 1010.0, 1000.0);
        let spans: Vec<(&str, u64, i64, i64)> = e
            .iter()
            .filter(|v| v["ph"] == "X")
            .map(|v| {
                let n = v["name"].as_str().unwrap();
                (
                    n,
                    v["tid"].as_u64().unwrap(),
                    v["ts"].as_i64().unwrap(),
                    v["dur"].as_i64().unwrap(),
                )
            })
            .collect();
        // The cook, then a and c (b was up to date), each where it ran.
        assert_eq!(
            spans,
            [
                ("cook", 4, 10_000, 6_000),
                ("a", u64::from(COOK_THREADS), 10_500, 2_500),
                ("c", u64::from(COOK_THREADS) + 1, 10_250, 2_500),
            ]
        );
        let named: Vec<&str> = e
            .iter()
            .filter(|v| v["name"] == "thread_name")
            .map(|v| v["args"]["name"].as_str().unwrap())
            .collect();
        assert_eq!(named, ["cook 0", "cook 1"]);
    }
}
