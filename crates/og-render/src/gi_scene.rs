//! The simplified scene the probe GI traces (`gi_probes.rs`), the software
//! path Lumen takes where there is no hardware ray tracing (WebGPU has
//! none): a two-level bounding volume hierarchy built on the CPU and read
//! by `gi_trace.wgsl` from one storage buffer.
//!
//! - Each mesh and terrain an instance draws is one bottom level, built
//!   once per mesh revision: its triangles in its own space (a mesh past
//!   [`MAX_BLAS_TRIANGLES`] is simplified by vertex clustering first; a
//!   terrain's heights at no more than [`TERRAIN_SAMPLES`] a side), under a
//!   binned surface-area-heuristic BVH.
//! - The top level is built every frame over the instances (each its
//!   world-to-local transform, its level's root, and its material's albedo
//!   and emission, which is all a probe ray's hit needs).
//!
//! The buffer is `[bottom levels][top level and instances]`: the bottom
//! levels are written again only when the set of meshes changes, the rest
//! every frame. Every link in it is the absolute vec4 index of what it
//! names, so the shader adds no bases. Layout, in vec4:
//!
//! - a node, two: `(min, link)`, `(max, count)`. With count 0 an interior
//!   node, `link` its left child (the right one follows it); else a leaf of
//!   `count` triangles (or instances at the top), `link` the first.
//! - a triangle, three: its corners (w unused).
//! - an instance, [`INSTANCE_STRIDE`]: the three rows of its world-to-local
//!   transform, `(its level's root, two-sided, 0, 0)`, `(albedo, 0)`,
//!   `(emission, 0)`.

use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

/// The most triangles a bottom level keeps; a larger mesh is simplified.
pub(crate) const MAX_BLAS_TRIANGLES: usize = 8192;
/// A terrain's samples a side in its bottom level, at most.
pub(crate) const TERRAIN_SAMPLES: u32 = 129;
/// The most instances the top level holds (the nearest the camera).
pub(crate) const MAX_INSTANCES: usize = 16384;
/// The deepest a tree goes (`gi_trace.wgsl`'s stacks hold 32).
pub(crate) const MAX_DEPTH: usize = 30;
/// The vec4 of an instance record.
pub(crate) const INSTANCE_STRIDE: usize = 6;
const LEAF_TRIANGLES: usize = 4;
const LEAF_INSTANCES: usize = 2;
const BINS: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    const EMPTY: Aabb = Aabb {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };

    fn grow(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    fn union(&mut self, b: &Aabb) {
        self.min = self.min.min(b.min);
        self.max = self.max.max(b.max);
    }

    fn area(&self) -> f32 {
        let d = (self.max - self.min).max(Vec3::ZERO);
        d.x * d.y + d.y * d.z + d.z * d.x
    }

    fn centre(&self) -> Vec3 {
        0.5 * (self.min + self.max)
    }
}

/// A node before it is packed: a leaf's `link` is its first item in the
/// tree's order, an interior node's its left child.
#[derive(Clone, Copy, Debug)]
struct Node {
    bounds: Aabb,
    link: u32,
    count: u32,
}

/// A hierarchy over items: its nodes (the root first) and the items in the
/// order its leaves name them.
#[derive(Clone, Debug, Default)]
struct Bvh {
    nodes: Vec<Node>,
    order: Vec<u32>,
}

/// Builds a binned-SAH hierarchy (Wald, "On fast construction of SAH-based
/// bounding volume hierarchies", 2007) over items with these bounds, with
/// at most `leaf` items a leaf where the tree is not too deep.
fn build(boxes: &[Aabb], leaf: usize) -> Bvh {
    let mut bvh = Bvh {
        nodes: Vec::with_capacity(2 * boxes.len().max(1)),
        order: (0..boxes.len() as u32).collect(),
    };
    let centres: Vec<Vec3> = boxes.iter().map(Aabb::centre).collect();
    bvh.nodes.push(Node {
        bounds: Aabb::EMPTY,
        link: 0,
        count: 0,
    });
    split(&mut bvh, boxes, &centres, 0, 0, boxes.len(), 0, leaf);
    bvh
}

