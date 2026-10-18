//! Shaped scripts and right-to-left layout (E6 localization), through
//! og-ui's public boundary: `shape::visual` with the UI kit's Arabic font
//! (`templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf`) and a `Ui`
//! laid out right to left.
//!
//! Ways it could fail, and the test that catches each:
//! - Arabic letters drawn in their isolated forms (no shaping), or the
//!   shaped glyphs in logical instead of visual order →
//!   `arabic_words_are_shaped_into_joined_forms_in_visual_order`
//! - a right-to-left paragraph's words, or its digits, in the wrong order;
//!   a left-to-right paragraph's embedded Arabic run not reversed; Hebrew
//!   (unshaped) not reversed → `paragraphs_order_their_words_by_direction`
//! - a right-to-left UI not mirrored, or its text not aligned to the right
//!   → `a_right_to_left_ui_mirrors_its_layout`

use og_i18n::Direction;
use og_ui::shape::{GLYPH_PUA, Shaper, visual};
use og_ui::*;
use std::sync::Arc;

fn shaper() -> Arc<Shaper> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../templates/ui-kit/assets/fonts/NotoSansArabic-subset.ttf"
    );
    Shaper::new(Arc::new(
        std::fs::read(path).expect("the kit's Arabic font"),
    ))
    .expect("a font rustybuzz reads")
}

fn glyphs(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32 - GLYPH_PUA).collect()
}

#[test]
fn arabic_words_are_shaped_into_joined_forms_in_visual_order() {
    let sh = shaper();
    let all = |_c: char| true;
    // "مرحبا" (hello): مـ initial, ـر final, ح initial, ـبـ medial, ـا final.
    let word = visual("مرحبا", Direction::Ltr, Some(&sh), &all);
    assert!(
        word.chars().all(|c| c as u32 >= GLYPH_PUA),
        "every glyph is a shaped form: {word:?}"
    );
    let isolated: Vec<u32> = "مرحبا"
        .chars()
        .map(|c| glyphs(&visual(&c.to_string(), Direction::Ltr, Some(&sh), &all))[0])
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
    let ma = glyphs(&visual("ما", Direction::Ltr, Some(&sh), &all));
    assert_eq!(
        joined.last(),
        ma.last(),
        "both end, at the right, with the initial مـ"
    );
    assert_ne!(
        ma[1], isolated[0],
        "the initial form differs from the isolated"
    );
}

#[test]
fn paragraphs_order_their_words_by_direction() {
    let sh = shaper();
    let all = |_c: char| true;
    let ar = |w: &str| visual(w, Direction::Ltr, Some(&sh), &all);
    // A right-to-left paragraph: the first word is drawn rightmost; digits keep their order.
    let line = visual("الإصدار 0.1.0", Direction::Ltr, Some(&sh), &all);
    assert_eq!(line, format!("0.1.0 {}", ar("الإصدار")));
    let two = visual("مغامرة صغيرة", Direction::Ltr, Some(&sh), &all);
    assert_eq!(two, format!("{} {}", ar("صغيرة"), ar("مغامرة")));
    // A left-to-right paragraph keeps its order; its Arabic run reads right to left inside.
    let mixed = visual("Level: مغامرة صغيرة!", Direction::Ltr, Some(&sh), &all);
    assert!(mixed.starts_with("Level: "), "{mixed:?}");
    assert!(
        mixed.contains(&format!("{} {}", ar("صغيرة!"), ar("مغامرة"))),
        "{mixed:?}"
    );
    // Hebrew is not shaped, only reversed; without a shaper Arabic is reversed too.
    assert_eq!(visual("שלום", Direction::Ltr, None, &all), "םולש");
    assert_eq!(
        visual("plain text", Direction::Rtl, Some(&sh), &all),
        "plain text"
    );
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
