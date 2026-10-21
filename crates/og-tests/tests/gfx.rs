//! gfx reference frames: each scene rendered headless must match its
//! committed reference for this backend tier (see og_tests::gfx), plus
//! behaviour checks read from rendered frames (shadows per cascade, edge
//! coverage, bloom, metering, sprite order) and from the frame stats.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Antialias, Background, Camera, DebugView, Exposure, FrameStats, Light, Material, RenderDesc,
    RenderError, RenderInfo, RenderScene, RenderTarget, Renderer, RgbaImage, Sprite, Text, Tier,
    Tonemap, shapes,
};
use og_tests::gfx;
use og_tests::scenes::{self, GfxScene};
use std::path::PathBuf;

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

/// The adapter the harness renders on (`OG_GFX_BACKEND` picks one).
fn adapter() -> RenderInfo {
    Renderer::new_blocking(&scenes::base_desc())
        .expect("a GPU adapter")
        .info()
        .clone()
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
fn taa_matches_its_reference_on_the_full_tier() {
    if adapter().tier == Tier::Full {
        let report = check("taa");
        assert_eq!(report.info.antialias, Antialias::Taa);
    } else {
        // `Antialias::Tier` is MSAA here; the msaa scene covers this tier.
        let r = Renderer::new_blocking(&built("taa").desc).expect("renderer");
        assert_eq!(r.info().antialias, Antialias::Msaa4);
    }
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
/// one side, the same in every eighth of the width to 0.15 px. WebGL2
/// before row 68's fix (TAA's depth read as 0 there) settled 0.5 to 0.75
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

#[test]
fn msaa_matches_its_reference_on_the_no_compute_tier() {
    if adapter().tier == Tier::NoCompute {
        let report = check("msaa");
        assert_eq!(report.info.antialias, Antialias::Msaa4);
    } else {
        // `Antialias::Tier` is TAA here; MSAA itself is exercised by
        // `antialiasing_smooths_edges` with `Antialias::Msaa4`.
        let r = Renderer::new_blocking(&built("msaa").desc).expect("renderer");
        assert_eq!(r.info().antialias, Antialias::Taa);
    }
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

#[test]
fn bloom_matches_its_reference() {
    check("bloom");
}

/// Bloom spreads light from the emissive spheres into the dark around them.
#[test]
fn bloom_spreads_light_beyond_the_emitters() {
    let g = built("bloom");
    let (bloomed, with) = frame(&g.desc, &g.scene, 1);
    let mut scene = g.scene;
    scene.post.bloom = None;
    let (plain, without) = frame(&g.desc, &scene, 1);
    assert!(
        with.draw_calls > without.draw_calls + 4,
        "bloom passes: {} vs {} draws",
        with.draw_calls,
        without.draw_calls
    );
    let brighter = bloomed
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(plain.pixels.as_chunks::<4>().0.iter())
        .filter(|(a, b)| luma(a) > luma(b) + 10.0)
        .count();
    let frac = brighter as f64 / (bloomed.width * bloomed.height) as f64;
    println!("{:.1}% of pixels brighter with bloom", frac * 100.0);
    assert!(frac > 0.05, "bloom brightened only {:.2}%", frac * 100.0);
}

#[test]
fn exposure_matches_its_reference() {
    check("exposure");
}

/// Metered exposure depends on the frame only: the same scene lit 16x
/// brighter comes out at the same brightness, where a manual exposure does not.
#[test]
fn auto_exposure_meters_the_frame() {
    let desc = scenes::base_desc();
    let (dim, _) = frame(&desc, &scenes::dim_scene(1.0), 1);
    let (bright, _) = frame(&desc, &scenes::dim_scene(16.0), 1);
    let (dim2, _) = frame(&desc, &scenes::dim_scene(1.0), 1);
    let (a, b) = (mean_luma(&dim), mean_luma(&bright));
    println!("auto: mean luma {a:.1} (x1) vs {b:.1} (x16)");
    assert!(
        (a / b - 1.0).abs() < 0.08,
        "auto exposure: {a:.1} vs {b:.1}"
    );
    assert_eq!(dim.pixels, dim2.pixels, "the same frame meters the same");
    let manual = |k: f32| {
        let mut s = scenes::dim_scene(k);
        s.camera.exposure = Exposure::Manual { ev: 2.0 };
        mean_luma(&frame(&desc, &s, 1).0)
    };
    let (ma, mb) = (manual(1.0), manual(16.0));
    println!("manual: mean luma {ma:.1} (x1) vs {mb:.1} (x16)");
    assert!(mb > 2.0 * ma, "manual exposure: {ma:.1} vs {mb:.1}");
}

/// 20,000 sprites submitted one by one, drawn in a handful of draws.
#[test]
fn sprites_20k_match_their_reference_in_a_handful_of_draws() {
    let report = check("sprites-20k");
    let s = report.stats;
    assert_eq!(s.sprites, 20_000);
    assert!(s.sprite_draws <= 6, "{} sprite draws", s.sprite_draws);
    assert_eq!(s.draw_calls, s.sprite_draws + 1, "sprites, then post");
}

/// Sprites drawn from texture arrays (`RenderScene::sprite_arrays`): the
/// sixteen flipbook sprites (eight layers, two rows) are one draw, the six
/// added ones from a second array another, the plain textured strip a
/// third. Broken once by giving each layer its own run: 8 + 3 + 1 draws.
#[test]
fn sprites_array_match_their_reference_in_one_draw_per_array() {
    let report = check("sprites-array");
    let s = report.stats;
    assert_eq!(s.sprites, 23);
    assert_eq!(
        s.sprite_draws, 3,
        "one run per array and blend, and the strip"
    );
}

/// Materials sampling texture arrays: eight cubes of one tile material on
/// four layers (and the material's own) are one batch; the blended panes
/// of another array material one more; with the ground, the plain cube and
/// the normal-array sphere, five (mesh, material) pairs for 13 instances.
#[test]
fn materials_array_match_their_reference_in_one_batch_per_material() {
    let report = check("materials-array");
    let s = report.stats;
    assert_eq!((s.batches, s.instances), (5, 13));
}

/// A sprite drawn from an array shows the layer it names, sampled with the
/// array's filter; a layer past the array's count, or an array that was
/// removed, is refused.
#[test]
fn a_sprite_draws_the_layer_of_its_texture_array() {
    use og_render::{
        ColorSpace, SpriteArray, TextureArrayData, TextureData, TextureFilter, TextureSource,
    };
    let solid = |rgb: [u8; 3]| {
        TextureSource::Rgba(TextureData {
            width: 4,
            height: 4,
            rgba: [rgb[0], rgb[1], rgb[2], 255].repeat(16),
            color_space: ColorSpace::Srgb,
            mips: og_render::Mips::None,
        })
    };
    let mut scene = RenderScene::new();
    let array = scene
        .add_texture_array(TextureArrayData {
            layers: vec![solid([255, 0, 0]), solid([0, 255, 0]), solid([0, 0, 255])],
        })
        .expect("three layers");
    assert!(scene.set_texture_array_filter(array, TextureFilter::Nearest));
    for (i, (x, layer)) in [(-3.0, 2u32), (3.0, 1)].into_iter().enumerate() {
        scene.add_sprite(Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            size: Vec2::splat(4.0),
            ..Default::default()
        });
        scene.sprite_arrays.push(SpriteArray {
            sprite: i as u32,
            array,
            layer,
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let desc = scenes::base_desc();
    let (img, stats) = frame(&desc, &scene, 1);
    let at = |x: u32| {
        let i = ((img.height / 2 * img.width + x) * 4) as usize;
        [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
    };
    assert_eq!(
        at(img.width * 3 / 10),
        [0, 0, 255],
        "the left sprite: layer 2"
    );
    assert_eq!(
        at(img.width * 7 / 10),
        [0, 255, 0],
        "the right sprite: layer 1"
    );
    assert_eq!(stats.sprite_draws, 1, "both layers in one draw");

    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    scene.sprite_arrays[0].layer = 3;
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("layer 3"), "{m}"),
        other => panic!("a layer past the array: {:?}", other.map(|_| ())),
    }
    scene.sprite_arrays[0].layer = 0;
    renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("frame");
    assert!(scene.remove_texture_array(array));
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidHandle(m)) => {
            assert!(m.contains("texture array 0 was removed"), "{m}")
        }
        other => panic!("a sprite of a removed array: {:?}", other.map(|_| ())),
    }
}

/// Texture arrays through every sprite path (`sprites-array-fx`): the
/// glowing flipbook sprites and the flipbook particles are one run, the
/// smooth array's particles another, the strip a third, and the overlay's
/// four array sprites one draw beside its plain one.
#[test]
fn sprites_array_fx_match_their_reference_in_one_draw_per_array() {
    let report = check("sprites-array-fx");
    let s = report.stats;
    assert_eq!(s.overlay_sprites, 5);
    assert_eq!(
        s.overlay_draws, 2,
        "the overlay's array run and its plain sprite"
    );
    assert_eq!(
        s.sprite_draws, 5,
        "the strip, the flipbook (sprites and particles), the motes, and the overlay's two"
    );
}

/// Solid layers for array tests: red, green, blue.
fn rgb_array(scene: &mut RenderScene) -> og_render::TextureArrayHandle {
    use og_render::{ColorSpace, TextureArrayData, TextureData, TextureSource};
    let solid = |rgb: [u8; 3]| {
        TextureSource::Rgba(TextureData {
            width: 4,
            height: 4,
            rgba: [rgb[0], rgb[1], rgb[2], 255].repeat(16),
            color_space: ColorSpace::Srgb,
            mips: og_render::Mips::None,
        })
    };
    let array = scene
        .add_texture_array(TextureArrayData {
            layers: vec![solid([255, 0, 0]), solid([0, 255, 0]), solid([0, 0, 255])],
        })
        .expect("three layers");
    scene.set_texture_array_filter(array, og_render::TextureFilter::Nearest);
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    array
}

fn pixel(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

/// An array sprite with a glow lights the glow buffer around it (the
/// array's glow pipeline), and an overlay sprite draws the layer it names
/// in screen space; a layer past the array is refused there too. Broken
/// once by leaving array sprites out of the glow pass: no halo.
#[test]
fn array_sprites_glow_and_draw_in_the_overlay() {
    use og_render::{Glow, Overlay, SpriteArray, SpriteFx};
    let mut scene = RenderScene::new();
    let array = rgb_array(&mut scene);
    scene.add_sprite(Sprite {
        size: Vec2::splat(2.0),
        ..Default::default()
    });
    scene.sprite_arrays.push(SpriteArray {
        sprite: 0,
        array,
        layer: 1,
    });
    scene.post.glow = Some(Glow { intensity: 1.0 });
    let desc = scenes::base_desc();
    // Just outside the sprite's right edge (2 of 10 units tall: 72 px each side).
    let (cx, cy) = (640, 360);
    let edge = cx + 72 + 12;
    let (plain, _) = frame(&desc, &scene, 1);
    assert_eq!(pixel(&plain, cx, cy), [0, 255, 0], "layer 1");
    assert_eq!(pixel(&plain, edge, cy), [0, 0, 0], "no glow");
    scene.sprite_fx.push(SpriteFx {
        sprite: 0,
        glow: 2.0,
        ..Default::default()
    });
    let (glowing, stats) = frame(&desc, &scene, 1);
    let halo = pixel(&glowing, edge, cy);
    assert!(
        halo[1] > 20 && halo[0] < halo[1] / 4,
        "the layer's green glows past the edge: {halo:?}"
    );
    assert_eq!(stats.sprite_draws, 1);

    let mut overlay = Overlay::new(1280.0, 720.0, 1.0);
    for (x, layer) in [(100.0, 2u32), (300.0, 0)] {
        let i = overlay.sprites.len() as u32;
        overlay.sprites.push(Sprite {
            position: Vec3::new(x, 100.0, 0.0),
            size: Vec2::splat(80.0),
            ..Default::default()
        });
        overlay.sprite_arrays.push(SpriteArray {
            sprite: i,
            array,
            layer,
        });
    }
    scene.overlay = Some(overlay);
    let (img, stats) = frame(&desc, &scene, 1);
    assert_eq!(pixel(&img, 100, 100), [0, 0, 255], "overlay layer 2");
    assert_eq!(pixel(&img, 300, 100), [255, 0, 0], "overlay layer 0");
    assert_eq!(
        stats.overlay_draws, 1,
        "one draw for the array's overlay sprites"
    );
    scene.overlay.as_mut().unwrap().sprite_arrays[1].layer = 5;
    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    match renderer.render(&scene, &RenderTarget::Offscreen) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("layer 5"), "{m}"),
        other => panic!("an overlay layer past the array: {:?}", other.map(|_| ())),
    }
}

/// A sprite-mode emitter with a texture array draws each particle from a
/// layer: by age (every particle of one burst is the same age, so one
/// layer shows) or at random (several show), in one draw; `round` gives
/// them the soft dot's edge (a big particle's corner stays dark); a
/// billboard emitter with an array is refused. Broken once by drawing the
/// particles without their array (as untextured soft dots): no layer's
/// colour shows.
#[test]
fn array_particles_take_their_layers_by_age_or_at_random() {
    use og_render::{
        Burst, EmitterShape, ParticleEmitter, ParticleRender, ParticleSystem, SheetMode,
        SpriteSheet,
    };
    let run = |mode: SheetMode, round: bool, size: f32, billboard: bool| {
        let mut scene = RenderScene::new();
        let array = rgb_array(&mut scene);
        // Forty small particles over the view, or one big one at the centre.
        let big = size > 1.0;
        let e = scene.add_emitter(ParticleEmitter {
            max_particles: 64,
            shape: match big {
                true => EmitterShape::Point,
                false => EmitterShape::Box {
                    half_extents: Vec3::new(3.0, 3.0, 0.0),
                },
            },
            rate: 0.0,
            bursts: vec![Burst::once(0.0, if big { 1 } else { 40 })],
            looping: false,
            lifetime: [3.0, 3.0],
            speed: [0.0, 0.0],
            size: [size, size],
            gravity: Vec3::ZERO,
            array: Some(array),
            round,
            sheet: SpriteSheet {
                mode,
                ..Default::default()
            },
            blend: og_render::ParticleBlend::Alpha,
            render: if billboard {
                ParticleRender::Billboard
            } else {
                ParticleRender::Sprite { layer: 0 }
            },
            ..Default::default()
        });
        scene
            .particles
            .push(ParticleSystem::new(1, e, Mat4::IDENTITY, 9, 0));
        let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
        // 1.5 s of a 3 s life at 60 ticks a second: the middle layer by age.
        let mut stats = FrameStats::default();
        for t in 0..=90 {
            scene.tick = t;
            scene.particles[0].tick = t;
            stats = renderer.render(&scene, &RenderTarget::Offscreen)?;
        }
        Ok::<_, RenderError>((renderer.read_offscreen().expect("capture"), stats))
    };
    let colours = |img: &RgbaImage| {
        let mut seen = [0usize; 3];
        for p in img.pixels.as_chunks::<4>().0 {
            match [p[0], p[1], p[2]] {
                [255, 0, 0] => seen[0] += 1,
                [0, 255, 0] => seen[1] += 1,
                [0, 0, 255] => seen[2] += 1,
                _ => {}
            }
        }
        seen
    };
    let (img, stats) = run(SheetMode::OverLife { cycles: 1.0 }, false, 0.4, false).expect("frame");
    let seen = colours(&img);
    assert!(
        seen[1] > 1000 && seen[0] == 0 && seen[2] == 0,
        "by age: every particle on layer 1: {seen:?}"
    );
    assert_eq!(stats.sprites, 40);
    assert_eq!(stats.sprite_draws, 1, "every layer in one draw");
    let (img, _) = run(SheetMode::Random, false, 0.4, false).expect("frame");
    let seen = colours(&img);
    assert!(
        seen.iter().all(|n| *n > 100),
        "at random: every layer shows: {seen:?}"
    );
    // One big particle at the centre: a round one's corner is dark.
    let (square, _) = run(SheetMode::OverLife { cycles: 1.0 }, false, 9.0, false).expect("frame");
    let (round, _) = run(SheetMode::OverLife { cycles: 1.0 }, true, 9.0, false).expect("frame");
    let corner = |img: &RgbaImage| pixel(img, 640 - 300, 360 - 300);
    assert_eq!(corner(&square), [0, 255, 0], "a square particle's corner");
    assert!(
        corner(&round)[1] < 40,
        "a round one's: {:?}",
        corner(&round)
    );
    match run(SheetMode::Off, false, 0.4, true) {
        Err(RenderError::InvalidData(m)) => assert!(m.contains("sprite-mode"), "{m}"),
        other => panic!("a billboard emitter with an array: {:?}", other.map(|_| ())),
    }
}

/// Higher layers draw over lower ones whatever the submission order.
#[test]
fn sprite_layers_order_the_draw() {
    let mut scene = RenderScene::new();
    let quad = |color: [f32; 4], layer: i32| Sprite {
        size: Vec2::splat(4.0),
        color,
        layer,
        ..Default::default()
    };
    scene.add_sprite(quad([1.0, 0.0, 0.0, 1.0], 1));
    scene.add_sprite(quad([0.0, 0.0, 1.0, 1.0], 0));
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let (img, stats) = frame(&scenes::base_desc(), &scene, 1);
    let center = ((img.height / 2 * img.width + img.width / 2) * 4) as usize;
    assert_eq!(
        &img.pixels[center..center + 3],
        &[255, 0, 0],
        "layer 1 on top"
    );
    assert_eq!(
        stats.sprite_draws, 1,
        "one texture: one draw for both layers"
    );
}

#[test]
fn text_msdf_matches_its_reference() {
    let report = check("text-msdf");
    let expected: usize = scenes::TEXT_LINES
        .iter()
        .map(|(t, ..)| t.chars().filter(|c| !c.is_whitespace()).count())
        .sum::<usize>()
        + "rotated".len()
        + "&".len();
    assert_eq!(report.stats.glyphs as usize, expected);
    assert_eq!(report.stats.missing_glyphs, 0);
}

/// Characters the atlas lacks are skipped and counted; a font handle of
/// another scene is an error, not a panic.
#[test]
fn text_reports_missing_glyphs_and_bad_fonts() {
    let mut scene = RenderScene::new();
    let font = scenes::load_font(&mut scene).expect("font");
    scene.texts.push(Text::new(font, "a☃b"));
    let (_, stats) = frame(&scenes::base_desc(), &scene, 1);
    assert_eq!((stats.glyphs, stats.missing_glyphs), (2, 1));
    let mut other = RenderScene::new();
    other.texts.push(Text::new(font, "x"));
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    assert!(matches!(
        renderer.render(&other, &RenderTarget::Offscreen),
        Err(RenderError::InvalidHandle(_))
    ));
}

#[test]
fn sky_only_matches_its_reference() {
    let report = check("sky-only");
    assert_eq!(report.stats.draw_calls, 2, "sky and post");
}

#[test]
fn showcase_matches_its_reference() {
    let report = check("showcase");
    let s = report.stats;
    // The campfire adds a fourth light; its three systems, two trails, a
    // beam and four decals join the M3 showcase.
    assert_eq!((s.lights, s.sprites), (4, 40));
    assert!(s.glyphs > 0 && s.shadow_passes == 3 + 1 + 6);
    assert_eq!((s.particle_systems, s.strips, s.decals), (3, 3, 4));
    assert!(s.prepass_draws > 0, "ambient occlusion runs the prepass");
}

#[test]
fn overlay_matches_its_reference() {
    let report = check("overlay");
    let s = report.stats;
    // 1 panel, 2 bar, 1 marker, 1 list back, 5 rows, 2 crosshair sprites.
    assert_eq!(s.overlay_sprites, 12);
    assert!(s.overlay_glyphs > 20, "{} overlay glyphs", s.overlay_glyphs);
    assert!(s.overlay_draws <= 6, "{} overlay draws", s.overlay_draws);
    assert_eq!(s.sprites, 40 + 12, "the totals count the overlay too");
}

fn overlay_px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
}

