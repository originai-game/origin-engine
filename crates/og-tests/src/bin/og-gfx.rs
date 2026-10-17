//! Renders gfx scenes headless, for evidence, timing and reference capture.
//!
//!   og-gfx SCENE...|--all [--out frame.png] [--frames N]
//!   og-gfx --all --capture DIR [--frames N]   frames to DIR/<tier>/<scene>.png, stats to DIR/report.txt
//!   og-gfx --costs [--frames N]               the feature-cost table (markdown)
//!
//! Prints the backend tier, the frame stats, setup, CPU-submit and frame
//! times, and the comparison with the committed reference when one exists.
//! Frame times run the scene's per-frame update (moving sprites) each frame.
//! `OG_GFX_DEBUG_CASCADES=1` draws the cascade debug view instead.

use og_render::{Antialias, Bloom, Exposure, FrameStats, Light, LightKind, RenderTarget, Renderer};
use og_tests::scenes::GfxScene;
use og_tests::{compare, gfx, scenes};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut names, mut out, mut frames, mut capture, mut costs) =
        (Vec::new(), None, 0u32, None, false);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--all" => names.extend(scenes::SCENES.iter().map(|s| s.to_string())),
            "--out" => out = Some(args.next().expect("--out PATH")),
            "--capture" => capture = Some(PathBuf::from(args.next().expect("--capture DIR"))),
            "--costs" => costs = true,
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--frames N")
            }
            "-h" | "--help" => {
                println!(
                    "og-gfx SCENE...|--all [--out frame.png] [--frames N] [--capture DIR]\nog-gfx --costs [--frames N]\nscenes: {}",
                    scenes::SCENES.join(", ")
                );
                return;
            }
            name => names.push(name.to_string()),
        }
    }
    if costs {
        match cost_table(frames.max(100)) {
            Ok(table) => print!("{table}"),
            Err(e) => {
                eprintln!("og-gfx --costs: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    if names.is_empty() {
        eprintln!(
            "og-gfx: name a scene ({}) or --all",
            scenes::SCENES.join(", ")
        );
        std::process::exit(2);
    }
    let mut failed = false;
    let mut report = String::new();
    for name in &names {
        match run(
            name,
            out.as_deref().filter(|_| names.len() == 1),
            frames,
            capture.as_deref(),
        ) {
            Ok(line) => {
                println!("{line}");
                report.push_str(&line);
                report.push('\n');
            }
            Err(e) => {
                eprintln!("og-gfx {name}: {e}");
                let _ = writeln!(report, "{name}: ERROR {e}");
                failed = true;
            }
        }
    }
    if let Some(dir) = capture {
        let path = dir.join("report.txt");
        if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&path, report)) {
            eprintln!("og-gfx: {}: {e}", path.display());
            failed = true;
        }
    }
    std::process::exit(if failed { 1 } else { 0 });
}

/// Renders `frames` frames of a built scene (after its warm-up frames) and
/// returns the mean wall time and CPU-submit time per frame in ms.
fn time_frames(
    renderer: &mut Renderer,
    built: &mut GfxScene,
    first_frame: u32,
    frames: u32,
) -> Result<(f64, f64), String> {
    let t = Instant::now();
    let mut cpu = 0.0;
    for f in 0..frames {
        if let Some(update) = built.update {
            update(&mut built.scene, first_frame + f);
        }
        let c = Instant::now();
        renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
        cpu += c.elapsed().as_secs_f64();
    }
    renderer.wait_idle().map_err(|e| e.to_string())?;
    Ok((
        t.elapsed().as_secs_f64() * 1000.0 / frames as f64,
        cpu * 1000.0 / frames as f64,
    ))
}

