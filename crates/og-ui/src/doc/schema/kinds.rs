//! Every element kind a document can name: the HTML-shaped ones, then the
//! world's, the touch controls' and SVG's. A kind is declared here once; the
//! build's match over [`Kind`] is exhaustive, so a kind cannot be named
//! without being built.

macro_rules! kinds {
    ($($variant:ident $content:ident $name:literal $doc:literal;)*) => {
        /// One element kind.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Kind {
            $(#[doc = $doc] $variant,)*
        }

        impl Kind {
            /// Every kind, in the catalogue's order.
            pub const ALL: &'static [Kind] = &[$(Kind::$variant),*];

            /// The name checkpoints, the probe and the catalogue know it by.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Kind::$variant => $name,)*
                }
            }

            /// What it is and does.
            pub const fn doc(self) -> &'static str {
                match self {
                    $(Kind::$variant => $doc,)*
                }
            }

            /// It has children.
            pub const fn container(self) -> bool {
                match self {
                    $(Kind::$variant => kinds!(@container $content),)*
                }
            }

            /// The kind named `name`.
            pub fn named(name: &str) -> Option<Kind> {
                match name {
                    $($name => Some(Kind::$variant),)*
                    _ => None,
                }
            }
        }
    };
    (@container children) => {
        true
    };
    (@container leaf) => {
        false
    };
}