fn srgb8(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92 * 255.0
    } else {
        (1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0
    }
}

fn near(p: [u8; 3], linear: [f32; 3], within: f32) -> bool {
    p.iter()
        .zip(linear)
        .all(|(&a, e)| (a as f32 - srgb8(e)).abs() <= within)
}

/// The overlay is drawn over the finished frame: after TAA, bloom, metered
/// exposure and ACES, an opaque overlay colour comes out exactly as authored
/// (sRGB-encoded), its edges are sharp on every TAA frame, it covers the
/// scene without a depth test, and its clip cuts rows at the clip's edge.
#[test]
fn the_overlay_is_drawn_after_post_untonemapped_and_clipped() {
    let g = built("overlay");
    let mut renderer = Renderer::new_blocking(&g.desc).expect("renderer");
    let mut scene = g.scene;
    let mut frames = Vec::new();
    for f in 0..g.frames {
        if let Some(update) = g.update {
            update(&mut scene, f);
        }
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        if f + 2 >= g.frames {
            frames.push(renderer.read_offscreen().expect("capture"));
        }
    }
    let img = frames.last().expect("a frame");
    assert_eq!((img.width, img.height), (1280, 720));
    // The health bar's fill and its track, one pixel either side of the
    // fill's end: exact colours, no TAA smear, no tone mapping.
    let fill = [0.9, 0.18, 0.05];
    let track = [0.1, 0.1, 0.1];
    for img in &frames {
        assert!(
            near(overlay_px(img, 100, 102), fill, 1.5),
            "fill {:?}",
            overlay_px(img, 100, 102)
        );
        assert!(
            near(overlay_px(img, 239, 102), fill, 1.5),
            "fill edge {:?}",
            overlay_px(img, 239, 102)
        );
        assert!(
            near(overlay_px(img, 240, 102), track, 1.5),
            "track edge {:?}",
            overlay_px(img, 240, 102)
        );
    }
    // The crosshair over the scene: white, whatever the scene's depth.
    assert!(near(overlay_px(img, 639, 352), [1.0, 1.0, 1.0], 12.0));
    // The list: the first row is cut at the clip's top (y 300), not dropped;
    // above the clip nothing of it shows.
    let row = [0.2, 0.22, 0.26];
    assert!(
        near(overlay_px(img, 1100, 302), row, 1.5),
        "the cut row at the clip's top {:?}",
        overlay_px(img, 1100, 302)
    );
    let without = {
        let g = built("overlay");
        let mut bare = g.scene;
        let o = bare.overlay.as_mut().expect("overlay");
        o.sprites.retain(|s| s.clip.is_none());
        o.texts.retain(|t| t.clip.is_none());
        let mut r = Renderer::new_blocking(&g.desc).expect("renderer");
        for f in 0..g.frames {
            if let Some(update) = g.update {
                update(&mut bare, f);
            }
            r.render(&bare, &RenderTarget::Offscreen).expect("render");
        }
        r.read_offscreen().expect("capture")
    };
    for x in (990..1230).step_by(7) {
        for y in [282u32, 290, 298, 452, 460, 470] {
            assert_eq!(
                overlay_px(img, x, y),
                overlay_px(&without, x, y),
                "a clipped row shows at ({x}, {y}), outside its clip"
            );
        }
    }
}

