//! `og cost [project] --off "<og run options>" [--on "<og run options>"]
//! [--runs N] [--ticks A..B] [--json] [-- <og run options for both>]`: what
//! a feature costs the real frame, measured as the difference of the GPU
//! frame's span (the trace's `gpu frame N`: its first timestamp to its
//! last) with it and without it. A pass's own timestamps cannot say it:
//! passes the GPU runs at once share their time, and on Metal a pass is in
//! flight long before it runs (frame/timing.rs in og-render).
//!
//! It runs the project `N` times a side (at least 4, default 4) with
//! `og run --trace`, the sides alternating in pairs, on then off, then off
//! then on, so a machine growing busier or cooler moves both alike. The
//! options a side adds turn the feature on or off (a `--setup` action or
//! `--commands` the game answers); the options after `--` go to every
//! run (`--size`, `--commands` that hold the camera, `--no-check`). Each
//! run is read over the frames the GPU began between the starts of ticks
//! `A` and `B` (default: from tick 60 to the run's end): its median frame
//! span. Each side reports the median of its runs' medians, their range,
//! the frames read and the machine's load average (1 minute) as each run
//! began; the cost is the on side's median less the off side's, with the
//! range the runs allow. A run without GPU timestamps has no frame span:
//! the command says so and stops.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: og cost [project] --off \"<og run options>\" [--on \"<og run options>\"] [--runs N] [--ticks A..B] [--json] [-- <og run options for both>]";

/// Runs a side takes at the least.
const MIN_RUNS: usize = 4;
/// The first tick read without `--ticks`: the first second's loads and
/// compiles are not the frame.
const FIRST_TICK: u64 = 60;

struct Args {
    project: String,
    on: Vec<String>,
    off: Vec<String>,
    common: Vec<String>,
    runs: usize,
    ticks: Option<(u64, u64)>,
    json: bool,
}

fn words(s: &str) -> Vec<String> {
    s.split_whitespace().map(str::to_string).collect()
}

fn parse(args: &[&str]) -> Result<Args, String> {
    let mut a = Args {
        project: ".".into(),
        on: Vec::new(),
        off: Vec::new(),
        common: Vec::new(),
        runs: MIN_RUNS,
        ticks: None,
        json: false,
    };
    let mut off = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--" => {
                a.common = args[i + 1..].iter().map(|s| s.to_string()).collect();
                break;
            }
            "--json" => a.json = true,
            flag @ ("--on" | "--off" | "--runs" | "--ticks") => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| format!("{flag} needs a value. {USAGE}"))?;
                match flag {
                    "--on" => a.on = words(v),
                    "--off" => off = Some(words(v)),
                    "--runs" => {
                        a.runs = v
                            .parse()
                            .map_err(|_| format!("--runs {v}: a number. {USAGE}"))?;
                        if a.runs < MIN_RUNS {
                            return Err(format!(
                                "--runs {v}: at least {MIN_RUNS} a side, or one busy run moves the median"
                            ));
                        }
                    }
                    _ => {
                        let (s, e) = v
                            .split_once("..")
                            .ok_or_else(|| format!("--ticks {v}: A..B. {USAGE}"))?;
                        let n =
                            |t: &str| t.parse::<u64>().map_err(|_| format!("--ticks {v}: A..B"));
                        let (s, e) = (n(s)?, n(e)?);
                        if s >= e {
                            return Err(format!("--ticks {v}: A before B"));
                        }
                        a.ticks = Some((s, e));
                    }
                }
                i += 1;
            }
            f if f.starts_with("--") => return Err(format!("unknown option {f}. {USAGE}")),
            p => a.project = p.to_string(),
        }
        i += 1;
    }
    a.off = off.ok_or_else(|| format!("--off is required: what turns the feature off. {USAGE}"))?;
    Ok(a)
}

/// One run's frames: each GPU frame's span (ms) begun between the starts
/// of ticks `from` and `to` (`to`: `None` for the run's end).
fn frame_spans(doc: &Value, from: u64, to: Option<u64>) -> Vec<f64> {
    let none = Vec::new();
    let events = doc["traceEvents"].as_array().unwrap_or(&none);
    let tick_ts = |t: u64| {
        events
            .iter()
            .find(|e| e["cat"] == "tick" && e["args"]["tick"].as_u64() == Some(t))
            .and_then(|e| e["ts"].as_i64())
    };
    let Some(start) = tick_ts(from) else {
        return Vec::new();
    };
    let end = match to {
        Some(t) => match tick_ts(t) {
            Some(e) => e,
            None => return Vec::new(),
        },
        None => i64::MAX,
    };
    events
        .iter()
        .filter(|e| {
            e["ph"] == "X"
                && e["cat"] == "gpu"
                && e["name"]
                    .as_str()
                    .is_some_and(|n| n.starts_with("gpu frame "))
        })
        .filter_map(|e| Some((e["ts"].as_i64()?, e["dur"].as_i64()?)))
        .filter(|&(ts, _)| start <= ts && ts < end)
        .map(|(_, dur)| dur as f64 / 1000.0)
        .collect()
}

