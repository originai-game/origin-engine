//! Inline SVG in documents (`docs/ui.md` §3.3): `<svg>`, `<g>` and the
//! basic shapes as retained elements, their attributes bindable.
//!
//! Each shape is one element whose draw is its path in the root `<svg>`'s
//! user space ([`crate::paint::ElementDraw::view_box`]): its fill is the
//! element's shape, drawn analytically by og-render's UI pass, and each of
//! its subpaths is a stroked outline. The view box is fitted into the box
//! when the frame is painted, so a box that resizes repaints only its
//! shapes and nothing is built again.
//!
//! SVG's presentation attributes inherit as CSS's inherited properties do:
//! an `<svg>` or `<g>` writes its effective values (and its accumulated
//! transform) into its computed map under names no stylesheet can write,
//! and each SVG element under it records reading them, so a group's bound
//! `fill` re-resolves that group and the shapes that read it, nothing
//! else.
use super::*;
use crate::doc::extern_element::{PRESENTATION, TEXT_PRESENTATION};
use crate::doc::schema_extra::svg_prop;
use crate::paint::{
    ColorRef, ElementDraw, Gradient, GradientKind, Paint, ShapeKind, Stop, Stroke, StrokeAlign,
};
use crate::svg::{Affine, GradientDef, PaintRef, Style, bounds, shape_path};
use og_core::glam::Vec2;

/// The kinds this module builds.
pub(super) const KINDS: &[&str] = &[
    "svg",
    "svgGroup",
    "svgShape",
    "svgUse",
    "svgDef",
    "svgText",
    "svgForeign",
];

const VIEW_BOX: &str = " svg viewBox";
const TRANSFORM: &str = " svg transform";

fn key(attr: &str) -> String {
    format!(" svg {attr}")
}

/// HTML's size of an `<svg>` with neither a size nor a view box.
const DEFAULT_SIZE: [f32; 2] = [300.0, 150.0];

fn fixed(s: Option<Size>) -> Option<f32> {
    match s? {
        Size::Fixed(v) => Some(v),
        _ => None,
    }
}

/// The view box `min-x min-y width height`.
fn view_box(text: &str) -> Option<[f32; 4]> {
    let n = crate::svg::numbers(text).ok()?;
    (n.len() == 4 && n[2] > 0.0 && n[3] > 0.0).then(|| [n[0], n[1], n[2], n[3]])
}

fn matrix(a: &Affine) -> String {
    let [a, b, c, d, e, f] = a.0;
    format!("matrix({a} {b} {c} {d} {e} {f})")
}

/// A presentation value the element sets: its CSS, else its attribute.
fn own(r: &Resolved, attr: &str) -> Option<String> {
    r.cascade
        .svg
        .iter()
        .rev()
        .find(|(name, _)| name == attr)
        .map(|(_, v)| v.clone())
        .or_else(|| svg_prop(attr).and_then(|p| r.str(p)).map(str::to_string))
}

/// An SVG element's inherited state, written into its computed map for
/// the SVG elements under it (see the module documentation).
pub(super) fn inherit(kind: &str, r: &mut Resolved) {
    if kind == "svgDef" {
        return;
    }
    let reads = &mut r.cascade.inherited_reads;
    reads.insert(VIEW_BOX.into());
    reads.insert(TRANSFORM.into());
    reads.extend(PRESENTATION.iter().chain(TEXT_PRESENTATION).map(|a| key(a)));
    let mut writes: Vec<(String, String)> = Vec::new();
    for attr in PRESENTATION.iter().chain(TEXT_PRESENTATION) {
        if let Some(v) = own(r, attr) {
            writes.push((key(attr), v));
        }
    }
    let computed = &r.cascade.computed;
    let inherited = |name: &str| computed.get(name).cloned().flatten();
    if kind == "svg" {
        let vb = match r.str("svgViewBox") {
            Some(v) => v.to_string(),
            None => {
                let w = fixed(r.size("width")).unwrap_or(DEFAULT_SIZE[0]);
                let h = fixed(r.size("height")).unwrap_or(DEFAULT_SIZE[1]);
                format!("0 0 {w} {h}")
            }
        };
        if view_box(&vb).is_none() {
            r.cascade
                .diagnostics
                .push(format!("viewBox \"{vb}\" is not four numbers with a size"));
        }
        writes.push((VIEW_BOX.into(), vb));
        writes.push((TRANSFORM.into(), matrix(&Affine::IDENTITY)));
    } else {
        let parent = inherited(TRANSFORM)
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let parent = if kind == "svgUse" {
            match use_transform(r) {
                Ok(t) => parent.then(&t),
                Err(e) => {
                    r.cascade.diagnostics.push(format!("<use>: {e}"));
                    parent
                }
            }
        } else {
            parent
        };
        let total = match r.str("svgTransform").map(Affine::parse) {
            Some(Ok(t)) => parent.then(&t),
            Some(Err(e)) => {
                r.cascade.diagnostics.push(format!("transform: {e}"));
                parent
            }
            None => parent,
        };
        writes.push((TRANSFORM.into(), matrix(&total)));
    }
    let computed = Arc::make_mut(&mut r.cascade.computed);
    for (name, value) in writes {
        computed.insert(name, Some(value));
    }
}

/// The widget memory holding when the element at `path` first showed:
/// its SMIL animations' clock starts there.
fn smil_clock(path: &str) -> u64 {
    crate::id::child_key(og_core::hash::hash_bytes(path.as_bytes()), "~smil")
}

/// Whether `n` holds SMIL animations.
fn animated(doc: &UiDocument, n: &DocNode) -> bool {
    n.children.iter().any(|&c| {
        doc.nodes[c]
            .selector_tag
            .as_deref()
            .is_some_and(|t| crate::svg::smil::TAGS.contains(&t))
    })
}

/// The path and transform of the shape `href="#id"` names, in its own
/// user space.
fn shape_by_href(
    doc: &UiDocument,
    href: &str,
    vb: [f32; 4],
) -> Result<(crate::svg::Path, f32), String> {
    let id = href.trim().strip_prefix('#').unwrap_or(href.trim());
    let node = doc
        .nodes
        .iter()
        .find(|n| {
            n.id == id
                && n.selector_tag.as_deref().is_some_and(|t| {
                    crate::doc::extern_element::SVG_SHAPES.contains(&t) && t != "image"
                })
        })
        .ok_or_else(|| format!("href=\"{href}\": no path or shape has id \"{id}\""))?;
    let attrs = literal_attrs(node)?;
    let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
    let tag = node.selector_tag.as_deref().unwrap_or("path");
    let shape = shape_path(tag, &get, vb)?.ok_or_else(|| format!("#{id} is empty"))?;
    let own = match get("transform") {
        Some(v) => Affine::parse(v)?,
        None => Affine::IDENTITY,
    };
    // A length along it measures the path in its pathLength.
    let k = match get("pathLength").map(|v| v.trim().parse::<f32>()) {
        Some(Ok(l)) if l > 0.0 => shape.length(vb[2].max(vb[3]) / 2000.0) / l,
        _ => 1.0,
    };
    Ok((shape.transformed(&own), k))
}

/// An animation element's begins in seconds of its element's clock,
/// following `id.begin` and `id.end` to the animations they name.
fn begin_times(doc: &UiDocument, a: &crate::svg::smil::Animation, depth: u32) -> Vec<f32> {
    use crate::svg::smil::{Animation, Begin};
    let mut out = Vec::new();
    for b in &a.begins {
        match b {
            Begin::At(t) => out.push(*t),
            Begin::Sync { id, end, offset } if depth < 8 => {
                let Some(node) = doc.nodes.iter().find(|n| {
                    &n.id == id
                        && n.selector_tag
                            .as_deref()
                            .is_some_and(|t| crate::svg::smil::TAGS.contains(&t))
                }) else {
                    continue;
                };
                let tag = node.selector_tag.as_deref().unwrap_or("animate");
                let Some(other) = literal_attrs(node)
                    .ok()
                    .and_then(|attrs| Animation::parse(tag, &attrs, Some(Default::default())).ok())
                else {
                    continue;
                };
                let Some(first) = begin_times(doc, &other, depth + 1)
                    .into_iter()
                    .reduce(f32::min)
                else {
                    continue;
                };
                match (end, other.active()) {
                    (false, _) => out.push(first + offset),
                    (true, Some(d)) => out.push(first + d + offset),
                    (true, None) => {}
                }
            }
            Begin::Sync { .. } => {}
        }
    }
    out
}

/// Applies `n`'s SMIL animations (`<animate>`, `<set>`,
/// `<animateTransform>`, `<animateMotion>`) at the UI clock: each
/// overrides an attribute, a presentation property or the transform as
/// SVG's override style sheet does. Presentation only: the clock is the
/// element's widget memory and the values never leave the UI. Returns
/// whether any still moves (or waits to begin), so it resolves again
/// next frame. Under reduced motion each shows its frozen end, or none.
pub(super) fn animate(
    ui: &Ui,
    doc: &UiDocument,
    n: &DocNode,
    path: &str,
    r: &mut Resolved,
) -> bool {
    use crate::svg::smil::{Animation, Target};
    if !animated(doc, n) {
        return false;
    }
    let vb = r
        .cascade
        .computed
        .get(VIEW_BOX)
        .cloned()
        .flatten()
        .and_then(|v| view_box(&v))
        .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
    let tps = ui.ticks_per_second().max(1) as f32;
    let start = ui
        .state
        .anims
        .get(&smil_clock(path))
        .map_or(ui.state.tick, |a| a.t);
    let still = ui.theme().reduced_motion;
    let now = ui.state.tick.saturating_sub(start) as f32 / tps;
    let mut running = false;
    let mut values: Vec<(String, String)> = Vec::new();
    let mut motion: Option<String> = None;
    for &c in &n.children {
        let node = &doc.nodes[c];
        let Some(tag) = node
            .selector_tag
            .as_deref()
            .filter(|t| crate::svg::smil::TAGS.contains(t))
        else {
            continue;
        };
        let parsed = literal_attrs(node).and_then(|attrs| {
            let mpath = match node
                .children
                .iter()
                .map(|&m| &doc.nodes[m])
                .find(|m| m.selector_tag.as_deref() == Some("mpath"))
            {
                Some(m) => {
                    let attrs = literal_attrs(m)?;
                    let href = attrs
                        .iter()
                        .find(|(k, _)| k == "href")
                        .map(|(_, v)| v.as_str())
                        .ok_or("<mpath> names its path: href=\"#id\"")?;
                    Some(
                        shape_by_href(doc, href, vb)
                            .map_err(|e| format!("<mpath {e}"))?
                            .0,
                    )
                }
                None => None,
            };
            Animation::parse(tag, &attrs, mpath)
        });
        let a = match parsed {
            Ok(a) => a,
            Err(e) => {
                r.cascade.diagnostics.push(format!("<{tag}>: {e}"));
                continue;
            }
        };
        let begins = begin_times(doc, &a, 0);
        let at = if still {
            match (a.active(), a.freeze) {
                (Some(d), true) => begins.iter().copied().reduce(f32::min).map(|b| (b, d)),
                _ => None,
            }
        } else {
            running |= begins.iter().any(|b| *b > now);
            Animation::current(&begins, now).map(|b| (b, now - b))
        };
        let Some((_, local)) = at else {
            continue;
        };
        let (progress, live) = a.progress(local);
        running |= live && !still;
        let Some(progress) = progress else {
            continue;
        };
        let current = |values: &[(String, String)], name: &str| -> String {
            if let Some((_, v)) = values.iter().rev().find(|(k, _)| k == name) {
                return v.clone();
            }
            if name == "transform" {
                return r.str("svgTransform").unwrap_or_default().to_string();
            }
            if name == "opacity" {
                return r.num("opacity").unwrap_or(1.0).to_string();
            }
            own(r, name)
                .or_else(|| svg_prop(name).and_then(|p| r.str(p)).map(str::to_string))
                .unwrap_or_default()
        };
        match &a.target {
            Target::Attr(name) => {
                let v = a.sample(&current(&values, name), progress);
                values.push((name.clone(), v));
            }
            Target::Transform(_) => {
                let v = a.sample(&current(&values, "transform"), progress);
                values.push(("transform".into(), v));
            }
            Target::Motion { .. } => motion = Some(a.sample("", progress)),
        }
    }
    if let Some(m) = motion {
        let base = values
            .iter()
            .rev()
            .find(|(k, _)| k == "transform")
            .map(|(_, v)| v.clone())
            .or_else(|| r.str("svgTransform").map(str::to_string))
            .unwrap_or_default();
        values.push(("transform".into(), format!("{m} {base}")));
    }
    for (name, v) in values {
        if PRESENTATION.contains(&name.as_str()) || TEXT_PRESENTATION.contains(&name.as_str()) {
            r.cascade.svg.push((name, v));
        } else if name == "opacity" {
            match v.trim().parse::<f64>() {
                Ok(o) => {
                    if let Some((spec, _)) = schema::prop("opacity") {
                        r.vals.push((spec, V::Num(o.clamp(0.0, 1.0))));
                    }
                }
                Err(_) => r
                    .cascade
                    .diagnostics
                    .push(format!("<animate> opacity \"{v}\" is not a number")),
            }
        } else if let Some((spec, _)) = svg_prop(&name).and_then(schema::prop) {
            r.vals.push((spec, V::Str(v)));
        } else {
            r.cascade.diagnostics.push(format!(
                "<animate attributeName=\"{name}\">: no SVG attribute {name} to animate"
            ));
        }
    }
    running
}

/// A `<use>`'s own placement: `translate(x y)`, then a symbol's view box
/// fitted into its width and height (`xMidYMid meet`, or `none`).
fn use_transform(r: &Resolved) -> Result<Affine, String> {
    let num = |p: &str| -> Result<f32, String> {
        r.str(p)
            .map_or(Ok(0.0), |v| crate::svg::length(v, 0.0))
            .map_err(|e| format!("{p}: {e}"))
    };
    let at = Affine([1.0, 0.0, 0.0, 1.0, num("svgX")?, num("svgY")?]);
    let Some(vb) = r.str("svgViewBox").and_then(view_box) else {
        return Ok(at);
    };
    let (w, h) = (num("svgWidth")?, num("svgHeight")?);
    let (w, h) = (
        if w > 0.0 { w } else { vb[2] },
        if h > 0.0 { h } else { vb[3] },
    );
    Ok(at.then(&fit_box(
        vb,
        [0.0, 0.0, w, h],
        r.str("svgPreserveAspectRatio"),
    )))
}

