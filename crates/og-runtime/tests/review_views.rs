//! The model review's cameras: `view:<entity>:front|back|side|top|orbit`
//! frames an entity's drawn bounds from a fixed direction in place of
//! og.Camera, and `turntable:<entity>:<n>` takes n orbit frames on one tick.
//!
//! The frame each capture is rendered from is read off the renderer (a
//! recorder backend keeps the camera of every capture), and a 2 × 1 × 0.5
//! box's corners are projected through it:
//! - every orthographic view collapses the axis it looks down (each of the
//!   box's corners lands on the corner across the box from it), so the
//!   front frame shows width by height and the top frame width by depth:
//!   the top's height in the frame is the front's times depth over height,
//!   its width the same;
//! - the binding extent fills the frame less the margin on each side;
//! - front keeps +X right and +Y up, back mirrors x, side shows +Z on the
//!   left, top puts the front (+Z) at the bottom;
//! - a child through og.Parent is framed with its parent;
//! - og.Camera is never written, and `view:off` gives the frame back to it.
//!
//! Before the model-review round there was no `view` or `turntable`: the
//! script's steps were unknown actions and every capture was og.Camera's.

use glam::{Mat4, Vec3};
use og_render::{Camera, Projection, RenderScene, RgbaImage};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_take_log, og_engine_take_outputs, og_game_game,
    og_host_begin_bundle, og_render_box_geometry, og_render_standard_material, og_world_insert,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::rc::Rc;

const W: u32 = 160;
const H: u32 = 90;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// A renderer that keeps the camera of every capture.
struct Recorder(Rc<RefCell<Vec<Camera>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (W, H)
    }
    fn render(&mut self, _scene: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, scene: &RenderScene, _: u32, _: u32) -> Option<RgbaImage> {
        self.0.borrow_mut().push(scene.camera);
        Some(RgbaImage {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 255],
        })
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn view_projection(c: &Camera) -> Mat4 {
    let aspect = W as f32 / H as f32;
    let view = glam::camera::rh::view::look_at_mat4(c.position, c.target, c.up);
    let proj = match c.projection {
        Projection::Orthographic { height } => {
            let h = height * 0.5;
            glam::camera::rh::proj::directx::orthographic(
                -h * aspect,
                h * aspect,
                -h,
                h,
                c.near,
                c.far,
            )
        }
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            aspect,
            c.near,
            c.far,
        ),
    };
    proj * view
}

/// The box's centre and half extents (it is spawned unrotated there).
const CENTRE: Vec3 = Vec3::new(3.0, 1.0, -2.0);
const HALF: Vec3 = Vec3::new(1.0, 0.5, 0.25);

/// The box's corner at signs (sx, sy, sz) in the frame.
fn corner(c: &Camera, s: Vec3) -> Vec3 {
    view_projection(c).project_point3(CENTRE + HALF * s)
}

fn corners(c: &Camera) -> Vec<Vec3> {
    (0..8)
        .map(|k| {
            let s = Vec3::new(
                if k & 1 == 0 { -1.0 } else { 1.0 },
                if k & 2 == 0 { -1.0 } else { 1.0 },
                if k & 4 == 0 { -1.0 } else { 1.0 },
            );
            corner(c, s)
        })
        .collect()
}

/// The frame's box around the corners: (half width, half height) in NDC.
fn extent(c: &Camera) -> (f32, f32) {
    let ps = corners(c);
    let fold = |f: fn(&Vec3) -> f32| ps.iter().map(f).fold(0.0f32, |m, v| m.max(v.abs()));
    (fold(|p| p.x), fold(|p| p.y))
}

fn close(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() < 1e-4, "{what}: {a}, expected {b}");
}

/// Runs a script for one frame and answers the cameras its captures were
/// drawn from, with the outputs' file names.
fn run(engine: u32, seen: &RefCell<Vec<Camera>>, script: &str) -> (Vec<Camera>, Vec<String>) {
    let report: Value =
        serde_json::from_str(&og_engine_probe_commands(engine, script.into())).unwrap();
    assert!(
        report["warnings"].as_array().is_none_or(|w| w.is_empty()),
        "{report}"
    );
    seen.borrow_mut().clear();
    for _ in 0..2 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
    let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
    let warnings: Vec<_> = log
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .collect();
    assert!(warnings.is_empty(), "{warnings:?}");
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(engine)).unwrap();
    let files = outputs
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["file"].as_str().unwrap().to_owned())
        .collect();
    (seen.borrow().clone(), files)
}

