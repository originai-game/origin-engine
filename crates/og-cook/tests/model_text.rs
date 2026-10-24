//! `type model`: a model written as text (og-geom's generators and
//! og-render's primitives by their og.render names, placed as
//! mergeGeometries places parts, a material per part, pivots as nodes),
//! cooked into the model blob a glTF cooks to.
//!
//! Ways this could fail: the parts are built differently from one cook to
//! the next (a hash map's order, a time); the cooked mesh is not where the
//! text puts it (a placement applied twice, a hole filled), so a ray finds
//! no triangle where the gear is, or one where its hole is; the levels of
//! detail of a text model are measured otherwise than a glTF's (or not at
//! all); a part placed three times is three meshes; the budgets are only
//! reported, not enforced.

use glam::Vec3;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

/// A 12-tooth gear: each tooth 15° wide at a 1.2 tip, a root circle of 1
/// between them, a hole of 0.3 in the middle, extruded 0.2 along +z with no
/// bevel (its front face at z = 0.2). Beside it a lathed vase (a smooth
/// mesh, so its levels of detail simplify), and a bolt placed three times
/// round the axle pivot.
fn gear_text() -> String {
    let mut outline = Vec::new();
    for i in 0..12 {
        let a = (i as f64 * 30.0).to_radians();
        let d = |deg: f64| a + deg.to_radians();
        for (r, t) in [(1.0, d(0.0)), (1.2, d(2.0)), (1.2, d(13.0)), (1.0, d(15.0))] {
            outline.push(format!("{:?} {:?}", r * t.cos(), r * t.sin()));
        }
    }
    format!(
        r#"type model
version 1
---
material "steel" {{
  color 0.6 0.62 0.65
  metalness 1
  roughness 0.4
}}
material "clay" {{
  color 0.7 0.35 0.2
}}
pivot "axle" {{
  position 3 0 0
}}
part "gear" {{
  material "steel"
  extrudeGeometry {{
    shape {{
      points {}
      hole {{
        absarc 0 0 0.3 0 6.283185307179586 true
      }}
    }}
    depth 0.2
    bevelEnabled false
    curveSegments 16
  }}
}}
part "vase" {{
  material "clay"
  position -3 0 0
  latheGeometry {{
    points 0 0 0.5 0.1 0.6 0.5 0.3 1 0.4 1.4
    segments 48
  }}
}}
part "bolt" {{
  pivot "axle"
  material "steel"
  cylinderGeometry {{
    radiusTop 0.05
    radiusBottom 0.05
    height 0.3
    radialSegments 12
  }}
  at {{
    position 0.5 0 0
  }}
  at {{
    position -0.25 0.433 0
  }}
  at {{
    position -0.25 -0.433 0
  }}
}}
"#,
        outline.join(" ")
    )
}

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = if path.starts_with("project.ogt") {
            root.join(path)
        } else {
            root.join("assets").join(path)
        };
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn cooked(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = project(name, files);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

/// Every file under `cooked/` but the cook's times, with its bytes.
fn files(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out: Vec<(String, Vec<u8>)> = std::fs::read_dir(root.join("cooked"))
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().unwrap().is_file())
        .map(|e| {
            (
                e.file_name().to_string_lossy().into_owned(),
                std::fs::read(e.path()).unwrap(),
            )
        })
        .filter(|(n, _)| n != ".cook-times.ogt")
        .collect();
    let models = root.join("cooked/models");
    for e in std::fs::read_dir(&models).unwrap() {
        let e = e.unwrap();
        out.push((
            format!("models/{}", e.file_name().to_string_lossy()),
            std::fs::read(e.path()).unwrap(),
        ));
    }
    out.sort();
    out
}

#[test]
fn a_model_cooked_twice_is_byte_identical() {
    let text = gear_text();
    let a = cooked(
        "og-cook-model-text-a",
        &[("models/gear.ogt", text.as_bytes())],
    );
    let b = cooked(
        "og-cook-model-text-b",
        &[("models/gear.ogt", text.as_bytes())],
    );
    let (fa, fb) = (files(&a), files(&b));
    assert!(
        fa.iter().any(|(n, _)| n == "models/gear.ogt.ogb"),
        "{:?}",
        fa.iter().map(|f| &f.0).collect::<Vec<_>>()
    );
    assert_eq!(fa, fb);
    let bundle = Bundle::open(&a.join("cooked")).unwrap();
    assert_eq!(bundle.output("models/gear.ogt").unwrap().ty, "og.Model");
}

