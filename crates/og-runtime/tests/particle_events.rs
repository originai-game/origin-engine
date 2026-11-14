//! og.ParticleEvent (R7): an emitter with an `events` line is simulated in
//! the tick once a game reads the event, so gameplay sees each birth, death
//! and bounce of the particles drawn, a restored world sends the same
//! events again, and the renderer draws the tick's simulation as it stands.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_emitter, og_system_define_system, og_world_events,
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
            .join(format!("og-runtime-particle-events-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &str| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };
        // Shards thrown up that fall on a slab, bounce, and puff when they
        // die: every kind of event, a sub-emitter and a mesh particle.
        write(
            "fx/shards.ogt",
            "type particles\nversion 1\n---\nrate 30\nlifetime 0.6 0.8\nspeed 2 3\nspread 30\n\
             gravity 0 -9.8 0\ncollision 0.4 0.2\nspace world\nrender mesh tetrahedron\nlit true\n\
             events birth death collision\nsubEmitter death \"fx/puff.ogt\" 3 0.5\n",
        );
        write(
            "fx/puff.ogt",
            "type particles\nversion 1\n---\nrate 0\nlifetime 0.3\nspeed 0.5\n",
        );
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// (tick, entity, kind, index, x, y, z) of every og.ParticleEvent read.
type Seen = Rc<RefCell<Vec<(u32, u64, String, u64, f64, f64, f64)>>>;

struct Game(Seen);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let evs: Value = serde_json::from_str(&og_runtime::host_impl::json(og_world_events(
                call.engine,
                "og.ParticleEvent".into(),
            )))
            .unwrap();
            let tick = og_world_get_tick(call.engine);
            for e in evs.as_array().into_iter().flatten() {
                let p = |k: &str| e["position"][k].as_f64().unwrap();
                self.0.borrow_mut().push((
                    tick,
                    e["entity"].as_u64().unwrap(),
                    e["kind"].as_str().unwrap_or_default().to_string(),
                    e["index"].as_u64().unwrap(),
                    p("x"),
                    p("y"),
                    p("z"),
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

/// The hosted systems' ids and live particles of the last frame.
struct Recorder(Rc<RefCell<Vec<(u64, usize)>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene
            .effects
            .hosted_particles
            .iter()
            .map(|(id, p)| (*id, p.slots.iter().filter(|s| s.alive()).count()))
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

fn ticks(engine: u32, n: u32) {
    let goal = og_world_get_tick(engine) + n;
    while og_world_get_tick(engine) < goal {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

fn spawn(engine: u32, parts: &[(&str, Value)]) -> u64 {
    let e = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some("{}".into()),
    )))
    .as_f64()
    .unwrap();
    for (rec, v) in parts {
        value(og_runtime::host_impl::answer(og_world_insert(
            engine,
            e,
            (*rec).into(),
            Some(v.to_string()),
        )));
    }
    e as u64
}

#[test]
fn an_emitter_with_events_sends_them_from_the_tick_and_replays_them() {
    let seen = Seen::default();
    set_hooks(Rc::new(Game(seen.clone())));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": [{ "kind": "event", "name": "og.ParticleEvent" }], "writes": [] })
            .to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 5, "systems": [watch] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let drawn = Rc::new(RefCell::new(Vec::new()));
    set_render_backend(engine, Some(Box::new(Recorder(drawn.clone())))).unwrap();
    ticks(engine, 1);

    let shards = value(og_runtime::host_impl::answer(og_load_emitter(
        engine,
        "fx/shards.ogt".into(),
    )));
    let at = |y: f64| json!({ "position": { "x": 0, "y": y, "z": 0 } });
    let system = spawn(
        engine,
        &[
            ("og.Transform", at(1.0)),
            ("og.ParticleSystem", json!({ "emitter": shards, "seed": 9 })),
        ],
    );
    spawn(
        engine,
        &[
            ("og.Transform", at(0.0)),
            (
                "og.Collider",
                json!({ "shape": "box", "size": { "x": 20, "y": 0.2, "z": 20 } }),
            ),
        ],
    );
    ticks(engine, 60);
    {
        let s = seen.borrow();
        let count = |k: &str| s.iter().filter(|e| e.2 == k).count();
        assert!(
            count("birth") >= 25,
            "about 30 births a second: {}",
            count("birth")
        );
        assert!(count("death") > 0, "shards die within a second");
        assert!(count("collision") > 0, "shards fall on the slab");
        assert!(s.iter().all(|e| e.1 == system));
        // A shard is born at the emitter (moved by its first part-tick, at
        // most 3 m/s for 1/60 s) and bounces on the slab's top.
        for e in s.iter().filter(|e| e.2 == "birth") {
            let d = (e.4 * e.4 + (e.5 - 1.0) * (e.5 - 1.0) + e.6 * e.6).sqrt();
            assert!(d < 0.06, "born at the emitter: {e:?}");
        }
        for e in s.iter().filter(|e| e.2 == "collision") {
            assert!(e.5 > -0.05 && e.5 < 0.3, "on the slab's top (0.1): {e:?}");
        }
        // Each particle is born once, in index order.
        let births: Vec<u64> = s.iter().filter(|e| e.2 == "birth").map(|e| e.3).collect();
        assert!(births.windows(2).all(|w| w[0] < w[1]), "{births:?}");
    }
    // The renderer draws the tick's simulation of the system.
    let d = drawn.borrow().clone();
    assert!(
        d.iter().any(|&(id, alive)| id == system && alive > 0),
        "the hosted system is drawn: {d:?}"
    );

    // A restored world sends the same events again.
    let snap = value(og_runtime::host_impl::answer(og_world_snapshot(
        engine, None,
    )))
    .as_str()
    .unwrap()
    .to_string();
    seen.borrow_mut().clear();
    ticks(engine, 40);
    let first = std::mem::take(&mut *seen.borrow_mut());
    assert!(!first.is_empty());
    value(og_runtime::host_impl::answer(og_world_restore(
        engine, snap, None,
    )));
    ticks(engine, 40);
    let again = seen.borrow().clone();
    assert_eq!(first, again, "a restore replays the particle events");
}