#[test]
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn views_frame_the_entity_orthographically_from_their_side() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let w = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(w, Some(Box::new(Recorder(seen.clone())))).unwrap();

    let geometry = value(og_render_box_geometry(
        w,
        Some(json!({ "width": 2, "height": 1, "depth": 0.5 }).to_string()),
    ));
    let material = value(og_render_standard_material(w, Some("{}".into())));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let at = json!({ "name": "crate", "position": { "x": 3, "y": 1, "z": -2 } });
    let e = value(og_world_spawn(w, Some(at.to_string())))
        .as_f64()
        .unwrap();
    value(og_world_insert(w, e, "og.Mesh".into(), Some(mesh.clone())));
    value(og_engine_frame(w, 1000.0 / 60.0));

    // og.Camera's own frame first, to tell it from the views.
    let (plain, _) = run(w, &seen, "0:screenshot:plain.png");
    assert!(matches!(
        plain[0].projection,
        Projection::Perspective { .. }
    ));

    let (cams, files) = run(
        w,
        &seen,
        "0:view:crate:front;0:screenshot:front.png;0:view:crate:top;0:screenshot:top.png;0:view:crate:side;0:screenshot:side.png;0:view:crate:back;0:screenshot:back.png",
    );
    assert_eq!(files, ["front.png", "top.png", "side.png", "back.png"]);
    let [front, top, side, back] = [cams[0], cams[1], cams[2], cams[3]];
    let fill = 1.0 / 1.2; // 10% free on each side of the binding extent
    let aspect = W as f32 / H as f32;

    for (name, c) in [
        ("front", front),
        ("top", top),
        ("side", side),
        ("back", back),
    ] {
        assert!(
            matches!(c.projection, Projection::Orthographic { .. }),
            "{name} is orthographic: {:?}",
            c.projection
        );
        // Orthographic: the corners an axis apart along the view collapse
        // onto each other, so the box shows four corners, not eight.
        let ps = corners(&c);
        let distinct = ps.iter().enumerate().filter(|(i, p)| {
            !ps[..*i]
                .iter()
                .any(|q| (q.x - p.x).abs() < 1e-4 && (q.y - p.y).abs() < 1e-4)
        });
        assert_eq!(distinct.count(), 4, "{name}: {ps:?}");
        for p in &ps {
            assert!(
                p.x.abs() <= fill + 1e-4 && p.y.abs() <= fill + 1e-4,
                "{name}: {p}"
            );
            assert!(
                (0.0..=1.0).contains(&p.z),
                "{name}: inside near and far: {p}"
            );
        }
    }

    // Front: width 2 across, height 1 up; at 16:9 the width binds (2
    // across needs 1.125 up, more than the height's 1), so x fills.
    let (fx, fy) = extent(&front);
    close(fx, fill, "front's width fills the frame less the margin");
    close(fy, fill * (HALF.y / HALF.x) * aspect, "front's height");
    // Top: width across, depth up: the same width, its height depth over
    // height of the front's, as an orthographic projection must show it.
    let (tx, ty) = extent(&top);
    close(tx, fx, "top's width");
    close(ty, fy * HALF.z / HALF.y, "top's height is the depth");
    // Side: depth across, height up; height binds.
    let (sx, sy) = extent(&side);
    close(sy, fill, "side's height fills the frame less the margin");
    close(
        sx,
        fill * (HALF.z / HALF.y) / aspect,
        "side's width is the depth",
    );
    // Back: the front's extents, mirrored.
    let (bx, by) = extent(&back);
    close(bx, fx, "back's width");
    close(by, fy, "back's height");

    // Which way round: the corner at +X +Y +Z.
    let c = Vec3::ONE;
    let f = corner(&front, c);
    assert!(f.x > 0.0 && f.y > 0.0, "front: +X right, +Y up: {f}");
    let b = corner(&back, c);
    assert!(b.x < 0.0 && b.y > 0.0, "back: +X left: {b}");
    let s = corner(&side, c);
    assert!(s.x < 0.0 && s.y > 0.0, "side from +X: +Z on the left: {s}");
    let t = corner(&top, c);
    assert!(
        t.x > 0.0 && t.y < 0.0,
        "top: +X right, the front (+Z) at the bottom: {t}"
    );
    // The front looks down -Z from in front of the box, the top down -Y.
    assert!(front.position.z > CENTRE.z + HALF.z && (front.position.x - CENTRE.x).abs() < 1e-4);
    assert!(top.position.y > CENTRE.y + HALF.y && (top.position.z - CENTRE.z).abs() < 1e-4);

    // A child through og.Parent is framed with its parent: a box on top
    // raises the front's frame (height now binds: 2 tall, 2 wide).
    let child = json!({ "name": "lid", "position": { "x": 0, "y": 1, "z": 0 }, "parent": e });
    let k = value(og_world_spawn(w, Some(child.to_string())))
        .as_f64()
        .unwrap();
    value(og_world_insert(w, k, "og.Mesh".into(), Some(mesh.clone())));
    let (cams, _) = run(w, &seen, "0:screenshot:framed.png");
    let framed = cams[0];
    let Projection::Orthographic { height } = framed.projection else {
        panic!("still the front view: {:?}", framed.projection);
    };
    close(
        height,
        2.0 * 1.2,
        "the lid's top and the crate's bottom bind",
    );
    close(framed.target.y, 1.5, "centred between y 0.5 and 2.5");

    // og.Camera was never written, and view:off gives the frame back.
    let (cams, _) = run(w, &seen, "0:view:off;0:screenshot:after.png");
    assert_eq!(cams[0], plain[0], "og.Camera's frame again");

    // A turntable: n orbit frames on one tick, then the view before.
    let (cams, files) = run(w, &seen, "0:view:crate:top;0:turntable:crate:4:spin");
    assert_eq!(
        files,
        ["spin-00.png", "spin-01.png", "spin-02.png", "spin-03.png"]
    );
    assert_eq!(cams.len(), 4);
    let centre = cams[0].target;
    for (i, c) in cams.iter().enumerate() {
        assert!(
            matches!(c.projection, Projection::Perspective { .. }),
            "frame {i}"
        );
        let d = (c.position - centre).normalize();
        let az = d.x.atan2(d.z).to_degrees().rem_euclid(360.0);
        close(az, 90.0 * i as f32, "the azimuth from the front towards +X");
        assert!(d.y > 0.3, "above the horizon");
    }
    // The view before the turntable is back: the top, looking down -Y.
    let (cams, _) = run(w, &seen, "0:screenshot:back-to-top.png");
    assert!(matches!(
        cams[0].projection,
        Projection::Orthographic { .. }
    ));
    assert_eq!(cams[0].up, Vec3::NEG_Z);
    assert!(cams[0].position.y > 2.5);
}
