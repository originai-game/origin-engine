//! gfx reference frames: each scene rendered headless must match its
//! committed reference for this backend tier (see og_tests::gfx), plus
//! behaviour checks read from rendered frames (shadows per cascade, edge
//! coverage, bloom, metering, sprite order) and from the frame stats.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Antialias, Background, Camera, DebugView, Exposure, FrameStats, Light, Material, RenderDesc,
    RenderError, RenderScene, RenderTarget, Renderer, RgbaImage, Sprite, Text, Tonemap, shapes,
};
use og_tests::gfx;
use og_tests::scenes::{self, GfxScene};
use std::path::PathBuf;

mod overlay_effects;
mod sprites_post;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn check(scene: &str) -> gfx::GfxReport {
    match gfx::check(scene, &out_dir()) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

/// Renders `frames` frames of `scene` with `desc` and returns the last one.
fn frame(desc: &RenderDesc, scene: &RenderScene, frames: u32) -> (RgbaImage, FrameStats) {
    let mut renderer = Renderer::new_blocking(desc).expect("renderer");
    let mut stats = FrameStats::default();
    for _ in 0..frames.max(1) {
        stats = renderer
            .render(scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    (renderer.read_offscreen().expect("capture"), stats)
}

fn built(name: &str) -> GfxScene {
    scenes::build(name).expect("scene")
}

fn luma(p: &[u8; 4]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn mean_luma(img: &RgbaImage) -> f32 {
    img.pixels.as_chunks::<4>().0.iter().map(luma).sum::<f32>() / (img.width * img.height) as f32
}

/// The spike's reference scene (M0 reproduced it byte for byte before the
/// sky pass; `tests/evidence/M0/spike-frame-reproduced.png`), now with CSM.
#[test]
fn gltf_reference_matches_its_reference() {
    check("gltf-reference");
}

/// 20,000 spheres submitted one instance at a time are drawn as two batches
/// (spheres, ground): one instanced draw per batch per pass that sees it.
#[test]
fn spheres_20k_are_batched_automatically() {
    let report = check("spheres-20k");
    let s = report.stats;
    assert_eq!(s.instances, 20_001, "20,000 spheres and the ground");
    assert_eq!(s.batches, 2, "one batch per (mesh, material)");
    assert_eq!(s.shadow_passes, 3, "three cascades");
    // At most one draw per batch per pass: 3 cascades + main, then sky and
    // post; GPU culling adds one occluder depth draw per opaque batch.
    assert!(
        s.occluder_draws <= 2,
        "{} occluder draws for 2 batches",
        s.occluder_draws
    );
    assert!(
        s.draw_calls - s.occluder_draws <= 2 * 4 + 2,
        "{} draw calls ({} occluder) for 2 batches",
        s.draw_calls,
        s.occluder_draws
    );
    assert!(s.triangles > 0 && s.triangles <= 20_000 * 384 + 2);
}

#[test]
fn cascades_200m_matches_its_reference() {
    check("cascades-200m");
}

/// The cascade debug view (red, green, blue for cascades 0, 1, 2) for
/// pixel classification, and the scene with and without shadows.
fn cascade_frames(tall: bool) -> (RgbaImage, RgbaImage, RgbaImage) {
    let g = built("cascades-200m");
    let mut scene = scenes::cascades_200m(tall);
    let (lit, _) = frame(&g.desc, &scene, 1);
    scene.light.cast_shadows = false;
    let (unshadowed, _) = frame(&g.desc, &scene, 1);
    scene.light.cast_shadows = true;
    scene.debug = DebugView::Cascades;
    let (debug, _) = frame(&g.desc, &scene, 1);
    (debug, lit, unshadowed)
}

fn cascade_of(p: &[u8]) -> Option<usize> {
    let max = *p[..3].iter().max().expect("rgb");
    if max < 100 {
        return None;
    }
    let c = p[..3].iter().position(|&v| v == max).expect("max");
    p[..3]
        .iter()
        .enumerate()
        .all(|(i, &v)| i == c || v < 60)
        .then_some(c)
}

/// All three cascades are in use, nearer pixels in nearer cascades: the
/// cascade covering the bottom row (the ground at the camera's feet) is 0,
/// and the far ground at the horizon is covered by cascade 2.
#[test]
fn cascades_cover_the_view_near_to_far() {
    let (debug, _, _) = cascade_frames(true);
    let w = debug.width as usize;
    let mut counts = [0usize; 3];
    for p in debug.pixels.as_chunks::<4>().0.iter() {
        if let Some(c) = cascade_of(p) {
            counts[c] += 1;
        }
    }
    println!("pixels per cascade: {counts:?}");
    for (c, n) in counts.iter().enumerate() {
        assert!(*n > 5_000, "cascade {c} covers only {n} pixels");
    }
    let row_cascade = |y: usize| {
        // The cascade most of row `y` is in.
        let mut row = [0usize; 3];
        for x in 0..w {
            if let Some(c) = cascade_of(&debug.pixels[(y * w + x) * 4..]) {
                row[c] += 1;
            }
        }
        (0..3).max_by_key(|&c| row[c]).expect("three")
    };
    assert_eq!(row_cascade(debug.height as usize - 1), 0, "bottom row");
    // Down the middle column, the first pixel below the sky is the farthest.
    let x = w / 2;
    let horizon = (0..debug.height as usize)
        .find(|&y| cascade_of(&debug.pixels[(y * w + x) * 4..]).is_some())
        .expect("ground in view");
    assert_eq!(
        cascade_of(&debug.pixels[(horizon * w + x) * 4..]),
        Some(2),
        "the far ground at the horizon"
    );
}

/// Shadows of short casters (0.5 m blocks, 0.6 m kerbs) are visible in every
/// cascade of a 400 m scene: in each cascade's pixels, a substantial number
/// are clearly darker with shadows than without. The single bias in
/// normalized depth this replaced amounted to about a metre here and lost
/// them.
#[test]
fn shadows_fall_in_every_cascade_at_200m_scale() {
    let (debug, lit, unshadowed) = cascade_frames(false);
    let mut shadowed = [0usize; 3];
    for ((d, a), b) in debug
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(lit.pixels.as_chunks::<4>().0.iter())
        .zip(unshadowed.pixels.as_chunks::<4>().0.iter())
    {
        if let Some(c) = cascade_of(d)
            && luma(a) < 0.8 * luma(b)
        {
            shadowed[c] += 1;
        }
    }
    println!("shadowed pixels per cascade: {shadowed:?}");
    for (c, n) in shadowed.iter().enumerate() {
        assert!(*n > 150, "cascade {c}: only {n} shadowed pixels");
    }
}

#[test]
fn point_spot_lights_match_their_reference() {
    let report = check("point-spot-lights");
    assert_eq!(report.stats.lights, 7);
    assert_eq!(
        report.stats.shadow_passes, 7,
        "a spot (1 face) and a point light (6 faces); the sun casts none"
    );
}

/// The two shadowed local lights darken the ground behind the pillars.
#[test]
fn local_lights_cast_shadows() {
    let g = built("point-spot-lights");
    let mut scene = g.scene;
    let (lit, _) = frame(&g.desc, &scene, 1);
    for l in &mut scene.lights {
        l.cast_shadows = false;
    }
    let (unshadowed, stats) = frame(&g.desc, &scene, 1);
    assert_eq!(stats.shadow_passes, 0);
    let darker = lit
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(unshadowed.pixels.as_chunks::<4>().0.iter())
        .filter(|(a, b)| luma(a) + 8.0 < 0.75 * luma(b))
        .count();
    println!("{darker} pixels darker with local shadows");
    assert!(darker > 10_000, "only {darker} shadowed pixels");
}

/// Visible lights beyond MAX_LIGHTS (256) are not drawn and are reported.
#[test]
fn lights_beyond_the_limit_are_counted() {
    let mut scene = scenes::dim_scene(1.0);
    scene.lights = vec![Light::default(); og_render::MAX_LIGHTS + 4];
    let (_, stats) = frame(&scenes::base_desc(), &scene, 1);
    assert_eq!(
        (stats.lights, stats.lights_dropped, stats.lights_culled),
        (og_render::MAX_LIGHTS as u32, 4, 0)
    );
}

#[test]
fn taa_matches_its_reference() {
    let report = check("taa");
    assert_eq!(report.info.antialias, Antialias::Taa);
}

/// TAA's frames `at` (each 1-based) of the thin geometry with the camera
/// at `pan(frame)` 2 cm steps to the side (`pan_thin_geometry`), and the
/// last camera's frame without antialiasing.
fn taa_frames(pan: impl Fn(u32) -> u32, at: &[u32]) -> (Vec<RgbaImage>, RgbaImage) {
    let mut built = built("taa-motion");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    assert_eq!(renderer.info().antialias, Antialias::Taa);
    let mut out = Vec::new();
    for f in 0..*at.iter().max().expect("a frame") {
        og_tests::shadow_scenes::pan_thin_geometry(&mut built.scene, pan(f));
        let stats = renderer
            .render(&built.scene, &RenderTarget::Offscreen)
            .expect("render");
        assert_eq!(stats.taa_history, f > 0);
        if at.contains(&(f + 1)) {
            out.push(renderer.read_offscreen().expect("capture"));
        }
    }
    let off = RenderDesc {
        antialias: Antialias::Off,
        ..built.desc
    };
    (out, frame(&off, &built.scene, 1).0)
}

/// `a` against `b`: (RMSE, the largest channel difference).
fn difference(a: &RgbaImage, b: &RgbaImage) -> (f64, u8) {
    let d = og_tests::compare::compare(
        &a.pixels,
        &b.pixels,
        b.width,
        b.height,
        &og_tests::compare::Tolerance::default(),
    )
    .expect("compare");
    (d.rmse, d.max_channel_diff)
}

/// The horizontal shift of `a` against `b` in each eighth of the frame's
/// width, in pixels: the sub-pixel `s` (to 0.05) that best matches `a(x)`
/// with `b(x - s)` linearly interpolated, over the luma of every other row
/// of the frame's lower two fifths (the thin geometry's checkered ground,
/// detail across the whole width).
fn shifts_by_eighth(a: &RgbaImage, b: &RgbaImage) -> [f32; 8] {
    let (w, h) = (a.width as usize, a.height as usize);
    let l = |img: &RgbaImage, x: usize, y: usize| {
        let i = (y * w + x) * 4;
        luma(&[
            img.pixels[i],
            img.pixels[i + 1],
            img.pixels[i + 2],
            img.pixels[i + 3],
        ])
    };
    let mut out = [0.0; 8];
    for (k, shift) in out.iter_mut().enumerate() {
        let (x0, x1) = (k * w / 8 + 4, (k + 1) * w / 8 - 4);
        let mut best = (f32::MAX, 0.0);
        for step in -30..=30 {
            let s = step as f32 * 0.05;
            let mut e = 0.0;
            for y in (h * 3 / 5..h).step_by(2) {
                for x in x0..x1 {
                    let xs = x as f32 - s;
                    let i = xs.floor() as usize;
                    let f = xs - i as f32;
                    let bb = l(b, i, y) * (1.0 - f) + l(b, i + 1, y) * f;
                    e += (l(a, x, y) - bb).powi(2);
                }
            }
            if e < best.0 {
                best = (e, s);
            }
        }
        *shift = best.1;
    }
    out
}

/// TAA's resolve with a still camera holds its history where it is. Once
/// converged it is a fixed point of the jitter's cycle: frames 8 apart (the
/// same Halton phase) agree within one level, the last bits of the 16-bit
/// history. And it sits where the frame without antialiasing does: the
/// jitter's weighted mean moves the whole frame a fraction of a pixel to
/// one side, the same in every eighth of the width to 0.15 px. The retired
/// WebGL2 tier before row 68's fix (TAA's depth read as 0 there) settled 0.5 to 0.75
/// px to the side, more at the right of the frame than at the left: a
/// steady state too, which the shift catches and the fixed point alone
/// does not (a history read 0.05 px to the left per frame, times x over
/// the width, fails here with shifts from -0.15 to 0.3 px).
#[test]
fn taa_with_a_still_camera_reaches_a_fixed_point_in_place() {
    let (frames, off) = taa_frames(|_| 0, &[128, 136, 144]);
    for pair in frames.windows(2) {
        let (rmse, max) = difference(&pair[1], &pair[0]);
        println!("still: frames 8 apart: rmse {rmse:.6}, max {max}");
        assert!(
            max <= 1 && rmse <= 1e-4,
            "TAA's still frames 8 apart differ (rmse {rmse:.6}, max {max})"
        );
    }
    let shifts = shifts_by_eighth(&frames[2], &off);
    println!("still: shift against the frame without antialiasing by eighth: {shifts:?}");
    let (lo, hi) = shifts
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &s| (lo.min(s), hi.max(s)));
    assert!(
        hi - lo <= 0.15 && lo >= -0.35 && hi <= 0.35,
        "TAA's converged frame moved against the unantialiased one: {shifts:?} px"
    );
}

/// A camera that slides for 12 frames and stops: the history the motion
/// left (reprojected, bilinear) is then held still, and the resolve
/// reaches the fixed point a camera still there from the first frame
/// reaches, within one level.
#[test]
fn taa_after_the_camera_stops_reaches_the_still_cameras_fixed_point() {
    let at = [12 + 128, 12 + 136];
    let (moved, _) = taa_frames(|f| f.min(12), &at);
    let (still, _) = taa_frames(|_| 12, &at);
    let (rmse, max) = difference(&moved[0], &moved[1]);
    println!("moved then stopped: frames 8 apart: rmse {rmse:.6}, max {max}");
    assert!(
        max <= 1 && rmse <= 1e-4,
        "frames 8 apart differ (rmse {rmse:.6}, max {max})"
    );
    let (rmse, max) = difference(&moved[1], &still[1]);
    println!("moved then stopped against still there throughout: rmse {rmse:.6}, max {max}");
    // One level apart in a few hundred more pixels than the cycle's own
    // (rmse 0.0003): the clamp's box admits more than one fixed point of
    // the 16-bit history's rounding.
    assert!(
        max <= 1 && rmse <= 1e-3,
        "TAA after the camera stopped differs from a camera still there (rmse {rmse:.6}, max {max})"
    );
}

/// Thin white bars on black, unlit (emissive only, no tonemap): without
/// antialiasing every pixel is black or white; MSAA, TAA and FXAA produce
/// partial coverage along the edges. Returns the partially covered pixels and the
/// antialiasing the renderer resolved.
fn edge_pixels(antialias: Antialias, frames: u32) -> (usize, Antialias) {
    let mut scene = RenderScene::new();
    let bar = scene.add_mesh(shapes::cuboid(Vec3::new(8.0, 0.06, 0.06)));
    let white = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [1.0, 1.0, 1.0],
        ..Default::default()
    });
    for i in 0..8 {
        scene.add_instance(
            bar,
            white,
            Mat4::from_translation(Vec3::new(0.0, -2.0 + i as f32 * 0.5, 0.0))
                * Mat4::from_rotation_z(0.05 + i as f32 * 0.11),
        );
    }
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment.intensity = 0.0;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene.camera.position = Vec3::new(0.0, 0.0, 6.0);
    scene.camera.target = Vec3::ZERO;
    let desc = RenderDesc {
        antialias,
        ..scenes::base_desc()
    };
    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    for _ in 0..frames {
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
    }
    let img = renderer.read_offscreen().expect("capture");
    let partial = img
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| (12..=243).contains(&p[0]))
        .count();
    (partial, renderer.info().antialias)
}

