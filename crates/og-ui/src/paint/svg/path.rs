//! SVG's basic shapes as kurbo paths, and what SVG reads from a path: its
//! bounding box, its dashes, its marker vertices and its cubics as
//! `ShapeKind::Path` points. Path data itself is kurbo's
//! ([`og_draw::parse_svg_path`]), arcs included.

use super::*;
use og_draw::kurbo::{
    self, BezPath, CubicBez, ParamCurveArclen as _, PathEl, PathSeg, Point, Shape as _, SvgArc,
};
use og_draw::{glam_vec2, kurbo_point};

/// The accuracy of a basic shape's arcs, user units: kurbo's own for path
/// data's arcs, so a `<circle>` and the same circle as `d` agree (no arc
/// is cut coarser than quarter turns).
const ARC_TOLERANCE: f64 = 0.1;

/// An elliptical arc from the end of `p` (SVG's endpoint parametrisation),
/// as kurbo's cubics; a line where the arc is one.
fn arc_to(p: &mut BezPath, r: Vec2, to: Vec2) {
    let from = p
        .elements()
        .last()
        .and_then(|e| e.end_point())
        .unwrap_or(Point::ORIGIN);
    let arc = SvgArc {
        from,
        to: kurbo_point(to),
        radii: kurbo_point(r).to_vec2(),
        x_rotation: 0.0,
        large_arc: false,
        sweep: true,
    };
    match kurbo::Arc::from_svg_arc(&arc) {
        Some(a) => p.extend(a.append_iter(ARC_TOLERANCE)),
        None => p.line_to(arc.to),
    }
}

/// An ellipse at `c` with radii `r`, clockwise from its right (where SVG
/// starts a `<circle>`'s and an `<ellipse>`'s dashes).
fn ellipse(c: Vec2, r: Vec2) -> BezPath {
    let arc = kurbo::Arc::new(
        kurbo_point(c),
        kurbo_point(r).to_vec2(),
        0.0,
        std::f64::consts::TAU,
        0.0,
    );
    let mut p: BezPath = arc.path_elements(ARC_TOLERANCE).collect();
    p.close_path();
    p
}

/// A closed or open line through `points`.
fn polyline(points: impl IntoIterator<Item = Vec2>, closed: bool) -> BezPath {
    let mut p = BezPath::new();
    for (k, q) in points.into_iter().enumerate() {
        if k == 0 {
            p.move_to(kurbo_point(q));
        } else {
            p.line_to(kurbo_point(q));
        }
    }
    if closed {
        p.close_path();
    }
    p
}

