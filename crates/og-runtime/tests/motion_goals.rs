//! og.motion's goals are the simulation's (docs/design.md §2.4, §2.7): a
//! driven og.MotionAgent walks to its goal at its speed with fixed math,
//! goals finish by their rules in order, og.motion.status answers the
//! queue, and a snapshot taken mid-walk restores into a run that hashes as
//! the uninterrupted one. No renderer is attached, so the model never runs:
//! nothing here can depend on it.
//!
//! Ways this could fail: the mover integrates speed wrongly (arrives early or
//! late); a goal finishes before the walk ends; the queue is not world state
//! (a restore loses it and the hashes part); walkTo is taken without drive.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_motion_pose, og_motion_presentation, og_motion_status,
    og_motion_walk_to, og_world_get, og_world_insert, og_world_restore, og_world_snapshot,
    og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn start(drive: bool) -> (u32, f64) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-motion-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let e = value(og_world_spawn(
        w,
        Some(json!({ "name": "hero" }).to_string()),
    ))
    .as_f64()
    .unwrap();
    value(og_world_insert(
        w,
        e,
        "og.MotionAgent".into(),
        Some(json!({ "drive": drive }).to_string()),
    ));
    (w, e)
}

fn frames(w: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(w, 1000.0 / 60.0));
    }
}

fn position(w: u32, e: f64) -> (f64, f64) {
    let t = value(og_world_get(w, e, "og.Transform".into()));
    (
        t["position"]["x"].as_f64().unwrap(),
        t["position"]["z"].as_f64().unwrap(),
    )
}

fn hash(w: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(w, None)).unwrap();
    dump["hash"].clone()
}

fn walk(w: u32, e: f64) {
    // 2.8 m along +X, braking from 1.4 m/s; then a 0.5 s cheer.
    value(og_motion_walk_to(
        w,
        e,
        Some(
            json!({ "position": { "x": 2.8, "y": 0, "z": 0 }, "speed": 1.4, "pose": "wave" })
                .to_string(),
        ),
    ));
    value(og_motion_pose(
        w,
        e,
        Some(json!({ "pose": "cheer", "duration": 0.5 }).to_string()),
    ));
}

#[test]
fn presentation_is_read_only_and_refuses_non_agents() {
    let (w, e) = start(true);
    let before = hash(w);
    let snapshot = og_world_snapshot(w, None);
    let status = og_motion_status(w, e);
    let presentation = value(og_motion_presentation(w, e));
    assert_eq!(presentation["state"], "loading");
    assert!(!presentation["reason"].as_str().unwrap().is_empty());
    assert_eq!(hash(w), before);
    assert_eq!(og_world_snapshot(w, None), snapshot);
    assert_eq!(og_motion_status(w, e), status);
    let other = value(og_world_spawn(w, None)).as_f64().unwrap();
    let refused: Value = serde_json::from_str(&og_motion_presentation(w, other)).unwrap();
    assert_eq!(refused["ok"], false);
    let invalid: Value = serde_json::from_str(&og_motion_presentation(u32::MAX, e)).unwrap();
    assert_eq!(invalid["ok"], false);
    let scheduled: Value = serde_json::from_str(&og_runtime::og_engine_probe_commands(
        w,
        "0:dump_motion".into(),
    ))
    .unwrap();
    assert_eq!(scheduled["scheduled"], 1);
    frames(w, 1);
    let outputs: Value = serde_json::from_str(&og_runtime::og_engine_take_outputs(w)).unwrap();
    let dump = outputs
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["action"] == "dump_motion")
        .unwrap();
    let dump: Value = serde_json::from_str(dump["data"].as_str().unwrap()).unwrap();
    assert_eq!(dump["agents"][0]["entity"], e);
    assert_eq!(
        dump["agents"][0]["presentation"],
        value(og_motion_presentation(w, e))
    );
}

