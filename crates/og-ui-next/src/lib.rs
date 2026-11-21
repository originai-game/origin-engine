//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). Styles are
//! compiled once, when a sheet or a pack loads, into flat typed records;
//! elements live in a flat arena indexed by u32 and lay out on rune-taffy's
//! dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

pub mod arena;
pub mod cooked;
pub mod doc;
pub mod icons;
pub mod look;
pub mod prim;
pub mod queue;
pub mod sheet;
pub mod style;
pub mod text;
pub mod tokens;
pub mod tree;

pub use tokens::Tokens;

#[cfg(test)]
mod tests {
    use super::*;
    use rune_taffy::CompactStyle;

    /// Fixed metrics: half the size a character, one line.
    fn measure(text: &str, run: &arena::Run, _: Option<f32>) -> (f32, f32) {
        let size = run.size;
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
        a.set_text(first, "07:15");
        assert!(
            !a.layout(1280.0, 720.0, &mut measure),
            "a text of the same size moves nothing"
        );
        assert_eq!(a.box_of(second).x, 10.0 + 50.0 + 4.0);
        // A new viewport alone lays out again.
        assert!(a.layout(640.0, 360.0, &mut measure), "a new viewport");
        assert_eq!(a.box_of(0).w, 640.0);
        assert!(!a.layout(640.0, 360.0, &mut measure), "the same viewport");
        a.set_text(first, "106:54");
        assert!(a.layout(1280.0, 720.0, &mut measure));
        assert_eq!(a.box_of(second).x, 10.0 + 60.0 + 4.0);
    }

