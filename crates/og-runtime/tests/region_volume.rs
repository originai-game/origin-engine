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
//!
//! og.FogVolume reaches the renderer with the volume beside it. Ways this
//! could fail: the fog is drawn without its entity's shape or pose (it
//! fills the world, or sits at the origin), or one without an og.Volume is
//! dropped instead of drawn everywhere.
//!
//! og.SightMap reaches the renderer as the cells its team sees. Ways this
//! could fail: the map is never cast (no cell seen), the frame is given
//! another team's cells or a stale tick's, or the map is laid somewhere
//! other than its entity's og.Volume.

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

/// What the renderer was last given of the world's fog volumes.
struct FogSeen(Rc<std::cell::RefCell<Vec<og_render::FogVolume>>>);

impl og_runtime::RenderBackend for FogSeen {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.atmosphere.fog_volumes.clone();
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

#[test]
fn a_fog_volume_reaches_the_renderer_with_the_volume_on_its_entity() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(std::cell::RefCell::new(Vec::new()));
    og_runtime::set_render_backend(engine, Some(Box::new(FogSeen(seen.clone())))).unwrap();
    // A smoke-filled room 12 m down the z axis, and a haze with no bounds.
    spawn(
        engine,
        12.0,
        &[
            (
                "og.Volume",
                json!({ "size": { "x": 6, "y": 3, "z": 4 }, "fade": 1.5 }),
            ),
            (
                "og.FogVolume",
                json!({ "density": 0.3, "scattering": 0.2, "color": { "r": 0.5, "g": 0.25, "b": 1 } }),
            ),
        ],
    );
    spawn(engine, 0.0, &[("og.FogVolume", json!({ "density": 0.02 }))]);
    answer(og_engine_frame(engine, 1000.0 / 60.0));
    let seen = seen.borrow();
    assert_eq!(seen.len(), 2, "{seen:?}");
    let room = &seen[0];
    assert_eq!(room.at.position, glam::Vec3::new(0.0, 1.0, 12.0));
    assert_eq!(
        room.volume,
        Some(og_env::volume::Volume {
            size: glam::Vec3::new(6.0, 3.0, 4.0),
            radius: 0.0,
            fade: 1.5,
        })
    );
    assert_eq!(
        (room.density, room.scattering, room.color),
        (0.3, 0.2, glam::Vec3::new(0.5, 0.25, 1.0))
    );
    assert_eq!((seen[1].volume, seen[1].density), (None, 0.02));
}

/// What the renderer was last given of the world's fog of war.
struct SightSeen(Rc<std::cell::RefCell<Option<og_render::FogOfWar>>>);

impl og_runtime::RenderBackend for SightSeen {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.look.fog_of_war.clone();
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

#[test]
fn a_sight_map_reaches_the_renderer_with_the_cells_its_team_sees() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = answer(og_engine_create(game, "{}".into()))["value"]
        .as_u64()
        .unwrap() as u32;
    let seen = Rc::new(std::cell::RefCell::new(None));
    og_runtime::set_render_backend(engine, Some(Box::new(SightSeen(seen.clone())))).unwrap();
    // A 16 by 8 m map of 2 m cells about (0, 1, 12), drawn for team 3.
    let map = spawn(
        engine,
        12.0,
        &[
            ("og.Volume", json!({ "size": { "x": 16, "y": 2, "z": 8 } })),
            (
                "og.SightMap",
                json!({ "cell": 2, "team": 3, "explored": 0.5, "color": { "r": 0, "g": 0, "b": 0.25 } }),
            ),
        ],
    );
    // Team 3's eye in the map's first cell, another team's in its last.
    let eye = |x: f32, z: f32, team: u32| {
        let desc = json!({ "position": { "x": x, "y": 1, "z": z } });
        let id = answer(og_runtime::host_impl::answer(og_world_spawn(
            engine,
            Some(desc.to_string()),
        )))["value"]
            .as_f64()
            .unwrap();
        insert(engine, id, "og.Sight", json!({ "team": team, "radius": 3 }));
        id
    };
    let ours = eye(-7.0, 9.0, 3);
    eye(7.0, 15.0, 0);
    answer(og_engine_frame(engine, 1000.0 / 60.0));
    let fog = seen.borrow().clone().expect("the map is drawn");
    assert_eq!((fog.columns, fog.rows, fog.cell), (8, 4, 2.0));
    assert_eq!(fog.min, glam::Vec2::new(-8.0, 8.0));
    assert_eq!(fog.plane, og_render::SightPlane::Ground);
    assert_eq!(
        (fog.color, fog.explored),
        (glam::Vec3::new(0.0, 0.0, 0.25), 0.5)
    );
    // Team 3 sees the cells within 1.5 cells of its own: the corner's four.
    let visible = og_env::sight::VISIBLE;
    let lit: Vec<usize> = (0..32).filter(|&i| fog.levels[i] == visible).collect();
    assert_eq!(lit, [0, 1, 8, 9], "{:?}", fog.levels);
    assert!(fog.levels.iter().all(|&l| l != og_env::sight::EXPLORED));

    // The eye walks to the far end: where it was is explored, at the next frame.
    insert(
        engine,
        ours,
        "og.Transform",
        json!({ "position": { "x": 7, "y": 1, "z": 9 } }),
    );
    answer(og_engine_frame(engine, 1000.0 / 60.0));
    let fog = seen.borrow().clone().expect("the map is drawn");
    assert_eq!(fog.levels[0], og_env::sight::EXPLORED, "{:?}", fog.levels);
    assert_eq!(fog.levels[7], visible, "{:?}", fog.levels);

    // Not drawn: the frame is given none, and the map still answers.
    insert(
        engine,
        map,
        "og.SightMap",
        json!({ "cell": 2, "team": 3, "drawn": false }),
    );
    answer(og_engine_frame(engine, 1000.0 / 60.0));
    assert!(seen.borrow().is_none());
    let at = json!({ "x": 7, "y": 1, "z": 9 }).to_string();
    assert_eq!(og_runtime::og_sight_visible(engine, 3, at), Ok(true));
}
