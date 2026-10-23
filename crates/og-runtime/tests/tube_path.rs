//! og.render.tubeGeometry along an og.Path (depth S1/S2's debt: the path
//! module's doc promised it): `desc.path` follows the path's own curve, its
//! rings spaced by arc length as three.js TubeGeometry spaces them, so a
//! tube along a Catmull-Rom path through evenly spaced points is the tube
//! through the same points (uniform and centripetal Catmull-Rom agree when
//! the chords are equal). Both are drawn side by side and raycast: every
//! ray hits each at the same distance.
//!
//! Ways this could fail: `path` is ignored (the desc has no points: the
//! old code refused it); the tube samples the path by its parameter
//! rather than its length, or takes the points' curve instead of the
//! path's; a dead path is accepted.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_path_create, og_render_geometry_info, og_render_raycast, og_render_standard_material,
    og_render_tube_geometry, og_world_insert, og_world_spawn, set_hooks,
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
fn a_tube_along_a_path_is_the_tube_through_its_points() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-tube-path-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = answer(og_engine_create(game, options)).as_u64().unwrap() as u32;

    // A helix: every chord the same length.
    let points: Vec<Value> = (0..9)
        .map(|i| {
            let a = i as f64 * 0.6;
            json!({ "x": 3.0 * a.cos(), "y": i as f64 * 0.4, "z": 3.0 * a.sin() })
        })
        .collect();
    let path = answer(og_path_create(
        w,
        Some(json!({ "points": points, "kind": "catmullRom" }).to_string()),
    ));
    let shape = json!({ "tubularSegments": 96, "radius": 0.3, "radialSegments": 12 });
    let mut along = shape.clone();
    along["path"] = path.clone();
    let mut through = shape.clone();
    through["points"] = json!(points);
    let a = answer(og_render_tube_geometry(w, Some(along.to_string())));
    let b = answer(og_render_tube_geometry(w, Some(through.to_string())));
    let (ia, ib) = (
        answer(og_render_geometry_info(w, a.as_f64().unwrap())),
        answer(og_render_geometry_info(w, b.as_f64().unwrap())),
    );
    assert_eq!(ia["vertices"], ib["vertices"]);
    assert_eq!(ia["triangles"], ib["triangles"]);
    for corner in ["min", "max"] {
        for c in ["x", "y", "z"] {
            let (x, y) = (
                ia[corner][c].as_f64().unwrap(),
                ib[corner][c].as_f64().unwrap(),
            );
            assert!((x - y).abs() < 0.02, "{corner}.{c}: {x} vs {y}");
        }
    }

    // Drawn side by side, 100 m apart in x, and raycast down at the same
    // offsets over the helix: the same hits.
    let material = answer(og_render_standard_material(w, Some("{}".into())));
    for (geometry, x) in [(&a, 0.0), (&b, 100.0)] {
        let e = answer(og_world_spawn(w, Some("{}".into())))
            .as_f64()
            .unwrap();
        answer(og_world_insert(
            w,
            e,
            "og.Transform".into(),
            Some(json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
        ));
        answer(og_world_insert(
            w,
            e,
            "og.Mesh".into(),
            Some(json!({ "geometry": geometry, "material": material }).to_string()),
        ));
    }
    answer(og_engine_frame(w, 1000.0 / 60.0));
    let down = |x: f64, z: f64| -> Option<f64> {
        let hits = answer(og_render_raycast(
            w,
            json!({ "origin": { "x": x, "y": 20, "z": z }, "direction": { "x": 0, "y": -1, "z": 0 } })
                .to_string(),
            Some(json!({ "first": true }).to_string()),
        ));
        hits.as_array()
            .unwrap()
            .first()
            .map(|h| h["distance"].as_f64().unwrap())
    };
    let mut hit = 0;
    for i in 0..9 {
        let s = i as f64 * 0.6 + 0.3;
        for r in [2.85, 3.0, 3.15] {
            let (x, z) = (r * s.cos(), r * s.sin());
            let (da, db) = (down(x, z), down(x + 100.0, z));
            match (da, db) {
                (Some(da), Some(db)) => {
                    hit += 1;
                    assert!((da - db).abs() < 0.02, "at ({x:.2}, {z:.2}): {da} vs {db}");
                }
                (None, None) => {}
                other => panic!("at ({x:.2}, {z:.2}): {other:?}"),
            }
        }
    }
    assert!(hit >= 15, "{hit} rays hit both tubes");

    // A dead path is refused, with its reason.
    let mut dead = shape.clone();
    dead["path"] = json!(path.as_f64().unwrap() + 40.0);
    let v: Value =
        serde_json::from_str(&og_render_tube_geometry(w, Some(dead.to_string()))).unwrap();
    assert_eq!(v["ok"], json!(false));
    assert!(
        v["reason"].as_str().unwrap().contains("not a live path"),
        "{v}"
    );
}