/// On a non-sRGB output (a WebGPU canvas's Bgra8Unorm) the overlay blends
/// in linear light through the output's sRGB view: the same pixels as on the
/// sRGB output, opaque and translucent (the panel at (30, 30), 85 % over a
/// light background, came out 29 levels darker when it blended encoded
/// values).
#[test]
fn the_overlay_blends_in_linear_light_on_a_non_srgb_output() {
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::new(0.5, 0.5, 0.5));
    og_tests::fx_scenes::overlay_hud(&mut scene).expect("hud");
    let desc = RenderDesc {
        width: 1280,
        height: 720,
        ..RenderDesc::default()
    };
    let (rgba, _) = frame(&desc, &scene, 1);
    let bgra_desc = RenderDesc {
        output_format: Some(og_render::wgpu::TextureFormat::Bgra8Unorm),
        ..desc
    };
    let (bgra, _) = frame(&bgra_desc, &scene, 1);
    for (x, y) in [(100, 102), (240, 102), (1100, 302), (1180, 90), (30, 30)] {
        let (a, b) = (overlay_px(&rgba, x, y), overlay_px(&bgra, x, y));
        assert!(
            a.iter()
                .zip(b)
                .all(|(p, q)| (*p as i32 - q as i32).abs() <= 2),
            "({x}, {y}): {a:?} on sRGB, {b:?} on BGRA"
        );
    }
    assert!(near(overlay_px(&bgra, 100, 102), [0.9, 0.18, 0.05], 1.5));
}

