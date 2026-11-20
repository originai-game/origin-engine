//! The run's performance record (`og run --perf`, the web shell's
//! `perf()`, guide perf): one fixed-size row of plain counters per drawn
//! frame, written into a ring allocated when it is turned on, and a report
//! made from it only when asked (at exit). Measurement only: nothing in the
//! tick reads it.
//!
//! A row holds the frame's interval (wall time since the frame before it,
//! read here: a scripted run's host interval is one tick whatever it took),
//! its CPU time per engine stage (ticks, physics, UI, the game's systems,
//! the engine's other systems; the render's wait for the GPU, its sync, its
//! build, its passes' encoding and its submit, from og-render's frame
//! stages), its GPU time (the frame's span, each pass's time alone,
//! [`og_render::GpuPass::alone`], and the time passes overlapped or the GPU
//! idled), what moved between the CPU and the GPU
//! ([`og_render::traffic`]: bytes uploaded, buffers made, readbacks
//! started, blocking waits for the GPU and pipelines made, each with its
//! call site) and what it drew. The report gives each line's median, p95 and max, flags a line
//! over its budget (the project's `perf.budgets`, passed in), and lists
//! every hitch (a frame whose interval passed twice the median) with its
//! cause: the line that grew most over its own median, or the first-use
//! work named (a pipeline compiled in the frame).

use og_core::IdMap;
use og_ecs::World;
use serde_json::{Value, json};
use std::cell::RefCell;

/// Rows kept: the last this many frames.
const ROWS: usize = 4096;
/// GPU passes told apart (a frame's passes past this go uncounted).
const PASSES: usize = 64;

/// The CPU lines of a row, in report order.
const CPU: [&str; 10] = [
    "cpu ticks",
    "cpu physics",
    "cpu ui",
    "cpu game systems",
    "cpu engine systems",
    "cpu wait for gpu",
    "cpu render sync",
    "cpu render build",
    "cpu render encode",
    "cpu render submit",
];
const TICKS: usize = 0;
const PHYSICS: usize = 1;
const UI: usize = 2;
const GAME: usize = 3;
const ENGINE: usize = 4;
const WAIT: usize = 5;
const SYNC: usize = 6;
const BUILD: usize = 7;
const ENCODE: usize = 8;
const SUBMIT: usize = 9;

#[derive(Clone, Copy)]
struct Row {
    /// The renderer's frame index ([`og_render::FrameStats::frame`]), which
    /// the GPU's timing names when it lands frames later.
    frame: u64,
    tick: u64,
    ticks: u32,
    interval: f32,
    cpu: [f32; CPU.len()],
    /// NaN until the GPU's timing lands (or where it has none).
    gpu_frame: f32,
    gpu_overlapped: f32,
    gpu_idle: f32,
    gpu: [f32; PASSES],
    /// Each pass's time in flight (begin to end, waits included).
    flight: [f32; PASSES],
    compile_us: u32,
    /// Fades held at nothing this frame, and whether a pipeline made ahead
    /// of a hidden instance was still compiling (the hold's reason).
    held: u32,
    compiling_ahead: bool,
    traffic: og_render::traffic::Traffic,
    draws: u32,
    triangles: u64,
}

const EMPTY: Row = Row {
    frame: 0,
    tick: 0,
    ticks: 0,
    interval: 0.0,
    cpu: [0.0; CPU.len()],
    gpu_frame: f32::NAN,
    gpu_overlapped: f32::NAN,
    gpu_idle: f32::NAN,
    gpu: [f32::NAN; PASSES],
    flight: [f32::NAN; PASSES],
    compile_us: 0,
    held: 0,
    compiling_ahead: false,
    traffic: og_render::traffic::Traffic {
        upload_bytes: 0,
        uploads: 0,
        buffers: 0,
        buffer_bytes: 0,
        readbacks: 0,
        waits: 0,
        wait_us: 0,
        pipelines: 0,
        pipeline_us: 0,
    },
    draws: 0,
    triangles: 0,
};

