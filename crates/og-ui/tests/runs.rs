//! A font per run inside one label: a text that mixes what the role's font
//! draws with what it does not is laid out run by run, each run from the
//! first of the UI's fonts that has it, and drawn with one text command per
//! font. The fonts: og-tests' MSDF test face cut to ASCII as the role's
//! (a Latin face), the whole face (ASCII and 原点引擎文字渲染) as a CJK
//! fallback, and the UI kit's Arabic file (shaped) after it.
//!
//! Ways it could fail, each covered below:
//! - one font per label again (the CJK fallback draws the Latin too), or a
//!   run from the wrong font → `each_run_takes_the_first_font_that_has_it`
//! - an Arabic run in a mixed label not shaped, or out of visual order →
//!   `an_arabic_run_is_shaped_with_its_own_font`
//! - spaces and punctuation start runs of their own (a lone comma in the
//!   Latin face between CJK) → `neutral_characters_stay_with_their_run`
//! - a locale's face, or a role's face that has everything, cut into runs
//!   → `a_text_one_font_sets_whole_is_not_cut`
//! - measured with one font and drawn with another (the runs overlap or
//!   gap), a missing glyph in a drawn command, a different frame on a
//!   second build → `a_mixed_label_draws_one_command_per_run`

mod support;

use og_i18n::Direction;
use og_render::RenderScene;
use og_ui::shape::{FontFile, GLYPH_PUA};
use og_ui::*;
use std::sync::Arc;
use support::*;

struct Faces {
    latin: UiFont,
    cjk: UiFont,
    arabic: UiFont,
}

/// The test face cut to ASCII (the role's), the whole face, and the kit's
/// Arabic file (fixed metrics: every shaped form is drawable).
fn faces(scene: &mut RenderScene) -> Faces {
    let (png, json, csv) = test_font_files();
    let ascii: String = csv
        .lines()
        .filter(|l| {
            l.split(',')
                .next()
                .and_then(|c| c.trim().parse::<u32>().ok())
                .is_some_and(|c| c < 0x80)
        })
        .map(|l| format!("{l}\n"))
        .collect();
    let latin = add_font(
        scene,
        &FontFiles {
            png: &png,
            json: &json,
            csv: &ascii,
        },
    );
    let cjk = add_font(
        scene,
        &FontFiles {
            png: &png,
            json: &json,
            csv: &csv,
        },
    );
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads");
    let arabic = UiFont {
        handle: cjk.handle,
        metrics: FontMetrics::fixed(0.5).with_file(Some(file)),
    };
    Faces { latin, cjk, arabic }
}

fn fonts(f: &Faces) -> Fonts {
    let mut fonts = Fonts {
        body: f.latin.clone(),
        fallbacks: vec![f.cjk.clone(), f.arabic.clone()],
        ..Fonts::default()
    };
    fonts.link();
    fonts
}

/// The font of each visual glyph.
fn runs(m: &FontMetrics, text: &str) -> Vec<u8> {
    m.visual(text, Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.font)
        .collect()
}

#[test]
fn each_run_takes_the_first_font_that_has_it() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let fonts = fonts(&f);
    let m = &fonts.for_text(FontRole::Body, "Score 原点 42").metrics;
    assert!(m.is_mixed(), "a Latin and CJK label is laid out per run");
    // "Score " from the role's face, 原点 from the first fallback (font 1),
    // the space after it stays with it, 42 back in the role's face.
    assert_eq!(runs(m, "Score 原点 42"), [0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0]);
    // The width is each run's own: no kerning or advance across fonts.
    let w = m.measure("Score 原点 42", 16.0);
    let parts = f.latin.metrics.measure("Score ", 16.0)
        + f.cjk.metrics.measure("原点 ", 16.0)
        + f.latin.metrics.measure("42", 16.0);
    assert!((w - parts).abs() < 1e-3, "{w} vs {parts} run by run");
    // No one face draws "原点 ☃" (the snowman is in none, the role's face
    // draws none of it): still a run per face, so only the snowman is missing.
    let mut cjk_only = Fonts {
        body: f.latin.clone(),
        fallbacks: vec![f.cjk.clone()],
        ..Fonts::default()
    };
    cjk_only.link();
    let m = &cjk_only.for_text(FontRole::Body, "原点 ☃").metrics;
    assert!(m.is_mixed());
    assert_eq!(runs(m, "原点 ☃"), [1, 1, 1, 1]);
    assert!(m.measure("原点", 16.0) > 0.0);
    // Without linking (a Fonts made by hand), one font for the label as before.
    let unlinked = Fonts {
        body: f.latin.clone(),
        fallbacks: vec![f.cjk.clone()],
        ..Fonts::default()
    };
    assert!(
        !unlinked
            .for_text(FontRole::Body, "Score 原点")
            .metrics
            .is_mixed()
    );
}

