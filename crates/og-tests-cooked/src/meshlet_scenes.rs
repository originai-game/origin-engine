//! The meshlet path's scenes (R1 batch 2, og-render's `meshlet` and
//! `meshlet_gpu`): the kit's pine (og-tests/assets/kit/README.md; 9,037
//! triangles, cooked with the default `lods 1 0.5 0.25` and cut into
//! clusters) planted as a plantation in an orchard's rolling country, its
//! instances drawn at level 0's mesh so the GPU picks each one's level and
//! culls its clusters. Where a scene shows the levels, it shows them in an
//! inset in the top right corner ([`INSET`], `View::lod_debug`), as an
//! editor's overlay beside the untinted shot.
//!
//! - `meshlet-forest`: 132 pines 8 to 12 m tall, 11 rows across and 12
//!   deep from 6 m to 116 m, seen from the lane's end at the walled plot's
//!   gate; the inset looks down the same rows from 30 m up, its near rows
//!   green (the full mesh) and its far rows yellow (level 1), no tree
//!   missing or cut open (a cluster culled wrongly shows as a hole in a
//!   crown).
//! - `meshlet-occlusion`: the plantation behind its old curtain wall, run
//!   at a slant across the left of the view to a broken end, three frames
//!   so the second and third cull in two phases (the clusters visible
//!   the frame before, drawn into the occluders' depth; the rest against
//!   the pyramid built again from it): the crowns over the wall and the
//!   rows past its broken end show whole, as `occlusion_scene(false)`
//!   draws them through the regular path.
//! - `meshlet-near`: one pine 7 m tall, 8 m away, drawn through the
//!   meshlet path; `near_scene(false)` draws it through the regular path,
//!   and the two frames match (the level is 0 at that distance).
//! - `meshlet-shadows` (batch 3): five pines under the evening's low sun
//!   and a shadowed spot light, seen from above so their long shadows lie
//!   on the grass: the casters go through the meshlet path's shadow views
//!   (each cascade's and the spot's face), and `shadow_scene(false)` draws
//!   the same frame through the regular path; the two match.
//! - `meshlet-views`: the plantation from the lawn, a chrome gazing ball
//!   before its first row reflecting it through a probe captured from the
//!   ball's centre, and a view from a watchtower's height in the inset,
//!   tinted by the levels its own culling picked. `views_scene(false, 0)`
//!   and `views_scene(true, 0)` draw it with every level 0 (bias 0)
//!   through the regular and the meshlet path; the two match.
//! - `meshlet-far` (batch 4): the plantation from 40 m up and 54 m behind
//!   the lane's end, so nearly every cluster covers a few pixels and goes
//!   through the visibility buffer (rasterised in compute, shaded by each
//!   batch's resolve): 132 whole pines with their shadows, no holes,
//!   specks or seams between a tree's compute and hardware clusters.
//!   `far_scene(false)` draws it with every cluster by the hardware; the
//!   two match.
//! - `meshlet-sway` (foliage transport): one pine given foliage (a pivot
//!   at its root, stiffness 0.2, each vertex its own anchor) in a strong
//!   wind blowing towards +x (screen right): the crown leans right of its
//!   trunk's foot and its shadow leans with it, the trunk's base stays
//!   where it was. `sway_scene(0.0, ..)` keeps it still in the same wind
//!   and matches the tree with no foliage. `sway_far_scene(vbuffer)`: the
//!   plantation from above, every tree swaying, through the visibility
//!   buffer and through the hardware; the two match (the compute
//!   rasteriser sways what the hardware does).

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Camera, DirectionalLight, Exposure, Light, LightKind, MaterialHandle, MeshHandle,
    ReflectionProbe, RenderScene, StandardMaterial, View, ViewTarget, shapes,
};

/// A model's parts: each mesh, its material and its place in the model.
type Parts = Vec<(MeshHandle, MaterialHandle, Mat4)>;

