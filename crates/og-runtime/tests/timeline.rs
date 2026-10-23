//! og.timeline on the runtime (docs/design.md §2.7, docs/depth.md §2):
//! seeking is exact, and playback raises its events and hands back what it
//! took.
//!
//! A project with one timeline (`timelines/t.ogt`: a camera on a rail, a
//! transform track on the entity "box", two markers and two event keys,
//! one retroactive) is cooked once. The game has one system (Watch), which
//! records the timeline events it reads, and one command (`mark`), which
//! records its arguments; the event keys run `mark:<x>`.
//!
//! Ways this could fail: the time is accumulated in seconds (`time += dt`
//! in f32), so 240 ticks at 60 Hz land beside 4 s and the world played to
//! 4 s hashes differently from the one sought to 4 s; a key is evaluated
//! from the previous tick's value instead of the time; a marker or event
//! fires twice, at the wrong tick, or not when the tick that crosses it is
//! the first; a seek runs non-retroactive events it jumps over, or skips a
//! retroactive one; the camera stays where the timeline left it after the
//! end. Broken once by applying a Stop queued before a Play (what
//! `timeline:play` queues) after it: the timeline never started.
//!
//! A hot reload (`og dev`: the `.ogt` edited and cooked, then the probe's
//! `reload`) while a timeline plays: the playback continues from the same
//! time with the edited keys, and the state block (the camera it owns, that
//! it started) survives. Broken before: the loaded timeline was cached by
//! id and never read again, so the old keys played on.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_commands,
    og_engine_probe_dump, og_engine_probe_reload, og_game_game, og_host_begin_bundle,
    og_load_timeline, og_system_define_system, og_timeline_info, og_timeline_pause,
    og_timeline_play, og_timeline_seek, og_world_events, og_world_find, og_world_get,
    og_world_get_tick, og_world_insert, og_world_resource, og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const WATCH: u32 = 1;
const MARK: u32 = 2;

const TIMELINE: &str = r#"type timeline
version 1
---
name "T"
duration 6
path "arc" {
  kind catmullRom
  point -4 3 8
  point 0 2 7
  point 4 3 8
}
track camera {
  rail "arc"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 6 {
    at 1
    target 2 1 0
    fov 40
    ease cubicInOut
  }
}
track transform "box" {
  key 0 {
    position 0 0 0
  }
  key 3 {
    position 3 0 0
    ease quadIn
  }
  key 6 {
    position 3 2 0
    rotation 0 90 0
  }
}
track event {
  key 1.5 {
    do "mark:early"
  }
  key 3.5 {
    do "mark:late"
    retroactive true
  }
}
track marker {
  key 1 {
    name "one"
  }
  key 5 {
    name "five"
  }
}
"#;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| fresh_project("og-runtime-timeline"))
}

/// A project of its own with `t.ogt`, cooked (`name` and the process id).
fn fresh_project(name: &str) -> PathBuf {
    let root =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_timeline(&root, TIMELINE);
    root
}

/// Writes `timelines/t.ogt` and cooks the project (what `og dev` does on an edit).
fn write_timeline(root: &Path, text: &str) {
    let p = root.join("assets/timelines/t.ogt");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
    let report = og_cook::cook_project(root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
}

#[derive(Default)]
struct Seen {
    /// (tick, event name, marker name or "").
    events: Vec<(u32, String, String)>,
    /// (tick, the `mark` command's arguments).
    marks: Vec<(u32, Vec<String>)>,
}

struct Game(Rc<RefCell<Seen>>);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let w = call.engine;
        if call.kind == 0 && call.id == WATCH {
            let tick = og_world_get_tick(w);
            for name in [
                "og.TimelineStarted",
                "og.TimelineMarker",
                "og.TimelineEnded",
            ] {
                let evs: Value = serde_json::from_str(&og_world_events(w, name.into())).unwrap();
                for e in evs.as_array().into_iter().flatten() {
                    let m = e["name"].as_str().unwrap_or("").to_string();
                    self.0.borrow_mut().events.push((tick, name.into(), m));
                }
            }
        } else if call.kind == 1 && call.id == MARK {
            let args: Vec<String> = serde_json::from_str(&call.payload).unwrap_or_default();
            self.0.borrow_mut().marks.push((og_world_get_tick(w), args));
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

struct Run {
    engine: u32,
    timeline: f64,
    seen: Rc<RefCell<Seen>>,
}

/// An engine with "box" and an entity whose og.Timeline holds `t.ogt`
/// (stopped), one frame in.
fn start() -> Run {
    start_in(project())
}

fn start_in(root: &Path) -> Run {
    let seen = Rc::new(RefCell::new(Seen::default()));
    set_hooks(Rc::new(Game(seen.clone())));
    og_host_begin_bundle();
    let ev = |n: &str| json!({ "kind": "event", "name": n });
    let watch = og_system_define_system(
        "Watch".into(),
        json!({
            "reads": [ev("og.TimelineStarted"), ev("og.TimelineMarker"), ev("og.TimelineEnded")],
            "writes": [],
        })
        .to_string(),
        WATCH,
    );
    let game = og_game_game(Some(
        json!({ "seed": 1, "systems": [watch], "commands": { "mark": MARK } }).to_string(),
    ));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    value(og_world_spawn(
        engine,
        Some(json!({ "name": "box" }).to_string()),
    ));
    let asset = value(og_load_timeline(engine, "timelines/t.ogt".into()));
    let timeline = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        timeline,
        "og.Timeline".into(),
        Some(json!({ "asset": asset }).to_string()),
    ));
    frame(engine, 1);
    Run {
        engine,
        timeline,
        seen,
    }
}

