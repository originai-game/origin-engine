//! A bounding volume hierarchy over a mesh's triangles, for rays against
//! what is drawn (og.render.raycast): binned SAH splits into leaves of at
//! most [`LEAF`] triangles, flattened depth first, traversed nearest child
//! first. Built once per geometry (when it is made or updated) and kept
//! with it; a ray in the mesh's local space answers every triangle it
//! crosses, or the nearest. A cooked model carries its primitives' BVHs in
//! its blob (og-cook), and [`Bvh::from_parts`] puts one back together over
//! the decoded mesh without building anything.

use crate::scene::MeshData;
use glam::Vec3;

/// The most triangles a leaf holds.
pub const LEAF: usize = 4;
const BINS: usize = 12;

/// One node of a [`Bvh`], in its depth-first order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BvhNode {
    pub min: Vec3,
    pub max: Vec3,
    /// A leaf: the first of `count` entries of [`Bvh::order`]. An inner
    /// node (`count` 0): its right child's index; the left one follows it.
    pub first: u32,
    pub count: u32,
}

type Node = BvhNode;

/// A mesh's BVH with the triangles' corners copied in (so a ray needs no
/// mesh beside it).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bvh {
    nodes: Vec<Node>,
    /// Triangle indices in leaf order.
    tris: Vec<u32>,
    /// Per triangle (by its index in the mesh): its three corners.
    corners: Vec<[Vec3; 3]>,
    /// Per triangle: its three corners' uvs.
    uvs: Vec<[[f32; 2]; 3]>,
}

/// One triangle a ray crossed, in the mesh's local space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriangleHit {
    /// Along the ray, in units of its direction's length.
    pub t: f32,
    /// The triangle's index (three.js `faceIndex`).
    pub face: u32,
    /// The triangle's geometric normal (counter-clockwise front), unit length.
    pub normal: Vec3,
    /// The hit's barycentric weights of corners b and c.
    pub bary: [f32; 2],
    /// The uv there, interpolated.
    pub uv: [f32; 2],
}

impl Bvh {
    pub fn build(mesh: &MeshData) -> Bvh {
        BUILDS.with(|b| b.set(b.get() + 1));
        let (corners, uvs) = corners_of(mesh);
        let mut tris: Vec<u32> = (0..corners.len() as u32)
            .filter(|&i| corners[i as usize].iter().all(|p| p.is_finite()))
            .collect();
        let centroids: Vec<Vec3> = corners.iter().map(|c| (c[0] + c[1] + c[2]) / 3.0).collect();
        let mut nodes = Vec::with_capacity(tris.len() / 2 + 1);
        if !tris.is_empty() {
            let len = tris.len();
            build_node(&mut nodes, &mut tris, 0, len, &corners, &centroids);
        }
        Bvh {
            nodes,
            tris,
            corners,
            uvs,
        }
    }

    /// A BVH built earlier (by [`Bvh::build`] over the same mesh, then
    /// stored: a cooked model's), put back together over `mesh` without
    /// building anything: the nodes and the triangle order as
    /// [`Bvh::node_list`] and [`Bvh::order`] gave them. Refused when they do
    /// not fit the mesh (an index past its triangles, a child that is not
    /// after its parent), so a ray never leaves them.
    pub fn from_parts(mesh: &MeshData, nodes: Vec<BvhNode>, tris: Vec<u32>) -> Result<Bvh, String> {
        let (corners, uvs) = corners_of(mesh);
        if let Some(t) = tris.iter().find(|&&t| t as usize >= corners.len()) {
            return Err(format!(
                "triangle {t} of the BVH is past the mesh's {}",
                corners.len()
            ));
        }
        let n = nodes.len();
        for (i, node) in nodes.iter().enumerate() {
            let ok = if node.count > 0 {
                node.first as usize + node.count as usize <= tris.len()
            } else {
                i + 1 < n && (node.first as usize) > i + 1 && (node.first as usize) < n
            };
            if !ok || !(node.min.is_finite() && node.max.is_finite()) {
                return Err(format!(
                    "BVH node {i} does not fit its {n} nodes and {} triangles",
                    tris.len()
                ));
            }
        }
        if n == 0 && !tris.is_empty() {
            return Err("a BVH of triangles without nodes".into());
        }
        Ok(Bvh {
            nodes,
            tris,
            corners,
            uvs,
        })
    }