/// Capture of a BGRA, non-sRGB output (a WebGPU canvas's format) encodes
/// sRGB in the shader and swizzles on readback: the same picture as the
/// default sRGB RGBA output.
#[test]
fn capture_of_a_bgra_output_matches_rgba() {
    let g = built("exposure");
    let (rgba, _) = frame(&g.desc, &g.scene, 1);
    let desc = RenderDesc {
        output_format: Some(og_render::wgpu::TextureFormat::Bgra8Unorm),
        ..g.desc
    };
    let (bgra, _) = frame(&desc, &g.scene, 1);
    let c = og_tests::compare::compare(
        &bgra.pixels,
        &rgba.pixels,
        rgba.width,
        rgba.height,
        &Default::default(),
    )
    .expect("same size");
    println!(
        "bgra vs rgba: rmse {:.5}, max diff {}",
        c.rmse, c.max_channel_diff
    );
    assert!(c.max_channel_diff <= 2, "max diff {}", c.max_channel_diff);
}

// ---------------------------------------------------------------------------
// The E1 effect scenes (og_tests::fx_scenes): each against its reference,
// with the counts that say the feature ran.
// ---------------------------------------------------------------------------

/// Night camp after 120 ticks: six systems (flames, smoke, embers, magic,
/// sparkles, sparks), GPU on the full tier.
#[test]
fn particles_match_their_reference() {
    let s = check("particles").stats;
    assert_eq!(s.particle_systems, 6);
    assert!(s.particle_capacity >= 1_000);
    if adapter().tier == Tier::Full {
        assert_eq!(s.particles_cpu, 0, "simulated in compute");
    } else {
        assert!(s.particles_cpu > 100);
    }
}