#[test]
fn a_cooked_gear_raycasts_where_its_triangles_are() {
    let root = cooked(
        "og-cook-model-text-ray",
        &[("models/gear.ogt", gear_text().as_bytes())],
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("models/gear.ogt").unwrap();
    let m = model
        .meshes
        .iter()
        .find(|m| m.name == "gear")
        .expect("the gear's mesh");
    let p = &model.primitives[m.primitives.start];
    let mesh = p.mesh(0);
    let bvh = p.bvh(&mesh).expect("a cooked BVH").unwrap();
    let down = Vec3::NEG_Z;
    let cast = |x: f32, y: f32| bvh.raycast(Vec3::new(x, y, 5.0), down, f32::INFINITY, true);
    // Every triangle of the front face is hit, by a ray down onto its
    // centre, at its own height.
    let pos = |i: u32| Vec3::from(mesh.vertices[i as usize].position);
    let mut front = 0;
    for (face, t) in mesh.indices.chunks(3).enumerate() {
        let [a, b, c] = [pos(t[0]), pos(t[1]), pos(t[2])];
        if (b - a).cross(c - a).normalize().z < 0.999 {
            continue;
        }
        front += 1;
        let centre = (a + b + c) / 3.0;
        let hit = cast(centre.x, centre.y);
        assert_eq!(hit.len(), 1, "no hit over front face {face} at {centre}");
        assert_eq!(hit[0].face as usize, face);
        assert!(
            (5.0 - hit[0].t - 0.2).abs() < 1e-4,
            "front face at z {}",
            5.0 - hit[0].t
        );
    }
    assert!(front > 40, "{front} front triangles");
    // The hole and the gaps between teeth are empty; a tooth is not.
    assert!(cast(0.0, 0.0).is_empty(), "the hole is filled");
    assert!(cast(0.2, 0.1).is_empty(), "the hole is filled");
    for i in 0..12 {
        let a = (i as f32 * 30.0).to_radians();
        let at = |deg: f32| {
            (
                (a + deg.to_radians()).cos() * 1.1,
                (a + deg.to_radians()).sin() * 1.1,
            )
        };
        let (x, y) = at(7.5);
        assert_eq!(cast(x, y).len(), 1, "tooth {i}");
        let (x, y) = at(22.5);
        assert!(cast(x, y).is_empty(), "gap {i}");
    }
    // Into the ring between hole and root circle, from below: the back face.
    let up = bvh.raycast(Vec3::new(0.6, 0.0, -5.0), Vec3::Z, f32::INFINITY, true);
    assert!(
        (up[0].t - 5.0).abs() < 1e-4 && up[0].normal.z < -0.99,
        "{up:?}"
    );
}

/// A text model's levels are simplified and measured as a glTF's: the same
/// parts written as a .glb (the glTF binary the cook makes of the text)
/// cook to the same levels with the same measured errors, above 0 and
/// growing.
#[test]
fn its_lod_errors_are_measured_as_a_gltf_s_are() {
    let text = gear_text();
    let glb = og_cook::cook::model_text::to_glb(&text).unwrap();
    let a = cooked(
        "og-cook-model-text-lod",
        &[("models/gear.ogt", text.as_bytes())],
    );
    let b = cooked("og-cook-model-text-lod-glb", &[("models/gear.glb", &glb)]);
    let load = |root: &Path, id: &str| {
        Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_model(id)
            .unwrap()
    };
    let (ma, mb) = (load(&a, "models/gear.ogt"), load(&b, "models/gear.glb"));
    assert_eq!(ma.primitives.len(), mb.primitives.len());
    for (pa, pb) in ma.primitives.iter().zip(&mb.primitives) {
        assert_eq!(pa.lods, pb.lods);
        assert_eq!(pa.mesh(0), pb.mesh(0));
    }
    let vase = &ma
        .meshes
        .iter()
        .find(|m| m.name == "vase")
        .unwrap()
        .primitives;
    let lods = &ma.primitives[vase.start].lods;
    assert_eq!(lods.len(), 3, "{lods:?}");
    assert_eq!(lods[0].error, 0.0);
    assert!(
        lods[1].error > 0.0 && lods[2].error > lods[1].error,
        "{lods:?}"
    );
    // In the model's units: under the 5 % of the vase's 1.4 m the simplifier was allowed.
    assert!(lods[2].error < 0.07, "{lods:?}");
}

#[test]
fn a_part_placed_three_times_is_one_mesh_under_three_nodes_of_its_pivot() {
    let root = cooked(
        "og-cook-model-text-nodes",
        &[("models/gear.ogt", gear_text().as_bytes())],
    );
    let model = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_model("models/gear.ogt")
        .unwrap();
    let names: Vec<&str> = model.meshes.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["gear", "vase", "bolt"]);
    let axle = model
        .nodes
        .iter()
        .position(|n| n.name == "axle")
        .expect("the pivot is a node");
    assert_eq!(model.nodes[axle].translation, Vec3::new(3.0, 0.0, 0.0));
    let bolts: Vec<_> = model
        .nodes
        .iter()
        .filter(|n| n.name.starts_with("bolt["))
        .collect();
    assert_eq!(bolts.len(), 3);
    for b in &bolts {
        assert_eq!(b.mesh, Some(2));
        assert_eq!(b.parent, Some(axle));
    }
    assert_eq!(bolts[1].translation, Vec3::new(-0.25, 0.433, 0.0));
    let vase = model.nodes.iter().find(|n| n.name == "vase").unwrap();
    assert_eq!(vase.translation, Vec3::new(-3.0, 0.0, 0.0));
    // Materials by name, then glTF's default.
    let materials: Vec<&str> = model.materials.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(materials, ["steel", "clay", "default"]);
}

