//! Elements a document names that HTML has no core name for (SVG, a
//! viewport, a joystick…), attached by the document runtime: a tag the
//! core table does not know comes here first.

/// SVG's shapes, each one element of kind `svgShape` (its tag says which).
pub(crate) const SVG_SHAPES: &[&str] = &[
    "path", "rect", "circle", "ellipse", "line", "polyline", "polygon", "image",
];

/// What SVG elements read past: definitions drawn only where referenced
/// (`<symbol>` by a `<use>`, `<clipPath>` by `clip-path`, `<mask>` by
/// `mask`, `<pattern>` and gradients by a paint), and text for people
/// (`<title>` names its parent, see [`finish`]).
const SVG_DEFS: &[&str] = &[
    "defs",
    "symbol",
    "clipPath",
    "mask",
    "pattern",
    "linearGradient",
    "radialGradient",
    "stop",
    "metadata",
];

/// The elements whose children are SVG.
const SVG_PARENTS: &[&str] = &["svg", "g", "defs", "symbol", "clipPath", "mask", "pattern"];

/// What a `<tspan>` inside a `<text>` takes: the run's look.
pub(super) const TSPAN_ATTRS: &[&str] = &["font-weight", "font-style", "text-decoration", "fill"];

/// `run` styled by a `<tspan>`'s attribute `name`.
pub(super) fn tspan(run: &mut super::InlineRun, name: &str, value: &str) -> Result<(), String> {
    let v = value.trim();
    match name {
        "font-weight" => {
            run.bold = match v {
                "bold" | "bolder" => true,
                "normal" | "lighter" => false,
                n => n
                    .parse::<u16>()
                    .map(|w| w >= 600)
                    .map_err(|_| format!("font-weight=\"{v}\" is normal, bold or 100–900"))?,
            }
        }
        "font-style" => run.italic = matches!(v, "italic" | "oblique"),
        "text-decoration" => {
            for w in v.split_whitespace() {
                match w {
                    "underline" => run.underline = true,
                    "line-through" => run.strike = true,
                    "none" => {}
                    w => {
                        return Err(format!(
                            "text-decoration \"{w}\" is underline, line-through or none"
                        ));
                    }
                }
            }
        }
        "fill" => {
            crate::svg::color(v).map_err(|e| format!("fill: {e}"))?;
            run.color = Some(v.to_string());
        }
        _ => {}
    }
    Ok(())
}

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

/// SVG's text presentation properties, inherited as [`PRESENTATION`] is
/// (a group's `font-size` reaches its texts). As CSS they are read where
/// no core property has the name (`font-size` and `font-weight` on SVG
/// text are written as attributes).
pub(crate) const TEXT_PRESENTATION: &[&str] = &[
    "font-size",
    "font-weight",
    "font-style",
    "text-anchor",
    "dominant-baseline",
];

/// A CSS property only SVG elements read (cascaded as text, read by the
/// SVG build).
pub(crate) fn svg_longhand(name: &str) -> bool {
    PRESENTATION.contains(&name) || matches!(name, "text-anchor" | "dominant-baseline")
}

pub(super) fn in_svg(parent_tag: Option<&str>) -> bool {
    parent_tag.is_some_and(|p| SVG_PARENTS.contains(&p))
}

/// Inside any SVG element (`<title>` and `<desc>` go in shapes too).
fn under_svg(parent_tag: Option<&str>) -> bool {
    in_svg(parent_tag) || parent_tag.is_some_and(|p| SVG_SHAPES.contains(&p) || p == "use")
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
        "viewport" => Some(Ok("viewport")),
        "joystick" => Some(Ok("joystick")),
        "anchor" => Some(Ok("anchor")),
        "dpad" => Some(Ok("dpad")),
        "g" => svg_child("svgGroup"),
        "tspan" => Some(Err(
            "<tspan> is a run inside an SVG <text>; write it there".into()
        )),
        "use" => svg_child("svgUse"),
        // Parsed as text (so its content binds), then lifted into its
        // parent's name by [`finish`].
        "title" | "desc" => Some(if under_svg(parent_tag) {
            Ok("label")
        } else {
            Err(format!(
                "<{tag}> names an SVG element; outside SVG, write aria-label (or the HTML title attribute)"
            ))
        }),
        // Outside SVG, `<image>` is the core's old name for `<img>`.
        "image" if !in_svg(parent_tag) => None,
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
            "<{t}> is not in the inline SVG subset: g, use, text, tspan, path, rect, circle, ellipse, line, polyline, polygon, image, defs, symbol, clipPath, mask, pattern, linearGradient, radialGradient, stop, title and desc (docs/ui.md §3.3)"
        ))),
        _ => None,
    }
}

/// The prop an attribute `attr` of an element of `kind` sets, when it is
/// an SVG attribute (SVG's names would collide with the core's `x`,
/// `width`, `fill`…). `None`: the core's mapping.
pub(super) fn property(kind: &str, attr: &str) -> Option<String> {
    if kind == "viewport" && attr == "src" {
        return Some("viewportSrc".into());
    }
    if kind == "panel" {
        let p = match attr {
            "at" => "worldAt",
            "size" => "worldSize",
            "rotation" => "worldRotation",
            "src" => "viewportSrc",
            _ => return None,
        };
        return Some(p.into());
    }
    if kind == "anchor" {
        let p = match attr {
            "at" => "worldAt",
            "offset" => "worldOffset",
            "clamp" => "worldClamp",
            "scale-at" => "worldScaleAt",
            _ => return None,
        };
        return Some(p.into());
    }
    let svg = match kind {
        "svg" => {
            matches!(attr, "viewBox")
                || PRESENTATION.contains(&attr)
                || TEXT_PRESENTATION.contains(&attr)
        }
        "svgGroup" => {
            matches!(attr, "transform" | "clip-path" | "mask")
                || PRESENTATION.contains(&attr)
                || TEXT_PRESENTATION.contains(&attr)
        }
        "svgShape" | "svgUse" => attr != "opacity",
        "svgText" => {
            matches!(
                attr,
                "x" | "y" | "dx" | "dy" | "transform" | "clip-path" | "mask"
            ) || PRESENTATION.contains(&attr)
                || TEXT_PRESENTATION.contains(&attr)
        }
        "svgDef" => true,
        _ => false,
    };
    svg.then(|| super::schema_extra::svg_prop(attr))
        .flatten()
        .map(str::to_string)
}

