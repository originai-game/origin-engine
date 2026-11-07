//! Vector paths at run time (`docs/ui.md` §3.3): [`UiPath`], contours of
//! lines and quadratic and cubic Béziers drawn as a [`super::UiGeometry::Path`]
//! with nothing cooked, so a path can change every frame (a chart, a morph,
//! an imported animation's shapes).
//!
//! The pass draws a path as every other shape: a signed distance evaluated
//! per pixel, so its fill, outline strokes, shadows and glows take the same
//! paints and effects at any scale. The CPU side ([`PathGpu`]) turns its
//! cubics into quadratics (kurbo's midpoint approximation) and sorts the
//! curves into horizontal bands (Slug's banding); the fragment gathers the
//! curves of its band only: the distance to each (Inigo Quilez's closed
//! form) and the winding number of a ray to +x through it (Slug's root
//! eligibility, exact at shared endpoints). See `third_party/vector-paths/`.

use glam::Vec2;

/// Which points a closed path covers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UiFillRule {
    /// Where the winding number is not zero (SVG's default, Rive's).
    #[default]
    NonZero,
    /// Where the winding number is odd.
    EvenOdd,
}

/// One step of a [`UiPath`]; its points follow in [`UiPath::points`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiVerb {
    /// Starts a contour at one point.
    Move,
    /// A line to one point.
    Line,
    /// A quadratic Bézier: its control and its end.
    Quad,
    /// A cubic Bézier: its two controls and its end.
    Cubic,
    /// Closes the contour back to its start. Fills close every contour.
    Close,
}

impl UiVerb {
    /// How many points it takes.
    pub fn points(self) -> usize {
        match self {
            UiVerb::Move | UiVerb::Line => 1,
            UiVerb::Quad => 2,
            UiVerb::Cubic => 3,
            UiVerb::Close => 0,
        }
    }
}

/// A vector path in a shape's local space (CSS px about its centre, y
/// down): contours of lines and Béziers, filled by its rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiPath {
    pub verbs: Vec<UiVerb>,
    pub points: Vec<Vec2>,
    pub fill_rule: UiFillRule,
}

/// A quadratic curve of a path (`line`: its control is on the chord).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Quad {
    pub p: [Vec2; 3],
    pub line: bool,
}

impl UiPath {
    pub fn new() -> UiPath {
        UiPath::default()
    }

    pub fn move_to(&mut self, p: Vec2) -> &mut UiPath {
        self.verbs.push(UiVerb::Move);
        self.points.push(p);
        self
    }

    pub fn line_to(&mut self, p: Vec2) -> &mut UiPath {
        self.verbs.push(UiVerb::Line);
        self.points.push(p);
        self
    }

    pub fn quad_to(&mut self, c: Vec2, p: Vec2) -> &mut UiPath {
        self.verbs.push(UiVerb::Quad);
        self.points.extend([c, p]);
        self
    }

    pub fn cubic_to(&mut self, c1: Vec2, c2: Vec2, p: Vec2) -> &mut UiPath {
        self.verbs.push(UiVerb::Cubic);
        self.points.extend([c1, c2, p]);
        self
    }

    pub fn close(&mut self) -> &mut UiPath {
        self.verbs.push(UiVerb::Close);
        self
    }

    /// An ellipse centred on `c` with radii `r` (four cubics, clockwise on
    /// screen).
    pub fn ellipse(&mut self, c: Vec2, r: Vec2) -> &mut UiPath {
        // The control distance of a quarter circle (0.5523).
        let k = 4.0 / 3.0 * (std::f32::consts::SQRT_2 - 1.0);
        let (x, y) = (Vec2::new(r.x, 0.0), Vec2::new(0.0, r.y));
        self.move_to(c + x);
        self.cubic_to(c + x + y * k, c + y + x * k, c + y);
        self.cubic_to(c + y - x * k, c - x + y * k, c - x);
        self.cubic_to(c - x - y * k, c - y - x * k, c - y);
        self.cubic_to(c - y + x * k, c + x - y * k, c + x);
        self.close()
    }

    /// Whether its verbs and points agree (each verb has its points, every
    /// it starts with a move) and every point is finite.
    pub fn check(&self) -> Result<(), String> {
        if self.verbs.first().is_some_and(|v| *v != UiVerb::Move) {
            return Err("a path starts with a move".into());
        }
        let at: usize = self.verbs.iter().map(|v| v.points()).sum();
        if at != self.points.len() {
            return Err(format!(
                "a path's verbs take {at} points, it has {}",
                self.points.len()
            ));
        }
        if let Some(p) = self.points.iter().find(|p| !p.is_finite()) {
            return Err(format!("a path's point {p} is not finite"));
        }
        Ok(())
    }