#[test]
fn a_driven_agent_walks_its_goals_and_restores_mid_walk() {
    let (w, e) = start(true);
    assert_eq!(value(og_motion_status(w, e))["goal"], json!("none"));
    walk(w, e);
    frames(w, 60);
    let s = value(og_motion_status(w, e));
    assert_eq!(s["goal"], json!("walkTo"));
    assert_eq!(s["pose"], json!("wave"));
    assert_eq!(s["queued"], json!(1));
    // The agent starts facing +Z and turns at 6 rad/s, so it is a little
    // short of 1 s at 1.4 m/s; it is past half way and has not overshot.
    let (x, z) = position(w, e);
    assert!(x > 1.2 && x < 1.4, "x {x} after 1 s");
    assert!(z.abs() < 0.2, "z {z}");
    let saved = value(og_world_snapshot(w, None))
        .as_str()
        .unwrap()
        .to_string();

    frames(w, 90);
    let s = value(og_motion_status(w, e));
    assert_eq!(s["goal"], json!("pose"), "the walk ended on arrival: {s}");
    assert_eq!(s["pose"], json!("cheer"));
    assert_eq!(s["completed"], json!(1));
    let (x, z) = position(w, e);
    assert!(
        (x - 2.8).abs() < 1e-4 && z.abs() < 1e-4,
        "arrived at ({x}, {z})"
    );
    frames(w, 40);
    let s = value(og_motion_status(w, e));
    assert_eq!(s["goal"], json!("none"));
    assert_eq!(s["completed"], json!(2));
    let ran = hash(w);

    // Restored at 1 s into a fresh engine with no goals: it finishes the same.
    let (v, f) = start(true);
    frames(v, 60);
    value(og_world_restore(v, saved, None));
    assert_eq!(value(og_motion_status(v, f))["goal"], json!("walkTo"));
    frames(v, 130);
    assert_eq!(hash(v), ran, "the restored run diverged");
}

fn status(w: u32, e: f64) -> Value {
    value(og_motion_status(w, e))
}

fn height(w: u32, e: f64) -> f64 {
    value(og_world_get(w, e, "og.Transform".into()))["position"]["y"]
        .as_f64()
        .unwrap()
}

fn jump(w: u32, e: f64, desc: Value) {
    value(og_runtime::og_motion_jump(w, e, Some(desc.to_string())));
}

// A jump the simulation owns (docs/design.md §2.7, "Jumps"). Without a
// controller the og.motion system flies the fixed ballistic arc. Ways this
// could fail: takeoff before the preparation ends; an apex off the goal's
// height (a v0 doubled or taken from the model); travel not along the
// heading or not exactly the distance; landing below or above the takeoff
// height; the goal ending at touchdown instead of after its recovery; the
// jump's state not in the snapshot (a restore mid-flight lands elsewhere).
#[test]
fn a_jump_without_a_controller_flies_its_fixed_arc_and_restores_mid_flight() {
    let (w, e) = start(true);
    let half_pi = std::f64::consts::FRAC_PI_2;
    let desc = json!({ "height": 0.5, "distance": 1.2, "heading": half_pi, "prepare": 0.25, "recover": 0.35 });
    jump(w, e, desc.clone());
    let dt = 1.0 / 60.0;
    let v0 = (2.0 * 9.81 * 0.5_f64).sqrt();
    let flight = 2.0 * v0 / 9.81;
    let (mut apex, mut air, mut saved, mut first_air, mut tick) = (0.0_f64, 0, None, None, 0);
    // Until the first tick after touchdown.
    while first_air.is_none() || status(w, e)["airborne"] == json!(true) {
        tick += 1;
        assert!(tick < 120, "never landed");
        frames(w, 1);
        let y = height(w, e);
        apex = apex.max(y);
        let s = status(w, e);
        if s["airborne"] == json!(true) {
            air += 1;
            first_air.get_or_insert(tick);
            if air == 10 {
                saved = Some(
                    value(og_world_snapshot(w, None))
                        .as_str()
                        .unwrap()
                        .to_string(),
                );
            }
        }
        if (tick as f64) * dt < 0.25 - 1e-6 {
            assert_eq!(y, 0.0, "tick {tick}: off the ground before takeoff");
            assert_eq!(s["goal"], json!("jump"));
        }
    }
    let takeoff = first_air.unwrap() as f64 * dt;
    assert!(
        (takeoff - 0.25).abs() <= dt + 1e-6,
        "takeoff at {takeoff} s"
    );
    assert!(apex <= 0.5 + 1e-5 && apex > 0.5 - 0.002, "apex {apex}");
    assert!(
        ((air as f64) * dt - flight).abs() <= 2.0 * dt,
        "{air} ticks in the air for a {flight} s flight"
    );
    let s = status(w, e);
    assert_eq!(s["goal"], json!("jump"), "recovering after touchdown: {s}");
    assert_eq!(height(w, e), 0.0, "landed at its takeoff height");
    let (x, z) = position(w, e);
    assert!(
        (x - 1.2).abs() < 1e-4 && z.abs() < 1e-4,
        "travelled to ({x}, {z})"
    );
    frames(w, 15);
    assert_eq!(
        status(w, e)["goal"],
        json!("jump"),
        "still recovering 0.25 s after landing"
    );
    frames(w, 30);
    let s = status(w, e);
    assert_eq!(
        s["goal"],
        json!("none"),
        "the jump ended after its recovery: {s}"
    );
    assert_eq!(s["completed"], json!(1));
    let ran = hash(w);

    let (v, f) = start(true);
    jump(v, f, desc);
    value(og_world_restore(v, saved.unwrap(), None));
    assert_eq!(status(v, f)["airborne"], json!(true));
    frames(v, tick + 15 + 30 - (first_air.unwrap() + 9));
    assert_eq!(
        hash(v),
        ran,
        "the jump restored mid-flight landed elsewhere"
    );
}

