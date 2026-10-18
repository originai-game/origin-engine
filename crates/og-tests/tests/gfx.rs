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
    // At most one draw per batch per pass: 3 cascades + main, then sky and post.
    assert!(
        s.draw_calls <= 2 * 4 + 2,
        "{} draw calls for 2 batches",
        s.draw_calls
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

/// Lights beyond the eighth are not drawn and are reported.
#[test]
fn lights_beyond_the_limit_are_counted() {
    let mut scene = scenes::dim_scene(1.0);
    scene.lights = vec![Light::default(); 10];
    let (_, stats) = frame(&scenes::base_desc(), &scene, 1);
    assert_eq!((stats.lights, stats.lights_dropped), (8, 2));
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
/// antialiasing every pixel is black or white; MSAA and TAA produce partial
/// coverage along the edges. Returns the partially covered pixels and the
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
    println!("partially covered pixels: off {off}, msaa {msaa} ({msaa_mode:?}), taa {taa}");
    assert_eq!(taa_mode, Antialias::Taa);
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

#[test]
fn skinning_matches_its_reference() {
    let s = check("skinning").stats;
    assert_eq!((s.skinned_instances, s.joints), (5, 20));
}
