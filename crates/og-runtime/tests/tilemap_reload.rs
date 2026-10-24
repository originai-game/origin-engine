use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_engine_probe_reload, og_engine_take_log, og_game_game, og_host_begin_bundle,
    og_load_tilemap, og_system_define_system, og_tilemap_get_tile, og_tilemap_set_tile,
    og_world_get_tick, og_world_insert, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::{cell::Cell, path::PathBuf, rc::Rc};

struct Game(Rc<Cell<u32>>);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        self.0.set(call.id);
        Ok(String::new())
    }

    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

fn bundle(version: u32) -> u32 {
    og_host_begin_bundle();
    let system = og_system_define_system("Version".into(), "{}".into(), version);
    og_game_game(Some(json!({ "seed": 7, "systems": [system] }).to_string()))
}

fn hash(engine: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].clone()
}

#[test]
fn reload_preserves_edited_tiles_and_runs_the_new_bundle() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-tilemap-reload-{}", std::process::id()));
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::write(
        assets.join("tiles.png"),
        og_runtime::png(&og_render::RgbaImage {
            width: 2,
            height: 1,
            pixels: vec![255; 8],
        }),
    )
    .unwrap();
    std::fs::write(
        assets.join("tiles.ogt"),
        "type tileset\nversion 1\n---\ntexture \"tiles.png\"\ngrid 1 1 2 1\nsolid 1\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("map.ogt"),
        "type tilemap\nversion 1\n---\nsize 3 2\ntile 1 1\ntileset \"tiles.ogt\"\nlayer floor {\n collision true\n legend \"#\" 1\n row \"#..\"\n row \"...\"\n}\n",
    )
    .unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    let seen = Rc::new(Cell::new(0));
    set_hooks(Rc::new(Game(seen.clone())));
    let game = bundle(1);
    let engine = value(og_engine_create(game, json!({ "root": root }).to_string()))
        .as_u64()
        .unwrap() as u32;
    let map = value(og_load_tilemap(engine, "map.ogt".into()))["map"].clone();
    let entity = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    for (name, fields) in [
        ("og.Transform", json!({})),
        ("og.Tilemap", json!({ "map": map, "collision": true })),
    ] {
        value(og_world_insert(
            engine,
            entity,
            name.into(),
            Some(fields.to_string()),
        ));
    }
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(seen.get(), 1);
    value(og_tilemap_set_tile(
        engine,
        Some(json!({ "entity": entity, "layer": "floor", "x": 2, "y": 1, "tile": 1 }).to_string()),
    ));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    for version in [2, 3] {
        let tick = og_world_get_tick(engine);
        let saved_hash = hash(engine);
        assert!(!saved_hash.is_null());
        bundle(version);
        let answer: Value = serde_json::from_str(&og_engine_probe_reload(engine)).unwrap();
        assert!(
            answer.is_object(),
            "reload refused: {answer}: {}",
            og_engine_take_log(engine)
        );
        assert_eq!(og_world_get_tick(engine), tick);
        assert_eq!(
            hash(engine),
            saved_hash,
            "reload preserves the entire world"
        );
        value(og_engine_frame(engine, 1000.0 / 60.0));
        assert_eq!(seen.get(), version, "the new system runs");
        assert_eq!(og_tilemap_get_tile(engine, entity, "floor".into(), 2, 1), 1);
        assert_eq!(
            og_tilemap_get_tile(engine, entity, "floor".into(), 1, 0),
            -1
        );
        assert_eq!(value(og_load_tilemap(engine, "map.ogt".into()))["map"], map);
    }
    std::fs::remove_dir_all(root).unwrap();
}
