//! The meshlet path's scenes (R1 batch 2, og-render's `meshlet` and
//! `meshlet_gpu`): the fx-showcase tree (3,160 triangles, cooked with the
//! default `lods 1 0.5 0.25` and cut into clusters), its instances drawn
//! at level 0's mesh so the GPU picks each one's level and culls its
//! clusters.
//!
//! - `meshlet-forest`: 132 trees on a grid 11 wide and 12 deep, 6 m to
//!   116 m from an eye 1.7 m up, each tinted by the level the GPU picked
//!   (`RenderScene::lod_debug`): the near rows green (the full mesh),
//!   then yellow, then orange; no tree missing or cut open (a cluster
//!   culled wrongly shows as a hole in a crown).
//! - `meshlet-occlusion`: the same forest without the tint, behind a wall
//!   4 m away that hides the left two thirds of the view, three frames
//!   so the third tests the clusters against the pyramid of the second's
//!   visible occluders: the right third shows the forest whole.
//! - `meshlet-near`: one tree 6 m away, drawn through the meshlet path;
//!   `meshlet_near_scene(false)` draws it through the regular path, and
//!   the two frames match (the level is 0 at that distance).
//! - `meshlet-shadows` (batch 3): five trees under a low sun and a
//!   shadowed spot light, seen from above so their long shadows lie on the
//!   ground: the casters go through the meshlet path's shadow views (each
//!   cascade's and the spot's face), and `shadow_scene(false)` draws the
//!   same frame through the regular path; the two match.

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, Exposure, Light, LightKind, MaterialHandle, MeshHandle, RenderScene,
    StandardMaterial, shapes,
};

/// A model's parts: each mesh, its material and its place in the model.
type Parts = Vec<(MeshHandle, MaterialHandle, Mat4)>;

/// The tree's parts in a new scene with its ground, sun and camera.
fn base() -> Result<(RenderScene, Parts), String> {
    let root = crate::depth_render_scenes::cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let model = bundle
        .load_model("models/tree.glb")
        .map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.4, 0.7, 0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.6;
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.7, 0.0),
        target: Vec3::new(0.0, 1.5, -20.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.32, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(400.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    let parts = model.add_to(&mut scene, &[]).parts;
    if !parts
        .iter()
        .any(|&(m, _, _)| scene.mesh_clusters(m).is_some())
    {
        return Err("the cooked tree has no clusters".into());
    }
    Ok((scene, parts))
}

/// The forest's trees: 11 across (5 m apart), 12 deep (10 m apart from
/// 6 m), each turned its own way.
fn plant_forest(scene: &mut RenderScene, parts: &[(MeshHandle, MaterialHandle, Mat4)]) {
    for row in 0..12 {
        for col in 0..11 {
            let at = Mat4::from_rotation_translation(
                Quat::from_rotation_y(0.7 * (row * 11 + col) as f32),
                Vec3::new(-25.0 + col as f32 * 5.0, 0.0, -6.0 - row as f32 * 10.0),
            );
            for &(mesh, material, local) in parts {
                scene.add_instance(mesh, material, at * local);
            }
        }
    }
}

/// `meshlet-forest` (module docs).
pub fn forest_scene() -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    scene.lod_debug = true;
    Ok(scene)
}

/// `meshlet-occlusion` (module docs; render it three frames).
pub fn occlusion_scene() -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    let wall = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.55, 0.5, 0.45),
        roughness: 0.9,
        ..Default::default()
    });
    // 20 m wide and 8 m high, 4 m ahead, its right edge a little right of
    // the line of sight.
    let cube = scene.add_mesh(shapes::cuboid(Vec3::new(20.0, 8.0, 0.4)));
    scene.add_instance(
        cube,
        wall,
        Mat4::from_translation(Vec3::new(-8.6, 4.0, -4.0)),
    );
    Ok(scene)
}

/// `meshlet-near` (module docs): one tree 6 m away and a little left,
/// through the meshlet path when `meshlets`.
pub fn near_scene(meshlets: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    let at =
        Mat4::from_rotation_translation(Quat::from_rotation_y(0.4), Vec3::new(-0.8, 0.0, -6.0));
    for &(mesh, material, local) in &parts {
        scene.add_instance(mesh, material, at * local);
    }
    scene.meshlets = meshlets;
    Ok(scene)
}

/// `meshlet-shadows` (module docs), through the meshlet path when
/// `meshlets`.
pub fn shadow_scene(meshlets: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    let sun = Vec3::new(-0.8, 0.35, 0.3);
    scene.light.direction_to_light = sun;
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.3;
    for (k, x) in [-8.0f32, -4.0, 0.0, 4.0, 8.0].into_iter().enumerate() {
        let at = Mat4::from_rotation_translation(
            Quat::from_rotation_y(0.9 * k as f32),
            Vec3::new(x, 0.0, -14.0 - 2.0 * k as f32),
        );
        for &(mesh, material, local) in &parts {
            scene.add_instance(mesh, material, at * local);
        }
    }
    // A spot over the middle tree, its shadow cast forward on the ground.
    scene.lights.push(Light {
        kind: LightKind::Spot {
            direction: Vec3::new(0.0, -1.0, 0.35).normalize(),
            angle_degrees: 40.0,
            penumbra: 0.3,
        },
        position: Vec3::new(0.0, 9.0, -21.0),
        color: Vec3::new(1.0, 0.85, 0.7),
        intensity: 120.0,
        range: 25.0,
        cast_shadows: true,
    });
    scene.camera.position = Vec3::new(0.0, 9.0, 2.0);
    scene.camera.target = Vec3::new(0.0, 0.0, -18.0);
    scene.meshlets = meshlets;
    Ok(scene)
}
