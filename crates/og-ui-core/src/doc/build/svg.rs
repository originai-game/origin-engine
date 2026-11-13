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
use crate::svg::{Affine, GradientDef, PaintRef, Style, bounds, shape_path, subpaths};
use og_core::glam::Vec2;

mod defs;
mod shapes;
mod smil;

pub use defs::*;
pub(crate) use defs::{css_clip, css_filter, srgba};
pub use shapes::*;
pub(super) use smil::animate;
pub use smil::*;

/// The kinds this module builds.
pub const KINDS: &[&str] = &[
    "svg",
    "svgGroup",
    "svgShape",
    "svgUse",
    "svgDef",
    "svgText",
    "svgForeign",
];

pub const VIEW_BOX: &str = " svg viewBox";
pub const TRANSFORM: &str = " svg transform";

pub fn key(attr: &str) -> String {
    format!(" svg {attr}")
}

/// HTML's size of an `<svg>` with neither a size nor a view box.
pub const DEFAULT_SIZE: [f32; 2] = [300.0, 150.0];

fn fixed(s: Option<Size>) -> Option<f32> {
    match s? {
        Size::Fixed(v) => Some(v),
        _ => None,
    }
}

/// The view box `min-x min-y width height`.
pub fn view_box(text: &str) -> Option<[f32; 4]> {
    let n = crate::svg::numbers(text).ok()?;
    (n.len() == 4 && n[2] > 0.0 && n[3] > 0.0).then(|| [n[0], n[1], n[2], n[3]])
}

pub fn matrix(a: &Affine) -> String {
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
pub enum Cut {
    /// Paths, cut by the path fill's coverage.
    Shape(ShapeKind),
    /// One rect or ellipse, clipped natively.
    Box(ShapeKind, [f32; 4]),
    Gradient(Gradient),
}

pub fn cut(
    doc: &UiDocument,
    r: &Resolved,
    t: &Affine,
    vb: [f32; 4],
) -> Result<Option<Cut>, String> {
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

/// A shape's draw, the tiles of a pattern fill drawn inside it, the box
/// they are cut to (`None`: the shape is a path), and its markers' draws.
pub struct ShapeOut {
    pub draw: ElementDraw,
    pub tiles: Vec<(ShapeKind, Paint)>,
    pub tile_box: Option<(ShapeKind, Option<[f32; 4]>)>,
    pub markers: Vec<ElementDraw>,
}

pub fn shape_draw(
    doc: &UiDocument,
    tag: &str,
    r: &Resolved,
    vb: [f32; 4],
) -> Result<ShapeOut, String> {
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
pub fn baseline_shift(base: &str, ascender: f32, descender: f32, size: f32) -> Result<f32, String> {
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

/// Where an element's element images are drawn: an SVG element's user
/// space (its map into the view box, and its bounding box there), or a
/// CSS box's (CSS px from its border box's top left).
#[derive(Clone, Copy)]
pub enum ImageSpace {
    Svg(Affine, [f32; 4]),
    Css,
}
