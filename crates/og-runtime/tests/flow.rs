//! og.flow on the runtime: a dialogue of flow nodes plays each node's
//! segment on a deck, opens and locks its choice window by the segment's
//! markers, cuts at `flow.cut` to the segment of the node the choice (or
//! the default) leads to, through logic nodes, and warms that segment on
//! another deck first.
//!
//! One project: `start` (choices Door, Knife #default, Wait; window 0.3 s
//! to 0.7 s, cut 0.8 s) leads to `hall` (no choices, a 1 s segment) and
//! on through the logic node `branch` to `brave` or `shy` by `$brave`, or
//! straight to `wait`; each of the three ends the flow.
//!
//! Ways this could fail: a timeout taking the first choice, not the one
//! marked #default; a choice outside the window accepted; the cut at the
//! segment's end instead of flow.cut, or a tick late; a logic node held
//! like a flow node (a segment-less stall) or its branch read before the
//! choice's `<<set>>`; the warm deck holding the default after a choice
//! (the cut then loads cold) or the cut not landing on the warm deck; a
//! restore inside the window losing the pending choice or the deck state
//! (its route or its hash then differs).
//!
//! A second dialogue, `duel` (window 0.2 s to 0.8 s, cut 0.9 s), tries F2:
//! Dodge is `#qte:hold:dodge:200` (Space), Grab a `#hotspot:knife` that
//! moves across the window, Freeze the #default; Dodge's ending dissolves
//! in over 0.25 s. Ways those fail: a hold counted after it was let go, or
//! before the window; a hotspot offered outside the window or not moving;
//! a skip landing past the window's open, or skipping the default; the
//! dissolve's deck let go at the cut, never let go, or warmed over.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_commands,
    og_engine_probe_dump, og_flow_chart, og_flow_choose, og_flow_load, og_flow_skip, og_flow_start,
    og_flow_view, og_game_game, og_host_begin_bundle, og_input_define_action, og_load_timeline,
    og_story_parse, og_system_define_system, og_world_events, og_world_get, og_world_get_tick,
    og_world_restore, og_world_snapshot, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const WATCH: u32 = 1;
const EVENTS: [&str; 3] = ["og.FlowSegment", "og.FlowChoice", "og.FlowEnded"];

fn segment(markers: &str) -> String {
    format!(
        "type timeline\nversion 1\n---\nduration 1\ntrack video {{\n  key 0 {{\n    video \"clips/a.ogt\"\n  }}\n}}\n{markers}"
    )
}

const WINDOW: &str = "track marker {\n  key 0.3 {\n    name \"flow.open\"\n  }\n  key 0.7 {\n    name \"flow.lock\"\n  }\n  key 0.8 {\n    name \"flow.cut\"\n  }\n}\n";

const DUEL: &str = "track marker {\n  key 0.2 {\n    name \"flow.open\"\n  }\n  key 0.8 {\n    name \"flow.lock\"\n  }\n  key 0.9 {\n    name \"flow.cut\"\n  }\n}\ntrack hotspot {\n  key 0.2 {\n    id \"knife\"\n    rect 0.1 0.1 0.2 0.2\n  }\n  key 0.6 {\n    id \"knife\"\n    rect 0.5 0.1 0.2 0.2\n  }\n}\n";

fn duel() -> String {
    format!(
        r#"#og-profile:{}
#og-name:duel
title: duel
og-flow: segments/duel.ogt
---
-> Dodge #line:dodge #qte:hold:dodge:200
    <<jump dodged>>
-> Grab #line:grab #hotspot:knife
    <<jump grabbed>>
-> Freeze #line:freeze #default
    <<jump frozen>>
===
title: dodged
og-flow: segments/brave.ogt
og-flow-dissolve: 0.25
og-flow-ending: dodged
---
===
title: grabbed
og-flow: segments/shy.ogt
og-flow-ending: grabbed
---
===
title: frozen
og-flow: segments/wait.ogt
og-flow-ending: frozen
---
===
"#,
        og_story::yarn::PROFILE
    )
}

