//! Sky, showcase and instanced model reference scenes.

use super::*;

pub(super) fn sky_only() -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(0.3, 0.12, -0.9);
    scene.environment.sky = ProceduralSky {
        sun_direction: sun,
        sun_color: Vec3::new(1.0, 0.85, 0.65),
        look: None,
    };
    scene.light.direction_to_light = sun;
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.0, 0.0),
        target: Vec3::new(0.15, 1.15, -1.0),
        projection: Projection::Perspective {
            fov_y_degrees: 70.0,
        },
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

pub(super) fn showcase() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let models = [load("trickster.glb")?, load("dropship.glb")?];
    let parts: Vec<_> = models.iter().map(|m| (m.add_to(&mut scene), m)).collect();
    crate::fx_scenes::terrain(&mut scene, 60.0);
    let place = |scene: &mut RenderScene, k: usize, pos: Vec3, yaw: f32, height: f32| {
        let (p, m) = &parts[k];
        let scale = height / (m.max - m.min).max_element().max(1e-3);
        let t = Mat4::from_translation(pos + Vec3::Y * (-m.min.y * scale))
            * Mat4::from_quat(Quat::from_rotation_y(yaw))
            * Mat4::from_scale(Vec3::splat(scale));
        p.instance(scene, t);
    };
    for (i, x) in [-3.0f32, 0.0, 3.0].iter().enumerate() {
        place(
            &mut scene,
            0,
            Vec3::new(*x, 0.0, -1.0 - i as f32),
            0.4 - i as f32 * 0.4,
            2.0,
        );
    }
    place(&mut scene, 1, Vec3::new(-5.0, 0.0, -5.0), 1.2, 3.0);
    place(&mut scene, 1, Vec3::new(5.5, 0.0, -4.0), -0.6, 3.0);
    let orb = scene.add_mesh(shapes::sphere(32, 24));
    for (i, c) in [[10.0, 3.0, 1.0], [1.0, 4.0, 12.0], [3.0, 12.0, 4.0]]
        .iter()
        .enumerate()
    {
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: *c,
            ..Default::default()
        });
        scene.add_instance(
            orb,
            m,
            Mat4::from_translation(Vec3::new(-2.0 + i as f32 * 2.2, 2.6, 0.8))
                * Mat4::from_scale(Vec3::splat(0.35)),
        );
    }
    scene.lights = vec![
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(0.2, -1.0, -0.3),
                angle_degrees: 30.0,
                penumbra: 0.4,
            },
            position: Vec3::new(-1.0, 7.0, 2.0),
            color: Vec3::new(1.0, 0.9, 0.75),
            intensity: 120.0,
            range: 20.0,
            cast_shadows: true,
        },
        Light {
            position: Vec3::new(4.0, 1.5, -1.5),
            color: Vec3::new(0.3, 0.6, 1.0),
            intensity: 10.0,
            range: 9.0,
            cast_shadows: true,
            ..Default::default()
        },
        Light {
            position: Vec3::new(-4.5, 1.0, 0.5),
            color: Vec3::new(1.0, 0.3, 0.1),
            intensity: 6.0,
            range: 6.0,
            ..Default::default()
        },
    ];
    let sun = Vec3::new(-0.7, 0.3, -0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.75, 0.5),
        intensity: 1.5,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.35,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.5, 3.2, 9.0),
        target: Vec3::new(0.0, 1.3, -1.5),
        exposure: Exposure::Auto {
            compensation: 0.3,
            min_ev: -4.0,
            max_ev: 4.0,
            adapt_speed_up: 0.0,
            adapt_speed_down: 0.0,
        },
        ..Default::default()
    };
    // A world-space label and a few sprites in the xy plane, facing the camera.
    let font = load_font(&mut scene)?;
    scene.texts.push(Text {
        position: Vec3::new(-3.1, 4.3, -2.0),
        size: 0.8,
        color: [3.0, 2.6, 1.6, 1.0],
        ..Text::new(font, "Origin Engine 原点引擎")
    });
    let [disc, _, ring] = sprite_textures(&mut scene);
    for i in 0..40 {
        let a = i as f32 * 0.9;
        scene.add_sprite(Sprite {
            texture: Some(if i % 3 == 0 { ring } else { disc }),
            position: Vec3::new(
                a.cos() * (2.0 + i as f32 * 0.12),
                3.0 + (a * 1.7).sin() * 0.8,
                -2.5,
            ),
            size: Vec2::splat(0.18 + (i % 4) as f32 * 0.05),
            color: [2.0, 1.4, 0.6, 0.8],
            ..Default::default()
        });
    }
    Ok(scene)
}

/// The spike's time for its last timed frame (frame 59 of 60).
const SPIKE_TIME: f32 = 59.0 / 60.0;
const SUN: Vec3 = Vec3::new(0.45, 0.8, 0.35);
const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);

/// The yaw that turns trickster.glb towards +Z, the camera's side in the
/// scenes. The asset faces +X: its inverse bind matrices cancel the 90°
/// yaw on its top node (`trickster_Model`), so glTF's rule (a skinned mesh
/// is placed by its joints, never by its node) draws it facing +X, as every
/// animated draw always did; until the unposed draw followed that rule it
/// faced -Z. The hero scenes add their own yaw to this one.
pub const TRICKSTER_TO_Z: f32 = -std::f32::consts::FRAC_PI_2;

