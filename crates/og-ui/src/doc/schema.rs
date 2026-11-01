//! The reflection of element props (docs/ui.md §10.1, AGENTS.md: reflection
//! is the single schema): every element kind a document can name and every
//! prop it takes, with its value type and what it does. The parser
//! validates against it, the probe exports it (`dump_ui`'s `schema`), `ui:set`
//! types its values by it, and the README's table is checked against it.

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
    /// `<fade|slideUp|slideDown|slideLeft|slideRight|scale|pop> [distance]
    /// [transition…]`.
    Presence,
    /// Grid tracks: `120`, `1fr`, `auto`, `minmax:80:200`, `25%`, as words
    /// or numbers.
    Tracks,
    /// Words or strings.
    Names,
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
            Ty::Presence => "presence",
            Ty::Tracks => "tracks",
            Ty::Names => "names",
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

pub const KINDS: &[KindSpec] = &[
    KindSpec {
        name: "column",
        container: true,
        doc: "Children top to bottom.",
    },
    KindSpec {
        name: "row",
        container: true,
        doc: "Children left to right (mirrored right to left).",
    },
    KindSpec {
        name: "stack",
        container: true,
        doc: "Children overlaid in one rectangle, each placed by its insets.",
    },
    KindSpec {
        name: "grid",
        container: true,
        doc: "Children in tracks and cells (columns, rows, areas).",
    },
    KindSpec {
        name: "panel",
        container: true,
        doc: "A column on the style's panel surface (its card frame).",
    },
    KindSpec {
        name: "scroll",
        container: true,
        doc: "A column that scrolls its content.",
    },
    KindSpec {
        name: "list",
        container: true,
        doc: "A virtualised list: its `for` builds only the rows in view, each a selectable row.",
    },
    KindSpec {
        name: "cells",
        container: true,
        doc: "A virtualised grid of equal cells: its `for` builds only the cells in view.",
    },
    KindSpec {
        name: "slot",
        container: true,
        doc: "A place code fills (og.ui.slot), its box laid out with the document.",
    },
    KindSpec {
        name: "label",
        container: false,
        doc: "A text.",
    },
    KindSpec {
        name: "button",
        container: false,
        doc: "A button with a label.",
    },
    KindSpec {
        name: "image",
        container: false,
        doc: "A texture (or a tint).",
    },
    KindSpec {
        name: "art",
        container: false,
        doc: "An art slot (docs/ui.md §13): a spec the art pipeline reads, its texture once there is one.",
    },
    KindSpec {
        name: "icon",
        container: false,
        doc: "An icon of the style's set.",
    },
    KindSpec {
        name: "progress",
        container: false,
        doc: "A progress bar.",
    },
    KindSpec {
        name: "slider",
        container: false,
        doc: "A slider (two-way `value`).",
    },
    KindSpec {
        name: "toggle",
        container: false,
        doc: "A switch with a label (two-way `checked`).",
    },
    KindSpec {
        name: "field",
        container: false,
        doc: "A one-line text field (two-way `input`).",
    },
    KindSpec {
        name: "badge",
        container: false,
        doc: "A small toned label.",
    },
    KindSpec {
        name: "spacer",
        container: false,
        doc: "Empty room (grows by default).",
    },
    KindSpec {
        name: "divider",
        container: false,
        doc: "A hairline.",
    },
];

const STEPS: &[&str] = &[
    "body",
    "xs",
    "meta",
    "caption",
    "sm",
    "reading",
    "base",
    "lg",
    "xl",
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "displaySm",
    "displayMd",
    "displayLg",
    "displayXl",
    "displayXl2",
];
const ANCHORS: &[&str] = &[
    "topLeft",
    "top",
    "topRight",
    "left",
    "center",
    "right",
    "bottomLeft",
    "bottom",
    "bottomRight",
];

macro_rules! p {
    ($name:literal, $ty:expr, $doc:literal) => {
        PropSpec { name: $name, ty: $ty, on: On::All, two_way: false, doc: $doc }
    };
    ($name:literal, $ty:expr, [$($k:literal),*], $doc:literal) => {
        PropSpec { name: $name, ty: $ty, on: On::Kinds(&[$($k),*]), two_way: false, doc: $doc }
    };
    ($name:literal, $ty:expr, [$($k:literal),*], two_way, $doc:literal) => {
        PropSpec { name: $name, ty: $ty, on: On::Kinds(&[$($k),*]), two_way: true, doc: $doc }
    };
}