/// The tree's parts in a new scene with its ground, sun and camera.
fn base() -> Result<(RenderScene, Parts), String> {
    let root = crate::depth_render_scenes::cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let (model, textures) = bundle
        .load_model_with_textures("models/pine.glb")
        .map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.4, 0.7, 0.5);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.intensity = 0.6;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.7, 0.0),
        target: Vec3::new(0.0, 1.5, -20.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene.frame.camera.far = 4000.0;
    // An orchard in rolling country: a terrain of the kit's grass (its
    // repeat broken by anti-tiling), level within 150 m and rising into
    // the kit's hills (just under them, where they take over), under the
    // fair afternoon's physical sky and cumulus. The hills are the
    // scene's first instance; the tree's parts follow.
    orchard_ground(&mut scene);
    og_tests::kit::hills(&mut scene, None);
    sky(&mut scene, 8.6);
    let parts = model.add_to(&mut scene, &textures).parts;
    if !parts
        .iter()
        .any(|&(m, _, _)| scene.mesh_clusters(m).is_some())
    {
        return Err("the cooked pine has no clusters".into());
    }
    Ok((scene, parts))
}

/// The orchard's ground: 512 m of terrain, level within 150 m of the
/// middle, then easing into the kit's hills 0.6 m under them.
fn orchard_ground(scene: &mut RenderScene) {
    use og_render::{Terrain, TerrainData, TerrainLayer};
    let (n, size) = (257u32, 512.0f32);
    let cell = size / (n - 1) as f32;
    let heights: Vec<f32> = (0..n * n)
        .map(|i| {
            let (x, z) = (
                (i % n) as f32 * cell - size * 0.5,
                (i / n) as f32 * cell - size * 0.5,
            );
            let t = ((x.hypot(z) - 150.0) / 50.0).clamp(0.0, 1.0);
            let t = t * t * (3.0 - 2.0 * t);
            t * (og_tests::kit::hills_height(x, z, None) - 0.6)
        })
        .collect();
    let h = scene.add_terrain(TerrainData {
        heights: std::sync::Arc::from(heights),
        ..TerrainData::flat(glam::Vec2::splat(size), n, n)
    });
    let (grass, grass_normal) = og_tests::kit::layer_maps(scene, "grass");
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(grass),
            normal: Some(grass_normal),
            color: Vec3::new(0.72, 1.05, 0.5),
            roughness: 0.95,
            tile: 3.5,
            anti_tiling: 0.6,
            macro_variation: 0.6,
            ..Default::default()
        }],
        fade_in: false,
        ..Terrain::new(h)
    });
}

/// The farm round the orchard, for the views that see past it: a dry
/// stone wall round the forest's plot (open at the camera's end), stands
/// of the kit's pines beyond it and a watchtower on the far side. Its
/// pines are plain props, never tinted by level. Not in `meshlet-near`
/// (whose test moves every instance after the first).
fn farmland(scene: &mut RenderScene) {
    use og_tests::kit;
    let stone = kit::surface_tinted(scene, "cobbles", [0.95, 0.9, 0.78]);
    let mut wall = |min: Vec3, max: Vec3| {
        let m = scene.add_mesh(kit::block(min, max, 1.6));
        scene.add_instance(m, stone, Mat4::IDENTITY);
    };
    let (x0, x1, z0, z1, h) = (-32.0f32, 32.0f32, 0.0f32, -124.0f32, 1.2f32);
    wall(Vec3::new(x0, -0.2, z1), Vec3::new(x1, h, z1 + 0.7));
    wall(Vec3::new(x0, -0.2, z1), Vec3::new(x0 + 0.7, h, z0));
    wall(Vec3::new(x1 - 0.7, -0.2, z1), Vec3::new(x1, h, z0));
    wall(Vec3::new(x0, -0.2, z0 - 0.7), Vec3::new(-4.0, h, z0));
    wall(Vec3::new(4.0, -0.2, z0 - 0.7), Vec3::new(x1, h, z0));
    let hash = |k: u32, i: u32| {
        let mut v = k.wrapping_mul(0x8da6_b343) ^ i.wrapping_mul(0xd816_3841) ^ 0x3c1;
        v = (v ^ (v >> 13)).wrapping_mul(0x85eb_ca6b);
        ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
    };
    let pine = kit::prop(scene, "pine");
    for k in 0..900u32 {
        let (x, z) = (-170.0 + 340.0 * hash(k, 1), 60.0 - 230.0 * hash(k, 2));
        // In stands, clear of the walled plot and the lane to its gate.
        let stand = (x * 0.045).sin() + (z * 0.06 + 1.3).cos();
        if stand < 0.3 || (x.abs() < 44.0 && z > -136.0) {
            continue;
        }
        pine.place(
            scene,
            Vec3::new(x, -0.2, z),
            6.3 * hash(k, 3),
            10.0 + 9.0 * hash(k, 4),
        );
    }
    kit::prop(scene, "watchtower").place(scene, Vec3::new(-54.0, -0.3, -70.0), 0.6, 17.0);
}

