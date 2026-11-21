//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). A node's
//! classes resolve once per distinct string into a flat typed record
//! (classes.rs, NSH's Tailwind); elements live in a flat arena indexed by
//! u32 and lay out on rune-taffy's dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

/// NSH's cursor, which a record's `cursor-*` names.
pub use nsh_tailwind::effects::Cursor;
pub mod a11y;
pub mod arena;
pub mod classes;
pub mod cover;
pub mod icons;
pub mod motion;
pub mod nav;
pub mod pack;
pub mod queue;
pub mod record;
pub mod sheet;
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

    /// Words of half the size a character, broken greedily at spaces
    /// to `width` (a word wider than it overflows whole, as CSS's
    /// `overflow-wrap: normal`).
    fn wrapped(text: &str, run: &arena::Run, width: Option<f32>) -> (f32, f32) {
        let ch = run.size * 0.5;
        let width = width.filter(|_| !run.nowrap);
        let (mut widest, mut line, mut lines) = (0.0f32, 0.0f32, 1.0f32);
        for word in text.split(' ') {
            let w = word.chars().count() as f32 * ch;
            let next = if line > 0.0 { line + ch + w } else { w };
            if line > 0.0 && width.is_some_and(|max| next > max) {
                widest = widest.max(line);
                line = w;
                lines += 1.0;
            } else {
                line = next;
            }
        }
        (widest.max(line), lines * run.size)
    }

    fn row_of(width: f32) -> CompactStyle {
        let mut row = CompactStyle::default();
        row.set_flex_direction(taffy::FlexDirection::Row);
        row.set_align_items(taffy::AlignItems::FLEX_START);
        row.set_size(taffy::Size {
            width: taffy::Dimension::length(width),
            height: taffy::Dimension::auto(),
        });
        row
    }

    /// A text's min-content width is its longest word, not its unbroken
    /// line: a flex item with the automatic minimum shrinks to it in a
    /// row too narrow for the line, and wraps there. Wrong (min-content
    /// measured unbroken, as before): the item keeps the line's 90 px.
    /// A nowrap line is one unbreakable segment.
    #[test]
    fn a_texts_min_content_is_its_longest_word() {
        let mut m = wrapped;
        for (nowrap, want) in [(false, (40.0, 60.0)), (true, (90.0, 20.0))] {
            let mut a = arena::Arena::new();
            let r = a.element(0, row_of(30.0));
            let e = a.element(r, CompactStyle::default());
            // 20 px: 10 px a character; "bbbb" is 40 px, the line 90.
            let t = a.text(e, "aa bbbb c", 20.0);
            let mut run = arena::Run::sized(20.0);
            run.nowrap = nowrap;
            a.set_run(t, run);
            a.layout(1280.0, 720.0, &mut m);
            let b = a.box_of(e);
            assert_eq!((b.w, b.h), want, "nowrap {nowrap}");
        }
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
