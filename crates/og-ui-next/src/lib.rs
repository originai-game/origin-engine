//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). Styles are
//! compiled once, when a sheet or a pack loads, into flat typed records;
//! elements live in a flat arena indexed by u32 and lay out on rune-taffy's
//! dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

pub mod arena;
pub mod sheet;
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
