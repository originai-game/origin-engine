//! The depth wave's render-lod scene that cooks (docs/design.md §10.6): mesh
//! levels of detail picked by projected error (`lod-levels`); og-tests'
//! depth_render_scenes has the probe and SSR scenes beside it.
//!
//! - `lod-levels`: the fx-showcase tree (3,160 triangles), cooked with the
//!   default `lods 1 0.5 0.25`, at 6, 30 and 70 m along a row seen from
//!   eye height. Each copy draws the level `og_render::lod::select` picks
//!   for its error projected to the 720 px view, tinted as `debugView lod`
//!   tints it: green the full mesh (near), yellow level 1, orange level 2.
//!   They stand by a farm track in a clearing of the kit's grass, a wall
//!   and a wood of its pines past them and its hills beyond, under the
//!   fair afternoon's sky: an editor's level-of-detail view of a scene.

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::Bundle;
use og_render::{Camera, Exposure, InstanceFx, RenderScene, lod};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The project holding the tree (`assets/tree.glb`) and the kit's pine
/// (`assets/kit/pine.glb`), cooked once per process.
pub(crate) fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-lod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        std::fs::write(models.join("tree.glb"), crate::scenes::asset("tree.glb")?)
            .map_err(|e| format!("tree.glb: {e}"))?;
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

/// The trees' distances along the row (m) and the camera of `lod-levels`.
pub const LOD_DISTANCES: [f32; 3] = [6.0, 30.0, 70.0];

fn lod_camera() -> Camera {
    Camera {
        position: Vec3::new(0.0, 1.7, 0.0),
        target: Vec3::new(0.0, 1.5, -20.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    }
}

/// `lod-levels` (module docs), with the level each tree draws and the
/// triangles of the three trees at those levels and at level 0.
pub type LodScene = (RenderScene, [usize; 3], (usize, usize));

/// `lod-levels`: [`LodScene`].
pub fn lod_scene() -> Result<LodScene, String> {
    let root = cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let model = bundle
        .load_model("models/tree.glb")
        .map_err(|e| e.to_string())?;
    use og_tests::kit;
    let mut scene = RenderScene::new();
    // A clearing in an orchard country: the kit's grass out to its hills,
    // a farm track running away beside the row, a stone wall and a wood
    // of the kit's pines closing the view past the farthest tree, under
    // the fair afternoon's sky with the sun behind the left shoulder.
    let meadow = kit::surface_tinted(&mut scene, "grass", [0.78, 1.0, 0.62]);
    let land = scene.add_mesh(kit::slab(
        glam::Vec2::splat(-200.0),
        glam::Vec2::splat(200.0),
        0.0,
        4.0,
    ));
    scene.add_instance(land, meadow, Mat4::IDENTITY);
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.95, 0.88, 0.74]);
    let track = scene.add_mesh(kit::slab(
        glam::Vec2::new(-0.8, -150.0),
        glam::Vec2::new(0.8, 4.0),
        0.02,
        2.5,
    ));
    scene.add_instance(track, setts, Mat4::IDENTITY);
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-40.0, -0.2, -96.0),
        Vec3::new(-2.0, 1.1, -95.3),
        2.5,
    ));
    scene.add_instance(wall, stone, Mat4::IDENTITY);
    let wall = scene.add_mesh(kit::block(
        Vec3::new(2.0, -0.2, -96.0),
        Vec3::new(40.0, 1.1, -95.3),
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
        let (x, z) = (-150.0 + 300.0 * h(1), -100.0 - 70.0 * h(2));
        if x.abs() < 3.0 {
            continue;
        }
        pine.place(
            &mut scene,
            Vec3::new(x, -0.2, z),
            6.3 * h(3),
            11.0 + 8.0 * h(4),
        );
    }
    kit::hills(&mut scene, None);
    og_tests::terrain_scenes::afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(-0.4, 0.7, 0.5).normalize();
    scene.lighting.light.max_shadow_distance = 200.0;
    og_tests::terrain_scenes::fair_sky(&mut scene, 8.6);
    scene.frame.tick = 0;
    scene.frame.camera = Camera {
        far: 4000.0,
        exposure: Exposure::linear(0.9),
        ..lod_camera()
    };
    let base = scene.mesh_count();
    let parts = model.add_to(&mut scene, &[]).parts;
    let levels = model.add_lod_levels(&mut scene);
    let prim_of =
        |mesh| (0..model.primitives.len()).find(|&p| scene.mesh_handle(base + p) == Some(mesh));
    let part_levels: Vec<Vec<(og_render::MeshHandle, f32)>> = parts
        .iter()
        .map(|&(m, _, _)| prim_of(m).map(|p| levels[p].clone()).unwrap_or_default())
        .collect();
    // The model's error per level: the largest of its parts' (the parts of
    // the tree share one node, unscaled).
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
    let mut chosen = [0; 3];
    let mut drawn = 0;
    for (i, d) in LOD_DISTANCES.iter().enumerate() {
        // Alternately left and right of the line of sight.
        let x = if i % 2 == 0 { -1.6 } else { 1.6 };
        let at = Mat4::from_rotation_translation(
            Quat::from_rotation_y(0.6 * i as f32),
            Vec3::new(x, 0.0, -d),
        );
        let per_unit = lod::pixels_per_unit(
            &scene.frame.camera,
            at.transform_point3(center),
            radius,
            720.0,
        );
        let level = lod::select(&errors, per_unit, 1.0, None);
        chosen[i] = level;
        drawn += tris(level);
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
                fx: InstanceFx {
                    flash: 0.45,
                    flash_color: lod::debug_color(level),
                    ..InstanceFx::NONE
                },
                cast_shadows: true,
                receive_shadows: true,
                sway: 0.0,
            });
        }
    }
    Ok((scene, chosen, (drawn, 3 * tris(0))))
}
