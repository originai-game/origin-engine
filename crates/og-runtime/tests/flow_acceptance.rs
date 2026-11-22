//! The film plan's acceptance (docs/design.md §2.9, guides/flow.md §9):
//! packages/engine/tests/fixtures/fused, one flow through a CG intro on
//! footage, a 3D physics level held for the game, an in-engine cutscene, a
//! 2D physics minigame (its scene switches og.Physics to profile 2) and a
//! live-action ending, or the lost ending from the level. The project
//! declares the flow (project.ogt `flow`), so the engine starts it; this
//! test takes each route `og flow routes` gives, with the commands its
//! steps name, standing in for the game's code (src/main.ts calls the same
//! og.flow.advance).
//!
//! At every cut, after boot: the deck cut to was warm with every scene its
//! segment places prepared, the engine modules the segment needs were
//! asked for before it, the frame of the cut makes no pipeline (the
//! renderer is asked to prepare nothing), and the tick advances on every
//! frame (the clock never holds for a scene still being read), and a live
//! segment shows no footage over its scene. A snapshot
//! taken while the level holds, restored in a fresh engine, ends on the
//! same tick and world hash as the run never interrupted.
//!
//! Ways this could fail: a warm deck that does not pre-roll a live
//! segment's scene (the cut then loads, decodes and compiles it in its
//! frame); a minigame's physics2d asked for only when it switches; the
//! held level's outcome decks let go of while it holds; a live segment on
//! a deck that played footage before showing that footage's last frame
//! over its scene; a route that
//! `og flow routes` lists and the runtime does not take; a restore that
//! loses the held node, the warm decks or the placed scene.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_commands, og_engine_probe_dump, og_engine_take_outputs, og_flow_advance,
    og_flow_choose, og_flow_current, og_flow_view, og_game_game, og_host_begin_bundle,
    og_world_get_tick, og_world_restore, og_world_snapshot, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// The fixture cooked into a directory of its own (None without an AV1
/// encoder, which its streamed footage needs).
fn project() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        og_cook::cook::video::av1_encoder()?;
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/engine/tests/fixtures/fused");
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-flow-acceptance-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        copy(&fixture.join("assets"), &root.join("assets"));
        std::fs::copy(fixture.join("project.ogt"), root.join("project.ogt")).unwrap();
        let options = og_cook::CookOptions {
            schemas: og_cook::cook::engine_schemas(),
            ..og_cook::CookOptions::default()
        };
        let report = og_cook::cook_project(&root, &options).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        Some(root)
    })
    .as_deref()
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &to.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

/// Every engine module asked for so far.
type Asked = Rc<RefCell<BTreeSet<String>>>;

struct Host(Asked);

impl Hooks for Host {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
    fn prefetch_modules(&self, modules: &[String]) {
        self.0.borrow_mut().extend(modules.iter().cloned());
    }
}

/// Per frame drawn: the pipelines it was asked to make ahead and the
/// instances it drew.
type Frames = Rc<RefCell<Vec<(usize, usize)>>>;

struct Recorder(Frames);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let o = &scene.objects;
        self.0
            .borrow_mut()
            .push((o.prepare.len(), o.instances.len()));
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

fn ok<T: og_runtime::wire::ToJson>(r: Result<T, String>) -> Value {
    value(og_runtime::host_impl::answer(r))
}

struct Run {
    engine: u32,
    flow: f64,
    asked: Asked,
    frames: Frames,
}

