//! Text on a path (`-og-text-path`, docs/ui.md §2): SVG 2's `<textPath>`
//! layout, each grapheme cluster turned to the path where its advance's
//! middle sits, its baseline on the path.
//!
//! Ways it could fail, each covered below:
//! - a glyph placed by its pen instead of its middle (the run drifts a
//!   half advance along the curve), turned the wrong way, or the circle
//!   started anywhere but its top and run anticlockwise →
//!   `each_glyph_sits_on_the_circle_turned_to_it`; so is a centred
//!   circle's box widened to its column's, which moves its centre
//! - glyphs past an open path's end drawn (piled at the end) instead of
//!   dropped, text-align not anchoring at the offset, a percent offset
//!   read as px → `an_open_path_drops_what_runs_off_it_and_anchors_by_align`
//! - `side: right` not running the path backwards →
//!   `the_right_side_runs_the_path_backwards`
//! - SVG path data flattened wrong (an arc's sweep or size flags, a
//!   relative command from the wrong point, a smooth curve's reflected
//!   control), or `offset-path`'s shapes taken silently →
//!   `path_data_and_its_errors` (and the unit tests in src/text_path.rs)
//! - the text painted again on frames where nothing changed →
//!   `an_unchanged_path_text_is_not_painted_again`
//! - its look → `text_on_a_path_matches_its_reference_frames` (GPU)

use crate::support;

use og_ui::*;
use serde_json::json;
use support::*;

