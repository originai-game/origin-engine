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
    og_engine_probe_capture, og_engine_probe_commands, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_render_box_geometry, og_render_snap, og_render_standard_material,
    og_system_define_system, og_world_get_tick, og_world_insert, og_world_set, og_world_spawn,
    set_hooks, set_render_backend, write_resource,
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
    previous: Vec<Mat4>,
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
        s.previous = scene.previous_transforms.clone();
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

/// Object motion blur moves each instance from where the display frame
/// before drew it, not from the last tick: between ticks the blur covers
/// one display frame's motion (the old code used the tick before's
/// transform, a blur up to a whole tick long on every frame).
#[test]
fn motion_blur_moves_from_the_frame_drawn_before() {
    let entities = Entities::default();
    let (e, seen) = engine(&entities);
    value(write_resource(
        e,
        "og.PostEffects",
        r#"{"motionBlur":true}"#,
    ));
    let mut last: Option<Mat4> = None;
    let mut checked = 0;
    for _ in 0..40 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        let s = seen.borrow();
        if w < 3 {
            last = None;
            continue;
        }
        let i = s
            .drawn
            .iter()
            .position(|m| m.w_axis.z.abs() < 0.5)
            .expect("A is drawn");
        assert_eq!(s.previous.len(), s.drawn.len(), "one previous per instance");
        if let Some(l) = last
            && behind > 0.0
        {
            let (got, want) = (s.previous[i].w_axis.x, l.w_axis.x);
            assert!(
                (got - want).abs() < 1e-4,
                "tick {w} alpha {:.3}: blur starts at x {got}, the frame before drew {want}",
                behind / TICK_MS
            );
            checked += 1;
        }
        last = Some(s.drawn[i]);
    }
    assert!(checked > 20, "only {checked} frames were checked");
}

/// The probe's `teleport` is a teleport: the entity is drawn where it
/// landed on every frame after, never gliding there from where it stood
/// (a move well under RenderSettings.snapDistance, which the old code
/// blended).
#[test]
fn the_probes_teleport_snaps() {
    let entities = Entities::default();
    let (e, seen) = engine(&entities);
    let geometry = value(og_render_box_geometry(e, Some("{}".into())));
    let material = value(og_render_standard_material(e, Some("{}".into())));
    let mesh = json!({ "geometry": geometry, "material": material }).to_string();
    let place = json!({ "position": { "x": 0, "y": 0, "z": 40 } }).to_string();
    let g = value(og_world_spawn(e, Some(place))).as_f64().unwrap();
    value(og_world_insert(e, g, "og.Mesh".into(), Some(mesh)));
    // The probe names an entity #<index>.<generation> (its bits, high
    // half the generation).
    let g = format!("#{}.{}", g as u64 & 0xffff_ffff, g as u64 >> 32);
    let mut sent = false;
    let mut between = 0;
    for _ in 0..40 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        let x = at(&seen.borrow().drawn, 40.0).w_axis.x;
        if !sent && w >= 3 {
            let script = format!("0:teleport:{g}:1,0,40");
            let answer: Value = serde_json::from_str(&og_engine_probe_commands(e, script)).unwrap();
            assert_eq!(answer["scheduled"], json!(1), "{answer}");
            sent = true;
        } else if sent {
            assert!(
                x == 0.0 || x == 1.0,
                "tick {w}: drawn at x {x}, between 0 and where it landed"
            );
            if x == 1.0 && behind > 0.0 {
                between += 1;
            }
        }
    }
    assert!(
        between > 10,
        "only {between} frames drew the landed entity between ticks"
    );
}

// ── skinned poses ───────────────────────────────────────────────────────────

/// A project with the template's walker, cooked once per test process.
fn walker_project() -> &'static std::path::Path {
    static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-interp-walker-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let to = root.join("assets/models/walker.glb");
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(repo.join("templates/game/assets/models/walker.glb"), &to).unwrap();
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

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

/// The last frame's first joint palette.
struct Palettes(Rc<RefCell<Vec<Mat4>>>);

impl RenderBackend for Palettes {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 36)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .joint_palettes
            .first()
            .map(|p| p.matrices.clone())
            .unwrap_or_default();
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// An engine with the walker walking, and what its renderer was handed.
fn walker() -> (u32, Rc<RefCell<Vec<Mat4>>>) {
    let root = walker_project();
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let e = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(e, Some(Box::new(Palettes(seen.clone())))).unwrap();
    let anim = value(og_runtime::og_load_animation(e, "models/walker.glb".into()));
    let walk = anim["clips"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "walk")
        .expect("the walker walks")
        .clone();
    let asset = value(og_runtime::og_load_gltf(
        e,
        "models/walker.glb".into(),
        None,
    ));
    let id = value(og_runtime::og_world_instantiate(
        e,
        asset.as_f64().unwrap(),
        None,
    ))
    .as_f64()
    .unwrap();
    let skeleton = json!({ "asset": anim["skeleton"] }).to_string();
    value(og_world_insert(e, id, "og.Skeleton".into(), Some(skeleton)));
    let player = json!({ "clip": walk["clip"], "loop": true }).to_string();
    value(og_world_insert(
        e,
        id,
        "og.AnimationPlayer".into(),
        Some(player),
    ));
    (e, seen)
}

/// The angle between two rotations, radians.
fn angle(a: og_core::glam::Quat, b: og_core::glam::Quat) -> f64 {
    2.0 * f64::from(a.dot(b).abs()).min(1.0).acos()
}

