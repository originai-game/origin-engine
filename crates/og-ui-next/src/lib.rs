//! og-ui's new core (the og-ui rewrite, branch ui-rewrite). Styles are
//! compiled once, when a sheet or a pack loads, into flat typed records;
//! elements live in a flat arena indexed by u32 and lay out on rune-taffy's
//! dense store; a frame with nothing dirty does no style, layout or paint
//! work. There is no CSS cascade per element.

pub mod sheet;
pub mod tokens;

pub use tokens::Tokens;

#[cfg(test)]
mod tests {
    use super::*;

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
