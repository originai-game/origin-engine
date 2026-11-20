//! A style as the runtime loads it: the default tokens and surfaces with a
//! pack's over them, in one scheme. The scheme picks the token table
//! ([`Tokens::scheme`]); switching scheme or pack is loading another look,
//! never a cascade over the elements.

use crate::style::Sheet;
use crate::tokens::Tokens;

/// One layer of a style: its tokens' CSS and its surfaces sheet.
#[derive(Clone, Copy, Debug)]
pub struct Layer<'a> {
    pub tokens: &'a str,
    pub surfaces: &'a str,
}

/// A loaded style in one scheme: its resolved token table and its surfaces
/// compiled against it.
#[derive(Clone, Debug)]
pub struct Look {
    pub scheme: Box<str>,
    pub tokens: Tokens,
    pub surfaces: Sheet,
    /// What the root sets for the text under it: the ink, the body size,
    /// its line height and weight.
    pub root: crate::style::Record,
}

impl Look {
    /// `layers` (the defaults first, then each pack) in `scheme`. A later
    /// layer's tokens outrank an earlier one's whatever their blocks, and
    /// its surfaces come after the earlier ones' in sheet order, so at an
    /// equal weight a pack's rule wins.
    pub fn load(layers: &[Layer<'_>], scheme: &str) -> Look {
        let mut all = Tokens::default();
        for l in layers {
            all.over(l.tokens);
        }
        let tokens = all.scheme(scheme);
        let css: String = layers
            .iter()
            .map(|l| l.surfaces)
            .collect::<Vec<_>>()
            .join("\n");
        let surfaces = Sheet::compile(&css, &tokens);
        let root = Sheet::compile(
            ".root { color: var(--ink, #ffffff); font-size: var(--font-scale-body-size, 16px); \
             line-height: var(--line-height-body, 1.25); font-weight: var(--font-weight-body, 400); }",
            &tokens,
        );
        let root = root.record(
            &[root.name(".root").unwrap_or(0)],
            crate::style::State::Base,
        );
        Look {
            scheme: scheme.into(),
            tokens,
            surfaces,
            root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_look_reads_its_schemes_table_and_the_packs_surfaces_over_the_defaults() {
        let defaults = Layer {
            tokens: ":root { --fill: #000000; } :root.light { --fill: #ffffff; }",
            surfaces: ".frame { background: var(--fill); opacity: 0.5; }",
        };
        let pack = Layer {
            tokens: ":root { --edge: 1px; }",
            surfaces: ".frame { opacity: 0.25; }",
        };
        let dark = Look::load(&[defaults, pack], "dark");
        let light = Look::load(&[defaults, pack], "light");
        let frame = |l: &Look| {
            let id = l.surfaces.name(".frame").unwrap();
            l.surfaces.record(&[id], crate::style::State::Base)
        };
        assert_eq!(frame(&dark).fill, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(frame(&light).fill, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(frame(&light).opacity, 0.25, "the pack's rule wins");
        assert_eq!(light.tokens.get("--edge"), Some("1px"));
    }
}