fn run(
    name: &str,
    out: Option<&str>,
    frames: u32,
    capture: Option<&Path>,
) -> Result<String, String> {
    let mut built = scenes::build(name)?;
    if std::env::var_os("OG_GFX_DEBUG_CASCADES").is_some() {
        built.scene.debug = og_render::DebugView::Cascades;
    }
    let t0 = Instant::now();
    let mut renderer = Renderer::new_blocking(&built.desc).map_err(|e| e.to_string())?;
    let mut stats = FrameStats::default();
    for f in 0..built.frames.max(1) {
        if let Some(update) = built.update {
            update(&mut built.scene, f);
        }
        stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
    }
    renderer.wait_idle().map_err(|e| e.to_string())?;
    let setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    // The reference frame is the last of the scene's own frames.
    let image = renderer.read_offscreen().map_err(|e| e.to_string())?;
    let timing = if frames > 0 {
        let first = built.frames.max(1);
        Some(time_frames(&mut renderer, &mut built, first, frames)?)
    } else {
        None
    };
    let info = renderer.info();
    let tier = info.backend_tier();
    let mut line = format!(
        "{name}: {}x{} tier={tier} adapter=\"{}\" aa={:?} draws={} batches={} instances={} culled={} triangles={} shadow_passes={} shadow_casters={} lights={} sprites={} glyphs={} sprite_draws={} setup+first={setup_ms:.1}ms",
        image.width, image.height, info.adapter, info.antialias, stats.draw_calls, stats.batches, stats.instances,
        stats.culled, stats.triangles, stats.shadow_passes, stats.shadow_casters, stats.lights, stats.sprites,
        stats.glyphs, stats.sprite_draws
    );
    if let Some((ms, cpu)) = timing {
        let _ = write!(line, " frame={ms:.3}ms cpu={cpu:.3}ms ({frames} frames)");
    }
    if !built.applies.includes(info.tier) {
        let _ = write!(line, " (no reference on this tier)");
    } else {
        let reference = gfx::reference_path(&tier, name);
        if reference.exists() {
            let expected = gfx::load_png(&reference)?;
            let c = compare::compare(
                &image.pixels,
                &expected.pixels,
                expected.width,
                expected.height,
                &built.tolerance,
            )?;
            let _ = write!(
                line,
                " vs reference: rmse={:.5} max_diff={} outliers={}",
                c.rmse, c.max_channel_diff, c.outliers
            );
        }
        if let Some(dir) = capture {
            gfx::save_png(&dir.join(&tier).join(format!("{name}.png")), &image)?;
        }
    }
    if let Some(path) = out {
        gfx::save_png(Path::new(path), &image)?;
        let _ = write!(line, "\n  -> {path}");
    }
    Ok(line)
}

