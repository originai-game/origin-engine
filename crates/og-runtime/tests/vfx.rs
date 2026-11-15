//! og.Vfx: a composed effect spawned with one call sends its events on the
//! ticks their times land on (once each, and again after a restore), draws
//! each element only inside its window, and despawns at its end.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_effect, og_load_timeline, og_load_vfx, og_render_box_geometry,
    og_system_define_system, og_timeline_play, og_timeline_seek, og_timeline_stop, og_vfx_info,
    og_vfx_spawn, og_world_alive, og_world_children, og_world_despawn, og_world_events,
    og_world_get_tick, og_world_insert, og_world_restore, og_world_snapshot, og_world_spawn,
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
        // A sound 0.1 s in, its playback rate picked from a range.
        let pcm: Vec<f32> = (0..2400)
            .map(|i| og_core::math::sin(i as f32 * 0.1) * 0.25)
            .collect();
        let p = root.join("assets/sfx/blip.wav");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, og_audio::wav(&pcm, 1, 48_000)).unwrap();
        write(
            "fx/loud.ogt",
            "type vfx\nversion 1\n---\nduration 0.4\nsound \"sfx/blip.wav\" {\n  at 0.1\n  volume 0.8\n  playbackRate 0.9 1.1\n}\n",
        );
        // A ring only in a critical instance (its param above 0).
        write(
            "fx/crit.ogt",
            "type vfx\nversion 2\n---\nduration 0.4\nparam crit 0\n\
             mesh ring {\n  life 0.4\n  effect \"fx/shock.ogt\"\n  when crit\n}\n\
             light {\n  life 0.4\n}\n",
        );
        // RenderSettings.maxVfx's effects: 100 particle slots each, two
        // seconds one-shot or looping every second.
        write(
            "fx/ember.ogt",
            "type particles\nversion 1\n---\nmaxParticles 100\nrate 20\nlifetime 1\n",
        );
        write(
            "fx/pop.ogt",
            "type vfx\nversion 1\n---\nduration 2\nemitter \"fx/ember.ogt\" {\n  at 0\n}\n",
        );
        write(
            "fx/hum.ogt",
            "type vfx\nversion 1\n---\nduration 1\nloop true\nemitter \"fx/ember.ogt\" {\n  at 0\n}\n",
        );
        // fx/hit.ogt from 1 s, on the entity named "target", 1 m up.
        write(
            "timelines/fx.ogt",
            "type timeline\nversion 1\n---\nname \"F\"\nduration 3\n\
             track vfx {\n  key 1 {\n    effect \"fx/hit.ogt\"\n    at \"target\"\n    position 0 1 0\n  }\n}\n",
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
            let evs: Value = serde_json::from_str(&og_runtime::host_impl::json(og_world_events(
                call.engine,
                "og.VfxEvent".into(),
            )))
            .unwrap();
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
            scene.frame.tick,
            scene.effects.particles.len(),
            scene.effects.effect_meshes.len(),
            scene.lighting.lights.len(),
            scene
                .sprite
                .sprites
                .iter()
                .map(|s| (s.size.x, s.size.y))
                .collect(),
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

    let hit = value(og_runtime::host_impl::answer(og_load_vfx(
        engine,
        "fx/hit.ogt".into(),
    )));
    let info = value(og_runtime::host_impl::answer(og_vfx_info(
        engine,
        hit.as_f64().unwrap(),
    )));
    assert_eq!(info["events"], json!(["impact", "linger"]));
    assert!((info["impact"].as_f64().unwrap() - 0.1).abs() < 1e-6);

    let start = og_world_get_tick(engine);
    let e = value(og_runtime::host_impl::answer(og_vfx_spawn(
        engine,
        Some(json!({ "asset": hit, "position": { "x": 0, "y": 1, "z": 0 } }).to_string()),
    )))
    .as_f64()
    .unwrap();
    let snap = value(og_runtime::host_impl::answer(og_world_snapshot(
        engine, None,
    )))
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
    value(og_runtime::host_impl::answer(og_world_restore(
        engine,
        snap.clone(),
        None,
    )));
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
    value(og_runtime::host_impl::answer(og_world_restore(
        engine, snap, None,
    )));
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
    let fx = value(og_runtime::host_impl::answer(og_load_vfx(
        engine,
        "fx/pixel.ogt".into(),
    )));
    let info = value(og_runtime::host_impl::answer(og_vfx_info(
        engine,
        fx.as_f64().unwrap(),
    )));
    assert_eq!(info["elements"], json!(["flipbook"]));
    let start = og_world_get_tick(engine);
    value(og_runtime::host_impl::answer(og_vfx_spawn(
        engine,
        Some(json!({ "asset": fx }).to_string()),
    )));
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

/// The archetype tables of the world dump that hold `component`: their rows.
fn rows_with(engine: u32, component: &str) -> u64 {
    og_runtime::og_engine_probe_commands(engine, "0:dump_world".into());
    ticks(engine, 1);
    let outputs: Value = serde_json::from_str(&og_runtime::og_engine_take_outputs(engine)).unwrap();
    let dump: Value = outputs
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["kind"] == "json")
        .map(|o| serde_json::from_str(o["data"].as_str().unwrap()).unwrap())
        .expect("a world dump");
    dump["archetypes"]
        .as_array()
        .expect("archetypes")
        .iter()
        .filter(|a| {
            a["components"]
                .as_array()
                .is_some_and(|c| c.iter().any(|n| n == component))
        })
        .map(|a| a["count"].as_u64().expect("an archetype's entity count"))
        .sum()
}

