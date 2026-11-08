//! Elements a document names that HTML has no core name for (SVG, a
//! viewport, a joystick…), attached by the document runtime: a tag the
//! core table does not know comes here first.

/// SVG's shapes, each one element of kind `svgShape` (its tag says which).
pub(crate) const SVG_SHAPES: &[&str] = &[
    "path", "rect", "circle", "ellipse", "line", "polyline", "polygon",
];

/// What SVG elements read past: definitions drawn only where referenced.
const SVG_DEFS: &[&str] = &[
    "defs",
    "linearGradient",
    "radialGradient",
    "stop",
    "title",
    "desc",
    "metadata",
];

/// SVG's presentation properties: attributes on SVG elements and CSS
/// properties of them (CSS wins over the attribute), inherited down the
/// `<svg>` tree as SVG inherits them.
pub(crate) const PRESENTATION: &[&str] = &[
    "fill",
    "fill-opacity",
    "fill-rule",
    "stroke",
    "stroke-opacity",
    "stroke-width",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-miterlimit",
];

/// A CSS property only SVG elements read (cascaded as text, read by the
/// SVG build).
pub(crate) fn svg_longhand(name: &str) -> bool {
    PRESENTATION.contains(&name)
}

fn in_svg(parent_tag: Option<&str>) -> bool {
    matches!(parent_tag, Some("svg" | "g" | "defs"))
}

/// The internal kind lowercase `tag` names (one [`super::schema_extra`]
/// registers), inside an element whose selector tag is `parent_tag`.
/// `Some(Err(hint))`: the tag is known but not valid there. `None`: not
/// an extern element.
pub(super) fn kind(tag: &str, parent_tag: Option<&str>) -> Option<Result<&'static str, String>> {
    let svg_child = |kind: &'static str| {
        Some(if in_svg(parent_tag) {
            Ok(kind)
        } else {
            Err(format!("<{tag}> is SVG: write it inside an <svg>"))
        })
    };
    match tag {
        "svg" => Some(Ok("svg")),
        "g" => svg_child("svgGroup"),
        t if SVG_SHAPES.contains(&t) => svg_child("svgShape"),
        "stop" => Some(
            if matches!(parent_tag, Some("linearGradient" | "radialGradient")) {
                Ok("svgDef")
            } else {
                Err("<stop> belongs in a <linearGradient> or <radialGradient>".into())
            },
        ),
        t if SVG_DEFS.contains(&t) => svg_child("svgDef"),
        t if in_svg(parent_tag) => Some(Err(format!(
            "<{t}> is not in the inline SVG subset: g, path, rect, circle, ellipse, line, polyline, polygon, defs, linearGradient, radialGradient and stop (docs/ui.md §3.3)"
        ))),
        _ => None,
    }
}

/// The prop an attribute `attr` of an element of `kind` sets, when it is
/// an SVG attribute (SVG's names would collide with the core's `x`,
/// `width`, `fill`…). `None`: the core's mapping.
pub(super) fn property(kind: &str, attr: &str) -> Option<String> {
    let svg = match kind {
        "svg" => matches!(attr, "viewBox") || PRESENTATION.contains(&attr),
        "svgGroup" => attr == "transform" || PRESENTATION.contains(&attr),
        "svgShape" => attr != "opacity",
        "svgDef" => true,
        _ => false,
    };
    svg.then(|| super::schema_extra::svg_prop(attr))
        .flatten()
        .map(str::to_string)
}