#[test]
fn past_the_project_s_budgets_it_is_refused_and_a_scene_names_it() {
    let text = gear_text();
    let root = project(
        "og-cook-model-text-budget",
        &[
            ("models/gear.ogt", text.as_bytes()),
            (
                "project.ogt",
                b"type project\nversion 1\n---\nbudgets {\n  triangles 500\n}\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = format!("{:?}", report.errors());
    assert!(
        errors.contains("over the triangles budget of 500"),
        "{errors}"
    );
    assert!(!root.join("cooked/models/gear.ogt.ogb").exists());

    let root = cooked(
        "og-cook-model-text-scene",
        &[
            ("models/gear.ogt", text.as_bytes()),
            (
                "scenes/main.ogt",
                b"type scene\nversion 1\n---\nentity \"gear\" {\n  model \"models/gear.ogt\"\n}\n",
            ),
        ],
    );
    let scene = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_scene("scenes/main.ogt")
        .unwrap();
    assert_eq!(
        scene.entities[0].instance.as_deref(),
        Some("models/gear.ogt")
    );
}

#[test]
fn a_bad_model_says_where() {
    for (text, want) in [
        (
            "part \"a\" {\n  material \"none\"\n  boxGeometry {\n  }\n}\n",
            "'none' is not a material",
        ),
        (
            "part \"a\" {\n  boxGeometry {\n    widht 2\n  }\n}\n",
            "boxGeometry has no field 'widht'",
        ),
        ("part \"a\" {\n  position 1 0 0\n}\n", "has no geometry"),
        (
            "part \"a\" {\n  extrudeGeometry {\n    shape {\n      lineTwo 1 0\n    }\n  }\n}\n",
            "'lineTwo' is not an og.Shape call",
        ),
    ] {
        let text = format!("type model\nversion 1\n---\n{text}");
        let root = project(
            "og-cook-model-text-bad",
            &[("models/a.ogt", text.as_bytes())],
        );
        let errors = format!(
            "{:?}",
            cook_project(&root, &CookOptions::default())
                .unwrap()
                .errors()
        );
        assert!(errors.contains(want), "{want}: {errors}");
    }
}

/// `og cook --report` of a text model: its triangles, its parts and the
/// draws they batch into (the three bolts are one), its BVH's share.
#[test]
fn the_report_counts_its_triangles_draws_and_bvh() {
    let root = project(
        "og-cook-model-text-report",
        &[("models/gear.ogt", gear_text().as_bytes())],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let budgets = og_cook::cook::stats::Budgets::default();
    let json = og_cook::cook::stats::cook_report_json(&report, &root.join("cooked"), &budgets);
    let gear = json["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["source"] == "models/gear.ogt")
        .expect("the model's entry");
    assert_eq!(gear["kind"], "model");
    let m = &gear["model"];
    assert_eq!(m["parts"], 5, "{m}");
    assert_eq!(m["draws"], 3, "{m}");
    let model = Bundle::open(&root.join("cooked"))
        .unwrap()
        .load_model("models/gear.ogt")
        .unwrap();
    let lod0: u64 = model
        .primitives
        .iter()
        .map(|p| p.triangle_count(0) as u64)
        .sum();
    assert_eq!(m["triangles"], lod0);
    assert!(
        m["bvhBytes"].as_u64().unwrap() > 0 && m["bvhShare"].as_f64().unwrap() > 0.0,
        "{m}"
    );
}
