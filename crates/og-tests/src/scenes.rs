//! The named gfx scenes. Each one is a fixed [`RenderDesc`] and [`RenderScene`]
//! plus the tolerance its reference comparison uses.

use crate::compare::Tolerance;
use glam::{Mat4, Quat, Vec3};
use og_render::gltf::{load_glb, GltfModel};
use og_render::{
    shapes, Background, Camera, DirectionalLight, Environment, Material, ProceduralSky, RenderDesc,
    RenderScene, ShadowBounds,
};
use std::path::PathBuf;

/// Every scene the harness knows, in the order `og-gfx --all` renders them.
pub const SCENES: &[&str] = &["gltf-reference", "spheres-20k"];

pub struct GfxScene {
    pub desc: RenderDesc,
    pub scene: RenderScene,
    pub tolerance: Tolerance,
}

/// The directory of the committed test assets (`crates/og-tests/assets`).
pub fn assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

/// Builds the scene called `name`.
pub fn build(name: &str) -> Result<GfxScene, String> {
    let desc = RenderDesc {
        width: 1280,
        height: 720,
        ..Default::default()
    };
    let tolerance = Tolerance::default();
    let scene = match name {
        // The spike's reference scene (two glTF models x 25 on a checker
        // ground), drawn with the sky pass and mipmapped textures.
        "gltf-reference" => spike_gltf_scene()?,
        // 20,000 low-poly spheres submitted one by one: two batches in all.
        "spheres-20k" => sphere_field(20_000),
        other => {
            return Err(format!(
                "unknown gfx scene '{other}' (known: {})",
                SCENES.join(", ")
            ))
        }
    };
    Ok(GfxScene {
        desc,
        scene,
        tolerance,
    })
}

/// The spike's time for its last timed frame (frame 59 of 60).
const SPIKE_TIME: f32 = 59.0 / 60.0;
const SUN: Vec3 = Vec3::new(0.45, 0.8, 0.35);
const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);

fn load(name: &str) -> Result<GltfModel, String> {
    let path = assets_dir().join(name);
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    load_glb(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn spike_gltf_scene() -> Result<RenderScene, String> {
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

fn sphere_field(count: u32) -> RenderScene {
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
        fov_y_degrees: 50.0,
        near: 0.1,
        far: radius * 8.0,
        exposure: 0.8,
    };
    scene.light = DirectionalLight {
        direction_to_light: SUN,
        color: SUN_COLOR,
        intensity: 4.0,
        cast_shadows: true,
        // The renderer fits the shadow box to the instances (the spike passed
        // this sphere explicitly; the fitted box is slightly tighter).
        shadow_bounds: ShadowBounds::Auto,
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: SUN,
            sun_color: SUN_COLOR,
        },
        intensity: 0.55,
        background: Background::Sky,
    };
}
