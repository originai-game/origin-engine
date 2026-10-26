//! The game widget set rendered headless and compared with
//! `references/<tier>/widget-set.png` (og-tests' tolerance, as `tests/gfx.rs`;
//! `OG_UI_CAPTURE=1` records). Covers what only drawing shows: cooldown pies
//! made of rotated half discs at 0, a quarter, past half and full; checkbox
//! ticks and chevrons as rotated bars; meter trails, notches and labels;
//! focus rings with their gap; the d-pad, stick and action buttons; a dialog
//! over a scrim with an open select menu over it; a toast.

use crate::support;

use og_render::{RenderTarget, Renderer};
use og_ui::*;
use std::path::PathBuf;
use support::*;

#[test]
fn the_widget_set_matches_its_reference_frame() {
    let shot = with_test_font(widget_set_shot);
    let frame = shot.ui.frame();
    assert!(frame.warnings.is_empty(), "{:?}", frame.warnings);
    assert!(
        frame.element("Difficulty.menu/Option#2").is_some(),
        "the menu is open"
    );
    assert!(frame.element("Toasts/Toast#1").is_some(), "the toast is up");

    // The pies: done 0.25 is the right half and a track half turned a
    // quarter; done 0.6 is the right half and a fill half turned 0.6 turn
    // less a half (the rings of 1 and 0 remaining draw no halves).
    let mut angles: Vec<f32> = frame
        .items
        .iter()
        .filter_map(|i| match i.cmd {
            DrawCmd::Half { angle, .. } => Some(angle),
            _ => None,
        })
        .collect();
    angles.sort_by(f32::total_cmp);
    let pi = std::f32::consts::PI;
    let expected = [0.0, 0.0, 0.2 * pi, 0.5 * pi];
    assert!(
        angles.len() == 4
            && angles
                .iter()
                .zip(expected)
                .all(|(a, e)| (a - e).abs() < 1e-4),
        "half-disc angles {angles:?}, expected {expected:?}"
    );

    let mut renderer = Renderer::new_blocking(&shot.desc).expect("a headless adapter");
    let stats = renderer
        .render(&shot.scene, &RenderTarget::Offscreen)
        .expect("render");
    let image = renderer.read_offscreen().expect("read back");
    let tier = renderer.info().backend_tier();
    let actual = out_dir().join(format!("widget-set-{tier}.png"));
    image::save_buffer(
        &actual,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    println!(
        "sprites {} glyphs {} sprite draws {} levels {} missing glyphs {}",
        stats.sprites,
        stats.glyphs,
        stats.sprite_draws,
        frame.levels(),
        stats.missing_glyphs
    );
    assert_eq!(stats.missing_glyphs, 0);

    let dir = |tier: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(tier)
    };
    let own = dir(&tier).join("widget-set.png");
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        std::fs::create_dir_all(own.parent().expect("dir")).expect("reference dir");
        std::fs::copy(&actual, &own).expect("record reference");
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        println!(
            "no {tier} reference; comparing with the metal one. Record it with OG_UI_CAPTURE=1 on that tier"
        );
        dir("metal").join("widget-set.png")
    };
    let expected = image::open(&reference)
        .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", reference.display()))
        .to_rgba8();
    assert_eq!(
        (expected.width(), expected.height()),
        (image.width, image.height)
    );
    match compare(&image.pixels, expected.as_raw()) {
        Ok((rmse, outliers)) => println!(
            "{tier}: RMSE {rmse:.5}, outliers {:.3}% against {}",
            outliers * 100.0,
            reference.display()
        ),
        Err(e) => panic!(
            "{} differs from {}: {e}",
            actual.display(),
            reference.display()
        ),
    }
}
