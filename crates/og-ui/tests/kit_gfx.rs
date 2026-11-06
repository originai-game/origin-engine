//! The game widget set rendered headless and compared with
//! `references/<tier>/widget-set.png` (og-tests' tolerance, as `tests/gfx.rs`;
//! `OG_UI_CAPTURE=1` records). Covers what only drawing shows: cooldown pies
//! made of rotated half discs at 0, a quarter, past half and full; checkbox
//! ticks and chevrons as rotated bars; meter trails, notches and labels;
//! focus rings with their gap; the d-pad, stick and action buttons; a dialog
//! over a scrim with an open select menu over it; a toast.

use crate::support;

use og_core::glam::Vec2;
use og_render::{RenderScene, RenderTarget, Renderer};
use og_ui::*;
use std::path::PathBuf;
use support::*;

#[test]
#[ignore = "requires GPU"]
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

    matches_reference("widget-set.png", &tier, &actual, &image);
}

/// `image` against `references/<tier>/<name>` (the metal one where the tier
/// has none); `OG_UI_CAPTURE=1` records it. `actual` is where it was saved.
fn matches_reference(
    name: &str,
    tier: &str,
    actual: &std::path::Path,
    image: &og_render::RgbaImage,
) {
    let dir = |tier: &str| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(tier)
    };
    let own = dir(tier).join(name);
    if std::env::var_os("OG_UI_CAPTURE").is_some() {
        record_reference(&own, image.width, image.height, &image.pixels);
        println!("recorded {}", own.display());
    }
    let reference = if own.exists() {
        own
    } else {
        println!(
            "no {tier} reference; comparing with the metal one. Record it with OG_UI_CAPTURE=1 on that tier"
        );
        dir("metal").join(name)
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

/// The on-screen stick at rest and held by a finger pushed past its rim up
/// and to the right, 240×240 at 2×, with its answer.
fn stick_shot(font: &FontFiles, push: Option<Vec2>) -> (Shot, JoystickResponse) {
    let vp = Viewport {
        width: 240.0,
        height: 240.0,
        dpr: 2.0,
        coarse_pointer: true,
        ..Viewport::default()
    };
    let mut scene = RenderScene::new();
    let font = add_font(&mut scene, font);
    let mut ui = ui_with_font(font);
    let mut out = JoystickResponse::default();
    let mut step = |ui: &mut Ui, input: InputFrame| {
        ui.begin(input, vp);
        ui.row(
            "Pad",
            Element {
                width: Size::grow(),
                height: Size::grow(),
                align: Align::Center,
                cross: Cross::Center,
                ..Element::default()
            },
            |ui| out = ui.joystick("Stick", JoystickDesc::default()),
        );
        ui.end();
    };
    step(&mut ui, InputFrame::default());
    if let Some(push) = push {
        let (x, y) = ui.frame().element("Pad/Stick").expect("the stick").center();
        let at = |p: Vec2, pressed: bool| InputFrame {
            pointer: p,
            pointer_inside: true,
            pressed,
            down: true,
            ..InputFrame::default()
        };
        step(&mut ui, at(Vec2::new(x, y), true));
        step(&mut ui, at(Vec2::new(x, y) + push, false));
        step(&mut ui, at(Vec2::new(x, y) + push, false));
    }
    (shot(ui, scene, vp), out)
}

/// A held stick draws its knob pushed toward the finger and held inside
/// the base however far the finger goes, with the rim lit toward the push,
/// against `references/<tier>/stick-held.png`; the frame at rest has its
/// knob centred.
#[test]
#[ignore = "requires GPU"]
fn a_held_stick_draws_its_knob_pushed_inside_its_base() {
    let render = |push: Option<Vec2>| {
        let (shot, out) = with_test_font(|f| stick_shot(f, push));
        let f = shot.ui.frame();
        assert!(f.warnings.is_empty(), "{:?}", f.warnings);
        let base = f.element("Pad/Stick").expect("the stick").clone();
        // The knob's rim disc (the widest round quad but the base) and the arcs.
        // A disc's corner radius is drawn a little under half its width (27.4
        // for a 55 px knob), so round is within half a pixel of it.
        let knob = f
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Quad { rect, radius, .. }
                    if *radius >= rect.width / 2.0 - 0.5 && rect.width < base.width - 1.0 =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .max_by(|a, b| a.width.total_cmp(&b.width))
            .expect("the knob");
        let arcs: Vec<(f32, f32)> = f
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Shape { shape, .. } => match shape.geometry {
                    og_render::UiGeometry::Arc { start, sweep, .. } => Some((start, sweep)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let mut renderer = Renderer::new_blocking(&shot.desc).expect("a headless adapter");
        renderer
            .render(&shot.scene, &RenderTarget::Offscreen)
            .expect("render");
        let image = renderer.read_offscreen().expect("read back");
        (image, out, base, knob, arcs, renderer.info().backend_tier())
    };
    let (_, still, base, knob, arcs, _) = render(None);
    let (bx, by) = base.center();
    let off = |k: &og_ui::Rect| Vec2::new(k.x + k.width / 2.0 - bx, k.y + k.height / 2.0 - by);
    assert!(!still.active && still.x == 0.0 && still.y == 0.0);
    assert!(
        off(&knob).length() < 0.01,
        "at rest the knob is centred: {knob:?}"
    );
    assert!(arcs.is_empty(), "at rest no rim is lit");

    // Pushed 100 CSS px up and right: past the 64 px rim, so full.
    let (held, out, _, knob, arcs, tier) = render(Some(Vec2::new(100.0, -100.0)));
    let full = (out.x * out.x + out.y * out.y).sqrt();
    assert!(out.active && out.x > 0.6 && out.y < -0.6, "{out:?}");
    assert!((full - 1.0).abs() < 1e-3, "clamped to the rim: {full}");
    let moved = off(&knob);
    let radius = base.width / 2.0;
    assert!(
        moved.x > 20.0 && (moved.x + moved.y).abs() < 0.5,
        "the knob moved {moved} along the push (45°)"
    );
    // The knob leans on the base's 2 px rim and never crosses it.
    let reach = moved.length() + knob.width / 2.0;
    assert!(
        reach <= radius - 2.0 && reach > radius - 4.0,
        "the knob reaches {reach} of the base's {radius}: inside its rim, against it"
    );
    // One arc of the rim, centred on the push.
    let [(start, sweep)] = arcs[..] else {
        panic!("one lit arc: {arcs:?}")
    };
    let mid = start + sweep / 2.0;
    assert!(
        (mid + std::f32::consts::FRAC_PI_4).abs() < 1e-3 && sweep > 0.5,
        "the rim lit toward the push: {start} + {sweep}"
    );
    let actual = out_dir().join(format!("stick-held-{tier}.png"));
    image::save_buffer(
        &actual,
        &held.pixels,
        held.width,
        held.height,
        image::ColorType::Rgba8,
    )
    .expect("write frame");
    matches_reference("stick-held.png", &tier, &actual, &held);
}
