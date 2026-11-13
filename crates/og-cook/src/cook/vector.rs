//! Vectors (`.ogt` of `type vector`, `docs/ui.md` §3.3): an SVG subset
//! cooked to triangles that og-render's UI pass draws with an analytic
//! antialiased edge (`og_draw::ui::UiVectorData`).
//!
//! ```text
//! type vector
//! version 1
//! ---
//! source "ui/emblem.svg"   # the SVG, an asset path (or: svg "<svg …>")
//! tolerance 0.05           # curves flattened to within this many view-box
//!                          # units (default: the view box's longer side / 2000)
//! ```
//!
//! The SVG subset: `<svg>` with a `viewBox` (or `width` and `height`);
//! `<g>`, `<path>` (every command, arcs included), `<rect>` (with `rx`,
//! `ry`), `<circle>`, `<ellipse>`, `<line>`, `<polyline>` and `<polygon>`;
//! `transform` (matrix, translate, scale, rotate, skewX, skewY) on any of
//! them; `fill` and `stroke` as a colour (`#rgb`, `#rgba`, `#rrggbb`,
//! `#rrggbbaa`, `rgb()`, `rgba()`, the CSS basic names, `currentColor`,
//! which is white so the draw's tint colours it, or `none`) or
//! `url(#id)` of a `<linearGradient>` or `<radialGradient>` in the file
//! (`gradientUnits`, `spreadMethod`, `href` for stops, up to 8 `<stop>`s
//! with `offset`, `stop-color` and `stop-opacity`; a `gradientTransform`
//! of translate and scale); `fill-rule`, `fill-opacity`, `stroke-width`,
//! `stroke-opacity`, `stroke-linecap`, `stroke-linejoin`,
//! `stroke-miterlimit`, `stroke-dasharray`, `opacity` (a group's
//! multiplies its children's alpha); each as an attribute or in `style`.
//! `<defs>`, `<title>`, `<desc>` and `<metadata>` are read past. Anything
//! else (text, images, `use`, clips, masks, filters, patterns) is refused
//! with the element named.
//!
//! Fills and strokes both become triangles in document order, so they
//! layer as the SVG does: a stroke is its outline (segments, joins and
//! caps) filled by the nonzero rule. The fill is cut into trapezoids
//! between the outline's vertices; each vertex on the outline carries its
//! outward miter, along which the UI pass moves it half a device pixel in,
//! and a fringe quad along every outline edge ramps the coverage from one
//! to nothing over the pixel that straddles it: the edge's antialiasing is
//! exact at any size the vector is drawn, with nothing cooked per size.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::glam::Vec2;
use og_core::ogt::{Document, Number, Value};
use og_draw::ui::{
    UiGradient, UiGradientKind, UiSpread, UiStop, UiVectorData, UiVectorPaint, UiVectorVertex,
};
use std::collections::HashMap;

mod tessellation;

use tessellation::*;

/// The `.ogt` type of a vector.
pub const VECTOR_TYPE: &str = "vector";

/// The most triangles a vector cooks to.
const MAX_TRIANGLES: usize = 400_000;

/// A parsed vector file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VectorFile {
    /// The SVG's asset path (`source`).
    pub source: Option<String>,
    /// The SVG inline (`svg`).
    pub svg: Option<String>,
    pub tolerance: Option<f32>,
}

/// Parses a vector file.
pub fn parse(text: &str) -> Result<VectorFile, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != VECTOR_TYPE {
        return Err(format!("a vector is `type vector`, not `type {}`", doc.ty));
    }
    if doc.version != 1 {
        return Err(format!(
            "vector version {} (this engine reads 1)",
            doc.version
        ));
    }
    let mut out = VectorFile::default();
    for e in &doc.body.entries {
        let line = e.line;
        match e.key.as_str() {
            "source" | "svg" => {
                if out.source.is_some() || out.svg.is_some() {
                    return Err(format!("line {line}: one `source` or `svg` per vector"));
                }
                let s = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: `{}` takes a string", e.key))?
                    .to_string();
                if e.key == "source" {
                    out.source = Some(s);
                } else {
                    out.svg = Some(s);
                }
            }
            "tolerance" => {
                let t = e
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .map(Number::as_f32)
                    .filter(|t| t.is_finite() && *t > 0.0)
                    .ok_or_else(|| format!("line {line}: `tolerance` takes a positive number"))?;
                out.tolerance = Some(t);
            }
            other => return Err(format!("line {line}: unknown vector entry `{other}`")),
        }
    }
    if out.source.is_none() && out.svg.is_none() {
        return Err("a vector needs `source` (an SVG's asset path) or `svg`".into());
    }
    Ok(out)
}

