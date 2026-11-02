//! World cells (G3-3): a cell's sub-scene loads and unloads by the anchors,
//! deterministically, as world state.
//!
//! Ways this could fail, and the test that catches each:
//! - a cell loads by I/O or wall time, or loads its entities in another
//!   order, so a replay, a save restored mid-way or a second run spawns
//!   other entity ids or hashes otherwise across a load and an unload →
//!   `a_replay_and_a_restored_save_go_tick_for_tick_across_a_cell_load`;
//! - the box, the hysteresis or the placement is off (a cell that loads at
//!   the wrong distance, flickers on its edge, or spawns its sub-scene at
//!   the origin instead of the cell's place), or an unload leaves members
//!   or their children → `a_cell_loads_near_an_anchor_and_unloads_past_unload`;
//! - a session (og.Lockstep.enabled) loads cells by the camera →
//!   `a_session_without_anchors_refuses_the_camera`;
//! - a plain sub-scene is not spawned under its entity →
//!   `a_plain_sub_scene_spawns_under_its_entity`.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_commands_schedule, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_probe_dump, og_engine_take_log, og_engine_take_outputs,
    og_game_game, og_host_begin_bundle, og_load_scene, og_reflect_define_component, og_world_find,
    og_world_get, og_world_instantiate, og_world_restore, og_world_set, og_world_snapshot,
    set_hooks,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const WORLD: &str = "type scene
version 1
---
name \"World\"
entity \"player\" {
  Transform {
    position 0 0 0
  }
  StreamAnchor {
    reach 0
  }
}
entity \"village\" {
  scene \"scenes/village.ogt\"
  Transform {
    position 100 0 0
  }
  StreamCell {
    size 20 10 20
    load 10
  }
}
entity \"farm\" {
  scene \"scenes/farm.ogt\"
  Transform {
    position 130 0 0
  }
  StreamCell {
    size 10 10 10
    load 5
  }
}
";

const VILLAGE: &str = "type scene
version 1
---
name \"Village\"
entity \"well\" {
  Transform {
    position 1 0 2
  }
  Health {
    hp 5
  }
  entity \"rope\" {
    Health {
      hp 1
    }
  }
}
entity \"hut\" {
  Transform {
    position -3 0 0
  }
}
";

const FARM: &str = "type scene
version 1
---
name \"Farm\"
entity \"barn\" {
  Health {
    hp 9
  }
}
";

