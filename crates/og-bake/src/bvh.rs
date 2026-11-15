//! The scene's triangles under og-core's bounding volume hierarchy:
//! closest-hit and any-hit queries (Möller–Trumbore), and the nearest
//! point of the surface.

use glam::Vec3;
use og_core::bvh::{BvhNode, build, slab, triangle};

/// A hit: the triangle, its barycentrics (of its second and third corners)
/// and the distance along the ray.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub triangle: u32,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    /// Whether the ray met the triangle's back (against its winding's normal).
    pub back: bool,
}

/// The point of the surface nearest a point ([`Bvh::nearest`]): its
/// triangle, its barycentrics (of its second and third corners) and how
/// far it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nearest {
    pub triangle: u32,
    pub u: f32,
    pub v: f32,
    pub distance: f32,
}

/// Triangles as three corners each, and the tree over them
/// ([`og_core::bvh`], the one the mesh raycasts use).
pub struct Bvh {
    corners: Vec<[Vec3; 3]>,
    nodes: Vec<BvhNode>,
    /// Triangle indices in leaf order.
    order: Vec<u32>,
}

impl Bvh {
    /// The tree over `corners` (one entry per triangle).
    pub fn new(corners: Vec<[Vec3; 3]>) -> Bvh {
        let (nodes, order) = build(&corners);
        Bvh {
            corners,
            nodes,
            order,
        }
    }

    pub fn len(&self) -> usize {
        self.corners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.corners.is_empty()
    }

    fn slab(node: &BvhNode, origin: Vec3, inv: Vec3, t_max: f32) -> Option<f32> {
        slab(node.min, node.max, origin, inv, t_max)
    }

    /// The triangle's hit strictly inside `0..t_max`.
    fn triangle(&self, t: u32, origin: Vec3, dir: Vec3, t_max: f32) -> Option<Hit> {
        let c = &self.corners[t as usize];
        let (dist, u, v) = triangle(c, origin, dir)?;
        (dist > 0.0 && dist < t_max).then(|| Hit {
            triangle: t,
            t: dist,
            u,
            v,
            back: (c[1] - c[0]).cross(c[2] - c[0]).dot(dir) > 0.0,
        })
    }

    /// The closest triangle along the ray within `t_max`.
    pub fn closest(&self, origin: Vec3, dir: Vec3, t_max: f32) -> Option<Hit> {
        if self.nodes.is_empty() {
            return None;
        }
        let inv = dir.recip();
        let mut best: Option<Hit> = None;
        let mut limit = t_max;
        let mut stack = [0u32; 64];
        let mut sp = 1;
        while sp > 0 {
            sp -= 1;
            let node = &self.nodes[stack[sp] as usize];
            if Self::slab(node, origin, inv, limit).is_none() {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    if let Some(h) = self.triangle(t, origin, dir, limit) {
                        limit = h.t;
                        best = Some(h);
                    }
                }
                continue;
            }
            // Nearer child last, so it is visited first.
            let first = stack[sp] + 1;
            let second = node.first;
            let d1 = Self::slab(&self.nodes[first as usize], origin, inv, limit);
            let d2 = Self::slab(&self.nodes[second as usize], origin, inv, limit);
            match (d1, d2) {
                (Some(a), Some(b)) => {
                    let (near, far) = if a <= b {
                        (first, second)
                    } else {
                        (second, first)
                    };
                    if sp + 2 <= stack.len() {
                        stack[sp] = far;
                        stack[sp + 1] = near;
                        sp += 2;
                    }
                }
                (Some(_), None) => {
                    stack[sp] = first;
                    sp += 1;
                }
                (None, Some(_)) => {
                    stack[sp] = second;
                    sp += 1;
                }
                (None, None) => {}
            }
        }
        best
    }

    /// A triangle's corners.
    pub fn corners(&self, triangle: u32) -> &[Vec3; 3] {
        &self.corners[triangle as usize]
    }

    /// The point of the surface nearest `p` within `limit` of it. Boxes
    /// farther than the best triangle found are not opened, the nearer
    /// child first; of two triangles equally near, the lower index.
    pub fn nearest(&self, p: Vec3, limit: f32) -> Option<Nearest> {
        if self.nodes.is_empty() || !p.is_finite() {
            return None;
        }
        let box_distance =
            |n: &BvhNode| (n.min - p).max(p - n.max).max(Vec3::ZERO).length_squared();
        let mut best: Option<Nearest> = None;
        let mut limit_sq = limit * limit;
        let mut stack = vec![0u32];
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i as usize];
            if box_distance(node) > limit_sq {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    let (q, u, v) = closest_on_triangle(&self.corners[t as usize], p);
                    let d = (q - p).length_squared();
                    let nearer = match &best {
                        Some(b) => d < limit_sq || (d == limit_sq && t < b.triangle),
                        None => d <= limit_sq,
                    };
                    if nearer {
                        limit_sq = d;
                        best = Some(Nearest {
                            triangle: t,
                            u,
                            v,
                            distance: d.sqrt(),
                        });
                    }
                }
                continue;
            }
            let (l, r) = (i + 1, node.first);
            if box_distance(&self.nodes[l as usize]) <= box_distance(&self.nodes[r as usize]) {
                stack.extend([r, l]);
            } else {
                stack.extend([l, r]);
            }
        }
        best
    }

    /// Whether any triangle lies along the ray within `t_max`.
    pub fn occluded(&self, origin: Vec3, dir: Vec3, t_max: f32) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let inv = dir.recip();
        let mut stack = [0u32; 64];
        let mut sp = 1;
        while sp > 0 {
            sp -= 1;
            let i = stack[sp];
            let node = &self.nodes[i as usize];
            if Self::slab(node, origin, inv, t_max).is_none() {
                continue;
            }
            if node.count > 0 {
                for &t in &self.order[node.first as usize..(node.first + node.count) as usize] {
                    if self.triangle(t, origin, dir, t_max).is_some() {
                        return true;
                    }
                }
                continue;
            }
            if sp + 2 <= stack.len() {
                stack[sp] = i + 1;
                stack[sp + 1] = node.first;
                sp += 2;
            }
        }
        false
    }
}

