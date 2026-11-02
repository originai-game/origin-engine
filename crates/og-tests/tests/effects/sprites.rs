//! 2D lighting, sprite glow, flash and decal frame tests.

use super::*;

// ---------------------------------------------------------------------------
// 2D lights, glow, flash.
// ---------------------------------------------------------------------------

fn lit_room(shadows: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.add_sprite(Sprite {
        size: Vec2::new(20.0, 12.0),
        color: [0.8, 0.8, 0.8, 1.0],
        ..Default::default()
    });
    let wall = Sprite {
        position: Vec3::new(0.0, 0.0, 0.0),
        size: Vec2::new(0.6, 2.0),
        color: [0.3, 0.2, 0.1, 1.0],
        layer: 1,
        ..Default::default()
    };
    scene.occluders_2d.push(Occluder2d::from_sprite(&wall, 0.0));
    scene.add_sprite(wall);
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.05),
        ..Default::default()
    });
    scene.lights_2d.push(Light2d {
        position: Vec2::new(-3.0, 0.0),
        radius: 9.0,
        intensity: 1.0,
        source_radius: 0.4,
        cast_shadows: shadows,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    scene
}

/// A wall between a 2D light and the floor shades the floor behind it with
/// a soft penumbra; the light side is unaffected; ambient fills the rest.
#[test]
fn two_d_lights_cast_soft_shadows_from_occluders() {
    let (lit, stats) = frame(&small(), &lit_room(true));
    let (open, _) = frame(&small(), &lit_room(false));
    save("light2d", &lit);
    assert_eq!((stats.lights_2d, stats.occluders_2d), (1, 1));
    let cam = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    let at = |p: Vec2| project(&cam, 320, 180, p.extend(0.0));
    let behind = at(Vec2::new(1.5, 0.0));
    let front = at(Vec2::new(-1.5, 0.0));
    let (b, bo) = (
        luma(px(&lit, behind.0, behind.1)),
        luma(px(&open, behind.0, behind.1)),
    );
    let (f, fo) = (
        luma(px(&lit, front.0, front.1)),
        luma(px(&open, front.0, front.1)),
    );
    println!("behind the wall {b} (open {bo}); in front {f} (open {fo})");
    // The umbra receives only the ambient light (0.04 linear: 56 in sRGB).
    assert!(b < 60.0 && bo > 120.0, "the umbra is not dark");
    assert!((f - fo).abs() < 2.0, "the lit side changed");
    // Soft: along a vertical line behind the wall, intermediate values.
    let column: Vec<f32> = (0..90).map(|y| luma(px(&lit, behind.0 + 20, y))).collect();
    let lo = column.iter().cloned().fold(f32::MAX, f32::min);
    let hi = column.iter().cloned().fold(0.0, f32::max);
    let mid = column
        .iter()
        .filter(|&&v| v > lo + 0.2 * (hi - lo) && v < hi - 0.2 * (hi - lo))
        .count();
    println!("penumbra pixels in the column: {mid}");
    assert!(mid >= 3, "the shadow edge is hard");
}

/// Layers above the lit range are drawn as authored.
#[test]
fn two_d_lighting_leaves_ui_layers_alone() {
    let mut scene = lit_room(true);
    scene.lights_2d.clear();
    scene.add_sprite(Sprite {
        position: Vec3::new(2.0, 2.0, 0.0),
        color: [0.5, 0.5, 0.5, 1.0],
        layer: 200,
        ..Default::default()
    });
    let (img, _) = frame(&small(), &scene);
    let cam = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    let (x, y) = project(&cam, 320, 180, Vec3::new(2.0, 2.0, 0.0));
    let ui = px(&img, x, y);
    let floor = px(&img, 20, 170);
    println!("ui {ui:?} floor {floor:?}");
    // 0.5 linear is 188 in sRGB: drawn as authored.
    assert!(
        (ui[0] as i32 - 188).abs() <= 2,
        "the UI sprite was lit: {ui:?}"
    );
    // The floor: 0.8 x ambient 0.05 = 0.04 linear, 56 in sRGB.
    assert!(
        (floor[0] as i32 - 56).abs() <= 3,
        "the floor should be ambient-dark: {floor:?}"
    );
}

/// A glowing sprite under an opaque one still glows around it; a flashed
/// sprite turns towards the flash colour.
#[test]
fn glow_is_order_independent_and_flash_tints() {
    let build = |glow: f32, flash: f32| {
        let mut scene = RenderScene::new();
        scene.add_sprite(Sprite {
            size: Vec2::splat(0.6),
            color: [1.0, 0.8, 0.2, 1.0],
            ..Default::default()
        });
        scene.add_sprite(Sprite {
            size: Vec2::splat(0.8),
            color: [0.2, 0.2, 0.25, 1.0],
            layer: 1,
            ..Default::default()
        });
        scene.add_sprite(Sprite {
            position: Vec3::new(2.0, 0.0, 0.0),
            color: [0.1, 0.2, 0.8, 1.0],
            ..Default::default()
        });
        scene.sprite_fx = vec![
            SpriteFx {
                sprite: 0,
                glow,
                ..Default::default()
            },
            SpriteFx {
                sprite: 2,
                flash: [1.0, 1.0, 1.0, flash],
                ..Default::default()
            },
        ];
        scene.post.glow = Some(Glow::default());
        scene.camera = Camera::orthographic_2d(Vec2::ZERO, 6.0);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        scene
    };
    let (glowing, stats) = frame(&small(), &build(3.0, 0.0));
    let (plain, plain_stats) = frame(&small(), &build(0.0, 0.0));
    let (flashed, _) = frame(&small(), &build(0.0, 1.0));
    assert!(stats.graph_passes > plain_stats.graph_passes, "glow passes");
    // Around the covering sprite (just outside it): glow.
    let around = region(&glowing, (160 - 28, 90 - 3, 160 - 22, 90 + 3));
    let around_plain = region(&plain, (160 - 28, 90 - 3, 160 - 22, 90 + 3));
    println!("around the cover: glow {around}, none {around_plain}");
    assert!(around > around_plain + 10.0);
    let cam = Camera::orthographic_2d(Vec2::ZERO, 6.0);
    let (x, y) = project(&cam, 320, 180, Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(px(&flashed, x, y), [255, 255, 255]);
    assert!(px(&plain, x, y)[2] > px(&plain, x, y)[0] + 50);
}

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

#[test]
fn decals_project_onto_surfaces_past_the_old_cap() {
    let base = fx::decals_scene();
    let desc = scenes::base_desc();
    let (with, stats) = frame(&desc, &base);
    let mut none = fx::decals_scene();
    none.decals.clear();
    let (without, _) = frame(&desc, &none);
    save("decals", &with);
    assert_eq!(stats.decals, base.decals.len() as u32);
    // The scorch at (-1.6, 0, 0.6) darkens the ground; the rune glows.
    let scorch = project(&base.camera, 1280, 720, Vec3::new(-1.6, 0.0, 0.6));
    let rune = project(&base.camera, 1280, 720, Vec3::new(0.2 + 0.8, 0.0, 1.6));
    let (s1, s0) = (
        luma(px(&with, scorch.0, scorch.1)),
        luma(px(&without, scorch.0, scorch.1)),
    );
    println!("scorch {s1} vs {s0}");
    assert!(s1 < 0.5 * s0);
    let changed = differing(&with, &without, 8.0);
    assert!(changed > 20_000, "decals changed {changed} pixels");
    let r = px(&with, rune.0, rune.1);
    println!("rune ring pixel {r:?}");
    // Past the 32 a frame once drew, every decal still draws: the base
    // scene's five after 40 others off to one side cover what they did.
    let mut many = fx::decals_scene();
    let mut decals: Vec<Decal> = (0..40)
        .map(|i| Decal {
            color: [0.1, 0.1, 0.9, 1.0],
            ..Decal::on_surface(
                Vec3::new(-6.0 + (i % 8) as f32 * 0.3, 0.0, 2.0 + (i / 8) as f32 * 0.3),
                Vec3::Y,
                Vec2::splat(0.25),
                0.3,
                0.0,
            )
        })
        .collect();
    decals.extend(base.decals.iter().copied());
    many.decals = decals;
    let (late, s) = frame(&desc, &many);
    assert_eq!((s.decals, s.decals_dropped), (45, 0));
    let s2 = luma(px(&late, scorch.0, scorch.1));
    assert!(s2 < 0.5 * s0, "the 41st decal's scorch: {s2} vs {s0}");
    // Past MAX_CLUSTER_DECALS in one place the oldest there are dropped.
    let mut pile = fx::decals_scene();
    pile.decals = vec![pile.decals[0]; MAX_CLUSTER_DECALS + 5];
    let (_, s) = frame(&small(), &pile);
    assert_eq!(s.decals, MAX_CLUSTER_DECALS as u32 + 5);
    assert!(s.decals_dropped >= 5, "{}", s.decals_dropped);
}

/// R11 batch 2's far edge: a large light (radius 30) and three small
/// occluders (0.3 wide) 24 units out, near its rim, where each angle of the
/// polar shadow map covers the widest strip (0.15 per texel at the default
/// 1024 angles). Each still casts a shadow behind it, and the frame at 4096
/// angles shows what the default loses there (saved for review, both).
#[test]
fn small_occluders_at_the_far_edge_of_a_large_light_still_cast_shadows() {
    let scene = |angles: u32| {
        let mut scene = RenderScene::new();
        scene.add_sprite(Sprite {
            position: Vec3::new(25.0, 0.0, 0.0),
            size: Vec2::new(20.0, 12.0),
            color: [0.8, 0.8, 0.8, 1.0],
            ..Default::default()
        });
        for y in [-2.0, 0.0, 2.0] {
            scene.occluders_2d.push(Occluder2d {
                center: Vec2::new(24.0, y),
                half_size: Vec2::splat(0.15),
                ..Default::default()
            });
        }
        scene.lighting_2d = Some(Lighting2d {
            ambient: Vec3::splat(0.02),
            shadow_angles: angles,
            ..Default::default()
        });
        scene.lights_2d.push(Light2d {
            radius: 30.0,
            falloff: 0.5,
            source_radius: 0.25,
            ..Default::default()
        });
        scene.camera = Camera::orthographic_2d(Vec2::new(25.0, 0.0), 8.0);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        scene
    };
    let (img, stats) = frame(&small(), &scene(1024));
    let (fine, _) = frame(&small(), &scene(4096));
    save("light2d-far-edge", &img);
    save("light2d-far-edge-4096", &fine);
    assert_eq!((stats.lights_2d, stats.occluders_2d), (1, 3));
    let cam = Camera::orthographic_2d(Vec2::new(25.0, 0.0), 8.0);
    for (name, img) in [("1024", &img), ("4096", &fine)] {
        for y in [-2.0f32, 0.0, 2.0] {
            // Behind the box on its ray from the light, and half a unit aside.
            let behind = project(&cam, 320, 180, Vec3::new(27.0, y * 27.0 / 24.0, 0.0));
            let aside = project(&cam, 320, 180, Vec3::new(27.0, y * 27.0 / 24.0 + 0.5, 0.0));
            let (b, a) = (
                luma(px(img, behind.0, behind.1)),
                luma(px(img, aside.0, aside.1)),
            );
            println!("{name} angles, box at y {y}: behind {b}, aside {a}");
            assert!(b < 0.6 * a, "{name}: no shadow behind the box at y {y}");
        }
    }
}

/// A concave polygon occluder (a U open towards the light): the notch is
/// lit, the floor behind the U's base is shadowed, and a pixel inside one
/// of its arms is not shadowed by the U itself.
#[test]
fn a_concave_polygon_occluder_shadows_behind_its_body_only() {
    let mut scene = RenderScene::new();
    scene.add_sprite(Sprite {
        size: Vec2::new(20.0, 12.0),
        color: [0.8, 0.8, 0.8, 1.0],
        ..Default::default()
    });
    let u = [
        (-1.0, 0.0),
        (1.0, 0.0),
        (1.0, 2.0),
        (0.5, 2.0),
        (0.5, 0.5),
        (-0.5, 0.5),
        (-0.5, 2.0),
        (-1.0, 2.0),
    ]
    .map(|(x, y)| Vec2::new(x, y));
    scene
        .occluders_2d
        .push(Occluder2d::polygon(Vec2::ZERO, 0.0, &u));
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.02),
        ..Default::default()
    });
    scene.lights_2d.push(Light2d {
        position: Vec2::new(0.0, 4.0),
        radius: 9.0,
        falloff: 1.0,
        source_radius: 0.1,
        ..Default::default()
    });
    scene.camera = Camera::orthographic_2d(Vec2::new(0.0, 1.0), 8.0);
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.post.tonemap = Tonemap::None;
    let (img, _) = frame(&small(), &scene);
    save("light2d-polygon", &img);
    let cam = Camera::orthographic_2d(Vec2::new(0.0, 1.0), 8.0);
    let at = |x: f32, y: f32| {
        luma(px(
            &img,
            project(&cam, 320, 180, Vec3::new(x, y, 0.0)).0,
            project(&cam, 320, 180, Vec3::new(x, y, 0.0)).1,
        ))
    };
    let (notch, below, arm, open) = (at(0.0, 1.2), at(0.0, -1.5), at(-0.8, 1.0), at(-2.5, 1.0));
    println!("notch {notch}, below the base {below}, inside an arm {arm}, open floor {open}");
    assert!(
        notch > 0.8 * open,
        "the notch is shadowed: {notch} vs {open}"
    );
    assert!(
        below < 0.3 * notch,
        "nothing shadows the floor behind the base: {below}"
    );
    assert!(arm > 0.5 * open, "the U shadows its own arm: {arm}");
}

