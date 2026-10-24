//! `uvRepeat` on og-geom's geometry descs (shape, extrude, lathe, the
//! solids, polyhedron, csg, merge), as boxGeometry's: a texture repeats
//! uvRepeat times across the geometry, its uvs scaled. Read back through a
//! raycast's uv, at the same point of the same geometry made without it.
//!
//! Ways this could fail: the field is ignored by the generators that go
//! through og-geom (the IDL had none before); it is applied to one axis or
//! twice.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_render_box_geometry, og_render_csg_geometry, og_render_extrude_geometry,
    og_render_icosahedron_geometry, og_render_lathe_geometry, og_render_merge_geometries,
    og_render_raycast, og_render_shape_geometry, og_render_standard_material, og_world_insert,
    og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::rc::Rc;

struct Disk;

impl Hooks for Disk {
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

fn answer(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

type Make = fn(u32, Option<String>) -> String;

#[test]
fn uv_repeat_scales_every_generated_geometry_s_uvs() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-uv-repeat-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = answer(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let material = answer(og_render_standard_material(w, Some("{}".into())));
    let cube = answer(og_render_box_geometry(w, None));
    let ball = answer(og_render_icosahedron_geometry(
        w,
        Some(json!({ "radius": 0.3, "detail": 1 }).to_string()),
    ));
    let cases: Vec<(&str, Make, Value)> = vec![
        ("shape", og_render_shape_geometry, json!({})),
        (
            "extrude",
            og_render_extrude_geometry,
            json!({ "depth": 0.5 }),
        ),
        (
            "lathe",
            og_render_lathe_geometry,
            json!({ "points": [{ "x": 0.2, "y": -0.6 }, { "x": 0.5, "y": 0 }, { "x": 0.2, "y": 0.6 }], "segments": 24 }),
        ),
        (
            "icosahedron",
            og_render_icosahedron_geometry,
            json!({ "detail": 2 }),
        ),
        (
            "csg",
            og_render_csg_geometry,
            json!({ "op": "subtract", "a": { "geometry": cube }, "b": { "geometry": ball, "position": { "x": 0.4, "y": 0.4, "z": 0.5 } } }),
        ),
        (
            "merge",
            og_render_merge_geometries,
            json!({ "parts": [{ "geometry": cube }] }),
        ),
    ];
    let mut entities = Vec::new();
    for (k, (name, make, desc)) in cases.iter().enumerate() {
        for (j, repeat) in [None, Some(json!({ "x": 3, "y": 2 }))]
            .into_iter()
            .enumerate()
        {
            let mut d = desc.clone();
            if let Some(r) = repeat {
                d["uvRepeat"] = r;
            }
            let g = answer(make(w, Some(d.to_string())));
            let e = answer(og_world_spawn(w, Some("{}".into())))
                .as_f64()
                .unwrap();
            let x = k as f64 * 10.0;
            let y = j as f64 * 10.0;
            for (c, v) in [
                (
                    "og.Transform",
                    json!({ "position": { "x": x, "y": y, "z": 0 } }),
                ),
                ("og.Mesh", json!({ "geometry": g, "material": material })),
            ] {
                answer(og_world_insert(w, e, c.into(), Some(v.to_string())));
            }
            entities.push((*name, x, y));
        }
    }
    answer(og_engine_frame(w, 1000.0 / 60.0));
    let uv = |x: f64, y: f64| -> (f64, f64) {
        let hits = answer(og_render_raycast(
            w,
            json!({ "origin": { "x": x + 0.13, "y": y + 0.07, "z": 5 }, "direction": { "x": 0, "y": 0, "z": -1 } })
                .to_string(),
            Some(json!({ "first": true }).to_string()),
        ));
        let h = &hits[0]["uv"];
        (h["x"].as_f64().unwrap(), h["y"].as_f64().unwrap())
    };
    for pair in entities.chunks(2) {
        let (name, x, y0) = pair[0];
        let y1 = pair[1].2;
        let (plain, repeated) = (uv(x, y0), uv(x, y1));
        assert!(
            plain.0.abs() > 1e-3 && plain.1.abs() > 1e-3,
            "{name}: a uv on an axis tells nothing: {plain:?}"
        );
        assert!(
            (repeated.0 - plain.0 * 3.0).abs() < 1e-4 && (repeated.1 - plain.1 * 2.0).abs() < 1e-4,
            "{name}: {plain:?} repeated 3 2 is {repeated:?}"
        );
    }
}
