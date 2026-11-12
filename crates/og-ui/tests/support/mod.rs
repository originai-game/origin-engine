//! The screens og-ui's integration tests drive (`screens.rs`, which the
//! browser tiers compile too), the MSDF test font's metrics and handles,
//! and the document harness. Nothing here renders: the frame tests
//! (the og-tests-ui crate) include this support by path and add the render
//! scenes and the reference-frame comparison beside it.
#![allow(dead_code)]

mod doc;
mod engine_documents;
mod screens;

// Not every binary that includes this support mounts documents.
#[allow(unused_imports)]
pub(crate) use doc::{Doc, both, id_text};
#[allow(unused_imports)]
pub use engine_documents::*;
use og_ui::*;
pub use screens::*;
use std::path::PathBuf;

pub fn dressed(css: &str) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(&format!("{DEFAULT_CSS}\n{css}"))
        .expect("test CSS");
    ui
}

pub fn fills_in(ui: &Ui, rect: Rect) -> Vec<[f32; 4]> {
    ui.frame()
        .items
        .iter()
        .filter_map(|i| i.cmd.filled())
        .filter(|(r, _)| {
            (r.x - rect.x).abs() < 1.0
                && (r.y - rect.y).abs() < 1.0
                && (r.width - rect.width).abs() < 1.0
        })
        .map(|(_, c)| c)
        .collect()
}

pub fn round_fill(cmd: &DrawCmd) -> Option<(Rect, f32)> {
    let (rect, _) = cmd.filled()?;
    let radius = match cmd {
        DrawCmd::Quad { radius, .. } => *radius,
        DrawCmd::Shape { shape, .. } => match shape.geometry {
            og_draw::UiGeometry::Rect { radii, .. } if radii.iter().all(|r| *r == radii[0]) => {
                radii[0]
            }
            _ => return None,
        },
        _ => return None,
    };
    Some((rect, radius))
}

pub fn near(a: [f32; 4], b: Srgba) -> bool {
    let l = b.linear();
    (0..3).all(|k| (a[k] - l[k]).abs() < 0.01)
}

fn fonts_dir() -> PathBuf {
    // The MSDF test atlas og-tests commits (Noto Sans SC subset, OFL).
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-tests/assets/fonts")
}

/// The MSDF test font's files, read from og-tests' assets.
pub fn test_font_files() -> (Vec<u8>, String, String) {
    let dir = fonts_dir();
    let read = |f: &str| {
        std::fs::read(dir.join(f)).unwrap_or_else(|e| panic!("{}: {e}", dir.join(f).display()))
    };
    (
        read("noto-sans-sc-msdf.png"),
        String::from_utf8(read("noto-sans-sc-msdf.json")).expect("font json"),
        String::from_utf8(read("noto-sans-sc-msdf.csv")).expect("font csv"),
    )
}

/// Runs `f` with the MSDF test font's files.
pub fn with_test_font<T>(f: impl FnOnce(&FontFiles) -> T) -> T {
    let (png, json, csv) = test_font_files();
    f(&FontFiles {
        png: &png,
        json: &json,
        csv: &csv,
    })
}

/// The font and texture handles a scene would give, in a scene's order,
/// for tests that render nothing: what the UI lays out and the commands
/// it builds take a font's metrics and handles to name, never an atlas.
#[derive(Default)]
pub struct Handles {
    textures: usize,
    fonts: usize,
}

impl Handles {
    pub fn texture(&mut self) -> og_draw::TextureHandle {
        self.textures += 1;
        og_draw::TextureHandle::from_index(self.textures - 1)
    }

    pub fn font_handle(&mut self) -> og_draw::FontHandle {
        self.fonts += 1;
        og_draw::FontHandle::from_index(self.fonts - 1)
    }

    /// The MSDF font of `files` as a UI font (its atlas takes a texture
    /// handle, as it does in a scene).
    pub fn font(&mut self, files: &FontFiles) -> UiFont {
        let (_, metrics) = msdf_font(files, self.texture());
        UiFont {
            handle: Some(self.font_handle()),
            metrics,
        }
    }

    pub fn test_font(&mut self) -> UiFont {
        with_test_font(|files| self.font(files))
    }
}

/// The MSDF test font under the first handle a scene gives.
pub fn test_font() -> UiFont {
    Handles::default().test_font()
}

/// The test font's metrics with nothing to draw with (tests without a renderer).
pub fn test_font_metrics() -> UiFont {
    UiFont {
        handle: None,
        ..test_font()
    }
}

/// Where tests write what they produce: `$CARGO_TARGET_TMPDIR/og-ui`.
pub fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui");
    std::fs::create_dir_all(&dir).expect("out dir");
    dir
}
