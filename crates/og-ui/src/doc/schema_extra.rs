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
        name: "emitter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-emitter: <rate> [square|circle|diamond] [<size>px] [<colour>…]`: particles a second from the element while it stands (`.rare { -og-emitter: 12 diamond 6px #ffd166 #fff; }`), drawn over it; `none` stops. Silent under reduced motion.",
    },
    PropSpec {
        name: "meta",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-meta: vignette [<colour>] | tint <colour>`: a full-screen overlay drawn with the element, never taking input, its strength the element's `opacity` (`#Hurt { -og-meta: vignette #a00; opacity: var(--hit); }`).",
    },
    PropSpec {
        name: "mass",
        ty: Ty::Text,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "`<anchor entity={h.entity} mass=\"damage\" value={h.amount}>`: a mass element of the template `mass` instead of a box (thousands per frame, drawn on the GPU). With a CSS `animation`, each row spawns one that plays its `@keyframes` (translate, scale, opacity) over the animation's duration; without one it stays while its row does. `value` is the number shown, or with `max` a bar filled to value/max; `color` tints it.",
    },
    PropSpec {
        name: "massValue",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "A mass element's value: the number it shows, or its bar's fill with `max`.",
    },
    PropSpec {
        name: "massMax",
        ty: Ty::Num,
        on: On::Kinds(&["anchor"]),
        two_way: false,
        doc: "A mass element's bar: drawn as a bar filled to value/max (an overhead health bar).",
    },
    PropSpec {
        name: "world",
        ty: Ty::Bool,
        on: On::Kinds(&["panel"]),
        two_way: false,
        doc: "`<panel world src={$target} entity={$e} size=\"1.2 0.8\" rotation=\"0 30 0\">`: a world panel, its subtree drawn into the render target `src` at its CSS width and height, shown on a quad `size` world units across at the entity (or `at=\"x y z\"`) turned by `rotation` (degrees about x, y, z); the pointer's ray reaches its controls.",
    },
    PropSpec {
        name: "worldSize",
        ty: Ty::Text,
        on: On::Kinds(&["panel", "ground"]),
        two_way: false,
        doc: "A world panel's quad, `width height` world units (default 1 1).",
    },
    PropSpec {
        name: "worldRotation",
        ty: Ty::Text,
        on: On::Kinds(&["panel", "ground"]),
        two_way: false,
        doc: "A world panel's turn, degrees about x, y and z (`0 30 0`); facing +z unturned.",
    },
    PropSpec {
        name: "entity",
        ty: Ty::Num,
        on: On::Kinds(&["anchor", "panel", "ground"]),
        two_way: false,
        doc: "The entity it follows, by its bits (`entity={u.entity}`): its world pose as the host hands it in.",
    },
    PropSpec {
        name: "worldAt",
        ty: Ty::Text,
        on: On::Kinds(&["anchor", "panel", "ground"]),
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
        on: On::Kinds(&["viewport", "panel"]),
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
    PropSpec {
        name: "backgroundImage",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background` or `background-image` layers over the background colour, the first on top (CSS Backgrounds 3): images `url(<name>)` (an icon of the UI's atlas, as `<image href>` names one) or `url(<handle>)` (a texture, `og.load.texture`), each `<image> [<position> [/ <size>]] [<repeat>]` (`background: url(coin) right 8px bottom 8px / 24px no-repeat, url(sky) center / cover, #123`), and gradients: `linear-gradient()`, `radial-gradient()`, `conic-gradient()` and their `repeating-` forms, which fill the box (up to 8 stops each, mixed in OKLab). The element's `border-radius` and `clip-path` cut every layer.",
    },
    PropSpec {
        name: "backgroundSize",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-size` of each image layer, a comma list repeated over the layers: `cover`, `contain`, or a width and height (`<n>px`, `<n>%` of the box, `auto` from the image's ratio).",
    },
    PropSpec {
        name: "backgroundPosition",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-position` of each image layer, a comma list: one to four of `left`, `center`, `right`, `top`, `bottom`, `<n>px` and `<n>%` (`right 10px bottom 20%` from those edges).",
    },
    PropSpec {
        name: "backgroundRepeat",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-repeat` of each image layer, a comma list: `repeat-x`, `repeat-y`, or one or two of `repeat`, `no-repeat`, `space` (whole images edge to edge) and `round` (rescaled to a whole number).",
    },
    PropSpec {
        name: "maskImage",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-image`: a gradient whose alpha fades the element and its subtree (`mask-image: linear-gradient(#000 70%, transparent)`).",
    },
    PropSpec {
        name: "backdropFilter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `backdrop-filter`: what is under the element blurred and its colour adjusted (glass), with `filter`'s functions; the colour functions fold into one matrix and `drop-shadow()` casts nothing on the opaque backdrop.",
    },
    PropSpec {
        name: "filter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `filter`: `blur()`, `brightness()`, `contrast()`, `drop-shadow()` (of the alpha, not the box), `grayscale()`, `hue-rotate()`, `invert()`, `opacity()`, `saturate()` and `sepia()` on the element and its subtree, applied in the order written (`filter: drop-shadow(0 4px 6px #0008) saturate(1.4)`).",
    },
    PropSpec {
        name: "mixBlendMode",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mix-blend-mode`: normal, multiply, screen, plus-lighter, overlay, soft-light or color-dodge.",
    },
    PropSpec {
        name: "textShadow",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `text-shadow: <x> <y> [<blur>] [<colour>], …` on the element's text.",
    },
    PropSpec {
        name: "clipPath",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `clip-path`: `inset()`, `circle()`, `ellipse()`, `polygon()` or `path()` cuts the element, its background and its children to the shape (by its path's coverage, either fill rule).",
    },
    PropSpec {
        name: "material",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-material: <name>(<n> …[, <colour>])`: the element and its subtree composited through one of the engine's built-in materials (dissolve, shine, hologram, glitch, ripple, pixelate, color-adjust, outline, blur), its parameters in the material's order. No CSS standard has shader materials.",
    },
    PropSpec {
        name: "scrollSnapType",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-snap-type: x | y | both [mandatory | proximity]` on a scroll container: a drag, fling or wheel settles on the snap position of the area nearest where it would stop (`proximity`: only within a third of the view).",
    },
    PropSpec {
        name: "scrollSnapAlign",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-snap-align: start | center | end` (block, then inline) of an area inside a snapping scroll container: the edge or centre that lines up with its view's.",
    },
    PropSpec {
        name: "sheetDetents",
        ty: Ty::Text,
        on: On::Kinds(&["dialog"]),
        two_way: false,
        doc: "`-og-sheet-detents: 40% 90%` on a modal `<dialog>`: a bottom sheet docked to the bottom at full width, dragged onto its detents (heights, `%` of the viewport's) and swiped down to close (writing `open` back). It opens at the first. No CSS standard has detents.",
    },
];
