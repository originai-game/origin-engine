//! Frame-time baselines (`docs/ci.md`: the nightly runs
//! `cargo test -p og-tests --release -- --ignored perf`).
//!
//! Each test renders a gfx scene (running its per-frame update, e.g. the
//! moving sprites), times 7 runs of 100 frames and compares the best run
//! with `crates/og-tests/perf/<tier>-<adapter>/<scene>.json`: more than 15 %
//! slower fails. The best run, not the median: on a machine shared with
//! other GPU and CPU work it is the steadiest estimate of the scene's cost. A missing baseline fails too; record one on the machine it
//! belongs to with `OG_PERF_RECORD=1`.
//!
//! **Per-pass budgets.** Where the device has timestamp queries, the
//! baseline also holds each render-graph pass's GPU milliseconds a frame
//! (`passes_ms`, measured after the timed runs so the queries cost them
//! nothing), and a pass that takes more than its baseline by the same
//! 15 % and [`PASS_SLACK_MS`] fails by name: a frame that got slower says
//! which pass did, and a pass that grew while another shrank is not hidden
//! in an unchanged frame time. A pass the baseline does not know is held
//! to the slack alone. A baseline recorded before this has no `passes_ms`
//! and compares the frame only.

use og_render::{RenderTarget, Renderer};
use og_tests::scenes;
use std::path::PathBuf;
use std::time::Instant;

const TOLERANCE: f64 = 0.15;
const RUNS: usize = 7;
const FRAMES: u32 = 100;
/// The frames of one run of a scene's pass timings ([`RUNS`] of them).
const PASS_FRAMES: u32 = 20;
/// What a pass may take over its baseline beyond the tolerance (ms): the
/// timestamps of a pass under a tenth of a millisecond vary by more than
/// 15 % of it.
const PASS_SLACK_MS: f64 = 0.1;

/// Each render-graph pass's GPU milliseconds a frame, in the order first
/// met: the best of [`RUNS`] runs of `frames` frames `render` draws, each
/// run a pass's mean over its frames (the best run for the reason the
/// frame's is: what else uses the GPU only ever adds). Empty where the
/// device has no timestamp queries.
fn pass_times(
    renderer: &mut Renderer,
    frames: u32,
    mut render: impl FnMut(&mut Renderer),
) -> Vec<(String, f64)> {
    renderer.set_clock(Some(clock_ms));
    // The first frames after the timer is made are not read.
    for _ in 0..4 {
        render(renderer);
    }
    renderer.gpu_timings(true);
    let mut best: Vec<(String, f64)> = Vec::new();
    for run in 0..RUNS {
        for _ in 0..frames {
            render(renderer);
        }
        let timed = renderer.gpu_timings(true);
        let mut passes: Vec<(String, f64)> = Vec::new();
        for pass in timed.iter().flat_map(|f| &f.passes) {
            match passes.iter_mut().find(|p| p.0 == pass.name) {
                Some(p) => p.1 += pass.ms(),
                None => passes.push((pass.name.clone(), pass.ms())),
            }
        }
        for (name, sum) in passes {
            let ms = sum / timed.len().max(1) as f64;
            match best.iter_mut().find(|p| p.0 == name) {
                Some(p) => p.1 = p.1.min(ms),
                // A pass first met in a later run ran in none before it.
                None if run == 0 => best.push((name, ms)),
                None => best.push((name, 0.0)),
            }
        }
    }
    renderer.set_clock(None);
    best
}

/// `passes` as a baseline's `passes_ms` holds them.
fn passes_json(passes: &[(String, f64)]) -> String {
    let rows: Vec<String> = passes
        .iter()
        .map(|(name, ms)| format!("\"{name}\": {ms:.3}"))
        .collect();
    format!("{{{}}}", rows.join(", "))
}

/// The `passes_ms` of a baseline's text; `None` in one recorded without.
fn baseline_passes(text: &str) -> Option<Vec<(String, f64)>> {
    let key = "\"passes_ms\": {";
    let rest = &text[text.find(key)? + key.len()..];
    let rows = &rest[..rest.find('}')?];
    Some(
        rows.split(',')
            .filter_map(|row| {
                // A pass's name may hold a colon ("cull: crowds").
                let (name, ms) = row.rsplit_once(':')?;
                Some((
                    name.trim().trim_matches('"').to_string(),
                    ms.trim().parse().ok()?,
                ))
            })
            .collect(),
    )
}

