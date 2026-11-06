//! The tick after a world.restore delivers og.WorldRestored (brasswick
//! FINDINGS #8): everything in the world rolls back with a save, request
//! numbers and session settings included, and a game is expected to
//! reset those after a restore; before, nothing marked the tick after one.
//!
//! A game of one system (Watch) records the og.WorldRestored events it
//! reads each tick. The world is saved between ticks at tick 10 and
//! restored at tick 20: the world between the restore and the next tick
//! hashes as the save did (the event is not in it yet), Watch reads one
//! event in that tick (`from` 20, `tick` 10) and none after.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_system_define_system, og_world_events, og_world_get_tick,
    og_world_restore, og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const WATCH: u32 = 1;

/// (tick, the og.WorldRestored events Watch read in it).
struct Watch(Rc<RefCell<Vec<(u32, Value)>>>);

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

impl Hooks for Watch {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let w = call.engine;
            let seen: Value =
                serde_json::from_str(&og_world_events(w, "og.WorldRestored".into())).unwrap();
            self.0.borrow_mut().push((og_world_get_tick(w), seen));
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

fn run_to(e: u32, tick: u32) {
    while og_world_get_tick(e) < tick {
        value(og_engine_frame(e, 1000.0 / 60.0));
    }
    assert_eq!(og_world_get_tick(e), tick);
}

#[test]
fn the_tick_after_a_restore_reads_og_world_restored_once() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_hooks(Rc::new(Watch(seen.clone())));
    og_host_begin_bundle();
    let restored = json!({ "kind": "event", "name": "og.WorldRestored" });
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [restored], "writes": [] }).to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [watch] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;

    run_to(e, 10);
    let save = value(og_world_snapshot(e, None))
        .as_str()
        .unwrap()
        .to_string();
    let saved = hash(e);
    run_to(e, 20);
    assert!(
        seen.borrow().iter().all(|(_, ev)| ev == &json!([])),
        "no restore yet: {:?}",
        seen.borrow()
    );
    seen.borrow_mut().clear();

    value(og_world_restore(e, save, None));
    assert_eq!(og_world_get_tick(e), 10);
    assert_eq!(hash(e), saved, "between ticks the world is the save");

    run_to(e, 13);
    let seen = seen.borrow();
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert_eq!(
        seen[0],
        (10, json!([{ "from": 20, "tick": 10 }])),
        "the tick after the restore reads it"
    );
    assert_eq!(seen[1], (11, json!([])), "once");
    assert_eq!(seen[2], (12, json!([])));
}

#[test]
fn controller_save_load_differs_only_by_the_restore_notification() {
    use og_ecs::SnapshotEdit;
    use og_runtime::{og_motion_walk_to, og_world_insert, og_world_spawn, save_snapshot_bytes};

    fn snapshot(e: u32) -> (String, Vec<u8>) {
        let text = value(og_world_snapshot(e, None))
            .as_str()
            .unwrap()
            .to_owned();
        let bytes = save_snapshot_bytes(&text).unwrap();
        (text, bytes)
    }

    set_hooks(Rc::new(Watch(Rc::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed": 11}).to_string()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let actor = value(og_world_spawn(w, None)).as_f64().unwrap();
    for (name, desc) in [
        ("og.Transform", json!({"position":{"x":0,"y":1,"z":0}})),
        ("og.MotionAgent", json!({"drive":true})),
        ("og.Collider", json!({"shape":2,"radius":0.4,"height":1})),
        ("og.CharacterController", json!({})),
    ] {
        value(og_world_insert(
            w,
            actor,
            name.into(),
            Some(desc.to_string()),
        ));
    }
    let floor = value(og_world_spawn(w, None)).as_f64().unwrap();
    for (name, desc) in [
        ("og.Transform", json!({"position":{"x":0,"y":-0.5,"z":0}})),
        (
            "og.Collider",
            json!({"shape":0,"size":{"x":20,"y":1,"z":20}}),
        ),
    ] {
        value(og_world_insert(
            w,
            floor,
            name.into(),
            Some(desc.to_string()),
        ));
    }
    value(og_motion_walk_to(
        w,
        actor,
        Some(json!({"position":{"x":2.8,"y":0,"z":0}}).to_string()),
    ));
    run_to(w, 60);
    let (save, saved_bytes) = snapshot(w);
    let saved_hash = hash(w);
    let mut expected = Vec::new();
    for tick in 61..=120 {
        run_to(w, tick);
        expected.push((hash(w), snapshot(w).1));
    }
    value(og_world_restore(w, save, None));
    assert_eq!(hash(w), saved_hash);
    assert_eq!(snapshot(w).1, saved_bytes);
    for (i, (expected_hash, bytes)) in expected.into_iter().enumerate() {
        run_to(w, 61 + i as u32);
        let actual = snapshot(w).1;
        if i == 0 {
            // Save/load adds an observable event; rollback deliberately does not.
            // Compare every remaining byte, including opaque physics state blocks.
            assert_ne!(hash(w), expected_hash);
            let mut before = SnapshotEdit::from_bytes(&bytes).unwrap();
            let mut after = SnapshotEdit::from_bytes(&actual).unwrap();
            assert!(!before.has_events("og.WorldRestored"));
            assert!(after.has_events("og.WorldRestored"));
            for edit in [&mut before, &mut after] {
                edit.replace_types(&["og.WorldRestored"], Vec::new())
                    .unwrap();
            }
            assert_eq!(after.to_bytes(), before.to_bytes());
        } else {
            assert_eq!(hash(w), expected_hash, "tick {}", i + 61);
            assert_eq!(actual, bytes, "tick {}", i + 61);
        }
    }
}
