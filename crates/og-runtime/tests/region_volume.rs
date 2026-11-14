//! og.Volume is the shape of a region on its entity: a game names it
//! through the host like any component, and the region beside it is
//! bounded by it from the next tick.
//!
//! Ways this could fail: og.Volume is registered when the game first names
//! it but og.audio is never granted it (the zone stays unbounded, with no
//! error), the zone reads a volume of its own instead of the entity's, or
//! a zone without one is heard nowhere instead of everywhere.
//!
//! og.ForceField accelerates the bodies inside the volume beside it. Ways
//! this could fail: the fields never reach physics (nothing moves), the
//! field ignores its volume (the far body moves) or its entity's turn (the
//! body rises instead of going where the turned field points), or a field
//! switched off keeps pushing.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_world_get, og_world_insert, og_world_spawn, set_hooks,
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

fn answer(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v
}

fn insert(engine: u32, id: f64, name: &str, v: Value) {
    answer(og_runtime::host_impl::answer(og_world_insert(
        engine,
        id,
        name.into(),
        Some(v.to_string()),
    )));
}

fn spawn(engine: u32, z: f32, parts: &[(&str, Value)]) -> f64 {
    let desc = json!({ "position": { "x": 0, "y": 1, "z": z } });
    let id = answer(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(desc.to_string()),
    )))["value"]
        .as_f64()
        .unwrap();
    for (name, v) in parts {
        insert(engine, id, name, v.clone());
    }
    id
}

#[test]
fn a_reverb_zone_is_bounded_by_the_volume_on_its_entity() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let ear = spawn(engine, 0.0, &[("og.AudioListener", json!({}))]);
    let crypt = spawn(engine, -20.0, &[("og.ReverbZone", json!({ "room": 0.9 }))]);
    let weight = || {
        answer(og_engine_frame(engine, 1000.0 / 60.0));
        answer(og_runtime::host_impl::answer(og_world_get(
            engine,
            crypt,
            "og.ReverbZone".into(),
        )))["value"]["weight"]
            .as_f64()
            .unwrap()
    };
    // No volume: the zone has no bounds, heard from 20 m away.
    assert_eq!(weight(), 1.0);
    // A 10 x 4 x 10 m box with a 4 m fade: its near face is 15 m away.
    insert(
        engine,
        crypt,
        "og.Volume",
        json!({ "size": { "x": 10, "y": 4, "z": 10 }, "fade": 4 }),
    );
    assert_eq!(weight(), 0.0);
    // Two metres from the face, half of the fade.
    insert(
        engine,
        ear,
        "og.Transform",
        json!({ "position": { "x": 0, "y": 1, "z": -13 } }),
    );
    assert!((weight() - 0.5).abs() < 1e-5);
    // A sphere of 3 m instead: the listener, 7 m from its centre, is out.
    insert(engine, crypt, "og.Volume", json!({ "radius": 3 }));
    assert_eq!(weight(), 0.0);
}

fn position(engine: u32, id: f64) -> glam::Vec3 {
    let t = answer(og_runtime::host_impl::answer(og_world_get(
        engine,
        id,
        "og.Transform".into(),
    )));
    let p = &t["value"]["position"];
    let f = |k: &str| p[k].as_f64().unwrap() as f32;
    glam::Vec3::new(f("x"), f("y"), f("z"))
}

#[test]
fn a_force_field_accelerates_the_bodies_in_the_volume_beside_it() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some("{}".into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    // Two weightless balls: one in the field's box, one 40 m down the z axis.
    let ball = |z: f32| {
        spawn(
            engine,
            z,
            &[
                ("og.RigidBody", json!({ "gravityScale": 0 })),
                ("og.Collider", json!({ "shape": 1, "radius": 0.25 })),
            ],
        )
    };
    let (near, far) = (ball(0.0), ball(-40.0));
    // An updraft of 6 m/s² in a 10 m cube, its entity turned a quarter
    // about z: its up is the world's -x.
    let half = std::f32::consts::FRAC_1_SQRT_2;
    let field = answer(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some(
            json!({
                "position": { "x": 0, "y": 1, "z": 0 },
                "quaternion": { "x": 0, "y": 0, "z": half, "w": half },
            })
            .to_string(),
        ),
    )))["value"]
        .as_f64()
        .unwrap();
    insert(engine, field, "og.ForceField", json!({ "strength": 6 }));
    insert(
        engine,
        field,
        "og.Volume",
        json!({ "size": { "x": 10, "y": 10, "z": 10 } }),
    );
    let frames = |n: u32| {
        for _ in 0..n {
            answer(og_engine_frame(engine, 1000.0 / 60.0));
        }
    };
    frames(30);
    // Half a second at 6 m/s²: 0.75 m along -x.
    let p = position(engine, near);
    assert!((p.x + 0.75).abs() < 0.08 && (p.y - 1.0).abs() < 1e-3, "{p}");
    assert_eq!(position(engine, far), glam::Vec3::new(0.0, 1.0, -40.0));
    // Switched off, the ball coasts: no faster after another half second.
    insert(
        engine,
        field,
        "og.ForceField",
        json!({ "strength": 6, "enabled": false }),
    );
    frames(1);
    let a = position(engine, near).x;
    frames(30);
    let b = position(engine, near).x;
    assert!(((a - b) - 1.5).abs() < 0.08, "coasting at 3 m/s: {}", a - b);
}
