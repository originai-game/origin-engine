//! Maps made and resized at run time (G2-6): og.tilemap.create and resize
//! are world state, saved whole in the og.tilemap block's trailer, so a
//! save restores them in a session that never made them; a world that
//! only loads maps saves and hashes as before.
//!
//! Ways this could fail: the trailer written for every world (moving every
//! hash); a restore that needs the map made first; a resize that drops the
//! edits, or that a restore of an earlier save cannot undo; handles that
//! differ between a run and its replay.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_load_tilemap, og_tilemap_create, og_tilemap_get_tile,
    og_tilemap_neighbours, og_tilemap_resize, og_tilemap_set_tile, og_world_insert,
    og_world_restore, og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::{path::PathBuf, rc::Rc};

struct Game;

impl Hooks for Game {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
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

fn hash(engine: u32) -> String {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].as_str().unwrap().to_string()
}

/// A project of a 2-tile tileset (tile 0 solid) and a 3 × 2 map.
fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-runtime-{name}"));
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
        "type tileset\nversion 1\n---\ntexture \"tiles.png\"\ngrid 1 1 2 1\nsolid 0\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("map.ogt"),
        "type tilemap\nversion 1\n---\nsize 3 2\ntile 1 1\ntileset \"tiles.ogt\"\nlayer floor {\n collision true\n legend \"#\" 0\n row \"#..\"\n row \"...\"\n}\n",
    )
    .unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    root
}

fn start(root: &PathBuf) -> u32 {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 3 }).to_string()));
    value(og_engine_create(game, json!({ "root": root }).to_string()))
        .as_u64()
        .unwrap() as u32
}

