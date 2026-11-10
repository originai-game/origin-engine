//! The reflection of element props (docs/ui.md §2.1, AGENTS.md: reflection
//! is the single schema): every element kind a document can name and every
//! prop it takes, with its value type and what it does. The parser
//! validates against it, the probe exports it (`dump_ui`'s `schema`), `ui:set`
//! types its values by it, and the README's table is checked against it.

use std::sync::LazyLock;

#[path = "schema_parts.rs"]
pub mod parts;
pub use parts::{PARTS, PartSpec, part};

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
            Ty::Presence => "presence",
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

/// Every element kind: the core's, then the document runtime's
/// ([`super::schema_extra`]).
pub static KINDS: LazyLock<Vec<KindSpec>> =
    LazyLock::new(|| [CORE_KINDS, super::schema_extra::KINDS].concat());

/// Author tags for engine semantics without an HTML equivalent, mapped
/// to the runtime kinds used by checkpoints.
pub const ELEMENT_TAGS: &[(&str, &str)] = &[
    ("og-icon", "icon"),
    ("og-counter", "label"),
    ("og-typewriter", "label"),
    ("og-spotlight", "dialog"),
    ("og-rive", "column"),
    ("og-emitter", "box"),
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
];

/// Every prop: the core's, then the document runtime's.
pub static PROPS: LazyLock<Vec<PropSpec>> =
    LazyLock::new(|| [CORE_PROPS, super::schema_extra::PROPS].concat());

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
    "Matches the configured gamepad input mode. Extension reason: standard media queries have no gamepad input feature. Forced Touch and Mouse instead set standard pointer/hover queries to coarse/none and fine/hover; Auto reports the device. No density metrics are exposed to CSS.",
)];

pub const CORE_KINDS: &[KindSpec] = &[
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
        doc: "A named place, its box laid out with the document; content a mount passes is placed with `{@render}`.",
    },
    KindSpec {
        name: "label",
        container: false,
        doc: "A text.",
    },
    KindSpec {
        name: "button",
        container: true,
        doc: "A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states; `data-hold` makes it a hold-to-confirm button that fills while held down.",
    },
    KindSpec {
        name: "kbd",
        container: false,
        doc: "A key or an action's glyph: `<kbd>Esc</kbd>` a keycap, `<kbd data-action=\"jump\"></kbd>` the action's glyph on the device used last.",
    },
    KindSpec {
        name: "image",
        container: false,
        doc: "A texture. CSS filter with an SVG colour matrix multiplies its RGBA channels; use a background for a solid fill.",
    },
    KindSpec {
        name: "art",
        container: false,
        doc: "An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one.",
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
        name: "color",
        container: false,
        doc: "A colour input: a swatch that opens a picker (two-way `value`, `#rrggbb`).",
    },
    KindSpec {
        name: "datetime",
        container: false,
        doc: "A date or time input (`<input type=\"date|time|datetime-local|month|week\">`): a stepper per part, two-way `dateValue` in HTML's format.",
    },
    KindSpec {
        name: "video",
        container: false,
        doc: "HTML's `<video>`: the frame of the og.VideoPlayer entity `src` binds, with Svelte's media bindings and optional controls.",
    },
    KindSpec {
        name: "audio",
        container: false,
        doc: "HTML's `<audio>`: the og.AudioSource entity `src` binds, played and paused by `paused`; drawn only with `controls`.",
    },
    KindSpec {
        name: "file",
        container: false,
        doc: "A file input (`<input type=\"file\">`), built only where the host opens a file picker (og.ui.filePicker): a button whose press sends `filepick`; the host writes the names chosen to two-way `fileValue`.",
    },
    KindSpec {
        name: "radio",
        container: false,
        doc: "One choice of a radio group with its caption: checked while its group (`bind:group`) holds its value, which a press writes.",
    },
    KindSpec {
        name: "tabs",
        container: false,
        doc: "A tab list (two-way `selected`), using the core tabs widget.",
    },
    KindSpec {
        name: "field",
        container: false,
        doc: "A text control (two-way `input`): `<input>` for one line, `<textarea>` for several; `type=\"number\"` keeps editable text and adds increment/decrement buttons. Min, max and step constrain stepping; data-wrap, data-large-step and data-format configure it.",
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
    KindSpec {
        name: "select",
        container: false,
        doc: "A select (two-way `value`): its `<option>`s, under `<optgroup>`s, in a list that opens over it.",
    },
    KindSpec {
        name: "dialog",
        container: true,
        doc: "A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns.",
    },
    KindSpec {
        name: "details",
        container: true,
        doc: "A disclosure (two-way `open`): its `<summary>` row, its content shown only while open.",
    },
    KindSpec {
        name: "box",
        container: true,
        doc: "A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker.",
    },
    KindSpec {
        name: "meter",
        container: false,
        doc: "A gauge of a value in a known range (HTML `<meter>`), its fill dressed by the region `low`, `high` and `optimum` put the value in.",
    },
];

const STEPS: &[&str] = &crate::style_file::STEP_NAMES;

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

const BORDER_STYLES: &[&str] = &[
    "none", "hidden", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
];
const OUTLINE_STYLES: &[&str] = &[
    "none", "auto", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
];
const OVERFLOW: &[&str] = &["visible", "hidden", "clip", "scroll", "auto"];

