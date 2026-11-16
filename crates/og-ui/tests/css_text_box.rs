//! CSS Inline 3 §6: trimmed boxes and drawn baselines in an asymmetric
//! font. Noto Sans's hhea ascent/descent are 1069/293, cap height 714 and
//! x-height 536 (1000 units/em, checked independently with fontTools).

use crate::support::Doc;
use og_ui::*;

fn label(text: &str, css: &str) -> (f32, Vec<f32>) {
    let mut font = crate::features::font_at("templates/fonts/latin/NotoSans-Latin.ttf");
    font.handle = crate::support::test_font().handle;
    let page = format!(
        r#"<meta name="component" content="t">
<div><div id="Root"><p id="Label">{text}</p></div></div>
<style>#Root {{ width: 700px; }} p {{ margin: 0; font-size: 100px; line-height: 160px; }} {css}</style>"#
    );
    let d = Doc::with_font(&page, &[], font);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    let r = d.get("Label");
    let mut baselines: Vec<f32> =
        d.ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Text { baseline, .. } => Some(*baseline - r.y),
                _ => None,
            })
            .collect();
    baselines.sort_by(f32::total_cmp);
    baselines.dedup();
    (r.height, baselines)
}

fn near(got: f32, want: f32) {
    assert!((got - want).abs() <= 0.6, "got {got}, want {want}");
}

#[test]
fn text_box_trims_only_the_requested_outer_edges() {
    // At 160px line height, half-leading is (160 - 106.9 - 29.3)/2
    // = 11.9. Thus untrimmed ascent/descent are 118.8/41.2. A wrong
    // cap, ascent, x-height or under edge cannot pass these expectations.
    let (normal, pens) = label("HIT", "");
    near(normal, 160.0);
    assert_eq!(pens.len(), 1);
    near(pens[0], 118.8);
    for css in [
        "p { text-box-trim: trim-both; text-box-edge: cap alphabetic; }",
        "p { text-box: cap alphabetic; }",
        "p { text-box: cap alphabetic trim-both; }",
        "#Root { text-box-edge: cap alphabetic; } p { text-box-trim: trim-both; }",
        "#Root { text-box: cap alphabetic; } p { text-box-trim: inherit; }",
    ] {
        let (height, pens) = label("HIT", css);
        near(height, 71.4);
        assert_eq!(pens.len(), 1);
        near(pens[0], 71.4);
    }
    let (height, pens) = label(
        "H",
        "p { font-size: 500px; line-height: 800px; text-box: cap alphabetic; }",
    );
    assert_eq!(height, 357.0); // 500 × 714/1000: no snapping tolerance
    assert_eq!(pens, [357.0]);
    for css in [
        "p { text-box-edge: cap alphabetic; text-box-trim: none; }",
        "p { text-box: cap alphabetic; text-box: normal; }",
        "#Root { text-box: cap alphabetic; }", // trim does not inherit
    ] {
        let (height, got) = label("HIT", css);
        near(height, normal);
        near(got[0], 118.8);
    }
    // Three lines, not one trim per forced line or one trim per span.
    // Only the requested outer side changes; both baseline intervals
    // remain 160px, including when wrapping rather than using <br>.
    for text in ["HIT<br />HIT<br />HIT", "HIT HIT HIT"] {
        for (trim, height, first) in [
            ("none", 480.0, 118.8),
            ("trim-start", 432.6, 71.4),
            ("trim-end", 438.8, 118.8),
            ("trim-both", 391.4, 71.4),
        ] {
            let css = format!(
                "p {{ width: 200px; text-box-edge: cap alphabetic; text-box-trim: {trim}; }}"
            );
            let (h, pens) = label(text, &css);
            near(h, height);
            assert_eq!(pens.len(), 3, "{text} {trim}: {pens:?}");
            for (k, pen) in pens.iter().enumerate() {
                near(*pen, first + 160.0 * k as f32);
            }
        }
    }
    for (edge, height, baseline) in [
        ("ex alphabetic", 53.6, 53.6),
        ("text", 136.2, 106.9),
        ("auto", 136.2, 106.9),
        ("trim-both", 136.2, 106.9),   // omitted edge resets to auto
        ("ideographic", 136.2, 106.9), // no BASE: synthesized
        ("cap text", 100.7, 71.4),
        ("text alphabetic", 106.9, 106.9),
    ] {
        let (h, pens) = label("HIT", &format!("p {{ text-box: {edge}; }}"));
        near(h, height);
        assert_eq!(pens.len(), 1);
        near(pens[0], baseline);
    }
    for (prop, value) in [
        ("text-box-trim", "both"),
        ("text-box-edge", "alphabetic cap"),
        ("text-box-edge", "cap ex"),
        ("text-box-edge", "cap alphabetic text"),
        ("text-box", "trim-start trim-end cap alphabetic"),
        ("text-box", "normal cap alphabetic"),
    ] {
        let page = format!(
            "<meta name=\"component\" content=\"t\"><p>x</p><style>p {{ {prop}: {value}; }}</style>"
        );
        let error = crate::support::ui_with_font(crate::support::test_font_metrics())
            .parse_document("ui/t.svelte", &page, Vec::new())
            .map(|_| ())
            .unwrap_err();
        assert!(format!("{error:?}").contains(prop), "{error:?}");
    }
}