/// What the cook stamp covers besides the file: the SVG's bytes.
pub(super) fn dependencies(s: &Source) -> String {
    let Ok(file) = parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    file.source
        .map(|src| super::read_sibling(s, &src).unwrap_or_default())
        .unwrap_or_default()
}

/// Cooks a vector into an `og.Vector` blob.
pub(super) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a vector is text (UTF-8)".into()))?;
    let file = parse(text).map_err(CookError)?;
    let svg = match (&file.source, &file.svg) {
        (Some(src), _) => {
            super::read_sibling(s, src).map_err(|e| CookError(format!("source \"{src}\": {e}")))?
        }
        (None, Some(svg)) => svg.clone(),
        (None, None) => unreachable!("parse requires one"),
    };
    let data = tessellate(&svg, file.tolerance).map_err(CookError)?;
    let triangles = data.indices.len() / 3;
    Ok((
        write(&data, stamp),
        vec![format!(
            "{} triangle(s), {} paint(s), view box {}×{}",
            triangles,
            data.paints.len(),
            data.view_box[2],
            data.view_box[3]
        )],
    ))
}

/// The blob of a tessellated vector.
pub(crate) fn write(data: &UiVectorData, stamp: u64) -> Vec<u8> {
    let mut stops = Vec::new();
    let paints: Vec<VectorPaintRow> = data
        .paints
        .iter()
        .map(|p| match p {
            UiVectorPaint::Solid(c) => VectorPaintRow {
                r: c[0],
                g: c[1],
                b: c[2],
                a: c[3],
                ..VectorPaintRow::default()
            },
            UiVectorPaint::Gradient(g) => {
                let first = stops.len() as u32;
                stops.extend(g.stops.iter().map(|s| VectorStopRow {
                    offset: s.offset,
                    r: s.color[0],
                    g: s.color[1],
                    b: s.color[2],
                    a: s.color[3],
                }));
                VectorPaintRow {
                    kind: match g.kind {
                        UiGradientKind::Linear => 1,
                        UiGradientKind::Radial => 2,
                        UiGradientKind::Conic => 3,
                        UiGradientKind::Diamond => 4,
                    },
                    spread: match g.spread {
                        UiSpread::Pad => 0,
                        UiSpread::Repeat => 1,
                        UiSpread::Reflect => 2,
                        UiSpread::RepeatStops => 3,
                    },
                    dither: u32::from(g.dither),
                    first_stop: first,
                    stop_count: g.stops.len() as u32,
                    from_x: g.from.x,
                    from_y: g.from.y,
                    to_x: g.to.x,
                    to_y: g.to.y,
                    ..VectorPaintRow::default()
                }
            }
        })
        .collect();
    let mut w = BlobWriter::new(
        format::Vector::type_hash(),
        format::Vector::schema_version(),
        stamp,
    );
    let [x, y, width, height] = data.view_box;
    w.table(
        "meta",
        &[VectorMeta {
            x,
            y,
            width,
            height,
            vertex_count: data.vertices.len() as u32,
            index_count: data.indices.len() as u32,
            paint_count: paints.len() as u32,
        }],
        format::row_hash::<VectorMeta>(),
    )
    .table("paints", &paints, format::row_hash::<VectorPaintRow>())
    .table("stops", &stops, format::row_hash::<VectorStopRow>())
    .raw(
        "vertices",
        bytemuck::cast_slice::<UiVectorVertex, u8>(&data.vertices).to_vec(),
        16,
    )
    .raw(
        "indices",
        bytemuck::cast_slice::<u32, u8>(&data.indices).to_vec(),
        16,
    );
    w.finish()
}

// ── the SVG model (og-ui's, shared with documents' inline `<svg>`) ───────

use og_ui::svg::{
    Cap, Element, GradientDef, Join, PaintRef, Style, bounds, gradient_defs, length_attr, numbers,
    parse_xml, shape_path,
};

/// An sRGB unit channel to linear.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// An sRGB colour (straight alpha) as linear RGBA.
fn linear_rgba(c: [f32; 4]) -> [f32; 4] {
    [linear(c[0]), linear(c[1]), linear(c[2]), c[3]]
}

/// A placed gradient as the UI pass's, its stops linear.
fn ui_gradient(g: og_ui::svg::PlacedGradient) -> UiGradient {
    UiGradient {
        kind: if g.radial {
            UiGradientKind::Radial
        } else {
            UiGradientKind::Linear
        },
        from: g.from,
        to: g.to,
        stops: g
            .stops
            .into_iter()
            .map(|(offset, color)| UiStop {
                offset,
                color: linear_rgba(color),
            })
            .collect(),
        spread: g.spread,
        dither: true,
    }
}

