//! Scenes of MagicaVoxel `.vox` models through the cook and og-anim, as a
//! game loads and plays them: the file is written here, cooked into a
//! project of its own (once per process) and loaded from its blobs, and
//! its clip is sampled and skinned as og-runtime poses a model.
//!
//! - `vox-keyframes`: a transform keyed across frames (MagicaVoxel's
//!   `nTRN` frames, owed row 63) moves the part under it. Two copies of
//!   one model: a grey slab with a blue post at its middle and a red block
//!   keyed at frame 0 on the slab's left, at frame 2 two voxels above its
//!   right. The left copy is posed at frame 0, the right one at frame 2,
//!   so the frame shows the block at both of its keys. Before the keys
//!   were read the first one placed the block throughout: both copies
//!   showed it on the left.

use glam::{Mat4, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Background, Camera, DirectionalLight, Exposure, JointPalette, RenderScene, SkinData,
    SkinnedInstance, StandardMaterial, shapes,
};
use std::path::PathBuf;
use std::sync::OnceLock;

fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(body);
}

fn dict(b: &mut Vec<u8>, pairs: &[(&str, String)]) {
    b.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
    for (k, v) in pairs {
        for s in [*k, v.as_str()] {
            b.extend_from_slice(&(s.len() as u32).to_le_bytes());
            b.extend_from_slice(s.as_bytes());
        }
    }
}

/// A transform's key frame: its `_f` and `_t`.
type Key = (u32, [i32; 3]);

/// An `nTRN` over `child` with key frames.
fn transform(out: &mut Vec<u8>, node: u32, child: u32, keys: &[Key]) {
    let mut b = node.to_le_bytes().to_vec();
    dict(&mut b, &[]);
    for v in [child, u32::MAX, 0, keys.len() as u32] {
        b.extend_from_slice(&v.to_le_bytes());
    }
    for (f, t) in keys {
        let at = format!("{} {} {}", t[0], t[1], t[2]);
        dict(&mut b, &[("_f", f.to_string()), ("_t", at)]);
    }
    chunk(out, b"nTRN", &b);
}

/// The keyed file: models 0 (a 10 x 4 x 1 slab, grey), 1 (a 2 x 2 x 2
/// block, red) and 2 (a 1 x 1 x 3 post, blue), MagicaVoxel z up; the
/// block's transform keyed at frame 0 (x -3, resting on the slab) and
/// frame 2 (x 3, two voxels above it).
fn keyed_vox() -> Vec<u8> {
    let solid = |size: [u8; 3], c: u8| {
        let mut v = Vec::new();
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    v.push([x, y, z, c]);
                }
            }
        }
        v
    };
    let models = [
        ([10u8, 4, 1], solid([10, 4, 1], 1)),
        ([2, 2, 2], solid([2, 2, 2], 2)),
        ([1, 1, 3], solid([1, 1, 3], 3)),
    ];
    let mut children = Vec::new();
    for (size, voxels) in &models {
        let body: Vec<u8> = size
            .iter()
            .flat_map(|s| (*s as u32).to_le_bytes())
            .collect();
        chunk(&mut children, b"SIZE", &body);
        let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
        for v in voxels {
            xyzi.extend_from_slice(v);
        }
        chunk(&mut children, b"XYZI", &xyzi);
    }
    // The graph: root 0 → group 1 → the slab (2, 3), the block (4, 5)
    // and the post (6, 7).
    transform(&mut children, 0, 1, &[(0, [0, 0, 0])]);
    let mut grp = 1u32.to_le_bytes().to_vec();
    dict(&mut grp, &[]);
    grp.extend_from_slice(&3u32.to_le_bytes());
    for c in [2u32, 4, 6] {
        grp.extend_from_slice(&c.to_le_bytes());
    }
    chunk(&mut children, b"nGRP", &grp);
    let parts: [(u32, u32, &[Key]); 3] = [
        (2, 0, &[(0, [0, 0, 0])]),
        (4, 1, &[(0, [-3, 0, 2]), (2, [3, 0, 4])]),
        (6, 2, &[(0, [0, 0, 2])]),
    ];
    for (node, model, keys) in parts {
        transform(&mut children, node, node + 1, keys);
        let mut shp = (node + 1).to_le_bytes().to_vec();
        dict(&mut shp, &[]);
        shp.extend_from_slice(&1u32.to_le_bytes());
        shp.extend_from_slice(&model.to_le_bytes());
        dict(&mut shp, &[]);
        chunk(&mut children, b"nSHP", &shp);
    }
    // The palette: 1 stone grey, 2 red, 3 blue (entry i is index i + 1).
    let mut rgba = Vec::new();
    for i in 0..256 {
        rgba.extend_from_slice(&match i {
            0 => [150, 148, 140, 255],
            1 => [210, 52, 40, 255],
            2 => [50, 96, 210, 255],
            _ => [0, 0, 0, 255],
        });
    }
    chunk(&mut children, b"RGBA", &rgba);
    let mut out = b"VOX ".to_vec();
    out.extend_from_slice(&150u32.to_le_bytes());
    out.extend_from_slice(b"MAIN");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(children.len() as u32).to_le_bytes());
    out.extend_from_slice(&children);
    out
}

