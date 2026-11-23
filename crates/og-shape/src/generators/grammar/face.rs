//! The shape a rule takes: a planar polygon in a frame of its own, with
//! the mapping that gives its texture coordinates, and the polygon
//! operations the rules are built from.

use glam::{DVec2, DVec3};
use og_core::hash::splitmix64;

/// Lengths and areas under this are nothing.
pub(super) const EPS: f64 = 1e-9;

/// Texture coordinates over a face's own coordinates: `base + du * x +
/// dv * y`, in metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Uv {
    pub base: DVec2,
    pub du: DVec2,
    pub dv: DVec2,
}

impl Uv {
    pub fn at(&self, p: DVec2) -> DVec2 {
        self.base + self.du * p.x + self.dv * p.y
    }

    /// A horizontal face's: the world's x and z.
    pub fn world(origin: DVec3, x: DVec3, y: DVec3) -> Uv {
        Uv {
            base: DVec2::new(origin.x, origin.z),
            du: DVec2::new(x.x, x.z),
            dv: DVec2::new(y.x, y.z),
        }
    }

    /// A wall's: `u` along the perimeter from `u`, `v` up from `v`.
    pub fn wall(u: f64, v: f64) -> Uv {
        Uv {
            base: DVec2::new(u, v),
            du: DVec2::X,
            dv: DVec2::Y,
        }
    }

    /// The mapping of a face whose own x runs along `e` and y along `o`
    /// of this one's coordinates, from the point `at`: the pattern goes on
    /// across the fold between them.
    pub fn turned(&self, at: DVec2, e: DVec2, o: DVec2) -> Uv {
        Uv {
            base: self.at(at),
            du: self.du * e.x + self.dv * e.y,
            dv: self.du * o.x + self.dv * o.y,
        }
    }

    /// The same mapping seen from behind (x negated).
    pub fn mirrored(&self) -> Uv {
        Uv {
            base: self.base,
            du: -self.du,
            dv: self.dv,
        }
    }
}

/// A planar shape: `poly` counter-clockwise in its frame (`x` and `y`,
/// unit and at right angles), facing `n = x × y`.
#[derive(Clone, Debug)]
pub(super) struct Face {
    pub origin: DVec3,
    pub x: DVec3,
    pub y: DVec3,
    pub n: DVec3,
    pub poly: Vec<DVec2>,
    pub uv: Uv,
    /// What its choices draw from: its lot's seed and the path that
    /// reached it.
    pub id: u64,
    pub detail: u32,
}

impl Face {
    pub fn new(
        origin: DVec3,
        x: DVec3,
        y: DVec3,
        poly: Vec<DVec2>,
        uv: Uv,
        id: u64,
        detail: u32,
    ) -> Face {
        Face {
            origin,
            x,
            y,
            n: x.cross(y),
            poly,
            uv,
            id,
            detail,
        }
    }

    pub fn world(&self, p: DVec2) -> DVec3 {
        self.origin + self.x * p.x + self.y * p.y
    }

    /// A direction in its plane, in the world.
    pub fn along(&self, d: DVec2) -> DVec3 {
        self.x * d.x + self.y * d.y
    }

    /// Its least and greatest coordinates.
    pub fn bounds(&self) -> (DVec2, DVec2) {
        bounds(&self.poly)
    }

    /// The id of its `k`th child made by the operation tagged `tag`.
    pub fn child(&self, tag: u64, k: u64) -> u64 {
        splitmix64(self.id ^ splitmix64((tag << 32) ^ k))
    }

    /// The same frame and mapping over another polygon.
    pub fn with_poly(&self, poly: Vec<DVec2>, id: u64) -> Face {
        Face {
            poly,
            id,
            ..self.clone()
        }
    }

    /// Seen from behind: its frame's x negated, its polygon mirrored.
    pub fn flipped(&self, id: u64) -> Face {
        let poly = self
            .poly
            .iter()
            .rev()
            .map(|p| DVec2::new(-p.x, p.y))
            .collect();
        Face::new(
            self.origin,
            -self.x,
            self.y,
            poly,
            self.uv.mirrored(),
            id,
            self.detail,
        )
    }
}

