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
pub use og_core::bvh::{BvhNode, LEAF};
use og_core::bvh::{build, slab, triangle};

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
        let (nodes, tris) = build(&corners);
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
        og_core::sort::sort_by(&mut hits, |a, b| {
            a.t.total_cmp(&b.t).then(a.face.cmp(&b.face))
        });
        if first {
            hits.truncate(1);
        }
        hits
    }

    fn entry(&self, i: u32, origin: Vec3, inv: Vec3, limit: f32) -> Option<f32> {
        let n = &self.nodes[i as usize];
        slab(n.min, n.max, origin, inv, limit)
    }

    /// How far `p` stands from the nearest triangle (its surface, not its
    /// corners): what a coarser mesh's corner is off the full one by
    /// (og-cook measures a glTF's own `MSFT_lod` levels so). `None` when
    /// it holds no triangle. Boxes farther than the best triangle found
    /// are not opened, the nearer child first.
    pub fn distance(&self, p: Vec3) -> Option<f32> {
        self.distance_above(p, f32::NEG_INFINITY)
    }

    /// [`Bvh::distance`] when it is more than `floor`; otherwise a distance
    /// of some triangle within `floor`, the search stopping at the first
    /// one found. The early break of an exact Hausdorff distance (Taha and
    /// Hanbury 2015): a point nearer than the largest distance so far
    /// cannot raise it, so it need not find its nearest triangle.
    pub fn distance_above(&self, p: Vec3, floor: f32) -> Option<f32> {
        if self.nodes.is_empty() || !p.is_finite() {
            return None;
        }
        let mut best = f32::INFINITY;
        let mut stack = vec![0u32];
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i as usize];
            if box_distance(node.min, node.max, p) >= best {
                continue;
            }
            if node.count > 0 {
                for &tri in &self.tris[node.first as usize..(node.first + node.count) as usize] {
                    let d = (closest_on_triangle(&self.corners[tri as usize], p) - p).length();
                    best = best.min(d);
                }
                if best <= floor {
                    break;
                }
                continue;
            }
            let (l, r) = (i + 1, node.first);
            let dl = box_distance(self.nodes[l as usize].min, self.nodes[l as usize].max, p);
            let dr = box_distance(self.nodes[r as usize].min, self.nodes[r as usize].max, p);
            if dl <= dr {
                stack.extend([r, l]);
            } else {
                stack.extend([l, r]);
            }
        }
        Some(best)
    }
}

/// How far `p` is from the box (0 inside).
fn box_distance(min: Vec3, max: Vec3, p: Vec3) -> f32 {
    (min - p).max(p - max).max(Vec3::ZERO).length()
}

/// The point of the triangle nearest `p` (Ericson, Real-Time Collision
/// Detection §5.1.5: the Voronoi region of a corner, an edge or the face).
fn closest_on_triangle(c: &[Vec3; 3], p: Vec3) -> Vec3 {
    let [a, b, cc] = *c;
    let ab = b - a;
    let ac = cc - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return a + ab * v;
    }
    let cp = p - cc;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return cc;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return a + ac * w;
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (cc - b) * w;
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a + ab * v + ac * w
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