#[allow(clippy::too_many_arguments)]
fn split(
    bvh: &mut Bvh,
    boxes: &[Aabb],
    centres: &[Vec3],
    node: usize,
    start: usize,
    end: usize,
    depth: usize,
    leaf: usize,
) {
    let mut bounds = Aabb::EMPTY;
    let mut spread = Aabb::EMPTY;
    for &i in &bvh.order[start..end] {
        bounds.union(&boxes[i as usize]);
        spread.grow(centres[i as usize]);
    }
    let count = end - start;
    let extent = spread.max - spread.min;
    let make_leaf = |bvh: &mut Bvh| {
        bvh.nodes[node] = Node {
            bounds,
            link: start as u32,
            count: count.max(1) as u32,
        };
    };
    if count <= leaf || depth >= MAX_DEPTH || extent.max_element() <= 0.0 {
        make_leaf(bvh);
        return;
    }
    // The cheapest of BINS - 1 planes on each axis.
    let mut best = (f32::INFINITY, 0usize, 0usize);
    for axis in 0..3 {
        if extent[axis] <= 0.0 {
            continue;
        }
        let scale = BINS as f32 / extent[axis];
        let bin_of = |c: Vec3| (((c[axis] - spread.min[axis]) * scale) as usize).min(BINS - 1);
        let mut bins = [(Aabb::EMPTY, 0usize); BINS];
        for &i in &bvh.order[start..end] {
            let b = &mut bins[bin_of(centres[i as usize])];
            b.0.union(&boxes[i as usize]);
            b.1 += 1;
        }
        let mut right = [(0.0f32, 0usize); BINS];
        let (mut acc, mut n) = (Aabb::EMPTY, 0);
        for k in (1..BINS).rev() {
            acc.union(&bins[k].0);
            n += bins[k].1;
            right[k] = (acc.area(), n);
        }
        let (mut acc, mut n) = (Aabb::EMPTY, 0);
        for k in 0..BINS - 1 {
            acc.union(&bins[k].0);
            n += bins[k].1;
            let (ra, rn) = right[k + 1];
            if n == 0 || rn == 0 {
                continue;
            }
            let cost = acc.area() * n as f32 + ra * rn as f32;
            if cost < best.0 {
                best = (cost, axis, k);
            }
        }
    }
    let mid = if best.0.is_finite() {
        let (axis, k) = (best.1, best.2);
        let scale = BINS as f32 / extent[axis];
        let order = &mut bvh.order[start..end];
        let mut i = 0;
        for j in 0..order.len() {
            let c = centres[order[j] as usize];
            if ((((c[axis] - spread.min[axis]) * scale) as usize).min(BINS - 1)) <= k {
                order.swap(i, j);
                i += 1;
            }
        }
        start + i
    } else {
        // Every centre in one bin on every axis: halves by the median.
        let axis = extent.max_position();
        bvh.order[start..end]
            .sort_by(|a, b| centres[*a as usize][axis].total_cmp(&centres[*b as usize][axis]));
        start + count / 2
    };
    let mid = if mid == start || mid == end {
        start + count / 2
    } else {
        mid
    };
    let left = bvh.nodes.len();
    bvh.nodes.extend(
        [Node {
            bounds: Aabb::EMPTY,
            link: 0,
            count: 0,
        }; 2],
    );
    bvh.nodes[node] = Node {
        bounds,
        link: left as u32,
        count: 0,
    };
    split(bvh, boxes, centres, left, start, mid, depth + 1, leaf);
    split(bvh, boxes, centres, left + 1, mid, end, depth + 1, leaf);
}

/// Packs `bvh`'s nodes at vec4 `at`, a leaf linking to its first item at
/// `items + stride × index` (the items laid out in the tree's order).
fn pack_nodes(bvh: &Bvh, at: usize, items: usize, stride: usize, out: &mut Vec<[f32; 4]>) {
    debug_assert_eq!(out.len(), at);
    for n in &bvh.nodes {
        let link = if n.count == 0 {
            at + 2 * n.link as usize
        } else {
            items + stride * n.link as usize
        };
        out.push(n.bounds.min.extend(link as f32).to_array());
        out.push(n.bounds.max.extend(n.count as f32).to_array());
    }
}

