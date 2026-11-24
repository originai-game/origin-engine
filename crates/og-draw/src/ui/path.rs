//! Vector paths at run time (`docs/ui.md` §3.3): [`UiPath`], a kurbo
//! `BezPath` of lines and quadratic and cubic Béziers drawn as a [`super::UiGeometry::Path`]
//! with nothing cooked, so a path can change every frame (a chart, a morph,
//! an imported animation's shapes).
//!
//! The pass draws a path as every other shape: a signed distance evaluated
//! per pixel, so its fill, outline strokes, shadows and glows take the same
//! paints and effects at any scale. The CPU side ([`PathGpu`]) turns its
//! cubics into quadratics (kurbo's `CubicBez::to_quads`) and sorts the
//! curves into horizontal bands (Slug's banding); the fragment gathers the
//! curves of its band only: the distance to each (Inigo Quilez's closed
//! form) and the winding number of a ray to +x through it (Slug's root
//! eligibility, exact at shared endpoints). See `third_party/vector-paths/`.

use glam::Vec2;
use kurbo::{BezPath, CubicBez, PathEl, Point, SvgParseError};

/// Which points a closed path covers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UiFillRule {
    /// Where the winding number is not zero (SVG's default, Rive's).
    #[default]
    NonZero,
    /// Where the winding number is odd.
    EvenOdd,
}

/// A vector path in a shape's local space (CSS px about its centre, y
/// down): kurbo's contours of lines and Béziers, filled by its rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiPath {
    pub path: BezPath,
    pub fill_rule: UiFillRule,
}

/// A quadratic curve of a path (`line`: its control is on the chord).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub p: [Vec2; 3],
    pub line: bool,
}

/// A point as kurbo's.
pub fn kurbo_point(v: Vec2) -> Point {
    Point::new(f64::from(v.x), f64::from(v.y))
}

/// A kurbo point as glam's.
pub fn glam_vec2(p: Point) -> Vec2 {
    Vec2::new(p.x as f32, p.y as f32)
}

impl UiPath {
    /// Whether it starts with a move (kurbo's rule) and every point is
    /// finite.
    pub fn check(&self) -> Result<(), String> {
        if self
            .path
            .elements()
            .first()
            .is_some_and(|e| !matches!(e, PathEl::MoveTo(_)))
        {
            return Err("a path starts with a move".into());
        }
        if !self.path.is_finite() {
            return Err("a path's points are not all finite".into());
        }
        Ok(())
    }