/// Every core prop, by kind group: the box, the element model (docs/ui.md
/// §6.1), then each kind's own.
pub const CORE_PROPS: &[PropSpec] = &[
    // The box.
    p!("width", Ty::Size, "Its width (fit by default)."),
    p!("height", Ty::Size, "Its height (fit by default)."),
    p!("padding", Ty::Insets, "Space inside its edges, CSS px."),
    p!(
        "paddingTop",
        Ty::Num,
        "Space inside its top edge, CSS px (over `padding`)."
    ),
    p!(
        "paddingRight",
        Ty::Num,
        "Space inside its right edge, CSS px (over `padding`)."
    ),
    p!(
        "paddingBottom",
        Ty::Num,
        "Space inside its bottom edge, CSS px (over `padding`)."
    ),
    p!(
        "paddingLeft",
        Ty::Num,
        "Space inside its left edge, CSS px (over `padding`)."
    ),
    p!(
        "minWidth",
        Ty::Size,
        "Its least width: CSS px, a percentage or calc() of the width its parent offers, or minContent or maxContent."
    ),
    p!(
        "maxWidth",
        Ty::Size,
        "Its greatest width: CSS px, a percentage or calc() of the width its parent offers, or minContent or maxContent."
    ),
    p!(
        "minHeight",
        Ty::Size,
        "Its least height: CSS px, a percentage or calc() of the height its parent offers, or minContent or maxContent."
    ),
    p!(
        "maxHeight",
        Ty::Size,
        "Its greatest height: CSS px, a percentage or calc() of the height its parent offers, or minContent or maxContent."
    ),
    p!(
        "boxSizing",
        Ty::Word(&["borderBox", "contentBox"]),
        "Whether its width and height hold its padding and border (borderBox, every element's default) or its content alone (contentBox)."
    ),
    p!(
        "marginTop",
        Ty::Num4,
        "Space outside its top edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). Adjacent block margins collapse."
    ),
    p!(
        "marginRight",
        Ty::Num4,
        "Space outside its right edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space)."
    ),
    p!(
        "marginBottom",
        Ty::Num4,
        "Space outside its bottom edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space). Adjacent block margins collapse."
    ),
    p!(
        "marginLeft",
        Ty::Num4,
        "Space outside its left edge: CSS px, em of its type step, a percentage of its parent's content width, or auto (the free space)."
    ),
    p!("gap", Ty::Num, "Space between its children, CSS px."),
    p!(
        "align",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "spaceBetween",
            "spaceAround",
            "spaceEvenly",
            "stretch",
            "flexStart",
            "flexEnd",
            "normal"
        ]),
        "Where children sit along the main axis (`start` and `end` are the writing mode's sides, `flexStart` and `flexEnd` the line's; `normal`: a line's start, a grid's `auto` tracks stretched)."
    ),
    p!(
        "cross",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "stretch",
            "baseline",
            "lastBaseline"
        ]),
        "Where children sit across it."
    ),
    p!(
        "wrap",
        Ty::Bool,
        "A row (or a column) breaks its children into lines."
    ),
    p!(
        "wrapReverse",
        Ty::Bool,
        "Its lines stack from the far side (CSS `wrap-reverse`)."
    ),
    p!(
        "rowGap",
        Ty::Num,
        "Space between its rows (a column's children, a row's lines), CSS px, over `gap`."
    ),
    p!(
        "columnGap",
        Ty::Num,
        "Space between its columns (a row's children, a column's lines), CSS px, over `gap`."
    ),
    p!(
        "alignContent",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "spaceBetween",
            "spaceAround",
            "spaceEvenly",
            "stretch"
        ]),
        "Where a wrapping container's lines sit across it, and how a grid's tracks sit down it (CSS `align-content`)."
    ),
    p!(
        "justifyItems",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "stretch",
            "baseline",
            "lastBaseline"
        ]),
        "Where a grid's items sit across their areas (CSS `justify-items`)."
    ),
    p!(
        "safeJustify",
        Ty::Bool,
        "Its main-axis alignment keeps an overflow at the start (CSS `safe`)."
    ),
    p!(
        "safeAlign",
        Ty::Bool,
        "Its cross-axis alignment keeps an overflow at the start (CSS `safe`)."
    ),
    p!(
        "flexGrow",
        Ty::Num,
        "Its share of a flex line's free space (CSS `flex-grow`)."
    ),
    p!(
        "flexShrink",
        Ty::Num,
        "How much it gives up when a flex line overflows (CSS `flex-shrink`)."
    ),
    p!(
        "flexBasis",
        Ty::Size,
        "Its size in a flex line before the free space is shared (CSS `flex-basis`)."
    ),
    p!(
        "order",
        Ty::Num,
        "Its place among its flex or grid siblings (CSS `order`)."
    ),
    p!(
        "alignSelf",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "stretch",
            "baseline",
            "lastBaseline"
        ]),
        "Where it sits across its flex line or down its grid area, over its parent's (CSS `align-self`)."
    ),
    p!(
        "justifySelf",
        Ty::Word(&[
            "start",
            "center",
            "end",
            "stretch",
            "baseline",
            "lastBaseline"
        ]),
        "Where it sits across its grid area or its absolute box, over its parent's (CSS `justify-self`)."
    ),
    p!(
        "direction",
        Ty::Word(&["row", "column", "rowReverse", "columnReverse"]),
        ["row", "column", "box"],
        "Lays a row's or a column's children out the other way (an `at` block's: a phone in landscape puts the actions beside the content)."
    ),
    p!("background", Ty::Paint, "Its fill."),
    p!("radius", Ty::Num, "Its corner radius, CSS px."),
    p!(
        "radiusTopLeft",
        Ty::Text,
        "Its top left corner's radii: `<h> [<v>]`, px or %."
    ),
    p!(
        "radiusTopRight",
        Ty::Text,
        "Its top right corner's radii: `<h> [<v>]`, px or %."
    ),
    p!(
        "radiusBottomRight",
        Ty::Text,
        "Its bottom right corner's radii: `<h> [<v>]`, px or %."
    ),
    p!(
        "radiusBottomLeft",
        Ty::Text,
        "Its bottom left corner's radii: `<h> [<v>]`, px or %."
    ),
    p!(
        "cornerTopLeftShape",
        Ty::Text,
        "Its top left corner's shape: `superellipse(<K>)`."
    ),
    p!(
        "cornerTopRightShape",
        Ty::Text,
        "Its top right corner's shape: `superellipse(<K>)`."
    ),
    p!(
        "cornerBottomRightShape",
        Ty::Text,
        "Its bottom right corner's shape: `superellipse(<K>)`."
    ),
    p!(
        "cornerBottomLeftShape",
        Ty::Text,
        "Its bottom left corner's shape: `superellipse(<K>)`."
    ),
    p!(
        "borderTopWidth",
        Ty::Num,
        "Its top border's width, CSS px: room inside its box, as padding is."
    ),
    p!(
        "borderTopStyle",
        Ty::Word(BORDER_STYLES),
        "Its top border's style (none draws nothing and takes no room)."
    ),
    p!(
        "borderTopColor",
        Ty::Paint,
        "Its top border's colour (unset: its text colour)."
    ),
    p!(
        "borderRightWidth",
        Ty::Num,
        "Its right border's width, CSS px: room inside its box, as padding is."
    ),
    p!(
        "borderRightStyle",
        Ty::Word(BORDER_STYLES),
        "Its right border's style (none draws nothing and takes no room)."
    ),
    p!(
        "borderRightColor",
        Ty::Paint,
        "Its right border's colour (unset: its text colour)."
    ),
    p!(
        "borderBottomWidth",
        Ty::Num,
        "Its bottom border's width, CSS px: room inside its box, as padding is."
    ),
    p!(
        "borderBottomStyle",
        Ty::Word(BORDER_STYLES),
        "Its bottom border's style (none draws nothing and takes no room)."
    ),
    p!(
        "borderBottomColor",
        Ty::Paint,
        "Its bottom border's colour (unset: its text colour)."
    ),
    p!(
        "borderLeftWidth",
        Ty::Num,
        "Its left border's width, CSS px: room inside its box, as padding is."
    ),
    p!(
        "borderLeftStyle",
        Ty::Word(BORDER_STYLES),
        "Its left border's style (none draws nothing and takes no room)."
    ),
    p!(
        "borderLeftColor",
        Ty::Paint,
        "Its left border's colour (unset: its text colour)."
    ),
    p!(
        "outlineWidth",
        Ty::Num,
        "Its outline's width, CSS px, drawn outside its box and taking no room."
    ),
    p!(
        "outlineStyle",
        Ty::Word(OUTLINE_STYLES),
        "Its outline's style; auto: the focus ring."
    ),
    p!(
        "outlineColor",
        Ty::Paint,
        "Its outline's colour (unset: its text colour)."
    ),
    p!(
        "outlineOffset",
        Ty::Num,
        "How far, CSS px, its outline stands off its box."
    ),
    p!(
        "boxShadow",
        Ty::Text,
        "Its shadows, as CSS box-shadow: `[inset] <x> <y> [<blur> [<spread>]] [<colour>]`, comma-separated, the first on top."
    ),
    p!(
        "overflowX",
        Ty::Word(OVERFLOW),
        "What its content past its left and right edges does: visible, hidden or clip (cut), scroll or auto (scrolls)."
    ),
    p!(
        "overflowY",
        Ty::Word(OVERFLOW),
        "What its content past its top and bottom edges does: visible, hidden or clip (cut), scroll or auto (scrolls)."
    ),
    p!(
        "scrollbarWidth",
        Ty::Word(&["auto", "thin", "none"]),
        "CSS `scrollbar-width`: thin draws its scrollbars at half the width (a half-width grab strip); none hides them (it still scrolls)."
    ),
    p!(
        "rive",
        Ty::Name,
        ["column"],
        "The Rive artboard played by `<og-rive src=\"ui/chest.riv#Chest\">`: `<file>.riv#<Artboard>` relative to assets/. Loaded with its document, advanced on the tick, drawn over its background and saved with the UI's state; its drawn objects are elements for dump_ui, describe and pick."
    ),
    p!("aspect", Ty::Num, "Width over height (0: none)."),
    p!(
        "fade",
        Ty::Num,
        ["scroll"],
        "How far, CSS px, its edges fade where its content runs on past them (a cut row reads as more, not as a clip); 0: none."
    ),
    p!(
        "fadeInto",
        Ty::Paint,
        ["scroll"],
        "The colour its fade runs into: the ground it sits on (default panel)."
    ),
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
        "player",
        Ty::Num,
        "A split screen's region: player 0 to 3's view. That player's pad focus moves only inside it, and what is pressed inside it says the player."
    ),
    p!(
        "role",
        Ty::Word(crate::a11y::ROLES),
        "Its ARIA role, over its tag's (`status`, `alert`, `log` are live regions)."
    ),
    p!(
        "ariaLive",
        Ty::Word(&["off", "polite", "assertive"]),
        "`aria-live`: a screen reader announces what changes in it, after what it is reading (`polite`) or at once (`assertive`); over its role's."
    ),
    p!(
        "ariaAtomic",
        Ty::Bool,
        "`aria-atomic`: a live region announces its whole text when any of it changes, not only what changed; over its role's."
    ),
    p!(
        "part",
        Ty::Name,
        "Its part name a style dresses (`ShopCard.frame`); the document's `parts` lists them."
    ),
    // The element model.
    p!("x", Ty::Num, "Moved right after layout, CSS px."),
    p!("y", Ty::Num, "Moved down after layout, CSS px."),
    p!(
        "xShift",
        Ty::Num,
        "Moved right after layout, a fraction of its own width (`translate`'s %)."
    ),
    p!(
        "yShift",
        Ty::Num,
        "Moved down after layout, a fraction of its own height (`translate`'s %)."
    ),
    p!("scale", Ty::Num, "Both scales."),
    p!("scaleX", Ty::Num, "Its horizontal scale."),
    p!("scaleY", Ty::Num, "Its vertical scale."),
    p!("rotate", Ty::Num, "A turn, radians, clockwise."),
    p!(
        "pivot",
        Ty::Num2,
        "The transform's origin, fractions of its box."
    ),
    p!("opacity", Ty::Num, "0 to 1, multiplied down the tree."),
    p!("z", Ty::Num, "Drawing order among its siblings."),
    p!(
        "position",
        Ty::Word(&["flow", "absolute", "screen", "relative", "sticky"]),
        "In the flow, placed by its insets in its parent, by its insets on the whole screen (full bleed, under the safe area), or against another element."
    ),
    p!(
        "start",
        Ty::Num,
        "The inset from its parent's start edge (absolute)."
    ),
    p!("top", Ty::Num, "The inset from the top."),
    p!("end", Ty::Num, "The inset from the end edge."),
    p!("bottom", Ty::Num, "The inset from the bottom."),
    p!(
        "dataHitSlop",
        Ty::Num,
        "Extra pointer reach in CSS px (`data-hit-slop`), without changing layout or paint. Overlapping targets keep their existing hit priority."
    ),
    p!(
        "columns",
        Ty::Tracks,
        "A grid's column tracks (CSS `grid-template-columns`: sizes, `[line names]`, `repeat()`, `subgrid` or `masonry`)."
    ),
    p!(
        "rows",
        Ty::Tracks,
        "A grid's row tracks (CSS `grid-template-rows`)."
    ),
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
    p!(
        "autoColumns",
        Ty::Tracks,
        "A grid's implicit column tracks, repeating (CSS `grid-auto-columns`)."
    ),
    p!(
        "autoRows",
        Ty::Tracks,
        "A grid's implicit row tracks, repeating (CSS `grid-auto-rows`)."
    ),
    p!(
        "columnStart",
        Ty::Name,
        "Its first grid column line: `<n> [<name>]`, `<name>`, `span [<n>] [<name>]` or auto (CSS `grid-column-start`)."
    ),
    p!(
        "columnEnd",
        Ty::Name,
        "The grid column line it ends at (CSS `grid-column-end`)."
    ),
    p!(
        "rowStart",
        Ty::Name,
        "Its first grid row line (CSS `grid-row-start`)."
    ),
    p!(
        "rowEnd",
        Ty::Name,
        "The grid row line it ends at (CSS `grid-row-end`)."
    ),
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
    p!(
        "colspan",
        Ty::Num,
        ["box"],
        "A table cell's columns (`<td colspan>`), 1 by default; CSS table layout reads it."
    ),
    p!(
        "rowspan",
        Ty::Num,
        ["box"],
        "A table cell's rows (`<td rowspan>`), 1 by default, 0 to the end of its row group; CSS table layout reads it."
    ),
    p!("enter", Ty::Presence, "How it arrives."),
    p!("exit", Ty::Presence, "How it leaves."),
    p!("inert", Ty::Bool, "It and its subtree take no input."),
    p!(
        "navUp",
        Ty::Name,
        "Where Up moves the focus from it (a path)."
    ),
    p!("navDown", Ty::Name, "Where Down moves the focus from it."),
    p!("navLeft", Ty::Name, "Where Left moves the focus from it."),
    p!("navRight", Ty::Name, "Where Right moves the focus from it."),
    p!("navNext", Ty::Name, "Where Tab moves the focus from it."),
    p!(
        "navPrev",
        Ty::Name,
        "Where Shift+Tab moves the focus from it."
    ),
    p!(
        "focusGroup",
        Ty::Word(&["stop", "wrap"]),
        "Its focusable controls are a focus group that stops (leaves) or wraps at its edge."
    ),
    p!(
        "focusMemory",
        Ty::Bool,
        "With focusGroup: focus coming back lands on its last member."
    ),
    p!("tap", Ty::Bool, "It recognises a tap (`on tap`)."),
    p!(
        "doubleTap",
        Ty::Bool,
        "It recognises a double tap (`on doubleTap`); its tap waits for it."
    ),
    p!(
        "longPress",
        Ty::Bool,
        "It recognises a long press (`on longPress`)."
    ),
    p!(
        "longPressMs",
        Ty::Num,
        "How long its long press is held, ms (0: 500)."
    ),
    p!(
        "gesturePriority",
        Ty::Num,
        "Among nested recognisers of one press: higher wins."
    ),
    p!(
        "cursor",
        Ty::Text,
        "CSS `cursor`: the pointer's shape over it and its subtree, any CSS keyword. `url(<file>) [<x> <y>]` images resolve beside their stylesheet and are tried in order. The first readable image is shown at its hotspot (at most 128x128), the keyword where none does."
    ),
    p!(
        "pointerEvents",
        Ty::Text,
        "CSS `pointer-events`, inherited: none lets the pointer through to what is under it (keys and pads still reach it; a child saying auto still takes it). On SVG shapes the SVG values pick the fill, the stroke or the bounding box."
    ),
    p!(
        "touchAction",
        Ty::Text,
        "CSS `touch-action`: the pans (pan-x, pan-left…, pan-y…) and pinch-zoom a touch starting on it may make; none keeps the touch for its own handlers. The used value intersects its ancestors'."
    ),
    p!(
        "imageRendering",
        Ty::Text,
        "CSS `image-rendering`, inherited: pixelated or crisp-edges samples every image it draws (its image, art, background and border image) at the nearest texel, so pixel art scaled up stays sharp; auto, smooth and high-quality keep the texture's own sampling."
    ),
    p!(
        "visibility",
        Ty::Text,
        "CSS `visibility`, inherited: hidden keeps its place but draws nothing and takes no pointer or focus, while a child saying visible is drawn; collapse also gives up a flex item's main size."
    ),
    p!(
        "textPath",
        Ty::Text,
        "Its text set along a path (`-og-text-path`): `none`, `path(\"<svg path data>\")` in px from its content box, or `circle([<n>px])` centred in it."
    ),
    p!(
        "textPathOffset",
        Ty::Text,
        "Where its text's anchor (text-align) sits along its path: `<n>px` or `<n>%` of the path's length."
    ),
    p!(
        "textPathSide",
        Ty::Word(&["left", "right"]),
        "Which side of its path the text stands on: right runs the path backwards (inside a circle)."
    ),
    p!(
        "userSelect",
        Ty::Word(&["auto", "none", "text", "all"]),
        "Whether its text and its subtree's can be selected and copied; auto takes its parent's (none at the top and under a control)."
    ),
    p!(
        "customize",
        Ty::Bool,
        "The player can move, scale and fade it in the HUD edit mode."
    ),
    p!(
        "customizeKey",
        Ty::Name,
        "With customize: its name in the saved HUD layout (default: its path)."
    ),
    // Tooltips and popovers (docs/ui.md §2: HTML's).
    p!(
        "title",
        Ty::Text,
        "Its tooltip, shown after the pointer rests on it (HTML `title`); also its description for a screen reader."
    ),
    p!(
        "popover",
        Ty::Word(&["auto", "manual", "hint"]),
        "Shown over the page against the control that opened it, not in its place (HTML `popover`): auto closes on a press outside or Back and closes other auto ones; manual only when a control closes it; hint is a rich tooltip an `interestfor` control shows."
    ),
    p!(
        "popovertarget",
        Ty::Name,
        "The id of the popover a press on it opens, closes or toggles (HTML `popovertarget`)."
    ),
    p!(
        "popovertargetaction",
        Ty::Word(&["toggle", "show", "hide"]),
        "What its press does to its popovertarget (default toggle)."
    ),
    p!(
        "interestfor",
        Ty::Name,
        "The id of the popover shown while the pointer rests on it or it holds the keyboard focus (HTML `interestfor`)."
    ),
    p!(
        "contextmenu",
        Ty::Name,
        "The id of the popover a secondary press or a long press on it opens at the pointer, its anchor that point (HTML's former `contextmenu`): a context menu."
    ),
    // ARIA (docs/ui.md §2: semantics through ARIA).
    p!(
        "role",
        Ty::Name,
        "Its accessibility role over its kind's (ARIA `role`; one of og.ui's roles)."
    ),
    p!(
        "ariaDescription",
        Ty::Text,
        "Read after its name (`aria-description`, or the text `aria-describedby` names)."
    ),
    p!(
        "ariaValuemin",
        Ty::Num,
        "Its range's low end for a screen reader (`aria-valuemin`)."
    ),
    p!(
        "ariaValuemax",
        Ty::Num,
        "Its range's high end for a screen reader (`aria-valuemax`)."
    ),
    p!(
        "ariaValuenow",
        Ty::Num,
        "Its value as a number for a screen reader (`aria-valuenow`)."
    ),
    p!(
        "ariaValuetext",
        Ty::Text,
        "Its value as a screen reader says it (`aria-valuetext`)."
    ),
    p!(
        "ariaChecked",
        Ty::Bool,
        "Checked, for a screen reader (`aria-checked`)."
    ),
    p!(
        "ariaSelected",
        Ty::Bool,
        "Selected, for a screen reader (`aria-selected`): a tab, an option, a row."
    ),
    p!(
        "ariaExpanded",
        Ty::Bool,
        "Expanded or collapsed (`aria-expanded`): a disclosure, a tree item, a menu button."
    ),
    p!(
        "ariaPressed",
        Ty::Bool,
        "A toggle button's pressed state (`aria-pressed`): a chip, a filter."
    ),
    p!(
        "ariaBusy",
        Ty::Bool,
        "Busy (`aria-busy`): loading or working."
    ),
    p!(
        "ariaDisabled",
        Ty::Bool,
        "Disabled for a screen reader and inert (`aria-disabled`)."
    ),
    p!(
        "ariaLevel",
        Ty::Num,
        "Its level, 1 and up (`aria-level`): a heading's, a tree item's."
    ),
    p!(
        "ariaCurrent",
        Ty::Word(&["page", "step", "location", "date", "time", "true", "false"]),
        "The current item in a set (`aria-current`): a pagination's page, a breadcrumb's last, a wizard's step."
    ),
    // Glyphs.
    p!(
        "dataAction",
        Ty::Name,
        ["kbd", "button"],
        "On `<kbd>`, the action whose glyph it shows on the device used last (its key, its pad button, or its touch icon); it follows the device live. On a `<button data-hold>`, the action whose hold holds it as a press does."
    ),
    p!(
        "glyphDevice",
        Ty::Text,
        ["kbd"],
        "The device it draws for (bound to `$ui.device` by the parser: a device change rebuilds it)."
    ),
    p!(
        "dataOgRebind",
        Ty::Bool,
        ["kbd"],
        "A press captures the next key as its binding (`bind:value`), Escape cancels."
    ),
    p!(
        "binding",
        Ty::Text,
        ["kbd"],
        two_way,
        "With dataOgRebind: the key code it captured (`bind:value`), written back."
    ),
    p!(
        "dataOgInstant",
        Ty::Bool,
        "Its first entrance is settled (`data-og-instant`): the screen a game opens on, which the web draws behind its play gate. Later entrances play."
    ),
    p!(
        "reduced",
        Ty::Word(&["fade", "none", "keep"]),
        "Its reduced-motion fallback."
    ),
    // Text.
    p!(
        "text",
        Ty::Text,
        ["label", "button", "badge", "toggle", "radio", "tabs", "kbd"],
        "What it says."
    ),
    p!(
        "key",
        Ty::Name,
        ["label", "button", "badge", "toggle", "radio", "tabs", "kbd"],
        "A string table key (og.i18n) its text comes from; `text` is the fallback."
    ),
    p!("step", Ty::Word(STEPS), ["label"], "The type step."),
    p!("color", Ty::Paint, ["label", "icon"], "Its ink."),
    p!(
        "spans",
        Ty::Spans,
        ["label"],
        "Read-only TextSpan[] ranges over plain text; offsets count Unicode scalars. Styles come from ::part(name)."
    ),
    p!(
        "uppercase",
        Ty::Bool,
        ["label"],
        "Uppercase in the UI's locale, as a type role with `case upper` is."
    ),
    p!(
        "secondary",
        Ty::Name,
        ["icon"],
        "The second icon layer, in the same atlas."
    ),
    p!(
        "textAlign",
        Ty::Word(&["start", "center", "end"]),
        ["label"],
        "How its lines align."
    ),
    p!("nowrap", Ty::Bool, ["label"], "One line."),
    p!("maxLines", Ty::Num, ["label"], "The most lines it shows."),
    p!(
        "fitMin",
        Ty::Word(STEPS),
        ["label"],
        "Shrinks by measured text fit through type steps down to this literal step (for example -og-fit-min: caption). Extension reason: clamp() and container units cannot choose a type step by measuring shaped text. Author variables may substitute a step name; no custom-property name has engine meaning."
    ),
    p!(
        "ellipsis",
        Ty::Bool,
        ["label"],
        "Cut with … where it does not fit."
    ),
    p!(
        "ellipsisText",
        Ty::Text,
        ["label"],
        "What its cut ends in instead of … (`text-overflow: \"<string>\"`)."
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
        Ty::Word(&["secondary", "primary", "quiet", "danger", "ghost"]),
        ["button"],
        "The button's role."
    ),
    p!(
        "disabled",
        Ty::Bool,
        [
            "button", "slider", "toggle", "radio", "color", "datetime", "file", "tabs", "field",
            "select", "details", "joystick", "dpad"
        ],
        "Takes no input."
    ),
    p!(
        "autofocus",
        Ty::Bool,
        ["button"],
        "Takes the focus when it appears."
    ),
    p!(
        "square",
        Ty::Bool,
        ["button"],
        "A square of the control height: an icon button."
    ),
    p!(
        "value",
        Ty::Num,
        ["progress", "slider", "meter"],
        two_way,
        "Its value (a slider writes it back)."
    ),
    p!(
        "min",
        Ty::Num,
        ["slider", "field", "meter"],
        "The range's low end; a number input under it is invalid (`rangeUnderflow`)."
    ),
    p!(
        "max",
        Ty::Num,
        ["slider", "field", "meter"],
        "The range's high end; a number input over it is invalid (`rangeOverflow`)."
    ),
    p!(
        "showValue",
        Ty::Bool,
        ["slider"],
        "Show the slider's value readout (default true); false when an adjacent label supplies units."
    ),
    p!(
        "dataFillOrigin",
        Ty::Num,
        ["slider"],
        "The value its fill grows from (`data-fill-origin`; default min): 0 on a -1 to 1 balance fills either way from the centre."
    ),
    p!(
        "markValues",
        Ty::Names,
        ["slider"],
        "The values of the `<datalist>` its `list` names: a tick at each."
    ),
    p!(
        "markLabels",
        Ty::Names,
        ["slider"],
        "The marks' labels, under the track (an `<option label>`'s, or its text); empty for a tick only."
    ),
    p!(
        "stepBy",
        Ty::Num,
        ["slider", "field", "datetime"],
        "A slider snaps to multiples of this (0: continuous). A number input's value off the step grid from `min` is invalid (default 1; `any` or 0: no step)."
    ),
    p!(
        "switch",
        Ty::Bool,
        ["toggle"],
        "Draws the checkbox as a switch (HTML `switch`); without it, a box with a check."
    ),
    p!(
        "indeterminate",
        Ty::Bool,
        ["toggle"],
        "Neither on nor off: a dash in its box (HTML `indeterminate`); a press turns it on."
    ),
    p!(
        "dataOn",
        Ty::Text,
        ["toggle"],
        "What a switch's value says while on (`data-on=\"On\"`), for a screen reader and the probe; empty for none."
    ),
    p!(
        "dataOff",
        Ty::Text,
        ["toggle"],
        "And while off (`data-off=\"Off\"`)."
    ),
    p!(
        "group",
        Ty::Text,
        ["radio"],
        two_way,
        "The store its group's choice is in (`bind:group`): it is checked while that holds its value, and a press writes its value."
    ),
    p!(
        "mediaSrc",
        Ty::Num,
        ["video", "audio"],
        "The player (HTML `src`): an entity with og.VideoPlayer (a video's) or og.AudioSource, from a store."
    ),
    p!(
        "paused",
        Ty::Bool,
        ["video", "audio"],
        two_way,
        "Whether it is paused (Svelte's `bind:paused`): written to play or pause it, written back as it ends."
    ),
    p!(
        "currentTime",
        Ty::Num,
        ["video"],
        two_way,
        "Seconds in (`bind:currentTime`): written back as it plays; a value off the player's seeks it."
    ),
    p!(
        "duration",
        Ty::Num,
        ["video"],
        two_way,
        "Its length in seconds (`bind:duration`; written back)."
    ),
    p!(
        "volume",
        Ty::Num,
        ["video", "audio"],
        two_way,
        "Its sound's gain, 0 to 1 (`bind:volume`; default 1)."
    ),
    p!(
        "muted",
        Ty::Bool,
        ["video", "audio"],
        two_way,
        "Silenced whatever its volume (`bind:muted`)."
    ),
    p!(
        "ended",
        Ty::Bool,
        ["video"],
        two_way,
        "It reached its end without loop (`bind:ended`; written back)."
    ),
    p!(
        "autoplay",
        Ty::Bool,
        ["video", "audio"],
        "Plays as it first appears."
    ),
    p!(
        "loop",
        Ty::Bool,
        ["video", "audio"],
        "Starts again at its end."
    ),
    p!(
        "poster",
        Ty::Num,
        ["video"],
        "A texture handle shown until the first frame plays (HTML `poster`), usually bound."
    ),
    p!(
        "controls",
        Ty::Bool,
        ["video", "audio"],
        "Draws a play and pause button and its time."
    ),
    p!(
        "fileValue",
        Ty::Text,
        ["file"],
        two_way,
        "The names of the files chosen, comma-separated (HTML `value`; the host writes it)."
    ),
    p!(
        "accept",
        Ty::Text,
        ["file"],
        "The types it takes (HTML `accept`: `.png,image/*`)."
    ),
    p!(
        "multiple",
        Ty::Bool,
        ["file"],
        "More than one file may be chosen."
    ),
    p!(
        "dateValue",
        Ty::Text,
        ["datetime"],
        two_way,
        "Its value in HTML's format (`2026-11-08`, `14:30`, `2026-11-08T14:30`, `2026-11`, `2026-W45`; written back); empty until a part is stepped."
    ),
    p!(
        "dateType",
        Ty::Word(&["date", "time", "datetime-local", "month", "week"]),
        ["datetime"],
        "Which parts it has (HTML `type`)."
    ),
    p!(
        "dateMin",
        Ty::Text,
        ["datetime"],
        "The earliest value (HTML `min`, in its format); the picker keeps to it and is the step base."
    ),
    p!(
        "dateMax",
        Ty::Text,
        ["datetime"],
        "The latest value (HTML `max`, in its format)."
    ),
    p!(
        "colorValue",
        Ty::Text,
        ["color"],
        two_way,
        "Its colour as `#rrggbb` (HTML `value`; written back)."
    ),
    p!(
        "choice",
        Ty::Text,
        ["radio"],
        "Its value (HTML `value`), what its group holds when it is checked."
    ),
    p!(
        "dataWrap",
        Ty::Bool,
        ["field"],
        "A number input past its last valid step at or below max goes to min and back (`data-wrap`; needs both bounds)."
    ),
    p!(
        "dataLargeStep",
        Ty::Num,
        ["field"],
        "The larger number-input step for Shift adjustment or a button held past a second (`data-large-step=\"10\"`); 0 is ten steps."
    ),
    p!(
        "dataFormat",
        Ty::Text,
        ["field"],
        "How an unfocused number input presents its value: an ICU message over `value` in the UI's locale (`data-format=\"{value, number} m\"`, `{value, number, compact}`, `{value, plural, =0 {Off} =1 {Low} other {High}}`). Editing retains the raw string."
    ),
    p!(
        "count",
        Ty::Num,
        ["label"],
        "The target value of <og-counter value={$score}>. Changes interpolate over its duration in engine ticks."
    ),
    p!(
        "countDuration",
        Ty::Num,
        ["label"],
        "The counter's duration in milliseconds (900 by default; zero also selects the default)."
    ),
    p!(
        "countDecimals",
        Ty::Num,
        ["label"],
        "The counter's digits after the decimal point."
    ),
    p!(
        "reveal",
        Ty::Num,
        ["label"],
        "The typewriter's extended grapheme clusters per second; onrevealed fires once when the text is whole. Text changes restart it. Reduced motion shows the whole text. Progress follows engine ticks and checkpoints."
    ),
    p!(
        "dataCycle",
        Ty::Bool,
        ["select"],
        "`<select data-cycle>` draws previous, its option and next; selection, events and bindings remain those of the select."
    ),
    p!(
        "dataFilterPlaceholder",
        Ty::Text,
        ["select"],
        "What the open list's filter field shows while empty (`data-filter-placeholder`); empty is Filter."
    ),
    p!(
        "spotlight",
        Ty::Name,
        ["dialog"],
        "The id tracked by <og-spotlight target=\"Target\">. While open it dims the page outside that element and places its content as a callout."
    ),
    p!(
        "dataHold",
        Ty::Num,
        "How long a press is held before `onhold` fires, in ms (`data-hold`; default 500). While it is held a fill grows across the element from its start edge, drawn over its background and under its content and coloured by `::-og-hold { background }` (else the style's primary at a third); with `data-action`, holding that action holds it too."
    ),
    p!(
        "checked",
        Ty::Bool,
        ["toggle", "radio"],
        two_way,
        "Whether it is on (written back)."
    ),
    p!(
        "description",
        Ty::Text,
        ["toggle", "radio"],
        "A line under its caption: a `<small>` in its `<label>`."
    ),
    p!(
        "side",
        Ty::Word(&["start", "end"]),
        ["toggle", "radio"],
        "Where its caption sits, from the source order in its `<label>`: start before the switch (caption first), end after it."
    ),
    p!(
        "labelDirection",
        Ty::Word(&["row", "rowReverse", "column", "columnReverse"]),
        ["toggle"],
        "`flex-direction` on a checkbox's label: rowReverse swaps its caption's side."
    ),
    p!(
        "items",
        Ty::Names,
        ["tabs", "select"],
        "The tab or option labels, literal names or a bound list of strings (a `<select>`'s `<option>`s set them)."
    ),
    p!(
        "keys",
        Ty::Names,
        ["tabs", "select"],
        "A string table key (og.i18n) per tab its label comes from; its item is the fallback."
    ),
    p!(
        "icons",
        Ty::Names,
        ["tabs"],
        "An icon of the style's set per tab, before its label (`icons=\"sword shield\"`, or a bound list); an empty name is none."
    ),
    p!(
        "badges",
        Ty::Names,
        ["tabs"],
        "A count badge per tab, after its label (`badges={$unread}`, a list of strings); an empty one is none."
    ),
    p!(
        "values",
        Ty::Names,
        ["tabs", "select"],
        "A name per tab (a screen's mode) that `current` holds instead of `selected`'s index."
    ),
    p!(
        "current",
        Ty::Name,
        ["tabs", "select"],
        two_way,
        "The selected item's name from `values` (a `<select>`'s `value`); on tabs it takes over from `selected` when `values` is set."
    ),
    p!(
        "selected",
        Ty::Num,
        ["tabs"],
        two_way,
        "The selected tab's zero-based index."
    ),
    p!(
        "groups",
        Ty::Names,
        ["select"],
        "An `<option>`'s `<optgroup>` label, one per option (empty: none); options of one group follow each other."
    ),
    p!(
        "disabledItems",
        Ty::Names,
        ["select"],
        "The `<option disabled>`s (and those of an `<optgroup disabled>`), by their zero-based index."
    ),
    p!(
        "open",
        Ty::Bool,
        ["dialog", "details"],
        two_way,
        "Whether it is open (HTML `open`; closing writes false back)."
    ),
    p!(
        "closedby",
        Ty::Word(&["any", "closerequest", "none"]),
        ["dialog"],
        "What closes it (HTML `closedby`): any adds a press outside a modal (light dismiss) to closerequest's Escape or a pad's B; none leaves closing to the document. Default: closerequest for a modal, none in place."
    ),
    p!(
        "appearance",
        Ty::Word(&["auto", "none"]),
        ["dialog", "field"],
        "none drops the control's own frame: its box, borders and background are the document's."
    ),
    p!(
        "display",
        Ty::Word(&[
            "block",
            "inline",
            "inlineBlock",
            "flowRoot",
            "listItem",
            "flex",
            "inlineFlex",
            "grid",
            "inlineGrid",
            "table",
            "inlineTable",
            "contents",
            "tableRowGroup",
            "tableHeaderGroup",
            "tableFooterGroup",
            "tableRow",
            "tableCell",
            "tableCaption",
            "tableColumn",
            "tableColumnGroup"
        ]),
        "Its CSS `display`. On a box its layout: `block` stacks its children top to bottom (the default), `flex` lays them out as a row (or `flex-direction`), `grid` in tracks, `table` (and its parts) as a table, `flowRoot` a block whose children's margins stay inside, `contents` no box at all (its children are its parent's). On any element `inline…` puts it in lines beside its inline-level siblings."
    ),
    p!(
        "verticalAlign",
        Ty::Word(&[
            "baseline",
            "top",
            "middle",
            "bottom",
            "textTop",
            "textBottom",
            "sub",
            "super"
        ]),
        "CSS `vertical-align`: where an inline-level element sits in its line, and a table cell's content in its row (`middle` by default)."
    ),
    p!(
        "captionSide",
        Ty::Word(&["top", "bottom"]),
        "CSS `caption-side`: a table caption above (default) or below its table."
    ),
    p!(
        "markerType",
        Ty::Word(&[
            "none",
            "disc",
            "circle",
            "square",
            "decimal",
            "lowerAlpha",
            "upperAlpha",
            "lowerRoman",
            "upperRoman"
        ]),
        ["box"],
        "An `<li>`'s marker, from the inherited `list-style-type` (`<ul>`: disc, nested circle then square; `<ol>`: decimal)."
    ),
    p!(
        "markerPosition",
        Ty::Word(&["outside", "inside"]),
        ["box"],
        "Where an `<li>`'s marker sits, from the inherited `list-style-position`: `outside` hangs it before the item's start edge (the default), `inside` lays the item out as a row after it."
    ),
    p!(
        "markerImage",
        Ty::Text,
        "CSS `list-style-image`: an inherited image URL replacing a list item's marker, or none. Only list items draw the image."
    ),
    p!(
        "href",
        Ty::Text,
        ["label"],
        "An `<a>`'s destination: a click sends a `link` event named by it (as `onclick` fires)."
    ),
    p!(
        "modal",
        Ty::Bool,
        ["dialog"],
        "Over a backdrop that takes every press, the focus trapped inside (HTML `showModal()`); else a panel in place (`show()`)."
    ),
    p!(
        "summary",
        Ty::Text,
        ["details"],
        "Its `<summary>`'s text; empty is `Details`. Written as the first child, `<summary>…</summary>`."
    ),
    p!(
        "exclusive",
        Ty::Name,
        ["details"],
        "HTML `name` on a `<details>`, an exclusive accordion: of the disclosures in a document sharing it, one is open at once, and opening one closes the others (its open state written back)."
    ),
    p!(
        "low",
        Ty::Num,
        ["meter", "slider"],
        two_way,
        "A meter's top of the low region (HTML `low`; default `min`). On a range input with `high`, its low knob (a slider writes it back)."
    ),
    p!(
        "high",
        Ty::Num,
        ["meter", "slider"],
        two_way,
        "A meter's bottom of the high region (HTML `high`; default `max`). On a range input, its high knob: two knobs on one track, `low` to `high` (a slider writes both back)."
    ),
    p!(
        "suggestions",
        Ty::Names,
        ["field"],
        "A text input's suggestions, from the `<datalist>` its `list` names (its options' values, or its `items={$names}`): a list under the field while it has the focus, those containing the text first by prefix, choosing one writes it to the field."
    ),
    p!(
        "suggestionLabels",
        Ty::Names,
        ["field"],
        "The labels of the input's datalist options, separate from the values written on selection."
    ),
    p!(
        "suggestionKeys",
        Ty::Names,
        ["field"],
        "Translation keys for datalist option labels; missing translations use the option's literal label."
    ),
    p!(
        "dataMinGap",
        Ty::Num,
        ["slider"],
        "The least distance between a range input's two knobs (`data-min-gap`, in its values; default 0)."
    ),
    p!(
        "optimum",
        Ty::Num,
        ["meter"],
        "The best value (HTML `optimum`; default the middle): its region is `optimum`, the next `suboptimum`, the far one `poor`."
    ),
    p!(
        "fill",
        Ty::Bool,
        ["tabs"],
        "Equal tabs filling the available width."
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
        ["field", "select"],
        "Shown while it is empty."
    ),
    p!(
        "readonly",
        Ty::Bool,
        ["field", "datetime"],
        "Its text is selected and copied but never edited; it keeps its focus stop."
    ),
    p!(
        "required",
        Ty::Bool,
        ["field", "datetime"],
        "Empty is invalid (`valueMissing`)."
    ),
    p!(
        "minChars",
        Ty::Num,
        ["field"],
        "Fewer characters are invalid (HTML `minlength`)."
    ),
    p!(
        "pattern",
        Ty::Text,
        ["field"],
        "A regular expression the whole text must match (HTML `pattern`); refused when it is not one."
    ),
    p!(
        "lines",
        Ty::Num,
        ["field"],
        "A `<textarea>`'s lines shown at once (HTML `rows`; 0 is 4); more scroll."
    ),
    p!(
        "multiline",
        Ty::Bool,
        ["field"],
        "Set by `<textarea>`: Enter starts a line and the text wraps. Not written as an attribute."
    ),
    p!(
        "maxChars",
        Ty::Num,
        ["field"],
        "The most characters it takes."
    ),
    p!(
        "tight",
        Ty::Bool,
        ["field"],
        "No inset and its text's line height: the text sits where the field is laid (in a box the screen draws)."
    ),
    p!(
        "component",
        Ty::Name,
        ["field"],
        "The component its style dresses it as (`Poem`, `Search[large]`): `<component>.frame`, `.text` and `.placeholder`; else Field."
    ),
    p!(
        "invalid",
        Ty::Bool,
        ["field", "select", "toggle"],
        "Its value is wrong (`aria-invalid`): the danger frame."
    ),
    p!(
        "type",
        Ty::Word(&crate::InputType::NAMES),
        ["field"],
        "What it takes, by HTML's names: `password` is drawn masked with a reveal toggle (`<component>.reveal[revealed]`) and never copied or cut; `number` takes only a number's characters; a `number`, `email` or `url` whose text is not one matches `:invalid`. Each picks its virtual keyboard."
    ),
    p!(
        "inputmode",
        Ty::Word(&crate::InputMode::NAMES),
        ["field"],
        "The virtual keyboard a touch device shows (HTML `inputmode`; `none`: the game shows its own); unset, its `type`'s."
    ),
    p!(
        "enterkeyhint",
        Ty::Word(&["enter", "done", "go", "next", "previous", "search", "send"]),
        ["field"],
        "The on-screen keyboard's enter key (HTML `enterkeyhint`): `Search`, `Send`, `Go` and so on; unset, `Done`."
    ),
    p!(
        "lang",
        Ty::Name,
        ["field"],
        "The language typed (HTML `lang`): `ja` offers the on-screen keyboard's kana table."
    ),
    p!(
        "autocomplete",
        Ty::Name,
        ["field"],
        "The page's autofill hint (HTML `autocomplete`: `username`, `current-password`, `new-password`, `one-time-code`, `email`, `off`…)."
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
        "uv",
        Ty::Num4,
        ["image"],
        "The part of the texture it shows (`object-view-box: inset(…)` in percentages of the texture, ordered top, right, bottom, left)."
    ),
    p!(
        "icon",
        Ty::Name,
        ["icon"],
        "The icon's name in the style's set."
    ),
    p!("iconSize", Ty::Num, ["icon"], "CSS px (24 by default)."),
    p!(
        "cue",
        Ty::Name,
        ["icon"],
        "On <og-icon cue=\"danger\">, the style's icon for that status, rarity, team or damage type, so its meaning never rests on colour alone. Its `icon`, when set, wins. Style the primary and secondary layers with ::part(image) and ::part(secondary)."
    ),
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
        Ty::Word(&[
            "cover",
            "contain",
            "fill",
            "none",
            "scaleDown",
            "nineSlice",
            "tile"
        ]),
        ["image", "art"],
        "CSS `object-fit`: how the image fills its box (fill, contain, cover, none, scale-down) or the art its slot (also nineSlice and tile)."
    ),
    p!(
        "objectPosition",
        Ty::Text,
        ["image", "art"],
        "CSS `object-position`: where contain, cover, none or scale-down place the image in its box, as background-position is written (50% 50% by default)."
    ),
    p!(
        "itemCells",
        Ty::Num2,
        ["art"],
        "An item icon's grid cells, columns and rows (1 1, 1 2, 2 3 or 2 4)."
    ),
    p!(
        "slice",
        Ty::Insets,
        ["art"],
        "A nine-slice's insets as fractions of the art: all, x y, or left top right bottom."
    ),
    p!(
        "artSet",
        Ty::Name,
        ["art"],
        "The set it is generated with for a consistent look (the kind's name by default)."
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
