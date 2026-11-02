//! R10's virtual cameras: og.CameraBrain picks the live og.VirtualCamera
//! (solo, then priority, then the later enabled) and blends og.Camera to
//! it by the incoming shot's curve and time; collision pulls a shot in at
//! once and eases it out; a vcam's focus drives og.PostEffects and is
//! handed back.
//!
//! Ways this could fail: a priority tie going to query order instead of
//! the later enabled; a disabled solo shot ignored, or a released solo
//! not blending back; a blend that is not the curve at the elapsed time
//! (a step late, the wrong ease, linear); an interrupted blend jumping
//! back to the outgoing shot instead of starting where the camera is;
//! collision easing in (the wall shows) or snapping out; the blend's state
//! kept outside the world (a restore mid-blend lands elsewhere); the depth
//! of field left on; the records registered in every world (moving every
//! hash).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_system_define_system,
    og_world_despawn, og_world_events, og_world_get_tick, og_world_insert, og_world_resource,
    og_world_restore, og_world_set, og_world_snapshot, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const WATCH: u32 = 1;

/// (from, to, blend) of every og.CameraCut a test's engines sent.
type Cuts = Rc<RefCell<Vec<(u64, u64, f64)>>>;

thread_local! {
    static CUTS: Cuts = Cuts::default();
}

struct Game(Cuts);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let evs: Value =
                serde_json::from_str(&og_world_events(call.engine, "og.CameraCut".into())).unwrap();
            for e in evs.as_array().into_iter().flatten() {
                let n = |k: &str| e[k].as_f64().unwrap_or(0.0);
                self.0
                    .borrow_mut()
                    .push((n("from") as u64, n("to") as u64, n("blend")));
            }
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn project() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-runtime-virtual-cameras");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    root
}

/// An engine; with `watch`, a game system that reads og.CameraCut (which
/// registers the virtual cameras' records at creation).
fn start(watch: bool) -> (u32, Cuts) {
    start_at(project(), watch)
}

fn start_at(root: PathBuf, watch: bool) -> (u32, Cuts) {
    let cuts = CUTS.with(Rc::clone);
    cuts.borrow_mut().clear();
    set_hooks(Rc::new(Game(cuts.clone())));
    og_host_begin_bundle();
    let mut systems = Vec::new();
    if watch {
        systems.push(og_system_define_system(
            "Watch".into(),
            json!({ "reads": [{ "kind": "event", "name": "og.CameraCut" }], "writes": [] })
                .to_string(),
            WATCH,
        ));
    }
    let game = og_game_game(Some(json!({ "seed": 3, "systems": systems }).to_string()));
    let engine = value(og_engine_create(game, json!({ "root": root }).to_string()))
        .as_u64()
        .unwrap() as u32;
    frames(engine, 1);
    (engine, cuts)
}

fn frames(engine: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

/// Frames until the tick has moved by `n` (a frame may run zero or two).
fn ticks(engine: u32, n: u32) {
    let goal = og_world_get_tick(engine) + n;
    while og_world_get_tick(engine) < goal {
        frames(engine, 1);
    }
    assert_eq!(
        og_world_get_tick(engine),
        goal,
        "a frame ran past the tick asked for"
    );
}

fn spawn(engine: u32, parts: &[(&str, Value)]) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    for (rec, v) in parts {
        value(og_world_insert(
            engine,
            e,
            (*rec).into(),
            Some(v.to_string()),
        ));
    }
    e
}

fn at(x: f64, y: f64, z: f64) -> Value {
    json!({ "position": { "x": x, "y": y, "z": z } })
}

/// A shot at its own entity's position (identity rotation).
fn shot(engine: u32, x: f64, vcam: Value) -> f64 {
    spawn(
        engine,
        &[
            ("og.Transform", at(x, 0.0, 0.0)),
            ("og.VirtualCamera", vcam),
        ],
    )
}

fn camera(engine: u32) -> Value {
    value(og_world_resource(engine, "og.Camera".into()))
}

fn cam_x(engine: u32) -> f64 {
    camera(engine)["position"]["x"].as_f64().unwrap()
}

fn brain(engine: u32) -> Value {
    value(og_world_resource(engine, "og.CameraBrain".into()))
}

fn set(engine: u32, e: f64, rec: &str, v: Value) {
    value(og_world_set(engine, e, rec.into(), v.to_string()));
}

