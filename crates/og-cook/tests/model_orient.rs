//! A model's stated orientation (`model { front "-x" up "+y" }` in its
//! import settings) is applied in the cook, so the cooked model faces +Z
//! with +Y up (the facing guide) and the game places it with no turn of
//! its own.
//!
//! Ways this could fail: the setting is ignored (the nose stays on -X, as
//! image-to-3D models from three-quarter concepts came); the turn goes the
//! wrong way round (the nose on -Z) or mirrors the model (handedness
//! lost: the right wing on -X); a model without the setting is turned.

use og_cook::runtime::{Bundle, CookedModel};
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

/// A pyramid with its nose at (-2, 0, 0), its base square at x = 0, a
/// fin on +Y and the right wing tip (its own right, nose ahead, up up) at
/// z = -1.5: authored nose -X, up +Y.
fn glb() -> Vec<u8> {
    let p: [[f32; 3]; 6] = [
        [-2.0, 0.0, 0.0],
        [0.0, 0.5, 0.5],
        [0.0, 0.5, -0.5],
        [0.0, -0.5, -0.5],
        [0.0, -0.5, 0.5],
        [0.0, 0.0, -1.5],
    ];
    let idx: [u32; 21] = [
        0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 1, 1, 3, 2, 1, 4, 3, 2, 3, 5,
    ];
    let mut bin: Vec<u8> = p.iter().flatten().flat_map(|x| x.to_le_bytes()).collect();
    let at = bin.len();
    bin.extend(idx.iter().flat_map(|x| x.to_le_bytes()));
    let json = serde_json::json!({
        "asset": { "version": "2.0" },
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": at },
            { "buffer": 0, "byteOffset": at, "byteLength": idx.len() * 4 },
        ],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 6, "type": "VEC3", "min": [-2.0, -0.5, -1.5], "max": [0.0, 0.5, 0.5] },
            { "bufferView": 1, "componentType": 5125, "count": idx.len(), "type": "SCALAR" },
        ],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "indices": 1 }] }],
        "nodes": [{ "mesh": 0, "name": "ship" }],
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

/// Every drawn vertex of the cooked model, in model space.
fn drawn(name: &str, settings: Option<&str>) -> Vec<glam::Vec3> {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    let models = root.join("assets/models");
    std::fs::create_dir_all(&models).unwrap();
    std::fs::write(models.join("ship.glb"), glb()).unwrap();
    if let Some(s) = settings {
        std::fs::write(models.join("ship.glb.ogt"), s).unwrap();
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model: CookedModel = bundle.load_model("models/ship.glb").unwrap();
    let world = model.node_world();
    let mut out = Vec::new();
    for n in model.scene_nodes(model.default_scene.unwrap()) {
        let Some(m) = model.nodes[n].mesh else {
            continue;
        };
        for p in model.meshes[m].primitives.clone() {
            for v in &model.primitives[p].vertices {
                out.push(world[n].transform_point3(glam::Vec3::from(v.position)));
            }
        }
    }
    out
}

/// The vertex furthest along `dir`.
fn furthest(v: &[glam::Vec3], dir: glam::Vec3) -> glam::Vec3 {
    *v.iter()
        .max_by(|a, b| a.dot(dir).total_cmp(&b.dot(dir)))
        .unwrap()
}

#[test]
fn a_model_authored_nose_minus_x_faces_plus_z_after_the_cook() {
    let settings = "type import\nversion 1\n---\nmodel {\n  front \"-x\"\n  up \"+y\"\n}\n";
    let v = drawn("og-cook-orient-turned", Some(settings));
    let nose = furthest(&v, glam::Vec3::Z);
    assert!(
        (nose - glam::Vec3::new(0.0, 0.0, 2.0)).length() < 1e-3,
        "the nose is at {nose}, not +Z"
    );
    // Facing +Z with +Y up, the model's right is -X: handedness kept.
    let wing = furthest(&v, glam::Vec3::NEG_X);
    assert!(
        (wing - glam::Vec3::new(-1.5, 0.0, 0.0)).length() < 1e-3,
        "the right wing is at {wing}, not -X (mirrored?)"
    );
    let top = furthest(&v, glam::Vec3::Y);
    assert!((top.y - 0.5).abs() < 1e-3, "up is not +Y: {top}");
}

#[test]
fn a_model_without_a_stated_orientation_is_cooked_as_authored() {
    let v = drawn("og-cook-orient-plain", None);
    let nose = furthest(&v, glam::Vec3::NEG_X);
    assert!(
        (nose - glam::Vec3::new(-2.0, 0.0, 0.0)).length() < 1e-3,
        "{nose}"
    );
}

#[test]
fn a_stated_orientation_along_one_axis_is_refused() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-orient-bad");
    let _ = std::fs::remove_dir_all(&root);
    let models = root.join("assets/models");
    std::fs::create_dir_all(&models).unwrap();
    std::fs::write(models.join("ship.glb"), glb()).unwrap();
    std::fs::write(
        models.join("ship.glb.ogt"),
        "type import\nversion 1\n---\nmodel {\n  front \"-x\"\n  up \"+x\"\n}\n",
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = format!("{:?}", report.errors());
    assert!(!report.ok() && errors.contains("right angles"), "{errors}");
}
