//! End-to-end commercial interactive-film & vertical short-drama acceptance
//! and snapshot/restore determinism tests for `og.flow` + `og.timeline` +
//! `og.story` + `og_cook`.
//!
//! Guards:
//! - Paused opening start (`paused: true`) holds frame 0 without firing events
//!   until `og.flow.play`.
//! - WebVTT rich captions (`<v Speaker>`, `<i>`, `<b>`, `<font color>`,
//!   `line:0%` top position) surface through `og.timeline.captions` and
//!   `og.flow.history`.
//! - Seamless idle choice loops (`flow.loopStart`..`flow.loopEnd`) keep the
//!   window open (`looping: true`, `window.held: true`) until chosen.
//! - Non-cutting `inspect` hotspots (`shape circle`, `kind inspect`) emit
//!   `og.FlowInspected` and flip `inspected: true` without cutting.
//! - `og.flow.setSpeed`, `og.flow.seek`, `#immediate` cut, `og-flow-dissolve`
//!   caption suppression & mix progress, persistent `og-flow-music` +
//!   `og.flow.setMusic`, `og.flow.checkpoint` + `og.flow.jump`, swipe/drag QTE
//!   via `og.flow.gesture` emitting `og.FlowQteResult`, `decks: 1` self-loop,
//!   and `og.flow.chart` metadata + barycenter layout.
//! - `og_cook::flow::routes` cycle detection & unreachable node reporting.
//! - Mid-window snapshot & restore produces identical `tick`, `world_hash`,
//!   `FlowView`, and transcript `history` against an uninterrupted run.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump, og_flow_chart,
    og_flow_checkpoint, og_flow_choose, og_flow_gesture, og_flow_history, og_flow_inspect,
    og_flow_jump, og_flow_load, og_flow_play, og_flow_seek, og_flow_set_music, og_flow_set_speed,
    og_flow_start, og_flow_view, og_game_game, og_host_begin_bundle, og_story_load,
    og_system_define_system, og_timeline_captions, og_world_events, og_world_get_tick,
    og_world_restore, og_world_snapshot, set_hooks,
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
            .join(format!("og-runtime-flow-accept-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |path: &str, bytes: &str| {
            let p = root.join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, bytes).unwrap();
        };

        write(
            "project.ogt",
            "type project\nversion 1\n---\nname \"FlowAccept\"\n",
        );
        write(
            "assets/scenes/room.ogt",
            "type scene\nversion 1\n---\nentity \"stage\" {\n  og.Transform {\n    position 0 0 0\n  }\n}\n",
        );

        let pcm: Vec<f32> = (0..12_000)
            .map(|i| og_core::math::sin(i as f32 * 0.05) * 0.2)
            .collect();
        let bgm = root.join("assets/bgm/theme.wav");
        std::fs::create_dir_all(bgm.parent().unwrap()).unwrap();
        std::fs::write(bgm, og_audio::wav(&pcm, 1, 48_000)).unwrap();

        write(
            "assets/subs/intro.vtt",
            "WEBVTT\n\n\
             00:00.000 --> 00:01.000 line:0%\n\
             <v Lin><i>Watch</i> <font color=\"#ffdd66\"><b>closely</b></font>!\n",
        );

        // Intro: idle choice loop 0.3..0.6, circle inspect hotspot, choice hotspot.
        write(
            "assets/timelines/intro.ogt",
            "type timeline\nversion 1\n---\nname \"Intro\"\nduration 2\n\
             track scene {\n  key 0 {\n    scene \"scenes/room.ogt\"\n  }\n}\n\
             track caption {\n  captions \"subs/intro.vtt\"\n}\n\
             track marker {\n\
               key 0.1 {\n    name \"flow.open\"\n  }\n\
               key 0.25 {\n    name \"flow.loopStart\"\n  }\n\
               key 0.5 {\n    name \"flow.loopEnd\"\n  }\n\
               key 1.4 {\n    name \"flow.lock\"\n  }\n\
               key 1.8 {\n    name \"flow.cut\"\n  }\n\
             }\n\
             track hotspot {\n\
               key 0 {\n    id \"clue_letter\"\n    kind inspect\n    shape circle\n    label \"Letter\"\n    rect 0.2 0.3 0.1 0.1\n  }\n\
               key 0.1 {\n    id \"door\"\n    kind choice\n    shape rect\n    rect 0.6 0.2 0.2 0.4\n  }\n\
             }\n",
        );

        // QTE scene: swipe + drag + default.
        write(
            "assets/timelines/qte.ogt",
            "type timeline\nversion 1\n---\nname \"Qte\"\nduration 1.5\n\
             track scene {\n  key 0 {\n    scene \"scenes/room.ogt\"\n  }\n}\n\
             track marker {\n\
               key 0.05 {\n    name \"flow.open\"\n  }\n\
               key 1.1 {\n    name \"flow.lock\"\n  }\n\
               key 1.3 {\n    name \"flow.cut\"\n  }\n\
             }\n",
        );

        // Dissolve scene: 0.3s dissolve, caption, chapter.
        write(
            "assets/timelines/dissolve.ogt",
            "type timeline\nversion 1\n---\nname \"Dissolve\"\nduration 1.2\n\
             track scene {\n  key 0 {\n    scene \"scenes/room.ogt\"\n  }\n}\n\
             track caption {\n  key 0 {\n    end 1\n    text \"Fresh scene line\"\n  }\n}\n",
        );

        // Ending scene.
        write(
            "assets/timelines/ending.ogt",
            "type timeline\nversion 1\n---\nname \"Ending\"\nduration 0.5\n\
             track scene {\n  key 0 {\n    scene \"scenes/room.ogt\"\n  }\n}\n",
        );

        // Short segment for decks:1 self-loop check.
        write(
            "assets/timelines/loop_one.ogt",
            "type timeline\nversion 1\n---\nname \"LoopOne\"\nduration 0.4\n\
             track scene {\n  key 0 {\n    scene \"scenes/room.ogt\"\n  }\n}\n\
             track marker {\n  key 0.3 {\n    name \"flow.cut\"\n  }\n}\n",
        );

        write(
            "assets/story/film.yarn",
            "#og-profile:yarn-spinner-3.2.2\n\
             #og-name:film\n\
             title: Intro\n\
             og-flow: timelines/intro.ogt\n\
             og-flow-title: Prologue\n\
             og-flow-music: bgm/theme.wav volume=0.8 fade=0.1 duck=0.35\n\
             ---\n\
             <<declare $trust = 1>>\n\
             -> Rush out #line:rush #immediate\n\
                 <<set $trust = 2>>\n\
                 <<jump DissolveNext>>\n\
             -> Inspect door #line:door #hotspot:door #default\n\
                 <<jump QteScene>>\n\
             ===\n\
             title: QteScene\n\
             og-flow: timelines/qte.ogt\n\
             og-flow-title: Rooftop\n\
             ---\n\
             -> Leap up #line:leap #qte:swipe:up #immediate\n\
                 <<jump DissolveNext>>\n\
             -> Pull lever #line:pull #qte:drag:right\n\
                 <<jump Ending>>\n\
             -> Wait #line:wait #default\n\
                 <<jump Ending>>\n\
             ===\n\
             title: DissolveNext\n\
             og-flow: timelines/dissolve.ogt\n\
             og-flow-title: Chapter Two\n\
             og-flow-chapter: ch2\n\
             og-flow-dissolve: 0.3\n\
             og-flow-music: bgm/theme.wav volume=0.7 fade=0.1 duck=0.35\n\
             ---\n\
             <<jump Ending>>\n\
             ===\n\
             title: Ending\n\
             og-flow: timelines/ending.ogt\n\
             og-flow-ending: true_end\n\
             og-flow-music: none fade=0.1\n\
             ---\n\
             ===\n\
             title: LoopOne\n\
             og-flow: timelines/loop_one.ogt\n\
             ---\n\
             <<jump LoopOne>>\n\
             ===\n",
        );

        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