fn hash(engine: u32) -> String {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].as_str().unwrap().to_string()
}

/// Robert Penner's cubic in-out, written out here.
fn cubic_in_out(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

#[test]
fn a_world_without_virtual_cameras_keeps_its_hash() {
    let (engine, _) = start(false);
    spawn(engine, &[("og.Transform", at(1.0, 2.0, 3.0))]);
    // Reading the brain answers its defaults without registering it.
    assert_eq!(brain(engine)["blendTime"], json!(1.0));
    frames(engine, 30);
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    let text = dump.to_string();
    for name in ["og.VirtualCamera", "og.CameraBrain", "og.CameraCut"] {
        assert!(
            !text.contains(name),
            "{name} is in a world that never named it"
        );
    }
    // The same world's hash as before R10's virtual cameras existed.
    assert_eq!(hash(engine), PINNED_HASH);
}

/// [`a_world_without_virtual_cameras_keeps_its_hash`]'s world hashed on
/// the code before og.VirtualCamera (main 00606971, re-pinned after R11
/// made the 2D lighting records lazy and again after ui-gaps' UI state
/// versions 14 and 15; R10 leaves it unmoved on either side).
const PINNED_HASH: &str = "8eff387c719f6801";

#[test]
fn the_brain_picks_by_priority_then_the_later_enabled_and_a_solo_outranks_both() {
    let (engine, cuts) = start(true);
    let a = shot(engine, 1.0, json!({ "priority": 5, "blendTime": 0 }));
    let b = shot(engine, 2.0, json!({ "priority": 10, "blendTime": 0 }));
    ticks(engine, 1);
    assert_eq!(cam_x(engine), 2.0, "the higher priority is live");
    assert_eq!(brain(engine)["live"].as_f64(), Some(b));
    // A tie: the one enabled later wins, whatever the query order.
    let c = shot(engine, 3.0, json!({ "priority": 10, "blendTime": 0 }));
    ticks(engine, 1);
    assert_eq!(
        cam_x(engine),
        3.0,
        "of equal priorities the later enabled is live"
    );
    // Re-enabling b makes it the later one.
    set(engine, b, "og.VirtualCamera", json!({ "enabled": false }));
    ticks(engine, 1);
    assert_eq!(cam_x(engine), 3.0);
    set(engine, b, "og.VirtualCamera", json!({ "enabled": true }));
    ticks(engine, 1);
    assert_eq!(cam_x(engine), 2.0, "b, enabled again, is now the later one");
    // A solo shot outranks every priority, even disabled.
    set(engine, a, "og.VirtualCamera", json!({ "enabled": false }));
    value(og_engine_write_resource(
        engine,
        "og.CameraBrain".into(),
        json!({ "solo": a }).to_string(),
    ));
    ticks(engine, 1);
    assert_eq!(cam_x(engine), 1.0, "the solo shot is live");
    value(og_engine_write_resource(
        engine,
        "og.CameraBrain".into(),
        json!({ "solo": 0 }).to_string(),
    ));
    ticks(engine, 1);
    assert_eq!(
        cam_x(engine),
        2.0,
        "released, the brain goes back to the pick"
    );
    ticks(engine, 1);
    let bits = |e: f64| e as u64;
    let seen: Vec<(u64, u64)> = cuts.borrow().iter().map(|c| (c.0, c.1)).collect();
    assert_eq!(
        seen,
        [
            (0, bits(b)),
            (bits(b), bits(c)),
            (bits(c), bits(b)),
            (bits(b), bits(a)),
            (bits(a), bits(b)),
        ],
        "one og.CameraCut per change of live shot"
    );
    assert!(
        cuts.borrow().iter().all(|c| c.2 == 0.0),
        "every change was a cut"
    );
}

#[test]
fn a_blend_is_the_incoming_curve_at_the_elapsed_time() {
    let (engine, cuts) = start(true);
    shot(engine, 0.0, json!({ "fov": 40 }));
    ticks(engine, 2);
    assert_eq!(cam_x(engine), 0.0, "the first live shot cuts");
    let b = shot(
        engine,
        10.0,
        json!({ "priority": 1, "fov": 60, "blendTime": 1, "blendEase": "cubicInOut" }),
    );
    ticks(engine, 1);
    let switch = og_world_get_tick(engine);
    assert_eq!(brain(engine)["blend"], json!(0.0));
    assert_eq!(
        cam_x(engine),
        0.0,
        "the switch tick shows the outgoing shot"
    );
    let mut seen = Vec::new();
    for n in [1, 15, 30, 45, 59, 60, 61] {
        let goal = switch + n;
        ticks(engine, goal - og_world_get_tick(engine));
        let t = (f64::from(n) / 60.0).min(1.0);
        let w = cubic_in_out(t);
        let cam = camera(engine);
        let x = cam["position"]["x"].as_f64().unwrap();
        let fov = cam["fov"].as_f64().unwrap();
        assert!(
            (x - 10.0 * w).abs() < 1e-3,
            "tick {n}: x {x}, the curve says {}",
            10.0 * w
        );
        assert!(
            (fov - (40.0 + 20.0 * w)).abs() < 1e-3,
            "tick {n}: fov {fov}"
        );
        seen.push(x);
    }
    // Not linear: a quarter of the way in, cubic in-out has done 1/16.
    assert!(seen[1] < 10.0 * 0.25 * 0.5, "{seen:?}");
    assert_eq!(brain(engine)["blend"], json!(1.0));
    assert_eq!(brain(engine)["from"], json!(0));
    // og.CameraCut, read the tick after: the blend's length.
    assert_eq!(
        cuts.borrow().last().map(|c| (c.1, c.2)),
        Some((b as u64, 1.0))
    );
}

#[test]
fn an_interrupted_blend_starts_where_the_camera_is() {
    let (engine, _) = start(false);
    shot(engine, 0.0, json!({}));
    ticks(engine, 2);
    shot(
        engine,
        10.0,
        json!({ "priority": 1, "blendTime": 1, "blendEase": "linear" }),
    );
    ticks(engine, 30);
    let before = cam_x(engine);
    assert!(before > 3.0 && before < 7.0, "mid-blend: {before}");
    let c = spawn(
        engine,
        &[
            ("og.Transform", at(0.0, 0.0, 20.0)),
            (
                "og.VirtualCamera",
                json!({ "priority": 2, "blendTime": 0.5, "blendEase": "linear" }),
            ),
        ],
    );
    ticks(engine, 1);
    let cam = camera(engine);
    assert_eq!(
        cam["position"]["x"].as_f64(),
        Some(before),
        "no jump at the switch"
    );
    let b = brain(engine);
    assert_eq!(b["live"].as_f64(), Some(c));
    assert_eq!(b["from"], json!(0), "from a held pose");
    assert_eq!(b["fromPosition"]["x"].as_f64(), Some(before));
    // Then linear over half a second from the held pose to c.
    ticks(engine, 15);
    let cam = camera(engine);
    let (x, z) = (
        cam["position"]["x"].as_f64().unwrap(),
        cam["position"]["z"].as_f64().unwrap(),
    );
    assert!((x - before * 0.5).abs() < 1e-3, "x {x}");
    assert!((z - 10.0).abs() < 1e-3, "z {z}");
}

#[test]
fn a_restore_mid_blend_continues_the_same_blend() {
    let (engine, _) = start(false);
    shot(engine, 0.0, json!({}));
    ticks(engine, 2);
    shot(engine, 10.0, json!({ "priority": 1, "blendTime": 1 }));
    ticks(engine, 20);
    let snap = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let mut first = Vec::new();
    for _ in 0..10 {
        ticks(engine, 1);
        first.push(cam_x(engine));
    }
    value(og_world_restore(engine, snap, None));
    let mut again = Vec::new();
    for _ in 0..10 {
        ticks(engine, 1);
        again.push(cam_x(engine));
    }
    assert_eq!(first, again);
    assert!(first[0] > 0.0 && first[9] < 10.0, "{first:?}");
}

#[test]
fn collision_pulls_in_at_once_and_eases_out() {
    let (engine, _) = start(false);
    let player = spawn(engine, &[("og.Transform", at(0.0, 0.0, 0.0))]);
    spawn(
        engine,
        &[(
            "og.VirtualCamera",
            json!({
                "follow": player,
                "followOffset": { "x": 0, "y": 0, "z": 6 },
                "smoothTime": 0,
                "lookAt": player,
                "lookOffset": { "x": 0, "y": 0, "z": 0 },
                "collide": true,
                "radius": 0.2,
                "damping": 0.5,
            }),
        )],
    );
    ticks(engine, 2);
    let z = |engine| camera(engine)["position"]["z"].as_f64().unwrap();
    assert!((z(engine) - 6.0).abs() < 1e-4, "clear: {}", z(engine));
    // A wall between the player and the camera: its near face at z 2.9.
    let wall = spawn(
        engine,
        &[
            ("og.Transform", at(0.0, 0.0, 3.0)),
            (
                "og.Collider",
                json!({ "shape": 0, "size": { "x": 4, "y": 4, "z": 0.2 } }),
            ),
        ],
    );
    // The tick after the colliders record it, the camera is in front of it.
    ticks(engine, 2);
    let pulled = z(engine);
    assert!(
        (pulled - 2.7).abs() < 1e-3,
        "pulled in at once to the sphere's stop: {pulled}"
    );
    value(og_world_despawn(engine, wall));
    ticks(engine, 1);
    // Out by damping: the pull shrinks by dt / damping of itself a tick.
    let mut last = z(engine);
    let mut pull = 6.0 - pulled;
    let mut eased = Vec::new();
    for _ in 0..40 {
        ticks(engine, 1);
        let now = z(engine);
        assert!(now > last, "easing out, never back in: {now} after {last}");
        assert!(now - last < 0.2, "never snapping out: {now} after {last}");
        last = now;
        eased.push(now);
    }
    // Independent of where the first easing tick fell: every tick after
    // one keeps (1 - dt / damping) of the pull.
    for w in eased.windows(2) {
        let (p0, p1) = (6.0 - w[0], 6.0 - w[1]);
        assert!(
            (p1 - p0 * (1.0 - (1.0 / 60.0) / 0.5)).abs() < 1e-3,
            "{p0} -> {p1}"
        );
        pull = p1;
    }
    assert!(pull < 1.0, "most of the way out after 40 ticks: {pull}");
}

#[test]
fn a_shot_with_focus_hands_the_depth_of_field_back() {
    let (engine, _) = start(false);
    let post = |engine| value(og_world_resource(engine, "og.PostEffects".into()));
    assert_eq!(post(engine)["depthOfField"], json!(false));
    let range0 = post(engine)["dofFocusRange"].clone();
    let a = shot(
        engine,
        0.0,
        json!({ "focus": 5, "focusRange": 2, "blendTime": 0 }),
    );
    ticks(engine, 2);
    let p = post(engine);
    assert_eq!(p["depthOfField"], json!(true));
    assert_eq!(p["dofFocusDistance"], json!(5.0));
    assert_eq!(p["dofFocusRange"], json!(2.0));
    set(engine, a, "og.VirtualCamera", json!({ "enabled": false }));
    ticks(engine, 1);
    let p = post(engine);
    assert_eq!(
        p["depthOfField"],
        json!(false),
        "handed back with the shot gone"
    );
    assert_eq!(p["dofFocusRange"], range0);
    assert_eq!(brain(engine)["live"], json!(0));
}

fn named_shot(engine: u32, name: &str, x: f64, vcam: Value) -> f64 {
    let e = value(og_world_spawn(
        engine,
        Some(json!({ "name": name }).to_string()),
    ))
    .as_f64()
    .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.Transform".into(),
        Some(at(x, 0.0, 0.0).to_string()),
    ));
    value(og_world_insert(
        engine,
        e,
        "og.VirtualCamera".into(),
        Some(vcam.to_string()),
    ));
    e
}