/// A skinned character's joints, not only its root, are drawn between the
/// ticks: each joint's drawn rotation is the fraction alpha of the way from
/// the tick before's to the last tick's (the old code drew the last tick's
/// palette whole, so its limbs stepped at 60 Hz while the body glided).
#[test]
fn a_skinned_characters_joints_are_drawn_between_the_ticks() {
    // The palettes of every tick, from whole-tick frames.
    let (ticks, at_tick) = walker();
    let mut by_tick: HashMap<u32, Vec<Mat4>> = HashMap::new();
    for _ in 0..40 {
        let (t, _) = frame(ticks, TICK_MS);
        by_tick.insert(t, at_tick.borrow().clone());
    }
    let (e, seen) = walker();
    let mut checked = 0;
    for _ in 0..80 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        if w < 3 {
            continue;
        }
        let (Some(prev), Some(cur)) = (by_tick.get(&(w - 1)), by_tick.get(&w)) else {
            continue;
        };
        let alpha = behind / TICK_MS;
        let drawn = seen.borrow().clone();
        assert_eq!(drawn.len(), cur.len());
        for j in 0..cur.len() {
            let r0 = prev[j].to_scale_rotation_translation().1;
            let r1 = cur[j].to_scale_rotation_translation().1;
            let d = drawn[j].to_scale_rotation_translation().1;
            let whole = angle(r0, r1);
            if whole < 0.01 {
                continue;
            }
            let (from, to) = (angle(r0, d), angle(d, r1));
            assert!(
                (from - alpha * whole).abs() < 2e-3 && (to - (1.0 - alpha) * whole).abs() < 2e-3,
                "tick {w} joint {j}: drawn {from:.4} rad from the tick before and {to:.4} from the last, of {whole:.4}, at alpha {alpha:.3}"
            );
            checked += 1;
        }
    }
    assert!(checked > 50, "only {checked} turning joints were checked");
}

// ── world-anchored UI ───────────────────────────────────────────────────────

const TAG: u32 = 2;

/// Tag: moves the entity 0.1 m a tick along x and anchors a box to it.
struct Tagger(Rc<std::cell::Cell<f64>>);

impl Hooks for Tagger {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == TAG {
            let w = call.engine;
            let e = self.0.get();
            let t = f64::from(og_world_get_tick(w));
            let at = json!({ "position": { "x": 0.1 * t, "y": 0, "z": 0 } }).to_string();
            value(og_world_set(w, e, "og.Transform".into(), at));
            let desc = json!({ "target": { "entity": e }, "pivot": "center" });
            og_runtime::og_ui_open_world_anchor("Tag".into(), Some(desc.to_string()));
            let block = json!({ "width": { "kind": "fixed", "px": 40 }, "height": { "kind": "fixed", "px": 20 }, "background": { "color": { "r": 1, "g": 0, "b": 0 } } });
            og_runtime::og_ui_open("Block".into(), Some(block.to_string()));
            og_runtime::og_ui_close();
            og_runtime::og_ui_close();
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// The centre of the last frame's first overlay UI shape, CSS px.
struct Shapes(Rc<RefCell<Option<(f32, f32)>>>);

impl RenderBackend for Shapes {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (1280, 720)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .overlay
            .as_ref()
            .and_then(|o| o.ui.shapes.first())
            .map(|s| (s.center.x, s.center.y));
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// An element anchored to a moving entity is drawn on the entity's drawn
/// place between ticks: its offset from the projection of the entity's
/// blended position is the same on every frame (the old code drew it where
/// the last tick placed it, up to a tick ahead of its entity).
#[test]
fn an_anchored_element_stays_on_its_entity_between_ticks() {
    let entity = Rc::new(std::cell::Cell::new(0.0));
    set_hooks(Rc::new(Tagger(entity.clone())));
    og_host_begin_bundle();
    let transform = json!({ "kind": "component", "name": "og.Transform" });
    let tag = og_system_define_system(
        "Tag".into(),
        json!({ "reads": [], "writes": [transform] }).to_string(),
        TAG,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [tag] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(None));
    set_render_backend(e, Some(Box::new(Shapes(seen.clone())))).unwrap();
    entity.set(
        value(og_world_spawn(e, Some("{}".into())))
            .as_f64()
            .unwrap(),
    );
    let camera = json!({ "position": { "x": 0, "y": 0, "z": 5 }, "fov": 50.0 }).to_string();
    value(write_resource(e, "og.Camera", &camera));
    let mut offsets = Vec::new();
    let mut between = 0;
    for _ in 0..60 {
        let (w, behind) = frame(e, 1000.0 / 144.0);
        if w < 4 {
            continue;
        }
        let alpha = behind / TICK_MS;
        // The entity's x was 0.1 T on tick T: drawn at the blend of the
        // last two ticks', W - 2 and W - 1.
        let x = 0.1 * (f64::from(w - 2) + alpha);
        let s: Value = serde_json::from_str(&og_runtime::og_camera_world_to_screen(
            e,
            json!({ "x": x, "y": 0, "z": 0 }).to_string(),
        ))
        .unwrap();
        let (cx, _) = seen.borrow().expect("the anchored box is drawn");
        offsets.push(f64::from(cx) - s["x"].as_f64().unwrap());
        if alpha > 0.2 && alpha < 0.8 {
            between += 1;
        }
    }
    assert!(between > 10, "only {between} frames fell between ticks");
    let first = offsets[0];
    for (i, o) in offsets.iter().enumerate() {
        assert!(
            (o - first).abs() < 1.0,
            "frame {i}: the box sits {o:.2} px from its entity's drawn place, {first:.2} on the first: {offsets:?}"
        );
    }
}
