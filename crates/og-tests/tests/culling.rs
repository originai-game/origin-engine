//! GPU-driven culling (og-render's cull_gpu.rs) against the CPU path it
//! replaces on the full tier: the same scene rendered for several frames
//! with `Culling::Auto` and with `Culling::Cpu` gives the same picture
//! (the gfx references' tolerance), the same frustum counts, and on the
//! street, where the nearest blocks hide most of the city, the occlusion
//! test hides most of what is in view. The frames and a stats line per
//! scene are written under the test's output directory.

use og_render::{Culling, FrameStats, RenderTarget, Renderer, RgbaImage, Tier};
use og_tests::compare::{Tolerance, compare};
use og_tests::gfx;
use og_tests::scenes;
use std::fmt::Write as _;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-culling")
}

/// Renders `frames` frames of the scene `name` with `culling` and returns
/// the last one with its exact statistics and the adapter's tier.
fn render(name: &str, culling: Culling, frames: u32) -> (RgbaImage, FrameStats, Tier) {
    let mut built = scenes::build(name).expect("scene");
    built.desc.culling = culling;
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let mut stats = FrameStats::default();
    for f in 0..frames {
        if let Some(update) = built.update {
            update(&mut built.scene, f);
        }
        stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    renderer.finish_stats(&mut stats);
    let image = renderer.read_offscreen().expect("capture");
    (image, stats, renderer.info().tier)
}

/// Scenes covering every culled pass: the city (40,000 instances, the
/// street walking so the occluders change each frame), glTF models with
/// cascades, local lights' shadow faces, the planar reflection, MSAA (the
/// multisampled occluder depth), material vertex hooks (never occluded)
/// and the showcase's mix.
const SCENES: &[(&str, u32)] = &[
    ("city-street", 8),
    ("city-aerial", 3),
    ("gltf-reference", 4),
    ("point-spot-lights", 4),
    ("water-planar", 4),
    ("msaa", 4),
    ("shader-hooks", 4),
    ("showcase", 4),
];

#[test]
fn gpu_culling_draws_the_cpu_paths_frame() {
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("output directory");
    let mut log = String::new();
    let mut failures = Vec::new();
    for &(name, frames) in SCENES {
        let (gpu, g, tier) = render(name, Culling::Auto, frames);
        let (cpu, c, _) = render(name, Culling::Cpu, frames);
        gfx::save_png(&dir.join(format!("{name}.gpu.png")), &gpu).expect("save");
        gfx::save_png(&dir.join(format!("{name}.cpu.png")), &cpu).expect("save");
        assert!(!c.gpu_culling, "{name}: Culling::Cpu culls on the CPU");
        // Every full-tier adapter the gate runs on (Metal, Vulkan, lavapipe)
        // has compute and indirect draws.
        assert_eq!(
            g.gpu_culling,
            tier == Tier::Full,
            "{name}: Culling::Auto culls on the GPU exactly on the full tier"
        );
        let tolerance = Tolerance::default();
        let diff = compare(&gpu.pixels, &cpu.pixels, cpu.width, cpu.height, &tolerance)
            .expect("same size");
        let _ = writeln!(
            log,
            "{name} ({frames} frames, {tier:?}): gpu draws={} culled={} occluded={} triangles={} shadow_casters={} | cpu draws={} culled={} triangles={} shadow_casters={} | rmse={:.5} outliers={}",
            g.draw_calls,
            g.culled,
            g.occluded,
            g.triangles,
            g.shadow_casters,
            c.draw_calls,
            c.culled,
            c.triangles,
            c.shadow_casters,
            diff.rmse,
            diff.outliers
        );
        if let Err(e) = diff.check(&tolerance) {
            failures.push(format!("{name}: GPU and CPU culling frames differ: {e}"));
        }
        // The frustum test is the same sphere test on both paths.
        if g.culled != c.culled {
            failures.push(format!(
                "{name}: culled {} on the GPU, {} on the CPU",
                g.culled, c.culled
            ));
        }
        // The CPU draws a batch whole when nine tenths of it is visible;
        // the GPU draws exactly the visible instances.
        if g.triangles > c.triangles || g.shadow_casters > c.shadow_casters {
            failures.push(format!(
                "{name}: the GPU drew more (triangles {} > {} or shadow casters {} > {})",
                g.triangles, c.triangles, g.shadow_casters, c.shadow_casters
            ));
        }
        if g.gpu_culling && name == "city-street" {
            let in_view = g.instances - g.culled;
            if g.occluded * 2 < in_view {
                failures.push(format!(
                    "city-street: the nearest blocks hide most of the city in view, but only {} of {in_view} were occluded",
                    g.occluded
                ));
            }
        }
    }
    std::fs::write(dir.join("report.txt"), &log).expect("report");
    print!("{log}");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
