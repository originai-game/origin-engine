//! A save a system takes mid-tick, restored, replays the run it came from
//! (meridian-loop FINDINGS #12): the restored world completes the saving
//! tick (the systems after the saver, and the physics step) on its next
//! step, then goes on tick for tick as the world that saved did. Before,
//! the restored world ran the whole saving tick again (the systems before
//! the saver twice, the physics step once) and a replay from it diverged.
//! Also og.physics.applyImpulse through the host ABI: a push from a system
//! is applied by that tick's step, and refused for an entity without an
//! og.RigidBody.
//!
//! Two engines run one game of three systems (Count, Save, Mix) over a
//! falling box: engine A saves inside Save at tick 8; engine B restores
//! that save inside Save at tick 3 (it applies when B's tick 3 ends) and
//! runs on. At tick 30 both hold the same world.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_physics_apply_impulse, og_reflect_define_component,
    og_system_define_system, og_world_find, og_world_get, og_world_get_tick, og_world_insert,
    og_world_restore, og_world_set, og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const COUNT: u32 = 1;
const SAVE: u32 = 2;
const MIX: u32 = 3;
const SAVE_AT: u32 = 8;
const RESTORE_AT: u32 = 3;
const PUSH_AT: u32 = 12;

#[derive(Default)]
struct Script {
    /// The engine that saves and the one that restores.
    saver: u32,
    loader: u32,
    saved: Option<String>,
    pushes: Vec<String>,
}

struct Game(Rc<RefCell<Script>>);

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind != 0 {
            return Ok(String::new());
        }
        let w = call.engine;
        let tick = og_world_get_tick(w);
        let counter: f64 = serde_json::from_str(&og_world_find(w, "counter".into())).unwrap();
        let mut c = value(og_world_get(w, counter, "RpCount".into()));
        let num = |c: &Value, k: &str| c[k].as_f64().unwrap();
        match call.id {
            COUNT => {
                let n = num(&c, "n") + 1.0;
                c["n"] = json!(n);
                c["x"] = json!(num(&c, "x") * 1.25 + n);
            }
            SAVE => {
                let mut s = self.0.borrow_mut();
                if w == s.saver && tick == SAVE_AT {
                    s.saved = Some(value(og_world_snapshot(w)).as_str().unwrap().to_string());
                }
                if w == s.loader && tick == RESTORE_AT {
                    let saved = s.saved.clone().expect("saved first");
                    value(og_world_restore(w, saved, None));
                }
                if tick == PUSH_AT {
                    let boxed: f64 = serde_json::from_str(&og_world_find(w, "box".into())).unwrap();
                    let up = json!({ "x": 0.5, "y": 6.0, "z": 0.0 }).to_string();
                    value(og_physics_apply_impulse(w, boxed, up.clone(), None));
                    s.pushes
                        .push(og_physics_apply_impulse(w, counter, up, None));
                }
                return Ok(String::new());
            }
            MIX => {
                c["y"] = json!(num(&c, "y") + num(&c, "x") * 0.5 - num(&c, "n"));
            }
            _ => return Ok(String::new()),
        }
        value(og_world_set(w, counter, "RpCount".into(), c.to_string()));
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

fn game() -> u32 {
    og_host_begin_bundle();
    let fields = json!([
        { "name": "n", "kind": "f64", "default": 0, "options": {} },
        { "name": "x", "kind": "f64", "default": 1, "options": {} },
        { "name": "y", "kind": "f64", "default": 0, "options": {} },
    ]);
    value(og_reflect_define_component(
        "RpCount".into(),
        fields.to_string(),
    ));
    let count = json!({ "kind": "component", "name": "RpCount" });
    let writes = json!({ "writes": [count] }).to_string();
    let reads = json!({ "reads": [count], "writes": [] }).to_string();
    let systems = [
        og_system_define_system("Count".into(), writes.clone(), COUNT),
        og_system_define_system("Save".into(), reads, SAVE),
        og_system_define_system("Mix".into(), writes, MIX),
    ];
    og_game_game(Some(json!({ "seed": 3, "systems": systems }).to_string()))
}