/// A bottom level before it is placed: its tree and its triangles in the
/// tree's order.
#[derive(Clone, Debug)]
struct Level {
    bvh: Bvh,
    triangles: Vec<[Vec3; 3]>,
}

impl Level {
    fn new(triangles: Vec<[Vec3; 3]>) -> Level {
        let boxes: Vec<Aabb> = triangles
            .iter()
            .map(|t| {
                let mut b = Aabb::EMPTY;
                t.iter().for_each(|&p| b.grow(p));
                b
            })
            .collect();
        let bvh = build(&boxes, LEAF_TRIANGLES);
        let triangles = bvh.order.iter().map(|&i| triangles[i as usize]).collect();
        Level { bvh, triangles }
    }

    fn words(&self) -> usize {
        2 * self.bvh.nodes.len() + 3 * self.triangles.len()
    }

    /// Packs it at vec4 `at` (its root there).
    fn pack(&self, at: usize, out: &mut Vec<[f32; 4]>) {
        let tris = at + 2 * self.bvh.nodes.len();
        pack_nodes(&self.bvh, at, tris, 3, out);
        for t in &self.triangles {
            for p in t {
                out.push(p.extend(0.0).to_array());
            }
        }
    }
}

/// A mesh's triangles, simplified by vertex clustering (Rossignac and
/// Borrel 1993) onto the coarsest grid that keeps at most `max` of them
/// when it has more; degenerate ones dropped.
pub(crate) fn mesh_triangles(positions: &[Vec3], indices: &[u32], max: usize) -> Vec<[Vec3; 3]> {
    let tri = |t: &[u32; 3]| -> Option<[Vec3; 3]> {
        let [a, b, c] = t.map(|i| positions.get(i as usize).copied());
        Some([a?, b?, c?])
    };
    let all: Vec<[Vec3; 3]> = indices.as_chunks::<3>().0.iter().filter_map(tri).collect();
    if all.len() <= max {
        return all;
    }
    let mut bounds = Aabb::EMPTY;
    positions.iter().for_each(|&p| bounds.grow(p));
    let size = (bounds.max - bounds.min).max(Vec3::splat(1e-6));
    let mut cells = 64.0f32;
    loop {
        let cell = size.max_element() / cells;
        let key = |p: Vec3| ((p - bounds.min) / cell).floor().as_ivec3();
        // Each cell's vertices averaged.
        let mut sums: HashMap<glam::IVec3, (Vec3, u32)> = HashMap::new();
        for &p in positions {
            let e = sums.entry(key(p)).or_insert((Vec3::ZERO, 0));
            e.0 += p;
            e.1 += 1;
        }
        let at = |p: Vec3| {
            let (s, n) = sums[&key(p)];
            s / n as f32
        };
        let out: Vec<[Vec3; 3]> = all
            .iter()
            .filter(|t| {
                let k = t.map(key);
                k[0] != k[1] && k[1] != k[2] && k[0] != k[2]
            })
            .map(|t| t.map(at))
            .collect();
        if out.len() <= max || cells <= 2.0 {
            return out;
        }
        cells *= 0.7;
    }
}

