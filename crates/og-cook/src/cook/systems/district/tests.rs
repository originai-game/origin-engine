use std::collections::BTreeMap;

use super::build::build;
use super::parse;

const GRAMMAR: &str = "type grammar
version 1
---
param height 9
start Lot
rule Lot {
  extrude height {
    sides Facade
    top Roof
  }
}
rule Facade {
  split y {
    repeat 3 Floor
  }
}
rule Floor {
  split x {
    part 0.5 Wall
    repeat 2.5 Bay
    part 0.5 Wall
  }
}
rule Bay {
  split x {
    fill 1 Wall
    part 1.2 Window
    fill 1 Wall
  }
}
rule Window {
  split y {
    part 0.9 Wall
    part 1.5 Glass
    fill 1 Wall
  }
}
rule Glass {
  detail 1
  recess 0.1
  label glass
}
rule Wall {
  label wall
}
rule Roof {
  label roof
}
";

const LIBRARY: &str = "type materials
version 1
---
material \"brick\" {
  label wall
  label jamb
  tile 1.2 0.9
  weight 2
  vary {
    value 0.1
    warmth 0.05
  }
  color 0.5 0.25 0.2
  roughness 0.9
}
material \"plaster\" {
  label wall
  label jamb
  color 0.7 0.68 0.6
}
material \"glass\" {
  label glass
  color 0.02 0.03 0.04
  roughness 0.05
}
material \"slate\" {
  label roof
  color 0.15 0.15 0.17
}
material \"asphalt\" {
  label road
  color 0.05 0.05 0.05
}
material \"stone\" {
  label pavement
  label kerb
  label ground
  color 0.4 0.4 0.38
}
";

fn district(extra: &str) -> String {
    format!(
        "type district
version 1
---
seed 7
region {{
  point 0 0
  point 160 0
  point 160 120
  point 0 120
}}
streets {{
  grid 80 60
  width 9
  sidewalk 2.5
  kerb 0.15
}}
lots {{
  width 6 12
  depth 12 20
}}
buildings {{
  density 0.8
  height 9 18
  rule \"grammars/terrace.ogt\"
}}
prop \"lamp\" {{
  model \"models/lamp.ogt\"
  every 18
  kerb 0.5
  collider capsule 0.1 4
}}
materials \"materials/town.ogt\"
{extra}"
    )
}

fn files(library: &str) -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([
        (
            "grammars/terrace.ogt".to_string(),
            GRAMMAR.as_bytes().to_vec(),
        ),
        (
            "materials/town.ogt".to_string(),
            library.as_bytes().to_vec(),
        ),
    ])
}

fn run(text: &str, library: &str) -> Result<super::build::Output, String> {
    let f = files(library);
    let read = |id: &str| f.get(id).cloned().ok_or_else(|| format!("no {id}"));
    build("districts/town.ogt", &parse::district(text)?, &read)
}

/// Every entity name in a scene, its children's included, depth first.
fn names(body: &og_core::ogt::Block, out: &mut Vec<String>) {
    for e in body.get_all("entity") {
        out.push(e.string(0).unwrap().to_string());
        names(e.block.as_ref().unwrap(), out);
    }
}

/// What the runtime spawns: no two entities of a district's scenes share
/// a name (names are unique in the whole world, across cells).
fn assert_unique_names(a: &super::build::Output) {
    let mut all = Vec::new();
    for text in std::iter::once(&a.scene).chain(a.scenes.iter().map(|(_, t)| t)) {
        let doc = og_core::ogt::Document::parse(text).unwrap();
        names(&doc.body, &mut all);
    }
    let unique: std::collections::BTreeSet<&String> = all.iter().collect();
    assert_eq!(unique.len(), all.len(), "a name is used twice");
}

#[test]
fn a_district_builds_the_same_bytes_and_another_seed_another_town() {
    let a = run(&district(""), LIBRARY).unwrap();
    let b = run(&district(""), LIBRARY).unwrap();
    assert_eq!(a.scene, b.scene);
    assert_eq!(a.models, b.models);
    let c = run(&district("").replace("seed 7", "seed 8"), LIBRARY).unwrap();
    assert_ne!(a.models, c.models);
}

