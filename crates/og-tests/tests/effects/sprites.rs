//! 2D lighting, sprite glow, flash and decal frame tests.

use super::*;

// ---------------------------------------------------------------------------
// Decals.
// ---------------------------------------------------------------------------

/// Wet coverage keeps the receiver's colour detail and replaces, rather
/// than adds to, its own normal map. Partial coverage must still retain
/// some receiver detail; fully covered stone shades as flat wet stone.
#[test]
#[ignore = "requires GPU"]
fn wet_decals_multiply_colour_and_cover_receiver_normals() {
    let capture = |wet, opacity, mapped: bool, reference, albedo_view| {
        let mut scene = RenderScene::new();
        scene.lighting.environment.intensity = 0.0;
        scene.lighting.light.direction_to_light = Vec3::Y;
        scene.lighting.light.intensity = 1.0;
        scene.frame.camera = Camera {
            position: Vec3::new(0.0, 3.0, 0.01),
            target: Vec3::ZERO,
            exposure: og_render::Exposure::linear(1.0),
            ..Default::default()
        };
        if albedo_view {
            scene.frame.debug = og_render::DebugView::Albedo;
        }
        let normal = scene.add_texture(og_render::TextureData {
            width: 4,
            height: 4,
            rgba: [240, 128, 180, 255].repeat(16),
            color_space: og_render::ColorSpace::Linear,
            mips: og_render::Mips::None,
        });
        let material = scene.add_material(Material {
            base_color: if reference {
                [0.4, 0.125, 0.05, 1.0]
            } else {
                [0.8, 0.25, 0.1, 1.0]
            },
            metallic: 0.0,
            roughness: if reference { 0.04 } else { 0.8 },
            normal_texture: mapped.then_some(normal),
            ..Default::default()
        });
        let plane = scene.add_mesh(shapes::plane(3.0, 3.0));
        scene.add_instance(plane, material, Mat4::IDENTITY);
        if !reference {
            scene.effects.decals.push(Decal {
                wet,
                color: [0.5, 0.5, 0.5, opacity],
                roughness: 0.04,
                ..Decal::on_surface(Vec3::ZERO, Vec3::Y, Vec2::splat(4.0), 0.4, 0.0)
            });
        }
        frame(&small(), &scene).0
    };
    let full = capture(true, 1.0, true, false, false);
    let flat = capture(false, 1.0, false, true, false);
    let mix = capture(false, 1.0, true, false, false);
    let partial = capture(true, 0.5, true, false, false);
    let centre = |image: &RgbaImage| region(image, (150, 80, 170, 100));
    assert!(
        (centre(&full) - centre(&flat)).abs() < 1.0,
        "wet coverage did not flatten the receiver"
    );
    assert!(
        centre(&full) > centre(&mix) + 20.0,
        "control must show the receiver's tilted normal"
    );
    assert!(
        centre(&partial) < centre(&full) - 5.0,
        "partial coverage erased the whole normal"
    );
    let multiplied = capture(true, 1.0, true, false, true);
    let expected = capture(false, 1.0, false, true, true);
    assert_eq!(
        px_clamped(&multiplied, 160, 90),
        px_clamped(&expected, 160, 90),
        "wet albedo is not receiver × tint"
    );
    save("og-effects", "wet-decal-full", &full);
    save("og-effects", "wet-decal-mix", &mix);
    save("og-effects", "wet-decal-partial", &partial);
}

/// A rough floor covered with a white wet decal must reflect the scene
/// exactly as the same floor authored mirror-smooth. This catches a
/// beauty-only change that leaves SSR reading the receiver's old gloss.
#[test]
#[ignore = "requires GPU"]
fn wet_decals_carry_their_gloss_into_screen_space_reflections() {
    let capture = |decal| {
        let mut scene = og_tests::depth_render_scenes::ssr_scene(true);
        if decal {
            let floor = scene.objects.instances[0].material;
            scene.set_standard_material(
                floor,
                og_render::StandardMaterial {
                    color: Vec3::splat(0.04),
                    roughness: 0.8,
                    ..Default::default()
                },
            );
            scene.effects.decals.push(Decal {
                wet: true,
                roughness: 0.04,
                ..Decal::on_surface(Vec3::ZERO, Vec3::Y, Vec2::splat(20.0), 0.1, 0.0)
            });
        }
        run(
            &small(),
            &mut scene,
            og_tests::depth_render_scenes::SSR_FRAMES,
            |_, _| {},
        )
        .0
    };
    let reference = capture(false);
    let wet = capture(true);
    save("og-effects", "wet-decal-ssr-reference", &reference);
    save("og-effects", "wet-decal-ssr", &wet);
    let red = px_clamped(&reference, 120, 140);
    assert!(red[0] > red[1] + 10, "control must reflect the red sphere");
    assert!(
        px_clamped(&reference, 202, 140)[0] > 120,
        "control must reflect the bright panel"
    );
    assert_eq!(
        wet.pixels, reference.pixels,
        "wet floor's reflection differs from a mirror floor"
    );
}

#[test]
#[ignore = "requires GPU"]
fn decals_project_onto_surfaces_past_the_old_cap() {
    let base = fx::decals_scene();
    let desc = scenes::base_desc();
    let (with, stats) = frame(&desc, &base);
    let mut none = fx::decals_scene();
    none.effects.decals.clear();
    let (without, _) = frame(&desc, &none);
    save("og-effects", "decals", &with);
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
        luma(&px_clamped(&with, scorch.0, scorch.1)),
        luma(&px_clamped(&without, scorch.0, scorch.1)),
    );
    println!("scorch {s1} vs {s0}");
    assert!(s1 < 0.5 * s0);
    let changed = differing(&with, &without, 8.0);
    assert!(changed > 20_000, "decals changed {changed} pixels");
    let r = px_clamped(&with, rune.0, rune.1);
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
    let s2 = luma(&px_clamped(&late, scorch.0, scorch.1));
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
    let at = |img: &RgbaImage| luma(&px_clamped(img, centre.0, centre.1));
    let tilt = Some(Vec3::new(0.7, 0.0, 0.7).normalize());
    let (right, left) = (lit(2.0, tilt, false), lit(-2.0, tilt, false));
    save("og-effects", "light2d-normal", &right);
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
