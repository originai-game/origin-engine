//! Time groups (G2-6): og.time.setGroupScale slows the entities with
//! og.TimeGroup in that group, and nothing else; the types are registered
//! only once a game names them, so a world without them hashes as before.
//!
//! Ways this could fail: a group's scale applied to every entity, or to
//! none; the global scale dropped for grouped entities (they must take
//! both); an unscaled tween slowed by its group; the types registered in
//! every world (moving every hash); a save of a grouped world that a fresh
//! session cannot restore, or restores at the default scales.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_time_group_dt, og_time_set_group_scale, og_time_set_scale,
    og_tween_to, og_world_get, og_world_insert, og_world_resource, og_world_restore,
    og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::{path::PathBuf, rc::Rc};

struct Game;

impl Hooks for Game {
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
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], true, "{text}");
    v["value"].clone()
}

fn hash(engine: u32) -> String {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].as_str().unwrap().to_string()
}

fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("og-runtime-{name}"));
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    root
}

fn start(root: &PathBuf) -> u32 {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    value(og_engine_create(game, json!({ "root": root }).to_string()))
        .as_u64()
        .unwrap() as u32
}

fn frames(engine: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

/// An entity with og.Transform (and og.TimeGroup when `group` is given),
/// its position.x tweened linearly from 0 to 60 over one second.
fn mover(engine: u32, group: Option<u32>, unscaled: bool) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.Transform".into(),
        Some("{}".into()),
    ));
    if let Some(g) = group {
        value(og_world_insert(
            engine,
            e,
            "og.TimeGroup".into(),
            Some(json!({ "group": g }).to_string()),
        ));
    }
    value(og_tween_to(
        engine,
        Some(
            json!({
                "entity": e, "field": "position.x", "from": [0.0], "to": [60.0],
                "duration": 1.0, "ease": "linear", "unscaled": unscaled,
            })
            .to_string(),
        ),
    ));
    e
}

fn x(engine: u32, e: f64) -> f64 {
    value(og_world_get(engine, e, "og.Transform".into()))["position"]["x"]
        .as_f64()
        .unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn a_group_runs_at_its_scale_times_the_global_one() {
    set_hooks(Rc::new(Game));
    let engine = start(&project("time-groups"));
    let normal = mover(engine, None, false);
    let other = mover(engine, Some(2), false);
    let slow = mover(engine, Some(1), false);
    let ui = mover(engine, Some(1), true);
    value(og_time_set_group_scale(engine, 1, 0.25));
    frames(engine, 30);
    // Half a second: 30 at the global pace, a quarter of it in group 1;
    // group 2 is untouched, an unscaled tween ignores its group.
    let (n, o, s, u) = (
        x(engine, normal),
        x(engine, other),
        x(engine, slow),
        x(engine, ui),
    );
    assert!(close(n, 30.0) && close(o, 30.0), "{n} {o}");
    assert!(close(s, 7.5), "{s}");
    assert!(close(u, 30.0), "{u}");
    let dt = 1.0 / 60.0;
    assert!(close(og_time_group_dt(engine, slow), dt * 0.25));
    assert!(close(og_time_group_dt(engine, normal), dt));
    // The global scale multiplies with the group's.
    value(og_time_set_scale(engine, 0.5));
    frames(engine, 10);
    assert!(
        close(x(engine, normal), 30.0 + 5.0),
        "{}",
        x(engine, normal)
    );
    assert!(close(x(engine, slow), 7.5 + 1.25), "{}", x(engine, slow));
    assert!(close(og_time_group_dt(engine, slow), dt * 0.5 * 0.25));
    // Frozen.
    value(og_time_set_group_scale(engine, 1, 0.0));
    let held = x(engine, slow);
    frames(engine, 10);
    assert_eq!(x(engine, slow), held);
    // Refusals.
    let r: Value = serde_json::from_str(&og_time_set_group_scale(engine, 16, 1.0)).unwrap();
    assert_eq!(r["ok"], false, "{r}");
    let r: Value = serde_json::from_str(&og_time_set_group_scale(engine, 3, 11.0)).unwrap();
    assert_eq!(r["ok"], false, "{r}");
}

#[test]
fn a_grouped_world_restores_its_scales_in_a_fresh_session() {
    set_hooks(Rc::new(Game));
    let root = project("time-groups-save");
    let a = start(&root);
    let slow = mover(a, Some(4), false);
    value(og_time_set_group_scale(a, 4, 0.5));
    frames(a, 12);
    let save = value(og_world_snapshot(a, None));
    frames(a, 20);
    let (end, end_hash) = (x(a, slow), hash(a));
    // A session that never named the time groups.
    let b = start(&root);
    value(og_world_restore(b, save.as_str().unwrap().into(), None));
    assert!(close(og_time_group_dt(b, slow), 0.5 / 60.0));
    frames(b, 20);
    assert_eq!(x(b, slow), end);
    assert_eq!(hash(b), end_hash);
}

/// A world that never names a time group: pinned on main e3ec45ab6, the
/// code before G2-6 batch 3 (66118ec82a83f36d before G2-7 batch 3's
/// og.Lockstep fields), then 808247095fefdc21 after U6d's UI state version
/// 13 moved every raw world hash from 94fabc6049378510, then 83f0005d31df5c77
/// once G3-2 made og.VideoPlayer lazy (it left the type table of every world
/// without footage), then 4251b57bfa40912f once R11 made og.Lighting2D,
/// og.Light2D and og.Occluder2D lazy (the same for a world without 2D
/// lights); neither type is registered until a game uses it. Then
/// 3564e52caad6a083 after ui-gaps' UI state version 14, and 05d77e7f93ca8034
/// after version 15.
#[test]
fn a_world_without_time_groups_keeps_its_hash() {
    set_hooks(Rc::new(Game));
    let engine = start(&project("time-groups-none"));
    mover(engine, None, false);
    value(og_time_set_scale(engine, 0.5));
    frames(engine, 20);
    // Reading the resource answers its default without registering it.
    assert_eq!(
        value(og_world_resource(engine, "og.TimeGroups".into())),
        json!({ "scales": [] })
    );
    assert_eq!(hash(engine), "05d77e7f93ca8034");
}