fn solo(engine: u32) -> f64 {
    brain(engine)["solo"].as_f64().unwrap_or(0.0)
}

const TALK: &str = "type dialogue
version 1
---
name talk
start one
speaker smith {
  name \"npc.smith\"
}
node one {
  shot close
  say smith \"l.one\"
  say smith \"l.two\" {
    shot smith
  }
  say smith \"l.three\" {
    shot none
  }
  say smith \"l.four\"
}
";

/// R10 batch 3: a dialogue line's shot is og.CameraBrain's solo while it
/// shows, a node's covers its lines, a cast role finds the virtual camera
/// that looks at its entity, `shot none` and the end hand the game's own
/// solo back.
///
/// Ways this could fail: the story writes the solo but never hands it
/// back (the camera stays on the close-up after the talk); it hands back
/// 0 instead of the game's solo; a role is looked up as an entity name;
/// a solo the game set during the talk is overwritten at the end.
#[test]
fn a_dialogue_takes_the_games_solo_for_its_shots_and_hands_it_back() {
    let (engine, _) = start(false);
    let game = shot(engine, 1.0, json!({ "priority": 1, "blendTime": 0 }));
    let close = named_shot(
        engine,
        "close",
        2.0,
        json!({ "enabled": false, "blendTime": 0 }),
    );
    let smith = spawn(engine, &[("og.Transform", at(9.0, 0.0, 0.0))]);
    let on_smith = shot(
        engine,
        3.0,
        json!({ "enabled": false, "lookAt": smith, "blendTime": 0 }),
    );
    value(og_engine_write_resource(
        engine,
        "og.CameraBrain".into(),
        json!({ "solo": game }).to_string(),
    ));
    value(og_runtime::og_story_parse(engine, TALK.into()));
    let conv = value(og_runtime::og_story_start(
        engine,
        Some(json!({ "dialogue": "talk", "cast": { "smith": smith } }).to_string()),
    ))
    .as_u64()
    .unwrap() as u32;
    ticks(engine, 2);
    let view: Value = serde_json::from_str(&og_runtime::og_story_view(engine, conv)).unwrap();
    assert_eq!(
        view["shot"],
        json!("close"),
        "the node's shot covers its line"
    );
    assert_eq!(solo(engine), close, "the line's shot is the solo");
    value(og_runtime::og_story_advance(engine, conv));
    ticks(engine, 2);
    assert_eq!(
        solo(engine),
        on_smith,
        "the cast role's camera: the one that looks at smith"
    );
    assert!((cam_x(engine) - 3.0).abs() < 1e-6, "and it is live");
    value(og_runtime::og_story_advance(engine, conv));
    ticks(engine, 2);
    assert_eq!(solo(engine), game, "`shot none`: the game's solo is back");
    value(og_runtime::og_story_advance(engine, conv));
    ticks(engine, 2);
    assert_eq!(solo(engine), close, "the node's shot again");
    // The game picks another solo during the talk: that one is what the
    // end hands back.
    let other = shot(engine, 4.0, json!({ "enabled": false, "blendTime": 0 }));
    value(og_engine_write_resource(
        engine,
        "og.CameraBrain".into(),
        json!({ "solo": other }).to_string(),
    ));
    ticks(engine, 2);
    value(og_runtime::og_story_advance(engine, conv));
    ticks(engine, 2);
    assert_eq!(
        solo(engine),
        other,
        "the end hands back the game's latest solo"
    );
    assert!((cam_x(engine) - 4.0).abs() < 1e-6);
}