struct Perf {
    rows: Vec<Row>,
    /// The next row's slot, and rows written in all.
    next: usize,
    written: u64,
    /// The row being filled by the ticks before the frame that ends it.
    open: Row,
    tick_start: f64,
    /// When the last frame's row closed (engine clock).
    last_frame: f64,
    /// GPU pass names, in order of first sight; a row's `gpu[i]` is pass i's.
    passes: Vec<String>,
}

thread_local! {
    static PERF: RefCell<IdMap<u32, Perf>> = const { RefCell::new(IdMap::new()) };
}

fn now() -> f64 {
    crate::author::probe::now_ms()
}

fn on() -> bool {
    PERF.with(|p| !p.borrow().is_empty())
}

/// Turns an engine's record on (a fresh ring) or off.
pub fn set(engine: u32, on: bool) {
    PERF.with(|p| {
        let mut p = p.borrow_mut();
        if on {
            p.insert(
                engine,
                Perf {
                    rows: vec![EMPTY; ROWS],
                    next: 0,
                    written: 0,
                    open: EMPTY,
                    tick_start: 0.0,
                    // The first frame's interval runs from here: a first
                    // build is a hitch like any other.
                    last_frame: now(),
                    passes: Vec::with_capacity(PASSES),
                },
            );
        } else {
            p.remove(&engine);
        }
        // The renderer's traffic belongs to the process: counted while any
        // engine records.
        og_render::traffic::set_clock((!p.is_empty()).then_some(now as fn() -> f64));
    });
}

pub(crate) fn before_tick(engine: u32) {
    if !on() {
        return;
    }
    let at = now();
    PERF.with(|p| {
        if let Some(pf) = p.borrow_mut().get_mut(&engine) {
            pf.tick_start = at;
        }
    });
}

/// The tick that just ran: its whole time, and its systems' by stage.
pub(crate) fn after_tick(engine: u32, world: &World) {
    if !on() {
        return;
    }
    let end = now();
    PERF.with(|p| {
        let mut p = p.borrow_mut();
        let Some(pf) = p.get_mut(&engine) else {
            return;
        };
        let row = &mut pf.open;
        row.cpu[TICKS] += (end - pf.tick_start) as f32;
        row.ticks += 1;
        world.each_system_ms(world.time().tick.saturating_sub(1), |name, ms| {
            let line = if name.starts_with("og.physics") {
                PHYSICS
            } else if name.starts_with("og.ui") {
                UI
            } else if name.starts_with("og.") {
                ENGINE
            } else {
                GAME
            };
            row.cpu[line] += ms as f32;
        });
    });
}

/// The frame's render stages (og-render's frame stages), into its row:
/// the wait for the GPU, the sync, the encode and the submit by name; the
/// rest is the build.
pub(crate) fn stages(engine: u32, backend: &dyn crate::present::extract::RenderBackend) {
    if !on() {
        return;
    }
    PERF.with(|p| {
        let mut p = p.borrow_mut();
        let Some(pf) = p.get_mut(&engine) else {
            return;
        };
        let row = &mut pf.open;
        backend.each_stage(&mut |name, ms| {
            let line = match name {
                "wait for frame N-2" => WAIT,
                "sync" => SYNC,
                "graph encode" => ENCODE,
                "submit" => SUBMIT,
                _ => BUILD,
            };
            row.cpu[line] += ms as f32;
        });
    });
}

/// The fades the frame held for pipelines (present/reveal.rs), into its
/// row: what a spawn's first frames still wait for.
pub(crate) fn held(engine: u32, compiling_ahead: bool, fades: u32) {
    if !on() {
        return;
    }
    PERF.with(|p| {
        if let Some(pf) = p.borrow_mut().get_mut(&engine) {
            pf.open.held = fades;
            pf.open.compiling_ahead = compiling_ahead;
        }
    });
}

