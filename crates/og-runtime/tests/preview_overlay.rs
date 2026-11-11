//! The screen overlay of a previewed viewport (og.ui.previewViewport): the
//! UI, screen sprites and screen texts share one overlay, laid on the CSS
//! surface the UI lays out on and fitted to the render target.
//!
//! A phone previewed at 390 x 844 CSS px, 2 device pixels per CSS pixel, on
//! a 780 x 1688 target: a screen text made the overlay at the target's
//! size, and the UI emitted into it at one CSS pixel per device pixel, drawn
//! in the frame's top left quarter (og ui check's phone cells, the kit's
//! phone style playtests, once a HUD page's scene became a screen sprite).
//! Ways this could fail: the overlay sized from the target while a preview
//! is set; the preview kept after it is cleared.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame, og_game_game,
    og_host_begin_bundle, og_load_font, og_render_text, og_ui_preview_viewport, set_hooks,
    set_render_backend,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A project with one font, cooked once per process.
fn project() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-preview-overlay-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("assets/fonts");
        std::fs::create_dir_all(&dir).unwrap();
        let style = repo().join("templates/styles/shared");
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

/// A 780 x 1688 target recording the last frame's overlay size.
struct Recorder(Rc<RefCell<Option<(f32, f32)>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (780, 1688)
    }
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        *self.0.borrow_mut() = scene.ui.overlay.as_ref().map(|o| (o.width, o.height));
        Ok(())
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

#[test]
fn a_previewed_viewport_lays_the_screen_overlay_on_its_css_surface() {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let options = json!({ "root": project().to_str().unwrap() }).to_string();
    let engine = value(og_engine_create(game, options)).as_u64().unwrap() as u32;
    let seen = Rc::new(RefCell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let font = value(og_runtime::host_impl::answer(og_load_font(
        engine,
        "fonts/ChakraPetch-BoldItalic.ttf".into(),
    )));
    value(og_engine_frame(engine, 1000.0 / 60.0));
    let screen_text = || {
        let d = json!({ "font": font, "text": "F", "screen": true, "position": { "x": 20, "y": 20, "z": 0 } });
        value(og_runtime::host_impl::answer(og_render_text(
            engine,
            Some(d.to_string()),
        )));
    };

    og_ui_preview_viewport(
        engine,
        json!({ "width": 390, "height": 844, "dpr": 2, "breakpoint": "phonePortrait" }).to_string(),
    );
    screen_text();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(
        *seen.borrow(),
        Some((390.0, 844.0)),
        "the overlay is the previewed CSS surface, fitted to the 780 x 1688 target"
    );

    // Cleared, the overlay is the target's again.
    og_ui_preview_viewport(engine, "null".into());
    screen_text();
    value(og_engine_frame(engine, 1000.0 / 60.0));
    assert_eq!(*seen.borrow(), Some((780.0, 1688.0)));
}
