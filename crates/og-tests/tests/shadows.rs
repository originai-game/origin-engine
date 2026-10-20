//! Shadows and antialiasing by tier (`crates/og-render/README.md`
//! §Shadows and antialiasing by tier): the reference frames of the round's
//! scenes, and behaviour read from frames and stats. More than two local
//! lights cast shadows, the ones left out past the tier's count are the
//! least radiant at the camera, dual-paraboloid point shadows draw the
//! cube's shadows in a third of the passes, PCSS widens the sun's penumbra
//! with the caster's height, and FXAA and TAA's sharpening work on edges.

use og_render::{
    Antialias, FrameStats, Material, PointShadows, RenderDesc, RenderScene, RenderTarget, Renderer,
    RgbaImage, TaaHistory, Tier,
};
use og_tests::compare::{Tolerance, compare};
use og_tests::{gfx, scenes, shadow_scenes};
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

fn tier() -> Tier {
    Renderer::new_blocking(&scenes::base_desc())
        .expect("a GPU adapter")
        .info()
        .tier
}

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

fn luma(p: &[u8]) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// Pixels of `a` darker than `b`'s by a quarter and 8 levels.
fn darker(a: &RgbaImage, b: &RgbaImage) -> usize {
    a.pixels
        .chunks(4)
        .zip(b.pixels.chunks(4))
        .filter(|(a, b)| luma(a) + 8.0 < 0.75 * luma(b))
        .count()
}

fn rmse(a: &RgbaImage, b: &RgbaImage) -> f64 {
    compare(
        &a.pixels,
        &b.pixels,
        a.width,
        a.height,
        &Tolerance::default(),
    )
    .expect("same size")
    .rmse
}

#[test]
fn shadows_soft_matches_its_reference() {
    let report = check("shadows-soft");
    assert_eq!(report.stats.shadow_passes, report.info.shadow_cascades);
}

#[test]
fn point_shadows_paraboloid_match_their_reference_in_three_passes() {
    let report = check("point-shadows-paraboloid");
    assert_eq!(report.info.point_shadows, PointShadows::Paraboloid);
    assert_eq!(
        (report.stats.shadow_passes, report.stats.shadowed_lights),
        (3, 2),
        "a spot (1 view) and a point light (2 hemispheres)"
    );
}

/// The paraboloid hemispheres draw the cube faces' shadows: the two frames
/// of point-spot-lights differ only along shadow edges.
#[test]
fn paraboloid_point_shadows_match_cube_faces_within_tolerance() {
    let scene = scenes::point_spot_lights();
    let cube = RenderDesc {
        point_shadows: PointShadows::Cube,
        ..scenes::base_desc()
    };
    let para = RenderDesc {
        point_shadows: PointShadows::Paraboloid,
        ..scenes::base_desc()
    };
    let (a, sa) = frame(&cube, &scene, 1);
    let (b, sb) = frame(&para, &scene, 1);
    assert_eq!((sa.shadow_passes, sb.shadow_passes), (7, 3));
    let c = compare(
        &a.pixels,
        &b.pixels,
        a.width,
        a.height,
        &Tolerance::default(),
    )
    .expect("same size");
    println!(
        "cube vs paraboloid: rmse {:.5}, {} outliers",
        c.rmse, c.outliers
    );
    if let Err(e) = c.check(&Tolerance::default()) {
        panic!("paraboloid vs cube: {e}");
    }
    // And the point light's shadows are there: without them the frame
    // loses its dark wedges behind the pillars.
    let mut unshadowed = scene;
    unshadowed.lights[1].cast_shadows = false;
    let (c2, _) = frame(&para, &unshadowed, 1);
    assert!(
        darker(&b, &c2) > 2_000,
        "{} pixels shadowed",
        darker(&b, &c2)
    );
}

/// Eight shadowed lights on the full tier (38 passes: six points with
/// cube faces, two spots); the two far, dim ones are left out.
#[test]
fn shadows_local_8_matches_its_reference_on_the_full_tier() {
    if tier() != Tier::Full {
        let scene = shadow_scenes::many_shadowed_lights(true);
        let (_, stats) = frame(&scenes::base_desc(), &scene, 1);
        assert_eq!(
            (stats.shadowed_lights, stats.shadowed_lights_dropped),
            (og_render::NO_COMPUTE_SHADOWED_LIGHTS as u32, 6)
        );
        return;
    }
    let report = check("shadows-local-8");
    let s = report.stats;
    assert_eq!(report.info.shadowed_lights, 8);
    assert_eq!((s.shadowed_lights, s.shadowed_lights_dropped), (8, 2));
    assert_eq!(s.shadow_passes, 6 * 6 + 2);
}

