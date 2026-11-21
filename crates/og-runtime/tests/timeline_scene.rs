//! A timeline's `scene` track on the runtime (guides/animation.md §13):
//! from a key to the next its scene is in the world, under the
//! og.Timeline's entity, put in place and taken away between ticks.
//!
//! One project: `kitchen` (a stove with a pot on it), `yard` (a tree) and
//! `hall` (a door, with a streaming group of one palette whose pack the
//! host keeps pending until the test lets it land); `cut.ogt` has the
//! kitchen from 1 s, the yard from 2 s and nothing from 3 s; `hold.ogt` is
//! one second of the kitchen; `nest.ogt` nests `hold.ogt` from 1 s;
//! `hall.ogt` has the hall from 0, `shop.ogt` the shop, which places a
//! model. A second, separately cooked bundle
//! holds `annex` and a timeline of it, its scene blob pending likewise (a
//! runtime bundle's files are read on demand on the web).
//!
//! Ways this could fail: the scene placed or removed inside the tick (a
//! frame between shows neither scene, or both); a scene not removed at the
//! next key, at `none` or on stop; a seek inside a key placing it again; a
//! timeline that ends taking its scene with it; pre-roll placing the scene
//! (physics and the game's systems then see it early) or preparing
//! nothing (its model is decoded and its pipelines made in the frame that
//! first draws it); a snapshot between the decision and the tick losing it; a
//! scene whose file is on its way refused instead of the tick being held,
//! or placed at another tick than a host that has the file; the clock
//! running on while the scene cut to is still being read.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_advance, og_engine_create, og_engine_frame,
    og_engine_probe_dump, og_game_game, og_host_begin_bundle, og_load_bundle, og_load_timeline,
    og_scene_progress, og_timeline_play, og_timeline_seek, og_timeline_stop, og_world_find,
    og_world_get, og_world_get_tick, og_world_insert, og_world_restore, og_world_snapshot,
    og_world_spawn, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const HEAD: &str = "type timeline\nversion 1\n---\n";

fn scene_track(keys: &[(f32, &str)]) -> String {
    let mut out = String::from("track scene {\n");
    for (time, scene) in keys {
        let scene = match *scene {
            "none" => "none".to_string(),
            id => format!("\"{id}\""),
        };
        out += &format!("  key {time} {{\n    scene {scene}\n  }}\n");
    }
    out + "}\n"
}

fn write(root: &Path, path: &str, text: &str) {
    let p = root.join("assets").join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn cooked(name: &str, files: &[(&str, String)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "og-runtime-timeline-scene-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        write(&root, path, text);
    }
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

fn scene(name: &str, body: &str) -> String {
    format!("type scene\nversion 1\n---\nname \"{name}\"\n{body}")
}

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let timeline = |duration: f32, body: String| format!("{HEAD}duration {duration}\n{body}");
        cooked(
            "main",
            &[
                ("p/a.hex", "ff0000\n00ff00\n0000ff\n".into()),
                (
                    "scenes/kitchen.ogt",
                    scene(
                        "Kitchen",
                        "entity \"stove\" {\n  entity \"pot\" {\n  }\n}\n",
                    ),
                ),
                ("scenes/yard.ogt", scene("Yard", "entity \"tree\" {\n}\n")),
                (
                    "scenes/hall.ogt",
                    scene(
                        "Hall",
                        "stream \"art\" {\n  asset \"p/a.hex\"\n}\nentity \"door\" {\n}\n",
                    ),
                ),
                (
                    "timelines/cut.ogt",
                    timeline(
                        4.0,
                        scene_track(&[
                            (1.0, "scenes/kitchen.ogt"),
                            (2.0, "scenes/yard.ogt"),
                            (3.0, "none"),
                        ]),
                    ),
                ),
                (
                    "timelines/hold.ogt",
                    timeline(1.0, scene_track(&[(0.0, "scenes/kitchen.ogt")])),
                ),
                (
                    "timelines/nest.ogt",
                    timeline(
                        3.0,
                        "track timeline {\n  key 1 {\n    play \"timelines/hold.ogt\"\n  }\n}\n"
                            .into(),
                    ),
                ),
                (
                    "timelines/hall.ogt",
                    timeline(2.0, scene_track(&[(0.0, "scenes/hall.ogt")])),
                ),
                (
                    "models/block.ogt",
                    "type model\nversion 1\n---\nmaterial \"paint\" {\n  color 0.2 0.5 0.8\n}\npart \"block\" {\n  material \"paint\"\n  boxGeometry {\n    width 1\n  }\n}\n"
                        .into(),
                ),
                (
                    "scenes/shop.ogt",
                    scene(
                        "Shop",
                        "entity \"crate\" {\n  model \"models/block.ogt\"\n}\n",
                    ),
                ),
                (
                    "timelines/shop.ogt",
                    timeline(2.0, scene_track(&[(0.0, "scenes/shop.ogt")])),
                ),
            ],
        )
    })
}