kinds! {
    Column children "column"
        "Children top to bottom.";
    Row children "row"
        "Children left to right (mirrored right to left).";
    Stack children "stack"
        "Children overlaid in one rectangle, each placed by its insets.";
    Grid children "grid"
        "Children in tracks and cells (columns, rows, areas).";
    Panel children "panel"
        "A column on the style's panel surface (its card frame).";
    Scroll children "scroll"
        "A column that scrolls its content.";
    List children "list"
        "A virtualised list: its `for` builds only the rows in view, each a selectable row.";
    Cells children "cells"
        "A virtualised grid of equal cells: its `for` builds only the cells in view.";
    Label leaf "label"
        "A text.";
    Button children "button"
        "A button: its text, or its content (an icon, a text, a badge) in a row inside its frame and states; `data-hold` makes it a hold-to-confirm button that fills while held down.";
    Kbd leaf "kbd"
        "A key or an action's glyph: `<kbd>Esc</kbd>` a keycap, `<kbd data-action=\"jump\"></kbd>` the action's glyph on the device used last.";
    Image leaf "image"
        "A texture. CSS filter with an SVG colour matrix multiplies its RGBA channels; use a background for a solid fill.";
    Art leaf "art"
        "An art slot (docs/ui.md §3.3): a spec the art pipeline reads, its texture once there is one.";
    Icon leaf "icon"
        "An icon of the style's set.";
    Progress leaf "progress"
        "A progress bar.";
    Slider leaf "slider"
        "A slider (two-way `value`).";
    Toggle leaf "toggle"
        "A switch with a label (two-way `checked`).";
    Color leaf "color"
        "A colour input: a swatch that opens a picker (two-way `value`, `#rrggbb`).";
    Datetime leaf "datetime"
        "A date or time input (`<input type=\"date|time|datetime-local|month|week\">`): a stepper per part, two-way `dateValue` in HTML's format.";
    Video leaf "video"
        "HTML's `<video>`: the frame of the og.VideoPlayer entity `src` binds, with Svelte's media bindings and optional controls.";
    Audio leaf "audio"
        "HTML's `<audio>`: the og.AudioSource entity `src` binds, played and paused by `paused`; drawn only with `controls`.";
    File leaf "file"
        "A file input (`<input type=\"file\">`), built only where the host opens a file picker (og.ui.filePicker): a button whose press sends `filepick`; the host writes the names chosen to two-way `fileValue`.";
    Radio leaf "radio"
        "One choice of a radio group with its caption: checked while its group (`bind:group`) holds its value, which a press writes.";
    Tabs leaf "tabs"
        "A tab list (two-way `selected`), using the core tabs widget.";
    Field leaf "field"
        "A text control (two-way `input`): `<input>` for one line, `<textarea>` for several; `type=\"number\"` keeps editable text and adds increment/decrement buttons. Min, max and step constrain stepping; data-wrap, data-large-step and data-format configure it.";
    Badge leaf "badge"
        "A small toned label.";
    Spacer leaf "spacer"
        "Empty room (grows by default).";
    Divider leaf "divider"
        "A hairline.";
    Select leaf "select"
        "A select (two-way `value`): its `<option>`s, under `<optgroup>`s, in a list that opens over it.";
    Dialog children "dialog"
        "A dialog (two-way `open`): `modal` over a backdrop that traps the focus, else a panel in place; Escape or B closes it and the focus returns.";
    Details children "details"
        "A disclosure (two-way `open`): its `<summary>` row, its content shown only while open.";
    Box children "box"
        "A generic HTML block (`<div>`, `<section>`, `<nav>`, `<ul>`, `<li>`…): its layout is its CSS `display`, `block` (children top to bottom) by default, `flex` (a row, or `flex-direction`) or `grid`. Text inside it is an anonymous text element; an `<li>` draws its list marker.";
    Meter leaf "meter"
        "A gauge of a value in a known range (HTML `<meter>`), its fill dressed by the region `low`, `high` and `optimum` put the value in.";
    Anchor children "anchor"
        "A box held over a world point (`<anchor entity={u.entity} offset=\"0 2 0\">`): placed every tick from the camera and the entity's pose, its children laid out as a column; `occlusion`, `declutter`, `clamp=\"edge\"`, `priority`, `max-distance`, `scale-at`.";
    Ground children "ground"
        "A box laid flat in the world (`<ground entity={u.entity} size=\"3 3\">…</ground>`): laid out at its CSS size and drawn by the UI onto the horizontal plane `size` world units across at the entity's (or `at`'s) height and draped over the terrain under it as a decal, seen in perspective under the screen's elements (a selection ring, a range marker, a decal); `rotation=\"0 45 0\"` turns it. Inert.";
    Joystick leaf "joystick"
        "A virtual stick (`<joystick action=\"move\">`): a base and a knob that follows the touch that started on it, the input source of og-input's two-axis action `action` (from the base's centre over its radius, y up, past `data-dead-zone`). Its CSS width sizes it (default 128 px).";
    Dpad leaf "dpad"
        "A virtual d-pad (`<dpad action=\"move\">`): four arms read by the angle of the touch that started on it, eight ways, the input source of og-input's two-axis action `action` (each part -1, 0 or 1). Its CSS width sizes it (default 132 px).";
    Viewport leaf "viewport"
        "A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit=\"30deg 75deg 3m\" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one (`camera-controls` orbits, zooms and flings it within `min-camera-orbit` and `max-camera-orbit`), its pointer events carrying the ray into its scene.";
    Svg children "svg"
        "Inline SVG (`<svg viewBox>`): its shapes drawn in its box, the view box fitted by `xMidYMid meet`; its `width` and `height` are CSS px (default: the view box's size).";
    SvgGroup children "svgGroup"
        "SVG's `<g>`: a transform and presentation attributes its shapes inherit.";
    SvgShape leaf "svgShape"
        "An SVG shape (`<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`): one element drawn as an analytic path, its attributes bindable.";
    SvgText leaf "svgText"
        "SVG's `<text x y text-anchor dominant-baseline font-size white-space>`: a line shaped by og-ui's text, its anchor on the point fitted into the `<svg>`'s box and scaled with the view box; `<tspan>` runs inside it take `font-weight`, `font-style`, `text-decoration` and `fill`, a `<tspan x>` (with `y` or `dy`) starts another line, and a `<textPath>` runs it along a path.";
    SvgForeign children "svgForeign"
        "SVG's `<foreignObject x y width height>`: document elements laid out in a box of `width` by `height` user units at `(x, y)`, scaled with the view box as the shapes are.";
    SvgUse children "svgUse"
        "SVG's `<use href=\"#id\" x y width height>`: the symbol, group or shape `#id` drawn again here, inheriting this element's presentation attributes; a `<symbol>`'s view box is fitted into its width and height.";
    SvgDef children "svgDef"
        "An SVG definition (`<defs>`, `<symbol>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`, `<radialGradient>`, `<stop>`, `<title>`, `<desc>`): not drawn where written; `<use>`, `clip-path`, `mask` and paints reference them by `#id`, and a `<title>` names its parent.";
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Kind {
    /// It is SVG's: `<svg>` or an element inside one.
    pub const fn svg(self) -> bool {
        matches!(
            self,
            Kind::Svg
                | Kind::SvgGroup
                | Kind::SvgShape
                | Kind::SvgUse
                | Kind::SvgDef
                | Kind::SvgText
                | Kind::SvgForeign
        )
    }
}
