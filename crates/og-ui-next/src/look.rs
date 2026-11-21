//! A style as the runtime loads it: the default tokens and surfaces with a
//! pack's over them, in one scheme. The scheme picks the token table
//! ([`Tokens::scheme`]); switching scheme or pack is loading another look,
//! never a cascade over the elements.

use crate::style::{Record, Sheet, Step};
use crate::tokens::Tokens;

/// The style compiler's sources, hashed when og is built: a style the
/// cook compiled with another compiler has another look key, so the
/// runtime compiles it again rather than map stale steps.
const COMPILER: u64 = {
    const SOURCES: [&[u8]; 7] = [
        include_bytes!("style.rs"),
        include_bytes!("doc.rs"),
        include_bytes!("prim.rs"),
        include_bytes!("sheet.rs"),
        include_bytes!("tokens.rs"),
        include_bytes!("cooked.rs"),
        include_bytes!("look.rs"),
    ];
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut f = 0;
    while f < SOURCES.len() {
        let b = SOURCES[f];
        let mut i = 0;
        while i < b.len() {
            h = (h ^ b[i] as u64).wrapping_mul(0x0000_0100_0000_01b3);
            i += 1;
        }
        f += 1;
    }
    h
};

/// One layer of a style: its tokens' CSS and its surfaces sheet.
#[derive(Clone, Copy, Debug)]
pub struct Layer<'a> {
    pub tokens: &'a str,
    pub surfaces: &'a str,
}

/// What the root sets for the text under it, as declarations over the
/// token table.
const ROOT: [(&str, &str); 4] = [
    ("color", "var(--ink, #ffffff)"),
    ("font-size", "var(--font-scale-body-size, 16px)"),
    ("line-height", "var(--line-height-body, 1.25)"),
    ("font-weight", "var(--font-weight-body, 400)"),
];

/// A style in one scheme. Its token table and surfaces sheet are read
/// from its layers' CSS only when a document has no style the cook
/// compiled against this look ([`Look::key`]): a cooked game's documents
/// map their compiled styles, and the look carries only its key and root.
#[derive(Debug)]
pub struct Look {
    pub scheme: Box<str>,
    /// The layers' sources and the scheme, hashed: a cooked style
    /// compiled against another look has another key.
    pub key: u64,
    /// What the root sets for the text under it (the ink, the body size,
    /// its line height and weight) as compiled steps, and folded.
    pub root_steps: Vec<Step>,
    pub root: Record,
    layers: Vec<(String, String)>,
    read: std::sync::OnceLock<(Tokens, Sheet)>,
}

