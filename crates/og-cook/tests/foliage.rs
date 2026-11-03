//! Foliage through the cook: a primitive's `_OG_PIVOT` and `_OG_ANCHOR`
//! (og-render's `meshlet::MeshFoliage`) and `extras.ogFoliage.leaves`.
//!
//! Ways this could fail: the custom attributes are dropped (gltf's `extras`
//! feature off) or follow the wrong vertices after welding and the fetch
//! reorder; two vertices at one position with other pivots weld into one;
//! a simplified level collapses across a pivot seam, joining two branches
//! that sway apart; leaf cards get simplified levels; the leaf flag or the
//! model's top and canopy centre are lost.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

const N: u32 = 24;

/// A waved `N` × `N` grid over the unit square in xz: the "bark".
fn grid() -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut p = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            let (x, z) = (i as f32 / N as f32, j as f32 / N as f32);
            p.push([x, 0.04 * (x * 6.0).sin() * (z * 5.0).cos(), z]);
        }
    }
    let w = N + 1;
    let mut idx = Vec::new();
    for j in 0..N {
        for i in 0..N {
            let a = j * w + i;
            idx.extend([a, a + w, a + 1, a + 1, a + w, a + w + 1]);
        }
    }
    (p, idx)
}

/// The pivot (and stiffness) of a bark vertex: two branches split at x 0.5.
fn pivot(p: [f32; 3]) -> [f32; 4] {
    if p[0] < 0.5 {
        [0.0, 0.0, 0.0, 0.25]
    } else {
        [1.0, 0.0, 0.0, 0.75]
    }
}