/// A world with ground (top at y = 0), an optional ledge (top at
/// `ledge` over 1 <= x <= 3) and a capsule character standing on the
/// ground at the origin with a CharacterController.
fn controller_world(ledge: Option<f64>) -> (u32, f64) {
    let (w, e) = start(true);
    for (name, desc) in [
        (
            "og.Transform",
            json!({ "position": { "x": 0, "y": 0.6, "z": 0 } }),
        ),
        (
            "og.Collider",
            json!({ "shape": 2, "radius": 0.3, "height": 0.6 }),
        ),
        ("og.CharacterController", json!({})),
    ] {
        value(og_world_insert(w, e, name.into(), Some(desc.to_string())));
    }
    let mut boxes = vec![(
        json!({ "x": 0, "y": -0.5, "z": 0 }),
        json!({ "x": 20, "y": 1, "z": 20 }),
    )];
    if let Some(top) = ledge {
        boxes.push((
            json!({ "x": 2, "y": top - 0.5, "z": 0 }),
            json!({ "x": 2, "y": 1, "z": 4 }),
        ));
    }
    for (at, size) in boxes {
        let b = value(og_world_spawn(w, None)).as_f64().unwrap();
        value(og_world_insert(
            w,
            b,
            "og.Transform".into(),
            Some(json!({ "position": at }).to_string()),
        ));
        value(og_world_insert(
            w,
            b,
            "og.Collider".into(),
            Some(json!({ "shape": 0, "size": size }).to_string()),
        ));
    }
    frames(w, 30);
    (w, e)
}

fn controller(w: u32, e: f64) -> Value {
    value(og_world_get(w, e, "og.CharacterController".into()))
}

// With a CharacterController the jump is the controller's. Ways this could
// fail: the apex misses the goal's height (jumpSpeed without the discrete
// correction peaks about 3 cm low at 60 Hz); `jump` left held, so the
// controller never sees the next rising edge; the goal ends where the
// fixed arc would have landed rather than on the controller's landing
// (a ledge up), or never ends.
#[test]
fn a_controller_jump_is_the_controllers_and_lands_on_a_ledge() {
    let (w, e) = controller_world(None);
    assert_eq!(
        controller(w, e)["grounded"],
        json!(true),
        "settled on the ground"
    );
    let base = height(w, e);
    jump(
        w,
        e,
        json!({ "height": 0.6, "prepare": 0.1, "recover": 0.2 }),
    );
    let mut apex = base;
    for _ in 0..60 {
        frames(w, 1);
        apex = apex.max(height(w, e));
    }
    assert!(
        (apex - base - 0.6).abs() < 0.01,
        "apex {} over the ground for 0.6",
        apex - base
    );
    assert_eq!(status(w, e)["goal"], json!("none"));
    assert_eq!(
        controller(w, e)["jump"],
        json!(false),
        "jump released after takeoff"
    );
    // A second jump starts: the controller saw a new rising edge.
    jump(
        w,
        e,
        json!({ "height": 0.3, "prepare": 0.0, "recover": 0.0 }),
    );
    let mut again = base;
    for _ in 0..40 {
        frames(w, 1);
        again = again.max(height(w, e));
    }
    assert!(
        (again - base - 0.3).abs() < 0.01,
        "second apex {}",
        again - base
    );

    // Up onto a ledge 0.3 m high, 1.6 m away: it lands on it, higher than it took off.
    let (w, e) = controller_world(Some(0.3));
    let base = height(w, e);
    jump(
        w,
        e,
        json!({ "height": 0.8, "distance": 1.6, "heading": std::f64::consts::FRAC_PI_2, "prepare": 0.1, "recover": 0.2 }),
    );
    let mut landed = None;
    for tick in 0..90 {
        frames(w, 1);
        if landed.is_none() && status(w, e)["goal"] == json!("none") {
            landed = Some(tick);
        }
    }
    assert!(landed.is_some(), "the jump onto the ledge ended");
    assert!(
        (height(w, e) - base - 0.3).abs() < 0.02,
        "standing {} above the start",
        height(w, e) - base
    );
    let (x, _) = position(w, e);
    // It flies at the flat-ground travel speed and lands early, on the ledge.
    assert!(x > 1.0 && x < 1.6, "on the ledge at x {x}");
    assert_eq!(controller(w, e)["grounded"], json!(true));
}
