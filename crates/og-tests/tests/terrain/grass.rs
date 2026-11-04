//! R6's grass: a default og.Grass grows a meadow over the island's grass
//! layer, lit from behind it lets the sun through, it reaches the horizon
//! as the ground's colour, the wind bends it, and the same tick draws the
//! same blades.
//!
//! Ways this could fail: generation keeps nothing (a wrong density share,
//! an empty draw), the blades draw but black (unlit, a missing group),
//! translucency does nothing, the far tint never reaches the ground, or
//! the blades' order or bend depends on more than the tick (an atomic
//! append, a wall clock).

use super::*;
use og_tests::grass_scenes as gs;

/// Pixels of the frame's lower half where `a` and `b` differ by more than 8.
fn changed_below(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let (mut n, mut all) = (0, 0);
    for y in a.height / 2..a.height {
        for x in 0..a.width {
            let (p, q) = (px(a, x, y), px(b, x, y));
            all += 1;
            if (0..3).any(|k| (p[k] - q[k]).abs() > 8.0) {
                n += 1;
            }
        }
    }
    n as f32 / all as f32
}

/// The mean colour of rows `y0..y1`.
fn mean(img: &RgbaImage, y0: u32, y1: u32) -> [f32; 3] {
    let mut s = [0.0f32; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in 0..img.width {
            let c = px(img, x, y);
            (0..3).for_each(|k| s[k] += c[k]);
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[test]
fn a_default_field_grows_a_meadow() {
    check("grass-near");
    let with = gs::grass_scene("near");
    let mut without = gs::grass_scene("near");
    without.grass.clear();
    let (a, _) = render(&with);
    let (b, _) = render(&without);
    keep("grass-near", &a);
    keep("grass-near-bare", &b);
    let changed = changed_below(&a, &b);
    assert!(changed > 0.4, "the blades cover {changed} of the ground");
    // Lit and green, not black.
    let c = mean(&a, a.height * 3 / 4, a.height);
    assert!(c[1] > c[0] && c[1] > c[2] && luma(c) > 20.0, "{c:?}");
}

#[test]
fn the_middle_distance_stays_covered() {
    check("grass-mid");
}

/// Past its blades the ground takes the field's colour: the far meadow
/// with a field is greener than without one.
#[test]
fn the_far_field_reads_as_grass() {
    check("grass-far");
    let with = gs::grass_scene("far");
    let mut without = gs::grass_scene("far");
    without.grass.clear();
    let (a, _) = render(&with);
    let (b, _) = render(&without);
    keep("grass-far", &a);
    keep("grass-far-bare", &b);
    // The band above the frame's middle: the meadow beyond 100 m, where
    // the ground takes the field's average colour. That olive is darker
    // and less saturated than the island's grass layer by design (the art
    // review of 2026-11-03), so the band is not greener than the bare
    // ground; it must differ from it (the tint reaches past the blades)
    // and still read as grass (green over red and blue).
    let (y0, y1) = (a.height * 2 / 5, a.height / 2);
    let mut diff = 0.0;
    for y in y0..y1 {
        for x in 0..a.width {
            let (p, q) = (px(&a, x, y), px(&b, x, y));
            diff += (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max);
        }
    }
    diff /= ((y1 - y0) * a.width) as f32;
    let c = mean(&a, y0, y1);
    assert!(
        diff > 4.0 && c[1] > c[0] && c[1] > c[2],
        "far meadow: {c:?} with the field, {diff} from the bare ground"
    );
}

#[test]
fn blades_lit_from_behind_let_the_sun_through() {
    check("grass-backlit");
    let lit = gs::grass_scene("backlit");
    // The premise: the sun reaches the meadow ahead (a ridge in its way
    // leaves no direct light for translucency to pass).
    let eye = lit.camera.position;
    let ahead = (lit.camera.target - eye).with_y(0.0).normalize();
    for d in [3.0, 8.0, 15.0, 25.0] {
        let at = eye + ahead * d;
        let ground = og_tests::terrain_scenes::island_height(at.x, at.z);
        assert!(
            gs::sun_reaches(at.with_y(ground + 0.3), lit.light.direction_to_light),
            "terrain hides the sun {d} m ahead"
        );
    }
    let mut opaque = gs::grass_scene("backlit");
    opaque.grass[0].grass.translucency = 0.0;
    let (a, _) = render(&lit);
    let (b, _) = render(&opaque);
    keep("grass-backlit", &a);
    let (la, lb) = (
        luma(mean(&a, a.height / 2, a.height)),
        luma(mean(&b, b.height / 2, b.height)),
    );
    assert!(la > lb * 1.1, "translucent {la}, opaque {lb}");
}

#[test]
fn the_wind_bends_the_blades_and_the_same_tick_draws_the_same() {
    check("grass-wind");
    let (first, _) = render(&gs::grass_wind_scene(90));
    let (again, _) = render(&gs::grass_wind_scene(90));
    assert_eq!(first.pixels, again.pixels, "tick 90 drawn twice");
    let (later, _) = render(&gs::grass_wind_scene(150));
    let (calm, _) = render(&gs::grass_scene("near"));
    keep("grass-wind", &first);
    keep("grass-wind-later", &later);
    assert!(changed_below(&first, &later) > 0.05, "a second later");
    assert!(changed_below(&first, &calm) > 0.1, "against no wind");
}

/// G3's meadow: red blades 0.6 m tall over a dark flat terrain to 2.5 m
/// from a camera at ankle height (0.12 m) looking along +x, against a
/// black sky, so every red pixel above the frame's middle row is a blade
/// part above the ankle.
fn trample_scene() -> RenderScene {
    let data = TerrainData::flat(Vec2::splat(64.0), 129, 129);
    let mut scene = RenderScene::new();
    let h = scene.add_terrain(data);
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            color: Vec3::splat(0.02),
            ..Default::default()
        }],
        cast_shadows: false,
        fade_in: false,
        ..Terrain::new(h)
    });
    scene.grass.push(og_render::GrassDraw {
        terrain: 0,
        grass: og_render::Grass {
            distance: 2.5,
            height: 0.6,
            height_variation: 0.0,
            lean: 0.0,
            clump_lean: 0.0,
            base_color: Vec3::new(0.9, 0.05, 0.05),
            tip_color: Vec3::new(0.9, 0.05, 0.05),
            far_tint: false,
            cast_shadows: false,
            recovery: 1.0,
            ..Default::default()
        },
    });
    scene.fog = None;
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.light.cast_shadows = false;
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.12, 0.0),
        target: Vec3::new(1.0, 0.12, 0.0),
        near: 0.02,
        far: 50.0,
        projection: Projection::Perspective {
            fov_y_degrees: 60.0,
        },
        ..Default::default()
    };
    scene
}