    #[test]
    fn an_empty_element_keeps_its_own_size_and_padding() {
        let mut a = arena::Arena::new();
        let mut row = CompactStyle::default();
        row.set_flex_direction(taffy::FlexDirection::Row);
        row.set_align_items(taffy::AlignItems::FLEX_START);
        let r = a.element(0, row);
        let mut bar = CompactStyle::default();
        bar.set_size(taffy::Size {
            width: taffy::Dimension::length(30.0),
            height: taffy::Dimension::length(12.0),
        });
        let sized = a.element(r, bar);
        let mut pad = CompactStyle::default();
        pad.set_padding_all(6.0);
        let padded = a.element(r, pad);
        a.layout(1280.0, 720.0, &mut measure);
        let b = a.box_of(sized);
        assert_eq!((b.w, b.h), (30.0, 12.0), "its width and height");
        let p = a.box_of(padded);
        assert_eq!((p.w, p.h), (12.0, 12.0), "its padding alone");
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
    fn a_side_one_element_sets_reaches_no_other_element() {
        let doc = og_ui::UiDocument::parse(
            "t.svelte",
            "<meta name=\"component\" content=\"t\">\n<div><div id=\"A\"></div><div id=\"B\"></div></div>\n\
             <style>\n#A { position: absolute; top: 16px; left: 16px; padding-top: 4px; row-gap: 5px; overflow-y: scroll; }\n\
             #B { position: absolute; right: 0; padding-left: 2px; column-gap: 3px; overflow-x: hidden; }\n</style>",
        )
        .unwrap();
        let tokens = Tokens::default();
        let d = doc::DocStyle::compile(&doc, &tokens, &style::Sheet::compile("", &tokens));
        let at = |id: &str| {
            let k = d
                .nodes
                .iter()
                .position(|n| doc.nodes[n.node as usize].id == id)
                .unwrap();
            taffy::Style::from(&d.record(k, |_| false).layout)
        };
        let b = at("B");
        assert_eq!(b.inset.top, taffy::LengthPercentageAuto::auto(), "A's top");
        assert_eq!(
            b.inset.left,
            taffy::LengthPercentageAuto::auto(),
            "A's left"
        );
        assert_eq!(b.inset.right, taffy::LengthPercentageAuto::length(0.0));
        assert_eq!(
            b.padding.top,
            taffy::LengthPercentage::length(0.0),
            "A's padding"
        );
        assert_eq!(
            b.gap.height,
            taffy::LengthPercentage::length(0.0),
            "A's row gap"
        );
        assert_eq!(b.gap.width, taffy::LengthPercentage::length(3.0));
        assert_eq!(b.overflow.y, taffy::Overflow::Visible, "A's overflow");
        assert_eq!(b.overflow.x, taffy::Overflow::Hidden);
        let a = at("A");
        assert_eq!(a.inset.right, taffy::LengthPercentageAuto::auto());
        assert_eq!(a.overflow.y, taffy::Overflow::Scroll);
    }

    #[test]
    fn a_documents_rules_compile_per_element_over_its_layout_fields_and_surface() {
        let doc = og_ui::UiDocument::parse(
            "t.svelte",
            "<meta name=\"component\" content=\"t\">\n<meta name=\"prop:on\" content=\"bool\">\n\
             <div id=\"Box\" class=\"card\" class:lit={on} layout=\"row\" gap=\"tight\" \
             part=\"frame\" variant=\"hud\" state:done={on}><span class=\"label\" tone=\"danger\" play=\"pulse\" tier={on}>Hi</span>\
             <svg viewBox=\"0 0 4 4\"><circle cx=\"2\" cy=\"2\" r=\"2\" fill=\"none\"/></svg></div>\n\
             <style>\n.card { --slot: 48px; padding: 2px; }\n.card .label { width: var(--slot); }\n\
             .label:dir(rtl) { padding: 4px; }\n\
             .label:size(short) { padding: 6px; }\n\
             .card:size(compact) { --slot: 20px; }\n\
             .card.lit { padding: 9px; }\n</style>",
        )
        .unwrap();
        let tokens = Tokens::parse(":root { --layout-gap-tight: 3px; }");
        let surfaces = style::Sheet::compile(
            ".frame { padding: 1px; flex-wrap: wrap; } .frame.hud:hover { background: #ff0000; } \
             .frame.bare { flex-wrap: wrap-reverse; } .frame:state(done) { opacity: 0.5; } \
             .danger { color: #ff0000; } .rare { opacity: 0.25; }",
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
        let hover =
            og_reactive::bind::Expr::Interaction(og_reactive::bind::InteractionState::Hover);
        let hovered = s.record(card, |c| *s.conditions[c as usize] == hover);
        assert_eq!(hovered.fill[0], 1.0, ".frame.hud:hover while hovered");
        assert_eq!(hovered.opacity, 1.0);
        let done = s.record(card, |c| *s.conditions[c as usize] != hover);
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
        let label_rec = s.record(label, |_| false);
        assert_eq!(
            label_rec.ink[0], 1.0,
            "tone=danger picks the .danger surface"
        );
        let rtl = s.record(label, |c| {
            matches!(
                *s.conditions[c as usize],
                og_reactive::bind::Expr::Dir(_, true)
            )
        });
        assert_eq!(
            taffy::Style::from(&rtl.layout).padding.top,
            taffy::LengthPercentage::length(4.0),
            ":dir(rtl) is a condition on the direction"
        );
        let rare = s.record(label, |c| {
            matches!(&*s.conditions[c as usize],
                og_reactive::bind::Expr::Bin(og_reactive::bind::Op::Eq, _, v) if **v == og_reactive::bind::Expr::Lit("rare".into()))
        });
        assert_eq!(
            (label_rec.opacity, rare.opacity),
            (1.0, 0.25),
            "a bound tier picks its surface"
        );
        // 844×390: short, not narrow; the condition is the size class.
        let landscape = s.record(label, |c| match &*s.conditions[c as usize] {
            og_reactive::bind::Expr::Screen(k) => k.holds(844.0, 390.0, true),
            _ => false,
        });
        assert_eq!(
            taffy::Style::from(&landscape.layout).padding.top,
            taffy::LengthPercentage::length(6.0),
            ":size(short) holds on a phone on its side"
        );
        assert_eq!(
            (
                taffy::Style::from(&label_rec.layout).size.width,
                taffy::Style::from(&landscape.layout).size.width
            ),
            (
                taffy::Dimension::length(48.0),
                taffy::Dimension::length(20.0)
            ),
            "the card's --slot under :size(compact) reaches the label's width"
        );
        assert!(!og_reactive::bind::Screen::Narrow.holds(844.0, 390.0, true));
        assert!(og_reactive::bind::Screen::Compact.holds(390.0, 844.0, true));
    }

    #[test]
    fn the_hud_safe_area_inset_is_added_at_layout_on_the_sides_that_read_it() {
        let sheet = style::Sheet::compile(
            ".c { position: absolute; left: calc(env(-og-hud-inset-left) + 8px); top: 4px; \
             padding: 2px calc(env(-og-hud-inset-right) + 1px) 2px 2px; }",
            &Tokens::default(),
        );
        assert!(sheet.skipped.is_empty(), "{:?}", sheet.skipped);
        let r = sheet.record(&[sheet.name(".c").unwrap()], style::State::Base);
        let s = taffy::Style::from(&r.layout_in([5.0, 7.0, 0.0, 30.0]));
        assert_eq!(
            (s.inset.left, s.inset.top),
            (
                taffy::LengthPercentageAuto::length(38.0),
                taffy::LengthPercentageAuto::length(4.0)
            ),
            "left reads the inset, top does not"
        );
        assert_eq!(
            (s.padding.right, s.padding.left),
            (
                taffy::LengthPercentage::length(8.0),
                taffy::LengthPercentage::length(2.0)
            )
        );
    }

    #[test]
    fn text_properties_compile_inherit_and_wrap_a_leaf_by_its_run() {
        let sheet = style::Sheet::compile(
            ".p { font-size: 20px; letter-spacing: 0.1em; line-height: 1.5; font-weight: 600; } \
             .c { white-space: nowrap; text-overflow: -og-scroll 6s 1500ms; overflow: hidden; }",
            &Tokens::default(),
        );
        assert!(sheet.skipped.is_empty(), "{:?}", sheet.skipped);
        let p = sheet.record(&[sheet.name(".p").unwrap()], style::State::Base);
        let c = sheet.record(&[sheet.name(".c").unwrap()], style::State::Base);
        let t = c.text.over(&p.text);
        assert_eq!(
            (
                t.weight,
                t.tracking(p.font_size),
                t.line_height(p.font_size)
            ),
            (600, 2.0, 30.0),
            "the child inherits what it does not set"
        );
        assert_eq!(
            (t.nowrap, t.overflow, t.scroll),
            (true, 2, [6000.0, 1500.0])
        );
        // Fixed advances: half an em a character, so 5 at 20 px are 50 px.
        let m = og_text::text::FontMetrics::fixed(0.5);
        let run = arena::Run {
            size: 20.0,
            tracking: 0.0,
            line: 30.0,
            weight: 600,
            nowrap: false,
        };
        assert_eq!(text::measure(&m, "aa bb", &run, Some(1000.0)), (50.0, 30.0));
        assert_eq!(
            text::measure(&m, "aa bb", &run, Some(25.0)),
            (20.0, 60.0),
            "wraps at the space"
        );
        let one = arena::Run {
            nowrap: true,
            ..run
        };
        assert_eq!(
            text::measure(&m, "aa bb", &one, Some(25.0)),
            (50.0, 30.0),
            "nowrap"
        );
    }

    #[test]
    fn a_gradient_background_compiles_over_its_base_colour() {
        let tokens =
            Tokens::parse(":root { --plate-fill: linear-gradient(180deg, #ff0000, #0000ff 80%); }");
        let sheet = style::Sheet::compile(
            ".a { background: var(--plate-fill); } \
             .b { background: linear-gradient(to right, #ff0000, #00ff00, #0000ff), #ffffff; } \
             .c { background: #00ff00; } .d { background: radial-gradient(#fff, #000); }",
            &tokens,
        );
        assert_eq!(sheet.skipped, ["background: radial-gradient(#fff, #000)"]);
        let rec = |n: &[&str]| {
            let names: Vec<u32> = n.iter().map(|n| sheet.name(n).unwrap()).collect();
            sheet.record(&names, style::State::Base)
        };
        let g = rec(&[".a"]).gradient.unwrap();
        assert!(
            (g.angle - std::f32::consts::PI).abs() < 1e-5,
            "180deg points down"
        );
        let offsets: Vec<f32> = g.stops().iter().map(|s| (s.0 * 100.0).round()).collect();
        assert_eq!(offsets, [0.0, 80.0]);
        assert!(
            g.stops()[0].1[0] > 0.9 && g.stops()[1].1[2] > 0.9,
            "red to blue"
        );
        let b = rec(&[".b"]);
        let g = b.gradient.unwrap();
        assert!(
            (g.angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
            "to right"
        );
        let offsets: Vec<f32> = g.stops().iter().map(|s| (s.0 * 100.0).round()).collect();
        assert_eq!(offsets, [0.0, 50.0, 100.0], "an unplaced stop halfway");
        assert_eq!(b.fill, [1.0; 4], "the base colour under it");
        assert_eq!(
            rec(&[".b", ".c"]).gradient,
            None,
            "a later colour replaces the gradient"
        );
    }

    #[test]
    fn borders_shadows_and_the_other_paint_fields_compile_into_the_record() {
        let sheet = style::Sheet::compile(
            ".a { border: 2px solid #ff0000; border-left: 0; border-radius: 1px 2px 3px 4px; \
             box-shadow: inset 0 1px 0 0 rgb(255 255 255 / 0.5), 0 4px 14px -6px #000000; \
             outline: 3px solid #00ff00; outline-offset: 2px; translate: 4px 5px; scale: 150%; \
             rotate: 90deg; transform-origin: top left; backdrop-filter: blur(12px); \
             text-shadow: 1px 2px 3px #0000ff; text-decoration: underline line-through 2px #ff0000; \
             background: linear-gradient(to top right, #000000, #ffffff); } \
             .b { box-shadow: none; } .c { box-shadow: 1px 1px; } .d { box-shadow: 1px; }",
            &Tokens::default(),
        );
        assert_eq!(
            sheet.skipped,
            ["box-shadow: 1px"],
            "one length is no shadow"
        );
        let rec = |n: &[&str]| {
            let names: Vec<u32> = n.iter().map(|n| sheet.name(n).unwrap()).collect();
            sheet.record(&names, style::State::Base)
        };
        let a = rec(&[".a"]);
        assert_eq!(a.border, [2.0, 2.0, 2.0, 0.0], "a side over the shorthand");
        assert_eq!(a.border_color[0], 1.0);
        let t = taffy::Style::from(&a.layout);
        assert_eq!(
            (t.border.top, t.border.left),
            (
                taffy::LengthPercentage::length(2.0),
                taffy::LengthPercentage::length(0.0)
            ),
            "borders take space"
        );
        assert_eq!(a.radius, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(a.shadow_count, 2);
        let [inner, outer] = [a.shadows[0], a.shadows[1]];
        assert!(inner.inset && !outer.inset, "the first layer is inset");
        assert_eq!(
            (outer.offset, outer.blur, outer.spread),
            ([0.0, 4.0], 14.0, -6.0)
        );
        assert!((inner.color[3] - 0.5).abs() < 1e-3 && outer.color[3] == 1.0);
        assert_eq!(
            (a.outline.width, a.outline.offset, a.outline.color[1]),
            (3.0, 2.0, 1.0)
        );
        let m = a.transform;
        assert_eq!(
            (m.translate, m.scale, m.origin),
            ([4.0, 5.0], [1.5, 1.5], [0.0, 0.0])
        );
        assert!((m.rotate - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(a.backdrop_blur, 12.0);
        assert_eq!(
            (a.text_shadow.offset, a.text_shadow.blur),
            ([1.0, 2.0], 3.0)
        );
        assert_eq!(
            (
                a.decoration.line,
                a.decoration.thickness,
                a.decoration.color[0]
            ),
            (
                style::Decoration::UNDERLINE | style::Decoration::LINE_THROUGH,
                2.0,
                1.0
            )
        );
        assert_eq!(
            a.gradient.unwrap().corner,
            [1, -1],
            "to top right: the corner"
        );
        assert_eq!(rec(&[".a", ".b"]).shadow_count, 0, "none clears the layers");
        let c = rec(&[".c"]).shadows[0];
        assert_eq!(
            (c.offset, c.color),
            ([1.0, 1.0], [0.0, 0.0, 0.0, 1.0]),
            "black when unnamed"
        );
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