    /// The box of its points (controls included, so it holds the curves):
    /// min and max; `None` when it has none.
    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        let mut it = self.points.iter().filter(|p| p.is_finite());
        let first = *it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(*p), hi.max(*p))))
    }

    /// Every point through `f` (an affine map keeps the curves exact).
    pub fn map(&self, f: impl Fn(Vec2) -> Vec2) -> UiPath {
        UiPath {
            verbs: self.verbs.clone(),
            points: self.points.iter().map(|p| f(*p)).collect(),
            fill_rule: self.fill_rule,
        }
    }

    /// The path `t` of the way from this one to `to` (path morphing): the
    /// points interpolated one by one, `None` unless both have the same
    /// verbs. `t` is not clamped, so an overshooting ease overshoots.
    pub fn lerp(&self, to: &UiPath, t: f32) -> Option<UiPath> {
        if self.verbs != to.verbs || self.points.len() != to.points.len() {
            return None;
        }
        Some(UiPath {
            verbs: self.verbs.clone(),
            points: self
                .points
                .iter()
                .zip(&to.points)
                .map(|(a, b)| *a + (*b - *a) * t)
                .collect(),
            fill_rule: if t < 0.5 {
                self.fill_rule
            } else {
                to.fill_rule
            },
        })
    }

    /// Each contour as quadratics, closed back to its start; cubics are
    /// split until each quadratic is within `tolerance` of its piece.
    pub(crate) fn quads(&self, tolerance: f32) -> Vec<Quad> {
        let mut out = Vec::new();
        let mut pts = self.points.iter().copied();
        let (mut start, mut at) = (Vec2::ZERO, Vec2::ZERO);
        let mut open = false;
        let close = |out: &mut Vec<Quad>, at: Vec2, start: Vec2| {
            if at != start {
                out.push(line(at, start));
            }
        };
        for v in &self.verbs {
            // Drawing on after a close starts a contour where it closed (SVG).
            if !open && matches!(v, UiVerb::Line | UiVerb::Quad | UiVerb::Cubic) {
                start = at;
                open = true;
            }
            match v {
                UiVerb::Move => {
                    if open {
                        close(&mut out, at, start);
                    }
                    start = pts.next().unwrap_or(at);
                    at = start;
                    open = true;
                }
                UiVerb::Line => {
                    let p = pts.next().unwrap_or(at);
                    if p != at {
                        out.push(line(at, p));
                    }
                    at = p;
                }
                UiVerb::Quad => {
                    let c = pts.next().unwrap_or(at);
                    let p = pts.next().unwrap_or(c);
                    out.push(quad(at, c, p));
                    at = p;
                }
                UiVerb::Cubic => {
                    let c1 = pts.next().unwrap_or(at);
                    let c2 = pts.next().unwrap_or(c1);
                    let p = pts.next().unwrap_or(c2);
                    cubic_to_quads(at, c1, c2, p, tolerance, &mut out);
                    at = p;
                }
                UiVerb::Close => {
                    if open {
                        close(&mut out, at, start);
                    }
                    at = start;
                    open = false;
                }
            }
        }
        if open {
            close(&mut out, at, start);
        }
        out.retain(|q| q.p.iter().all(|p| p.is_finite()));
        out
    }

    /// Each contour flattened into a polyline no further than `tolerance`
    /// from the curves, and whether it is closed: what a line along the
    /// path (a [`super::UiGeometry::Polyline`] with joins and caps) draws.
    pub fn flatten(&self, tolerance: f32) -> Vec<(Vec<Vec2>, bool)> {
        let mut out: Vec<(Vec<Vec2>, bool)> = Vec::new();
        let mut cur: Vec<Vec2> = Vec::new();
        let mut pts = self.points.iter().copied();
        let mut at = Vec2::ZERO;
        let flush = |cur: &mut Vec<Vec2>, out: &mut Vec<(Vec<Vec2>, bool)>, closed: bool| {
            if cur.len() > 1 {
                out.push((std::mem::take(cur), closed));
            }
            cur.clear();
        };
        for v in &self.verbs {
            match v {
                UiVerb::Move => {
                    flush(&mut cur, &mut out, false);
                    at = pts.next().unwrap_or(at);
                    cur.push(at);
                }
                UiVerb::Line => {
                    at = pts.next().unwrap_or(at);
                    cur.push(at);
                }
                UiVerb::Quad => {
                    let c = pts.next().unwrap_or(at);
                    let p = pts.next().unwrap_or(c);
                    cur.extend(super::flatten_quad(at, c, p, tolerance).into_iter().skip(1));
                    at = p;
                }
                UiVerb::Cubic => {
                    let c1 = pts.next().unwrap_or(at);
                    let c2 = pts.next().unwrap_or(c1);
                    let p = pts.next().unwrap_or(c2);
                    cur.extend(
                        super::flatten_cubic(at, c1, c2, p, tolerance)
                            .into_iter()
                            .skip(1),
                    );
                    at = p;
                }
                UiVerb::Close => {
                    if let Some(first) = cur.first().copied() {
                        if cur.len() > 1 && cur.last() == Some(&first) {
                            cur.pop();
                        }
                        flush(&mut cur, &mut out, true);
                        at = first;
                        cur.push(first);
                    }
                }
            }
        }
        flush(&mut cur, &mut out, false);
        out
    }
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

