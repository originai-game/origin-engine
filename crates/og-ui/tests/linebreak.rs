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
//! - Latin text or a CJK line with no rule to apply breaks differently
//!   than before → `text_without_a_rule_breaks_as_before`

mod support;

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