fn median(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    match v.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => v[n / 2],
        n => (v[n / 2 - 1] + v[n / 2]) / 2.0,
    }
}

/// The 1-minute load average, where the system reports one.
fn load() -> Option<f64> {
    #[cfg(unix)]
    {
        let mut l = [0f64; 3];
        // SAFETY: getloadavg writes at most the 3 doubles it is given.
        let n = unsafe { libc::getloadavg(l.as_mut_ptr(), 3) };
        (n >= 1).then_some(l[0])
    }
    #[cfg(not(unix))]
    None
}

struct Run {
    on: bool,
    median: f64,
    frames: usize,
    load: Option<f64>,
}

fn one_run(a: &Args, on: bool, k: usize, dir: &Path) -> Result<Run, String> {
    let trace: PathBuf = dir.join(format!("{}-{k}.json", if on { "on" } else { "off" }));
    let load = load();
    let exe = std::env::current_exe().map_err(|e| format!("og cost: this og's path: {e}"))?;
    let status = std::process::Command::new(exe)
        .arg("run")
        .arg(&a.project)
        .args(["--headless", "--trace"])
        .arg(&trace)
        .args(&a.common)
        .args(if on { &a.on } else { &a.off })
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("og cost: og run: {e}"))?;
    if !status.success() {
        return Err(format!(
            "og cost: the {} run {k} failed ({status})",
            side(on)
        ));
    }
    let text = std::fs::read(&trace).map_err(|e| format!("{}: {e}", trace.display()))?;
    let doc: Value =
        serde_json::from_slice(&text).map_err(|e| format!("{}: {e}", trace.display()))?;
    let (from, to) = match a.ticks {
        Some((s, e)) => (s, Some(e)),
        None => (FIRST_TICK, None),
    };
    let spans = frame_spans(&doc, from, to);
    if spans.is_empty() {
        return Err(format!(
            "og cost: the {} run {k} has no GPU frame between its ticks {from} and {}: no GPU timestamps on this device (RenderCapabilities.gpuTimestamps), or the run ended first ({})",
            side(on),
            to.map_or("its end".into(), |t| t.to_string()),
            trace.display()
        ));
    }
    Ok(Run {
        on,
        median: median(&spans),
        frames: spans.len(),
        load,
    })
}

