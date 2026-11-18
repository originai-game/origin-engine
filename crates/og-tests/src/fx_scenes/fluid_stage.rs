//! `fluid-3d`: a stone basin in the shrine court on a clear afternoon. The
//! pool is particle fluid drawn as one screen-space surface over a glazed
//! mosaic, which shows through it tinted and bent by its refraction; a
//! drop of amber, in its particles' own colours, falls into it under a
//! stone lantern's light. The measured fixture is
//! [`crate::p2_scenes::fluid_3d_scene`].

use super::*;
use crate::p2_scenes::{fluid_3d_drop, fluid_3d_pool};

pub fn fluid_stage_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    let coping = kit::surface_tinted(&mut scene, "stone", [0.62, 0.58, 0.52]);
    // The basin: a raised rim round the pool (the pool spans x ±1.5,
    // z ±0.9).
    for (min, max) in [
        (Vec3::new(-1.9, 0.0, -1.3), Vec3::new(1.9, 0.6, -0.95)),
        (Vec3::new(-1.9, 0.0, -0.95), Vec3::new(-1.55, 0.6, 0.95)),
        (Vec3::new(1.55, 0.0, -0.95), Vec3::new(1.9, 0.6, 0.95)),
        (Vec3::new(-1.9, 0.0, 0.95), Vec3::new(1.9, 0.5, 1.3)),
    ] {
        let wall = scene.add_mesh(kit::block(min, max, 1.0));
        scene.add_instance(wall, coping, Mat4::IDENTITY);
    }
    // The basin's floor: glazed tiles, sea glass to indigo and white.
    let glazes: Vec<_> = [
        [0.1, 0.42, 0.45],
        [0.75, 0.78, 0.74],
        [0.06, 0.16, 0.42],
        [0.18, 0.55, 0.5],
        [0.85, 0.62, 0.22],
    ]
    .into_iter()
    .map(|c| {
        scene.add_material(Material {
            base_color: [c[0], c[1], c[2], 1.0],
            roughness: 0.25,
            ..Default::default()
        })
    })
    .collect();
    let tile = scene.add_mesh(shapes::cuboid(Vec3::new(0.27, 0.03, 0.27)));
    for gx in 0..10 {
        for gz in 0..6 {
            let (x, z) = (-1.35 + gx as f32 * 0.3, -0.75 + gz as f32 * 0.3);
            // Rings of colour round the middle, a gold star at its heart.
            let ring = ((x * x + z * z * 2.2).sqrt() / 0.45) as usize;
            let glaze = if ring == 0 { 4 } else { ring % 4 };
            scene.add_instance(
                tile,
                glazes[glaze],
                Mat4::from_translation(Vec3::new(x, 0.015, z)),
            );
        }
    }
    let spacing = 0.1;
    scene.world.fluids_3d.push(og_render::Fluid3d {
        particles: fluid_3d_pool(spacing).into(),
        color: Vec3::new(0.12, 0.45, 0.55),
        opacity: 0.5,
        radius: 0.12,
        volume: spacing.powi(3),
        ..Default::default()
    });
    let drop = fluid_3d_drop(Vec3::new(0.35, 1.25, 0.1), 0.3, spacing);
    let colors: Vec<Vec3> = drop
        .iter()
        .map(|p| {
            Vec3::new(1.0, 0.62, 0.1).lerp(Vec3::new(0.8, 0.18, 0.04), (1.25 - p.y) / 0.6 + 0.5)
        })
        .collect();
    scene.world.fluids_3d.push(og_render::Fluid3d {
        particles: drop.into(),
        colors: Some(colors.into()),
        color: Vec3::new(0.95, 0.5, 0.1),
        opacity: 0.95,
        radius: 0.12,
        volume: spacing.powi(3),
        ..Default::default()
    });
    // A stone lantern at the basin's corner, its light on the drop.
    let toro = kit::prop(&mut scene, "toro");
    let fire = crate::shadow_scenes::toro(&mut scene, &toro, Vec3::new(-2.6, 0.0, 0.6), 0.6, 1.6);
    scene.lighting.lights.push(Light {
        position: fire,
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 5.0,
        range: 5.0,
        ..Default::default()
    });
    // Afternoon: the sun 35° up over the left shoulder.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.45, 0.6, 0.65));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 2.7, 3.0),
        target: Vec3::new(0.0, 0.35, -0.25),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}
