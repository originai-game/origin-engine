//! A timeline's caption tracks on the runtime: they draw nothing. The keys
//! showing at the timeline's time are the UI's `$captions` (docs/ui.md
//! §2), which the built-in `subtitles` screen shows once the game pushes
//! it, and og.timeline.captions, which a TSX screen takes as a prop.
//!
//! `timelines/t.ogt` has two caption tracks: "First." from 1 s to 2 s and
//! "Second." from 3 s to 5 s on one, "Aside." from 1.5 s to 4 s on the
//! other.
//!
//! Ways this could fail: a caption shown before its time or after its end;
//! two tracks' captions in the order they started instead of the tracks'
//! order; a caption kept after a seek back, a stop, or its entity losing
//! its og.Timeline; a second caption on a track replacing the first's row
//! instead of being a row of its own; the screen showing nothing because a
//! pushed screen held its own empty `$captions`; a timeline that is not
//! playing leaving the lines of the one that was.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_game_game, og_host_begin_bundle,
    og_i18n_set_locale, og_load_timeline, og_timeline_captions, og_timeline_seek, og_timeline_stop,
    og_world_insert, og_world_spawn, set_hooks, shared_ui,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

const TIMELINE: &str = r#"type timeline
version 1
---
name "Said"
duration 6
track caption {
  key 1 {
    text "First."
    end 2
  }
  key 3 {
    text "Second."
    end 5
  }
}
track caption {
  key 1.5 {
    text "Aside."
    end 4
  }
}
"#;

fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "og-runtime-timeline-captions-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let p = root.join("assets/timelines/t.ogt");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, TIMELINE).unwrap();
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Game;

