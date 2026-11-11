//! og-ui's test support (crates/og-ui/tests/support: the screens, the MSDF
//! test font, the document harness), shared by path, and beside it what a
//! frame test adds: the render scenes of the screens (`scenes.rs`, which
//! the browser tiers, crates/og-tests-web, compile too), the test font in
//! a scene and the reference-frame comparison (og-tests' tolerance rules).
#![allow(dead_code)]

#[path = "../../../og-ui/tests/support/mod.rs"]
mod shared;
// `scenes` reaches `super::screens`, as it does in og-tests-web.
use shared as screens;
mod scenes;

use og_render::RenderScene;
use og_ui::*;
pub use scenes::*;
pub use shared::*;
use std::path::Path;

/// Adds the MSDF test font (msdf-atlas-gen's PNG, CSV and JSON) to a scene;
/// the UI font for it, with the font's own ascender and descender.
pub fn add_test_font(scene: &mut RenderScene) -> UiFont {
    with_test_font(|files| add_font(scene, files))
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