/// The transform that fits view box `vb` into `into` by
/// `preserveAspectRatio` (`none`, else centred `meet`; `slice` covers).
fn fit_box(vb: [f32; 4], into: [f32; 4], par: Option<&str>) -> Affine {
    let (sx, sy) = (into[2] / vb[2], into[3] / vb[3]);
    let par = par.unwrap_or("xMidYMid meet");
    let (sx, sy) = if par.trim() == "none" {
        (sx, sy)
    } else if par.contains("slice") {
        (sx.max(sy), sx.max(sy))
    } else {
        (sx.min(sy), sx.min(sy))
    };
    let tx = into[0] + (into[2] - vb[2] * sx) / 2.0 - vb[0] * sx;
    let ty = into[1] + (into[3] - vb[3] * sy) / 2.0 - vb[1] * sy;
    Affine([sx, 0.0, 0.0, sy, tx, ty])
}

fn invert(a: &Affine) -> Option<Affine> {
    let [a, b, c, d, e, f] = a.0;
    let det = a * d - b * c;
    (det.abs() > 1e-12).then(|| {
        let (ia, ib, ic, id) = (d / det, -b / det, -c / det, a / det);
        Affine([ia, ib, ic, id, -(ia * e + ic * f), -(ib * e + id * f)])
    })
}

/// The definition `#id` of `doc` whose tag is one of `tags`.
fn def<'d>(doc: &'d UiDocument, id: &str, tags: &[&str]) -> Option<&'d DocNode> {
    doc.nodes
        .iter()
        .find(|n| n.id == id && n.selector_tag.as_deref().is_some_and(|t| tags.contains(&t)))
}

/// The id a `url(#id)` names.
fn url_id(v: &str) -> Option<&str> {
    v.trim()
        .strip_prefix("url(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('#')
}

/// A definition's shape: its tag, literal attributes and transform.
type DefShape = (String, Vec<(String, String)>, Affine);

/// The shapes of a definition (its children, and its groups' children),
/// each its tag, literal attributes and transform in the definition's
/// space.
fn def_shapes(
    doc: &UiDocument,
    node: &DocNode,
    t: Affine,
    out: &mut Vec<DefShape>,
) -> Result<(), String> {
    for &c in &node.children {
        let n = &doc.nodes[c];
        let Some(tag) = n.selector_tag.as_deref() else {
            continue;
        };
        let attrs = literal_attrs(n)?;
        let own = match attrs.iter().find(|(k, _)| k == "transform") {
            Some((_, v)) => t.then(&Affine::parse(v)?),
            None => t,
        };
        if tag == "g" {
            def_shapes(doc, n, own, out)?;
        } else if crate::doc::extern_element::SVG_SHAPES.contains(&tag) {
            out.push((tag.to_string(), attrs, own));
        }
    }
    Ok(())
}

/// A rect, circle or ellipse under an axis-aligned transform as a box
/// shape and its view rect (what og-render clips to natively); `None`
/// for any other shape or a turned one.
fn basic_box<'a>(
    tag: &str,
    get: &dyn Fn(&str) -> Option<&'a str>,
    local: &crate::svg::Path,
    t: &Affine,
) -> Option<(ShapeKind, [f32; 4])> {
    let [a, b, c, d, ..] = t.0;
    if b != 0.0 || c != 0.0 || !matches!(tag, "rect" | "circle" | "ellipse") {
        return None;
    }
    let [x, y, w, h] = bounds(local);
    let p = t.apply(Vec2::new(x, y));
    let q = t.apply(Vec2::new(x + w, y + h));
    let rect = [
        p.x.min(q.x),
        p.y.min(q.y),
        (q.x - p.x).abs(),
        (q.y - p.y).abs(),
    ];
    let shape = if tag == "rect" {
        let rx = get("rx")
            .or(get("ry"))
            .and_then(|v| crate::svg::length(v, w).ok());
        let r = rx.unwrap_or(0.0).min(w / 2.0).min(h / 2.0) * a.abs().min(d.abs());
        ShapeKind::Rect {
            radii: Some([r; 4]),
            corner: crate::paint::Corner::Round,
        }
    } else {
        ShapeKind::Ellipse
    };
    Some((shape, rect))
}

/// Paths' subpaths as one filled path (its contours), for a clip.
fn filled(parts: Vec<(Vec<[f32; 2]>, bool)>, fill_rule: crate::paint::FillRule) -> ShapeKind {
    ShapeKind::Path {
        points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
        closed: true,
        filled: true,
        curves: true,
        fill_rule,
        contours: parts.iter().map(|(p, _)| p.len() as u32).collect(),
        width: 0.0,
        cap: crate::paint::Cap::Butt,
        join: crate::paint::Join::Round,
    }
}

/// What a `clip-path` or `mask` reference cuts an element to, under its
/// transform `t`: the definition's shapes as one clip path, or (a mask of
/// one gradient-filled shape) a gradient mask.
enum Cut {
    /// Paths, cut by the path fill's coverage.
    Shape(ShapeKind),
    /// One rect or ellipse, clipped natively.
    Box(ShapeKind, [f32; 4]),
    Gradient(Gradient),
}

fn cut(doc: &UiDocument, r: &Resolved, t: &Affine, vb: [f32; 4]) -> Result<Option<Cut>, String> {
    let (reference, tags): (_, &[&str]) = match (r.str("svgClipPath"), r.str("svgMask")) {
        (Some(c), _) => (c, &["clipPath"]),
        (None, Some(m)) => (m, &["mask"]),
        (None, None) => return Ok(None),
    };
    let id = url_id(reference).ok_or_else(|| format!("\"{reference}\" is not url(#id)"))?;
    let node = def(doc, id, tags).ok_or_else(|| format!("url(#{id}) names no <{}>", tags[0]))?;
    let attrs = literal_attrs(node)?;
    if attrs.iter().any(|(k, v)| {
        matches!(k.as_str(), "clipPathUnits" | "maskContentUnits") && v == "objectBoundingBox"
    }) {
        return Err(format!(
            "#{id}: objectBoundingBox units are not supported; write user-space shapes"
        ));
    }
    let mut shapes = Vec::new();
    def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
    if tags[0] == "mask" && shapes.len() == 1 {
        let (tag, attrs, own) = &shapes[0];
        if let Some(gid) = attrs
            .iter()
            .find(|(k, _)| k == "fill")
            .and_then(|(_, v)| url_id(v))
        {
            let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
            let local = shape_path(tag, &get, vb)?.ok_or("the mask's shape is empty")?;
            let at = t.then(own);
            let mut g = gradient(doc, gid, bounds(&local), &at, 1.0)?;
            // A mask is luminance times alpha: white, as alpha.
            for stop in &mut g.stops {
                if let ColorRef::Literal(c) = &mut stop.color {
                    let [r, gr, b] = c.rgb.map(|v| f32::from(v) / 255.0);
                    let lum = 0.2125 * r + 0.7154 * gr + 0.0721 * b;
                    *c = Srgba {
                        rgb: [255; 3],
                        alpha: c.alpha * lum,
                    };
                }
            }
            return Ok(Some(Cut::Gradient(g)));
        }
    }
    if let [(tag, attrs, own)] = shapes.as_slice() {
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        if let Some(local) = shape_path(tag, &get, vb)?
            && let Some((shape, rect)) = basic_box(tag, &get, &local, &t.then(own))
        {
            return Ok(Some(Cut::Box(shape, rect)));
        }
    }
    let (rule, parts) = def_path(&shapes, tags[0] == "mask", id, vb, t)?;
    Ok(Some(Cut::Shape(filled(parts, rule))))
}

/// A clip's or mask's shapes as one path under `t`: its rule (`clip-rule`
/// of a clip's shapes, `fill-rule` of a mask's, when every shape takes it,
/// else nonzero) and its subpaths. The shapes are one path, so two
/// evenodd shapes that overlap cut a hole where they do; a mask's black or
/// unfilled shapes reveal nothing.
#[allow(clippy::type_complexity)]
fn def_path(
    shapes: &[DefShape],
    mask: bool,
    id: &str,
    vb: [f32; 4],
    t: &Affine,
) -> Result<(crate::paint::FillRule, Vec<(Vec<[f32; 2]>, bool)>), String> {
    let tags = [if mask { "mask" } else { "clipPath" }];
    let rule_attr = if tags[0] == "mask" {
        "fill-rule"
    } else {
        "clip-rule"
    };
    let rules: Vec<bool> = shapes
        .iter()
        .map(|(_, attrs, _)| {
            match attrs
                .iter()
                .find(|(k, _)| k == rule_attr)
                .map(|(_, v)| v.trim())
            {
                None | Some("nonzero") => Ok(false),
                Some("evenodd") => Ok(true),
                Some(v) => Err(format!("{rule_attr} \"{v}\" is nonzero or evenodd")),
            }
        })
        .collect::<Result<_, _>>()?;
    if rules.iter().any(|&e| e) && !rules.iter().all(|&e| e) {
        return Err(format!(
            "#{id}: its shapes mix nonzero and evenodd {rule_attr}; give them one rule"
        ));
    }
    let rule = if rules.first() == Some(&true) {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let mut parts = Vec::new();
    for (tag, attrs, own) in shapes {
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        // A mask's black or unfilled shapes reveal nothing.
        if tags[0] == "mask"
            && get("fill").is_some_and(|f| {
                f == "none" || crate::svg::color(f).is_ok_and(|c| c[0] + c[1] + c[2] < 0.05)
            })
        {
            continue;
        }
        if let Some(local) = shape_path(tag, &get, vb)? {
            parts.extend(subpaths(&local.transformed(&t.then(own))));
        }
    }
    Ok((rule, parts))
}

/// CSS `clip-path: url(#id)` on an element: the `<clipPath>`'s shapes as
/// one path in the reference box's user space (CSS px from its top left),
/// its rule, and whether they are fractions of the box
/// (`clipPathUnits="objectBoundingBox"`).
#[allow(clippy::type_complexity)]
pub(crate) fn css_clip(
    doc: &UiDocument,
    reference: &str,
) -> Result<(crate::paint::FillRule, Vec<(Vec<[f32; 2]>, bool)>, bool), String> {
    let id = url_id(reference).ok_or_else(|| format!("\"{reference}\" is not url(#id)"))?;
    let node =
        def(doc, id, &["clipPath"]).ok_or_else(|| format!("url(#{id}) names no <clipPath>"))?;
    let attrs = literal_attrs(node)?;
    let units = attrs
        .iter()
        .any(|(k, v)| k == "clipPathUnits" && v == "objectBoundingBox");
    let mut shapes = Vec::new();
    def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
    // A percentage is of the box: of 1 in its units, else of 100 px.
    let vb = if units {
        [0.0, 0.0, 1.0, 1.0]
    } else {
        [0.0, 0.0, 100.0, 100.0]
    };
    let (rule, parts) = def_path(&shapes, false, id, vb, &Affine::IDENTITY)?;
    Ok((rule, parts, units))
}

/// A `<filter>` and its descendants as the filter reader takes them.
fn filter_node<'d>(
    doc: &'d UiDocument,
    n: &'d DocNode,
) -> Result<crate::svg::filter::Node<'d>, String> {
    Ok(crate::svg::filter::Node {
        tag: n.selector_tag.as_deref().unwrap_or_default(),
        attrs: literal_attrs(n)?,
        children: n
            .children
            .iter()
            .map(|&c| filter_node(doc, &doc.nodes[c]))
            .collect::<Result<_, _>>()?,
    })
}

/// The `<filter>` `reference` (`url(#id)`) names; a user-unit percentage
/// is of `viewport`.
fn filter_def(
    doc: &UiDocument,
    reference: &str,
    viewport: [f32; 2],
    image: crate::svg::filter::FindImage,
) -> Result<crate::svg::filter::Filter, String> {
    let id = url_id(reference).ok_or_else(|| format!("\"{reference}\" is not url(#id)"))?;
    let node = def(doc, id, &["filter"]).ok_or_else(|| format!("url(#{id}) names no <filter>"))?;
    crate::svg::filter::parse(&filter_node(doc, node)?, viewport, image)
        .map_err(|e| format!("#{id}: {e}"))
}

/// CSS `filter: url(#id)` or `backdrop-filter: url(#id)` on a box: the
/// `<filter>` in the box's user space (CSS px from its border box's top
/// left; a percentage of user units is of 100 px).
pub(crate) fn css_filter(
    doc: &UiDocument,
    reference: &str,
    image: crate::svg::filter::FindImage,
) -> Result<std::sync::Arc<crate::paint::FilterUse>, String> {
    let vb = [0.0, 0.0, 100.0, 100.0];
    let filter = filter_def(doc, reference, [vb[2], vb[3]], image)?;
    Ok(std::sync::Arc::new(crate::paint::FilterUse {
        filter,
        svg: None,
    }))
}