/// The lights left out past the tier's count are the least radiant at the
/// camera, whatever their order in the scene: the frame is the one of the
/// scene where only they never cast.
#[test]
fn the_shadowed_lights_kept_are_the_brightest_at_the_camera() {
    let desc = scenes::base_desc();
    let max = Renderer::new_blocking(&desc)
        .expect("renderer")
        .info()
        .shadowed_lights as usize;
    let mut scene = shadow_scenes::many_shadowed_lights(true);
    // The far pair first in the scene: order does not pick them.
    scene.lights.rotate_right(2);
    let (kept, stats) = frame(&desc, &scene, 1);
    assert_eq!(stats.shadowed_lights as usize, max.min(10));
    let mut only = scene;
    let eye = only.camera.position;
    let mut by_score: Vec<usize> = (0..only.lights.len()).collect();
    let score = |l: &og_render::Light| {
        l.color.max_element() * l.intensity / (l.position - eye).length_squared().max(1.0)
    };
    by_score.sort_by(|&a, &b| {
        score(&only.lights[b])
            .total_cmp(&score(&only.lights[a]))
            .then(a.cmp(&b))
    });
    for &i in &by_score[max.min(10)..] {
        only.lights[i].cast_shadows = false;
    }
    let (expected, _) = frame(&desc, &only, 1);
    assert_eq!(
        kept.pixels, expected.pixels,
        "the dimmest lost their shadows"
    );
    // The far pair never cast here (they are the dimmest).
    assert!(by_score[8..].iter().all(|&i| i < 2), "{by_score:?}");
}

/// Every shadowed light darkens the ground behind its pillar: eight
/// shadowed lights shadow far more of it than the first two.
#[test]
fn more_than_two_local_lights_cast_shadows() {
    let desc = scenes::base_desc();
    let scene = shadow_scenes::many_shadowed_lights(false);
    let (all, stats) = frame(&desc, &scene, 1);
    let mut scene = scene;
    for l in scene.lights.iter_mut().skip(2) {
        l.cast_shadows = false;
    }
    let (first_two, two_stats) = frame(&desc, &scene, 1);
    for l in &mut scene.lights {
        l.cast_shadows = false;
    }
    let (unshadowed, _) = frame(&desc, &scene, 1);
    let (d_all, d_two) = (darker(&all, &unshadowed), darker(&first_two, &unshadowed));
    println!(
        "shadowed pixels: {d_all} with {} lights, {d_two} with {}",
        stats.shadowed_lights, two_stats.shadowed_lights
    );
    assert_eq!(two_stats.shadowed_lights, 2);
    assert!(stats.shadowed_lights >= 4);
    assert!(d_all > 2 * d_two, "{d_all} vs {d_two}");
}

/// The shadow array grows past its first twelve layers once a scene needs
/// it, and a renderer that grew draws the same frames as a fresh one.
#[test]
fn the_local_shadow_array_grows_and_frames_stay_the_same() {
    let desc = scenes::base_desc();
    let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
    let two = scenes::point_spot_lights();
    renderer
        .render(&two, &RenderTarget::Offscreen)
        .expect("render");
    let before = renderer.read_offscreen().expect("capture");
    let many = shadow_scenes::many_shadowed_lights(true);
    renderer
        .render(&many, &RenderTarget::Offscreen)
        .expect("render");
    let grown = renderer.read_offscreen().expect("capture");
    renderer
        .render(&two, &RenderTarget::Offscreen)
        .expect("render");
    let after = renderer.read_offscreen().expect("capture");
    assert_eq!(before.pixels, after.pixels);
    let (fresh, _) = frame(&desc, &many, 1);
    assert_eq!(grown.pixels, fresh.pixels);
}

/// A plain white ground under the soft-shadow scene's casters, the sun
/// alone lighting it: shadow or lit, and penumbra pixels between.
fn plain_soft_scene(soft: bool) -> RenderScene {
    let mut scene = shadow_scenes::soft_shadows_scene(soft);
    let ground = scene.instances[0].material;
    scene.set_material(
        ground,
        Material {
            base_color: [0.8, 0.8, 0.8, 1.0],
            roughness: 1.0,
            ..Default::default()
        },
    );
    scene.environment.intensity = 0.0;
    scene
}

/// Ground pixels of the lower 40 % (the ground near the casters) strictly
/// between shadow and lit: those whose unshadowed frame `lit` shows the
/// plain ground's own level, at 15 % to 85 % of it in `img`.
fn penumbra(img: &RgbaImage, lit: &RgbaImage) -> usize {
    let start = (img.width * img.height * 3 / 5) as usize * 4;
    let mut levels: Vec<f32> = lit.pixels[start..].chunks(4).map(luma).collect();
    levels.sort_by(f32::total_cmp);
    let ground = levels[levels.len() / 2];
    img.pixels[start..]
        .chunks(4)
        .zip(lit.pixels[start..].chunks(4))
        .filter(|(p, l)| {
            let (p, l) = (luma(p), luma(l));
            (l - ground).abs() < 4.0 && p > 0.15 * l && p < 0.85 * l
        })
        .count()
}

