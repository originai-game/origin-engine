//! The phase 2 render round's scenes: many local lights (Forward+), the M6
//! material features, a displaced skinned mesh and the moon.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, Environment, Exposure, Light, LightKind, Material, RenderScene, shapes,
};

/// A small integer hash to [0, 1) (deterministic colours and placements).
fn hash01(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// A saturated colour of hue `h` (0..1).
fn hue(h: f32) -> Vec3 {
    let k = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        1.0 - k.min(4.0 - k).clamp(0.0, 1.0)
    };
    Vec3::new(k(5.0), k(3.0), k(1.0))
}

/// The lights of [`many_lights`]: `count` of them over a 48 m square, every
/// eighth a spot pointing down, the rest points, hues and heights hashed.
pub fn many_lights_list(count: u32) -> Vec<Light> {
    let side = (count as f32).sqrt().ceil() as u32;
    let spacing = 48.0 / side as f32;
    (0..count)
        .map(|i| {
            let (gx, gz) = (i % side, i / side);
            let x = -24.0 + (gx as f32 + 0.5 + (hash01(i, 1) - 0.5) * 0.6) * spacing;
            let z = -24.0 + (gz as f32 + 0.5 + (hash01(i, 2) - 0.5) * 0.6) * spacing;
            let color = hue(hash01(i, 3)).lerp(Vec3::ONE, 0.25);
            if i % 8 == 7 {
                Light {
                    kind: LightKind::Spot {
                        direction: Vec3::new(hash01(i, 4) - 0.5, -1.0, hash01(i, 5) - 0.5),
                        angle_degrees: 28.0,
                        penumbra: 0.4,
                    },
                    position: Vec3::new(x, 3.2, z),
                    color,
                    intensity: 14.0,
                    range: 6.0,
                    cast_shadows: false,
                }
            } else {
                Light {
                    position: Vec3::new(x, 0.35 + hash01(i, 6) * 0.9, z),
                    color,
                    intensity: 2.2 + hash01(i, 7) * 1.6,
                    range: 2.6 + hash01(i, 8) * 1.4,
                    ..Default::default()
                }
            }
        })
        .collect()
}

/// Night over a 60 m plaza of pillars and spheres lit by `count` point and
/// spot lights (no sun, a faint sky light): the Forward+ scene.
pub fn many_lights(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 60.0, 0.5, 0.6);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 2.4, 0.5)));
    let sphere = scene.add_mesh(shapes::sphere(24, 16));
    let stone = scene.add_material(Material {
        base_color: [0.8, 0.78, 0.74, 1.0],
        roughness: 0.55,
        ..Default::default()
    });
    let metal = scene.add_material(Material {
        base_color: [0.92, 0.9, 0.86, 1.0],
        metallic: 1.0,
        roughness: 0.25,
        ..Default::default()
    });
    for i in 0..144u32 {
        let (gx, gz) = (i % 12, i / 12);
        let p = Vec3::new(-22.0 + gx as f32 * 4.0, 0.0, -22.0 + gz as f32 * 4.0);
        if (gx + gz) % 2 == 0 {
            scene.add_instance(pillar, stone, Mat4::from_translation(p + Vec3::Y * 1.2));
        } else {
            scene.add_instance(
                sphere,
                metal,
                Mat4::from_translation(p + Vec3::Y * 0.5) * Mat4::from_scale(Vec3::splat(0.5)),
            );
        }
    }
    scene.lights = many_lights_list(count);
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.03,
        background: Background::Color(Vec3::new(0.003, 0.004, 0.01)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 26.0, 30.0),
        target: Vec3::new(0.0, 0.0, -2.0),
        far: 200.0,
        exposure: Exposure::linear(1.4),
        ..Default::default()
    };
    scene
}