/// A cubic as quadratics within `tolerance` of it: kurbo's midpoint
/// approximation (`CubicBez::to_quads`), whose error over n equal pieces is
/// √3/36 · |p3 − 3p2 + 3p1 − p0| / n³.
fn cubic_to_quads(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, tolerance: f32, out: &mut Vec<Quad>) {
    let err = ((3.0 * p2 - p3) - (3.0 * p1 - p0)).length_squared();
    let max = 432.0 * tolerance.max(1e-4).powi(2);
    let n = ((err / max).powf(1.0 / 6.0).ceil() as usize).clamp(1, 64);
    let at = |t: f32| {
        let u = 1.0 - t;
        p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
    };
    // The derivative, for each piece's end tangents.
    let d = |t: f32| {
        let u = 1.0 - t;
        3.0 * ((p1 - p0) * (u * u) + (p2 - p1) * (2.0 * u * t) + (p3 - p2) * (t * t))
    };
    for i in 0..n {
        let (t0, t1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
        let (a, b) = (at(t0), at(t1));
        let h = (t1 - t0) / 3.0;
        // The piece's own controls, then the midpoint quadratic of them.
        let (c1, c2) = (a + d(t0) * h, b - d(t1) * h);
        let c = (3.0 * (c1 + c2) - a - b) * 0.25;
        out.push(quad(a, c, b));
    }
}

/// What a path writes into the pass's point table, in the shape's local
/// space: its curves, two records each (p0 p1; p2 and 1 for a line), a
/// header (the curves' box; the first band's top and the margin), a head
/// per band (where its curve list starts, its length), then the lists,
/// four curve indices a record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PathGpu {
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
pub(crate) fn write(
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

    fn square(path: &mut UiPath, at: Vec2, size: f32, clockwise: bool) {
        let c = [
            at,
            at + Vec2::new(size, 0.0),
            at + Vec2::splat(size),
            at + Vec2::new(0.0, size),
        ];
        path.move_to(c[0]);
        if clockwise {
            path.line_to(c[1]).line_to(c[2]).line_to(c[3]);
        } else {
            path.line_to(c[3]).line_to(c[2]).line_to(c[1]);
        }
        path.close();
    }

    #[test]
    fn a_cubic_becomes_quadratics_within_its_tolerance() {
        let (p0, p1, p2, p3) = (
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 0.0),
        );
        let mut out = Vec::new();
        cubic_to_quads(p0, p1, p2, p3, 0.05, &mut out);
        assert!(out.len() > 1 && out.len() < 16, "{}", out.len());
        let cubic = |t: f32| {
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        };
        let n = out.len();
        for (i, q) in out.iter().enumerate() {
            for k in 0..=8 {
                let s = k as f32 / 8.0;
                let u = 1.0 - s;
                let on = q.p[0] * u * u + q.p[1] * 2.0 * u * s + q.p[2] * s * s;
                // The quadratic's point against the cubic at the same share
                // of its piece: an upper bound on the distance between them.
                let d = on.distance(cubic((i as f32 + s) / n as f32));
                assert!(d < 0.1, "piece {i} at {s}: {d}");
            }
        }
        assert_eq!(out[0].p[0], p0);
        assert!(out[n - 1].p[2].distance(p3) < 1e-4);
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
            let mut p = UiPath::new();
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
        let mut p = UiPath::new();
        p.ellipse(Vec2::new(10.0, -4.0), Vec2::splat(60.0));
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
            let mut p = UiPath::new();
            p.move_to(v(0));
            for k in 1..5 {
                p.line_to(v(k));
            }
            p.close();
            p.fill_rule = rule;
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
        let mut a = UiPath::new();
        a.ellipse(Vec2::ZERO, Vec2::splat(10.0));
        let b = a.map(|p| p * 3.0);
        let mid = a.lerp(&b, 0.5).expect("same verbs");
        assert_eq!(mid.points[0], Vec2::new(20.0, 0.0));
        let mut other = UiPath::new();
        square(&mut other, Vec2::ZERO, 1.0, true);
        assert!(a.lerp(&other, 0.5).is_none());
        assert!(a.check().is_ok());
        let mut bad = a.clone();
        bad.points.pop();
        assert!(bad.check().is_err());
    }

    #[test]
    fn a_path_flattens_into_its_contours() {
        let mut p = UiPath::new();
        square(&mut p, Vec2::ZERO, 10.0, true);
        p.move_to(Vec2::new(20.0, 0.0)).cubic_to(
            Vec2::new(20.0, 10.0),
            Vec2::new(30.0, 10.0),
            Vec2::new(30.0, 0.0),
        );
        let lines = p.flatten(0.1);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0.len(), 4);
        assert!(lines[0].1);
        assert!(!lines[1].1);
        assert_eq!(lines[1].0.last(), Some(&Vec2::new(30.0, 0.0)));
    }
}