/// A drawn frame: closes its row with the render's numbers, and fills
/// the rows of the frames the GPU's timing names. `_host_interval` is the
/// host's (one tick in a scripted run): the row's is the wall time here.
pub(crate) fn frame(
    engine: u32,
    tick: u64,
    _host_interval: f64,
    render_ms: f64,
    passes: &[og_render::PassTiming],
    gpu: &[og_render::GpuFrame],
    stats: &og_render::FrameStats,
) {
    if !on() {
        return;
    }
    PERF.with(|p| {
        let mut p = p.borrow_mut();
        let Some(pf) = p.get_mut(&engine) else {
            return;
        };
        let at = now();
        let mut row = std::mem::replace(&mut pf.open, EMPTY);
        row.frame = stats.frame;
        row.tick = tick;
        row.interval = (at - pf.last_frame) as f32;
        pf.last_frame = at;
        // Without the renderer's stages (a backend that keeps none), the
        // passes' encoding and the rest of the render call.
        let staged: f32 = row.cpu[WAIT..=SUBMIT].iter().sum();
        if staged == 0.0 {
            let encode: f64 = passes.iter().map(|p| p.ms).sum();
            row.cpu[ENCODE] = encode as f32;
            row.cpu[BUILD] = (render_ms - encode).max(0.0) as f32;
        }
        row.compile_us = stats.compile_wait_us;
        row.traffic = stats.traffic;
        row.draws = stats.draw_calls;
        row.triangles = stats.triangles;
        let slot = pf.next;
        pf.rows[slot] = row;
        pf.next = (slot + 1) % ROWS;
        pf.written += 1;
        for f in gpu {
            let Some(i) = (1..=ROWS.min(pf.written as usize))
                .map(|back| (pf.next + ROWS - back) % ROWS)
                .take(16)
                .find(|&i| pf.rows[i].frame == f.frame)
            else {
                continue;
            };
            pf.rows[i].gpu_frame = f.ms() as f32;
            pf.rows[i].gpu_overlapped = f.overlapped_ms() as f32;
            pf.rows[i].gpu_idle = f.idle as f32;
            for pass in &f.passes {
                let (Some(ms), Some(flight)) = (pass.ms(), pass.in_flight_ms()) else {
                    continue;
                };
                let k = match pf.passes.iter().position(|n| *n == pass.name) {
                    Some(k) => k,
                    None if pf.passes.len() < PASSES => {
                        pf.passes.push(pass.name.clone());
                        pf.passes.len() - 1
                    }
                    None => continue,
                };
                let add = |cell: &mut f32, ms: f64| {
                    *cell = if cell.is_nan() {
                        ms as f32
                    } else {
                        *cell + ms as f32
                    }
                };
                add(&mut pf.rows[i].gpu[k], ms);
                add(&mut pf.rows[i].flight[k], flight);
            }
        }
    });
}

/// A report line: its name and how a row gives its value.
type Line = (String, Box<dyn Fn(&Row) -> f32>);

/// One line's values over the rows that have it, sorted.
fn sorted(values: impl Iterator<Item = f32>) -> Vec<f32> {
    let mut v: Vec<f32> = values.filter(|x| x.is_finite()).collect();
    v.sort_by(f32::total_cmp);
    v
}

fn at(v: &[f32], q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    f64::from(v[((v.len() - 1) as f64 * q).round() as usize])
}

fn r2(x: f64) -> f64 {
    // + 0.0: a pass that only ever overlapped reads 0, not -0.
    (x * 100.0).round() / 100.0 + 0.0
}