impl Look {
    /// `layers` (the defaults first, then each pack) in `scheme`, read
    /// now. A later layer's tokens outrank an earlier one's whatever their
    /// blocks, and its surfaces come after the earlier ones' in sheet
    /// order, so at an equal weight a pack's rule wins.
    pub fn load(layers: &[Layer<'_>], scheme: &str) -> Look {
        let mut look = Look::new(layers, scheme, Vec::new());
        let mut steps = Vec::new();
        for (name, value) in ROOT {
            if let Some(v) = look.tokens().substitute_str(value) {
                crate::style::expand(&mut steps, name, v.trim());
            }
        }
        look.set_root(steps);
        look
    }

    /// `layers` in `scheme` with the root the cook compiled for it: its
    /// tokens and surfaces are read only if a document needs compiling.
    pub fn cooked(layers: &[Layer<'_>], scheme: &str, root_steps: Vec<Step>) -> Look {
        Look::new(layers, scheme, root_steps)
    }

    /// The key a look of `layers` in `scheme` has, without reading it.
    pub fn key_of(layers: &[Layer<'_>], scheme: &str) -> u64 {
        // Eight bytes a step (half a megabyte of tokens is read on every
        // load): the same on every host, as the cook's key must be.
        fn eat(h: &mut u64, s: &str) {
            let b = s.as_bytes();
            let (words, rest) = b.as_chunks::<8>();
            for w in words {
                let v = u64::from_le_bytes(*w);
                *h = (*h ^ v).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(29);
            }
            let mut tail = [0u8; 8];
            tail[..rest.len()].copy_from_slice(rest);
            *h = (*h ^ u64::from_le_bytes(tail) ^ b.len() as u64)
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .rotate_left(29);
        }
        let mut h = COMPILER;
        eat(&mut h, "og-ui-next look 1");
        eat(&mut h, scheme);
        for l in layers {
            eat(&mut h, l.tokens);
            eat(&mut h, l.surfaces);
        }
        h
    }

    fn new(layers: &[Layer<'_>], scheme: &str, root_steps: Vec<Step>) -> Look {
        let mut look = Look {
            scheme: scheme.into(),
            key: Look::key_of(layers, scheme),
            root_steps: Vec::new(),
            root: Record::default(),
            layers: layers
                .iter()
                .map(|l| (l.tokens.to_owned(), l.surfaces.to_owned()))
                .collect(),
            read: std::sync::OnceLock::new(),
        };
        look.set_root(root_steps);
        look
    }

    fn set_root(&mut self, steps: Vec<Step>) {
        let mut root = Record::default();
        for s in &steps {
            crate::style::fold(&mut root, s);
        }
        self.root = root;
        self.root_steps = steps;
    }

    fn read(&self) -> &(Tokens, Sheet) {
        self.read.get_or_init(|| {
            let mut all = Tokens::default();
            for (tokens, _) in &self.layers {
                all.over(tokens);
            }
            let tokens = all.scheme(&self.scheme);
            let css: String = self
                .layers
                .iter()
                .map(|(_, s)| s.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let surfaces = Sheet::compile(&css, &tokens);
            (tokens, surfaces)
        })
    }

    /// The resolved token table in this scheme.
    pub fn tokens(&self) -> &Tokens {
        &self.read().0
    }

    /// The surfaces sheet, compiled against the tokens.
    pub fn surfaces(&self) -> &Sheet {
        &self.read().1
    }

    /// Whether the tokens and surfaces were read (a document compiled at
    /// run time).
    pub fn was_read(&self) -> bool {
        self.read.get().is_some()
    }
}

/// A pack's layer from its files (asset id, CSS), as og.load.style and
/// the cook hold them: its `tokens.css` and `surfaces.css` beside each
/// other at the pack's top. `None` when it has neither (a pack only the
/// old core reads).
pub fn pack_layer(files: &[(String, String)]) -> Option<Layer<'_>> {
    let top = |name: &str| {
        files
            .iter()
            .filter(|(id, _)| id.rsplit_once('/').map_or(id.as_str(), |(_, f)| f) == name)
            .min_by_key(|(id, _)| id.matches('/').count())
            .map(|(_, css)| css.as_str())
    };
    let (tokens, surfaces) = (top("tokens.css"), top("surfaces.css"));
    (tokens.is_some() || surfaces.is_some()).then(|| Layer {
        tokens: tokens.unwrap_or(""),
        surfaces: surfaces.unwrap_or(""),
    })
}

/// The section of a document blob that holds its style compiled against
/// the look of key `key` ([`crate::cooked`]).
pub fn section(key: u64) -> String {
    format!("next-{key:016x}")
}

/// The default style's tokens: the design language's token files, then
/// the old default style's (`og_ui::DEFAULT_CSS`, which holds the
/// generated theme's tokens) until the switch removes it. Joined once.
pub fn default_tokens() -> &'static str {
    static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CSS.get_or_init(|| {
        let mut css = String::new();
        for f in [
            include_str!("../../../packages/design/tokens/seed.css"),
            include_str!("../../../packages/design/tokens/color.css"),
            include_str!("../../../packages/design/tokens/role.css"),
            include_str!("../../../packages/design/tokens/derived.css"),
            include_str!("../../../packages/design/tokens/contrast.css"),
            include_str!("../../../packages/design/tokens/state.css"),
            include_str!("../../../packages/design/tokens/space.css"),
            include_str!("../../../packages/design/tokens/density.css"),
            include_str!("../../../packages/design/tokens/type.css"),
            include_str!("../../../packages/design/tokens/effect.css"),
            include_str!("../../../packages/design/tokens/motion.css"),
            og_ui::DEFAULT_CSS,
        ] {
            css += f;
            css.push('\n');
        }
        css
    })
}

/// The default surfaces sheet.
pub const DEFAULT_SURFACES: &str = include_str!("../default/surfaces.css");

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
            let id = l.surfaces().name(".frame").unwrap();
            l.surfaces().record(&[id], crate::style::State::Base)
        };
        assert_eq!(frame(&dark).skin.fill, [0.0, 0.0, 0.0, 1.0]);
        assert_ne!(dark.key, light.key, "a scheme is another look");
        assert_eq!(frame(&light).skin.fill, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(frame(&light).skin.opacity, 0.25, "the pack's rule wins");
        assert_eq!(light.tokens().get("--edge"), Some("1px"));
    }
}
