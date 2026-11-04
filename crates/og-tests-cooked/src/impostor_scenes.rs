//! A cooked model's octahedral impostor beside the model itself (G5's
//! card shading, og-render's `impostor`): the fx-showcase tree cooked with
//! `impostor true`, drawn once as its meshes on the left and once as its
//! card on the right, under a high sun in front of them, so the front of
//! each trunk faces the sun and only the crown above keeps it off.
//!
//! - `impostor_self_shadow_scene()`: the two trees on a floor that casts
//!   and takes shadows; the mesh tree's trunk is in its crown's cascade
//!   shadow, and the card's must be in its own: a card inside its sphere
//!   is shaded by its depth atlas, the cascades only reaching it past the
//!   sphere (impostor.wgsl `imp_self_shadow`).

use glam::{Mat4, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, Exposure, Impostor, ImpostorDraw, RenderScene, StandardMaterial,
    shapes,
};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The tree cooked with its impostor, once per process (its own project,
/// so the other cooked scenes' trees stay as they are).
fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-impostor-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        std::fs::write(models.join("tree.glb"), crate::scenes::asset("tree.glb")?)
            .map_err(|e| format!("tree.glb: {e}"))?;
        std::fs::write(
            models.join("tree.glb.ogt"),
            "type import\nversion 1\n---\nmodel {\n  impostor true\n  impostorSize 256\n}\n",
        )
        .map_err(|e| format!("tree.glb.ogt: {e}"))?;
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

/// Where the mesh tree and the card stand (their roots).
pub const MESH_AT: Vec3 = Vec3::new(-2.6, 0.0, 0.0);
pub const CARD_AT: Vec3 = Vec3::new(2.6, 0.0, 0.0);

/// The scene ([module docs](self)), and the tree's bounds in its own
/// space (min, max).
pub fn impostor_self_shadow_scene() -> Result<(RenderScene, (Vec3, Vec3)), String> {
    let root = cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let model = bundle
        .load_model("models/tree.glb")
        .map_err(|e| e.to_string())?;
    let i = model
        .impostor
        .clone()
        .ok_or("the tree was cooked without its impostor")?;
    let texture = |id: &str| {
        bundle
            .load_texture(id)
            .and_then(|t| t.to_texture_data())
            .map_err(|e| format!("{id}: {e}"))
    };
    let mut scene = RenderScene::new();
    // High, and in front of the trees (towards the camera at +z).
    let sun = Vec3::new(0.15, 0.85, 0.5).normalize();
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        max_shadow_distance: 30.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.5;
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.32, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(60.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    let textures: Vec<_> = model
        .textures
        .iter()
        .map(|t| texture(&t.id))
        .collect::<Result<_, _>>()?;
    let parts = model.add_to(&mut scene, &textures).parts;
    for &(mesh, material, local) in &parts {
        scene.add_instance(mesh, material, Mat4::from_translation(MESH_AT) * local);
    }
    // The card, as og-runtime makes it (assets.rs `insert_model`).
    let albedo = scene.add_texture(texture(&i.albedo)?);
    let normal = scene.add_texture(texture(&i.normal)?);
    let atlas = match &i.material {
        Some(m) => Some(scene.add_texture(texture(m)?)),
        None => None,
    };
    let material = scene.add_standard_material(StandardMaterial {
        map: albedo.slot(),
        normalMap: normal.slot(),
        roughnessMap: atlas.map_or(0, |a| a.slot()),
        alphaTest: 0.5,
        side: 2,
        roughness: 0.85,
        ..Default::default()
    });
    scene.impostors.push(ImpostorDraw {
        impostor: Impostor {
            material,
            grid: i.grid.clamp(2, og_render::MAX_IMPOSTOR_GRID),
            center: i.center,
            radius: i.radius,
            material_atlas: atlas.is_some(),
        },
        transform: Mat4::from_translation(CARD_AT),
        cast_shadows: true,
    });
    let mid = (model.min + model.max) * 0.5;
    scene.camera = Camera {
        position: Vec3::new(0.0, mid.y, 11.0),
        target: Vec3::new(0.0, mid.y, 0.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    Ok((scene, (model.min, model.max)))
}
