//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). Styles are
//! compiled once, when a sheet or a pack loads, into flat typed records;
//! elements live in a flat arena indexed by u32 and lay out on rune-taffy's
//! dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

pub mod arena;
pub mod doc;
pub mod prim;
pub mod sheet;
pub mod style;
pub mod tokens;

pub use tokens::Tokens;

#[cfg(test)]
mod tests {
    use super::*;
    use rune_taffy::CompactStyle;

    /// Fixed metrics: half the size a character, one line.
    fn measure(text: &str, size: f32, _: Option<f32>) -> (f32, f32) {
        (text.chars().count() as f32 * size * 0.5, size)
    }

    #[test]
    fn a_clean_arena_lays_out_nothing_and_a_changed_text_moves_its_siblings() {
        let mut a = arena::Arena::new();
        let mut row = CompactStyle::default();
        row.set_flex_direction(taffy::FlexDirection::Row);
        row.set_gap(4.0, 0.0);
        row.set_padding_all(10.0);
        let r = a.element(0, row);
        let first = a.text(r, "06:54", 20.0);
        let second = a.text(r, "Clear", 20.0);
        assert!(a.layout(1280.0, 720.0, &mut measure));
        // 5 chars at 20 px: 50 px wide; the second starts after it and the gap.
        assert_eq!(a.box_of(first).w, 50.0);
        assert_eq!(a.box_of(second).x, 10.0 + 50.0 + 4.0);
        assert!(!a.layout(1280.0, 720.0, &mut measure), "nothing changed");
        a.set_text(first, "06:54");
        assert!(
            !a.layout(1280.0, 720.0, &mut measure),
            "the same text marks nothing"
        );
        a.set_text(first, "106:54");
        assert!(a.layout(1280.0, 720.0, &mut measure));
        assert_eq!(a.box_of(second).x, 10.0 + 60.0 + 4.0);
    }

    #[test]
    fn a_sheet_compiles_to_records_with_tokens_resolved_and_states_over_the_base() {
        let tokens = Tokens::parse(":root { --space-lg: 12px; --panel: #336699; }");
        let sheet = style::Sheet::compile(
            "#Clock { position: fixed; left: calc(var(--space-lg) + 4px); padding: 2px var(--space-lg); background: var(--panel); }\n\
             .hud:hover { opacity: 0.5; } .hud .x { color: red; }",
            &tokens,
        );
        assert_eq!(
            sheet.skipped,
            ["selector .hud .x"],
            "a combinator is listed, not guessed"
        );
        let clock = sheet.name("#Clock").unwrap();
        let hud = sheet.name(".hud").unwrap();
        let base = sheet.record(&[clock, hud], style::State::Base);
        assert_eq!(base.opacity, 1.0);
        assert_eq!(
            sheet.record(&[clock, hud], style::State::Hover).opacity,
            0.5
        );
        assert_eq!(
            sheet.record(&[clock], style::State::Hover).opacity,
            1.0,
            "the state rule needs .hud"
        );
        let mut a = arena::Arena::new();
        let e = a.element(0, base.layout.clone());
        a.text(e, "06:54", 20.0);
        a.layout(1280.0, 720.0, &mut measure);
        let b = a.box_of(e);
        assert_eq!((b.x, b.inset), (16.0, [12.0, 2.0, 12.0, 2.0]));
        assert!(
            base.fill[2] > base.fill[0],
            "the panel colour, bluer than red"
        );
        // What the cook writes, loaded back, answers the same records.
        let loaded = style::Sheet::from_bytes(&sheet.to_bytes()).unwrap();
        for state in [style::State::Base, style::State::Hover] {
            assert_eq!(
                loaded.record(&[clock, hud], state),
                sheet.record(&[clock, hud], state)
            );
        }
        assert_eq!(loaded.name(".hud"), Some(hud));
        let t = Tokens::from_bytes(&tokens.to_bytes()).unwrap();
        assert_eq!(t.get("--space-lg"), Some("12px"));
    }

    #[test]
    fn shorthands_and_logical_properties_expand_at_compile_time() {
        let sheet = style::Sheet::compile(
            "#A { flex: 1 0 40px; margin-inline: 3px 5px; padding-block: 7px 8px; padding-left: 2px; \
             inset: 0; top: 9px; max-inline-size: none; list-style: none; }",
            &Tokens::default(),
        );
        assert!(sheet.skipped.is_empty(), "{:?}", sheet.skipped);
        let r = sheet.record(&[sheet.name("#A").unwrap()], style::State::Base);
        let s = taffy::Style::from(&r.layout);
        assert_eq!(
            (s.flex_grow, s.flex_shrink, s.flex_basis),
            (1.0, 0.0, taffy::Dimension::length(40.0))
        );
        assert_eq!(
            (s.margin.left, s.margin.right),
            (
                taffy::LengthPercentageAuto::length(3.0),
                taffy::LengthPercentageAuto::length(5.0)
            )
        );
        assert_eq!(
            (s.padding.top, s.padding.bottom, s.padding.left),
            (
                taffy::LengthPercentage::length(7.0),
                taffy::LengthPercentage::length(8.0),
                taffy::LengthPercentage::length(2.0)
            )
        );
        assert_eq!(s.inset.top, taffy::LengthPercentageAuto::length(9.0));
        assert_eq!(s.inset.bottom, taffy::LengthPercentageAuto::length(0.0));
    }

