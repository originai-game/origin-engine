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
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_write_resource,
    og_game_game, og_host_begin_bundle, og_system_define_system, og_world_despawn, og_world_events,
    og_world_get_tick, og_world_insert, og_world_resource, og_world_restore, og_world_snapshot,
    og_world_spawn, set_hooks,
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

const TALK: &str = "#og-profile:yarn-spinner-3.2.2
#og-name:talk
title: one
---
smith: One #line:l.one #shot:close
smith: Two #line:l.two #shot:smith
smith: Three #line:l.three #shot:none
smith: Four #line:l.four #shot:close
===
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
