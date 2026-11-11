//! og-ui's test support (crates/og-ui/tests/support: the screens, the MSDF
//! test font, the document harness), shared by path, and beside it what a
//! frame test adds: the render scenes of the screens (`scenes.rs`, which
//! the browser tiers, crates/og-tests-web, compile too), the test font in
//! a scene, the frame a scene renders to ([`render_frame`], [`render_ui`])
//! and its check against a reference frame ([`matches_reference`]: where
//! the run's frame is saved, which reference it is judged against, what
//! `OG_UI_CAPTURE=1` records and og-tests' tolerance rules), so a frame
//! test states its scene, its size and its reference's name.
#![allow(dead_code)]

#[path = "../../../og-ui/tests/support/mod.rs"]
mod shared;
// `scenes` reaches `super::screens`, as it does in og-tests-web.
use shared as screens;
mod scenes;

use og_render::{RenderDesc, RenderInfo, RenderScene, RenderTarget, Renderer, RgbaImage};
use og_ui::*;
pub use scenes::*;
pub use shared::*;
use std::path::{Path, PathBuf};

/// Adds the MSDF test font (msdf-atlas-gen's PNG, CSV and JSON) to a scene;
/// the UI font for it, with the font's own ascender and descender.
pub fn add_test_font(scene: &mut RenderScene) -> UiFont {
    with_test_font(|files| add_font(scene, files))
}

/// A renderer description of `width` by `height` device pixels, the rest
/// as it defaults.
pub fn sized(width: u32, height: u32) -> RenderDesc {
    RenderDesc {
        width,
        height,
        ..RenderDesc::default()
    }
}

/// A frame a test rendered: its pixels, what the renderer counted drawing
/// it and the adapter that drew it.
pub struct Rendered {
    pub image: RgbaImage,
    pub stats: og_render::FrameStats,
    pub info: RenderInfo,
}

/// `scene` drawn offscreen and read back, by a renderer made for this one
/// frame: it starts with no frame history, so the frame is the scene's
/// alone.
pub fn render_frame(scene: &RenderScene, desc: &RenderDesc) -> Rendered {
    let mut renderer = Renderer::new_blocking(desc).expect("a headless adapter");
    let stats = renderer
        .render(scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    Rendered {
        image,
        stats,
        info: renderer.info().clone(),
    }
}

/// `ui`'s frame drawn as `scene`'s overlay at `width` by `height` device
/// pixels (1 per CSS px), over the theme's backdrop ([`on_backdrop`]).
pub fn render_ui(ui: &Ui, scene: &mut RenderScene, width: u32, height: u32) -> Rendered {
    let mut overlay = og_render::Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    on_backdrop(ui, scene);
    render_frame(scene, &sized(width, height))
}

/// `references/<tier>/`, where og-ui's reference frames are kept: in
/// crates/og-tests-ui, whichever crate compiles this file (og-tests-cooked
/// includes it by path).
pub fn reference_dir(tier: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .join("og-tests-ui/references")
        .join(tier)
}

/// Whether this run records reference frames (`OG_UI_CAPTURE=1`).
pub fn capturing() -> bool {
    std::env::var_os("OG_UI_CAPTURE").is_some()
}

/// Saves the frame a run rendered as `file` in [`out_dir`] (fast encoded:
/// a person reads it after a failure); where it is.
pub fn save_actual(file: &str, image: &RgbaImage) -> PathBuf {
    let path = out_dir().join(file);
    image::save_buffer(
        &path,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    path
}

/// Which reference a frame is judged against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Against {
    /// The recording tier's (`og_tests::compare::RECORDING_TIER`) on every
    /// tier: `references/vulkan/<name>.png`, which `OG_UI_CAPTURE=1`
    /// records. The run's frame is saved as `<name>.png`.
    Recorded,
    /// The rendering tier's own, `references/<tier>/<name>.png`, which
    /// `OG_UI_CAPTURE=1` records; the recording tier's where the tier has
    /// none. The run's frame is saved as `<name>-<tier>.png`.
    OwnTier,
}

/// `image` (rendered by `info`'s adapter) checked against the reference
/// frame `name`: saved in [`out_dir`], recorded under `OG_UI_CAPTURE=1`,
/// then compared with the reference `against` chooses, under [`compare`]'s
/// rule. Prints the difference it measured; answers what was exceeded, or
/// a reference of another size.
pub fn check_reference(
    name: &str,
    image: &RgbaImage,
    info: &RenderInfo,
    against: Against,
) -> Result<(), String> {
    let tier = info.backend_tier();
    let recorded = og_tests::compare::RECORDING_TIER;
    let file = format!("{name}.png");
    let (actual, own) = match against {
        Against::Recorded => (
            save_actual(&file, image),
            reference_dir(recorded).join(&file),
        ),
        Against::OwnTier => (
            save_actual(&format!("{name}-{tier}.png"), image),
            reference_dir(&tier).join(&file),
        ),
    };
    if capturing() {
        record_reference(&own, image.width, image.height, &image.pixels);
        println!("recorded {}", own.display());
    }
    let reference = if against == Against::Recorded || own.exists() {
        own
    } else {
        println!(
            "no {tier} reference for {name}; comparing with the {recorded} one (OG_UI_CAPTURE=1 on that tier records it)"
        );
        reference_dir(recorded).join(&file)
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    if expected.dimensions() != (image.width, image.height) {
        return Err(format!(
            "{} is {}x{}, {} is {}x{}",
            actual.display(),
            image.width,
            image.height,
            reference.display(),
            expected.width(),
            expected.height()
        ));
    }
    match compare(&image.pixels, expected.as_raw(), info, &reference) {
        Ok((rmse, outliers)) => {
            println!(
                "{name} {tier}: RMSE {rmse:.6}, outliers {:.3}% against {}",
                outliers * 100.0,
                reference.display()
            );
            Ok(())
        }
        Err(e) => Err(format!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        )),
    }
}

/// [`check_reference`], failing the test on a difference (at the test's
/// own line).
#[track_caller]
pub fn matches_reference(name: &str, image: &RgbaImage, info: &RenderInfo, against: Against) {
    if let Err(e) = check_reference(name, image, info, against) {
        panic!("{e}");
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