/// The same scene on the CPU path (any tier): collision is skipped.
#[test]
fn particles_cpu_match_their_reference() {
    let s = check("particles-cpu").stats;
    assert!(
        s.particles_cpu > 200,
        "{} live CPU particles",
        s.particles_cpu
    );
    assert_eq!(s.particle_collision_skipped, 2);
}

#[test]
fn fx_2d_matches_its_reference() {
    let s = check("fx-2d").stats;
    assert_eq!((s.lights_2d, s.occluders_2d), (3, 5));
    assert!(s.sprites > 24 * 14 + 5, "{} sprites", s.sprites);
}

/// Hollowbell FINDINGS #9: three tile layers in 2D lighting's range, two
/// lights and two occluders; the window's sky layer opts out.
#[test]
fn tilemap_lit_matches_its_reference() {
    let s = check("tilemap-lit").stats;
    assert_eq!((s.lights_2d, s.occluders_2d), (2, 2));
    assert_eq!((s.tile_chunks, s.sprites), (3, 1));
}

#[test]
fn trails_beams_match_their_reference() {
    let s = check("trails-beams").stats;
    // Three trails, one ribbon, two beams.
    assert_eq!(s.strips, 6);
}

#[test]
fn decals_match_their_reference() {
    assert_eq!(check("decals").stats.decals, 5);
}