fn glb() -> Vec<u8> {
    let (bp, bi) = grid();
    // Two leaf cards, high up, each its own anchor and flutter.
    let lp: Vec<[f32; 3]> = vec![
        [0.0, 2.0, 0.0],
        [0.2, 2.0, 0.0],
        [0.2, 2.2, 0.0],
        [0.0, 2.2, 0.0],
        [0.6, 3.0, 0.0],
        [0.8, 3.0, 0.0],
        [0.8, 3.2, 0.0],
        [0.6, 3.2, 0.0],
    ];
    let li: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
    let bark_pivot: Vec<[f32; 4]> = bp.iter().map(|&p| pivot(p)).collect();
    let bark_anchor: Vec<[f32; 4]> = bp.iter().map(|_| [0.0; 4]).collect();
    let leaf_pivot: Vec<[f32; 4]> = (0..8).map(|_| [0.0, 0.0, 0.0, 0.25]).collect();
    let leaf_anchor: Vec<[f32; 4]> = (0..8)
        .map(|k| {
            if k < 4 {
                [0.1, 2.0, 0.0, 0.5]
            } else {
                [0.7, 3.0, 0.0, 1.0]
            }
        })
        .collect();
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut put = |bytes: Vec<u8>, count: usize, kind: &str, ct: u32, extra: serde_json::Value| {
        let at = bin.len();
        let len = bytes.len();
        bin.extend(bytes);
        views.push(serde_json::json!({ "buffer": 0, "byteOffset": at, "byteLength": len }));
        let mut a = serde_json::json!({ "bufferView": views.len() - 1, "componentType": ct, "count": count, "type": kind });
        if let serde_json::Value::Object(m) = extra {
            a.as_object_mut().unwrap().extend(m);
        }
        accessors.push(a);
        accessors.len() - 1
    };
    let f32s = |v: &[[f32; 3]]| {
        v.iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>()
    };
    let f4s = |v: &[[f32; 4]]| {
        v.iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<u8>>()
    };
    let u32s = |v: &[u32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    let bounds = |p: &[[f32; 3]]| {
        let lo = (0..3)
            .map(|a| p.iter().map(|v| v[a]).fold(f32::MAX, f32::min))
            .collect::<Vec<_>>();
        let hi = (0..3)
            .map(|a| p.iter().map(|v| v[a]).fold(f32::MIN, f32::max))
            .collect::<Vec<_>>();
        serde_json::json!({ "min": lo, "max": hi })
    };
    let b_pos = put(f32s(&bp), bp.len(), "VEC3", 5126, bounds(&bp));
    let b_idx = put(u32s(&bi), bi.len(), "SCALAR", 5125, serde_json::json!({}));
    let b_piv = put(
        f4s(&bark_pivot),
        bp.len(),
        "VEC4",
        5126,
        serde_json::json!({}),
    );
    let b_anc = put(
        f4s(&bark_anchor),
        bp.len(),
        "VEC4",
        5126,
        serde_json::json!({}),
    );
    let l_pos = put(f32s(&lp), lp.len(), "VEC3", 5126, bounds(&lp));
    let l_idx = put(u32s(&li), li.len(), "SCALAR", 5125, serde_json::json!({}));
    let l_piv = put(f4s(&leaf_pivot), 8, "VEC4", 5126, serde_json::json!({}));
    let l_anc = put(f4s(&leaf_anchor), 8, "VEC4", 5126, serde_json::json!({}));
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": views,
        "accessors": accessors,
        "meshes": [{ "primitives": [
            { "attributes": { "POSITION": b_pos, "_OG_PIVOT": b_piv, "_OG_ANCHOR": b_anc }, "indices": b_idx },
            { "attributes": { "POSITION": l_pos, "_OG_PIVOT": l_piv, "_OG_ANCHOR": l_anc }, "indices": l_idx,
              "extras": { "ogFoliage": { "leaves": true } } },
        ] }],
        "nodes": [{ "mesh": 0 }],
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

#[test]
fn foliage_follows_its_vertices_and_keeps_its_seams_through_every_level() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("foliage-cook");
    let _ = std::fs::remove_dir_all(&root);
    let path = root.join("assets/models/tree.glb");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, glb()).unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle
        .load_model_with_textures("models/tree.glb")
        .expect("decodes");

    // The bark: each cooked vertex carries the pivot of its own position,
    // after welding and the fetch reorder (the cooked positions are
    // quantised: x within a step of the source's).
    let bark = &model.primitives[0];
    let (fv, leaves) = bark.foliage.as_ref().expect("the bark's foliage");
    assert!(!leaves);
    assert_eq!(fv.len(), bark.vertices.len());
    for (v, f) in bark.vertices.iter().zip(fv) {
        assert_eq!(f.branch, pivot(v.position), "{:?}", v.position);
        assert_eq!(f.leaf, [0.0; 4]);
    }
    // The grid is simplified, and no level has a triangle whose corners
    // belong to both branches other than across the seam's own edges:
    // every vertex the source had on the seam (an edge to the other
    // pivot) is in every level, so the coarser levels close up to it.
    assert!(bark.lods.len() > 1, "{} levels", bark.lods.len());
    // (The grid has no normals: flat shading, every face its own corners,
    // simplified over positions; the seam's points are compared by place.)
    let x0 = (N / 2 - 1) as f32 / N as f32;
    let x1 = (N / 2) as f32 / N as f32;
    let seam_points = |l: usize| {
        let lod = bark.lods[l];
        let mut points: Vec<(u32, u32)> = bark.indices[lod.first..lod.first + lod.count]
            .iter()
            .map(|&i| bark.vertices[i as usize].position)
            .filter(|p| (p[0] - x0).abs() < 1e-3 || (p[0] - x1).abs() < 1e-3)
            .map(|p| {
                (
                    (p[0] * N as f32).round() as u32,
                    (p[2] * N as f32).round() as u32,
                )
            })
            .collect();
        points.sort();
        points.dedup();
        points
    };
    let full = seam_points(0);
    assert_eq!(full.len(), 2 * (N as usize + 1));
    for l in 1..bark.lods.len() {
        assert_eq!(seam_points(l), full, "level {l} moved the seam");
    }
    // And no coarser triangle spans the seam with corners of both
    // branches pulled off it (a level that joined them would).
    for l in 1..bark.lods.len() {
        let lod = bark.lods[l];
        for t in bark.indices[lod.first..lod.first + lod.count].chunks(3) {
            let xs: Vec<f32> = t
                .iter()
                .map(|&i| bark.vertices[i as usize].position[0])
                .collect();
            let off_seam = |x: f32| (x - x0).abs() > 1e-3 && (x - x1).abs() > 1e-3;
            let left = xs.iter().any(|&x| x < x0 - 1e-3);
            let right = xs.iter().any(|&x| x > x1 + 1e-3);
            assert!(
                !(left && right && xs.iter().all(|&x| off_seam(x))),
                "level {l}: {xs:?}"
            );
        }
    }

    // The leaf cards: flagged, one level, each card's anchor and flutter.
    let leaf = &model.primitives[1];
    let (lf, leaves) = leaf.foliage.as_ref().expect("the leaves' foliage");
    assert!(leaves);
    assert_eq!(leaf.lods.len(), 1, "leaf cards are never simplified");
    for (v, f) in leaf.vertices.iter().zip(lf) {
        let want = if v.position[1] < 2.5 {
            [0.1, 2.0, 0.0, 0.5]
        } else {
            [0.7, 3.0, 0.0, 1.0]
        };
        assert_eq!(f.leaf, want, "{:?}", v.position);
    }

    // What og-render receives: one top and one canopy centre for the tree.
    let mb = model.mesh_foliage(0).unwrap();
    let ml = model.mesh_foliage(1).unwrap();
    assert!((mb.top - 3.2).abs() < 1e-3, "{}", mb.top);
    assert_eq!((mb.top, mb.canopy_center), (ml.top, ml.canopy_center));
    assert!((mb.canopy_center - glam::Vec3::new(0.4, 2.5, 0.0)).length() < 1e-3);
    assert!(mb.validate(bark.vertices.len()).is_ok());
}