/// The local path of the shape element `name` whose attributes `get`
/// reads, in a view box `vb` (percentages are of its size).
pub fn shape_path<'a>(
    name: &str,
    get: &dyn Fn(&str) -> Option<&'a str>,
    vb: [f32; 4],
) -> Result<Option<BezPath>, String> {
    let (w, h) = (vb[2], vb[3]);
    let diag = (w * w + h * h).sqrt() / std::f32::consts::SQRT_2;
    let len = |attr: &str, whole: f32| {
        get(attr)
            .map_or(Ok(0.0), |v| length(v, whole))
            .map_err(|e| format!("<{name}> {attr}: {e}"))
    };
    Ok(Some(match name {
        "path" => match get("d") {
            Some(d) => og_draw::parse_svg_path(d).map_err(|e| format!("<path>: {e}"))?,
            None => return Ok(None),
        },
        "rect" => {
            let (x, y, rw, rh) = (
                len("x", w)?,
                len("y", h)?,
                len("width", w)?,
                len("height", h)?,
            );
            if rw <= 0.0 || rh <= 0.0 {
                return Ok(None);
            }
            let opt = |name: &str, whole: f32| {
                get(name)
                    .map(|v| length(v, whole))
                    .transpose()
                    .map_err(|e| format!("<rect> {name}: {e}"))
            };
            // One radius given stands for both.
            let (rx_v, ry_v) = match (opt("rx", w)?, opt("ry", h)?) {
                (Some(a), Some(b)) => (a, b),
                (Some(a), None) => (a, a),
                (None, Some(b)) => (b, b),
                (None, None) => (0.0, 0.0),
            };
            let (rx, ry) = (rx_v.clamp(0.0, rw / 2.0), ry_v.clamp(0.0, rh / 2.0));
            if rx <= 0.0 || ry <= 0.0 {
                polyline(
                    [
                        Vec2::new(x, y),
                        Vec2::new(x + rw, y),
                        Vec2::new(x + rw, y + rh),
                        Vec2::new(x, y + rh),
                    ],
                    true,
                )
            } else {
                let r = Vec2::new(rx, ry);
                let mut p = BezPath::new();
                p.move_to(kurbo_point(Vec2::new(x + rx, y)));
                p.line_to(kurbo_point(Vec2::new(x + rw - rx, y)));
                arc_to(&mut p, r, Vec2::new(x + rw, y + ry));
                p.line_to(kurbo_point(Vec2::new(x + rw, y + rh - ry)));
                arc_to(&mut p, r, Vec2::new(x + rw - rx, y + rh));
                p.line_to(kurbo_point(Vec2::new(x + rx, y + rh)));
                arc_to(&mut p, r, Vec2::new(x, y + rh - ry));
                p.line_to(kurbo_point(Vec2::new(x, y + ry)));
                arc_to(&mut p, r, Vec2::new(x + rx, y));
                p.close_path();
                p
            }
        }
        "circle" => {
            let r = len("r", diag)?;
            if r <= 0.0 {
                return Ok(None);
            }
            ellipse(Vec2::new(len("cx", w)?, len("cy", h)?), Vec2::splat(r))
        }
        "ellipse" => {
            let (rx, ry) = (len("rx", w)?, len("ry", h)?);
            if rx <= 0.0 || ry <= 0.0 {
                return Ok(None);
            }
            ellipse(Vec2::new(len("cx", w)?, len("cy", h)?), Vec2::new(rx, ry))
        }
        "line" => polyline(
            [
                Vec2::new(len("x1", w)?, len("y1", h)?),
                Vec2::new(len("x2", w)?, len("y2", h)?),
            ],
            false,
        ),
        "polyline" | "polygon" => {
            let n = numbers(get("points").unwrap_or(""))
                .map_err(|e| format!("<{name}> points: {e}"))?;
            if n.len() < 4 {
                return Ok(None);
            }
            polyline(
                n.as_chunks::<2>().0.iter().map(|q| Vec2::new(q[0], q[1])),
                name == "polygon",
            )
        }
        _ => return Ok(None),
    }))
}

/// The bounding box `[x, y, w, h]` of a path's curves (SVG's
/// `objectBoundingBox`: the curves' extremes, not their controls).
pub fn bounds(p: &BezPath) -> [f32; 4] {
    let b = p.bounding_box();
    [
        b.x0 as f32,
        b.y0 as f32,
        (b.width() as f32).max(1e-6),
        (b.height() as f32).max(1e-6),
    ]
}

/// Its length along every subpath, kurbo's arc length to `accuracy`.
pub fn path_length(path: &BezPath, accuracy: f32) -> f32 {
    path.segments()
        .map(|s| s.arclen(f64::from(accuracy.max(1e-6))))
        .sum::<f64>() as f32
}

/// The dashes of `pattern` (dash, gap…, an odd one repeated) from `offset`
/// along each subpath, kurbo's (each subpath starts the pattern again, as
/// SVG's do, and a closed one's dash across its start is one): each dash
/// a subpath of its own, its curves kept. A dash of no length is a tiny
/// one along the path, so its caps still draw a dot.
pub fn dashed(path: &BezPath, pattern: &[f32], offset: f32) -> BezPath {
    let mut dashes: Vec<f64> = pattern.iter().map(|d| f64::from(*d)).collect();
    if dashes.len() % 2 == 1 {
        dashes.extend_from_within(..);
    }
    let total: f64 = dashes.iter().sum();
    if dashes.is_empty() || total <= 0.0 || !total.is_finite() {
        return BezPath::new();
    }
    const DOT: f64 = 1e-3;
    for k in (0..dashes.len()).step_by(2) {
        if dashes[k] <= 0.0 {
            dashes[k] = DOT;
            dashes[k + 1] = (dashes[k + 1] - DOT).max(0.0);
        }
    }
    kurbo::dash(path.iter(), f64::from(offset), &dashes).collect()
}

