//! SVG's attributes and the props they set.
use super::super::{On, PropSpec, Ty};

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
pub fn svg_prop(attr: &str) -> Option<&'static str> {
    SVG_ATTRS.iter().find(|(a, _)| *a == attr).map(|(_, p)| *p)
}

/// The SVG attribute a prop is written as (the catalogue's column).
pub fn svg_attr(prop: &str) -> Option<&'static str> {
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
];