/// A literal attribute of `n` by its SVG name.
fn literal(n: &super::DocNode, name: &str) -> Option<String> {
    n.selector_attrs
        .iter()
        .find(|(a, _)| a == name)
        .and_then(|(_, e)| match e {
            crate::bind::Expr::Lit(v) => Some(crate::bind::text(v)),
            _ => None,
        })
}

/// Copies node `from`'s subtree under `parent`, with the selector entries
/// of its elements, so the stylesheet styles the copy as the original.
fn copy<A: Clone>(
    doc: &mut super::UiDocument,
    selectors: &mut Vec<(usize, String, A)>,
    from: usize,
    parent: usize,
) -> usize {
    let at = doc.nodes.len();
    let mut node = doc.nodes[from].clone();
    node.parent = parent;
    node.children.clear();
    doc.nodes.push(node);
    if let Some(entry) = selectors.iter().find(|(n, ..)| *n == from).cloned() {
        selectors.push((at, entry.1, entry.2));
    }
    for c in doc.nodes[from].children.clone() {
        let child = copy(doc, selectors, c, at);
        doc.nodes[at].children.push(child);
    }
    at
}

/// After parsing, before the stylesheet: each `<use href="#id">` takes a
/// copy of what it names as its children (a `<symbol>`'s children, with
/// its `viewBox` and `preserveAspectRatio`, or the element itself), so it
/// builds as a group whose copies inherit its presentation attributes, as
/// SVG's shadow tree does. Then each SVG `<title>` names its parent (an
/// `<svg>`, a group, a `<use>` or a shape) as `aria-label` does, unless it
/// has one, so a bound title is the parent's own binding; titles and
/// `<desc>` leave the tree.
pub(super) fn finish<A: Clone>(
    doc: &mut super::UiDocument,
    selectors: &mut Vec<(usize, String, A)>,
) -> Result<(), String> {
    use super::NodeKind;
    let start = doc.nodes.len();
    let mut i = 0;
    while i < doc.nodes.len() {
        if doc.nodes[i].kind == NodeKind::Element("svgUse") {
            if doc.nodes.len() > start + 20_000 {
                return Err("<use> copies past 20000 elements: a <use> that draws itself?".into());
            }
            let href = literal(&doc.nodes[i], "href")
                .or_else(|| literal(&doc.nodes[i], "xlink:href"))
                .ok_or_else(|| "<use> takes a literal href=\"#id\"".to_string())?;
            let id = href.strip_prefix('#').unwrap_or(&href).to_string();
            let target = doc
                .nodes
                .iter()
                .position(|n| {
                    n.id == id
                        && matches!(n.kind, NodeKind::Element(k) if k.starts_with("svg") && k != "svg")
                })
                .ok_or_else(|| format!("<use href=\"#{id}\">: no SVG element has id \"{id}\""))?;
            if ancestor_of(doc, target, i) {
                return Err(format!("<use href=\"#{id}\"> is inside what it draws"));
            }
            if doc.nodes[target].selector_tag.as_deref() == Some("symbol") {
                for name in ["svgViewBox", "svgPreserveAspectRatio"] {
                    let own = doc.nodes[i].props.iter().any(|p| p.name() == name);
                    if !own
                        && let Some(p) = doc.nodes[target]
                            .props
                            .iter()
                            .find(|p| p.name() == name)
                            .cloned()
                    {
                        doc.nodes[i].props.push(p);
                    }
                }
                for c in doc.nodes[target].children.clone() {
                    let child = copy(doc, selectors, c, i);
                    doc.nodes[i].children.push(child);
                }
            } else {
                let child = copy(doc, selectors, target, i);
                doc.nodes[i].children.push(child);
            }
        }
        i += 1;
    }
    let name = super::schema::prop("name").expect("name is a prop").0;
    for i in 0..doc.nodes.len() {
        let tag = doc.nodes[i].selector_tag.as_deref();
        if !matches!(tag, Some("title" | "desc")) || doc.nodes[i].kind != NodeKind::Element("label")
        {
            continue;
        }
        let parent = doc.nodes[i].parent;
        if parent == super::NONE {
            continue;
        }
        let text = doc.nodes[i]
            .props
            .iter()
            .find(|p| p.name() == "text")
            .cloned();
        if tag == Some("title")
            && let Some(mut text) = text
            && !doc.nodes[parent].props.iter().any(|p| p.name() == "name")
        {
            text.spec = name;
            doc.nodes[parent].props.push(text);
        }
        doc.nodes[parent].children.retain(|&c| c != i);
    }
    Ok(())
}

/// Node `a` is `b` or one of its ancestors.
fn ancestor_of(doc: &super::UiDocument, a: usize, mut b: usize) -> bool {
    while b != super::NONE {
        if a == b {
            return true;
        }
        b = doc.nodes[b].parent;
    }
    false
}
