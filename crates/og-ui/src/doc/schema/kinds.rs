//! Every element kind a document can name: the HTML-shaped ones, then the
//! world's, the touch controls' and SVG's.
use super::KindSpec;

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

pub const KINDS: &[KindSpec] = &[
    KindSpec {
        name: "anchor",
        container: true,
        doc: "A box held over a world point (`<anchor entity={u.entity} offset=\"0 2 0\">`): placed every tick from the camera and the entity's pose, its children laid out as a column; `occlusion`, `declutter`, `clamp=\"edge\"`, `priority`, `max-distance`, `scale-at`.",
    },
    KindSpec {
        name: "ground",
        container: true,
        doc: "A box laid flat in the world (`<ground entity={u.entity} size=\"3 3\">…</ground>`): laid out at its CSS size and drawn by the UI onto the horizontal plane `size` world units across at the entity's (or `at`'s) height and draped over the terrain under it as a decal, seen in perspective under the screen's elements (a selection ring, a range marker, a decal); `rotation=\"0 45 0\"` turns it. Inert.",
    },
    KindSpec {
        name: "joystick",
        container: false,
        doc: "A virtual stick (`<joystick action=\"move\">`): a base and a knob that follows the touch that started on it, the input source of og-input's two-axis action `action` (from the base's centre over its radius, y up, past `data-dead-zone`). Its CSS width sizes it (default 128 px).",
    },
    KindSpec {
        name: "dpad",
        container: false,
        doc: "A virtual d-pad (`<dpad action=\"move\">`): four arms read by the angle of the touch that started on it, eight ways, the input source of og-input's two-axis action `action` (each part -1, 0 or 1). Its CSS width sizes it (default 132 px).",
    },
    KindSpec {
        name: "viewport",
        container: false,
        doc: "A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit=\"30deg 75deg 3m\" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one (`camera-controls` orbits, zooms and flings it within `min-camera-orbit` and `max-camera-orbit`), its pointer events carrying the ray into its scene.",
    },
    KindSpec {
        name: "svg",
        container: true,
        doc: "Inline SVG (`<svg viewBox>`): its shapes drawn in its box, the view box fitted by `xMidYMid meet`; its `width` and `height` are CSS px (default: the view box's size).",
    },
    KindSpec {
        name: "svgGroup",
        container: true,
        doc: "SVG's `<g>`: a transform and presentation attributes its shapes inherit.",
    },
    KindSpec {
        name: "svgShape",
        container: false,
        doc: "An SVG shape (`<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`, `<polyline>`, `<polygon>`): one element drawn as an analytic path, its attributes bindable.",
    },
    KindSpec {
        name: "svgText",
        container: false,
        doc: "SVG's `<text x y text-anchor dominant-baseline font-size white-space>`: a line shaped by og-ui's text, its anchor on the point fitted into the `<svg>`'s box and scaled with the view box; `<tspan>` runs inside it take `font-weight`, `font-style`, `text-decoration` and `fill`, a `<tspan x>` (with `y` or `dy`) starts another line, and a `<textPath>` runs it along a path.",
    },
    KindSpec {
        name: "svgForeign",
        container: true,
        doc: "SVG's `<foreignObject x y width height>`: document elements laid out in a box of `width` by `height` user units at `(x, y)`, scaled with the view box as the shapes are.",
    },
    KindSpec {
        name: "svgUse",
        container: true,
        doc: "SVG's `<use href=\"#id\" x y width height>`: the symbol, group or shape `#id` drawn again here, inheriting this element's presentation attributes; a `<symbol>`'s view box is fitted into its width and height.",
    },
    KindSpec {
        name: "svgDef",
        container: true,
        doc: "An SVG definition (`<defs>`, `<symbol>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`, `<radialGradient>`, `<stop>`, `<title>`, `<desc>`): not drawn where written; `<use>`, `clip-path`, `mask` and paints reference them by `#id`, and a `<title>` names its parent.",
    },
];

/// The tags a document writes an extern kind as, each with an example
/// (the catalogue's headings).
pub(in crate::doc) fn tags(kind: &str) -> Option<Vec<(String, String)>> {
    let one = |tag: &str, example: &str| Some(vec![(tag.to_string(), example.to_string())]);
    match kind {
        "viewport" => one(
            "viewport",
            "<viewport id=\"Preview\" src={$target} camera-orbit=\"30deg 75deg 3m\" bind:clientWidth={$w} onpointermove={orbit} />",
        ),
        "joystick" => one(
            "joystick",
            "<joystick id=\"Stick\" action=\"move\" data-dead-zone=\"0.2\" customize />",
        ),
        "anchor" => one(
            "anchor",
            "<anchor entity={u.entity} offset=\"0 2.1 0\" occlusion=\"fade\" declutter=\"stack\" clamp=\"edge\">…</anchor>",
        ),
        "dpad" => one("dpad", "<dpad id=\"Pad\" action=\"move\" />"),
        "ground" => one(
            "ground",
            "<ground entity={u.entity} size=\"3 3\"><svg viewBox=\"0 0 40 40\">…</svg></ground>",
        ),
        "svg" => one(
            "svg",
            "<svg id=\"Name\" viewBox=\"0 0 40 40\">…</svg>",
        ),
        "svgGroup" => one("g", "<g transform=\"rotate(-90 20 20)\" stroke=\"var(--accent)\">…</g>"),
        "svgUse" => one("use", "<use href=\"#Coin\" x={i * 12} y=\"0\" width=\"10\" height=\"10\" />"),
        "svgForeign" => one(
            "foreignObject",
            "<foreignObject x=\"10\" y=\"10\" width=\"80\" height=\"30\"><button onclick={buy}>Buy</button></foreignObject>",
        ),
        "svgShape" => Some(vec![
            ("path".into(), "<path d={$line} fill=\"none\" stroke=\"currentColor\" />".into()),
            ("circle".into(), "<circle cx=\"20\" cy=\"20\" r=\"18\" pathLength=\"1\" stroke-dasharray=\"1 1\" stroke-dashoffset={1 - $ready} />".into()),
            ("rect".into(), "<rect x={b.x} y={40 - b.h} width=\"8\" height={b.h} rx=\"2\" />".into()),
            ("polygon".into(), "<polygon points=\"0,0 10,0 5,8\" /> (ellipse, line, polyline)".into()),
        ]),
        "svgDef" => Some(vec![(
            "linearGradient".into(),
            "<linearGradient id=\"Up\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#fff\" />…</linearGradient> (radialGradient, defs)".into(),
        ), (
            "symbol".into(),
            "<symbol id=\"Coin\" viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"10\" /></symbol> (clipPath, mask, pattern, title)".into(),
        )]),
        _ => None,
    }
}
