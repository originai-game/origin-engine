//! `world.despawn` inside a system takes the entity's descendants with it,
//! as it does between ticks: every entity whose og.Parent chain reaches it
//! when the despawn applies, a child the same system spawned under it just
//! before included. It failed while a system's despawn queued the entity
//! alone, leaving its children alive with og.Parent naming a dead entity.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_system_define_system, og_world_alive, og_world_despawn, og_world_find, og_world_get_tick,
    og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::rc::Rc;

const KILL: u32 = 1;

struct Game;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn spawn(w: u32, name: &str, parent: Option<f64>) -> f64 {
    let mut desc = json!({ "name": name });
    if let Some(p) = parent {
        desc["parent"] = json!(p);
    }
    og_world_spawn(w, Some(desc.to_string()))
        .expect("spawn")
        .as_f64()
        .expect("an entity number")
}

fn find(w: u32, name: &str) -> Option<f64> {
    og_world_find(w, name.into()).map(|b| b as f64)
}

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let w = call.engine;
        // At tick 2, Kill gives "root" one more child, then despawns it.
        if call.kind == 0 && call.id == KILL && og_world_get_tick(w) == 2 {
            let root = find(w, "root").expect("root");
            spawn(w, "late", Some(root));
            og_world_despawn(w, root).unwrap();
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

#[test]
fn a_systems_despawn_takes_the_descendants() {
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let component = |name: &str| json!({ "kind": "component", "name": name });
    let writes = json!({ "writes": [component("og.Transform"), component("og.Parent")] });
    let systems = [og_system_define_system(
        "Kill".into(),
        writes.to_string(),
        KILL,
    )];
    let game = og_game_game(Some(json!({ "seed": 1, "systems": systems }).to_string()));
    let w = value(og_engine_create(game, json!({}).to_string()))
        .as_u64()
        .unwrap() as u32;

    let root = spawn(w, "root", None);
    let child = spawn(w, "child", Some(root));
    let grandchild = spawn(w, "grandchild", Some(child));
    let other = spawn(w, "other", None);
    let beside = spawn(w, "beside", Some(other));
    for _ in 0..4 {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
    for (name, e) in [("root", root), ("child", child), ("grandchild", grandchild)] {
        assert!(!og_world_alive(w, e), "{name} is still alive");
    }
    assert!(
        find(w, "late").is_none(),
        "the child spawned before the despawn"
    );
    for (name, e) in [("other", other), ("beside", beside)] {
        assert!(og_world_alive(w, e), "{name}, not under root, is gone");
    }
}