pub fn tessellate(svg: &str, tolerance: Option<f32>) -> Result<UiVectorData, String> {
    let root = parse_xml(svg)?;
    let view_box = match root.get("viewBox") {
        Some(v) => {
            let n = numbers(v)?;
            if n.len() != 4 || n[2] <= 0.0 || n[3] <= 0.0 {
                return Err(format!("viewBox \"{v}\" is not four numbers with a size"));
            }
            [n[0], n[1], n[2], n[3]]
        }
        None => {
            let w = length_attr(&root, "width", 0.0, 0.0)?;
            let h = length_attr(&root, "height", 0.0, 0.0)?;
            if w <= 0.0 || h <= 0.0 {
                return Err("an <svg> needs a viewBox, or a width and a height".into());
            }
            [0.0, 0.0, w, h]
        }
    };
    let tolerance = tolerance.unwrap_or(view_box[2].max(view_box[3]) / 2000.0);
    let gradients = gradient_defs(&root)?;
    let mut mesh = Mesh::default();
    let style = Style::of(&root, &Style::default())?;
    let ctx = Ctx {
        gradients: &gradients,
        tolerance,
        view_box,
    };
    for child in &root.children {
        draw(&ctx, child, &style, &mut mesh)?;
    }
    Ok(UiVectorData {
        view_box,
        vertices: mesh.vertices,
        indices: mesh.indices,
        paints: mesh.paints,
        strokes: Vec::new(),
    })
}

struct Ctx<'a> {
    gradients: &'a HashMap<String, GradientDef>,
    tolerance: f32,
    view_box: [f32; 4],
}

fn draw(ctx: &Ctx, el: &Element, parent: &Style, mesh: &mut Mesh) -> Result<(), String> {
    match el.name.as_str() {
        "defs" | "title" | "desc" | "metadata" | "linearGradient" | "radialGradient" | "style" => {
            return Ok(());
        }
        "g" | "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" => {}
        other => {
            return Err(format!(
                "<{other}> is not supported (paths, basic shapes, groups and gradients)"
            ));
        }
    }
    let style = Style::of(el, parent)?;
    if style.opacity <= 0.0 {
        return Ok(());
    }
    if el.name == "g" {
        for c in &el.children {
            draw(ctx, c, &style, mesh)?;
        }
        return Ok(());
    }
    if style.dash_offset != 0.0 {
        return Err(format!("<{}>: stroke-dashoffset is not supported", el.name));
    }
    let Some(local) = shape_path(&el.name, &|k| el.get(k), ctx.view_box)? else {
        return Ok(());
    };
    let bbox = bounds(&local);
    let t = style.transform;
    let path = local.transformed(&t);
    let tol = ctx.tolerance;
    let eps = (ctx.view_box[2].max(ctx.view_box[3]) * 1e-5).max(1e-6);
    let resolve = |p: &PaintRef, alpha: f32, mesh: &mut Mesh| -> Result<Option<u32>, String> {
        Ok(match p {
            PaintRef::None => None,
            // White: the draw's tint is the colour.
            PaintRef::Current => Some(mesh.paint(UiVectorPaint::Solid([1.0, 1.0, 1.0, alpha]))),
            PaintRef::Color(c) => {
                let c = linear_rgba(*c);
                Some(mesh.paint(UiVectorPaint::Solid([c[0], c[1], c[2], c[3] * alpha])))
            }
            PaintRef::Token(name) => {
                return Err(format!(
                    "<{}>: var(--{name}) is a document's; a cooked vector takes literal colours (or currentColor, its tint)",
                    el.name
                ));
            }
            PaintRef::Url(id) => {
                let g = ctx
                    .gradients
                    .get(id)
                    .ok_or_else(|| format!("<{}>: url(#{id}) names no gradient", el.name))?;
                Some(mesh.paint(UiVectorPaint::Gradient(ui_gradient(
                    g.place(bbox, &t, alpha)?,
                ))))
            }
        })
    };
    let lines = path.flatten(tol);
    // A line has no inside; a polyline fills as SVG does, closed.
    if el.name != "line"
        && let Some(paint) = resolve(&style.fill, style.fill_opacity * style.opacity, mesh)?
    {
        let contours: Vec<Vec<Vec2>> = lines
            .iter()
            .filter(|(p, _)| p.len() > 2)
            .map(|(p, _)| p.clone())
            .collect();
        fill(mesh, &contours, style.even_odd, paint, eps)?;
    }
    let width = style.stroke_width * t.scale();
    if width > 0.0
        && let Some(paint) = resolve(&style.stroke, style.stroke_opacity * style.opacity, mesh)?
    {
        let dashes: Vec<f32> = style.dashes.iter().map(|d| d * t.scale()).collect();
        let mut contours = Vec::new();
        for (pts, closed) in &lines {
            let pieces = if dashes.is_empty() {
                vec![(pts.clone(), *closed)]
            } else {
                dash(pts, *closed, &dashes)
                    .into_iter()
                    .map(|d| (d, false))
                    .collect()
            };
            for (p, c) in pieces {
                contours.extend(stroke_outline(
                    &p,
                    c,
                    width / 2.0,
                    style.cap,
                    style.join,
                    style.miter_limit,
                    tol,
                ));
            }
        }
        fill(mesh, &contours, false, paint, eps)?;
    }
    Ok(())
}

