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
    ("marker-start", "svgMarkerStart"),
    ("marker-mid", "svgMarkerMid"),
    ("marker-end", "svgMarkerEnd"),
    ("refX", "svgRefX"),
    ("refY", "svgRefY"),
    ("markerWidth", "svgMarkerWidth"),
    ("markerHeight", "svgMarkerHeight"),
    ("markerUnits", "svgMarkerUnits"),
    ("orient", "svgOrient"),
    ("startOffset", "svgStartOffset"),
    ("side", "svgSide"),
    ("method", "svgMethod"),
    ("spacing", "svgSpacing"),
    ("path", "svgPath"),
    ("filter", "svgFilter"),
    ("filterUnits", "svgFilterUnits"),
    ("primitiveUnits", "svgPrimitiveUnits"),
    (
        "color-interpolation-filters",
        "svgColorInterpolationFilters",
    ),
    ("in", "svgIn"),
    ("in2", "svgIn2"),
    ("result", "svgResult"),
    ("stdDeviation", "svgStdDeviation"),
    ("flood-color", "svgFloodColor"),
    ("flood-opacity", "svgFloodOpacity"),
    ("operator", "svgOperator"),
    ("k1", "svgK1"),
    ("k2", "svgK2"),
    ("k3", "svgK3"),
    ("k4", "svgK4"),
    ("mode", "svgMode"),
    ("type", "svgType"),
    ("values", "svgValues"),
    ("tableValues", "svgTableValues"),
    ("slope", "svgSlope"),
    ("intercept", "svgIntercept"),
    ("amplitude", "svgAmplitude"),
    ("exponent", "svgExponent"),
    ("radius", "svgRadius"),
    ("scale", "svgScale"),
    ("xChannelSelector", "svgXChannelSelector"),
    ("yChannelSelector", "svgYChannelSelector"),
    ("baseFrequency", "svgBaseFrequency"),
    ("numOctaves", "svgNumOctaves"),
    ("seed", "svgSeed"),
    ("stitchTiles", "svgStitchTiles"),
    ("order", "svgOrder"),
    ("kernelMatrix", "svgKernelMatrix"),
    ("divisor", "svgDivisor"),
    ("bias", "svgBias"),
    ("targetX", "svgTargetX"),
    ("targetY", "svgTargetY"),
    ("edgeMode", "svgEdgeMode"),
    ("preserveAlpha", "svgPreserveAlpha"),
    ("kernelUnitLength", "svgKernelUnitLength"),
    ("surfaceScale", "svgSurfaceScale"),
    ("diffuseConstant", "svgDiffuseConstant"),
    ("specularConstant", "svgSpecularConstant"),
    ("specularExponent", "svgSpecularExponent"),
    ("lighting-color", "svgLightingColor"),
    ("azimuth", "svgAzimuth"),
    ("elevation", "svgElevation"),
    ("z", "svgZ"),
    ("pointsAtX", "svgPointsAtX"),
    ("pointsAtY", "svgPointsAtY"),
    ("pointsAtZ", "svgPointsAtZ"),
    ("limitingConeAngle", "svgLimitingConeAngle"),
];

/// The prop SVG's attribute `attr` sets.
pub(super) fn svg_prop(attr: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(a, _)| *a == attr).map(|(_, p)| *p)
}

/// The SVG attribute a prop is written as (the catalogue's column).
pub(super) fn svg_attr(prop: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(_, p)| *p == prop).map(|(a, _)| *a)
}