/// A second bundle, opened at run time: a scene and a timeline of it.
fn annex() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        cooked(
            "annex",
            &[
                ("scenes/annex.ogt", scene("Annex", "entity \"desk\" {\n}\n")),
                (
                    "timelines/annex.ogt",
                    format!(
                        "{HEAD}duration 2\n{}",
                        scene_track(&[(0.0, "scenes/annex.ogt")])
                    ),
                ),
            ],
        )
    })
}

/// What the host keeps on its way: every streaming pack, and the file at
/// `held`, until `landed`.
#[derive(Default)]
struct Net {
    landed: bool,
    held: String,
}

struct Host(Rc<RefCell<Net>>);

impl Hooks for Host {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        let n = self.0.borrow();
        if !n.landed && (path.contains("/stream/") || path == n.held) {
            return Fetch::Pending;
        }
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

fn ok<T: og_runtime::wire::ToJson>(r: Result<T, String>) -> Value {
    value(og_runtime::host_impl::answer(r))
}

struct Run {
    engine: u32,
    /// The entity with the og.Timeline, and the timeline's handle.
    timeline: f64,
    asset: Value,
    net: Rc<RefCell<Net>>,
}

/// An engine whose host has everything (`landed`) or keeps the packs and
/// `held` on their way, with an entity `director` whose og.Timeline holds
/// `timeline` (stopped), one frame in.
fn start_on(timeline: &str, net: Net, bundle: Option<&Path>) -> Run {
    let net = Rc::new(RefCell::new(net));
    set_hooks(Rc::new(Host(net.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    if let Some(root) = bundle {
        ok(og_load_bundle(
            engine,
            root.join("cooked").to_str().unwrap().into(),
        ));
    }
    let asset = ok(og_load_timeline(engine, timeline.into()));
    let e = ok(og_world_spawn(
        engine,
        Some(json!({ "name": "director" }).to_string()),
    ))
    .as_f64()
    .unwrap();
    ok(og_world_insert(
        engine,
        e,
        "og.Timeline".into(),
        Some(json!({ "asset": asset }).to_string()),
    ));
    frame(engine, 1);
    Run {
        engine,
        timeline: e,
        asset,
        net,
    }
}

fn start(timeline: &str) -> Run {
    let net = Net {
        landed: true,
        held: String::new(),
    };
    start_on(timeline, net, None)
}

fn frame(engine: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

fn find(engine: u32, name: &str) -> Option<u64> {
    og_world_find(engine, name.into())
}

fn alive(r: &Run, names: &[&str]) -> Vec<bool> {
    names.iter().map(|n| find(r.engine, n).is_some()).collect()
}

fn time(r: &Run) -> f64 {
    ok(og_world_get(r.engine, r.timeline, "og.Timeline".into()))["time"]
        .as_f64()
        .unwrap()
}

fn parent(engine: u32, e: u64) -> u64 {
    ok(og_world_get(engine, e as f64, "og.Parent".into()))["entity"]
        .as_f64()
        .unwrap() as u64
}

/// Frames until the timeline's time is `seconds` (the tick that decides
/// what is in the world from the next).
fn play_to(r: &Run, seconds: f64) {
    for _ in 0..600 {
        if time(r) >= seconds - 1e-6 {
            return;
        }
        frame(r.engine, 1);
    }
    panic!("the timeline never reached {seconds} s (at {})", time(r));
}

fn hash(engine: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].clone()
}

const ALL: [&str; 3] = ["stove", "pot", "tree"];

/// Per frame drawn: the (mesh, material) pairs it was asked to make
/// pipelines for ahead of anything drawing them, and the instances drawn.
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

fn prepared(r: &Run, scene: &str) -> Value {
    ok(og_scene_progress(r.engine, scene.into()))["prepared"].clone()
}

fn set_preroll(r: &Run, preroll: bool) {
    ok(og_world_insert(
        r.engine,
        r.timeline,
        "og.Timeline".into(),
        Some(json!({ "asset": r.asset, "preroll": preroll }).to_string()),
    ));
}

#[test]
fn a_scene_is_in_the_world_from_its_key_to_the_next_and_cuts_between_ticks() {
    let r = start("timelines/cut.ogt");
    ok(og_timeline_play(r.engine, r.timeline));
    play_to(&r, 0.5);
    assert_eq!(
        alive(&r, &ALL),
        [false, false, false],
        "before the first key"
    );
    // The tick whose time reaches the key decides; the next starts with it.
    play_to(&r, 1.0);
    assert_eq!(alive(&r, &ALL), [false, false, false]);
    frame(r.engine, 1);
    assert_eq!(alive(&r, &ALL), [true, true, false]);
    // Under a root that is the og.Timeline's entity's child.
    let stove = find(r.engine, "stove").unwrap();
    let root = parent(r.engine, stove);
    assert_eq!(parent(r.engine, root), r.timeline as u64);
    assert_eq!(parent(r.engine, find(r.engine, "pot").unwrap()), stove);
    // A cut: the frame before has the kitchen whole, the next the yard
    // alone; none has neither or both.
    play_to(&r, 2.0);
    assert_eq!(alive(&r, &ALL), [true, true, false]);
    frame(r.engine, 1);
    assert_eq!(alive(&r, &ALL), [false, false, true]);
    assert!(
        !og_runtime::og_world_alive(r.engine, root as f64),
        "the kitchen's root went with it"
    );
    play_to(&r, 3.0);
    assert_eq!(alive(&r, &ALL), [false, false, true]);
    frame(r.engine, 1);
    assert_eq!(alive(&r, &ALL), [false, false, false], "`scene none`");

    // A seek lands where playing does, and one inside a key keeps the
    // scene that is there.
    let s = start("timelines/cut.ogt");
    ok(og_timeline_seek(s.engine, s.timeline, 2.5));
    frame(s.engine, 2);
    assert_eq!(alive(&s, &ALL), [false, false, true]);
    ok(og_timeline_seek(s.engine, s.timeline, 1.5));
    frame(s.engine, 2);
    assert_eq!(alive(&s, &ALL), [true, true, false]);
    let stove = find(s.engine, "stove").unwrap();
    ok(og_timeline_seek(s.engine, s.timeline, 1.2));
    frame(s.engine, 2);
    assert_eq!(find(s.engine, "stove"), Some(stove), "not placed again");
    ok(og_timeline_seek(s.engine, s.timeline, 0.5));
    frame(s.engine, 2);
    assert_eq!(alive(&s, &ALL), [false, false, false]);
}

#[test]
fn a_timeline_that_ends_keeps_its_scene_until_it_stops_and_a_nested_one_places_its_own() {
    let r = start("timelines/hold.ogt");
    ok(og_timeline_play(r.engine, r.timeline));
    frame(r.engine, 90);
    let t = ok(og_world_get(r.engine, r.timeline, "og.Timeline".into()));
    assert_eq!(
        (&t["playing"], t["time"].as_f64()),
        (&json!(false), Some(1.0))
    );
    assert_eq!(alive(&r, &ALL), [true, true, false], "kept past its end");
    let stove = find(r.engine, "stove").unwrap();
    // Stopped and played in one tick: the scene anew, never two of it.
    ok(og_timeline_stop(r.engine, r.timeline));
    ok(og_timeline_play(r.engine, r.timeline));
    frame(r.engine, 2);
    let again = find(r.engine, "stove").expect("placed again");
    assert_ne!(again, stove);
    assert!(!og_runtime::og_world_alive(r.engine, stove as f64));
    ok(og_timeline_stop(r.engine, r.timeline));
    frame(r.engine, 2);
    assert_eq!(alive(&r, &ALL), [false, false, false], "a stop takes it");

    let n = start("timelines/nest.ogt");
    ok(og_timeline_play(n.engine, n.timeline));
    play_to(&n, 0.5);
    assert_eq!(alive(&n, &ALL), [false, false, false]);
    play_to(&n, 1.0);
    frame(n.engine, 1);
    assert_eq!(alive(&n, &ALL), [true, true, false]);
    let root = parent(n.engine, find(n.engine, "stove").unwrap());
    assert_eq!(parent(n.engine, root), n.timeline as u64);
}

#[test]
fn pre_roll_prepares_the_scenes_and_places_none() {
    let r = start("timelines/cut.ogt");
    assert_eq!(prepared(&r, "scenes/kitchen.ogt"), json!(false));
    set_preroll(&r, true);
    frame(r.engine, 3);
    assert_eq!(alive(&r, &ALL), [false, false, false]);
    assert_eq!(prepared(&r, "scenes/kitchen.ogt"), json!(true));
    assert_eq!(prepared(&r, "scenes/yard.ogt"), json!(true));
    // No longer in pre-roll and not played: let go of.
    set_preroll(&r, false);
    frame(r.engine, 2);
    assert_eq!(prepared(&r, "scenes/kitchen.ogt"), json!(false));

    // A scene that places a model: pre-roll loads the model and has a
    // frame start its pipelines, with nothing of it in the world or drawn.
    let s = start("timelines/shop.ogt");
    let frames = Frames::default();
    set_render_backend(s.engine, Some(Box::new(Recorder(frames.clone())))).unwrap();
    frame(s.engine, 2);
    assert!(frames.borrow().iter().all(|f| *f == (0, 0)), "{frames:?}");
    set_preroll(&s, true);
    frame(s.engine, 3);
    assert!(find(s.engine, "crate").is_none());
    let seen: Vec<_> = std::mem::take(&mut *frames.borrow_mut());
    assert!(
        seen.iter().any(|f| f.0 > 0) && seen.iter().all(|f| f.1 == 0),
        "{seen:?}"
    );
    assert_eq!(prepared(&s, "scenes/shop.ogt"), json!(true));
    // The cut then only spawns: no frame is asked to prepare anything.
    ok(og_timeline_play(s.engine, s.timeline));
    frame(s.engine, 3);
    assert!(find(s.engine, "crate").is_some());
    let seen = frames.borrow().clone();
    assert!(seen.iter().all(|f| f.0 == 0), "{seen:?}");
    assert_eq!(seen.last().map(|f| f.1), Some(1), "{seen:?}");
}

#[test]
fn a_snapshot_carries_the_scene_in_place_and_the_one_due() {
    // Straight through, for the hashes.
    let a = start("timelines/cut.ogt");
    ok(og_timeline_play(a.engine, a.timeline));
    play_to(&a, 1.0);
    let decided = og_world_get_tick(a.engine);
    // A tick past the one that starts with it (the tick after a restore
    // carries og.WorldRestored, so that one's hash is its own).
    frame(a.engine, 2);
    let placed = hash(a.engine);
    frame(a.engine, 28);
    let mid = hash(a.engine);
    frame(a.engine, 60);
    let later = (og_world_get_tick(a.engine), hash(a.engine));
    assert_eq!(alive(&a, &ALL), [false, false, true]);

    let b = start("timelines/cut.ogt");
    ok(og_timeline_play(b.engine, b.timeline));
    play_to(&b, 1.0);
    assert_eq!(og_world_get_tick(b.engine), decided);
    let snap = |r: &Run| {
        ok(og_world_snapshot(r.engine, None))
            .as_str()
            .unwrap()
            .to_string()
    };
    // Decided, not yet in the world.
    let due = snap(&b);
    frame(b.engine, 30);
    assert_eq!(hash(b.engine), mid);
    let inside = snap(&b);
    frame(b.engine, 60);
    assert_eq!((og_world_get_tick(b.engine), hash(b.engine)), later);
    // Back inside the kitchen: it is there again, the yard gone.
    assert!(ok(og_world_restore(b.engine, inside, None)).is_null());
    assert_eq!(alive(&b, &ALL), [true, true, false]);
    assert_eq!(hash(b.engine), mid);
    frame(b.engine, 60);
    assert_eq!((og_world_get_tick(b.engine), hash(b.engine)), later);
    // Back to the decision: the next tick places it.
    assert!(ok(og_world_restore(b.engine, due, None)).is_null());
    assert_eq!(alive(&b, &ALL), [false, false, false]);
    frame(b.engine, 1);
    assert_eq!(alive(&b, &ALL), [true, true, false]);
    frame(b.engine, 1);
    assert_eq!(hash(b.engine), placed);
}

#[test]
fn the_clock_holds_while_the_scene_cut_to_is_still_being_read() {
    let r = start_on("timelines/hall.ogt", Net::default(), None);
    ok(og_timeline_play(r.engine, r.timeline));
    frame(r.engine, 1);
    let decided = og_world_get_tick(r.engine);
    // Its group's pack is on its way: no tick runs, the frames say so.
    for _ in 0..5 {
        let answer: Value =
            serde_json::from_str(&og_engine_frame(r.engine, 1000.0 / 60.0)).unwrap();
        assert_eq!(
            (&answer["ticks"], &answer["buffering"]),
            (&json!(0), &json!(true)),
            "{answer}"
        );
    }
    assert_eq!(og_world_get_tick(r.engine), decided);
    assert!(find(r.engine, "door").is_none());
    r.net.borrow_mut().landed = true;
    for _ in 0..600 {
        if find(r.engine, "door").is_some() {
            break;
        }
        frame(r.engine, 1);
    }
    // Placed as the very next tick started, however long it took.
    assert!(find(r.engine, "door").is_some());
    assert_eq!(og_world_get_tick(r.engine), decided + 1);
}

#[test]
fn a_tick_that_places_a_scene_is_held_for_its_file() {
    let held = {
        let bundle = og_bundle::runtime::Bundle::open(&annex().join("cooked")).unwrap();
        let file = &bundle.output("scenes/annex.ogt").unwrap().file;
        annex()
            .join("cooked")
            .join(file)
            .to_str()
            .unwrap()
            .to_string()
    };
    // A host that has the file, for the tick it is placed at and the hash.
    let net = Net {
        landed: true,
        held: held.clone(),
    };
    let a = start_on("timelines/annex.ogt", net, Some(annex()));
    ok(og_timeline_play(a.engine, a.timeline));
    frame(a.engine, 2);
    assert!(find(a.engine, "desk").is_some());
    let want = (og_world_get_tick(a.engine), hash(a.engine));

    let net = Net {
        landed: false,
        held,
    };
    let b = start_on("timelines/annex.ogt", net, Some(annex()));
    ok(og_timeline_play(b.engine, b.timeline));
    // Stepped as a script steps it, with no display frame to hold the
    // clock: the gate alone holds the tick.
    value(og_engine_advance(b.engine, 8));
    assert!(find(b.engine, "desk").is_none());
    assert_eq!(og_world_get_tick(b.engine), want.0 - 1, "held, not skipped");
    b.net.borrow_mut().landed = true;
    value(og_engine_advance(b.engine, 1));
    assert!(find(b.engine, "desk").is_some());
    assert_eq!((og_world_get_tick(b.engine), hash(b.engine)), want);
}
