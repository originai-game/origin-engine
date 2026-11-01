//! Render interpolation between ticks (RenderSettings.interpolate,
//! docs/design.md §2.4): a 60 Hz tick drawn at 144 Hz draws each moving
//! entity and og.Camera between the last two ticks at the accumulator's
//! fraction; og.render.snap, a jump past RenderSettings.snapDistance and
//! og.Interpolation.off draw the tick; the probe's capture and frames on
//! tick boundaries draw the tick; and the world hashes the same whether
//! its frames blended or not.
//!
//! A game of one system (Move) sets, on tick T, each entity's x to T (its
//! z names it): A also turns 0.1 T radians about y and scales to
//! 1 + 0.1 T; B jumps 3 m at tick K and asks og.render.snap; C jumps 50 m
//! at K (past the 10 m snapDistance); D carries og.Interpolation.off, and
//! F is D's child. og.Camera's x is T too. After a frame at world tick W,
//! the last tick set T = W - 1 and the one before W - 2, so the drawn x of
//! a blended entity is W - 2 + alpha, alpha = the frame's `behind` over
//! the tick's length.
//!
//! Ways this could fail: extraction draws the last tick (the code before
//! this: every assertion on A is off by 1 - alpha), the blend runs the
//! wrong way (W - 1 - alpha), the rotation is lerped as a matrix (its scale
//! drifts), the snaps smear a teleport, the capture draws the blend (its
//! reference frames would move), or the blend writes into the world.

use og_core::glam::{Mat4, Vec3};
use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_capture, og_engine_probe_dump, og_game_game, og_host_begin_bundle,
    og_render_box_geometry, og_render_snap, og_render_standard_material, og_system_define_system,
    og_world_get_tick, og_world_insert, og_world_set, og_world_spawn, set_hooks,
    set_render_backend, write_resource,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const MOVE: u32 = 1;
/// The tick B and C jump at.
const K: u32 = 20;
const TICK_MS: f64 = 1000.0 / 60.0;

/// Per engine: the entities A, B, C, D (and F, D's child, which Move leaves).
type Entities = Rc<RefCell<HashMap<u32, [f64; 4]>>>;

struct Mover(Entities);

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

impl Hooks for Mover {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind != 0 || call.id != MOVE {
            return Ok(String::new());
        }
        let w = call.engine;
        let Some([a, b, c, d]) = self.0.borrow().get(&w).copied() else {
            return Ok(String::new());
        };
        let t = f64::from(og_world_get_tick(w));
        let set = |e: f64, x: f64, z: f64, extra: Value| {
            let mut v = json!({ "position": { "x": x, "y": 0, "z": z } });
            if let (Value::Object(v), Value::Object(extra)) = (&mut v, extra) {
                v.extend(extra);
            }
            value(og_world_set(w, e, "og.Transform".into(), v.to_string()));
        };
        let half = 0.05 * t;
        let turn = json!({
            "quaternion": { "x": 0, "y": half.sin(), "z": 0, "w": half.cos() },
            "scale": { "x": 1.0 + 0.1 * t, "y": 1.0 + 0.1 * t, "z": 1.0 + 0.1 * t },
        });
        set(a, t, 0.0, turn);
        let jumped = t >= f64::from(K);
        set(b, t + if jumped { 3.0 } else { 0.0 }, 10.0, json!({}));
        set(c, t + if jumped { 50.0 } else { 0.0 }, 20.0, json!({}));
        set(d, t, 30.0, json!({}));
        if t == f64::from(K) {
            value(og_render_snap(w, b));
            let dead = og_render_snap(w, 987_654_321.0);
            assert!(
                dead.contains("\"ok\":false"),
                "a dead entity is refused: {dead}"
            );
        }
        let camera = json!({ "position": { "x": t, "y": 2, "z": 100 } }).to_string();
        value(write_resource(w, "og.Camera", &camera));
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// What the renderer was handed last: each instance's world matrix, the
/// camera's position, and the same of the last capture.
#[derive(Default)]
struct Seen {
    drawn: Vec<Mat4>,
    camera: Vec3,
    captured: Vec<Mat4>,
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
        let mut s = self.0.borrow_mut();
        s.drawn = scene.instances.iter().map(|i| i.transform).collect();
        s.camera = scene.camera.position;
        Ok(())
    }
    fn capture(&mut self, scene: &RenderScene, w: u32, h: u32) -> Option<og_render::RgbaImage> {
        self.0.borrow_mut().captured = scene.instances.iter().map(|i| i.transform).collect();
        Some(og_render::RgbaImage {
            width: w.max(1),
            height: h.max(1),
            pixels: vec![0; (w.max(1) * h.max(1) * 4) as usize],
        })
    }
}

/// The instance of `drawn` at depth z (each entity's z names it).
fn at(drawn: &[Mat4], z: f32) -> Mat4 {
    *drawn
        .iter()
        .find(|m| (m.w_axis.z - z).abs() < 0.5)
        .unwrap_or_else(|| panic!("nothing drawn at z {z}: {drawn:?}"))
}