/// Every prop, by kind group: the box, the element model (docs/ui.md
/// §6.1), then each kind's own.
pub const PROPS: &[PropSpec] = &[
    // The box.
    p!("width", Ty::Size, "Its width (fit by default)."),
    p!("height", Ty::Size, "Its height (fit by default)."),
    p!("padding", Ty::Insets, "Space inside its edges, CSS px."),
    p!("gap", Ty::Num, "Space between its children, CSS px."),
    p!(
        "align",
        Ty::Word(&["start", "center", "end", "spaceBetween"]),
        "Where children sit along the main axis."
    ),
    p!(
        "cross",
        Ty::Word(&["start", "center", "end", "stretch", "baseline"]),
        "Where children sit across it."
    ),
    p!("wrap", Ty::Bool, "A row breaks its children into lines."),
    p!("background", Ty::Paint, "Its fill."),
    p!("radius", Ty::Num, "Its corner radius, CSS px."),
    p!(
        "frame",
        Ty::Word(&[
            "panel",
            "card",
            "button",
            "buttonHover",
            "buttonPressed",
            "buttonDisabled",
            "field",
            "slot",
            "tooltip",
            "hud",
            "track",
            "fill"
        ]),
        "The style frame its background takes."
    ),
    p!("aspect", Ty::Num, "Width over height (0: none)."),
    p!("interactive", Ty::Bool, "It reports clicks (`on click`)."),
    p!(
        "focusable",
        Ty::Bool,
        "With interactive: keys and pads reach it and press it."
    ),
    p!(
        "adjustable",
        Ty::Bool,
        "With focusable: Left and Right step it while it has the focus instead of moving the focus."
    ),
    p!(
        "visible",
        Ty::Bool,
        "False leaves it (and its subtree) out of the tree."
    ),
    p!("name", Ty::Text, "What a screen reader calls it."),
    p!(
        "part",
        Ty::Name,
        "Its part name a style dresses (`ShopCard.frame`); the document's `parts` lists them."
    ),
    // The element model.
    p!("x", Ty::Num, "Moved right after layout, CSS px."),
    p!("y", Ty::Num, "Moved down after layout, CSS px."),
    p!("scale", Ty::Num, "Both scales."),
    p!("scaleX", Ty::Num, "Its horizontal scale."),
    p!("scaleY", Ty::Num, "Its vertical scale."),
    p!("rotate", Ty::Num, "A turn, radians, clockwise."),
    p!("skewX", Ty::Num, "A skew, radians."),
    p!("skewY", Ty::Num, "A skew, radians."),
    p!(
        "pivot",
        Ty::Num2,
        "The transform's origin, fractions of its box."
    ),
    p!("opacity", Ty::Num, "0 to 1, multiplied down the tree."),
    p!("z", Ty::Num, "Drawing order among its siblings."),
    p!(
        "position",
        Ty::Word(&["flow", "absolute", "anchored"]),
        "In the flow, placed by its insets, or against another element."
    ),
    p!(
        "start",
        Ty::Num,
        "The inset from its parent's start edge (absolute)."
    ),
    p!("top", Ty::Num, "The inset from the top."),
    p!("end", Ty::Num, "The inset from the end edge."),
    p!("bottom", Ty::Num, "The inset from the bottom."),
    p!("centerX", Ty::Bool, "Centred across its parent."),
    p!("centerY", Ty::Bool, "Centred down its parent."),
    p!(
        "anchorTo",
        Ty::Name,
        "The path of the element an anchored element sits against."
    ),
    p!(
        "anchorAt",
        Ty::Word(ANCHORS),
        "The target's point it sits on."
    ),
    p!(
        "anchorSelf",
        Ty::Word(ANCHORS),
        "Its own point that sits there."
    ),
    p!("hitSlop", Ty::Num, "Extra pointer reach, CSS px."),
    p!("columns", Ty::Tracks, "A grid's column tracks."),
    p!("rows", Ty::Tracks, "A grid's row tracks."),
    p!(
        "areas",
        Ty::Names,
        "A grid's named areas, one string per row."
    ),
    p!(
        "flow",
        Ty::Word(&["row", "column"]),
        "How a grid auto-places."
    ),
    p!("dense", Ty::Bool, "A grid back-fills holes."),
    p!("area", Ty::Name, "The named grid area it takes."),
    p!("cell", Ty::Num2, "Its grid column and row, 1-based."),
    p!("span", Ty::Num2, "Columns and rows it spans."),
    p!("overlaps", Ty::Names, "Paths it overlaps on purpose."),
    p!("match", Ty::Name, "A shared-element key."),
    p!(
        "motion",
        Ty::Transition,
        "How its animatable values move when they change."
    ),
    p!(
        "layoutMotion",
        Ty::Transition,
        "Moves from its old rectangle to its new one."
    ),
    p!("enter", Ty::Presence, "How it arrives."),
    p!("exit", Ty::Presence, "How it leaves."),
    p!("inert", Ty::Bool, "It and its subtree take no input."),
    p!(
        "reduced",
        Ty::Word(&["fade", "none", "keep"]),
        "Its reduced-motion fallback."
    ),
    // Text.
    p!(
        "text",
        Ty::Text,
        ["label", "button", "badge", "toggle"],
        "What it says."
    ),
    p!(
        "key",
        Ty::Name,
        ["label", "button", "badge", "toggle"],
        "A string table key (og.i18n) its text comes from; `text` is the fallback."
    ),
    p!("step", Ty::Word(STEPS), ["label"], "The type step."),
    p!("color", Ty::Paint, ["label", "icon"], "Its ink."),
    p!(
        "textAlign",
        Ty::Word(&["start", "center", "end"]),
        ["label"],
        "How its lines align."
    ),
    p!("nowrap", Ty::Bool, ["label"], "One line."),
    p!("maxLines", Ty::Num, ["label"], "The most lines it shows."),
    p!(
        "ellipsis",
        Ty::Bool,
        ["label"],
        "Cut with … where it does not fit."
    ),
    p!(
        "font",
        Ty::Word(&["body", "display", "mono", "rounded"]),
        ["label"],
        "The font role."
    ),
    p!(
        "heading",
        Ty::Bool,
        ["label"],
        "A heading (a style's flourish frames it)."
    ),
    // Controls.
    p!(
        "variant",
        Ty::Word(&["secondary", "primary", "quiet", "danger"]),
        ["button"],
        "The button's role."
    ),
    p!(
        "size",
        Ty::Word(&["small", "medium", "large"]),
        ["button"],
        "The control height."
    ),
    p!(
        "disabled",
        Ty::Bool,
        ["button", "slider", "toggle", "field"],
        "Takes no input."
    ),
    p!(
        "autofocus",
        Ty::Bool,
        ["button"],
        "Takes the focus when it appears."
    ),
    p!(
        "value",
        Ty::Num,
        ["progress", "slider"],
        two_way,
        "Its value (a slider writes it back)."
    ),
    p!("min", Ty::Num, ["slider"], "The range's low end."),
    p!("max", Ty::Num, ["slider"], "The range's high end."),
    p!(
        "stepBy",
        Ty::Num,
        ["slider"],
        "Snap to multiples of this (0: continuous)."
    ),
    p!(
        "checked",
        Ty::Bool,
        ["toggle"],
        two_way,
        "Whether it is on (written back)."
    ),
    p!(
        "input",
        Ty::Text,
        ["field"],
        two_way,
        "The field's text (written back)."
    ),
    p!(
        "placeholder",
        Ty::Text,
        ["field"],
        "Shown while it is empty."
    ),
    p!(
        "maxChars",
        Ty::Num,
        ["field"],
        "The most characters it takes."
    ),
    p!(
        "tone",
        Ty::Word(&["neutral", "info", "success", "warn", "danger", "accent"]),
        ["badge"],
        "Its tone."
    ),
    p!("vertical", Ty::Bool, ["divider"], "Upright."),
    // Images and art.
    p!(
        "texture",
        Ty::Num,
        ["image", "art"],
        "A texture handle (og.load.texture), usually bound."
    ),
    p!(
        "tint",
        Ty::Paint,
        ["image", "art"],
        "Multiplies the texture (or fills without one)."
    ),
    p!(
        "icon",
        Ty::Name,
        ["icon"],
        "The icon's name in the style's set."
    ),
    p!("iconSize", Ty::Num, ["icon"], "CSS px (24 by default)."),
    p!(
        "kind",
        Ty::Name,
        ["art"],
        "The slot's kind (Appendix F: itemIcon, bust, keyArt…)."
    ),
    p!(
        "ratio",
        Ty::Num2,
        ["art"],
        "The slot's aspect, width and height in whole numbers."
    ),
    p!("focal", Ty::Num2, ["art"], "The point cropping keeps, 0–1."),
    p!(
        "safe",
        Ty::Num4,
        ["art"],
        "Where text or controls lie over it: x0 y0 x1 y1, 0–1."
    ),
    p!(
        "fit",
        Ty::Word(&["cover", "contain", "fill", "nineSlice", "tile"]),
        ["art"],
        "How the art fills the slot."
    ),
    // Virtualised containers.
    p!(
        "rowHeight",
        Ty::Num,
        ["list"],
        "Every row's height, CSS px."
    ),
    p!("cellWidth", Ty::Num, ["cells"], "A cell's width, CSS px."),
    p!("cellHeight", Ty::Num, ["cells"], "A cell's height, CSS px."),
    p!(
        "cellColumns",
        Ty::Num,
        ["cells"],
        "Cells across (0: as many as fit)."
    ),
];

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
    for p in PROPS {
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
