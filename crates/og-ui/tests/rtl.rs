//! Shaped scripts and right-to-left layout (E6 localization), through
//! og-ui's public boundary: `FontMetrics::visual` with the UI kit's Arabic font
//! (`templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf`) and a `Ui`
//! laid out right to left.
//!
//! Ways it could fail, and the test that catches each:
//! - Arabic letters drawn in their isolated forms (no shaping), or the
//!   shaped glyphs in logical instead of visual order →
//!   `arabic_words_are_shaped_into_joined_forms_in_visual_order`
//! - a right-to-left paragraph's words, or its digits, in the wrong order;
//!   a left-to-right paragraph's embedded Arabic run not reversed; Hebrew
//!   (unshaped) not reversed; brackets not mirrored; neutrals not resolved
//!   by UAX #9 → `paragraphs_order_their_runs_by_the_bidi_algorithm`
//! - kerning ignored, or measured and drawn differently →
//!   `kerning_pairs_tighten_the_measure`
//! - a right-to-left UI not mirrored, or its text not aligned to the right
//!   → `a_right_to_left_ui_mirrors_its_layout`

use og_i18n::Direction;
use og_ui::shape::{FontFile, GLYPH_PUA};
use og_ui::*;
use std::sync::Arc;

fn arabic() -> FontMetrics {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    );
    let file = FontFile::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads");
    // Fixed metrics have every glyph, so every shaped form is drawable.
    FontMetrics::fixed(0.5).with_file(Some(file))
}

/// A line as drawn: its visual glyphs' characters.
fn visual(m: &FontMetrics, s: &str) -> String {
    m.visual(s, Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect()
}

fn glyphs(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32 - GLYPH_PUA).collect()
}

#[test]
fn arabic_words_are_shaped_into_joined_forms_in_visual_order() {
    let m = arabic();
    // "مرحبا" (hello): مـ initial, ـر final, ح initial, ـبـ medial, ـا final.
    let word = visual(&m, "مرحبا");
    assert!(
        word.chars().all(|c| c as u32 >= GLYPH_PUA),
        "every glyph is a shaped form: {word:?}"
    );
    let isolated: Vec<u32> = "مرحبا"
        .chars()
        .map(|c| glyphs(&visual(&m, &c.to_string()))[0])
        .collect();
    let joined = glyphs(&word);
    assert!(joined.len() >= 5, "{joined:?}");
    let forms = joined.iter().filter(|g| !isolated.contains(g)).count();
    assert!(
        forms >= 3,
        "most letters take a joined form ({forms} of {}): {joined:?} vs {isolated:?}",
        joined.len()
    );
    // Visual order: the word's first letter (مـ, initial) is drawn last, at the right,
    // as it is in "ما".
    let ma = glyphs(&visual(&m, "ما"));
    assert_eq!(
        joined.last(),
        ma.last(),
        "both end, at the right, with the initial مـ"
    );
    assert_ne!(
        ma[1], isolated[0],
        "the initial form differs from the isolated"
    );
    // The glyphs carry where they were in the text: the rightmost is the first character.
    let g = m.visual("مرحبا", Direction::Ltr, 0.0);
    assert_eq!(g.last().map(|g| g.index), Some(0));
    assert!(g.iter().all(|g| g.rtl));
}

