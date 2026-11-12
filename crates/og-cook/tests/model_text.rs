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
//! reported, not enforced; a plant's leaves lose their wind (the anchors
//! follow other vertices after the weld), are simplified, or are drawn as
//! bark is (one face, no light through them); a plant's mistakes cook.

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

/// A flowering shrub, its leaves' translucency and dome normals its own.
const SHRUB: &str = r#"type model
version 1
---
plant "shrub" {
  seed 7
  levels 2
  trunk {
    length 0.4
    radius 0.06
    children 5
  }
  branch1 {
    length 0.9
    children 3
  }
  branch2 {
    length 0.5
  }
  leaves {
    count 12
    size 0.2
    translucency 0.35
    domeNormals 0.8
  }
  flowers {
    share 0.5
    size 0.1
  }
}
"#;

#[test]
fn a_plant_cooks_to_wood_and_leaves_that_sway_and_let_light_through() {
    let root = cooked(
        "og-cook-model-text-plant",
        &[("plants/shrub.ogt", SHRUB.as_bytes())],
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("plants/shrub.ogt").unwrap();
    assert_eq!(model.meshes.len(), 1, "one mesh, the wind's one tree");
    let [wood, leaves] = &model.primitives[..] else {
        panic!("{} primitives", model.primitives.len());
    };

    // The wood: bark, every vertex about a pivot inside the plant, the
    // root's own rigid (the trunk stands where the model does).
    let (wf, leaf) = wood.foliage.as_ref().expect("the wood's wind");
    assert!(!leaf);
    assert_eq!(wf.len(), wood.vertices.len());
    let (lo, hi) = (wood.min - 0.01, wood.max + 0.01);
    for (v, f) in wood.vertices.iter().zip(wf) {
        let pivot = Vec3::new(f.branch[0], f.branch[1], f.branch[2]);
        assert!(pivot.cmpge(lo).all() && pivot.cmple(hi).all(), "{pivot}");
        assert!((0.0..=1.0).contains(&f.branch[3]), "{:?}", f.branch);
        assert_eq!(f.leaf, [0.0; 4], "bark flutters at {:?}", v.position);
    }
    assert!(wood.min.y.abs() < 1e-3, "the root is at {}", wood.min.y);

    // The leaves and petals: flagged, each vertex of the full mesh within
    // a leaf's reach of its own anchor (so the anchors followed their
    // vertices through the weld and the reorder), which is on the wood.
    let (lf, leaf) = leaves.foliage.as_ref().expect("the leaves' wind");
    assert!(leaf);
    let level = |l: usize| {
        let lod = leaves.lods[l];
        &leaves.indices[lod.first..lod.first + lod.count]
    };
    let anchor = |i: u32| Vec3::from_slice(&lf[i as usize].leaf[..3]);
    let at = |i: u32| Vec3::from(leaves.vertices[i as usize].position);
    assert!(level(0).len() > 600, "{}", level(0).len());
    let mut flutter = 0.0f32;
    for &i in level(0) {
        let reach = (at(i) - anchor(i)).length();
        // A leaf of 0.2 at its largest (sizeVariance up to 1.7 times).
        assert!(reach < 0.36, "{reach} from its anchor at {:?}", at(i));
        assert!(anchor(i).cmpge(lo).all() && anchor(i).cmple(hi).all());
        flutter = flutter.max(lf[i as usize].leaf[3]);
    }

    // With distance they are thinned, not simplified: a level keeps every
    // second leaf of the one before, whole, and makes each as much larger
    // about its anchor, so the crown covers as much with half the
    // triangles; its error is how far that moved the farthest corner.
    assert_eq!(leaves.lods.len(), 3, "the mesh, a half and a quarter");
    let leaves_of = |l: usize| -> std::collections::BTreeSet<[u32; 8]> {
        let key = |&i: &u32| {
            let f = &lf[i as usize];
            std::array::from_fn(|k| if k < 4 { f.branch[k] } else { f.leaf[k - 4] }.to_bits())
        };
        level(l).iter().map(key).collect()
    };
    let area = |l: usize| -> f32 {
        let tri = |t: &[u32]| 0.5 * (at(t[1]) - at(t[0])).cross(at(t[2]) - at(t[0])).length();
        level(l).chunks(3).map(tri).sum()
    };
    for (l, stride) in [(1, 2usize), (2, 4)] {
        let (kept, before) = (leaves_of(l), leaves_of(l - 1));
        assert!(kept.is_subset(&before), "level {l} grew a leaf");
        assert_eq!(kept.len(), leaves_of(0).len().div_ceil(stride), "level {l}");
        let cover = area(l) / area(0);
        assert!((cover - 1.0).abs() < 0.15, "level {l} covers {cover}");
        let reach = level(l).iter().map(|&i| (at(i) - anchor(i)).length());
        let moved = reach.fold(0.0, f32::max) * (1.0 - 1.0 / (stride as f32).sqrt());
        let error = leaves.lods[l].error;
        assert!(
            (error - moved).abs() < 1e-4,
            "level {l}: {error} for {moved}"
        );
    }
    assert!(flutter > 0.5, "no leaf flutters: {flutter}");

    // Their materials: bark as any opaque one, the leaves lit from both
    // faces and through, by the canopy's dome.
    let bark = &model.materials[wood.material];
    assert!(!bark.leaf() && !bark.double_sided, "{bark:?}");
    let green = &model.materials[leaves.material];
    assert!(green.double_sided && green.needs_standard());
    assert!(
        (green.translucency - 0.35).abs() < 1e-6 && (green.dome_normals - 0.8).abs() < 1e-6,
        "{} {}",
        green.translucency,
        green.dome_normals
    );

    // What og-render receives: one top for both, the canopy over the root.
    let (mw, ml) = (
        model.mesh_foliage(0).unwrap(),
        model.mesh_foliage(1).unwrap(),
    );
    assert_eq!((mw.top, mw.canopy_center), (ml.top, ml.canopy_center));
    // The top is the full meshes': the thinned levels' larger leaves
    // reach past it, and the plant must sway the same at every level.
    let reach = |i: &u32| at(*i).y;
    let top = level(0).iter().map(reach).fold(wood.max.y, f32::max);
    assert!((mw.top - top).abs() < 1e-3 && top > 1.0, "{} {top}", mw.top);
    assert!(level(2).iter().map(reach).fold(top, f32::max) > top + 1e-3);
    assert!(mw.canopy_center.y > 0.4 && mw.canopy_center.y < top);
    assert!(mw.validate(wood.vertices.len()).is_ok());
    assert!(ml.validate(leaves.vertices.len()).is_ok());
}

/// Ways this could fail: a kind that does not cook or grows nothing; a
/// block's own keys lost to its kind's (or the kind's lost to the
/// defaults'); a colour named that moves the plant's shape.
#[test]
fn a_kind_is_a_plant_s_defaults_and_its_own_keys_change_only_what_they_name() {
    const KINDS: [&str; 12] = [
        "oak", "birch", "willow", "pine", "palm", "bamboo", "cactus", "shrub", "fern", "reed",
        "daisy", "tulip",
    ];
    let text = |kind: &str, more: &str| {
        format!("type model\nversion 1\n---\nplant \"p\" {{\n  kind {kind}\n{more}}}\n")
    };
    let mut files: Vec<(String, String)> = KINDS
        .iter()
        .map(|k| (format!("plants/{k}.ogt"), text(k, "")))
        .collect();
    files.push((
        "plants/yellow.ogt".into(),
        text("tulip", "  flowers {\n    color 1 0.7 0\n  }\n"),
    ));
    files.push(("plants/other.ogt".into(), text("tulip", "  seed 21\n")));
    files.push((
        "plants/tall.ogt".into(),
        text("tulip", "  trunk {\n    length 0.8\n  }\n"),
    ));
    let files: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_bytes()))
        .collect();
    let root = cooked("og-cook-model-text-plant-kinds", &files);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let load = |name: &str| bundle.load_model(&format!("plants/{name}.ogt")).unwrap();
    // Each kind is a plant of its own size: from a tulip to a pine.
    let mut heights = Vec::new();
    for kind in KINDS {
        let model = load(kind);
        let top = model.primitives.iter().map(|p| p.max.y).fold(0.0, f32::max);
        let triangles: usize = model.primitives.iter().map(|p| p.lods[0].count / 3).sum();
        assert!(top > 0.3 && triangles > 40, "{kind}: {top} m, {triangles}");
        // Its wood, and its leaves or petals (a cactus its flowers).
        assert_eq!(model.primitives.len(), 2, "{kind}");
        heights.push(top);
    }
    assert!(heights[3] > 9.0 && heights[11] < 0.6, "{heights:?}");

    // A tulip of another colour is the same tulip; another seed or a
    // longer stem is another.
    let positions = |name: &str| -> Vec<Vec<[f32; 3]>> {
        load(name)
            .primitives
            .iter()
            .map(|p| p.vertices.iter().map(|v| v.position).collect())
            .collect()
    };
    let colors = |name: &str| -> Vec<[f32; 4]> {
        load(name).primitives[1]
            .vertices
            .iter()
            .map(|v| v.color)
            .collect()
    };
    let tulip = positions("tulip");
    assert_eq!(positions("yellow"), tulip);
    assert_ne!(colors("yellow"), colors("tulip"));
    assert_ne!(positions("other"), tulip);
    let top = |name: &str| load(name).primitives[0].max.y;
    assert!(
        (top("tall") - top("tulip") - 0.42).abs() < 0.03,
        "{}",
        top("tall")
    );
}