const ALL: On = On::Kinds(&[
    "svg",
    "svgGroup",
    "svgShape",
    "svgUse",
    "svgDef",
    "svgText",
    "svgForeign",
]);
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
        on: On::Kinds(&["box"]),
        two_way: false,
        doc: "`<og-emitter effect=\"confetti 70 burst 110 for 1.4s\">`: particles from the element while it stands, drawn over it; `none` stops. Grammar: `[<preset>] [<rate>] [burst <n>] [for <time>] [life <time>] [<speed>px/s [<speed>px/s]] [up|down|<angle>deg] [square|circle|diamond] [<size>px] [<colour>…]`. Presets: sparkles, confetti, embers, snow, bubbles, dust, sparks, fireworks. Later words override the preset's look, rate and palette colours. `burst` throws that many once when the element first stands, `for` stops its rate after that time, `life` is each particle's life and speed (one or a range) is travel per second. Silent under reduced motion; state and seeded replay are checkpointed.",
    },
    PropSpec {
        name: "meta",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-meta: vignette [<colour>] | tint <colour> | material <material>`: a full-screen overlay drawn with the element, never taking input, its strength the element's `opacity` (`#Hurt { -og-meta: vignette #a00; opacity: var(--hit); }`); `material` composites the frame under it through a material written as `-og-material` writes one (`-og-meta: material ripple(6 48 60)` for sanity, `-og-meta: material \"fx/static.ogt\" var(--k)`). A pulse is a CSS animation of its opacity.",
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
        two_way: true,
        doc: "`<model-viewer>`'s camera-orbit: yaw, pitch from straight up, and distance from its target (`30deg 75deg 3m`; default `0deg 75deg 3m`), for its pointer events' rays. With `camera-controls`, `bind:cameraOrbit={$orbit}` reads the orbit back as the player turns it (draw the scene from it).",
    },
    PropSpec {
        name: "cameraControls",
        ty: Ty::Bool,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s camera-controls: a drag orbits the camera (a fling keeps it turning, slowing to rest; still under reduced motion), the wheel and a two-finger pinch zoom it, and with the focus (Tab, a pad) Left and Right turn it 15° and Up and Down tilt it 10°, within `min-camera-orbit` and `max-camera-orbit`. The orbit is UI state; a new authored `camera-orbit` takes over.",
    },
    PropSpec {
        name: "minCameraOrbit",
        ty: Ty::Text,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s min-camera-orbit: the least yaw, pitch and distance camera-controls reach, each a value or `auto` (`auto 40deg 1.5m`; default `auto 22.5deg auto`).",
    },
    PropSpec {
        name: "maxCameraOrbit",
        ty: Ty::Text,
        on: On::Kinds(&["viewport"]),
        two_way: false,
        doc: "`<model-viewer>`'s max-camera-orbit: the most yaw, pitch and distance (`auto 100deg 6m`; default `auto 157.5deg auto`).",
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
        "SVG `stroke-dasharray`: dash and gap lengths, an odd list repeated (user units, or of `pathLength`)."
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
        On::Kinds(&["svgText", "svgDef"]),
        "A `<text>`'s `dx` (added to `x`, user units or `em`), or `feOffset`'s and `feDropShadow`'s."
    ),
    svg!(
        "svgDy",
        On::Kinds(&["svgText", "svgDef"]),
        "A `<text>`'s `dy` (added to `y`, user units or `em`; `1.2em`: a line down), or `feOffset`'s and `feDropShadow`'s."
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
    svg!(
        "svgMarkerStart",
        ALL,
        "SVG `marker-start=\"url(#id)\"`: the `<marker>` drawn at a shape's first vertex."
    ),
    svg!(
        "svgMarkerMid",
        ALL,
        "SVG `marker-mid`: the `<marker>` drawn at every vertex between the first and the last."
    ),
    svg!(
        "svgMarkerEnd",
        ALL,
        "SVG `marker-end`: the `<marker>` drawn at a shape's last vertex (an arrowhead)."
    ),
    svg!(
        "svgRefX",
        DEFS,
        "A `<marker>`'s `refX`: the x of its content placed on the vertex."
    ),
    svg!("svgRefY", DEFS, "A `<marker>`'s `refY`."),
    svg!(
        "svgMarkerWidth",
        DEFS,
        "A `<marker>`'s `markerWidth` (default 3)."
    ),
    svg!(
        "svgMarkerHeight",
        DEFS,
        "A `<marker>`'s `markerHeight` (default 3)."
    ),
    svg!(
        "svgMarkerUnits",
        DEFS,
        "A `<marker>`'s `markerUnits`: strokeWidth (scaled by the shape's stroke width, the default) or userSpaceOnUse."
    ),
    svg!(
        "svgOrient",
        DEFS,
        "A `<marker>`'s `orient`: auto (along the path), auto-start-reverse, or an angle in degrees."
    ),
    svg!(
        "svgStartOffset",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `startOffset`: where the text's anchor sits along the path, user units or a percentage of its length."
    ),
    svg!(
        "svgSide",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `side`: left (the default) or right (the path run backwards)."
    ),
    svg!(
        "svgMethod",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `method`: align (each glyph turned rigidly onto the path; stretch draws as align, as browsers do)."
    ),
    svg!(
        "svgSpacing",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `spacing`: exact or auto (both keep the font's advances)."
    ),
    svg!(
        "svgPath",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `path`: its own path data instead of `href`."
    ),
    svg!(
        "svgTextPathHref",
        On::Kinds(&["svgText"]),
        "A `<textPath>`'s `href=\"#id\"`: the path or shape the text runs along."
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
        "SVG `href`: an `<image>`'s atlas icon by name or render texture handle (`href={$target}`), what a `<use>` draws (`#id` of a symbol, group or shape), the gradient a gradient takes its stops from, or an `feImage`'s atlas image, texture handle or element (`#id` of a shape or group, drawn as `<use>` draws it)."
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
    svg!(
        "svgFilter",
        PAINTED,
        "SVG `filter=\"url(#id)\"`: the `<filter>` the element and its subtree are drawn through."
    ),
    svg!(
        "svgFilterUnits",
        DEFS,
        "A `<filter>`'s `filterUnits`: `objectBoundingBox` (default; its region in fractions of the box) or `userSpaceOnUse`."
    ),
    svg!(
        "svgPrimitiveUnits",
        DEFS,
        "A `<filter>`'s `primitiveUnits`: `userSpaceOnUse` (default) or `objectBoundingBox`."
    ),
    svg!(
        "svgColorInterpolationFilters",
        DEFS,
        "A `<filter>`'s or primitive's `color-interpolation-filters`: `linearRGB` (default) or `sRGB`."
    ),
    svg!(
        "svgIn",
        DEFS,
        "A filter primitive's `in`: `SourceGraphic`, `SourceAlpha` or an earlier `result` (left out: the previous result)."
    ),
    svg!(
        "svgIn2",
        DEFS,
        "A filter primitive's second input `in2` (`feComposite`, `feBlend`, `feDisplacementMap`)."
    ),
    svg!(
        "svgResult",
        DEFS,
        "A filter primitive's `result` name, read by later `in`/`in2`."
    ),
    svg!(
        "svgStdDeviation",
        DEFS,
        "`feGaussianBlur`'s or `feDropShadow`'s `stdDeviation`: one or two numbers."
    ),
    svg!(
        "svgFloodColor",
        DEFS,
        "`feFlood`'s or `feDropShadow`'s `flood-color`."
    ),
    svg!(
        "svgFloodOpacity",
        DEFS,
        "`feFlood`'s or `feDropShadow`'s `flood-opacity`."
    ),
    svg!(
        "svgOperator",
        DEFS,
        "`feComposite`'s `operator` (over, in, out, atop, xor, lighter, arithmetic) or `feMorphology`'s (erode, dilate)."
    ),
    svg!("svgK1", DEFS, "`feComposite` arithmetic's `k1`."),
    svg!("svgK2", DEFS, "`feComposite` arithmetic's `k2`."),
    svg!("svgK3", DEFS, "`feComposite` arithmetic's `k3`."),
    svg!("svgK4", DEFS, "`feComposite` arithmetic's `k4`."),
    svg!(
        "svgMode",
        DEFS,
        "`feBlend`'s `mode`: a `mix-blend-mode` name."
    ),
    svg!(
        "svgType",
        DEFS,
        "`feColorMatrix`'s `type` (matrix, saturate, hueRotate, luminanceToAlpha), a `feFunc*`'s (identity, table, discrete, linear, gamma) or `feTurbulence`'s (turbulence, fractalNoise)."
    ),
    svg!("svgValues", DEFS, "`feColorMatrix`'s `values`."),
    svg!(
        "svgTableValues",
        DEFS,
        "A `feFunc*`'s `tableValues` (at most 16 are read)."
    ),
    svg!("svgSlope", DEFS, "A linear `feFunc*`'s `slope`."),
    svg!("svgIntercept", DEFS, "A linear `feFunc*`'s `intercept`."),
    svg!("svgAmplitude", DEFS, "A gamma `feFunc*`'s `amplitude`."),
    svg!("svgExponent", DEFS, "A gamma `feFunc*`'s `exponent`."),
    svg!(
        "svgRadius",
        DEFS,
        "`feMorphology`'s `radius`: one or two numbers."
    ),
    svg!("svgScale", DEFS, "`feDisplacementMap`'s `scale`."),
    svg!(
        "svgXChannelSelector",
        DEFS,
        "`feDisplacementMap`'s `xChannelSelector`: R, G, B or A."
    ),
    svg!(
        "svgYChannelSelector",
        DEFS,
        "`feDisplacementMap`'s `yChannelSelector`: R, G, B or A."
    ),
    svg!(
        "svgBaseFrequency",
        DEFS,
        "`feTurbulence`'s `baseFrequency`: one or two numbers."
    ),
    svg!("svgNumOctaves", DEFS, "`feTurbulence`'s `numOctaves`."),
    svg!("svgSeed", DEFS, "`feTurbulence`'s `seed`."),
    svg!(
        "svgStitchTiles",
        DEFS,
        "`feTurbulence`'s `stitchTiles`: stitch or noStitch."
    ),
    svg!(
        "svgOrder",
        DEFS,
        "`feConvolveMatrix`'s `order`: one or two numbers."
    ),
    svg!(
        "svgKernelMatrix",
        DEFS,
        "`feConvolveMatrix`'s `kernelMatrix` (at most 64 values)."
    ),
    svg!("svgDivisor", DEFS, "`feConvolveMatrix`'s `divisor`."),
    svg!("svgBias", DEFS, "`feConvolveMatrix`'s `bias`."),
    svg!("svgTargetX", DEFS, "`feConvolveMatrix`'s `targetX`."),
    svg!("svgTargetY", DEFS, "`feConvolveMatrix`'s `targetY`."),
    svg!(
        "svgEdgeMode",
        DEFS,
        "`feConvolveMatrix`'s `edgeMode`: duplicate, wrap or none."
    ),
    svg!(
        "svgPreserveAlpha",
        DEFS,
        "`feConvolveMatrix`'s `preserveAlpha`."
    ),
    svg!(
        "svgKernelUnitLength",
        DEFS,
        "`kernelUnitLength` of `feConvolveMatrix` and the lighting primitives: a cell's size in primitive units (left out, one device px)."
    ),
    svg!(
        "svgSurfaceScale",
        DEFS,
        "A lighting primitive's `surfaceScale`."
    ),
    svg!(
        "svgDiffuseConstant",
        DEFS,
        "`feDiffuseLighting`'s `diffuseConstant`."
    ),
    svg!(
        "svgSpecularConstant",
        DEFS,
        "`feSpecularLighting`'s `specularConstant`."
    ),
    svg!(
        "svgSpecularExponent",
        DEFS,
        "`feSpecularLighting`'s or `feSpotLight`'s `specularExponent`."
    ),
    svg!(
        "svgLightingColor",
        DEFS,
        "A lighting primitive's `lighting-color`."
    ),
    svg!("svgAzimuth", DEFS, "`feDistantLight`'s `azimuth`, degrees."),
    svg!(
        "svgElevation",
        DEFS,
        "`feDistantLight`'s `elevation`, degrees."
    ),
    svg!("svgZ", DEFS, "`fePointLight`'s or `feSpotLight`'s `z`."),
    svg!("svgPointsAtX", DEFS, "`feSpotLight`'s `pointsAtX`."),
    svg!("svgPointsAtY", DEFS, "`feSpotLight`'s `pointsAtY`."),
    svg!("svgPointsAtZ", DEFS, "`feSpotLight`'s `pointsAtZ`."),
    svg!(
        "svgLimitingConeAngle",
        DEFS,
        "`feSpotLight`'s `limitingConeAngle`, degrees."
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
        doc: "CSS `mask-image` or the `mask` shorthand: layers of `url(…)` and gradients whose alpha (or luminance) cuts the element and its subtree, each sized, placed and combined as the `mask-*` longhands say (`mask-image: linear-gradient(#000 70%, transparent)`).",
    },
    PropSpec {
        name: "backgroundOrigin",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-origin` of each image layer, a comma list: the box it is sized and placed in, `border-box`, `padding-box` (default) or `content-box`.",
    },
    PropSpec {
        name: "backgroundClip",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-clip` of each layer, a comma list: where it is painted, `border-box` (default), `padding-box`, `content-box` or `text` (the element's glyphs); the last layer's cuts the colour.",
    },
    PropSpec {
        name: "backgroundAttachment",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-attachment` of each layer: `scroll` (default), `fixed` (placed in the viewport) or `local` (scrolls with a scroller's content).",
    },
    PropSpec {
        name: "backgroundBlendMode",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `background-blend-mode` of each layer, a comma list of `mix-blend-mode`'s modes: how it blends with the layers and colour under it.",
    },
    PropSpec {
        name: "maskSize",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-size` of each mask layer, as `background-size`.",
    },
    PropSpec {
        name: "maskPosition",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-position` of each mask layer, as `background-position`.",
    },
    PropSpec {
        name: "maskRepeat",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-repeat` of each mask layer, as `background-repeat`.",
    },
    PropSpec {
        name: "maskOrigin",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-origin` of each mask layer: `border-box` (default), `padding-box` or `content-box`.",
    },
    PropSpec {
        name: "maskClip",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-clip` of each mask layer: `border-box` (default), `padding-box`, `content-box` or `no-clip`.",
    },
    PropSpec {
        name: "maskMode",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-mode` of each mask layer: `match-source` or `alpha` (its alpha), or `luminance`.",
    },
    PropSpec {
        name: "maskComposite",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-composite` of each mask layer with those under it: `add`, `subtract`, `intersect` or `exclude`.",
    },
    PropSpec {
        name: "borderImageSource",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `border-image-source`: `url(…)`, a gradient or `none`.",
    },
    PropSpec {
        name: "borderImageSlice",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `border-image-slice`: one to four numbers (image px) or percentages, and `fill`.",
    },
    PropSpec {
        name: "borderImageWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `border-image-width`: one to four of `<n>px`, `<n>%`, a number (times the border width) or `auto`.",
    },
    PropSpec {
        name: "borderImageOutset",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `border-image-outset`: one to four of `<n>px` or a number (times the border width).",
    },
    PropSpec {
        name: "borderImageRepeat",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `border-image-repeat`: one or two of `stretch`, `repeat`, `round`, `space`.",
    },
    PropSpec {
        name: "maskBorderSource",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-source`: `url(…)`, a gradient or `none`.",
    },
    PropSpec {
        name: "maskBorderSlice",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-slice`, as `border-image-slice`.",
    },
    PropSpec {
        name: "maskBorderWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-width`, as `border-image-width`.",
    },
    PropSpec {
        name: "maskBorderOutset",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-outset`, as `border-image-outset`.",
    },
    PropSpec {
        name: "maskBorderRepeat",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-repeat`, as `border-image-repeat`.",
    },
    PropSpec {
        name: "maskBorderMode",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `mask-border-mode`: `alpha` (default) or `luminance`.",
    },
    PropSpec {
        name: "backdropFilter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `backdrop-filter`: what is really under the element (the screen, or the group it is drawn in) through `filter`'s functions and `url(#id)` in order, laid in its border-box shape under its background.",
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
        doc: "CSS `mix-blend-mode`: any of CSS Compositing's modes (normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion, hue, saturation, color, luminosity) or plus-lighter. The element and its subtree composite as one group over what is under it.",
    },
    PropSpec {
        name: "isolation",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `isolation`, `auto` or `isolate`: `isolate` makes the element a group its descendants' blends blend within, over a transparent backdrop.",
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
        doc: "CSS `clip-path`: a basic shape (`inset()`, `rect()`, `xywh()`, `circle()`, `ellipse()`, `polygon()`, `path()` or `shape()`) over a geometry box (`border-box` by default, `padding-box`, `content-box`, `margin-box`; `fill-box` is the content box, `stroke-box` and `view-box` the border box), a geometry box alone (that box with its corners), or `url(#id)` of a `<clipPath>` in the document. It cuts everything the element draws (shadows, background, borders) and its children, by the shape's coverage with either fill rule.",
    },
    PropSpec {
        name: "material",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-material: glass <regular|thick> <lift|float>` supplies glass tint, edge, elevation and reduced-transparency fallback; standard `backdrop-filter` supplies its blur and colour filters. Alternatively `<name>(<n> …[, <colour>])` composites through a built-in shader (dissolve, shine, hologram, glitch, ripple, pixelate, color-adjust, outline, blur), or `\"<asset>\" [<n> …]` loads a `type ui-material` asset with four numbers per parameter. `none` removes the material. Values may substitute author variables, but the engine reads no named custom property. Extension reason: standard CSS has no shader-material or native glass-weight/elevation primitive.",
    },
    PropSpec {
        name: "sound",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "`-og-sound: none | [ <event> [ <url>+ [volume(<number> <number>?)]? [pitch(<number> <number>?)]? | none ] ]#`. Each event takes up to four URLs, selected without immediate repeats from the checkpointed engine UI random stream. Volume is an ordered 0..1 range, pitch an ordered positive range. URLs resolve beside the CSS source. `hover none` silences one event. Extension reason: standard CSS has no interaction audio playback. Haptics remain in the og.* API.",
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
        name: "overscrollBehaviorX",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `overscroll-behavior-x` (and `overscroll-behavior`'s first, `-inline`) of a scroll container: `contain` or `none` keeps a wheel, drag or fling that reaches its end from scrolling the container around it; `auto` hands the rest on.",
    },
    PropSpec {
        name: "overscrollBehaviorY",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `overscroll-behavior-y` (and `overscroll-behavior`'s second, `-block`): as `overscrollBehaviorX`, vertically.",
    },
    PropSpec {
        name: "scrollBehavior",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-behavior: auto | smooth` of a scroll container: smooth glides a scroll into view (a focus move, a pad's step, og.ui.scrollIntoView) there along the momentum curve; the player's reduced motion jumps.",
    },
    PropSpec {
        name: "scrollMarginTop",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-margin-top` (`scroll-margin`, `-block`, `-block-start`): the element's box grown by it where a scroll into view or a snap position lines it up.",
    },
    PropSpec {
        name: "scrollMarginRight",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-margin-right` (`scroll-margin`, `-inline`, `-inline-end`).",
    },
    PropSpec {
        name: "scrollMarginBottom",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-margin-bottom` (`scroll-margin`, `-block`, `-block-end`).",
    },
    PropSpec {
        name: "scrollMarginLeft",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-margin-left` (`scroll-margin`, `-inline`, `-inline-start`).",
    },
    PropSpec {
        name: "scrollPaddingTop",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-padding-top` (`scroll-padding`, `-block`, `-block-start`) of a scroll container: px or % of its view inset from its top where a scroll into view or a snap position lines things up (under a sticky header).",
    },
    PropSpec {
        name: "scrollPaddingRight",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-padding-right` (`scroll-padding`, `-inline`, `-inline-end`).",
    },
    PropSpec {
        name: "scrollPaddingBottom",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-padding-bottom` (`scroll-padding`, `-block`, `-block-end`).",
    },
    PropSpec {
        name: "scrollPaddingLeft",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-padding-left` (`scroll-padding`, `-inline`, `-inline-start`).",
    },
    PropSpec {
        name: "scrollSnapStop",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scroll-snap-stop: normal | always` of a snap area: always stops a fling or a settling wheel at it rather than passing it for a farther one.",
    },
    PropSpec {
        name: "scrollbarColor",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scrollbar-color: auto | <thumb> <track>` of a scroll container: the scrollbars' thumb colour and a track drawn under it; inherited.",
    },
    PropSpec {
        name: "scrollbarGutter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `scrollbar-gutter: auto | stable [both-edges]` of a vertical scroll container: stable keeps the scrollbar's room on its end side whether or not it overflows (both-edges on its start side too), so its content does not shift when it starts to.",
    },
    PropSpec {
        name: "dataDetents",
        ty: Ty::Text,
        on: On::Kinds(&["dialog"]),
        two_way: false,
        doc: "`<dialog modal data-detents=\"40% 90%\">`: a sheet docked to its `data-edge` (the bottom at full width by default), dragged onto its detents and swiped off its edge to close (writing `open` back). Detents are positive px or percentages of viewport height, or width for a side sheet. It opens at the first; drag and snap state is checkpointed.",
    },
    PropSpec {
        name: "dataEdge",
        ty: Ty::Word(&["block-end", "inline-start", "inline-end"]),
        on: On::Kinds(&["dialog"]),
        two_way: false,
        doc: "`data-edge=\"block-end|inline-start|inline-end\"` beside `data-detents`: the edge the sheet docks to, in CSS's logical names, so the inline ones mirror right to left (`inline-end` is the right left to right, the left in Arabic); a side sheet is full height, its detents widths, and swipes off sideways.",
    },
    PropSpec {
        name: "insetTop",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `top` / `inset-block-start` as written: px, `%` of the containing block, `anchor()` or `calc()` of one plus px.",
    },
    PropSpec {
        name: "insetRight",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `right` / `inset-inline-end` as written (see insetTop).",
    },
    PropSpec {
        name: "insetBottom",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `bottom` / `inset-block-end` as written (see insetTop).",
    },
    PropSpec {
        name: "insetLeft",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `left` / `inset-inline-start` as written (see insetTop).",
    },
    PropSpec {
        name: "viewTransitionName",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `view-transition-name`: the shared element's name in a view transition (empty: none; `auto` and `match-element` name it by its identity). Its old and new selves morph as one `::view-transition-group(name)`.",
    },
    PropSpec {
        name: "viewTransitionClass",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `view-transition-class`: the classes its view-transition group is styled by (`::view-transition-group(*.card)`), space-separated.",
    },
    PropSpec {
        name: "viewTransitionGroup",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `view-transition-group`: where its view-transition group nests (empty: `normal`, in the nearest shared ancestor's that is not normal; `contain`: as normal, and its own nest in it; `nearest`: in the nearest shared ancestor's; a name: in that ancestor's). A nested group moves with its parent, inside its `::view-transition-group-children`.",
    },
    PropSpec {
        name: "anchorName",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `anchor-name`: the dashed names (`--menu`) other elements anchor to it by.",
    },
    PropSpec {
        name: "positionAnchor",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `position-anchor`: its default anchor's dashed name (empty: its implicit anchor, a popover's invoker).",
    },
    PropSpec {
        name: "positionArea",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `position-area`: the cell or span of the 3×3 grid its anchor cuts its containing block into (`bottom center`, `span-right top`, `block-end`).",
    },
    PropSpec {
        name: "positionTry",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `position-try-fallbacks`: flips (`flip-block`, `flip-inline`, `flip-start`), `@position-try` names and position-areas tried in order while it overflows.",
    },
    PropSpec {
        name: "positionTryOrder",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `position-try-order`: normal, most-width or most-height.",
    },
    PropSpec {
        name: "positionVisibility",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `position-visibility`: always, anchors-visible (the default) or no-overflow.",
    },
    PropSpec {
        name: "anchorWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `width: anchor-size(…)` as written.",
    },
    PropSpec {
        name: "anchorHeight",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `height: anchor-size(…)` as written.",
    },
    PropSpec {
        name: "columnCount",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-count`: `auto` or how many columns its flow lays out in (a multi-column container).",
    },
    PropSpec {
        name: "columnWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-width`: `auto` or the least width of a column; with `column-count`, as many columns as fit, at most that count.",
    },
    PropSpec {
        name: "float",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `float` (`none`, `left`, `right`, `inline-start`, `inline-end`): in a block's flow it leaves the flow for that side of its containing block, shrinking to fit; the lines beside it shorten and boxes that start a formatting context of their own keep clear of it.",
    },
    PropSpec {
        name: "clear",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `clear` (`none`, `left`, `right`, `both`, `inline-start`, `inline-end`): it is placed below the earlier floats on those sides.",
    },
    PropSpec {
        name: "columnSpan",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-span: none | all`: in a multi-column container, `all` spans every column and starts a new set of columns below it.",
    },
    PropSpec {
        name: "columnFill",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-fill`: `balance` (and `balance-all`) evens the columns' heights; `auto` fills them in turn to the container's definite height.",
    },
    PropSpec {
        name: "columnRuleWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-rule-width`: a px length, `thin`, `medium` or `thick`.",
    },
    PropSpec {
        name: "columnRuleStyle",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `column-rule-style`: a line style (`none`, `solid`, `dashed`, `dotted`, `double`…).",
    },
    PropSpec {
        name: "columnRuleColor",
        ty: Ty::Paint,
        on: On::All,
        two_way: false,
        doc: "CSS `column-rule-color` (`currentcolor`: its text colour).",
    },
    PropSpec {
        name: "breakBefore",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `break-before` (and `page-break-before`): `column`, `page` or `always` starts a new column in a multi-column flow.",
    },
    PropSpec {
        name: "breakAfter",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `break-after` (and `page-break-after`): `column`, `page` or `always` puts what follows in a new column.",
    },
    PropSpec {
        name: "breakInside",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `break-inside` (and `page-break-inside`): `avoid` keeps a text's lines in one column.",
    },
    PropSpec {
        name: "contain",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `contain`: `size` (or `inline-size`) sizes it as if empty (or as `contain-intrinsic-size` says), `layout` and `paint` make it a formatting context and clip what it draws to its box, `style` keeps its counters and quotes inside it; `strict` is all four, `content` all but size.",
    },
    PropSpec {
        name: "contentVisibility",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `content-visibility`: `hidden` skips its contents (not laid out, drawn, hit or in the accessibility tree); `auto` skips them while it is far from the screen and holds no focus, sized by `contain-intrinsic-size`; both contain its layout, style and paint.",
    },
    PropSpec {
        name: "containIntrinsicWidth",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `contain-intrinsic-width` (and `contain-intrinsic-size`): the content width size containment gives it, `none` (0) or a px length; `auto` uses its last laid-out width while its contents are skipped.",
    },
    PropSpec {
        name: "containIntrinsicHeight",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `contain-intrinsic-height` (and `contain-intrinsic-size`): as `containIntrinsicWidth`, down.",
    },
    PropSpec {
        name: "resize",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `resize` on an element whose overflow is not visible: `both`, `horizontal` (`inline`) or `vertical` (`block`) puts a grab handle on its bottom end corner whose drag sets its width or height.",
    },
    PropSpec {
        name: "willChange",
        ty: Ty::Text,
        on: On::All,
        two_way: false,
        doc: "CSS `will-change`: properties it is about to change; one that makes a group (transform, opacity, filter…) draws it on a layer of its own.",
    },
];
