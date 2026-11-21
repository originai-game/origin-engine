//! og.flow asks for the first chunk of every segment a choice may cut to
//! while the segment before it plays, not only those a spare deck warms:
//! a flow on one deck (no deck to warm on) whose `start` offers Red
//! (#default), Green and Blue, each its own streamed video, has the host
//! read each target's stream index and its first chunk before the cut,
//! and has them read again from the new node's targets after it.
//!
//! Ways this could fail: only the default (or the pending choice) asked
//! for; nothing asked for until the window opens or the cut (the old
//! behaviour on one deck); a target's chunk asked for without its index
//! read first (no units, so nothing is wanted); the chunks let go of again
//! at once because no player wants them.
//!
//! The renderer is a stand-in that draws nothing: what is asked for is
//! decided before anything is drawn, and nothing here needs a GPU.

use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_flow_load,
    og_flow_start, og_flow_view, og_game_game, og_host_begin_bundle, og_story_parse, set_hooks,
    set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const TARGETS: [&str; 3] = ["red", "green", "blue"];

fn segment(video: &str, markers: &str) -> String {
    format!(
        "type timeline\nversion 1\n---\nduration 1\ntrack video {{\n  key 0 {{\n    video \"clips/{video}.ogt\"\n  }}\n}}\n{markers}"
    )
}

const WINDOW: &str = "track marker {\n  key 0.5 {\n    name \"flow.open\"\n  }\n  key 0.8 {\n    name \"flow.lock\"\n  }\n  key 0.9 {\n    name \"flow.cut\"\n  }\n}\n";

fn script() -> String {
    format!(
        r#"#og-profile:{}
#og-name:doors
title: start
og-flow: segments/start.ogt
---
-> Red #line:red #default
    <<jump red>>
-> Green #line:green
    <<jump green>>
-> Blue #line:blue
    <<jump blue>>
===
title: red
og-flow: segments/red.ogt
---
===
title: green
og-flow: segments/green.ogt
---
===
title: blue
og-flow: segments/blue.ogt
---
===
"#,
        og_story::yarn::PROFILE
    )
}

/// Four streamed clips and their segments, cooked once (None without an
/// ffmpeg with AV1).
fn project() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        og_cook::cook::video::av1_encoder()?;
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-flow-prefetch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let clips = root.join("assets/clips");
        std::fs::create_dir_all(&clips).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/engine/tests/fixtures/video/assets/clips/count.mp4"),
            clips.join("count.mp4"),
        )
        .unwrap();
        let segments = root.join("assets/segments");
        std::fs::create_dir_all(&segments).unwrap();
        for name in ["start", "red", "green", "blue"] {
            std::fs::write(
                clips.join(format!("{name}.ogt")),
                "type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize 128 72\naudio false\nrenditions none\n",
            )
            .unwrap();
            let markers = if name == "start" { WINDOW } else { "" };
            std::fs::write(segments.join(format!("{name}.ogt")), segment(name, markers)).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        Some(root)
    })
    .as_deref()
}

/// Reads from disk and records every path asked for.
struct Host(Rc<RefCell<Vec<String>>>);

impl Hooks for Host {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        self.0.borrow_mut().push(path.to_string());
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// A renderer that draws nothing.
struct Blank;

impl RenderBackend for Blank {
    fn tier(&self) -> String {
        "full".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, _scene: &og_render::RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(
        &mut self,
        _scene: &og_render::RenderScene,
        _width: u32,
        _height: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

/// The distinct stream files of clip `name` read so far (its index and
/// chunks; never the `.ogt` asset itself).
fn streams(reads: &Rc<RefCell<Vec<String>>>, name: &str) -> Vec<String> {
    let mut out: Vec<String> = reads
        .borrow()
        .iter()
        .filter(|p| p.contains(&format!("clips/{name}.ogt.stream")))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

#[test]
fn every_cut_target_has_its_first_chunk_read_before_the_cut_on_one_deck() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let reads = Rc::new(RefCell::new(Vec::new()));
    set_hooks(Rc::new(Host(reads.clone())));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    set_render_backend(engine, Some(Box::new(Blank))).unwrap();
    value(og_runtime::host_impl::answer(og_story_parse(
        engine,
        script(),
    )));
    value(og_runtime::host_impl::answer(og_flow_load(
        engine,
        "doors".into(),
    )));
    let flow = value(og_runtime::host_impl::answer(og_flow_start(
        engine,
        Some(json!({ "dialogue": "doors", "decks": 1 }).to_string()),
    )))
    .as_f64()
    .unwrap();
    for name in TARGETS {
        assert!(
            streams(&reads, name).is_empty(),
            "{name} read at load: {:?}",
            streams(&reads, name)
        );
    }
    let view = || -> Value {
        serde_json::from_str(&og_runtime::host_impl::json(og_flow_view(engine, flow))).unwrap()
    };
    // A few ticks into `start`, well before its window opens at 0.5 s.
    for _ in 0..6 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
    }
    assert_eq!(view()["node"], json!("start"), "{}", view());
    assert!(view()["time"].as_f64().unwrap() < 0.5, "{}", view());
    for name in TARGETS {
        let read = streams(&reads, name);
        assert!(
            read.len() >= 2,
            "{name}: its index and its first chunk are read before the window opens, on one deck: {read:?}"
        );
    }
}