/// Each glyph command of the frame: its pen, size, advance (from its
/// bounds) and the turn its effects give it.
fn glyphs(ui: &Ui) -> Vec<(f32, f32, f32, f32, f32)> {
    ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::StyledText { text, effects } => match &**text {
                DrawCmd::Text {
                    x,
                    baseline,
                    size,
                    bounds,
                    ..
                } => Some((
                    *x,
                    *baseline,
                    bounds.width - 2.0 * size,
                    effects.transform[1].atan2(effects.transform[0]),
                    *size,
                )),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn frame(css: &str, text: &str) -> Ui {
    let doc = format!(
        "<meta name=\"component\" content=\"tp\">\n<column id=\"Root\"><p id=\"Ring\">{text}</p></column>\n<style>\n#Root {{ padding: 20px; }}\n#Ring {{ {css} }}\n</style>"
    );
    // A font with a handle: glyphs are drawn, not only measured.
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.define_document(UiDocument::parse("ui/tp.svelte", &doc).expect("parses"));
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.component("T", "tp", &mut json!({}), &bind::NoEnv);
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    ui
}

fn ring(ui: &Ui) -> UiElement {
    ui.element("Ring").cloned().expect("Ring")
}

#[test]
fn each_glyph_sits_on_the_circle_turned_to_it() {
    let ui = frame("-og-text-path: circle(100px);", "ABCDEFGH");
    let e = ring(&ui);
    // Its box is the circle's.
    assert_eq!((e.width, e.height), (200.0, 200.0));
    let c = (e.x + 100.0, e.y + 100.0);
    let g = glyphs(&ui);
    assert_eq!(g.len(), 8, "{g:?}");
    let mut last = -1.0f32;
    for (x, y, adv, angle, _) in &g {
        assert!(*adv > 1.0);
        // The advance's middle: on the circle.
        let mid = (x + angle.cos() * adv / 2.0, y + angle.sin() * adv / 2.0);
        let r = ((mid.0 - c.0).powi(2) + (mid.1 - c.1).powi(2)).sqrt();
        assert!((r - 100.0).abs() < 0.5, "{r} off the circle at {mid:?}");
        // Turned to the tangent there (clockwise on screen: the radius
        // turned a quarter).
        let radial = (mid.1 - c.1).atan2(mid.0 - c.0);
        let tangent = radial + std::f32::consts::FRAC_PI_2;
        let d = (angle - tangent).rem_euclid(std::f32::consts::TAU);
        assert!(
            d < 0.03 || d > std::f32::consts::TAU - 0.03,
            "{angle} vs {tangent}"
        );
        // From the top, clockwise: the angle from the top grows.
        let from_top = (mid.0 - c.0)
            .atan2(c.1 - mid.1)
            .rem_euclid(std::f32::consts::TAU);
        assert!(from_top > last, "{from_top} after {last}");
        last = from_top;
    }
    // The first glyph's pen is the circle's top: its middle half an advance on.
    let (x, y, _, _, _) = g[0];
    assert!(
        (x - c.0).abs() < 0.5 && (y - (c.1 - 100.0)).abs() < 0.5,
        "{x} {y}"
    );
    // Centred text in a column keeps the circle's box (a centred line
    // takes the column's width; a circle would centre off its text).
    let doc = "<meta name=\"component\" content=\"tp\">\n<column id=\"Root\"><p id=\"Ring\">ABCDEFGH</p></column>\n<style>\n#Root { width: 600px; }\n#Ring { -og-text-path: circle(100px); text-align: center; }\n</style>";
    let mut ui = ui_with_font(add_test_font(&mut og_render::RenderScene::new()));
    ui.define_document(UiDocument::parse("ui/tp.svelte", doc).expect("parses"));
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.component("T", "tp", &mut json!({}), &bind::NoEnv);
    ui.end();
    let e = ring(&ui);
    assert_eq!((e.width, e.height), (200.0, 200.0));
}

#[test]
fn an_open_path_drops_what_runs_off_it_and_anchors_by_align() {
    // A 100 px line: far more text than fits.
    let long = "MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM";
    let ui = frame("-og-text-path: path(\"M0 20 H100\");", long);
    let g = glyphs(&ui);
    assert!(!g.is_empty() && g.len() < long.len(), "{}", g.len());
    let x0 = ring(&ui).x;
    // Every middle on the line, none piled past its end.
    for (x, y, adv, angle, _) in &g {
        assert!(angle.abs() < 1e-4);
        let mid = x + adv / 2.0;
        assert!(mid >= x0 - 0.01 && mid <= x0 + 100.01, "{mid}");
        assert!((y - (ring(&ui).y + 20.0)).abs() < 0.01);
    }
    // Centred at 50 %: the run's middle is the line's.
    let ui = frame(
        "-og-text-path: path(\"M0 20 H300\"); -og-text-path-offset: 50%; text-align: center;",
        "ABC",
    );
    let g = glyphs(&ui);
    let (first, last) = (g[0], g[g.len() - 1]);
    let mid = (first.0 + last.0 + last.2) / 2.0;
    assert!((mid - (ring(&ui).x + 150.0)).abs() < 0.5, "{mid}");
    // End-aligned at 40px: the run ends there.
    let ui = frame(
        "-og-text-path: path(\"M0 20 H300\"); -og-text-path-offset: 40px; text-align: end;",
        "ABC",
    );
    let g = glyphs(&ui);
    let last = g[g.len() - 1];
    assert!((last.0 + last.2 - (ring(&ui).x + 40.0)).abs() < 0.5);
}

#[test]
fn the_right_side_runs_the_path_backwards() {
    let ui = frame(
        "-og-text-path: path(\"M0 20 H300\"); -og-text-path-side: right;",
        "AB",
    );
    let g = glyphs(&ui);
    // Upside down, from the line's right end leftwards.
    for (_, _, _, angle, _) in &g {
        assert!((angle.abs() - std::f32::consts::PI).abs() < 1e-3, "{angle}");
    }
    assert!(g[0].0 > g[1].0);
    assert!((g[0].0 - (ring(&ui).x + 300.0)).abs() < 0.5);
}

#[test]
fn path_data_and_its_errors() {
    // A half circle by an arc (sweep 1: clockwise on screen, through the
    // bottom), then a relative line: the box holds both.
    let ui = frame(
        "-og-text-path: path(\"M0 50 A50 50 0 0 1 100 50 l0 50\");",
        "AB",
    );
    let e = ring(&ui);
    assert_eq!((e.width, e.height), (100.0, 100.0));
    for (bad, says) in [
        ("-og-text-path: offset(10px);", "offset-path"),
        ("-og-text-path: path(\"L10 10\");", "starts with M"),
        (
            "-og-text-path: path(\"M0 0 A10 10 0 2 1 5 5\");",
            "arc flag",
        ),
        ("-og-text-path: circle(-3px);", "radius"),
        ("-og-text-path-offset: 3em;", "<n>px or <n>%"),
    ] {
        let doc = format!(
            "<meta name=\"component\" content=\"bad\">\n<p>x</p>\n<style>\np {{ {bad} }}\n</style>"
        );
        let err = UiDocument::parse("ui/bad.svelte", &doc)
            .unwrap_err()
            .join("\n");
        assert!(err.contains(says), "{bad}: {err}");
    }
}

#[test]
fn an_unchanged_path_text_is_not_painted_again() {
    let mut ui = frame("-og-text-path: circle(80px);", "Around the world");
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.component("T", "tp", &mut json!({}), &bind::NoEnv);
        ui.end();
    }
    assert_eq!(ui.frame().stats.painted, 0);
    assert!(!glyphs(&ui).is_empty(), "kept commands still draw");
}

/// `references/<tier>/text-path-<width>.png` (`support::text_path_scene`,
/// drawn in Chrome too by tests/web/reference-gfx.ts): a title on an arch,
/// a badge's ring of text and a wave at 1280 and 390 wide.
#[test]
#[ignore = "requires GPU"]
fn text_on_a_path_matches_its_reference_frames() {
    use og_render::{RenderTarget, Renderer};
    with_test_font(|files| {
        for name in ["text-path-1280", "text-path-390"] {
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
                dir.join("metal").join(format!("{name}.png"))
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
