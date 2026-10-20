//! MagicaVoxel `.vox` through the cook: its import settings' `model {
//! voxScale voxOrigin }` reach the mesh (a settings file sorts after the
//! `.vox` it names, so the conversion waits for it), an edit of them
//! re-cooks the model, and a bad one is refused by line.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

/// A `.vox` of one model of `size` (MagicaVoxel z up), no scene graph.
fn vox(size: [u32; 3], voxels: &[[u8; 4]]) -> Vec<u8> {
    let mut children = Vec::new();
    let chunk = |out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]| {
        out.extend_from_slice(id);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(body);
    };
    let body: Vec<u8> = size.iter().flat_map(|s| s.to_le_bytes()).collect();
    chunk(&mut children, b"SIZE", &body);
    let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
    for v in voxels {
        xyzi.extend_from_slice(v);
    }
    chunk(&mut children, b"XYZI", &xyzi);
    let mut out = b"VOX ".to_vec();
    out.extend_from_slice(&150u32.to_le_bytes());
    out.extend_from_slice(b"MAIN");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(children.len() as u32).to_le_bytes());
    out.extend_from_slice(&children);
    out
}

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-cook-vox-{}", std::process::id()))
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn settings(body: &str) -> Vec<u8> {
    format!("type import\nversion 1\n---\nmodel {{\n{body}\n}}\n").into_bytes()
}

fn bounds(root: &Path) -> ([f32; 3], [f32; 3]) {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let m = bundle.load_model("models/crate.vox").unwrap();
    (m.min.to_array(), m.max.to_array())
}

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

#[test]
fn import_settings_scale_and_ground_a_vox_and_an_edit_recooks_it() {
    // A 4 x 2 x 6 block (6 voxels tall in MagicaVoxel z, y in glTF).
    let mut voxels = Vec::new();
    for z in 0..6u8 {
        for y in 0..2u8 {
            for x in 0..4u8 {
                voxels.push([x, y, z, 9]);
            }
        }
    }
    let block = vox([4, 2, 6], &voxels);
    let root = project(
        "crate",
        &[
            ("models/crate.vox", &block),
            (
                "models/crate.vox.ogt",
                &settings("voxScale 0.25\nvoxOrigin bottom"),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let (lo, hi) = bounds(&root);
    // 1 m x 1.5 m x 0.5 m (a file without a scene graph puts its model's
    // corner at the origin), on the ground.
    assert!(
        close(lo, [0.0, 0.0, -0.5]) && close(hi, [1.0, 1.5, 0.0]),
        "{lo:?} {hi:?}"
    );

    // Unchanged: nothing cooks. The settings edited: the model again.
    let again = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!again.cooked_outputs().iter().any(|o| o.cooked));
    std::fs::write(
        root.join("assets/models/crate.vox.ogt"),
        settings("voxScale 1"),
    )
    .unwrap();
    let third = cook_project(&root, &CookOptions::default()).unwrap();
    let cooked: Vec<&str> = third
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(cooked, vec!["models/crate.vox"]);
    let (lo, hi) = bounds(&root);
    assert!(
        close(lo, [0.0, 0.0, -2.0]) && close(hi, [4.0, 6.0, 0.0]),
        "one metre a voxel: {lo:?} {hi:?}"
    );

    // A bad setting fails the model, naming the line.
    std::fs::write(
        root.join("assets/models/crate.vox.ogt"),
        settings("voxOrigin top"),
    )
    .unwrap();
    let bad = cook_project(&root, &CookOptions::default()).unwrap();
    let e = bad.errors().join("\n");
    assert!(
        e.contains("models/crate.vox") && e.contains("line 5") && e.contains("voxOrigin"),
        "{e}"
    );
}
