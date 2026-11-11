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

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
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
    scene.atmosphere.fog = None;
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
#[ignore = "requires GPU"]
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
    scene.atmosphere.grass_pushers = vec![og_render::GrassPusher {
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
    scene.atmosphere.grass_pushers.clear();
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

/// G5: under a sun 10 degrees up, a dense meadow (one colour root to tip,
/// no sky light) is more than twice as bright at its tips as at its roots:
/// the canopy between a blade's foot and the sun, and the blades' own
/// cascade shadows (now cast by default), darken it from inside. Seen from
/// the side through an orthographic camera, so each screen row is one
/// height above the ground. Before G5 a blade was lit alike root to tip
/// (no canopy, no shadows by default): a ratio near 1.
#[test]
#[ignore = "requires GPU"]
fn a_low_sun_darkens_the_meadow_towards_its_roots() {
    let mut scene = trample_scene();
    let g = &mut scene.grass[0].grass;
    g.distance = 30.0;
    g.height = 0.5;
    g.height_variation = 0.0;
    g.density = 400.0;
    g.base_color = Vec3::new(0.3, 0.5, 0.15);
    g.tip_color = g.base_color;
    g.translucency = 0.0;
    g.cast_shadows = og_render::Grass::default().cast_shadows;
    scene.light.cast_shadows = true;
    let up = 10f32.to_radians();
    scene.light.direction_to_light = Vec3::new(0.0, up.sin(), up.cos());
    scene.camera = Camera {
        position: Vec3::new(-20.0, 0.25, 0.0),
        target: Vec3::new(0.0, 0.25, 0.0),
        near: 0.1,
        far: 60.0,
        projection: Projection::Orthographic { height: 0.8 },
        ..Default::default()
    };
    let (img, _) = render(&scene);
    keep("grass-low-sun", &img);
    // Rows 0.8 m tall over the frame's height, centred 0.25 m up: the
    // band 0.02..0.1 m over the ground and the band 0.38..0.46 m.
    let row = |y: f32| ((0.25 + 0.4 - y) / 0.8 * img.height as f32) as u32;
    let band = |lo: f32, hi: f32| {
        let (mut sum, mut n) = (0.0, 0);
        for y in row(hi)..row(lo) {
            for x in 0..img.width {
                let c = px(&img, x, y);
                if c[1] > 2.0 {
                    sum += luma(c);
                    n += 1;
                }
            }
        }
        (sum / n.max(1) as f32, n)
    };
    let (roots, nr) = band(0.02, 0.1);
    let (tips, nt) = band(0.38, 0.46);
    println!("luma at the roots {roots:.2} ({nr} px), at the tips {tips:.2} ({nt} px)");
    assert!(nr > 1000 && nt > 1000, "the field covers both bands");
    assert!(
        tips > 2.0 * roots,
        "tips {tips:.2} against roots {roots:.2}"
    );
}
