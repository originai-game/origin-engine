//! Overlap removal for a glyph outline before its distance field: the
//! nonzero union of its contours, kept as Bézier curves.
//!
//! Fonts drawn with overlapping contours (the TrueType OVERLAP flag:
//! variable-font instances, CJK strokes) and contours that cross
//! themselves (Inter's "e") put edges inside the fill; a distance field of
//! the raw outline says "edge here" along them, a seam inside the glyph.
//! fdsm has no overlap support, and msdfgen's overlapping-contour combiner
//! leaves a self-crossing contour as it is, so the outline itself is
//! resolved: every segment is split where it crosses another (or itself),
//! a piece is kept when the fill differs on its two sides, and the kept
//! pieces are chained into closed contours, each the exact sub-curve of
//! the segment it came from. An outline with no crossing and no edge
//! inside the fill comes back unchanged.

use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Order, Point, Segment, Vect};
use fdsm::shape::{Contour, Shape};

/// Flatness, in font units, under which a piece of curve is its chord when
/// intersecting.
const FLAT: f64 = 0.01;
/// A crossing this close to a segment's end (font units) is that end.
const TINY: f64 = 0.05;
/// How far either side of a piece the fill is sampled (font units).
const SIDE: f64 = 0.01;
/// The largest gap (font units) between pieces chained into one contour.
const SNAP: f64 = 0.5;
/// Subdivision steps one glyph may spend finding crossings; past it (two
/// coincident curves) the outline is left as it is.
const BUDGET: usize = 400_000;

/// A Bézier segment as its control points (2 to 4).
#[derive(Clone, Copy, Debug)]
struct Bez {
    p: [Point; 4],
    n: usize,
}

impl Bez {
    fn of(s: &Segment) -> Bez {
        let n = match s.order() {
            Order::Linear => 2,
            Order::Quadratic => 3,
            Order::Cubic => 4,
        };
        let mut p = [Point::origin(); 4];
        for (i, q) in p.iter_mut().enumerate().take(n) {
            *q = s.control_point(i);
        }
        Bez { p, n }
    }

    fn segment(&self) -> Segment {
        match self.n {
            2 => Segment::line(self.p[0], self.p[1]),
            3 => Segment::quad(self.p[0], self.p[1], self.p[2]),
            _ => Segment::cubic(self.p[0], self.p[1], self.p[2], self.p[3]),
        }
    }

    fn start(&self) -> Point {
        self.p[0]
    }

    fn end(&self) -> Point {
        self.p[self.n - 1]
    }

    /// The two halves at `t` (de Casteljau).
    fn split(&self, t: f64) -> (Bez, Bez) {
        let n = self.n;
        let mut q = self.p;
        let (mut a, mut b) = (*self, *self);
        a.p[0] = q[0];
        b.p[n - 1] = q[n - 1];
        for k in 1..n {
            for i in 0..n - k {
                q[i] = q[i] + (q[i + 1] - q[i]) * t;
            }
            a.p[k] = q[0];
            b.p[n - 1 - k] = q[n - 1 - k];
        }
        (a, b)
    }

    fn reversed(&self) -> Bez {
        let mut r = *self;
        for i in 0..self.n {
            r.p[i] = self.p[self.n - 1 - i];
        }
        r
    }

    fn bbox(&self) -> (Point, Point) {
        let (mut lo, mut hi) = (self.p[0], self.p[0]);
        for q in &self.p[1..self.n] {
            lo = Point::new(lo.x.min(q.x), lo.y.min(q.y));
            hi = Point::new(hi.x.max(q.x), hi.y.max(q.y));
        }
        (lo, hi)
    }

    fn extent(&self) -> f64 {
        let (lo, hi) = self.bbox();
        (hi.x - lo.x).max(hi.y - lo.y)
    }

    /// Whether the control points lie within [`FLAT`] of the chord.
    fn flat(&self) -> bool {
        let (a, b) = (self.start(), self.end());
        let d = b - a;
        let len = d.norm();
        self.p[1..self.n - 1].iter().all(|q| {
            let off = if len > 1e-12 {
                d.perp(&(q - a)).abs() / len
            } else {
                (q - a).norm()
            };
            off <= FLAT
        })
    }
}

fn boxes_meet(a: &Bez, b: &Bez) -> bool {
    let (alo, ahi) = a.bbox();
    let (blo, bhi) = b.bbox();
    alo.x <= bhi.x + FLAT && blo.x <= ahi.x + FLAT && alo.y <= bhi.y + FLAT && blo.y <= ahi.y + FLAT
}

/// A crossing of two segments: the parameter on each and the point.
type Crossing = (f64, f64, Point);

