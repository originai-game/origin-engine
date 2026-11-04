//! GPU-driven culling (og-render's cull_gpu.rs) against the CPU path it
//! replaces for large scenes: the same scene rendered for several frames
//! with `Culling::Gpu` (the GPU path whatever the scene's size, so the
//! small scenes cover every culled pass) and with `Culling::Cpu` gives the
//! same picture (no channel off by more than 2 outside 0.01 % of the
//! pixels, RMSE at most 0.002: far tighter than the gfx references'
//! tolerance, which would hide a missing object), the same frustum counts,
//! no fewer triangles drawn or hidden than the CPU path has in view, the
//! GPU path on every frame, and on the street,
//! where the nearest blocks hide most of the city, the occlusion test
//! hides most of what is in view; behind a wall, the skinned crowd's back
//! rows are hidden as well. The frames and a stats line per scene
//! are written under the test's output directory.

use og_render::{Culling, FrameStats, RenderTarget, Renderer, RgbaImage};
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
    }
}

/// Scenes covering every culled pass: the city (40,000 instances, the
/// street walking so the occluders change each frame), glTF models with
/// cascades, local lights' shadow faces, the planar reflection, MSAA (the
/// multisampled occluder depth), material vertex hooks (never occluded),
/// the showcase's mix, and the CPU lists drawn beside the GPU's: views (a
/// monitor's target, a mirror, split-screen) and reflection probe faces.
const SCENES: &[(&str, u32)] = &[
    ("city-street", 8),
    ("city-aerial", 3),
    ("gltf-reference", 4),
    ("point-spot-lights", 4),
    ("water-planar", 4),
    // The second planar reflection slot culls on the CPU beside the GPU's
    // first (RenderOptions.planarReflections 2).
    ("water-planar-two", 4),
    ("shader-hooks", 4),
    ("showcase", 4),
    // Secondary cameras and probe faces cull on the CPU beside the GPU's
    // passes (their lists append what they draw).
    ("views-monitor", 2),
    ("views-mirror", 2),
    ("views-split", 2),
    ("probe-interior", 2),
    // More passes than the old fixed table's 18: three cascades, six cube
    // point lights' and two spots' faces (41 shadow passes), and four
    // cascades.
    ("shadows-local-8", 2),
    ("shadows-cascades-4", 2),
    // Skinned meshes and baked crowds culled by their posed boxes beside
    // the instances (cull_gpu.rs `SkinnedCull`), the crowd's back rows
    // behind a wall; and object motion blur, which GPU culling takes too.
    ("skinning", 2),
    ("crowd-culling", 3),
    // Crowds (og-render's crowd.rs): 2,000 walkers among buildings,
    // culled, given their levels and drawn on the GPU path (crowd_gpu.rs)
    // against the CPU path's skinned lists, shadows and TAA included.
    ("crowd-2k", 3),
    ("motion-blur", 4),
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
        let (gpu, g) = (&gpu_run.image, &gpu_run.stats);
        let (cpu, c) = (&cpu_run.image, &cpu_run.stats);
        gfx::save_png(&dir.join(format!("{name}.gpu.png")), gpu).expect("save");
        gfx::save_png(&dir.join(format!("{name}.cpu.png")), cpu).expect("save");
        assert!(
            cpu_run.gpu_frames.iter().all(|&x| !x),
            "{name}: Culling::Cpu culls on the CPU every frame: {:?}",
            cpu_run.gpu_frames
        );
        // Every adapter the gate runs on (Metal, Vulkan, lavapipe) has
        // compute and indirect draws.
        assert!(
            gpu_run.gpu_frames.iter().all(|&x| x),
            "{name}: Culling::Gpu culls on the GPU on every frame: {:?}",
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
            "{name} ({frames} frames): gpu draws={} culled={} occluded={} triangles={} in_view_triangles={} occluded_triangles={} shadow_casters={} | cpu draws={} culled={} triangles={} in_view_triangles={} shadow_casters={} | rmse={:.5} outliers={} max_channel_diff={}",
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
        // The crowd's three rows behind the wall (21 cylinders) are hidden
        // by the occlusion test, as the instances behind an occluder are.
        if g.gpu_culling && name == "crowd-culling" && g.occluded < 21 {
            failures.push(format!(
                "crowd-culling: the wall hides 21 of the crowd, but only {} were occluded",
                g.occluded
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

/// `Culling::Auto` takes the GPU path for the city's
/// 40,000 instances and the CPU path for the showcase's few dozen
/// (`og_render::GPU_CULLING_MIN_INSTANCES`).
#[test]
fn auto_culls_only_large_scenes_on_the_gpu() {
    for (name, large) in [("city-street", true), ("showcase", false)] {
        let Run { stats, .. } = render(name, Culling::Auto, 1);
        assert_eq!(
            stats.instances >= og_render::GPU_CULLING_MIN_INSTANCES,
            large,
            "{name}: {} instances",
            stats.instances
        );
        assert_eq!(
            stats.gpu_culling, large,
            "{name}: Culling::Auto culls {} instances on the GPU only from the threshold",
            stats.instances
        );
    }
}

/// A device without `INDIRECT_FIRST_INSTANCE` (some WebGPU adapters)
/// draws crowds through the CPU's skinned lists while the rest of the
/// scene stays GPU-culled, says so through `RenderInfo::gpu_crowds`
/// (og.render.capabilities' `gpuCrowds`), and draws the GPU-driven path's
/// frame: crowd-lods with the feature treated as absent against itself
/// with it, and against the avenue without its crowd (the fallback draws
/// the walkers rather than nothing).
#[test]
fn crowds_without_indirect_first_instance_fall_back_to_the_cpu_lists() {
    let run = |without: og_render::wgpu::Features, crowd: bool| {
        let mut built = scenes::build("crowd-lods").expect("scene");
        built.desc.culling = Culling::Gpu;
        built.desc.without_features = without;
        if !crowd {
            built.scene.crowds.clear();
        }
        let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
        let mut stats = FrameStats::default();
        for _ in 0..built.frames.max(1) {
            stats = renderer
                .render(&built.scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        let image = renderer.read_offscreen().expect("capture");
        (renderer.info().gpu_crowds, stats.gpu_culling, image)
    };
    let none = og_render::wgpu::Features::empty();
    let (gpu_crowds, _, gpu) = run(none, true);
    // Every adapter the gate runs on has the feature.
    assert!(gpu_crowds, "this adapter draws crowds on the GPU");
    let (fallback_crowds, gpu_culled, fallback) =
        run(og_render::wgpu::Features::INDIRECT_FIRST_INSTANCE, true);
    assert!(!fallback_crowds, "without the feature, gpu_crowds is false");
    assert!(gpu_culled, "the rest of the scene stays on the GPU path");
    let (_, _, empty) = run(none, false);
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("output directory");
    gfx::save_png(&dir.join("crowd-lods.fallback.png"), &fallback).expect("save");
    let tight = Tolerance {
        channel: 2,
        max_outlier_fraction: 0.0001,
        rmse: 0.002,
    };
    let same =
        compare(&fallback.pixels, &gpu.pixels, gpu.width, gpu.height, &tight).expect("same size");
    same.check(&tight)
        .unwrap_or_else(|e| panic!("the fallback draws the GPU path's frame: {e}"));
    let crowd = compare(
        &fallback.pixels,
        &empty.pixels,
        gpu.width,
        gpu.height,
        &tight,
    )
    .expect("same size");
    assert!(
        crowd.rmse > 0.05,
        "the fallback draws the crowd (RMSE {} from the avenue without it)",
        crowd.rmse
    );
}

/// Crowd impostors (og-render's crowd.rs, **Impostors**): crowd-lods drawn
/// with its cards, with its coarsest mesh in their place, and without its
/// crowd. Near the camera the cards change nothing (they are the far
/// level); at the gate end they draw people where the meshes did (nearer
/// the meshes' frame than the empty avenue's) but not the meshes' pixels
/// (they are drawn).
#[test]
fn crowd_cards_draw_the_far_members_and_only_them() {
    let run = |cards: bool, crowd: bool| {
        let mut built = scenes::build("crowd-lods").expect("scene");
        built.desc.culling = Culling::Gpu;
        for c in &mut built.scene.crowds {
            if !cards {
                c.impostor = None;
            }
            if !crowd {
                c.members.clear();
            }
        }
        let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
        for _ in 0..built.frames.max(1) {
            renderer
                .render(&built.scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        renderer.read_offscreen().expect("capture")
    };
    let (cards, meshes, empty) = (run(true, true), run(false, true), run(true, false));
    let dir = out_dir();
    std::fs::create_dir_all(&dir).expect("output directory");
    gfx::save_png(&dir.join("crowd-lods.cards.png"), &cards).expect("save");
    gfx::save_png(&dir.join("crowd-lods.meshes.png"), &meshes).expect("save");
    // RMSE over a window of rows and columns, 0 to 1.
    let rmse = |a: &RgbaImage, b: &RgbaImage, (x0, x1): (u32, u32), (y0, y1): (u32, u32)| {
        let mut sum = 0.0f64;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * a.width + x) * 4) as usize;
                for c in 0..3 {
                    let d = (a.pixels[i + c] as f64 - b.pixels[i + c] as f64) / 255.0;
                    sum += d * d;
                }
            }
        }
        (sum / ((x1 - x0) * (y1 - y0) * 3) as f64).sqrt()
    };
    let (w, h) = (cards.width, cards.height);
    // The nearest rows: members a few metres away, full meshes either way.
    let near = rmse(&cards, &meshes, (0, w), (h * 7 / 8, h));
    assert!(near < 0.002, "the cards moved the near crowd (RMSE {near})");
    // Through the gate (the far end of the avenue).
    let gate = ((w * 45 / 100), (w * 55 / 100));
    let far = (h * 42 / 100, h * 48 / 100);
    let changed = rmse(&cards, &meshes, gate, far);
    let to_meshes = changed;
    let to_empty = rmse(&cards, &empty, gate, far);
    assert!(
        changed > 0.003,
        "the far members drew as meshes (RMSE {changed})"
    );
    assert!(
        to_meshes < to_empty * 0.6,
        "the cards draw the far crowd (RMSE {to_meshes} from the meshes, {to_empty} from the empty avenue)"
    );
}

/// The crowd cull's GPU time is reported (the culling cost): on the GPU
/// path its compute pass runs before any graph pass, and the frame's GPU
/// timings list it as their first pass, "cull: crowds", with a time.
#[test]
fn the_crowd_cull_is_timed_as_the_frames_first_gpu_pass() {
    let mut built = scenes::build("crowd-lods").expect("scene");
    built.desc.culling = Culling::Gpu;
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    fn clock() -> f64 {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1000.0
    }
    renderer.set_clock(Some(clock));
    if let Some(why) = renderer.gpu_timing_absent() {
        println!("no GPU pass timing ({why})");
        return;
    }
    renderer.gpu_timings(true);
    for _ in 0..4 {
        renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    let frames = renderer.gpu_timings(true);
    let last = frames.last().expect("a frame timed on the GPU");
    let names: Vec<&str> = last.passes.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names.first(), Some(&"cull: crowds"), "passes: {names:?}");
    let cull = &last.passes[0];
    assert!(
        cull.ms() > 0.0,
        "the cull took GPU time: {:.4} ms",
        cull.ms()
    );
    println!(
        "cull: crowds {:.3} ms of the frame's passes {names:?}",
        cull.ms()
    );
}