const CUTSCENE: &str = "type timeline
version 1
---
duration 1
track shot {
  key 0.2 {
    camera \"wide\"
    blend 0.5
    ease linear
  }
  key 0.6 {
    none
  }
}
";

/// A timeline's shot track sets og.CameraBrain.solo from its keys with
/// the key's blend and ease, and a `none` key hands the found solo back.
#[test]
fn a_shot_track_soloes_its_cameras_with_its_blend_and_hands_the_solo_back() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-runtime-vcam-shot-track");
    std::fs::create_dir_all(root.join("assets/timelines")).unwrap();
    std::fs::write(root.join("assets/timelines/cut.ogt"), CUTSCENE).unwrap();
    let cooked = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(cooked.ok(), "{:?}", cooked.errors());
    let (engine, _) = start_at(root, false);
    let game = shot(engine, 1.0, json!({ "priority": 1, "blendTime": 0 }));
    let wide = named_shot(
        engine,
        "wide",
        5.0,
        json!({ "enabled": false, "blendTime": 0 }),
    );
    let tl = value(og_runtime::og_load_timeline(
        engine,
        "timelines/cut.ogt".into(),
    ));
    spawn(
        engine,
        &[("og.Timeline", json!({ "asset": tl, "playing": true }))],
    );
    ticks(engine, 18);
    assert_eq!(solo(engine), wide, "the key at 0.2 s soloes the wide shot");
    let b = brain(engine);
    assert_eq!(
        b["blendDuration"].as_f64(),
        Some(0.5),
        "with the key's blend: {b}"
    );
    assert_eq!(b["blendCurve"], json!("linear"));
    let x = cam_x(engine);
    assert!(x > 1.0 && x < 5.0, "mid-blend: {x}");
    ticks(engine, 24);
    assert_eq!(
        solo(engine),
        0.0,
        "the `none` key hands back the solo it found (none)"
    );
    let _ = game;
}
