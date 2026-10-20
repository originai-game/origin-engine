//! The environment-map scene: spheres lit only by a prefiltered HDR
//! environment (`assets/env/studio.hdr`, a synthetic 64×32 studio made by
//! `scripts/make-studio-hdr.py`), baked by og-render's `bake` as the cook
//! does, drawn as the background too.

use crate::scenes::asset;
use glam::{Mat4, Vec3};
use og_render::bake::{BakeDesc, Equirect, bake};
use og_render::{
    Background, Camera, DirectionalLight, EnvironmentMap, EnvironmentMapData, Exposure, Material,
    RenderScene, shapes,
};

/// The studio image baked as og-cook bakes an environment of `size`.
pub fn studio_map(size: u32) -> Result<EnvironmentMapData, String> {
    let bytes = asset("env/studio.hdr")?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Hdr)
        .map_err(|e| format!("env/studio.hdr: {e}"))?
        .to_rgb32f();
    bake(
        &Equirect {
            width: img.width(),
            height: img.height(),
            rgb: img.as_raw(),
        },
        &BakeDesc {
            size,
            samples: 512,
            ..Default::default()
        },
    )
}

/// Two rows of five spheres, roughness 0 to 1 from left to right: chrome
/// above, a white dielectric below, lit by the studio alone (no sun), the
/// map turned 0.6 rad and drawn behind them.
pub fn environment_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let map = scene.add_environment_map(studio_map(64)?);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    for row in 0..2 {
        for i in 0..5 {
            let roughness = i as f32 / 4.0;
            let m = scene.add_material(if row == 0 {
                Material {
                    base_color: [0.95, 0.93, 0.88, 1.0],
                    metallic: 1.0,
                    roughness,
                    ..Default::default()
                }
            } else {
                Material {
                    base_color: [0.7, 0.7, 0.7, 1.0],
                    metallic: 0.0,
                    roughness,
                    ..Default::default()
                }
            });
            let at = Vec3::new(-4.4 + i as f32 * 2.2, 1.1 - row as f32 * 2.2, 0.0);
            scene.add_instance(
                sphere,
                m,
                Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(1.8)),
            );
        }
    }
    scene.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment.intensity = 1.0;
    scene.environment.background = Background::Sky;
    scene.environment_map = Some(EnvironmentMap {
        rotation: 0.6,
        ..EnvironmentMap::new(map)
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 0.3, 8.0),
        target: Vec3::new(0.0, 0.0, 0.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}
