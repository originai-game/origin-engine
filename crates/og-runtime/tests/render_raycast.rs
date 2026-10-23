//! og.render.raycast against a procedural mesh answers the hit og.physics.raycast
//! answers against the collider made from the same geometry (depth S1's
//! acceptance), at a turned and moved transform, and both follow an
//! og.render.updateGeometry.
//!
//! Ways this could fail: the ray is not taken into the mesh's space (or its
//! distance not back out of it), the BVH misses or reorders triangles, the
//! normal is not carried through the transform's inverse transpose, or an
//! update leaves the old BVH (or the old physics mesh) in place.
//!
//! A cooked model (templates/game's dropship) raycasts without building a
//! BVH: each primitive's was cooked into its blob and loaded with its mesh
//! (before, the first raycast that reached it built it then).

use og_render::geometry::Bvh;
use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_load_gltf, og_physics_raycast, og_render_buffer_geometry, og_render_geometry_info,
    og_render_raycast, og_render_standard_material, og_render_update_geometry, og_world_insert,
    og_world_instantiate, og_world_spawn, set_hooks,
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

/// A 16 by 16 height field of quads over 8 by 8 metres, its height a
/// lopsided bump (`lift` raises it all), as positions and indices.
fn terrain(lift: f32) -> Value {
    let n = 16;
    let mut positions = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let (x, z) = (i as f32 * 0.5 - 4.0, j as f32 * 0.5 - 4.0);
            let y =
                lift + 1.5 * (-(x - 1.0).powi(2) / 4.0 - (z + 0.5).powi(2) / 2.0).exp() + 0.1 * x;
            positions.extend([x, y, z]);
        }
    }
    let mut indices = Vec::new();
    let at = |i: u32, j: u32| j * (n + 1) + i;
    for j in 0..n {
        for i in 0..n {
            let (a, b, c, d) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
            // Counter-clockwise seen from above.
            indices.extend([a, d, c, a, c, b]);
        }
    }
    json!({ "positions": positions, "indices": indices, "dynamic": true })
}

fn close(a: &Value, b: &Value, key: &str, tol: f64) {
    for c in ["x", "y", "z"] {
        let (x, y) = (a[key][c].as_f64().unwrap(), b[key][c].as_f64().unwrap());
        assert!((x - y).abs() < tol, "{key}.{c}: render {x}, physics {y}");
    }
}

