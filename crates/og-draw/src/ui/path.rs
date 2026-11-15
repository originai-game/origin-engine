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
use kurbo::{BezPath, CubicBez, PathEl, Point};

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

/// The curve records of the band local height `y` is in: (first, count)
/// of its list.
#[cfg(test)]
fn band_of(table: &[[f32; 4]], g: &PathGpu, y: f32) -> (usize, usize) {
    let (header, nb, bh) = (g.geo[1] as usize, g.geo[2] as usize, g.geo[3]);
    let top = table[header + 1][0];
    let band = (((y - top) / bh).floor().max(0.0) as usize).min(nb - 1);
    let head = table[header + 2 + band];
    (head[0] as usize, head[1] as usize)
}

/// Curve `k` of the band list at `start`, relative to `p`.
#[cfg(test)]
fn curve_of(table: &[[f32; 4]], g: &PathGpu, start: usize, k: usize, p: Vec2) -> [Vec2; 3] {
    let ci = table[start + k / 4][k % 4] as usize;
    let first = g.geo[0] as usize;
    let (r0, r1) = (table[first + 2 * ci], table[first + 2 * ci + 1]);
    [
        Vec2::new(r0[0], r0[1]) - p,
        Vec2::new(r0[2], r0[3]) - p,
        Vec2::new(r1[0], r1[1]) - p,
    ]
}

/// The winding number at local `p`: a ray to +x through its band.
#[cfg(test)]
fn winding_at(table: &[[f32; 4]], g: &PathGpu, p: Vec2) -> i32 {
    let (start, count) = band_of(table, g, p.y);
    (0..count)
        .map(|k| {
            let [a, b, c] = curve_of(table, g, start, k, p);
            crossings(a, b, c)
        })
        .sum()
}

/// The signed distances of local `p` to a path [`write`] wrote, as the
/// shader's `sd_path` measures them (by sampling a curve instead of its
/// closed form), for tests: the fill's, to the curves within the margin
/// where inside meets outside (a self-crossing line inside a nonzero fill
/// is none), and the line's, to every curve (what strokes follow). A
/// curve's sides are judged at its nearest point: the winding just on the
/// sample's side of it (a second ray from there) and across it.
#[cfg(test)]
pub(crate) fn sd2(table: &[[f32; 4]], g: &PathGpu, p: Vec2) -> (f32, f32) {
    let margin = table[g.geo[1] as usize + 1][1];
    let inside = |w: i32| {
        if g.rule > 0.5 { w % 2 != 0 } else { w != 0 }
    };
    let (start, count) = band_of(table, g, p.y);
    let here = inside(winding_at(table, g, p));
    let (mut fill, mut line) = (1e9f32, 1e9f32);
    for k in 0..count {
        let [a, b, c] = curve_of(table, g, start, k, p);
        let (d, t) = (0..=1024)
            .map(|i| {
                let t = i as f32 / 1024.0;
                let u = 1.0 - t;
                ((a * u * u + b * 2.0 * u * t + c * t * t).length(), t)
            })
            .fold((f32::MAX, 0.0), |m, x| if x.0 < m.0 { x } else { m });
        line = line.min(d);
        if d >= fill || d > margin {
            continue;
        }
        let q = a * (1.0 - t) * (1.0 - t) + b * 2.0 * (1.0 - t) * t + c * t * t;
        let s = side((b - a) * (1.0 - t) + (c - b) * t, q);
        // Just on the sample's side of the curve's nearest point.
        let m = q * (1.0 - (NEAR / d.max(NEAR)).min(1.0));
        let w = winding_at(table, g, p + m);
        if s == 0 || inside(w) != inside(w + s) {
            fill = d;
        }
    }
    let sign = if here { -1.0 } else { 1.0 };
    (sign * fill, sign * line)
}

/// How far from a curve its near side is judged, local units.
#[cfg(test)]
const NEAR: f32 = 1e-2;

/// The fill's signed distance ([`sd2`]).
#[cfg(test)]
pub(crate) fn sd(table: &[[f32; 4]], g: &PathGpu, p: Vec2) -> f32 {
    sd2(table, g, p).0
}