#[test]
fn post_grade_matches_its_reference() {
    check("post-grade");
}

/// AgX: the bloom scene's emissive spheres keep their hue in the glow but
/// their over-exposed cores go towards white, where ACES skews them.
#[test]
fn post_agx_matches_its_reference() {
    check("post-agx");
    // The same frame through ACES (the bloom scene): AgX's halo around the
    // orange sphere is less saturated (mean max-min channel spread).
    let chroma = |img: &RgbaImage| {
        let mut sum = 0.0;
        for y in 330..470 {
            for x in 180..360 {
                let i = ((y * img.width + x) * 4) as usize;
                let p = &img.pixels[i..i + 3];
                sum += (p.iter().max().unwrap() - p.iter().min().unwrap()) as f32;
            }
        }
        sum / (140.0 * 180.0)
    };
    let mut agx = built("post-agx");
    let mut aces = built("bloom");
    let (agx, aces) = (
        gfx::render_built(&mut agx).expect("agx").image,
        gfx::render_built(&mut aces).expect("aces").image,
    );
    let (a, b) = (chroma(&agx), chroma(&aces));
    println!("orange halo chroma: agx {a:.1}, aces {b:.1}");
    assert!(a < b * 0.9, "agx {a:.1} against aces {b:.1}");
}

/// A cooked environment map lights the spheres and fills the background:
/// the mirror chrome shows the studio's key light as a sharp highlight that
/// spreads and dims with roughness, and turning the map moves it.
#[test]
fn environment_map_matches_its_reference() {
    check("environment-map");
    let render = |turn: Option<f32>| {
        let mut g = built("environment-map");
        match turn {
            Some(t) => g.scene.environment_map.as_mut().expect("a map").rotation = t,
            None => g.scene.environment_map = None,
        }
        gfx::render_built(&mut g).expect("render").image
    };
    let lit = render(Some(0.6));
    // The sharpest step of luma down a sphere's middle column (top row,
    // chrome): the mirror shows the studio's horizon as an edge, the
    // roughest blurs it into a gradient.
    let sharpest = |img: &RgbaImage, cx: u32| {
        let at = |y: u32| {
            let i = ((y * img.width + cx) * 4) as usize;
            luma(&[img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], 255])
        };
        (180..320)
            .map(|y| (at(y + 1) - at(y)).abs())
            .fold(0.0f32, f32::max)
    };
    let (mirror, rough) = (sharpest(&lit, 208), sharpest(&lit, 1072));
    println!("chrome sharpest step: mirror {mirror:.1}, rough {rough:.1}");
    assert!(
        mirror > 3.0 * rough,
        "mirror {mirror:.1} against rough {rough:.1}"
    );
    let diff = |a: &RgbaImage, b: &RgbaImage| {
        a.pixels
            .iter()
            .zip(&b.pixels)
            .map(|(x, y)| x.abs_diff(*y) as f32)
            .sum::<f32>()
            / a.pixels.len() as f32
    };
    // Turned half way round, the bright rim strip leaves the view.
    let turned = render(Some(0.6 + std::f32::consts::PI));
    let unlit = render(None);
    let (t, u) = (diff(&lit, &turned), diff(&lit, &unlit));
    println!("mean channel change: turned {t:.1}, the sky instead {u:.1}");
    assert!(t > 4.0, "turning the map moves it: {t:.1}");
    assert!(u > 4.0, "the map replaces the sky: {u:.1}");
}

