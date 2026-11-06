//! FSR 3.1's upscaler (`Upscaler::Fsr3`) in place of the TAA resolve.
//! GPU tests: macrunner runs them on Metal.
//!
//! Ways this could fail: the choice ignored (the frame is TAAU's); a
//! bilinear upscale after FSR (the graph still has `upscale`); motion of
//! the wrong sign or a broken history (the converged frame is no nearer
//! the full-scale one than a plain upscale, or it is black or smeared).

use og_render::{Antialias, RenderDesc, RenderScene, RenderTarget, Renderer, RgbaImage, Upscaler};
use og_tests::scenes;

fn desc(antialias: Antialias) -> RenderDesc {
    RenderDesc {
        width: 320,
        height: 180,
        antialias,
        ..scenes::base_desc()
    }
}

fn draw(renderer: &mut Renderer, scene: &RenderScene, frames: u32) -> RgbaImage {
    for _ in 0..frames.max(1) {
        renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("frame");
    }
    renderer.read_offscreen().expect("capture")
}

fn diff(a: &RgbaImage, b: &RgbaImage) -> f64 {
    assert_eq!((a.width, a.height), (b.width, b.height));
    a.pixels
        .iter()
        .zip(&b.pixels)
        .map(|(x, y)| f64::from(x.abs_diff(*y)))
        .sum::<f64>()
        / a.pixels.len() as f64
}

fn mean(image: &RgbaImage) -> f64 {
    image.pixels.iter().map(|&b| f64::from(b)).sum::<f64>() / image.pixels.len() as f64
}

/// At render scale 0.5 FSR resolves the quarter-pixel scene to the
/// output in the `taa` pass, with no upscale after it; the converged
/// frame is nearer the full-scale TAA frame than a bilinear upscale and
/// differs from TAAU's (the choice is honoured).
#[test]
#[ignore = "requires GPU"]
fn fsr_resolves_the_half_scale_scene_to_the_output() {
    let mut scene = scenes::thin_geometry();
    let mut full = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    assert_eq!(
        full.info().antialias,
        Antialias::Taa,
        "GPU lacks TAA support"
    );
    let reference = draw(&mut full, &scene, 32);

    let mut taau = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    taau.set_render_scale(0.5);
    let by_taau = draw(&mut taau, &scene, 64);

    scene.quality.upscaler = Upscaler::Fsr3;
    let mut fsr = Renderer::new_blocking(&desc(Antialias::Taa)).expect("renderer");
    fsr.set_render_scale(0.5);
    let by_fsr = draw(&mut fsr, &scene, 64);
    assert_eq!(fsr.scene_size_drawn(), (160, 90));
    assert_eq!((by_fsr.width, by_fsr.height), (320, 180));
    let names = fsr.frame_graph().expect("a graph").ordered_names();
    assert!(names.contains(&"taa"), "{names:?}");
    assert!(
        !names.contains(&"upscale"),
        "FSR upscales in TAA: {names:?}"
    );
    if let Ok(dir) = std::env::var("OG_FSR_OUT") {
        let dir = std::path::Path::new(&dir);
        og_tests::gfx::save_png(&dir.join("fsr-reference.png"), &reference).unwrap();
        og_tests::gfx::save_png(&dir.join("fsr-taau.png"), &by_taau).unwrap();
        og_tests::gfx::save_png(&dir.join("fsr-fsr.png"), &by_fsr).unwrap();
    }

    let mut plain = Renderer::new_blocking(&desc(Antialias::Off)).expect("renderer");
    plain.set_render_scale(0.5);
    let bilinear = draw(&mut plain, &scene, 1);
    let (d_fsr, d_taau, d_plain) = (
        diff(&by_fsr, &reference),
        diff(&by_taau, &reference),
        diff(&bilinear, &reference),
    );
    println!(
        "mean byte difference from full scale: FSR {d_fsr:.2}, TAAU {d_taau:.2}, bilinear {d_plain:.2}"
    );
    let (m_fsr, m_ref) = (mean(&by_fsr), mean(&reference));
    assert!(
        (m_fsr - m_ref).abs() < 8.0,
        "FSR's mean {m_fsr} is far from the reference's {m_ref}"
    );
    assert!(
        d_fsr < d_plain,
        "FSR {d_fsr} is no nearer than bilinear {d_plain}"
    );
    assert!(
        diff(&by_fsr, &by_taau) > 0.05,
        "FSR's frame is TAAU's: the choice was ignored"
    );
}

