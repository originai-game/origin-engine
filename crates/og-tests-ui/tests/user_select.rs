//! The selection: the frames og-render draws, against `references/<tier>/`.
//! What og-ui lays out and builds for them is checked in
//! crates/og-ui/tests/user_select.rs.

use crate::support;
use support::*;

/// `references/<tier>/user-select-<width>.png` (`support::user_select_scene`,
/// which Chrome draws too: tests/web/reference-gfx.ts): a drag's selection
/// over a wrapped paragraph and into the next at 1280 and 390 wide.
#[test]
#[ignore = "requires GPU"]
fn the_selection_matches_its_reference_frames() {
    use og_render::{RenderTarget, Renderer};
    with_test_font(|files| {
        for name in ["user-select-1280", "user-select-390"] {
            let (scene, desc) = reference_scene(name, files).unwrap();
            let mut renderer = Renderer::new_blocking(&desc).expect("a headless adapter");
            let stats = renderer.render(&scene, &RenderTarget::Offscreen).unwrap();
            assert_eq!(stats.missing_glyphs, 0);
            let image = renderer.read_offscreen().unwrap();
            let tier = renderer.info().backend_tier();
            let actual = out_dir().join(format!("{name}-{tier}.png"));
            image::save_buffer(
                &actual,
                &image.pixels,
                image.width,
                image.height,
                image::ColorType::Rgba8,
            )
            .unwrap();
            let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references");
            let own = dir.join(&tier).join(format!("{name}.png"));
            if std::env::var_os("OG_UI_CAPTURE").is_some() {
                record_reference(&own, image.width, image.height, &image.pixels);
            }
            let reference = if own.exists() {
                own
            } else {
                dir.join(og_tests::compare::RECORDING_TIER)
                    .join(format!("{name}.png"))
            };
            let expected = image::open(&reference)
                .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1)", reference.display()))
                .to_rgba8();
            match compare(
                &image.pixels,
                expected.as_raw(),
                renderer.info(),
                &reference,
            ) {
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
        }
    });
}