#[test]
fn antialiasing_smooths_edges() {
    let (off, _) = edge_pixels(Antialias::Off, 1);
    let (msaa, msaa_mode) = edge_pixels(Antialias::Msaa4, 1);
    let (taa, taa_mode) = edge_pixels(Antialias::Taa, 16);
    let (fxaa, fxaa_mode) = edge_pixels(Antialias::Fxaa, 1);
    println!(
        "partially covered pixels: off {off}, msaa {msaa} ({msaa_mode:?}), taa {taa}, fxaa {fxaa}"
    );
    assert_eq!(taa_mode, Antialias::Taa);
    assert_eq!(fxaa_mode, Antialias::Fxaa);
    assert!(fxaa > 2_000, "FXAA left {fxaa} partial pixels");
    assert!(off < 50, "{off} partial pixels without antialiasing");
    assert!(taa > 3_000, "TAA left {taa} partial pixels");
    if msaa_mode == Antialias::Msaa4 {
        assert!(msaa > 3_000, "MSAA left {msaa} partial pixels");
    }
}

/// `RenderInfo::msaa_samples` is the MSAA the device gives, the cap of
/// `RenderQuality::msaa_samples` (`RenderCapabilities.maxMsaaSamples`):
/// 4 where `Msaa4` resolved to MSAA (it stays 4 while a quality of 1 draws
/// FXAA, which `antialias` says), 1 where the renderer resolved TAA, FXAA
/// or none, which a quality asking 4 does not change. Before the debt
/// round nothing said so: a preset's 4 was drawn at 1 unseen.
#[test]
fn the_device_msaa_caps_the_qualitys() {
    use og_render::RenderQuality;
    let scene = RenderScene::new();
    let mut msaa = Renderer::new_blocking(&RenderDesc {
        antialias: Antialias::Msaa4,
        ..scenes::base_desc()
    })
    .expect("renderer");
    let resolved = msaa.info().antialias;
    println!(
        "{}: Msaa4 resolved to {resolved:?}, msaa_samples {}",
        msaa.info().backend_tier(),
        msaa.info().msaa_samples
    );
    if resolved == Antialias::Msaa4 {
        assert_eq!(msaa.info().msaa_samples, og_render::MSAA_SAMPLES);
        let mut one = RenderScene::new();
        one.quality = RenderQuality {
            msaa_samples: 1,
            ..RenderQuality::default()
        };
        msaa.render(&one, &RenderTarget::Offscreen).expect("render");
        assert_eq!(msaa.info().antialias, Antialias::Fxaa);
        assert_eq!(msaa.info().msaa_samples, og_render::MSAA_SAMPLES);
    } else {
        assert_eq!(resolved, Antialias::Fxaa);
        assert_eq!(msaa.info().msaa_samples, 1);
    }
    for antialias in [Antialias::Taa, Antialias::Fxaa, Antialias::Off] {
        let mut r = Renderer::new_blocking(&RenderDesc {
            antialias,
            ..scenes::base_desc()
        })
        .expect("renderer");
        assert_eq!(r.info().msaa_samples, 1, "{antialias:?}");
        r.render(&scene, &RenderTarget::Offscreen).expect("render");
        assert_eq!(
            r.info().antialias,
            antialias,
            "a quality of 4 draws one sample"
        );
    }
}