/// The measured passes over their budget, each as a line that names it:
/// more than its baseline by [`TOLERANCE`] and [`PASS_SLACK_MS`], a pass
/// the baseline does not know by the slack alone.
fn passes_over(baseline: &[(String, f64)], measured: &[(String, f64)]) -> Vec<String> {
    measured
        .iter()
        .filter_map(|(name, ms)| {
            let was = baseline.iter().find(|b| &b.0 == name).map_or(0.0, |b| b.1);
            let limit = was * (1.0 + TOLERANCE) + PASS_SLACK_MS;
            (*ms > limit)
                .then(|| format!("{name} {ms:.3} ms (baseline {was:.3} ms, limit {limit:.3} ms)"))
        })
        .collect()
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// Times `scene`, checks it against its baseline, and returns the best
/// run's milliseconds per frame and the adapter.
fn perf(scene: &str) -> (f64, String) {
    perf_of(scene, false)
}

/// [`perf`], or with `capture` each frame also read back and encoded as
/// the probe's `capture` does (`read_offscreen`, then `og_render::png`),
/// against `<scene>-capture.json`.
fn perf_of(scene: &str, capture: bool) -> (f64, String) {
    if cfg!(debug_assertions) {
        panic!(
            "perf tests measure release builds: cargo test -p og-tests --release -- --ignored perf"
        );
    }
    let mut built = scenes::build(scene).expect("scene");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let info = renderer.info().clone();
    let mut frame = 0u32;
    let mut render = |renderer: &mut Renderer, built: &mut scenes::GfxScene| {
        if let Some(update) = built.update {
            update(&mut built.scene, frame);
        }
        frame += 1;
        renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
        if capture {
            let image = renderer.read_offscreen().expect("capture");
            std::hint::black_box(og_render::png(&image));
        }
    };
    for _ in 0..built.frames.max(1) + 10 {
        render(&mut renderer, &mut built);
    }
    renderer.wait_idle().expect("idle");
    let mut runs = Vec::new();
    for _ in 0..RUNS {
        let t = Instant::now();
        for _ in 0..FRAMES {
            render(&mut renderer, &mut built);
        }
        renderer.wait_idle().expect("idle");
        runs.push(t.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64);
    }
    runs.sort_by(|a, b| a.total_cmp(b));
    let ms = runs[0];
    // After the timed runs: the queries cost them nothing.
    let passes = pass_times(&mut renderer, PASS_FRAMES, |r| render(r, &mut built));
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("perf")
        .join(format!("{}-{}", info.backend_tier(), slug(&info.adapter)));
    let name = if capture {
        format!("{scene}-capture")
    } else {
        scene.to_string()
    };
    let path = dir.join(format!("{name}.json"));
    let runs_text: Vec<String> = runs.iter().map(|r| format!("{r:.3}")).collect();
    println!(
        "perf {name} [{} on {}]: best {ms:.3} ms per frame (runs: {})",
        info.backend_tier(),
        info.adapter,
        runs_text.join(", ")
    );
    for (pass, ms) in &passes {
        println!("  pass {pass:<28} {ms:.3} ms");
    }
    if std::env::var_os("OG_PERF_RECORD").is_some() {
        std::fs::create_dir_all(&dir).expect("perf dir");
        let json = format!(
            "{{\n  \"scene\": \"{name}\",\n  \"tier\": \"{}\",\n  \"adapter\": \"{}\",\n  \"width\": {},\n  \"height\": {},\n  \"frames_per_run\": {FRAMES},\n  \"runs_ms\": [{}],\n  \"frame_ms\": {ms:.3},\n  \"passes_ms\": {}\n}}\n",
            info.backend_tier(),
            info.adapter,
            built.desc.width,
            built.desc.height,
            runs_text.join(", "),
            passes_json(&passes)
        );
        std::fs::write(&path, json).expect("write baseline");
        println!("  recorded {}", path.display());
        return (ms, info.adapter.clone());
    }
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "no perf baseline at {} for this machine; record one with OG_PERF_RECORD=1",
            path.display()
        )
    });
    let at = text
        .find("\"frame_ms\":")
        .expect("frame_ms in the baseline");
    let rest = &text[at + "\"frame_ms\":".len()..];
    let end = rest.find(['\n', ',', '}']).unwrap_or(rest.len());
    let baseline: f64 = rest[..end].trim().parse().expect("a number");
    let limit = baseline * (1.0 + TOLERANCE);
    println!("  baseline {baseline:.3} ms, limit {limit:.3} ms");
    let over = match (baseline_passes(&text), passes.is_empty()) {
        (Some(was), false) => passes_over(&was, &passes),
        (None, false) => {
            println!("  the baseline holds no passes_ms: the frame alone is compared");
            Vec::new()
        }
        (_, true) => {
            println!("  no timestamp queries on this device: the frame alone is compared");
            Vec::new()
        }
    };
    assert!(
        ms <= limit,
        "{name}: {ms:.3} ms per frame exceeds the baseline {baseline:.3} ms by more than {:.0} %{}",
        TOLERANCE * 100.0,
        if over.is_empty() {
            String::new()
        } else {
            format!("; over their budget: {}", over.join(", "))
        }
    );
    assert!(
        over.is_empty(),
        "{name}: passes over their budget: {}",
        over.join(", ")
    );
    (ms, info.adapter.clone())
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_gltf_reference() {
    perf("gltf-reference");
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_spheres_20k() {
    perf("spheres-20k");
}

/// The per-instance cost when every transform changes each frame (owed
/// row 50): the sphere field bobbing, every instance uploaded again.
#[test]
#[ignore = "requires GPU; perf"]
fn perf_spheres_20k_moving() {
    perf("spheres-20k-moving");
}

/// The M3 budget: 20,000 moving sprites in at most 2 ms on the M4 Pro.
#[test]
#[ignore = "requires GPU; perf"]
fn perf_sprites_20k() {
    perf("sprites-20k");
}

/// Phase 2's Forward+ pair: the night scene's seven lights and the
/// 256-light plaza.
#[test]
#[ignore = "requires GPU; perf"]
fn perf_point_spot_lights() {
    perf("point-spot-lights");
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_lights_256() {
    perf("lights-256");
}

/// Five transmissive objects: every frame copies the opaque scene and
/// fills its mip chain for them to refract. The copy's view and its
/// chain's views and bind groups are made with the copy, not each frame
/// (the former test sample a view made each frame of a
/// texture still in flight waited for the GPU on lavapipe).
#[test]
#[ignore = "requires GPU; perf"]
fn perf_materials_physical() {
    perf("materials-physical");
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_showcase() {
    perf("showcase");
}

/// The capture budget (docs/design.md §7: 30 ms per 1280×720 capture
/// natively): the showcase drawn, read back and encoded as a PNG each
/// frame, as the probe's `capture` and `screenshot` do. Level-6 zlib took
/// about 90 ms of a 103 ms capture of Meridian Loop on the M4 Pro
/// (the former test sample
#[test]
#[ignore = "requires GPU; perf"]
fn perf_showcase_capture() {
    let (ms, adapter) = perf_of("showcase", true);
    if adapter.contains("M4 Pro") {
        assert!(
            ms < 30.0,
            "a showcase capture takes {ms:.3} ms on {adapter}: the budget is 30 ms"
        );
    }
}

/// The E4 budget: a 256x256 map with four layers (one animated, one at half
/// parallax) panned at 1280x720 in under 1 ms per frame on the M4 Pro.
#[test]
#[ignore = "requires GPU; perf"]
fn perf_tilemap_256() {
    let (ms, adapter) = perf("tilemap-256");
    if adapter.contains("M4 Pro") {
        assert!(
            ms < 1.0,
            "tilemap-256 takes {ms:.3} ms per frame on {adapter}: the budget is 1 ms"
        );
    }
}

/// The renderer's clock for pass timings: milliseconds since first read.
fn clock_ms() -> f64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// Where `scene`'s frame goes: each render-graph pass's CPU encoding and
/// GPU time, averaged over 60 frames after a warm-up, and the CPU's time
/// in `render` per frame (submitting without waiting for the GPU).
fn profile(scene: &str) {
    let mut built = scenes::build(scene).expect("scene");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    renderer.set_clock(Some(clock_ms));
    let frames = 60u32;
    let mut cpu: Vec<(String, f64)> = Vec::new();
    let (mut render_ms, mut update_ms) = (0.0, 0.0);
    for frame in 0..frames + 20 {
        let t = Instant::now();
        if let Some(update) = built.update {
            update(&mut built.scene, frame);
        }
        let u = t.elapsed().as_secs_f64() * 1000.0;
        let t = Instant::now();
        renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
        let r = t.elapsed().as_secs_f64() * 1000.0;
        if frame < 20 {
            continue;
        }
        render_ms += r;
        update_ms += u;
        for p in renderer.pass_timings() {
            match cpu.iter_mut().find(|c| c.0 == p.name) {
                Some(c) => c.1 += p.ms,
                None => cpu.push((p.name.clone(), p.ms)),
            }
        }
    }
    let mut frame = frames + 20;
    let gpu = pass_times(&mut renderer, frames, |r| {
        if let Some(update) = built.update {
            update(&mut built.scene, frame);
        }
        frame += 1;
        r.render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
    });
    let n = f64::from(frames);
    println!(
        "profile {scene}: render {:.3} ms CPU per frame, the scene's update {:.3} ms",
        render_ms / n,
        update_ms / n
    );
    let total: f64 = gpu.iter().map(|g| g.1).sum();
    println!("  GPU {total:.3} ms per frame:");
    for (name, ms) in &gpu {
        println!("    {name:<28} {ms:.3}");
    }
    println!("  CPU encoding:");
    for (name, ms) in &cpu {
        if ms / n >= 0.05 {
            println!("    {name:<28} {:.3}", ms / n);
        }
    }
}

/// Crowds at scale: 5,000, 20,000 and 50,000 walkers on one baked walk,
/// every one moving and stepping each frame (fx_scenes/crowds.rs), under
/// TAA and a shadowing sun, culled, given their levels and drawn on the
/// GPU-driven path (og-render's crowd.rs). With `OG_CROWD_PROFILE`, where
/// each frame goes as well. One test a scene, so each measurement is its
/// own exclusive GPU acquisition (scripts/lock.sh).
fn crowd(scene: &str) {
    if std::env::var_os("OG_CROWD_PROFILE").is_some() {
        profile(scene);
    }
    perf(scene);
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_crowd_5k() {
    crowd("crowd-5k");
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_crowd_20k() {
    crowd("crowd-20k");
}

#[test]
#[ignore = "requires GPU; perf"]
fn perf_crowd_50k() {
    crowd("crowd-50k");
}

/// R11's 2D lighting: 64 shadowed lights over 128 occluders, at the default
/// shadow angles, at 4096, and with normal-mapped receivers (for the
/// record: the baselines catch a regression).
#[test]
#[ignore = "requires GPU; perf"]
fn perf_lights_2d() {
    for scene in ["lights-2d-64", "lights-2d-64-4096", "lights-2d-64-normal"] {
        let (ms, adapter) = perf(scene);
        println!("{scene}: {ms:.3} ms per frame on {adapter}");
    }
}

/// A pass is held to its own baseline by name, whatever order the frame
/// ran them in, a pass the baseline does not know to the slack alone, and
/// a baseline's passes read back as they were written (a name may hold a
/// colon).
#[test]
fn a_pass_over_its_budget_is_named() {
    let pass = |name: &str, ms: f64| (name.to_string(), ms);
    let baseline = [
        pass("cull: crowds", 0.3),
        pass("prepass", 6.0),
        pass("main", 8.0),
        pass("taa", 0.05),
    ];
    // main: 8 x 1.15 + 0.1 = 9.3; prepass: 7.0; taa: 0.1575; ssr: 0.1.
    let measured = [
        pass("main", 9.25),
        pass("prepass", 7.1),
        pass("taa", 0.14),
        pass("ssr", 0.3),
        pass("cull: crowds", 0.2),
    ];
    let over = passes_over(&baseline, &measured);
    assert_eq!(over.len(), 2, "{over:?}");
    assert!(over[0].starts_with("prepass 7.100 ms (baseline 6.000 ms"));
    assert!(over[1].starts_with("ssr 0.300 ms (baseline 0.000 ms"));
    let text = format!(
        "{{\n  \"frame_ms\": 16.5,\n  \"passes_ms\": {}\n}}\n",
        passes_json(&baseline)
    );
    assert_eq!(baseline_passes(&text).as_deref(), Some(&baseline[..]));
    assert_eq!(baseline_passes("{\n  \"frame_ms\": 16.5\n}\n"), None);
}