/// The report of the rows recorded so far, as JSON and text:
/// `{ json, text }`. `budgets` is a JSON object of line name to the most
/// milliseconds its median may take (`"gpu frame"`, `"gpu <pass>"`, a
/// [`CPU`] line, `"frame interval"`); a line over its budget is flagged.
pub fn report(engine: u32, budgets: &str) -> String {
    let budgets: serde_json::Map<String, Value> = serde_json::from_str(budgets).unwrap_or_default();
    PERF.with(|p| {
        let p = p.borrow();
        let Some(pf) = p.get(&engine) else {
            return json!({ "json": Value::Null, "text": "perf: not recording" }).to_string();
        };
        let n = (pf.written as usize).min(ROWS);
        let rows: Vec<&Row> = (0..n).map(|k| &pf.rows[(pf.next + ROWS - n + k) % ROWS]).collect();
        // The lines: name and each row's value.
        let mut lines: Vec<Line> = vec![
            ("frame interval".into(), Box::new(|r: &Row| r.interval)),
            ("gpu frame".into(), Box::new(|r: &Row| r.gpu_frame)),
            ("gpu overlapped".into(), Box::new(|r: &Row| r.gpu_overlapped)),
            ("gpu idle".into(), Box::new(|r: &Row| r.gpu_idle)),
        ];
        for (i, name) in CPU.iter().enumerate() {
            lines.push(((*name).into(), Box::new(move |r: &Row| r.cpu[i])));
        }
        for (i, name) in pf.passes.iter().enumerate() {
            lines.push((format!("gpu {name}"), Box::new(move |r: &Row| r.gpu[i])));
        }
        for (i, name) in pf.passes.iter().enumerate() {
            lines.push((format!("gpu {name} (in flight)"), Box::new(move |r: &Row| r.flight[i])));
        }
        // What moved between the CPU and the GPU: two times (the frame's
        // blocking waits for the GPU, and its pipelines made), then counts.
        let us = |x: u32| x as f32 / 1000.0;
        lines.push(("cpu blocking gpu waits".into(), Box::new(move |r: &Row| us(r.traffic.wait_us))));
        lines.push(("cpu pipelines made".into(), Box::new(move |r: &Row| us(r.traffic.pipeline_us))));
        let times = lines.len();
        lines.push(("upload kb".into(), Box::new(|r: &Row| r.traffic.upload_bytes as f32 / 1024.0)));
        lines.push(("uploads".into(), Box::new(|r: &Row| r.traffic.uploads as f32)));
        lines.push(("buffers made".into(), Box::new(|r: &Row| r.traffic.buffers as f32)));
        lines.push(("buffer kb made".into(), Box::new(|r: &Row| r.traffic.buffer_bytes as f32 / 1024.0)));
        lines.push(("readbacks".into(), Box::new(|r: &Row| r.traffic.readbacks as f32)));
        lines.push(("blocking gpu waits".into(), Box::new(|r: &Row| r.traffic.waits as f32)));
        lines.push(("pipelines made".into(), Box::new(|r: &Row| r.traffic.pipelines as f32)));
        lines.push(("fades held".into(), Box::new(|r: &Row| r.held as f32)));
        lines.push((
            "frames compiling ahead".into(),
            Box::new(|r: &Row| f32::from(u8::from(r.compiling_ahead))),
        ));
        // Lines that are another line's consequence, never a hitch's cause:
        // the interval itself, the GPU's whole frame and overlap, and the
        // CPU's wait for the GPU.
        // Under `--gpu-serial` every pass waits for the GPU as it encodes,
        // so the encode is the GPU's time too.
        let serial = cfg!(not(target_arch = "wasm32")) && std::env::var_os("OG_GPU_SERIAL").is_some();
        let effect = |name: &str| {
            matches!(name, "frame interval" | "gpu frame" | "gpu overlapped" | "cpu wait for gpu")
                || (serial && name == "cpu render encode")
        };
        let medians: Vec<f64> = lines
            .iter()
            .map(|(_, f)| at(&sorted(rows.iter().map(|r| f(r))), 0.5))
            .collect();
        let mut out_lines = Vec::new();
        let mut text = format!("og perf: {n} frames\n{:<34} {:>9} {:>9} {:>9}  budget\n", "line", "median", "p95", "max");
        let frame_median = medians[0].max(1e-6);
        for ((name, f), &median) in lines.iter().zip(&medians) {
            let v = sorted(rows.iter().map(|r| f(r)));
            if v.is_empty() || (median < 0.05 && at(&v, 1.0) < 1.0) {
                continue;
            }
            let budget = budgets.get(name).and_then(Value::as_f64);
            let over = budget.is_some_and(|b| median > b);
            let (p95, max) = (at(&v, 0.95), at(&v, 1.0));
            out_lines.push(json!({
                "line": name, "median": r2(median), "p95": r2(p95), "max": r2(max),
                "share": r2(median / frame_median), "budget": budget, "over": over,
            }));
            text += &format!(
                "{:<34} {:>9.2} {:>9.2} {:>9.2}  {}\n",
                name,
                r2(median),
                r2(p95),
                r2(max),
                match budget {
                    Some(b) if over => format!("OVER {b}"),
                    Some(b) => format!("ok {b}"),
                    None => String::new(),
                }
            );
        }
        // Hitches: each with the line that grew most over its median, or
        // the pipeline compiled in it.
        let mut hitches = Vec::new();
        for r in &rows {
            if f64::from(r.interval) <= 2.0 * frame_median {
                continue;
            }
            let mut cause = if r.compile_us > 1000 {
                format!("pipeline compile {:.1} ms", f64::from(r.compile_us) / 1000.0)
            } else {
                lines[..times]
                    .iter()
                    .zip(&medians)
                    .filter(|((name, _), _)| !effect(name))
                    .map(|((name, f), m)| (name.as_str(), f64::from(f(r)) - m))
                    .filter(|(_, d)| d.is_finite())
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map_or("unknown".into(), |(name, d)| format!("{name} +{d:.1} ms"))
            };
            // The first-use work the frame did, named: a frame after load
            // or Play has no steady median to stand out from.
            let t = &r.traffic;
            let mut work = Vec::new();
            if t.pipelines > 0 {
                work.push(format!("{} pipelines made", t.pipelines));
            }
            if t.buffers > 0 {
                work.push(format!("{} buffers made ({} kb)", t.buffers, t.buffer_bytes / 1024));
            }
            if t.waits > 0 {
                work.push(format!("{} blocking gpu waits ({:.1} ms)", t.waits, us(t.wait_us)));
            }
            if t.upload_bytes >= 1 << 20 {
                work.push(format!("{} kb uploaded", t.upload_bytes / 1024));
            }
            if !work.is_empty() {
                cause += &format!("; {}", work.join(", "));
            }
            hitches.push(json!({ "tick": r.tick, "interval": r2(f64::from(r.interval)), "ticks": r.ticks, "cause": cause }));
        }
        text += &format!("hitches (interval over 2x the median {frame_median:.2} ms): {}\n", hitches.len());
        for h in hitches.iter().take(40) {
            text += &format!(
                "  tick {:>7}  {:>8.2} ms  {} ticks  {}\n",
                h["tick"], h["interval"].as_f64().unwrap_or(0.0), h["ticks"], h["cause"].as_str().unwrap_or("")
            );
        }
        let (sites, sites_text) = sites(n);
        text += &sites_text;
        let drawn = |f: fn(&Row) -> f32| r2(at(&sorted(rows.iter().map(|r| f(r))), 0.5));
        let json = json!({
            "frames": n, "lines": out_lines, "hitches": hitches, "sites": sites,
            "draws": drawn(|r| r.draws as f32), "triangles": drawn(|r| r.triangles as f32),
        });
        json!({ "json": json, "text": text }).to_string()
    })
}

