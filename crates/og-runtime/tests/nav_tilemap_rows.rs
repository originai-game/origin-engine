//! A nav grid made from a tilemap counts its cells as the map does: row 0
//! at the top. og.tilemap.cellAt and og.nav.cellAt name the same cell for
//! one point, and a tile's cell given to og.nav.setCost blocks that tile.
//!
//! Before, the grid counted its rows up from the bottom, so both named
//! the mirrored row: these fail on that.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_load_tilemap, og_nav_cell_at, og_nav_find_path, og_nav_grid, og_nav_grid_from_tilemap,
    og_nav_set_cost, og_tilemap_cell_at, og_world_insert, og_world_spawn, set_hooks,
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

/// A 3 × 3 map of 1 m tiles whose top row is a wall with a gap in the
/// middle, placed with its top-left corner at the origin: it covers x 0
/// to 3 and y -3 to 0, and the tile (x, y) is centred on (x + 0.5,
/// -y - 0.5).
fn world(name: &str) -> (u32, f64, f64) {
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
        "type tilemap\nversion 1\n---\nsize 3 3\ntile 1 1\ntileset \"tiles.ogt\"\nlayer floor {\n collision true\n legend \"#\" 0\n row \"#.#\"\n row \"...\"\n row \"...\"\n}\n",
    )
    .unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 3 }).to_string()));
    let engine = value(og_engine_create(
        game,
        json!({ "root": root, "profile": "2d" }).to_string(),
    ))
    .as_u64()
    .unwrap() as u32;
    let map = value(og_runtime::host_impl::answer(og_load_tilemap(
        engine,
        "map.ogt".into(),
    )))["map"]
        .clone();
    let entity = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some("{}".into()),
    )))
    .as_f64()
    .unwrap();
    for (name, fields) in [
        ("og.Transform", json!({})),
        ("og.Tilemap", json!({ "map": map, "collision": true })),
    ] {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            entity,
            name.into(),
            Some(fields.to_string()),
        )));
    }
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let grid = value(og_runtime::host_impl::answer(og_nav_grid_from_tilemap(
        engine,
        Some(json!({ "entity": entity }).to_string()),
    )))
    .as_f64()
    .unwrap();
    (engine, entity, grid)
}

/// The centre of the map's tile (x, y), rows counted from the top.
fn centre(x: u32, y: u32) -> Value {
    json!({ "x": x as f32 + 0.5, "y": -(y as f32) - 0.5, "z": 0 })
}

/// (x, y, inside) of a cell answer.
fn cell(v: Value) -> (u64, u64, bool) {
    (
        v["x"].as_u64().unwrap(),
        v["y"].as_u64().unwrap(),
        v["inside"].as_bool().unwrap(),
    )
}

fn nav_cell(engine: u32, grid: f64, point: &Value) -> (u64, u64, bool) {
    let c = og_nav_cell_at(engine, grid, point.to_string());
    cell(serde_json::from_str(&og_runtime::host_impl::json(c)).unwrap())
}

/// A path between two tiles' centres: its length when it reaches the end.
fn path(engine: u32, grid: f64, from: (u32, u32), to: (u32, u32)) -> Option<f64> {
    let desc = json!({ "nav": grid, "from": centre(from.0, from.1), "to": centre(to.0, to.1) });
    let p: Value = serde_json::from_str(&og_runtime::host_impl::answer(og_nav_find_path(
        engine,
        Some(desc.to_string()),
    )))
    .unwrap();
    (p["ok"] == true && p["value"]["complete"] == true)
        .then(|| p["value"]["length"].as_f64().unwrap())
}

#[test]
fn a_tilemap_grid_names_its_cells_as_the_map_does() {
    set_hooks(Rc::new(Game));
    let (engine, entity, grid) = world("nav-tilemap-cells");
    // A point in the wall's top-left tile, one in the gap, one at the
    // bottom right: both modules name the same cell.
    for (x, y) in [(0, 0), (1, 0), (2, 2), (0, 1)] {
        let p = centre(x, y);
        let tile = og_tilemap_cell_at(engine, entity, p.to_string());
        let tile = cell(serde_json::from_str(&og_runtime::host_impl::json(tile)).unwrap());
        assert_eq!(
            tile,
            (u64::from(x), u64::from(y), true),
            "og.tilemap.cellAt"
        );
        assert_eq!(nav_cell(engine, grid, &p), tile, "og.nav.cellAt at {p}");
    }
    // The wall stands where the map draws it: the top row is shut but for
    // its gap, the bottom row is open.
    assert!(path(engine, grid, (1, 2), (1, 0)).is_some(), "the gap");
    assert!(path(engine, grid, (1, 2), (0, 0)).is_none(), "the wall");
    assert!(path(engine, grid, (0, 2), (2, 2)).is_some(), "the floor");
}

#[test]
fn a_tiles_cell_given_to_set_cost_blocks_that_tile_and_not_its_mirror() {
    set_hooks(Rc::new(Game));
    let (engine, _, grid) = world("nav-tilemap-set-cost");
    let straight = path(engine, grid, (0, 2), (2, 2)).expect("the bottom row is open");
    assert!((straight - 2.0).abs() < 1e-3, "{straight}");
    // Shut the gap in the top row: the tile (1, 0).
    og_nav_set_cost(engine, grid, 1, 0, 0).unwrap();
    assert!(
        path(engine, grid, (1, 2), (1, 0)).is_none(),
        "the gap's tile is blocked"
    );
    // Its mirror, the bottom row's middle tile (1, 2), is still floor.
    let after = path(engine, grid, (0, 2), (2, 2)).expect("the bottom row is still open");
    assert!((after - straight).abs() < 1e-3, "{after}");
    // Outside the map is refused, whichever way the rows count.
    assert!(og_nav_set_cost(engine, grid, 1, 3, 0).is_err());
}

#[test]
fn a_grid_made_with_og_nav_grid_keeps_its_rows_running_up() {
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = value(og_engine_create(
        game,
        json!({ "profile": "2d" }).to_string(),
    ))
    .as_u64()
    .unwrap() as u32;
    let grid = value(og_runtime::host_impl::answer(og_nav_grid(
        engine,
        Some(json!({ "width": 3, "height": 3 }).to_string()),
    )))
    .as_f64()
    .unwrap();
    // Its origin is cell (0, 0)'s lower corner: row 0 at the bottom.
    let low = json!({ "x": 0.5, "y": 0.5, "z": 0 });
    let high = json!({ "x": 0.5, "y": 2.5, "z": 0 });
    assert_eq!(nav_cell(engine, grid, &low), (0, 0, true));
    assert_eq!(nav_cell(engine, grid, &high), (0, 2, true));
}