/// Normal-mapped sprites (R11 batch 3): a sprite whose normal map tilts
/// towards +u is brighter lit from its right than from its left, the same
/// map on a mirrored sprite (its uvs flipped) turns with the art, and a
/// flat map draws exactly as no map does.
#[test]
fn a_normal_mapped_sprite_is_lit_by_where_its_normal_faces() {
    let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
    let map = |n: Vec3| og_render::TextureData {
        width: 4,
        height: 4,
        rgba: [enc(n.x), enc(n.y), enc(n.z), 255].repeat(16),
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::None,
    };
    // Lit from x = light_x at height 1; `normal` None draws without a map.
    let lit = |light_x: f32, normal: Option<Vec3>, mirrored: bool| {
        let mut scene = RenderScene::new();
        let white = scene.add_texture(og_render::TextureData {
            width: 4,
            height: 4,
            rgba: [255u8; 4].repeat(16),
            color_space: og_render::ColorSpace::Srgb,
            mips: og_render::Mips::None,
        });
        if let Some(n) = normal {
            let n = scene.add_texture(map(n));
            assert!(scene.set_texture_normal(white, Some(n)));
        }
        scene.add_sprite(Sprite {
            texture: Some(white),
            size: Vec2::new(4.0, 4.0),
            color: [0.8, 0.8, 0.8, 1.0],
            uv_rect: if mirrored {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 1.0, 1.0]
            },
            ..Default::default()
        });
        scene.lighting_2d = Some(Lighting2d {
            ambient: Vec3::splat(0.05),
            ..Default::default()
        });
        scene.lights_2d.push(Light2d {
            position: Vec2::new(light_x, 0.0),
            radius: 8.0,
            falloff: 1.0,
            height: 1.0,
            cast_shadows: false,
            ..Default::default()
        });
        scene.camera = Camera::orthographic_2d(Vec2::ZERO, 8.0);
        scene.environment.background = Background::Color(Vec3::ZERO);
        scene.post.tonemap = Tonemap::None;
        frame(&small(), &scene).0
    };
    let cam = Camera::orthographic_2d(Vec2::ZERO, 8.0);
    let centre = project(&cam, 320, 180, Vec3::ZERO);
    let at = |img: &RgbaImage| luma(px(img, centre.0, centre.1));
    let tilt = Some(Vec3::new(0.7, 0.0, 0.7).normalize());
    let (right, left) = (lit(2.0, tilt, false), lit(-2.0, tilt, false));
    save("light2d-normal", &right);
    let (r, l) = (at(&right), at(&left));
    let (mr, ml) = (at(&lit(2.0, tilt, true)), at(&lit(-2.0, tilt, true)));
    let flat = lit(2.0, Some(Vec3::Z), false);
    let none = lit(2.0, None, false);
    println!("tilted +u: from the right {r}, the left {l}; mirrored: {mr}, {ml}");
    assert!(
        r > l + 20.0,
        "a normal towards +x is not lit more from +x: {r} vs {l}"
    );
    assert!(
        ml > mr + 20.0,
        "the mirrored sprite's normal did not turn: {mr} vs {ml}"
    );
    let worst = flat
        .pixels
        .iter()
        .zip(&none.pixels)
        .map(|(a, b)| (*a as i32 - *b as i32).abs())
        .max()
        .unwrap_or(0);
    println!("flat map against no map: worst channel difference {worst}");
    // An 8-bit 128 is 1/255 off flat.
    assert!(worst <= 2, "a flat normal map changes the light: {worst}");
}