fn script() -> String {
    format!(
        r#"#og-profile:{}
#og-name:flow
title: start
og-flow: segments/start.ogt
og-flow-chapter: chapter.start
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
og-flow: segments/hall.ogt
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
og-flow: segments/brave.ogt
og-flow-ending: brave
---
===
title: shy
og-flow: segments/shy.ogt
og-flow-ending: shy
---
===
title: wait
og-flow: segments/wait.ogt
og-flow-ending: wait
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
            .join(format!("og-runtime-flow-{}", std::process::id()));
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
            ("duel", DUEL),
        ] {
            std::fs::write(segments.join(format!("{name}.ogt")), segment(markers)).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        Some(root)
    })
    .as_deref()
}

/// (tick, event name, event) of every flow event of one engine.
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
    flow: f64,
    decks: u32,
    seen: Seen,
}

/// An engine with the dialogue loaded and prepared and the flow started
/// on `decks` decks.
fn start(root: &Path, decks: u32) -> Run {
    start_in(root, decks, "flow")
}

fn start_in(root: &Path, decks: u32, dialogue: &str) -> Run {
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
    ok(og_story_parse(engine, duel()));
    ok(og_input_define_action(
        engine,
        Some(
            json!({ "name": "dodge", "kind": "button", "bindings": [{ "control": "Space" }] })
                .to_string(),
        ),
    ));
    let info = ok(og_flow_load(engine, dialogue.into()));
    if dialogue == "flow" {
        assert_eq!(
            info["segments"],
            json!(["start", "hall", "brave", "shy", "wait"]),
            "{info}"
        );
        assert_eq!(info["endings"], json!(["brave", "shy", "wait"]), "{info}");
    }
    let flow = ok(og_flow_start(
        engine,
        Some(json!({ "dialogue": dialogue, "decks": decks }).to_string()),
    ))
    .as_f64()
    .unwrap();
    Run {
        engine,
        flow,
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
    serde_json::from_str(&og_runtime::host_impl::json(og_flow_view(r.engine, r.flow))).unwrap()
}

/// Frames until the flow ends (at most 10 s), and one more, so the watch
/// has read that tick's events.
fn to_end(r: &Run) {
    for _ in 0..600 {
        if view(r)["ended"] == json!(true) {
            frame(r.engine, 1);
            return;
        }
        frame(r.engine, 1);
    }
    panic!("the flow never ended: {}", view(r));
}

/// The flow nodes whose segments started, in order, with their ticks.
fn segments(r: &Run) -> Vec<(u32, String)> {
    r.seen
        .borrow()
        .iter()
        .filter(|(_, n, _)| n == "og.FlowSegment")
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

/// Each deck and the og.Timeline asset it holds (0: none). og.flow.start
/// spawns the decks right after the flow, on a fresh world the next
/// entity ids.
fn decks(r: &Run) -> Vec<(f64, u64)> {
    (1..=r.decks)
        .map(|k| {
            let e = r.flow + f64::from(k);
            let d = ok(og_world_get(r.engine, e, "og.FlowDeck".into()));
            assert_eq!(
                d["flow"].as_f64(),
                Some(r.flow),
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
    ok(og_world_get(r.engine, r.flow, "og.Flow".into()))["current"]
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
    let chosen = events(&a, "og.FlowChoice");
    assert_eq!(chosen.len(), 1, "{chosen:?}");
    assert_eq!(chosen[0]["index"], json!(1), "the #default, not the first");
    assert_eq!(chosen[0]["timedOut"], json!(true));
    assert_eq!(events(&a, "og.FlowEnded")[0]["ending"], json!("brave"));
    assert_eq!(view(&a)["ending"], json!("brave"));
    assert_eq!(view(&a)["chapter"], json!("chapter.start"));
    // The cut at flow.cut (0.8 s: 48 ticks at 60 Hz), not the end (60);
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
        "closed before flow.open: {}",
        view(&b)
    );
    let early = og_runtime::host_impl::answer(og_flow_choose(b.engine, b.flow, 0));
    assert!(early.contains("no choice window"), "{early}");
    frame(b.engine, 15);
    let w = view(&b);
    assert_eq!(w["window"]["locked"], json!(false), "{w}");
    assert_eq!(w["window"]["defaultChoice"], json!(1), "{w}");
    assert_eq!(w["choices"].as_array().map(Vec::len), Some(3), "{w}");
    ok(og_flow_choose(b.engine, b.flow, 0));
    frame(b.engine, 1);
    assert_eq!(view(&b)["window"]["pending"], json!(0));
    to_end(&b);
    assert_eq!(route(&b), ["start", "hall", "shy"]);
    assert_eq!(events(&b, "og.FlowChoice")[0]["timedOut"], json!(false));

    // Wait (index 2) through the command channel, then a choice too late.
    let c = start(root, 3);
    frame(c.engine, 25);
    let s: Value = serde_json::from_str(&og_engine_probe_commands(
        c.engine,
        "0:flow:choose:2".into(),
    ))
    .unwrap();
    assert_eq!(s["scheduled"], json!(1), "{s}");
    frame(c.engine, 20);
    assert_eq!(view(&c)["window"]["locked"], json!(true), "{}", view(&c));
    let late = og_runtime::host_impl::answer(og_flow_choose(c.engine, c.flow, 0));
    assert!(late.contains("locked"), "{late}");
    to_end(&c);
    assert_eq!(route(&c), ["start", "wait"]);
    assert_eq!(events(&c, "og.FlowEnded")[0]["ending"], json!("wait"));
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
    ok(og_flow_choose(r.engine, r.flow, 2));
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
    ok(og_flow_choose(a.engine, a.flow, 0));
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
    // restored over the flow it started.
    let b = start(root, 3);
    ok(og_world_restore(b.engine, snap, None));
    frame(b.engine, 1);
    assert_eq!(view(&b)["window"]["pending"], json!(0), "{}", view(&b));
    frame(b.engine, tick - og_world_get_tick(b.engine));
    assert_eq!(hash(b.engine), want, "the same world as never interrupted");
    to_end(&b);
    assert_eq!(view(&b)["ending"], json!("shy"));
}

fn command(r: &Run, cmd: &str) {
    let s: Value =
        serde_json::from_str(&og_engine_probe_commands(r.engine, format!("0:{cmd}"))).unwrap();
    assert_eq!(s["scheduled"], json!(1), "{cmd}: {s}");
}

/// Frames until the window opens.
fn to_window(r: &Run) {
    for _ in 0..120 {
        if !view(r)["window"].is_null() {
            return;
        }
        frame(r.engine, 1);
    }
    panic!("the window never opened: {}", view(r));
}

#[test]
fn a_held_qte_takes_its_choice_and_letting_go_starts_it_again() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    // Held before the window: nothing counts until it opens.
    let a = start_in(root, 3, "duel");
    command(&a, "key_down:Space");
    frame(a.engine, 5);
    to_window(&a);
    let q = &view(&a)["qtes"][0];
    assert_eq!(
        (q["choice"].clone(), q["kind"].clone(), q["action"].clone()),
        (json!(0), json!("hold"), json!("dodge")),
        "{q}"
    );
    assert!(q["progress"].as_f64().unwrap() < 0.2, "{q}");
    // 200 ms is 12 ticks at 60 Hz.
    frame(a.engine, 6);
    let half = view(&a)["qtes"][0]["progress"].as_f64().unwrap();
    assert!((0.4..0.7).contains(&half), "{}", view(&a));
    assert!(view(&a)["window"]["pending"].is_null());
    frame(a.engine, 8);
    assert_eq!(view(&a)["qtes"][0]["done"], json!(true), "{}", view(&a));
    assert_eq!(view(&a)["window"]["pending"], json!(0));
    command(&a, "key_up:Space");
    to_end(&a);
    assert_eq!(route(&a), ["duel", "dodged"]);
    assert_eq!(events(&a, "og.FlowChoice")[0]["timedOut"], json!(false));

    // Let go after 8 ticks and held 8 more: never 12 in a row, the default.
    let b = start_in(root, 3, "duel");
    to_window(&b);
    command(&b, "key_down:Space");
    frame(b.engine, 9);
    command(&b, "key_up:Space");
    frame(b.engine, 2);
    assert_eq!(view(&b)["qtes"][0]["progress"], json!(0.0), "{}", view(&b));
    command(&b, "key_down:Space");
    frame(b.engine, 9);
    assert_eq!(view(&b)["qtes"][0]["done"], json!(false), "{}", view(&b));
    command(&b, "key_up:Space");
    to_end(&b);
    assert_eq!(route(&b), ["duel", "frozen"]);
    assert_eq!(events(&b, "og.FlowChoice")[0]["timedOut"], json!(true));
}

#[test]
fn a_hotspot_moves_with_its_keys_only_while_the_window_is_open() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let r = start_in(root, 3, "duel");
    frame(r.engine, 3);
    assert_eq!(view(&r)["hotspots"], json!([]), "closed: {}", view(&r));
    to_window(&r);
    frame(r.engine, 12);
    let v = view(&r);
    let h = &v["hotspots"];
    assert_eq!(h.as_array().map(Vec::len), Some(1), "{v}");
    assert_eq!(
        (h[0]["choice"].clone(), h[0]["id"].clone()),
        (json!(1), json!("knife"))
    );
    // x moves 0.1 → 0.5 from 0.2 s to 0.6 s: x = t − 0.1 between them.
    let t = v["time"].as_f64().unwrap();
    assert!((0.25..0.55).contains(&t), "{v}");
    let x = h[0]["x"].as_f64().unwrap();
    assert!((x - (t - 0.1)).abs() < 1e-4, "x {x} at {t}");
    assert!((h[0]["width"].as_f64().unwrap() - 0.2).abs() < 1e-6);
    while view(&r)["window"]["locked"] != json!(true) {
        assert!(
            !view(&r)["window"].is_null(),
            "the cut came before the lock: {}",
            view(&r)
        );
        frame(r.engine, 1);
    }
    let v = view(&r);
    assert_eq!(
        v["hotspots"],
        json!([]),
        "locked: no hotspot takes a choice"
    );
    // The hotspot's choice is taken as any choice is.
    let b = start_in(root, 3, "duel");
    to_window(&b);
    ok(og_flow_choose(b.engine, b.flow, 1));
    to_end(&b);
    assert_eq!(route(&b), ["duel", "grabbed"]);
}

#[test]
fn skip_goes_to_the_window_then_to_the_cut_and_takes_the_default() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let r = start_in(root, 3, "duel");
    frame(r.engine, 2);
    ok(og_flow_skip(r.engine, r.flow));
    frame(r.engine, 1);
    let v = view(&r);
    assert!(
        (v["time"].as_f64().unwrap() - 0.2).abs() < 0.02,
        "at flow.open: {v}"
    );
    assert!(!v["window"].is_null(), "{v}");
    let before = segments(&r).len();
    ok(og_flow_skip(r.engine, r.flow));
    frame(r.engine, 2);
    assert_eq!(segments(&r).len(), before + 1, "the cut came at once");
    assert_eq!(route(&r), ["duel", "frozen"]);
    assert_eq!(events(&r, "og.FlowChoice")[0]["timedOut"], json!(true));
    to_end(&r);
    let after = og_runtime::host_impl::answer(og_flow_skip(r.engine, r.flow));
    assert!(after.contains("ended"), "{after}");
}

#[test]
fn a_dissolve_plays_the_old_deck_under_the_new_and_lets_it_go() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let r = start_in(root, 3, "duel");
    to_window(&r);
    ok(og_flow_choose(r.engine, r.flow, 0));
    let left = current(&r);
    while route(&r).len() < 2 {
        frame(r.engine, 1);
    }
    let flow = ok(og_world_get(r.engine, r.flow, "og.Flow".into()));
    assert_eq!(flow["previous"].as_f64(), Some(left), "{flow}");
    // 0.25 s at 60 Hz.
    assert_eq!(flow["dissolve"], json!(15), "{flow}");
    let v = view(&r);
    let mix = v["mix"].as_f64().unwrap();
    assert!(mix < 0.2, "{v}");
    assert_ne!(v["previous"], json!(0), "the old picture: {v}");
    assert!(
        decks(&r)
            .iter()
            .any(|(e, a)| *e == left && *a == asset(&r, "duel")),
        "the old deck plays on: {:?}",
        decks(&r)
    );
    frame(r.engine, 7);
    let later = view(&r)["mix"].as_f64().unwrap();
    assert!(later > mix && later < 1.0, "{}", view(&r));
    frame(r.engine, 10);
    let v = view(&r);
    assert_eq!(
        (v["mix"].clone(), v["previous"].clone()),
        (json!(1.0), json!(0)),
        "{v}"
    );
    let flow = ok(og_world_get(r.engine, r.flow, "og.Flow".into()));
    assert_eq!(flow["previous"], json!(0), "{flow}");
    let freed = decks(&r).into_iter().find(|(e, _)| *e == left).unwrap();
    assert_eq!(freed.1, 0, "its deck is let go");
}