/// A pattern's tiles under a shape of local bounds `bbox` and transform
/// `t`: for each of its shapes, all its tiles as one filled path, with
/// that shape's paint. At most 4096 tiles.
fn pattern(
    doc: &UiDocument,
    id: &str,
    bbox: [f32; 4],
    t: &Affine,
    vb: [f32; 4],
    alpha: f32,
) -> Result<Vec<(ShapeKind, Paint)>, String> {
    let node = def(doc, id, &["pattern"]).ok_or_else(|| format!("url(#{id}) names nothing"))?;
    let attrs = literal_attrs(node)?;
    let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
    let bbox_units = get("patternUnits") != Some("userSpaceOnUse");
    let len = |a: &str, whole: f32, offset: f32| -> Result<f32, String> {
        let Some(v) = get(a) else { return Ok(0.0) };
        Ok(if bbox_units {
            let f = match v.trim().strip_suffix('%') {
                Some(p) => p.trim().parse::<f32>().map_err(|e| e.to_string())? / 100.0,
                None => v.trim().parse::<f32>().map_err(|e| e.to_string())?,
            };
            offset + f * whole
        } else {
            crate::svg::length(v, whole)?
        })
    };
    let tile = [
        len("x", bbox[2], bbox[0])?,
        len("y", bbox[3], bbox[1])?,
        len("width", bbox[2], 0.0)?,
        len("height", bbox[3], 0.0)?,
    ];
    if tile[2] <= 0.0 || tile[3] <= 0.0 {
        return Ok(Vec::new());
    }
    if get("patternContentUnits") == Some("objectBoundingBox") {
        return Err(format!(
            "#{id}: patternContentUnits objectBoundingBox is not supported"
        ));
    }
    let pt = match get("patternTransform") {
        Some(v) => Affine::parse(v)?,
        None => Affine::IDENTITY,
    };
    let content = match get("viewBox").and_then(view_box) {
        Some(pvb) => fit_box(
            pvb,
            [0.0, 0.0, tile[2], tile[3]],
            get("preserveAspectRatio"),
        ),
        None => Affine::IDENTITY,
    };
    // The shape's bounds in pattern space, and the tiles over them.
    let inv = invert(&pt).ok_or("patternTransform is not invertible")?;
    let corners = [
        [bbox[0], bbox[1]],
        [bbox[0] + bbox[2], bbox[1]],
        [bbox[0], bbox[1] + bbox[3]],
        [bbox[0] + bbox[2], bbox[1] + bbox[3]],
    ]
    .map(|[x, y]| inv.apply(Vec2::new(x, y)));
    let lo = corners.iter().fold(Vec2::splat(f32::MAX), |a, c| a.min(*c));
    let hi = corners.iter().fold(Vec2::splat(f32::MIN), |a, c| a.max(*c));
    let i0 = ((lo.x - tile[0]) / tile[2]).floor() as i64;
    let i1 = ((hi.x - tile[0]) / tile[2]).ceil() as i64;
    let j0 = ((lo.y - tile[1]) / tile[3]).floor() as i64;
    let j1 = ((hi.y - tile[1]) / tile[3]).ceil() as i64;
    if (i1 - i0) * (j1 - j0) > 4096 {
        return Err(format!(
            "#{id}: more than 4096 tiles; make the pattern larger"
        ));
    }
    let mut shapes = Vec::new();
    def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
    let mut out = Vec::new();
    for (tag, attrs, own) in &shapes {
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        let mut style = Style::default();
        for attr in PRESENTATION {
            if let Some(v) = get(attr) {
                let parent = style.clone();
                style
                    .apply(attr, v, &parent)
                    .map_err(|e| format!("{attr}: {e}"))?;
            }
        }
        let Some(paint) = solid(&style.fill, style.fill_opacity * alpha) else {
            continue;
        };
        let Some(local) = shape_path(tag, &get, vb)? else {
            continue;
        };
        let mut parts = Vec::new();
        for i in i0..i1 {
            for j in j0..j1 {
                let at = Affine([
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    tile[0] + i as f32 * tile[2],
                    tile[1] + j as f32 * tile[3],
                ]);
                let to = t.then(&pt).then(&at).then(&content).then(own);
                parts.extend(subpaths(&local.transformed(&to)));
            }
        }
        out.push((filled(parts, crate::paint::FillRule::NonZero), paint));
    }
    Ok(out)
}

/// A solid paint (`None` for none, and for a reference).
fn solid(p: &PaintRef, alpha: f32) -> Option<Paint> {
    match p {
        PaintRef::Color(c) => Some(Paint::Solid(ColorRef::Literal(srgba(*c, alpha)))),
        PaintRef::Current => Some(Paint::Solid(ColorRef::Token {
            name: "ink".into(),
            alpha,
        })),
        PaintRef::Token(name) => Some(Paint::Solid(ColorRef::Token {
            name: crate::doc::css::camel(name),
            alpha,
        })),
        PaintRef::None | PaintRef::Url(_) => None,
    }
}

pub(crate) fn srgba(c: [f32; 4], alpha: f32) -> Srgba {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Srgba {
        rgb: [byte(c[0]), byte(c[1]), byte(c[2])],
        alpha: (c[3] * alpha).clamp(0.0, 1.0),
    }
}

/// A gradient node's literal attributes by their SVG names.
fn literal_attrs(n: &DocNode) -> Result<Vec<(String, String)>, String> {
    n.selector_attrs
        .iter()
        .filter(|(name, _)| name != "class" && !name.starts_with("class:"))
        .map(|(name, e)| match e {
            crate::bind::Expr::Lit(v) => Ok((name.clone(), crate::bind::text(v))),
            _ => Err(format!(
                "<{}> {name}: a gradient's attributes are literal",
                n.selector_tag.as_deref().unwrap_or("gradient")
            )),
        })
        .collect()
}

/// The gradient `#id` of `doc`, placed for a shape of local bounds `bbox`
/// under `t`, in view-box units.
fn gradient(
    doc: &UiDocument,
    id: &str,
    bbox: [f32; 4],
    t: &Affine,
    alpha: f32,
) -> Result<Gradient, String> {
    let find = |id: &str| {
        doc.nodes.iter().find(|n| {
            n.id == id
                && matches!(
                    n.selector_tag.as_deref(),
                    Some("linearGradient" | "radialGradient")
                )
        })
    };
    let first = find(id).ok_or_else(|| format!("url(#{id}) names no gradient"))?;
    let mut chain = vec![first];
    let mut attrs = vec![literal_attrs(first)?];
    while let Some(href) = attrs
        .last()
        .and_then(|a| a.iter().find(|(k, _)| k == "href" || k == "xlink:href"))
        .map(|(_, v)| v.clone())
    {
        let next = href
            .strip_prefix('#')
            .and_then(find)
            .ok_or_else(|| format!("gradient \"{id}\": href \"{href}\" names no gradient"))?;
        if chain.len() > 16 || chain.iter().any(|c| std::ptr::eq(*c, next)) {
            return Err(format!("gradient \"{id}\": an href loop"));
        }
        chain.push(next);
        attrs.push(literal_attrs(next)?);
    }
    let stops: Vec<Vec<(String, String)>> =
        chain
            .iter()
            .find(|n| !n.children.is_empty())
            .map_or(Ok(Vec::new()), |n| {
                n.children
                    .iter()
                    .map(|&c| literal_attrs(&doc.nodes[c]))
                    .collect()
            })?;
    let get = |k: &str| {
        attrs
            .iter()
            .find_map(|a| a.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()))
    };
    let stop_gets: Vec<_> = stops
        .iter()
        .map(|s| move |k: &str| s.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()))
        .collect();
    let radial = first.selector_tag.as_deref() == Some("radialGradient");
    let placed = GradientDef::new(id, radial, &get, &stop_gets)?.place(bbox, t, alpha)?;
    Ok(Gradient {
        kind: if placed.radial {
            GradientKind::Radial
        } else {
            GradientKind::Linear
        },
        from: placed.from.into(),
        to: placed.to.into(),
        stops: placed
            .stops
            .into_iter()
            .map(|(offset, c)| Stop {
                offset,
                color: ColorRef::Literal(srgba(c, 1.0)),
            })
            .collect(),
        spread: placed.spread,
        dither: true,
    })
}

/// A path's subpaths as `ShapeKind::Path` points: each its start, then
/// three points per cubic.
pub(crate) fn subpaths(p: &crate::svg::Path) -> Vec<(Vec<[f32; 2]>, bool)> {
    p.subpaths
        .iter()
        .filter(|(_, cubics, _)| !cubics.is_empty())
        .map(|(start, cubics, closed)| {
            let mut pts: Vec<[f32; 2]> = vec![(*start).into()];
            pts.extend(cubics.iter().flatten().map(|v: &Vec2| <[f32; 2]>::from(*v)));
            (pts, *closed)
        })
        .collect()
}

/// A shape's draw, the tiles of a pattern fill drawn inside it, the box
/// they are cut to (`None`: the shape is a path), and its markers' draws.
struct ShapeOut {
    draw: ElementDraw,
    tiles: Vec<(ShapeKind, Paint)>,
    tile_box: Option<(ShapeKind, Option<[f32; 4]>)>,
    markers: Vec<ElementDraw>,
}

fn cap_of(c: crate::svg::Cap) -> crate::paint::Cap {
    match c {
        crate::svg::Cap::Butt => crate::paint::Cap::Butt,
        crate::svg::Cap::Round => crate::paint::Cap::Round,
        crate::svg::Cap::Square => crate::paint::Cap::Square,
    }
}