fn engine(game: u32) -> u32 {
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let spawn = |desc: Value, components: &[(&str, Value)]| {
        let id = value(og_world_spawn(e, Some(desc.to_string())))
            .as_f64()
            .unwrap();
        for (name, v) in components {
            value(og_world_insert(
                e,
                id,
                name.to_string(),
                Some(v.to_string()),
            ));
        }
    };
    spawn(json!({ "name": "counter" }), &[("RpCount", json!({}))]);
    spawn(
        json!({ "position": { "x": 0, "y": -0.5, "z": 0 } }),
        &[
            ("og.RigidBody", json!({ "kind": 1 })),
            (
                "og.Collider",
                json!({ "shape": 0, "size": { "x": 20, "y": 1, "z": 20 } }),
            ),
        ],
    );
    spawn(
        json!({ "name": "box", "position": { "x": 0.3, "y": 1.5, "z": 0 } }),
        &[
            ("og.RigidBody", json!({ "mass": 2 })),
            (
                "og.Collider",
                json!({ "shape": 0, "size": { "x": 0.5, "y": 0.5, "z": 0.5 } }),
            ),
        ],
    );
    e
}

/// Steps `e` until its world reaches `tick`; answers its probe dump.
fn run_to(e: u32, tick: u32) -> Value {
    let mut frames = 0;
    while og_world_get_tick(e) < tick {
        value(og_engine_frame(e, 1000.0 / 60.0));
        frames += 1;
        assert!(frames < 200, "stuck at tick {}", og_world_get_tick(e));
    }
    assert_eq!(og_world_get_tick(e), tick);
    serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap()
}

#[test]
fn a_replay_from_a_save_taken_inside_a_system_matches_the_run_that_saved() {
    let script = Rc::new(RefCell::new(Script::default()));
    set_hooks(Rc::new(Game(script.clone())));
    let game = game();
    let a = engine(game);
    let b = engine(game);
    {
        let mut s = script.borrow_mut();
        s.saver = a;
        s.loader = b;
    }
    let ran = run_to(a, 30);
    assert!(script.borrow().saved.is_some());

    // B restores inside its tick 3: when that tick ends it holds A's
    // world as Save saw it in tick 8, the rest of tick 8 still to run.
    run_to(b, RESTORE_AT);
    value(og_engine_frame(b, 1000.0 / 60.0));
    assert_eq!(og_world_get_tick(b), SAVE_AT);
    let replayed = run_to(b, 30);
    assert_eq!(replayed["hash"], ran["hash"], "the replay diverged");
    let n = |e: u32| {
        let counter: f64 = serde_json::from_str(&og_world_find(e, "counter".into())).unwrap();
        value(og_world_get(e, counter, "RpCount".into()))["n"].clone()
    };
    assert_eq!(n(b), json!(30.0), "Count ran once a tick");
    assert_eq!(n(a), n(b));

    // The impulse from Save at tick 12 was applied by that tick's step:
    // the box (at rest on the floor at y 0.25 by then) is in the air at
    // tick 30; the counter, without a body, was refused.
    for e in [a, b] {
        let boxed: f64 = serde_json::from_str(&og_world_find(e, "box".into())).unwrap();
        let y = value(og_world_get(e, boxed, "og.Transform".into()))["position"]["y"]
            .as_f64()
            .unwrap();
        assert!(y > 0.6, "the box is at {y}");
    }
    let pushes = script.borrow().pushes.clone();
    assert_eq!(pushes.len(), 2, "{pushes:?}");
    for p in &pushes {
        let v: Value = serde_json::from_str(p).unwrap();
        assert_eq!(v["ok"], json!(false), "{p}");
        assert!(
            v["reason"].as_str().unwrap().contains("no og.RigidBody"),
            "{p}"
        );
    }
}