#[test]
fn ao_gtao_matches_its_reference() {
    assert!(check("ao-gtao").stats.prepass_draws > 0);
}

#[test]
fn ao_ssao_matches_its_reference() {
    assert!(check("ao-ssao").stats.prepass_draws > 0);
}

#[test]
fn dof_low_matches_its_reference() {
    check("dof-low");
}

#[test]
fn dof_high_matches_its_reference() {
    check("dof-high");
}

#[test]
fn god_rays_match_their_reference() {
    check("god-rays");
}

#[test]
fn motion_blur_matches_its_reference() {
    check("motion-blur");
}

#[test]
fn stylised_matches_its_reference() {
    assert!(check("stylised").stats.prepass_draws > 0);
}

/// One glTF model's three instances (one flashing, one tinted) and two
/// skinned cylinders (one flashing) share their materials: each look rides
/// in its instance's record, so the flash is drawn without a draw of its
/// own, and the flashed instance alone is lighter than its twin.
#[test]
fn model_flash_matches_its_reference() {
    let report = check("model-flash");
    let s = report.stats;
    assert_eq!((s.skinned_instances, s.joints), (2, 8));
    // Without the looks: the same draws.
    let looks = built("model-flash");
    let mut plain = built("model-flash");
    for i in &mut plain.scene.instances {
        i.fx = og_render::InstanceFx::NONE;
    }
    for i in &mut plain.scene.skinned_instances {
        i.fx = og_render::InstanceFx::NONE;
    }
    let (with, stats) = frame(&looks.desc, &looks.scene, looks.frames);
    let (without, plain_stats) = frame(&plain.desc, &plain.scene, plain.frames);
    assert_eq!(stats.draw_calls, plain_stats.draw_calls);
    // The middle model is lighter with its flash, the left one unchanged.
    let column = |img: &RgbaImage, x0: u32, x1: u32| {
        let mut sum = 0.0;
        for y in 150..560 {
            for x in x0..x1 {
                let i = ((y * img.width + x) * 4) as usize;
                sum += luma(&[img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], 255]);
            }
        }
        sum
    };
    let (mid_with, mid_without) = (column(&with, 560, 720), column(&without, 560, 720));
    assert!(
        mid_with > mid_without * 1.15,
        "the flashed model is lighter: {mid_with} against {mid_without}"
    );
    assert_eq!(
        column(&with, 200, 360),
        column(&without, 200, 360),
        "the unflashed twin is untouched"
    );
}

