//! Plants the cook grows (a `plant` block in a `type model`, og-geom's
//! `plant`): og-tests' assets/plants, the models guide's four kinds (an
//! oak, a pine, a flowering shrub, a daisy), cooked in a project of their
//! own.
//!
//! - `grown_scene()` (`plant-grown`): the oak, the pine and the shrub
//!   standing in a row on a floor with three daisies near the camera,
//!   each drawn as its cooked parts (the wood, then the leaves and petals
//!   with the leaf's material), under a sun from the front right.

use glam::{Mat4, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, MaterialHandle, MeshHandle, Projection, RenderScene,
    StandardMaterial, shapes,
};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The plants, cooked once per process.
const PLANTS: [&str; 4] = ["oak", "pine", "bush", "daisy"];

fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-plants-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/plants");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for name in PLANTS {
            let file = format!("{name}.ogt");
            std::fs::write(
                dir.join(&file),
                crate::scenes::asset(&format!("plants/{file}"))?,
            )
            .map_err(|e| format!("{file}: {e}"))?;
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        Ok(root)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// A cooked plant in a scene: its parts (the wood first).
struct Plant {
    parts: Vec<(MeshHandle, MaterialHandle, Mat4)>,
}

fn plant(scene: &mut RenderScene, bundle: &Bundle, name: &str) -> Result<Plant, String> {
    let id = format!("plants/{name}.ogt");
    let model = bundle.load_model(&id).map_err(|e| format!("{id}: {e}"))?;
    let texture = |t: &str| {
        bundle
            .load_texture(t)
            .and_then(|t| t.to_texture_data())
            .map_err(|e| format!("{t}: {e}"))
    };
    let textures: Vec<_> = model
        .textures
        .iter()
        .map(|t| texture(&t.id))
        .collect::<Result<_, _>>()?;
    let parts = model.add_to(scene, &textures).parts;
    Ok(Plant { parts })
}

fn sun(scene: &mut RenderScene, direction: Vec3, max_shadow_distance: f32) {
    let direction = direction.normalize();
    scene.lighting.light = DirectionalLight {
        direction_to_light: direction,
        intensity: 3.0,
        max_shadow_distance,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = direction;
    scene.lighting.environment.intensity = 0.7;
}

/// Where `plant-grown` stands its oak, pine and shrub (their roots).
pub const GROWN_AT: [Vec3; 3] = [
    Vec3::new(-7.5, 0.0, 0.0),
    Vec3::new(1.5, 0.0, -1.0),
    Vec3::new(7.5, 0.0, 2.0),
];

/// `plant-grown` ([module docs](self)).
pub fn grown_scene() -> Result<RenderScene, String> {
    let bundle = Bundle::open(&cooked()?.join("cooked")).map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    sun(&mut scene, Vec3::new(0.5, 0.7, 0.45), 60.0);
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.32, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(120.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    for (name, at) in ["oak", "pine", "bush"].into_iter().zip(GROWN_AT) {
        let p = plant(&mut scene, &bundle, name)?;
        for (mesh, material, local) in p.parts {
            scene.add_instance(mesh, material, Mat4::from_translation(at) * local);
        }
    }
    let daisy = plant(&mut scene, &bundle, "daisy")?;
    for (k, x) in [2.6f32, 3.1, 3.5].into_iter().enumerate() {
        let at = Mat4::from_translation(Vec3::new(x, 0.0, 15.2 + 0.4 * k as f32))
            * Mat4::from_rotation_y(1.7 * k as f32);
        for &(mesh, material, local) in &daisy.parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    scene.frame.camera = Camera {
        position: Vec3::new(1.0, 3.2, 25.0),
        target: Vec3::new(0.0, 5.0, 0.0),
        far: 200.0,
        projection: Projection::Perspective {
            fov_y_degrees: 50.0,
        },
        ..Default::default()
    };
    Ok(scene)
}
