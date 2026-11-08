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
use crate::doc::extern_element::PRESENTATION;
use crate::doc::schema_extra::svg_prop;
use crate::paint::{
    ColorRef, ElementDraw, Gradient, GradientKind, Paint, ShapeKind, Stop, Stroke, StrokeAlign,
};
use crate::svg::{Affine, GradientDef, PaintRef, Style, bounds, shape_path};
use og_core::glam::Vec2;

/// The kinds this module builds.
pub(super) const KINDS: &[&str] = &["svg", "svgGroup", "svgShape", "svgUse", "svgDef"];

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
    reads.extend(PRESENTATION.iter().map(|a| key(a)));
    let mut writes: Vec<(String, String)> = Vec::new();
    for attr in PRESENTATION {
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
fn filled(parts: Vec<(Vec<[f32; 2]>, bool)>) -> ShapeKind {
    ShapeKind::Path {
        points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
        closed: true,
        filled: true,
        curves: true,
        fill_rule: crate::paint::FillRule::NonZero,
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
    let mut parts = Vec::new();
    for (tag, attrs, own) in &shapes {
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
    Ok(Some(Cut::Shape(filled(parts))))
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
        out.push((filled(parts), paint));
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

fn srgba(c: [f32; 4], alpha: f32) -> Srgba {
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
fn subpaths(p: &crate::svg::Path) -> Vec<(Vec<[f32; 2]>, bool)> {
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

/// A shape's draw, in its `<svg>`'s user space.
/// A shape's draw, the tiles of a pattern fill drawn inside it, and the
/// box they are cut to (`None`: the shape is a path, which og-render
/// cannot clip to yet).
type ShapeOut = (
    ElementDraw,
    Vec<(ShapeKind, Paint)>,
    Option<(ShapeKind, [f32; 4])>,
);

fn shape_draw(doc: &UiDocument, tag: &str, r: &Resolved, vb: [f32; 4]) -> Result<ShapeOut, String> {
    let computed = &r.cascade.computed;
    let value = |name: &str| computed.get(name).cloned().flatten();
    let mut style = Style::default();
    for attr in PRESENTATION {
        if let Some(v) = value(&key(attr)) {
            let parent = style.clone();
            style
                .apply(attr, &v, &parent)
                .map_err(|e| format!("{attr}: {e}"))?;
        }
    }
    let t = value(TRANSFORM)
        .and_then(|t| Affine::parse(&t).ok())
        .unwrap_or(Affine::IDENTITY);
    let get = |attr: &str| svg_prop(attr).and_then(|p| r.str(p));
    let mut draw = ElementDraw {
        view_box: Some(vb),
        ..ElementDraw::default()
    };
    let Some(local) = shape_path(tag, &get, vb)? else {
        return Ok((draw, Vec::new(), None));
    };
    let bbox = bounds(&local);
    let path = local.transformed(&t);
    let parts = subpaths(&path);
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
    let rule = if style.even_odd {
        crate::paint::FillRule::EvenOdd
    } else {
        crate::paint::FillRule::NonZero
    };
    let cap = match style.cap {
        crate::svg::Cap::Butt => crate::paint::Cap::Butt,
        crate::svg::Cap::Round => crate::paint::Cap::Round,
        crate::svg::Cap::Square => crate::paint::Cap::Square,
    };
    let join = match style.join {
        crate::svg::Join::Miter => crate::paint::Join::Miter(style.miter_limit),
        crate::svg::Join::Round => crate::paint::Join::Round,
        crate::svg::Join::Bevel => crate::paint::Join::Bevel,
    };
    let width = style.stroke_width * t.scale();
    draw.shape = ShapeKind::Path {
        points: parts.iter().flat_map(|(p, _)| p.iter().copied()).collect(),
        closed: true,
        filled: true,
        curves: true,
        fill_rule: rule,
        contours: parts.iter().map(|(p, _)| p.len() as u32).collect(),
        width,
        cap,
        join,
    };
    // A line has no inside; a polyline fills as SVG does, closed.
    if tag != "line"
        && let Some(fill) = paint(&style.fill, style.fill_opacity)?
    {
        draw.fills.push(fill);
    }
    if width > 0.0
        && let Some(stroke) = paint(&style.stroke, style.stroke_opacity)?
    {
        // Dashes measure the outline in user units, or in `pathLength`.
        let per = match get("pathLength").map(|v| v.trim().parse::<f32>()) {
            Some(Ok(l)) if l > 0.0 => local.length(vb[2].max(vb[3]) / 2000.0) / l,
            Some(_) => return Err("pathLength is a positive number".into()),
            None => 1.0,
        } * t.scale();
        let dash = match style.dashes.as_slice() {
            [] => [0.0; 2],
            [d, g] => [d * per, g * per],
            more => {
                return Err(format!(
                    "stroke-dasharray takes one dash and one gap, not {} lengths",
                    more.len()
                ));
            }
        };
        for (points, closed) in parts {
            draw.outlines.push((
                ShapeKind::Path {
                    points,
                    closed,
                    filled: false,
                    curves: true,
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
                    dash_offset: style.dash_offset * per,
                    dash_cap: cap,
                    paint: stroke.clone(),
                },
            ));
        }
    }
    let tile_box = if tiles.is_empty() {
        None
    } else {
        basic_box(tag, &get, &local, &t)
    };
    Ok((draw, tiles, tile_box))
}

impl Ui {
    /// An `<image href x y width height preserveAspectRatio>`: an atlas
    /// icon by name, or a render texture by handle, filling its rect.
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
                        "\"{href}\" is a glyph icon, not an image: write <icon name=\"{href}\"> beside the svg"
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
                tint: ColorRef::Literal(Srgba {
                    rgb: [255; 3],
                    alpha: 1.0,
                }),
                fit,
                focal: [0.5, 0.5],
            })],
            view_box: Some(vb),
            view_rect: Some(rect),
            ..ElementDraw::default()
        }))
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
        let computed = r.cascade.computed.clone();
        let vb = computed
            .get(VIEW_BOX)
            .cloned()
            .flatten()
            .and_then(|v| view_box(&v))
            .unwrap_or([0.0, 0.0, DEFAULT_SIZE[0], DEFAULT_SIZE[1]]);
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
        if let Some(c) = &cut {
            self.set_draw(ElementDraw {
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
            });
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
                Ok(Some(draw)) => self.set_draw(draw),
                Ok(None) => {}
                Err(e) => warn(self, e),
            }
        } else if kind == "svgShape" {
            let tag = n.selector_tag.as_deref().unwrap_or("path");
            match shape_draw(&b.doc, tag, r, vb) {
                Ok((draw, tiles, tile_box)) => {
                    if cut.is_some() {
                        over(self, "~shape");
                    }
                    self.set_draw(draw);
                    let boxed = !tiles.is_empty();
                    if boxed {
                        over(self, "~pattern");
                        match tile_box {
                            Some((shape, rect)) => self.set_draw(ElementDraw {
                                shape,
                                clip: crate::paint::Clip::Shape,
                                view_box: Some(vb),
                                view_rect: Some(rect),
                                ..ElementDraw::default()
                            }),
                            None => warn(
                                self,
                                "a pattern fills a rect, circle or ellipse (og-render does not clip to a path yet)".into(),
                            ),
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
                    if cut.is_some() {
                        self.close();
                    }
                }
                Err(e) => warn(self, e),
            }
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
