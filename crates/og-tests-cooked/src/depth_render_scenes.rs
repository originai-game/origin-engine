//! The depth wave's render-lod scene that cooks (docs/design.md §10.6): mesh
//! levels of detail picked by projected error (`lod-levels`); og-tests'
//! depth_render_scenes has the probe and SSR scenes beside it.
//!
//! - `lod-levels`: the kit's pine (9,037 triangles), cooked with the
//!   default `lods 1 0.5 0.25`, lining a farm track as an avenue 5 m tall
//!   every 15 m out to 330 m, seen from eye height. Each copy draws the
//!   level `og_render::lod::select` picks for its error projected to the
//!   720 px view; the three at 15, 210 and 315 m (left, right, left) draw
//!   levels 0, 1 and 2. The inset in the top right corner shows those
//!   three side by side at one size, tinted as `debugView lod` tints them:
//!   green the full mesh, yellow level 1, orange level 2. Dry-stone walls,
//!   a watchtower and an arch at the track's end, a wood of the kit's
//!   pines and its hills beyond, under the fair afternoon's sky.

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::Bundle;
use og_render::{Camera, Exposure, InstanceFx, RenderScene, lod};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The project holding the kit's pine (`assets/kit/pine.glb`), cooked
/// once per process.
pub(crate) fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-lod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        // The kit's pine (og-tests/assets/kit/README.md), cooked as any
        // model is: clusters for the meshlet path and its mesh levels; its
        // foot moved to the origin, where the wind's bend roots a model.
        let pine = rooted_glb(&crate::scenes::asset("kit/pine.glb")?)?;
        std::fs::write(models.join("pine.glb"), pine).map_err(|e| format!("pine.glb: {e}"))?;
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

/// A one-mesh GLB `bytes` with its first primitive's positions raised so
/// the lowest stands at y = 0 (the kit's pine is modelled about its
/// middle; og.Model.sway and the meshlet foliage bend a model from its
/// origin). Float positions only; the bounds are written again.
fn rooted_glb(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let u32_at = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap()) as usize;
    let json_len = u32_at(12);
    let mut json: serde_json::Value =
        serde_json::from_slice(&bytes[20..20 + json_len]).map_err(|e| e.to_string())?;
    let bin_at = 20 + json_len;
    let mut bin = bytes[bin_at + 8..bin_at + 8 + u32_at(bin_at)].to_vec();
    let pos = json["meshes"][0]["primitives"][0]["attributes"]["POSITION"]
        .as_u64()
        .ok_or("no POSITION")? as usize;
    let acc = json["accessors"][pos].clone();
    if acc["componentType"] != 5126 {
        return Err("positions are not floats".into());
    }
    let view = &json["bufferViews"][acc["bufferView"].as_u64().unwrap_or(0) as usize];
    let stride = view["byteStride"].as_u64().unwrap_or(12) as usize;
    let first = view["byteOffset"].as_u64().unwrap_or(0) as usize
        + acc["byteOffset"].as_u64().unwrap_or(0) as usize;
    let lift = -acc["min"][1].as_f64().ok_or("no bounds")? as f32;
    for k in 0..acc["count"].as_u64().unwrap_or(0) as usize {
        let o = first + k * stride + 4;
        let y = f32::from_le_bytes(bin[o..o + 4].try_into().unwrap()) + lift;
        bin[o..o + 4].copy_from_slice(&y.to_le_bytes());
    }
    for b in ["min", "max"] {
        let y = json["accessors"][pos][b][1].as_f64().unwrap_or(0.0) + lift as f64;
        json["accessors"][pos][b][1] = serde_json::json!(y);
    }
    let mut text = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    text.resize(text.len().next_multiple_of(4), b' ');
    let total = 12 + 8 + text.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&bytes[0..8]);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    Ok(out)
}

/// The row's three pines' distances along the track (m) and the camera
/// of `lod-levels`.
pub const LOD_DISTANCES: [f32; 3] = [15.0, 210.0, 315.0];

/// How tall the pines of `lod-levels` stand (m); where its camera stands
/// along the track, at the flat ground's near edge (the kit's hills rise
/// past 190 m from the origin), and where the inset's strip of the levels
/// stands, out of the camera's sight behind it.
const PINE_HEIGHT: f32 = 5.0;
const CAMERA_Z: f32 = 160.0;
const STRIP_Z: f32 = 172.0;

