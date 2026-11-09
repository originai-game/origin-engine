//! The sample screen every og-ui integration test drives, the MSDF test font
//! and the reference-frame comparison (og-tests' tolerance rules). The
//! screens and reference scenes themselves are in `scenes.rs`, which the
//! browser tiers (crates/og-tests-web) compile too.
#![allow(dead_code)]

mod scenes;

use og_render::RenderScene;
use og_ui::*;
pub use scenes::*;
use std::path::{Path, PathBuf};

pub fn metadata_style() -> &'static str {
    static TEXT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let choices =
            og_ui::style_gen::StyleChoices::for_look("test", "minimal modern", "metadata test");
        og_ui::style_gen::generate(&choices)
            .expect("metadata generation")
            .text
    });
    &TEXT
}

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
            og_render::UiGeometry::Rect { radii, .. } if radii.iter().all(|r| *r == radii[0]) => {
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

/// Adds the MSDF test font (msdf-atlas-gen's PNG, CSV and JSON) to a scene;
/// the UI font for it, with the font's own ascender and descender.
pub fn add_test_font(scene: &mut RenderScene) -> UiFont {
    with_test_font(|files| add_font(scene, files))
}

/// The test font's metrics with nothing to draw with (tests without a renderer).
pub fn test_font_metrics() -> UiFont {
    let mut scene = RenderScene::new();
    UiFont {
        handle: None,
        ..add_test_font(&mut scene)
    }
}

/// Records a reference frame (`OG_UI_CAPTURE`) as og-render's `png_best`,
/// the smallest PNG of its pixels: never a copy of the fast-encoded frame
/// a run writes.
pub fn record_reference(own: &Path, width: u32, height: u32, pixels: &[u8]) {
    std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
    let image = og_render::RgbaImage {
        width,
        height,
        pixels: pixels.to_vec(),
    };
    std::fs::write(own, og_render::png_best(&image)).expect("record reference");
}

/// `actual` (rendered by `info`'s adapter) against the frame read from
/// `reference`, under og-tests' rule (`og_tests::compare::tolerance_for`):
/// the default tolerance against its own tier's reference, the
/// cross-platform one against another tier's. Answers the RMSE and the
/// outliers' share, or what was exceeded.
pub fn compare(
    actual: &[u8],
    expected: &[u8],
    info: &og_render::RenderInfo,
    reference: &Path,
) -> Result<(f64, f64), String> {
    use og_tests::compare::{Tolerance, tolerance_for};
    let tolerance = tolerance_for(
        Tolerance::default(),
        Tolerance::CROSS_PLATFORM,
        &info.backend_tier(),
        info.software(),
        reference,
    );
    // One row of pixels: the comparison reads them in order, whatever the shape.
    let c = og_tests::compare::compare(actual, expected, (actual.len() / 4) as u32, 1, &tolerance)?;
    c.check(&tolerance)?;
    Ok((c.rmse, c.outlier_fraction()))
}

/// Where tests write what they produce: `$CARGO_TARGET_TMPDIR/og-ui`.
pub fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui");
    std::fs::create_dir_all(&dir).expect("out dir");
    dir
}