/// The crossings of `a` (over `ta` of its segment) and `b` (over `tb`), by
/// subdivision down to chords; false when the budget ran out.
fn crossings(
    a: &Bez,
    ta: (f64, f64),
    b: &Bez,
    tb: (f64, f64),
    budget: &mut usize,
    out: &mut Vec<Crossing>,
) -> bool {
    if *budget == 0 {
        return false;
    }
    *budget -= 1;
    if !boxes_meet(a, b) {
        return true;
    }
    let fa = a.flat() || ta.1 - ta.0 < 1e-9;
    let fb = b.flat() || tb.1 - tb.0 < 1e-9;
    if fa && fb {
        chords(a, ta, b, tb, out);
        return true;
    }
    let lerp = |r: (f64, f64), t: f64| r.0 + (r.1 - r.0) * t;
    if !fa && (fb || a.extent() >= b.extent()) {
        let (a0, a1) = a.split(0.5);
        let m = lerp(ta, 0.5);
        crossings(&a0, (ta.0, m), b, tb, budget, out)
            && crossings(&a1, (m, ta.1), b, tb, budget, out)
    } else {
        let (b0, b1) = b.split(0.5);
        let m = lerp(tb, 0.5);
        crossings(a, ta, &b0, (tb.0, m), budget, out)
            && crossings(a, ta, &b1, (m, tb.1), budget, out)
    }
}

/// The crossings of two chords; for collinear ones, the ends of each that
/// lie inside the other (where a shared stretch begins and ends).
fn chords(a: &Bez, ta: (f64, f64), b: &Bez, tb: (f64, f64), out: &mut Vec<Crossing>) {
    let lerp = |r: (f64, f64), t: f64| r.0 + (r.1 - r.0) * t;
    let (p0, p1, q0, q1) = (a.start(), a.end(), b.start(), b.end());
    let (r, s) = (p1 - p0, q1 - q0);
    let (rl, sl) = (r.norm(), s.norm());
    if rl < 1e-12 || sl < 1e-12 {
        return;
    }
    let denom = r.perp(&s);
    if denom.abs() > 1e-9 * rl * sl {
        let w = q0 - p0;
        let t = w.perp(&s) / denom;
        let u = w.perp(&r) / denom;
        const E: f64 = 1e-9;
        if (-E..=1.0 + E).contains(&t) && (-E..=1.0 + E).contains(&u) {
            let t = t.clamp(0.0, 1.0);
            out.push((lerp(ta, t), lerp(tb, u.clamp(0.0, 1.0)), p0 + r * t));
        }
        return;
    }
    if r.perp(&(q0 - p0)).abs() / rl > FLAT {
        return;
    }
    // Collinear: each end inside the other chord.
    for (q, u) in [(q0, 0.0), (q1, 1.0)] {
        let t = (q - p0).dot(&r) / (rl * rl);
        if t > 0.0 && t < 1.0 {
            out.push((lerp(ta, t), lerp(tb, u), q));
        }
    }
    for (p, t) in [(p0, 0.0), (p1, 1.0)] {
        let u = (p - q0).dot(&s) / (sl * sl);
        if u > 0.0 && u < 1.0 {
            out.push((lerp(ta, t), lerp(tb, u), p));
        }
    }
}

/// The nonzero union of `shape`'s contours (font units), or `shape` itself
/// when nothing in it overlaps or the union cannot be formed.
pub(super) fn remove_overlaps(shape: Shape<Contour>) -> Shape<Contour> {
    resolve(&shape).unwrap_or(shape)
}