    /// The box of its points (controls included, so it holds the curves):
    /// min and max; `None` when it has none.
    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        if self.path.elements().is_empty() {
            return None;
        }
        let b = self.path.control_box();
        Some((
            glam_vec2(Point::new(b.x0, b.y0)),
            glam_vec2(Point::new(b.x1, b.y1)),
        ))
    }

    /// The path `t` of the way from this one to `to` (path morphing): the
    /// points interpolated one by one, `None` unless both have the same
    /// elements. `t` is not clamped, so an overshooting ease overshoots.
    pub fn lerp(&self, to: &UiPath, t: f32) -> Option<UiPath> {
        let (a, b) = (self.path.elements(), to.path.elements());
        if a.len() != b.len() {
            return None;
        }
        let t = f64::from(t);
        let l = |p: Point, q: Point| p.lerp(q, t);
        let elements = a
            .iter()
            .zip(b)
            .map(|(x, y)| {
                Some(match (*x, *y) {
                    (PathEl::MoveTo(p), PathEl::MoveTo(q)) => PathEl::MoveTo(l(p, q)),
                    (PathEl::LineTo(p), PathEl::LineTo(q)) => PathEl::LineTo(l(p, q)),
                    (PathEl::QuadTo(p1, p2), PathEl::QuadTo(q1, q2)) => {
                        PathEl::QuadTo(l(p1, q1), l(p2, q2))
                    }
                    (PathEl::CurveTo(p1, p2, p3), PathEl::CurveTo(q1, q2, q3)) => {
                        PathEl::CurveTo(l(p1, q1), l(p2, q2), l(p3, q3))
                    }
                    (PathEl::ClosePath, PathEl::ClosePath) => PathEl::ClosePath,
                    _ => return None,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(UiPath {
            path: BezPath::from_vec(elements),
            fill_rule: if t < 0.5 {
                self.fill_rule
            } else {
                to.fill_rule
            },
        })
    }

    /// Each contour as quadratics, closed back to its start; a cubic is
    /// kurbo's quadratics within `tolerance` of it (`CubicBez::to_quads`,
    /// the midpoint approximation).
    pub fn quads(&self, tolerance: f32) -> Vec<Quad> {
        let tolerance = f64::from(tolerance.max(1e-4));
        let mut out = Vec::new();
        let (mut start, mut at) = (None::<Point>, Point::ORIGIN);
        let close = |out: &mut Vec<Quad>, at: Point, start: Option<Point>| {
            if let Some(s) = start
                && at != s
            {
                out.push(line(glam_vec2(at), glam_vec2(s)));
            }
        };
        for el in self.path.iter() {
            match el {
                PathEl::MoveTo(p) => {
                    close(&mut out, at, start);
                    (start, at) = (Some(p), p);
                }
                PathEl::LineTo(p) => {
                    if p != at {
                        out.push(line(glam_vec2(at), glam_vec2(p)));
                    }
                    at = p;
                }
                PathEl::QuadTo(c, p) => {
                    out.push(quad(glam_vec2(at), glam_vec2(c), glam_vec2(p)));
                    at = p;
                }
                PathEl::CurveTo(c1, c2, p) => {
                    for (_, _, q) in CubicBez::new(at, c1, c2, p).to_quads(tolerance) {
                        out.push(quad(glam_vec2(q.p0), glam_vec2(q.p1), glam_vec2(q.p2)));
                    }
                    at = p;
                }
                PathEl::ClosePath => {
                    close(&mut out, at, start);
                    at = start.unwrap_or(at);
                }
            }
        }
        close(&mut out, at, start);
        out.retain(|q| q.p.iter().all(|p| p.is_finite()));
        out
    }
}

/// SVG path data (`d`) as kurbo parses it (`BezPath::from_svg`, arcs as
/// cubics), its errors said for the path's author.
pub fn parse_svg_path(d: &str) -> Result<BezPath, String> {
    BezPath::from_svg(d).map_err(|e| match e {
        SvgParseError::UninitializedPath => {
            "path data starts with M (a move to its first point)".into()
        }
        SvgParseError::UnknownCommand(c) => {
            format!("'{c}' is not a path command (M L H V C S Q T A Z)")
        }
        SvgParseError::Wrong => "path data has a malformed number or arc flag (0 or 1)".into(),
        SvgParseError::UnexpectedEof => {
            "path data ends inside a command: a number is missing".into()
        }
        other => format!("path data: {other}"),
    })
}

/// Path elements flattened by kurbo into polylines no further than
/// `tolerance` from the curves, each with whether it closes: what a line
/// along the path (a [`super::UiGeometry::Polyline`] with joins and caps)
/// draws. A point repeated in place is dropped (a joint of no length has
/// no direction for its miter), as is a closed contour's last point where
/// it is its first.
pub fn polylines(path: impl IntoIterator<Item = PathEl>, tolerance: f32) -> Vec<(Vec<Vec2>, bool)> {
    let mut out: Vec<(Vec<Vec2>, bool)> = Vec::new();
    let mut cur: Vec<Vec2> = Vec::new();
    let flush = |cur: &mut Vec<Vec2>, out: &mut Vec<(Vec<Vec2>, bool)>, closed: bool| {
        if closed
            && cur.len() > 2
            && cur
                .first()
                .zip(cur.last())
                .is_some_and(|(a, b)| a.distance_squared(*b) <= 1e-12)
        {
            cur.pop();
        }
        if cur.len() > 1 {
            out.push((std::mem::take(cur), closed));
        }
        cur.clear();
    };
    kurbo::flatten(path, f64::from(tolerance.max(1e-4)), |el| match el {
        PathEl::MoveTo(p) => {
            flush(&mut cur, &mut out, false);
            cur.push(glam_vec2(p));
        }
        PathEl::LineTo(p) => {
            let p = glam_vec2(p);
            if cur.last().is_none_or(|l| l.distance_squared(p) > 1e-12) {
                cur.push(p);
            }
        }
        PathEl::ClosePath => {
            let first = cur.first().copied();
            flush(&mut cur, &mut out, true);
            // Drawing on without a move starts where it closed.
            cur.extend(first);
        }
        PathEl::QuadTo(..) | PathEl::CurveTo(..) => unreachable!("kurbo flattens to lines"),
    });
    flush(&mut cur, &mut out, false);
    out
}

/// One cubic flattened by kurbo into points (the first included) no
/// further than `tolerance` from it, for a caller that holds its curves as
/// points (CSS `shape-outside`).
pub fn flatten_cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32) -> Vec<Vec2> {
    let mut out = vec![p0];
    let c = [
        PathEl::MoveTo(kurbo_point(p0)),
        PathEl::CurveTo(kurbo_point(p1), kurbo_point(p2), kurbo_point(p3)),
    ];
    kurbo::flatten(c, f64::from(tolerance.max(1e-4)), |el| {
        if let PathEl::LineTo(p) = el {
            out.push(glam_vec2(p));
        }
    });
    out
}

fn line(a: Vec2, b: Vec2) -> Quad {
    Quad {
        p: [a, (a + b) * 0.5, b],
        line: true,
    }
}

/// A quadratic, a line when its control is (nearly) on its chord: the
/// closed form of its distance loses precision there.
fn quad(a: Vec2, c: Vec2, b: Vec2) -> Quad {
    let chord = b - a;
    let len = chord.length();
    let off = if len > 0.0 {
        (c - a).perp_dot(chord).abs() / len
    } else {
        (c - a).length()
    };
    if off <= 1e-3 * len.max(1e-6) && (c - a).dot(chord) >= 0.0 && (c - b).dot(chord) <= 0.0 {
        return line(a, b);
    }
    Quad {
        p: [a, c, b],
        line: false,
    }
}

/// What a path writes into the pass's point table, in the shape's local
/// space: its curves, two records each (p0 p1; p2 and 1 for a line), a
/// header (the curves' box; the first band's top and the margin), a head
/// per band (where its curve list starts, its length), then the lists,
/// four curve indices a record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathGpu {
    /// The curves' first record, the header's, the bands' count and a
    /// band's height.
    pub geo: [f32; 4],
    /// The fill rule (0 nonzero, 1 even-odd).
    pub rule: f32,
}

/// The most bands a path is cut into.
const MAX_BANDS: usize = 64;

/// Writes `path` into `table` for an instance that reads its distance up
/// to `margin` local units past the outline (antialiasing, a stroke's
/// reach, a shadow's blur); curves are within `tolerance` local units.
/// `None` when it has no curves.
pub fn write(
    path: &UiPath,
    margin: f32,
    tolerance: f32,
    table: &mut Vec<[f32; 4]>,
) -> Option<PathGpu> {
    let quads = path.quads(tolerance);
    if quads.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for q in &quads {
        for p in q.p {
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    let margin = margin.max(0.0);
    let first = table.len();
    for q in &quads {
        let [a, b, c] = q.p;
        table.push([a.x, a.y, b.x, b.y]);
        table.push([c.x, c.y, f32::from(u8::from(q.line)), 0.0]);
    }
    // Bands over the box grown by the margin; a curve is in every band its
    // own y range, grown by the margin, meets, so a band holds every curve
    // within the margin of its points and every curve a ray to +x from them
    // crosses.
    let top = lo.y - margin;
    let height = hi.y - lo.y + 2.0 * margin;
    let n = quads.len().div_ceil(4).clamp(1, MAX_BANDS);
    let bh = (height / n as f32).max(1e-6);
    let mut lists: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, q) in quads.iter().enumerate() {
        let (y0, y1) =
            q.p.iter()
                .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
        let band = |y: f32| (((y - top) / bh).floor().max(0.0) as usize).min(n - 1);
        for list in &mut lists[band(y0 - margin)..=band(y1 + margin)] {
            list.push(i as u32);
        }
    }
    let header = table.len();
    table.push([lo.x, lo.y, hi.x, hi.y]);
    table.push([top, margin, 0.0, 0.0]);
    let heads = table.len();
    table.resize(heads + n, [0.0; 4]);
    for (b, list) in lists.iter().enumerate() {
        table[heads + b] = [table.len() as f32, list.len() as f32, 0.0, 0.0];
        for chunk in list.chunks(4) {
            let mut r = [0.0; 4];
            for (k, i) in chunk.iter().enumerate() {
                r[k] = *i as f32;
            }
            table.push(r);
        }
    }
    Some(PathGpu {
        geo: [first as f32, header as f32, n as f32, bh],
        rule: f32::from(u8::from(path.fill_rule == UiFillRule::EvenOdd)),
    })
}
