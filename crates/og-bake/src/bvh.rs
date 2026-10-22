//! A bounding volume hierarchy over the scene's triangles: binned SAH,
//! flattened depth first, closest-hit and any-hit queries (Möller–Trumbore).

use glam::Vec3;

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

#[derive(Clone, Copy, Debug)]
struct Node {
    min: Vec3,
    max: Vec3,
    /// A leaf: its first triangle in `order`; an inner node: its second child
    /// (the first follows it).
    start: u32,
    /// Triangles in a leaf, 0 for an inner node.
    count: u32,
}

/// Triangles as three corners each, and the tree over them.
pub struct Bvh {
    corners: Vec<[Vec3; 3]>,
    nodes: Vec<Node>,
    /// Triangle indices in leaf order.
    order: Vec<u32>,
}

const LEAF: usize = 4;
const BINS: usize = 12;

fn grow(min: &mut Vec3, max: &mut Vec3, p: Vec3) {
    *min = min.min(p);
    *max = max.max(p);
}

fn area(min: Vec3, max: Vec3) -> f32 {
    let d = (max - min).max(Vec3::ZERO);
    d.x * d.y + d.y * d.z + d.z * d.x
}

impl Bvh {
    /// The tree over `corners` (one entry per triangle).
    pub fn new(corners: Vec<[Vec3; 3]>) -> Bvh {
        let n = corners.len();
        let centroids: Vec<Vec3> = corners.iter().map(|c| (c[0] + c[1] + c[2]) / 3.0).collect();
        let mut bvh = Bvh {
            corners,
            nodes: Vec::with_capacity(2 * n.max(1)),
            order: (0..n as u32).collect(),
        };
        if n > 0 {
            bvh.build(0, n, &centroids);
        }
        bvh
    }

    pub fn len(&self) -> usize {
        self.corners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.corners.is_empty()
    }

    fn bounds(&self, range: &[u32]) -> (Vec3, Vec3) {
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for &t in range {
            for p in self.corners[t as usize] {
                grow(&mut min, &mut max, p);
            }
        }
        (min, max)
    }

    /// Builds the node of order[start..end] and its subtree; returns its index.
    fn build(&mut self, start: usize, end: usize, centroids: &[Vec3]) -> usize {
        let (min, max) = self.bounds(&self.order[start..end]);
        let index = self.nodes.len();
        self.nodes.push(Node {
            min,
            max,
            start: start as u32,
            count: (end - start) as u32,
        });
        let count = end - start;
        if count <= LEAF {
            return index;
        }
        // The centroids' bounds pick the bins.
        let (mut cmin, mut cmax) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for &t in &self.order[start..end] {
            grow(&mut cmin, &mut cmax, centroids[t as usize]);
        }
        let extent = cmax - cmin;
        let mut best = (f32::MAX, 0usize, 0usize);
        for axis in 0..3 {
            if extent[axis] <= 1e-9 {
                continue;
            }
            let mut bins = [(Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0u32); BINS];
            let scale = BINS as f32 / extent[axis];
            for &t in &self.order[start..end] {
                let b =
                    (((centroids[t as usize][axis] - cmin[axis]) * scale) as usize).min(BINS - 1);
                for p in self.corners[t as usize] {
                    grow(&mut bins[b].0, &mut bins[b].1, p);
                }
                bins[b].2 += 1;
            }
            // Sweep from the left and the right.
            let mut left = [(0.0f32, 0u32); BINS];
            let (mut lmin, mut lmax, mut ln) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0u32);
            for i in 0..BINS {
                if bins[i].2 > 0 {
                    lmin = lmin.min(bins[i].0);
                    lmax = lmax.max(bins[i].1);
                }
                ln += bins[i].2;
                left[i] = (if ln > 0 { area(lmin, lmax) } else { 0.0 }, ln);
            }
            let (mut rmin, mut rmax, mut rn) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0u32);
            for i in (1..BINS).rev() {
                if bins[i].2 > 0 {
                    rmin = rmin.min(bins[i].0);
                    rmax = rmax.max(bins[i].1);
                }
                rn += bins[i].2;
                let (la, lc) = left[i - 1];
                if lc == 0 || rn == 0 {
                    continue;
                }
                let cost = la * lc as f32 + area(rmin, rmax) * rn as f32;
                if cost < best.0 {
                    best = (cost, axis, i);
                }
            }
        }
        let parent_cost = area(min, max) * count as f32;
        let mid = if best.0 < parent_cost && best.0 < f32::MAX {
            let (axis, split) = (best.1, best.2);
            let scale = BINS as f32 / extent[axis];
            let slice = &mut self.order[start..end];
            let mut i = 0;
            for j in 0..slice.len() {
                let b = (((centroids[slice[j] as usize][axis] - cmin[axis]) * scale) as usize)
                    .min(BINS - 1);
                if b < split {
                    slice.swap(i, j);
                    i += 1;
                }
            }
            start + i
        } else if count > LEAF * 4 {
            // No useful split (stacked centroids): halve by the widest axis.
            let axis = if extent.x >= extent.y && extent.x >= extent.z {
                0
            } else if extent.y >= extent.z {
                1
            } else {
                2
            };
            self.order[start..end].sort_by(|&a, &b| {
                centroids[a as usize][axis]
                    .total_cmp(&centroids[b as usize][axis])
                    .then(a.cmp(&b))
            });
            start + count / 2
        } else {
            return index;
        };
        if mid == start || mid == end {
            return index;
        }
        self.nodes[index].count = 0;
        self.build(start, mid, centroids);
        let second = self.build(mid, end, centroids);
        self.nodes[index].start = second as u32;
        index
    }

    fn slab(node: &Node, origin: Vec3, inv: Vec3, t_max: f32) -> Option<f32> {
        let t0 = (node.min - origin) * inv;
        let t1 = (node.max - origin) * inv;
        let near = t0.min(t1).max_element().max(0.0);
        let far = t0.max(t1).min_element().min(t_max);
        (near <= far).then_some(near)
    }

    fn triangle(&self, t: u32, origin: Vec3, dir: Vec3, t_max: f32) -> Option<Hit> {
        let [a, b, c] = self.corners[t as usize];
        let e1 = b - a;
        let e2 = c - a;
        let p = dir.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-12 {
            return None;
        }
        let inv = 1.0 / det;
        let s = origin - a;
        let u = s.dot(p) * inv;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }
        let q = s.cross(e1);
        let v = dir.dot(q) * inv;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let dist = e2.dot(q) * inv;
        (dist > 0.0 && dist < t_max).then_some(Hit {
            triangle: t,
            t: dist,
            u,
            v,
            back: det < 0.0,
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
                for &t in &self.order[node.start as usize..(node.start + node.count) as usize] {
                    if let Some(h) = self.triangle(t, origin, dir, limit) {
                        limit = h.t;
                        best = Some(h);
                    }
                }
                continue;
            }
            // Nearer child last, so it is visited first.
            let first = stack[sp] + 1;
            let second = node.start;
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
                for &t in &self.order[node.start as usize..(node.start + node.count) as usize] {
                    if self.triangle(t, origin, dir, t_max).is_some() {
                        return true;
                    }
                }
                continue;
            }
            if sp + 2 <= stack.len() {
                stack[sp] = i + 1;
                stack[sp + 1] = node.start;
                sp += 2;
            }
        }
        false
    }
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
}