// The comparison behind docs/upscalers.md: `OG_FSR_OUT=<dir> cargo test
// -p og-tests --test tests fsr::compare -- --ignored --nocapture` under
// `og_gpu_acquire`, and `fsr::cost` under `og_gpu_acquire_all`.

type Update = fn(&mut RenderScene, u32);

struct Case {
    name: &'static str,
    scene: fn() -> RenderScene,
    update: Option<Update>,
    frames: u32,
    /// The crop judged at 4x: x, y, w, h in output pixels.
    crop: (u32, u32, u32, u32),
}

fn cases() -> Vec<Case> {
    use og_tests::{fx_scenes as fx, grass_scenes, shadow_scenes};
    vec![
        Case {
            name: "thin-static",
            scene: scenes::thin_geometry,
            update: None,
            frames: 128,
            crop: (200, 100, 160, 90),
        },
        Case {
            name: "thin-pan-disocclusion",
            scene: scenes::thin_geometry,
            update: Some(shadow_scenes::pan_and_roll_sphere),
            frames: 48,
            crop: (280, 120, 160, 90),
        },
        Case {
            name: "grass-pan",
            scene: || grass_scenes::grass_scene("near"),
            update: Some(shadow_scenes::pan_thin_geometry),
            frames: 48,
            crop: (240, 260, 160, 90),
        },
        Case {
            name: "particles",
            scene: fx::particles_scene,
            update: Some(fx::tick_update),
            frames: 120,
            crop: (240, 120, 160, 90),
        },
        Case {
            name: "smoke-transparency",
            scene: fx::particles_lit_scene,
            update: Some(fx::tick_update),
            frames: 90,
            crop: (240, 120, 160, 90),
        },
    ]
}

/// Draws `case` and returns its last `keep` frames.
fn run(
    case: &Case,
    antialias: Antialias,
    scale: f32,
    upscaler: Upscaler,
    keep: u32,
) -> Vec<RgbaImage> {
    let mut d = desc(antialias);
    (d.width, d.height) = (640, 360);
    let mut renderer = Renderer::new_blocking(&d).expect("renderer");
    renderer.set_render_scale(scale);
    let mut scene = (case.scene)();
    scene.quality.upscaler = upscaler;
    let mut out = Vec::new();
    for f in 0..case.frames {
        if let Some(u) = case.update {
            u(&mut scene, f);
        }
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("frame");
        if f + keep >= case.frames {
            out.push(renderer.read_offscreen().expect("capture"));
        }
    }
    out
}

fn psnr(a: &RgbaImage, b: &RgbaImage) -> f64 {
    let mse = a
        .pixels
        .chunks(4)
        .zip(b.pixels.chunks(4))
        .flat_map(|(x, y)| (0..3).map(move |c| (f64::from(x[c]) - f64::from(y[c])).powi(2)))
        .sum::<f64>()
        / (a.pixels.len() / 4 * 3) as f64;
    10.0 * (255.0f64.powi(2) / mse.max(1e-9)).log10()
}

/// The frame-to-frame change that the reference does not have: flicker
/// on a still scene, crawl and smear on a moving one.
fn flicker(out: &[RgbaImage], reference: &[RgbaImage]) -> f64 {
    let mut sum = 0.0;
    let mut n = 0usize;
    for t in 1..out.len() {
        for i in 0..out[t].pixels.len() {
            if i % 4 == 3 {
                continue;
            }
            let d = f64::from(out[t].pixels[i]) - f64::from(out[t - 1].pixels[i]);
            let r = f64::from(reference[t].pixels[i]) - f64::from(reference[t - 1].pixels[i]);
            sum += (d - r).abs();
            n += 1;
        }
    }
    sum / n.max(1) as f64
}

fn crop(image: &RgbaImage, (x0, y0, w, h): (u32, u32, u32, u32), zoom: u32) -> RgbaImage {
    let mut pixels = Vec::with_capacity((w * h * zoom * zoom * 4) as usize);
    for y in 0..h * zoom {
        for x in 0..w * zoom {
            let i = (((y0 + y / zoom) * image.width + x0 + x / zoom) * 4) as usize;
            pixels.extend_from_slice(&image.pixels[i..i + 4]);
        }
    }
    RgbaImage {
        width: w * zoom,
        height: h * zoom,
        pixels,
    }
}