/// PCSS: the sun's penumbrae widen (many more pixels between shadow and
/// lit) while the hard frame has edges only.
#[test]
fn soft_shadows_widen_the_penumbra() {
    let desc = scenes::base_desc();
    let (hard, _) = frame(&desc, &plain_soft_scene(false), 1);
    let (soft, _) = frame(&desc, &plain_soft_scene(true), 1);
    if let Ok(dir) = std::env::var("OG_SHADOW_FRAMES") {
        for (name, img) in [("hard", &hard), ("soft", &soft)] {
            let path = PathBuf::from(format!("{dir}/plain-{name}.png"));
            gfx::save_png(&path, img).expect("write");
        }
    }
    let mut unshadowed = plain_soft_scene(false);
    unshadowed.light.cast_shadows = false;
    let (lit, _) = frame(&desc, &unshadowed, 1);
    let (h, s) = (penumbra(&hard, &lit), penumbra(&soft, &lit));
    println!("penumbra pixels: hard {h}, soft {s}");
    assert!(s > 3 * h, "soft {s} vs hard {h}");
    assert!(rmse(&hard, &soft) > 0.002);
}

/// Cascade blending is off by default and leaves frames as they were; on,
/// it changes the frame only where cascades meet or reach (their extended
/// fit), never by much.
#[test]
fn cascade_blending_is_opt_in() {
    let desc = scenes::base_desc();
    let mut scene = scenes::cascades_200m(true);
    assert_eq!(
        (scene.light.cascade_blend, scene.light.shadow_softness),
        (0.0, 0.0),
        "off by default"
    );
    let (plain, _) = frame(&desc, &scene, 1);
    scene.light.cascade_blend = 0.2;
    let (b, _) = frame(&desc, &scene, 1);
    let r = rmse(&plain, &b);
    println!("cascade blend 0.2 vs none: rmse {r:.5}");
    assert!(r > 0.0 && r < 0.02, "rmse {r}");
}

#[test]
fn fxaa_matches_its_reference() {
    let report = check("fxaa");
    assert_eq!(report.info.antialias, Antialias::Fxaa);
}

#[test]
fn taa_sharp_matches_its_reference_on_the_full_tier() {
    if tier() != Tier::Full {
        return;
    }
    let report = check("taa-sharp");
    assert_eq!(report.info.antialias, Antialias::Taa);
}

/// Mean absolute Laplacian of the luma: how much detail the frame keeps.
fn detail(img: &RgbaImage) -> f64 {
    let (w, h) = (img.width as usize, img.height as usize);
    let l = |x: usize, y: usize| luma(&img.pixels[(y * w + x) * 4..]) as f64;
    let mut sum = 0.0;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            sum += (4.0 * l(x, y) - l(x - 1, y) - l(x + 1, y) - l(x, y - 1) - l(x, y + 1)).abs();
        }
    }
    sum / ((w - 2) * (h - 2)) as f64
}

/// Under a moving camera TAA's history softens the frame; Catmull-Rom
/// history keeps more of it, sharpening more again.
#[test]
fn catmull_rom_history_and_sharpening_keep_detail_under_motion() {
    let render = |history: TaaHistory, sharpen: f32| {
        let desc = RenderDesc {
            antialias: Antialias::Taa,
            taa_history: history,
            taa_sharpen: sharpen,
            ..scenes::base_desc()
        };
        let mut scene = scenes::thin_geometry();
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        let mut stats = FrameStats::default();
        for f in 0..16 {
            shadow_scenes::pan_thin_geometry(&mut scene, f);
            stats = renderer
                .render(&scene, &RenderTarget::Offscreen)
                .expect("render");
        }
        (renderer.read_offscreen().expect("capture"), stats)
    };
    let (bilinear, plain_stats) = render(TaaHistory::Bilinear, 0.0);
    let (catmull, _) = render(TaaHistory::CatmullRom, 0.0);
    let (sharp, sharp_stats) = render(TaaHistory::CatmullRom, 0.5);
    let (b, c, s) = (detail(&bilinear), detail(&catmull), detail(&sharp));
    println!("detail: bilinear {b:.3}, catmull-rom {c:.3}, + sharpen 0.5 {s:.3}");
    assert!(c > b, "Catmull-Rom {c} vs bilinear {b}");
    assert!(s > c * 1.05, "sharpened {s} vs {c}");
    assert_eq!(
        sharp_stats.graph_passes,
        plain_stats.graph_passes + 1,
        "one sharpening pass"
    );
}

/// FXAA on every tier: one pass after the tonemap; `Fxaa` is never
/// resolved to anything else.
#[test]
fn fxaa_is_one_pass_on_every_tier() {
    let scene = scenes::thin_geometry();
    let (off, off_stats) = frame(&scenes::base_desc(), &scene, 1);
    let desc = RenderDesc {
        antialias: Antialias::Fxaa,
        ..scenes::base_desc()
    };
    let r = Renderer::new_blocking(&desc).expect("renderer");
    assert_eq!(r.info().antialias, Antialias::Fxaa);
    let (fxaa, stats) = frame(&desc, &scene, 1);
    assert_eq!(stats.graph_passes, off_stats.graph_passes + 1);
    // Edges move, flat areas do not: most pixels are untouched.
    let changed = off
        .pixels
        .chunks(4)
        .zip(fxaa.pixels.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    let frac = changed as f64 / (off.width * off.height) as f64;
    println!("FXAA changed {:.2}% of pixels", frac * 100.0);
    assert!(frac > 0.005 && frac < 0.25, "{frac}");
}
