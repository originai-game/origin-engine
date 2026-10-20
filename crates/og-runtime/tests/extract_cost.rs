//! The engine's per-instance cost (owed row 50, docs/phase-1.md): a field of
//! og.Mesh entities is extracted into the renderer's scene every frame, and
//! that must stay a pass over the columns. 20,000 moving spheres once took
//! 174 ms a frame on the web because extraction decoded every og.Mesh row to
//! JSON on the lanes (tests/evidence/P4/perf/).
//!
//! - `extraction_is_linear_and_never_decodes_meshes_on_the_lanes` (in the
//!   gate): the frame of 20,000 meshes costs per instance at most three times
//!   the frame of 2,000 and under 40 ms (best of several runs, so load on the
//!   machine does not decide it), and natively no og.Mesh row reaches the
//!   lanes' extraction site. Broken once by putting back 5dc3aa0's
//!   extraction: 46 lane calls at the site, 102 ms for 20,000 meshes.
//! - `perf_extract_spheres_20k` (ignored, the perf lane: `cargo test -p
//!   og-runtime --release -- --ignored perf`): the 20,000-mesh frame against
//!   `crates/og-runtime/perf/<cpu>/extract-spheres-20k.json`, 15 % tolerance;
//!   `OG_PERF_RECORD=1` records one.

use og_runtime::{
    Dispatch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_render_sphere_geometry, og_render_standard_material, og_world_insert,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

struct Clock(Instant);

impl Hooks for Clock {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn now_ms(&self) -> f64 {
        self.0.elapsed().as_secs_f64() * 1000.0
    }
}

/// A renderer that draws nothing and remembers how many instances it was handed.
struct Counting(Rc<Cell<usize>>);

impl RenderBackend for Counting {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (1280, 720)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.0.set(scene.instances.len());
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// An engine with `count` spheres (one geometry, one material) on a grid,
/// and the counter of what its renderer was handed.
fn field(count: usize) -> (u32, Rc<Cell<usize>>) {
    set_hooks(Rc::new(Clock(Instant::now())));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, "{}".into()))
        .as_u64()
        .expect("engine id") as u32;
    let world = engine;
    let geometry = value(og_render_sphere_geometry(
        world,
        Some(r#"{"radius":0.5,"widthSegments":16,"heightSegments":12}"#.into()),
    ));
    let material = value(og_render_standard_material(world, Some("{}".into())));
    for i in 0..count {
        let (x, z) = ((i % 200) as f64 * 1.2, (i / 200) as f64 * 1.2);
        let desc = json!({ "position": { "x": x, "y": 0, "z": z } }).to_string();
        let e = value(og_world_spawn(world, Some(desc)));
        let mesh = json!({ "geometry": geometry, "material": material }).to_string();
        value(og_world_insert(
            world,
            e.as_f64().expect("entity"),
            "og.Mesh".into(),
            Some(mesh),
        ));
    }
    let seen = Rc::new(Cell::new(0));
    set_render_backend(engine, Some(Box::new(Counting(seen.clone())))).expect("backend");
    (engine, seen)
}

/// The best of `runs` runs of `frames` frames, in milliseconds per frame.
fn best_frame_ms(engine: u32, runs: usize, frames: usize) -> f64 {
    for _ in 0..3 {
        value_ok(og_engine_frame(engine, 1000.0 / 60.0));
    }
    let mut best = f64::INFINITY;
    for _ in 0..runs {
        let t = Instant::now();
        for _ in 0..frames {
            value_ok(og_engine_frame(engine, 1000.0 / 60.0));
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / frames as f64);
    }
    best
}

fn value_ok(text: String) {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
}

fn extract_calls() -> u64 {
    og_lanes::report().sites[og_lanes::Site::Extract as usize].calls
}

#[test]
fn extraction_is_linear_and_never_decodes_meshes_on_the_lanes() {
    let (small, small_seen) = field(2_000);
    let (large, large_seen) = field(20_000);
    let before = extract_calls();
    let small_ms = best_frame_ms(small, 5, 4);
    let large_ms = best_frame_ms(large, 5, 4);
    assert_eq!(small_seen.get(), 2_000, "every sphere is an instance");
    assert_eq!(large_seen.get(), 20_000, "every sphere is an instance");
    let (per_small, per_large) = (small_ms / 2_000.0, large_ms / 20_000.0);
    println!(
        "2,000 meshes {small_ms:.3} ms a frame ({:.3} us each), 20,000 {large_ms:.3} ms ({:.3} us each)",
        per_small * 1000.0,
        per_large * 1000.0
    );
    assert!(
        per_large <= per_small * 3.0,
        "an instance of 20,000 costs {:.3} us, of 2,000 {:.3} us: extraction is no longer linear",
        per_large * 1000.0,
        per_small * 1000.0
    );
    // 3.8 ms in the dev profile on an M4 Pro; the JSON rows on the lanes took 102 ms.
    assert!(
        large_ms < 40.0,
        "20,000 meshes take {large_ms:.3} ms a frame to extract (3.8 ms in the dev profile on an M4 Pro)"
    );
    assert_eq!(
        extract_calls(),
        before,
        "og.Mesh rows went through the lanes' extraction site: they are read from the columns"
    );
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

#[test]
#[ignore]
fn perf_extract_spheres_20k() {
    if cfg!(debug_assertions) {
        panic!(
            "perf tests measure release builds: cargo test -p og-runtime --release -- --ignored perf"
        );
    }
    let (engine, seen) = field(20_000);
    let ms = best_frame_ms(engine, 7, 50);
    assert_eq!(seen.get(), 20_000);
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("perf")
        .join(cpu_slug());
    let path = dir.join("extract-spheres-20k.json");
    println!(
        "perf extract-spheres-20k: best {ms:.3} ms per frame ({:.3} us an instance)",
        ms * 1000.0 / 20_000.0
    );
    if std::env::var_os("OG_PERF_RECORD").is_some() {
        std::fs::create_dir_all(&dir).expect("perf dir");
        let text = format!(
            "{{\n  \"scene\": \"extract-spheres-20k\",\n  \"cpu\": \"{}\",\n  \"instances\": 20000,\n  \"frame_ms\": {ms:.3}\n}}\n",
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
    let baseline: f64 = serde_json::from_str::<Value>(&text).expect("baseline JSON")["frame_ms"]
        .as_f64()
        .expect("frame_ms");
    let limit = baseline * 1.15;
    println!("  baseline {baseline:.3} ms, limit {limit:.3} ms");
    assert!(
        ms <= limit,
        "extract-spheres-20k: {ms:.3} ms per frame exceeds the baseline {baseline:.3} ms by more than 15 %"
    );
}
