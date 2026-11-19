//! `og bake lightmaps` on a scene with HLOD (docs/design.md §10.6): its
//! proxies are lit by its static models' bake.
//!
//! Four houses under one leaf on dark ground, lit by a white sky alone: a
//! roof, facing it, takes more light than a wall, which sees half of it
//! and half the ground.
//!
//! Ways this could fail: no proxy gets an entry, or one whose hash is not
//! its geometry's (the runtime would leave it unbaked); a proxy reads its
//! light from the wrong page or at uvs not scaled to the shared page size
//! (black padding, or another chart's light); the transfer reads the
//! models' pages through the wrong uvs, so a proxy's roof is as dark as
//! its walls.

use super::*;
use glam::{Vec2, Vec3};

const HOUSE: &str = r#"type model
version 1
---
material "wall" {
  color 0.8 0.8 0.8
}
material "roof" {
  color 0.8 0.8 0.8
}
part "walls" {
  material "wall"
  position 0 2 0
  boxGeometry {
    width 6
    height 4
    depth 6
  }
}
part "roof" {
  material "roof"
  position 0 5.5 0
  coneGeometry {
    radius 4.2
    height 3
    radialSegments 4
  }
}
"#;

/// Dark ground, not static: it shades the bake and is in no proxy.
const GROUND: &str = r#"type model
version 1
---
material "ground" {
  color 0.05 0.05 0.05
}
part "ground" {
  material "ground"
  quaternion -0.7071068 0 0 0.7071068
  planeGeometry {
    width 200
    height 200
  }
}
"#;

fn town() -> String {
    let mut t = String::from(
        "type scene\nversion 1\n---\nname \"Town\"\nhlod {\n  size 32\n  error 0.25\n  texels 256\n}\nentity \"ground\" {\n  model \"models/ground.ogt\"\n}\n",
    );
    for (i, (x, z)) in [(3, 3), (15, 3), (3, 15), (15, 15)].into_iter().enumerate() {
        t.push_str(&format!(
            "entity \"house-{i}\" {{\n  model \"models/house.ogt\"\n  static true\n  Transform {{\n    position {x} 0 {z}\n  }}\n}}\n"
        ));
    }
    t
}

/// The irradiance a page holds at `uv` (0..1) of page `page`, its mean.
fn read(lm: &og_cook::runtime::CookedLightmap, uv: Vec2, page: usize) -> f32 {
    let x = ((uv.x * lm.width as f32) as u32).min(lm.width - 1);
    let y = ((uv.y * lm.height as f32) as u32).min(lm.height - 1);
    let t = &lm.pages[page][((y * lm.width + x) * 4) as usize..][..3];
    t.iter()
        .map(|&h| og_cook::lightmap::f16_to_f32(h))
        .sum::<f32>()
        / 3.0
}

/// Per triangle of a baked part: its world normal and the light at its
/// centre. `positions`: the part's own vertices, in the scene's space.
fn lit(
    part: &og_cook::lightmap::BakedPart,
    positions: &[Vec3],
    lm: &og_cook::runtime::CookedLightmap,
) -> Vec<(Vec3, f32)> {
    part.indices
        .chunks(3)
        .map(|t| {
            let p = t
                .iter()
                .map(|&i| positions[part.xref[i as usize] as usize])
                .collect::<Vec<_>>();
            let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
            let uv = t
                .iter()
                .map(|&i| Vec2::new(part.uv[i as usize][0], part.uv[i as usize][1]))
                .sum::<Vec2>()
                / 3.0;
            (n, read(lm, uv, part.uv[t[0] as usize][2] as usize))
        })
        .collect()
}

/// The mean light of the roofs' (upward) and the walls' (sideways)
/// triangles.
fn roofs_and_walls(tris: &[(Vec3, f32)]) -> (f32, f32) {
    let mean = |f: &dyn Fn(Vec3) -> bool| {
        let v: Vec<f32> = tris.iter().filter(|t| f(t.0)).map(|t| t.1).collect();
        assert!(!v.is_empty());
        v.iter().sum::<f32>() / v.len() as f32
    };
    (mean(&|n| n.y > 0.3), mean(&|n| n.y.abs() < 0.05))
}

#[test]
fn a_bake_lights_the_proxies_as_it_lights_their_houses() {
    let dir = TempDir::new("og-bake-hlod");
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Town\"\nentry \"main.ts\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("main.ts"), "export default {};").unwrap();
    for (id, text) in [
        ("models/house.ogt", HOUSE.to_string()),
        ("models/ground.ogt", GROUND.to_string()),
        ("scenes/town.ogt", town()),
    ] {
        let p = dir.join("assets").join(id);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    let out = og()
        .args(["bake", "lightmaps", dir.to_str().unwrap()])
        .args(["--samples", "64", "--bounces", "1", "--sky", "1,1,1"])
        .args(["--texels-per-metre", "2", "--max-size", "256"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let lm = bundle
        .load_lightmap(&og_cook::lightmap::lightmap_id("scenes/town.ogt"))
        .unwrap();
    let tree_id = og_cook::hlod::hlod_id("scenes/town.ogt");
    let tree = bundle
        .hlod(&tree_id, &bundle.read(&tree_id).unwrap())
        .unwrap();
    assert!(!tree.nodes.is_empty());

    // The houses, as the bake lit them.
    let house = bundle.load_model("models/house.ogt").unwrap();
    let scene = bundle.load_scene("scenes/town.ogt").unwrap();
    let mut houses = Vec::new();
    for m in lm
        .file
        .models
        .iter()
        .filter(|m| m.asset == "models/house.ogt")
    {
        let e = scene
            .entities
            .iter()
            .position(|e| e.name == m.entity)
            .unwrap();
        let at = tree.members.iter().find(|x| x.entity == e).unwrap().matrix;
        for (part, prim) in m.parts.iter().zip(&house.primitives) {
            let mesh = prim.mesh(0);
            assert_eq!(mesh.vertices.len(), part.vertices as usize);
            let world: Vec<Vec3> = mesh
                .vertices
                .iter()
                .map(|v| at.transform_point3(Vec3::from(v.position)))
                .collect();
            houses.extend(lit(part, &world, &lm));
        }
    }
    let (house_roofs, house_walls) = roofs_and_walls(&houses);
    assert!(
        house_roofs > house_walls * 1.2,
        "the houses' roofs {house_roofs}, walls {house_walls}"
    );

    for n in 0..tree.nodes.len() {
        let pid = og_cook::hlod::node_id("scenes/town.ogt", n);
        let entry = lm
            .file
            .models
            .iter()
            .find(|m| m.asset == pid)
            .unwrap_or_else(|| panic!("no entry for {pid}"));
        assert_eq!(
            Some(entry.hash),
            bundle.model_geometry_hash(&pid).ok(),
            "{pid}"
        );
        let model = bundle.load_model(&pid).unwrap();
        let world = model.node_world()[model.scene_nodes(model.default_scene.unwrap())[0]];
        let mesh = model.primitives[0].mesh(0);
        let positions: Vec<Vec3> = mesh
            .vertices
            .iter()
            .map(|v| world.transform_point3(Vec3::from(v.position)))
            .collect();
        let (roofs, walls) = roofs_and_walls(&lit(&entry.parts[0], &positions, &lm));
        for (what, proxy, model) in [("roofs", roofs, house_roofs), ("walls", walls, house_walls)] {
            assert!(
                (proxy - model).abs() <= model * 0.25,
                "{pid}: its {what} take {proxy}, the houses' {model}"
            );
        }
    }
}