#[test]
fn paragraphs_order_their_runs_by_the_bidi_algorithm() {
    let m = arabic();
    let ar = |w: &str| visual(&m, w);
    let space = visual(&m, "ا ا").chars().nth(1).expect("a shaped space");
    // A right-to-left paragraph: the first word is drawn rightmost; digits keep their order.
    let line = visual(&m, "الإصدار 0.1.0");
    assert_eq!(line, format!("0.1.0{space}{}", ar("الإصدار")));
    // One right-to-left run is shaped whole: its words from right to left.
    let two = visual(&m, "مغامرة صغيرة");
    assert_eq!(two, format!("{}{space}{}", ar("صغيرة"), ar("مغامرة")));
    // A left-to-right paragraph keeps its order; its Arabic run reads right
    // to left inside, and the final "!" (neutral, at the paragraph's end)
    // takes the paragraph's direction (UAX #9 N1/N2).
    let mixed = visual(&m, "Level: مغامرة صغيرة!");
    assert_eq!(mixed, format!("Level: {two}!"));
    // Brackets mirror inside a right-to-left run.
    let hebrew = FontMetrics::fixed(0.5);
    assert_eq!(
        hebrew
            .visual("שלום (עולם)", Direction::Ltr, 0.0)
            .iter()
            .map(|g| g.ch)
            .collect::<String>(),
        "(םלוע) םולש"
    );
    // Hebrew is not shaped, only reversed; without a font file Arabic is reversed too.
    assert_eq!(
        hebrew
            .visual("שלום", Direction::Ltr, 0.0)
            .iter()
            .map(|g| g.ch)
            .collect::<String>(),
        "םולש"
    );
    assert_eq!(visual(&m, "plain text"), "plain text");
    // A number inside a right-to-left paragraph stays left to right, and the
    // paragraph's own words around it run right to left (UAX #9 W/L rules).
    let h = hebrew
        .visual("שלב 12 מתוך 40", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.ch)
        .collect::<String>();
    assert_eq!(h, "40 ךותמ 12 בלש");
}

/// Kerning: the font's pairs move the pen between neighbours in measuring
/// and in the visual layout alike; a font without pairs measures as before.
#[test]
fn kerning_pairs_tighten_the_measure() {
    use og_render::{Glyph, Kern, MsdfFont, TextureHandle};
    let glyph = |c: char| Glyph {
        codepoint: c,
        advance: 0.6,
        plane_bounds: [0.0, 0.0, 0.5, 0.7],
        atlas_bounds: [0.0, 0.0, 1.0, 1.0],
    };
    let font = |kerning: Vec<Kern>| MsdfFont {
        atlas: TextureHandle::from_slot(1).expect("a handle"),
        distance_range: 4.0,
        line_height: 1.2,
        glyphs: "AVT o".chars().map(glyph).collect(),
        kerning,
    };
    let plain = FontMetrics::from_msdf(&font(Vec::new()));
    let kerned = FontMetrics::from_msdf(&font(vec![
        Kern {
            left: 'A',
            right: 'V',
            adjust: -0.1,
        },
        Kern {
            left: 'T',
            right: 'o',
            adjust: -0.05,
        },
    ]));
    assert_eq!(plain.measure("AVTo", 10.0), 4.0 * 0.6 * 10.0);
    assert!((kerned.measure("AVTo", 10.0) - (2.4 - 0.15) * 10.0).abs() < 1e-4);
    assert!(
        (kerned.measure("VA", 10.0) - 12.0).abs() < 1e-4,
        "pairs are ordered"
    );
    let sum: f32 = kerned
        .visual("AVTo", Direction::Ltr, 0.0)
        .iter()
        .map(|g| g.advance)
        .sum();
    assert!((sum * 10.0 - kerned.measure("AVTo", 10.0)).abs() < 1e-4);
}

#[test]
fn a_right_to_left_ui_mirrors_its_layout() {
    let frame = |dir: Direction| {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_direction(dir);
        let vp = Viewport {
            width: 400.0,
            height: 200.0,
            dpr: 1.0,
            ..Viewport::default()
        };
        ui.begin(InputFrame::default(), vp);
        ui.row(
            "Bar",
            Element {
                width: Size::grow(),
                gap: 10.0,
                ..Element::default()
            },
            |ui| {
                ui.label("First", "one", LabelDesc::default());
                ui.label("Second", "two", LabelDesc::default());
            },
        );
        ui.end().clone()
    };
    let ltr = frame(Direction::Ltr);
    let rtl = frame(Direction::Rtl);
    let x = |f: &UiFrame, id: &str| f.element(id).unwrap().x;
    let w = |f: &UiFrame, id: &str| f.element(id).unwrap().width;
    assert!(x(&ltr, "First") < x(&ltr, "Second"));
    assert!(
        x(&rtl, "First") > x(&rtl, "Second"),
        "right to left, the first element is rightmost"
    );
    for id in ["First", "Second", "Bar"] {
        assert!(
            (x(&rtl, id) - (400.0 - x(&ltr, id) - w(&ltr, id))).abs() < 0.01,
            "{id} is mirrored about the centre"
        );
    }
}
