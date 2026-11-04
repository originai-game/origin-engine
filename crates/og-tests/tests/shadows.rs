//! Shadows and antialiasing (`crates/og-render/README.md`
//! §Shadows and antialiasing): the reference frames of the round's
//! scenes, and behaviour read from frames and stats. More than two local
//! lights cast shadows, the ones left out past the tier's count are the
//! least radiant at the camera, dual-paraboloid point shadows draw the
//! cube's shadows in a third of the passes, PCSS widens the sun's penumbra
//! with the caster's height, and FXAA and TAA's sharpening work on edges.

use og_render::{
    Antialias, Culling, FrameStats, Material, PointShadows, RenderDesc, RenderScene, RenderTarget,
    Renderer, RgbaImage, TaaHistory,
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
    let cam = shadow_scenes::local_contact_camera();
    let (w, h) = (img.width as f32, img.height as f32);
    let view_proj =
        glam::camera::rh::proj::directx::perspective(50f32.to_radians(), w / h, 0.1, 100.0)
            * glam::camera::rh::view::look_at_mat4(cam.position, cam.target, glam::Vec3::Y);
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

/// Local lights' shadows start at the caster's base: the floor right
/// beside a box standing on it, on the side away from the light, is in
/// shadow for a spot and a point light (cube and paraboloid), not lit
/// through a bias gap; and their edges are soft rather than stepped.
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
        let (shadowed, _) = frame(&desc, &shadow_scenes::local_contact_scene(point, true), 1);
        let (lit, _) = frame(&desc, &shadow_scenes::local_contact_scene(point, false), 1);
        let strip = contact_strip(&shadowed);
        let at = |img: &RgbaImage, (x, y): (u32, u32)| {
            let i = ((y * img.width + x) * 4) as usize;
            luma(&img.pixels[i..i + 4])
        };
        let ratio: f32 = strip
            .iter()
            .map(|&q| at(&shadowed, q) / at(&lit, q).max(1.0))
            .sum::<f32>()
            / strip.len() as f32;
        println!(
            "point {point} {storage:?}: floor beside the box at {ratio:.2} of its unshadowed light"
        );
        assert!(
            ratio < 0.35,
            "the floor beside the box is lit through the bias: {ratio:.2}"
        );
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
        let mut stats = Vec::new();
        for tick in 0..14 {
            stats.push(
                renderer
                    .render(&scene(wind, tick), &RenderTarget::Offscreen)
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