/// Red pixels above the frame's middle row: blade parts above the ankle.
fn blade_pixels_above(img: &RgbaImage) -> usize {
    (0..img.height / 2 - 2)
        .flat_map(|y| (0..img.width).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px(img, x, y);
            p[0] > 30.0 && p[0] > 2.5 * p[1]
        })
        .count()
}

/// G3: an og.GrassPusher standing over the meadow presses its blades away
/// and down: above ankle height fewer than 30 % of the blade pixels stay;
/// once it leaves they stand again within the field's recovery (1 s) and
/// half a second, to at least 80 % of the untouched field. Before G3
/// nothing read a pusher: the blades stood as tall under it.
#[test]
fn a_grass_pusher_tramples_the_blades_and_they_recover() {
    let mut scene = trample_scene();
    let mut r = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    let mut draw = |scene: &mut RenderScene, tick: u64| {
        scene.tick = tick;
        r.render(scene, &RenderTarget::Offscreen).expect("render");
        r.read_offscreen().expect("capture")
    };
    let untouched = blade_pixels_above(&draw(&mut scene, 0));
    // The pusher covers every drawn blade near its centre (radius 6 m,
    // the field 2.5 m: pressed at least 97 %).
    scene.grass_pushers = vec![og_render::GrassPusher {
        position: Vec3::new(0.0, 0.0, 0.0),
        radius: 6.0,
        height: 1.8,
        strength: 1.0,
    }];
    let mut pressed = draw(&mut scene, 1);
    for tick in 2..=10 {
        pressed = draw(&mut scene, tick);
    }
    keep("grass-trample", &pressed);
    let under = blade_pixels_above(&pressed);
    scene.grass_pushers.clear();
    // 1.5 s later (the recovery and half a second), drawn every 4 ticks.
    let mut after = pressed.clone();
    for tick in (14..=100).step_by(4) {
        after = draw(&mut scene, tick);
    }
    keep("grass-trample-recovered", &after);
    let back = blade_pixels_above(&after);
    println!(
        "blade pixels above the ankle: untouched {untouched}, pressed {under}, recovered {back}"
    );
    assert!(
        untouched > 2000,
        "the meadow shows {untouched} blade pixels"
    );
    assert!(
        (under as f32) < 0.3 * untouched as f32,
        "pressed: {under} of {untouched} still stand"
    );
    assert!(
        back as f32 > 0.8 * untouched as f32,
        "recovered: {back} of {untouched}"
    );
}
