//! Shadows and antialiasing (`crates/og-render/README.md`
//! §Shadows and antialiasing): the reference frames of the round's
//! scenes, and behaviour read from frames and stats. More than two local
//! lights cast shadows, the ones left out past the tier's count are the
//! least radiant at the camera, dual-paraboloid point shadows draw the
//! cube's shadows in a third of the passes, PCSS widens the sun's penumbra
//! with the caster's height, and FXAA and TAA's sharpening work on edges.

use og_render::{
    Antialias, Culling, FrameStats, Material, PointShadows, RenderDesc, RenderScene, RenderTarget,
    Renderer, RgbaImage, TaaHistory, Tonemap,
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
    assert_eq!(
        report.stats.shadow_passes,
        report.info.shadow_cascades.min(3)
    );
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

/// The crypt: the sun's cascades and the lamp's point shadow; the lamp's
/// own glass box casts none.
#[test]
fn shadows_crypt_matches_its_reference() {
    let report = check("shadows-crypt");
    assert_eq!(report.stats.shadowed_lights, 1);
    let faces = match report.info.point_shadows {
        PointShadows::Paraboloid => 2,
        _ => 6,
    };
    assert_eq!(
        report.stats.shadow_passes,
        report.info.shadow_cascades.min(3) + faces
    );
}

/// A light inside its own glass that casts no shadow lights the room on
/// cube faces and paraboloids alike, and the two draw it within
/// tolerance; glass that casts shuts the light in on both (web findings
/// #4: every mesh cast, and paraboloids stored the faces turned towards
/// the light, so The Kestrel Stair's crypt lamp lit its crypt with
/// paraboloids and not with cube faces).
#[test]
fn a_light_in_a_non_casting_fixture_lights_its_room_on_both_point_shadow_storages() {
    let shut = || {
        let mut scene = shadow_scenes::crypt_scene();
        assert_eq!(
            scene.instances.iter().filter(|i| !i.cast_shadows).count(),
            1,
            "the lamp's glass alone casts no shadow"
        );
        for i in &mut scene.instances {
            i.cast_shadows = true;
        }
        scene
    };
    let mut lit_frames = Vec::new();
    for mode in [PointShadows::Cube, PointShadows::Paraboloid] {
        let desc = RenderDesc {
            point_shadows: mode,
            ..scenes::base_desc()
        };
        let (lit, stats) = frame(&desc, &shadow_scenes::crypt_scene(), 1);
        let (dark, dark_stats) = frame(&desc, &shut(), 1);
        assert!(
            dark_stats.shadow_casters > stats.shadow_casters,
            "the casting glass is drawn into the shadow passes, the other not"
        );
        let shut_out = darker(&dark, &lit);
        println!("{mode:?}: {shut_out} pixels darker with the glass casting");
        assert!(
            shut_out > 100_000,
            "{mode:?}: only {shut_out} pixels lost the lamp"
        );
        lit_frames.push(lit);
    }
    let (cube, para) = (&lit_frames[0], &lit_frames[1]);
    let c = compare(
        &cube.pixels,
        &para.pixels,
        cube.width,
        cube.height,
        &Tolerance::default(),
    )
    .expect("same size");
    println!(
        "cube vs paraboloid: rmse {:.5}, {} outliers",
        c.rmse, c.outliers
    );
    if let Err(e) = c.check(&Tolerance::default()) {
        panic!("paraboloid vs cube in the crypt: {e}");
    }
}

/// Eight shadowed lights (38 passes: six points with cube faces, two
/// spots), the default quality's budget; the two far, dim ones are left
/// out, though the renderer allows sixteen.
#[test]
fn shadows_local_8_matches_its_reference() {
    let report = check("shadows-local-8");
    let s = report.stats;
    assert_eq!(report.info.shadowed_lights, 16);
    assert_eq!((s.shadowed_lights, s.shadowed_lights_dropped), (8, 2));
    assert_eq!(s.shadow_passes, 6 * 6 + 2);
}

/// The lights left out past the count are the least radiant at the
/// camera, whatever their order in the scene: the frame is the one of the
/// scene where only they never cast.
#[test]
fn the_shadowed_lights_kept_are_the_brightest_at_the_camera() {
    let desc = scenes::base_desc();
    let mut scene = shadow_scenes::many_shadowed_lights(true);
    // The frame's budget: the renderer's cap (16) and the quality's (8 by
    // default), so the scene's ten shadowed lights lose two.
    let max = (Renderer::new_blocking(&desc)
        .expect("renderer")
        .info()
        .shadowed_lights
        .min(scene.quality.shadowed_lights)) as usize;
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

/// `img`'s luma in linear light (sRGB decoded) at pixel coordinates
/// (`x`, `y`), bilinear between pixel centres.
fn linear_at(img: &RgbaImage, x: f32, y: f32) -> f32 {
    let decode = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let texel = |x: i64, y: i64| {
        let x = x.clamp(0, img.width as i64 - 1) as u32;
        let y = y.clamp(0, img.height as i64 - 1) as u32;
        let i = ((y * img.width + x) * 4) as usize;
        let p = &img.pixels[i..i + 3];
        0.2126 * decode(p[0]) + 0.7152 * decode(p[1]) + 0.0722 * decode(p[2])
    };
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let (x0, y0) = (x0 as i64, y0 as i64);
    let top = texel(x0, y0) * (1.0 - tx) + texel(x0 + 1, y0) * tx;
    let bottom = texel(x0, y0 + 1) * (1.0 - tx) + texel(x0 + 1, y0 + 1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// The sun's direct light reaching `img`'s ground pixels of the lower 40 %
/// (the ground near the casters), as a share of what reaches them
/// unshadowed (`lit`), in linear light: every pixel whose unshadowed level
/// is the plain ground's.
fn ground_shares(img: &RgbaImage, lit: &RgbaImage) -> Vec<(usize, f32)> {
    let (w, h) = (img.width, img.height);
    let at = |m: &RgbaImage, i: usize| {
        linear_at(m, (i as u32 % w) as f32 + 0.5, (i as u32 / w) as f32 + 0.5)
    };
    let start = (w * h * 3 / 5) as usize;
    let mut levels: Vec<f32> = (start..(w * h) as usize).map(|i| at(lit, i)).collect();
    levels.sort_by(f32::total_cmp);
    let ground = levels[levels.len() / 2];
    (start..(w * h) as usize)
        .filter_map(|i| {
            let l = at(lit, i);
            ((l - ground).abs() < 0.02 * ground && l > 0.01).then(|| (i, at(img, i) / l))
        })
        .collect()
}

/// The weather's cover reaches the sun's shadow
/// (`DirectionalLight::cover`): under rain's dimming (0.35 of the sun
/// left) most of the lost light is scattered across the sky (diffusion
/// 0.80, a smoothstep at 5/7), so the umbra keeps about half the direct
/// light the open ground takes (0.52) instead of none, and the edges
/// soften (PCSS at a fifth of a metre of penumbra per metre): many more
/// ground pixels lie between the umbra's level and the open ground's.
/// The rain frame's sun shadows stayed hard and black under a full
/// overcast (the site imagery audit, 2026-11-04).
#[test]
fn a_cover_fills_and_softens_the_suns_shadow() {
    let desc = scenes::base_desc();
    let scene = |cover: Option<f32>, shadows: bool| {
        let mut s = plain_soft_scene(false);
        s.post.tonemap = Tonemap::None;
        if let Some(beam) = cover {
            s.light.cover(beam);
        }
        s.light.cast_shadows = shadows;
        s
    };
    let (clear, _) = frame(&desc, &scene(None, true), 1);
    let (clear_lit, _) = frame(&desc, &scene(None, false), 1);
    let (rain, _) = frame(&desc, &scene(Some(0.35), true), 1);
    let (rain_lit, _) = frame(&desc, &scene(Some(0.35), false), 1);
    if let Ok(dir) = std::env::var("OG_SHADOW_FRAMES") {
        for (name, img) in [("clear", &clear), ("rain", &rain)] {
            let path = PathBuf::from(format!("{dir}/cover-{name}.png"));
            gfx::save_png(&path, img).expect("write");
        }
    }
    let clear_shares = ground_shares(&clear, &clear_lit);
    let rain_shares: std::collections::HashMap<usize, f32> =
        ground_shares(&rain, &rain_lit).into_iter().collect();
    // The clear sky's umbra, read in both frames.
    let mut umbra: Vec<f32> = clear_shares
        .iter()
        .filter(|&&(_, v)| v < 0.05)
        .filter_map(|(i, _)| rain_shares.get(i).copied())
        .collect();
    assert!(umbra.len() > 2_000, "umbra pixels: {}", umbra.len());
    umbra.sort_by(f32::total_cmp);
    let filled = umbra[umbra.len() / 2];
    // Between the umbra and the open ground: the clear sky's from 15 % to
    // 85 % of the light, the rain's from 15 % to 85 % of the way from its
    // fill (0.521) to the open ground.
    const FILL: f32 = 0.521;
    let between = |v: f32, floor: f32| {
        let t = (v - floor) / (1.0 - floor);
        t > 0.15 && t < 0.85
    };
    let hard = clear_shares
        .iter()
        .filter(|&&(_, v)| between(v, 0.0))
        .count();
    let soft = rain_shares.values().filter(|&&v| between(v, FILL)).count();
    println!(
        "umbra under rain takes {filled:.3} of the direct light ({} pixels); between: clear {hard}, rain {soft}",
        umbra.len()
    );
    assert!(
        (0.45..0.9).contains(&filled),
        "the rain's umbra takes {filled:.3} of the direct light"
    );
    assert!(soft > 3 * hard, "rain {soft} vs clear {hard}");
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
fn smaa_matches_its_reference() {
    for (scene, antialias) in [
        ("smaa", Antialias::Smaa),
        ("smaa-t2x", Antialias::SmaaT2x),
        ("smaa-t2x-moving", Antialias::SmaaT2x),
    ] {
        let report = check(scene);
        assert_eq!(report.info.antialias, antialias);
    }
}

#[test]
fn taa_sharp_matches_its_reference() {
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

/// Pixel `i` of `img`.
fn px(img: &RgbaImage, i: usize) -> &[u8] {
    &img.pixels[i * 4..i * 4 + 4]
}

/// shadow-flags (per-instance shadow flags, og.Mesh's and og.Model's
/// castShadow and receiveShadow): its reference, one pass per cascade.
#[test]
fn shadow_flags_matches_its_reference() {
    let report = check("shadow-flags");
    assert_eq!(
        report.stats.shadow_passes,
        report.info.shadow_cascades.min(3)
    );
}

fn flags_scene() -> shadow_scenes::FlagsScene {
    shadow_scenes::shadow_flags_scene().expect("trickster.glb")
}

/// A model's instances cast as their flag says: the trickster added as is
/// throws its shadow on the ground (gone when its instances cast none),
/// the one with `cast_shadows` off throws none (there once it casts), and
/// only casters are drawn into the cascades.
#[test]
fn a_model_casts_as_its_instances_say() {
    let desc = scenes::base_desc();
    let (base, base_stats) = frame(&desc, &flags_scene().scene, 1);
    let mut casting = flags_scene();
    for i in casting.model_no_cast.clone() {
        casting.scene.instances[i].cast_shadows = true;
    }
    let (both, both_stats) = frame(&desc, &casting.scene, 1);
    let appeared = darker(&both, &base);
    println!("the second trickster casting: {appeared} pixels shadowed");
    assert!(appeared > 2_000, "{appeared}");
    assert!(both_stats.shadow_casters > base_stats.shadow_casters);
    let mut none = flags_scene();
    for i in none.model_as_is.clone() {
        none.scene.instances[i].cast_shadows = false;
    }
    let (without, _) = frame(&desc, &none.scene, 1);
    let gone = darker(&base, &without);
    println!("the first trickster as added: {gone} pixels of its shadow");
    assert!(gone > 2_000, "{gone}");
}

/// A box and a model with `receive_shadows` off, in a wall's shadow, are
/// lit as if unshadowed: by the sun's cascades and by a shadowed point
/// light alike, every pixel that moves from their receiving selves gains
/// light and is drawn as when the wall casts nothing; none darkens. Fails on shaders that read the shadow maps for every instance
/// (no pixel gains).
#[test]
fn surfaces_that_receive_no_shadows_are_lit_as_if_unshadowed() {
    let desc = scenes::base_desc();
    for point in [false, true] {
        let build = |receive_all: bool, wall_casts: bool| {
            let mut f = flags_scene();
            if point {
                f.scene.light.intensity = 0.0;
                f.scene.light.cast_shadows = false;
                f.scene.lights = vec![og_render::Light {
                    position: glam::Vec3::new(6.5, 1.6, -3.0),
                    color: glam::Vec3::new(1.0, 0.85, 0.6),
                    intensity: 60.0,
                    range: 16.0,
                    cast_shadows: true,
                    ..Default::default()
                }];
            }
            if receive_all {
                for i in &mut f.scene.instances {
                    i.receive_shadows = true;
                }
            }
            f.scene.instances[f.wall].cast_shadows = wall_casts;
            f.scene
        };
        let (flags, _) = frame(&desc, &build(false, true), 1);
        let (all, _) = frame(&desc, &build(true, true), 1);
        let (no_wall, _) = frame(&desc, &build(false, false), 1);
        let light = if point { "point light" } else { "sun" };
        let mut gained = 0;
        let mut as_unshadowed = 0;
        let mut elsewhere = 0;
        for i in 0..(flags.width * flags.height) as usize {
            let (f, a) = (px(&flags, i), px(&all, i));
            // A pixel within one level on every channel has not moved: it
            // is the 8-bit rounding of the same lit value. lavapipe compiles
            // the shader for the host CPU, and on some of GitHub's runners
            // (AMD EPYC 9V45 and 7763; not on Intel Xeon 8573C, nor on the
            // orbs) the receiving and non-receiving paths' HDR values
            // differ in their last bit at one silhouette pixel, which ACES
            // rounds one blue level apart (tests/evidence/ci/known-frames/
            // NOTES.md). A surface that read the shadow maps would move by
            // far more.
            if f.iter().zip(a).take(3).all(|(x, y)| x.abs_diff(*y) <= 1) {
                continue;
            }
            if luma(f) >= luma(a) {
                gained += 1;
                if (luma(f) - luma(px(&no_wall, i))).abs() <= 2.0 {
                    as_unshadowed += 1;
                }
            } else {
                elsewhere += 1;
            }
        }
        println!(
            "{light}: {gained} pixels lit as if unshadowed ({as_unshadowed} as with no wall), {elsewhere} elsewhere moved"
        );
        assert!(gained > 1_000, "{light}: {gained}");
        assert!(as_unshadowed as f64 >= 0.98 * gained as f64, "{light}");
        assert_eq!(elsewhere, 0, "{light}");
    }
}

/// The floor just past the box's lit-away face, in pixels of the contact
/// scene's frame: points 1 to 4 cm out from the face, along its middle.
fn contact_strip(img: &RgbaImage) -> Vec<(u32, u32)> {
    let (w, h) = (img.width as f32, img.height as f32);
    let view_proj = contact_view_proj(img);
    let face = shadow_scenes::CONTACT_BOX.x * 0.5;
    let mut out = Vec::new();
    for dx in [0.01f32, 0.02, 0.03, 0.04] {
        for k in -3..=3 {
            let p = view_proj.project_point3(glam::Vec3::new(face + dx, 0.0, k as f32 * 0.08));
            out.push((
                ((p.x * 0.5 + 0.5) * w) as u32,
                ((0.5 - p.y * 0.5) * h) as u32,
            ));
        }
    }
    out
}

/// The contact scene's floor pixels whose centre sees the floor between
/// `from` and `to` metres past the box's lit-away face (x = 0.4), with
/// nothing in front: along that face's base, rows at its middle seven
/// eighths.
fn base_strip(img: &RgbaImage, from: f32, to: f32) -> Vec<(u32, u32)> {
    let (w, h) = (img.width as f32, img.height as f32);
    let view_proj = contact_view_proj(img);
    let inverse = view_proj.inverse();
    let face = shadow_scenes::CONTACT_BOX.x * 0.5;
    let half = shadow_scenes::CONTACT_BOX * 0.5;
    let block = Caster::Box(
        glam::Vec3::new(-half.x, 0.0, -half.z),
        glam::Vec3::new(half.x, shadow_scenes::CONTACT_BOX.y, half.z),
    );
    let mut out = Vec::new();
    for k in -7..=7 {
        let base = view_proj.project_point3(glam::Vec3::new(face, 0.0, k as f32 * 0.05));
        let (bx, by) = ((base.x * 0.5 + 0.5) * w, (0.5 - base.y * 0.5) * h);
        for dy in -1i32..=1 {
            for dx in -4i32..=12 {
                let (x, y) = (bx as i32 + dx, by as i32 + dy);
                let ndc = glam::Vec2::new(
                    (x as f32 + 0.5) / w * 2.0 - 1.0,
                    1.0 - (y as f32 + 0.5) / h * 2.0,
                );
                let near = inverse.project_point3(ndc.extend(0.0));
                let far = inverse.project_point3(ndc.extend(1.0));
                let d = (far - near).normalize();
                if d.y >= 0.0 {
                    continue;
                }
                let t = -near.y / d.y;
                let q = near + d * t;
                let past = q.x - face;
                if past < from || past >= to || q.z.abs() > 0.35 {
                    continue;
                }
                if block.enter(near, d).is_some_and(|b| b < t - 1e-4) {
                    continue;
                }
                if !out.contains(&(x as u32, y as u32)) {
                    out.push((x as u32, y as u32));
                }
            }
        }
    }
    out
}

/// Local lights' shadows start at the caster's base: the floor right
/// beside a box standing on it, on the side away from the light, is in
/// shadow for a spot and a point light (cube and paraboloid), not lit
/// through a bias gap; and their edges are soft rather than stepped.
/// Right at the base too: the floor's pixels within a centimetre of the
/// face (the first one or two) take no more light than those 1 to 4 cm
/// out. The box's bottom face lies in the floor's plane, and a receiver
/// filtered over its own plane with a bias towards the light read it as
/// lit there: a broken 1 px grey line along the base (shadows-local-
/// contact, art review of 73e6c771a; a CPU model of the spot: 24 % to 53 %
/// of the light 1 mm out). A closed receiver now compares past its depth
/// (lighting.wgsl shadow_closed): under 1 % in a CPU model, and 2 % for a
/// paraboloid, which compares half a texel past to cover its straight-
/// edged triangles' misdrawn texels (Metal at a quarter texel: mean
/// 0.039, 0.286 at most). Local layers now hold their casters' midpoint
/// (shadow_mid.rs): the box's bottom face and top face meet halfway, well
/// above the floor, and every receiver compares as an open one.
///
/// Measured as the share of the light's direct light that reaches the
/// strip, in linear light (no tonemap, sRGB decoded) with the ambient
/// taken out (a frame with the light off): a ratio of display values
/// counts the sky's ambient, which sRGB lifts to a third of the lit
/// floor's value, and moved with every change to the ambient.
#[test]
fn local_shadows_touch_their_casters() {
    check("shadows-local-contact");
    let desc = scenes::base_desc();
    for (point, storage) in [
        (false, PointShadows::Cube),
        (true, PointShadows::Cube),
        (true, PointShadows::Paraboloid),
    ] {
        let desc = RenderDesc {
            point_shadows: storage,
            ..desc
        };
        let scene = |shadows: bool, on: bool| {
            let mut s = shadow_scenes::local_contact_scene(point, shadows);
            s.post.tonemap = Tonemap::None;
            if !on {
                s.lights[0].intensity = 0.0;
            }
            s
        };
        let (shadowed, _) = frame(&desc, &scene(true, true), 1);
        let (lit, _) = frame(&desc, &scene(false, true), 1);
        let (dark, _) = frame(&desc, &scene(false, false), 1);
        let strip = contact_strip(&shadowed);
        let at = |img: &RgbaImage, (x, y): (u32, u32)| {
            let i = ((y * img.width + x) * 4) as usize;
            let c = |v: u8| {
                let v = v as f32 / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            let p = &img.pixels[i..i + 3];
            0.2126 * c(p[0]) + 0.7152 * c(p[1]) + 0.0722 * c(p[2])
        };
        let direct = |img: &RgbaImage| -> f32 {
            strip
                .iter()
                .map(|&q| at(img, q) - at(&dark, q))
                .sum::<f32>()
                / strip.len() as f32
        };
        let (reached, full) = (direct(&shadowed), direct(&lit));
        assert!(
            full > 0.05,
            "the light reaches the strip unshadowed: {full:.3}"
        );
        let base = base_strip(&shadowed, 0.0, 0.01);
        assert!(base.len() >= 15, "base pixels: {}", base.len());
        let share = |q: &(u32, u32)| {
            let (x, y) = (q.0 as f32 + 0.5, q.1 as f32 + 0.5);
            let dark = linear_at(&dark, x, y);
            (linear_at(&shadowed, x, y) - dark) / (linear_at(&lit, x, y) - dark).max(1e-4)
        };
        let shares: Vec<f32> = base.iter().map(share).collect();
        let worst = shares.iter().copied().fold(0.0, f32::max);
        let mean = shares.iter().sum::<f32>() / shares.len() as f32;
        println!(
            "point {point} {storage:?}: the floor's {} pixels within 1 cm of the base take {mean:.3} of their direct light, {worst:.3} at most",
            base.len()
        );
        assert!(
            mean < 0.05 && worst < 0.2,
            "a lit line along the base: {mean:.3} of the direct light on average, {worst:.3} at most"
        );
        let ratio = reached / full;
        println!(
            "point {point} {storage:?}: floor beside the box takes {ratio:.3} of its direct light ({reached:.4} of {full:.4}, ambient {:.4})",
            strip.iter().map(|&q| at(&dark, q)).sum::<f32>() / strip.len() as f32
        );
        assert!(
            ratio < 0.1,
            "the floor beside the box is lit through the bias: {ratio:.3} of its direct light"
        );
    }
}

/// A floor of two coplanar single-sided quads facing apart (window-room.glb
/// in fx-showcase's volumetric station) is lit by a local light: its
/// down-facing quad is a caster at the floor's own depth, and a layer of
/// far sides compared under the closed rule (past its depth, lighting.wgsl
/// shadow_closed) shadowed the floor with itself, halving the station's
/// spot pool. With midpoints (shadow_mid.rs) the two quads resolve to
/// their shared plane, which the floor compares before. Measured as in
/// [`local_shadows_touch_their_casters`]: the share of the direct light
/// that reaches the floor in the lower half of the frame.
#[test]
fn a_twin_floor_is_lit_by_local_lights() {
    let desc = scenes::base_desc();
    for (point, storage) in [
        (false, PointShadows::Cube),
        (true, PointShadows::Cube),
        (true, PointShadows::Paraboloid),
    ] {
        let desc = RenderDesc {
            point_shadows: storage,
            ..desc
        };
        let scene = |shadows: bool, on: bool| {
            let mut s = shadow_scenes::twin_floor_scene(point, shadows);
            s.post.tonemap = Tonemap::None;
            if !on {
                s.lights[0].intensity = 0.0;
            }
            s
        };
        let (shadowed, _) = frame(&desc, &scene(true, true), 1);
        let (lit, _) = frame(&desc, &scene(false, true), 1);
        let (dark, _) = frame(&desc, &scene(false, false), 1);
        let (mut reached, mut full) = (0.0f32, 0.0f32);
        for y in (shadowed.height * 3 / 4..shadowed.height).step_by(2) {
            for x in (0..shadowed.width).step_by(2) {
                let (x, y) = (x as f32 + 0.5, y as f32 + 0.5);
                let d = linear_at(&dark, x, y);
                reached += linear_at(&shadowed, x, y) - d;
                full += linear_at(&lit, x, y) - d;
            }
        }
        let share = reached / full.max(1e-4);
        println!("point {point} {storage:?}: the twin floor takes {share:.3} of its direct light");
        assert!(full > 1.0, "the light reaches the floor: {full:.3}");
        assert!(
            share > 0.97,
            "the twin floor shadows itself: {share:.3} of its direct light"
        );
    }
}

/// A spot's shadow edge far from its caster runs straight: the box's top
/// edge's shadow on the floor, about 0.73 m past the box, crosses the
/// spot's texels at a shallow angle (about 11 degrees), where a 3x3
/// filter printed their steps along it (art review of 73e6c771a). Where
/// half the direct light reaches, found along the floor at 2 mm steps
/// for each z from -0.5 to 0.25 m (clear of its corners' penumbrae), the
/// edge strays under 0.3 cm rms from a
/// straight fit. A CPU model of the scene (texels 2.45 cm there) put it at
/// 0.53 cm under the tent and 0.14 under local_wide's 7x7.
#[test]
fn a_spot_shadows_far_edge_runs_straight() {
    let desc = scenes::base_desc();
    let scene = |shadows: bool, on: bool| {
        let mut s = shadow_scenes::local_contact_scene(false, shadows);
        s.post.tonemap = Tonemap::None;
        if !on {
            s.lights[0].intensity = 0.0;
        }
        s
    };
    let (shadowed, _) = frame(&desc, &scene(true, true), 1);
    let (lit, _) = frame(&desc, &scene(false, true), 1);
    let (dark, _) = frame(&desc, &scene(false, false), 1);
    let (w, h) = (shadowed.width as f32, shadowed.height as f32);
    let view_proj = contact_view_proj(&shadowed);
    let share = |p: glam::Vec3| {
        let q = view_proj.project_point3(p);
        let (x, y) = ((q.x * 0.5 + 0.5) * w, (0.5 - q.y * 0.5) * h);
        let d = linear_at(&dark, x, y);
        (linear_at(&shadowed, x, y) - d) / (linear_at(&lit, x, y) - d).max(1e-4)
    };
    let mut edge = Vec::new();
    for k in 0..=75 {
        let z = -0.5 + 0.01 * k as f32;
        let mut last = (0.95f32, share(glam::Vec3::new(0.95, 0.0, z)));
        assert!(
            last.1 < 0.2,
            "z {z}: the floor at x 0.95 is in shadow ({:.3})",
            last.1
        );
        for i in 1..=200 {
            let x = 0.95 + 0.002 * i as f32;
            let v = share(glam::Vec3::new(x, 0.0, z));
            if v >= 0.5 {
                edge.push((
                    z,
                    last.0 + (x - last.0) * (0.5 - last.1) / (v - last.1).max(1e-6),
                ));
                break;
            }
            last = (x, v);
        }
    }
    assert_eq!(edge.len(), 76, "the edge is found along every row");
    let n = edge.len() as f32;
    let (mz, mx) = (
        edge.iter().map(|e| e.0).sum::<f32>() / n,
        edge.iter().map(|e| e.1).sum::<f32>() / n,
    );
    let b = edge.iter().map(|e| (e.0 - mz) * (e.1 - mx)).sum::<f32>()
        / edge.iter().map(|e| (e.0 - mz).powi(2)).sum::<f32>();
    let wander = edge
        .iter()
        .map(|e| e.1 - (mx + b * (e.0 - mz)))
        .collect::<Vec<_>>();
    let rms = (wander.iter().map(|r| r * r).sum::<f32>() / n).sqrt() * 100.0;
    let most = wander.iter().fold(0.0f32, |m, r| m.max(r.abs())) * 100.0;
    println!("the spot's far edge at x {mx:.3} m strays {rms:.3} cm rms, {most:.3} cm at most");
    assert!(
        rms < 0.3,
        "the far edge steps: {rms:.3} cm rms ({most:.3} at most)"
    );
}

/// The contact scenes' camera (`shadow_scenes::local_contact_camera`)
/// for a frame of `img`'s size.
fn contact_view_proj(img: &RgbaImage) -> glam::Mat4 {
    let cam = shadow_scenes::local_contact_camera();
    glam::camera::rh::proj::directx::perspective(
        50f32.to_radians(),
        img.width as f32 / img.height as f32,
        0.1,
        100.0,
    ) * glam::camera::rh::view::look_at_mat4(cam.position, cam.target, glam::Vec3::Y)
}

/// A caster of the contact scenes, for the false-contact probes' geometry.
#[derive(Clone, Copy)]
enum Caster {
    /// Least and greatest corners.
    Box(glam::Vec3, glam::Vec3),
    /// Centre and radius.
    Ball(glam::Vec3, f32),
}

impl Caster {
    /// Where the ray from `o` along unit `d` enters it, if ahead of `o`.
    fn enter(self, o: glam::Vec3, d: glam::Vec3) -> Option<f32> {
        match self {
            Caster::Box(lo, hi) => {
                let inv = d.recip();
                let (a, b) = ((lo - o) * inv, (hi - o) * inv);
                let near = a.min(b).max_element();
                let far = a.max(b).min_element();
                (near <= far && near > 0.0).then_some(near)
            }
            Caster::Ball(c, r) => {
                let oc = o - c;
                let b = oc.dot(d);
                let disc = b * b - (oc.length_squared() - r * r);
                let t = -b - disc.max(0.0).sqrt();
                (disc >= 0.0 && t > 0.0).then_some(t)
            }
        }
    }

    fn distance(self, q: glam::Vec3) -> f32 {
        match self {
            Caster::Box(lo, hi) => (lo - q).max(q - hi).max(glam::Vec3::ZERO).length(),
            Caster::Ball(c, r) => q.distance(c) - r,
        }
    }
}

/// Whether the segment from `a` to `b` passes through one of `casters`.
fn blocked(a: glam::Vec3, b: glam::Vec3, casters: &[Caster]) -> bool {
    let (d, len) = ((b - a).normalize(), a.distance(b));
    casters
        .iter()
        .any(|c| c.enter(a, d).is_some_and(|t| t < len - 1e-4))
}

/// Where a contact scene's frame shows lit floor or wall that lies just
/// past the silhouette of one of `behind`, at a depth gap: points the
/// light reaches, clear of every caster by 8 cm at their distance from it
/// (more than a local map's filter and normal offset reach there, so no
/// map shades them), whose pixel and its neighbours two pixels away show
/// that receiver, and from which the ray towards the light passes behind
/// one of `behind` as the camera sees it within 48 pixels (inside a
/// screen-space trace's reach) without touching anything. A trace that
/// takes a silhouette in front of a far surface for an occluder darkens
/// them. With no `behind`, every such point clear of the casters by 30 cm,
/// passing behind nothing: open lit floor and wall, which shadow acne
/// (a map comparing a lit surface against itself) darkens.
fn behind_silhouettes(
    img: &RgbaImage,
    casters: &[Caster],
    behind: Option<&[Caster]>,
) -> Vec<(u32, u32)> {
    use glam::Vec3;
    let light = shadow_scenes::CONTACT_LIGHT;
    let eye = shadow_scenes::local_contact_camera().position;
    let view_proj = contact_view_proj(img);
    let inv = view_proj.inverse();
    let (w, h) = (img.width as f32, img.height as f32);
    let to_px = |p: Vec3| {
        let q = view_proj.project_point3(p);
        ((q.x * 0.5 + 0.5) * w, (0.5 - q.y * 0.5) * h)
    };
    // What the camera sees through a pixel's centre: 0 the floor, 1 the
    // wall, 2 a caster.
    let seen = |x: f32, y: f32| -> u8 {
        let ndc = Vec3::new((x + 0.5) / w * 2.0 - 1.0, 1.0 - (y + 0.5) / h * 2.0, 0.5);
        let d = (inv.project_point3(ndc) - eye).normalize();
        let floor = if d.y < 0.0 { -eye.y / d.y } else { f32::MAX };
        let wall = if d.z < 0.0 {
            (-2.0 - eye.z) / d.z
        } else {
            f32::MAX
        };
        let caster = casters
            .iter()
            .filter_map(|c| c.enter(eye, d))
            .fold(f32::MAX, f32::min);
        if caster < floor.min(wall) {
            2
        } else if floor < wall {
            0
        } else {
            1
        }
    };
    let clearance = |p: Vec3| {
        let far = p.distance(light);
        (1..100)
            .map(|i| p.lerp(light, i as f32 / 100.0))
            .flat_map(|q| {
                casters
                    .iter()
                    .map(move |c| c.distance(q) * far / q.distance(light))
            })
            .fold(f32::MAX, f32::min)
    };
    let floor = (-60..=60).flat_map(|x| {
        (-39..40).map(move |z| (Vec3::new(x as f32 * 0.05, 0.0, z as f32 * 0.05), 0u8))
    });
    let wall = (-60..=60).flat_map(|x| {
        (1..50).map(move |y| (Vec3::new(x as f32 * 0.05, y as f32 * 0.05, -2.0), 1u8))
    });
    let mut out = Vec::new();
    for (p, receiver) in floor.chain(wall) {
        let (x, y) = to_px(p);
        if x < 4.0 || y < 4.0 || x >= w - 4.0 || y >= h - 4.0 || blocked(eye, p, casters) {
            continue;
        }
        let (x, y) = (x.floor(), y.floor());
        let clean = [-2.0, 0.0, 2.0].iter().all(|&ox| {
            [-2.0, 0.0, 2.0]
                .iter()
                .all(|&oy| seen(x + ox, y + oy) == receiver)
        });
        let Some(behind) = behind else {
            if clean && clearance(p) >= 0.3 {
                out.push((x as u32, y as u32));
            }
            continue;
        };
        if !clean || clearance(p) < 0.08 {
            continue;
        }
        let d = (light - p).normalize();
        let passes_behind = (1..200)
            .map(|i| p + d * (i as f32 * 0.01))
            .take_while(|&q| {
                let (qx, qy) = to_px(q);
                (qx - x).hypot(qy - y) <= 48.0
            })
            .any(|q| blocked(eye, q, behind));
        if passes_behind {
            out.push((x as u32, y as u32));
        }
    }
    out
}

/// The share of their direct light that `points` keep in `shadowed`
/// (`lit`: the light unshadowed, `dark`: off): the least and the mean, and
/// how many points the light reaches by 16 levels.
fn kept_light(
    points: &[(u32, u32)],
    shadowed: &RgbaImage,
    lit: &RgbaImage,
    dark: &RgbaImage,
) -> (f32, f32, usize) {
    let at = |img: &RgbaImage, (x, y): (u32, u32)| {
        let i = ((y * img.width + x) * 4) as usize;
        luma(&img.pixels[i..i + 3])
    };
    // Only where the light gives 16 levels or more (inside a spot's cone).
    let ratios: Vec<f32> = points
        .iter()
        .filter(|&&q| at(lit, q) - at(dark, q) >= 16.0)
        .map(|&q| (at(shadowed, q) - at(dark, q)) / (at(lit, q) - at(dark, q)))
        .collect();
    let least = ratios.iter().copied().fold(f32::MAX, f32::min);
    (
        least,
        ratios.iter().sum::<f32>() / ratios.len().max(1) as f32,
        ratios.len(),
    )
}

/// The standard deviation of the share of their direct light that the
/// lit `points` keep (as `kept_light`), and their count.
fn kept_spread(
    points: &[(u32, u32)],
    shadowed: &RgbaImage,
    lit: &RgbaImage,
    dark: &RgbaImage,
) -> (usize, f32) {
    let at = |img: &RgbaImage, (x, y): (u32, u32)| {
        let i = ((y * img.width + x) * 4) as usize;
        luma(&img.pixels[i..i + 3])
    };
    let r: Vec<f32> = points
        .iter()
        .filter(|&&q| at(lit, q) - at(dark, q) >= 16.0)
        .map(|&q| (at(shadowed, q) - at(dark, q)) / (at(lit, q) - at(dark, q)))
        .collect();
    let n = r.len().max(1) as f32;
    let mean = r.iter().sum::<f32>() / n;
    (
        r.len(),
        (r.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n).sqrt(),
    )
}

/// Contact shadows never darken what lies behind a silhouette, and the
/// local maps never darken open lit surfaces (acne): the lit
/// floor and wall just past the contact scene's box, and past a large
/// ball and a thin pole, at a depth gap behind them, where a screen-space
/// trace towards the light passes behind them (the false contact the art
/// director held on contact 497: a blurred bar beside the box's corner,
/// a dark column along the ball). Each keeps its direct light, for a spot
/// and for both point shadow storages.
#[test]
fn contact_shadows_never_darken_what_lies_behind_a_silhouette() {
    use shadow_scenes::{CONTACT_BOX, SILHOUETTE_BALL, SILHOUETTE_POLE};
    let desc = scenes::base_desc();
    let half = CONTACT_BOX * 0.5;
    let boxed = Caster::Box(
        glam::Vec3::new(-half.x, 0.0, -half.z),
        glam::Vec3::new(half.x, CONTACT_BOX.y, half.z),
    );
    let small_ball = Caster::Ball(glam::Vec3::new(-1.0, 0.25, 0.9), 0.25);
    let ball = Caster::Ball(SILHOUETTE_BALL.0, SILHOUETTE_BALL.1);
    let pole = Caster::Box(SILHOUETTE_POLE.0, SILHOUETTE_POLE.1);
    for (point, storage) in [
        (false, PointShadows::Cube),
        (true, PointShadows::Cube),
        (true, PointShadows::Paraboloid),
    ] {
        let desc = RenderDesc {
            point_shadows: storage,
            ..desc
        };
        // The scene shadowed, its light unshadowed, and its light off.
        let frames = |scene: &dyn Fn() -> RenderScene| {
            let take = |shadows: bool, on: bool| {
                let mut s = scene();
                s.post.tonemap = Tonemap::None;
                s.lights[0].cast_shadows = shadows;
                if !on {
                    s.lights[0].intensity = 0.0;
                }
                frame(&desc, &s, 1).0
            };
            (take(true, true), take(false, true), take(false, false))
        };
        let contact = frames(&|| shadow_scenes::local_contact_scene(point, true));
        let silhouettes = frames(&|| shadow_scenes::contact_silhouettes_scene(point));
        for (name, (shadowed, lit, dark), casters, behind, least) in [
            ("box", &contact, vec![boxed, small_ball], vec![boxed], 60),
            ("ball", &silhouettes, vec![ball, pole], vec![ball], 25),
            ("thin pole", &silhouettes, vec![ball, pole], vec![pole], 60),
        ] {
            let points = behind_silhouettes(shadowed, &casters, Some(&behind));
            let (min, mean, n) = kept_light(&points, shadowed, lit, dark);
            println!(
                "point {point} {storage:?}: {n} lit points behind the {name}'s silhouette keep {min:.3} to {mean:.3} (mean) of their direct light"
            );
            assert!(n >= least, "{name}: {n} lit points of {}", points.len());
            assert!(
                min > 0.85 && mean > 0.95,
                "{name}: the floor and wall behind its silhouette darkened (least {min:.3}, mean {mean:.3})"
            );
        }
        // Open lit floor and wall, far from every caster: no acne.
        for (name, (shadowed, lit, dark), casters) in [
            ("contact", &contact, vec![boxed, small_ball]),
            ("silhouettes", &silhouettes, vec![ball, pole]),
        ] {
            let points = behind_silhouettes(shadowed, &casters, None);
            let (min, mean, n) = kept_light(&points, shadowed, lit, dark);
            println!(
                "point {point} {storage:?}: {n} open lit points of the {name} scene keep {min:.3} to {mean:.3} (mean) of their direct light"
            );
            assert!(n >= 1_000, "{name}: {n} lit points of {}", points.len());
            // Stripes or banding: the share kept varies from point to point.
            let (_, spread) = kept_spread(&points, shadowed, lit, dark);
            println!("  spread of the share kept: {spread:.4}");
            assert!(
                spread < 0.02,
                "{name}: open lit floor and wall vary by {spread:.4} (stripes or banding)"
            );
            assert!(
                min > 0.9 && mean > 0.98,
                "{name}: open lit floor and wall darkened, shadow acne (least {min:.3}, mean {mean:.3})"
            );
        }
    }
}

/// Four cascades (the high preset's count) draw four passes, and over the
/// same shadow distance the near shadows come out sharper than three's:
/// the frame against three cascades' differs in the foreground.
#[test]
fn shadows_cascades_4_matches_its_reference() {
    let report = check("shadows-cascades-4");
    assert_eq!(report.stats.shadow_passes, 4);
    let desc = scenes::base_desc();
    let mut scene = scenes::cascades_200m(true);
    scene.quality.shadow_cascades = 3;
    let (three, _) = frame(&desc, &scene, 1);
    scene.quality.shadow_cascades = 4;
    let (four, stats) = frame(&desc, &scene, 1);
    assert_eq!(stats.shadow_passes, 4);
    let w = four.width as usize;
    let h = four.height as usize;
    let (mut sum, mut n) = (0.0f64, 0usize);
    for y in h * 2 / 3..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            for c in 0..3 {
                sum += four.pixels[i + c].abs_diff(three.pixels[i + c]) as f64;
            }
            n += 3;
        }
    }
    let near = sum / n as f64;
    println!("four cascades against three, the foreground: {near:.2}");
    assert!(near > 0.05, "the near cascade is finer: {near:.2}");
}

/// The static shadow cache (og-render's shadow_cache.rs): a sphere rolls
/// for four frames among the pillars, under the sun's cascades, a spot and
/// a point light's cube faces, then stands still. While it rolls, the
/// layers it does not reach keep their static casters and the ones it
/// reaches draw it over theirs; once it has stood still long enough every
/// layer is kept and nothing is drawn. The last frame is the one a
/// renderer that saw only the final scene draws, on the CPU and the GPU
/// culling paths.
#[test]
fn cached_shadow_layers_draw_what_fresh_ones_do() {
    for culling in [Culling::Cpu, Culling::Gpu] {
        let desc = RenderDesc {
            culling,
            ..scenes::base_desc()
        };
        let mut scene = scenes::point_spot_lights();
        scene.light.cast_shadows = true;
        // The sphere at (0.2, 0.5, -2.0), the last instance.
        let mover = scene.instances.len() - 1;
        let start = scene.instances[mover].transform;
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        let mut stats = Vec::new();
        for f in 0..16 {
            let x = 0.25 * f.min(4) as f32;
            scene.instances[mover].transform =
                glam::Mat4::from_translation(glam::Vec3::new(x, 0.0, 0.0)) * start;
            stats.push(
                renderer
                    .render(&scene, &RenderTarget::Offscreen)
                    .expect("render"),
            );
        }
        let cached = renderer.read_offscreen().expect("capture");
        let still = frame(&desc, &scene, 16).0;
        let fresh = frame(&desc, &scene, 1).0;
        for (i, s) in stats.iter().enumerate() {
            println!(
                "{culling:?} frame {i}: {} shadow passes, {} cached, {} casters drawn",
                s.shadow_passes, s.shadow_cached, s.shadow_casters
            );
        }
        let passes = stats[0].shadow_passes;
        assert!(passes >= 8, "the cascades and seven local faces: {passes}");
        assert_eq!(
            stats[0].shadow_cached, 0,
            "the first frame draws every layer"
        );
        // The first frame it moves, the layers its old sphere reaches draw
        // their static casters again (and save them); the others are kept.
        let first = &stats[0];
        let moved = &stats[1];
        assert!(
            moved.shadow_cached > 0 && moved.shadow_cached < passes,
            "{culling:?}: {moved:?}"
        );
        // Rolling on (frames 2 to 11, dynamic until 8 still frames): every
        // layer comes from the cache, and only the mover and the casters
        // drawn every frame are drawn over them: far fewer than all.
        // GPU culling counts its casters on the GPU and reads none back
        // here (`counts_lag` unset): the CPU path checks the counts, both
        // check which layers are kept and the frames.
        let counts = culling == Culling::Cpu;
        for s in &stats[2..=11] {
            assert_eq!(s.shadow_cached, passes, "{culling:?}: {s:?}");
            assert!(
                !counts || (s.shadow_casters > 0 && 2 * s.shadow_casters < first.shadow_casters),
                "{culling:?}: {s:?} against {first:?}"
            );
        }
        // Static again at frame 12: the layers its sphere reaches redraw.
        assert!(
            stats[12].shadow_cached < passes,
            "{culling:?}: {:?}",
            stats[12]
        );
        // Nothing drawn from frame 13 on.
        let last = stats.last().expect("frames");
        assert_eq!(last.shadow_passes, passes);
        assert_eq!(last.shadow_cached, passes, "{culling:?}: every layer kept");
        if counts {
            assert_eq!(last.shadow_casters, 0, "{culling:?}: nothing drawn");
        }
        let exact = Tolerance {
            channel: 1,
            max_outlier_fraction: 0.0,
            rmse: 0.001,
        };
        for (name, other) in [("still", &still), ("fresh", &fresh)] {
            let diff = compare(
                &cached.pixels,
                &other.pixels,
                cached.width,
                cached.height,
                &exact,
            )
            .expect("same size");
            println!("{culling:?} cached against {name}: {diff:?}");
            diff.check(&exact)
                .unwrap_or_else(|e| panic!("{culling:?}: cached against {name}: {e}"));
        }
    }
}

/// The static shadow cache never keeps what a vertex shader moves: a
/// foliage tree swaying in a steady wind under a still sun and camera is
/// drawn into the cascades every frame (its transform never changes), while
/// the still ground stays cached; without wind it joins the cache. The
/// last swaying frame is what a fresh renderer draws at that tick.
#[test]
fn a_swaying_caster_is_never_cached() {
    use glam::{Mat4, Vec3};
    use og_render::{Wind, foliage::FoliageVertex, meshlet::MeshFoliage, shapes};
    let desc = RenderDesc {
        culling: Culling::Cpu,
        ..scenes::base_desc()
    };
    let scene = |wind: bool, tick: u64| {
        let mut s = scenes::point_spot_lights();
        s.lights.clear();
        s.light.cast_shadows = true;
        let tree = s.add_mesh(shapes::cuboid(Vec3::new(0.4, 3.0, 0.4)));
        let vertices = s
            .mesh(tree)
            .expect("tree")
            .vertices
            .iter()
            .map(|v| {
                let up = (v.position[1] + 1.5) / 3.0;
                FoliageVertex {
                    branch: [0.0, -1.5, 0.0, 0.0],
                    leaf: [v.position[0], v.position[1], v.position[2], up],
                }
            })
            .collect();
        assert!(s.set_mesh_foliage(
            tree,
            Some(MeshFoliage {
                vertices,
                top: 3.0,
                canopy_center: Vec3::new(0.0, 1.0, 0.0),
                leaves: false,
            })
        ));
        let bark = s.add_material(Material::default());
        s.add_instance(tree, bark, Mat4::from_translation(Vec3::new(1.0, 1.5, 1.0)));
        s.instances.last_mut().expect("the tree").sway = 1.0;
        s.wind = wind.then(|| Wind {
            strength: 0.6,
            gust: 0.8,
            ..Wind::default()
        });
        s.tick = tick;
        s
    };
    for wind in [true, false] {
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        // One scene whose clock advances: a new scene (a new `id`) would
        // reset the renderer's caches, the shadow cache among them.
        let mut s = scene(wind, 0);
        let mut stats = Vec::new();
        for tick in 0..14 {
            s.tick = tick;
            stats.push(
                renderer
                    .render(&s, &RenderTarget::Offscreen)
                    .expect("render"),
            );
        }
        for (i, s) in stats.iter().enumerate() {
            println!(
                "wind {wind} frame {i}: {} passes, {} cached, {} casters drawn",
                s.shadow_passes, s.shadow_cached, s.shadow_casters
            );
        }
        let last = stats.last().expect("frames");
        // The ground and pillars are static: every layer keeps them.
        assert_eq!(last.shadow_cached, last.shadow_passes, "wind {wind}");
        if wind {
            // The tree, drawn over the cache into every cascade it reaches.
            for s in &stats[1..] {
                assert!(s.shadow_casters > 0, "a swaying tree was cached: {s:?}");
            }
            let swaying = renderer.read_offscreen().expect("capture");
            let (fresh, _) = frame(&desc, &scene(true, 13), 1);
            let exact = Tolerance {
                channel: 1,
                max_outlier_fraction: 0.0,
                rmse: 0.001,
            };
            let diff = compare(
                &swaying.pixels,
                &fresh.pixels,
                fresh.width,
                fresh.height,
                &exact,
            )
            .expect("same size");
            println!("swaying against fresh: {diff:?}");
            diff.check(&exact)
                .unwrap_or_else(|e| panic!("swaying against fresh: {e}"));
        } else {
            assert_eq!(last.shadow_casters, 0, "a still tree is cached: {last:?}");
        }
    }
}