/// An engine of the fixture's project, its declared flow started.
fn start(root: &Path) -> Run {
    let asked = Asked::default();
    set_hooks(Rc::new(Host(asked.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let project = std::fs::read_to_string(root.join("project.ogt")).unwrap();
    let options = json!({ "root": root.to_str().unwrap(), "project": project }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let frames = Frames::default();
    set_render_backend(engine, Some(Box::new(Recorder(frames.clone())))).unwrap();
    let flow = og_flow_current(engine).expect("the declared flow is started") as f64;
    Run {
        engine,
        flow,
        asked,
        frames,
    }
}

impl Run {
    fn view(&self) -> Value {
        serde_json::from_str(&og_runtime::host_impl::json(og_flow_view(
            self.engine,
            self.flow,
        )))
        .unwrap()
    }
    fn hash(&self) -> Value {
        let d: Value = serde_json::from_str(&og_engine_probe_dump(self.engine, None)).unwrap();
        d["hash"].clone()
    }
}

/// The fixture's segments with no footage of their own.
const LIVE: [&str; 3] = ["level", "cutscene", "arcade"];

/// One frame of a sixtieth of a second, checked: a frame cutting to a node
/// is checked against the flow as it stood before the frame's tick (its
/// `dump_flow`, a command, runs between ticks).
fn frame(r: &Run, cuts: &mut Vec<String>) -> Value {
    let s: Value =
        serde_json::from_str(&og_engine_probe_commands(r.engine, "0:dump_flow".into())).unwrap();
    assert_eq!(s["scheduled"], json!(1), "{s}");
    let tick = og_world_get_tick(r.engine);
    let drawn = r.frames.borrow().len();
    value(og_engine_frame(r.engine, 1000.0 / 60.0));
    let outputs: Value = serde_json::from_str(&og_engine_take_outputs(r.engine)).unwrap();
    let dump = outputs
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["action"] == "dump_flow")
        .unwrap_or_else(|| panic!("no dump_flow: {outputs}"));
    let dump: Value = serde_json::from_str(dump["data"].as_str().unwrap()).unwrap();
    let before = dump["flows"][0].clone();
    assert_eq!(
        og_world_get_tick(r.engine),
        tick + 1,
        "the clock held at tick {tick} in {}",
        before["node"]
    );
    let view = r.view();
    let node = view["node"].as_str().unwrap_or_default().to_string();
    // A live segment (no footage of its own) shows the scene, never a
    // picture over it: least of all the one its deck played last.
    if LIVE.contains(&node.as_str()) {
        assert_eq!(view["texture"], json!(0), "{node}: {view}");
    }
    // The first segment starts at boot, behind the loading screen.
    if node == before["node"] || before["node"] == "" {
        return before;
    }
    cuts.push(node.clone());
    let deck = before["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["node"] == node.as_str() && d["role"] == "warm")
        .unwrap_or_else(|| panic!("the cut to {node} had no warm deck: {before:#}"));
    for s in deck["scenes"].as_array().unwrap() {
        assert_eq!(
            s["prepared"], "yes",
            "the cut to {node}: scene {s} was not prepared"
        );
    }
    let asked = r.asked.borrow();
    for m in deck["modules"].as_array().unwrap() {
        assert!(
            asked.contains(m.as_str().unwrap()),
            "the cut to {node}: module {m} was not asked for before it (asked {asked:?})"
        );
    }
    let frames = r.frames.borrow();
    let made: Vec<_> = frames[drawn..].iter().map(|f| f.0).collect();
    assert!(
        made.iter().all(|&n| n == 0),
        "the cut to {node} made pipelines in its frame: {made:?}"
    );
    drop(frames);
    drop(asked);
    before
}

/// The nodes the spare decks hold warm, as the flow stood before a frame.
fn warm(flow: &Value) -> Vec<String> {
    flow["decks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["role"] == "warm")
        .map(|d| d["node"].as_str().unwrap().to_string())
        .collect()
}

/// Frames until `node` plays and its step can be taken (a window open, or
/// a held segment), at most 10 s.
fn until(r: &Run, node: &str, held: bool, cuts: &mut Vec<String>) {
    for _ in 0..600 {
        let v = r.view();
        let open = v["window"].is_object() && v["window"]["locked"] == json!(false);
        if v["node"] == node && (if held { v["held"] == json!(true) } else { open }) {
            return;
        }
        frame(r, cuts);
    }
    panic!("{node} never took its step: {}", r.view());
}

/// Takes `step` of a route (`og flow routes`) from where the run is.
fn take(r: &Run, step: &Value, cuts: &mut Vec<String>) {
    let node = step["node"].as_str().unwrap();
    if let Some(i) = step["choose"].as_u64() {
        until(r, node, false, cuts);
        ok(og_flow_choose(r.engine, r.flow, i as u32));
    } else if let Some(o) = step["advance"].as_str() {
        until(r, node, true, cuts);
        ok(og_flow_advance(r.engine, r.flow, Some(o.into())));
    }
}

/// Frames until the flow ends (at most 10 s).
fn to_end(r: &Run, cuts: &mut Vec<String>) {
    for _ in 0..600 {
        if r.view()["ended"] == json!(true) {
            return;
        }
        frame(r, cuts);
    }
    panic!("the flow never ended: {}", r.view());
}

#[test]
fn every_route_cuts_onto_a_prepared_segment_and_a_restore_keeps_the_hash() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let facts = og_cook::cook::flow::routes(root).unwrap();
    let flow = &facts["flows"][0];
    assert_eq!(flow["unreached"], json!([]), "{facts:#}");
    let routes = flow["routes"].as_array().unwrap();
    let ends: Vec<&str> = routes.iter().map(|r| r["end"].as_str().unwrap()).collect();
    assert_eq!(ends, ["ending", "lost"], "{facts:#}");
    for route in routes {
        let r = start(root);
        let mut cuts = Vec::new();
        for step in route["steps"].as_array().unwrap() {
            take(&r, step, &mut cuts);
        }
        to_end(&r, &mut cuts);
        // An ended flow holds nothing warm: no segment is left to cut to.
        let after = frame(&r, &mut cuts);
        assert_eq!(warm(&after), Vec::<String>::new(), "{after:#}");
        let nodes: Vec<&str> = route["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        assert_eq!(cuts, nodes[1..], "{route}");
        assert_eq!(r.view()["ending"], route["ending"], "{}", r.view());
        assert!(
            r.frames.borrow().iter().any(|f| f.0 > 0),
            "pre-roll made the live segments' pipelines ahead, in frames before their cuts"
        );
        if nodes.contains(&"arcade") {
            assert!(
                r.asked.borrow().contains("physics2d"),
                "{:?}",
                r.asked.borrow()
            );
        }
    }

    // The long route with a snapshot taken while the level holds: the run
    // goes on, and the snapshot is restored into a fresh engine that takes
    // the same steps from the same tick.
    let steps = routes[0]["steps"].as_array().unwrap();
    let first = start(root);
    let mut cuts = Vec::new();
    take(&first, &steps[0], &mut cuts);
    until(&first, "level", true, &mut cuts);
    let save = ok(og_world_snapshot(first.engine, None))
        .as_str()
        .unwrap()
        .to_string();
    let restored = start(root);
    ok(og_world_restore(restored.engine, save, None));
    for r in [&first, &restored] {
        let mut cuts = Vec::new();
        frame(r, &mut cuts);
        assert_eq!(r.view()["node"], json!("level"), "{}", r.view());
        assert_eq!(r.view()["held"], json!(true), "{}", r.view());
        take(r, &steps[1], &mut cuts);
        until(r, "arcade", true, &mut cuts);
        // The level's other outcome is let go once it was won.
        let held = frame(r, &mut cuts);
        assert_eq!(warm(&held), ["ending"], "{held:#}");
        take(r, &steps[steps.len() - 1], &mut cuts);
        to_end(r, &mut cuts);
        assert_eq!(cuts, ["cutscene", "arcade", "ending"]);
    }
    assert_eq!(
        og_world_get_tick(restored.engine),
        og_world_get_tick(first.engine)
    );
    assert!(first.hash().is_string(), "{}", first.hash());
    assert_eq!(
        restored.hash(),
        first.hash(),
        "the same world as never interrupted"
    );
}
