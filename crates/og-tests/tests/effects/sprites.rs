//! 2D lighting, sprite glow, flash and decal frame tests.

use super::*;

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires GPU"]
fn decals_project_onto_surfaces_past_the_old_cap() {
    let base = fx::decals_scene();
    let desc = scenes::base_desc();
    let (with, stats) = frame(&desc, &base);
    let mut none = fx::decals_scene();
    none.effects.decals.clear();
    let (without, _) = frame(&desc, &none);
    save("decals", &with);
    assert_eq!(stats.decals, base.effects.decals.len() as u32);
    // The scorch at (-1.6, 0, 0.6) darkens the ground; the rune glows.
    let scorch = project(&base.frame.camera, 1280, 720, Vec3::new(-1.6, 0.0, 0.6));
    let rune = project(
        &base.frame.camera,
        1280,
        720,
        Vec3::new(0.2 + 0.8, 0.0, 1.6),
    );
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
    decals.extend(base.effects.decals.iter().copied());
    many.effects.decals = decals;
    let (late, s) = frame(&desc, &many);
    assert_eq!((s.decals, s.decals_dropped), (45, 0));
    let s2 = luma(px(&late, scorch.0, scorch.1));
    assert!(s2 < 0.5 * s0, "the 41st decal's scorch: {s2} vs {s0}");
    // Past MAX_CLUSTER_DECALS in one place the oldest there are dropped.
    let mut pile = fx::decals_scene();
    pile.effects.decals = vec![pile.effects.decals[0]; MAX_CLUSTER_DECALS + 5];
    let (_, s) = frame(&small(), &pile);
    assert_eq!(s.decals, MAX_CLUSTER_DECALS as u32 + 5);
    assert!(s.decals_dropped >= 5, "{}", s.decals_dropped);
}

/// Normal-mapped sprites (R11 batch 3): a sprite whose normal map tilts
/// towards +u is brighter lit from its right than from its left, the same
/// map on a mirrored sprite (its uvs flipped) turns with the art, and a
/// flat map draws exactly as no map does.
#[test]
#[ignore = "requires GPU"]
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
        scene.sprite.lighting_2d = Some(Lighting2d {
            ambient: Vec3::splat(0.05),
            ..Default::default()
        });
        scene.sprite.lights_2d.push(Light2d {
            position: Vec2::new(light_x, 0.0),
            radius: 8.0,
            falloff: 1.0,
            height: 1.0,
            cast_shadows: false,
            ..Default::default()
        });
        scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 8.0);
        scene.lighting.environment.background = Background::Color(Vec3::ZERO);
        scene.look.post.tonemap = Tonemap::None;
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
