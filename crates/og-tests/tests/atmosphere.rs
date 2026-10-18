//! The E5 round's rendering (`crates/og-render/README.md` §Atmosphere,
//! §Water, §Material shaders): each scene matches its reference for this
//! backend tier, and behaviour read from rendered frames: fog hides the
//! distance, not the foreground; the day-night look darkens from noon to
//! dusk to night and draws stars only at night; water darkens with depth,
//! foams where it meets geometry, ripples with its height field and
//! reflects; the 2D water line follows its surface and draws in the sprite
//! order; metaballs merge particles within reach and keep apart those that
//! are not; a dissolve shader cuts holes by its amount and the grass
//! shader moves only the grass with the clock.

use glam::{Mat4, Vec2, Vec3, Vec4};
use og_render::{
    Camera, RenderScene, RenderTarget, Renderer, RgbaImage, Sprite, Water2d,
};
use og_tests::atmo_scenes as atmo;
use og_tests::gfx;
use og_tests::scenes;
use std::path::PathBuf;
use std::sync::Arc;

fn check(scene: &str) -> gfx::GfxReport {
    let out = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx");
    match gfx::check(scene, &out) {
        Ok(report) => {
            println!("{report}");
            report
        }
        Err(e) => panic!("{e}"),
    }
}

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    r.render(scene, &RenderTarget::Offscreen).expect("render");
    r.read_offscreen().expect("capture")
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [f32; 3] {
    let i = ((y * img.width + x) * 4) as usize;
    [
        img.pixels[i] as f32,
        img.pixels[i + 1] as f32,
        img.pixels[i + 2] as f32,
    ]
}

fn luma(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

/// Mean luma over a rectangle.
fn mean(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += luma(px(img, x, y));
        }
    }
    sum / ((x1 - x0) * (y1 - y0)) as f32
}

/// Mean absolute difference over a rectangle (all channels).
fn diff(a: &RgbaImage, b: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            let (p, q) = (px(a, x, y), px(b, x, y));
            sum += (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
        }
    }
    sum / ((x1 - x0) * (y1 - y0) * 3) as f32
}

/// Luma's standard deviation over a rectangle (how much detail shows).
fn contrast(img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> f32 {
    let m = mean(img, x0, y0, x1, y1);
    let mut sum = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += (luma(px(img, x, y)) - m).powi(2);
        }
    }
    (sum / ((x1 - x0) * (y1 - y0)) as f32).sqrt()
}

#[test]
fn atmosphere_scenes_match_their_references() {
    for scene in ["atmosphere-fog", "atmosphere-dusk", "atmosphere-night"] {
        check(scene);
    }
}

#[test]
fn water_scenes_match_their_references() {
    let w = check("water-3d");
    assert!(w.stats.prepass_draws > 0, "water reads the depth prepass");
    check("water-2d");
    check("fluid-2d");
}

#[test]
fn shader_hooks_match_their_reference() {
    check("shader-hooks");
}

/// Fog takes the detail out of the far colonnade and leaves the near
/// ground almost as it was.
#[test]
fn fog_hides_the_distance_not_the_foreground() {
    // Distance fog only (height fog covers the near ground on purpose).
    let mut fogged = atmo::fog_scene();
    if let Some(f) = &mut fogged.fog {
        f.height_density = 0.0;
    }
    let fogged = frame(&fogged);
    let mut clear = atmo::fog_scene();
    clear.fog = None;
    let clear = frame(&clear);
    // The vanishing point of the colonnade, and the ground at the bottom.
    let far = (560, 300, 720, 400);
    let near = (300, 660, 980, 720);
    let far_clear = contrast(&clear, far.0, far.1, far.2, far.3);
    let far_fogged = contrast(&fogged, far.0, far.1, far.2, far.3);
    assert!(
        far_fogged < far_clear * 0.5,
        "far detail {far_clear:.1} -> {far_fogged:.1} with fog"
    );
    let d_near = diff(&clear, &fogged, near.0, near.1, near.2, near.3);
    let d_far = diff(&clear, &fogged, far.0, far.1, far.2, far.3);
    assert!(
        d_far > d_near * 2.0,
        "fog changes the distance ({d_far:.1}) more than the foreground ({d_near:.1})"
    );
}

