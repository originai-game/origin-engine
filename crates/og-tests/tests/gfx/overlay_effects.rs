use super::*;

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
/// sparkles, sparks), simulated in compute.
#[test]
fn particles_match_their_reference() {
    let s = check("particles").stats;
    assert_eq!(s.particle_systems, 6);
    assert!(s.particle_capacity >= 1_000);
    assert_eq!(s.particles_cpu, 0, "simulated in compute");
}

/// The same scene on the CPU path: collision is skipped.
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

/// 642 normal-mapped decals, every one drawn (R7: no 32 cap).
#[test]
fn many_normal_mapped_decals_match_their_reference() {
    let s = check("decals-many").stats;
    assert_eq!((s.decals, s.decals_dropped), (642, 0));
}

#[test]
fn post_grade_matches_its_reference() {
    check("post-grade");
}

/// Colour adjustments baked with a grading LUT at 0.6 into one table.
#[test]
fn post_adjust_matches_its_reference() {
    check("post-adjust");
}

/// Khronos PBR Neutral: the bloom scene's spheres keep their colour under
/// the knee and roll to white above it.
#[test]
fn post_neutral_matches_its_reference() {
    check("post-neutral");
}

/// A centred barrel lens over the spike's scene, fitted to its corners.
#[test]
fn post_lens_matches_its_reference() {
    check("post-lens");
}

/// The bloom scene through a zoom blur, bloomed after it.
#[test]
fn post_blur_matches_its_reference() {
    check("post-blur");
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

// FXC cannot compile this paint pipeline on Windows; the shipping native
// host must carry DXC rather than falling back to the legacy compiler.
#[test]
fn stylise_paint_matches_its_reference() {
    check("stylise-paint");
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

/// `skeleton-2d` (S9; reference `skeleton-2d.png`): a 2D rig posed by
/// og-anim on three planks (flat, rising, falling; the third mirrored by
/// flipX) and drawn as sprite-batch triangles under 2D lighting. The planks
/// are one sprite run and each rig its own triangle run; the cape, a
/// weighted mesh, hangs behind each keeper: left of the two facing right,
/// right of the mirrored one. Drawn without the triangle path the capes are
/// missing; with flipX ignored the third cape is on the wrong side.
#[test]
fn skeleton_2d_draws_rigs_as_sprite_triangles() {
    let s = check("skeleton-2d").stats;
    assert_eq!(
        (s.sprites, s.sprite_draws),
        (3, 4),
        "three planks in one run, three rigs"
    );
    assert_eq!(s.lights_2d, 1);
    let img = gfx::render("skeleton-2d").expect("render").image;
    // The cape's crimson (the coat is teal, the plank brown, the face pale).
    let cape = |p: &[u8]| {
        p[0] > 110 && u32::from(p[1]) * 2 < u32::from(p[0]) && u32::from(p[2]) * 2 < u32::from(p[0])
    };
    // Each keeper stands at x -4, 0, 4 of a 6-unit-high frame: 120 px a unit.
    for (band, keeper, behind_left) in [
        (0..427u32, 160.0, true),
        (427..853, 640.0, true),
        (853..1280, 1120.0, false),
    ] {
        let (mut sx, mut n) = (0.0f64, 0.0f64);
        for y in 0..img.height {
            for x in band.clone() {
                let i = ((y * img.width + x) * 4) as usize;
                if cape(&img.pixels[i..i + 3]) {
                    sx += f64::from(x);
                    n += 1.0;
                }
            }
        }
        assert!(
            n > 1500.0,
            "the cape of the keeper at {keeper} px: {n} pixels"
        );
        let cx = sx / n;
        assert_eq!(
            cx < keeper,
            behind_left,
            "the cape of the keeper at {keeper} px is centred at {cx:.0}"
        );
    }
}

/// The mean linear-ish RGB (0..255) of `img` over `x`, `y`.
fn region_mean(img: &RgbaImage, x: std::ops::Range<u32>, y: std::ops::Range<u32>) -> [f64; 3] {
    let mut sum = [0.0f64; 3];
    let mut n = 0.0f64;
    for py in y {
        for px in x.clone() {
            let i = ((py * img.width + px) * 4) as usize;
            for (c, s) in sum.iter_mut().enumerate() {
                *s += f64::from(img.pixels[i + c]);
            }
            n += 1.0;
        }
    }
    sum.map(|s| s / n.max(1.0))
}

/// `probe-interior` (depth S6; reference `probe-interior.png`): a chrome
/// sphere in a closed room reflecting the room's reflection probe.
#[test]
fn probe_interior_matches_its_reference() {
    check("probe-interior");
}

/// The chrome sphere shows the room, not the sky: without the probe its
/// lower half reflects the procedural sky's brown ground (red above blue);
/// with it, the room's grey floor and blue wall (blue above red). A probe
/// sampled without box projection, or not at all, keeps the sky's ground.
#[test]
fn a_reflection_probe_puts_the_room_in_the_chrome_sphere() {
    let desc = scenes::base_desc();
    let with = frame(&desc, &og_tests::depth_render_scenes::probe_scene(true), 1).0;
    let without = frame(&desc, &og_tests::depth_render_scenes::probe_scene(false), 1).0;
    // The sphere spans about x 520..690, y 270..445 of the 1280x720 frame.
    let (x, y) = (560..650, 380..430);
    let (a, b) = (
        region_mean(&with, x.clone(), y.clone()),
        region_mean(&without, x, y),
    );
    println!("chrome sphere, lower half: with the probe {a:?}, without {b:?}");
    assert!(
        b[0] > b[2] + 10.0,
        "without the probe, the sky's ground: {b:?}"
    );
    assert!(a[2] > a[0] + 10.0, "with the probe, the room: {a:?}");
}

/// `ssr-wet-floor` (depth S6; reference `ssr-wet-floor.png`):
/// screen-space reflections on a mirror-smooth floor under a black sky.
#[test]
fn ssr_wet_floor_matches_its_reference() {
    check("ssr-wet-floor");
}

/// The floor under the red sphere shows its reflection with SSR on and is
/// dark without it (the environment is black and there is no probe, so
/// nothing else reflects it); a reflection drawn upside down or offset
/// would leave this patch dark.
#[test]
fn ssr_reflects_the_sphere_in_the_floor() {
    let desc = scenes::base_desc();
    let on = frame(&desc, &og_tests::depth_render_scenes::ssr_scene(true), 1).0;
    let off = frame(&desc, &og_tests::depth_render_scenes::ssr_scene(false), 1).0;
    // The reflection lies about x 440..550, y 500..620.
    let (x, y) = (460..530, 540..600);
    let (a, b) = (
        region_mean(&on, x.clone(), y.clone()),
        region_mean(&off, x, y),
    );
    println!("floor under the sphere: ssr on {a:?}, off {b:?}");
    assert!(
        a[0] > b[0] + 20.0 && a[0] > a[1] * 1.5,
        "no red reflection: {a:?} against {b:?}"
    );
}
