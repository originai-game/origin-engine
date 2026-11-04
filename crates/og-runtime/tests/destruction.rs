//! og.destruction (docs/design.md §2.7): a wall cooked with a `fracture`
//! block stands on its anchored chunks; shot out at its foot, the stress
//! solver brings down what it held up, the loose chunks fall as bodies, a
//! snapshot taken mid-collapse restores into a run that hashes as the
//! uninterrupted one, and the budgets despawn the oldest loose chunks.
//!
//! Ways this could fail: the bonds' health or the chunks' bodies are not
//! world state (a restore loses them, the hashes part); the stress solver
//! is not run (the wall stands on air above its hole); islands become
//! bodies in an order that depends on something other than the world (the
//! replay diverges); the budget counts the wrong chunks or none.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_destruction_damage, og_destruction_info, og_engine_create,
    og_engine_frame, og_engine_probe_dump, og_engine_write_resource, og_game_game,
    og_host_begin_bundle, og_load_gltf, og_world_events, og_world_get, og_world_insert,
    og_world_instantiate, og_world_restore, og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

const WALL: &str = r#"type model
version 1
---
material "brick" {
  color 0.6 0.3 0.2
  roughness 0.9
}
part "wall" {
  material "brick"
  position 0 1 0
  boxGeometry {
    width 4
    height 2
    depth 0.3
  }
}
fracture {
  chunks 24
  seed 3
  children 4
}
"#;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A project with the wall, cooked once.
fn project() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-destruction-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets/props")).unwrap();
        std::fs::write(root.join("assets/props/wall.ogt"), WALL).unwrap();
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
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

/// An engine with the wall standing at the origin, as an og.Destructible.
fn start() -> (u32, f64) {
    let root = project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let asset = value(og_load_gltf(w, "props/wall.ogt".into(), None))
        .as_f64()
        .unwrap();
    let wall = value(og_world_instantiate(w, asset, None))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        w,
        wall,
        "og.Destructible".into(),
        Some(json!({ "toughness": 400, "strength": 4000 }).to_string()),
    ));
    (w, wall)
}

fn frames(w: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
}

fn hash(w: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(w, None)).unwrap();
    dump["hash"].clone()
}

fn destructible(w: u32, e: f64) -> Value {
    value(og_world_get(w, e, "og.Destructible".into()))
}

/// A blow at the wall's foot, in its middle.
fn shoot_foot(w: u32, wall: f64) {
    value(og_destruction_damage(
        w,
        wall,
        Some(
            json!({ "point": { "x": 0, "y": 0.1, "z": 0 }, "radius": 0.9, "innerRadius": 0.5, "amount": 4000 })
                .to_string(),
        ),
    ));
}

#[test]
fn a_wall_stands_until_shot_then_its_middle_comes_down() {
    let (w, wall) = start();
    frames(w, 30);
    let d = destructible(w, wall);
    assert_eq!(d["fractured"], false, "it stands under its own weight: {d}");
    let info = value(og_destruction_info(w, wall));
    assert_eq!(info["chunks"], 24);
    assert_eq!(info["islands"], 1);
    assert_eq!(info["health"], 1.0);
    assert!(info["anchored"].as_u64().unwrap() >= 3, "{info}");

    shoot_foot(w, wall);
    frames(w, 1);
    let events: Value = serde_json::from_str(&og_world_events(w, "og.Fractured".into())).unwrap();
    let events = events.as_array().unwrap();
    assert!(!events.is_empty(), "the shot breaks something loose");
    assert_eq!(events[0]["entity"].as_f64(), Some(wall));
    // og.destruction.damage, or the chunks it shattered.
    assert!(
        events
            .iter()
            .all(|e| [1, 3, 4].contains(&e["cause"].as_u64().unwrap())),
        "{events:?}"
    );
    let shot = destructible(w, wall)["loose"].as_u64().unwrap();
    assert!(shot > 0);

    // Over the next seconds the stress solver brings down what stood on
    // the chunks that were shot away.
    frames(w, 120);
    let d = destructible(w, wall);
    let info = value(og_destruction_info(w, wall));
    assert_eq!(d["fractured"], true);
    assert!(
        d["loose"].as_u64().unwrap() > shot,
        "more fell than the shot broke ({shot}): {d} {info}"
    );
    assert!(info["health"].as_f64().unwrap() < 1.0);
}

#[test]
fn a_collapse_replays_and_restores_bit_for_bit() {
    let run = |restore_at: Option<u32>| {
        let (w, wall) = start();
        frames(w, 5);
        shoot_foot(w, wall);
        frames(w, 12);
        let mid = value(og_world_snapshot(w, None))
            .as_str()
            .unwrap()
            .to_string();
        frames(w, 40);
        let end = hash(w);
        if let Some(n) = restore_at {
            // Restored mid-collapse, it runs on as the uninterrupted world.
            value(og_world_restore(w, mid, None));
            frames(w, n);
            assert_eq!(hash(w), end, "restored mid-collapse");
        }
        (end, destructible(w, wall))
    };
    let (a, da) = run(None);
    let (b, db) = run(Some(40));
    assert_eq!(a, b, "the same shot replays to the same world");
    assert_eq!(da, db);
    assert_eq!(da["fractured"], true);
}

#[test]
fn the_chunk_budget_despawns_the_oldest_loose_chunks() {
    let (w, wall) = start();
    value(og_engine_write_resource(
        w,
        "og.Destruction".into(),
        json!({ "maxChunks": 3 }).to_string(),
    ));
    frames(w, 2);
    shoot_foot(w, wall);
    frames(w, 90);
    let d = destructible(w, wall);
    assert_eq!(d["fractured"], true);
    let loose = d["loose"].as_u64().unwrap();
    assert!(loose <= 3, "at most maxChunks loose chunks remain: {d}");
}