fn lod_camera() -> Camera {
    Camera {
        position: Vec3::new(0.0, 1.7, CAMERA_Z),
        target: Vec3::new(0.0, 4.3, CAMERA_Z - 40.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    }
}

/// `lod-levels` (module docs), with the level each of the row's three
/// pines draws and their triangles at those levels and at level 0.
pub type LodScene = (RenderScene, [usize; 3], (usize, usize));

/// `lod-levels`: [`LodScene`].
pub fn lod_scene() -> Result<LodScene, String> {
    let root = cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let (model, textures) = bundle
        .load_model_with_textures("models/pine.glb")
        .map_err(|e| e.to_string())?;
    use og_tests::kit;
    let mut scene = RenderScene::new();
    // A pine avenue down a farm track in open country: the kit's grass out
    // to its hills, a stone wall and a wood of the kit's pines closing the
    // view past the avenue's end, under the fair afternoon's sky with the
    // sun behind the left shoulder.
    let meadow = kit::surface_tinted(&mut scene, "grass", [0.78, 1.0, 0.62]);
    let land = scene.add_mesh(kit::slab(
        glam::Vec2::splat(-600.0),
        glam::Vec2::splat(600.0),
        0.0,
        4.0,
    ));
    scene.add_instance(land, meadow, Mat4::IDENTITY);
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.95, 0.88, 0.74]);
    let track = scene.add_mesh(kit::slab(
        glam::Vec2::new(-1.2, -172.0),
        glam::Vec2::new(1.2, 165.0),
        0.02,
        2.5,
    ));
    scene.add_instance(track, setts, Mat4::IDENTITY);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-60.0, -0.2, -176.0),
        Vec3::new(-2.0, 1.1, -175.3),
        2.5,
    ));
    scene.add_instance(wall, stone, Mat4::IDENTITY);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(2.0, -0.2, -176.0),
        Vec3::new(60.0, 1.1, -175.3),
        2.5,
    ));
    scene.add_instance(wall, stone, Mat4::IDENTITY);
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..260i32 {
        let h = |i: i32| {
            let mut v = (k as u32).wrapping_mul(0x8da6_b343) ^ (i as u32).wrapping_mul(0xd816_3841);
            v = (v ^ (v >> 13)).wrapping_mul(0x85eb_ca6b);
            ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
        };
        let (x, z) = (-200.0 + 400.0 * h(1), -182.0 - 60.0 * h(2));
        if x.abs() < 3.0 {
            continue;
        }
        pine.place(
            &mut scene,
            // Up the hills' first rise where the wood reaches it.
            Vec3::new(x, kit::hills_height(x, z, None).max(0.0) - 0.2, z),
            6.3 * h(3),
            11.0 + 8.0 * h(4),
        );
    }
    // Dry-stone field walls behind the avenue on either side, broken for
    // field gates; boulders cleared to the verges; a watchtower over the
    // fields and an arch where the track meets the far wall.
    let mut dyke = |x0: f32, x1: f32, z0: f32, z1: f32| {
        let m = scene.add_mesh(kit::block(
            Vec3::new(x0, -0.2, z1),
            Vec3::new(x1, 0.9, z0),
            1.6,
        ));
        scene.add_instance(m, stone, Mat4::IDENTITY);
    };
    for side in [-1.0f32, 1.0] {
        let x = 9.0 * side;
        let (a, b) = (x.min(x + 0.6 * side), x.max(x + 0.6 * side));
        dyke(a, b, CAMERA_Z - 6.0, CAMERA_Z - 70.0);
        dyke(a, b, CAMERA_Z - 78.0, CAMERA_Z - 190.0);
        dyke(a, b, CAMERA_Z - 198.0, -175.3);
    }
    let boulder = kit::prop(&mut scene, "boulder");
    for k in 0..30u32 {
        let h = |i: u32| {
            let mut v = k.wrapping_mul(0x27d4_eb2d) ^ i.wrapping_mul(0x1656_67b1);
            v = (v ^ (v >> 15)).wrapping_mul(0x85eb_ca6b);
            ((v ^ (v >> 16)) & 0xffff) as f32 / 65536.0
        };
        let side = if h(1) < 0.5 { -1.0 } else { 1.0 };
        let (x, z) = (
            side * (5.6 + 2.4 * h(2)),
            CAMERA_Z - 7.0 - 320.0 * h(3) * h(3),
        );
        boulder.place(
            &mut scene,
            Vec3::new(x, -0.15, z),
            6.3 * h(4),
            0.5 + 0.9 * h(5),
        );
    }
    kit::prop(&mut scene, "watchtower").place(
        &mut scene,
        Vec3::new(-62.0, -0.3, CAMERA_Z - 150.0),
        0.9,
        16.0,
    );
    kit::prop(&mut scene, "arch").place(&mut scene, Vec3::new(0.0, -0.1, -175.6), 0.0, 6.5);
    kit::hills(&mut scene, None);
    og_tests::terrain_scenes::afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(-0.4, 0.7, 0.5).normalize();
    scene.lighting.light.max_shadow_distance = 350.0;
    og_tests::terrain_scenes::fair_sky(&mut scene, 8.6);
    scene.frame.tick = 0;
    scene.frame.camera = Camera {
        far: 4000.0,
        exposure: Exposure::linear(0.9),
        ..lod_camera()
    };
    let base = scene.mesh_count();
    let parts = model.add_to(&mut scene, &textures).parts;
    let levels = model.add_lod_levels(&mut scene);
    let prim_of =
        |mesh| (0..model.primitives.len()).find(|&p| scene.mesh_handle(base + p) == Some(mesh));
    let part_levels: Vec<Vec<(og_render::MeshHandle, f32)>> = parts
        .iter()
        .map(|&(m, _, _)| prim_of(m).map(|p| levels[p].clone()).unwrap_or_default())
        .collect();
    // The model's error per level: the largest of its parts' (the parts of
    // the pine share one node, unscaled).
    let n = part_levels.iter().map(|l| l.len()).max().unwrap_or(0);
    let errors: Vec<f32> = std::iter::once(0.0)
        .chain((0..n).map(|k| {
            part_levels
                .iter()
                .filter_map(|l| l.get(k).or(l.last()))
                .map(|x| x.1)
                .fold(0.0, f32::max)
        }))
        .collect();
    // The model's bounding sphere, over its primitives' level-0 vertices.
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &model.primitives {
        for v in &p.vertices {
            lo = lo.min(Vec3::from(v.position));
            hi = hi.max(Vec3::from(v.position));
        }
    }
    let (center, radius) = ((lo + hi) * 0.5, (hi - lo).length() * 0.5);
    let tris = |level: usize| -> usize {
        model
            .primitives
            .iter()
            .map(|p| p.triangle_count(level.min(p.lods.len() - 1)))
            .sum()
    };
    // The kit's pine stands PINE_HEIGHT tall wherever it is placed.
    let scale = PINE_HEIGHT / (hi.y - lo.y);
    let mut chosen = [0; 3];
    let mut drawn = 0;
    let place = |scene: &mut RenderScene, at: Mat4, level: usize, fx: InstanceFx| {
        for (k, &(mesh, material, local)) in parts.iter().enumerate() {
            let mesh = match part_levels.get(k) {
                Some(l) if level > 0 && !l.is_empty() => l[(level - 1).min(l.len() - 1)].0,
                _ => mesh,
            };
            scene.objects.instances.push(og_render::Instance {
                mesh,
                material,
                transform: at * local,
                material_layer: None,
                fx,
                cast_shadows: true,
                receive_shadows: true,
                sway: 0.0,
            });
        }
    };
    // The avenue: a pine each side every 15 m out to 330 m, each drawing
    // the level `lod::select` picks for it; the row's three are among
    // them, left, right and left.
    for k in 1..=22 {
        for side in [-1.0f32, 1.0] {
            let d = 15.0 * k as f32;
            let row = LOD_DISTANCES
                .iter()
                .enumerate()
                .position(|(i, &r)| r == d && side == if i % 2 == 0 { -1.0 } else { 1.0 });
            let yaw = 2.4 * k as f32 + 1.3 * side;
            // A little variety in height; the row's three at the exact one.
            let s = match row {
                Some(_) => scale,
                None => scale * (0.92 + 0.08 * yaw.sin()),
            };
            let at = Mat4::from_scale_rotation_translation(
                Vec3::splat(s),
                Quat::from_rotation_y(yaw),
                Vec3::new(4.0 * side, 0.0, CAMERA_Z - d),
            );
            // The entity's scale folded in (`lod::select`).
            let per_unit = s * lod::pixels_per_unit(
                &scene.frame.camera,
                at.transform_point3(center),
                radius * s,
                720.0,
            );
            let level = lod::select(&errors, per_unit, 1.0, None);
            place(&mut scene, at, level, InstanceFx::NONE);
            let Some(i) = row else { continue };
            chosen[i] = level;
            drawn += tris(level);
            // The same level beside its fellows in the editor's strip, tinted
            // as `debugView lod` tints it, out of sight behind the camera.
            let strip = Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                Quat::from_rotation_y(0.6),
                Vec3::new(4.2 * (i as f32 - 1.0), 0.0, STRIP_Z),
            );
            let fx = InstanceFx {
                flash: 0.45,
                flash_color: lod::debug_color(level),
                ..InstanceFx::NONE
            };
            place(&mut scene, strip, level, fx);
        }
    }
    // The strip seen from the row's side, in the corner of the frame:
    // levels 0, 1 and 2 left to right, each at the size of the nearest.
    scene.frame.views.push(og_render::View {
        camera: Camera {
            position: Vec3::new(0.0, 0.5 * PINE_HEIGHT, STRIP_Z + 10.0),
            target: Vec3::new(0.0, 0.45 * PINE_HEIGHT, STRIP_Z),
            ..scene.frame.camera
        },
        target: og_render::ViewTarget::Viewport(crate::meshlet_scenes::INSET),
        ..Default::default()
    });
    Ok((scene, chosen, (drawn, 3 * tris(0))))
}