    #[test]
    fn a_documents_rules_compile_per_element_over_its_layout_fields_and_surface() {
        let doc = og_ui::UiDocument::parse(
            "t.svelte",
            "<meta name=\"component\" content=\"t\">\n<meta name=\"prop:on\" content=\"bool\">\n\
             <div id=\"Box\" class=\"card\" class:lit={on} layout=\"row\" gap=\"tight\" \
             part=\"frame\" variant=\"hud\" state:done={on}><span class=\"label\">Hi</span>\
             <svg viewBox=\"0 0 4 4\"><circle cx=\"2\" cy=\"2\" r=\"2\" fill=\"none\"/></svg></div>\n\
             <style>\n.card { --slot: 48px; padding: 2px; }\n.card .label { width: var(--slot); }\n\
             .card.lit { padding: 9px; }\n</style>",
        )
        .unwrap();
        let tokens = Tokens::parse(":root { --layout-gap-tight: 3px; }");
        let surfaces = style::Sheet::compile(
            ".frame { padding: 1px; flex-wrap: wrap; } .frame.hud:hover { background: #ff0000; } \
             .frame.bare { flex-wrap: wrap-reverse; } .frame:state(done) { opacity: 0.5; }",
            &tokens,
        );
        let s = doc::DocStyle::compile(&doc, &tokens, &surfaces);
        let card = s
            .nodes
            .iter()
            .position(|n| doc.nodes[n.node as usize].id == "Box")
            .unwrap();
        let card_node = s.nodes[card].node as usize;
        let label = s
            .nodes
            .iter()
            .position(|n| doc.nodes[n.node as usize].parent == card_node)
            .expect("the label, the card's element child");
        let pad = |r: &style::Record| taffy::Style::from(&r.layout).padding.top;
        let base = s.record(card, |_| false);
        assert_eq!(
            pad(&base),
            taffy::LengthPercentage::length(2.0),
            "the document's rule over its surface's"
        );
        let t = taffy::Style::from(&base.layout);
        assert_eq!(
            (t.display, t.flex_direction, t.gap.width),
            (
                taffy::Display::Flex,
                taffy::FlexDirection::Row,
                taffy::LengthPercentage::length(3.0)
            ),
            "layout=row gap=tight"
        );
        assert_eq!(
            t.flex_wrap,
            taffy::FlexWrap::Wrap,
            "the frame surface over the layout field, without the other variant's rule"
        );
        assert_eq!(base.fill, [0.0; 4]);
        let hover = og_ui::bind::Expr::Interaction(og_ui::bind::InteractionState::Hover);
        let hovered = s.record(card, |c| s.conditions[c as usize] == hover);
        assert_eq!(hovered.fill[0], 1.0, ".frame.hud:hover while hovered");
        assert_eq!(hovered.opacity, 1.0);
        let done = s.record(card, |c| s.conditions[c as usize] != hover);
        assert_eq!(done.opacity, 0.5, ".frame:state(done) under state:done");
        assert!(
            !s.skipped.iter().any(|k| matches!(k, doc::Skip::Field(_))),
            "SVG's fill is paint: {:?}",
            s.skipped
        );
        assert_eq!(
            pad(&s.record(card, |_| true)),
            taffy::LengthPercentage::length(9.0),
            "class:lit's condition"
        );
        // The descendant rule reached the label, with the card's --slot.
        let w = taffy::Style::from(&s.record(label, |_| false).layout)
            .size
            .width;
        assert_eq!(w, taffy::Dimension::length(48.0));
    }

    #[test]
    fn tokens_resolve_through_var_chains_and_fallbacks() {
        let t = Tokens::parse(
            ":root { --a: 4px; --b: calc(var(--a) * 2); --c: var(--missing, var(--b)); }\n\
             /* a comment */ @media (width < 600px) { :root { --d: red; } }",
        );
        assert_eq!(t.get("--b"), Some("calc(4px * 2)"));
        assert_eq!(t.get("--c"), Some("calc(4px * 2)"));
        assert_eq!(t.get("--d"), Some("red"));
        let mut over = t.clone();
        over.over(":root { --a: 6px; }");
        assert_eq!(
            over.get("--c"),
            Some("calc(6px * 2)"),
            "a pack's token re-resolves its users"
        );
    }
}
