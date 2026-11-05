//! Line breaking of CJK text (E7): kinsoku and katakana words, through a
//! wrapping label's drawn lines.
//!
//! Ways it could fail, and the check that catches each:
//! - a line starts with closing punctuation, a small kana or the prolonged
//!   sound mark (`？` alone on the last line) → `kinsoku_keeps_...` (the
//!   question, the small kana)
//! - a line ends with an opening bracket → the same test (the quote)
//! - a katakana word split across lines (the kit's ja title) → the same
//!   test (the title)
//! - a katakana run longer than the line never breaks → the same test (the
//!   long run still fits the width)
//! - a word split between characters because a box's max was narrower
//!   than it (`Moonbla` / `de` in a card) → `a_fit_box_never_splits_a_word`
//! - Latin text or a CJK line with no rule to apply breaks differently
//!   than before → `text_without_a_rule_breaks_as_before`

use crate::support;

use og_ui::*;
use support::*;

/// The lines a label of `text` draws in `chars` character widths (every
/// glyph one em wide).
fn lines(text: &str, chars: f32) -> Vec<String> {
    let mut scene = og_render::RenderScene::new();
    let font = UiFont {
        metrics: FontMetrics::fixed(1.0),
        ..add_test_font(&mut scene)
    };
    let mut ui = ui_with_font(font);
    let size = ui.theme().type_ramp.body.size;
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.label(
        "Text",
        text,
        LabelDesc {
            width: Size::Fixed(chars * size + 0.01),
            ..LabelDesc::default()
        },
    );
    ui.end();
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn kinsoku_keeps_punctuation_and_small_kana_off_the_wrong_end_and_katakana_words_whole() {
    // The kit's ja title: the break falls after the middle dot, not inside クエスト.
    assert_eq!(lines("オリジン・クエスト", 7.0), ["オリジン・", "クエスト"]);
    // 13 characters in 12: the question mark never starts a line.
    assert_eq!(
        lines("このセーブを削除しますか？", 12.0),
        ["このセーブを削除します", "か？"]
    );
    // An opening bracket never ends a line ("押す「" before), a closing one
    // never starts one.
    assert_eq!(lines("押す「はい」を", 3.0), ["押す", "「は", "い」を"]);
    // Small kana and the prolonged sound mark stay with the kana before them.
    assert_eq!(lines("まちょっと", 3.0), ["ま", "ちょっ", "と"]);
    assert_eq!(lines("中文标点，好", 4.0), ["中文标", "点，好"]);
    // A katakana run longer than the line still breaks to fit.
    let long = lines("アイウエオカキクケコ", 4.0);
    assert!(
        long.len() >= 3 && long.iter().all(|l| l.chars().count() <= 4),
        "{long:?}"
    );
}

#[test]
fn text_without_a_rule_breaks_as_before() {
    assert_eq!(lines("one two three", 8.0), ["one two", "three"]);
    assert_eq!(lines("設定終了再開", 4.0), ["設定終了", "再開"]);
    assert_eq!(lines("한국어 설정", 3.0), ["한국어", "설정"]);
}

#[test]
fn unicode_line_breaks_keep_nbsp_and_allow_tabs_hyphens_slashes_and_zwsp() {
    assert_eq!(lines("ab\tcd", 3.0), ["ab", "cd"]);
    assert_eq!(lines("ab\u{200b}cd", 3.0), ["ab", "cd"]);
    assert_eq!(lines("a-bcd", 4.0), ["a-", "bcd"]);
    assert_eq!(lines("a/bcd", 4.0), ["a/", "bcd"]);
    assert_eq!(lines("x a\u{a0}b", 4.0), ["x", "a\u{a0}b"]);
}

#[test]
fn overflow_never_tears_an_extended_grapheme() {
    assert_eq!(lines("ae\u{301}b", 1.0), ["a", "e\u{301}", "b"]);
    assert_eq!(lines("a👩‍👩‍👧‍👦b", 1.0), ["a", "👩‍👩‍👧‍👦", "b"]);
}

#[test]
fn thai_breaks_at_dictionary_words_before_the_line_overflows() {
    // ภาษา (language), ไทย (Thai): the old char overflow cut ภาษาไ / ทย.
    assert_eq!(lines("ภาษาไทย", 5.0), ["ภาษา", "ไทย"]);
    assert_eq!(lines("ພາສາລາວ", 5.0), ["ພາສາ", "ລາວ"]);
    assert_eq!(lines("မြန်မာစာ", 7.0), ["မြန်မာ", "စာ"]);
    assert_eq!(lines("ភាសាខ្មែរភាសាខ្មែរ", 10.0), ["ភាសាខ្មែរ", "ភាសាខ្មែរ"]);
}

/// A wrapping label's lines in a column fitted between 0 and `max`
/// character widths.
fn lines_in_fit(text: &str, max: f32) -> Vec<String> {
    let mut scene = og_render::RenderScene::new();
    let font = UiFont {
        metrics: FontMetrics::fixed(1.0),
        ..add_test_font(&mut scene)
    };
    let mut ui = ui_with_font(font);
    let size = ui.theme().type_ramp.body.size;
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.open(
        "Card",
        Element {
            width: Size::Fit {
                min: 0.0,
                max: max * size + 0.01,
            },
            ..Element::column()
        },
    );
    ui.label(
        "Text",
        text,
        LabelDesc {
            width: Size::Grow {
                min: 0.0,
                max: f32::INFINITY,
            },
            ..LabelDesc::default()
        },
    );
    ui.close();
    ui.end();
    ui.frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_fit_box_never_splits_a_word() {
    // The box grows to its longest word, and the words still wrap between them.
    assert_eq!(
        lines_in_fit("Moonblade of dusk", 6.0),
        ["Moonblade", "of dusk"]
    );
    assert_eq!(lines_in_fit("one two three", 8.0), ["one two", "three"]);
}

#[test]
fn cooked_soft_hyphens_draw_only_at_a_taken_break() {
    let mut scene = og_render::RenderScene::new();
    let mut font = crate::features::font("Inter-Display.ttf");
    font.handle = add_test_font(&mut scene).handle;
    let m = font.metrics.clone();
    let mut ui = ui_with_font(font);
    let mut step = ui.theme().type_ramp.body;
    step.tracking_em = 0.0;
    let narrow = m.measure("ab-", step.size) + 0.01;
    let wide = m.measure("abcd", step.size) + 1.0;
    let no_hyphen = m.measure("ab", step.size).max(m.measure("cd", step.size)) + 0.01;
    assert!(no_hyphen < narrow);
    for (width, count) in [(narrow, 2), (wide, 1), (no_hyphen, 2)] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.label(
            "Text",
            "ab\u{ad}cd",
            LabelDesc {
                step: Some(step),
                width: Size::Fixed(width),
                ..Default::default()
            },
        );
        ui.end();
        let drawn: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Text { text, bounds, .. } => Some((text, bounds.width)),
                _ => None,
            })
            .collect();
        assert_eq!(drawn.len(), count, "{drawn:?}");
        assert!(drawn.iter().all(|(_, w)| *w <= width + 0.01), "{drawn:?}");
        if width == narrow {
            assert_eq!(drawn[0].0, "ab-");
        } else {
            assert!(!drawn[0].0.contains('-'));
        }
    }
}