#[test]
fn every_building_collides_and_every_prop_stands_with_its_collider() {
    let a = run(&district(""), LIBRARY).unwrap();
    let doc = og_core::ogt::Document::parse(&a.scene).unwrap();
    let names: Vec<String> = doc
        .body
        .get_all("entity")
        .map(|e| e.string(0).unwrap().to_string())
        .collect();
    let buildings = names
        .iter()
        .filter(|n| n.starts_with("districts/town.ogt/building-"))
        .count();
    let lamps = names
        .iter()
        .filter(|n| n.starts_with("districts/town.ogt/lamp-"))
        .count();
    assert!(buildings > 10, "{buildings} buildings");
    assert!(lamps > 10, "{lamps} lamps");
    let mut highest = f32::MIN;
    for e in doc.body.get_all("entity") {
        let b = e.block.as_ref().unwrap();
        let name = e.string(0).unwrap();
        if name.starts_with("districts/town.ogt/building-") {
            let c = b.get("Collider").unwrap().block.as_ref().unwrap();
            assert_eq!(c.get("shape").unwrap().word(0).unwrap(), "box");
            let size = c.get("size").unwrap().numbers(0).unwrap();
            // As tall as the tallest building the rules allow, never more.
            assert!(size[1] >= 9.0 - 1e-3 && size[1] <= 18.0 + 1e-3, "{size:?}");
            // Standing on the pavement (the kerb's 0.15) up to the roof.
            let t = b.get("Transform").unwrap().block.as_ref().unwrap();
            let y = t.get("position").unwrap().numbers(0).unwrap()[1];
            assert!((y - size[1] / 2.0 - 0.15).abs() < 1e-3, "{y} {size:?}");
            highest = highest.max(y + size[1] / 2.0);
        }
        if name.starts_with("districts/town.ogt/lamp-") {
            assert_eq!(
                b.get("model").unwrap().string(0).unwrap(),
                "models/lamp.ogt"
            );
            let child = b.get("entity").unwrap().block.as_ref().unwrap();
            let c = child.get("Collider").unwrap().block.as_ref().unwrap();
            assert_eq!(c.get("shape").unwrap().word(0).unwrap(), "capsule");
        }
    }
    assert_unique_names(&a);
    // Without cells it is one model, every part in it.
    assert_eq!(a.models.len(), 1);
    assert!(a.scenes.is_empty());
    // The tallest collider reaches the highest roof and no further.
    let (model, _, _) = gltf::import_slice(&a.models[0].1).unwrap();
    let roof = model.meshes().next().unwrap().primitives();
    let roof = roof
        .map(|p| p.bounding_box().max[1])
        .fold(f32::MIN, f32::max);
    assert!((highest - roof).abs() < 1e-3, "{highest} {roof}");
}

#[test]
fn cells_stream_their_sub_scenes_and_merge_false_keeps_buildings_apart() {
    let a = run(
        &district("cells {\n  size 64\n  load 120\n}\nhlod {\n  size 64\n}\n"),
        LIBRARY,
    )
    .unwrap();
    let doc = og_core::ogt::Document::parse(&a.scene).unwrap();
    let cells: Vec<_> = doc.body.get_all("entity").collect();
    assert!(cells.len() >= 4, "{} cells", cells.len());
    assert_eq!(cells.len(), a.scenes.len());
    for c in &cells {
        let b = c.block.as_ref().unwrap();
        assert!(b.get("StreamCell").is_some() && b.get("Volume").is_some());
        let id = b.get("scene").unwrap().string(0).unwrap();
        let (_, text) = a.scenes.iter().find(|(r, _)| r == id).unwrap();
        let sub = og_core::ogt::Document::parse(text).unwrap();
        let stream = sub.body.get("stream").unwrap().block.as_ref().unwrap();
        assert!(stream.get("cell").is_some());
        assert!(sub.body.get("hlod").is_some());
        // The cell's group packs its own model.
        let assets: Vec<_> = stream
            .get_all("asset")
            .map(|e| e.string(0).unwrap().to_string())
            .collect();
        assert_eq!(assets, [format!("{}.glb", id.trim_end_matches(".ogt"))]);
    }
    assert_unique_names(&a);
    // One model per cell merged; one per building and per cell kept apart.
    assert_eq!(a.models.len(), a.scenes.len());
    let apart = run(
        &district("cells {\n  size 64\n  load 120\n}\nmerge false\n"),
        LIBRARY,
    )
    .unwrap();
    assert_unique_names(&apart);
    assert!(
        apart.models.len() > a.models.len() + 10,
        "{} models",
        apart.models.len()
    );
}

#[test]
fn a_label_no_material_covers_is_refused_naming_the_labels() {
    let without_glass = LIBRARY.replace("label glass", "label window");
    let e = run(&district(""), &without_glass).err().unwrap();
    assert!(
        e.contains("`glass`") && e.contains("window") && e.contains("wall"),
        "{e}"
    );
}

#[test]
fn a_building_draws_one_material_per_label_and_neighbours_differ() {
    // Read each building's wall colour from the merged model's vertex
    // colours: with `vary`, two lots seldom share one.
    let a = run(&district("merge false\n"), LIBRARY).unwrap();
    let mut walls = std::collections::BTreeSet::new();
    for (id, glb) in a.models.iter().filter(|(id, _)| id.contains(".b")) {
        let (doc, buffers, _) = gltf::import_slice(glb).unwrap_or_else(|e| panic!("{id}: {e}"));
        let mesh = doc.meshes().next().unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for p in mesh.primitives() {
            let name = p.material().name().unwrap_or("").to_string();
            if name == "brick" || name == "plaster" {
                seen.insert(name.clone());
                let r = p.reader(|b| Some(&buffers[b.index()]));
                let c = r.read_colors(0).unwrap().into_rgba_f32().next().unwrap();
                walls.insert(c.map(|x| (x * 1e5) as i64));
            }
        }
        assert_eq!(
            seen.len(),
            1,
            "{id} draws its walls in one material: {seen:?}"
        );
    }
    assert!(walls.len() > 5, "{} wall colours", walls.len());
}

#[test]
fn a_detail_part_leaves_the_coarser_level() {
    let a = run(&district(""), LIBRARY).unwrap();
    let (doc, _, _) = gltf::import_slice(&a.models[0].1).unwrap();
    let mats = |m: gltf::Mesh| -> Vec<String> {
        m.primitives()
            .map(|p| p.material().name().unwrap_or("").to_string())
            .collect()
    };
    let lods: Vec<Vec<String>> = doc.meshes().map(mats).collect();
    assert_eq!(lods.len(), 2);
    assert!(lods[0].contains(&"glass".to_string()));
    assert!(!lods[1].contains(&"glass".to_string()));
    assert!(lods[1].contains(&"slate".to_string()));
}
