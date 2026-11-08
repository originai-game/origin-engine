//! The element kinds and props the document runtime adds to the core
//! schema ([`super::schema`]): its `KINDS` and `PROPS` are the core's
//! followed by these, so the parser, the probe, the catalogue and the
//! README table read them as any other.
use super::schema::{KindSpec, On, PropSpec, Ty};

pub const KINDS: &[KindSpec] = &[
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
        name: "svgDef",
        container: true,
        doc: "An SVG definition (`<defs>`, `<linearGradient>`, `<radialGradient>`, `<stop>`, `<title>`): not drawn; shapes reference gradients with `url(#id)`.",
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
        "svg" => one(
            "svg",
            "<svg id=\"Name\" viewBox=\"0 0 40 40\">…</svg>",
        ),
        "svgGroup" => one("g", "<g transform=\"rotate(-90 20 20)\" stroke=\"var(--accent)\">…</g>"),
        "svgShape" => Some(vec![
            ("path".into(), "<path d={$line} fill=\"none\" stroke=\"currentColor\" />".into()),
            ("circle".into(), "<circle cx=\"20\" cy=\"20\" r=\"18\" pathLength=\"1\" stroke-dasharray=\"1 1\" stroke-dashoffset={1 - $ready} />".into()),
            ("rect".into(), "<rect x={b.x} y={40 - b.h} width=\"8\" height={b.h} rx=\"2\" />".into()),
            ("polygon".into(), "<polygon points=\"0,0 10,0 5,8\" /> (ellipse, line, polyline)".into()),
        ]),
        "svgDef" => Some(vec![(
            "linearGradient".into(),
            "<linearGradient id=\"Up\" x2=\"0\" y2=\"1\"><stop offset=\"0\" stop-color=\"#fff\" />…</linearGradient> (radialGradient, defs)".into(),
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
];

/// The prop SVG's attribute `attr` sets.
pub(super) fn svg_prop(attr: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(a, _)| *a == attr).map(|(_, p)| *p)
}

/// The SVG attribute a prop is written as (the catalogue's column).
pub(super) fn svg_attr(prop: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(_, p)| *p == prop).map(|(a, _)| *a)
}

const ALL: On = On::Kinds(&["svg", "svgGroup", "svgShape", "svgDef"]);
const SHAPES: On = On::Kinds(&["svgShape"]);
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
        On::Kinds(&["svg"]),
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
    svg!("svgWidth", SHAPES, "A `<rect>`'s `width`."),
    svg!("svgHeight", SHAPES, "A `<rect>`'s `height`."),
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
        DEFS,
        "A gradient's `href`: the gradient it takes its stops from."
    ),
];