/// The same place at noon, dusk and night: the light falls, the sky turns,
/// and only the night sky has stars (isolated bright pixels).
#[test]
fn day_night_darkens_towards_night_with_stars() {
    let noon = frame(&atmo::day_night_scene(12.0, false));
    let dusk = frame(&atmo::day_night_scene(18.2, false));
    let night = frame(&atmo::day_night_scene(23.0, false));
    let all = |img: &RgbaImage| mean(img, 0, 0, 1280, 720);
    let (a, b, c) = (all(&noon), all(&dusk), all(&night));
    assert!(a > b && b > c, "noon {a:.1} > dusk {b:.1} > night {c:.1}");
    // The light itself: the sun weakens towards dusk; at night the moon,
    // opposite the sun and dim, lights the scene.
    let (l12, l18, l23) = (
        og_render::day_night(12.0, 25.0, -20.0),
        og_render::day_night(18.2, 25.0, -20.0),
        og_render::day_night(23.0, 25.0, -20.0),
    );
    assert!(l12.light_intensity > 2.0 * l18.light_intensity, "{} vs {}", l12.light_intensity, l18.light_intensity);
    assert!(l23.light_direction.dot(l23.sun_direction) < -0.99, "the moon is opposite the sun");
    assert!(l23.light_intensity > 0.0 && l23.light_intensity < 0.2 * l12.light_intensity);
    // Stars: sky pixels much brighter than their neighbours, between the
    // two nearest pillars (x 420..840, y 20..300).
    let stars = |img: &RgbaImage| {
        let mut n = 0;
        for y in 21..300 {
            for x in 421..840 {
                let l = luma(px(img, x, y));
                let around = (luma(px(img, x - 1, y)) + luma(px(img, x + 1, y)) + luma(px(img, x, y - 1)) + luma(px(img, x, y + 1))) / 4.0;
                if l > around + 40.0 {
                    n += 1;
                }
            }
        }
        n
    };
    assert!(stars(&night) > 20, "night has stars ({})", stars(&night));
    assert_eq!(stars(&noon), 0, "noon has none");
}

/// The pool seen from above: the shallow near end is lighter than the
/// deep far end; foam rings the posts; the height field moves the surface;
/// reflectivity changes what the surface shows.
#[test]
fn water_darkens_with_depth_foams_ripples_and_reflects() {
    let above = |f: &dyn Fn(&mut og_render::Water)| {
        let mut s = atmo::water_scene();
        s.camera = Camera {
            position: Vec3::new(0.0, 14.0, -5.0),
            target: Vec3::new(0.0, 0.0, -5.01),
            ..s.camera
        };
        // A steeper bed: 0.2 m deep at the image's bottom edge, 3.5 m at its top.
        s.instances[0].transform = Mat4::from_translation(Vec3::new(0.0, -5.55, -22.0))
            * Mat4::from_rotation_x(-0.206);
        f(&mut s.water[0]);
        frame(&s)
    };
    let base = above(&|_| {});
    // Near end (image bottom, under 1 m deep) against the far end (top, over 3 m).
    let shallow = mean(&base, 400, 620, 880, 700);
    let deep = mean(&base, 400, 20, 880, 100);
    assert!(shallow > deep + 15.0, "shallow {shallow:.1} vs deep {deep:.1}");
    let no_foam = above(&|w| w.foam_width = 0.0);
    let changed = (0..720)
        .flat_map(|y| (0..1280).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let (p, q) = (px(&base, x, y), px(&no_foam, x, y));
            (luma(p) - luma(q)).abs() > 20.0
        })
        .count();
    assert!(
        changed > 200 && changed < 1280 * 720 / 10,
        "foam lights a band where water meets geometry ({changed} pixels)"
    );
    let matte = above(&|w| w.reflectivity = 0.0);
    let side = frame(&atmo::water_scene());
    let mut flat = atmo::water_scene();
    flat.water[0].heights = None;
    let flat = frame(&flat);
    let moved = diff(&side, &flat, 0, 230, 1280, 720);
    assert!(moved > 1.0, "the height field moves the surface ({moved:.2})");
    let mut side_matte = atmo::water_scene();
    side_matte.water[0].reflectivity = 0.0;
    let side_matte = frame(&side_matte);
    assert!(
        diff(&side, &side_matte, 0, 300, 1280, 720) > diff(&base, &matte, 0, 0, 1280, 720),
        "reflections matter more at a grazing view than from above"
    );
}

/// The topmost row in column `x` whose colour is the water lip's (bright
/// blue-white) between rows `y0..y1`.
fn lip_row(img: &RgbaImage, x: u32) -> Option<u32> {
    (0..720).find(|&y| {
        let p = px(img, x, y);
        p[2] > 180.0 && p[1] > 160.0 && p[0] > 120.0
    })
}