#[test]
fn skinning_matches_its_reference() {
    let s = check("skinning").stats;
    assert_eq!((s.skinned_instances, s.joints), (5, 20));
}

/// `vox-keyframes` (owed row 63): a `.vox` whose block sits under a
/// transform keyed at frames 0 and 2, cooked and posed at each key side by
/// side, matches its reference, and the frame shows the block at both
/// keys: left of its copy's blue post on the left copy (frame 0), right of
/// it and higher on the right copy (frame 2). On the importer that read
/// only a transform's first key, the file cooked static (no clip: the
/// scene is not built) and both copies held the block on the left.
#[test]
fn vox_keyframes_show_the_keyed_block_at_both_keys() {
    let s = check("vox-keyframes").stats;
    assert_eq!(
        (s.skinned_instances, s.joints),
        (2, 2),
        "the block, skinned once per copy"
    );
    let img = gfx::render("vox-keyframes").expect("render").image;
    // The centroid (x, y) of the pixels `is` picks in columns [x0, x1).
    let centroid = |x0: u32, x1: u32, is: &dyn Fn(&[u8]) -> bool| {
        let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0.0f64);
        for y in 0..img.height {
            for x in x0..x1 {
                let i = ((y * img.width + x) * 4) as usize;
                if is(&img.pixels[i..i + 3]) {
                    sx += x as f64;
                    sy += y as f64;
                    n += 1.0;
                }
            }
        }
        assert!(n > 500.0, "pixels to measure in {x0}..{x1}: {n}");
        (sx / n, sy / n)
    };
    let red = |p: &[u8]| p[0] > 120 && p[1] < 80 && p[2] < 80;
    let blue = |p: &[u8]| p[2] > 120 && p[0] < 80 && p[1] < 120;
    let half = img.width / 2;
    let (left_block, left_post) = (centroid(0, half, &red), centroid(0, half, &blue));
    let (right_block, right_post) = (
        centroid(half, img.width, &red),
        centroid(half, img.width, &blue),
    );
    assert!(
        left_block.0 < left_post.0 - 60.0,
        "frame 0: the block left of the post ({left_block:?}, post {left_post:?})"
    );
    assert!(
        right_block.0 > right_post.0 + 60.0,
        "frame 2: the block right of the post ({right_block:?}, post {right_post:?})"
    );
    assert!(
        right_block.1 < left_block.1 - 40.0,
        "frame 2: the block lifted ({right_block:?} against {left_block:?})"
    );
}