/// The project holding the keyed file (`voxScale 0.25`, 10 frames a
/// second), cooked once per process.
fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root =
            std::env::temp_dir().join(format!("og-tests-vox-keyframes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        let write = |name: &str, bytes: &[u8]| {
            std::fs::write(models.join(name), bytes).map_err(|e| format!("{name}: {e}"))
        };
        write("lift.vox", &keyed_vox())?;
        write(
            "lift.vox.ogt",
            b"type import\nversion 1\n---\nmodel {\n  voxScale 0.25\n}\n",
        )?;
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

/// `vox-keyframes` (module docs).
pub fn keyframes_scene() -> Result<RenderScene, String> {
    let root = cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let model = bundle
        .load_model("models/lift.vox")
        .map_err(|e| e.to_string())?;
    let anim = bundle
        .load_animation("models/lift.vox")
        .map_err(|e| e.to_string())?;
    let clip = anim
        .clip("frames")
        .ok_or("the keyed file has no clip `frames`")?;
    let binding = og_anim::bind(&anim.skeleton, clip);
    let mut scene = RenderScene::new();
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.42, 0.5, 0.38),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    // The model's meshes and materials once; its skinned primitives (the
    // block's) once more with their joints, as og-runtime draws a posed
    // model.
    let base = scene.mesh_count();
    let parts = model.add_to(&mut scene, &[]).parts;
    let skinned: Vec<_> = (0..model.primitives.len())
        .map(|p| {
            let stream = anim.skin_streams.iter().find(|s| s.primitive == p)?;
            let mesh = scene.mesh(scene.mesh_handle(base + p)?)?.clone();
            Some(scene.add_skinned_mesh(
                mesh,
                SkinData {
                    joints: stream.joints.clone(),
                    weights: stream.weights.clone(),
                },
            ))
        })
        .collect();
    if skinned.iter().flatten().count() != 1 {
        return Err("the keyed file should skin one primitive, the block".into());
    }
    // Frame 0 on the left, frame 2 on the right.
    for (x, frame) in [(-1.6f32, 0.0f32), (1.6, 2.0)] {
        let mut pose = og_anim::Pose::rest(&anim.skeleton);
        og_anim::sample(
            &anim.skeleton,
            clip,
            &binding,
            frame / clip.rate,
            og_anim::NONE,
            &mut pose,
        );
        let mut matrices = Vec::new();
        anim.skeleton.palette(0, &pose.nodes, &mut matrices);
        let palette = scene.joint_palettes.len() as u32;
        scene.joint_palettes.push(JointPalette { matrices });
        let world = Mat4::from_translation(Vec3::new(x, 0.0, 0.0));
        for &(mesh, material, local) in &parts {
            match skinned.get(mesh.index() - base).copied().flatten() {
                Some(s) => scene
                    .skinned_instances
                    .push(SkinnedInstance::new(s, material, world, palette)),
                None => scene.add_instance(mesh, material, world * local),
            }
        }
    }
    let sun = Vec3::new(-0.5, 0.8, 0.45);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.8;
    scene.environment.background = Background::Color(Vec3::new(0.55, 0.68, 0.82));
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.4, 4.6),
        target: Vec3::new(0.0, 0.45, 0.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    Ok(scene)
}