/// The fair afternoon's physical sky over the scene's sun as it stands,
/// its light the colour Earth's air gives it at `hours`, the clouds where
/// their shadows leave the orchard in the sun.
fn sky(scene: &mut RenderScene, hours: f32) {
    og_tests::terrain_scenes::fair_sky(scene, hours);
    scene.frame.tick = 0;
}

/// The cooked pine's height in its own units (its foot at the origin).
const PINE_UNITS: f32 = 1.898;

/// A hash of `k` and `i` to 0..1 (the plantation's jitter).
fn hash(k: u32, i: u32) -> f32 {
    let mut v = k.wrapping_mul(0x8da6_b343) ^ i.wrapping_mul(0xd816_3841) ^ 0x3c1;
    v = (v ^ (v >> 13)).wrapping_mul(0x85eb_ca6b);
    ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
}

/// A pine standing at (`x`, `z`), turned `yaw`, `height` metres tall.
fn pine(x: f32, z: f32, yaw: f32, height: f32) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        Vec3::splat(height / PINE_UNITS),
        Quat::from_rotation_y(yaw),
        Vec3::new(x, 0.0, z),
    )
}

fn place(scene: &mut RenderScene, parts: &[(MeshHandle, MaterialHandle, Mat4)], at: Mat4) {
    for &(mesh, material, local) in parts {
        scene.add_instance(mesh, material, at * local);
    }
}

/// The plantation: 11 rows across (5 m apart), 12 deep (10 m apart from
/// 6 m), each pine a little off its mark, turned its own way and 8 to
/// 12 m tall.
fn plant_forest(scene: &mut RenderScene, parts: &[(MeshHandle, MaterialHandle, Mat4)]) {
    for row in 0..12u32 {
        for col in 0..11u32 {
            let k = row * 11 + col;
            let at = pine(
                -25.0 + col as f32 * 5.0 + 2.0 * hash(k, 1) - 1.0,
                -6.0 - row as f32 * 10.0 + 3.0 * hash(k, 2) - 1.5,
                0.7 * k as f32,
                8.0 + 4.0 * hash(k, 3),
            );
            place(scene, parts, at);
        }
    }
}

/// The level-of-detail inset: a second view small in the top right
/// corner, its clustered pines tinted by the level it picked
/// (`View::lod_debug`): an editor's overlay beside the untinted shot.
pub const INSET: [f32; 4] = [0.6, 0.04, 0.37, 0.37];

fn lod_inset(scene: &mut RenderScene, camera: Camera) {
    scene.frame.views.push(View {
        camera,
        target: ViewTarget::Viewport(INSET),
        lod_debug: true,
        ..Default::default()
    });
}

/// `meshlet-forest` (module docs).
pub fn forest_scene() -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    farmland(&mut scene);
    // From the lane's end at the plot's gate, a little raised: the rows
    // run away from the camera to the far wall.
    scene.frame.camera.position = Vec3::new(1.0, 4.5, 16.0);
    scene.frame.camera.target = Vec3::new(-1.0, 3.0, -60.0);
    // The inset looks down the same rows from 30 m up, so every row
    // shows: the near ones low in it, the far ones high.
    let camera = Camera {
        position: Vec3::new(1.0, 30.0, 16.0),
        target: Vec3::new(-1.0, 0.0, -70.0),
        ..scene.frame.camera
    };
    lod_inset(&mut scene, camera);
    Ok(scene)
}

