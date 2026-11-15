//! og.film on the runtime: a dialogue of film nodes plays each node's
//! segment on a deck, opens and locks its choice window by the segment's
//! markers, cuts at `film.cut` to the segment of the node the choice (or
//! the default) leads to, through logic nodes, and warms that segment on
//! another deck first.
//!
//! One project: `start` (choices Door, Knife #default, Wait; window 0.3 s
//! to 0.7 s, cut 0.8 s) leads to `hall` (no choices, a 1 s segment) and
//! on through the logic node `branch` to `brave` or `shy` by `$brave`, or
//! straight to `wait`; each of the three ends the film.
//!
//! Ways this could fail: a timeout taking the first choice, not the one
//! marked #default; a choice outside the window accepted; the cut at the
//! segment's end instead of film.cut, or a tick late; a logic node held
//! like a film node (a segment-less stall) or its branch read before the
//! choice's `<<set>>`; the warm deck holding the default after a choice
//! (the cut then loads cold) or the cut not landing on the warm deck; a
//! restore inside the window losing the pending choice or the deck state
//! (its route or its hash then differs).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_commands,
    og_engine_probe_dump, og_film_choose, og_film_load, og_film_start, og_film_view, og_game_game,
    og_host_begin_bundle, og_load_timeline, og_story_parse, og_system_define_system,
    og_world_events, og_world_get, og_world_get_tick, og_world_restore, og_world_snapshot,
    set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const WATCH: u32 = 1;
const EVENTS: [&str; 3] = ["og.FilmSegment", "og.FilmChoice", "og.FilmEnded"];

fn segment(markers: &str) -> String {
    format!(
        "type timeline\nversion 1\n---\nduration 1\ntrack video {{\n  key 0 {{\n    video \"clips/a.ogt\"\n  }}\n}}\n{markers}"
    )
}

const WINDOW: &str = "track marker {\n  key 0.3 {\n    name \"film.open\"\n  }\n  key 0.7 {\n    name \"film.lock\"\n  }\n  key 0.8 {\n    name \"film.cut\"\n  }\n}\n";

fn script() -> String {
    format!(
        r#"#og-profile:{}
#og-name:film
title: start
og-film: segments/start.ogt
og-film-chapter: chapter.start
---
<<declare $brave = false>>
-> Door #line:door
    <<jump hall>>
-> Knife #line:knife #default
    <<set $brave = true>>
    <<jump hall>>
-> Wait #line:wait
    <<jump wait>>
===
title: hall
og-film: segments/hall.ogt
---
<<jump branch>>
===
title: branch
---
<<if $brave>>
    <<jump brave>>
<<else>>
    <<jump shy>>
<<endif>>
===
title: brave
og-film: segments/brave.ogt
og-film-ending: brave
---
===
title: shy
og-film: segments/shy.ogt
og-film-ending: shy
---
===
title: wait
og-film: segments/wait.ogt
og-film-ending: wait
---
===
"#,
        og_story::yarn::PROFILE
    )
}

/// The project, cooked once (None without an ffmpeg with AV1).
fn project() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        og_cook::cook::video::av1_encoder()?;
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-film-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let clips = root.join("assets/clips");
        std::fs::create_dir_all(&clips).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/engine/tests/fixtures/video/assets/clips/count.mp4"),
            clips.join("count.mp4"),
        )
        .unwrap();
        std::fs::write(
            clips.join("a.ogt"),
            "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize 128 72\naudio false\n",
        )
        .unwrap();
        let segments = root.join("assets/segments");
        std::fs::create_dir_all(&segments).unwrap();
        for (name, markers) in [
            ("start", WINDOW),
            ("hall", ""),
            ("brave", ""),
            ("shy", ""),
            ("wait", ""),
        ] {
            std::fs::write(segments.join(format!("{name}.ogt")), segment(markers)).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        Some(root)
    })
    .as_deref()
}

/// (tick, event name, event) of every film event of one engine.
type Seen = Rc<RefCell<Vec<(u32, String, Value)>>>;

struct Game(Seen);

