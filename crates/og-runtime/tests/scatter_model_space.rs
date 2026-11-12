//! A scatter uses the model's placed geometry, not the primitive's raw units.
use glam::Vec3;
use og_render::RenderScene;
use og_runtime::*;
use serde_json::{Value, json};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

struct Disk;
impl Hooks for Disk {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

type DrawBounds = (og_render::ScatterDraw, (Vec3, Vec3));
struct Seen(Rc<RefCell<Vec<DrawBounds>>>);
impl RenderBackend for Seen {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .world
            .scatters
            .iter()
            .map(|draw| (*draw, scene.mesh(draw.mesh).unwrap().bounds().unwrap()))
            .collect();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

fn project(test: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("scatter-model-space-{test}-{}", std::process::id()));
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    let positions: [f32; 12] = [-1., -1., -1., 1., -1., -1., 0., 1., -1., 0., 0., 1.];
    let indices: [u16; 12] = [0, 2, 1, 0, 1, 3, 1, 2, 3, 2, 0, 3];
    let bytes: Vec<_> = positions
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .chain(indices.iter().flat_map(|v| v.to_le_bytes()))
        .collect();
    std::fs::write(assets.join("mesh.bin"), bytes).unwrap();
    let gltf = json!({
        "asset":{"version":"2.0"}, "scene":0, "scenes":[{"nodes":[0,1]}],
        "nodes":[
            {"mesh":0,"translation":[0.2,0.4,-0.1],"scale":[0.17,0.23,0.31]},
            {"mesh":0,"translation":[-0.5,0.6,0.2],"scale":[0.1,0.1,0.1]}
        ],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}],
        "buffers":[{"uri":"mesh.bin","byteLength":72}],
        "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":48},{"buffer":0,"byteOffset":48,"byteLength":24}],
        "accessors":[
            {"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[-1,-1,-1],"max":[1,1,1]},
            {"bufferView":1,"componentType":5123,"count":12,"type":"SCALAR"}
        ]
    });
    std::fs::write(assets.join("scaled.gltf"), gltf.to_string()).unwrap();
    let card = "type import\nversion 1\n---\nmodel {\n impostor true\n impostorGrid 4\n impostorSize 64\n}\n";
    std::fs::write(assets.join("scaled.gltf.ogt"), card).unwrap();
    // The same model with no impostor, and a grown plant (wood and leaves)
    // with one (its settings beside the model text) and without.
    std::fs::write(assets.join("moved.gltf"), gltf.to_string()).unwrap();
    let sprig = "type model\nversion 1\n---\nplant \"sprig\" {\n levels 1\n trunk {\n  length 1\n  radius 0.05\n  sections 3\n  segments 3\n  children 2\n }\n branch1 {\n  length 0.5\n  radius 0.6\n  sections 2\n  segments 3\n }\n leaves {\n  count 4\n  size 0.2\n }\n}\n";
    std::fs::write(assets.join("sprig.ogt"), sprig).unwrap();
    std::fs::write(assets.join("sprig.ogt.ogt"), card).unwrap();
    std::fs::write(assets.join("bare.ogt"), sprig).unwrap();
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::copy(
        repo.join("packages/engine/tests/fixtures/terrain/assets/terrain/hill.png"),
        assets.join("hill.png"),
    )
    .unwrap();
    std::fs::write(assets.join("ground.ogt"), "type terrain\nversion 1\n---\nheightmap \"hill.png\"\nheight 0 1\nsize 16 16\nlayer {\n color 0.2 0.3 0.1\n}\n").unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