#[derive(Default)]
struct Events {
    segments: Vec<String>,
    chapters: Vec<String>,
    inspected: Vec<(String, String)>,
    qte_results: Vec<(String, u32, String)>,
    ended: Vec<String>,
}

struct TestHost(Rc<RefCell<Events>>);

impl Hooks for TestHost {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == WATCH {
            let read_ev = |name: &str| -> Vec<Value> {
                serde_json::from_str::<Value>(&og_runtime::host_impl::json(og_world_events(
                    call.engine,
                    name.into(),
                )))
                .ok()
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default()
            };
            let mut ev = self.0.borrow_mut();
            for e in read_ev("og.FlowSegment") {
                ev.segments
                    .push(e["node"].as_str().unwrap_or_default().to_string());
            }
            for e in read_ev("og.FlowChapter") {
                ev.chapters
                    .push(e["chapter"].as_str().unwrap_or_default().to_string());
            }
            for e in read_ev("og.FlowInspected") {
                ev.inspected.push((
                    e["node"].as_str().unwrap_or_default().to_string(),
                    e["hotspot"].as_str().unwrap_or_default().to_string(),
                ));
            }
            for e in read_ev("og.FlowQteResult") {
                ev.qte_results.push((
                    e["node"].as_str().unwrap_or_default().to_string(),
                    e["choice"].as_u64().unwrap_or(0) as u32,
                    e["grade"].as_str().unwrap_or_default().to_string(),
                ));
            }
            for e in read_ev("og.FlowEnded") {
                ev.ended
                    .push(e["ending"].as_str().unwrap_or_default().to_string());
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

fn ticks(engine: u32, n: u32) {
    let goal = og_world_get_tick(engine) + n;
    while og_world_get_tick(engine) < goal {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
}

fn setup_engine(events: Rc<RefCell<Events>>) -> u32 {
    set_hooks(Rc::new(TestHost(events)));
    og_host_begin_bundle();
    let watch = og_system_define_system(
        "WatchFlow".into(),
        json!({
            "reads": [
                { "kind": "event", "name": "og.FlowSegment" },
                { "kind": "event", "name": "og.FlowChapter" },
                { "kind": "event", "name": "og.FlowInspected" },
                { "kind": "event", "name": "og.FlowQteResult" },
                { "kind": "event", "name": "og.FlowEnded" }
            ],
            "writes": []
        })
        .to_string(),
        WATCH,
    );
    let game = og_game_game(Some(json!({ "seed": 17, "systems": [watch] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    og_story_load(engine, "story/film.yarn".into()).unwrap();
    og_flow_load(engine, "film".into()).unwrap();
    engine
}

fn world_hash(engine: u32) -> String {
    let dump: Value = serde_json::from_str(&og_engine_probe_dump(engine, None)).unwrap();
    dump["hash"].as_str().expect("world hash").to_string()
}

#[test]
fn commercial_flow_mechanisms_routes_and_fixes() {
    // 1. Cook route graph detects orphan LoopOne node cleanly.
    let routes = og_cook::cook::flow::routes(project()).unwrap();
    let film = &routes["flows"][0];
    let unreachable: Vec<&str> = film["unreachableNodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        unreachable.contains(&"LoopOne"),
        "expected LoopOne in unreachableNodes: {routes}"
    );
    assert!(film["unreached"].as_array().unwrap().is_empty());

    let events = Rc::new(RefCell::new(Events::default()));
    let engine = setup_engine(events.clone());

    // 2. Flowchart metadata & barycenter layout.
    let chart = og_flow_chart(engine, "film".into()).unwrap();
    let intro_node = chart.nodes.iter().find(|n| n.node == "Intro").unwrap();
    assert_eq!(intro_node.title, "Prologue");
    assert_eq!(intro_node.kind, "live");
    assert_eq!(intro_node.depth, 0);
    let dissolve_node = chart
        .nodes
        .iter()
        .find(|n| n.node == "DissolveNext")
        .unwrap();
    assert_eq!(dissolve_node.title, "Chapter Two");
    assert_eq!(dissolve_node.chapter, "ch2");
    assert!(!chart.links.is_empty());

    // 3. Paused opening start holds frame 0 without firing FlowSegment.
    let flow = og_flow_start(
        engine,
        Some(json!({ "dialogue": "film", "paused": true }).to_string()),
    )
    .unwrap()
    .as_f64()
    .unwrap();
    ticks(engine, 5);
    let v0 = og_flow_view(engine, flow).unwrap();
    assert!(v0.paused);
    assert_eq!(v0.time, 0.0);
    assert!(events.borrow().segments.is_empty());

    // 4. Play starts Intro, surfaces rich WebVTT caption + transcript history.
    og_flow_play(engine, flow).unwrap();
    ticks(engine, 2);
    assert_eq!(events.borrow().segments, vec!["Intro"]);
    let caps = og_timeline_captions(engine);
    assert_eq!(caps.len(), 1, "expected WebVTT cue: {caps:?}");
    assert_eq!(caps[0].speaker, "Lin");
    assert_eq!(caps[0].position, "top");
    assert!(
        caps[0].spans.len() >= 2,
        "expected rich spans: {:?}",
        caps[0].spans
    );
    let hist = og_flow_history(engine, flow);
    assert_eq!(hist.len(), 1);
    assert_eq!(hist[0].speaker, "Lin");
    assert!(!hist[0].chosen);

    // 5. Idle choice loop (`flow.loopStart`..`flow.loopEnd`) holds window open
    //    well past normal loopEnd without advancing to cut.
    ticks(engine, 55); // ~0.92s > loopEnd (0.5s)
    let v_loop = og_flow_view(engine, flow).unwrap();
    assert_eq!(v_loop.node, "Intro");
    assert!(v_loop.looping, "expected idle choice loop: {v_loop:?}");
    assert!(
        v_loop.time >= 0.24 && v_loop.time <= 0.55,
        "time={}",
        v_loop.time
    );
    assert!(v_loop.window.as_ref().unwrap().held);

    // 6. Inspect circle hotspot emits og.FlowInspected without cutting.
    og_flow_inspect(engine, flow, "clue_letter".into()).unwrap();
    ticks(engine, 2);
    assert_eq!(
        events.borrow().inspected,
        vec![("Intro".into(), "clue_letter".into())]
    );
    let v_insp = og_flow_view(engine, flow).unwrap();
    assert_eq!(v_insp.node, "Intro");
    let clue = v_insp
        .hotspots
        .iter()
        .find(|h| h.id == "clue_letter")
        .unwrap();
    assert_eq!(clue.kind, "inspect");
    assert_eq!(clue.shape, "circle");
    assert_eq!(clue.choice, -1);
    assert!(clue.inspected);

    // 7. setSpeed + seek + checkpoint.
    og_flow_set_speed(engine, flow, 2.0).unwrap();
    og_flow_seek(engine, flow, 0.2).unwrap();
    ticks(engine, 1);
    let v_seek = og_flow_view(engine, flow).unwrap();
    assert_eq!(v_seek.speed, 2.0);
    assert!((v_seek.time - 0.2).abs() < 0.06, "time={}", v_seek.time);
    let cp = og_flow_checkpoint(engine, flow).unwrap();
    assert_eq!(cp.node, "Intro");
    assert_eq!(cp.vars["trust"], json!(1.0));

    // 8. `#immediate` choice cuts immediately into `DissolveNext` with dissolve
    //    mix < 1, suppresses outgoing Intro captions, and emits FlowChapter.
    og_flow_choose(engine, flow, 0).unwrap();
    ticks(engine, 2);
    let v_dis = og_flow_view(engine, flow).unwrap();
    assert_eq!(v_dis.node, "DissolveNext");
    assert!(
        v_dis.mix < 1.0,
        "expected active dissolve mix: {}",
        v_dis.mix
    );
    assert_eq!(events.borrow().chapters, vec!["ch2"]);
    let dis_caps = og_timeline_captions(engine);
    assert!(
        dis_caps.iter().all(|c| c.text == "Fresh scene line"),
        "outgoing Intro captions must be suppressed during dissolve: {dis_caps:?}"
    );
    assert!(
        og_flow_history(engine, flow).iter().any(|l| l.chosen),
        "chosen option must be recorded in history"
    );

    // 9. Jump directly to `QteScene` and trigger swipe QTE via `og.flow.gesture`.
    og_flow_jump(
        engine,
        flow,
        Some(json!({ "node": "QteScene", "vars": cp.vars }).to_string()),
    )
    .unwrap();
    ticks(engine, 6); // enter QteScene window (`flow.open` = 0.05s)
    let v_qte = og_flow_view(engine, flow).unwrap();
    assert_eq!(v_qte.node, "QteScene");
    assert!(
        v_qte
            .qtes
            .iter()
            .any(|q| q.kind == "swipe" && q.direction == "up")
    );
    og_flow_gesture(
        engine,
        flow,
        Some(json!({ "kind": "swipe", "direction": "up", "progress": 1.0 }).to_string()),
    )
    .unwrap();
    ticks(engine, 2);
    assert!(
        events
            .borrow()
            .qte_results
            .iter()
            .any(|(n, c, g)| n == "QteScene" && *c == 0 && g == "perfect"),
        "expected QteResult perfect: {:?}",
        events.borrow().qte_results
    );
    // Swipe choice is `#immediate` -> already cut into `DissolveNext`.
    assert_eq!(og_flow_view(engine, flow).unwrap().node, "DissolveNext");

    // 10. `decks: 1` self-loop (`LoopOne` -> `LoopOne`) resets time to 0 instead of stalling.
    let loop_flow = og_flow_start(
        engine,
        Some(json!({ "dialogue": "film", "node": "LoopOne", "decks": 1 }).to_string()),
    )
    .unwrap()
    .as_f64()
    .unwrap();
    ticks(engine, 25); // 0.42s > cut (0.3s)
    let loop_count = events
        .borrow()
        .segments
        .iter()
        .filter(|n| n.as_str() == "LoopOne")
        .count();
    assert!(
        loop_count >= 2,
        "decks:1 self-loop must restart segment instead of stalling (seen {loop_count})"
    );
    let v_lo = og_flow_view(engine, loop_flow).unwrap();
    assert!(v_lo.time < 0.25, "expected wrapped time, got {}", v_lo.time);
}

#[test]
fn flow_snapshot_restore_determinism() {
    let ev_a = Rc::new(RefCell::new(Events::default()));
    let a = setup_engine(ev_a);
    let flow = og_flow_start(a, Some(json!({ "dialogue": "film" }).to_string()))
        .unwrap()
        .as_f64()
        .unwrap();

    // Advance into Intro's idle loop, inspect clue, set speed & music, queue choice.
    ticks(a, 24);
    og_flow_inspect(a, flow, "clue_letter".into()).unwrap();
    og_flow_set_speed(a, flow, 1.5).unwrap();
    og_flow_set_music(
        a,
        flow,
        Some(
            json!({ "sound": "bgm/theme.wav", "volume": 0.6, "fade": 0.2, "duck": 0.5 })
                .to_string(),
        ),
    )
    .unwrap();
    ticks(a, 2);
    og_flow_choose(a, flow, 0).unwrap();

    // Snapshot mid-window with queued op, inspected clue, history, and persistent BGM!
    let snap = value(og_runtime::host_impl::answer(og_world_snapshot(a, None)))
        .as_str()
        .unwrap()
        .to_string();

    // Continue engine A through DissolveNext to Ending.
    ticks(a, 110);
    let hash_a = world_hash(a);
    let tick_a = og_world_get_tick(a);
    let view_a = og_flow_view(a, flow).unwrap();
    let hist_a = og_flow_history(a, flow);
    assert!(view_a.ended && view_a.ending == "true_end", "{view_a:?}");

    // Restore snapshot into fresh engine B and advance the exact same 110 ticks.
    let ev_b = Rc::new(RefCell::new(Events::default()));
    let b = setup_engine(ev_b);
    value(og_runtime::host_impl::answer(og_world_restore(
        b, snap, None,
    )));
    // Immediate post-restore view must already be valid before the next tick.
    assert!(og_flow_view(b, flow).is_some());
    ticks(b, 110);

    assert_eq!(og_world_get_tick(b), tick_a);
    assert_eq!(world_hash(b), hash_a, "restored flow world hash diverged");
    let view_b = og_flow_view(b, flow).unwrap();
    assert_eq!(view_b.node, view_a.node);
    assert_eq!(view_b.time, view_a.time);
    assert_eq!(view_b.ended, view_a.ended);
    assert_eq!(view_b.ending, view_a.ending);
    assert_eq!(og_flow_history(b, flow), hist_a);
}
