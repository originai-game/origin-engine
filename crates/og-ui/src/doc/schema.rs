//! The reflection of element props (docs/ui.md §2.1, AGENTS.md: reflection
//! is the single schema): every element kind a document can name and every
//! prop it takes, with its value type and what it does. The parser
//! validates against it, the probe exports it (`dump_ui`'s `schema`), `ui:set`
//! types its values by it, and the README's table is checked against it.

use std::sync::LazyLock;

pub mod kinds;
pub mod parts;
pub mod props;
pub use parts::{PARTS, PartSpec, part};
pub use props::PROPS;

/// A prop's value type, as a document line writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    /// One number.
    Num,
    /// Two numbers (`pivot 0.5 1`, `focal 0.5 0.3`).
    Num2,
    /// One, two or four numbers: all sides, x y, or left top right bottom.
    Insets,
    /// Four numbers.
    Num4,
    /// `true` or `false`.
    Bool,
    /// A string; one with `{` is a template (`"{gold, compact} gold"`).
    Text,
    /// One of the words.
    Word(&'static [&'static str]),
    /// A name: a word or a string (a timeline, an icon, a path).
    Name,
    /// `fit [min max]`, `grow [min max]`, `fixed <px>` (or a bare number),
    /// `percent <0–1>`.
    Size,
    /// A colour: `token <role>` (a palette role) or `"#rrggbb[aa]"`.
    Paint,
    /// `spring <ms> <bounce>`, `tween <ms> [ease]`, `token <name>`, `instant`.
    Transition,
    /// Grid tracks: `120`, `1fr`, `auto`, `minmax:80:200`, `25%`, as words
    /// or numbers.
    Tracks,
    /// Words or strings.
    Names,
    /// A read-only array of typed narrative ranges, with Unicode scalar offsets.
    Spans,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Num => "number",
            Ty::Num2 => "x y",
            Ty::Insets => "insets",
            Ty::Num4 => "4 numbers",
            Ty::Bool => "bool",
            Ty::Text => "text",
            Ty::Word(_) => "word",
            Ty::Name => "name",
            Ty::Size => "size",
            Ty::Paint => "paint",
            Ty::Transition => "transition",
            Ty::Tracks => "tracks",
            Ty::Names => "names",
            Ty::Spans => "TextSpan[]",
        }
    }
}

/// Which kinds take a prop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum On {
    /// Every element.
    All,
    Kinds(&'static [&'static str]),
}

/// One prop of the schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropSpec {
    pub name: &'static str,
    pub ty: Ty,
    pub on: On,
    /// A control's value a binding writes back to (two-way).
    pub two_way: bool,
    pub doc: &'static str,
}

impl PropSpec {
    pub fn applies(&self, kind: &str) -> bool {
        match self.on {
            On::All => true,
            On::Kinds(k) => k.contains(&kind),
        }
    }
}

/// One element kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindSpec {
    pub name: &'static str,
    /// It has children.
    pub container: bool,
    pub doc: &'static str,
}

/// Every element kind ([`kinds`]).
pub static KINDS: LazyLock<Vec<KindSpec>> =
    LazyLock::new(|| [kinds::CORE_KINDS, kinds::KINDS].concat());

/// Author tags for engine semantics without an HTML equivalent, mapped
/// to the runtime kinds used by checkpoints.
pub const ELEMENT_TAGS: &[(&str, &str)] = &[
    ("og-icon", "icon"),
    ("og-counter", "label"),
    ("og-typewriter", "label"),
    ("og-spotlight", "dialog"),
    ("og-rive", "column"),
    ("og-emitter", "box"),
    ("og-overlay", "box"),
];

