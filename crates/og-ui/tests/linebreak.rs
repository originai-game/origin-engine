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
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.label(
        "Text",
        "ab\u{ad}\n\ncd\n",
        LabelDesc {
            step: Some(step),
            width: Size::Fixed(wide),
            ..Default::default()
        },
    );
    ui.end();
    assert_eq!(
        ui.frame().element("Text").unwrap().height,
        step.line_height * 4.0
    );
}

#[test]
fn rich_soft_hyphens_reserve_the_styled_glyph_at_a_break() {
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
    ui.define_action(ActionDesc {
        name: "jump".into(),
        keys: vec!["KeyE".into()],
        ..Default::default()
    });
    for (width, hyphen) in [(narrow, true), (wide, false), (no_hyphen, false)] {
        ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
        ui.rich_text(
            "Text",
            "a[color=danger]b\u{ad}[/color]cd[action=jump]",
            RichTextDesc {
                step: Some(step),
                width: Size::Fixed(width),
                ..Default::default()
            },
        );
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let drawn: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Text {
                    text,
                    color,
                    x,
                    bounds,
                    ..
                } => Some((text, color, x, bounds)),
                _ => None,
            })
            .collect();
        assert_eq!(
            drawn.iter().any(|(s, _, _, _)| s.ends_with('-')),
            hyphen,
            "{drawn:?}"
        );
        if hyphen {
            let (_, color, _, _) = drawn.iter().find(|(s, _, _, _)| s.ends_with('-')).unwrap();
            assert_eq!(**color, ui.theme().palette.danger.linear());
        }
        assert_eq!(
            drawn
                .iter()
                .filter(|(s, _, _, _)| s.as_str() == "E")
                .count(),
            1,
            "the inline keycap is not split or lost: {drawn:?}"
        );
        let left = ui.frame().element("Text").unwrap().x;
        assert!(
            drawn
                .iter()
                .all(|(_, _, _, bounds)| bounds.x + bounds.width <= left + width + 0.02),
            "{drawn:?}"
        );
    }
}

#[test]
#[ignore = "scaling measurement: run alone with --nocapture"]
fn long_kerned_line_measurement_is_not_quadratic() {
    use og_render::{Glyph, Kern, MsdfFont, TextureHandle};
    use std::time::Instant;
    // Every CJK character is a break unit. Summed isolated advances exceed
    // the box, but the kerned line fits: the old fits_whole loop reshaped
    // each growing prefix here. No OpenType file or layout cache hides it.
    let font = UiFont {
        handle: Some(og_render::FontHandle::from_bits(1)),
        metrics: FontMetrics::from_msdf(&MsdfFont {
            atlas: TextureHandle::from_slot(1).unwrap(),
            distance_range: 4.0,
            line_height: 1.2,
            glyphs: vec![Glyph {
                codepoint: '漢',
                advance: 1.0,
                plane_bounds: [0.0, 0.0, 1.0, 0.7],
                atlas_bounds: [0.0, 0.0, 1.0, 1.0],
            }],
            kerning: vec![Kern {
                left: '漢',
                right: '漢',
                adjust: -0.99,
            }],
        }),
    };
    let measure = |n: usize| {
        let mut ui = ui_with_font(font.clone());
        let size = ui.theme().type_ramp.body.size;
        let width = (n as f32 * 0.02 + 1.0) * size;
        let mut step = ui.theme().type_ramp.body;
        step.tracking_em = 0.0;
        ui.begin(InputFrame::default(), Viewport::new(4000.0, 600.0));
        let text = "漢".repeat(n);
        ui.label(
            "Text",
            &text,
            LabelDesc {
                step: Some(step),
                width: Size::Fixed(width),
                ..Default::default()
            },
        );
        let start = Instant::now();
        ui.end();
        let elapsed = start.elapsed().as_secs_f64();
        let lines: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Text { text, bounds, .. } => Some((text, bounds)),
                _ => None,
            })
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, &text);
        assert!(lines[0].1.width <= width);
        elapsed
    };
    measure(2000);
    measure(8000);
    let (mut small, mut large) = (Vec::new(), Vec::new());
    for i in 0..7 {
        if i % 2 == 0 {
            small.push(measure(2000));
            large.push(measure(8000));
        } else {
            large.push(measure(8000));
            small.push(measure(2000));
        }
    }
    small.sort_by(f64::total_cmp);
    large.sort_by(f64::total_cmp);
    let ratio = large[3] / small[3];
    println!(
        "2000 characters seconds {small:?}; 8000 characters seconds {large:?}; median ratio {ratio}"
    );
    assert!(
        ratio < 10.0,
        "4x input must stay below quadratic 16x growth: {ratio}"
    );
}

#[test]
fn rich_discretionary_lines_keep_rtl_and_empty_paragraphs() {
    let mut scene = og_render::RenderScene::new();
    let mut font = crate::features::font("NotoSansArabic-subset.ttf");
    font.handle = add_test_font(&mut scene).handle;
    let metrics = font.metrics.clone();
    let mut latin = crate::features::font("Inter-Display.ttf");
    latin.handle = font.handle;
    let hyphen = latin.metrics.clone();
    let mut ui = Ui::new(UiDesc {
        fonts: Fonts {
            body: font,
            fallbacks: vec![latin],
            ..Default::default()
        },
        ..Default::default()
    });
    let mut step = ui.theme().type_ramp.body;
    step.tracking_em = 0.0;
    let width = metrics.measure("مرحبا", step.size) + hyphen.measure("-", step.size) + 0.01;
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.rich_text(
        "Text",
        "[color=danger]مرحبا\u{ad}[/color]بك\n\n",
        RichTextDesc {
            step: Some(step),
            width: Size::Fixed(width),
            ..Default::default()
        },
    );
    ui.end();
    let lines: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Text {
                text,
                bounds,
                color,
                ..
            } => Some((text, bounds, color)),
            _ => None,
        })
        .collect();
    assert!(
        lines[0].0.starts_with('-'),
        "the discretionary hyphen is at the RTL line's left end: {lines:?}"
    );
    assert_eq!(*lines[0].2, ui.theme().palette.danger.linear());
    assert_eq!(
        ui.frame().element("Text").unwrap().height,
        step.line_height * 4.0
    );
    assert!(
        lines
            .iter()
            .all(|(_, bounds, _)| bounds.width <= width + 0.02)
    );
}