/// A named change to a scene for the cost table.
type Variant = (&'static str, fn(&mut GfxScene));

/// Four local lights around the camera's target (alternating point and
/// spot), the first two shadowed when `shadowed`.
fn local_lights(g: &mut GfxScene, shadowed: bool) {
    let c = g.scene.camera.target;
    for i in 0..4 {
        let a = i as f32 * std::f32::consts::FRAC_PI_2;
        g.scene.lights.push(Light {
            kind: if i % 2 == 0 {
                LightKind::Point
            } else {
                LightKind::Spot {
                    direction: glam::Vec3::NEG_Y,
                    angle_degrees: 40.0,
                    penumbra: 0.3,
                }
            },
            position: c + glam::Vec3::new(a.cos() * 4.0, 3.0, a.sin() * 4.0),
            color: glam::Vec3::ONE,
            intensity: 20.0,
            range: 12.0,
            cast_shadows: shadowed && i < 2,
        });
    }
}

const VARIANTS: &[Variant] = &[
    ("features off (CSM on)", |_| {}),
    ("no shadows at all", |g| g.scene.light.cast_shadows = false),
    ("TAA", |g| g.desc.antialias = Antialias::Taa),
    ("MSAA 4x", |g| g.desc.antialias = Antialias::Msaa4),
    ("bloom", |g| g.scene.post.bloom = Some(Bloom::default())),
    ("auto exposure", |g| {
        g.scene.camera.exposure = Exposure::auto()
    }),
    ("4 point/spot lights", |g| local_lights(g, false)),
    ("4 lights, 2 shadowed", |g| local_lights(g, true)),
    (
        "all (TAA, bloom, auto exposure, 4 lights / 2 shadowed)",
        |g| {
            g.desc.antialias = Antialias::Taa;
            g.scene.post.bloom = Some(Bloom::default());
            g.scene.camera.exposure = Exposure::auto();
            local_lights(g, true);
        },
    ),
];

/// The best frame time of five runs of `frames` frames, after 30 warm-up
/// frames: on a machine shared with other GPU and CPU work the minimum is the
/// steadiest estimate of the uncontended cost.
fn best_ms(built: &mut GfxScene, frames: u32) -> Result<(f64, FrameStats), String> {
    let mut renderer = Renderer::new_blocking(&built.desc).map_err(|e| e.to_string())?;
    let mut stats = FrameStats::default();
    for f in 0..30 {
        if let Some(update) = built.update {
            update(&mut built.scene, f);
        }
        stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .map_err(|e| e.to_string())?;
    }
    let mut best = f64::MAX;
    for r in 0..5 {
        best = best.min(time_frames(&mut renderer, built, 30 + r * frames, frames)?.0);
    }
    Ok((best, stats))
}

fn cost_table(frames: u32) -> Result<String, String> {
    let mut table = String::new();
    let probe = Renderer::new_blocking(&scenes::base_desc()).map_err(|e| e.to_string())?;
    let _ = writeln!(
        table,
        "Feature costs on {} ({}), 1280x720, best of 2 sweeps x 5 runs of {frames} frames (`og-gfx --costs`).\n",
        probe.info().adapter,
        probe.info().backend_tier()
    );
    drop(probe);
    // Bring the GPU to its working clock before the first measurement.
    let mut warm = scenes::build("spheres-20k")?;
    let t = Instant::now();
    while t.elapsed().as_secs_f64() < 1.0 {
        best_ms(&mut warm, 20)?;
    }
    let _ = writeln!(
        table,
        "| Variant | gltf-reference ms | Δ | draws | spheres-20k ms | Δ | draws |"
    );
    let _ = writeln!(table, "|---|---|---|---|---|---|---|");
    // Two sweeps over every cell, keeping each cell's best: slow drift from
    // other work on the machine hits both sweeps differently.
    let scenes_timed = ["gltf-reference", "spheres-20k"];
    let mut cells = vec![[(f64::MAX, 0u32); 2]; VARIANTS.len()];
    for _sweep in 0..2 {
        for (i, (_, apply)) in VARIANTS.iter().enumerate() {
            for (k, scene) in scenes_timed.iter().enumerate() {
                let mut built = scenes::build(scene)?;
                apply(&mut built);
                let (ms, stats) = best_ms(&mut built, frames)?;
                cells[i][k] = (cells[i][k].0.min(ms), stats.draw_calls);
            }
        }
    }
    for (i, (label, _)) in VARIANTS.iter().enumerate() {
        let mut row = format!("| {label} |");
        for (k, &(ms, draws)) in cells[i].iter().enumerate() {
            let delta = if i == 0 {
                String::new()
            } else {
                format!("{:+.3}", ms - cells[0][k].0)
            };
            let _ = write!(row, " {ms:.3} | {delta} | {draws} |");
        }
        let _ = writeln!(table, "{row}");
    }
    let mut sprites = scenes::build("sprites-20k")?;
    let (ms, stats) = best_ms(&mut sprites, frames)?;
    let _ = writeln!(
        table,
        "\n`sprites-20k` (20,000 sprites moved and resubmitted every frame, orthographic): {ms:.3} ms per frame, {} draws ({} sprite draws).",
        stats.draw_calls, stats.sprite_draws
    );
    let mut text = scenes::build("text-msdf")?;
    let (ms, stats) = best_ms(&mut text, frames)?;
    let _ = writeln!(
        table,
        "`text-msdf` ({} glyphs): {ms:.3} ms per frame, {} draws.",
        stats.glyphs, stats.draw_calls
    );
    Ok(table)
}