/// A terrain's triangles in its own space: its heights at no more than
/// [`TERRAIN_SAMPLES`] a side, holes left out, wound to face up.
pub(crate) fn terrain_triangles(t: &crate::TerrainData) -> Vec<[Vec3; 3]> {
    let (cols, rows) = (t.columns.max(2), t.rows.max(2));
    let step = ((cols.max(rows) - 1).div_ceil(TERRAIN_SAMPLES - 1)).max(1);
    let cell = t.size / Vec2::new(cols as f32 - 1.0, rows as f32 - 1.0);
    let half = t.size * 0.5;
    let at = |c: u32, r: u32| {
        let (c, r) = (c.min(cols - 1), r.min(rows - 1));
        let h = t
            .heights
            .get((r * t.columns + c) as usize)
            .copied()
            .unwrap_or(0.0);
        Vec3::new(c as f32 * cell.x - half.x, h, r as f32 * cell.y - half.y)
    };
    let hole = |c: u32, r: u32| {
        t.holes.as_ref().is_some_and(|h| {
            h.get((r.min(rows - 2) * (cols - 1) + c.min(cols - 2)) as usize)
                .is_some_and(|&b| b != 0)
        })
    };
    let mut out = Vec::new();
    let mut r = 0;
    while r < rows - 1 {
        let r1 = (r + step).min(rows - 1);
        let mut c = 0;
        while c < cols - 1 {
            let c1 = (c + step).min(cols - 1);
            if !hole(c, r) {
                let (p00, p10, p01, p11) = (at(c, r), at(c1, r), at(c, r1), at(c1, r1));
                out.push([p00, p01, p10]);
                out.push([p10, p01, p11]);
            }
            c = c1;
        }
        r = r1;
    }
    out
}

/// What a bottom level is built from, and the revision it was built at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Source {
    Mesh(usize, u64),
    Terrain(usize, u64),
}

/// One instance the top level holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placed {
    pub source: Source,
    pub transform: Mat4,
    pub albedo: Vec3,
    pub emissive: Vec3,
    pub two_sided: bool,
}

/// The traced scene's CPU side: the bottom levels built so far and the
/// buffer's words.
#[derive(Default)]
pub(crate) struct GiScene {
    levels: HashMap<Source, Level>,
    /// The bottom levels laid out, by source, with their roots.
    layout: Vec<(Source, usize)>,
    /// The packed bottom levels (`[0, static_len)` of the buffer).
    pub words: Vec<[f32; 4]>,
    pub static_len: usize,
    /// The bottom levels changed since the buffer was written.
    pub static_dirty: bool,
    /// The top level's root (its vec4 index), and the instances placed.
    pub root: usize,
    pub instances: usize,
    /// The mean base colour of each texture (by index and revision).
    texture_means: HashMap<(usize, u64), Vec3>,
}

impl GiScene {
    /// Builds what `placed` needs that is not built yet (`triangles` makes a
    /// source's triangles), lays the bottom levels out again when the set
    /// of them changed, and packs the top level after them.
    pub fn update(
        &mut self,
        placed: &[Placed],
        mut triangles: impl FnMut(Source) -> Vec<[Vec3; 3]>,
    ) {
        let mut sources: Vec<Source> = placed.iter().map(|p| p.source).collect();
        sources.sort();
        sources.dedup();
        for s in &sources {
            if !self.levels.contains_key(s) {
                self.levels.insert(*s, Level::new(triangles(*s)));
            }
        }
        self.levels.retain(|s, _| sources.binary_search(s).is_ok());
        let same = self.layout.len() == sources.len()
            && self.layout.iter().zip(&sources).all(|(a, b)| a.0 == *b);
        if !same {
            self.words.clear();
            self.layout.clear();
            for s in &sources {
                let level = &self.levels[s];
                let at = self.words.len();
                self.words.reserve(level.words());
                level.pack(at, &mut self.words);
                self.layout.push((*s, at));
            }
            self.static_len = self.words.len();
            self.static_dirty = true;
        }
        self.words.truncate(self.static_len);
        let roots: HashMap<Source, usize> = self.layout.iter().copied().collect();
        // The top level over the instances' world bounds.
        let mut boxes = Vec::with_capacity(placed.len());
        let mut keep = Vec::with_capacity(placed.len());
        for p in placed {
            let level = &self.levels[&p.source];
            let Some(root) = level.bvh.nodes.first() else {
                continue;
            };
            if root.bounds.min.x > root.bounds.max.x {
                continue;
            }
            boxes.push(transformed(&root.bounds, &p.transform));
            keep.push(p);
        }
        let top = build(&boxes, LEAF_INSTANCES);
        let at = self.words.len();
        let records = at + 2 * top.nodes.len();
        if keep.is_empty() {
            // An empty leaf (a box nothing enters).
            self.words
                .push(Aabb::EMPTY.min.extend(records as f32).to_array());
            self.words.push(Aabb::EMPTY.max.extend(0.0).to_array());
        } else {
            pack_nodes(&top, at, records, INSTANCE_STRIDE, &mut self.words);
        }
        for &i in &top.order {
            let p = keep[i as usize];
            let inv = p.transform.inverse();
            let rows = [inv.row(0), inv.row(1), inv.row(2)];
            for r in rows {
                self.words.push(r.to_array());
            }
            self.words
                .push([roots[&p.source] as f32, p.two_sided as u32 as f32, 0.0, 0.0]);
            self.words.push(p.albedo.extend(0.0).to_array());
            self.words.push(p.emissive.extend(0.0).to_array());
        }
        self.root = at;
        self.instances = keep.len();
    }