#[test]
fn a_plant_s_mistakes_are_refused_with_their_line() {
    let refused = |name: &str, text: String, line: &str, says: &str| {
        let root = project(name, &[("plants/bad.ogt", text.as_bytes())]);
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        let errors = report.errors();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(
            errors[0].starts_with("assets/plants/bad.ogt")
                && errors[0].contains(line)
                && errors[0].contains(says),
            "{errors:?}"
        );
    };
    // Leaves that end before they start.
    refused(
        "og-cook-model-text-plant-end",
        SHRUB.replace("count 12\n", "count 12\n    start 0.6\n    end 0.2\n"),
        "line 22",
        "before start",
    );
    // A second plant in one model: the wind bends a model as one tree.
    refused(
        "og-cook-model-text-plant-two",
        format!("{SHRUB}plant \"other\" {{\n  seed 2\n}}\n"),
        "line 30",
        "another plant is another model",
    );
    // Leaves that name a material take their translucency from it.
    refused(
        "og-cook-model-text-plant-material",
        SHRUB
            .replace(
                "plant \"shrub\" {",
                "material \"leaf\" {\n  translucency 0.4\n}\nplant \"shrub\" {",
            )
            .replace("count 12\n", "count 12\n    material \"leaf\"\n"),
        "line 26",
        "is its material's",
    );
    // A kind the cook does not grow: the ones it does, by name.
    refused(
        "og-cook-model-text-plant-kind",
        SHRUB.replace("seed 7\n", "seed 7\n  kind mangrove\n"),
        "line 6",
        "is one of: oak, birch, willow, pine, palm, bamboo, cactus, shrub, fern, reed, daisy, tulip",
    );
    // A number that would grow past what a model holds.
    refused(
        "og-cook-model-text-plant-runaway",
        SHRUB
            .replace("children 5", "children 400")
            .replace("children 3", "children 400"),
        "plant 'shrub'",
        "fewer children, levels or leaves",
    );
}
