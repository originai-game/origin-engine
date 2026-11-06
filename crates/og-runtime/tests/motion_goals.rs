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
