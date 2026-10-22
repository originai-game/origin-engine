//! Mesh levels of detail through the cook (docs/depth.md §6): every model
//! mesh is simplified with meshoptimizer into the levels its import
//! settings name (`model { lods 1 0.5 0.25 }` by default), and a glTF's own
//! `MSFT_lod` levels are kept instead of simplified ones.
//!
//! Ways this could fail: the default chain stops after level 0 (the error
//! cap or the 95 % rule refusing a smooth mesh); a `lods` setting is
//! ignored, or a malformed one cooked silently; MSFT_lod's coarse mesh is
//! simplified again, or its vertices are not offset when the levels share
//! one vertex stream (a level drawing the full mesh's vertices).

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

/// An `n` × `n` quad grid over the unit square in xz, gently waved in y.
fn grid(n: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut p = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let (x, z) = (i as f32 / n as f32, j as f32 / n as f32);
            let y = 0.04 * (x * 6.0).sin() * (z * 5.0).cos();
            p.push([x, y, z]);
        }
    }
    let mut idx = Vec::new();
    let w = n + 1;
    for j in 0..n {
        for i in 0..n {
            let a = j * w + i;
            idx.extend([a, a + w, a + 1, a + 1, a + w, a + w + 1]);
        }
    }
    (p, idx)
}

/// A glb of one mesh per grid; node 0 draws mesh 0 and, with `lod`, names
/// node 1 (mesh 1, outside the scene) as its MSFT_lod level.
fn glb(grids: &[(Vec<[f32; 3]>, Vec<u32>)], lod: bool) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();
    for (p, idx) in grids {
        let at = bin.len();
        bin.extend(p.iter().flatten().flat_map(|x| x.to_le_bytes()));
        views
            .push(serde_json::json!({ "buffer": 0, "byteOffset": at, "byteLength": p.len() * 12 }));
        let (lo, hi) = p
            .iter()
            .fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), v| {
                (
                    [lo[0].min(v[0]), lo[1].min(v[1]), lo[2].min(v[2])],
                    [hi[0].max(v[0]), hi[1].max(v[1]), hi[2].max(v[2])],
                )
            });
        accessors.push(serde_json::json!({ "bufferView": views.len() - 1, "componentType": 5126, "count": p.len(), "type": "VEC3", "min": lo, "max": hi }));
        let at = bin.len();
        bin.extend(idx.iter().flat_map(|x| x.to_le_bytes()));
        views.push(
            serde_json::json!({ "buffer": 0, "byteOffset": at, "byteLength": idx.len() * 4 }),
        );
        accessors.push(serde_json::json!({ "bufferView": views.len() - 1, "componentType": 5125, "count": idx.len(), "type": "SCALAR" }));
        meshes.push(serde_json::json!({ "primitives": [{ "attributes": { "POSITION": accessors.len() - 2 }, "indices": accessors.len() - 1 }] }));
    }
    let mut nodes = vec![serde_json::json!({ "mesh": 0 })];
    if lod {
        nodes[0]["extensions"] = serde_json::json!({ "MSFT_lod": { "ids": [1] } });
        nodes.push(serde_json::json!({ "mesh": 1 }));
    }
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "extensionsUsed": if lod { vec!["MSFT_lod"] } else { vec![] },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": views,
        "accessors": accessors,
        "meshes": meshes,
        "nodes": nodes,
        "scenes": [{ "nodes": [0] }],
        "scene": 0,
    });
    let mut text = serde_json::to_vec(&json).unwrap();
    while !text.len().is_multiple_of(4) {
        text.push(b' ');
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&((12 + 8 + text.len() + 8 + bin.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

/// Triangles per level of the model's first primitive.
fn levels(root: &Path, id: &str) -> Vec<usize> {
    let report = cook_project(root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures(id).expect("decodes");
    let p = &model.primitives[0];
    (0..p.lods.len()).map(|l| p.triangle_count(l)).collect()
}

#[test]
fn every_mesh_is_simplified_into_the_default_levels() {
    let g = grid(48);
    let full = g.1.len() / 3;
    let root = project(
        "og-cook-lod-default",
        &[("models/field.glb", &glb(&[g], false))],
    );
    let tris = levels(&root, "models/field.glb");
    assert_eq!(tris.len(), 3, "levels 1, 0.5, 0.25: {tris:?}");
    assert_eq!(tris[0], full);
    // meshoptimizer lands near its target, never past the next level's.
    assert!(
        tris[1] <= full * 55 / 100 && tris[1] > full * 35 / 100,
        "{tris:?}"
    );
    assert!(
        tris[2] <= full * 30 / 100 && tris[2] > full / 10,
        "{tris:?}"
    );
}

#[test]
fn a_model_names_its_own_levels_and_bad_ones_are_refused() {
    let g = grid(48);
    let full = g.1.len() / 3;
    let root = project(
        "og-cook-lod-settings",
        &[
            ("models/field.glb", &glb(&[g.clone()], false)),
            (
                "models/field.glb.ogt",
                b"type import\nversion 1\n---\nmodel {\n  lods 1 0.2\n  lodError 0.2\n}\n",
            ),
        ],
    );
    let tris = levels(&root, "models/field.glb");
    assert_eq!(tris.len(), 2, "{tris:?}");
    assert!(tris[1] <= full * 22 / 100, "{tris:?}");

    let root = project(
        "og-cook-lod-bad",
        &[
            ("models/field.glb", &glb(&[g], false)),
            (
                "models/field.glb.ogt",
                b"type import\nversion 1\n---\nmodel {\n  lods 1 0.5 0.7\n}\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = format!("{:?}", report.errors());
    assert!(errors.contains("each below the one before"), "{errors}");
}

#[test]
fn msft_lod_levels_are_kept_as_the_gltf_gives_them() {
    let (fine, coarse) = (grid(32), grid(6));
    let coarse_tris = coarse.1.len() / 3;
    let root = project(
        "og-cook-lod-msft",
        &[(
            "models/rock.glb",
            &glb(&[fine.clone(), coarse.clone()], true),
        )],
    );
    let tris = levels(&root, "models/rock.glb");
    assert_eq!(
        tris,
        vec![fine.1.len() / 3, coarse_tris],
        "the glTF's two levels, not simplified ones"
    );
    // Level 1's triangles are the coarse grid's own, on its own vertices.
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures("models/rock.glb").unwrap();
    let mesh = model.primitives[0].mesh(1);
    let mut corners: Vec<[i32; 3]> = mesh
        .indices
        .iter()
        .map(|&i| {
            mesh.vertices[i as usize]
                .position
                .map(|x| (x * 60.0).round() as i32)
        })
        .collect();
    let mut want: Vec<[i32; 3]> = coarse
        .1
        .iter()
        .map(|&i| coarse.0[i as usize].map(|x| (x * 60.0).round() as i32))
        .collect();
    corners.sort();
    want.sort();
    assert_eq!(corners, want);
    // The level's node is not drawn as a part of its own.
    assert_eq!(model.scene_nodes(model.default_scene.unwrap()).len(), 1);
}
