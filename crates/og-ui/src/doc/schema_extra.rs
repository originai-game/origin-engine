//! The element kinds and props the document runtime adds to the core
//! schema ([`super::schema`]): its `KINDS` and `PROPS` are the core's
//! followed by these, so the parser, the probe, the catalogue and the
//! README table read them as any other.
use super::schema::{KindSpec, On, PropSpec, Ty};

pub const KINDS: &[KindSpec] = &[
    KindSpec {
        name: "anchor",
        container: true,
        doc: "A box held over a world point (`<anchor entity={u.entity} offset=\"0 2 0\">`): placed every tick from the camera and the entity's pose, its children laid out as a column; `occlusion`, `declutter`, `clamp=\"edge\"`, `priority`, `max-distance`, `scale-at`.",
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
        doc: "A camera's view of a scene layer in the layout (`<viewport src={$target} camera-orbit=\"30deg 75deg 3m\" bind:clientWidth={$w}>`): the render target `src` names, its camera written as `<model-viewer>` writes one, its pointer events carrying the ray into its scene.",
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
        doc: "SVG's `<text x y text-anchor dominant-baseline font-size>`: one line shaped by og-ui's text, its anchor on the point fitted into the `<svg>`'s box and scaled with the view box; `<tspan>` runs inside it take `font-weight`, `font-style`, `text-decoration` and `fill`.",
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
pub(super) fn tags(kind: &str) -> Option<Vec<(String, String)>> {
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
        "svg" => one(
            "svg",
            "<svg id=\"Name\" viewBox=\"0 0 40 40\">…</svg>",
        ),
        "svgGroup" => one("g", "<g transform=\"rotate(-90 20 20)\" stroke=\"var(--accent)\">…</g>"),
        "svgUse" => one("use", "<use href=\"#Coin\" x={i * 12} y=\"0\" width=\"10\" height=\"10\" />"),
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

/// SVG attributes and their props: `(attribute, prop)`.
const SVG_ATTRS: &[(&str, &str)] = &[
    ("viewBox", "svgViewBox"),
    ("transform", "svgTransform"),
    ("fill", "svgFill"),
    ("fill-opacity", "svgFillOpacity"),
    ("fill-rule", "svgFillRule"),
    ("stroke", "svgStroke"),
    ("stroke-opacity", "svgStrokeOpacity"),
    ("stroke-width", "svgStrokeWidth"),
    ("stroke-dasharray", "svgStrokeDasharray"),
    ("stroke-dashoffset", "svgStrokeDashoffset"),
    ("stroke-linecap", "svgStrokeLinecap"),
    ("stroke-linejoin", "svgStrokeLinejoin"),
    ("stroke-miterlimit", "svgStrokeMiterlimit"),
    ("pathLength", "svgPathLength"),
    ("d", "svgD"),
    ("x", "svgX"),
    ("y", "svgY"),
    ("dx", "svgDx"),
    ("dy", "svgDy"),
    ("font-size", "svgFontSize"),
    ("font-weight", "svgFontWeight"),
    ("font-style", "svgFontStyle"),
    ("text-anchor", "svgTextAnchor"),
    ("dominant-baseline", "svgDominantBaseline"),
    ("width", "svgWidth"),
    ("height", "svgHeight"),
    ("rx", "svgRx"),
    ("ry", "svgRy"),
    ("cx", "svgCx"),
    ("cy", "svgCy"),
    ("r", "svgR"),
    ("x1", "svgX1"),
    ("y1", "svgY1"),
    ("x2", "svgX2"),
    ("y2", "svgY2"),
    ("points", "svgPoints"),
    ("fx", "svgFx"),
    ("fy", "svgFy"),
    ("offset", "svgOffset"),
    ("stop-color", "svgStopColor"),
    ("stop-opacity", "svgStopOpacity"),
    ("gradientUnits", "svgGradientUnits"),
    ("gradientTransform", "svgGradientTransform"),
    ("spreadMethod", "svgSpreadMethod"),
    ("href", "svgHref"),
    ("clip-path", "svgClipPath"),
    ("clip-rule", "svgClipRule"),
    ("mask", "svgMask"),
    ("clipPathUnits", "svgClipPathUnits"),
    ("maskContentUnits", "svgMaskContentUnits"),
    ("patternUnits", "svgPatternUnits"),
    ("patternContentUnits", "svgPatternContentUnits"),
    ("patternTransform", "svgPatternTransform"),
    ("preserveAspectRatio", "svgPreserveAspectRatio"),
];

/// The prop SVG's attribute `attr` sets.
pub(super) fn svg_prop(attr: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(a, _)| *a == attr).map(|(_, p)| *p)
}

/// The SVG attribute a prop is written as (the catalogue's column).
pub(super) fn svg_attr(prop: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(_, p)| *p == prop).map(|(a, _)| *a)
}

const ALL: On = On::Kinds(&["svg", "svgGroup", "svgShape", "svgUse", "svgDef", "svgText"]);
const TEXT: On = On::Kinds(&["svg", "svgGroup", "svgText"]);
const SHAPES: On = On::Kinds(&["svgShape", "svgUse"]);
const PAINTED: On = On::Kinds(&["svgGroup", "svgShape", "svgUse"]);
const DEFS: On = On::Kinds(&["svgDef"]);

macro_rules! svg {
    ($name:literal, $on:expr, $doc:literal) => {
        PropSpec {
            name: $name,
            ty: Ty::Text,
            on: $on,
            two_way: false,
            doc: $doc,
        }
    };
}

pub const PROPS: &[PropSpec] = &[
    PropSpec {
        name: "entity",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "The entity it follows, by its bits (`entity={u.entity}`): its world pose as the host hands it in.",
    },
    PropSpec {
        name: "worldAt",
        ty: Ty::Text,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`at=\"x y z\"`: a world point it follows, without an entity.",
    },
    PropSpec {
        name: "worldOffset",
        ty: Ty::Text,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`offset=\"0 2 0\"`: added to the point, world units (over a head).",
    },
    PropSpec {
        name: "occlusion",
        ty: Ty::Word(&["none", "fade", "hide"]),
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "What the scene in front of its point does to it: faded or hidden.",
    },
    PropSpec {
        name: "declutter",
        ty: Ty::Word(&["none", "shift", "stack", "hide"]),
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "How it resolves overlapping an anchor of higher priority.",
    },
    PropSpec {
        name: "worldClamp",
        ty: Ty::Word(&["none", "edge"]),
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`clamp=\"edge\"`: held on the screen's edge with an arrow while its point is off screen (a waypoint).",
    },
    PropSpec {
        name: "priority",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "Higher keeps its place when anchors overlap.",
    },
    PropSpec {
        name: "maxDistance",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`max-distance`: hidden past this many world units, fading over its last tenth.",
    },
    PropSpec {
        name: "worldScaleAt",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`scale-at=\"10\"`: scaled 1 at this distance, larger nearer and smaller farther (0.5 to 2).",
    },
    PropSpec {
        name: "action",
        ty: Ty::Name,
        on: On::Kinds(&["joystick", "dpad"]),
        two_way: false,
        doc: "The og-input action it is an input source of (`og.input.defineAction`; a two-axis one), read in the tick from the touches that started on it.",
    },
    PropSpec {
        name: "dataDeadZone",
        ty: Ty::Num,
        on: On::Kinds(&["joystick"]),
        two_way: false,
        doc: "A stick's dead zone, a ratio of its radius (`data-dead-zone=\"0.2\"`; default 0.15): past it the value rises from 0.",
    },
    PropSpec {
        name: "dataOgTouch",
        ty: Ty::Name,
        on: On::Kinds(&["button"]),
        two_way: false,
        doc: "A touch button (`<button data-og-touch=\"jump\">`): the input source of og-input's action `jump`, held while a touch that went down on it is held, whatever else is held (a thumb on the stick).",
    },
    PropSpec {
        name: "viewportSrc",
        ty: Ty::Num,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "The render target its camera draws into (`src={$target}`, the target's colour texture handle).",
    },
    PropSpec {
        name: "cameraOrbit",
        ty: Ty::Text,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s camera-orbit: yaw, pitch from straight up, and distance from its target (`30deg 75deg 3m`; default `0deg 75deg 3m`), for its pointer events' rays.",
    },
    PropSpec {
        name: "cameraTarget",
        ty: Ty::Text,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s camera-target: the point it orbits (`0m 1m 0m`, the default).",
    },
    PropSpec {
        name: "fieldOfView",
        ty: Ty::Text,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s field-of-view, vertical (`50deg`, the default).",
    },
    PropSpec {
        name: "clientWidth",
        ty: Ty::Num,
        on: On::Kinds(&["viewport"]),
        two_way: true,
        doc: "Svelte's `bind:clientWidth`: its laid-out width, CSS px, written when it changes (size its render target by it times the pixel ratio).",
    },
    PropSpec {
        name: "clientHeight",
        ty: Ty::Num,
        on: On::Kinds(&["viewport"]),
        two_way: true,
        doc: "Svelte's `bind:clientHeight`: its laid-out height, CSS px.",
    },
    PropSpec {
        name: "draggable",
        ty: Ty::Bool,
        on: On::All,
        two_way: false,
        doc: "HTML `draggable`: a drag that starts on it moves it with the pointer and returns it home on release, firing `ondragstart`, `ondragend`, and `ondragover`/`ondrop` on the element it is over (any element with one of those handlers takes drops).",
    },
    PropSpec {
        name: "dataDragType",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "What a draggable element is to drop targets (`data-drag-type=\"weapon\"`), as a drag's data type is in HTML.",
    },
    PropSpec {
        name: "dataAccept",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "The drag types a drop target takes, space-separated (`data-accept=\"weapon shield\"`; default any).",
    },
    svg!(
        "svgViewBox",
        On::Kinds(&["svg", "svgDef"]),
        "SVG `viewBox`: the user space `min-x min-y width height` its box shows."
    ),
    svg!(
        "svgTransform",
        On::Kinds(&["svgGroup", "svgShape", "svgDef"]),
        "SVG `transform` (matrix, translate, scale, rotate, skewX, skewY); on a gradient, `gradientTransform`."
    ),
    svg!(
        "svgFill",
        ALL,
        "SVG `fill`: a colour, `var(--role)`, `currentColor` (the style's ink), `url(#gradient)` or `none`."
    ),
    svg!("svgFillOpacity", ALL, "SVG `fill-opacity`."),
    svg!("svgFillRule", ALL, "SVG `fill-rule`: nonzero or evenodd."),
    svg!(
        "svgStroke",
        ALL,
        "SVG `stroke`: a colour, `var(--role)`, `currentColor`, `url(#gradient)` or `none`."
    ),
    svg!("svgStrokeOpacity", ALL, "SVG `stroke-opacity`."),
    svg!(
        "svgStrokeWidth",
        ALL,
        "SVG `stroke-width`, user units (scaled with the view box)."
    ),
    svg!(
        "svgStrokeDasharray",
        ALL,
        "SVG `stroke-dasharray`: a dash and a gap (user units, or of `pathLength`)."
    ),
    svg!("svgStrokeDashoffset", ALL, "SVG `stroke-dashoffset`."),
    svg!("svgStrokeLinecap", ALL, "SVG `stroke-linecap`."),
    svg!("svgStrokeLinejoin", ALL, "SVG `stroke-linejoin`."),
    svg!("svgStrokeMiterlimit", ALL, "SVG `stroke-miterlimit`."),
    svg!(
        "svgPathLength",
        SHAPES,
        "SVG `pathLength`: the length its dashes and offset measure the outline in (`pathLength=\"1\"` makes `stroke-dashoffset` a fraction)."
    ),
    svg!("svgD", SHAPES, "A `<path>`'s path data."),
    svg!("svgX", ALL, "SVG `x`."),
    svg!("svgY", ALL, "SVG `y`."),
    svg!(
        "svgDx",
        On::Kinds(&["svgText"]),
        "A `<text>`'s `dx`: added to `x`."
    ),
    svg!(
        "svgDy",
        On::Kinds(&["svgText"]),
        "A `<text>`'s `dy`: added to `y`."
    ),
    svg!(
        "svgFontSize",
        TEXT,
        "SVG `font-size`, user units (scaled with the view box; default 16)."
    ),
    svg!(
        "svgFontWeight",
        TEXT,
        "SVG `font-weight`: normal, bold or 100–900 (600 and up is bold)."
    ),
    svg!("svgFontStyle", TEXT, "SVG `font-style`: normal or italic."),
    svg!(
        "svgTextAnchor",
        TEXT,
        "SVG `text-anchor`: start, middle or end of the line on `x`."
    ),
    svg!(
        "svgDominantBaseline",
        TEXT,
        "SVG `dominant-baseline`: auto (alphabetic), middle, central, hanging, text-before-edge, text-after-edge, ideographic or mathematical on `y`."
    ),
    svg!(
        "svgWidth",
        ALL,
        "SVG `width` (a `<rect>`, `<use>`, `<pattern>`, `<mask>`)."
    ),
    svg!("svgHeight", ALL, "SVG `height`."),
    svg!("svgRx", SHAPES, "SVG `rx`."),
    svg!("svgRy", SHAPES, "SVG `ry`."),
    svg!("svgCx", ALL, "SVG `cx`."),
    svg!("svgCy", ALL, "SVG `cy`."),
    svg!("svgR", ALL, "SVG `r`."),
    svg!("svgX1", ALL, "SVG `x1`."),
    svg!("svgY1", ALL, "SVG `y1`."),
    svg!("svgX2", ALL, "SVG `x2`."),
    svg!("svgY2", ALL, "SVG `y2`."),
    svg!(
        "svgPoints",
        SHAPES,
        "A `<polyline>`'s or `<polygon>`'s points."
    ),
    svg!("svgFx", DEFS, "A radial gradient's `fx`."),
    svg!("svgFy", DEFS, "A radial gradient's `fy`."),
    svg!("svgOffset", DEFS, "A `<stop>`'s `offset`."),
    svg!("svgStopColor", DEFS, "A `<stop>`'s `stop-color`."),
    svg!("svgStopOpacity", DEFS, "A `<stop>`'s `stop-opacity`."),
    svg!("svgGradientUnits", DEFS, "A gradient's `gradientUnits`."),
    svg!(
        "svgGradientTransform",
        DEFS,
        "A gradient's `gradientTransform`."
    ),
    svg!("svgSpreadMethod", DEFS, "A gradient's `spreadMethod`."),
    svg!(
        "svgHref",
        On::Kinds(&["svgDef", "svgUse", "svgShape"]),
        "SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), or the gradient a gradient takes its stops from."
    ),
    svg!(
        "svgClipPath",
        PAINTED,
        "SVG `clip-path=\"url(#id)\"`: the `<clipPath>` whose shapes cut it."
    ),
    svg!(
        "svgMask",
        PAINTED,
        "SVG `mask=\"url(#id)\"`: the `<mask>` whose luminance cuts it (white shapes, or one gradient-filled shape)."
    ),
    svg!(
        "svgClipRule",
        SHAPES,
        "SVG `clip-rule` of a shape in a `<clipPath>`: nonzero or evenodd (a ring cuts a hole)."
    ),
    svg!("svgClipPathUnits", DEFS, "A clip path's `clipPathUnits`."),
    svg!("svgMaskContentUnits", DEFS, "A mask's `maskContentUnits`."),
    svg!("svgPatternUnits", DEFS, "A pattern's `patternUnits`."),
    svg!(
        "svgPatternContentUnits",
        DEFS,
        "A pattern's `patternContentUnits`."
    ),
    svg!(
        "svgPatternTransform",
        DEFS,
        "A pattern's `patternTransform`."
    ),
    svg!(
        "svgPreserveAspectRatio",
        On::Kinds(&["svg", "svgDef", "svgShape"]),
        "SVG `preserveAspectRatio` of a `<symbol>`, a `<pattern>` or an `<image>`."
    ),
];
