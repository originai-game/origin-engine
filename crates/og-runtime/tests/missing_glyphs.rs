//! The missing-glyph warning (Meridian Loop's finding #13): a frame that
//! draws characters its font lacks is warned of on the tick it draws them,
//! naming each character once, the font's asset and what its atlas holds
//! (the charset its `.ogt` cooked), so the minus sign a race first draws at
//! its second lap is named at that tick even after another character was
//! warned of earlier.
//!
//! Ways this could fail (each the old warning): it says tick 0; it fires
//! once a process, so a later character is never named; it names neither
//! the font nor its charset.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_engine_take_log,
    og_game_game, og_host_begin_bundle, og_load_font, og_render_text, og_world_get_tick, set_hooks,
    set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project with the kit's neon display face (`charset latin punct`),
/// cooked once per test process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-missing-glyphs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/fonts");
        std::fs::create_dir_all(&dir).unwrap();
        let style = repo().join("templates/styles/neon");
        for f in [
            "ChakraPetch-BoldItalic.ttf",
            "ChakraPetch-BoldItalic.ttf.ogt",
        ] {
            std::fs::copy(style.join(f), dir.join(f)).unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        root
    })
}

struct Disk;

impl Hooks for Disk {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
}

/// Counts what og-render's FrameStats::missing_glyphs counts: drawn
/// characters (not spaces) the text's font has no glyph for, in the world and on the
/// screen overlay.
struct Counter(Cell<u32>);

impl RenderBackend for Counter {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (320, 180)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        let mut n = 0;
        let texts = scene
            .texts
            .iter()
            .chain(scene.overlay.iter().flat_map(|o| o.texts.iter()));
        for t in texts {
            let font = scene.font(t.font).expect("the text's font");
            n += t
                .text
                .chars()
                .filter(|c| !c.is_whitespace() && !font.glyphs.iter().any(|g| g.codepoint == *c))
                .count() as u32;
        }
        self.0.set(n);
        Ok(())
    }
    fn missing_glyphs(&self) -> u32 {
        self.0.get()
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

/// Steps one tick, then draws `text` and renders it (a frame of no tick).
fn show(engine: u32, font: &Value, text: &str) {
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let desc = json!({ "font": font, "text": text, "screen": true, "size": 24 });
    value(og_render_text(engine, Some(desc.to_string())));
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
    set_render_backend(engine, Some(Box::new(Counter(Cell::new(0))))).unwrap();
    let font = value(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    ));
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
