//! og.transform.matrixWorld reads any entity's world matrix: its og.Transform
//! composed down its og.Parent chain, inside a system that reads og.Transform
//! and between ticks. A game had no way to place a child in the world
//! (og.MatrixWorld is written for bone attachments only), so Star Focus's
//! armour pods, children of a scaled Boss, could not aim or burst.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_system_define_system, og_transform_decompose, og_transform_matrix_world, og_world_find,
    og_world_get_tick, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const READ: u32 = 1;

thread_local! {
    static SEEN: RefCell<Option<Value>> = const { RefCell::new(None) };
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// The world position of `e`: its matrixWorld's translation.
fn place(w: u32, e: f64) -> Value {
    let m = og_transform_matrix_world(w, e).expect("matrixWorld");
    og_transform_decompose(serde_json::to_string(&m).unwrap())["position"].clone()
}

struct Game;

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let w = call.engine;
        if call.kind == 0 && call.id == READ && og_world_get_tick(w) == 2 {
            let pod = og_world_find(w, "pod".into()).expect("pod") as f64;
            SEEN.with(|s| *s.borrow_mut() = Some(place(w, pod)));
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

#[test]
fn a_childs_world_matrix_composes_its_parents() {
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let component = |name: &str| json!({ "kind": "component", "name": name });
    let reads = json!({ "reads": [component("og.Transform"), component("og.Parent")] });
    let systems = [og_system_define_system(
        "Read".into(),
        reads.to_string(),
        READ,
    )];
    let game = og_game_game(Some(json!({ "seed": 1, "systems": systems }).to_string()));
    let w = value(og_engine_create(game, json!({}).to_string()))
        .as_u64()
        .unwrap() as u32;

    // A Boss at (2, 0, 1), twice its size and turned half a turn about y; a
    // pod at (1, 0, 0.5) in its frame stands at 2 + -2·1, 0, 1 + -2·0.5.
    let boss = og_world_spawn(
        w,
        Some(json!({ "name": "boss", "position": {"x": 2, "y": 0, "z": 1}, "scale": {"x": 2, "y": 2, "z": 2}, "quaternion": {"x": 0, "y": 1, "z": 0, "w": 0} }).to_string()),
    )
    .unwrap()
    .as_f64()
    .unwrap();
    og_world_spawn(
        w,
        Some(
            json!({ "name": "pod", "parent": boss, "position": {"x": 1, "y": 0, "z": 0.5} })
                .to_string(),
        ),
    )
    .unwrap();
    for _ in 0..3 {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
    let expected = [0.0, 0.0, 0.0];
    let inside = SEEN
        .with(|s| s.borrow().clone())
        .expect("the system read the pod");
    let pod = og_world_find(w, "pod".into()).unwrap() as f64;
    for (when, at) in [
        ("inside the system", inside),
        ("between ticks", place(w, pod)),
    ] {
        for (axis, want) in ["x", "y", "z"].iter().zip(expected) {
            let got = at[axis].as_f64().unwrap();
            assert!(
                (got - want).abs() < 1e-5,
                "{when}: {axis} is {got}, not {want} ({at})"
            );
        }
    }
}
