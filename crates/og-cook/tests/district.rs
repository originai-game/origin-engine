//! Districts (`type district`): a district cooks through the whole cook
//! into a scene of streaming cells, each a cell group packing its own
//! model, whose materials read the library's textures; the grammars and
//! the library are part of it, not assets of their own.

use og_cook::cook::{CookOptions, Report, cook_project};
use og_cook::runtime::Bundle;
use std::path::{Path, PathBuf};

fn project(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("og-cook-district-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for d in ["districts", "grammars", "materials", "models", "textures"] {
        std::fs::create_dir_all(root.join("assets").join(d)).unwrap();
    }
    root
}

fn write(root: &Path, id: &str, text: &str) {
    std::fs::write(root.join("assets").join(id), text).unwrap();
}

fn cook(root: &Path) -> Report {
    let options = CookOptions {
        threads: 2,
        // As `og cook` passes them: the colliders are og.Collider rows.
        schemas: og_cook::cook::engine_schemas(),
        ..CookOptions::default()
    };
    cook_project(root, &options).expect("the cook runs")
}

const GRAMMAR: &str = "type grammar\nversion 1\n---\nparam height 8\nstart Lot\nrule Lot {\n  extrude height {\n    sides Wall\n    top Roof\n  }\n}\nrule Wall {\n  label wall\n}\nrule Roof {\n  roof hip 30 {\n    slope Tiles\n  }\n}\nrule Tiles {\n  label roof\n}\n";

const LIBRARY: &str = "type materials\nversion 1\n---\nmaterial \"brick\" {\n  label wall\n  tile 1 1\n  map \"textures/brick.png\"\n}\nmaterial \"slate\" {\n  label roof\n  color 0.1 0.1 0.12\n}\nmaterial \"asphalt\" {\n  label road\n  color 0.05 0.05 0.05\n}\nmaterial \"stone\" {\n  label pavement\n  label kerb\n  label ground\n  color 0.4 0.4 0.4\n}\n";

const LAMP: &str = "type model\nversion 1\n---\nmaterial \"iron\" {\n  color 0.05 0.05 0.05\n}\npart \"post\" {\n  material \"iron\"\n  cylinderGeometry {\n    radiusTop 0.06\n    radiusBottom 0.08\n    height 4\n  }\n  at {\n    position 0 2 0\n  }\n}\n";

const DISTRICT: &str = "type district\nversion 1\n---\nseed 3\nregion {\n  point 0 0\n  point 120 0\n  point 120 80\n  point 0 80\n}\nstreets {\n  grid 60 40\n  width 8\n  sidewalk 2\n  kerb 0.15\n}\nlots {\n  width 6 12\n  depth 10 16\n}\nbuildings {\n  density 1\n  height 6 12\n  rule \"grammars/house.ogt\"\n}\nprop \"lamp\" {\n  model \"models/lamp.ogt\"\n  every 20\n  kerb 0.4\n  collider capsule 0.1 3.8\n}\nmaterials \"materials/town.ogt\"\ncells {\n  size 64\n  load 100\n}\n";

