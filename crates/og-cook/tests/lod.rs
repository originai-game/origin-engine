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

/// A UV sphere of radius 1, `n` segments around and `n / 2` rings, wound
/// counter-clockwise seen from outside (glTF's front).
fn sphere(n: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let rings = n / 2;
    let mut p = Vec::new();
    for j in 0..=rings {
        let (st, ct) = (std::f32::consts::PI * j as f32 / rings as f32).sin_cos();
        for i in 0..=n {
            let (sp, cp) = (std::f32::consts::TAU * i as f32 / n as f32).sin_cos();
            p.push([st * cp, ct, st * sp]);
        }
    }
    let mut idx = Vec::new();
    let w = n + 1;
    for j in 0..rings {
        for i in 0..n {
            let a = j * w + i;
            idx.extend([a, a + 1, a + w, a + 1, a + w + 1, a + w]);
        }
    }
    (p, idx)
}

/// Clusters (og-render's `meshlet`): every level is cut into clusters of at
/// most 64 triangles that cover it in order, each sphere holds its
/// triangles, and the normal cone culls a cluster only when every one of
/// its triangles faces away from the eye.
///
/// Ways this could fail: a level's indices kept in their old order while
/// the cluster ranges assume the new one (a cluster draws another's
/// triangles); a coarse level's clusters numbered from the primitive's
/// first cluster instead of the level's; the cone written in the winding's
/// opposite sense (front faces culled); clusters cut far below 64 (the
/// vertex cap at 64 on a faceted mesh).
#[test]
fn every_level_is_cut_into_clusters_that_bound_and_face_their_triangles() {
    use og_render::meshlet::{CLUSTER_TRIANGLES, backfacing};
    let s = sphere(96);
    let full = s.1.len() / 3;
    let root = project(
        "og-cook-lod-clusters",
        &[("models/ball.glb", &glb(&[s], false))],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures("models/ball.glb").unwrap();
    let p = &model.primitives[0];
    assert_eq!(p.triangle_count(0), full);
    let mc = p.mesh_clusters().expect("every level has clusters");
    assert_eq!(mc.levels.len(), p.lods.len());
    mc.validate(mc.indices.len() as u32).unwrap();
    assert_ne!(
        mc.indices, p.indices,
        "draw order differs from simulation order"
    );
    // Level 0's clusters are nearly full: 9216 triangles in at most 160.
    let l0 = &mc.levels[0];
    let n0 = (l0.clusters.end - l0.clusters.start) as usize;
    assert!(
        n0 * CLUSTER_TRIANGLES < full * 11 / 10,
        "{n0} clusters for {full}"
    );
    let pos = |i: u32| glam::Vec3::from(p.vertices[i as usize].position);
    let eyes: Vec<glam::Vec3> = (0..64)
        .map(|k| {
            let (a, b) = (k as f32 * 2.4, (k as f32 / 64.0) * 2.0 - 1.0);
            let r = (1.0 - b * b).sqrt();
            glam::Vec3::new(r * a.cos(), b, r * a.sin()) * (1.5 + (k % 5) as f32)
        })
        .collect();
    let mut culled = 0;
    for (k, level) in mc.levels.iter().enumerate() {
        // Same oriented triangles, including duplicates; the codec may
        // rotate a triangle's corners but must not reverse its winding.
        let triangles = |indices: &[u32]| {
            let mut tris: Vec<_> = indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|&[a, b, c]| [a, b, c].min([b, c, a]).min([c, a, b]))
                .collect();
            tris.sort_unstable();
            tris
        };
        let range = level.first as usize..(level.first + level.count) as usize;
        assert_eq!(
            triangles(&mc.indices[range.clone()]),
            triangles(&p.indices[range])
        );
        // Selection per cluster in the per-level chain: the level's error
        // (never under the one before), the next level's as the parent's,
        // the level as the group, one sphere holding the whole mesh.
        let chain = mc.levels[..=k].iter().map(|l| l.error).fold(0.0, f32::max);
        let parent = mc
            .levels
            .get(k + 1)
            .map_or(f32::MAX, |l| l.error.max(chain));
        for c in &mc.clusters[level.clusters.start as usize..level.clusters.end as usize] {
            assert_eq!(
                (c.error, c.parent_error, c.group),
                (chain, parent, k as u32)
            );
            assert_eq!(
                (c.error_center, c.error_radius),
                (mc.clusters[0].error_center, mc.clusters[0].error_radius)
            );
            let tris = &mc.indices[c.first as usize..(c.first + c.triangles * 3) as usize];
            for tri in tris.chunks(3) {
                for &i in tri {
                    assert!(pos(i).distance(c.error_center) <= c.error_radius * 1.001 + 1e-5);
                    assert!(
                        pos(i).distance(c.center) <= c.radius * 1.001 + 1e-5,
                        "a vertex outside its cluster's sphere: {} > {} ({:?}, level at {})",
                        pos(i).distance(c.center),
                        c.radius,
                        c,
                        level.first
                    );
                }
            }
            for &eye in &eyes {
                if !backfacing(c.center, c.radius, c.cone_axis, c.cone_cutoff, eye) {
                    continue;
                }
                culled += 1;
                for tri in tris.chunks(3) {
                    let [a, b, d] = [pos(tri[0]), pos(tri[1]), pos(tri[2])];
                    let n = (b - a).cross(d - a);
                    assert!(
                        n.dot(eye - a) <= 1e-6,
                        "a triangle facing the eye in a cluster its cone culled"
                    );
                }
            }
        }
    }
    // Seen from outside, about half of a sphere's clusters face away.
    assert!(
        culled > 64 * n0 / 4,
        "the cone culled {culled} cluster views"
    );
}
