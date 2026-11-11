//! R10's timeline tracks on the runtime: property, light, visibility, post,
//! blendshape and material tracks, a nested timeline, and the camera's roll
//! and focus.
//!
//! One project: `timelines/t.ogt` drives a lamp (og.PointLight's distance
//! by a property track, its intensity by a light track, its og.Mesh's
//! material emissive intensity), hides a crate, raises og.PostEffects'
//! bloom, opens a face's blendshape, rolls the camera and keeps "hero" in
//! focus, and from 1 s nests `timelines/door.ogt` at double speed (a door
//! sliding 4 m over 2 s and a marker at 1 s).
//!
//! Ways this could fail: a value accumulated over ticks instead of read at
//! the time (a seek lands elsewhere than playing does); a nested timeline
//! evaluated at the parent's time, not shifted to its key and scaled by its
//! speed, or its marker sent twice or under the bare name; the roll
//! accumulating tick over tick; the focus or the material not handed back
//! at the end; a material track writing world state (a restore mid-way then
//! draws the later material until the next tick, or a fresh session never
//! draws it).

use og_render::{MaterialHandle, RenderScene};
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_game_game, og_host_begin_bundle, og_load_timeline, og_render_box_geometry,
    og_render_standard_material, og_system_define_system, og_timeline_pause, og_timeline_play,
    og_timeline_seek, og_world_events, og_world_find, og_world_get, og_world_get_tick,
    og_world_insert, og_world_resource, og_world_restore, og_world_snapshot, og_world_spawn,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const WATCH: u32 = 1;

const MAIN: &str = r#"type timeline
version 1
---
name "R10"
duration 6
track camera {
  key 0 {
    position 0 2 6
    target 0 1 0
    roll 0
    focusOn "hero"
    focusRange 1
  }
  key 4 {
    position 0 2 6
    target 0 1 0
    roll 30
  }
}
track property "lamp" {
  field "og.PointLight.distance"
  key 0 {
    value 2
  }
  key 4 {
    value 10
  }
}
track light "lamp" {
  key 0 {
    intensity 0
  }
  key 4 {
    intensity 8
    ease power1.in
  }
}
track visibility "crate" {
  key 0 {
    visible true
  }
  key 2 {
    visible false
  }
}
track post {
  key 0 {
    vignetteIntensity 0
  }
  key 4 {
    vignetteIntensity 1
  }
}
track blendshape "face" {
  key 0 {
    w0 0
  }
  key 4 {
    w0 1
  }
}
track material "lamp" {
  key 0 {
    emissiveIntensity 0
  }
  key 4 {
    emissiveIntensity 4
  }
}
track timeline {
  key 1 {
    play "timelines/door.ogt"
    speed 2
  }
}
"#;

const DOOR: &str = r#"type timeline
version 1
---
name "Door"
duration 2
track transform "door" {
  key 0 {
    position 0 0 0
  }
  key 2 {
    position 4 0 0
  }
}
track marker {
  key 1 {
    name "open"
  }
}
"#;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-timeline-tracks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (path, text) in [("timelines/t.ogt", MAIN), ("timelines/door.ogt", DOOR)] {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// (engine, tick, marker name) of every og.TimelineMarker, across the
/// engines a test makes (the hooks are one per thread).
type Markers = Rc<RefCell<Vec<(u32, u32, String)>>>;

thread_local! {
    static MARKERS: Markers = Markers::default();
}

struct Game(Markers);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let w = call.engine;
            let evs: Value =
                serde_json::from_str(&og_world_events(w, "og.TimelineMarker".into())).unwrap();
            for e in evs.as_array().into_iter().flatten() {
                let m = e["name"].as_str().unwrap_or("").to_string();
                self.0.borrow_mut().push((w, og_world_get_tick(w), m));
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

/// The lamp material's emissive intensity in the last frame drawn.
type Drawn = Rc<RefCell<Option<f32>>>;

/// Records the lamp's material: the one whose emissive is (1 0.5 0.1).
struct Recorder(Drawn);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let lamp = |h: MaterialHandle| {
            scene
                .standard_material(h)
                .filter(|m| m.emissive == og_core::glam::Vec3::new(1.0, 0.5, 0.1))
        };
        *self.0.borrow_mut() = scene
            .objects
            .instances
            .iter()
            .find_map(|i| lamp(i.material))
            .map(|m| m.emissiveIntensity);
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
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
    markers: Markers,
    drawn: Drawn,
}