impl Hooks for Game {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
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

/// An entity whose og.Timeline holds `t.ogt`, stopped.
fn timeline(engine: u32) -> f64 {
    timeline_of(engine, "timelines/t.ogt")
}

fn timeline_of(engine: u32, path: &str) -> f64 {
    let asset = value(og_runtime::host_impl::answer(og_load_timeline(
        engine,
        path.into(),
    )));
    let entity = value(og_runtime::host_impl::answer(og_world_spawn(
        engine,
        Some("{}".into()),
    )))
    .as_f64()
    .unwrap();
    value(og_runtime::host_impl::answer(og_world_insert(
        engine,
        entity,
        "og.Timeline".into(),
        Some(json!({ "asset": asset }).to_string()),
    )));
    entity
}

/// Seeks `entity`'s timeline to `seconds` and runs the tick that shows it.
fn seek(engine: u32, entity: f64, seconds: f64) {
    value(og_runtime::host_impl::answer(og_timeline_seek(
        engine, entity, seconds,
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
}

/// The lines the subtitles screen draws, top to bottom.
fn said(engine: u32) -> Vec<String> {
    let ui = shared_ui(engine).expect("the engine's UI");
    let ui = ui.lock();
    let mut lines: Vec<_> = ui
        .probe_elements()
        .into_iter()
        .filter(|e| e.role == "text" && e.path.contains("Subtitles") && !e.text.is_empty())
        .collect();
    lines.sort_by(|a, b| a.y.total_cmp(&b.y));
    lines.into_iter().map(|e| e.text).collect()
}

#[test]
fn a_caption_track_feeds_the_subtitles_screen_and_draws_nothing_itself() {
    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let a = timeline(engine);
    value(og_engine_frame(engine, 1000.0 / 60.0));

    // With no screen that reads `$captions`, a showing caption draws
    // nothing: the engine has no caption of its own.
    seek(engine, a, 1.75);
    {
        let ui = shared_ui(engine).unwrap();
        let ui = ui.lock();
        assert!(ui.frame().items.is_empty(), "{:?}", ui.frame().items);
    }

    // The game pushes the built-in screen with no lines of its own.
    {
        let ui = shared_ui(engine).unwrap();
        let mut ui = ui.lock();
        ui.define_document(
            og_ui::UiDocument::parse(
                "ui/screens/subtitles.svelte",
                include_str!("../../../packages/engine/src/ui/screens/subtitles.svelte"),
            )
            .unwrap(),
        );
        ui.push_screen("subtitles", json!({ "lines": [] })).unwrap();
    }
    seek(engine, a, 1.75);
    assert_eq!(said(engine), ["First.", "Aside."], "both tracks at 1.75 s");
    seek(engine, a, 0.5);
    assert_eq!(said(engine), [""; 0], "before the first key");
    seek(engine, a, 1.25);
    assert_eq!(said(engine), ["First."]);
    seek(engine, a, 2.5);
    assert_eq!(said(engine), ["Aside."], "First. ended at 2 s");
    // The tracks' order, not the order they started in.
    seek(engine, a, 3.5);
    assert_eq!(said(engine), ["Second.", "Aside."]);
    seek(engine, a, 4.5);
    assert_eq!(said(engine), ["Second."]);
    seek(engine, a, 5.5);
    assert_eq!(said(engine), [""; 0], "after the last end");

    // A second timeline's lines follow the first's; one that is stopped
    // says nothing.
    let b = timeline(engine);
    seek(engine, a, 1.25);
    assert_eq!(said(engine), ["First."]);
    seek(engine, b, 3.5);
    assert_eq!(said(engine), ["First.", "Second.", "Aside."]);
    // og.timeline.captions answers the same lines, each with an id of its
    // own (a TSX Subtitles keys its rows by it).
    let rows = og_timeline_captions(engine);
    let texts: Vec<_> = rows.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(texts, ["First.", "Second.", "Aside."]);
    let ids: std::collections::BTreeSet<_> = rows.iter().map(|c| &c.id).collect();
    assert_eq!(ids.len(), 3, "{rows:?}");
    value(og_runtime::host_impl::answer(og_timeline_stop(engine, a)));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(said(engine), ["Second.", "Aside."], "a's caption went");
    value(og_runtime::host_impl::answer(og_timeline_stop(engine, b)));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(said(engine), [""; 0]);
    assert!(og_timeline_captions(engine).is_empty());
}

/// A caption file's translations beside it (`lines.zh.srt`,
/// `lines.pt-BR.srt` beside `lines.srt`) cook with the timeline, and the
/// track says the one of the locale the player asked for, its language's
/// for a regional tag (zh-Hans), else its own. A translation keeps its own
/// times (pt-BR's line starts at 2 s, not 1 s).
///
/// Ways this could fail: the translations not found (only `lines.srt`
/// cooked); a translation's text put on the base's times; zh-Hans missing
/// `zh`; a locale with no translation saying nothing instead of the base.
#[test]
fn a_caption_file_s_translations_are_said_in_the_player_s_locale() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "og-runtime-timeline-captions-locales-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("assets/timelines");
    std::fs::create_dir_all(&dir).unwrap();
    let srt = |at: &str, text: &str| format!("1\n{at}\n{text}\n");
    for (file, text) in [
        (
            "l.ogt",
            "type timeline\nversion 1\n---\nduration 4\ntrack caption {\n  captions \"lines.srt\"\n}\n".to_string(),
        ),
        ("lines.srt", srt("00:00:01,000 --> 00:00:02,000", "First.")),
        ("lines.zh.srt", srt("00:00:01,000 --> 00:00:02,000", "第一句。")),
        ("lines.pt-BR.srt", srt("00:00:02,000 --> 00:00:03,000", "Primeira.")),
    ] {
        std::fs::write(dir.join(file), text).unwrap();
    }
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());

    set_hooks(Rc::new(Game));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [] }).to_string()));
    let options = json!({ "root": root.to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let t = timeline_of(engine, "timelines/l.ogt");
    {
        let ui = shared_ui(engine).unwrap();
        let mut ui = ui.lock();
        ui.define_document(
            og_ui::UiDocument::parse(
                "ui/screens/subtitles.svelte",
                include_str!("../../../packages/engine/src/ui/screens/subtitles.svelte"),
            )
            .unwrap(),
        );
        ui.push_screen("subtitles", json!({ "lines": [] })).unwrap();
    }
    let at = |locale: &str, seconds: f64| {
        og_i18n_set_locale(engine, locale.into());
        seek(engine, t, seconds);
        said(engine)
    };
    assert_eq!(at("en", 1.5), ["First."]);
    assert_eq!(at("zh-Hans", 1.5), ["第一句。"], "zh for zh-Hans");
    assert_eq!(at("pt-BR", 1.5), [""; 0], "pt-BR's line starts at 2 s");
    assert_eq!(at("pt-BR", 2.5), ["Primeira."]);
    assert_eq!(at("fr", 1.5), ["First."], "no French: the base");
}
