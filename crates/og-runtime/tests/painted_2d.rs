//! Painted 2D (docs/art.md §2.3): in a `profile "2d"` project under the
//! art contract, a look's paint and watercolour fields are the cook's,
//! applied in each sprite's own texels, and the frame draws neither the
//! paint nor the watercolour pass, so a sprite is painted and washed once
//! and the look is tuned once for both. A 3D project paints its frame on
//! screen as before.
//!
//! Ways this could fail: the frame still runs the paint or watercolour
//! pass over sprites the cook already painted (the old code did, so every
//! painted 2D sprite was washed twice); a 3D frame loses its paint pass;
//! or the skip ignores the contract's switch, so a 2D project whose look
//! has paint fields but no art blocks (which the cook leaves alone) draws
//! unpainted.

use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_destroy, og_engine_frame,
    og_game_game, og_host_begin_bundle, set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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

/// Keeps the paint of the last frame's stylisation.
struct Recorder(Rc<Cell<Option<Option<og_render::Paint>>>>);

impl RenderBackend for Recorder {
    fn tier(&self) -> String {
        "test".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.0.set(Some(scene.stylise.paint));
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

/// A 4 x 4 sprite in two colours, as a PNG.
fn png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(4, 4, |x, _| {
        image::Rgba(if x < 2 {
            [220, 40, 40, 255]
        } else {
            [40, 60, 220, 255]
        })
    });
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

const PAINT: &str = "Stylise {\n  paintRadius 2\n  paintEdgeDarkening 0.3\n  watercolorEdgeDarkening 0.4\n  watercolorTurbulence 0.3\n}\n";

/// A project with one sprite texture under a look, cooked; `profile` is
/// project.ogt's line, `art` the look's art blocks.
fn project(name: &str, profile: &str, art: &str) -> (PathBuf, String, og_cook::Report) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "og-runtime-painted-2d-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let text = format!(
        "type project\nversion 1\n---\nname \"Painted\"\nentry \"src/main.ts\"\n{profile}look \"looks/p.ogt\"\n"
    );
    let write = |path: &str, bytes: &[u8]| {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    };
    write("project.ogt", text.as_bytes());
    write(
        "assets/looks/p.ogt",
        format!("type look\nversion 1\n---\nname \"P\"\n{PAINT}{art}").as_bytes(),
    );
    write("assets/sprites/hero.png", &png());
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    (root, text, report)
}

/// The paint the renderer is handed for the project's first frame.
fn frame_paint(root: &Path, text: &str, profile: &str) -> Option<og_render::Paint> {
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1 }).to_string()));
    let options =
        json!({ "root": root.to_str().unwrap(), "project": text, "profile": profile }).to_string();
    let a: Value = serde_json::from_str(&og_engine_create(game, options)).unwrap();
    assert_eq!(a["ok"], json!(true), "{a}");
    let engine = a["value"].as_u64().unwrap() as u32;
    let seen = Rc::new(Cell::new(None));
    set_render_backend(engine, Some(Box::new(Recorder(seen.clone())))).unwrap();
    let f: Value = serde_json::from_str(&og_engine_frame(engine, 1000.0 / 60.0)).unwrap();
    assert_eq!(f["ok"], json!(true), "{f}");
    let paint = seen.get().expect("the renderer drew a frame");
    assert!(og_engine_destroy(engine));
    paint
}

/// The cook's note on the sprite: what it painted in texel space.
fn painted(report: &og_cook::Report) -> Option<String> {
    report
        .conform
        .iter()
        .find(|e| e.source == "sprites/hero.png")
        .map(|e| e.what.clone())
        .filter(|w| w.contains("painted in texel space"))
}

#[test]
fn a_painted_2d_sprite_is_painted_by_the_cook_alone() {
    let art = "ArtPalette {\n  maxColors 8\n}\n";
    let (root, text, report) = project("2d", "profile \"2d\"\n", art);
    assert!(painted(&report).is_some(), "{:?}", report.conform);
    let paint = frame_paint(&root, &text, "2d");
    assert_eq!(paint, None, "the frame paints the painted sprites again");
}

#[test]
fn without_the_contracts_switch_a_2d_frame_paints_on_screen() {
    // The look sets paint fields but no art block: the cook leaves the
    // sprite as drawn, so the frame is the one place it is painted.
    let (root, text, report) = project("2d-frame", "profile \"2d\"\n", "");
    assert_eq!(painted(&report), None, "{:?}", report.conform);
    let paint = frame_paint(&root, &text, "2d").expect("the frame's paint");
    assert!(paint.paints() && paint.watercolor.active(), "{paint:?}");
}
