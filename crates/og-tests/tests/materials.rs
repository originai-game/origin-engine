//! The M6 material features (phase 2, owed row 31): the `materials`
//! reference board, and from frames: normals under a non-uniform scale,
//! normal maps in the vertex tangent frame (generated when a mesh has none,
//! mirrored uvs included), and clearcoat.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, RenderScene, RenderTarget,
    Renderer, RgbaImage, StandardMaterial, shapes,
};
use og_tests::{gfx, scenes};
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-gfx")
}

fn frame(scene: &RenderScene) -> RgbaImage {
    let mut renderer = Renderer::new_blocking(&scenes::base_desc()).expect("renderer");
    renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    renderer.read_offscreen().expect("capture")
}

fn luma(img: &RgbaImage, x: u32, y: u32) -> f32 {
    let p = &img.pixels[((y * img.width + x) * 4) as usize..][..3];
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// One object on black, lit from the upper left, seen from +z.
fn studio() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.6, 0.7, 0.6),
        intensity: 3.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment = Environment {
        intensity: 0.15,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.0, 4.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// Thin-leaf transmission is tinted direct light, including on the back
/// side, and an intervening caster extinguishes it. Test both lit pipelines.
#[test]
fn leaf_backlighting_is_tinted_and_shadowed() {
    for clearcoat in [0.0, 0.4] {
        let mut scene = studio();
        scene.environment.intensity = 0.0;
        scene.light.direction_to_light = Vec3::new(-0.2, 0.3, -1.0).normalize();
        scene.light.cast_shadows = true;
        scene.light.max_shadow_distance = 12.0;
        let card = scene.add_mesh(shapes::plane_xy(2.0, 2.0));
        let mut leaf = StandardMaterial {
            color: Vec3::new(0.04, 0.3, 0.025),
            roughness: 0.9,
            side: 2,
            clearcoat,
            ..Default::default()
        };
        let material = scene.add_standard_material(leaf);
        scene.add_instance(card, material, Mat4::IDENTITY);
        let opaque = frame(&scene);
        leaf.foliage.x = 0.65;
        scene.set_standard_material(material, leaf);
        let lit = frame(&scene);
        let x = lit.width / 2;
        let y = lit.height / 2;
        assert!(
            luma(&lit, x, y) > luma(&opaque, x, y) + 20.0,
            "backlight never reached the leaf (coat {clearcoat})"
        );
        let p = &lit.pixels[((y * lit.width + x) * 4) as usize..][..3];
        assert!(
            p[1] > p[0] && p[1] > p[2],
            "transmission lost the leaf tint: {p:?}"
        );
        let blocker = scene.add_mesh(shapes::cuboid(Vec3::new(3.0, 3.0, 0.2)));
        let black = scene.add_standard_material(StandardMaterial {
            color: Vec3::ZERO,
            ..Default::default()
        });
        scene.add_instance(
            blocker,
            black,
            Mat4::from_translation(scene.light.direction_to_light),
        );
        let shadowed = frame(&scene);
        assert!(
            luma(&shadowed, x, y) < luma(&lit, x, y) * 0.4,
            "shadow failed to attenuate transmission (coat {clearcoat})"
        );
        gfx::save_png(
            &out_dir().join(format!("foliage-backlit-{clearcoat}.png")),
            &lit,
        )
        .unwrap();
        gfx::save_png(
            &out_dir().join(format!("foliage-shadowed-{clearcoat}.png")),
            &shadowed,
        )
        .unwrap();
    }
}

/// A glossy coat over a rough red base adds a sharp highlight the base
/// alone does not have, sharper the smoother the coat; no coat is the
/// plain material.
#[test]
fn clearcoat_adds_a_glossy_layer() {
    let ball = |coat: f32, coat_roughness: f32| {
        let mut scene = studio();
        let sphere = scene.add_mesh(shapes::sphere(64, 48));
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::new(0.7, 0.06, 0.05),
            roughness: 0.8,
            clearcoat: coat,
            clearcoatRoughness: coat_roughness,
            ..Default::default()
        });
        scene.add_instance(sphere, m, Mat4::IDENTITY);
        frame(&scene)
    };
    let peak = |img: &RgbaImage| {
        let mut best = 0.0f32;
        for y in 160..560 {
            for x in 440..840 {
                best = best.max(luma(img, x, y));
            }
        }
        best
    };
    let (bare, glossy, rough) = (ball(0.0, 0.0), ball(1.0, 0.03), ball(1.0, 0.5));
    let (pb, pg, pr) = (peak(&bare), peak(&glossy), peak(&rough));
    println!("highlight: bare {pb:.0}, glossy coat {pg:.0}, rough coat {pr:.0}");
    assert!(pg > pb + 60.0, "the glossy coat adds a highlight");
    assert!(pg > pr + 20.0, "a smoother coat is sharper");
    let mut plain = studio();
    let sphere = plain.add_mesh(shapes::sphere(64, 48));
    let m = plain.add_standard_material(StandardMaterial {
        color: Vec3::new(0.7, 0.06, 0.05),
        roughness: 0.8,
        ..Default::default()
    });
    plain.add_instance(sphere, m, Mat4::IDENTITY);
    assert_eq!(frame(&plain).pixels, bare.pixels);
}