#[test]
fn scaled_node_placements_match_scatter_impostor_bounds_and_report_mismatches() {
    let root = project("parts");
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(None);
    let engine = value(og_engine_create(game, json!({"root":root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Seen(seen.clone())))).unwrap();
    let model = value(og_runtime::host_impl::answer(og_load_gltf(
        engine,
        "scaled.gltf".into(),
        None,
    )))
    .as_f64()
    .unwrap();
    let info = value(og_runtime::host_impl::answer(og_load_gltf_info(
        engine, model,
    )));
    let data = value(og_runtime::host_impl::answer(og_terrain_load(
        engine,
        "ground.ogt".into(),
    )))["data"]
        .clone();
    let entity = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(json!({"name":"scatter"}).to_string()),
    )))
    .as_f64()
    .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        entity,
        "og.Terrain".into(),
        Some(json!({"data":data}).to_string()),
    )));
    let mut desc = json!({"data":data,"geometry":info["geometries"][0],"material":info["materials"][0],"impostor":model,"impostorDistance":12,"sway":0});
    for distance in [0, 12, 40] {
        desc["impostorDistance"] = json!(distance);
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            entity,
            "og.TerrainScatter".into(),
            Some(desc.to_string()),
        )));
        value(og_engine_frame(engine, 1000.0 / 60.0));
        let borrowed = seen.borrow();
        assert_eq!(borrowed.len(), 1);
        let (draw, (lo, hi)) = borrowed[0];
        // Union of the two explicitly authored node placements above.
        assert!(lo.abs_diff_eq(Vec3::new(-0.6, 0.17, -0.41), 2e-5), "{lo:?}");
        assert!(hi.abs_diff_eq(Vec3::new(0.37, 0.7, 0.3), 2e-5), "{hi:?}");
        let card = draw.impostor.unwrap();
        assert!(card.center.abs_diff_eq((lo + hi) * 0.5, 2e-5));
        assert!((card.radius - (hi - lo).length() * 0.5).abs() < 2e-5);
    }
    assert_eq!(info["parts"].as_array().unwrap().len(), 2);
    assert!(
        (info["parts"][0]["transform"]["elements"][0]
            .as_f64()
            .unwrap()
            - 0.17)
            .abs()
            < 1e-6
    );
    desc["geometry"] = value(og_runtime::host_impl::answer(og_render_box_geometry(
        engine, None,
    )));
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        entity,
        "og.TerrainScatter".into(),
        Some(desc.to_string()),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert!(seen.borrow().is_empty());
    let id = og_ecs::Entity::from_bits(entity as u64);
    let dump = og_engine_probe_dump(
        engine,
        Some(
            json!({"what":"entity", "id":format!("#{}.{}", id.index(), id.generation())})
                .to_string(),
        ),
    );
    assert!(
        dump.contains("geometry bounds") && dump.contains("impostor bounds"),
        "{dump}"
    );
    desc["geometry"] = info["geometries"][0].clone();
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        entity,
        "og.TerrainScatter".into(),
        Some(desc.to_string()),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let dump = og_engine_probe_dump(
        engine,
        Some(
            json!({"what":"entity", "id":format!("#{}.{}", id.index(), id.generation())})
                .to_string(),
        ),
    );
    assert!(!dump.contains("scatterIssue"), "{dump}");
    assert_eq!(seen.borrow().len(), 1);
}

/// `model` scatters a whole model as one copy: a draw for each part with
/// its own material, the later ones drawn with the first, in the model's
/// space; a model its nodes place needs the impostor's bake to say where.
#[test]
fn a_model_scatters_whole_with_every_part_as_one_copy() {
    let root = project("whole");
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(None);
    let engine = value(og_engine_create(game, json!({"root":root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Seen(seen.clone())))).unwrap();
    let load = |path: &str| {
        value(og_runtime::host_impl::answer(og_load_gltf(
            engine,
            path.into(),
            None,
        )))
    };
    let data = value(og_runtime::host_impl::answer(og_terrain_load(
        engine,
        "ground.ogt".into(),
    )))["data"]
        .clone();
    let spawn = |name: &str| {
        value(og_runtime::host_impl::answer(og_world_spawn(
            engine,
            Some(json!({"name":name}).to_string()),
        )))
        .as_f64()
        .unwrap()
    };
    let scatter = |entity: f64, model: &Value, seed: u32| {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            entity,
            "og.TerrainScatter".into(),
            Some(json!({"data":data,"model":model,"seed":seed,"impostorDistance":12}).to_string()),
        )));
    };
    let ground = spawn("ground");
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        ground,
        "og.Terrain".into(),
        Some(json!({"data":data}).to_string()),
    )));
    let (carded, bare, moved) = (spawn("carded"), spawn("bare"), spawn("moved"));
    scatter(carded, &load("sprig.ogt"), 3);
    scatter(bare, &load("bare.ogt"), 5);
    scatter(moved, &load("moved.gltf"), 7);
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let draws = seen.borrow();
    for (seed, card) in [(3, true), (5, false)] {
        let parts: Vec<_> = draws
            .iter()
            .enumerate()
            .filter(|(_, (d, _))| d.scatter.seed == seed)
            .collect();
        assert_eq!(parts.len(), 2, "wood and leaves, seed {seed}");
        let (first, (wood, _)) = parts[0];
        let (_, (leaves, _)) = parts[1];
        assert_eq!((wood.with, leaves.with), (None, Some(first)));
        assert_ne!(wood.material, leaves.material);
        assert_eq!(wood.impostor.is_some(), card, "seed {seed}");
        if let Some(c) = wood.impostor {
            // Both parts in the space the card was baked in.
            for (_, (_, (lo, hi))) in &parts {
                assert!(lo.distance(c.center) <= c.radius * 1.001, "{lo:?} {c:?}");
                assert!(hi.distance(c.center) <= c.radius * 1.001, "{hi:?} {c:?}");
            }
        }
    }
    assert!(
        draws.iter().all(|(d, _)| d.scatter.seed != 7),
        "no card says where the moved model's parts stand"
    );
    let id = og_ecs::Entity::from_bits(moved as u64);
    let dump = og_engine_probe_dump(
        engine,
        Some(
            json!({"what":"entity", "id":format!("#{}.{}", id.index(), id.generation())})
                .to_string(),
        ),
    );
    assert!(dump.contains("nodes move or scale"), "{dump}");
}