fn side(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// The order of the runs: on/off pairs, every other one turned round.
fn order(runs: usize) -> Vec<bool> {
    (0..runs)
        .flat_map(|k| {
            if k % 2 == 0 {
                [true, false]
            } else {
                [false, true]
            }
        })
        .collect()
}

pub fn run(args: &[&str]) -> ExitCode {
    let a = match parse(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og cost: {e}");
            return ExitCode::from(2);
        }
    };
    let dir = std::env::temp_dir().join(format!("og-cost-{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("og cost: {}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    let mut runs = Vec::new();
    let mut seen = [0usize; 2];
    for on in order(a.runs) {
        let k = seen[usize::from(on)];
        seen[usize::from(on)] += 1;
        if !a.json {
            eprintln!("og cost: {} run {} of {}", side(on), k + 1, a.runs);
        }
        match one_run(&a, on, k, &dir) {
            Ok(r) => runs.push(r),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let report = report(&runs, a.ticks);
    if a.json {
        println!("{report}");
    } else {
        print!("{}", text(&report));
    }
    ExitCode::SUCCESS
}

fn report(runs: &[Run], ticks: Option<(u64, u64)>) -> Value {
    let of = |on: bool| {
        let rs: Vec<&Run> = runs.iter().filter(|r| r.on == on).collect();
        let medians: Vec<f64> = rs.iter().map(|r| r.median).collect();
        let loads: Vec<f64> = rs.iter().filter_map(|r| r.load).collect();
        json!({
            "median": median(&medians),
            "min": medians.iter().copied().fold(f64::INFINITY, f64::min),
            "max": medians.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            "runs": rs.iter().map(|r| json!({ "median": r.median, "frames": r.frames, "load": r.load })).collect::<Vec<_>>(),
            "load": if loads.is_empty() { Value::Null } else {
                json!({ "min": loads.iter().copied().fold(f64::INFINITY, f64::min), "max": loads.iter().copied().fold(f64::NEG_INFINITY, f64::max) })
            },
        })
    };
    let (on, off) = (of(true), of(false));
    let f = |v: &Value, k: &str| v[k].as_f64().unwrap_or(f64::NAN);
    json!({
        "ticks": ticks.map(|(a, b)| json!([a, b])),
        "on": on,
        "off": off,
        "cost": {
            "median": f(&on, "median") - f(&off, "median"),
            "min": f(&on, "min") - f(&off, "max"),
            "max": f(&on, "max") - f(&off, "min"),
        },
    })
}

fn text(r: &Value) -> String {
    let f = |v: &Value| v.as_f64().unwrap_or(f64::NAN);
    let line = |name: &str, s: &Value| {
        let load = match &s["load"] {
            Value::Null => "load unknown".to_string(),
            l => format!("load {:.1} to {:.1}", f(&l["min"]), f(&l["max"])),
        };
        let frames: Vec<String> = s["runs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|r| format!("{:.2} ({})", f(&r["median"]), r["frames"]))
            .collect();
        format!(
            "  {name:<4} median {:.2} ms, runs {:.2} to {:.2}, {load}; per run, ms (frames): {}\n",
            f(&s["median"]),
            f(&s["min"]),
            f(&s["max"]),
            frames.join(", ")
        )
    };
    let ticks = match &r["ticks"] {
        Value::Array(t) => format!("ticks {}..{}", t[0], t[1]),
        _ => format!("from tick {FIRST_TICK}"),
    };
    let c = &r["cost"];
    format!(
        "og cost: GPU frame span, median per run, {ticks}\n{}{}  cost  {:+.2} ms (on less off), the runs allow {:+.2} to {:+.2}\n",
        line("on", &r["on"]),
        line("off", &r["off"]),
        f(&c["median"]),
        f(&c["min"]),
        f(&c["max"]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run is read over the GPU frames begun between the two ticks'
    /// starts, by their span: not the render thread's frames, not the
    /// passes inside a GPU frame, not a frame begun past the last tick.
    #[test]
    fn a_run_reads_the_gpu_frames_between_its_ticks() {
        let tick = |t: u64, ts: i64| json!({ "name": format!("tick {t}"), "cat": "tick", "ph": "X", "ts": ts, "dur": 10, "args": { "tick": t } });
        let gpu = |name: &str, ts: i64, dur: i64| json!({ "name": name, "cat": "gpu", "ph": "X", "ts": ts, "dur": dur });
        let doc = json!({ "traceEvents": [
            tick(10, 1000), tick(12, 3000),
            gpu("gpu frame 1", 900, 50_000),
            gpu("gpu frame 2", 1500, 20_000),
            gpu("main", 1500, 19_000),
            gpu("gpu frame 3", 2500, 30_000),
            gpu("gpu frame 4", 3000, 90_000),
            json!({ "name": "frame 2", "cat": "render", "ph": "X", "ts": 1600, "dur": 5000 }),
        ]});
        assert_eq!(frame_spans(&doc, 10, Some(12)), [20.0, 30.0]);
        assert_eq!(frame_spans(&doc, 10, None), [20.0, 30.0, 90.0]);
        assert!(frame_spans(&doc, 11, Some(12)).is_empty(), "no tick 11");
    }

    /// Sides alternate in pairs turned round every other time, and the
    /// cost is the on side's median of medians less the off side's.
    #[test]
    fn the_cost_is_the_difference_of_the_sides_medians() {
        assert_eq!(
            order(4),
            [true, false, false, true, true, false, false, true]
        );
        let run = |on, median| Run {
            on,
            median,
            frames: 30,
            load: Some(2.0),
        };
        let runs = [
            run(true, 92.0),
            run(false, 90.0),
            run(false, 89.0),
            run(true, 95.0),
            run(true, 93.0),
            run(false, 88.0),
            run(false, 91.0),
            run(true, 120.0),
        ];
        let r = report(&runs, Some((900, 1015)));
        assert_eq!(r["on"]["median"], 94.0, "one busy run moves no median");
        assert_eq!(r["off"]["median"], 89.5);
        assert_eq!(r["cost"]["median"], 4.5);
        assert_eq!(r["cost"]["min"], 1.0);
        assert_eq!(r["cost"]["max"], 32.0);
        assert!(parse(&["."]).is_err(), "--off is required");
        assert!(parse(&["--off", "--setup off", "--runs", "3"]).is_err());
        let a = parse(&[
            "g",
            "--off",
            "--setup scatter:off",
            "--ticks",
            "900..1015",
            "--",
            "--size",
            "2560x1440",
        ])
        .unwrap();
        assert_eq!(
            (a.project.as_str(), a.off.len(), a.ticks, a.common.len()),
            ("g", 2, Some((900, 1015)), 2)
        );
    }
}