#[test]
fn streets_carve_their_terrain_without_cutting_block_interiors() {
    use og_ai::nav::cooked::BundleNav;
    use og_core::glam::{Vec2, Vec3};
    let area = "region {\n point -48 -48\n point 48 -48\n point 48 48\n point -48 48\n}\nstreets {\n grid 48 48\n width 8\n sidewalk 2\n kerb 0.15\n}\n";
    let path =
        "along {\n point -48 0\n point 0 0\n point 32 32\n width 8\n sidewalk 2\n depth 16\n}\n";
    for (name, region, road, untouched) in [
        ("area", area, Vec2::new(24.0, 0.0), Vec2::new(24.0, 24.0)),
        (
            "concave",
            &area.replace(" point 48 48", " point 48 0\n point 0 0\n point 0 48"),
            Vec2::new(0.0, -24.0),
            Vec2::new(-24.0, 24.0),
        ),
        ("path", path, Vec2::new(16.0, 16.0), Vec2::new(24.0, -24.0)),
    ] {
        let root = project(&format!("carve-{name}"));
        write(&root, "grammars/house.ogt", GRAMMAR);
        write(
            &root,
            "materials/town.ogt",
            &LIBRARY.replace("  map \"textures/brick.png\"\n", ""),
        );
        let district = format!(
            "type district\nversion 1\n---\nseed 3\n{region}lots {{\n width 6 12\n depth 10 16\n}}\nbuildings {{\n density 1\n height 6 12\n rule \"grammars/house.ogt\"\n}}\nmaterials \"materials/town.ogt\"\nterrain \"land.ogt\" {{\n depth 0.5\n}}\n"
        );
        write(&root, "districts/town.ogt", &district);
        write(
            &root,
            "land.ogt",
            "type terrain\nversion 1\n---\nheightmap \"h.png\"\nheight -2 8\nsize 128 128\ntile 64\n",
        );
        // One already-low point must stay low. Everything else is above
        // the street; the asymmetric points distinguish a street from a block.
        let image = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_fn(129, 129, |x, y| {
            image::Luma([if (x, y) == (40, 64) { 0 } else { 65535 }])
        });
        image.save(root.join("assets/h.png")).unwrap();
        write(
            &root,
            "walk.ogt",
            "type navmesh\nversion 1\n---\nterrain \"land.ogt\"\ncellSize 0.5\ncellHeight 0.05\nbounds {\n min -36 -10 -2\n max -30 12 2\n}\nagent {\n radius 0.2\n height 1\n}\n",
        );
        for depth in [0.5, 4.0] {
            write(
                &root,
                "districts/town.ogt",
                &district.replace("depth 0.5", &format!("depth {depth}")),
            );
            let report = cook(&root);
            assert!(report.ok(), "{name}: {:?}", report.errors());
            let bundle = Bundle::open(&root.join("cooked")).unwrap();
            let terrain = bundle.load_terrain("land.ogt").unwrap().field().unwrap();
            assert!(
                (terrain.height(road) + depth).abs() < 0.001,
                "{name}: street not carved to {}",
                -depth
            );
            assert!(
                (terrain.height(untouched) - 8.0).abs() < 0.001,
                "{name}: block/outside was carved"
            );
            assert!((terrain.height(Vec2::new(-24.0, 0.0)) - (-2.0f32).min(-depth)).abs() < 0.001);
            assert!((terrain.height(Vec2::new(60.0, 40.0)) - 8.0).abs() < 0.001);
            // Nav reads the same flattened surface, including after only
            // the district changes. A bake of the uncarved heightmap fails.
            let bytes = bundle.read("walk.ogt").unwrap();
            let (field, tiles) = bundle.navmesh("walk.ogt", &bytes).unwrap();
            let nav = og_ai::nav::NavMesh::from_tiles(field, tiles).unwrap();
            let p = nav
                .find_path(
                    Vec3::new(-34.0, -depth, 0.0),
                    Vec3::new(-32.0, -depth, 0.0),
                    Vec3::splat(0.5),
                )
                .expect("street is walkable");
            assert!(p.complete);
            // An unchanged cook keeps the same public height samples.
            assert!(cook(&root).ok());
            assert_eq!(
                terrain,
                Bundle::open(&root.join("cooked"))
                    .unwrap()
                    .load_terrain("land.ogt")
                    .unwrap()
                    .field()
                    .unwrap()
            );
        }
        write(
            &root,
            "districts/town.ogt",
            &district.replace("terrain \"land.ogt\" {\n depth 0.5\n}\n", ""),
        );
        assert!(cook(&root).ok());
        let field = Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_terrain("land.ogt")
            .unwrap()
            .field()
            .unwrap();
        assert_eq!(
            field.height(road),
            8.0,
            "removing the carve restores the source"
        );
    }
}

