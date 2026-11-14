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

#[test]
fn rich_soft_hyphens_reserve_the_styled_glyph_at_a_break() {
    let mut font = crate::features::font("Inter-Display.ttf");
    font.handle = test_font().handle;
    let m = font.metrics.clone();
    let size = ui_with_font(font.clone()).theme().type_ramp.body.size;
    let narrow = m.measure("ab-", size) + 0.01;
    let wide = m.measure("abcd", size) + 1.0;
    let no_hyphen = m.measure("ab", size).max(m.measure("cd", size)) + 0.01;
    for (width, hyphen) in [(narrow, true), (wide, false), (no_hyphen, false)] {
        let mut ui = ui_with_font(font.clone());
        ui.define_action(ActionDesc {
            name: "jump".into(),
            keys: vec!["KeyE".into()],
            ..Default::default()
        });
        // The body step without tracking; `b` and its soft hyphen in the
        // danger ink, the action's keycap after `cd`.
        let mut d = Doc::with_ui(
            ui,
            &format!(
                "<meta name=\"component\" content=\"t\">
<div><text id=\"Text\">a<span part=\"d\">b\u{ad}</span>cd<kbd data-action=\"jump\"></kbd></text></div>
<style>#Text {{ width: {width}px; letter-spacing: 0; }} text::part(d) {{ color: var(--danger); }}</style>"
            ),
        );
        d.viewport = Viewport::new(800.0, 600.0);
        d.step(InputFrame::default());
        let ui = &d.ui;
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
        let left = d.get("Text").x;
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
    use og_draw::{Glyph, Kern, MsdfFont, TextureHandle};
    use std::time::Instant;
    // Every CJK character is a break unit. Summed isolated advances exceed
    // the box, but the kerned line fits: the old fits_whole loop reshaped
    // each growing prefix here. No OpenType file or layout cache hides it.
    let font = UiFont {
        handle: Some(og_draw::FontHandle::from_bits(1)),
        metrics: FontMetrics::from_msdf(&MsdfFont {
            atlas: TextureHandle::from_slot(1).unwrap(),
            distance_range: 4.0,
            foreground: false,
            silhouette: false,
            line_height: 1.2,
            glyphs: vec![Glyph {
                key: '漢'.into(),
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
        let text = "漢".repeat(n);
        // The body step without tracking, mounted and built once: the
        // first layout is the one timed.
        let doc = UiDocument::parse(
            "ui/t.svelte",
            &format!(
                r#"<meta name="component" content="t">
<div><text id="Text">{text}</text></div>
<style>#Text {{ width: {width}px; letter-spacing: 0; }}</style>"#
            ),
        )
        .expect("parses");
        ui.define_document(doc);
        ui.mount_document(
            og_ui::reactive::SourceId::new("test", "t"),
            "t",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .expect("mounts");
        ui.begin(InputFrame::default(), Viewport::new(4000.0, 600.0));
        ui.build_retained_documents(&bind::NoEnv).expect("builds");
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