pub(crate) fn load(name: &str) -> Result<GltfModel, String> {
    load_glb(&asset(name)?).map_err(|e| format!("{name}: {e}"))
}

pub(super) fn spike_gltf_scene() -> Result<RenderScene, String> {
    let models = [load("trickster.glb")?, load("dropship.glb")?];
    let mut scene = RenderScene::new();
    let mut placed = Vec::new();
    for model in &models {
        let parts = model.add_to(&mut scene);
        placed.push((parts, model.min, model.max));
    }
    let parts: Vec<_> = placed
        .iter()
        .map(|(p, min, max)| (p.parts.clone(), *min, *max))
        .collect();
    layout(&mut scene, &parts, 25, 3.0);
    Ok(scene)
}

/// Moves every instance of the sphere field up or down on its own phase.
pub(super) fn bob_spheres(scene: &mut RenderScene, frame: u32) {
    let t = frame as f32 / 60.0;
    for (i, inst) in scene.instances.iter_mut().enumerate() {
        let speed = 1.0 + (i % 7) as f32 * 0.25;
        let phase = (i as f32 * 0.37) % std::f32::consts::TAU;
        let dy = (t * speed + phase).sin() - ((t - 1.0 / 60.0) * speed + phase).sin();
        inst.transform.w_axis.y += dy * 0.5;
    }
}

pub(super) fn sphere_field(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    let mesh = scene.add_mesh(shapes::sphere(16, 12));
    let material = scene.add_material(Material {
        base_color: [0.85, 0.35, 0.2, 1.0],
        metallic: 0.1,
        roughness: 0.45,
        ..Default::default()
    });
    layout(
        &mut scene,
        &[(vec![(mesh, material)], Vec3::splat(-0.5), Vec3::splat(0.5))],
        count,
        1.2,
    );
    scene
}

type Parts = (
    Vec<(og_render::MeshHandle, og_render::MaterialHandle)>,
    Vec3,
    Vec3,
);

/// The spike's `reference_scene`: `count` instances of each model on a grid,
/// a checker ground, the camera orbiting the bounds at `SPIKE_TIME`, and the
/// shadow box fitted to the same bounds. The arithmetic follows the spike
/// exactly (this reproduced `.spike-out/native.png` byte for byte, see
/// `tests/evidence/M0/NOTES.md`).
fn layout(scene: &mut RenderScene, models: &[Parts], count: u32, spacing: f32) {
    let n = models.len().max(1) as f32;
    let side_count = (count as f32).sqrt().ceil();
    let ground_size = (side_count * spacing + 6.0).max(12.0);
    let mut min = Vec3::new(-ground_size / 2.0, 0.0, -ground_size / 2.0);
    let mut max = Vec3::new(ground_size / 2.0, 0.0, ground_size / 2.0);
    for (k, (parts, model_min, model_max)) in models.iter().enumerate() {
        let extent = *model_max - *model_min;
        let scale = 2.0 / extent.max_element().max(1e-3);
        let offset = Vec3::new(0.0, 0.0, (k as f32 - (n - 1.0) / 2.0) * spacing * 0.5);
        let side = side_count as u32;
        let corners = [*model_min, *model_max];
        for i in 0..count {
            let x = (i % side) as f32 - (side as f32 - 1.0) / 2.0;
            let z = (i / side) as f32 - (side as f32 - 1.0) / 2.0;
            let rot = Quat::from_rotation_y((i as f32 * 0.7).sin() * 1.2);
            let grid =
                Mat4::from_translation(Vec3::new(x * spacing, -model_min.y * scale, z * spacing))
                    * Mat4::from_quat(rot)
                    * Mat4::from_scale(Vec3::splat(scale));
            let m = Mat4::from_translation(offset) * grid;
            for &(mesh, material) in parts {
                scene.add_instance(mesh, material, m);
            }
            for c in 0..8 {
                let corner = Vec3::new(
                    corners[c & 1].x,
                    corners[(c >> 1) & 1].y,
                    corners[(c >> 2) & 1].z,
                );
                let w = m.transform_point3(corner);
                min = min.min(w);
                max = max.max(w);
            }
        }
    }
    let checker = scene.add_texture(shapes::checker(512, 16));
    let ground_material = scene.add_material(Material {
        base_color: [1.0; 4],
        base_color_texture: Some(checker),
        metallic: 0.0,
        roughness: 0.85,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(ground_size, ground_size / 2.0));
    scene.add_instance(ground, ground_material, Mat4::IDENTITY);

    let center = (min + max) / 2.0;
    let radius = (max - min).length() / 2.0;
    let angle = 0.6 + SPIKE_TIME * 0.25;
    let dist = radius * 1.05;
    scene.camera = Camera {
        position: center + Vec3::new(angle.cos() * dist, radius * 0.5, angle.sin() * dist),
        target: center,
        up: Vec3::Y,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        near: 0.1,
        far: radius * 8.0,
        exposure: Exposure::linear(0.8),
    };
    scene.light = DirectionalLight {
        direction_to_light: SUN,
        color: SUN_COLOR,
        intensity: 4.0,
        cast_shadows: true,
        // Cascades reach the far side of the instances' bounds.
        shadow_bounds: ShadowBounds::Auto,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: SUN,
            sun_color: SUN_COLOR,
            look: None,
        },
        intensity: 0.55,
        background: Background::Sky,
        ..Environment::default()
    };
}