    /// Its nodes, depth first (the root first).
    pub fn node_list(&self) -> &[BvhNode] {
        &self.nodes
    }

    /// The triangle indices in leaf order.
    pub fn order(&self) -> &[u32] {
        &self.tris
    }

    /// How many BVHs [`Bvh::build`] has built on this thread: what a load
    /// that should build none is measured by.
    pub fn builds() -> u64 {
        BUILDS.with(|b| b.get())
    }

    /// How many triangles it holds.
    pub fn triangles(&self) -> usize {
        self.tris.len()
    }

    /// How many nodes it has (a measure of its build).
    pub fn nodes(&self) -> usize {
        self.nodes.len()
    }

    /// The bounds of every triangle (`None` when it has none).
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        self.nodes.first().map(|n| (n.min, n.max))
    }

    /// The triangles the ray `origin + t·dir` crosses for t in `0..=max_t`,
    /// nearest first; only the nearest with `first`. Both faces of a
    /// triangle are hit.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max_t: f32, first: bool) -> Vec<TriangleHit> {
        let mut hits = Vec::new();
        if self.nodes.is_empty() || !dir.is_finite() || dir == Vec3::ZERO {
            return hits;
        }
        let inv = dir.recip();
        let mut limit = max_t;
        let mut stack = vec![0u32];
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i as usize];
            // Pushed before a nearer hit shortened the ray: test it again.
            if slab(node.min, node.max, origin, inv, limit).is_none() {
                continue;
            }
            if node.count > 0 {
                for &tri in &self.tris[node.first as usize..(node.first + node.count) as usize] {
                    if let Some((t, b1, b2)) = triangle(&self.corners[tri as usize], origin, dir)
                        && t <= limit
                    {
                        let [a, b, c] = self.corners[tri as usize];
                        let [ua, ub, uc] = self.uvs[tri as usize];
                        let w0 = 1.0 - b1 - b2;
                        hits.push(TriangleHit {
                            t,
                            face: tri,
                            normal: (b - a).cross(c - a).normalize_or(Vec3::Y),
                            bary: [b1, b2],
                            uv: [
                                ua[0] * w0 + ub[0] * b1 + uc[0] * b2,
                                ua[1] * w0 + ub[1] * b1 + uc[1] * b2,
                            ],
                        });
                        if first {
                            limit = t;
                        }
                    }
                }
                continue;
            }
            // Children: the nearer one is visited first (pushed last).
            let (l, r) = (i + 1, node.first);
            let dl = self.entry(l, origin, inv, limit);
            let dr = self.entry(r, origin, inv, limit);
            match (dl, dr) {
                (Some(a), Some(b)) if a <= b => stack.extend([r, l]),
                (Some(_), Some(_)) => stack.extend([l, r]),
                (Some(_), None) => stack.push(l),
                (None, Some(_)) => stack.push(r),
                (None, None) => {}
            }
        }
        hits.sort_by(|a, b| a.t.total_cmp(&b.t).then(a.face.cmp(&b.face)));
        if first {
            hits.truncate(1);
        }
        hits
    }

    fn entry(&self, i: u32, origin: Vec3, inv: Vec3, limit: f32) -> Option<f32> {
        let n = &self.nodes[i as usize];
        slab(n.min, n.max, origin, inv, limit)
    }
}