fn frame(engine: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

fn hash(engine: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].clone()
}

fn get(r: &Run, name: &str) -> Value {
    let id: f64 = og_world_find(r.engine, name.into()).parse().unwrap();
    value(og_world_get(r.engine, id, "og.Transform".into()))
}

fn state(r: &Run) -> (Value, Value) {
    (
        value(og_world_get(r.engine, r.timeline, "og.Timeline".into())),
        value(og_world_resource(r.engine, "og.Camera".into())),
    )
}

#[test]
fn seeking_to_4s_hashes_as_playing_to_4s() {
    // Played: 1 tick at 0, then 240 to 4 s, still playing.
    let a = start();
    value(og_timeline_play(a.engine, a.timeline));
    frame(a.engine, 241);
    // Sought: played one tick, paused at 0, waited, then sought to 4 s and
    // resumed in the same tick.
    let b = start();
    value(og_timeline_play(b.engine, b.timeline));
    frame(b.engine, 1);
    value(og_timeline_pause(b.engine, b.timeline));
    frame(b.engine, 239);
    value(og_timeline_seek(b.engine, b.timeline, 4.0));
    value(og_timeline_play(b.engine, b.timeline));
    frame(b.engine, 1);

    assert_eq!(og_world_get_tick(a.engine), og_world_get_tick(b.engine));
    let (ta, ca) = state(&a);
    let (tb, cb) = state(&b);
    assert_eq!(ta["time"], json!(4.0), "exactly 4 s after 240 ticks: {ta}");
    assert_eq!(ta["playing"], json!(true));
    assert_eq!(ta, tb);
    assert_eq!(ca, cb);
    // The box at 4 s: past the key at 3 s (3 0 0), a third of the way to
    // (3 2 0), and turning.
    let p = &get(&a, "box")["position"];
    assert!((p["x"].as_f64().unwrap() - 3.0).abs() < 1e-5, "{p}");
    assert!((p["y"].as_f64().unwrap() - 2.0 / 3.0).abs() < 1e-4, "{p}");
    assert_eq!(get(&a, "box"), get(&b, "box"));
    assert_eq!(
        hash(a.engine),
        hash(b.engine),
        "the whole world, state blocks included"
    );

    // The seek ran the retroactive event it jumped over (3.5 s), not the
    // other (1.5 s); playing ran both, once each. Event actions are
    // commands, run at the next tick's start.
    frame(b.engine, 1);
    let marks = |r: &Run| -> Vec<String> {
        r.seen
            .borrow()
            .marks
            .iter()
            .map(|m| m.1.join(":"))
            .collect()
    };
    assert_eq!(marks(&a), vec!["early", "late"]);
    assert_eq!(marks(&b), vec!["late"]);
}

#[test]
fn playback_raises_markers_once_and_hands_the_camera_back_at_the_end() {
    let r = start();
    let (_, before) = state(&r);
    let info = value(og_timeline_info(
        r.engine,
        state(&r).0["asset"].as_f64().unwrap(),
    ));
    assert_eq!(info["duration"], json!(6.0));
    assert_eq!(info["markers"], json!(["one", "five"]));
    let t0 = og_world_get_tick(r.engine);
    value(og_timeline_play(r.engine, r.timeline));
    frame(r.engine, 180);
    // Mid-play the camera is the rail's.
    let (tl, mid) = state(&r);
    assert_eq!(tl["playing"], json!(true));
    assert_ne!(mid["position"], before["position"], "{mid}");
    frame(r.engine, 200);
    let (tl, after) = state(&r);
    assert_eq!(tl["playing"], json!(false), "{tl}");
    assert_eq!(tl["time"], json!(6.0));
    for k in ["position", "quaternion", "fov"] {
        assert_eq!(after[k], before[k], "the camera's {k} is handed back");
    }
    let seen = r.seen.borrow();
    let names: Vec<(&str, &str)> = seen
        .events
        .iter()
        .map(|e| (e.1.as_str(), e.2.as_str()))
        .collect();
    assert_eq!(
        names,
        vec![
            ("og.TimelineStarted", ""),
            ("og.TimelineMarker", "one"),
            ("og.TimelineMarker", "five"),
            ("og.TimelineEnded", ""),
        ],
        "{:?}",
        seen.events
    );
    // The marker at 1 s is raised by the tick whose time reaches 1 s (the
    // play's first tick is at 0 s, 60 ticks a second), read one tick later.
    let one = seen.events[1].0 - t0;
    assert_eq!(one, 60 + 1, "{:?}", seen.events);
    assert_eq!(seen.marks.len(), 2);
}