pub(super) fn bounds(poly: &[DVec2]) -> (DVec2, DVec2) {
    poly.iter().fold(
        (DVec2::splat(f64::INFINITY), DVec2::splat(f64::NEG_INFINITY)),
        |(lo, hi), p| (lo.min(*p), hi.max(*p)),
    )
}

/// Twice the signed area: positive when counter-clockwise.
pub(super) fn area2(poly: &[DVec2]) -> f64 {
    let n = poly.len();
    (0..n).map(|i| poly[i].perp_dot(poly[(i + 1) % n])).sum()
}

/// Without repeated points (the last against the first too) or points on
/// a straight run between their neighbours.
pub(super) fn simplify(poly: &[DVec2]) -> Vec<DVec2> {
    let mut out: Vec<DVec2> = Vec::with_capacity(poly.len());
    for &p in poly {
        if out.last().is_none_or(|q| q.distance(p) > EPS) {
            out.push(p);
        }
    }
    while out.len() > 1 && out[0].distance(out[out.len() - 1]) <= EPS {
        out.pop();
    }
    loop {
        let n = out.len();
        if n < 3 {
            return out;
        }
        let straight = (0..n).find(|&i| {
            let (a, b, c) = (out[(i + n - 1) % n], out[i], out[(i + 1) % n]);
            let (u, v) = (b - a, c - b);
            u.perp_dot(v).abs() <= EPS * u.length() * v.length() && u.dot(v) > 0.0
        });
        match straight {
            Some(i) => {
                out.remove(i);
            }
            None => return out,
        }
    }
}

/// The part of `poly` where `p · g <= c` (Sutherland and Hodgman). A cut
/// along an axis lands exactly on `c`, so the strips of a split share
/// their edges bit for bit.
pub(super) fn clip(poly: &[DVec2], g: DVec2, c: f64) -> Vec<DVec2> {
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (dp, dq) = (p.dot(g) - c, q.dot(g) - c);
        if dp <= 0.0 {
            out.push(p);
        }
        if (dp < 0.0 && dq > 0.0) || (dp > 0.0 && dq < 0.0) {
            let t = dp / (dp - dq);
            let mut r = p + (q - p) * t;
            if g.y == 0.0 {
                r.x = c / g.x;
            } else if g.x == 0.0 {
                r.y = c / g.y;
            }
            out.push(r);
        }
    }
    out
}

/// Whether a counter-clockwise polygon turns left (or runs straight) at
/// every corner.
pub(super) fn convex(poly: &[DVec2]) -> bool {
    let n = poly.len();
    (0..n).all(|i| {
        let (a, b, c) = (poly[i], poly[(i + 1) % n], poly[(i + 2) % n]);
        let (u, v) = (b - a, c - b);
        u.perp_dot(v) >= -EPS * u.length() * v.length()
    })
}

/// Edge `i`'s direction and its normal into the polygon.
pub(super) fn edge(poly: &[DVec2], i: usize) -> (DVec2, DVec2) {
    let e = (poly[(i + 1) % poly.len()] - poly[i]).normalize();
    (e, e.perp())
}

/// Every edge of a counter-clockwise polygon moved `w` into it (out of it
/// for a negative `w`), its corners where the moved edges meet; `None`
/// when an edge turns round or the polygon vanishes.
pub(super) fn offset(poly: &[DVec2], w: f64) -> Option<Vec<DVec2>> {
    let n = poly.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let j = (i + n - 1) % n;
        let (ej, nj) = edge(poly, j);
        let (ei, ni) = edge(poly, i);
        let pj = poly[j] + nj * w;
        let pi = poly[i] + ni * w;
        let den = ej.perp_dot(ei);
        if den.abs() <= EPS {
            return None;
        }
        out.push(pj + ej * ((pi - pj).perp_dot(ei) / den));
    }
    let turned = (0..n).any(|i| {
        let (e, _) = edge(poly, i);
        (out[(i + 1) % n] - out[i]).dot(e) <= EPS
    });
    (!turned && area2(&out) > EPS).then_some(out)
}

/// A draw in [0, 1) from a hash.
pub(super) fn unit(h: u64) -> f64 {
    (h >> 11) as f64 / (1u64 << 53) as f64
}