/// A `sound` element plays its sound from its time on a source of the
/// effect's, which goes with the effect at its end.
///
/// Ways it could fail: the tick the sound is due stops the run (the source
/// spawned from a value without og.AudioSource's defaults), the source
/// never spawned, or left behind when the effect ends.
#[test]
fn a_sound_element_plays_on_a_source_that_ends_with_the_effect() {
    set_hooks(Rc::new(Game(Seen::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    ticks(engine, 1);
    let fx = value(og_runtime::host_impl::answer(og_load_vfx(
        engine,
        "fx/loud.ogt".into(),
    )));
    value(og_runtime::host_impl::answer(og_vfx_spawn(
        engine,
        Some(json!({ "asset": fx }).to_string()),
    )));
    ticks(engine, 3);
    assert_eq!(rows_with(engine, "og.AudioSource"), 0, "none before 0.1 s");
    ticks(engine, 8);
    assert_eq!(
        rows_with(engine, "og.AudioSource"),
        1,
        "its source, playing"
    );
    ticks(engine, 20);
    assert_eq!(
        rows_with(engine, "og.AudioSource"),
        0,
        "gone with the effect"
    );
}

/// og.vfx.spawn's params reach the frame: of two instances of an effect
/// whose ring plays `when crit`, only the one spawned with crit 1 draws
/// it, while both draw their light; og.vfx.info names the param, and a
/// desc with more values than an effect can have is refused.
#[test]
fn a_params_value_given_at_spawn_gates_the_element_that_names_it() {
    set_hooks(Rc::new(Game(Seen::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Drawn::default();
    set_render_backend(
        engine,
        Some(Box::new(Recorder(drawn.clone(), Default::default()))),
    )
    .unwrap();
    ticks(engine, 1);
    let crit = value(og_runtime::host_impl::answer(og_load_vfx(
        engine,
        "fx/crit.ogt".into(),
    )));
    let info = value(og_runtime::host_impl::answer(og_vfx_info(
        engine,
        crit.as_f64().unwrap(),
    )));
    assert_eq!(info["params"], json!(["crit"]));
    let spawn =
        |desc: Value| og_runtime::host_impl::answer(og_vfx_spawn(engine, Some(desc.to_string())));
    value(spawn(json!({ "asset": crit })));
    value(spawn(json!({ "asset": crit, "params": [1] })));
    let refused: Value =
        serde_json::from_str(&spawn(json!({ "asset": crit, "params": [1, 1, 1, 1, 1] }))).unwrap();
    assert_eq!(refused["ok"], json!(false), "{refused}");
    drawn.borrow_mut().clear();
    ticks(engine, 6);
    let d = drawn.borrow().clone();
    let last = d.last().expect("a frame");
    assert_eq!((last.2, last.3), (1, 2), "one ring, two lights: {d:?}");
}

/// Per frame: each beam's (end, drawn with an effect material) and each
/// trail's style.
type Ribbons = Rc<RefCell<Vec<(Vec<([f32; 3], bool)>, Vec<u32>)>>>;

struct RibbonRecorder(Ribbons);

impl RenderBackend for RibbonRecorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().push((
            scene
                .effects
                .beams
                .iter()
                .map(|b| (b.end.to_array(), b.effect.is_some()))
                .collect(),
            scene
                .effects
                .trails
                .iter()
                .map(|t| t.style.index() as u32)
                .collect(),
        ));
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        Default::default()
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// og.load.effect's material reaches the beam and the trail that name it,
/// and a beam's far end follows its target: drawn at the target's place,
/// not `end`, and not drawn once the target is gone. Two trails that
/// differ only in their material are drawn with two styles.
#[test]
fn a_beam_follows_its_target_and_ribbons_draw_their_effect_material() {
    set_hooks(Rc::new(Game(Seen::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Ribbons::default();
    set_render_backend(engine, Some(Box::new(RibbonRecorder(seen.clone())))).unwrap();
    ticks(engine, 1);
    let shock = value(og_runtime::host_impl::answer(og_load_effect(
        engine,
        "fx/shock.ogt".into(),
    )));
    let spawn = |x: f32| {
        value(og_runtime::host_impl::answer(og_world_spawn(
            engine,
            Some(json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
        )))
        .as_f64()
        .unwrap()
    };
    let insert = |e: f64, component: &str, v: Value| {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            e,
            component.into(),
            Some(v.to_string()),
        )));
    };
    let target = spawn(4.0);
    let beam = spawn(0.0);
    insert(
        beam,
        "og.Beam",
        json!({ "target": target, "end": { "x": 0, "y": 9, "z": 0 }, "effect": shock }),
    );
    for (x, material) in [(1.0, json!(shock)), (2.0, json!(0))] {
        let t = spawn(x);
        insert(t, "og.Trail", json!({ "effect": material }));
    }
    ticks(engine, 2);
    let (beams, trails) = seen.borrow().last().cloned().expect("a frame");
    assert_eq!(beams, vec![([4.0, 0.0, 0.0], true)], "at its target");
    assert_eq!(trails.len(), 2);
    assert_ne!(trails[0], trails[1], "a material makes its own style");
    value(og_runtime::host_impl::answer(og_world_despawn(
        engine, target,
    )));
    ticks(engine, 2);
    let (beams, _) = seen.borrow().last().cloned().expect("a frame");
    assert!(beams.is_empty(), "a gone target draws no beam: {beams:?}");
}

/// Per frame: where the scene's lights are.
type Lit = Rc<RefCell<Vec<Vec<[f32; 3]>>>>;

struct LightRecorder(Lit);

impl RenderBackend for LightRecorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().push(
            scene
                .lighting
                .lights
                .iter()
                .map(|l| l.position.to_array())
                .collect(),
        );
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        Default::default()
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// A timeline's vfx key plays fx/hit.ogt (its light from 0.1 s to 0.3 s,
/// its events `impact` at 0.1 s and `linger` at 0.3 s) from 1 s on the
/// entity it names. Played through, both events are sent once, the light
/// at the target's place plus the key's offset. A seek into the effect
/// joins it there: lit at once, only the event still ahead sent, also when
/// it was playing. A seek past it and a stop leave nothing lit.
#[test]
fn a_timelines_vfx_key_plays_its_effect_and_a_seek_joins_it_midway() {
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
    let lit = Lit::default();
    set_render_backend(engine, Some(Box::new(LightRecorder(lit.clone())))).unwrap();
    ticks(engine, 1);
    macro_rules! answer {
        ($call:expr) => {
            value(og_runtime::host_impl::answer($call))
        };
    }
    let asset = answer!(og_load_timeline(engine, "timelines/fx.ogt".into()));
    let target = answer!(og_world_spawn(
        engine,
        Some(json!({ "name": "target", "position": { "x": 4, "y": 0, "z": 0 } }).to_string()),
    ))
    .as_f64()
    .unwrap();
    let timeline = answer!(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    answer!(og_world_insert(
        engine,
        timeline,
        "og.Timeline".into(),
        Some(json!({ "asset": asset }).to_string()),
    ));
    let names = |from: usize| -> Vec<String> {
        seen.borrow()[from..].iter().map(|s| s.1.clone()).collect()
    };
    let lit_now = || lit.borrow().last().cloned().unwrap_or_default();

    answer!(og_timeline_play(engine, timeline));
    ticks(engine, 50);
    assert!(names(0).is_empty() && lit_now().is_empty(), "before 1 s");
    let mut lights = Vec::new();
    // The target's children while it plays: one og.Vfx throughout.
    let mut played = Vec::new();
    for _ in 0..50 {
        ticks(engine, 1);
        lights.extend(lit_now());
        for c in og_world_children(engine, target).as_array().unwrap() {
            if !played.contains(c) {
                played.push(c.clone());
            }
        }
    }
    assert_eq!(played.len(), 1, "one effect entity, kept: {played:?}");
    assert_eq!(names(0), ["impact", "linger"], "each once, played through");
    assert!(!lights.is_empty(), "its light shows");
    for p in &lights {
        assert!(
            (p[0] - 4.0).abs() < 1e-4 && (p[1] - 1.0).abs() < 1e-4,
            "at the target plus the key's offset: {p:?}"
        );
    }

    // Into the effect, 0.2 s in: lit at once, only `linger` still to come.
    let before = seen.borrow().len();
    answer!(og_timeline_seek(engine, timeline, 1.2));
    ticks(engine, 1);
    assert_eq!(lit_now().len(), 1, "joined at 0.2 s: its light shows");
    ticks(engine, 20);
    assert_eq!(names(before), ["linger"], "the impact passed is not sent");

    // Past it, then a stop inside it: nothing lit.
    answer!(og_timeline_seek(engine, timeline, 2.0));
    ticks(engine, 2);
    assert!(lit_now().is_empty(), "seeked past its end");
    // Back from 1.15 s, past its impact, to 1.02 s: the effect playing is
    // joined again there, so its impact is ahead and sent once more.
    answer!(og_timeline_seek(engine, timeline, 1.15));
    ticks(engine, 1);
    assert_eq!(lit_now().len(), 1);
    let before = seen.borrow().len();
    answer!(og_timeline_seek(engine, timeline, 1.02));
    ticks(engine, 10);
    assert_eq!(names(before), ["impact"], "re-joined before its impact");
    assert_eq!(lit_now().len(), 1, "one effect, not two");
    answer!(og_timeline_stop(engine, timeline));
    ticks(engine, 2);
    assert!(lit_now().is_empty(), "a stop takes its effect");
}

/// Per frame: each effect mesh's (position, life, colour).
type Meshes = Rc<RefCell<Vec<Vec<([f32; 3], f32, [f32; 4])>>>>;

struct MeshRecorder(Meshes);

impl RenderBackend for MeshRecorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.0.borrow_mut().push(
            scene
                .effects
                .effect_meshes
                .iter()
                .map(|m| (m.transform.w_axis.truncate().to_array(), m.life, m.color))
                .collect(),
        );
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        Default::default()
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// An og.EffectMesh draws its geometry with its effect material at its
/// entity, at the life and colour it holds; without a geometry or a
/// material it draws nothing.
#[test]
fn an_effect_mesh_draws_its_geometry_with_its_material_at_its_life() {
    set_hooks(Rc::new(Game(Seen::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Meshes::default();
    set_render_backend(engine, Some(Box::new(MeshRecorder(seen.clone())))).unwrap();
    ticks(engine, 1);
    macro_rules! answer {
        ($call:expr) => {
            value(og_runtime::host_impl::answer($call))
        };
    }
    let shock = answer!(og_load_effect(engine, "fx/shock.ogt".into()));
    let dome = answer!(og_render_box_geometry(engine, Some("{}".into())));
    for (x, mesh) in [
        (
            3.0,
            json!({ "geometry": dome, "effect": shock, "life": 0.25, "color": { "r": 2, "g": 1, "b": 0.5 }, "opacity": 0.5 }),
        ),
        (5.0, json!({ "geometry": dome })),
        (7.0, json!({ "effect": shock })),
    ] {
        let e = answer!(og_world_spawn(
            engine,
            Some(json!({ "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
        ))
        .as_f64()
        .unwrap();
        answer!(og_world_insert(
            engine,
            e,
            "og.EffectMesh".into(),
            Some(mesh.to_string())
        ));
    }
    ticks(engine, 2);
    assert_eq!(
        seen.borrow().last().cloned().expect("a frame"),
        vec![([3.0, 0.0, 0.0], 0.25, [2.0, 1.0, 0.5, 0.5])]
    );
}

/// Each frame's particle systems, by where they stand (x, rounded).
struct Placed(Rc<RefCell<Vec<Vec<i32>>>>);

impl RenderBackend for Placed {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let mut xs: Vec<i32> = scene
            .effects
            .particles
            .iter()
            .map(|p| p.transform.w_axis.x.round() as i32)
            .collect();
        xs.sort();
        self.0.borrow_mut().push(xs);
        Ok(())
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        Default::default()
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
}

/// Past RenderSettings.maxVfx or maxVfxParticles the oldest one-shot
/// effects are left out, then looping ones, and the render dump counts
/// them; the effects' entities stay.
///
/// Ways it could fail: the newest left out instead of the oldest, a
/// looping effect left out while a one-shot draws, the particle limit
/// ignored or counted per effect, the count wrong, or the cap reaching
/// the world (an effect despawned).
#[test]
fn past_max_vfx_the_oldest_one_shot_effects_are_left_out() {
    set_hooks(Rc::new(Game(Seen::default())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 5 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let placed = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Placed(placed.clone())))).unwrap();
    ticks(engine, 1);
    let load = |path: &str| {
        value(og_runtime::host_impl::answer(og_load_vfx(
            engine,
            path.into(),
        )))
    };
    let (hum, pop) = (load("fx/hum.ogt"), load("fx/pop.ogt"));
    // The looping hum first, then a pop a tick later and another after it.
    let mut pops = Vec::new();
    for (asset, x) in [(&hum, -5), (&pop, 0), (&pop, 5)] {
        let e = value(og_runtime::host_impl::answer(og_vfx_spawn(
            engine,
            Some(json!({ "asset": asset, "position": { "x": x, "y": 0, "z": 0 } }).to_string()),
        )));
        if x >= 0 {
            pops.push(e.as_f64().unwrap());
        }
        ticks(engine, 1);
    }
    let frame = |settings: Value| {
        value(og_runtime::og_engine_write_resource(
            engine,
            "og.RenderSettings".into(),
            settings.to_string(),
        ));
        // A frame drawn with them, which the dump then reports.
        ticks(engine, 1);
        og_runtime::og_engine_probe_commands(engine, "0:dump_render".into());
        ticks(engine, 1);
        let outputs: Value =
            serde_json::from_str(&og_runtime::og_engine_take_outputs(engine)).unwrap();
        let dump: Value = outputs
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["kind"] == "json")
            .map(|o| serde_json::from_str(o["data"].as_str().unwrap()).unwrap())
            .expect("a render dump");
        let xs = placed.borrow().last().cloned().unwrap();
        (xs, dump["vfx"].clone())
    };
    let counts = |drawn: u32, dropped: u32| json!({ "drawn": drawn, "dropped": dropped });
    assert_eq!(frame(json!({})), (vec![-5, 0, 5], counts(3, 0)));
    assert_eq!(frame(json!({ "maxVfx": 2 })), (vec![-5, 5], counts(2, 1)));
    assert_eq!(frame(json!({ "maxVfx": 1 })), (vec![-5], counts(1, 2)));
    // 300 slots past 250: the older pop goes; under 100 none fits.
    assert_eq!(
        frame(json!({ "maxVfx": 64, "maxVfxParticles": 250 })),
        (vec![-5, 5], counts(2, 1))
    );
    assert_eq!(
        frame(json!({ "maxVfxParticles": 99 })),
        (vec![], counts(0, 3))
    );
    assert!(
        pops.iter().all(|e| og_world_alive(engine, *e)),
        "drawing only"
    );
    assert_eq!(
        frame(json!({ "maxVfxParticles": 65536 })),
        (vec![-5, 0, 5], counts(3, 0))
    );
}