    /// The mean colour of an sRGB texture's texels (linear), cached by its
    /// index and revision; white for one with no texels on the CPU.
    pub fn texture_mean(
        &mut self,
        index: usize,
        revision: u64,
        data: Option<&crate::TextureData>,
    ) -> Vec3 {
        *self
            .texture_means
            .entry((index, revision))
            .or_insert_with(|| {
                let Some(t) = data.filter(|t| !t.rgba.is_empty()) else {
                    return Vec3::ONE;
                };
                let srgb = t.color_space == crate::ColorSpace::Srgb;
                let decode = |b: u8| {
                    let c = b as f32 / 255.0;
                    if !srgb {
                        c
                    } else if c <= 0.04045 {
                        c / 12.92
                    } else {
                        ((c + 0.055) / 1.055).powf(2.4)
                    }
                };
                // At most 4096 texels, evenly through the image.
                let texels = t.rgba.len() / 4;
                let step = (texels / 4096).max(1);
                let (mut sum, mut n) = (Vec3::ZERO, 0.0);
                for i in (0..texels).step_by(step) {
                    let p = &t.rgba[4 * i..4 * i + 4];
                    sum += Vec3::new(decode(p[0]), decode(p[1]), decode(p[2]));
                    n += 1.0;
                }
                sum / n
            })
    }
}

