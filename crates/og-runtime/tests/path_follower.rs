//! og.PathFollower with `align`: the entity faces along the path as an
//! object does (its +Z along the tangent, `og_core::facing`), and `offset`
//! is in the path's frame: x to the right of travel, y up, z backwards.
//!
//! Ways this could fail: the entity is turned as a camera (-Z along the
//! tangent, so a model travels backwards); the offset's x lands on the
//! left of travel; the offset is applied in world axes.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_path_create, og_world_get, og_world_insert, og_world_spawn, set_hooks,
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

fn answer(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

#[test]
fn an_aligned_follower_faces_along_the_path_with_its_offset_to_the_right() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-path-follower-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = answer(og_engine_create(game, options)).as_u64().unwrap() as u32;

    // A straight track along +X.
    let points = json!([{ "x": 0, "y": 0, "z": 0 }, { "x": 20, "y": 0, "z": 0 }]);
    let path = answer(og_runtime::host_impl::answer(og_path_create(
        w,
        Some(json!({ "points": points, "kind": "linear" }).to_string()),
    )));
    let e = answer(og_runtime::host_impl::answer(og_world_spawn(
        w,
        Some("{}".into()),
    )))
    .as_f64()
    .unwrap();
    for (ty, fields) in [
        ("og.Transform", json!({})),
        (
            "og.PathFollower",
            json!({ "path": path, "speed": 6, "loop": false, "align": true, "offset": { "x": 1, "y": 2, "z": 3 } }),
        ),
    ] {
        answer(og_runtime::host_impl::answer(og_world_insert(
            w,
            e,
            ty.into(),
            Some(fields.to_string()),
        )));
    }
    for _ in 0..60 {
        answer(og_engine_frame(w, 1000.0 / 60.0));
    }
    let t = answer(og_runtime::host_impl::answer(og_world_get(
        w,
        e,
        "og.Transform".into(),
    )));
    let n = |v: &Value, k: &str| v[k].as_f64().unwrap() as f32;
    let q = &t["quaternion"];
    let q = glam::Quat::from_xyzw(n(q, "x"), n(q, "y"), n(q, "z"), n(q, "w"));
    let forward = og_core::facing::forward(q);
    assert!(
        forward.distance(glam::Vec3::X) < 1e-4,
        "an aligned follower faces along the path: {forward}"
    );
    // Six metres along; one to the right of travel (+Z when heading +X),
    // two up, three back.
    let p = &t["position"];
    let p = glam::Vec3::new(n(p, "x"), n(p, "y"), n(p, "z"));
    assert!(
        p.distance(glam::Vec3::new(3.0, 2.0, 1.0)) < 1e-3,
        "the offset is right, up and back in the path's frame: {p}"
    );
}