#[test]
fn an_arabic_run_is_shaped_with_its_own_font() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let fonts = fonts(&f);
    let m = &fonts.for_text(FontRole::Body, "Level 3 مرحبا").metrics;
    let glyphs = m.visual("Level 3 مرحبا", Direction::Ltr, 0.0);
    let latin: Vec<&VisualGlyph> = glyphs.iter().filter(|g| g.font == 0).collect();
    let arabic: Vec<&VisualGlyph> = glyphs.iter().filter(|g| g.font == 2).collect();
    assert_eq!(
        latin.iter().map(|g| g.ch).collect::<String>(),
        "Level 3 ",
        "the Latin, the digit and the spaces in the role's face"
    );
    assert!(
        arabic.len() >= 4 && arabic.iter().all(|g| g.ch as u32 >= GLYPH_PUA && g.rtl),
        "the Arabic run is shaped by the Arabic file: {arabic:?}"
    );
    // Visual order: the Latin first, then the Arabic run (its first letter last).
    assert!(glyphs[..8].iter().all(|g| g.font == 0));
    assert_eq!(
        glyphs.last().map(|g| g.index),
        Some(8),
        "مـ drawn at the right"
    );
    // The same shaping as the Arabic file alone gives the word.
    let alone: Vec<char> = f
        .arabic
        .metrics
        .visual("مرحبا", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect();
    assert_eq!(arabic.iter().map(|g| g.ch).collect::<Vec<_>>(), alone);
}

#[test]
fn neutral_characters_stay_with_their_run() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let fonts = fonts(&f);
    // The comma and space after 原点 are in both faces: they stay in the CJK
    // run; the leading bracket goes with the first letter's font.
    let text = "(原点, Engine)";
    let m = &fonts.for_text(FontRole::Body, text).metrics;
    assert_eq!(runs(m, text), [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn a_text_one_font_sets_whole_is_not_cut() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let mut fonts = fonts(&f);
    // The role's face has every character.
    assert!(
        !fonts
            .for_text(FontRole::Body, "Score 42")
            .metrics
            .is_mixed()
    );
    // A CJK text the role's face draws none of: the fallback, whole.
    let whole = fonts.for_text(FontRole::Body, "原点引擎");
    assert!(!whole.metrics.is_mixed());
    assert_eq!(whole.metrics, f.cjk.metrics.clone());
    // A locale's face (the chain) that sets the string whole keeps it, its
    // Latin included: a locale's strings are set in its face.
    fonts.chain = vec![f.cjk.clone()];
    fonts.link();
    let locale = fonts.for_text(FontRole::Body, "第 原点 42");
    assert!(locale.metrics.is_mixed(), "第 is in no face: per run");
    let locale = fonts.for_text(FontRole::Body, "原点 42");
    assert!(!locale.metrics.is_mixed(), "the locale's face has it all");
}

fn label_frame(f: &Faces, text: &str) -> UiFrame {
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: f.latin.clone(),
            fallbacks: vec![f.cjk.clone(), f.arabic.clone()],
            ..Fonts::default()
        },
        ..UiDesc::default()
    });
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 200.0));
        ui.label("Mixed", text, LabelDesc::default());
        ui.end();
    }
    ui.frame().clone()
}

#[test]
fn a_mixed_label_draws_one_command_per_run() {
    let mut scene = RenderScene::new();
    let f = faces(&mut scene);
    let frame = label_frame(&f, "Score 原点 42");
    let texts: Vec<(f32, String, og_render::FontHandle, usize)> = frame
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text {
                x,
                text,
                font,
                placements,
                ..
            } => Some((*x, text.clone(), *font, placements.len())),
            _ => None,
        })
        .collect();
    let (latin, cjk) = (f.latin.handle.unwrap(), f.cjk.handle.unwrap());
    let shown: Vec<(&str, og_render::FontHandle)> =
        texts.iter().map(|t| (t.1.as_str(), t.2)).collect();
    assert_eq!(
        shown,
        [("Score ", latin), ("原点 ", cjk), ("42", latin)],
        "a command per run, each with its run's font"
    );
    // Every character a command draws is in its font's atlas.
    for (_, text, font, n) in &texts {
        let m = if *font == latin { &f.latin } else { &f.cjk };
        assert!(text.chars().all(|c| m.metrics.in_atlas(c)), "{text:?}");
        assert_eq!(*n, text.chars().count(), "placed glyph by glyph");
    }
    // Each run starts where the run before it ends (16 px body text).
    let size = UiDesc::default().theme.type_ramp.body.size;
    let w0 = f.latin.metrics.measure("Score ", size);
    let w1 = f.cjk.metrics.measure("原点 ", size);
    assert!((texts[1].0 - texts[0].0 - w0).abs() < 1e-3, "{texts:?}");
    assert!((texts[2].0 - texts[1].0 - w1).abs() < 1e-3, "{texts:?}");
    // The label is as wide as its runs.
    let e = frame.element("Mixed").expect("label");
    assert!((e.width - (w0 + w1 + f.latin.metrics.measure("42", size))).abs() < 0.5);
    // A second UI built the same way draws the same frame.
    assert_eq!(label_frame(&f, "Score 原点 42").items, frame.items);
}
