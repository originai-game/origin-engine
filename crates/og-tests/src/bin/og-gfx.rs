//! Renders gfx scenes headless, for evidence, timing and reference capture.
//!
//!   og-gfx SCENE...|--all [--out frame.png] [--frames N]
//!   og-gfx --all --capture DIR [--frames N]   frames to DIR/<tier>/<scene>.png, stats to DIR/report.txt
//!   og-gfx --costs [--frames N]               the feature-cost table (markdown)
//!   og-gfx --costs-e5 [--frames N]            the E5 rows alone (slow tiers: lavapipe)
//!   og-gfx --costs-p2 [--frames N]            the phase 2 rows alone (terrain, water)
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
    let mut only: Option<&str> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--all" => names.extend(scenes::SCENES.iter().map(|s| s.to_string())),
            "--out" => out = Some(args.next().expect("--out PATH")),
            "--capture" => capture = Some(PathBuf::from(args.next().expect("--capture DIR"))),
            "--costs" => costs = true,
            "--costs-e5" => {
                costs = true;
                only = Some("E5");
            }
            "--costs-p2" => {
                costs = true;
                only = Some("P2");
            }
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
        let table = if let Some(only) = only {
            effect_table(frames.max(10), Some(only))
        } else {
            cost_table(frames.max(100))
        };
        match table {
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
    if std::env::var_os("OG_GFX_DEBUG_AO").is_some() {
        built.scene.debug = og_render::DebugView::AmbientOcclusion;
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
        image.width,
        image.height,
        info.adapter,
        info.antialias,
        stats.draw_calls,
        stats.batches,
        stats.instances,
        stats.culled,
        stats.triangles,
        stats.shadow_passes,
        stats.shadow_casters,
        stats.lights,
        stats.sprites,
        stats.glyphs,
        stats.sprite_draws
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
    ("E1: AO (GTAO)", |g| {
        g.scene.post.ao = Some(og_render::AmbientOcclusion {
            method: og_render::AoMethod::Gtao,
            ..Default::default()
        })
    }),
    ("E1: AO (SSAO)", |g| {
        g.scene.post.ao = Some(og_render::AmbientOcclusion {
            method: og_render::AoMethod::Ssao,
            ..Default::default()
        })
    }),
    ("E1: outline (scene-wide)", |g| {
        g.scene.post.outline = Some(og_render::Outline::default())
    }),
    ("E1: depth of field (low)", |g| {
        g.scene.post.depth_of_field = Some(og_render::DepthOfField::default())
    }),
    ("E1: depth of field (high)", |g| {
        g.scene.post.depth_of_field = Some(og_render::DepthOfField {
            quality: og_render::DofQuality::High,
            ..Default::default()
        })
    }),
    ("E1: motion blur", |g| {
        g.scene.post.motion_blur = Some(og_render::MotionBlur::default())
    }),
    ("E1: god rays", |g| {
        g.scene.post.god_rays = Some(og_render::GodRays::default())
    }),
    ("E1: grading (LUT, vignette, aberration, grain)", |g| {
        og_tests::fx_scenes::grade(&mut g.scene)
    }),
    ("E1: 8 decals", |g| {
        let c = g.scene.camera.target;
        for i in 0..8 {
            let p = c + glam::Vec3::new(i as f32 - 4.0, 0.0, (i % 3) as f32 - 1.0);
            g.scene.decals.push(og_render::Decal::on_surface(
                p,
                glam::Vec3::Y,
                glam::Vec2::splat(2.0),
                1.0,
                i as f32,
            ));
        }
    }),
    ("E1: toon + rim on every material", |g| {
        let handles: Vec<_> = g.scene.material_handles().collect();
        for h in handles {
            if let Some(m) = g.scene.standard_material(h).copied() {
                g.scene.set_standard_material(
                    h,
                    og_render::StandardMaterial {
                        toon: 1,
                        rimIntensity: 0.5,
                        ..m
                    },
                );
            }
        }
    }),
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

/// The effect scenes, each against itself with the effect removed (`only`:
/// the rows of one round, "E5" or "P2", with their own header).
fn effect_table(frames: u32, only: Option<&str>) -> Result<String, String> {
    let mut table = String::new();
    if let Some(round) = only {
        let probe = Renderer::new_blocking(&scenes::base_desc()).map_err(|e| e.to_string())?;
        let _ = writeln!(
            table,
            "{round} feature costs on {} ({}), 1280x720, best of 5 runs of {frames} frames (`og-gfx --costs-{}`).\n",
            probe.info().adapter,
            probe.info().backend_tier(),
            round.to_lowercase()
        );
    } else {
        let _ = writeln!(
            table,
            "\nEffect scenes of E1, E5 and phase 2 (each against itself with the effect removed):\n"
        );
    }
    let _ = writeln!(table, "| Scene | ms | without the effect | Δ | draws |");
    let _ = writeln!(table, "|---|---|---|---|---|");
    type Strip = fn(&mut GfxScene);
    let effect_rows: &[(&str, &str, Strip)] = &[
        ("particles", "particles (GPU, 6 systems)", |g| {
            g.scene.particles.clear()
        }),
        ("particles-cpu", "particles (CPU path, 6 systems)", |g| {
            g.scene.particles.clear()
        }),
        ("fx-2d", "2D lights + soft shadows", |g| {
            g.scene.lighting_2d = None;
        }),
        ("fx-2d", "2D glow buffer", |g| g.scene.post.glow = None),
        ("trails-beams", "trails, ribbon, beams", |g| {
            g.scene.trails.clear();
            g.scene.ribbons.clear();
            g.scene.beams.clear();
        }),
        ("decals", "5 decals", |g| g.scene.decals.clear()),
        ("stylised", "outlines (prepass + pass)", |g| {
            g.scene.post.outline = None;
            let handles: Vec<_> = g.scene.material_handles().collect();
            for h in handles {
                if let Some(m) = g.scene.standard_material(h).copied() {
                    g.scene.set_standard_material(
                        h,
                        og_render::StandardMaterial {
                            outlineWidth: 0.0,
                            ..m
                        },
                    );
                }
            }
        }),
        ("skinning", "5 skinned instances (vs none)", |g| {
            g.scene.skinned_instances.clear()
        }),
        // The E5 round.
        ("atmosphere-fog", "E5: fog (distance and height)", |g| {
            g.scene.fog = None
        }),
        (
            "atmosphere-dusk",
            "E5: sky look drawn per pixel (vs the cubemap)",
            |g| g.scene.environment.background = og_render::Background::Sky,
        ),
        (
            "water-3d",
            "E5: planar water (refraction, foam, SSR on the full tier)",
            |g| g.scene.water.clear(),
        ),
        ("water-2d", "E5: 2D water line", |g| {
            g.scene.water_2d.clear()
        }),
        (
            "fluid-2d",
            "E5: 2D fluids as metaballs (2 fluids, 1,500 particles)",
            |g| g.scene.metaballs.clear(),
        ),
        (
            "shader-hooks",
            "E5: material shader hooks (3 dissolving, 900 grass blades)",
            |g| {
                let handles: Vec<_> = g.scene.material_handles().collect();
                for h in handles {
                    g.scene.set_material_shader(h, None);
                }
            },
        ),
        (
            "weather-rain",
            "E5: rain preset (particles, fog, dimming)",
            |g| {
                g.scene.particles.clear();
                g.scene.fog = None;
            },
        ),
        // Phase 2: terrain and water.
        (
            "terrain-near",
            "P2: terrain from 1.7 m (2 km, 4 layers, clipmap)",
            |g| g.scene.terrains.clear(),
        ),
        ("terrain-mid", "P2: terrain from 120 m", |g| {
            g.scene.terrains.clear()
        }),
        ("terrain-far", "P2: terrain from 1.5 km", |g| {
            g.scene.terrains.clear()
        }),
        (
            "terrain-mid",
            "P2: terrain casting shadows (its shadow passes)",
            |g| {
                for t in &mut g.scene.terrains {
                    t.cast_shadows = false;
                }
            },
        ),
        (
            "water-planar",
            "P2: planar reflection (half resolution)",
            |g| {
                for w in &mut g.scene.water {
                    w.planar_reflection = 0.0;
                }
            },
        ),
        ("water-shadows", "P2: shadows received on water", |g| {
            for w in &mut g.scene.water {
                w.shadows = 0.0;
            }
        }),
        ("water-underwater", "P2: caustics on the bed", |g| {
            for w in &mut g.scene.water {
                w.caustics = 0.0;
            }
        }),
        (
            "water-underwater",
            "P2: underwater view (fog, the surface from below; vs no water)",
            |g| g.scene.water.clear(),
        ),
        (
            "terrain-grass",
            "P2: grass scatter (36,000 tufts, shadowed)",
            |g| g.scene.scatters.clear(),
        ),
    ];
    for (scene, label, strip) in effect_rows {
        if only.is_some_and(|round| !label.starts_with(round)) {
            continue;
        }
        let mut with = scenes::build(scene)?;
        let (ms, stats) = best_ms(&mut with, frames)?;
        let mut without = scenes::build(scene)?;
        strip(&mut without);
        let (ms0, _) = best_ms(&mut without, frames)?;
        let _ = writeln!(
            table,
            "| `{scene}`: {label} | {ms:.3} | {ms0:.3} | {:+.3} | {} |",
            ms - ms0,
            stats.draw_calls
        );
    }
    Ok(table)
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
    table.push_str(&effect_table(frames, None)?);
    let mut text = scenes::build("text-msdf")?;
    let (ms, stats) = best_ms(&mut text, frames)?;
    let _ = writeln!(
        table,
        "`text-msdf` ({} glyphs): {ms:.3} ms per frame, {} draws.",
        stats.glyphs, stats.draw_calls
    );
    Ok(table)
}
