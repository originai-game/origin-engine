//! The tick after a world.restore delivers og.WorldRestored (brasswick
//! FINDINGS #8): everything in the world rolls back with a save, request
//! numbers and session settings included, and og-game §17 asks a game to
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
    let save = value(og_world_snapshot(e)).as_str().unwrap().to_string();
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
