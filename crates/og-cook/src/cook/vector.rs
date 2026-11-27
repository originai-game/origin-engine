//! Typed vector declarations (`docs/ui.md` §3.3): an SVG subset
//! cooked to triangles that og-render's UI pass draws with an analytic
//! antialiased edge (`og_draw::ui::UiVectorData`).
//!
//! ```ts
//! import {file} from '@origingame/engine/author';
//! export default file.vector({source: 'assets/ui/emblem.svg', tolerance: 0.05});
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
//! `stroke-miterlimit`, `stroke-dasharray`, `stroke-dashoffset`, `opacity` (a group's
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

use super::source_schemas::world::source_data::VectorInput;
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::glam::Vec2;
use og_draw::kurbo;
use og_draw::ui::{
    UiGradient, UiGradientKind, UiSpread, UiStop, UiVectorData, UiVectorPaint, UiVectorVertex,
};
use std::collections::HashMap;

mod tessellation;

use tessellation::*;

/// The most triangles a vector cooks to.
const MAX_TRIANGLES: usize = 400_000;

/// Capture the explicit module-local input before planning/cache lookup.
fn source(s: &Source) -> Result<(String, Option<f32>), String> {
    match s.input::<VectorInput>()? {
        VectorInput::File { source, tolerance } => {
            let root = super::assets_root(s);
            let path = super::source_path::checked(&root, &source)
                .map_err(|e| format!("source '{source}': {e}"))?;
            let svg =
                std::fs::read_to_string(path).map_err(|e| format!("source '{source}': {e}"))?;
            Ok((svg, tolerance))
        }
        VectorInput::Inline { svg, tolerance } => Ok((svg, tolerance)),
    }
}

/// Cooks a vector into an `og.Vector` blob.
fn cook(
    svg: &str,
    tolerance: Option<f32>,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let data = tessellate(svg, tolerance).map_err(CookError)?;
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

use og_ui_next::svg::{
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
fn ui_gradient(g: og_ui_next::svg::PlacedGradient) -> UiGradient {
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
    let Some(local) = shape_path(&el.name, &|k| el.get(k), ctx.view_box)? else {
        return Ok(());
    };
    let bbox = bounds(&local);
    let t = style.transform;
    let path = t.kurbo() * &local;
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
    let lines = og_draw::polylines(&path, tol);
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
        // The stroke's outline is kurbo's stroke expansion, filled by the
        // nonzero rule; its dashes are og-ui's (kurbo's, a zero dash a dot).
        let dashes: Vec<f32> = style.dashes.iter().map(|d| d * t.scale()).collect();
        let along = if dashes.is_empty() {
            path.clone()
        } else {
            og_ui_next::svg::dashed(&path, &dashes, style.dash_offset * t.scale())
        };
        let stroke = kurbo::Stroke::new(f64::from(width))
            .with_caps(match style.cap {
                Cap::Butt => kurbo::Cap::Butt,
                Cap::Round => kurbo::Cap::Round,
                Cap::Square => kurbo::Cap::Square,
            })
            .with_join(match style.join {
                Join::Miter => kurbo::Join::Miter,
                Join::Round => kurbo::Join::Round,
                Join::Bevel => kurbo::Join::Bevel,
            })
            .with_miter_limit(f64::from(style.miter_limit));
        let outline = kurbo::stroke(
            along.iter(),
            &stroke,
            &kurbo::StrokeOpts::default(),
            f64::from(tol),
        );
        let contours: Vec<Vec<Vec2>> = og_draw::polylines(&outline, tol)
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        fill(mesh, &contours, false, paint, eps)?;
    }
    Ok(())
}

pub(crate) static VECTOR: KindDef = KindDef {
    name: "vector",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_vector,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&VECTOR];

fn plan_vector<'a>(
    _cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    match source(s) {
        Ok((svg, tolerance)) => {
            let st = stamp(&[Area::Vector], &[b"vector", &hash, svg.as_bytes()]);
            outs.push((
                s.rel.clone(),
                format::Vector::NAME,
                st,
                Box::new(VectorJob(svg, tolerance)),
            ));
        }
        Err(e) => state.early_errors.push((s.rel.clone(), "vector", e)),
    }
}

/// A vector drawing's one output.
struct VectorJob(String, Option<f32>);

impl Job for VectorJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook(&self.0, self.1, p.stamp)
    }
}
