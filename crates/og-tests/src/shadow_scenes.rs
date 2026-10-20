//! Shadow and antialiasing scenes of the shadows-aa round
//! (`crates/og-render/README.md` §Shadows and antialiasing by tier): the
//! sun's soft shadows with cascade blending, eight shadowed local lights,
//! and the thin geometry under TAA with motion, Catmull-Rom history and
//! sharpening.

use crate::scenes::ground;
use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, DirectionalLight, Environment, Exposure, Light, LightKind, Material,
    RenderScene, shapes,
};

/// The sun's PCSS softness and cascade blend of `shadows-soft`.
pub const SOFTNESS: f32 = 0.03;
pub const CASCADE_BLEND: f32 = 0.2;

/// A low sun over a long field: a cube on the ground (contact shadows stay
/// sharp), a plank on two posts 2.5 m up and a ball floating at 3 m (their
/// penumbrae widen with the height), and a row of thin poles running away
/// from the camera across the cascade splits. `soft` turns PCSS and the
/// cascade blend on.
pub fn soft_shadows_scene(soft: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 160.0, 1.0, 0.9);
    let stone = scene.add_material(Material {
        base_color: [0.78, 0.74, 0.68, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let red = scene.add_material(Material {
        base_color: [0.75, 0.16, 0.1, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.2)));
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(0.25, 2.5, 0.25)));
    let plank = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 0.15, 1.2)));
    let ball = scene.add_mesh(shapes::sphere(32, 24));
    let pole = scene.add_mesh(shapes::cuboid(Vec3::new(0.15, 5.0, 0.15)));
    scene.add_instance(cube, red, Mat4::from_translation(Vec3::new(-3.5, 0.6, 1.0)));
    for x in [-1.6f32, 1.6] {
        scene.add_instance(
            post,
            stone,
            Mat4::from_translation(Vec3::new(1.5 + x, 1.25, -1.0)),
        );
    }
    scene.add_instance(
        plank,
        stone,
        Mat4::from_translation(Vec3::new(1.5, 2.575, -1.0)),
    );
    scene.add_instance(
        ball,
        red,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.4),
            Quat::IDENTITY,
            Vec3::new(-1.0, 3.0, -5.0),
        ),
    );
    for i in 0..10 {
        let z = -9.0 - i as f32 * 6.0;
        let x = if i % 2 == 0 { -3.0 } else { 3.0 };
        scene.add_instance(pole, stone, Mat4::from_translation(Vec3::new(x, 2.5, z)));
    }
    let sun = Vec3::new(0.75, 0.42, 0.3);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 4.0,
        max_shadow_distance: 90.0,
        shadow_softness: if soft { SOFTNESS } else { 0.0 },
        cascade_blend: if soft { CASCADE_BLEND } else { 0.0 },
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.45;
    scene.camera = Camera {
        position: Vec3::new(0.0, 5.5, 9.0),
        target: Vec3::new(0.0, 0.0, -14.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    scene
}

/// Night over a plaza: eight shadowed lights (six point, two spot), each
/// beside a pillar, the brightest nearest the camera, plus two more
/// shadowed point lights far behind (dim at the camera: past the tier's
/// eight, their shadows are the ones left out). `extra` adds those two.
pub fn many_shadowed_lights(extra: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.5, 0.7);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 2.4, 0.5)));
    let white = scene.add_material(Material {
        base_color: [0.85, 0.85, 0.85, 1.0],
        roughness: 0.6,
        ..Default::default()
    });
    let colors = [
        Vec3::new(1.0, 0.55, 0.25),
        Vec3::new(0.3, 0.6, 1.0),
        Vec3::new(0.35, 1.0, 0.4),
        Vec3::new(1.0, 0.3, 0.7),
        Vec3::new(1.0, 0.9, 0.4),
        Vec3::new(0.6, 0.4, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(0.3, 1.0, 1.0),
    ];
    let mut lights = Vec::new();
    for (i, color) in colors.iter().enumerate() {
        let col = (i % 4) as f32;
        let row = (i / 4) as f32;
        let at = Vec3::new(-6.0 + col * 4.0, 0.0, -1.0 - row * 5.0);
        scene.add_instance(
            pillar,
            white,
            Mat4::from_translation(at + Vec3::new(0.0, 1.2, 0.0)),
        );
        let kind = if i == 3 || i == 6 {
            LightKind::Spot {
                direction: Vec3::new(0.35, -1.0, -0.3),
                angle_degrees: 38.0,
                penumbra: 0.3,
            }
        } else {
            LightKind::Point
        };
        let position = if matches!(kind, LightKind::Spot { .. }) {
            at + Vec3::new(-1.0, 3.6, 0.8)
        } else {
            at + Vec3::new(-0.9, 1.0, 0.9)
        };
        lights.push(Light {
            kind,
            position,
            color: *color,
            intensity: if matches!(kind, LightKind::Spot { .. }) {
                40.0
            } else {
                9.0
            },
            range: 7.0,
            cast_shadows: true,
        });
    }
    if extra {
        for x in [-8.0f32, 8.0] {
            let at = Vec3::new(x, 0.0, -24.0);
            scene.add_instance(
                pillar,
                white,
                Mat4::from_translation(at + Vec3::new(0.0, 1.2, 0.0)),
            );
            lights.push(Light {
                position: at + Vec3::new(-0.9, 1.0, 0.9),
                color: Vec3::new(1.0, 0.8, 0.6),
                intensity: 6.0,
                range: 7.0,
                cast_shadows: true,
                ..Default::default()
            });
        }
    }
    scene.lights = lights;
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.03,
        background: Background::Color(Vec3::new(0.004, 0.006, 0.015)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 8.5, 9.0),
        target: Vec3::new(0.0, 0.0, -5.0),
        exposure: Exposure::linear(1.1),
        ..Default::default()
    };
    scene
}

/// The thin geometry's camera sliding sideways, 2 cm a frame: TAA's
/// history is resampled every frame, where Catmull-Rom keeps it sharper
/// than a bilinear tap.
pub fn pan_thin_geometry(scene: &mut RenderScene, frame: u32) {
    let x = frame as f32 * 0.02;
    scene.camera.position.x = x;
    scene.camera.target.x = x;
}