/// The renderer's call sites that moved data, for the report: the
/// biggest uploaders and buffer makers, every blocking wait and readback,
/// and the pipelines made after the first frames (late: a frame waited or
/// will wait for them). `frames` is the record's, for per-frame bytes.
fn sites(frames: usize) -> (Value, String) {
    use og_render::traffic::{Kind, Site};
    let mut all: Vec<Site> = Vec::new();
    let lost = og_render::traffic::each_site(|s| all.push(*s));
    let frames = frames.max(1) as f64;
    let name = |s: &Site| {
        let file = s.at.file();
        let file = file.rsplit_once("src/").map_or(file, |(_, f)| f);
        match s.name() {
            "" => format!("{file}:{}", s.at.line()),
            label => format!("{label} ({file}:{})", s.at.line()),
        }
    };
    let mut out = Vec::new();
    let mut text = String::new();
    // `steady`: Some(true) only sites still moving after the first frames
    // (late, per frame), Some(false) only those done by then (load), None
    // every site.
    let mut section = |title: &str,
                       kind: Kind,
                       steady: Option<bool>,
                       key: fn(&Site) -> f64,
                       take: usize| {
        let mut v: Vec<&Site> = all
            .iter()
            .filter(|s| s.kind == kind && steady.is_none_or(|on| (s.last_frame > 2) == on))
            .collect();
        v.sort_by(|a, b| key(b).total_cmp(&key(a)));
        if v.is_empty() {
            return;
        }
        let total: u64 = v.iter().map(|s| s.bytes).sum();
        text += &format!(
            "{title} ({} sites, {:.1} MB)\n",
            v.len(),
            total as f64 / 1048576.0
        );
        for s in v.iter().take(take) {
            let site = name(s);
            text += &format!(
                "  {site:<60} {:>8} calls {:>10.1} MB {:>10.1} kb/frame {:>8.1} ms  frames {}..{}\n",
                s.count,
                s.bytes as f64 / 1048576.0,
                s.bytes as f64 / 1024.0 / frames,
                s.ms,
                s.first_frame,
                s.last_frame
            );
            out.push(json!({
                "kind": format!("{kind:?}").to_lowercase(), "site": site, "count": s.count,
                "bytes": s.bytes, "ms": r2(s.ms), "first_frame": s.first_frame,
                "last_frame": s.last_frame, "late": s.last_frame > 2,
            }));
        }
    };
    section(
        "uploads in steady play, by bytes",
        Kind::Upload,
        Some(true),
        |s| s.bytes as f64,
        12,
    );
    section(
        "uploads at load only, by bytes",
        Kind::Upload,
        Some(false),
        |s| s.bytes as f64,
        12,
    );
    section(
        "buffers made, by bytes",
        Kind::Buffer,
        None,
        |s| s.bytes as f64,
        16,
    );
    section("readbacks", Kind::Readback, None, |s| s.count as f64, 12);
    section(
        "blocking gpu waits, by time",
        Kind::Wait,
        None,
        |s| s.ms,
        12,
    );
    section(
        "pipelines made after the first frames",
        Kind::Pipeline,
        Some(true),
        |s| s.ms,
        20,
    );
    if lost > 0 {
        text += &format!("({lost} events past the site table)\n");
    }
    (Value::Array(out), text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A budget flags only its own line, and a hitch is put on the line
    /// that grew, not on the CPU's wait for the GPU it caused.
    #[test]
    fn report_flags_budgets_and_names_a_hitch_cause() {
        set(7, true);
        PERF.with(|p| {
            let mut p = p.borrow_mut();
            let pf = p.get_mut(&7).unwrap();
            pf.passes.push("main".into());
            pf.passes.push("cascade.0".into());
            for i in 0..20 {
                let hitch = i == 10;
                let mut r = EMPTY;
                r.tick = i;
                r.interval = if hitch { 80.0 } else { 20.0 };
                r.gpu_frame = r.interval;
                r.cpu[WAIT] = r.interval - 2.0;
                r.gpu[0] = 12.0;
                r.gpu[1] = if hitch { 45.0 } else { 3.0 };
                pf.rows[i as usize] = r;
            }
            pf.next = 20;
            pf.written = 20;
        });
        let out: Value =
            serde_json::from_str(&report(7, r#"{"gpu main": 10, "gpu cascade.0": 5}"#)).unwrap();
        set(7, false);
        let lines = out["json"]["lines"].as_array().unwrap();
        let over = |name: &str| lines.iter().find(|l| l["line"] == name).unwrap()["over"].clone();
        assert_eq!(over("gpu main"), json!(true));
        assert_eq!(over("gpu cascade.0"), json!(false));
        let hitches = out["json"]["hitches"].as_array().unwrap();
        assert_eq!(hitches.len(), 1);
        assert_eq!(hitches[0]["tick"], 10);
        assert!(
            hitches[0]["cause"]
                .as_str()
                .unwrap()
                .starts_with("gpu cascade.0 +"),
            "{}",
            hitches[0]
        );
    }
}