thread_local! {
    static BUILDS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Per triangle of `mesh`: its corners (NaN for one indexing past the
/// vertices) and their uvs.
#[allow(clippy::type_complexity)]
fn corners_of(mesh: &MeshData) -> (Vec<[Vec3; 3]>, Vec<[[f32; 2]; 3]>) {
    let n = mesh.vertices.len();
    let mut corners = Vec::with_capacity(mesh.indices.len() / 3);
    let mut uvs = Vec::with_capacity(mesh.indices.len() / 3);
    for t in mesh.indices.as_chunks::<3>().0 {
        if t.iter().any(|&i| i as usize >= n) {
            corners.push([Vec3::NAN; 3]);
            uvs.push([[0.0; 2]; 3]);
            continue;
        }
        let v = |k: usize| &mesh.vertices[t[k] as usize];
        corners.push([0, 1, 2].map(|k| Vec3::from(v(k).position)));
        uvs.push([0, 1, 2].map(|k| v(k).uv));
    }
    (corners, uvs)
}

/// Where the ray enters a box (0 inside), if it does before `limit`. An
/// axis the ray runs parallel to bounds nothing, but a ray outside the
/// box's slab on it misses (a ray along a face's plane hits that face's
/// edges, as the triangle test does).
fn slab(min: Vec3, max: Vec3, origin: Vec3, inv: Vec3, limit: f32) -> Option<f32> {
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
fn triangle(c: &[Vec3; 3], origin: Vec3, dir: Vec3) -> Option<(f32, f32, f32)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes;

    /// Every triangle tried one by one: what the BVH must agree with.
    fn brute(mesh: &MeshData, origin: Vec3, dir: Vec3) -> Vec<(f32, u32)> {
        let mut out = Vec::new();
        for (f, t) in mesh.indices.chunks(3).enumerate() {
            let c = [0, 1, 2].map(|k| Vec3::from(mesh.vertices[t[k] as usize].position));
            if let Some((t, ..)) = triangle(&c, origin, dir) {
                out.push((t, f as u32));
            }
        }
        out.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        out
    }

    #[test]
    fn the_bvh_answers_what_every_triangle_answers() {
        let mesh = shapes::sphere(48, 32);
        let bvh = Bvh::build(&mesh);
        assert_eq!(bvh.triangles(), mesh.indices.len() / 3);
        assert!(bvh.nodes() > 100, "{} nodes", bvh.nodes());
        // A fan of rays through the sphere from outside, some missing it.
        for k in 0..200 {
            let a = k as f32 * 0.37;
            let origin = Vec3::new(a.cos() * 2.0, (k as f32 * 0.11).sin() * 0.6, a.sin() * 2.0);
            let target = Vec3::new((k as f32 * 0.53).sin() * 0.6, 0.0, 0.0);
            let dir = (target - origin).normalize();
            let got: Vec<(f32, u32)> = bvh
                .raycast(origin, dir, f32::INFINITY, false)
                .iter()
                .map(|h| (h.t, h.face))
                .collect();
            let want = brute(&mesh, origin, dir);
            assert_eq!(got, want, "ray {k}");
            let first = bvh.raycast(origin, dir, f32::INFINITY, true);
            assert_eq!(
                first.first().map(|h| h.face),
                want.first().map(|w| w.1),
                "ray {k} nearest"
            );
        }
    }

    #[test]
    fn a_bvh_from_its_parts_is_the_bvh_built() {
        let mesh = shapes::sphere(24, 16);
        let built = Bvh::build(&mesh);
        let builds = Bvh::builds();
        let back =
            Bvh::from_parts(&mesh, built.node_list().to_vec(), built.order().to_vec()).unwrap();
        assert_eq!(back, built);
        assert_eq!(Bvh::builds(), builds, "from_parts builds nothing");
        // Parts that do not fit the mesh are refused, not traversed.
        let mut bad = built.node_list().to_vec();
        bad[0].first = 0;
        assert!(Bvh::from_parts(&mesh, bad, built.order().to_vec()).is_err());
        let mut order = built.order().to_vec();
        order[0] = 1 << 20;
        assert!(Bvh::from_parts(&mesh, built.node_list().to_vec(), order).is_err());
    }

    #[test]
    fn a_hit_carries_its_face_normal_and_uv() {
        let mesh = shapes::plane_xy(2.0, 2.0);
        let bvh = Bvh::build(&mesh);
        let hits = bvh.raycast(Vec3::new(0.5, 0.25, 3.0), Vec3::NEG_Z, 10.0, false);
        assert_eq!(hits.len(), 1);
        let h = hits[0];
        assert!((h.t - 3.0).abs() < 1e-6);
        assert!(h.normal.distance(Vec3::Z) < 1e-6);
        // plane_xy: u right, v down: (0.5, 0.25), off the diagonal, is uv (0.75, 0.375).
        assert!(
            (h.uv[0] - 0.75).abs() < 1e-5 && (h.uv[1] - 0.375).abs() < 1e-5,
            "{:?}",
            h.uv
        );
        assert!(
            bvh.raycast(Vec3::new(0.5, 0.25, 3.0), Vec3::NEG_Z, 2.0, false)
                .is_empty()
        );
    }
}
