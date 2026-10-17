//! Renders gfx scenes headless, for evidence and timing.
//!
//!   og-gfx SCENE|--all [--out frame.png] [--frames N]
//!
//! Prints the backend tier, the frame stats, setup and frame times, and the
//! comparison with the committed reference when one exists.

use og_render::{RenderTarget, Renderer};
use og_tests::{compare, gfx, scenes};
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut names, mut out, mut frames) = (Vec::new(), None, 0u32);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--all" => names.extend(scenes::SCENES.iter().map(|s| s.to_string())),
            "--out" => out = Some(args.next().expect("--out PATH")),
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--frames N")
            }
            "-h" | "--help" => {
                println!(
                    "og-gfx SCENE|--all [--out frame.png] [--frames N]\nscenes: {}",
                    scenes::SCENES.join(", ")
                );
                return;
            }
            name => names.push(name.to_string()),
        }
    }
    if names.is_empty() {
        eprintln!(
            "og-gfx: name a scene ({}) or --all",
            scenes::SCENES.join(", ")
        );
        std::process::exit(2);
    }
    let mut failed = false;
    for name in &names {
        if let Err(e) = run(name, out.as_deref().filter(|_| names.len() == 1), frames) {
            eprintln!("og-gfx {name}: {e}");
            failed = true;
        }
    }
    std::process::exit(if failed { 1 } else { 0 });
}

fn run(name: &str, out: Option<&str>, frames: u32) -> Result<(), String> {
    let built = scenes::build(name)?;
    let t0 = Instant::now();
    let mut renderer = Renderer::new_blocking(&built.desc).map_err(|e| e.to_string())?;
    let stats = renderer
        .render(&built.scene, &RenderTarget::Offscreen)
        .map_err(|e| e.to_string())?;
    renderer.wait_idle().map_err(|e| e.to_string())?;
    let setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let mut frame_ms = None;
    if frames > 0 {
        let t1 = Instant::now();
        for _ in 0..frames {
            renderer
                .render(&built.scene, &RenderTarget::Offscreen)
                .map_err(|e| e.to_string())?;
        }
        renderer.wait_idle().map_err(|e| e.to_string())?;
        frame_ms = Some(t1.elapsed().as_secs_f64() * 1000.0 / frames as f64);
    }
    let image = renderer.read_offscreen().map_err(|e| e.to_string())?;
    let info = renderer.info();
    let tier = info.backend_tier();
    print!(
        "{name}: {}x{} tier={tier} adapter=\"{}\" draws={} batches={} instances={} triangles={} setup+first={setup_ms:.1}ms",
        image.width, image.height, info.adapter, stats.draw_calls, stats.batches, stats.instances, stats.triangles
    );
    if let Some(ms) = frame_ms {
        print!(" frame={ms:.3}ms ({frames} frames)");
    }
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
        print!(
            " vs reference: rmse={:.5} max_diff={} outliers={}",
            c.rmse, c.max_channel_diff, c.outliers
        );
    }
    println!();
    if let Some(path) = out {
        gfx::save_png(std::path::Path::new(path), &image)?;
        println!("  -> {path}");
    }
    Ok(())
}
