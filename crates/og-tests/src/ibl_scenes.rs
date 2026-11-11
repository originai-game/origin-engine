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

/// The vertical field of view of [`environment_scene`]'s camera (degrees).
pub const ENVIRONMENT_FOV: f32 = 40.0;

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
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.environment_map = Some(EnvironmentMap {
        rotation: 0.6,
        ..EnvironmentMap::new(map)
    });
    // A 40° lens from 10 m frames the rows as a 50° one from 8 m did, with
    // the end spheres round rather than stretched by the wide angle.
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.375, 10.0),
        target: Vec3::new(0.0, 0.0, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: ENVIRONMENT_FOV,
        },
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// A synthetic outdoor equirect (256×128): a blue sky paling to the
/// horizon with a small bright sun up to the right of +z, and a ground of
/// grey and green checks (16 across the longitude, 8 down to the nadir)
/// below the horizon, baked like [`studio_map`]. Projected onto a grounded
/// dome its checks become a flat polar grid.
pub fn outdoor_map(size: u32) -> Result<EnvironmentMapData, String> {
    let (w, h, rgb) = outdoor_pixels();
    bake(
        &Equirect {
            width: w,
            height: h,
            rgb: &rgb,
        },
        &BakeDesc {
            size,
            samples: 256,
            ..Default::default()
        },
    )
}

/// [`outdoor_map`]'s image: its width, height and linear RGB.
pub fn outdoor_pixels() -> (u32, u32, Vec<f32>) {
    let (w, h) = (256u32, 128u32);
    let sun = Vec3::new(0.4, 0.55, 0.73).normalize();
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            // Bake's mapping (Equirect): the centre column +x, longitude
            // from +x towards +z; rows from +y down.
            let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * std::f32::consts::TAU;
            let lat =
                std::f32::consts::FRAC_PI_2 - (y as f32 + 0.5) / h as f32 * std::f32::consts::PI;
            let d = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
            let c = if d.y >= 0.0 {
                let up = d.y.powf(0.5);
                let sky = Vec3::new(0.75, 0.85, 1.0).lerp(Vec3::new(0.15, 0.35, 0.9), up) * 1.6;
                if d.dot(sun) > 0.9990 {
                    Vec3::splat(60.0)
                } else {
                    sky
                }
            } else {
                let check = (x / 16 + y / 8) % 2 == 0;
                if check {
                    Vec3::new(0.32, 0.32, 0.30)
                } else {
                    Vec3::new(0.12, 0.30, 0.08)
                }
            };
            rgb.extend_from_slice(&c.to_array());
        }
    }
    (w, h, rgb)
}

/// [`environment_scene`] with the studio's background blurred half way
/// (`Environment::background_blurriness` 0.5): the spheres are lit exactly
/// as in it, only the background softens.
pub fn blurry_scene() -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    scene.lighting.environment.background_blurriness = 0.5;
    Ok(scene)
}

/// [`environment_scene`] half way through a blend from the studio to
/// [`outdoor_map`] (light and background).
pub fn blend_scene(weight: f32) -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    let to = scene.add_environment_map(outdoor_map(64)?);
    if let Some(m) = &mut scene.lighting.environment_map {
        m.blend = Some(og_render::EnvironmentBlend { to, weight });
    }
    Ok(scene)
}

/// A photograph's worth of outdoor environment taken 1.5 m above a flat
/// floor (2048×1024, as a real HDRI is large): the outdoor map's sky and
/// sun above the horizon, and below it a floor of 1 m grey and green
/// squares as the camera saw it, fading to their mean colour from 20 to
/// 35 m where a photo's pixels would average them. Projected onto a
/// grounded dome of the same height the squares come back as a flat grid.
pub fn ground_pixels() -> (u32, u32, Vec<f32>) {
    let (w, h) = (2048u32, 1024u32);
    let sun = Vec3::new(0.4, 0.55, 0.73).normalize();
    let grey = Vec3::new(0.32, 0.32, 0.30);
    let green = Vec3::new(0.12, 0.30, 0.08);
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * std::f32::consts::TAU;
            let lat =
                std::f32::consts::FRAC_PI_2 - (y as f32 + 0.5) / h as f32 * std::f32::consts::PI;
            let d = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
            let c = if d.y >= 0.0 {
                let up = d.y.powf(0.5);
                let sky = Vec3::new(0.75, 0.85, 1.0).lerp(Vec3::new(0.15, 0.35, 0.9), up) * 1.6;
                if d.dot(sun) > 0.9990 {
                    Vec3::splat(60.0)
                } else {
                    sky
                }
            } else {
                let p = d * (1.5 / -d.y.min(-1e-4));
                let check = (p.x.floor() as i64 + p.z.floor() as i64).rem_euclid(2) == 0;
                let fade = ((p.x.hypot(p.z) - 20.0) / 15.0).clamp(0.0, 1.0);
                (if check { grey } else { green }).lerp((grey + green) * 0.5, fade)
            };
            rgb.extend_from_slice(&c.to_array());
        }
    }
    (w, h, rgb)
}

