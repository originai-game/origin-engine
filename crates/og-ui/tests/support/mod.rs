//! The sample screen every og-ui integration test drives, the MSDF test font
//! and the reference-frame comparison (og-tests' tolerance rules). The
//! screens and reference scenes themselves are in `scenes.rs`, which the
//! browser tiers (crates/og-tests-web) compile too.
#![allow(dead_code)]

mod scenes;

use og_render::RenderScene;
use og_ui::*;
pub use scenes::*;
use std::path::PathBuf;

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

/// og-tests' rule: a pixel is an outlier when a channel differs by more than
/// 32/255; the frame fails above 0.5 % outliers or an RGB RMSE above 0.01.
pub fn compare(actual: &[u8], reference: &[u8]) -> Result<(f64, f64), String> {
    if actual.len() != reference.len() {
        return Err(format!("size {} vs {}", actual.len(), reference.len()));
    }
    let mut sum = 0.0f64;
    let mut outliers = 0usize;
    let pixels = actual.len() / 4;
    for (a, r) in actual
        .as_chunks::<4>()
        .0
        .iter()
        .zip(reference.as_chunks::<4>().0.iter())
    {
        let mut out = false;
        for c in 0..3 {
            let d = (f64::from(a[c]) - f64::from(r[c])) / 255.0;
            sum += d * d;
            out |= a[c].abs_diff(r[c]) > 32;
        }
        outliers += usize::from(out);
    }
    let rmse = (sum / (pixels * 3) as f64).sqrt();
    let fraction = outliers as f64 / pixels as f64;
    let mut failures = Vec::new();
    if rmse > 0.01 {
        failures.push(format!("RMSE {rmse:.5} > 0.01"));
    }
    if fraction > 0.005 {
        failures.push(format!("{:.3}% outlier pixels > 0.5%", fraction * 100.0));
    }
    if failures.is_empty() {
        Ok((rmse, fraction))
    } else {
        Err(failures.join("; "))
    }
}

/// Where tests write what they produce: `$CARGO_TARGET_TMPDIR/og-ui`.
pub fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui");
    std::fs::create_dir_all(&dir).expect("out dir");
    dir
}