/// The point of a triangle nearest `p`, with the weights of its second and
/// third corners (Ericson, Real-Time Collision Detection §5.1.5: the
/// Voronoi region of a corner, an edge or the face).
fn closest_on_triangle(c: &[Vec3; 3], p: Vec3) -> (Vec3, f32, f32) {
    let [a, b, cc] = *c;
    let (ab, ac, ap) = (b - a, cc - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, 0.0, 0.0);
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return (b, 1.0, 0.0);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let u = d1 / (d1 - d3);
        return (a + ab * u, u, 0.0);
    }
    let cp = p - cc;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return (cc, 0.0, 1.0);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let v = d2 / (d2 - d6);
        return (a + ac * v, 0.0, v);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let v = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (b + (cc - b) * v, 1.0 - v, v);
    }
    let sum = va + vb + vc;
    if sum.abs() < 1e-30 {
        // A degenerate triangle: its first corner.
        return (a, 0.0, 0.0);
    }
    let (u, v) = (vb / sum, vc / sum);
    (a + ab * u + ac * v, u, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A brute-force closest hit, to compare the tree with.
    fn brute(bvh: &Bvh, o: Vec3, d: Vec3) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32)> = None;
        for t in 0..bvh.len() as u32 {
            if let Some(h) = bvh.triangle(t, o, d, f32::MAX)
                && best.is_none_or(|b| h.t < b.1)
            {
                best = Some((t, h.t));
            }
        }
        best
    }

    fn lcg(state: &mut u64) -> f32 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*state >> 40) as f32) / (1u64 << 24) as f32
    }

    #[test]
    fn the_tree_finds_what_brute_force_finds() {
        let mut s = 7u64;
        let mut tris = Vec::new();
        for _ in 0..500 {
            let c = Vec3::new(lcg(&mut s), lcg(&mut s), lcg(&mut s)) * 20.0 - 10.0;
            let r = |s: &mut u64| Vec3::new(lcg(s), lcg(s), lcg(s)) - 0.5;
            tris.push([c + r(&mut s), c + r(&mut s), c + r(&mut s)]);
        }
        let bvh = Bvh::new(tris);
        let mut hits = 0;
        for _ in 0..2000 {
            let o = Vec3::new(lcg(&mut s), lcg(&mut s), lcg(&mut s)) * 24.0 - 12.0;
            let d = (Vec3::new(lcg(&mut s), lcg(&mut s), lcg(&mut s)) - 0.5).normalize();
            let got = bvh.closest(o, d, f32::MAX).map(|h| (h.triangle, h.t));
            let want = brute(&bvh, o, d);
            assert_eq!(got.is_some(), want.is_some());
            if let (Some(g), Some(w)) = (got, want) {
                assert!((g.1 - w.1).abs() < 1e-4, "{g:?} vs {w:?}");
                hits += 1;
            }
            assert_eq!(bvh.occluded(o, d, f32::MAX), want.is_some());
        }
        assert!(hits > 50, "the rays met triangles ({hits})");
    }

    #[test]
    fn a_back_face_is_reported_as_one() {
        let bvh = Bvh::new(vec![[Vec3::ZERO, Vec3::X, Vec3::Y]]);
        // Counter-clockwise seen from +z: the front faces +z.
        let front = bvh
            .closest(Vec3::new(0.2, 0.2, 1.0), Vec3::NEG_Z, 10.0)
            .unwrap();
        assert!(!front.back);
        let back = bvh
            .closest(Vec3::new(0.2, 0.2, -1.0), Vec3::Z, 10.0)
            .unwrap();
        assert!(back.back);
        assert!((front.t - 1.0).abs() < 1e-6 && (front.u - 0.2).abs() < 1e-6);
    }

    #[test]
    fn the_nearest_point_is_the_one_brute_force_finds() {
        let mut s = 11u64;
        let mut tris = Vec::new();
        for _ in 0..300 {
            let c = Vec3::new(lcg(&mut s), lcg(&mut s), lcg(&mut s)) * 20.0 - 10.0;
            let r = |s: &mut u64| Vec3::new(lcg(s), lcg(s), lcg(s)) - 0.5;
            tris.push([c + r(&mut s), c + r(&mut s), c + r(&mut s)]);
        }
        let bvh = Bvh::new(tris.clone());
        for _ in 0..500 {
            let p = Vec3::new(lcg(&mut s), lcg(&mut s), lcg(&mut s)) * 24.0 - 12.0;
            let want = tris
                .iter()
                .map(|t| (closest_on_triangle(t, p).0 - p).length())
                .fold(f32::INFINITY, f32::min);
            let got = bvh.nearest(p, f32::INFINITY).expect("a surface");
            assert!((got.distance - want).abs() < 1e-4, "{got:?} vs {want}");
            // The weights name the point they measured.
            let [a, b, c] = tris[got.triangle as usize];
            let q = a + (b - a) * got.u + (c - a) * got.v;
            assert!(((q - p).length() - want).abs() < 1e-3);
            // Nothing is found inside a reach shorter than the distance.
            assert!(bvh.nearest(p, want * 0.5).is_none() || want < 1e-4);
        }
    }
}
