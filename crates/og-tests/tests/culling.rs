//! GPU-driven culling (og-render's cull_gpu.rs) against the CPU path it
//! replaces on the full tier: the same scene rendered for several frames
//! with `Culling::Gpu` (the GPU path whatever the scene's size, so the
//! small scenes cover every culled pass) and with `Culling::Cpu` gives the
//! same picture (no channel off by more than 2 outside 0.01 % of the
//! pixels, RMSE at most 0.002: far tighter than the gfx references'
//! tolerance, which would hide a missing object), the same frustum counts,
//! no fewer triangles drawn or hidden than the CPU path has in view, the
//! GPU path on every frame exactly on the full tier, and on the street,
//! where the nearest blocks hide most of the city, the occlusion test
//! hides most of what is in view. The frames and a stats line per scene
//! are written under the test's output directory.

use og_render::{Culling, FrameStats, RenderTarget, Renderer, RgbaImage, Tier};
use og_tests::compare::{Tolerance, compare};
use og_tests::gfx;
use og_tests::scenes;
use std::fmt::Write as _;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-culling")
}

/// One scene rendered for some frames.
struct Run {
    /// The last frame and its exact statistics.
    image: RgbaImage,
    stats: FrameStats,
    /// Whether the GPU culled each frame.
    gpu_frames: Vec<bool>,
    tier: Tier,
}

/// Renders `frames` frames of the scene `name` with `culling`.
fn render(name: &str, culling: Culling, frames: u32) -> Run {
    let mut built = scenes::build(name).expect("scene");
    built.desc.culling = culling;
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    let mut stats = FrameStats::default();
    let mut gpu_frames = Vec::new();
    for f in 0..frames {
        if let Some(update) = built.update {
            update(&mut built.scene, f);
        }
        stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
        gpu_frames.push(stats.gpu_culling);
    }
    renderer.finish_stats(&mut stats);
    let image = renderer.read_offscreen().expect("capture");
    Run {
        image,
        stats,
        gpu_frames,
        tier: renderer.info().tier,
    }
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
        let gpu_run = render(name, Culling::Gpu, frames);
        let cpu_run = render(name, Culling::Cpu, frames);
        let (gpu, g, tier) = (&gpu_run.image, &gpu_run.stats, gpu_run.tier);
        let (cpu, c) = (&cpu_run.image, &cpu_run.stats);
        gfx::save_png(&dir.join(format!("{name}.gpu.png")), gpu).expect("save");
        gfx::save_png(&dir.join(format!("{name}.cpu.png")), cpu).expect("save");
        assert!(
            cpu_run.gpu_frames.iter().all(|&x| !x),
            "{name}: Culling::Cpu culls on the CPU every frame: {:?}",
            cpu_run.gpu_frames
        );
        // Every full-tier adapter the gate runs on (Metal, Vulkan, lavapipe)
        // has compute and indirect draws.
        assert!(
            gpu_run
                .gpu_frames
                .iter()
                .all(|&x| x == (tier == Tier::Full)),
            "{name}: Culling::Gpu culls on the GPU on every frame exactly on the full tier ({tier:?}): {:?}",
            gpu_run.gpu_frames
        );
        // The paths differ only by rounding (instance order within a batch
        // is the same, so depth ties resolve alike).
        let tolerance = Tolerance {
            channel: 2,
            max_outlier_fraction: 0.0001,
            rmse: 0.002,
        };
        let diff = compare(&gpu.pixels, &cpu.pixels, cpu.width, cpu.height, &tolerance)
            .expect("same size");
        let _ = writeln!(
            log,
            "{name} ({frames} frames, {tier:?}): gpu draws={} culled={} occluded={} triangles={} in_view_triangles={} occluded_triangles={} shadow_casters={} | cpu draws={} culled={} triangles={} in_view_triangles={} shadow_casters={} | rmse={:.5} outliers={} max_channel_diff={}",
            g.draw_calls,
            g.culled,
            g.occluded,
            g.triangles,
            g.in_view_triangles,
            g.occluded_triangles,
            g.shadow_casters,
            c.draw_calls,
            c.culled,
            c.triangles,
            c.in_view_triangles,
            c.shadow_casters,
            diff.rmse,
            diff.outliers,
            diff.max_channel_diff
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
        // Every opaque instance the CPU has in view is drawn or counted as
        // hidden on the GPU (the hidden ones are counted on their own, so
        // an instance dropped from both shows here).
        if g.gpu_culling && g.in_view_triangles < c.in_view_triangles {
            failures.push(format!(
                "{name}: the GPU drew {} and hid {} of the triangles in view, fewer than the CPU's {}",
                g.in_view_triangles - g.occluded_triangles,
                g.occluded_triangles,
                c.in_view_triangles
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

/// `Culling::Auto` takes the GPU path on the full tier for the city's
/// 40,000 instances and the CPU path for the showcase's few dozen
/// (`og_render::GPU_CULLING_MIN_INSTANCES`).
#[test]
fn auto_culls_only_large_scenes_on_the_gpu() {
    for (name, large) in [("city-street", true), ("showcase", false)] {
        let Run { stats, tier, .. } = render(name, Culling::Auto, 1);
        assert_eq!(
            stats.instances >= og_render::GPU_CULLING_MIN_INSTANCES,
            large,
            "{name}: {} instances",
            stats.instances
        );
        assert_eq!(
            stats.gpu_culling,
            large && tier == Tier::Full,
            "{name}: Culling::Auto culls {} instances on the GPU only on the full tier and from the threshold",
            stats.instances
        );
    }
}
