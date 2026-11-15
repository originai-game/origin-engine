//! og.scene (guides/loading.md §12): preload asks for every streaming
//! group a scene declares and progress sums the bytes of their packs as
//! the host receives them; change puts the scene in place as the next tick
//! starts, the scene before it gone with its descendants, what the game
//! spawned kept; a change asked for and not yet in place is world state, so
//! a snapshot taken between the call and the tick carries it. og.loading's
//! stages reach the host and its status; project.ogt names the screen.
//!
//! One project: `main` (crate `boxA` with a child `lidA`) and `level2`
//! (`door`, with a manual streaming group `art` of one palette). The host
//! answers the pack `Pending`, with a progress the test sets, until the
//! test lets it land (as the web host does while it fetches).
//!
//! Ways this could fail: progress counting a group before it is admitted,
//! or answering 1 while its pack is still on its way; preload not asking
//! for the scene's groups, or release letting go of the scene in place's;
//! the change happening at the call (a system's call then tears the world
//! mid-tick, and two hosts change at different ticks), or a tick late; the
//! boot scene's child left behind; an entity the game spawned despawned
//! with the scene; a pending change lost by a snapshot; a world that never
//! changes scene registering og.SceneState (its hash moves); a stage with a
//! fraction outside 0 to 1 accepted.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_game_game,
    og_host_begin_bundle, og_loading_done, og_loading_stage, og_loading_status, og_scene_change,
    og_scene_current, og_scene_preload, og_scene_progress, og_scene_release, og_stream_request,
    og_stream_status, og_system_define_system, og_world_find, og_world_restore, og_world_snapshot,
    og_world_spawn, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const MAIN: &str = "type scene
version 1
---
name \"Main\"
entity \"boxA\" {
  entity \"lidA\" {
  }
}
";

const LEVEL2: &str = "type scene
version 1
---
name \"Level 2\"
stream \"art\" {
  asset \"p/a.hex\"
}
entity \"door\" {
}
";

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-scene-change-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, text: &str| {
            let p = root.join("assets").join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        };
        write("p/a.hex", "ff0000\n00ff00\n0000ff\n");
        write("scenes/main.ogt", MAIN);
        write("scenes/level2.ogt", LEVEL2);
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// A pack's fetch: pending with `received` bytes until it lands.
#[derive(Default)]
struct Net {
    received: f64,
    landed: bool,
    asked: bool,
    /// og.loading's reports, as the host heard them.
    loading: Vec<(String, f64, String)>,
    /// What og.scene.current answered inside the Watch system, a tick each.
    current: Vec<Option<String>>,
}

/// The system id of [`a_system_reads_the_scene_in_place`]'s Watch.
const WATCH: u32 = 1;

struct Host(Rc<RefCell<Net>>);

