//! og.WorldOrigin (R12): the world shifts as a tick starts once its focus
//! strays past `step`, and the simulation goes on across the shift as if
//! nothing moved but the numbers.
//!
//! A stack of three boxes rests asleep on a static floor 2,000 m out along x, with
//! og.WorldOrigin { step 2048 } and og.Camera (the focus, no anchor) beside
//! it. At tick 60 the camera moves to x 2,100, so the next tick starts
//! with a shift of -2048 on x.
//!
//! Ways this could fail, and what catches each:
//! - a body, a static collider or the rapier world is left where it was
//!   (the stack falls through or is flung), or contacts and warm starts are
//!   lost so the resting stack jolts, or the bodies are teleported (read
//!   as game code moving them: the sleeping stack wakes) instead of moved
//!   in place → `a_resting_stack_rides_the_shift`;
//! - the shift depends on anything but world state (wall time, I/O), or
//!   rapier's broad phase after the in-place move orders pairs otherwise
//!   from run to run → `a_replay_and_a_restore_go_tick_for_tick_across_the_shift`.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_world_get, og_world_get_tick, og_world_insert, og_world_restore,
    og_world_snapshot, og_world_spawn, set_hooks, write_resource,
};
use serde_json::{Value, json};
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
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

fn resource(e: u32, name: &str, v: Value) {
    let w: Value = serde_json::from_str(&write_resource(e, name, &v.to_string())).unwrap();
    assert_eq!(w["ok"], json!(true), "{w}");
}

const X: f64 = 2000.0;

/// The floor and the stack (bottom to top), and the engine.
fn engine() -> (u32, f64, Vec<f64>) {
    engine_at(X, 0.0, true)
}

/// The scene at `x`, its boxes `gap` apart (so they fall onto each other),
/// with or without og.WorldOrigin.
fn engine_at(x: f64, gap: f64, shifts: bool) -> (u32, f64, Vec<f64>) {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let body = |y: f64, kind: u32, size: Value| {
        let id = value(og_runtime::host_impl::answer(og_world_spawn(
            e,
            Some(json!({ "position": { "x": x, "y": y, "z": 0 } }).to_string()),
        )))
        .as_f64()
        .unwrap();
        for (name, v) in [
            ("og.RigidBody", json!({ "kind": kind })),
            ("og.Collider", json!({ "shape": "box", "size": size })),
        ] {
            value(og_runtime::host_impl::answer(og_world_insert(
                e,
                id,
                name.into(),
                Some(v.to_string()),
            )));
        }
        id
    };
    let floor = body(-0.5, 1, json!({ "x": 20, "y": 1, "z": 20 }));
    let unit = json!({ "x": 1, "y": 1, "z": 1 });
    let stack = (0..3)
        .map(|i| body(0.5 + i as f64 * (1.0 + gap), 0, unit.clone()))
        .collect();
    // A settled pile sleeps (og.Physics.sleepTicks, as new projects set it).
    resource(e, "og.Physics", json!({ "sleepTicks": 30 }));
    if shifts {
        resource(e, "og.WorldOrigin", json!({ "step": 2048 }));
    }
    camera(e, x);
    (e, floor, stack)
}

fn camera(e: u32, x: f64) {
    resource(
        e,
        "og.Camera",
        json!({ "position": { "x": x, "y": 3, "z": 8 } }),
    );
}

fn frame(e: u32) {
    value(og_engine_frame(e, 1000.0 / 60.0));
}

fn get(e: u32, id: f64, comp: &str) -> Value {
    value(og_runtime::host_impl::answer(og_world_get(
        e,
        id,
        comp.into(),
    )))
}

