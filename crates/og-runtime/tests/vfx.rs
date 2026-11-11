//! og.Vfx: a composed effect spawned with one call sends its events on the
//! ticks their times land on (once each, and again after a restore), draws
//! each element only inside its window, and despawns at its end.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_vfx, og_system_define_system, og_vfx_info, og_vfx_spawn,
    og_world_alive, og_world_events, og_world_get_tick, og_world_restore, og_world_snapshot,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const WATCH: u32 = 1;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-vfx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &str| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        write(
            "fx/sparks.ogt",
            "type particles\nversion 1\n---\nrate 40\nlifetime 0.2\nspeed 2\n",
        );
        write(
            "fx/shock.ogt",
            "type effect\nversion 1\n---\nfresnel 2\nalpha curve 0 1 1 0\n",
        );
        // 0.5 s: sparks for its first 0.2 s, an impact at 0.1 s with a ring,
        // a light and a shake, a second event at 0.3 s.
        write(
            "fx/hit.ogt",
            "type vfx\nversion 1\n---\nduration 0.5\nimpact 0.1\nevent 0.3 \"linger\"\n\
             emitter \"fx/sparks.ogt\" {\n  at 0\n  until 0.2\n}\n\
             mesh ring {\n  at 0.1\n  life 0.2\n  effect \"fx/shock.ogt\"\n  scale curve 0 0.5 1 2\n}\n\
             light {\n  at 0.1\n  life 0.2\n  intensity curve 0 10 1 0\n}\n\
             shake {\n  at 0.1\n  trauma 0.2\n}\n",
        );
        // A pixel effect (type vfx2d): 6 frames at 12 fps, its impact on
        // frame 3 (0.25 s), played 0.1 s into a composed effect.
        write(
            "fx/burst.ogt",
            "type vfx2d\nversion 1\n---\nimpact 0.25\n\
             {\"w\": 16, \"h\": 32, \"frames\": 6, \"fps\": 12, \"layers\": [{\"type\": \"flash\", \"r\": 0.8, \"dur\": 1}]}\n",
        );
        write(
            "fx/pixel.ogt",
            "type vfx\nversion 1\n---\nduration 1\nflipbook \"fx/burst.ogt\" {\n  at 0.1\n  size 2\n}\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// (tick read, name, time) of every og.VfxEvent.
type Seen = Rc<RefCell<Vec<(u32, String, f64)>>>;

struct Game(Seen);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let evs: Value =
                serde_json::from_str(&og_world_events(call.engine, "og.VfxEvent".into())).unwrap();
            let tick = og_world_get_tick(call.engine);
            for e in evs.as_array().into_iter().flatten() {
                self.0.borrow_mut().push((
                    tick,
                    e["name"].as_str().unwrap_or_default().to_string(),
                    e["time"].as_f64().unwrap(),
                ));
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

/// Per frame: the scene's tick and its particle systems, effect meshes,
/// lights and (width, height) of its sprites.
type Drawn = Rc<RefCell<Vec<(u64, usize, usize, usize, Vec<(f32, f32)>)>>>;

struct Recorder(Drawn, og_render::FrameStats);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().push((
            scene.tick,
            scene.effects.particles.len(),
            scene.effects.effect_meshes.len(),
            scene.lighting.lights.len(),
            scene.sprites.iter().map(|s| (s.size.x, s.size.y)).collect(),
        ));
        // Stats that follow the scene, so the render dump can be checked
        // against the frame that made them.
        self.1 = og_render::FrameStats {
            draw_calls: 3 + 2 * scene.effects.effect_meshes.len() as u32,
            particle_systems: scene.effects.particles.len() as u32,
            particle_capacity: 64 * scene.effects.particles.len() as u32,
            transparent_draws: scene.effects.effect_meshes.len() as u32,
            ..Default::default()
        };
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        self.1
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

fn ticks(engine: u32, n: u32) {
    let goal = og_world_get_tick(engine) + n;
    while og_world_get_tick(engine) < goal {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

#[test]
fn a_spawned_effect_sends_its_events_draws_its_elements_and_ends() {
    let seen = Seen::default();
    set_hooks(Rc::new(Game(seen.clone())));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [{ "kind": "event", "name": "og.VfxEvent" }], "writes": [] }).to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 5, "systems": [watch] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Drawn::default();
    set_render_backend(
        engine,
        Some(Box::new(Recorder(drawn.clone(), Default::default()))),
    )
    .unwrap();
    ticks(engine, 1);

    let hit = value(og_load_vfx(engine, "fx/hit.ogt".into()));
    let info = value(og_vfx_info(engine, hit.as_f64().unwrap()));
    assert_eq!(info["events"], json!(["impact", "linger"]));
    assert!((info["impact"].as_f64().unwrap() - 0.1).abs() < 1e-6);

    let start = og_world_get_tick(engine);
    let e = value(og_vfx_spawn(
        engine,
        Some(json!({ "asset": hit, "position": { "x": 0, "y": 1, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    let snap = value(og_world_snapshot(engine, None))
        .as_str()
        .unwrap()
        .to_string();
    // The render dump reports what the backend says the frame cost (the
    // effect's preview and cost sheets read it): at 0.2 s, its sparks and
    // its ring.
    ticks(engine, 12);
    og_runtime::og_engine_probe_commands(engine, "0:dump_render".into());
    ticks(engine, 1);
    let outputs: Value = serde_json::from_str(&og_runtime::og_engine_take_outputs(engine)).unwrap();
    let dump: Value = outputs
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["kind"] == "json")
        .map(|o| serde_json::from_str(o["data"].as_str().unwrap()).unwrap())
        .expect("a render dump");
    assert_eq!(
        (
            &dump["draws"],
            &dump["transparentDraws"],
            &dump["particles"]
        ),
        (
            &json!(5),
            &json!(1),
            &json!({ "systems": 1, "capacity": 64, "cpu": 0 })
        ),
        "{dump}"
    );
    value(og_world_restore(engine, snap.clone(), None));
    seen.borrow_mut().clear();
    drawn.borrow_mut().clear();
    ticks(engine, 40);
    let first = seen.borrow().clone();
    let names: Vec<&str> = first.iter().map(|s| s.1.as_str()).collect();
    assert_eq!(names, ["impact", "linger"], "each event once: {first:?}");
    // 0.2 s apart at 60 ticks a second, the impact 6 ticks after the start
    // (read the tick after it is sent).
    assert_eq!(first[1].0 - first[0].0, 12, "{first:?}");
    assert!(
        (6..=8).contains(&(first[0].0 - start)),
        "impact at tick {} of a start at {start}",
        first[0].0
    );
    // Ended (0.5 s): its entity is gone.
    assert!(!og_world_alive(engine, e), "despawned at its end");

    // The ring and the light only between 0.1 and 0.3 s; sparks from the
    // start, none drawn once the effect is gone.
    let d = drawn.borrow().clone();
    let at = |s: f64| -> (usize, usize, usize) {
        let t = start as u64 + (s * 60.0).round() as u64;
        let f = d
            .iter()
            .find(|f| f.0 == t)
            .unwrap_or_else(|| panic!("no frame at {t}: {d:?}"));
        (f.1, f.2, f.3)
    };
    assert_eq!(at(0.05), (1, 0, 0), "sparks alone before the impact");
    assert_eq!(at(0.2), (1, 1, 1), "the ring and the light after it");
    assert_eq!(at(0.4).1 + at(0.4).2, 0, "the ring and light are over");
    assert_eq!(d.last().map(|f| (f.1, f.2, f.3)), Some((0, 0, 0)));
    assert!(d.iter().all(|f| f.4.is_empty()), "no sprite: {d:?}");

    // A restore to just after the spawn sends the same events again.
    seen.borrow_mut().clear();
    value(og_world_restore(engine, snap, None));
    ticks(engine, 40);
    let again: Vec<(u32, String)> = seen.borrow().iter().map(|s| (s.0, s.1.clone())).collect();
    let first: Vec<(u32, String)> = first.into_iter().map(|s| (s.0, s.1)).collect();
    assert_eq!(first, again, "a restore replays the effect's events");
}

/// A `flipbook` element plays a pixel effect (type vfx2d) as a sprite from
/// its time for its clip's length, sized by `size` and the frame's shape,
/// and its clip's impact reaches the game as the effect's og.VfxEvent.
///
/// Ways it could fail: the sprite never drawn, drawn before its time or
/// left behind after its clip, the impact lost or sent twice, or sent at
/// the composed effect's start instead of the flipbook's frame.
#[test]
fn a_flipbook_element_plays_its_sheet_and_passes_its_impact_on() {
    let seen = Seen::default();
    set_hooks(Rc::new(Game(seen.clone())));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [{ "kind": "event", "name": "og.VfxEvent" }], "writes": [] }).to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 5, "systems": [watch] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Drawn::default();
    set_render_backend(
        engine,
        Some(Box::new(Recorder(drawn.clone(), Default::default()))),
    )
    .unwrap();
    ticks(engine, 1);
    let fx = value(og_load_vfx(engine, "fx/pixel.ogt".into()));
    let info = value(og_vfx_info(engine, fx.as_f64().unwrap()));
    assert_eq!(info["elements"], json!(["flipbook"]));
    let start = og_world_get_tick(engine);
    value(og_vfx_spawn(
        engine,
        Some(json!({ "asset": fx }).to_string()),
    ));
    ticks(engine, 80);

    let s = seen.borrow().clone();
    assert_eq!(s.len(), 1, "one impact: {s:?}");
    assert_eq!(s[0].1, "impact");
    // 0.1 s to the flipbook, 0.25 s into its clip: 21 ticks, give or take
    // the tick the event is read on.
    assert!((0.32..=0.4).contains(&s[0].2), "impact at {} s", s[0].2);
    assert!(
        (21..=24).contains(&(s[0].0 - start)),
        "read at tick {}",
        s[0].0 - start
    );

    let d = drawn.borrow().clone();
    let sprites = |t: f64| {
        let tick = start as u64 + (t * 60.0).round() as u64;
        d.iter()
            .find(|f| f.0 == tick)
            .map(|f| f.4.clone())
            .unwrap_or_else(|| panic!("no frame at {tick}"))
    };
    assert!(sprites(0.05).is_empty(), "nothing before 0.1 s");
    // 2 m tall, its width by the 16 x 32 frame.
    assert_eq!(sprites(0.3), vec![(1.0, 2.0)]);
    assert!(sprites(0.75).is_empty(), "gone after its 0.5 s clip");
}
