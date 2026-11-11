//! Extraction draws a world far from its zero from a render origin near the
//! camera (`extract::render_origin`, `og_render::RenderScene::rebase`): the
//! scene handed to the renderer holds positions within 4096 m of it, moved
//! exactly, and the camera looks along its rotation, not at a target
//! rounded 100 km out. A camera within 4096 m of the world's zero draws at
//! it, exactly as before render origins.
//!
//! Ways this could fail: extraction hands the renderer world positions (the
//! camera 100 km out, its look rounded to 7.8 mm there: the frame wobbles
//! as it turns), the origin is off the grid (the moves are not exact), an
//! instance or a light is left in the world's coordinates, or a camera near
//! the zero gets an origin anyway (every existing frame would move).

use og_core::glam::{Mat4, Quat, Vec3};
use og_render::RenderScene;
use og_runtime::{
    RenderBackend, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_render_box_geometry, og_render_standard_material, og_world_insert, og_world_spawn,
    set_render_backend, write_resource,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

#[derive(Default)]
struct Seen {
    origin: Vec3,
    position: Vec3,
    target: Vec3,
    instances: Vec<Mat4>,
    lights: Vec<Vec3>,
}

struct Recorder(Rc<RefCell<Seen>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 36)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = Seen {
            origin: scene.origin,
            position: scene.camera.position,
            target: scene.camera.target,
            instances: scene
                .objects
                .instances
                .iter()
                .map(|i| i.transform)
                .collect(),
            lights: scene.lighting.lights.iter().map(|l| l.position).collect(),
        };
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// A yaw of 0.002 rad about +Y and a pitch of -0.001 rad, written out:
/// the camera's look is a hair off -z, so a target rounded 100 km out
/// turns it visibly.
fn turn() -> Quat {
    Quat::from_xyzw(-0.0005, 0.001, 0.000_000_5, 0.999_999_4).normalize()
}

/// One frame of a world with a box at `boxed`, a point light at `lit` and
/// og.Camera at `eye` turned by `turn()`: what extraction handed over.
fn frame_at(eye: Vec3, boxed: Vec3, lit: Vec3) -> Seen {
    frame_with(eye, boxed, lit, None)
}

/// [`frame_at`] in a world with og.WorldOrigin set to `origin` (R12).
fn frame_with(eye: Vec3, boxed: Vec3, lit: Vec3, origin: Option<Value>) -> Seen {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(e, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let geometry = value(og_render_box_geometry(e, Some("{}".into())));
    let material = value(og_render_standard_material(e, Some("{}".into())));
    let place = |p: Vec3| json!({ "position": { "x": p.x, "y": p.y, "z": p.z } }).to_string();
    let b = value(og_world_spawn(e, Some(place(boxed))))
        .as_f64()
        .unwrap();
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    value(og_world_insert(e, b, "og.Mesh".into(), Some(mesh)));
    let l = value(og_world_spawn(e, Some(place(lit)))).as_f64().unwrap();
    value(og_world_insert(
        e,
        l,
        "og.PointLight".into(),
        Some(json!({ "intensity": 5.0, "distance": 30.0 }).to_string()),
    ));
    let q = turn();
    let camera = json!({
        "position": { "x": eye.x, "y": eye.y, "z": eye.z },
        "quaternion": { "x": q.x, "y": q.y, "z": q.z, "w": q.w },
    });
    let w: Value =
        serde_json::from_str(&write_resource(e, "og.Camera", &camera.to_string())).unwrap();
    assert_eq!(w["ok"], json!(true), "{w}");
    if let Some(o) = origin {
        let w: Value =
            serde_json::from_str(&write_resource(e, "og.WorldOrigin", &o.to_string())).unwrap();
        assert_eq!(w["ok"], json!(true), "{w}");
    }
    let answer: Value = serde_json::from_str(&og_engine_frame(e, 1000.0 / 60.0)).unwrap();
    assert_eq!(answer["rendered"], json!(true), "{answer}");
    seen.take()
}

#[test]
fn a_camera_100_km_out_draws_from_a_render_origin_near_it() {
    // Positions on the f32 grid 100 km out (1/128 m there).
    let eye = Vec3::new(100_000.5, 2.0, -60_000.25);
    let boxed = Vec3::new(100_003.0, 0.5, -60_010.0);
    let lit = Vec3::new(99_998.0, 3.0, -60_004.5);
    let seen = frame_at(eye, boxed, lit);
    // round(100000.5 / 8192) = 12, round(-60000.25 / 8192) = -7.
    let origin = Vec3::new(98_304.0, 0.0, -57_344.0);
    assert_eq!(
        seen.origin, origin,
        "the render origin: the 8192 m grid point nearest the camera"
    );
    assert_eq!(
        seen.position,
        eye - origin,
        "the camera from the origin, moved exactly"
    );
    let at = seen
        .instances
        .first()
        .expect("the box is drawn")
        .w_axis
        .truncate();
    assert_eq!(at, boxed - origin, "the box from the origin, moved exactly");
    assert_eq!(seen.lights, vec![lit - origin], "the light from the origin");
    // The look: the rotation's -z, as precise as a target 1 m ahead is
    // within 4096 m of the origin (2.4e-4 m there), not rounded to the
    // 7.8 mm of a target 100 km out (here x -0.002 rounds to 0).
    let look = (seen.target - seen.position).normalize();
    let want = turn() * Vec3::NEG_Z;
    assert!(
        (look - want).length() < 5e-4,
        "the camera looks along its rotation {want}, drawn looking along {look}"
    );
}

#[test]
fn a_shifted_world_draws_from_the_render_origin_it_had_in_the_absolute_world() {
    // R12: og.WorldOrigin shifts the world by -49 x 2048 m on x and -29 x
    // 2048 m on z as the first tick starts (the camera is its focus), so
    // the camera is at (-351.5, 2, 1391.75) in the world's coordinates. The
    // render origin is laid on the grid of the absolute world (98304, 0,
    // -57344, as unshifted), so the frame handed to the renderer is the
    // unshifted world's, bit for bit, and a shift moves nothing drawn.
    let eye = Vec3::new(100_000.5, 2.0, -60_000.25);
    let boxed = Vec3::new(100_003.0, 0.5, -60_010.0);
    let lit = Vec3::new(99_998.0, 3.0, -60_004.5);
    let plain = frame_at(eye, boxed, lit);
    let shifted = frame_with(eye, boxed, lit, Some(json!({ "step": 2048 })));
    assert_eq!(shifted.origin, plain.origin, "the absolute grid point");
    assert_eq!(shifted.position, plain.position);
    assert_eq!(shifted.target, plain.target);
    assert_eq!(shifted.instances, plain.instances);
    assert_eq!(shifted.lights, plain.lights);
}
