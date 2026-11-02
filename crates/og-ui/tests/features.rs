//! OpenType features of a type step (docs/ui.md §15.5.4): a run with
//! features is shaped with its font file, so it draws the feature's glyphs,
//! and it is measured exactly as it is drawn.
//!
//! Ways it could fail, each covered below:
//! - the features never reach the shaper, so tabular figures keep their
//!   proportional advances → `tabular_figures_share_one_advance`
//! - measuring and drawing take different paths (one shaped, one not, or
//!   one tracked and the other not), so a small-caps label is cut or
//!   overflows its box → `a_small_caps_line_measures_as_it_draws`

use og_i18n::Direction;
use og_render::{Glyph, MsdfFont, TextureHandle};
use og_ui::shape::{FontFile, GLYPH_PUA, Features};
use og_ui::*;
use std::sync::Arc;

/// A kit font as the cook makes it with `features`: its printable ASCII by
/// code point and every glyph by id (U+F0000 + id), advances from `hmtx`.
fn font(file: &str) -> UiFont {
    let path = format!("{}/../../templates/ui-kit/assets/fonts/{file}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).expect("a kit font");
    let face = ttf_parser::Face::parse(&bytes, 0).expect("a font ttf-parser reads");
    let em = f32::from(face.units_per_em());
    let advance = |g: ttf_parser::GlyphId| f32::from(face.glyph_hor_advance(g).unwrap_or(0)) / em;
    let glyph = |codepoint: char, advance: f32| Glyph {
        codepoint,
        advance,
        plane_bounds: [0.0, 0.0, advance, 0.7],
        atlas_bounds: [0.0, 0.0, 1.0, 1.0],
    };
    let mut glyphs: Vec<Glyph> = (0x20u8..0x7f)
        .map(char::from)
        .filter_map(|c| Some(glyph(c, advance(face.glyph_index(c)?))))
        .collect();
    glyphs.extend((0..face.number_of_glyphs()).filter_map(|g| {
        Some(glyph(char::from_u32(GLYPH_PUA + u32::from(g))?, advance(ttf_parser::GlyphId(g))))
    }));
    let msdf = MsdfFont {
        atlas: TextureHandle::from_slot(1).expect("a handle"),
        distance_range: 4.0,
        line_height: 1.2,
        glyphs,
        kerning: Vec::new(),
    };
    let file = FontFile::new(Arc::new(bytes)).expect("a font rustybuzz reads");
    // No render handle: the pieces are laid out, not drawn.
    UiFont {
        handle: None,
        metrics: FontMetrics::from_msdf(&msdf).with_file(Some(file)),
    }
}

fn feature(tags: &[&str]) -> Features {
    Features::parse(tags.iter().copied()).expect("known features")
}

/// Inter's default figures are proportional (1 narrower than 8); `tnum`
/// gives every figure one advance, in measuring and in the drawn glyphs.
#[test]
fn tabular_figures_share_one_advance() {
    let m = font("Inter-Display.ttf").metrics;
    let width = |s: &str, f| m.measure_with(s, 100.0, 0.0, f);
    assert!(
        (width("1", Features::NONE) - width("8", Features::NONE)).abs() > 10.0,
        "the fixture's default figures are proportional"
    );
    let tnum = feature(&["tnum"]);
    assert!(
        (width("1", tnum) - width("8", tnum)).abs() < 1e-3,
        "tabular 1 is {} and 8 is {}",
        width("1", tnum),
        width("8", tnum)
    );
    // A counter does not shift as its digits change.
    assert!((width("1111", tnum) - width("8808", tnum)).abs() < 1e-3);
    let drawn = |s: &str| -> f32 {
        m.visual_with(s, Direction::Ltr, 0.0, tnum)
            .iter()
            .map(|g| g.advance)
            .sum()
    };
    assert!((drawn("1") - drawn("8")).abs() < 1e-6);
}

/// With `smcp` (and tracking) a line draws the font's small-cap glyphs,
/// and its measured width is the width of what is drawn.
#[test]
fn a_small_caps_line_measures_as_it_draws() {
    let f = font("EBGaramond-Medium.ttf");
    let smcp = feature(&["smcp"]);
    let line = "Moonlit Tide 3d 5h";
    for tracking in [0.0, 0.08] {
        let pieces = pieces_with(&f, line, Direction::Ltr, tracking, smcp);
        assert_eq!(pieces.len(), 1);
        let p = &pieces[0];
        assert_eq!(p.placements.len(), p.text.chars().count(), "placed glyph by glyph");
        let drawn: f32 = p.placements.iter().map(|g| g.advance).sum::<f32>() * 20.0;
        let measured = f.metrics.measure_with(line, 20.0, tracking, smcp);
        assert!(
            (drawn - measured).abs() < 1e-3,
            "tracking {tracking}: drawn {drawn}, measured {measured}"
        );
        // The lower case became small caps: other glyphs, other widths.
        let plain = pieces_with(&f, line, Direction::Ltr, tracking, Features::NONE);
        assert_ne!(p.text, plain[0].text, "the small-cap forms are drawn");
        let plain_w = f.metrics.measure_with(line, 20.0, tracking, Features::NONE);
        assert!((plain_w - measured).abs() > 1.0, "small caps change the width");
    }
}