/// A path (in the `<svg>`'s user space) filled with `fill` and stroked
/// with `stroke` by `style`: `scale` is its transform's (the stroke's
/// width scales by it), `per` the user units of one dash unit. A dash
/// pattern of one dash and one gap is og-render's; a longer one is cut
/// into its dashes here.
#[allow(clippy::too_many_arguments)]
fn painted(
    path: &crate::svg::Path,
    style: &Style,
    scale: f32,
    per: f32,
    fill: Option<Paint>,
    stroke: Option<Paint>,
    vb: [f32; 4],
) -> ElementDraw {
    let parts = subpaths(path);
    let rule = if style.even_odd {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let cap = cap_of(style.cap);
    let join = match style.join {
        crate::svg::Join::Miter => crate::paint::Join::Miter(style.miter_limit),
        crate::svg::Join::Round => crate::paint::Join::Round,
        crate::svg::Join::Bevel => crate::paint::Join::Bevel,
    };
    let width = style.stroke_width * scale;
    let mut draw = ElementDraw {
        view_box: Some(vb),
        shape: ShapeKind::Path {
            points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
            closed: true,
            filled: true,
            curves: true,
            fill_rule: rule,
            contours: parts.iter().map(|(p, _)| p.len() as u32).collect(),
            width,
            cap,
            join,
        },
        ..ElementDraw::default()
    };
    if let Some(fill) = fill {
        draw.fills.push(fill);
    }
    let Some(stroke) = stroke.filter(|_| width > 0.0) else {
        return draw;
    };
    let outline =
        |points: Vec<[f32; 2]>, closed: bool, curves: bool, dash: [f32; 2], offset: f32| {
            (
                ShapeKind::Path {
                    points,
                    closed,
                    filled: false,
                    curves,
                    fill_rule: rule,
                    contours: Vec::new(),
                    width,
                    cap,
                    join,
                },
                Stroke {
                    width,
                    align: StrokeAlign::Center,
                    dash,
                    dash_offset: offset,
                    dash_cap: cap,
                    paint: stroke.clone(),
                },
            )
        };
    match style.dashes.as_slice() {
        [] => {
            for (points, closed) in parts {
                draw.outlines
                    .push(outline(points, closed, true, [0.0; 2], 0.0));
            }
        }
        // Zero-length dashes (an empty `pathLength` meter's "0 1"): butt
        // caps draw nothing; og-render reads a zero dash as solid.
        [d, g] if *d <= 0.0 && *g > 0.0 && style.cap == crate::svg::Cap::Butt => {}
        [d, g] if *d > 0.0 || *g <= 0.0 => {
            for (points, closed) in parts {
                draw.outlines.push(outline(
                    points,
                    closed,
                    true,
                    [d * per, g * per],
                    style.dash_offset * per,
                ));
            }
        }
        more => {
            // Every dash its own open polyline, its caps the stroke's.
            let pattern: Vec<f32> = more.iter().map(|d| d * per).collect();
            let tolerance = vb[2].max(vb[3]) / 4000.0;
            for dash in path.dashed(&pattern, style.dash_offset * per, tolerance) {
                let points = dash.into_iter().map(<[f32; 2]>::from).collect();
                draw.outlines
                    .push(outline(points, false, false, [0.0; 2], 0.0));
            }
        }
    }
    draw
}

/// A presentation value as SVG writes it, a marker's `context-fill` and
/// `context-stroke` (SVG 2) taken from the shape that draws it.
fn style_of(attrs: &[(String, String)], context: Option<&Style>) -> Result<Style, String> {
    let mut style = Style::default();
    for attr in PRESENTATION {
        let Some((_, v)) = attrs.iter().find(|(k, _)| k == attr) else {
            continue;
        };
        let parent = style.clone();
        match (v.trim(), context) {
            ("context-fill", Some(c)) => style.apply(attr, "none", &parent).map(|_| {
                if *attr == "fill" {
                    style.fill = c.fill.clone();
                } else {
                    style.stroke = c.fill.clone();
                }
            }),
            ("context-stroke", Some(c)) => style.apply(attr, "none", &parent).map(|_| {
                if *attr == "fill" {
                    style.fill = c.stroke.clone();
                } else {
                    style.stroke = c.stroke.clone();
                }
            }),
            (v, _) => style.apply(attr, v, &parent),
        }
        .map_err(|e| format!("{attr}: {e}"))?;
    }
    Ok(style)
}

/// Which vertices a marker property draws at.
#[derive(Clone, Copy, PartialEq)]
enum Which {
    Start,
    Mid,
    End,
}

/// The draws of a shape's `marker-start`, `marker-mid` and `marker-end`:
/// each `<marker>`'s shapes at the vertices, turned by its `orient`, its
/// `refX refY` on the vertex, scaled by the stroke's width (markerUnits
/// strokeWidth) and its view box fitted into `markerWidth markerHeight`.
fn markers(
    doc: &UiDocument,
    value: &dyn Fn(&str) -> Option<String>,
    local: &crate::svg::Path,
    t: &Affine,
    style: &Style,
    vb: [f32; 4],
) -> Result<Vec<ElementDraw>, String> {
    let verts = local.vertices();
    let n = verts.len();
    let mut out = Vec::new();
    for (attr, which) in [
        ("marker-start", Which::Start),
        ("marker-mid", Which::Mid),
        ("marker-end", Which::End),
    ] {
        let Some(v) = value(attr).filter(|v| v.trim() != "none") else {
            continue;
        };
        let id = url_id(&v).ok_or_else(|| format!("{attr} \"{v}\" is url(#id) or none"))?;
        let node = def(doc, id, &["marker"])
            .ok_or_else(|| format!("{attr}: url(#{id}) names no <marker>"))?;
        let attrs = literal_attrs(node)?;
        let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
        let num = |a: &str, default: f32| -> Result<f32, String> {
            get(a)
                .map_or(Ok(default), |v| crate::svg::length(v, 1.0))
                .map_err(|e| format!("#{id} {a}: {e}"))
        };
        let (mw, mh) = (num("markerWidth", 3.0)?, num("markerHeight", 3.0)?);
        if mw <= 0.0 || mh <= 0.0 {
            continue;
        }
        let scale = match get("markerUnits").map(str::trim) {
            None | Some("strokeWidth") => style.stroke_width,
            Some("userSpaceOnUse") => 1.0,
            Some(u) => {
                return Err(format!(
                    "#{id} markerUnits \"{u}\" is strokeWidth or userSpaceOnUse"
                ));
            }
        };
        let content = match get("viewBox").and_then(view_box) {
            Some(mvb) => fit_box(mvb, [0.0, 0.0, mw, mh], get("preserveAspectRatio")),
            None => Affine::IDENTITY,
        };
        let refp = content.apply(Vec2::new(num("refX", 0.0)?, num("refY", 0.0)?));
        let orient = get("orient").unwrap_or("0").trim();
        let fixed = match orient {
            "auto" | "auto-start-reverse" => None,
            o => Some(
                o.trim_end_matches("deg")
                    .trim()
                    .parse::<f32>()
                    .map_err(|_| {
                        format!("#{id} orient \"{o}\" is auto, auto-start-reverse or degrees")
                    })?
                    .to_radians(),
            ),
        };
        let mut shapes = Vec::new();
        def_shapes(doc, node, Affine::IDENTITY, &mut shapes)?;
        let at: Vec<usize> = match which {
            Which::Start => (n > 0).then_some(0).into_iter().collect(),
            Which::End => n.checked_sub(1).into_iter().collect(),
            Which::Mid => (1..n.saturating_sub(1)).collect(),
        };
        for i in at {
            let (p, came, goes) = verts[i];
            let angle = fixed.unwrap_or_else(|| {
                let d = match (came, goes) {
                    (Some(a), Some(b)) => (a + b).try_normalize().unwrap_or(a),
                    (Some(a), None) => a,
                    (None, Some(b)) => b,
                    (None, None) => Vec2::X,
                };
                let a = d.y.atan2(d.x);
                if orient == "auto-start-reverse" && which == Which::Start {
                    a + std::f32::consts::PI
                } else {
                    a
                }
            });
            let (sin, cos) = angle.sin_cos();
            let m = t
                .then(&Affine([1.0, 0.0, 0.0, 1.0, p.x, p.y]))
                .then(&Affine([cos, sin, -sin, cos, 0.0, 0.0]))
                .then(&Affine([scale, 0.0, 0.0, scale, 0.0, 0.0]))
                .then(&Affine([1.0, 0.0, 0.0, 1.0, -refp.x, -refp.y]))
                .then(&content);
            for (tag, attrs, own) in &shapes {
                let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
                let Some(shape) = shape_path(tag, &get, vb)? else {
                    continue;
                };
                let s = style_of(attrs, Some(style))?;
                let at = m.then(own);
                let fill = if *tag == "line" {
                    None
                } else {
                    solid(&s.fill, s.fill_opacity)
                };
                let stroke = solid(&s.stroke, s.stroke_opacity);
                out.push(painted(
                    &shape.transformed(&at),
                    &s,
                    at.scale(),
                    at.scale(),
                    fill,
                    stroke,
                    vb,
                ));
            }
        }
    }
    Ok(out)
}

fn shape_draw(doc: &UiDocument, tag: &str, r: &Resolved, vb: [f32; 4]) -> Result<ShapeOut, String> {
    let computed = &r.cascade.computed;
    let value = |name: &str| computed.get(&key(name)).cloned().flatten();
    let mut style = Style::default();
    for attr in PRESENTATION {
        if let Some(v) = value(attr) {
            let parent = style.clone();
            style
                .apply(attr, &v, &parent)
                .map_err(|e| format!("{attr}: {e}"))?;
        }
    }
    let t = computed
        .get(TRANSFORM)
        .cloned()
        .flatten()
        .and_then(|t| Affine::parse(&t).ok())
        .unwrap_or(Affine::IDENTITY);
    let get = |attr: &str| svg_prop(attr).and_then(|p| r.str(p));
    let Some(local) = shape_path(tag, &get, vb)? else {
        return Ok(ShapeOut {
            draw: ElementDraw {
                view_box: Some(vb),
                ..ElementDraw::default()
            },
            tiles: Vec::new(),
            tile_box: None,
            markers: Vec::new(),
        });
    };
    let bbox = bounds(&local);
    let path = local.transformed(&t);
    let paint = |p: &PaintRef, alpha: f32| -> Result<Option<Paint>, String> {
        Ok(match p {
            PaintRef::Url(id) if def(doc, id, &["pattern"]).is_some() => None,
            PaintRef::Url(id) => Some(Paint::Gradient(gradient(doc, id, bbox, &t, alpha)?)),
            other => solid(other, alpha),
        })
    };
    let tiles = match &style.fill {
        PaintRef::Url(id) if tag != "line" && def(doc, id, &["pattern"]).is_some() => {
            pattern(doc, id, bbox, &t, vb, style.fill_opacity)?
        }
        _ => Vec::new(),
    };
    // A line has no inside; a polyline fills as SVG does, closed.
    let fill = if tag == "line" {
        None
    } else {
        paint(&style.fill, style.fill_opacity)?
    };
    let stroke = if style.stroke_width * t.scale() > 0.0 {
        paint(&style.stroke, style.stroke_opacity)?
    } else {
        None
    };
    // Dashes measure the outline in user units, or in `pathLength`.
    let per = match get("pathLength").map(|v| v.trim().parse::<f32>()) {
        Some(Ok(l)) if l > 0.0 => local.length(vb[2].max(vb[3]) / 2000.0) / l,
        Some(_) => return Err("pathLength is a positive number".into()),
        None => 1.0,
    } * t.scale();
    let draw = painted(&path, &style, t.scale(), per, fill, stroke, vb);
    // The tiles are cut to the shape: a rect, circle or ellipse as a box,
    // any other by its own path.
    let tile_box = if tiles.is_empty() {
        None
    } else {
        Some(
            basic_box(tag, &get, &local, &t)
                .map_or((draw.shape.clone(), None), |(s, r)| (s, Some(r))),
        )
    };
    let markers = if tag == "image" {
        Vec::new()
    } else {
        markers(doc, &value, &local, &t, &style, vb)?
    };
    Ok(ShapeOut {
        draw,
        tiles,
        tile_box,
        markers,
    })
}

/// Where `dominant-baseline` puts `y` in a line box whose top is 0 and
/// whose height is the font's extents (`ascender` to `descender`, em, at
/// `size` CSS px): CSS px from the box's top.
fn baseline_shift(base: &str, ascender: f32, descender: f32, size: f32) -> Result<f32, String> {
    let alphabetic = ascender * size;
    Ok(match base {
        "auto" | "alphabetic" | "" => alphabetic,
        // Half the x-height (0.5 em) over the baseline, as browsers take
        // it without the font's OS/2 table.
        "middle" => alphabetic - 0.25 * size,
        "central" => (ascender - descender) * size / 2.0,
        "hanging" => alphabetic - 0.8 * size,
        "mathematical" => alphabetic - 0.5 * size,
        "text-before-edge" | "text-top" => 0.0,
        "text-after-edge" | "text-bottom" | "ideographic" => (ascender - descender) * size,
        other => {
            return Err(format!(
                "dominant-baseline \"{other}\" is auto, alphabetic, middle, central, hanging, mathematical, ideographic, text-before-edge or text-after-edge"
            ));
        }
    })
}

/// A `<text>`'s `<textPath>` as og-ui's text on a path: the path `href`
/// names (a `<path>` or any basic shape, its own transform applied) or its
/// own `path` data, under the text's transform, in the `<svg>`'s user
/// space from the view box's origin. `None`: the text is not on a path.
fn text_path(
    doc: &UiDocument,
    r: &Resolved,
    t: &Affine,
    vb: [f32; 4],
) -> Result<Option<crate::text_path::TextPath>, String> {
    let (local, length_scale) = if let Some(d) = r.str("svgPath") {
        (
            crate::svg::Path::parse(d).map_err(|e| format!("<textPath> path: {e}"))?,
            1.0,
        )
    } else if let Some(href) = r.str("svgTextPathHref") {
        shape_by_href(doc, href, vb).map_err(|e| format!("<textPath {e}"))?
    } else {
        return Ok(None);
    };
    let path = local.transformed(t);
    let tolerance = vb[2].max(vb[3]) / 4000.0;
    let origin = Vec2::new(vb[0], vb[1]);
    let subs: Vec<Vec<Vec2>> = path
        .flatten(tolerance)
        .into_iter()
        .map(|(mut pts, closed)| {
            if closed && let Some(&p) = pts.first() {
                pts.push(p);
            }
            pts.into_iter().map(|p| p - origin).collect()
        })
        .collect();
    let (offset, percent) = match r.str("svgStartOffset").map(str::trim) {
        None => (0.0, false),
        Some(v) => match v.strip_suffix('%') {
            Some(p) => (
                p.trim()
                    .parse::<f32>()
                    .map_err(|_| format!("startOffset \"{v}\""))?
                    / 100.0,
                true,
            ),
            None => (
                crate::svg::length(v, 0.0)? * length_scale * t.scale(),
                false,
            ),
        },
    };
    let reverse = match r.str("svgSide").map(str::trim) {
        None | Some("left") => false,
        Some("right") => true,
        Some(v) => return Err(format!("<textPath> side \"{v}\" is left or right")),
    };
    match r.str("svgMethod").map(str::trim) {
        None | Some("align" | "stretch") => {}
        Some(v) => return Err(format!("<textPath> method \"{v}\" is align or stretch")),
    }
    match r.str("svgSpacing").map(str::trim) {
        None | Some("exact" | "auto") => {}
        Some(v) => return Err(format!("<textPath> spacing \"{v}\" is exact or auto")),
    }
    Ok(Some(crate::text_path::TextPath {
        shape: crate::text_path::PathShape::Data(std::sync::Arc::new(subs)),
        offset,
        percent,
        reverse,
    }))
}

/// Where an element's element images are drawn: an SVG element's user
/// space (its map into the view box, and its bounding box there), or a
/// CSS box's (CSS px from its border box's top left).
#[derive(Clone, Copy)]
pub(super) enum ImageSpace {
    Svg(Affine, [f32; 4]),
    Css,
}

impl Ui {
    /// The elements the `feImage`s of the open element's filters name
    /// (`uses`, numbered as [`crate::paint::filter_uses`] numbers them),
    /// each built as `<use>` would draw it (Filter Effects 1 §15.17): in
    /// the filtered element's user space, moved by its `feImage`'s `x` and
    /// `y`, inheriting from the filtered element. Each is a child of the
    /// open element under an anonymous box whose draw is its element image
    /// (`ElementDraw::element_image`), so it draws everything the element
    /// draws (gradients, patterns, text, images, groups, `<use>`) into a
    /// layer only the filter reads, takes no input and is in no tree a
    /// reader walks. `of` is the filtered element's node, `computed` its
    /// computed values.
    pub(super) fn build_element_images(
        &mut self,
        b: &Build<'_>,
        of: &DocNode,
        computed: &crate::doc::variables::Computed,
        uses: &[&crate::paint::FilterUse],
        space: ImageSpace,
    ) {
        if self.docs.element_images >= 4 {
            return;
        }
        let doc = b.doc.clone();
        let Some(of) = doc.nodes.iter().position(|d| std::ptr::eq(d, of)) else {
            return;
        };
        let mut n = 0u32;
        for u in uses {
            for (k, id) in u.filter.elements.iter().enumerate() {
                let at = n;
                n += 1;
                let Some(target) = doc.nodes.iter().position(|d| {
                    d.id == *id
                        && matches!(d.kind, NodeKind::Element(k) if KINDS.contains(&k) && k != "svgDef")
                }) else {
                    // A missing element: the filter reads nothing there.
                    continue;
                };
                let mut up = of;
                let mut inside = false;
                while up != crate::doc::NONE {
                    inside |= up == target;
                    up = doc.nodes[up].parent;
                }
                if inside {
                    self.warnings.push(format!(
                        "{}:{}: <feImage href=\"#{id}\"> draws an element the filter is on or inside",
                        doc.source, doc.nodes[of].line
                    ));
                    continue;
                }
                // Its feImage's x and y, in the filter's primitive units.
                let layer = crate::svg::filter::element_layer(k as u32);
                let sub = u
                    .filter
                    .primitives
                    .iter()
                    .find(
                        |p| matches!(p.op, og_render::UiFilterOp::Layer { layer: l } if l == layer),
                    )
                    .map_or([f32::NAN; 4], |p| p.subregion);
                let bbox = match space {
                    ImageSpace::Svg(_, bbox) => bbox,
                    ImageSpace::Css => [0.0; 4],
                };
                // (user units, fraction of the box) per axis.
                let shift = |v: f32, axis: usize| match (v.is_nan(), u.filter.primitive_bbox) {
                    (true, _) => (0.0, 0.0),
                    (false, true) => (bbox[axis], v),
                    (false, false) => (v, 0.0),
                };
                let (sx, sy) = (shift(sub[0], 0), shift(sub[1], 1));
                let mut inherited = computed.clone();
                self.open(
                    format!("~image{at}"),
                    match space {
                        ImageSpace::Svg(..) => Element {
                            width: Size::grow(),
                            height: Size::grow(),
                            ..Element::default()
                        },
                        ImageSpace::Css => Element {
                            width: Size::px(1.0),
                            height: Size::px(1.0),
                            ..Element::default()
                        },
                    },
                );
                let i = self.parent();
                let p = &mut self.nodes[i].props;
                p.position = crate::element::Position::Absolute;
                p.inert = true;
                p.interact.pointer_events = crate::element::PointerEvents::None;
                match space {
                    ImageSpace::Svg(t, _) => {
                        // Over the whole `<svg>`, as its shapes are.
                        p.insets = crate::element::Edges {
                            start: Some(0.0),
                            top: Some(0.0),
                            end: Some(0.0),
                            bottom: Some(0.0),
                            ..Default::default()
                        };
                        let to = Affine([
                            1.0,
                            0.0,
                            0.0,
                            1.0,
                            sx.0 + sx.1 * bbox[2],
                            sy.0 + sy.1 * bbox[3],
                        ]);
                        inherited.insert(TRANSFORM.into(), Some(matrix(&t.then(&to))));
                    }
                    ImageSpace::Css => {
                        // A one-px view box at the box's top left (moved
                        // by the feImage): one user unit is a CSS px.
                        use crate::layout::anchor::{AnchorSpec, Inset};
                        let inset = |(px, q): (f32, f32)| Some(Inset::Percent(q, px));
                        p.anchor = Some(Box::new(AnchorSpec {
                            base: crate::layout::anchor::Placement {
                                insets: [inset(sy), None, None, inset(sx)],
                                ..Default::default()
                            },
                            ..Default::default()
                        }));
                        inherited.insert(VIEW_BOX.into(), Some("0 0 1 1".into()));
                        inherited.insert(TRANSFORM.into(), Some(matrix(&Affine::IDENTITY)));
                    }
                }
                self.set_draw(ElementDraw {
                    element_image: Some(at),
                    ..ElementDraw::default()
                });
                let mut ib = Build {
                    doc: doc.clone(),
                    mount: b.mount.clone(),
                    vm: b.vm,
                    env: b.env,
                    locals: b.locals.clone(),
                    row_key: b.row_key.clone(),
                    inherited: std::sync::Arc::new(inherited),
                    events: Vec::new(),
                    writes: Vec::new(),
                    cached: None,
                    recording: Default::default(),
                };
                self.docs.element_images += 1;
                self.build_node(&mut ib, target, None);
                self.docs.element_images -= 1;
                self.close();
            }
        }
    }

    /// An SVG `<text>`: one line shaped as any label, at its view-box size,
    /// placed and scaled by layout through [`crate::element::ViewPlace`].
    #[allow(clippy::too_many_arguments)]
    fn build_svg_text(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: &str,
        key: u64,
        vb: [f32; 4],
    ) {
        let computed = r.cascade.computed.clone();
        let value = |attr: &str| computed.get(&self::key(attr)).cloned().flatten();
        let mut warnings = Vec::new();
        let mut style = Style::default();
        for attr in PRESENTATION {
            if let Some(v) = value(attr) {
                let parent = style.clone();
                if let Err(e) = style.apply(attr, &v, &parent) {
                    warnings.push(format!("{attr}: {e}"));
                }
            }
        }
        let num = |attr: &str, default: f32, warnings: &mut Vec<String>| match value(attr)
            .as_deref()
            .or_else(|| svg_prop(attr).and_then(|p| r.str(p)))
            .map(|v| v.trim().trim_end_matches("px").parse::<f32>())
        {
            Some(Ok(v)) => v,
            Some(Err(_)) => {
                warnings.push(format!("{attr} is a number of user units"));
                default
            }
            None => default,
        };
        let own = |attr: &str, warnings: &mut Vec<String>| {
            svg_prop(attr)
                .and_then(|p| r.str(p))
                .map_or(Ok(0.0), |v| v.trim().trim_end_matches("px").parse::<f32>())
                .unwrap_or_else(|_| {
                    warnings.push(format!("{attr} is a number of user units"));
                    0.0
                })
        };
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let font_size = num("font-size", 16.0, &mut warnings).max(0.0);
        // dx and dy: user units or em, a list summed (a line's dy is the
        // sum of the dys since its y).
        let shift = |attr: &str, warnings: &mut Vec<String>| -> f32 {
            let Some(v) = svg_prop(attr).and_then(|p| r.str(p)) else {
                return 0.0;
            };
            v.split([' ', ','])
                .filter(|t| !t.is_empty())
                .map(|t| match t.strip_suffix("em") {
                    Some(n) => n.parse::<f32>().map(|n| n * font_size),
                    None => t.trim_end_matches("px").parse::<f32>(),
                })
                .sum::<Result<f32, _>>()
                .unwrap_or_else(|_| {
                    warnings.push(format!("{attr} \"{v}\" is user units or em"));
                    0.0
                })
        };
        let x = own("x", &mut warnings) + shift("dx", &mut warnings);
        let y = own("y", &mut warnings) + shift("dy", &mut warnings);
        let at = t.apply(og_core::glam::Vec2::new(x, y));
        let size = font_size * t.scale();
        let weight = match value("font-weight").as_deref().map(str::trim) {
            None | Some("normal") => 0,
            Some("bold" | "bolder") => 700,
            Some(v) => v.parse::<u16>().unwrap_or_else(|_| {
                warnings.push(format!("font-weight \"{v}\" is normal, bold or 100–900"));
                0
            }),
        };
        let italic = matches!(
            value("font-style").as_deref().map(str::trim),
            Some("italic" | "oblique")
        );
        let text = r.str("text").unwrap_or_default().to_string();
        let font = r.label_desc(&self.theme, &self.fonts, false).font;
        let (ascender, descender) = {
            let m = &self.fonts.for_styled(font, weight, italic, &text).0.metrics;
            m.extents(&text)
        };
        let anchor_x = match value("text-anchor").as_deref().map(str::trim) {
            None | Some("start") => 0.0,
            Some("middle") => 0.5,
            Some("end") => 1.0,
            Some(v) => {
                warnings.push(format!("text-anchor \"{v}\" is start, middle or end"));
                0.0
            }
        };
        let base = value("dominant-baseline").unwrap_or_default();
        let anchor_y = baseline_shift(base.trim(), ascender, descender, size).unwrap_or_else(|e| {
            warnings.push(e);
            ascender * size
        });
        let mut step = self.theme.type_ramp.body;
        step.size = size;
        // One line exactly the font's extents tall: the baseline sits at
        // the ascender, so `dominant-baseline` is measured from its top.
        step.line_height = (ascender - descender) * size;
        step.cjk_line_height = step.line_height;
        if weight > 0 {
            step.weight = weight;
        }
        let color = solid(&style.fill, style.fill_opacity).and_then(|p| match p {
            Paint::Solid(c) => c.srgb(&self.theme),
            _ => None,
        });
        // `fill="none"`: transparent, still laid out and named.
        let color = color.or(Some(Srgba {
            rgb: [0; 3],
            alpha: 0.0,
        }));
        // Lines a kept line break starts align on the anchor too.
        let align = match anchor_x {
            0.5 => crate::layout::TextAlign::Center,
            1.0 => crate::layout::TextAlign::End,
            _ => crate::layout::TextAlign::Start,
        };
        let desc = crate::ui::LabelDesc {
            step: Some(step),
            color,
            nowrap: true,
            align,
            font,
            ..Default::default()
        };
        let text_path = match text_path(&b.doc, r, &t, vb) {
            Ok(tp) => tp,
            Err(e) => {
                warnings.push(e);
                None
            }
        };
        let width = desc.width;
        // A whole bold or italic text is one span over it (bold synthesised
        // where the role has no such face, as a `<tspan>`'s is).
        let bold = weight >= 600;
        let whole;
        let spans = match r.get("spans") {
            Some(V::Spans(Ok(spans))) => Some(spans.as_slice()),
            _ if italic || bold => {
                whole = [crate::TextSpan {
                    name: "svgWhole".into(),
                    position: 0,
                    length: text.chars().count() as u32,
                    properties: Default::default(),
                }];
                Some(&whole[..])
            }
            _ => None,
        };
        match spans {
            Some(spans) => {
                self.rich_text_spans(
                    id,
                    &text,
                    spans,
                    crate::RichTextDesc {
                        step: desc.step,
                        color: desc.color,
                        align: desc.align,
                        nowrap: true,
                        width: desc.width,
                        font,
                    },
                    |span| {
                        let mut s = r
                            .span_styles
                            .iter()
                            .find(|(attribute, _)| attribute == span)
                            .map(|(_, style)| style.clone())
                            .unwrap_or_default();
                        if italic {
                            s.italic.get_or_insert(true);
                        }
                        if bold {
                            s.bold.get_or_insert(true);
                        }
                        s
                    },
                );
            }
            None => self.label(id, &text, desc),
        }
        let i = self.last_closed;
        // Its anchor places it, so it keeps its text's width (a centred
        // label in a column would take the column's).
        self.nodes[i].el.width = width;
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        p.position = crate::element::Position::Absolute;
        p.view_place = Some(match &text_path {
            // On a path: its box from the view box's origin, the path in
            // it (scaled with the view box as the shapes are).
            Some(_) => crate::element::ViewPlace {
                view_box: vb,
                at: [vb[0], vb[1]],
                anchor_x: 0.0,
                anchor_y: 0.0,
            },
            None => crate::element::ViewPlace {
                view_box: vb,
                at: [at.x, at.y],
                anchor_x,
                anchor_y,
            },
        });
        p.text_path = text_path;
        self.nodes[i].props = p;
        for e in warnings {
            let tag = n.selector_tag.as_deref().unwrap_or("text");
            self.warnings.push(format!(
                "{}:{} ({path}): <{tag}>: {e}",
                b.doc.source, n.line
            ));
        }
        self.fire(b, n, key, path, crate::ui::Response::default());
    }

    /// An `<image href x y width height preserveAspectRatio>`: an atlas
    /// icon by name, or a render texture by handle, filling its rect.
    /// An SVG element's `filter` reference: the `<filter>` in its user
    /// space (`t` takes it into the view box) and its bounding box there:
    /// a shape's geometry, a group's shapes' (literal ones).
    #[allow(clippy::too_many_arguments)]
    fn svg_filter(
        &self,
        b: &Build<'_>,
        n: &DocNode,
        kind: &str,
        r: &Resolved,
        reference: &str,
        t: &Affine,
        vb: [f32; 4],
    ) -> Result<std::sync::Arc<crate::paint::FilterUse>, String> {
        let image = |href: &str| -> Result<(og_render::TextureHandle, [f32; 4]), String> {
            match href.parse::<f64>() {
                Ok(h) => b
                    .env
                    .texture(h)
                    .map(|t| (t, [0.0, 0.0, 1.0, 1.0]))
                    .ok_or_else(|| format!("href {h} names no texture")),
                Err(_) => match self.icons.find(href) {
                    Some(crate::ui::IconCell::Image(texture, uv)) => Ok((texture, uv)),
                    _ => Err(format!("href \"{href}\" names no atlas image")),
                },
            }
        };
        let filter = filter_def(&b.doc, reference, [vb[2], vb[3]], &image)?;
        let union = |a: [f32; 4], b: [f32; 4]| {
            if a[2] <= 0.0 && a[3] <= 0.0 {
                return b;
            }
            let x = a[0].min(b[0]);
            let y = a[1].min(b[1]);
            [
                x,
                y,
                (a[0] + a[2]).max(b[0] + b[2]) - x,
                (a[1] + a[3]).max(b[1] + b[3]) - y,
            ]
        };
        let mut bbox = [0.0; 4];
        let tag = n.selector_tag.as_deref().unwrap_or("path");
        if kind == "svgShape" {
            let get = |attr: &str| svg_prop(attr).and_then(|p| r.str(p));
            if tag == "image" {
                let len =
                    |a: &str, whole: f32| get(a).map_or(Ok(0.0), |v| crate::svg::length(v, whole));
                bbox = [
                    len("x", vb[2])?,
                    len("y", vb[3])?,
                    len("width", vb[2])?,
                    len("height", vb[3])?,
                ];
            } else if let Some(local) = shape_path(tag, &get, vb)? {
                bbox = bounds(&local);
            }
        } else {
            let mut shapes = Vec::new();
            def_shapes(&b.doc, n, Affine::IDENTITY, &mut shapes)?;
            for (tag, attrs, own) in &shapes {
                let get = |a: &str| attrs.iter().find(|(k, _)| k == a).map(|(_, v)| v.as_str());
                if let Some(local) = shape_path(tag, &get, vb)? {
                    bbox = union(bbox, bounds(&local.transformed(own)));
                }
            }
        }
        Ok(std::sync::Arc::new(crate::paint::FilterUse {
            filter,
            svg: Some((*t, bbox)),
        }))
    }

    fn image_draw(
        &self,
        b: &Build<'_>,
        r: &Resolved,
        t: &Affine,
        vb: [f32; 4],
    ) -> Result<Option<ElementDraw>, String> {
        let get = |p: &str| r.str(p);
        let len = |p: &str, whole: f32| -> Result<f32, String> {
            get(p)
                .map_or(Ok(0.0), |v| crate::svg::length(v, whole))
                .map_err(|e| format!("{p}: {e}"))
        };
        let (w, h) = (len("svgWidth", vb[2])?, len("svgHeight", vb[3])?);
        if w <= 0.0 || h <= 0.0 {
            return Ok(None);
        }
        let [_, bb, c, _, ..] = t.0;
        if bb != 0.0 || c != 0.0 {
            return Err("an <image> takes no turn or skew".into());
        }
        let (x, y) = (len("svgX", vb[2])?, len("svgY", vb[3])?);
        let p = t.apply(Vec2::new(x, y));
        let q = t.apply(Vec2::new(x + w, y + h));
        let rect = [
            p.x.min(q.x),
            p.y.min(q.y),
            (q.x - p.x).abs(),
            (q.y - p.y).abs(),
        ];
        let href = get("svgHref").unwrap_or_default().trim();
        let (texture, uv) = match href.parse::<f64>() {
            Ok(h) => (
                b.env
                    .texture(h)
                    .ok_or_else(|| format!("href {h} names no texture"))?,
                [0.0, 0.0, 1.0, 1.0],
            ),
            Err(_) => match self.icons.find(href) {
                Some(crate::ui::IconCell::Image(texture, uv)) => (texture, uv),
                Some(_) => {
                    return Err(format!(
                        "\"{href}\" is a glyph icon, not an image: write <og-icon icon=\"{href}\"> beside the svg"
                    ));
                }
                None => return Err(format!("href \"{href}\" names no atlas image")),
            },
        };
        let fit = match get("svgPreserveAspectRatio").map(str::trim) {
            Some("none") => og_render::UiFit::Fill,
            Some(v) if v.contains("slice") => og_render::UiFit::Cover,
            _ => og_render::UiFit::Contain,
        };
        Ok(Some(ElementDraw {
            shape: ShapeKind::Rect {
                radii: Some([0.0; 4]),
                corner: crate::paint::Corner::Round,
            },
            fills: vec![Paint::Image(crate::paint::ImagePaint {
                texture,
                uv,
                size: [
                    (uv[2] - uv[0]).abs() * 1024.0,
                    (uv[3] - uv[1]).abs() * 1024.0,
                ],
                density: 1.0,
                tint: ColorRef::Literal(Srgba {
                    rgb: [255; 3],
                    alpha: 1.0,
                }),
                fit,
                focal: [0.5, 0.5],
                offset: [0.0; 2],
                nearest: false,
            })],
            view_box: Some(vb),
            view_rect: Some(rect),
            ..ElementDraw::default()
        }))
    }

    /// A `<foreignObject x y width height>`: a column box `width` by
    /// `height` user units laid out at that size, its corner on `(x, y)`
    /// fitted into the `<svg>`'s box and scaled with the view box, its
    /// document elements cut to it (SVG's overflow hidden).
    #[allow(clippy::too_many_arguments)]
    fn build_svg_foreign(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        vb: [f32; 4],
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        let computed = r.cascade.computed.clone();
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let mut warnings = Vec::new();
        let len = |p: &str, whole: f32, warnings: &mut Vec<String>| {
            r.str(p)
                .map_or(Ok(0.0), |v| crate::svg::length(v, whole))
                .unwrap_or_else(|e| {
                    warnings.push(format!(
                        "{}: {e}",
                        crate::doc::schema_extra::svg_attr(p).unwrap_or(p)
                    ));
                    0.0
                })
        };
        let (x, y) = (
            len("svgX", vb[2], &mut warnings),
            len("svgY", vb[3], &mut warnings),
        );
        let (w, h) = (
            len("svgWidth", vb[2], &mut warnings).max(0.0),
            len("svgHeight", vb[3], &mut warnings).max(0.0),
        );
        let [a, bb, c, d, ..] = t.0;
        if bb != 0.0 || c != 0.0 || (a - 1.0).abs() > 1e-6 || (d - 1.0).abs() > 1e-6 {
            warnings.push(
                "a <foreignObject> takes only a translation from its transform (its content is laid out, not drawn as shapes)".into(),
            );
        }
        let at = Vec2::new(x + t.0[4], y + t.0[5]);
        let el = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        let mut el = Self::box_of(r, el, self.em_of(r));
        el.width = Size::px(w);
        el.height = Size::px(h);
        self.open(id, el);
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        p.position = crate::element::Position::Absolute;
        p.view_place = Some(crate::element::ViewPlace {
            view_box: vb,
            at: [at.x, at.y],
            anchor_x: 0.0,
            anchor_y: 0.0,
        });
        self.nodes[i].props = p;
        self.set_draw(ElementDraw {
            shape: ShapeKind::Rect {
                radii: Some([0.0; 4]),
                corner: crate::paint::Corner::Round,
            },
            clip: crate::paint::Clip::Shape,
            ..ElementDraw::default()
        });
        for e in warnings {
            self.warnings.push(format!(
                "{}:{} ({path}): <foreignObject>: {e}",
                b.doc.source, n.line
            ));
        }
        self.note_built(built);
        let inherited = std::mem::replace(&mut b.inherited, computed);
        build_children(self, b, None);
        b.inherited = inherited;
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }

    /// Builds an SVG element: `<svg>` as a box of its size, a group or a
    /// shape as a box over the whole `<svg>`, a definition as nothing.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_svg(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        kind: &'static str,
        id: &str,
        r: &Resolved,
        props: Props,
        path: String,
        key: u64,
        built: Built,
        build_children: impl FnOnce(&mut Ui, &mut Build<'_>, Option<Props>),
    ) -> Option<String> {
        if kind == "svgDef" {
            return None;
        }
        if animated(&b.doc, n) {
            // Its animations' clock starts as it first shows.
            let clock = smil_clock(&path);
            if self.has_anim(clock) {
                self.anim(clock);
            } else {
                let t = self.state.tick;
                self.set_anim(
                    clock,
                    crate::ui::Anim {
                        t,
                        ..Default::default()
                    },
                );
            }
            if r.smil
                && let Some(owned) = self.owning.last_mut()
            {
                owned.restyle = true;
            }
        }
        let computed = r.cascade.computed.clone();
        let vb = computed
            .get(VIEW_BOX)
            .cloned()
            .flatten()
            .and_then(|v| view_box(&v))
            .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
        if kind == "svgText" {
            self.build_svg_text(b, n, id, r, props, &path, key, vb);
            self.note_built(built);
            return Some(path);
        }
        if kind == "svgForeign" {
            return self.build_svg_foreign(
                b,
                n,
                id,
                r,
                props,
                path,
                key,
                built,
                vb,
                build_children,
            );
        }
        let mut el = Element {
            direction: Direction::Column,
            ..Element::default()
        };
        if kind == "svg" {
            el.width = Size::px(vb[2]);
            el.height = Size::px(vb[3]);
        }
        let mut el = Self::box_of(r, el, self.em_of(r));
        if kind != "svg" {
            el.width = Size::grow();
            el.height = Size::grow();
        }
        self.open(id, el);
        let i = self.parent();
        let mut p = Self::resolved_props(n, r);
        p.states = props.states;
        p.state = props.state;
        if kind != "svg" {
            // Over the whole `<svg>`: its user space is the root's box.
            p.position = crate::element::Position::Absolute;
            p.insets = crate::element::Edges {
                start: Some(0.0),
                top: Some(0.0),
                end: Some(0.0),
                bottom: Some(0.0),
                ..Default::default()
            };
        }
        self.nodes[i].props = p;
        if let Some(role) = n.role {
            self.nodes[i].a11y.role = Some(role);
        }
        if let Some(name) = r.str("name").filter(|n| !n.is_empty()) {
            self.nodes[i].a11y.name = Some(name.to_string());
            // A named `<svg>` is an image, as browsers expose one.
            if kind == "svg" && n.role.is_none() {
                self.nodes[i].a11y.role = Some("image");
            }
        }
        let t = computed
            .get(TRANSFORM)
            .cloned()
            .flatten()
            .and_then(|t| Affine::parse(&t).ok())
            .unwrap_or(Affine::IDENTITY);
        let warn = |ui: &mut Ui, e: String| {
            let tag = n.selector_tag.as_deref().unwrap_or("svg");
            ui.warnings.push(format!(
                "{}:{} ({path}): <{tag}>: {e}",
                b.doc.source, n.line
            ));
        };
        // A clip or a mask cuts the element's children; a cut shape
        // draws as the cut element's child, an anonymous box over it.
        let cut = match cut(&b.doc, r, &t, vb) {
            Ok(c) => c,
            Err(e) => {
                warn(self, e);
                None
            }
        };
        // `filter="url(#id)"`: the element and its subtree drawn through
        // an SVG `<filter>`, the innermost of its layers.
        let filter = match r.str("svgFilter").map(str::trim) {
            Some(v) if kind != "svg" && v != "none" && !v.is_empty() => {
                match self.svg_filter(b, n, kind, r, v, &t, vb) {
                    Ok(f) => Some(f),
                    Err(e) => {
                        warn(self, format!("filter: {e}"));
                        None
                    }
                }
            }
            _ => None,
        };
        let filtered = |mut d: ElementDraw| {
            if let Some(f) = &filter {
                d.filters.insert(
                    0,
                    crate::paint::Layer {
                        filter: Some(f.clone()),
                        ..Default::default()
                    },
                );
            }
            d
        };
        if cut.is_none() && filter.is_some() && kind != "svgShape" {
            self.set_draw(filtered(ElementDraw {
                view_box: Some(vb),
                ..ElementDraw::default()
            }));
        }
        if let Some(c) = &cut {
            self.set_draw(filtered(ElementDraw {
                shape: match c {
                    Cut::Shape(s) | Cut::Box(s, _) => s.clone(),
                    Cut::Gradient(_) => ShapeKind::default(),
                },
                clip: match c {
                    Cut::Shape(_) | Cut::Box(..) => crate::paint::Clip::Shape,
                    Cut::Gradient(g) => {
                        crate::paint::Clip::Mask(crate::paint::Mask::Gradient(g.clone()))
                    }
                },
                view_box: Some(vb),
                view_rect: match c {
                    Cut::Box(_, r) => Some(*r),
                    _ => None,
                },
                ..ElementDraw::default()
            }));
        }
        let over = |ui: &mut Ui, id: &str| {
            ui.open(
                id,
                Element {
                    width: Size::grow(),
                    height: Size::grow(),
                    ..Element::default()
                },
            );
            let i = ui.parent();
            ui.nodes[i].props.position = crate::element::Position::Absolute;
            ui.nodes[i].props.insets = crate::element::Edges {
                start: Some(0.0),
                top: Some(0.0),
                end: Some(0.0),
                bottom: Some(0.0),
                ..Default::default()
            };
        };
        if kind == "svgShape" && n.selector_tag.as_deref() == Some("image") {
            match self.image_draw(b, r, &t, vb) {
                Ok(Some(draw)) if cut.is_none() => self.set_draw(filtered(draw)),
                Ok(Some(draw)) => self.set_draw(draw),
                Ok(None) => {}
                Err(e) => warn(self, e),
            }
        } else if kind == "svgShape" {
            let tag = n.selector_tag.as_deref().unwrap_or("path");
            match shape_draw(&b.doc, tag, r, vb) {
                Ok(ShapeOut {
                    draw,
                    tiles,
                    tile_box,
                    markers,
                }) => {
                    if cut.is_some() {
                        over(self, "~shape");
                        self.set_draw(draw);
                    } else {
                        self.set_draw(filtered(draw));
                    }
                    let boxed = !tiles.is_empty();
                    if boxed {
                        over(self, "~pattern");
                        if let Some((shape, view_rect)) = tile_box {
                            self.set_draw(ElementDraw {
                                shape,
                                clip: crate::paint::Clip::Shape,
                                view_box: Some(vb),
                                view_rect,
                                ..ElementDraw::default()
                            });
                        }
                    }
                    for (k, (shape, paint)) in tiles.into_iter().enumerate() {
                        over(self, &format!("~tile{k}"));
                        self.set_draw(ElementDraw {
                            shape,
                            fills: vec![paint],
                            view_box: Some(vb),
                            ..ElementDraw::default()
                        });
                        self.close();
                    }
                    if boxed {
                        self.close();
                    }
                    for (k, draw) in markers.into_iter().enumerate() {
                        over(self, &format!("~marker{k}"));
                        self.set_draw(draw);
                        self.close();
                    }
                    if cut.is_some() {
                        self.close();
                    }
                }
                Err(e) => warn(self, e),
            }
        }
        if let Some(f) = &filter {
            let bbox = f.svg.map_or([0.0; 4], |(_, bbox)| bbox);
            self.build_element_images(b, n, &computed, &[&**f], ImageSpace::Svg(t, bbox));
        }
        self.note_built(built);
        if kind != "svgShape" {
            let inherited = std::mem::replace(&mut b.inherited, computed);
            build_children(self, b, None);
            b.inherited = inherited;
        }
        let resp = self.close();
        self.fire(b, n, key, &path, resp);
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{DrawCmd, InputFrame, Ui, UiDesc, Viewport};
    use serde_json::json;
    use std::collections::BTreeMap;

    const RING: &str = r##"
<meta name="component" content="ring">
<div id="Box">
  <svg id="Ring" viewBox="0 0 40 40" style:--s={$size}>
    <g id="G" stroke="#ff0000">
      <circle id="Track" cx="20" cy="20" r="18" fill="none" stroke-width="1"/>
      <circle id="Fill" cx="20" cy="20" r="18" fill="none" pathLength="1"
        stroke-dasharray="1 1" stroke-dashoffset={1 - $ready}/>
    </g>
    <rect id="Bar" x="0" y={40 - $h} width="8" height={$h} fill="url(#Grad)"/>
    <linearGradient id="Grad"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#fff"/></linearGradient>
  </svg>
</div>
<style>#Ring { width: calc(var(--s) * 1px); height: calc(var(--s) * 1px); }</style>
"##;

    /// The frame's stroke widths and whether a gradient fill drew.
    fn strokes(ui: &Ui) -> Vec<f32> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => match shape.mode {
                    og_render::UiMode::Stroke(s) => Some(s.width),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    /// An empty meter's `stroke-dasharray="0 1"` draws nothing with butt
    /// caps, as a browser does; a round cap's zero dashes are dots. Fails
    /// when the zero dash reaches og-render, which reads it as solid (the
    /// radial meter's empty wheels drew full rings).
    #[test]
    fn zero_length_dashes_draw_no_ring() {
        let ring = |cap: &str| {
            let doc = format!(
                r##"<meta name="component" content="z">
<svg id="S" viewBox="0 0 40 40" style="width: 80px; height: 80px">
  <circle cx="20" cy="20" r="18" fill="none" stroke="#f00" stroke-width="4"
    stroke-linecap="{cap}" pathLength="1" stroke-dasharray="0 1"/>
</svg>"##
            );
            let mut ui = Ui::new(UiDesc::default());
            ui.define_document(UiDocument::parse("z.svelte", &doc).unwrap());
            ui.mount_document(
                SourceId::new("test", "z"),
                "z",
                BTreeMap::new(),
                BTreeMap::new(),
                &crate::bind::NoEnv,
            )
            .unwrap();
            ui.begin(InputFrame::default(), Viewport::new(200.0, 200.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            strokes(&ui).len()
        };
        assert_eq!(ring("butt"), 0, "butt caps: nothing");
        assert!(ring("round") > 0, "round caps: a dot");
    }

    #[test]
    fn inline_svg_shapes_bind_attributes_and_repaint_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("ring.svelte", RING).unwrap());
        let mut inputs = BTreeMap::new();
        for (name, v) in [("ready", 0.25), ("h", 10.0), ("size", 80.0)] {
            let source = SourceId::new("test", name);
            ui.reactive
                .execute(Command::Signal {
                    source: source.clone(),
                    initial: json!(v),
                    readable: false,
                })
                .unwrap();
            inputs.insert(name.to_string(), source);
        }
        ui.mount_document(
            SourceId::new("test", "ring"),
            "ring",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui, w: f32| {
            ui.begin(InputFrame::default(), Viewport::new(w, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui, 400.0);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        // A 1-unit stroke in a 2× box is 2 px, as in browsers.
        let widths = strokes(&ui);
        assert_eq!(widths.len(), 2, "{widths:?}");
        assert!(widths.iter().all(|w| (w - 2.0).abs() < 1e-4), "{widths:?}");
        let gradient = ui.frame().items.iter().any(|it| {
            matches!(&it.cmd, DrawCmd::Shape { shape, .. }
                if matches!(shape.paint, og_render::UiPaint::Gradient(_)))
        });
        assert!(gradient, "the bar's url(#Grad) fill draws");
        let idle = tick(&mut ui, 400.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        // One bound attribute: that shape alone (the store and the
        // expression `1 - $ready` are its two evaluations).
        ui.reactive
            .execute(Command::Set {
                source: SourceId::new("test", "ready"),
                value: json!(0.75),
                force: false,
            })
            .unwrap();
        let changed = tick(&mut ui, 400.0);
        assert_eq!(
            (changed.bindings, changed.reconciled),
            (2, 1),
            "{changed:?}"
        );
        // The ring's dash offset is a quarter of its circumference, in px.
        let offset = ui.frame().items.iter().find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } => match shape.mode {
                og_render::UiMode::Stroke(s) if s.dash[0] > 0.0 => Some((s.dash, s.dash_offset)),
                _ => None,
            },
            _ => None,
        });
        let (dash, off) = offset.expect("the dashed ring");
        let circumference = 2.0 * std::f32::consts::PI * 18.0 * 2.0;
        assert!((dash[0] - circumference).abs() < 0.5, "{dash:?}");
        assert!((off - 0.25 * circumference).abs() < 0.5, "{off}");
        // Only the box changes: its shapes' attributes stay, their draws
        // are placed in the new box (a 1-unit stroke at 3× is 3 px).
        ui.reactive
            .execute(Command::Set {
                source: SourceId::new("test", "size"),
                value: json!(120.0),
                force: false,
            })
            .unwrap();
        for _ in 0..3 {
            tick(&mut ui, 400.0);
        }
        let widths = strokes(&ui);
        assert_eq!(widths.len(), 2, "{widths:?}");
        assert!(widths.iter().all(|w| (w - 3.0).abs() < 1e-4), "{widths:?}");
        let idle = tick(&mut ui, 400.0);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    }

    const DEFS: &str = r##"
<meta name="component" content="icons">
<div id="Box" style:--s={$size} style:--m={$margin}>
  <svg id="Icons" viewBox="0 0 100 40">
    <title>{$label}</title>
    <defs>
      <symbol id="Coin" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" fill="#f0b429"/></symbol>
      <clipPath id="Half"><rect x="40" y="0" width="10" height="40"/></clipPath>
      <mask id="Fade"><rect x="60" y="0" width="20" height="40" fill="url(#Ramp)"/></mask>
      <linearGradient id="Ramp"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#000"/></linearGradient>
      <pattern id="Dots" patternUnits="userSpaceOnUse" width="5" height="5"><circle cx="2.5" cy="2.5" r="1" fill="#fff"/></pattern>
      <clipPath id="Wedge"><path d="M82 22 L98 22 Q90 30 82 38 Z"/></clipPath>
    </defs>
    <use id="A" href="#Coin" x={$x} y="0" width="20" height="20"/>
    <use id="B" href="#Coin" x="20" y="0" width="10" height="10" fill="#000"/>
    <g id="Clipped" clip-path="url(#Half)"><circle id="Big" cx="45" cy="20" r="20" fill="#f00"/></g>
    <g id="Masked" mask="url(#Fade)"><rect id="Under" x="60" y="0" width="20" height="40" fill="#0f0"/></g>
    <rect id="Dotted" x="80" y="0" width="20" height="20" fill="url(#Dots)"/>
    <g id="Wedged" clip-path="url(#Wedge)"><circle id="Disc" cx="90" cy="30" r="8" fill="#00f"/></g>
  </svg>
</div>
<style>#Box { padding: calc(var(--m) * 1px); } #Icons { width: calc(var(--s) * 1px); height: calc(var(--s) * 0.4px); }</style>
"##;

    /// `<use>` of a `<symbol>`, `clip-path` (a rect and a path), a gradient `<mask>`, a
    /// pattern fill and `<title>`: each draws through the native draw, a
    /// bound attribute of a copy costs that copy, and moving or resizing
    /// the `<svg>` builds nothing.
    ///
    /// Ways it could fail: a use drawn at the symbol's own size, or not
    /// inheriting the use's fill; a clip that cuts nothing or the unfitted
    /// shape; a mask gradient left in view-box units; tiles that miss the
    /// shape's edge; a title drawn as text instead of naming the svg.
    #[test]
    fn svg_definitions_draw_and_cost_only_what_changed() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("icons.svelte", DEFS).unwrap());
        let mut inputs = BTreeMap::new();
        for (name, v) in [
            ("x", json!(0)),
            ("label", json!("Purse")),
            ("size", json!(200)),
            ("margin", json!(0)),
        ] {
            let source = SourceId::new("test", name);
            ui.reactive
                .execute(Command::Signal {
                    source: source.clone(),
                    initial: v,
                    readable: false,
                })
                .unwrap();
            inputs.insert(name.to_string(), source);
        }
        ui.mount_document(
            SourceId::new("test", "icons"),
            "icons",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        for _ in 0..3 {
            tick(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let shapes = |ui: &Ui| -> Vec<og_render::UiShape> {
            ui.frame()
                .items
                .iter()
                .filter_map(|it| match &it.cmd {
                    DrawCmd::Shape { shape, .. } => Some((**shape).clone()),
                    _ => None,
                })
                .collect()
        };
        let all = shapes(&ui);
        // The coins: one filled with the symbol's gold at 2× (20 units in
        // a 2 px/unit box: 40 px wide), the other black (the use's fill
        // does not reach a child that sets its own: still gold).
        let gold = all.iter().filter(|s| matches!(&s.paint, og_render::UiPaint::Solid(c) if c[0] > 0.8 && c[2] < 0.1 && c[1] > 0.3)).count();
        assert_eq!(gold, 2, "both coins draw the symbol's gold");
        let clipped = ui
            .frame()
            .items
            .iter()
            .any(|it| matches!(it.cmd, DrawCmd::PushClip { .. }));
        assert!(clipped, "clip-path pushes a clip");
        // A clip to a path cuts by the path (no warning, above).
        let path_clip = ui.frame().items.iter().any(|it| {
            matches!(
                &it.cmd,
                DrawCmd::PushClip {
                    geometry: og_render::UiGeometry::Path(_),
                    ..
                }
            )
        });
        assert!(path_clip, "the wedge clips to its path");
        let masked = ui.frame().items.iter().find_map(|it| match &it.cmd {
            DrawCmd::BeginLayer {
                mask: Some(og_render::UiMask::Gradient(g)),
                ..
            } => Some(g.clone()),
            _ => None,
        });
        let g = masked.expect("a gradient mask");
        // x 60..80 of 100 units: 0.6..0.8 of the box.
        assert!(
            (g.from.x - 0.6).abs() < 1e-3 && (g.to.x - 0.8).abs() < 1e-3,
            "{g:?}"
        );
        let dots = all.iter().find(|s| matches!(&s.geometry, og_render::UiGeometry::Path(p) if p.verbs.iter().filter(|v| matches!(v, og_render::UiVerb::Move)).count() == 16));
        assert!(dots.is_some(), "16 dot tiles over the 20×20 rect");
        let name = |ui: &Ui| {
            ui.a11y()
                .nodes
                .iter()
                .find(|a| a.path == "test:icons/Icons")
                .map(|a| (a.role.clone(), a.name.clone()))
        };
        assert_eq!(name(&ui), Some(("image".into(), "Purse".into())));
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        let set = |ui: &mut Ui, n: &str, v: serde_json::Value| {
            ui.reactive
                .execute(Command::Set {
                    source: SourceId::new("test", n),
                    value: v,
                    force: false,
                })
                .unwrap();
        };
        // A use's bound x: the use and its copy, nothing else.
        set(&mut ui, "x", json!(5));
        let moved = tick(&mut ui);
        assert_eq!(moved.reconciled, 2, "{moved:?}");
        // The title: one binding; the svg (and the defs it re-seeds), not
        // the shapes.
        set(&mut ui, "label", json!("Wallet"));
        let named = tick(&mut ui);
        assert!(named.bindings == 1 && named.reconciled <= 2, "{named:?}");
        assert_eq!(name(&ui).unwrap().1, "Wallet");
        // The svg moves without resizing, then resizes: no binding of a
        // shape is evaluated, and both settle.
        for (n, v) in [("margin", json!(30)), ("size", json!(300))] {
            set(&mut ui, n, v);
            for _ in 0..3 {
                tick(&mut ui);
            }
            assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
            let idle = tick(&mut ui);
            assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{n}: {idle:?}");
        }
        let g = ui
            .frame()
            .items
            .iter()
            .find_map(|it| match &it.cmd {
                DrawCmd::BeginLayer {
                    mask: Some(og_render::UiMask::Gradient(g)),
                    ..
                } => Some(g.clone()),
                _ => None,
            })
            .unwrap();
        assert!((g.from.x - 0.6).abs() < 1e-3, "the mask refits: {g:?}");
    }

    const TEXT: &str = r##"
<meta name="component" content="chart">
<div id="Box" style:--s={$size} style:--m={$margin}>
  <svg id="Chart" viewBox="0 0 100 50">
    <text id="Mid" x="50" y="25" text-anchor="middle" dominant-baseline="central" font-size="10">{$label}</text>
    <text id="End" x={$x} y="40" text-anchor="end" font-size="8">Max <tspan font-weight="bold" fill="#f00">99</tspan></text>
  </svg>
</div>
<style>#Box { padding: calc(var(--m) * 1px); } #Chart { width: calc(var(--s) * 1px); height: calc(var(--s) * 0.5px); }</style>
"##;

    /// SVG `<text>`: its anchor on the point fitted into the `<svg>`'s box,
    /// scaled with it (hit and accessibility bounds too), right to left
    /// where its view box shows it; a bound `x` costs that text, and moving
    /// or resizing the `<svg>` builds nothing.
    ///
    /// Ways it could fail: the text left at its unscaled size or place; a
    /// start-anchored rather than end-anchored run; bounds of the unscaled
    /// box; mirrored inside the view box, or scaled twice.
    #[test]
    fn svg_text_sits_on_its_point_and_scales_with_the_view_box() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("chart.svelte", TEXT).unwrap());
        let mut inputs = BTreeMap::new();
        for (name, v) in [
            ("x", json!(90)),
            ("label", json!("Gold")),
            ("size", json!(200)),
            ("margin", json!(0)),
        ] {
            let source = SourceId::new("test", name);
            ui.reactive
                .execute(Command::Signal {
                    source: source.clone(),
                    initial: v,
                    readable: false,
                })
                .unwrap();
            inputs.insert(name.to_string(), source);
        }
        ui.mount_document(
            SourceId::new("test", "chart"),
            "chart",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let tick = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end().stats
        };
        let set = |ui: &mut Ui, n: &str, v: serde_json::Value| {
            ui.reactive
                .execute(Command::Set {
                    source: SourceId::new("test", n),
                    value: v,
                    force: false,
                })
                .unwrap();
        };
        let rect = |ui: &Ui, id: &str| {
            let a = ui
                .a11y()
                .nodes
                .iter()
                .find(|a| a.path == format!("test:chart/Chart/{id}"))
                .unwrap_or_else(|| {
                    panic!(
                        "{id} in the a11y tree: {:?}",
                        ui.a11y().nodes.iter().map(|a| &a.path).collect::<Vec<_>>()
                    )
                });
            (a.x, a.y, a.width, a.height)
        };
        for _ in 0..3 {
            tick(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        // Checks against the svg's box `(sx, sy)`, 2 px a unit at size 200.
        // The svg's box: at the padding's corner, or right to left at
        // the surface's far side.
        let check = |ui: &Ui, k: f32, x: f32, m: f32| {
            let sx = if ui.direction() == og_i18n::Direction::Rtl {
                400.0 - m - 100.0 * k
            } else {
                m
            };
            let sy = m;
            let (mx, my, mw, mh) = rect(ui, "Mid");
            // Centred on (50, 25): central baseline, middle anchor.
            assert!(
                (mx + mw / 2.0 - (sx + 50.0 * k)).abs() < 0.5,
                "Mid x {mx} {mw}"
            );
            assert!(
                (my + mh / 2.0 - (sy + 25.0 * k)).abs() < 0.5,
                "Mid y {my} {mh}"
            );
            // The font's extents at 10 units, scaled: taller than 10 px at k 2.
            assert!(mh > 10.0 * k * 0.9 && mh < 10.0 * k * 1.6, "Mid h {mh}");
            let (ex, _, ew, _) = rect(ui, "End");
            assert!(ew > 0.0);
            assert!(
                (ex + ew - (sx + x * k)).abs() < 0.5,
                "End right {ex}+{ew} k {k} sx {sx} rtl {:?}",
                ui.direction()
            );
        };
        check(&ui, 2.0, 90.0, 0.0);
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
        set(&mut ui, "x", json!(70));
        // That text alone: rebuilt with its own bindings, nothing else.
        let moved = tick(&mut ui);
        assert!(moved.bindings <= 2 && moved.reconciled == 1, "{moved:?}");
        tick(&mut ui);
        check(&ui, 2.0, 70.0, 0.0);
        set(&mut ui, "label", json!("Silver"));
        let named = tick(&mut ui);
        assert!(named.bindings == 1 && named.reconciled <= 2, "{named:?}");
        tick(&mut ui);
        check(&ui, 2.0, 70.0, 0.0);
        // The svg moves, then resizes: nothing is built, and the texts follow.
        for (n, v, k) in [("margin", json!(30), 2.0), ("size", json!(300), 3.0)] {
            set(&mut ui, n, v);
            for _ in 0..3 {
                tick(&mut ui);
            }
            let idle = tick(&mut ui);
            assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{n}: {idle:?}");
            check(&ui, k, 70.0, 30.0);
        }
        // Right to left the svg moves across the surface, but its texts
        // stay where its view box shows them.
        ui.set_direction(og_i18n::Direction::Rtl);
        for _ in 0..3 {
            tick(&mut ui);
        }
        check(&ui, 3.0, 70.0, 30.0);
    }

    /// An `<image href>` of an atlas icon fills its rect through the view
    /// box (contained), and resizing the `<svg>` refits it unbuilt.
    #[test]
    fn an_svg_image_fills_its_rect_from_the_atlas() {
        let mut ui = Ui::new(UiDesc::default());
        ui.set_icons(crate::IconAtlas {
            texture: og_render::TextureHandle::from_slot(3),
            columns: 2,
            rows: 1,
            names: vec!["coin".into(), "gem".into()],
            ..Default::default()
        });
        let doc = r##"<meta name="component" content="pic">
<div id="Box"><svg id="S" viewBox="0 0 100 50"><image id="I" href="gem" x="50" y="0" width="50" height="50"/></svg></div>
<style>#S { width: 200px; height: 100px; } #Box { padding: 0; }</style>"##;
        ui.define_document(UiDocument::parse("pic.svelte", doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "pic"),
            "pic",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..3 {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let s = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == "test:pic/S")
            .unwrap();
        let img = ui.frame().items.iter().find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } => match &shape.paint {
                og_render::UiPaint::Image(im) => Some((shape.center, shape.half, im.uv)),
                _ => None,
            },
            _ => None,
        });
        let (c, h, uv) = img.expect("the image draws");
        // The right half of a 200×100 box: centre (150, 50), 50 px half.
        assert!(
            (c.x - (s.x + 150.0)).abs() < 1e-3 && (c.y - (s.y + 50.0)).abs() < 1e-3,
            "{c:?}"
        );
        assert!(
            (h.x - 50.0).abs() < 1e-3 && (h.y - 50.0).abs() < 1e-3,
            "{h:?}"
        );
        assert_eq!(uv, [0.5, 0.0, 1.0, 1.0]);
    }
}

#[cfg(test)]
mod more_tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::{DrawCmd, InputFrame, Ui, UiDesc, Viewport};
    use og_core::glam::Vec2;
    use std::collections::BTreeMap;

    /// A dash list cut by hand: pattern 4 2 1 2 along a 20-unit line.
    /// Ways it could fail: only the first dash and gap honoured; the
    /// offset entering the pattern at the wrong element; a zero dash with
    /// no dot for its caps; a dash cut short at a segment's end.
    #[test]
    fn dash_lists_cut_every_dash_and_gap() {
        let line = crate::svg::Path::parse("M0 0 L8 0 L20 0").unwrap();
        let spans = |pattern: &[f32], offset: f32| -> Vec<(f32, f32)> {
            line.dashed(pattern, offset, 0.01)
                .iter()
                .map(|d| (d[0].x, d.last().unwrap().x))
                .map(|(a, b)| ((a * 1000.0).round() / 1000.0, (b * 1000.0).round() / 1000.0))
                .collect()
        };
        let p = [4.0, 2.0, 1.0, 2.0];
        assert_eq!(
            spans(&p, 0.0),
            [
                (0.0, 4.0),
                (6.0, 7.0),
                (9.0, 13.0),
                (15.0, 16.0),
                (18.0, 20.0)
            ]
        );
        assert_eq!(
            spans(&p, 5.0),
            [
                (1.0, 2.0),
                (4.0, 8.0),
                (10.0, 11.0),
                (13.0, 17.0),
                (19.0, 20.0)
            ]
        );
        // Zero-length dashes are dots, each a tiny dash its caps round.
        assert_eq!(
            spans(&[0.0, 5.0], 0.0)
                .iter()
                .map(|s| s.0)
                .collect::<Vec<_>>(),
            [0.0, 5.0, 10.0, 15.0, 20.0]
        );
    }

    const DOC: &str = r##"
<meta name="component" content="pic">
<div id="Box">
  <svg id="S" viewBox="0 0 100 100">
    <defs>
      <marker id="Tip" viewBox="0 0 10 10" refX="0" refY="5" markerWidth="2" markerHeight="2" orient="auto-start-reverse">
        <path d="M0 0 L10 5 L0 10 z" fill="#0000ff"/>
      </marker>
      <path id="Arc" d="M10 80 L90 80"/>
    </defs>
    <polyline id="P" points="10,10 30,10 30,30" fill="none" stroke="#000" stroke-width="2"
      marker-start="url(#Tip)" marker-mid="url(#Tip)" marker-end="url(#Tip)"/>
    <text id="T" x="50" y="40" text-anchor="middle" font-size="6"><tspan x="50">One</tspan><tspan x="50" dy="2em">Two</tspan></text>
    <text id="W" x="80" y="20" font-size="6" white-space="pre-line">Up
      Down</text>
    <text id="On" font-size="6" text-anchor="middle"><textPath href="#Arc" startOffset="50%">{$word}</textPath></text>
    <foreignObject id="F" x="10" y="50" width="40" height="20"><div id="B"></div></foreignObject>
  </svg>
</div>
<style>#S { width: 200px; height: 200px; } #Box { padding: 0; } #B { width: 30px; height: 10px; }</style>
"##;

    /// Markers, lines of a text, a text on a path and a foreignObject, in
    /// a 2× box. Ways it could fail: markers not drawn at every vertex, or
    /// not turned (auto-start-reverse not flipping the start); a tspan's
    /// dy not moving its line, or lines not each on their anchor; a kept
    /// line break dropped; text on a path drawn in a line; a foreignObject
    /// laid out at its user size but not placed and scaled into the box.
    #[test]
    fn markers_lines_text_paths_and_foreign_objects_draw_in_the_view_box() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("pic.svelte", DOC).unwrap());
        let word = SourceId::new("test", "word");
        ui.reactive
            .execute(crate::reactive::session::Command::Signal {
                source: word.clone(),
                initial: serde_json::json!("Along"),
                readable: false,
            })
            .unwrap();
        let mut inputs = BTreeMap::new();
        inputs.insert("word".to_string(), word);
        ui.mount_document(
            SourceId::new("test", "pic"),
            "pic",
            inputs,
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        for _ in 0..3 {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let s = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == "test:pic/S")
            .unwrap();
        let o = Vec2::new(s.x, s.y);
        // The three blue triangles, 4 px long (2 user units, stroke 2, ×2
        // box... markerWidth 2 × stroke 2 = 4 units = 8 px), tail on the vertex.
        let mut tips: Vec<(Vec2, Vec2)> = ui
            .frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => match (&shape.paint, &shape.geometry) {
                    (og_render::UiPaint::Solid(c), og_render::UiGeometry::Path(p))
                        if c[2] > 0.9 && c[0] < 0.1 =>
                    {
                        let lo = p
                            .points
                            .iter()
                            .fold(Vec2::splat(f32::MAX), |m, q| m.min(*q));
                        let hi = p
                            .points
                            .iter()
                            .fold(Vec2::splat(f32::MIN), |m, q| m.max(*q));
                        let mid = shape.center + (lo + hi) / 2.0;
                        Some(((mid - o) / 2.0, (hi - lo) / 4.0))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();
        tips.sort_by(|a, b| (a.0.x + a.0.y).total_cmp(&(b.0.x + b.0.y)));
        assert_eq!(tips.len(), 3, "{tips:?}");
        let near = |a: Vec2, b: Vec2| (a - b).length() < 0.2;
        // Start (10,10), reversed: pointing back along -x, 4 units long.
        assert!(near(tips[0].0, Vec2::new(8.0, 10.0)), "{tips:?}");
        // Mid (30,10): the bisector of +x and +y, a diagonal.
        let (c, _) = tips[1];
        assert!(
            c.x > 30.0 && c.y > 10.0 && near(c - Vec2::new(30.0, 10.0), Vec2::splat(c.x - 30.0)),
            "{tips:?}"
        );
        // End (30,30): pointing down +y.
        assert!(near(tips[2].0, Vec2::new(30.0, 32.0)), "{tips:?}");
        let rect = |id: &str| {
            ui.a11y()
                .nodes
                .iter()
                .filter(|a| a.path.starts_with(&format!("test:pic/S/{id}")) && !a.name.is_empty())
                .map(|a| (a.name.clone(), a.x, a.y, a.width, a.height))
                .collect::<Vec<_>>()
        };
        // Two lines centred on x 50, the second 2em (12 units, 24 px) down.
        let lines = rect("T");
        assert_eq!(
            lines.iter().map(|l| l.0.as_str()).collect::<Vec<_>>(),
            ["One", "Two"],
            "{lines:?}"
        );
        for l in &lines {
            assert!((l.1 + l.3 / 2.0 - (o.x + 100.0)).abs() < 0.6, "{l:?}");
        }
        assert!((lines[1].2 - lines[0].2 - 24.0).abs() < 0.6, "{lines:?}");
        // pre-line: its break kept, so two lines tall.
        let w = rect("W");
        assert_eq!(w[0].0, "Up\nDown");
        assert!(w[0].4 > 1.8 * lines[0].4, "{w:?} vs {lines:?}");
        // On the path: the text's box is the path's extent from the view
        // box's origin (glyphs are drawn along it; no font draws in tests).
        let on = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/On"))
            .unwrap();
        assert!(
            (on.x - o.x).abs() < 0.5
                && (on.y - o.y).abs() < 0.5
                && (on.width - 180.0).abs() < 0.5
                && (on.height - 160.0).abs() < 0.5,
            "{on:?}"
        );
        // The 30×10 box inside the foreignObject: at its corner (10,50),
        // laid out in user units and scaled 2× with the view box.
        let b = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path.ends_with("/B"))
            .unwrap();
        let got = [b.x - o.x, b.y - o.y, b.width, b.height];
        assert!(
            got.iter()
                .zip([20.0, 100.0, 60.0, 20.0])
                .all(|(g, e)| (g - e).abs() < 0.5),
            "{b:?}"
        );
    }

    const SMIL: &str = r##"
<meta name="component" content="smil">
<div id="Box">
<svg id="S" viewBox="0 0 100 100">
  <g id="G">
    <animateTransform attributeName="transform" type="translate" from="0 0" to="40 0" dur="1s" fill="freeze"/>
    <path d="M0 0 H10 V10 Z" fill="#00ff00"/>
  </g>
  <polygon id="P" points="0,50 10,50 10,60" fill="#ff0000">
    <animate id="Grow" attributeName="points" values="0,50 10,50 10,60;0,50 30,50 30,60" begin="0.5s" dur="1s" fill="freeze"/>
    <set attributeName="fill" to="#0000ff" begin="Grow.end"/>
  </polygon>
</svg>
</div>
<style>#S { width: 100px; height: 100px; } #Box { padding: 0; }</style>
"##;

    /// SMIL on the UI clock (10 ticks a second): a group's
    /// animateTransform moving the path it holds, a polygon's points
    /// growing from a delayed begin, and a set chained to its end.
    /// Ways it could fail: a group's animated transform not reaching its
    /// children; the element not resolving again each frame (frozen at its
    /// first value); begin read from the wrong clock; fill="freeze" not
    /// holding the end; a syncbase begin never firing; reduced motion
    /// still playing.
    #[test]
    fn smil_animations_play_on_the_ui_clock() {
        let run = |reduced: bool, frames: u32| -> Vec<Vec<([f32; 3], f32, f32)>> {
            let mut ui = Ui::new(UiDesc {
                ticks_per_second: 10,
                ..UiDesc::default()
            });
            if reduced {
                let mut theme = *ui.theme();
                theme.reduced_motion = true;
                ui.set_theme(theme);
            }
            ui.define_document(UiDocument::parse("smil.svelte", SMIL).unwrap());
            ui.mount_document(
                SourceId::new("test", "smil"),
                "smil",
                BTreeMap::new(),
                BTreeMap::new(),
                &crate::bind::NoEnv,
            )
            .unwrap();
            let mut out = Vec::new();
            for _ in 0..frames {
                ui.begin(InputFrame::default(), Viewport::new(200.0, 200.0));
                ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
                ui.end();
                assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
                let s = ui
                    .probe_elements()
                    .into_iter()
                    .find(|e| e.path == "test:smil/S")
                    .unwrap();
                // Each solid path's color and its left and right edges.
                let mut shapes: Vec<_> = ui
                    .frame()
                    .items
                    .iter()
                    .filter_map(|it| match &it.cmd {
                        DrawCmd::Shape { shape, .. } => match (&shape.paint, &shape.geometry) {
                            (og_render::UiPaint::Solid(c), og_render::UiGeometry::Path(p)) => {
                                let lo = p.points.iter().fold(f32::MAX, |m, q| m.min(q.x));
                                let hi = p.points.iter().fold(f32::MIN, |m, q| m.max(q.x));
                                let x = shape.center.x - s.x;
                                Some(([c[0], c[1], c[2]], x + lo, x + hi))
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                shapes.sort_by(|a, b| a.0[1].total_cmp(&b.0[1]).reverse());
                out.push(shapes);
            }
            out
        };
        let frames = run(false, 25);
        let at = |tick: usize| {
            assert_eq!(frames[tick].len(), 2, "{tick}: {:?}", frames[tick]);
            (frames[tick][0], frames[tick][1])
        };
        let near = |a: f32, b: f32| (a - b).abs() < 0.05;
        // Tick 0: the clock starts as it shows; nothing has moved.
        let (g, p) = at(0);
        assert!(
            near(g.1, 0.0) && near(p.2, 10.0) && p.0[0] > 0.9,
            "{g:?} {p:?}"
        );
        // 0.5 s: the group halfway (20 units), the polygon about to grow.
        let (g, p) = at(5);
        assert!(near(g.1, 20.0) && near(p.2, 10.0), "{g:?} {p:?}");
        // 1 s: the group frozen at 40; the polygon halfway (20).
        let (g, p) = at(10);
        assert!(near(g.1, 40.0) && near(p.2, 20.0), "{g:?} {p:?}");
        // 1.5 s: Grow ends at 30 and freezes; its end turns the fill blue.
        let (g, p) = at(15);
        assert!(
            near(g.1, 40.0) && near(p.2, 30.0) && p.0[2] > 0.9,
            "{g:?} {p:?}"
        );
        let (g, p) = at(24);
        assert!(
            near(g.1, 40.0) && near(p.2, 30.0) && p.0[2] > 0.9,
            "{g:?} {p:?}"
        );
        // Reduced motion: each shows its frozen end from the first frame
        // (the set has no end, so no effect).
        let still = run(true, 2);
        let (g, p) = (still[1][0], still[1][1]);
        assert!(
            near(g.1, 40.0) && near(p.2, 30.0) && p.0[0] > 0.9,
            "{still:?}"
        );
    }
}
