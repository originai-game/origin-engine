//! og.CollisionStarted through the host ABI carries the hit (Rumpus Isle
//! FINDINGS #9): `impulse` (N·s) and `relativeVelocity` (a relative to b
//! at the contact point, before the solver), as a game's system reads it
//! with `world.events`, in both profiles; a save taken before the hit and
//! restored gives the same event, value for value.
//!
//! Ways this could fail: the runtime's reflection serves the old layout
//! (the fields are missing, or read from the wrong bytes: a zero impulse,
//! a velocity in the impulse's place); the velocity is post-solver (its
//! closing speed is 0); a restore replays another hit.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_system_define_system, og_world_events, og_world_get_tick, og_world_insert, og_world_restore,
    og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const WATCH: u32 = 1;

/// (tick, each og.CollisionStarted Watch read in it).
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
                serde_json::from_str(&og_world_events(w, "og.CollisionStarted".into())).unwrap();
            for ev in seen.as_array().unwrap() {
                self.0.borrow_mut().push((og_world_get_tick(w), ev.clone()));
            }
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// A 1 kg ball at 6 m/s along +x into a 10 kg ball at rest beside it, both
/// falling; frictionless. Returns the engine and the two entities.
fn engine(game: u32, profile: &str) -> (u32, f64, f64) {
    let e = value(og_engine_create(
        game,
        json!({ "profile": profile }).to_string(),
    ))
    .as_u64()
    .unwrap() as u32;
    let ball = |x: f32, mass: f32, vx: f32| {
        let id = value(og_world_spawn(
            e,
            Some(json!({ "position": { "x": x, "y": 3, "z": 0 } }).to_string()),
        ))
        .as_f64()
        .unwrap();
        for (name, v) in [
            ("og.RigidBody", json!({ "mass": mass })),
            (
                "og.Collider",
                json!({ "shape": 1, "radius": 0.5, "friction": 0 }),
            ),
            (
                "og.Velocity",
                json!({ "linear": { "x": vx, "y": 0, "z": 0 } }),
            ),
        ] {
            value(og_world_insert(e, id, name.into(), Some(v.to_string())));
        }
        id
    };
    let light = ball(-2.0, 1.0, 6.0);
    let heavy = ball(0.0, 10.0, 0.0);
    (e, light, heavy)
}

fn frame(e: u32) {
    value(og_engine_frame(e, 1000.0 / 60.0));
}

/// Steps until Watch has read a CollisionStarted; returns it and the tick.
fn until_hit(e: u32, seen: &RefCell<Vec<(u32, Value)>>) -> (u32, Value) {
    for _ in 0..120 {
        frame(e);
        if let Some(hit) = seen.borrow().first() {
            assert_eq!(seen.borrow().len(), 1, "{:?}", seen.borrow());
            return hit.clone();
        }
    }
    panic!("no og.CollisionStarted read in 120 ticks");
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

#[test]
fn a_game_reads_the_impulse_and_the_relative_velocity_and_a_restore_replays_them() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_hooks(Rc::new(Watch(seen.clone())));
    og_host_begin_bundle();
    let started = json!({ "kind": "event", "name": "og.CollisionStarted" });
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [started], "writes": [] }).to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [watch] }).to_string()));

    for profile in ["3d", "2d"] {
        seen.borrow_mut().clear();
        let (e, light, heavy) = engine(game, profile);
        for _ in 0..3 {
            frame(e);
        }
        let save = value(og_world_snapshot(e)).as_str().unwrap().to_string();
        let (tick, hit) = until_hit(e, &seen);
        assert_eq!(f(&hit["a"]), light, "{profile}: {hit}");
        assert_eq!(f(&hit["b"]), heavy, "{profile}: {hit}");
        assert!(
            (f(&hit["normal"]["x"]) - 1.0).abs() < 1e-3,
            "{profile}: {hit}"
        );
        // The light ball's 6 m/s relative to the heavy one, a to b; both
        // fall alike, so nothing along y.
        let rv = &hit["relativeVelocity"];
        assert!((f(&rv["x"]) - 6.0).abs() < 1e-3, "{profile}: {hit}");
        assert!(f(&rv["y"]).abs() < 1e-3, "{profile}: {hit}");
        // Restitution 0: the closing speed taken out at the reduced mass.
        assert!(
            (f(&hit["impulse"]) - 6.0 * 10.0 / 11.0).abs() < 0.1,
            "{profile}: {hit}"
        );

        seen.borrow_mut().clear();
        value(og_world_restore(e, save, None));
        let (again_tick, again) = until_hit(e, &seen);
        assert_eq!((again_tick, &again), (tick, &hit), "{profile}: the replay");
    }
}