fn cook(root: &Path) {
    use og_core::reflect::{FieldSchema, FieldType, Schema};
    let health = Schema {
        name: "Health".into(),
        version: 1,
        doc: String::new(),
        fields: vec![FieldSchema::new("hp", FieldType::F32)],
    };
    let options = og_cook::CookOptions {
        threads: 1,
        schemas: vec![health],
        ..og_cook::CookOptions::default()
    };
    let report = og_cook::cook_project(root, &options).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

fn project(name: &str, world: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-cells-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
    std::fs::write(root.join("assets/scenes/world.ogt"), world).unwrap();
    std::fs::write(root.join("assets/scenes/village.ogt"), VILLAGE).unwrap();
    std::fs::write(root.join("assets/scenes/farm.ogt"), FARM).unwrap();
    cook(&root);
    root
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

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// A game declaring Health, with the world scene instantiated.
fn engine(root: &Path) -> u32 {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    value(og_reflect_define_component(
        "Health".into(),
        json!([{ "name": "hp", "kind": "f32", "default": 0, "options": {} }]).to_string(),
    ));
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    value(og_engine_create(game, options)).as_u64().unwrap() as u32
}

fn placed(root: &Path) -> u32 {
    let e = engine(root);
    let scene = value(og_load_scene(e, "scenes/world.ogt".into()));
    value(og_world_instantiate(e, scene.as_f64().unwrap(), None));
    e
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

fn step(e: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
}

fn find(e: u32, name: &str) -> Option<f64> {
    serde_json::from_str::<Value>(&og_world_find(e, name.into()))
        .unwrap()
        .as_f64()
}

fn get(e: u32, name: &str, comp: &str) -> Value {
    value(og_world_get(e, find(e, name).unwrap(), comp.into()))
}

/// Moves the player (the anchor) to `x`.
fn walk(e: u32, x: f64) {
    value(og_world_set(
        e,
        find(e, "player").unwrap(),
        "og.Transform".into(),
        json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string(),
    ));
}

fn has(e: u32, name: &str, comp: &str) -> bool {
    og_runtime::og_world_has(e, find(e, name).unwrap(), comp.into())
}

/// The probe's dump_stream `cells`, through the command channel (it runs
/// at the next tick, so this steps one).
fn cells(e: u32) -> Value {
    value_of(&og_engine_probe_commands(e, "0:dump_stream".into()));
    step(e, 1);
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(e)).unwrap();
    let data = outputs
        .as_array()
        .and_then(|a| a.iter().rev().find(|o| o["action"] == json!("dump_stream")))
        .and_then(|o| o["data"].as_str())
        .unwrap_or_else(|| panic!("no dump_stream output: {outputs}"));
    serde_json::from_str::<Value>(data).unwrap()["cells"].clone()
}

fn value_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn loaded(e: u32, cell: &str) -> bool {
    get(e, cell, "og.StreamCell")["loaded"] == json!(true)
}

#[test]
fn a_cell_loads_near_an_anchor_and_unloads_past_unload() {
    let root = project("load", WORLD);
    let e = placed(&root);
    step(e, 1);
    // 90 m from the village's box (x 90..110): nothing of it is here.
    assert!(!loaded(e, "village"));
    assert_eq!(find(e, "well"), None);
    // 5 m from the box: it loads as the next tick starts.
    walk(e, 85.0);
    step(e, 1);
    assert!(loaded(e, "village"));
    let well = get(e, "well", "og.Transform");
    assert_eq!(
        well["position"],
        json!({ "x": 101.0, "y": 0.0, "z": 2.0 }),
        "placed at the cell after its own transform"
    );
    assert!(!has(e, "well", "og.Parent"), "a top-level entity is a root");
    assert!(has(e, "rope", "og.Parent"));
    assert_eq!(get(e, "rope", "Health")["hp"], json!(1.0));
    let member = get(e, "rope", "og.CellMember");
    assert_eq!(member["cell"].as_f64(), find(e, "village"));
    // The farm (x 125..135, load 5) is 40 m off: still unloaded.
    assert!(!loaded(e, "farm"));
    // 11 m: past `load` but within `unload` (12.5): it stays.
    walk(e, 79.0);
    step(e, 1);
    assert!(loaded(e, "village"));
    // 13 m: past `unload`; every member and child goes.
    walk(e, 77.0);
    step(e, 1);
    assert!(!loaded(e, "village"));
    for n in ["well", "rope", "hut"] {
        assert_eq!(find(e, n), None, "{n} despawned");
    }
    // Back in: the sub-scene again, as authored.
    walk(e, 95.0);
    step(e, 1);
    assert!(loaded(e, "village"));
    assert_eq!(get(e, "well", "Health")["hp"], json!(5.0));
    let dump = cells(e);
    assert_eq!(dump[0]["scene"], json!("scenes/village.ogt"), "{dump}");
    assert_eq!(dump[0]["loads"], json!(2));
    assert_eq!(dump[0]["entities"], json!(3));
    assert_eq!(dump[1]["loaded"], json!(false));
}

/// One scripted walk past both cells: the hash after every tick, and the
/// village well's entity id while it is loaded.
fn walk_script(e: u32, from: u32, out: &mut Vec<(Value, Option<f64>)>) {
    for t in from..60 {
        // Out to the farm and back, two metres a tick.
        let x = if t < 30 {
            2.0 * t as f64 + 70.0
        } else {
            190.0 - 2.0 * t as f64
        };
        walk(e, x);
        step(e, 1);
        out.push((hash(e), find(e, "well")));
    }
}

#[test]
fn a_replay_and_a_restored_save_go_tick_for_tick_across_a_cell_load() {
    let root = project("replay", WORLD);
    let a = placed(&root);
    let mut first = Vec::new();
    walk_script(a, 0, &mut first);
    // The run loaded both cells and unloaded the village on the way.
    let dump = cells(a);
    assert_eq!(dump[0]["loads"], json!(2), "{dump}");
    assert_eq!(dump[1]["loads"], json!(1), "{dump}");

    // A second run of the same script: every tick's hash and the ids the
    // cells spawned are the same.
    let b = placed(&root);
    let mut second = Vec::new();
    walk_script(b, 0, &mut second);
    assert_eq!(first, second);

    // The ids are pinned: the village's well is its first entity, in the
    // slot after the instance root and the world's three (e4v1); the reload
    // after the unload takes the same slot again, a generation on (e4v2).
    // A change to the order of loads or spawns moves these.
    let ids: Vec<f64> = first.iter().filter_map(|(_, w)| *w).collect();
    assert_eq!(ids.first(), Some(&WELL_FIRST), "{ids:?}");
    assert_eq!(ids.last(), Some(&WELL_AFTER_RELOAD), "{ids:?}");
    assert_eq!(first[59].0, json!(END_HASH), "the run's last hash");

    // A save taken while the village is loaded (tick 10), restored into a
    // fresh session, goes on as the first run did.
    let c = placed(&root);
    let mut third = Vec::new();
    for t in 0..10 {
        walk(c, 2.0 * t as f64 + 70.0);
        step(c, 1);
    }
    assert!(loaded(c, "village"));
    let save = value(og_world_snapshot(c, None))
        .as_str()
        .unwrap()
        .to_string();
    let saved = hash(c);
    // The session boots as the game does (its scene, which loads the
    // sub-scenes the cells' handles name), then restores the save.
    let d = placed(&root);
    value(og_world_restore(d, save, None));
    assert_eq!(hash(d), saved);
    walk_script(d, 10, &mut third);
    // The tick after a restore delivers og.WorldRestored, so its hash is
    // its own; every tick after it is the first run's, the reload included.
    assert_eq!(third[1..], first[11..]);
}

/// [`a_replay_and_a_restored_save_go_tick_for_tick_across_a_cell_load`]'s
/// pins: the well's id as the village first loads and after it reloads, and
/// the last tick's hash. The hash folds the world's type table, so it moved
/// (2c17f3c42615a4b9 -> a4f0740b668ce590) when og.VideoPlayer became lazy
/// (G3-2: a world without footage no longer has the type), and again
/// (a4f0740b668ce590 -> 07d9679424bad231) when R11 made og.Lighting2D,
/// og.Light2D and og.Occluder2D lazy, and again (07d9679424bad231 ->
/// af7faee87b6dea25) for ui-gaps' UI state version 14, and again
/// (af7faee87b6dea25 -> d68afb5b35367aa5) for version 15, with no change to
/// the cells' own behaviour.
const WELL_FIRST: f64 = (1u64 << 32 | 4) as f64;
const WELL_AFTER_RELOAD: f64 = (2u64 << 32 | 4) as f64;
const END_HASH: &str = "d68afb5b35367aa5";

#[test]
fn a_session_without_anchors_refuses_the_camera() {
    let root = project(
        "session",
        &WORLD.replace("  StreamAnchor {\n    reach 0\n  }\n", ""),
    );
    let e = placed(&root);
    // The camera at the village: it loads by the camera alone.
    value(og_commands_schedule(
        e,
        "0:set:og.Camera.position=100,0,0".into(),
    ));
    step(e, 2);
    assert!(loaded(e, "village"));
    // In a session the camera is each peer's own: refused, and nothing
    // more loads or unloads by it.
    value(og_commands_schedule(
        e,
        "0:set:og.Lockstep.enabled=true;0:set:og.Camera.position=500,0,0".into(),
    ));
    step(e, 3);
    assert!(loaded(e, "village"), "not unloaded by the camera");
    let log = og_engine_take_log(e);
    assert!(log.contains("need an og.StreamAnchor"), "{log}");
}

#[test]
fn a_plain_sub_scene_spawns_under_its_entity() {
    let root = project(
        "plain",
        "type scene\nversion 1\n---\nentity \"yard\" {\n  scene \"scenes/farm.ogt\"\n  Transform {\n    position 5 0 0\n  }\n}\n",
    );
    let e = placed(&root);
    let barn = get(e, "barn", "og.Parent");
    assert_eq!(barn["entity"].as_f64(), find(e, "yard"));
    assert!(!has(e, "barn", "og.CellMember"));
}