impl Hooks for Host {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let now = og_scene_current(call.engine);
            self.0.borrow_mut().current.push(now);
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        if path.contains("/stream/") {
            let mut n = self.0.borrow_mut();
            n.asked = true;
            if !n.landed {
                return Fetch::Pending;
            }
        }
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
    fn read_progress(&self, path: &str) -> Option<(f64, f64)> {
        let n = self.0.borrow();
        (path.contains("/stream/") && n.asked && !n.landed).then_some((n.received, 0.0))
    }
    fn loading(&self, _engine: u32, stage: &str, fraction: f64, label: &str) {
        self.0
            .borrow_mut()
            .loading
            .push((stage.into(), fraction, label.into()));
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn ok<T: og_runtime::wire::ToJson>(r: Result<T, String>) -> Value {
    value(og_runtime::host_impl::answer(r))
}

fn start(project_text: Option<&str>) -> (u32, Rc<RefCell<Net>>) {
    start_with(project_text, false)
}

/// The engine, with the Watch system when `watch`.
fn start_with(project_text: Option<&str>, watch: bool) -> (u32, Rc<RefCell<Net>>) {
    let net = Rc::new(RefCell::new(Net::default()));
    set_hooks(Rc::new(Host(net.clone())));
    og_host_begin_bundle();
    let mut desc = json!({
        "seed": 1,
        "scenes": { "main": "scenes/main.ogt", "level2": "scenes/level2.ogt" },
        "scene": "main",
    });
    if watch {
        let system = og_system_define_system(
            "Watch".into(),
            json!({ "reads": [], "writes": [] }).to_string(),
            WATCH,
        );
        desc["systems"] = json!([system]);
    }
    let game = og_game_game(Some(desc.to_string()));
    let mut options = json!({ "root": project().to_str().unwrap() });
    if let Some(text) = project_text {
        options["project"] = json!(text);
    }
    let engine = value(og_engine_create(game, options.to_string()))
        .as_u64()
        .unwrap() as u32;
    (engine, net)
}

fn frame(engine: u32) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

fn alive(engine: u32, name: &str) -> bool {
    og_world_find(engine, name.into()).is_some()
}

fn hash(engine: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].clone()
}

#[test]
fn progress_sums_the_scenes_packs_as_they_arrive_and_preload_asks_for_them() {
    let (engine, net) = start(None);
    frame(engine);
    let p = ok(og_scene_progress(engine, "level2".into()));
    assert_eq!(p["scene"], json!("scenes/level2.ogt"));
    assert_eq!(p["groups"], json!(["art"]));
    assert_eq!(p["ready"], json!(false));
    assert_eq!(p["fraction"], json!(0.0), "not admitted: nothing counted");
    let bytes = p["bytes"].as_f64().unwrap();
    assert!(bytes > 0.0, "{p}");
    // The boot scene declares no group: ready at once.
    assert_eq!(
        ok(og_scene_progress(engine, "main".into()))["ready"],
        json!(true)
    );

    ok(og_scene_preload(engine, "level2".into()));
    assert_eq!(
        ok(og_stream_status(engine, "art".into()))["requested"],
        json!(true)
    );
    frame(engine);
    net.borrow_mut().received = bytes / 4.0;
    let s = ok(og_stream_status(engine, "art".into()));
    assert_eq!(s["state"], json!("loading"), "{s}");
    assert_eq!(s["received"].as_f64().unwrap(), bytes / 4.0, "{s}");
    let p = ok(og_scene_progress(engine, "level2".into()));
    assert!((p["fraction"].as_f64().unwrap() - 0.25).abs() < 1e-6, "{p}");
    assert_eq!(p["waiting"], json!(["art"]));

    net.borrow_mut().landed = true;
    frame(engine);
    frame(engine);
    let p = ok(og_scene_progress(engine, "level2".into()));
    assert_eq!(p["ready"], json!(true), "{p}");
    assert_eq!(p["fraction"], json!(1.0), "{p}");
    assert_eq!(
        ok(og_stream_status(engine, "art".into()))["progress"],
        json!(1.0)
    );

    ok(og_scene_release(engine, "level2".into()));
    assert_eq!(
        ok(og_stream_status(engine, "art".into()))["requested"],
        json!(false)
    );
    // A scene that is neither a name nor a cooked scene is refused by name.
    let refused = og_scene_progress(engine, "level9".into()).unwrap_err();
    assert!(
        refused.contains("main, level2") || refused.contains("level2, main"),
        "{refused}"
    );
}

#[test]
fn a_change_takes_effect_as_the_next_tick_starts_and_survives_a_snapshot() {
    let (a, _) = start(None);
    frame(a);
    let untouched = hash(a);
    assert!(alive(a, "boxA") && alive(a, "lidA"));
    assert_eq!(og_scene_current(a).as_deref(), Some("scenes/main.ogt"));
    let hero = ok(og_world_spawn(
        a,
        Some(json!({ "name": "hero" }).to_string()),
    ));
    assert_ne!(hash(a), untouched, "a spawn moves the hash");

    ok(og_scene_change(a, "level2".into()));
    // Nothing has changed yet: the next tick puts it in place.
    assert!(alive(a, "boxA") && !alive(a, "door"));
    let snap = ok(og_world_snapshot(a, None)).as_str().unwrap().to_string();
    frame(a);
    assert!(
        !alive(a, "boxA") && !alive(a, "lidA"),
        "the boot scene and its child go"
    );
    assert!(alive(a, "door"), "level2 spawned");
    assert_eq!(
        og_world_find(a, "hero".into()).map(|b| b as f64),
        hero.as_f64(),
        "the game's own entity stays"
    );
    assert_eq!(og_scene_current(a).as_deref(), Some("scenes/level2.ogt"));
    let state = ok(og_runtime::og_world_resource(a, "og.SceneState".into()));
    assert_eq!(state["changes"], json!(1), "{state}");
    assert_eq!(state["pending"], json!(""), "{state}");
    assert_ne!(state["root"], json!(0), "{state}");
    frame(a);
    let after = hash(a);

    // Another process restores the snapshot taken before the tick: the
    // change it asked for is in it, and happens at the same tick.
    let (b, _) = start(None);
    frame(b);
    ok(og_world_spawn(
        b,
        Some(json!({ "name": "hero" }).to_string()),
    ));
    ok(og_world_restore(b, snap, None));
    frame(b);
    assert!(alive(b, "door") && !alive(b, "boxA"));
    // The tick after a restore delivers og.WorldRestored, so its hash is
    // its own; the tick after it is the first run's.
    frame(b);
    assert_eq!(hash(b), after, "the same world as never interrupted");

    // And back: level2's root goes with its door, main comes back.
    ok(og_scene_change(a, "main".into()));
    frame(a);
    assert!(alive(a, "boxA") && alive(a, "lidA") && !alive(a, "door"));
    let state = ok(og_runtime::og_world_resource(a, "og.SceneState".into()));
    assert_eq!(state["changes"], json!(2), "{state}");
}

#[test]
fn a_world_that_never_changes_scene_keeps_its_hash_and_loading_reaches_the_host() {
    let (a, _) = start(None);
    let (b, net) = start(Some(
        "type project\nversion 1\n---\nname \"T\"\npublish {\n  loading game\n}\n",
    ));
    // Reading progress and the project's loading screen are not world
    // state. (A preload's stream requests are: og.StreamEvent, as for
    // og.stream.request.)
    ok(og_scene_progress(b, "level2".into()));
    frame(a);
    frame(b);
    assert_eq!(hash(a), hash(b));
    ok(og_scene_preload(b, "level2".into()));
    frame(b);

    assert_eq!(ok(Ok(og_loading_status(a)))["screen"], json!("engine"));
    assert!(
        og_loading_stage(
            b,
            Some(json!({ "id": "assets", "fraction": 1.5 }).to_string())
        )
        .is_err()
    );
    ok(og_loading_stage(
        b,
        Some(json!({ "id": "assets", "fraction": 0.5, "label": "Art" }).to_string()),
    ));
    let s = ok(Ok(og_loading_status(b)));
    assert_eq!(s["screen"], json!("game"));
    assert_eq!(
        (s["stage"].clone(), s["fraction"].clone(), s["done"].clone()),
        (json!("assets"), json!(0.5), json!(false))
    );
    ok(og_loading_done(b));
    ok(og_loading_done(b));
    assert_eq!(ok(Ok(og_loading_status(b)))["done"], json!(true));
    assert_eq!(
        net.borrow().loading,
        [
            ("assets".to_string(), 0.5, "Art".to_string()),
            ("done".to_string(), 1.0, String::new())
        ],
        "done reaches the host once"
    );
    frame(a);
    frame(b);
    assert_eq!(
        ok(og_stream_status(b, "art".into()))["requested"],
        json!(true)
    );
    let (c, _) = start(None);
    frame(c);
    ok(og_stream_request(c, "art".into()));
    frame(c);
    frame(c);
    assert_eq!(
        hash(b),
        hash(c),
        "a preload is its stream requests; loading is not world state"
    );
}

/// og.scene.current read by a system (the world is the step's then) names
/// the scene in place, from the tick og.SceneChanged arrives in: not the
/// boot scene it answered before (the loading guide's example found it).
#[test]
fn a_system_reads_the_scene_in_place() {
    let (e, net) = start_with(None, true);
    frame(e);
    ok(og_scene_change(e, "level2".into()));
    frame(e);
    frame(e);
    let main = Some("scenes/main.ogt".to_string());
    let level2 = Some("scenes/level2.ogt".to_string());
    let seen = net.borrow().current.clone();
    assert_eq!(seen.first(), Some(&main), "{seen:?}");
    assert_eq!(seen[seen.len() - 2..], [level2.clone(), level2], "{seen:?}");
}
