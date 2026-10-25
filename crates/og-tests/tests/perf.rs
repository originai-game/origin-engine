//! Frame-time baselines (`docs/ci.md`: the nightly runs
//! `cargo test -p og-tests --release -- --ignored perf`).
//!
//! Each test renders a gfx scene (running its per-frame update, e.g. the
//! moving sprites), times 7 runs of 100 frames and compares the best run
//! with `crates/og-tests/perf/<tier>-<adapter>/<scene>.json`: more than 15 %
//! slower fails. The best run, not the median: on a machine shared with
//! other GPU and CPU work it is the steadiest estimate of the scene's cost. A missing baseline fails too; record one on the machine it
//! belongs to with `OG_PERF_RECORD=1`.

use og_render::{RenderTarget, Renderer};
use og_tests::scenes;
use std::path::PathBuf;
use std::time::Instant;

const TOLERANCE: f64 = 0.15;
const RUNS: usize = 7;
const FRAMES: u32 = 100;

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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("perf")
        .join(format!("{}-{}", info.backend_tier(), slug(&info.adapter)));
    let path = dir.join(format!("{scene}.json"));
    let runs_text: Vec<String> = runs.iter().map(|r| format!("{r:.3}")).collect();
    println!(
        "perf {scene} [{} on {}]: best {ms:.3} ms per frame (runs: {})",
        info.backend_tier(),
        info.adapter,
        runs_text.join(", ")
    );
    if std::env::var_os("OG_PERF_RECORD").is_some() {
        std::fs::create_dir_all(&dir).expect("perf dir");
        let json = format!(
            "{{\n  \"scene\": \"{scene}\",\n  \"tier\": \"{}\",\n  \"adapter\": \"{}\",\n  \"width\": {},\n  \"height\": {},\n  \"frames_per_run\": {FRAMES},\n  \"runs_ms\": [{}],\n  \"frame_ms\": {ms:.3}\n}}\n",
            info.backend_tier(),
            info.adapter,
            built.desc.width,
            built.desc.height,
            runs_text.join(", ")
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
    assert!(
        ms <= limit,
        "{scene}: {ms:.3} ms per frame exceeds the baseline {baseline:.3} ms by more than {:.0} %",
        TOLERANCE * 100.0
    );
    (ms, info.adapter.clone())
}

#[test]
#[ignore]
fn perf_gltf_reference() {
    perf("gltf-reference");
}

#[test]
#[ignore]
fn perf_spheres_20k() {
    perf("spheres-20k");
}

/// The per-instance cost when every transform changes each frame (owed
/// row 50): the sphere field bobbing, every instance uploaded again.
#[test]
#[ignore]
fn perf_spheres_20k_moving() {
    perf("spheres-20k-moving");
}

/// The M3 budget: 20,000 moving sprites in at most 2 ms on the M4 Pro.
#[test]
#[ignore]
fn perf_sprites_20k() {
    perf("sprites-20k");
}

/// Phase 2's Forward+ pair: the night scene's seven lights and the
/// 256-light plaza.
#[test]
#[ignore]
fn perf_point_spot_lights() {
    perf("point-spot-lights");
}

#[test]
#[ignore]
fn perf_lights_256() {
    perf("lights-256");
}

/// Five transmissive objects: every frame copies the opaque scene and
/// fills its mip chain for them to refract. The copy's view and its
/// chain's views and bind groups are made with the copy, not each frame
/// (tests/evidence/r4/perf-ui/NOTES.md: a view made each frame of a
/// texture still in flight waited for the GPU on lavapipe).
#[test]
#[ignore]
fn perf_materials_physical() {
    perf("materials-physical");
}

#[test]
#[ignore]
fn perf_showcase() {
    perf("showcase");
}

/// The E4 budget: a 256x256 map with four layers (one animated, one at half
/// parallax) panned at 1280x720 in under 1 ms per frame on the M4 Pro.
#[test]
#[ignore]
fn perf_tilemap_256() {
    let (ms, adapter) = perf("tilemap-256");
    if adapter.contains("M4 Pro") {
        assert!(
            ms < 1.0,
            "tilemap-256 takes {ms:.3} ms per frame on {adapter}: the budget is 1 ms"
        );
    }
}