#[test]
fn the_chart_lays_flow_nodes_out_by_depth_through_logic_nodes() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let r = start(root, 3);
    let c = ok(og_flow_chart(r.engine, "flow".into()));
    let at: Vec<(String, u64, f64, f64)> = c["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["node"].as_str().unwrap().to_string(),
                n["depth"].as_u64().unwrap(),
                n["x"].as_f64().unwrap(),
                n["y"].as_f64().unwrap(),
            )
        })
        .collect();
    let want = [
        ("start", 0, 60.0, 60.0),
        ("hall", 1, 260.0, 60.0),
        ("brave", 2, 460.0, 60.0),
        ("shy", 2, 460.0, 180.0),
        ("wait", 1, 260.0, 180.0),
    ];
    assert_eq!(at, want.map(|(n, d, x, y)| (n.to_string(), d, x, y)), "{c}");
    assert_eq!(
        (c["width"].clone(), c["height"].clone()),
        (json!(520.0), json!(240.0))
    );
    let links: Vec<(String, String, i64)> = c["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["from"].as_str().unwrap().into(),
                l["to"].as_str().unwrap().into(),
                l["choice"].as_i64().unwrap(),
            )
        })
        .collect();
    let s = |a: &str, b: &str, i: i64| (a.to_string(), b.to_string(), i);
    // hall reaches brave and shy through the logic node branch.
    assert_eq!(
        links,
        [
            s("start", "hall", 0),
            s("start", "hall", 1),
            s("start", "wait", 2),
            s("hall", "brave", -1),
            s("hall", "shy", -1)
        ]
    );
    // start (60, 60) to wait (260, 180): a curve leaving and arriving
    // level, so a quarter along it is still near the start's row (a
    // straight line would be at (110, 90)).
    let p: Vec<f64> = c["links"][2]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(p.len(), 34, "{}", c["links"][2]);
    assert_eq!((p[0], p[1], p[32], p[33]), (60.0, 60.0, 260.0, 180.0));
    assert_eq!((p[8], p[9]), (119.375, 78.75));
    assert_eq!((p[16], p[17]), (160.0, 120.0));
}