/// `meshlet-occlusion` (module docs; render it three frames), through
/// the meshlet path when `meshlets`.
pub fn occlusion_scene(meshlets: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    // The plantation's old curtain wall, 6 m high and 1.2 m thick, run
    // from the left of the view across it at a slant to its broken end a
    // little right of the line of sight: buttresses on its face, merlons
    // along its walk, fallen blocks at its foot. It hides the lower
    // two thirds of the rows behind it on the left; the crowns stand over.
    use og_tests::kit;
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let turn =
        Mat4::from_rotation_translation(Quat::from_rotation_y(-0.42), Vec3::new(0.9, 0.0, -9.0));
    let mut block = |min: Vec3, max: Vec3| {
        let m = scene.add_mesh(kit::block(min, max, 1.8));
        scene.add_instance(m, stone, turn);
    };
    block(Vec3::new(-22.0, -0.3, -0.6), Vec3::new(0.0, 6.0, 0.6));
    for k in 0..9 {
        let x = -21.5 + k as f32 * 2.4;
        block(Vec3::new(x, 6.0, -0.6), Vec3::new(x + 1.3, 7.1, 0.6));
    }
    for x in [-17.0f32, -9.5] {
        block(Vec3::new(x, -0.3, 0.6), Vec3::new(x + 1.6, 4.6, 2.0));
    }
    // The broken end: steps down in courses.
    block(Vec3::new(0.0, -0.3, -0.6), Vec3::new(1.2, 4.2, 0.6));
    block(Vec3::new(1.2, -0.3, -0.6), Vec3::new(2.3, 2.1, 0.6));
    let boulder = kit::prop(&mut scene, "boulder");
    let foot = |x: f32, z: f32| turn.transform_point3(Vec3::new(x, -0.1, z));
    boulder.place(&mut scene, foot(3.4, 1.4), 0.4, 0.9);
    boulder.place(&mut scene, foot(-4.0, 2.2), 1.9, 0.6);
    farmland(&mut scene);
    scene.frame.camera.position = Vec3::new(1.5, 2.2, 1.0);
    scene.frame.camera.target = Vec3::new(-1.0, 3.2, -20.0);
    scene.objects.meshlets = meshlets;
    Ok(scene)
}

/// `meshlet-near` (module docs): one pine 8 m away and a little left,
/// through the meshlet path when `meshlets`.
pub fn near_scene(meshlets: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    place(&mut scene, &parts, pine(-0.8, -8.0, 0.4, 7.0));
    scene.frame.camera.target = Vec3::new(0.0, 3.0, -8.0);
    scene.objects.meshlets = meshlets;
    Ok(scene)
}

/// `meshlet-shadows` (module docs), through the meshlet path when
/// `meshlets`.
pub fn shadow_scene(meshlets: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    // Evening: the sun low in the west, its long shadows over the grass.
    scene.lighting.light.direction_to_light = Vec3::new(-0.8, 0.35, 0.3);
    sky(&mut scene, 7.6);
    farmland(&mut scene);
    // The low sun's light is weaker: open the exposure as a game's
    // adaptation would.
    scene.frame.camera.exposure = Exposure::linear(1.4);
    for (k, x) in [-8.0f32, -4.0, 0.0, 4.0, 8.0].into_iter().enumerate() {
        let at = pine(
            x,
            -14.0 - 2.0 * k as f32,
            0.9 * k as f32,
            7.0 + hash(k as u32, 5) * 3.0,
        );
        place(&mut scene, &parts, at);
    }
    // A spot over the middle tree, its shadow cast forward on the ground.
    scene.lighting.lights.push(Light {
        kind: LightKind::Spot {
            direction: Vec3::new(0.0, -1.0, 0.35).normalize(),
            angle_degrees: 40.0,
            penumbra: 0.3,
        },
        position: Vec3::new(0.0, 9.0, -21.0),
        color: Vec3::new(1.0, 0.85, 0.7),
        intensity: 120.0,
        range: 25.0,
        radius: 0.0,
        cast_shadows: true,
    });
    scene.frame.camera.position = Vec3::new(0.0, 9.0, 2.0);
    scene.frame.camera.target = Vec3::new(0.0, 0.0, -18.0);
    scene.objects.meshlets = meshlets;
    Ok(scene)
}

/// `meshlet-views` (module docs), through the meshlet path when
/// `meshlets`, at LOD bias `bias` (0: every level 0), tinted by level when
/// `bias` is positive.
pub fn views_scene(meshlets: bool, bias: f32) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    let chrome = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.95),
        metalness: 1.0,
        roughness: 0.05,
        ..Default::default()
    });
    let ball = scene.add_mesh(shapes::sphere(32, 16));
    // A gazing ball on the lawn before the first row, its probe captured
    // from its centre: the capture sees out through the ball's back faces,
    // so the ball reflects the plantation, not itself.
    let centre = Vec3::new(-1.3, 0.8, -4.5);
    scene.add_instance(
        ball,
        chrome,
        // 1.6 m across the unit sphere's 1 (radius 0.5).
        Mat4::from_scale_rotation_translation(Vec3::splat(1.6), Quat::IDENTITY, centre),
    );
    scene.lighting.reflection_probes.push(ReflectionProbe::new(
        1,
        centre,
        Vec3::new(14.0, 6.0, 14.0),
    ));
    farmland(&mut scene);
    // The plantation from a watchtower's height, an inset in the top right
    // corner, tinted by the levels its own culling picked.
    scene.frame.views.push(View {
        camera: Camera {
            position: Vec3::new(0.0, 70.0, -30.0),
            target: Vec3::new(0.0, 0.0, -62.0),
            exposure: Exposure::linear(0.8),
            far: 4000.0,
            ..Default::default()
        },
        target: ViewTarget::Viewport(INSET),
        lod_debug: bias > 0.0,
        ..Default::default()
    });
    scene.objects.meshlets = meshlets;
    scene.objects.lod_bias = bias;
    Ok(scene)
}