/// Where markers go: every vertex (each subpath's start and each
/// segment's end, a closed subpath's closing line too), with the
/// direction the path comes in and goes out (`None` at an open end). A
/// subpath of no segments has none.
pub fn vertices(path: &BezPath) -> Vec<(Vec2, Option<Vec2>, Option<Vec2>)> {
    let mut out: Vec<(Vec2, Option<Vec2>, Option<Vec2>)> = Vec::new();
    let dir = |v: Vec2| (v.length() > 1e-9).then(|| v.normalize());
    // Each segment's end and its start and end tangents.
    let mut segs: Vec<(Vec2, Option<Vec2>, Option<Vec2>)> = Vec::new();
    let mut start = Vec2::ZERO;
    let mut at = Vec2::ZERO;
    let mut flush = |segs: &mut Vec<(Vec2, Option<Vec2>, Option<Vec2>)>,
                     start: Vec2,
                     at: Vec2,
                     closed: bool| {
        if segs.is_empty() {
            return;
        }
        if closed && (at - start).length() > 1e-6 {
            let d = dir(start - at);
            segs.push((start, d, d));
        }
        let closing_in = if closed {
            segs.last().and_then(|s| s.2)
        } else {
            None
        };
        out.push((start, closing_in, segs.first().and_then(|s| s.1)));
        for i in 0..segs.len() {
            let next = if i + 1 < segs.len() {
                segs[i + 1].1
            } else if closed {
                segs.first().and_then(|s| s.1)
            } else {
                None
            };
            out.push((segs[i].0, segs[i].2, next));
        }
        segs.clear();
    };
    for el in path.iter() {
        let c = match el {
            PathEl::MoveTo(p) => {
                flush(&mut segs, start, at, false);
                start = glam_vec2(p);
                at = start;
                continue;
            }
            PathEl::ClosePath => {
                flush(&mut segs, start, at, true);
                at = start;
                continue;
            }
            PathEl::LineTo(p) => {
                let p = glam_vec2(p);
                [p, p, p]
            }
            PathEl::QuadTo(q, p) => [glam_vec2(q), glam_vec2(q), glam_vec2(p)],
            PathEl::CurveTo(c1, c2, p) => [glam_vec2(c1), glam_vec2(c2), glam_vec2(p)],
        };
        let out_dir = dir(c[0] - at).or(dir(c[1] - at)).or(dir(c[2] - at));
        let in_dir = dir(c[2] - c[1]).or(dir(c[2] - c[0])).or(dir(c[2] - at));
        segs.push((c[2], out_dir, in_dir));
        at = c[2];
    }
    flush(&mut segs, start, at, false);
    out
}

/// A path's subpaths as `ShapeKind::Path` points: each its start, then
/// three points per cubic (a line or a quadratic raised to one), and
/// whether it closes. A subpath of no segments is left out.
pub fn subpaths(path: &BezPath) -> Vec<(Vec<[f32; 2]>, bool)> {
    let mut out: Vec<(Vec<[f32; 2]>, bool)> = Vec::new();
    let mut cur: Vec<[f32; 2]> = Vec::new();
    let mut flush = |cur: &mut Vec<[f32; 2]>, closed: bool| {
        if cur.len() > 1 {
            out.push((std::mem::take(cur), closed));
        }
        cur.clear();
    };
    let mut start = Point::ORIGIN;
    let mut at = Point::ORIGIN;
    for el in path.iter() {
        let seg = match el {
            PathEl::MoveTo(p) => {
                flush(&mut cur, false);
                (start, at) = (p, p);
                cur.push(glam_vec2(p).into());
                continue;
            }
            PathEl::ClosePath => {
                flush(&mut cur, true);
                at = start;
                continue;
            }
            PathEl::LineTo(p) => PathSeg::Line(kurbo::Line::new(at, p)),
            PathEl::QuadTo(q, p) => PathSeg::Quad(kurbo::QuadBez::new(at, q, p)),
            PathEl::CurveTo(c1, c2, p) => PathSeg::Cubic(CubicBez::new(at, c1, c2, p)),
        };
        if cur.is_empty() {
            cur.push(glam_vec2(at).into());
        }
        let c = seg.to_cubic();
        cur.extend([c.p1, c.p2, c.p3].map(|p| <[f32; 2]>::from(glam_vec2(p))));
        at = c.p3;
    }
    flush(&mut cur, false);
    out
}