#[test]
fn a_district_cooks_into_streaming_cells_of_its_own_models() {
    let root = project("cells");
    write(&root, "grammars/house.ogt", GRAMMAR);
    let library = LIBRARY.replace("  tile 1 1", "  tile 1 12\n  grime 0.7\n  grimeColor 0.08 0.05 0.02\n  grimeHeight 1.8\n  stains 0.3\n  stochastic 0.6\n  brief \"red brick facade\"\n  facade true\n  relief 0.03\n  normalMap \"textures/brick-n.png\"\n  roughnessRange 0.3 0.9\n  metalnessMap \"textures/brick-mr.png\"\n  occlusion 0.7 2\n  aoMap \"textures/brick-ao.png\"");
    write(&root, "materials/town.ogt", &library);
    write(&root, "models/lamp.ogt", LAMP);
    write(&root, "districts/town.ogt", DISTRICT);
    let px: Vec<u8> = (0..16 * 16)
        .flat_map(|i| [150, 70 + (i % 7) as u8 * 10, 50, 255])
        .collect();
    image::save_buffer(
        root.join("assets/textures/brick.png"),
        &px,
        16,
        16,
        image::ColorType::Rgba8,
    )
    .unwrap();
    let source = std::fs::read(root.join("assets/textures/brick.png")).unwrap();
    let request = og_cook::cook::materials::requests(&library)
        .unwrap()
        .remove(0);
    for (id, bytes) in og_cook::cook::materials::make(&request, &source)
        .unwrap()
        .files
    {
        std::fs::write(root.join("assets").join(id), bytes).unwrap();
    }
    let report = cook(&root);
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    // The grammar and the library are the district's, not assets.
    assert!(bundle.output("grammars/house.ogt").is_err());
    assert!(bundle.output("materials/town.ogt").is_err());
    let town = bundle.load_scene("districts/town.ogt").unwrap();
    let cells: Vec<&str> = town
        .entities
        .iter()
        .filter_map(|e| e.subscene.as_deref())
        .collect();
    assert!(cells.len() >= 2, "{cells:?}");
    let mut buildings = 0;
    for cell in &cells {
        // Each cell's sub-scene and model are packed in its own group.
        let out = bundle.output(cell).unwrap();
        let group = out.group.clone().expect("a cell group");
        let model = format!("{}.glb", cell.trim_end_matches(".ogt"));
        assert_eq!(
            bundle.output(&model).unwrap().group.as_deref(),
            Some(group.as_str())
        );
        let scene = bundle.load_scene(cell).unwrap();
        buildings += scene
            .entities
            .iter()
            .filter(|e| e.name.contains("/building-"))
            .count();
        assert!(scene.components.iter().any(|t| t.name == "Collider"));
        let m = bundle.load_model(&model).unwrap();
        let names: Vec<&str> = m.materials.iter().map(|x| x.name.as_str()).collect();
        assert!(names.contains(&"asphalt"), "{names:?}");
        if names.contains(&"brick") {
            for p in m
                .primitives
                .iter()
                .filter(|p| m.materials[p.material].name == "brick")
            {
                let bottom = p
                    .vertices
                    .iter()
                    .map(|v| v.uv[1])
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!(
                    bottom.abs() < 1e-6,
                    "grime reads metric height before texture transforms: {bottom}"
                );
                // A 12 m facade still tiles at 12 m after the UV-preserving
                // cook, while the raw v is height above each building's base.
                let brick = &m.materials[p.material];
                let t = brick.uv_transforms[0];
                assert_eq!(t.apply([0.0, 0.0])[1], 1.0);
                assert!((t.apply([0.0, -12.0])[1]).abs() < 1e-6);
                let raw_top = p.vertices.iter().map(|v| v.uv[1]).fold(0.0, f32::min);
                assert!(
                    raw_top < -5.0,
                    "height was baked into texture units: {raw_top}"
                );
            }
            let brick = m.materials.iter().find(|m| m.name == "brick").unwrap();
            assert_eq!(brick.grime, 0.7);
            assert_eq!(brick.grime_color, [0.08, 0.05, 0.02]);
            assert_eq!(brick.grime_height, 1.8);
            assert_eq!(brick.stains, 0.3);
            assert_eq!(brick.stochastic, 0.6);
            for slot in [0, 1, 2, 4] {
                assert_eq!(brick.uv_transforms[slot], brick.uv_transforms[0]);
            }
            // Each slot must survive the district's asset-id rewriting
            // and packing, not just the ordinary model parser.
            let mut slots = std::collections::BTreeSet::new();
            for slot in [
                brick.base_color_texture,
                brick.normal_texture,
                brick.metallic_roughness_texture,
                brick.occlusion_texture,
            ] {
                assert!(
                    slots.insert(slot.expect("a derived map")),
                    "each slot has its own map"
                );
            }
        }
    }
    assert!(buildings > 10, "{buildings} buildings");
    // A second cook of nothing changed is the same bytes.
    let first = std::fs::read(
        root.join("cooked")
            .join(&bundle.output(cells[0]).unwrap().file),
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(root.join("cooked"));
    let _ = std::fs::remove_dir_all(root.join(".og"));
    assert!(cook(&root).ok());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let again = std::fs::read(
        root.join("cooked")
            .join(&bundle.output(cells[0]).unwrap().file),
    )
    .unwrap();
    assert_eq!(first, again);
}

#[test]
fn a_district_naming_a_missing_grammar_fails_naming_it() {
    let root = project("missing");
    write(&root, "materials/town.ogt", LIBRARY);
    write(&root, "models/lamp.ogt", LAMP);
    write(&root, "districts/town.ogt", DISTRICT);
    let report = cook(&root);
    let errors = format!("{:?}", report.errors());
    assert!(
        !report.ok() && errors.contains("grammars/house.ogt"),
        "{errors}"
    );
}
