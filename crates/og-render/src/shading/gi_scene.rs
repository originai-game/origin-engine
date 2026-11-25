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
    /// The top level changed since the buffer was written.
    pub tlas_dirty: bool,
    /// The top level's root (its vec4 index), and the instances placed.
    pub root: usize,
    pub instances: usize,
    /// Last frame's placed instances, so static scenes skip CPU SAH TLAS rebuild and GPU upload.
    last_placed: Vec<Placed>,
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
        if !self.words.is_empty() && self.last_placed == placed {
            return;
        }
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
        self.last_placed.clear();
        self.last_placed.extend_from_slice(placed);
        self.tlas_dirty = true;
    }

    /// The mean colour of an sRGB texture's texels (linear), cached by its
    /// index and revision; white for one with no texels on the CPU.
    pub fn texture_mean<'a>(
        &mut self,
        index: usize,
        revision: u64,
        data: impl FnOnce() -> Option<&'a crate::TextureData>,
    ) -> Vec3 {
        *self
            .texture_means
            .entry((index, revision))
            .or_insert_with(|| {
                let Some(t) = data().filter(|t| !t.rgba.is_empty()) else {
                    return Vec3::ONE;
                };
                let srgb = t.color_space == crate::ColorSpace::Srgb;
                let decode = |b: u8| {
                    if srgb {
                        og_core::color::srgb8_to_linear(b)
                    } else {
                        b as f32 / 255.0
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
