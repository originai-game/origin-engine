//! The 2D profile's renderer (`--features profile2d`): the 3D world
//! subsystems are not drawn, so a frame with planar water and volumetric
//! fog in its scene is the frame without them, and nothing of theirs is
//! compiled (`cargo test -p og-render --features profile2d --test
//! profile2d`). Without the feature the same scene draws its water.
#![cfg(feature = "profile2d")]

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, Material, RenderDesc, RenderScene, VolumetricFog, WORLD_3D, Water, render_to_rgba,
    shapes,
};

fn scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let ball = scene.add_mesh(shapes::sphere(16, 12));
    let red = scene.add_material(Material {
        base_color: [0.8, 0.2, 0.1, 1.0],
        ..Default::default()
    });
    scene.add_instance(ball, red, Mat4::from_translation(Vec3::new(0.0, 0.6, 0.0)));
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 6.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene
}

// The feature's capability: a 2D build says it has no 3D world.
const _: () = assert!(!WORLD_3D);

#[test]
fn water_and_volumetric_fog_are_not_drawn_in_the_2d_profile() {
    let desc = RenderDesc {
        width: 96,
        height: 64,
        ..Default::default()
    };
    let Ok(plain) = render_to_rgba(&desc, &scene()) else {
        eprintln!("no adapter: skipped");
        return;
    };
    let mut world = scene();
    world.water.push(Water {
        transform: Mat4::IDENTITY,
        size: Vec2::new(40.0, 40.0),
        deep: Vec3::new(0.0, 0.2, 0.9),
        opacity: 1.0,
        ..Default::default()
    });
    world.volumetric_fog = Some(VolumetricFog {
        density: 0.5,
        ..Default::default()
    });
    let with = render_to_rgba(&desc, &world).expect("the frame renders");
    assert_eq!(plain.pixels, with.pixels, "the water and the fog drew");
}
