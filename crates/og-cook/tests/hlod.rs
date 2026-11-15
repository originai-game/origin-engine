//! HLOD through the cook (docs/design.md §10.6): a scene's `hlod { }`
//! block merges its static models into proxies, one cooked model a node.
//!
//! Ways this could fail: a proxy stands farther off its members than the
//! error the runtime switches by says (a part dropped and not counted, a
//! child's error forgotten by its parent); a member is in no node, or in
//! two that are not one above the other; a proxy's texture shows the wrong
//! surface (its page read upside down, a roof's colour on a wall); an
//! edit to one house cooks every proxy again, or none; a model that
//! cannot merge is merged anyway, or vanishes without a word; a bad block
//! cooks silently.

use glam::Vec3;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, Report, cook_project};
use std::path::{Path, PathBuf};

/// Linear colours of a house's walls and roof.
const WALL: [f32; 3] = [0.6, 0.08, 0.02];
const ROOF: [f32; 3] = [0.02, 0.1, 0.5];

/// A house on the ground at its origin: walls 6 × 4 × 6 m, a pyramid roof
/// 3 m high over them, and on its tip a ball 0.6 m across of about 900
/// triangles, the detail a proxy has no use for.
fn house() -> String {
    format!(
        r#"type model
version 1
---
material "wall" {{
  color {} {} {}
  roughness 0.9
}}
material "roof" {{
  color {} {} {}
  roughness 0.4
}}
part "walls" {{
  material "wall"
  position 0 2 0
  boxGeometry {{
    width 6
    height 4
    depth 6
  }}
}}
part "roof" {{
  material "roof"
  position 0 5.5 0
  coneGeometry {{
    radius 4.2
    height 3
    radialSegments 4
  }}
}}
part "ball" {{
  material "roof"
  position 0 7.3 0
  sphereGeometry {{
    radius 0.3
  }}
}}
"#,
        WALL[0], WALL[1], WALL[2], ROOF[0], ROOF[1], ROOF[2]
    )
}

/// Where house `(i, j)` of the town stands: 12 m apart from (3, 0, 3).
fn at(i: u32, j: u32) -> Vec3 {
    Vec3::new(3.0 + 12.0 * i as f32, 0.0, 3.0 + 12.0 * j as f32)
}

/// A town of 6 × 6 houses over 72 m, with 32 m leaves: four leaves of
/// nine houses under one root. `moved` shifts house (0, 0) along x.
fn town(block: &str, moved: f32, extra: &str) -> String {
    let mut text = format!("type scene\nversion 1\n---\nname \"Town\"\n{block}\n");
    for j in 0..6 {
        for i in 0..6 {
            let mut p = at(i, j);
            if (i, j) == (0, 0) {
                p.x += moved;
            }
            text.push_str(&format!(
                "entity \"house-{i}-{j}\" {{\n  model \"models/house.ogt\"\n  static true\n  Transform {{\n    position {} {} {}\n  }}\n}}\n",
                p.x, p.y, p.z
            ));
        }
    }
    text.push_str(extra);
    text
}

const BLOCK: &str = "hlod {\n  size 32\n  error 0.25\n  texels 512\n}";

fn project(name: &str, scene: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "models/house.ogt", &house());
    write(&root, "scenes/town.ogt", scene);
    root
}