fn engine(entities: &Entities) -> (u32, Rc<RefCell<Seen>>) {
    set_hooks(Rc::new(Mover(entities.clone())));
    og_host_begin_bundle();
    let transform = json!({ "kind": "component", "name": "og.Transform" });
    let camera = json!({ "kind": "resource", "name": "og.Camera" });
    let mover = og_system_define_system(
        "Move".into(),
        json!({ "reads": [], "writes": [transform, camera] }).to_string(),
        MOVE,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [mover] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_render_backend(e, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let geometry = value(og_render_box_geometry(e, Some("{}".into())));
    let material = value(og_render_standard_material(e, Some("{}".into())));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let spawn = |z: f64| {
        let at = json!({ "position": { "x": 0, "y": 0, "z": z } }).to_string();
        let id = value(og_world_spawn(e, Some(at))).as_f64().unwrap();
        value(og_world_insert(e, id, "og.Mesh".into(), Some(mesh.clone())));
        id
    };
    let ids = [spawn(0.0), spawn(10.0), spawn(20.0), spawn(30.0)];
    let off = json!({ "off": true }).to_string();
    value(og_world_insert(
        e,
        ids[3],
        "og.Interpolation".into(),
        Some(off),
    ));
    let child = json!({ "position": { "x": 0, "y": 0, "z": 5 }, "parent": ids[3] }).to_string();
    let f = value(og_world_spawn(e, Some(child))).as_f64().unwrap();
    value(og_world_insert(e, f, "og.Mesh".into(), Some(mesh)));
    entities.borrow_mut().insert(e, ids);
    (e, seen)
}

fn frame(e: u32, ms: f64) -> (u32, f64) {
    let answer: Value = serde_json::from_str(&og_engine_frame(e, ms)).unwrap();
    assert_eq!(answer["ok"], json!(true), "{answer}");
    assert_eq!(answer["rendered"], json!(true), "{answer}");
    (
        answer["tick"].as_u64().unwrap() as u32,
        answer["behind"].as_f64().unwrap(),
    )
}

fn hash(e: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(e, None)).unwrap();
    dump["hash"].clone()
}

fn close(got: f32, want: f64, what: &str) {
    assert!(
        (f64::from(got) - want).abs() < 1e-4,
        "{what}: drawn {got}, between the ticks at {want}"
    );
}

#[test]
fn a_60_hz_tick_drawn_at_144_hz_is_blended_by_the_accumulator() {
    let entities = Entities::default();
    let (e, seen) = engine(&entities);
    let mut blended = 0;
    let mut checked_snap = [false; 2];
    for _ in 0..90 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        if w < 2 {
            continue;
        }
        let alpha = behind / TICK_MS;
        assert!((0.0..1.0).contains(&alpha), "alpha {alpha}");
        let between = f64::from(w - 2) + alpha;
        let s = seen.borrow();
        let a = at(&s.drawn, 0.0);
        let (scale, rot, pos) = a.to_scale_rotation_translation();
        close(pos.x, between, "A's x");
        close(scale.x, 1.0 + 0.1 * between, "A's scale");
        close(
            (2.0 * f64::from(rot.y).atan2(f64::from(rot.w))) as f32,
            0.1 * between,
            "A's turn about y",
        );
        close(s.camera.x, between, "the camera's x");
        if alpha > 0.05 && alpha < 0.95 {
            blended += 1;
        }
        // B and C on the frames whose last tick is K: B asked for a snap,
        // C jumped past snapDistance; both draw tick K. A tick later they
        // blend again, from the jumped tick.
        let last = w - 1;
        let (b, c) = (at(&s.drawn, 10.0).w_axis.x, at(&s.drawn, 20.0).w_axis.x);
        if last == K {
            close(b, f64::from(K) + 3.0, "B on the tick it was snapped");
            close(c, f64::from(K) + 50.0, "C on the tick it jumped 50 m");
            checked_snap[0] = true;
        } else if last == K + 1 {
            close(b, between + 3.0, "B a tick after its snap");
            close(c, between + 50.0, "C a tick after its jump");
            checked_snap[1] = true;
        } else if last < K {
            close(b, between, "B before its jump");
        }
        // og.Interpolation.off: D and its child F draw the last tick.
        close(at(&s.drawn, 30.0).w_axis.x, f64::from(last), "D (off)");
        close(at(&s.drawn, 35.0).w_axis.x, f64::from(last), "F, D's child");
    }
    assert!(blended > 20, "only {blended} frames fell between ticks");
    assert_eq!(
        checked_snap,
        [true, true],
        "the frames around tick K were drawn"
    );

    // The probe's capture draws the tick, whatever the last frame blended.
    let (w, behind) = frame(e, 1000.0 / 144.0);
    assert!(behind > 0.0);
    let shot: Value = serde_json::from_str(&og_engine_probe_capture(e, Some(16), Some(9))).unwrap();
    assert_eq!(shot["tick"], json!(w), "{shot}");
    let last = f64::from(w - 1);
    close(
        at(&seen.borrow().captured, 0.0).w_axis.x,
        last,
        "A in a capture",
    );

    // Frames on tick boundaries draw the tick, and the world those blended
    // frames drew is the world whole-tick frames drew.
    let (e2, seen2) = engine(&entities);
    let mut w2 = 0;
    while w2 < w {
        let (t, behind) = frame(e2, TICK_MS);
        assert_eq!(behind, 0.0);
        w2 = t;
        if t >= 1 {
            let x = at(&seen2.borrow().drawn, 0.0).w_axis.x;
            assert_eq!(x, (t - 1) as f32, "a whole-tick frame draws the tick");
        }
    }
    assert_eq!(w2, w);
    assert_eq!(hash(e2), hash(e), "the blend never reaches the world");
}

#[test]
fn render_settings_interpolate_false_draws_the_tick() {
    let entities = Entities::default();
    let (e, seen) = engine(&entities);
    value(write_resource(
        e,
        "og.RenderSettings",
        r#"{"interpolate":false}"#,
    ));
    for _ in 0..12 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        if w >= 1 {
            assert!(behind < TICK_MS);
            let x = at(&seen.borrow().drawn, 0.0).w_axis.x;
            assert_eq!(x, (w - 1) as f32);
        }
    }
}