/// How the winding number changes from the sample (the origin) across a
/// curve at its nearest point `q`, where it runs along `tangent`: the
/// sign of tangent × (sample − q). The shader's `quad_side`.
#[cfg(test)]
fn side(tangent: Vec2, q: Vec2) -> i32 {
    let c = tangent.perp_dot(-q);
    if c.abs() < 1e-9 {
        0
    } else if c > 0.0 {
        1
    } else {
        -1
    }
}

/// The signed crossings of the ray from the origin to +x with the
/// quadratic `a b c` (relative to the sample): Slug's root eligibility.
#[cfg(test)]
fn crossings(a: Vec2, b: Vec2, c: Vec2) -> i32 {
    let shift =
        (u32::from(a.y > 0.0) << 1) | (u32::from(b.y > 0.0) << 2) | (u32::from(c.y > 0.0) << 3);
    let code = (0x2E74u32 >> shift) & 3;
    if code == 0 {
        return 0;
    }
    let (ay, by) = (a.y - 2.0 * b.y + c.y, a.y - b.y);
    let (ax, bx) = (a.x - 2.0 * b.x + c.x, a.x - b.x);
    let (t1, t2) = if ay.abs() < 1.0 / 65536.0 {
        let t = a.y * 0.5 / by;
        (t, t)
    } else {
        let d = (by * by - ay * a.y).max(0.0).sqrt();
        ((by - d) / ay, (by + d) / ay)
    };
    let x = |t: f32| (ax * t - bx * 2.0) * t + a.x;
    let mut w = 0;
    if code & 1 != 0 && x(t1) > 0.0 {
        w += 1;
    }
    if code > 1 && x(t2) > 0.0 {
        w -= 1;
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    fn square(path: &mut UiPath, at: Vec2, size: f32, clockwise: bool) {
        let c = [
            at,
            at + Vec2::new(size, 0.0),
            at + Vec2::splat(size),
            at + Vec2::new(0.0, size),
        ]
        .map(kurbo_point);
        let p = &mut path.path;
        p.move_to(c[0]);
        if clockwise {
            [1, 2, 3].into_iter().for_each(|k| p.line_to(c[k]));
        } else {
            [3, 2, 1].into_iter().for_each(|k| p.line_to(c[k]));
        }
        p.close_path();
    }

    fn circle(c: Vec2, r: f32) -> UiPath {
        UiPath {
            path: kurbo::Circle::new(kurbo_point(c), f64::from(r)).to_path(0.01),
            ..UiPath::default()
        }
    }

    /// The winding number counts each contour with its direction: a
    /// square inside another of the same direction stays filled by the
    /// nonzero rule and is a hole by even-odd; drawn the other way, it is a
    /// hole by both.
    #[test]
    fn winding_and_fill_rules_cut_holes_where_they_should() {
        for (inner_cw, rule, hole) in [
            (true, UiFillRule::NonZero, false),
            (true, UiFillRule::EvenOdd, true),
            (false, UiFillRule::NonZero, true),
            (false, UiFillRule::EvenOdd, true),
        ] {
            let mut p = UiPath::default();
            square(&mut p, Vec2::ZERO, 100.0, true);
            square(&mut p, Vec2::splat(30.0), 40.0, inner_cw);
            p.fill_rule = rule;
            let mut table = Vec::new();
            // A margin past every distance read here, so each is exact.
            let g = write(&p, 30.0, 0.05, &mut table).expect("curves");
            let local = Vec2::new;
            // In the ring, 15 px from the outer edge and the hole.
            assert!((sd(&table, &g, local(15.0, 50.0)) + 15.0).abs() < 1e-3);
            let centre = sd(&table, &g, local(50.0, 50.0));
            assert_eq!(centre > 0.0, hole, "{inner_cw} {rule:?}: {centre}");
            if inner_cw && rule == UiFillRule::NonZero {
                // Filled on both sides (winding 2 and 1), the inner square is
                // no edge: the outer one, 50 px off, is past the margin.
                assert!(centre < -30.0, "{centre}");
            } else {
                assert!((centre.abs() - 20.0).abs() < 1e-3, "{centre}");
            }
            // Outside, at the level of a shared vertex's row (a ray along an
            // edge): outside, 5 px off.
            assert!((sd(&table, &g, local(-5.0, 0.0)) - 5.0).abs() < 1e-3);
            assert!((sd(&table, &g, local(-5.0, 30.0)) - 5.0).abs() < 1e-3);
        }
    }

    /// A circle of cubics: its distance is the circle's within the
    /// quadratics' tolerance, inside and out, in every band, up to the
    /// margin; past it the distance is at least the margin.
    #[test]
    fn a_circle_measures_as_a_circle() {
        let p = circle(Vec2::new(10.0, -4.0), 60.0);
        let mut table = Vec::new();
        let g = write(&p, 3.0, 0.01, &mut table).expect("curves");
        assert!(g.geo[2] > 1.0, "bands: {}", g.geo[2]);
        for k in 0..64 {
            let a = k as f32 / 64.0 * std::f32::consts::TAU;
            for r in [20.0, 58.0, 59.5, 61.5] {
                let at = Vec2::new(10.0, -4.0) + Vec2::from_angle(a) * r;
                let d = sd(&table, &g, at);
                if r == 20.0 {
                    // Past the margin only the sign and that it is past.
                    assert!(d <= -3.0, "angle {a} radius {r}: {d}");
                } else {
                    // The cubic circle itself is 0.03 % off a true one.
                    assert!((d - (r - 60.0)).abs() < 0.03, "angle {a} radius {r}: {d}");
                }
            }
        }
    }

    /// One self-crossing star, nonzero: its pentagon is inside, and the
    /// lines that cross through it are no edge of the fill (a pixel on one
    /// is not antialiased into a seam), though a stroke still follows them.
    /// Even-odd, the same lines are the pentagon's edges.
    #[test]
    fn a_self_crossing_nonzero_fill_has_no_inner_edges() {
        let star = |rule| {
            let v = |k: usize| {
                let a = -std::f32::consts::FRAC_PI_2 + k as f32 * 4.0 * std::f32::consts::PI / 5.0;
                Vec2::from_angle(a) * 64.0
            };
            let mut p = UiPath {
                fill_rule: rule,
                ..UiPath::default()
            };
            p.path.move_to(kurbo_point(v(0)));
            for k in 1..5 {
                p.path.line_to(kurbo_point(v(k)));
            }
            p.path.close_path();
            p
        };
        // The middle of the pentagon's top edge: 64·cos 72° up, on a line
        // that crosses the star.
        let on = Vec2::new(0.0, -64.0 * (72f32).to_radians().cos());
        let mut table = Vec::new();
        let g = write(&star(UiFillRule::NonZero), 2.0, 0.05, &mut table).expect("curves");
        let (fill, line) = sd2(&table, &g, on);
        // Inside, and the nearest real edge is the star's outline, the
        // spike's sides at the inner vertices about 15 px away.
        assert!(fill < -10.0, "nonzero on an inner line: {fill}");
        assert!(line.abs() < 1e-2, "the line through it: {line}");
        let mut table = Vec::new();
        let g = write(&star(UiFillRule::EvenOdd), 2.0, 0.05, &mut table).expect("curves");
        assert!(sd(&table, &g, on).abs() < 1e-2);
    }

    #[test]
    fn paths_morph_only_between_the_same_verbs() {
        let a = circle(Vec2::ZERO, 10.0);
        let b = UiPath {
            path: kurbo::Affine::scale(3.0) * &a.path,
            ..a.clone()
        };
        let mid = a.lerp(&b, 0.5).expect("same elements");
        assert_eq!(
            mid.path.elements()[0],
            PathEl::MoveTo(Point::new(20.0, 0.0))
        );
        let mut other = UiPath::default();
        square(&mut other, Vec2::ZERO, 1.0, true);
        assert!(a.lerp(&other, 0.5).is_none());
        assert!(a.check().is_ok());
        let mut bad = a.clone();
        bad.path.elements_mut()[1] = PathEl::LineTo(Point::new(f64::NAN, 0.0));
        assert!(bad.check().is_err());
    }

    #[test]
    fn a_path_flattens_into_its_contours() {
        let mut p = UiPath::default();
        square(&mut p, Vec2::ZERO, 10.0, true);
        p.path.move_to((20.0, 0.0));
        p.path.curve_to((20.0, 10.0), (30.0, 10.0), (30.0, 0.0));
        let lines = polylines(&p.path, 0.1);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0.len(), 4);
        assert!(lines[0].1);
        assert!(!lines[1].1);
        assert_eq!(lines[1].0.last(), Some(&Vec2::new(30.0, 0.0)));
    }
}
