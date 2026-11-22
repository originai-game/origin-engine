//! Garland and Heckbert's error quadrics, and the cost of a bend in the attributes they carry.

use super::*;

/// The largest difference between two vertices' weights on any bone.
pub(crate) fn weight_step(a: &Weights, b: &Weights) -> f64 {
    let mut d: f64 = 0.0;
    for (bone, x) in a.pairs() {
        let y = b.pairs().find(|p| p.0 == bone).map_or(0.0, |p| p.1);
        d = d.max((x - y).abs());
    }
    for (bone, y) in b.pairs() {
        if !a.pairs().any(|p| p.0 == bone) {
            d = d.max(y);
        }
    }
    d
}

/// The bend scale, as a fraction of the solid's bounding-box diagonal: a
/// collapse whose removed vertex differs from a neighbour's weights by `Δ`
/// on some bone costs as if it moved the surface by `Δ ×` this. The
/// smallest of the scales swept on the acceptance meshes (0.005 to 0.1,
/// `docs/research.md` §11.5): all keep a bending joint exact, and larger
/// ones cost more surface error elsewhere.
pub(crate) const BEND_FRACTION: f64 = 0.005;

/// A symmetric 4×4 quadric, `[a², ab, ac, ad, b², bc, bd, c², cd, d²]`
/// summed over planes, and the weight (area) behind it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Quadric {
    pub(crate) q: [f64; 10],
    pub(crate) w: f64,
}

impl Quadric {
    pub(crate) fn plane(n: DVec3, p: DVec3, w: f64) -> Quadric {
        let d = -n.dot(p);
        let (a, b, c) = (n.x, n.y, n.z);
        Quadric {
            q: [
                a * a * w,
                a * b * w,
                a * c * w,
                a * d * w,
                b * b * w,
                b * c * w,
                b * d * w,
                c * c * w,
                c * d * w,
                d * d * w,
            ],
            w,
        }
    }

    pub(crate) fn add(&mut self, o: &Quadric) {
        for k in 0..10 {
            self.q[k] += o.q[k];
        }
        self.w += o.w;
    }

    /// Weighted sum of squared distances of `p` to the planes.
    pub(crate) fn eval(&self, p: DVec3) -> f64 {
        let q = &self.q;
        let (x, y, z) = (p.x, p.y, p.z);
        let v = q[0] * x * x
            + 2.0 * q[1] * x * y
            + 2.0 * q[2] * x * z
            + 2.0 * q[3] * x
            + q[4] * y * y
            + 2.0 * q[5] * y * z
            + 2.0 * q[6] * y
            + q[7] * z * z
            + 2.0 * q[8] * z
            + q[9];
        v.max(0.0)
    }
}
