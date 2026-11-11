//! The missing-glyph warning (Meridian Loop's finding #13): a frame that
//! draws characters its font lacks is warned of on the tick it draws them,
//! naming each character once, the font's asset and what its atlas holds
//! (the charset its `.ogt` cooked), so the minus sign a race first draws at
//! its second lap is named at that tick even after another character was
//! warned of earlier.
//!
//! Ways this could fail (each the old warning): it says tick 0; it fires
//! once a process, so a later character is never named; it names neither
//! the font nor its charset; it needs a renderer (it once counted what the
//! GPU drew, so a run without one, or whose adapter request failed, never
//! warned: ui-kit's settings-saved missed 국 one run in five); it is once a
//! character, so a second face lacking the same one is never named.
//!
//! Every test here runs with no render backend: the check reads each
//! face's characters on the CPU (og-ui's frame, a queued text).

use og_runtime::{
    Dispatch, Fetch, Hooks, og_engine_create, og_engine_frame, og_engine_take_log, og_game_game,
    og_host_begin_bundle, og_load_font, og_render_floating_text, og_render_text,
    og_system_define_system, og_ui_configure, og_world_get_tick, set_hooks, shared_ui,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project with the kit's neon display face cooked with `charset latin
/// punct` alone (no `chars`: the minus and arrows stay out of its atlas),
/// once per test process.
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
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
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
}

impl Hooks for Disk {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0
            && call.id == BUILD
            && let Some(text) = LABEL.with(|l| l.borrow().clone())
        {
            let ui = shared_ui(call.engine).expect("the engine's UI");
            ui.lock()
                .label("Status", &text, og_ui::LabelDesc::default());
            BUILT.with(|b| b.set((b.get().0, og_world_get_tick(b.get().0) as u64)));
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
    value(og_engine_frame(engine, 0.0));
    let _ = warnings(engine);

    // The first lap: Latin-1 and the typographic punctuation the font cooked.
    for _ in 0..3 {
        show(engine, &font, "Lap 1/3 – 0:31.200");
    }
    assert_eq!(warnings(engine), vec![], "every character is in the atlas");

    // The arrows the title once drew: warned of at their tick.
    show(engine, &font, "↑ ↓ Steer");
    let arrows = og_world_get_tick(engine) as u64;
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, arrows, "the tick that drew them, not 0: {w:?}");
    for part in [
        "\"↑\" U+2191",
        "\"↓\" U+2193",
        "font fonts/ChakraPetch-BoldItalic.ttf",
        "charset latin punct",
        "font { chars \"↑↓\" }",
    ] {
        assert!(w[0].1.contains(part), "names {part}: {}", w[0].1);
    }
    for _ in 0..3 {
        show(engine, &font, "↑ ↓ Steer");
    }
    assert_eq!(warnings(engine), vec![], "each character once");

    // The second lap's first split, much later: its minus sign is named too.
    for _ in 0..40 {
        show(engine, &font, "Lap 2/3");
    }
    show(engine, &font, "S1 −0.412 ↑");
    let split = og_world_get_tick(engine) as u64;
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, split);
    assert!(w[0].1.contains("\"−\" U+2212"), "{}", w[0].1);
    assert!(
        !w[0].1.contains("U+2191"),
        "the arrow was named already: {}",
        w[0].1
    );
}

/// og-ui's text: a label the UI draws in a face that lacks some of its
/// characters is named on the tick the label is built, with no renderer;
/// once per (face, character), so the same arrow in a second face is named
/// again for that face, and a floating text in the UI's face that draws a
/// character the UI already named stays quiet.
#[test]
fn ui_text_warns_per_face_with_no_renderer() {
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

    LABEL.with(|l| *l.borrow_mut() = Some("Lap 1/3 – 0:31".into()));
    step();
    assert_eq!(warnings(engine), vec![], "every character is in the atlas");

    LABEL.with(|l| *l.borrow_mut() = Some("Lap 2/3 ↑".into()));
    step();
    let tick = BUILT.with(|b| b.get().1);
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(w[0].0, tick, "the tick that built it: {w:?}");
    for part in [
        "\"↑\" U+2191",
        "font fonts/ChakraPetch-BoldItalic.ttf",
        "charset latin punct",
    ] {
        assert!(w[0].1.contains(part), "names {part}: {}", w[0].1);
    }
    for _ in 0..3 {
        step();
    }
    assert_eq!(warnings(engine), vec![], "each (face, character) once");

    // A floating text in the UI's face: the arrow was named for it already.
    let float = json!({ "text": "↑ 12", "position": { "x": 0, "y": 0, "z": 0 } });
    value(og_runtime::host_impl::answer(og_render_floating_text(
        engine,
        Some(float.to_string()),
    )));
    step();
    assert_eq!(warnings(engine), vec![], "named for this face already");

    // The same arrow and a dash in the Latin face: named again, for it.
    show(engine, &latin, "– ↑");
    let w = warnings(engine);
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].1.contains("font fonts/Latin.ttf"), "{}", w[0].1);
    assert!(
        w[0].1.contains("U+2191") && w[0].1.contains("U+2013"),
        "{}",
        w[0].1
    );
}