pub(crate) static VECTOR: KindDef = KindDef {
    name: "vector",
    extensions: &[],
    types: &[VECTOR_TYPE],
    by_content: false,
    streams: false,
    plan: plan_vector,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&VECTOR];

fn plan_vector<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers the SVG the vector names.
    let svg = dependencies(s);
    let st = stamp(&[b"vector", &hash, svg.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Vector::NAME,
        st,
        Box::new(VectorJob(s)),
    ));
}

/// A vector drawing's one output.
struct VectorJob<'a>(&'a Source);

impl Job for VectorJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VectorJob(s) = self;
        cook(s, p.stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The area the mesh covers before the UI pass moves its outline (the
    /// fringe quads have none there), and its triangles' total.
    fn area(d: &UiVectorData) -> f32 {
        d.indices
            .chunks(3)
            .map(|t| {
                let p = |i: u32| Vec2::from(d.vertices[i as usize].position);
                ((p(t[1]) - p(t[0])).perp_dot(p(t[2]) - p(t[0]))).abs() / 2.0
            })
            .sum()
    }

    fn svg(body: &str) -> String {
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\">{body}</svg>")
    }

    /// A rectangle covers its area once; an even-odd hole and a nonzero
    /// overlap cover what the rules say, so trapezoids neither overlap nor
    /// leave gaps.
    #[test]
    fn fills_cover_their_area_by_the_fill_rule() {
        let d = tessellate(
            &svg(r#"<rect x="10" y="20" width="30" height="40"/>"#),
            None,
        )
        .unwrap();
        assert!((area(&d) - 1200.0).abs() < 1e-2, "{}", area(&d));
        let ring = r#"<path fill-rule="evenodd" d="M0 0H100V100H0Z M25 25H75V75H25Z"/>"#;
        let d = tessellate(&svg(ring), None).unwrap();
        assert!((area(&d) - 7500.0).abs() < 1e-1, "{}", area(&d));
        // The same with nonzero fills the hole (both wound clockwise).
        let d = tessellate(&svg(&ring.replace("evenodd", "nonzero")), None).unwrap();
        assert!((area(&d) - 10000.0).abs() < 1e-1, "{}", area(&d));
        // Two squares overlapping by a quarter: the union.
        let two = r#"<path d="M0 0H50V50H0Z M25 25H75V75H25Z"/>"#;
        let d = tessellate(&svg(two), None).unwrap();
        assert!(
            (area(&d) - (2500.0 * 2.0 - 625.0)).abs() < 1e-1,
            "{}",
            area(&d)
        );
    }

    /// A circle by arcs, and by `<circle>`, is a circle's area within the
    /// flattening's tolerance; every outline vertex moves along a unit-ish
    /// miter, and the fringe quads hold an inner and an outer vertex.
    #[test]
    fn curves_flatten_within_the_tolerance_and_the_outline_carries_its_miters() {
        let want = std::f32::consts::PI * 40.0 * 40.0;
        for body in [
            r#"<circle cx="50" cy="50" r="40"/>"#,
            r#"<path d="M10 50a40 40 0 1 0 80 0A40 40 0 1 0 10 50z"/>"#,
            // One arc between nearly coincident ends: a whole circle.
            r#"<path d="M50 10 a40 40 0 1 0 0.001 0 Z"/>"#,
        ] {
            let d = tessellate(&svg(body), Some(0.01)).unwrap();
            let a = area(&d);
            assert!(a < want && a > want * 0.999, "{body}: {a} vs {want}");
            let moved: Vec<f32> = d
                .vertices
                .iter()
                .map(|v| Vec2::from(v.normal).length())
                .filter(|l| *l > 0.0)
                .collect();
            assert!(!moved.is_empty());
            assert!(moved.iter().all(|l| (0.99..1.01).contains(l)), "{moved:?}");
            assert!(d.vertices.iter().any(|v| v.aa > 0.0) && d.vertices.iter().any(|v| v.aa < 0.0));
        }
    }

    /// A stroke is its outline: a straight butt line is length × width, a
    /// square cap adds half a width at each end, a dashed line half of it.
    #[test]
    fn strokes_cover_their_outline_with_caps_and_dashes() {
        let line = |extra: &str| {
            tessellate(
                &svg(&format!(r#"<line x1="10" y1="50" x2="90" y2="50" stroke="red" stroke-width="10" {extra}/>"#)),
                None,
            )
            .unwrap()
        };
        assert!((area(&line("")) - 800.0).abs() < 0.1, "{}", area(&line("")));
        assert!((area(&line(r#"stroke-linecap="square""#)) - 900.0).abs() < 0.1);
        let round = area(&line(r#"stroke-linecap="round""#));
        let want = 800.0 + std::f32::consts::PI * 25.0;
        assert!((round - want).abs() < 1.0, "{round} vs {want}");
        let dashed = area(&line(r#"stroke-dasharray="10 10""#));
        assert!((dashed - 400.0).abs() < 0.1, "{dashed}");
        // A polyline's miter join fills its corner: an L of two 40-long arms
        // 10 wide is the two arms (overlapping by 5×5 inside the corner)
        // and the join's 5×5 outside it.
        let l = tessellate(
            &svg(r##"<polyline points="10 10 50 10 50 50" fill="none" stroke="#000" stroke-width="10"/>"##),
            None,
        )
        .unwrap();
        assert!(
            (area(&l) - (400.0 + 400.0 - 25.0 + 25.0)).abs() < 0.1,
            "{}",
            area(&l)
        );
    }

    /// Paints: a colour is linear, `currentColor` white; a gradient in
    /// bounding box units lands on the shape's box under its transform; a
    /// group's opacity multiplies its children's alpha.
    #[test]
    fn paints_resolve_colours_gradients_and_opacity() {
        let d = tessellate(
            &svg(r##"<defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#f00"/><stop offset="1" stop-color="#00f" stop-opacity="0.5"/></linearGradient></defs>
            <g opacity="0.5" transform="translate(10 0)"><rect x="10" y="10" width="40" height="20" fill="url(#g)"/></g>
            <rect x="0" y="80" width="10" height="10" fill="currentColor"/>
            <rect x="20" y="80" width="10" height="10" fill="#808080"/>"##),
            None,
        )
        .unwrap();
        let UiVectorPaint::Gradient(g) = &d.paints[0] else {
            panic!("{:?}", d.paints)
        };
        assert_eq!(g.kind, UiGradientKind::Linear);
        assert_eq!(
            (g.from, g.to),
            (Vec2::new(20.0, 10.0), Vec2::new(60.0, 10.0))
        );
        assert_eq!(g.stops[0].color, [1.0, 0.0, 0.0, 0.5]);
        assert_eq!(g.stops[1].color, [0.0, 0.0, 1.0, 0.25]);
        assert_eq!(d.paints[1], UiVectorPaint::Solid([1.0, 1.0, 1.0, 1.0]));
        let UiVectorPaint::Solid(grey) = d.paints[2] else {
            panic!()
        };
        assert!((grey[0] - 0.2158).abs() < 1e-3, "{grey:?}");
    }

    /// What the subset does not hold is refused with the element named.
    #[test]
    fn unsupported_svg_is_refused_by_name() {
        for (body, want) in [
            ("<text>hi</text>", "<text>"),
            (r#"<image href="a.png"/>"#, "<image>"),
            (
                r#"<rect width="1" height="1" clip-path="url(#c)"/>"#,
                "clip-path",
            ),
            (
                r#"<rect width="1" height="1" fill="url(#nowhere)"/>"#,
                "nowhere",
            ),
        ] {
            let e = tessellate(&svg(body), None).expect_err(body);
            assert!(e.contains(want), "{body}: {e}");
        }
        assert!(
            tessellate("<svg><path d='M0 0L1 1'/></svg>", None)
                .unwrap_err()
                .contains("viewBox")
        );
    }

    /// The vector file names its SVG once.
    #[test]
    fn the_file_names_one_svg() {
        let f = parse("type vector\nversion 1\n---\nsource \"ui/a.svg\"\ntolerance 0.1\n").unwrap();
        assert_eq!(f.source.as_deref(), Some("ui/a.svg"));
        assert_eq!(f.tolerance, Some(0.1));
        assert!(parse("type vector\nversion 1\n---\n").is_err());
    }
}