/// Tag-exclusive attributes and the runtime properties they populate.
pub const ELEMENT_ATTRIBUTES: &[(&str, &str, &str)] = &[
    ("og-counter", "value", "count"),
    ("og-counter", "duration", "countDuration"),
    ("og-counter", "decimals", "countDecimals"),
    ("og-typewriter", "rate", "reveal"),
    ("og-spotlight", "target", "spotlight"),
    ("og-rive", "src", "rive"),
    ("og-emitter", "effect", "emitter"),
    ("og-overlay", "effect", "meta"),
];

/// Registered CSS value functions: name, grammar and extension reason.
pub const CSS_FUNCTIONS: &[(&str, &str, &str)] = &[(
    "-og-spring",
    "-og-spring(<time [0,∞]> [, <number>])",
    "Transition timing with a perceptual duration (ms or s) and optional finite bounce (default 0). Bounce is clamped to -0.95..1 by the spring solver. Extension reason: CSS linear() cannot preserve velocity when a transition is interrupted. Springs time transitions, not keyframe animations.",
)];

/// Registered media extensions: name, grammar and extension reason.
pub const MEDIA_FEATURES: &[(&str, &str, &str)] = &[(
    "-og-input",
    "(-og-input: pad)",
    "Matches the configured gamepad input mode. Extension reason: standard media queries have no gamepad input feature. Forced Touch and Mouse instead set standard pointer/hover queries to coarse/none and fine/hover; Auto reports the device. The input's density metrics are the root variables --control-sm, --control-md, --control-lg, --row, --header, --gutter and --section (the --density-compact-*, --density-pad-* or --density-touch-* set the input takes) and --touch-target.",
)];

/// The prop named `name`.
pub fn prop(name: &str) -> Option<(usize, &'static PropSpec)> {
    PROPS.iter().enumerate().find(|(_, p)| p.name == name)
}

pub fn kind(name: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|k| k.name == name)
}

/// The schema as JSON (`dump_ui`'s `schema`, og.ui.documentSchema).
pub fn to_json() -> serde_json::Value {
    use serde_json::json;
    json!({
        "parts": parts::to_json(),
        "cssFunctions": CSS_FUNCTIONS.iter().map(|(name, grammar, reason)| json!({ "name": name, "grammar": grammar, "reason": reason })).collect::<Vec<_>>(),
        "mediaFeatures": MEDIA_FEATURES.iter().map(|(name, grammar, reason)| json!({ "name": name, "grammar": grammar, "reason": reason })).collect::<Vec<_>>(),
        "elementTags": ELEMENT_TAGS.iter().map(|(name, kind)| json!({ "name": name, "kind": kind,
            "attributes": ELEMENT_ATTRIBUTES.iter().filter(|(tag, _, _)| tag == name).map(|(_, attribute, property)| json!({"name": attribute, "property": property})).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "kinds": KINDS.iter().map(|k| json!({ "name": k.name, "container": k.container, "doc": k.doc })).collect::<Vec<_>>(),
        "props": PROPS.iter().map(|p| {
            let mut v = json!({ "name": p.name, "type": p.ty.name(), "doc": p.doc, "twoWay": p.two_way });
            if let Ty::Word(w) = p.ty {
                v["values"] = json!(w);
            }
            v["kinds"] = match p.on {
                On::All => json!("all"),
                On::Kinds(k) => json!(k),
            };
            v
        }).collect::<Vec<_>>(),
    })
}

/// The README's table of props, as Markdown rows (checked against the
/// README by this crate's tests, so the schema is described once).
pub fn markdown() -> String {
    let mut out = String::from("| prop | value | on | what it does |\n|---|---|---|---|\n");
    for p in PROPS.iter() {
        let ty = match p.ty {
            Ty::Word(w) => w.join(" \\| "),
            t => t.name().to_string(),
        };
        let on = match p.on {
            On::All => "all".to_string(),
            On::Kinds(k) => k.join(", "),
        };
        out.push_str(&format!(
            "| `{}` | {ty} | {on} | {}{} |\n",
            p.name,
            p.doc,
            if p.two_way { " Two-way." } else { "" }
        ));
    }
    out
}