impl Hooks for Game {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let w = call.engine;
            for name in EVENTS {
                let evs: Value = serde_json::from_str(&og_runtime::host_impl::json(
                    og_world_events(w, name.into()),
                ))
                .unwrap();
                for e in evs.as_array().into_iter().flatten() {
                    self.0
                        .borrow_mut()
                        .push((og_world_get_tick(w), name.to_string(), e.clone()));
                }
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

fn ok<T: og_runtime::wire::ToJson>(r: Result<T, String>) -> Value {
    value(og_runtime::host_impl::answer(r))
}

struct Run {
    engine: u32,
    film: f64,
    decks: u32,
    seen: Seen,
}

/// An engine with the dialogue loaded and prepared and the film started
/// on `decks` decks.
fn start(root: &Path, decks: u32) -> Run {
    let seen = Seen::default();
    set_hooks(Rc::new(Game(seen.clone())));
    og_host_begin_bundle();
    let reads: Vec<Value> = EVENTS
        .iter()
        .map(|n| json!({ "kind": "event", "name": n }))
        .collect();
    let watch = og_system_define_system(
        "Watch".into(),
        json!({ "reads": reads, "writes": [] }).to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [watch] }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    ok(og_story_parse(engine, script()));
    let info = ok(og_film_load(engine, "film".into()));
    assert_eq!(
        info["segments"],
        json!(["start", "hall", "brave", "shy", "wait"]),
        "{info}"
    );
    assert_eq!(info["endings"], json!(["brave", "shy", "wait"]), "{info}");
    let film = ok(og_film_start(
        engine,
        Some(json!({ "dialogue": "film", "decks": decks }).to_string()),
    ))
    .as_f64()
    .unwrap();
    Run {
        engine,
        film,
        decks,
        seen,
    }
}

fn frame(engine: u32, n: u32) {
    for _ in 0..n {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

fn view(r: &Run) -> Value {
    serde_json::from_str(&og_runtime::host_impl::json(og_film_view(r.engine, r.film))).unwrap()
}

/// Frames until the film ends (at most 10 s), and one more, so the watch
/// has read that tick's events.
fn to_end(r: &Run) {
    for _ in 0..600 {
        if view(r)["ended"] == json!(true) {
            frame(r.engine, 1);
            return;
        }
        frame(r.engine, 1);
    }
    panic!("the film never ended: {}", view(r));
}

/// The film nodes whose segments started, in order, with their ticks.
fn segments(r: &Run) -> Vec<(u32, String)> {
    r.seen
        .borrow()
        .iter()
        .filter(|(_, n, _)| n == "og.FilmSegment")
        .map(|(t, _, e)| (*t, e["node"].as_str().unwrap().to_string()))
        .collect()
}

fn route(r: &Run) -> Vec<String> {
    segments(r).into_iter().map(|(_, n)| n).collect()
}

fn events(r: &Run, name: &str) -> Vec<Value> {
    r.seen
        .borrow()
        .iter()
        .filter(|(_, n, _)| n == name)
        .map(|(_, _, e)| e.clone())
        .collect()
}

/// Each deck and the og.Timeline asset it holds (0: none). og.film.start
/// spawns the decks right after the film, on a fresh world the next
/// entity ids.
fn decks(r: &Run) -> Vec<(f64, u64)> {
    (1..=r.decks)
        .map(|k| {
            let e = r.film + f64::from(k);
            let d = ok(og_world_get(r.engine, e, "og.FilmDeck".into()));
            assert_eq!(
                d["film"].as_f64(),
                Some(r.film),
                "entity {e} is no deck: {d}"
            );
            let t = ok(og_world_get(r.engine, e, "og.Timeline".into()));
            (e, t["asset"].as_f64().unwrap_or(0.0) as u64)
        })
        .collect()
}

fn asset(r: &Run, name: &str) -> u64 {
    ok(og_load_timeline(r.engine, format!("segments/{name}.ogt")))
        .as_f64()
        .unwrap() as u64
}

fn current(r: &Run) -> f64 {
    ok(og_world_get(r.engine, r.film, "og.Film".into()))["current"]
        .as_f64()
        .unwrap()
}

fn hash(engine: u32) -> Value {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].clone()
}

#[test]
fn every_route_reaches_its_ending_at_the_segments_cuts() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    // Nothing chosen: the #default (Knife, index 1) sets $brave.
    let a = start(root, 3);
    to_end(&a);
    assert_eq!(route(&a), ["start", "hall", "brave"]);
    let chosen = events(&a, "og.FilmChoice");
    assert_eq!(chosen.len(), 1, "{chosen:?}");
    assert_eq!(chosen[0]["index"], json!(1), "the #default, not the first");
    assert_eq!(chosen[0]["timedOut"], json!(true));
    assert_eq!(events(&a, "og.FilmEnded")[0]["ending"], json!("brave"));
    assert_eq!(view(&a)["ending"], json!("brave"));
    assert_eq!(view(&a)["chapter"], json!("chapter.start"));
    // The cut at film.cut (0.8 s: 48 ticks at 60 Hz), not the end (60);
    // `hall` without markers cuts at its end; the logic node costs no tick.
    let ticks: Vec<u32> = segments(&a).into_iter().map(|(t, _)| t).collect();
    assert_eq!(
        [ticks[1] - ticks[0], ticks[2] - ticks[1]],
        [48, 60],
        "{ticks:?}"
    );

    // Door (index 0) through the API: $brave stays false.
    let b = start(root, 3);
    frame(b.engine, 10);
    assert!(
        view(&b)["window"].is_null(),
        "closed before film.open: {}",
        view(&b)
    );
    let early = og_runtime::host_impl::answer(og_film_choose(b.engine, b.film, 0));
    assert!(early.contains("no choice window"), "{early}");
    frame(b.engine, 15);
    let w = view(&b);
    assert_eq!(w["window"]["locked"], json!(false), "{w}");
    assert_eq!(w["window"]["defaultChoice"], json!(1), "{w}");
    assert_eq!(w["choices"].as_array().map(Vec::len), Some(3), "{w}");
    ok(og_film_choose(b.engine, b.film, 0));
    frame(b.engine, 1);
    assert_eq!(view(&b)["window"]["pending"], json!(0));
    to_end(&b);
    assert_eq!(route(&b), ["start", "hall", "shy"]);
    assert_eq!(events(&b, "og.FilmChoice")[0]["timedOut"], json!(false));

    // Wait (index 2) through the command channel, then a choice too late.
    let c = start(root, 3);
    frame(c.engine, 25);
    let s: Value = serde_json::from_str(&og_engine_probe_commands(
        c.engine,
        "0:film:choose:2".into(),
    ))
    .unwrap();
    assert_eq!(s["scheduled"], json!(1), "{s}");
    frame(c.engine, 20);
    assert_eq!(view(&c)["window"]["locked"], json!(true), "{}", view(&c));
    let late = og_runtime::host_impl::answer(og_film_choose(c.engine, c.film, 0));
    assert!(late.contains("locked"), "{late}");
    to_end(&c);
    assert_eq!(route(&c), ["start", "wait"]);
    assert_eq!(events(&c, "og.FilmEnded")[0]["ending"], json!("wait"));
}

#[test]
fn the_choice_warms_its_segment_and_the_cut_lands_on_that_deck() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    // Two decks: one plays, one warms: in the window the default's
    // segment, after a choice the chosen one.
    let r = start(root, 2);
    let (start_seg, hall, wait) = (asset(&r, "start"), asset(&r, "hall"), asset(&r, "wait"));
    frame(r.engine, 25);
    let playing = current(&r);
    let warm = |r: &Run| -> Vec<u64> {
        let cur = current(r);
        decks(r)
            .into_iter()
            .filter(|(e, _)| *e != cur)
            .map(|(_, a)| a)
            .collect()
    };
    assert!(
        decks(&r)
            .iter()
            .any(|(e, a)| *e == playing && *a == start_seg),
        "{:?}",
        decks(&r)
    );
    assert_eq!(warm(&r), [hall], "the default's segment warms first");
    ok(og_film_choose(r.engine, r.film, 2));
    frame(r.engine, 2);
    assert_eq!(warm(&r), [wait], "the chosen segment replaces it");
    let warm_deck = decks(&r)
        .into_iter()
        .find(|(e, _)| *e != playing)
        .unwrap()
        .0;
    frame(r.engine, 30);
    assert_eq!(view(&r)["node"], json!("wait"));
    assert_eq!(current(&r), warm_deck, "the cut plays the warm deck");
    let left = decks(&r).into_iter().find(|(e, _)| *e == playing).unwrap();
    assert_eq!(left.1, 0, "the deck it left is free");

    // Three decks warm every segment a choice leads to (hall, twice: once).
    let three = start(root, 3);
    frame(three.engine, 25);
    let mut held = warm(&three);
    held.sort();
    let mut want = vec![hall, wait];
    want.sort();
    assert_eq!(held, want);
}

#[test]
fn a_snapshot_inside_the_window_restores_the_pending_choice_and_the_decks() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let a = start(root, 3);
    frame(a.engine, 25);
    ok(og_film_choose(a.engine, a.film, 0));
    frame(a.engine, 1);
    let snap = ok(og_world_snapshot(a.engine, None))
        .as_str()
        .unwrap()
        .to_string();
    frame(a.engine, 90);
    let (tick, want) = (og_world_get_tick(a.engine), hash(a.engine));
    to_end(&a);
    assert_eq!(view(&a)["ending"], json!("shy"));

    // A fresh process: the dialogue loaded and prepared, the snapshot
    // restored over the film it started.
    let b = start(root, 3);
    ok(og_world_restore(b.engine, snap, None));
    frame(b.engine, 1);
    assert_eq!(view(&b)["window"]["pending"], json!(0), "{}", view(&b));
    frame(b.engine, tick - og_world_get_tick(b.engine));
    assert_eq!(hash(b.engine), want, "the same world as never interrupted");
    to_end(&b);
    assert_eq!(view(&b)["ending"], json!("shy"));
}
