//! A secondary view is lit by the lights its own frustum reaches, also
//! those the main camera never sees (a hangar turntable, a portrait).

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, LightDesc, LightKind, Material, RenderDesc, RenderScene, View, ViewTarget,
    render_to_rgba, shapes,
};

#[test]
#[ignore = "requires GPU"]
fn a_view_is_lit_by_a_light_only_it_sees() {
    let far = Vec3::new(0.0, 400.0, 0.0);
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.0, 10.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.environment.intensity = 0.0;
    scene.lighting.light.intensity = 0.0;
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(2.0)));
    let white = scene.add_material(Material::default());
    scene.add_instance(cube, white, Mat4::from_translation(far));
    scene.lighting.lights.push(LightDesc {
        kind: LightKind::Point,
        position: far + Vec3::new(0.0, 1.5, 3.0),
        intensity: 60.0,
        range: 10.0,
        ..Default::default()
    });
    // The right half of the frame: the view's eye in front of the cube.
    scene.frame.views.push(View {
        camera: Camera {
            position: far + Vec3::new(0.0, 0.0, 6.0),
            target: far,
            ..Default::default()
        },
        target: ViewTarget::Viewport([0.5, 0.0, 0.5, 1.0]),
        ..Default::default()
    });
    let (w, h) = (128u32, 64u32);
    let image = render_to_rgba(
        &RenderDesc {
            width: w,
            height: h,
            ..Default::default()
        },
        &scene,
    )
    .expect("render");
    let at = |x: u32, y: u32| {
        let i = ((y * w + x) * 4) as usize;
        image.pixels[i..i + 3]
            .iter()
            .map(|&c| u32::from(c))
            .sum::<u32>()
    };
    // The cube's face at the view's centre is lit; the main camera's half
    // (nothing in view) stays black.
    assert!(
        at(3 * w / 4, h / 2) > 60,
        "the view's cube is lit: {}",
        at(3 * w / 4, h / 2)
    );
    assert_eq!(at(w / 4, h / 2), 0, "the main half is empty");
}