fn write(root: &Path, id: &str, text: &str) {
    let p = root.join("assets").join(id);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

/// og.Model's fields these scenes set, so an entity may leave one out.
fn options() -> CookOptions {
    use og_core::{FieldSchema, FieldType, Schema};
    CookOptions {
        schemas: vec![Schema {
            name: "Model".into(),
            version: 1,
            doc: String::new(),
            fields: vec![
                FieldSchema::new("static", FieldType::Bool),
                FieldSchema::new("visible", FieldType::Bool),
                FieldSchema::new("tint", FieldType::Vec3),
            ],
        }],
        ..CookOptions::default()
    }
}

fn cook(root: &Path) -> Report {
    cook_project(root, &options()).unwrap()
}

fn notes(report: &Report) -> Vec<String> {
    report
        .assets
        .iter()
        .find(|a| a.source == "scenes/town.ogt")
        .expect("the scene is reported")
        .notes
        .clone()
}

fn srgb(linear: f32) -> f32 {
    if linear <= 0.003_130_8 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

#[test]
fn a_town_cooks_to_a_tree_whose_proxies_stand_within_their_error_of_the_houses() {
    let root = project("og-cook-hlod-town", &town(BLOCK, 0.0, ""));
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = og_cook::hlod::hlod_id("scenes/town.ogt");
    let tree = bundle.hlod(&id, &bundle.read(&id).unwrap()).unwrap();

    // Four leaves of nine under a root of all thirty-six.
    assert_eq!(tree.nodes.len(), 5, "{:?}", tree.nodes);
    assert_eq!(tree.members.len(), 36);
    assert_eq!(tree.children(None).collect::<Vec<_>>(), [0]);
    let leaves: Vec<usize> = tree.children(Some(0)).collect();
    assert_eq!(leaves, [1, 2, 3, 4]);
    assert_eq!((tree.nodes[0].level, tree.nodes[0].members), (1, (0, 36)));
    let mut seen = vec![0; 36];
    for &l in &leaves {
        let n = &tree.nodes[l];
        assert_eq!((n.level, n.members.1 - n.members.0), (0, 9));
        assert_eq!(n.parent, Some(0));
        for count in &mut seen[n.members.0..n.members.1] {
            *count += 1;
        }
    }
    assert_eq!(seen, vec![1; 36], "each house in one leaf");
    // The members are the scene's entities, where the scene put them.
    let scene = bundle.load_scene("scenes/town.ogt").unwrap();
    let mut stood: Vec<(String, [i32; 3])> = tree
        .members
        .iter()
        .map(|m| {
            let p = m.matrix.transform_point3(Vec3::ZERO);
            (
                scene.entities[m.entity].name.clone(),
                p.to_array().map(|x| x.round() as i32),
            )
        })
        .collect();
    stood.sort();
    let mut want: Vec<(String, [i32; 3])> = (0..6)
        .flat_map(|j| (0..6).map(move |i| (i, j)))
        .map(|(i, j)| {
            (
                format!("house-{i}-{j}"),
                at(i, j).to_array().map(|x| x.round() as i32),
            )
        })
        .collect();
    want.sort();
    assert_eq!(stood, want);

    let house_triangles = bundle
        .load_model("models/house.ogt")
        .unwrap()
        .primitives
        .iter()
        .map(|p| p.triangle_count(0))
        .sum::<usize>();
    for (n, node) in tree.nodes.iter().enumerate() {
        let budget = 0.25 * (1 << node.level) as f32;
        assert!(
            node.error > 0.0 && node.error <= budget * 1.001,
            "node {n}: {} m against a budget of {budget}",
            node.error
        );
        let pid = og_cook::hlod::node_id("scenes/town.ogt", n);
        let (model, textures) = bundle.load_model_with_textures(&pid).expect("decodes");
        assert_eq!(model.primitives.len(), 1);
        assert!(
            bundle.output(&pid).unwrap().group.is_none(),
            "a proxy is in the first group"
        );
        // The proxy in the scene's space: its one node carries its centre.
        let scene_nodes = model.scene_nodes(model.default_scene.unwrap());
        assert_eq!(scene_nodes.len(), 1);
        let world = model.node_world()[scene_nodes[0]];
        let prim = &model.primitives[0];
        let mut mesh = prim.mesh(0);
        for v in &mut mesh.vertices {
            v.position = world.transform_point3(Vec3::from(v.position)).to_array();
        }
        let members = &tree.members[node.members.0..node.members.1];
        let triangles = prim.triangle_count(0);
        assert!(
            triangles * 4 < members.len() * house_triangles,
            "node {n}: {triangles} triangles for {} houses of {house_triangles}",
            members.len()
        );
        assert_eq!(prim.lods.len(), 1, "the tree is its levels of detail");

        // Nothing of a house stands farther from the proxy than its error
        // says: the walls' corners, the roof's corners and tip, the ball's
        // top (which the proxy drops).
        let bvh = og_render_data::geometry::Bvh::build(&mesh);
        for m in members {
            let mut points = vec![Vec3::new(0.0, 7.0, 0.0), Vec3::new(0.0, 7.6, 0.0)];
            for k in 0..8 {
                points.push(Vec3::new(
                    if k & 1 == 0 { -3.0 } else { 3.0 },
                    if k & 2 == 0 { 0.0 } else { 4.0 },
                    if k & 4 == 0 { -3.0 } else { 3.0 },
                ));
            }
            for p in points {
                let d = bvh.distance(m.matrix.transform_point3(p)).unwrap();
                assert!(
                    d <= node.error + 2e-3,
                    "node {n}: {p} of a house stands {d} m off a proxy said to be within {}",
                    node.error
                );
            }
        }
        // And no proxy vertex stands outside its houses.
        for v in &mesh.vertices {
            let p = Vec3::from(v.position);
            let inside = members.iter().any(|m| {
                let l = m.matrix.inverse().transform_point3(p);
                l.x.abs() <= 4.21 && l.z.abs() <= 4.21 && (-0.01..=7.61).contains(&l.y)
            });
            assert!(inside, "node {n}: a vertex at {p} belongs to no house");
            assert!((p - node.center).length() <= node.radius + 0.05);
        }

        // Its base colour shows walls where the walls are and roofs where
        // the roofs are: read at the middle of each proxy triangle that is
        // plainly one or the other.
        let material = &model.materials[prim.material];
        let base = &textures[material.base_color_texture.expect("a base colour page")];
        assert!(material.metallic_roughness_texture.is_some());
        assert!(material.emissive_texture.is_none(), "nothing emits");
        let (mut walls, mut roofs) = (0, 0);
        for t in mesh.indices.chunks(3) {
            let v = [0, 1, 2].map(|k| mesh.vertices[t[k] as usize]);
            let p = v.map(|v| Vec3::from(v.position));
            let normal = (p[1] - p[0]).cross(p[2] - p[0]);
            if normal.length() < 2.0 {
                continue;
            }
            let normal = normal.normalize();
            let c = (p[0] + p[1] + p[2]) / 3.0;
            let want = if normal.y.abs() < 0.05 && (1.0..3.0).contains(&c.y) {
                walls += 1;
                WALL
            } else if (0.3..0.95).contains(&normal.y) && (4.6..6.2).contains(&c.y) {
                roofs += 1;
                ROOF
            } else {
                continue;
            };
            let uv =
                (glam::Vec2::from(v[0].uv) + glam::Vec2::from(v[1].uv) + glam::Vec2::from(v[2].uv))
                    / 3.0;
            let x = ((uv.x * base.width as f32) as u32).min(base.width - 1);
            let y = ((uv.y * base.height as f32) as u32).min(base.height - 1);
            let at = ((y * base.width + x) * 4) as usize;
            for k in 0..3 {
                let got = base.rgba[at + k] as f32 / 255.0;
                assert!(
                    (got - srgb(want[k])).abs() < 0.12,
                    "node {n}: channel {k} at {c} (normal {normal}) is {got}, the {} is {}",
                    if want == WALL { "wall" } else { "roof" },
                    srgb(want[k])
                );
            }
        }
        assert!(
            walls >= members.len() * 4 && roofs >= members.len() * 2,
            "node {n}: {walls} wall and {roofs} roof triangles read"
        );
    }
    let said = notes(&report);
    assert!(
        said.iter()
            .any(|n| n.starts_with("hlod: 5 proxies over 36 models in 2 level(s)")),
        "{said:?}"
    );
}

/// The outputs of the scene's HLOD that cooked in `report`, sorted.
fn cooked(report: &Report) -> Vec<String> {
    let mut out: Vec<String> = report
        .assets
        .iter()
        .flat_map(|a| &a.outputs)
        .filter(|o| o.cooked && og_cook::hlod::is_hlod_output(&o.id))
        .map(|o| o.id.trim_start_matches("scenes/town.ogt#").to_string())
        .collect();
    out.sort();
    out
}

#[test]
fn only_the_nodes_whose_houses_moved_cook_again() {
    let root = project("og-cook-hlod-incremental", &town(BLOCK, 0.0, ""));
    let first = cook(&root);
    assert!(first.ok(), "{:?}", first.errors());
    assert_eq!(
        cooked(&first).len(),
        1 + 5 * 3,
        "the tree, 5 models, 10 pages"
    );
    assert_eq!(cooked(&cook(&root)), Vec::<String>::new());
    // A light added to the scene moves no house.
    write(
        &root,
        "scenes/town.ogt",
        &town(
            BLOCK,
            0.0,
            "entity \"lamp\" {\n  Transform {\n    position 1 9 1\n  }\n}\n",
        ),
    );
    let lamp = cook(&root);
    assert!(lamp.ok(), "{:?}", lamp.errors());
    assert_eq!(cooked(&lamp), Vec::<String>::new());
    // House (0, 0) a metre along x: its leaf (node 1) and the root.
    write(&root, "scenes/town.ogt", &town(BLOCK, 1.0, ""));
    let moved = cook(&root);
    assert!(moved.ok(), "{:?}", moved.errors());
    assert_eq!(
        cooked(&moved),
        [
            "hlod",
            "hlod-0",
            "hlod-0-base",
            "hlod-0-surface",
            "hlod-1",
            "hlod-1-base",
            "hlod-1-surface"
        ]
    );
    // The same scene cooked elsewhere is the same bytes.
    let again = project("og-cook-hlod-incremental-b", &town(BLOCK, 1.0, ""));
    assert!(cook(&again).ok());
    for id in ["hlod", "hlod-0", "hlod-0-base", "hlod-3-surface"] {
        let file = format!("cooked/scenes/town.ogt.{id}.ogb");
        assert_eq!(
            std::fs::read(root.join(&file)).unwrap(),
            std::fs::read(again.join(&file)).unwrap(),
            "{file}"
        );
    }
    // Without the block the scene has no HLOD, and its outputs go.
    write(&root, "scenes/town.ogt", &town("", 1.0, ""));
    let none = cook(&root);
    assert!(none.ok(), "{:?}", none.errors());
    assert!(!root.join("cooked/scenes/town.ogt.hlod.ogb").exists());
    assert!(!root.join("cooked/scenes/town.ogt.hlod-0.ogb").exists());
}

#[test]
fn a_model_that_cannot_merge_draws_itself_and_the_cook_says_which() {
    // Beside the town: a house with a tint of its own, one hidden, one not
    // static, and one past the widest cell from every other (32 m leaves
    // under ten levels: 32 km).
    let extra = r#"entity "tinted" {
  model "models/house.ogt"
  Model {
    static true
    tint 1 0.5 0.5
  }
  Transform {
    position 15 0 15
  }
}
entity "hidden" {
  model "models/house.ogt"
  static true
  Model {
    visible false
  }
  Transform {
    position 27 0 27
  }
}
entity "moving" {
  model "models/house.ogt"
  Transform {
    position 39 0 39
  }
}
entity "hermit" {
  model "models/house.ogt"
  static true
  Transform {
    position 40000 0 40000
  }
}
"#;
    let root = project("og-cook-hlod-left", &town(BLOCK, 0.0, extra));
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = og_cook::hlod::hlod_id("scenes/town.ogt");
    let tree = bundle.hlod(&id, &bundle.read(&id).unwrap()).unwrap();
    assert_eq!(tree.members.len(), 36, "the town's houses and no other");
    let said = notes(&report);
    let left = said
        .iter()
        .find(|n| n.contains("draw themselves"))
        .unwrap_or_else(|| panic!("{said:?}"));
    assert!(
        left.starts_with("hlod: 3 static model(s) draw themselves"),
        "{left}"
    );
    for why in ["1 hidden", "1 with a tint", "1 with no other model near"] {
        assert!(left.contains(why), "{left}");
    }
}

#[test]
fn a_block_the_cook_cannot_read_is_refused_with_its_line() {
    for (block, want) in [
        (
            "hlod {\n  size 1\n}",
            "line 5: hlod.size 1 is a leaf cell's side in metres, 4 to 4096",
        ),
        (
            "hlod {\n  size 32\n  error 20\n}",
            "at most a quarter of size (8)",
        ),
        (
            "hlod {\n  cells 4\n}",
            "line 6: 'cells' is not an hlod entry: size, error, texels",
        ),
        ("hlod {\n}\nhlod {\n}", "line 7: a scene has one hlod block"),
        ("hlod 64", "line 5: hlod takes a block"),
    ] {
        let root = project("og-cook-hlod-refused", &town(block, 0.0, ""));
        let errors = cook(&root).errors().join("\n");
        assert!(errors.contains(want), "{block:?}: {errors}");
    }
}
