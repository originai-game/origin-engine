//! The raycast record's cost of its own (debt round 2, `physics-push`;
//! tests/evidence/depth/geometry/NOTES.md owed it): the probe's
//! before_tick reads the drawn meshes and their world matrices once a tick,
//! so a raycast from inside a system sees them as the tick started. It is
//! the same read as og.render.raycast outside a system, timed here through
//! that call with `layers: 0`, which tests no mesh: what is left is the
//! hierarchy's refresh and the list of targets.
//!
//! - `perf_raycast_record_20k` (ignored, the perf lane: `cargo test -p
//!   og-runtime --release -- --ignored perf`): 20,000 meshes (a fifth of
//!   them parented under another), nothing moving, against
//!   `crates/og-runtime/perf/<cpu>/raycast-record-20k.json`; the
//!   refresh keeps every world matrix that cannot have moved, so this is
//!   the read of the columns and the rows.
//! - `perf_raycast_record_20k_moving`: the same field with every mesh
//!   moved before each read, the refresh's full cost (every matrix
//!   composed again), against `raycast-record-20k-moving.json`.
//!
//! 15 % tolerance; `OG_PERF_RECORD=1` records one. The unit test
//! `geometry::tests::the_raycast_record_composes_only_what_moved` checks,
//! in the gate, that only what moved is composed and that the record stays
//! bit for bit a fresh read.

use og_runtime::{
    Dispatch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_render_raycast, og_render_sphere_geometry, og_render_standard_material, og_world_insert,
    og_world_set, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

struct Quiet;

impl Hooks for Quiet {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

const COUNT: usize = 20_000;

/// An engine with 20,000 spheres on a grid, every fifth a child of the
/// one before it; its entities.
fn field() -> (u32, Vec<f64>) {
    set_hooks(Rc::new(Quiet));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let geometry = value(og_render_sphere_geometry(
        w,
        Some(r#"{"radius":0.5}"#.into()),
    ));
    let material = value(og_render_standard_material(w, Some("{}".into())));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let mut all = Vec::with_capacity(COUNT);
    for i in 0..COUNT {
        let (x, z) = ((i % 200) as f64 * 1.2, (i / 200) as f64 * 1.2);
        let at = json!({ "position": { "x": x, "y": 0, "z": z } }).to_string();
        let e = value(og_world_spawn(w, Some(at))).as_f64().unwrap();
        value(og_world_insert(w, e, "og.Mesh".into(), Some(mesh.clone())));
        if i % 5 == 4 {
            let parent = json!({ "entity": all[i - 1] }).to_string();
            value(og_world_insert(w, e, "og.Parent".into(), Some(parent)));
        }
        all.push(e);
    }
    for _ in 0..3 {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
    (w, all)
}

/// The best of `runs` reads, in milliseconds; with `moving`, every mesh is
/// moved (untimed) before each.
fn best_read_ms(w: u32, all: &[f64], runs: usize, moving: bool) -> f64 {
    let ray =
        json!({ "origin": { "x": 0, "y": 10, "z": 0 }, "direction": { "x": 0, "y": -1, "z": 0 } })
            .to_string();
    let desc = Some(json!({ "layers": 0 }).to_string());
    let read = || {
        let t = Instant::now();
        let hits = value(og_render_raycast(w, ray.clone(), desc.clone()));
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(hits, json!([]), "layers 0 tests no mesh");
        ms
    };
    read();
    let mut best = f64::INFINITY;
    for run in 0..runs {
        if moving {
            let y = 0.01 * (run + 1) as f64;
            for &e in all {
                let at = json!({ "position": { "x": e % 97.0, "y": y, "z": 0 } }).to_string();
                value(og_world_set(w, e, "og.Transform".into(), at));
            }
        }
        best = best.min(read());
    }
    best
}

fn cpu_slug() -> String {
    let brand = if cfg!(target_os = "macos") {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
    }
    .unwrap_or_else(|| "unknown".into());
    let mut out = String::new();
    for c in brand.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn perf(scene: &str, moving: bool) {
    if cfg!(debug_assertions) {
        panic!(
            "perf tests measure release builds: cargo test -p og-runtime --release -- --ignored perf"
        );
    }
    let (w, all) = field();
    let ms = best_read_ms(w, &all, if moving { 7 } else { 50 }, moving);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("perf")
        .join(cpu_slug());
    let path = dir.join(format!("{scene}.json"));
    println!("perf {scene}: best {ms:.3} ms a read of {COUNT} meshes");
    if std::env::var_os("OG_PERF_RECORD").is_some() {
        std::fs::create_dir_all(&dir).expect("perf dir");
        let text = format!(
            "{{\n  \"scene\": \"{scene}\",\n  \"cpu\": \"{}\",\n  \"meshes\": {COUNT},\n  \"read_ms\": {ms:.3}\n}}\n",
            cpu_slug()
        );
        std::fs::write(&path, text).expect("write baseline");
        println!("  recorded {}", path.display());
        return;
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "no perf baseline at {} for this machine; record one with OG_PERF_RECORD=1",
            path.display()
        )
    });
    let baseline: f64 = serde_json::from_str::<Value>(&text).expect("baseline JSON")["read_ms"]
        .as_f64()
        .expect("read_ms");
    let limit = baseline * 1.15;
    println!("  baseline {baseline:.3} ms, limit {limit:.3} ms");
    assert!(
        ms <= limit,
        "{scene}: {ms:.3} ms a read exceeds the baseline {baseline:.3} ms by more than 15 %"
    );
}

#[test]
#[ignore]
fn perf_raycast_record_20k() {
    perf("raycast-record-20k", false);
}

#[test]
#[ignore]
fn perf_raycast_record_20k_moving() {
    perf("raycast-record-20k-moving", true);
}
