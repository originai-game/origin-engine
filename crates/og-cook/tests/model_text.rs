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
use std::path::PathBuf;

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

#[test]
fn past_the_project_s_budgets_it_is_cooked_with_a_warning_and_a_scene_names_it() {
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
    // The budgets are reported numbers: the model is made, and the cook says it is over.
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    assert!(
        report.warnings.iter().any(|w| w
            .starts_with("assets/models/gear.ogt: warning: over budget")
            && w.contains("over the triangles budget of 500")),
        "{:?}",
        report.warnings
    );
    assert!(root.join("cooked/models/gear.ogt.ogb").exists());

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