fn spawn(engine: u32, name: &str, parts: &[(&str, Value)]) -> f64 {
    let e = value(og_world_spawn(
        engine,
        Some(json!({ "name": name }).to_string()),
    ))
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

/// An engine with the lamp, crate, face, hero and door, and an entity
/// whose og.Timeline holds `t.ogt` (stopped), one frame in.
fn start() -> Run {
    let markers = MARKERS.with(Rc::clone);
    set_hooks(Rc::new(Game(markers.clone())));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [{ "kind": "event", "name": "og.TimelineMarker" }], "writes": [] })
            .to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [watch] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Drawn::default();
    set_render_backend(engine, Some(Box::new(Recorder(drawn.clone())))).unwrap();
    let material = value(og_render_standard_material(
        engine,
        Some(
            json!({ "emissive": { "r": 1, "g": 0.5, "b": 0.1 }, "emissiveIntensity": 0.5 })
                .to_string(),
        ),
    ));
    let geometry = value(og_render_box_geometry(engine, Some("{}".into())));
    let mesh = json!({ "geometry": geometry, "material": material });
    spawn(
        engine,
        "lamp",
        &[
            ("og.Transform", json!({})),
            ("og.PointLight", json!({ "intensity": 1, "distance": 1 })),
            ("og.Mesh", mesh.clone()),
        ],
    );
    let crate_mesh = value(og_render_standard_material(engine, Some("{}".into())));
    spawn(
        engine,
        "crate",
        &[
            (
                "og.Transform",
                json!({ "position": { "x": 100, "y": 0, "z": 0 } }),
            ),
            (
                "og.Mesh",
                json!({ "geometry": geometry, "material": crate_mesh }),
            ),
        ],
    );
    spawn(engine, "face", &[("og.Blendshapes", json!({}))]);
    spawn(
        engine,
        "hero",
        &[(
            "og.Transform",
            json!({ "position": { "x": 0, "y": 1, "z": 0 } }),
        )],
    );
    spawn(engine, "door", &[("og.Transform", json!({}))]);
    let asset = value(og_load_timeline(engine, "timelines/t.ogt".into()));
    let timeline = spawn(engine, "", &[("og.Timeline", json!({ "asset": asset }))]);
    frame(engine, 1);
    Run {
        engine,
        timeline,
        markers,
        drawn,
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

fn get(r: &Run, name: &str, rec: &str) -> Value {
    let id: f64 = og_world_find(r.engine, name.into()).parse().unwrap();
    value(og_world_get(r.engine, id, rec.into()))
}

fn res(r: &Run, rec: &str) -> Value {
    value(og_world_resource(r.engine, rec.into()))
}

fn close(v: &Value, want: f64) -> bool {
    v.as_f64().is_some_and(|x| (x - want).abs() < 1e-4)
}

#[test]
fn every_new_track_seeks_as_it_plays_and_a_nested_timeline_runs_on_its_own_time() {
    // Played to 1.25 s: one tick at 0, then 75.
    let a = start();
    value(og_timeline_play(a.engine, a.timeline));
    frame(a.engine, 76);
    // Sought to 1.25 s from a pause at 0.
    let b = start();
    value(og_timeline_play(b.engine, b.timeline));
    frame(b.engine, 1);
    value(og_timeline_pause(b.engine, b.timeline));
    frame(b.engine, 74);
    value(og_timeline_seek(b.engine, b.timeline, 1.25));
    value(og_timeline_play(b.engine, b.timeline));
    frame(b.engine, 1);
    assert_eq!(og_world_get_tick(a.engine), og_world_get_tick(b.engine));

    let light = get(&a, "lamp", "og.PointLight");
    // power1.in over 4 s at 1.25: 8 × 0.3125²; the property track's distance linearly.
    assert!(close(&light["intensity"], 0.78125), "{light}");
    assert!(close(&light["distance"], 4.5), "{light}");
    assert_eq!(
        get(&a, "crate", "og.Mesh")["visible"],
        json!(true),
        "hidden only from 2 s"
    );
    assert!(close(
        &res(&a, "og.PostEffects")["vignetteIntensity"],
        0.3125
    ));
    let face = get(&a, "face", "og.Blendshapes");
    assert!(
        close(&face["w0"], 0.3125) && face["source"] == json!("component"),
        "{face}"
    );
    // The door, nested from 1 s at speed 2: its own time is 0.5 s, a quarter.
    let door = &get(&a, "door", "og.Transform")["position"];
    assert!(close(&door["x"], 1.0), "{door}");
    // Focus on the hero: |(0 2 6) − (0 1 0)|.
    let post = res(&a, "og.PostEffects");
    assert_eq!(post["depthOfField"], json!(true));
    assert!(close(&post["dofFocusDistance"], 37f64.sqrt()), "{post}");
    assert!(close(&post["dofFocusRange"], 1.0), "{post}");

    for (name, rec) in [
        ("lamp", "og.PointLight"),
        ("crate", "og.Mesh"),
        ("face", "og.Blendshapes"),
        ("door", "og.Transform"),
    ] {
        assert_eq!(get(&a, name, rec), get(&b, name, rec), "{name} {rec}");
    }
    for rec in ["og.PostEffects", "og.Camera"] {
        assert_eq!(res(&a, rec), res(&b, rec), "{rec}");
    }
    assert_eq!(hash(a.engine), hash(b.engine), "the whole world");

    // Past 2 s the crate hides; the door's marker came once, named by its
    // timeline, at the parent's 1 + 1/2 s (tick 91: 1 tick at 0, then 90),
    // read by the game the tick after.
    frame(a.engine, 55);
    assert_eq!(get(&a, "crate", "og.Mesh")["visible"], json!(false));
    let seen: Vec<(u32, String)> = a
        .markers
        .borrow()
        .iter()
        .filter(|m| m.0 == a.engine)
        .map(|m| (m.1, m.2.clone()))
        .collect();
    assert_eq!(
        seen,
        vec![(92, "door/open".to_string())],
        "read the tick after"
    );
}

/// A material track is presentation reapplied from the time: a snapshot
/// restored mid-way draws, on its next frame, the material a straight run
/// draws at that tick; a stop puts the material back as it was.
#[test]
fn a_restore_mid_timeline_draws_the_restored_times_material() {
    // Straight: 61 ticks of playing (one at 0, then 60: 1 s).
    let a = start();
    value(og_timeline_play(a.engine, a.timeline));
    frame(a.engine, 61);
    let at_1s = a.drawn.borrow().unwrap();
    assert!((at_1s - 1.0).abs() < 1e-5, "4 × 1/4 at 1 s: {at_1s}");

    // Saved a tick before 1 s, played on two seconds, restored, one frame.
    let b = start();
    value(og_timeline_play(b.engine, b.timeline));
    frame(b.engine, 60);
    let save = value(og_world_snapshot(b.engine, None))
        .as_str()
        .unwrap()
        .to_string();
    frame(b.engine, 120);
    let at_3s = b.drawn.borrow().unwrap();
    assert!((at_3s - 179.0 / 60.0).abs() < 1e-5, "{at_3s}");
    assert!(value(og_world_restore(b.engine, save, None)).is_null());
    frame(b.engine, 1);
    assert_eq!(
        b.drawn.borrow().unwrap(),
        at_1s,
        "the restored time's material"
    );
    // The world restored and stepped is the straight run's (a restore's
    // own og.Restored event aside).
    for (name, rec) in [("lamp", "og.PointLight"), ("face", "og.Blendshapes")] {
        assert_eq!(get(&a, name, rec), get(&b, name, rec), "{name} {rec}");
    }
    assert_eq!(res(&a, "og.PostEffects"), res(&b, "og.PostEffects"));

    // Stopped: the material as it was made (0.5).
    value(og_runtime::og_timeline_stop(b.engine, b.timeline));
    frame(b.engine, 1);
    assert_eq!(b.drawn.borrow().unwrap(), 0.5);
}
