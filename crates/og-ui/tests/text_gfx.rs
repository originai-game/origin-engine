//! og-ui's text frames, rendered headless with fonts the cook makes from the
//! UI kit's sources (`templates/ui-kit/assets/fonts`: kerning, shaped
//! scripts), compared with `references/<tier>/text-*.png` under og-tests'
//! tolerance (`OG_UI_CAPTURE=1` records this tier's). The scenes are
//! `support/text_scenes.rs` (what each frame shows), which the browser
//! tiers draw too: the fonts are the project `tests/text-fonts` cooked into
//! a scratch directory and read back from the blobs' bytes, as the browser
//! harness (packages/engine/tests/web/reference-gfx.ts) cooks the same
//! project with `og cook` and hands og-tests-web the bytes.
//!
//!
//! Ways they could fail: a frame that moves (a kerning pair or a placement
//! applied twice or not at all, a run in the wrong order, a mark off its
//! base), a missing glyph (asserted 0), or the kerned and unkerned rows
//! coming out the same width (asserted different).

use crate::support;

use og_cook::runtime::{Bundle, CookedFont};
use og_cook::{CookOptions, cook_project};
use og_render::{RenderDesc, RenderScene, RenderTarget, Renderer};
use std::path::{Path, PathBuf};
use support::text_scenes::{FONTS, TEXT_SCENES, fonts_from, text_scene};
use support::*;

/// Copies `from` (a directory) into `to`.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("dir");
    for e in std::fs::read_dir(from).expect("read dir") {
        let e = e.expect("entry");
        let dst = to.join(e.file_name());
        if e.file_type().expect("type").is_dir() {
            copy_dir(&e.path(), &dst);
        } else {
            std::fs::copy(e.path(), dst).expect("copy");
        }
    }
}

/// The fonts project (`tests/text-fonts` and the kit's font files) cooked
/// into a scratch directory, the fonts read back from the blobs' bytes
/// (one test: one cook).
fn cooked() -> Vec<CookedFont> {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-ui-text-fonts");
    let _ = std::fs::remove_dir_all(&root);
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    copy_dir(&here.join("tests/text-fonts"), &root);
    let src = here.join("../../templates/ui-kit/assets/fonts");
    for file in FONTS {
        std::fs::copy(src.join(file), root.join("assets/fonts").join(file)).expect("copy font");
    }
    let report = cook_project(&root, &CookOptions::default()).expect("cook");
    assert!(report.ok(), "{:?}", report.errors());
    let cooked = root.join("cooked");
    let manifest = std::fs::read_to_string(cooked.join(".cook.ogt")).expect("manifest");
    let bundle = Bundle::from_manifest(&manifest).expect("bundle");
    fonts_from(&bundle, |file| std::fs::read(cooked.join(file)).ok()).expect("fonts")
}

/// Renders a frame and compares it with its reference; answers the frame's
/// stats.
fn check(name: &str, (scene, desc): (RenderScene, RenderDesc)) -> og_render::FrameStats {
    let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
    let stats = renderer
        .render(&scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let tier = renderer.info().backend_tier();
    let actual = out_dir().join(format!("{name}-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    let refs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references");
    let own = refs.join(&tier).join(format!("{name}.png"));
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        println!("no {tier} reference for {name}; comparing with metal");
        refs.join("metal").join(format!("{name}.png"))
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    match compare(&image.pixels, expected.as_raw()) {
        Ok((rmse, outliers)) => println!(
            "{name} {tier}: RMSE {rmse:.5}, outliers {:.3}%",
            outliers * 100.0
        ),
        Err(e) => panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        ),
    }
    stats
}

#[test]
fn text_frames_match_their_references() {
    let fonts = cooked();
    assert!(
        fonts[0].kerning.len() > 1000 && fonts[1].kerning.len() > 1000,
        "the display and body faces kern"
    );
    for name in TEXT_SCENES {
        let stats = check(name, text_scene(name, &fonts).expect("a text frame"));
        assert_eq!(stats.missing_glyphs, 0, "{name}");
    }
}