/// The 2D water line follows its surface: raising one end raises the lip
/// there and nowhere else; it draws between the sprites below and above
/// its layer.
#[test]
fn water_2d_follows_its_surface_in_the_sprite_order() {
    let body = |surface: Vec<f32>| {
        let mut s = RenderScene::new();
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.post.tonemap = og_render::Tonemap::None;
        s.water_2d.push(Water2d {
            origin: Vec2::new(-10.0, -1.0),
            width: 20.0,
            depth: 4.0,
            surface,
            lip_width: 0.12,
            layer: 2,
            ..Default::default()
        });
        s
    };
    let flat = frame(&body(vec![0.0; 11]));
    let mut raised = vec![0.0; 11];
    raised[2] = 2.0;
    let raised_img = frame(&body(raised));
    // x = -6 world is pixel 640 - 6 * 60 = 280; 1 unit is 60 pixels.
    let (a, b) = (lip_row(&flat, 280).unwrap(), lip_row(&raised_img, 280).unwrap());
    assert!(a - b > 100 && a - b < 140, "the lip rose by 2 units: {a} -> {b}");
    let (c, d) = (lip_row(&flat, 1000).unwrap(), lip_row(&raised_img, 1000).unwrap());
    assert!(c.abs_diff(d) <= 1, "the far end stays: {c} -> {d}");
    // A red sprite below the water's layer is tinted by it; one above is not.
    let mut s = body(vec![0.0; 11]);
    for (x, layer) in [(-3.0, 1), (3.0, 3)] {
        s.add_sprite(Sprite {
            position: Vec3::new(x, -3.0, 0.0),
            size: Vec2::splat(1.0),
            color: [1.0, 0.0, 0.0, 1.0],
            layer,
            ..Default::default()
        });
    }
    let img = frame(&s);
    let below = px(&img, 640 - 180, 360 + 180);
    let above = px(&img, 640 + 180, 360 + 180);
    assert!(above[0] > 250.0 && above[2] < 5.0, "above the water: pure red {above:?}");
    assert!(below[0] < 200.0 && below[2] > 40.0, "under the water: tinted {below:?}");
}

/// Two particles closer than their kernels' reach merge into one blob
/// (the midpoint is covered); two further apart stay two.
#[test]
fn metaballs_merge_within_reach() {
    let fluid = |gap: f32| {
        let mut s = RenderScene::new();
        s.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.post.tonemap = og_render::Tonemap::None;
        s.metaballs.push(og_render::Metaballs {
            particles: Arc::from(vec![Vec2::new(-gap / 2.0, 0.0), Vec2::new(gap / 2.0, 0.0)]),
            radius: 1.0,
            threshold: 0.5,
            color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            rim_width: 0.0,
            ..Default::default()
        });
        frame(&s)
    };
    // Each kernel is (1 - d²)²: at the midpoint of a gap g it adds
    // 2 (1 - g²/4)², over 0.5 for g < 1.53.
    let near = fluid(1.2);
    let far = fluid(1.8);
    let mid = |img: &RgbaImage| px(img, 640, 360)[2];
    assert!(mid(&near) > 200.0, "merged: {}", mid(&near));
    assert!(mid(&far) < 5.0, "apart: {}", mid(&far));
    // Each blob's own centre is covered either way.
    assert!(px(&far, 640 + 54, 360)[2] > 200.0);
}

/// The dissolve shader cuts holes by its amount: a white quad over a
/// black background keeps every pixel at amount 0 and loses a share at 0.6
/// (with no edge glow, a hole is background); unknown parameters are
/// refused. The grass shader moves the grass with the clock and nothing
/// else.
#[test]
fn shader_hooks_dissolve_and_sway() {
    let with_amount = |amount: f32| {
        let mut s = RenderScene::new();
        let shader = s.add_shader(atmo::dissolve_shader());
        let quad = s.add_mesh(og_render::shapes::cuboid(Vec3::new(4.0, 4.0, 0.1)));
        let m = s.add_standard_material(og_render::StandardMaterial {
            unlit: 1,
            alphaTest: 0.5,
            ..Default::default()
        });
        assert!(s.set_material_shader(m, Some(shader)));
        assert!(s.set_shader_param(m, "amount", &[amount]));
        assert!(s.set_shader_param(m, "edgeColor", &[0.0, 0.0, 0.0]));
        assert!(!s.set_shader_param(m, "nothing", &[1.0]), "unknown parameter");
        s.add_instance(quad, m, Mat4::IDENTITY);
        s.environment.background = og_render::Background::Color(Vec3::ZERO);
        s.camera = Camera {
            position: Vec3::new(0.0, 0.0, 6.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        frame(&s)
    };
    let lit = |img: &RgbaImage| {
        (260..460)
            .flat_map(|y| (540..740).map(move |x| (x, y)))
            .filter(|&(x, y)| luma(px(img, x, y)) > 30.0)
            .count()
    };
    let (whole, cut) = (lit(&with_amount(0.0)), lit(&with_amount(0.6)));
    assert_eq!(whole, 200 * 200, "amount 0 keeps the whole quad");
    assert!(
        cut < whole * 3 / 4 && cut > whole / 20,
        "amount 0.6 cuts holes: {whole} -> {cut}"
    );
    let early = frame(&atmo::hooks_scene(0));
    let late = frame(&atmo::hooks_scene(40));
    let grass = diff(&early, &late, 0, 500, 1280, 720);
    let objects = diff(&early, &late, 250, 200, 1000, 380);
    assert!(grass > 1.0, "the grass moved ({grass:.2})");
    assert!(objects < grass * 0.1, "the rest did not ({objects:.2})");
}
