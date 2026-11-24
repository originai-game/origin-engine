//! Polygons whose edges know the street they front, and the cuts and
//! offsets the plan is made with.

use glam::DVec2;

/// Lengths and areas under this are nothing.
pub(super) const EPS: f64 = 1e-9;

/// A polygon in x and z, each edge (`points[i]` to the next) with the
/// street it fronts, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub points: Vec<DVec2>,
    pub streets: Vec<Option<usize>>,
}

impl Poly {
    /// Every edge fronting `street`.
    pub fn new(points: Vec<DVec2>, street: Option<usize>) -> Poly {
        let streets = vec![street; points.len()];
        Poly { points, streets }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Its signed area: positive when counter-clockwise.
    pub fn area(&self) -> f64 {
        area2(&self.points) / 2.0
    }

    /// Counter-clockwise, each edge keeping its street.
    pub fn ccw(self) -> Poly {
        if area2(&self.points) >= 0.0 {
            return self;
        }
        let n = self.len();
        let points = self.points.iter().rev().copied().collect();
        // Reversed, edge k runs along the old edge n - 2 - k.
        let streets = (0..n).map(|k| self.streets[(2 * n - 2 - k) % n]).collect();
        Poly { points, streets }
    }

    /// Without points repeated, or on a straight run between neighbours
    /// whose edges front the same street.
    pub fn tidy(self) -> Poly {
        let (mut pts, mut tags) = (self.points, self.streets);
        'again: while pts.len() >= 3 {
            let n = pts.len();
            for i in 0..n {
                let (h, j) = ((i + n - 1) % n, (i + 1) % n);
                let (u, v) = (pts[i] - pts[h], pts[j] - pts[i]);
                let repeated = v.length() <= EPS;
                let straight = tags[h] == tags[i]
                    && u.perp_dot(v).abs() <= EPS * u.length() * v.length()
                    && u.dot(v) > 0.0;
                if repeated || straight {
                    pts.remove(i);
                    tags.remove(i);
                    continue 'again;
                }
            }
            break;
        }
        Poly {
            points: pts,
            streets: tags,
        }
    }

    /// The part where `p · g <= c`, the edges along the cut fronting
    /// `street` (Sutherland and Hodgman).
    pub fn clip(&self, g: DVec2, c: f64, street: Option<usize>) -> Poly {
        let n = self.len();
        let mut out = Poly::new(Vec::with_capacity(n + 2), None);
        let mut push = |p: DVec2, s: Option<usize>| {
            out.points.push(p);
            out.streets.push(s);
        };
        for i in 0..n {
            let (p, q, s) = (self.points[i], self.points[(i + 1) % n], self.streets[i]);
            let (dp, dq) = (p.dot(g) - c, q.dot(g) - c);
            let cross = || p + (q - p) * (dp / (dp - dq));
            if dp <= 0.0 {
                if dq > 0.0 && dp < 0.0 {
                    push(p, s);
                    push(cross(), street);
                } else if dq > 0.0 {
                    // On the cut and leaving: the edge to where it comes
                    // back runs along the cut.
                    push(p, street);
                } else {
                    push(p, s);
                }
            } else if dq < 0.0 {
                push(cross(), s);
            }
        }
        out
    }
}

/// Twice the signed area: positive when counter-clockwise.
pub(super) fn area2(poly: &[DVec2]) -> f64 {
    let n = poly.len();
    (0..n).map(|i| poly[i].perp_dot(poly[(i + 1) % n])).sum()
}

/// Edge `i`'s direction and its normal to its left (into a
/// counter-clockwise polygon).
pub(super) fn edge(poly: &[DVec2], i: usize) -> (DVec2, DVec2) {
    let e = (poly[(i + 1) % poly.len()] - poly[i]).normalize();
    (e, e.perp())
}

/// The mean of its points.
pub(super) fn middle(poly: &[DVec2]) -> DVec2 {
    poly.iter().copied().sum::<DVec2>() / poly.len() as f64
}

/// The convex polygon inside every edge line of a convex, counter-clockwise
/// `poly` moved `d[i]` out of it (into it when negative), each of its edges
/// with the index of the edge it lies on. An edge whose line no longer
/// bounds it (a short edge between two that move out) is gone, so it never
/// turns inside out as a moved ring would. `None` when nothing is left.
pub(super) fn grown(poly: &[DVec2], d: &[f64]) -> Option<Poly> {
    let max_d = d.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    let (lo, hi) = poly.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    );
    // Outward miters at acute block corners extend up to max_d / sin(theta/2);
    // give the initial bounding box ample room so mitered vertices stay inside.
    let reach = (max_d * 20.0).max(max_d + 10.0) + (hi - lo).length();
    let (lo, hi) = (lo - reach, hi + reach);
    let mut out = Poly::new(
        vec![lo, DVec2::new(hi.x, lo.y), hi, DVec2::new(lo.x, hi.y)],
        None,
    );
    for i in 0..poly.len() {
        let (_, n) = edge(poly, i);
        out = out.clip(-n, d[i] - poly[i].dot(n), Some(i)).tidy();
        if out.len() < 3 {
            return None;
        }
    }
    (out.area() > EPS && out.streets.iter().all(Option::is_some)).then_some(out)
}

/// A counter-clockwise polygon cut into convex parts, each cut along the
/// line of an edge that ends in a reflex corner; the edges along a cut
/// front no street.
pub(super) fn convex_parts(poly: Poly) -> Vec<Poly> {
    let mut todo = vec![poly];
    let mut out = Vec::new();
    let mut guard = 0;
    while let Some(p) = todo.pop() {
        let n = p.len();
        let reflex = (0..n).find(|&i| {
            let (a, b, c) = (
                p.points[(i + n - 1) % n],
                p.points[i],
                p.points[(i + 1) % n],
            );
            let (u, v) = (b - a, c - b);
            u.perp_dot(v) < -EPS * u.length() * v.length()
        });
        guard += 1;
        match reflex {
            Some(i) if guard < 4096 => {
                let u = (p.points[i] - p.points[(i + n - 1) % n]).normalize();
                let g = u.perp();
                let c = g.dot(p.points[i]);
                for half in [p.clip(g, c, None), p.clip(-g, -c, None)] {
                    let half = half.tidy();
                    if half.len() >= 3 && half.area() > EPS {
                        todo.push(half);
                    }
                }
            }
            _ => out.push(p),
        }
    }
    out.reverse();
    out
}