#[test]
fn the_mesh_raycast_answers_the_collider_raycast() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-render-raycast-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = answer(og_engine_create(game, options)).as_u64().unwrap() as u32;

    let geometry = answer(og_render_buffer_geometry(w, Some(terrain(0.0).to_string())));
    let info = answer(og_render_geometry_info(w, geometry.as_f64().unwrap()));
    assert_eq!(info["vertices"], json!(289));
    assert_eq!(info["triangles"], json!(512));
    assert_eq!(info["dynamic"], json!(true));
    let material = answer(og_render_standard_material(w, Some("{}".into())));
    let e = answer(og_world_spawn(w, Some("{}".into())))
        .as_f64()
        .unwrap();
    // An eighth of a turn about y and a move: the ray must go into mesh space.
    let (s, c) = (
        std::f64::consts::FRAC_PI_8.sin(),
        std::f64::consts::FRAC_PI_8.cos(),
    );
    for (name, v) in [
        (
            "og.Transform",
            json!({ "position": { "x": 3, "y": -1, "z": 2 }, "quaternion": { "x": 0, "y": s, "z": 0, "w": c } }),
        ),
        (
            "og.Mesh",
            json!({ "geometry": geometry, "material": material }),
        ),
        ("og.RigidBody", json!({ "kind": 1 })),
        ("og.Collider", json!({ "shape": 5, "mesh": geometry })),
    ] {
        answer(og_world_insert(w, e, name.into(), Some(v.to_string())));
    }
    for _ in 0..3 {
        answer(og_engine_frame(w, 1000.0 / 60.0));
    }

    let rays = [
        ((3.13, 10.0, 2.21), (0.0, -1.0, 0.0)),
        ((1.0, 6.0, 4.5), (0.3, -1.0, -0.2)),
        ((0.5, 3.0, 0.1), (1.0, -1.2, 0.6)),
        ((6.0, 4.0, 0.0), (-0.7, -0.5, 0.3)),
    ];
    let check = |label: &str| {
        let mut hit = 0;
        for ((ox, oy, oz), (dx, dy, dz)) in rays {
            let o = json!({ "x": ox, "y": oy, "z": oz });
            let d = json!({ "x": dx, "y": dy, "z": dz });
            let physics = answer(og_physics_raycast(
                w,
                Some(json!({ "origin": o, "direction": d, "maxDistance": 100 }).to_string()),
            ));
            let hits = answer(og_render_raycast(
                w,
                json!({ "origin": o, "direction": d }).to_string(),
                Some(json!({ "first": true }).to_string()),
            ));
            if physics.is_null() {
                assert_eq!(hits, json!([]), "{label}: physics misses {o} {d}");
                continue;
            }
            hit += 1;
            let render = &hits[0];
            assert_eq!(render["entity"].as_f64(), Some(e), "{label}");
            assert_eq!(render["entity"], physics["entity"], "{label}");
            let (rd, pd) = (
                render["distance"].as_f64().unwrap(),
                physics["distance"].as_f64().unwrap(),
            );
            assert!((rd - pd).abs() < 1e-3, "{label}: render {rd}, physics {pd}");
            close(render, &physics, "point", 1e-3);
            close(render, &physics, "normal", 1e-3);
        }
        assert!(hit >= 3, "{label}: {hit} rays hit");
    };
    check("as made");

    // A ray across the bump (in the mesh's own space at height 0.7, along
    // +x through its ridge) crosses it twice: both hits, nearest first.
    let turn = og_core::glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
    let at =
        turn * og_core::glam::Vec3::new(-6.0, 0.7, -0.5) + og_core::glam::Vec3::new(3.0, -1.0, 2.0);
    let along = turn * og_core::glam::Vec3::X;
    let all = answer(og_render_raycast(
        w,
        json!({
            "origin": { "x": at.x, "y": at.y, "z": at.z },
            "direction": { "x": along.x, "y": along.y, "z": along.z }
        })
        .to_string(),
        None,
    ));
    let d: Vec<f64> = all
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["distance"].as_f64().unwrap())
        .collect();
    assert_eq!(d.len(), 2, "{all}");
    assert!(d[0] < d[1], "{d:?}");

    // Raised by a metre: the render answer moves at once, physics after a step.
    answer(og_render_update_geometry(
        w,
        geometry.as_f64().unwrap(),
        Some(terrain(1.0).to_string()),
    ));
    answer(og_engine_frame(w, 1000.0 / 60.0));
    check("updated");
}

#[test]
fn a_cooked_model_raycasts_without_building_a_bvh() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-runtime-cooked-bvh-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/models")).unwrap();
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../templates/game/assets/models/dropship.glb"),
        root.join("assets/models/dropship.glb"),
    )
    .unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let w = answer(og_engine_create(game, options)).as_u64().unwrap() as u32;
    answer(og_engine_frame(w, 1000.0 / 60.0));

    let builds = Bvh::builds();
    let asset = answer(og_load_gltf(w, "models/dropship.glb".into(), None));
    let e = answer(og_world_instantiate(w, asset.as_f64().unwrap(), None))
        .as_f64()
        .unwrap();
    answer(og_engine_frame(w, 1000.0 / 60.0));
    // Down through the model from above, and across it: both hit it.
    let mut hits = 0;
    for (o, d) in [
        (
            json!({ "x": 0, "y": 50, "z": 0 }),
            json!({ "x": 0, "y": -1, "z": 0 }),
        ),
        (
            json!({ "x": 50, "y": 0.5, "z": 0.2 }),
            json!({ "x": -1, "y": 0, "z": 0 }),
        ),
        (
            json!({ "x": 0.3, "y": 0.4, "z": 50 }),
            json!({ "x": 0, "y": 0, "z": -1 }),
        ),
    ] {
        let all = answer(og_render_raycast(
            w,
            json!({ "origin": o, "direction": d }).to_string(),
            None,
        ));
        for h in all.as_array().unwrap() {
            hits += 1;
            assert!(h["distance"].as_f64().unwrap() > 0.0, "{h}");
        }
    }
    assert!(hits >= 2, "the rays hit the dropship {hits} times");
    let _ = e;
    assert_eq!(
        Bvh::builds(),
        builds,
        "loading and raycasting the cooked model built a BVH"
    );
}
