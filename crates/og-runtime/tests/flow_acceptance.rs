//! og.flow snapshot and restore determinism: a flow restored mid-window
//! (with its queued ops, gestures, transcript and music) gives the same
//! `tick`, world hash, `FlowView` and `og.flow.history` as an uninterrupted
//! run.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_probe_dump,
    og_flow_choose, og_flow_history, og_flow_inspect, og_flow_load, og_flow_set_music,
    og_flow_set_speed, og_flow_start, og_flow_view, og_game_game, og_host_begin_bundle,
    og_story_load, og_system_define_system, og_world_events, og_world_get_tick, og_world_restore,
    og_world_snapshot, set_hooks,
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

        write("og-project.json", r#"{"version":1,"name":"FlowAccept","engine":"0.9.0"}"#);
        write("project.ts", "import {defineProject} from '@origingame/engine/author'; export default defineProject({id:'flow-accept',entry:'src/main.ts'});");
        write("src/main.ts", "export default {};\n");
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
