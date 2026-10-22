//! A one-shot emitter spawned ahead of its first use waits for it
//! (brasswick FINDINGS #3): og.ParticleSystem.startTick's default,
//! PARTICLES_ARMED, is "never written", so a one-shot (`looping false`)
//! emits nothing until a tick is written there, while a looping emitter
//! without one emits as if started at tick 0 (as before). Before, the
//! default was 0, so every one-shot spawned in `start` burst once at its
//! spawn point on the first tick.
//!
//! A project with a one-shot and a looping emitter is cooked once; a
//! renderer that draws nothing records the frame's particle systems.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_emitter, og_world_get, og_world_get_tick, og_world_insert,
    og_world_set, og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The project, cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-armed-emitters-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &[u8]| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        write(
            "fx/hit.ogt",
            b"type particles\nversion 1\n---\nrender sprite 1\nrate 0\nlooping false\nduration 0.3\nburst 0 12\n",
        );
        write(
            "fx/embers.ogt",
            b"type particles\nversion 1\n---\nrender sprite 1\nrate 20\n",
        );
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

/// (system id, start tick) of every particle system the last frame drew.
struct Recorder(Rc<RefCell<Vec<(u64, u64)>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .particles
            .iter()
            .map(|p| (p.id, p.start_tick))
            .collect();
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

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn spawn(engine: u32, system: Value) -> f64 {
    let e = value(og_world_spawn(engine, Some("{}".into())))
        .as_f64()
        .unwrap();
    value(og_world_insert(
        engine,
        e,
        "og.ParticleSystem".into(),
        Some(system.to_string()),
    ));
    e
}

#[test]
fn a_one_shot_never_given_a_start_tick_waits_and_a_looping_one_runs() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    frame(engine);

    let hit = value(og_load_emitter(engine, "fx/hit.ogt".into()));
    let embers = value(og_load_emitter(engine, "fx/embers.ogt".into()));
    let armed = spawn(engine, json!({ "emitter": hit, "seed": 1 }));
    let lasting = spawn(engine, json!({ "emitter": embers, "seed": 2 }));
    let start = value(og_world_get(engine, armed, "og.ParticleSystem".into()))["startTick"].clone();
    assert_eq!(
        start,
        json!(4294967295u32),
        "the default is PARTICLES_ARMED"
    );
    let id = |e: f64| e as u64;

    for _ in 0..5 {
        frame(engine);
        let s = seen.borrow();
        assert!(
            !s.iter().any(|&(e, _)| e == id(armed)),
            "the armed one-shot emits nothing: {s:?}"
        );
        assert!(
            s.contains(&(id(lasting), 0)),
            "the looping emitter runs from tick 0: {s:?}"
        );
    }

    // Its first use: the tick written fires it.
    let tick = og_world_get_tick(engine);
    value(og_world_set(
        engine,
        armed,
        "og.ParticleSystem".into(),
        json!({ "startTick": tick }).to_string(),
    ));
    frame(engine);
    assert!(
        seen.borrow().contains(&(id(armed), u64::from(tick))),
        "fired at {tick}: {:?}",
        seen.borrow()
    );
}
