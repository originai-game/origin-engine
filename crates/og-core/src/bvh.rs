//! A bounding volume hierarchy over triangles: binned SAH splits into
//! leaves of at most [`LEAF`] triangles, flattened depth first, and the
//! ray tests its traversals share. Each use (og-render-data's mesh
//! raycasts, the light bake's path tracer) keeps its own hit record and
//! traversal.

use glam::Vec3;

/// The most triangles a leaf holds.
pub const LEAF: usize = 4;
const BINS: usize = 12;

/// One node of the tree, in its depth-first order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhNode {
    pub min: Vec3,
    pub max: Vec3,
    /// A leaf: the first of `count` entries of [`build`]'s order. An inner
    /// node (`count` 0): its right child's index; the left one follows it.
    pub first: u32,
    pub count: u32,
}

type Node = BvhNode;

/// The tree over `corners` (one entry per triangle) and the triangle
/// indices in leaf order; a triangle with a corner that is not finite is
/// in neither.
pub fn build(corners: &[[Vec3; 3]]) -> (Vec<BvhNode>, Vec<u32>) {
    let mut tris: Vec<u32> = (0..corners.len() as u32)
        .filter(|&i| corners[i as usize].iter().all(|p| p.is_finite()))
        .collect();
    let centroids: Vec<Vec3> = corners.iter().map(|c| (c[0] + c[1] + c[2]) / 3.0).collect();
    let mut nodes = Vec::with_capacity(tris.len() / 2 + 1);
    if !tris.is_empty() {
        let len = tris.len();
        build_node(&mut nodes, &mut tris, 0, len, corners, &centroids);
    }
    (nodes, tris)
}

/// Where the ray enters a box (0 inside), if it does before `limit`. An
/// axis the ray runs parallel to bounds nothing, but a ray outside the
/// box's slab on it misses (a ray along a face's plane hits that face's
/// edges, as the triangle test does).
pub fn slab(min: Vec3, max: Vec3, origin: Vec3, inv: Vec3, limit: f32) -> Option<f32> {
    let (mut near, mut far) = (0.0f32, limit);
    for k in 0..3 {
        if inv[k].is_infinite() {
            if origin[k] < min[k] || origin[k] > max[k] {
                return None;
            }
            continue;
        }
        let t0 = (min[k] - origin[k]) * inv[k];
        let t1 = (max[k] - origin[k]) * inv[k];
        near = near.max(t0.min(t1));
        far = far.min(t0.max(t1));
    }
    (near <= far).then_some(near)
}

/// Möller-Trumbore: the ray's t and the barycentric weights of b and c.
pub fn triangle(c: &[Vec3; 3], origin: Vec3, dir: Vec3) -> Option<(f32, f32, f32)> {
    let e1 = c[1] - c[0];
    let e2 = c[2] - c[0];
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = origin - c[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t >= 0.0).then_some((t, u, v))
}

fn bounds_of(tris: &[u32], corners: &[[Vec3; 3]]) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for &t in tris {
        for p in corners[t as usize] {
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    (lo, hi)
}

fn area(lo: Vec3, hi: Vec3) -> f32 {
    let d = (hi - lo).max(Vec3::ZERO);
    d.x * d.y + d.y * d.z + d.z * d.x
}

/// Builds the node over `tris[from..to]` at the end of `nodes`.
fn build_node(
    nodes: &mut Vec<Node>,
    tris: &mut [u32],
    from: usize,
    to: usize,
    corners: &[[Vec3; 3]],
    centroids: &[Vec3],
) {
    let (min, max) = bounds_of(&tris[from..to], corners);
    let me = nodes.len();
    nodes.push(Node {
        min,
        max,
        first: from as u32,
        count: (to - from) as u32,
    });
    let count = to - from;
    if count <= LEAF {
        return;
    }
    // The centroids' extent: the bins' axis and span.
    let mut clo = Vec3::splat(f32::INFINITY);
    let mut chi = Vec3::splat(f32::NEG_INFINITY);
    for &t in &tris[from..to] {
        clo = clo.min(centroids[t as usize]);
        chi = chi.max(centroids[t as usize]);
    }
    let mut best: Option<(f32, usize, f32)> = None;
    for axis in 0..3 {
        let span = chi[axis] - clo[axis];
        if span <= 1e-12 {
            continue;
        }
        let mut bins = [(
            0usize,
            Vec3::splat(f32::INFINITY),
            Vec3::splat(f32::NEG_INFINITY),
        ); BINS];
        let scale = BINS as f32 / span;
        for &t in &tris[from..to] {
            let b = (((centroids[t as usize][axis] - clo[axis]) * scale) as usize).min(BINS - 1);
            let bin = &mut bins[b];
            bin.0 += 1;
            for p in corners[t as usize] {
                bin.1 = bin.1.min(p);
                bin.2 = bin.2.max(p);
            }
        }
        // Sweep: the cost of splitting after bin k.
        let mut left = [(0usize, 0.0f32); BINS];
        let (mut n, mut lo, mut hi) = (
            0,
            Vec3::splat(f32::INFINITY),
            Vec3::splat(f32::NEG_INFINITY),
        );
        for k in 0..BINS - 1 {
            n += bins[k].0;
            lo = lo.min(bins[k].1);
            hi = hi.max(bins[k].2);
            left[k] = (n, if n > 0 { area(lo, hi) } else { 0.0 });
        }
        let (mut n, mut lo, mut hi) = (
            0,
            Vec3::splat(f32::INFINITY),
            Vec3::splat(f32::NEG_INFINITY),
        );
        for k in (1..BINS).rev() {
            n += bins[k].0;
            lo = lo.min(bins[k].1);
            hi = hi.max(bins[k].2);
            let (ln, la) = left[k - 1];
            if ln == 0 || n == 0 {
                continue;
            }
            let cost = ln as f32 * la + n as f32 * area(lo, hi);
            if best.is_none_or(|b| cost < b.0) {
                best = Some((cost, axis, clo[axis] + k as f32 / scale));
            }
        }
    }
    let leaf_cost = count as f32 * area(min, max);
    let mid = match best {
        Some((cost, axis, at)) if cost < leaf_cost || count > LEAF * 8 => {
            let slice = &mut tris[from..to];
            let mut i = 0;
            for j in 0..slice.len() {
                if centroids[slice[j] as usize][axis] < at {
                    slice.swap(i, j);
                    i += 1;
                }
            }
            from + i
        }
        // Coincident centroids: halve by order, so the depth stays bounded.
        None if count > LEAF * 8 => from + count / 2,
        _ => return,
    };
    if mid == from || mid == to {
        return;
    }
    nodes[me].count = 0;
    build_node(nodes, tris, from, mid, corners, centroids);
    nodes[me].first = nodes.len() as u32;
    build_node(nodes, tris, mid, to, corners, centroids);
}