fn resolve(shape: &Shape<Contour>) -> Option<Shape<Contour>> {
    for (ci, c) in shape.contours.iter().enumerate() {
        for s in &c.segments {
            let b = Bez::of(s);
            let near = |p: Point| (p.x - 855.0).abs() < 200.0 && (p.y - 523.0).abs() < 200.0;
            if near(b.start()) || near(b.end()) {
                eprintln!("DBGS {ci} {:?}", &b.p[..b.n]);
            }
        }
    }
    let segs: Vec<Bez> = shape
        .contours
        .iter()
        .flat_map(|c| c.segments.iter().map(Bez::of))
        .collect();
    // Where each segment is cut: (parameter, point).
    let mut cuts: Vec<Vec<(f64, Point)>> = vec![Vec::new(); segs.len()];
    let mut budget = BUDGET;
    let mut found = Vec::new();
    for i in 0..segs.len() {
        for j in i + 1..segs.len() {
            if !boxes_meet(&segs[i], &segs[j]) {
                continue;
            }
            found.clear();
            if !crossings(
                &segs[i],
                (0.0, 1.0),
                &segs[j],
                (0.0, 1.0),
                &mut budget,
                &mut found,
            ) {
                return None;
            }
            for &(t, u, p) in &found {
                for (k, v) in [(i, t), (j, u)] {
                    let s = &segs[k];
                    if (p - s.start()).norm() > TINY && (p - s.end()).norm() > TINY {
                        cuts[k].push((v, p));
                    }
                }
            }
        }
    }
    let mut split = false;
    let mut pieces: Vec<Bez> = Vec::new();
    for (s, cut) in segs.iter().zip(&mut cuts) {
        cut.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut rest = *s;
        let mut done = 0.0;
        let mut last = s.start();
        for &(t, p) in cut.iter() {
            if (p - last).norm() <= TINY || t <= done || t >= 1.0 {
                continue;
            }
            let (mut a, mut b) = rest.split((t - done) / (1.0 - done));
            a.p[a.n - 1] = p;
            b.p[0] = p;
            pieces.push(a);
            rest = b;
            done = t;
            last = p;
            split = true;
        }
        pieces.push(rest);
    }

    // Keep the pieces with fill on one side only, filled side on the right
    // (TrueType's direction).
    let prepared = shape.prepare();
    let filled = |p: Point| {
        prepared
            .scanline(p.y)
            .cursor()
            .filled(p.x, FillRule::Nonzero)
    };
    let mut dropped = false;
    let mut kept: Vec<Bez> = Vec::new();
    for b in pieces {
        if b.extent() <= TINY && (b.end() - b.start()).norm() <= TINY {
            continue;
        }
        let seg = b.segment();
        let m = seg.get(0.5);
        let mut d = seg.direction_at(0.5);
        if d.norm() < 1e-12 {
            d = b.end() - b.start();
        }
        let d = d.normalize();
        let left: Vect = Vect::new(-d.y, d.x) * SIDE;
        let (l, r) = (filled(m + left), filled(m - left));
        if l == r {
            dropped = true;
            continue;
        }
        let b = if l { b.reversed() } else { b };
        // Two contours that share a stretch of edge, drawn the same way,
        // give it once.
        let near = |p: Point, q: Point| (p - q).norm() <= TINY;
        let twin = kept.iter().any(|k| {
            near(k.start(), b.start())
                && near(k.end(), b.end())
                && near(k.segment().get(0.5), b.segment().get(0.5))
        });
        if twin {
            dropped = true;
        } else {
            kept.push(b);
        }
    }
    if !split && !dropped {
        return None;
    }

    // Chain the pieces into closed contours.
    let mut used = vec![false; kept.len()];
    let mut contours = Vec::new();
    for first in 0..kept.len() {
        if used[first] {
            continue;
        }
        used[first] = true;
        let mut chain = vec![kept[first]];
        loop {
            let end = chain.last().expect("one piece").end();
            let next = (0..kept.len())
                .filter(|&k| !used[k])
                .map(|k| (k, (kept[k].start() - end).norm()))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let home = (chain[0].start() - end).norm();
            match next {
                Some((k, gap)) if gap < home && gap <= SNAP => {
                    used[k] = true;
                    let mut b = kept[k];
                    b.p[0] = end;
                    chain.push(b);
                }
                _ if home <= SNAP => {
                    let start = chain[0].start();
                    let last = chain.last_mut().expect("one piece");
                    last.p[last.n - 1] = start;
                    break;
                }
                _ => return None,
            }
        }
        contours.push(Contour {
            segments: chain.iter().map(Bez::segment).collect(),
        });
    }
    if contours.is_empty() {
        return None;
    }
    Some(Shape { contours })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Contour {
        // Clockwise in y-up, filled on the right as TrueType draws.
        let p = [
            Point::new(x0, y0),
            Point::new(x0, y1),
            Point::new(x1, y1),
            Point::new(x1, y0),
        ];
        Contour {
            segments: (0..4)
                .map(|i| Segment::line(p[i], p[(i + 1) % 4]))
                .collect(),
        }
    }

    /// Two crossing squares become one contour whose edges all lie on the
    /// boundary; two apart come back as they were.
    #[test]
    fn crossing_squares_merge_and_apart_ones_stay() {
        let crossing = Shape {
            contours: vec![
                square(0.0, 0.0, 100.0, 100.0),
                square(50.0, 50.0, 150.0, 150.0),
            ],
        };
        let union = remove_overlaps(crossing);
        assert_eq!(union.contours.len(), 1);
        let segs = &union.contours[0].segments;
        assert_eq!(segs.len(), 8, "{segs:?}");
        for s in segs {
            let m = s.get(0.5);
            let inside_both = m.x > 50.0 && m.x < 100.0 && m.y > 50.0 && m.y < 100.0;
            assert!(!inside_both, "an edge inside the fill: {s:?}");
        }
        let apart = Shape {
            contours: vec![square(0.0, 0.0, 10.0, 10.0), square(20.0, 0.0, 30.0, 10.0)],
        };
        let same = remove_overlaps(apart.clone());
        assert_eq!(
            format!("{:?}", same.contours),
            format!("{:?}", apart.contours)
        );
    }
}