fn pos(e: u32, id: f64) -> [f64; 3] {
    let p = &get(e, id, "og.Transform")["position"];
    ["x", "y", "z"].map(|k| p[k].as_f64().unwrap())
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

fn origin(e: u32) -> Value {
    let dump =
        og_runtime::host_impl::answer(og_runtime::og_world_resource(e, "og.WorldOrigin".into()));
    value(dump)
}

/// Steps to tick 120, moving the camera to 2,100 m at tick 60; each tick's
/// hash from `from`.
fn script(e: u32, from: u32) -> Vec<Value> {
    let mut out = Vec::new();
    for t in from..120 {
        if t == 60 {
            camera(e, 2100.0);
        }
        frame(e);
        out.push(hash(e));
    }
    out
}

#[test]
fn a_resting_stack_rides_the_shift() {
    let (e, floor, stack) = engine();
    while og_world_get_tick(e) < 60 {
        frame(e);
    }
    let top = *stack.last().unwrap();
    let before: Vec<[f64; 3]> = stack.iter().map(|&b| pos(e, b)).collect();
    assert_eq!(origin(e)["shifts"].as_f64(), Some(0.0), "not yet");
    let asleep = |e: u32| {
        stack
            .iter()
            .all(|&b| get(e, b, "og.RigidBody")["sleeping"] == json!(true))
    };
    assert!(asleep(e), "the stack has settled asleep");
    camera(e, 2100.0);
    frame(e);
    // The next tick started with the shift: the zero is 2048 m out, the floor
    // (a static, never written by physics) moved by exactly that.
    let o = origin(e);
    assert_eq!(
        (o["x"].as_f64(), o["shifts"].as_f64()),
        (Some(2048.0), Some(1.0)),
        "{o}"
    );
    assert_eq!(pos(e, floor), [X - 2048.0, -0.5, 0.0]);
    // The resting boxes moved with it, to a hair of what a tick does.
    for (b, was) in stack.iter().zip(&before) {
        let now = pos(e, *b);
        assert!(
            (now[0] - (was[0] - 2048.0)).abs() < 1e-3,
            "{now:?} vs {was:?}"
        );
        assert!((now[1] - was[1]).abs() < 1e-3, "{now:?} vs {was:?}");
    }
    // Moved in rapier as they lie, not teleported: still asleep.
    assert!(asleep(e), "the shift woke the stack");
    // And stay at rest on the floor's collider for a second: nothing jolts.
    for _ in 0..60 {
        frame(e);
    }
    let p = pos(e, top);
    assert!(
        (p[0] - (X - 2048.0)).abs() < 1e-2 && (p[1] - 2.5).abs() < 1e-2,
        "{p:?}"
    );
    let v = &get(e, top, "og.Velocity")["linear"];
    let speed = ["x", "y", "z"]
        .map(|k| v[k].as_f64().unwrap_or(0.0))
        .iter()
        .map(|c| c * c)
        .sum::<f64>()
        .sqrt();
    assert!(speed < 1e-2, "the top box moves at {speed} m/s");
}

#[test]
fn a_replay_and_a_restore_go_tick_for_tick_across_the_shift() {
    let (a, ..) = engine();
    let first = script(a, 0);
    assert_eq!(origin(a)["shifts"].as_f64(), Some(1.0));
    let (b, ..) = engine();
    assert_eq!(script(b, 0), first, "a second run");
    // A save taken before the shift, restored into a fresh session, goes on
    // as the first run did; the tick after a restore delivers
    // og.WorldRestored, so its hash is its own.
    let (c, ..) = engine();
    while og_world_get_tick(c) < 40 {
        frame(c);
    }
    let save = value(og_runtime::host_impl::answer(og_world_snapshot(c, None)))
        .as_str()
        .unwrap()
        .to_string();
    let (d, ..) = engine();
    value(og_runtime::host_impl::answer(og_world_restore(
        d, save, None,
    )));
    let third = script(d, 40);
    assert_eq!(third[1..], first[41..]);
    // And a save taken after it (a rollback to a tick past the shift):
    // the restored world keeps its zero and does not shift again.
    while og_world_get_tick(c) < 80 {
        if og_world_get_tick(c) == 60 {
            camera(c, 2100.0);
        }
        frame(c);
    }
    let save = value(og_runtime::host_impl::answer(og_world_snapshot(c, None)))
        .as_str()
        .unwrap()
        .to_string();
    let (f, ..) = engine();
    value(og_runtime::host_impl::answer(og_world_restore(
        f, save, None,
    )));
    let fourth = script(f, 80);
    assert_eq!(fourth[1..], first[81..]);
    assert_eq!(origin(f)["shifts"].as_f64(), Some(1.0));
}
