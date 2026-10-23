//! `og run --trace out.json` (`docs/design.md` §3.4): the Chrome trace
//! format (the Trace Event Format's JSON object form, which Perfetto and
//! chrome://tracing open). og writes the cook's spans first, with the
//! trace's origin; the game process (`packages/engine/src/node/run.ts`,
//! `--trace-append`) adds the ticks with their systems, the frames with
//! their render passes and the loads, on the same clock (Unix ms). Lanes
//! are threads of process 1: 1 engine (ticks, systems), 2 render (frames,
//! passes), 3 loads, 4 cook.

use og_cook::Report;
use serde_json::{Value, json};
use std::path::Path;

/// The trace's thread ids and names.
pub const LANES: [(u32, &str); 4] = [(1, "engine"), (2, "render"), (3, "load"), (4, "cook")];

/// Unix time in milliseconds.
pub fn epoch_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

fn span(name: &str, tid: u32, ts_ms: f64, dur_ms: f64, args: Value) -> Value {
    json!({
        "name": name, "cat": "cook", "ph": "X", "pid": 1, "tid": tid,
        "ts": (ts_ms * 1000.0).round() as i64, "dur": (dur_ms * 1000.0).round().max(0.0) as i64,
        "args": args,
    })
}

/// The cook's spans: the whole cook, and inside it each output cooked in
/// this run. The cook records how long each output took, not when it
/// started (outputs cook on several threads), so they are laid end to end
/// in the report's order from the cook's start (`args.laidEndToEnd`).
pub fn cook_events(report: &Report, start_ms: f64, origin_ms: f64) -> Vec<Value> {
    let at = start_ms - origin_ms;
    let mut out = vec![span(
        "cook",
        4,
        at,
        report.elapsed_ms,
        json!({ "assets": report.assets.len() }),
    )];
    let mut t = at;
    for a in &report.assets {
        for o in a.outputs.iter().filter(|o| o.cooked) {
            out.push(span(
                &o.id,
                4,
                t,
                o.ms,
                json!({ "source": a.source, "kind": a.kind, "bytes": o.bytes, "laidEndToEnd": true }),
            ));
            t += o.ms;
        }
    }
    out
}

/// The trace's metadata events: the process and each lane's name.
pub fn metadata() -> Vec<Value> {
    let mut out = vec![json!({
        "name": "process_name", "ph": "M", "pid": 1, "tid": 0, "args": { "name": "og run" },
    })];
    for (tid, name) in LANES {
        out.push(json!({
            "name": "thread_name", "ph": "M", "pid": 1, "tid": tid, "args": { "name": name },
        }));
        out.push(json!({
            "name": "thread_sort_index", "ph": "M", "pid": 1, "tid": tid, "args": { "sort_index": tid },
        }));
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
    fn cook_spans_lie_inside_the_cook_end_to_end() {
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
                    })
                    .collect(),
                notes: vec![],
                error: None,
                cook_ms: 5.0,
            }],
            ignored: vec![],
            removed: vec![],
            warnings: vec![],
            elapsed_ms: 6.0,
        };
        let e = cook_events(&report, 1010.0, 1000.0);
        // The cook, then a and c (b was up to date), c after a.
        let names: Vec<&str> = e.iter().map(|v| v["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["cook", "a", "c"]);
        assert_eq!(e[0]["ts"], 10_000);
        assert_eq!(e[0]["dur"], 6_000);
        assert_eq!(e[1]["ts"], 10_000);
        assert_eq!(e[2]["ts"], 12_500);
        assert_eq!(e[2]["dur"], 2_500);
    }
}
