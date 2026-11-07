//! og.PixelTerrain (2D destructible ground): its chunks collide as the
//! cells' outline, a carve changes what stands and what falls, and the
//! cells, the pieces cut loose and the queued changes are world state.
//!
//! Ways this could fail: a chunk's collider not rebuilt after a carve (a
//! ball stops on ground that is gone), or built from the wrong cells or
//! place (a ball falls through solid ground); an island cut loose left in
//! the grid; the state outside the hash or the save (a replay or a restore
//! diverges); a piece's or a chunk's mesh missing after a restore.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_destruction_carve2d, og_destruction_cell2d,
    og_destruction_explode2d, og_destruction_fill2d, og_engine_create, og_engine_frame,
    og_engine_probe_dump, og_game_game, og_host_begin_bundle, og_world_get, og_world_insert,
    og_world_restore, og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct Game;

impl Hooks for Game {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn hash(engine: u32) -> String {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].as_str().unwrap().to_string()
}

fn frame(e: u32) {
    value(og_engine_frame(e, 1000.0 / 60.0));
}

/// A 2D world with a 64 × 32 terrain of 0.25 m cells at the origin: 16 m
/// wide, ground (bedrock its bottom row) up to y = 4.
fn world() -> (u32, f64) {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 7 }).to_string()));
    let e = value(og_engine_create(
        game,
        json!({ "profile": "2d" }).to_string(),
    ))
    .as_u64()
    .unwrap() as u32;
    let terrain = value(og_world_spawn(e, Some("{}".into())))
        .as_f64()
        .unwrap();
    let fields = json!({ "width": 64, "height": 32, "cellSize": 0.25, "chunk": 16, "seed": 3 });
    value(og_world_insert(
        e,
        terrain,
        "og.PixelTerrain".into(),
        Some(fields.to_string()),
    ));
    frame(e);
    (e, terrain)
}

fn ball(e: u32, x: f32, y: f32) -> f64 {
    let id = value(og_world_spawn(
        e,
        Some(json!({ "position": { "x": x, "y": y, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    for (name, v) in [
        ("og.RigidBody", json!({})),
        ("og.Collider", json!({ "shape": 1, "radius": 0.25 })),
    ] {
        value(og_world_insert(e, id, name.into(), Some(v.to_string())));
    }
    id
}

fn y_of(e: u32, entity: f64) -> f64 {
    value(og_world_get(e, entity, "og.Transform".into()))["position"]["y"]
        .as_f64()
        .unwrap()
}

fn cell(e: u32, terrain: f64, x: f32, y: f32) -> u64 {
    value(og_destruction_cell2d(
        e,
        terrain,
        json!({ "x": x, "y": y, "z": 0 }).to_string(),
    ))
    .as_u64()
    .unwrap()
}

fn carve(e: u32, terrain: f64, x: f32, y: f32, r: f32) {
    value(og_destruction_carve2d(
        e,
        terrain,
        Some(json!({ "point": { "x": x, "y": y, "z": 0 }, "radius": r }).to_string()),
    ));
}

#[test]
fn a_ball_stops_on_the_ground_and_falls_down_a_carved_shaft() {
    set_hooks(Rc::new(Game));
    let (e, terrain) = world();
    assert_eq!(cell(e, terrain, 4.0, 3.9), 1);
    assert_eq!(cell(e, terrain, 4.0, 4.1), 0);
    assert_eq!(cell(e, terrain, 4.0, 0.1), 255);
    // A shaft 1.5 m wide at x = 12, down to the bedrock.
    for k in 0..8 {
        carve(e, terrain, 12.0, 4.0 - k as f32 * 0.5, 0.75);
    }
    frame(e);
    assert_eq!(cell(e, terrain, 12.0, 2.0), 0);
    assert_eq!(cell(e, terrain, 12.0, 0.1), 255, "bedrock is never carved");
    let solid = ball(e, 4.0, 6.0);
    let shaft = ball(e, 12.0, 6.0);
    for _ in 0..180 {
        frame(e);
    }
    let (ys, yh) = (y_of(e, solid), y_of(e, shaft));
    assert!((ys - 4.25).abs() < 0.1, "on the ground: y {ys}");
    assert!(
        (yh - 0.5).abs() < 0.15,
        "on the bedrock down the shaft: y {yh}"
    );

    // Ground filled on the ground stands; ground filled in the air falls.
    for (x, y) in [(2.0, 4.4), (12.0, 9.0)] {
        value(og_destruction_fill2d(
            e,
            terrain,
            Some(json!({ "point": { "x": x, "y": y, "z": 0 }, "radius": 0.5 }).to_string()),
        ));
    }
    frame(e);
    assert_eq!(cell(e, terrain, 2.0, 4.4), 1);
    assert_eq!(cell(e, terrain, 12.0, 9.0), 0, "an unsupported fill falls");
}

#[test]
fn an_island_cut_loose_leaves_the_ground() {
    set_hooks(Rc::new(Game));
    let (e, terrain) = world();
    // A square trench round (8, 2.6) cuts a block free of the rest of the
    // ground; it runs into the bedrock below, so the block touches nothing.
    for k in 0..=16 {
        let s = -1.6 + k as f32 * 0.2;
        for (x, y) in [(s, -1.6), (s, 1.6), (-1.6, s), (1.6, s)] {
            carve(e, terrain, 8.0 + x, 2.6 + y, 0.35);
        }
    }
    frame(e);
    assert_eq!(
        cell(e, terrain, 8.0, 2.6),
        0,
        "the block was cut from the grid"
    );
    assert_eq!(cell(e, terrain, 2.0, 2.0), 1, "the ground beside it stands");
}

/// A carve, a fill and an explosion that throws pieces, over 60 ticks.
fn script(e: u32, terrain: f64, tick: u32) {
    match tick {
        2 => carve(e, terrain, 5.0, 4.0, 1.2),
        5 => {
            value(og_destruction_fill2d(
                e,
                terrain,
                Some(json!({ "point": { "x": 9, "y": 5, "z": 0 }, "radius": 0.8 }).to_string()),
            ));
        }
        9 => {
            value(og_destruction_explode2d(
                e,
                Some(
                    json!({ "point": { "x": 9, "y": 4, "z": 0 }, "radius": 1, "debris": 0.75 })
                        .to_string(),
                ),
            ));
        }
        _ => {}
    }
}

#[test]
fn a_carve_sequence_replays_and_restores_to_the_same_hash() {
    set_hooks(Rc::new(Game));
    let run = |save_at: Option<u32>| {
        let (e, terrain) = world();
        let mut save = None;
        for t in 0..60 {
            if Some(t) == save_at {
                save = Some(
                    value(og_world_snapshot(e, None))
                        .as_str()
                        .unwrap()
                        .to_string(),
                );
            }
            script(e, terrain, t);
            frame(e);
        }
        (e, terrain, save)
    };
    let (a, _, _) = run(None);
    let (b, terrain, save) = run(Some(8));
    assert_eq!(hash(a), hash(b), "a replay hashes the same");
    let end = hash(b);
    // Restored to tick 8 (after the carve and the fill, before the blast),
    // the rest of the script gives the same world.
    value(og_world_restore(b, save.unwrap(), None));
    for t in 8..60 {
        script(b, terrain, t);
        frame(b);
    }
    assert_eq!(hash(b), end, "a restore replays to the same hash");
    // And the hash moved with the carves (they are in it).
    let (c, _, _) = {
        let (e, terrain) = world();
        for _ in 0..60 {
            frame(e);
        }
        (e, terrain, ())
    };
    assert_ne!(hash(c), end, "the carves are world state");
}