/// Three spheres (chrome, white, red) standing on [`ground_pixels`]' floor,
/// projected onto a grounded dome (height 1.5 m, radius 40 m, centred on
/// the origin), seen from 1.6 m up; `grounded` false draws the same map at
/// infinity, where its checks bend away towards the horizon instead.
pub fn grounded_scene(grounded: bool) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let (w, h, rgb) = ground_pixels();
    let mut irradiance = og_render::EquirectIrradiance::new(w, h);
    for (y, row) in rgb.chunks(w as usize * 3).enumerate() {
        irradiance.add_row(y as u32, row);
    }
    let texels = rgb
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| og_render::rgb9e5(*c))
        .collect();
    let map = scene.add_environment_equirect(
        og_render::EnvironmentEquirect {
            width: w,
            height: h,
            texels,
            size: 512,
        },
        irradiance.finish(),
    )?;
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let looks = [
        ([0.95, 0.93, 0.88, 1.0], 1.0, 0.05),
        ([0.8, 0.8, 0.8, 1.0], 0.0, 0.5),
        ([0.7, 0.06, 0.04, 1.0], 0.0, 0.15),
    ];
    for (i, (base_color, metallic, roughness)) in looks.into_iter().enumerate() {
        let m = scene.add_material(Material {
            base_color,
            metallic,
            roughness,
            ..Default::default()
        });
        let r = 0.6;
        let at = Vec3::new(-1.6 + i as f32 * 1.6, r, -0.5 * (i as f32 - 1.0).abs());
        scene.add_instance(
            sphere,
            m,
            Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(2.0 * r)),
        );
    }
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.lighting.environment.intensity = 1.0;
    scene.lighting.environment.background = Background::Sky;
    scene.lighting.environment.ground = grounded.then_some(og_render::GroundProjection {
        height: 1.5,
        radius: 40.0,
        center: Vec3::ZERO,
    });
    scene.lighting.environment_map = Some(EnvironmentMap::new(map));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.6, 5.5),
        target: Vec3::new(0.0, 0.4, 0.0),
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// [`environment_scene`] lit by [`outdoor_map`]'s image loaded uncooked:
/// the renderer resamples and GGX-prefilters it on the GPU
/// (`RenderScene::add_environment_equirect`), with the irradiance gathered
/// from its rows as og.load.environmentData gathers it. With `cooked` the
/// cook's CPU bake of the same image instead, the frame it should match.
pub fn uncooked_scene(cooked: bool) -> Result<RenderScene, String> {
    let mut scene = environment_scene()?;
    let (w, h, rgb) = outdoor_pixels();
    let map = if cooked {
        scene.add_environment_map(outdoor_map(64)?)
    } else {
        let mut irradiance = og_render::EquirectIrradiance::new(w, h);
        for (y, row) in rgb.chunks(w as usize * 3).enumerate() {
            irradiance.add_row(y as u32, row);
        }
        let texels = rgb
            .as_chunks::<3>()
            .0
            .iter()
            .map(|c| og_render::rgb9e5(*c))
            .collect();
        scene.add_environment_equirect(
            og_render::EnvironmentEquirect {
                width: w,
                height: h,
                texels,
                size: 64,
            },
            irradiance.finish(),
        )?
    };
    if let Some(m) = &mut scene.lighting.environment_map {
        m.map = map;
    }
    Ok(scene)
}