#[test]
#[ignore = "evidence for docs/upscalers.md: run by hand on the GPU"]
fn compare() {
    let dir = std::env::var("OG_FSR_OUT").unwrap_or_else(|_| "target/fsr-compare".into());
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir).unwrap();
    let only = std::env::var("OG_FSR_CASE").ok();
    let mut rows = vec!["case\tscale\tupscaler\tmad\tpsnr\tcrop_mad\tflicker".to_string()];
    for case in cases() {
        if only.as_deref().is_some_and(|o| o != case.name) {
            continue;
        }
        // Ground truth: 4 samples a pixel (render scale 2, no antialiasing).
        let reference = run(&case, Antialias::Off, 2.0, Upscaler::Taau, 5);
        let last = reference.last().unwrap();
        og_tests::gfx::save_png(&dir.join(format!("{}-reference.png", case.name)), last).unwrap();
        og_tests::gfx::save_png(
            &dir.join(format!("{}-reference-crop.png", case.name)),
            &crop(last, case.crop, 4),
        )
        .unwrap();
        for scale in [0.5f32, 0.67, 0.77] {
            for (label, upscaler) in [("taau", Upscaler::Taau), ("fsr3", Upscaler::Fsr3)] {
                let out = run(&case, Antialias::Taa, scale, upscaler, 5);
                let frame = out.last().unwrap();
                let stem = format!("{}-{scale}-{label}", case.name);
                og_tests::gfx::save_png(&dir.join(format!("{stem}.png")), frame).unwrap();
                let (c, rc) = (crop(frame, case.crop, 4), crop(last, case.crop, 4));
                og_tests::gfx::save_png(&dir.join(format!("{stem}-crop.png")), &c).unwrap();
                let row = format!(
                    "{}\t{scale}\t{label}\t{:.3}\t{:.2}\t{:.3}\t{:.3}",
                    case.name,
                    diff(frame, last),
                    psnr(frame, last),
                    diff(&c, &rc),
                    flicker(&out, &reference),
                );
                println!("{row}");
                rows.push(row);
            }
        }
    }
    std::fs::write(dir.join("metrics.tsv"), rows.join("\n") + "\n").unwrap();
}

/// One output size and render scale per run (`OG_FSR_COST=1920x1080@0.5`),
/// each under `og_gpu_acquire_all`: the resolve's GPU time (the `taa` and
/// `taa-sharpen` passes) and the frame's, the median of 64 frames after
/// 16 to settle, for each upscaler.
#[test]
#[ignore = "evidence for docs/upscalers.md: run by hand on the GPU, alone"]
fn cost() {
    fn clock() -> f64 {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
            * 1e3
    }
    let spec = std::env::var("OG_FSR_COST").unwrap_or_else(|_| "1920x1080@0.5".into());
    let (size, scale) = spec.split_once('@').expect("WxH@scale");
    let (w, h) = size.split_once('x').expect("WxH");
    let (w, h, scale): (u32, u32, f32) = (
        w.parse().unwrap(),
        h.parse().unwrap(),
        scale.parse().unwrap(),
    );
    for (label, upscaler) in [("taau", Upscaler::Taau), ("fsr3", Upscaler::Fsr3)] {
        let mut d = desc(Antialias::Taa);
        (d.width, d.height) = (w, h);
        d.taa_sharpen = 0.5;
        let mut renderer = Renderer::new_blocking(&d).expect("renderer");
        if let Some(why) = renderer.gpu_timing_absent() {
            panic!("GPU lacks timing support: {why}");
        }
        renderer.set_clock(Some(clock));
        renderer.set_render_scale(scale);
        let mut scene = match std::env::var("OG_FSR_COST_SCENE").as_deref() {
            Ok("thin") => scenes::thin_geometry(),
            _ => og_tests::grass_scenes::grass_scene("near"),
        };
        scene.quality.upscaler = upscaler;
        let mut shown = false;
        let mut resolve = Vec::new();
        let mut total = Vec::new();
        for f in 0..80 {
            renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("frame");
            for frame in renderer.gpu_timings(true) {
                if f < 16 {
                    continue;
                }
                let pass = |n: &str| {
                    frame
                        .passes
                        .iter()
                        .filter(|p| p.name == n)
                        .map(|p| p.ms())
                        .sum::<f64>()
                };
                if !shown && f == 79 {
                    shown = true;
                    let all: Vec<String> = frame
                        .passes
                        .iter()
                        .map(|p| format!("{} {:.3}", p.name, p.ms()))
                        .collect();
                    println!("passes\t{label}\t{}", all.join(", "));
                }
                resolve.push(pass("taa") + pass("taa-sharpen"));
                total.push(frame.passes.iter().map(|p| p.ms()).sum::<f64>());
            }
        }
        let median = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        println!(
            "cost\t{w}x{h}\t{scale}\t{label}\tresolve {:.3} ms\tframe {:.3} ms\t({} frames)",
            median(&mut resolve),
            median(&mut total),
            resolve.len()
        );
    }
}