/// `meshlet-far` (module docs), its small clusters through the visibility
/// buffer when `visibility_buffer`.
pub fn far_scene(visibility_buffer: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    plant_forest(&mut scene, &parts);
    farmland(&mut scene);
    scene.frame.camera.position = Vec3::new(0.0, 40.0, 54.0);
    scene.frame.camera.target = Vec3::new(0.0, 12.0, -50.0);
    scene.objects.visibility_buffer = visibility_buffer;
    Ok(scene)
}

/// Each part of the tree's foliage: one pivot at the root, bending
/// easily, every vertex its own anchor without flutter (bark).
fn give_foliage(scene: &mut RenderScene, parts: &[(MeshHandle, MaterialHandle, Mat4)]) {
    let top = parts
        .iter()
        .flat_map(|&(m, _, local)| {
            let mesh = scene.mesh(m).expect("live").clone();
            mesh.vertices
                .iter()
                .map(move |v| local.transform_point3(Vec3::from(v.position)).y)
                .collect::<Vec<_>>()
        })
        .fold(0.0f32, f32::max);
    let mut seen = std::collections::HashSet::new();
    for &(m, _, _) in parts {
        if !seen.insert(m) {
            continue;
        }
        let vertices = scene
            .mesh(m)
            .expect("live")
            .vertices
            .iter()
            .map(|v| og_render::foliage::FoliageVertex {
                branch: [0.0, 0.0, 0.0, 0.2],
                leaf: [v.position[0], v.position[1], v.position[2], 0.0],
            })
            .collect();
        let f = og_render::meshlet::MeshFoliage {
            vertices,
            top,
            canopy_center: Vec3::new(0.0, top * 0.7, 0.0),
            leaves: false,
            ..Default::default()
        };
        assert!(scene.set_mesh_foliage(m, Some(f)), "foliage refused");
    }
}

/// The strong wind `meshlet-sway` blows (towards +x), at a tick past 0.
fn windy(scene: &mut RenderScene) {
    scene.atmosphere.wind = Some(og_render::Wind {
        strength: 0.5,
        direction: Vec3::X,
        speed: 3.0,
        gust: 0.3,
        ..og_render::Wind::default()
    });
    scene.frame.tick = 90;
    scene.frame.tick_rate = 60.0;
}

/// `meshlet-sway` (module docs): the near tree seen from the side, its
/// instances swaying by `sway`; without `foliage` the tree has none.
pub fn sway_scene(sway: f32, foliage: bool) -> Result<RenderScene, String> {
    let (mut scene, parts) = base()?;
    let at = pine(0.0, -9.0, 0.3, 7.0);
    for &(mesh, material, local) in &parts {
        scene.add_instance(mesh, material, at * local);
        scene.objects.instances.last_mut().expect("added").sway = sway;
    }
    if foliage {
        give_foliage(&mut scene, &parts);
    }
    windy(&mut scene);
    scene.frame.camera.position = Vec3::new(0.0, 2.5, 0.0);
    scene.frame.camera.target = Vec3::new(0.0, 3.4, -9.0);
    Ok(scene)
}

/// `sway_far_scene` (module docs): `meshlet-far` with every tree swaying,
/// its small clusters through the visibility buffer when `vbuffer`.
pub fn sway_far_scene(vbuffer: bool) -> Result<RenderScene, String> {
    let mut scene = far_scene(vbuffer)?;
    let parts: Parts = scene
        .objects
        .instances
        .iter()
        .filter(|i| scene.mesh_clusters(i.mesh).is_some())
        .map(|i| (i.mesh, i.material, Mat4::IDENTITY))
        .collect();
    give_foliage(&mut scene, &parts);
    // The trees sway; the land and the hills stand.
    for i in &mut scene.objects.instances {
        if parts.iter().any(|p| p.0 == i.mesh) {
            i.sway = 1.0;
        }
    }
    windy(&mut scene);
    Ok(scene)
}
