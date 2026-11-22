//! The missing-glyph warning (Meridian Loop's finding #13): a frame that
//! draws characters no face of the UI's collection covers is warned of on
//! the tick it draws them, naming each character once and the font that
//! drew the text, so the Hangul a race first draws at its second lap is
//! named at that tick even after another character was warned of earlier.
//!
//! Ways this could fail (each the old warning): it says tick 0; it fires
//! once a process, so a later character is never named; it names neither
//! the character nor the font; it needs a renderer (it once counted what
//! the GPU drew, so a run without one, or whose adapter request failed,
//! never warned: ui-kit's settings-saved missed 국 one run in five); it is
//! once a face, so a character one face covers is still named for another
//! face's text.
//!
//! Every test here runs with no render backend: the check reads each
//! face's coverage on the CPU (og-ui's frame, a queued text).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_load_font, og_render_floating_text, og_render_text,
    og_system_define_system, og_ui_configure, og_world_get_tick, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project with the kit's neon display face, cooked twice under two
/// names (the `.ogt`s' charsets are the cook's; at run time both faces'
/// coverage is their file's, read whole), once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-missing-glyphs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/fonts");
        std::fs::create_dir_all(&dir).unwrap();
        let style = repo().join("templates/styles/shared");
        std::fs::copy(
            style.join("ChakraPetch-BoldItalic.ttf"),
            dir.join("ChakraPetch-BoldItalic.ttf"),
        )
        .unwrap();
        std::fs::write(
            dir.join("ChakraPetch-BoldItalic.ttf.ogt"),
            "type import\nversion 1\n---\nfont {\n  charset latin punct\n  em_px 48\n  px_range 4\n}\n",
        )
        .unwrap();
        // A second face: the same file, its atlas Latin alone.
        std::fs::copy(
            style.join("ChakraPetch-BoldItalic.ttf"),
            dir.join("Latin.ttf"),
        )
        .unwrap();
        std::fs::write(
            dir.join("Latin.ttf.ogt"),
            "type import\nversion 1\n---\nfont {\n  charset latin\n  em_px 48\n  px_range 4\n}\n",
        )
        .unwrap();
        let report = og_cook::cook_project(
            &root,
            // Its label is TSX: the cook bundles the default UI's font and
            // pack art, as for code that reaches the TSX UI.
            &og_cook::CookOptions {
                code: og_cook::cook::CodeGraph {
                    tsx: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

/// Reads the project from disk; its one system shows `LABEL` as a UI label.
struct Disk;

const BUILD: u32 = 1;

thread_local! {
    /// The text the UI shows this tick (none: no label).
    static LABEL: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The engine and the tick its system last built the label on.
    static BUILT: std::cell::Cell<(u32, u64)> = const { std::cell::Cell::new((0, 0)) };
    /// The engines whose UI shows the label.
    static MOUNTED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// Shows `text` as a TSX label in engine `e`'s tree, as the TS renderer's
/// op batch does (og-ui-next CONVENTIONS.md §Ops): on the first call a View
/// holding a text run mounted at the root, after it the run's new text.
fn label(e: u32, first: bool, text: &str) {
    let mut ops = Vec::new();
    let u32s = |ops: &mut Vec<u8>, vs: &[u32]| {
        for v in vs {
            ops.extend_from_slice(&v.to_le_bytes());
        }
    };
    let str = |ops: &mut Vec<u8>, s: &str| {
        ops.extend_from_slice(&(s.len() as u32).to_le_bytes());
        ops.extend_from_slice(s.as_bytes());
    };
    let count = if first {
        // create 1 (a View), createText 2, insert 2 into 1, 1 into the root.
        ops.push(1);
        u32s(&mut ops, &[1]);
        ops.push(0);
        ops.push(2);
        u32s(&mut ops, &[2]);
        str(&mut ops, text);
        ops.push(8);
        u32s(&mut ops, &[1, 2, u32::MAX]);
        ops.push(8);
        u32s(&mut ops, &[0, 1, u32::MAX]);
        4u32
    } else {
        ops.push(3);
        u32s(&mut ops, &[2]);
        str(&mut ops, text);
        1
    };
    let mut batch = og_ui_next::tree::MAGIC.to_vec();
    batch.extend_from_slice(&count.to_le_bytes());
    batch.extend_from_slice(&ops);
    let reply = og_runtime::og_engine_ui_patch(e, &batch);
    assert!(
        !reply.starts_with(og_ui_next::tree::REFUSED),
        "the ops apply: {}",
        String::from_utf8_lossy(&reply)
    );
}

impl Hooks for Disk {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == BUILD {
            let shown = LABEL.with(|l| l.borrow().clone());
            let first = MOUNTED.with(|m| !m.borrow().contains(&call.engine));
            if shown.is_none() && first {
                return Ok(String::new());
            }
            let text = shown.clone().unwrap_or_default();
            if first {
                MOUNTED.with(|m| m.borrow_mut().push(call.engine));
            }
            label(call.engine, first, &text);
            if shown.is_some() {
                BUILT.with(|b| b.set((b.get().0, og_world_get_tick(b.get().0) as u64)));
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

/// Steps one tick, then draws `text` and renders it (a frame of no tick).
fn show(engine: u32, font: &Value, text: &str) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let desc = json!({ "font": font, "text": text, "screen": true, "size": 24 });
    value(og_runtime::host_impl::answer(og_render_text(
        engine,
        Some(desc.to_string()),
    )));
    value(og_engine_frame(engine, 0.0));
}

/// The warnings logged since the last call: (tick, text).
fn warnings(engine: u32) -> Vec<(u64, String)> {
    let log: Value = serde_json::from_str(&og_engine_take_log(engine)).unwrap();
    log.as_array()
        .unwrap()
        .iter()
        .filter(|l| l["warning"] == json!(true))
        .map(|l| {
            (
                l["tick"].as_u64().unwrap(),
                l["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn a_missing_glyph_is_named_once_on_the_tick_it_is_first_drawn() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let font = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    )));
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({ "font": font }).to_string()),
    )));
    value(og_engine_frame(engine, 0.0));
    let _ = warnings(engine);

    // The first lap: Latin-1 and the typographic punctuation the file has.
    for _ in 0..3 {
        show(engine, &font, "Lap 1/3 – 0:31.200");
    }
    assert_eq!(
        warnings(engine),
        vec![],
        "every character is in the font's file"
    );

    // The Hangul the file lacks: warned of at its tick.
    show(engine, &font, "국 ↑ ↓ Steer");
    let hangul = og_world_get_tick(engine) as u64;
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, hangul, "the tick that drew it, not 0: {w:?}");
    for part in [
        "\"국\" U+AD6D",
        "no font of the UI has",
        "font fonts/ChakraPetch-BoldItalic.ttf among those it asks",
        "fallbackFonts",
    ] {
        assert!(w[0].1.contains(part), "names {part}: {}", w[0].1);
    }
    for _ in 0..3 {
        show(engine, &font, "국 ↑ ↓ Steer");
    }
    assert_eq!(warnings(engine), vec![], "each character once");

    // A much later split draws another character the file lacks: named too.
    for _ in 0..40 {
        show(engine, &font, "Lap 2/3");
    }
    show(engine, &font, "S1 −0.412 한");
    let split = og_world_get_tick(engine) as u64;
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, split);
    assert!(w[0].1.contains("\"한\" U+D55C"), "{}", w[0].1);
    assert!(
        !w[0].1.contains("U+AD6D"),
        "국 was named already: {}",
        w[0].1
    );
}

/// The TSX UI's text, with no renderer. A face draws what its file has, so
/// the punctuation the label mixes draws and is not named. A character no
/// font of the UI has is named on the tick the label is built, once — a
/// queued or floating text that draws it after is not: the UI's
/// collection is one, whatever drew through it.
#[test]
fn ui_text_names_what_no_font_has_with_no_renderer() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let system = og_system_define_system(
        "Status".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let punct = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    )));
    let latin = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/Latin.ttf".into(),
    )));
    BUILT.with(|b| b.set((engine, 0)));
    value(og_runtime::host_impl::answer(og_ui_configure(
        engine,
        Some(json!({ "font": punct }).to_string()),
    )));
    let step = || value(og_engine_frame(engine, 1000.0 / 60.0));
    step();
    let _ = warnings(engine);

    LABEL.with(|l| *l.borrow_mut() = Some("Lap 1/3 – 0:31 ↑".into()));
    step();
    assert_eq!(warnings(engine), vec![], "the face's file has them all");

    LABEL.with(|l| *l.borrow_mut() = Some("Lap 2/3 국".into()));
    step();
    let tick = BUILT.with(|b| b.get().1);
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, tick, "the tick that built it: {w:?}");
    for part in ["\"국\" U+AD6D", "no font of the UI has", "fallbackFonts"] {
        assert!(w[0].1.contains(part), "names {part}: {}", w[0].1);
    }
    for _ in 0..3 {
        step();
    }
    assert_eq!(warnings(engine), vec![], "each character once");

    // A floating text draws a character the file lacks too: named there.
    let float = json!({
        "text": "↑ 12 한", "font": punct,
        "position": { "x": 0, "y": 0, "z": 0 }
    });
    value(og_runtime::host_impl::answer(og_render_floating_text(
        engine,
        Some(float.to_string()),
    )));
    step();
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].1.contains("U+D55C") && w[0].1.contains("font fonts/ChakraPetch-BoldItalic.ttf"),
        "{}",
        w[0].1
    );

    // The same character through the second face's queued text: not named
    // again — the collection is one, whatever drew through it.
    show(engine, &latin, "한 – ↑");
    assert_eq!(warnings(engine), vec![], "each character once");
}