fn place(engine: u32, map: &Value) -> f64 {
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
    entity
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn set(engine: u32, entity: f64, x: u32, y: u32, tile: i32) {
    value(og_tilemap_set_tile(
        engine,
        Some(
            json!({ "entity": entity, "layer": "floor", "x": x, "y": y, "tile": tile }).to_string(),
        ),
    ));
}

/// A world that only loads a map (and edits it) hashes as it did before
/// run-time maps: pinned on the code before G2-6 batch 2, then re-pinned
/// for G2-7 batch 2's og.Lockstep fields (3b30884ea43b16c7 to
/// 69496dd3ffb3e378), batch 3's stallTimeout and rejoin (to
/// 7d3e58d6b53fbd34), U6d's UI state version 13 (to a814ee913e53a5d5) and
/// G3-2's lazy og.VideoPlayer (to d30157f0d09a1253, the type gone from a
/// world without footage) and R11's lazy 2D lighting records (to
/// cb4cb2e1c4e0f7e1, og.Lighting2D, og.Light2D and og.Occluder2D gone from
/// a world without 2D lights) and ui-gaps' UI state version 14 (to
/// 40cb7e39b84923d5) and version 15 (to cdd436f5674a3768) and R9's og.Blendshapes version 2
/// (`influences`, to d09658f91e092780) and R6's og.TerrainScatter impostor fields
/// (to 418646f663e539b4) and og.Model.sway (to 93d5097782334060) and R5's lazy og.Water
/// (to 8a66396f924b22ba, gone from a world without water) and og.Environment's blurriness,
/// grounded dome and blend fields (to 82ae1d6124a13be2) and T5's lazy og.Terrain (to
/// 5017615dce8e0194, gone from a world without terrain) and area 7's Stylise and
/// MaterialStyle fields (to 4da281adabd646cf) and area 4's lazy og.TerrainScatter and og.Wind
/// (to 52ef5ac7cfa6538a) and UI state version 16, the touch control holding a finger
/// (to c950ccbacfe91a63) and RenderSettings.upscaler (to 0ece0b71c16b6488) and
/// PostEffects' ssrMaxRoughness and ssrResolution (to f917a356541f94e3), then Stylise.hatchingPaper (to 87e387f06096ffc6), then Cloth, SoftBody
/// and Garment's simulation (to 3e79197f349b1840), then MaterialStyle.wireframeColor
/// (to f6c957ee4976ed4a), then Fluid3D's simulation (to c53a68e4d5af79aa), each of
/// which moves every raw world hash.
/// PostEffects' ssrMaxRoughness and ssrResolution (to f917a356541f94e3) and
/// Stylise's watercolour fields (to da2b397551633113), then Stylise.posterizeMode (to 8cebc431ee6b9219), each of which moves every raw world hash.
#[test]
fn a_world_of_loaded_maps_keeps_its_hash() {
    set_hooks(Rc::new(Game));
    let root = project("tilemap-loaded");
    let engine = start(&root);
    let map = value(og_load_tilemap(engine, "map.ogt".into()))["map"].clone();
    let entity = place(engine, &map);
    frame(engine);
    set(engine, entity, 2, 1, 0);
    frame(engine);
    assert_eq!(hash(engine), "075ecab545345be5");
}

#[test]
fn a_made_and_resized_map_saves_whole_and_restores_in_a_fresh_session() {
    set_hooks(Rc::new(Game));
    let root = project("tilemap-runtime");
    let engine = start(&root);
    let desc = json!({
        "width": 9, "height": 7, "tileWidth": 16, "tileHeight": 16,
        "orientation": "hex", "tilesets": ["tiles.ogt"],
        "layers": [{ "name": "floor", "collision": true }],
    });
    let info = value(og_tilemap_create(engine, Some(desc.to_string())));
    assert_eq!(info["orientation"], "hex");
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(9), Some(7))
    );
    let entity = place(engine, &info["map"]);
    frame(engine);
    set(engine, entity, 2, 1, 0);
    set(engine, entity, 3, 1, 1);
    // Pointy hex, odd row 1 (shifted right): (4, 0) and (3, 0) above it.
    let n: Value = serde_json::from_str(&og_tilemap_neighbours(
        engine,
        info["map"].as_f64().unwrap(),
        3,
        1,
    ))
    .unwrap();
    let cells: Vec<(u64, u64)> = n
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["x"].as_u64().unwrap(), c["y"].as_u64().unwrap()))
        .collect();
    assert_eq!(cells, [(4, 0), (4, 1), (4, 2), (3, 2), (2, 1), (3, 0)]);
    frame(engine);
    let first = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let first_hash = hash(engine);

    // A second map from the same call in the same order: a handle of its own.
    let other = value(og_tilemap_create(engine, Some(desc.to_string())));
    assert_ne!(other["map"], info["map"]);

    let refused: Value = serde_json::from_str(&og_tilemap_resize(
        engine,
        Some(json!({ "map": info["map"], "width": 12, "height": 9, "x": 1, "y": 1 }).to_string()),
    ))
    .unwrap();
    assert_eq!(
        refused["ok"], false,
        "an odd shift re-staggers a hex map: {refused}"
    );
    value(og_tilemap_resize(
        engine,
        Some(json!({ "map": info["map"], "width": 12, "height": 9, "x": 1, "y": 2 }).to_string()),
    ));
    let at = |e: u32, x: u32, y: u32| og_tilemap_get_tile(e, entity, "floor".into(), x, y);
    assert_eq!(
        (at(engine, 3, 3), at(engine, 4, 3)),
        (0, 1),
        "the edits moved by (1, 2)"
    );
    assert_eq!(at(engine, 2, 1), -1);
    frame(engine);
    let resized = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let resized_hash = hash(engine);
    assert_ne!(resized_hash, first_hash);

    // A session that never made a map restores the resized one.
    let fresh = start(&root);
    value(og_world_restore(fresh, resized.clone(), None));
    assert_eq!(
        (at(fresh, 3, 3), at(fresh, 4, 3), at(fresh, 11, 8)),
        (0, 1, -1)
    );
    assert_eq!(hash(fresh), resized_hash);
    // Both restored (a restore is an event of the next tick), both go on alike.
    value(og_world_restore(engine, resized, None));
    for _ in 0..3 {
        frame(fresh);
        frame(engine);
    }
    assert_eq!(hash(fresh), hash(engine), "both go on alike");
    // The counter came with it: the next map is the one the run made next.
    let next = value(og_tilemap_create(fresh, Some(desc.to_string())));
    let after = value(og_tilemap_create(engine, Some(desc.to_string())));
    assert_eq!(next["map"], after["map"]);

    // The earlier save puts the map back at 9 × 7.
    value(og_world_restore(engine, first, None));
    assert_eq!(
        (at(engine, 2, 1), at(engine, 3, 1), at(engine, 3, 3)),
        (0, 1, -1)
    );
    assert_eq!(hash(engine), first_hash);
}

#[test]
fn a_resized_loaded_map_saves_whole_and_an_earlier_save_undoes_it() {
    set_hooks(Rc::new(Game));
    let root = project("tilemap-loaded-resize");
    let engine = start(&root);
    let map = value(og_load_tilemap(engine, "map.ogt".into()))["map"].clone();
    let entity = place(engine, &map);
    frame(engine);
    let before = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let before_hash = hash(engine);
    value(og_tilemap_resize(
        engine,
        Some(json!({ "map": map, "width": 5, "height": 4, "x": 2, "y": 1 }).to_string()),
    ));
    let at = |e: u32, x: u32, y: u32| og_tilemap_get_tile(e, entity, "floor".into(), x, y);
    assert_eq!((at(engine, 2, 1), at(engine, 0, 0)), (0, -1));
    assert_eq!(value(og_load_tilemap(engine, "map.ogt".into()))["width"], 5);
    frame(engine);
    let resized = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let resized_hash = hash(engine);

    // A session that loads the map as cooked restores it resized.
    let fresh = start(&root);
    assert_eq!(value(og_load_tilemap(fresh, "map.ogt".into()))["map"], map);
    value(og_world_restore(fresh, resized, None));
    assert_eq!(at(fresh, 2, 1), 0);
    assert_eq!(hash(fresh), resized_hash);

    value(og_world_restore(engine, before, None));
    assert_eq!((at(engine, 0, 0), at(engine, 2, 1)), (0, -1));
    assert_eq!(hash(engine), before_hash);
}