/// The world bounds of local bounds `b` under `m`.
fn transformed(b: &Aabb, m: &Mat4) -> Aabb {
    let mut out = Aabb::EMPTY;
    for k in 0..8 {
        let p = Vec3::new(
            if k & 1 == 0 { b.min.x } else { b.max.x },
            if k & 2 == 0 { b.min.y } else { b.max.y },
            if k & 4 == 0 { b.min.z } else { b.max.z },
        );
        out.grow(m.transform_point3(p));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The buffer's first hit along a ray, traversed as gi_trace.wgsl
    /// traverses it (a stack of absolute vec4 links, each instance's ray in
    /// its space): the reference the shader is written against.
    fn trace(words: &[[f32; 4]], root: usize, o: Vec3, d: Vec3) -> Option<(f32, usize)> {
        let v = |i: usize| glam::Vec4::from(words[i]);
        let slab = |at: usize, o: Vec3, inv: Vec3, t_max: f32| {
            let (lo, hi) = (v(at).truncate(), v(at + 1).truncate());
            let a = (lo - o) * inv;
            let b = (hi - o) * inv;
            let t0 = a.min(b).max_element().max(0.0);
            let t1 = a.max(b).min_element().min(t_max);
            t0 <= t1
        };
        let mut best: Option<(f32, usize)> = None;
        let mut stack = vec![root];
        let inv = d.recip();
        while let Some(n) = stack.pop() {
            if !slab(n, o, inv, best.map_or(f32::INFINITY, |b| b.0)) {
                continue;
            }
            let (link, count) = (v(n).w as usize, v(n + 1).w as usize);
            if count == 0 {
                stack.extend([link, link + 2]);
                continue;
            }
            for k in 0..count {
                let rec = link + INSTANCE_STRIDE * k;
                let row = |r: usize| v(rec + r);
                let lo = Vec3::new(
                    row(0).dot(o.extend(1.0)),
                    row(1).dot(o.extend(1.0)),
                    row(2).dot(o.extend(1.0)),
                );
                let ld = Vec3::new(
                    row(0).truncate().dot(d),
                    row(1).truncate().dot(d),
                    row(2).truncate().dot(d),
                );
                let linv = ld.recip();
                let mut inner = vec![v(rec + 3).x as usize];
                while let Some(m) = inner.pop() {
                    if !slab(m, lo, linv, best.map_or(f32::INFINITY, |b| b.0)) {
                        continue;
                    }
                    let (l2, c2) = (v(m).w as usize, v(m + 1).w as usize);
                    if c2 == 0 {
                        inner.extend([l2, l2 + 2]);
                        continue;
                    }
                    for j in 0..c2 {
                        let t = l2 + 3 * j;
                        let tri = [v(t).truncate(), v(t + 1).truncate(), v(t + 2).truncate()];
                        if let Some(h) = hit(tri, lo, ld)
                            && best.is_none_or(|b| h < b.0)
                        {
                            best = Some((h, rec));
                        }
                    }
                }
            }
        }
        best
    }

    /// Möller and Trumbore's ray-triangle test, both faces.
    fn hit(t: [Vec3; 3], o: Vec3, d: Vec3) -> Option<f32> {
        let (e1, e2) = (t[1] - t[0], t[2] - t[0]);
        let p = d.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-12 {
            return None;
        }
        let s = o - t[0];
        let u = s.dot(p) / det;
        let q = s.cross(e1);
        let w = d.dot(q) / det;
        let h = e2.dot(q) / det;
        (u >= 0.0 && w >= 0.0 && u + w <= 1.0 && h > 0.0).then_some(h)
    }

    fn rng(seed: &mut u64) -> f32 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*seed >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Every ray finds, through both levels, the same first hit as testing
    /// every triangle of every instance: a sphere and a box placed four
    /// times (scaled unevenly, rotated), rays from random points in random
    /// directions, half of which miss.
    #[test]
    fn the_hierarchy_finds_the_first_hit_brute_force_finds() {
        let sphere = crate::shapes::sphere(16, 12);
        let cube = crate::shapes::cuboid(Vec3::ONE);
        let tris = |m: &crate::MeshData| {
            let p: Vec<Vec3> = m.vertices.iter().map(|v| Vec3::from(v.position)).collect();
            mesh_triangles(&p, &m.indices, MAX_BLAS_TRIANGLES)
        };
        let meshes = [tris(&sphere), tris(&cube)];
        let placed: Vec<Placed> = (0..4)
            .map(|k| Placed {
                source: Source::Mesh(k % 2, 0),
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::new(1.0 + k as f32, 0.5 + 0.25 * k as f32, 1.5),
                    glam::Quat::from_rotation_y(0.7 * k as f32),
                    Vec3::new(3.0 * k as f32 - 4.0, 0.3 * k as f32, -2.0 + k as f32),
                ),
                albedo: Vec3::ONE,
                emissive: Vec3::ZERO,
                two_sided: false,
            })
            .collect();
        let mut scene = GiScene::default();
        scene.update(&placed, |s| match s {
            Source::Mesh(i, _) => meshes[i].clone(),
            Source::Terrain(..) => unreachable!(),
        });
        assert_eq!(scene.instances, 4);
        let mut seed = 7;
        let (mut hits, mut agree) = (0, 0);
        for _ in 0..2000 {
            let o = Vec3::new(
                rng(&mut seed) * 16.0 - 8.0,
                rng(&mut seed) * 6.0 - 3.0,
                rng(&mut seed) * 8.0 - 4.0,
            );
            // Half aimed near an instance, half anywhere.
            let jitter = Vec3::new(
                rng(&mut seed) - 0.5,
                rng(&mut seed) - 0.5,
                rng(&mut seed) - 0.5,
            );
            let aim = placed[(rng(&mut seed) * 4.0) as usize % 4]
                .transform
                .w_axis
                .truncate();
            let d = if rng(&mut seed) < 0.5 {
                (aim + 2.0 * jitter - o).normalize()
            } else {
                jitter.normalize()
            };
            let mut want: Option<f32> = None;
            for p in &placed {
                let m = &meshes[match p.source {
                    Source::Mesh(i, _) => i,
                    _ => 0,
                }];
                for t in m {
                    let w = t.map(|q| p.transform.transform_point3(q));
                    if let Some(h) = hit(w, o, d) {
                        want = Some(want.map_or(h, |b: f32| b.min(h)));
                    }
                }
            }
            let got = trace(&scene.words, scene.root, o, d).map(|g| g.0);
            match (want, got) {
                (None, None) => agree += 1,
                (Some(a), Some(b)) => {
                    hits += 1;
                    assert!(
                        (a - b).abs() < 1e-3 * a.max(1.0),
                        "first hit {a} against {b}"
                    );
                    agree += 1;
                }
                other => panic!(
                    "from {o} along {d}: brute force {:?}, hierarchy {:?}",
                    other.0, other.1
                ),
            }
        }
        assert_eq!(agree, 2000);
        assert!(
            hits > 300 && hits < 1700,
            "a test that both hits and misses: {hits}"
        );
    }

    /// The tree stays within the shader's stack: no deeper than MAX_DEPTH
    /// even over items that all share one centre.
    #[test]
    fn a_tree_never_outgrows_the_shader_s_stack() {
        fn depth(b: &Bvh, n: usize) -> usize {
            let node = b.nodes[n];
            if node.count > 0 {
                return 0;
            }
            1 + depth(b, node.link as usize).max(depth(b, node.link as usize + 1))
        }
        let same = vec![
            Aabb {
                min: Vec3::ZERO,
                max: Vec3::ONE
            };
            5000
        ];
        let b = build(&same, LEAF_TRIANGLES);
        assert!(depth(&b, 0) <= MAX_DEPTH);
        let leaves: u32 = b
            .nodes
            .iter()
            .filter(|n| n.count > 0)
            .map(|n| n.count)
            .sum();
        assert_eq!(leaves, 5000, "every item in exactly one leaf");
        // A chain of tiny boxes far apart (the worst case for SAH depth).
        let chain: Vec<Aabb> = (0..5000)
            .map(|i| {
                let p = Vec3::splat(1.5f32.powi(i % 60));
                Aabb {
                    min: p,
                    max: p + 1e-3,
                }
            })
            .collect();
        assert!(depth(&build(&chain, LEAF_TRIANGLES), 0) <= MAX_DEPTH);
    }

    /// A mesh past the budget is simplified within it and keeps its
    /// extent: a 128 × 64 sphere (16 256 triangles) to at most 2048.
    #[test]
    fn a_large_mesh_is_clustered_within_its_budget() {
        let m = crate::shapes::sphere(128, 64);
        let p: Vec<Vec3> = m.vertices.iter().map(|v| Vec3::from(v.position)).collect();
        let full = mesh_triangles(&p, &m.indices, usize::MAX);
        assert!(full.len() > 16000);
        let small = mesh_triangles(&p, &m.indices, 2048);
        assert!(small.len() <= 2048 && small.len() > 200, "{}", small.len());
        let r = small
            .iter()
            .flatten()
            .map(|q| q.length())
            .fold(0.0f32, f32::max);
        assert!(
            (r - full
                .iter()
                .flatten()
                .map(|q| q.length())
                .fold(0.0f32, f32::max))
            .abs()
                < 0.05
        );
    }

    /// A terrain faces up, spans its size, and leaves its holes out.
    #[test]
    fn a_terrain_is_its_heights_facing_up() {
        let mut t = crate::TerrainData::flat(Vec2::new(10.0, 6.0), 6, 4);
        let mut holes = vec![0u8; 5 * 3];
        holes[0] = 1;
        t.holes = Some(holes.into());
        let tris = terrain_triangles(&t);
        assert_eq!(tris.len(), 2 * (5 * 3 - 1));
        for q in &tris {
            assert!((q[1] - q[0]).cross(q[2] - q[0]).y > 0.0, "faces up");
        }
        let xs: Vec<f32> = tris.iter().flatten().map(|q| q.x).collect();
        assert_eq!(xs.iter().copied().fold(f32::INFINITY, f32::min), -5.0);
        assert_eq!(xs.iter().copied().fold(f32::NEG_INFINITY, f32::max), 5.0);
    }
}