/// The command channel's path (`og run --commands "0:timeline:play:<name>"`,
/// what depth S3's video capture runs): the play makes `timeline-<name>`,
/// loads `timelines/<name>.ogt` and plays it from 0 (a Stop is queued
/// before the Play, and both apply); a seek moves it.
#[test]
fn timeline_play_and_seek_from_the_command_channel() {
    let r = start();
    let run = |script: &str| {
        let v: Value = serde_json::from_str(&og_engine_probe_commands(r.engine, script.into()))
            .expect("wire JSON");
        assert_eq!(v["scheduled"], json!(1), "{v}");
        assert_eq!(v["warnings"], json!([]), "{v}");
    };
    run("0:timeline:play:t");
    frame(r.engine, 31);
    let e: f64 = og_world_find(r.engine, "timeline-t".into())
        .parse()
        .expect("timeline:play made timeline-t");
    let tl = |r: &Run| value(og_world_get(r.engine, e, "og.Timeline".into()));
    let now = tl(&r);
    assert_eq!(now["playing"], json!(true), "{now}");
    let t = now["time"].as_f64().unwrap();
    assert!((0.4..=0.55).contains(&t), "{now}");
    run("0:timeline:seek:5:t");
    frame(r.engine, 2);
    let t = tl(&r)["time"].as_f64().unwrap();
    assert!((5.0..=5.05).contains(&t), "{t}");
    // The start()'s own og.Timeline was never played.
    assert_eq!(state(&r).0["playing"], json!(false));
}

#[test]
fn a_hot_reload_keeps_playing_from_the_same_time_with_the_edited_keys() {
    let root = fresh_project("og-runtime-timeline-reload");
    let r = start_in(&root);
    value(og_timeline_play(r.engine, r.timeline));
    // 2 s in: the box on its way to the key at 3 s (3 0 0), eased quadIn.
    frame(r.engine, 121);
    let (before, camera) = state(&r);
    assert_eq!(before["time"], json!(2.0), "{before}");
    let x = |p: &Value| p["position"]["x"].as_f64().unwrap();
    let y = |p: &Value| p["position"]["y"].as_f64().unwrap();
    let box_before = get(&r, "box");
    assert!(y(&box_before).abs() < 1e-6, "{box_before}");

    // The edit: the key at 3 s now lifts the box 4 m, and the camera's
    // last key frames a wider view.
    let edited = TIMELINE
        .replace("key 3 {\n    position 3 0 0", "key 3 {\n    position 3 4 0")
        .replace("fov 40", "fov 30");
    assert_ne!(edited, TIMELINE);
    write_timeline(&root, &edited);
    let answer: Value = serde_json::from_str(&og_engine_probe_reload(r.engine)).unwrap();
    assert!(answer.is_object(), "the reload swapped the world: {answer}");

    // The state block survived: the same time, still playing, still
    // holding the camera (its rail's position, not the one it found), and
    // no second og.TimelineStarted.
    let (after, camera_after) = state(&r);
    assert_eq!(after["time"], before["time"], "{after}");
    assert_eq!(after["playing"], json!(true), "{after}");
    assert_eq!(camera_after["position"], camera["position"]);
    frame(r.engine, 1);
    // The next tick evaluates the edited keys at the time it reached: the
    // quadIn ease of (0 0 0) to (3 4 0) at (2 + 1/60) / 3.
    let s = (2.0 + 1.0 / 60.0) / 3.0;
    let k = s * s;
    let moved = get(&r, "box");
    assert!((x(&moved) - 3.0 * k).abs() < 1e-4, "{moved}");
    assert!(
        (y(&moved) - 4.0 * k).abs() < 1e-4,
        "edited key not played: {moved}"
    );
    let t = state(&r).0["time"].as_f64().unwrap();
    assert!((t - (2.0 + 1.0 / 60.0)).abs() < 1e-5, "{t}");
    // To the end: the edited camera key (fov 30) holds at 6 s, then the
    // camera is handed back as it was found.
    frame(r.engine, 238);
    let (_, cam) = state(&r);
    assert!((cam["fov"].as_f64().unwrap() - 30.0).abs() < 1e-3, "{cam}");
    let started = r
        .seen
        .borrow()
        .events
        .iter()
        .filter(|e| e.1 == "og.TimelineStarted")
        .count();
    assert_eq!(started, 1, "{:?}", r.seen.borrow().events);
}
