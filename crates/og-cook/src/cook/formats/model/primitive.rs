use super::*;

/// Full-precision vertex before quantisation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FVertex {
    pub(crate) p: [f32; 3],
    pub(crate) n: [f32; 3],
    pub(crate) uv: [f32; 2],
    pub(crate) c: [f32; 4],
    pub(crate) t: [f32; 4],
    /// A hash of the source vertex's skin and morph data (zero without
    /// either), so welding never merges vertices that skin or morph apart.
    pub(crate) key: [u32; 2],
}

/// The quantised vertex (unused attributes stay zero and are not stored).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct QVertex {
    p: [u16; 4],
    n: [i16; 2],
    uv: [u16; 2],
    c: [u8; 4],
    t: [i16; 4],
    key: [u32; 2],
}

fn oct_encode(v: Vec3) -> [i16; 2] {
    let v = v.normalize_or(Vec3::Z);
    let l1 = v.x.abs() + v.y.abs() + v.z.abs();
    let (mut x, mut y) = (v.x / l1, v.y / l1);
    if v.z < 0.0 {
        let (ox, oy) = (x, y);
        x = (1.0 - oy.abs()) * ox.signum();
        y = (1.0 - ox.abs()) * oy.signum();
    }
    [
        (x.clamp(-1.0, 1.0) * 32767.0).round() as i16,
        (y.clamp(-1.0, 1.0) * 32767.0).round() as i16,
    ]
}

fn unorm16(v: f32, lo: f32, scale: f32) -> u16 {
    if scale <= 0.0 {
        0
    } else {
        (((v - lo) / scale).clamp(0.0, 1.0) * 65535.0).round() as u16
    }
}

pub(crate) struct Tangents<'a> {
    pub(crate) v: &'a mut [FVertex],
}

impl mikktspace::Geometry for Tangents<'_> {
    fn num_faces(&self) -> usize {
        self.v.len() / 3
    }
    fn num_vertices_of_face(&self, _: usize) -> usize {
        3
    }
    fn position(&self, f: usize, i: usize) -> [f32; 3] {
        self.v[f * 3 + i].p
    }
    fn normal(&self, f: usize, i: usize) -> [f32; 3] {
        self.v[f * 3 + i].n
    }
    // glTF's uvs run down the image, its normal maps' green up it: MikkTSpace
    // sees v flipped so the bitangent (cross(normal, tangent) × w) points up
    // the image, as exporters' tangents and the renderer's generated ones do.
    fn tex_coord(&self, f: usize, i: usize) -> [f32; 2] {
        let [u, v] = self.v[f * 3 + i].uv;
        [u, 1.0 - v]
    }
    fn set_tangent_encoded(&mut self, t: [f32; 4], f: usize, i: usize) {
        self.v[f * 3 + i].t = t;
    }
}

/// Welds identical vertices; `sources` (the source vertex of each input
/// vertex) follows the vertices.
pub(crate) fn weld<T: bytemuck::Pod>(
    vertices: &[T],
    indices: &[u32],
    sources: &mut Vec<u32>,
) -> (Vec<T>, Vec<u32>) {
    let (count, remap) = meshopt::generate_vertex_remap(vertices, Some(indices));
    let mut out = vec![T::zeroed(); count];
    let mut src = vec![0u32; count];
    for (old, &new) in remap.iter().enumerate() {
        if new != u32::MAX {
            out[new as usize] = vertices[old];
            src[new as usize] = sources[old];
        }
    }
    *sources = src;
    (out, indices.iter().map(|&i| remap[i as usize]).collect())
}

/// The weld key of every source vertex of a primitive: a hash of its
/// JOINTS_0/WEIGHTS_0 and morph-target deltas, or zero without either.
/// A primitive's foliage (og-render's `meshlet::MeshFoliage`): each
/// vertex's `_OG_PIVOT` (branch pivot, stiffness) and `_OG_ANCHOR` (leaf
/// anchor, flutter), VEC4 floats in the mesh's own space. `None` without
/// them; both or neither, stiffness and flutter within 0..1.
pub(in crate::cook) fn read_foliage(
    src: &GltfSource,
    prim: &gltf::Primitive,
    count: usize,
) -> Result<Option<Vec<[f32; 8]>>, String> {
    let read = |name: &str| -> Result<Option<Vec<[f32; 4]>>, String> {
        let Some(a) = prim.get(&gltf::Semantic::Extras(name.into())) else {
            return Ok(None);
        };
        if a.data_type() != gltf::accessor::DataType::F32
            || a.dimensions() != gltf::accessor::Dimensions::Vec4
            || a.count() != count
        {
            return Err(format!(
                "_{name}: a VEC4 float accessor of {count} elements"
            ));
        }
        let it = gltf::accessor::Iter::<[f32; 4]>::new(a, |b| Some(&src.buffers[b.index()].0[..]))
            .ok_or_else(|| format!("_{name}: no data"))?;
        Ok(Some(it.collect()))
    };
    let (pivot, anchor) = match (read("OG_PIVOT")?, read("OG_ANCHOR")?) {
        (Some(p), Some(a)) => (p, a),
        (None, None) => return Ok(None),
        _ => return Err("foliage needs both _OG_PIVOT and _OG_ANCHOR".into()),
    };
    let mut out = Vec::with_capacity(count);
    for (i, (p, a)) in pivot.iter().zip(&anchor).enumerate() {
        let unit = |x: f32| (0.0..=1.0).contains(&x);
        if !p.iter().chain(a).all(|x| x.is_finite()) || !unit(p[3]) || !unit(a[3]) {
            return Err(format!(
                "foliage vertex {i}: pivot {p:?}, anchor {a:?} (finite, stiffness and flutter 0..1)"
            ));
        }
        out.push([p[0], p[1], p[2], p[3], a[0], a[1], a[2], a[3]]);
    }
    Ok(Some(out))
}

/// Whether a primitive's triangles are leaf cards: its glTF `extras`
/// `{ "ogFoliage": { "leaves": true } }`.
pub(in crate::cook) fn leaf_cards(prim: &gltf::Primitive) -> bool {
    prim.extras()
        .as_ref()
        .and_then(|e| serde_json::from_str::<serde_json::Value>(e.get()).ok())
        .and_then(|v| v["ogFoliage"]["leaves"].as_bool())
        .unwrap_or(false)
}

/// The weld key of a foliage vertex: vertices with other pivots or
/// anchors never weld (a seam between two branches keeps both sides').
fn foliage_key(f: &[f32; 8]) -> [u32; 2] {
    let mut h = og_core::hash::StableHasher::new();
    f.iter().for_each(|&x| h.write_u64(x.to_bits() as u64));
    let v = h.finish();
    [v as u32, (v >> 32) as u32]
}

/// The fewest leaves a thinned level keeps: fewer no longer cover a crown
/// as the full mesh's do.
const THIN_LEAVES_MIN: usize = 8;

/// Leaf cards' coarser levels: level `k` keeps every `1 / fractions[k]`th
/// leaf (the triangles of one anchor, in the order the mesh draws them, so
/// a coarser level's leaves are among the finer one's) and draws each as
/// much larger in area about its anchor, so the crown covers as much.
/// Returns each level's range of `indices`, its error (the farthest any
/// kept vertex moves) and its leaf scale. A level indexes the full mesh's
/// own vertices and the runtime draws them `scale` times as far from their
/// anchors (`LodRow::leaf_scale`), so the levels cost no vertices; with
/// `copy` (a streaming proxy, which keeps one level and draws it as its
/// full mesh) the kept leaves' vertices are appended where the level draws
/// them (`sources` to the vertex each came from) and its scale is 1.
/// `foliage` is per source vertex and `full` the full mesh's range of
/// `indices`.
fn thin_leaves(
    verts: &mut Vec<FVertex>,
    indices: &mut Vec<u32>,
    sources: &mut Vec<u32>,
    foliage: &[[f32; 8]],
    full: std::ops::Range<usize>,
    fractions: &[f32],
    copy: bool,
) -> Vec<(std::ops::Range<usize>, f32, f32)> {
    // Each triangle's leaf: the anchor (and pivot) of its first corner.
    let mut leaf_of: std::collections::HashMap<[u32; 8], usize> = Default::default();
    let triangles: Vec<([u32; 3], usize)> = indices[full]
        .as_chunks::<3>()
        .0
        .iter()
        .map(|&tri| {
            let key = foliage[sources[tri[0] as usize] as usize].map(f32::to_bits);
            let next = leaf_of.len();
            (tri, *leaf_of.entry(key).or_insert(next))
        })
        .collect();
    let leaves = leaf_of.len();
    let mut out = Vec::new();
    let mut previous = 1;
    for &fraction in fractions.iter().skip(1) {
        if !(fraction > 0.0 && fraction < 1.0) {
            continue;
        }
        let stride = (1.0 / fraction).round() as usize;
        if stride <= previous || leaves.div_ceil(stride) < THIN_LEAVES_MIN {
            break;
        }
        previous = stride;
        let scale = (stride as f32).sqrt();
        let start = indices.len();
        let mut moved = 0.0f32;
        let mut made: std::collections::HashMap<u32, u32> = Default::default();
        for (tri, leaf) in &triangles {
            if leaf % stride != 0 {
                continue;
            }
            for &i in tri {
                let at = *made.entry(i).or_insert_with(|| {
                    let mut v = verts[i as usize];
                    let f = &foliage[sources[i as usize] as usize];
                    let anchor = Vec3::new(f[4], f[5], f[6]);
                    let arm = Vec3::from(v.p) - anchor;
                    moved = moved.max(arm.length() * (scale - 1.0));
                    if !copy {
                        return i;
                    }
                    v.p = (anchor + arm * scale).to_array();
                    verts.push(v);
                    sources.push(sources[i as usize]);
                    verts.len() as u32 - 1
                });
                indices.push(at);
            }
        }
        out.push((start..indices.len(), moved, if copy { 1.0 } else { scale }));
    }
    out
}

/// The vertices a foliage mesh's simplification keeps in place: those on
/// an edge between two pivots or two anchors (`foliage` per vertex), so a
/// coarser level never joins two branches (or two leaves) that sway apart.
pub(in crate::cook) fn foliage_locks(indices: &[u32], foliage: &[[f32; 8]]) -> Vec<bool> {
    let mut lock = vec![false; foliage.len()];
    for tri in indices.as_chunks::<3>().0 {
        for k in 0..3 {
            let (a, b) = (tri[k] as usize, tri[(k + 1) % 3] as usize);
            let (fa, fb) = (&foliage[a], &foliage[b]);
            if fa[0..3] != fb[0..3] || fa[4..7] != fb[4..7] {
                lock[a] = true;
                lock[b] = true;
            }
        }
    }
    lock
}

pub(crate) fn vertex_keys(src: &GltfSource, prim: &gltf::Primitive, count: usize) -> Vec<[u32; 2]> {
    let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
    let joints: Option<Vec<[u16; 4]>> = reader.read_joints(0).map(|j| j.into_u16().collect());
    let weights: Option<Vec<[f32; 4]>> = reader.read_weights(0).map(|w| w.into_f32().collect());
    type Deltas = Vec<[f32; 3]>;
    let targets: Vec<(Deltas, Deltas)> = reader
        .read_morph_targets()
        .map(|(p, n, _)| {
            (
                p.map(|p| p.collect()).unwrap_or_default(),
                n.map(|n| n.collect()).unwrap_or_default(),
            )
        })
        .collect();
    if joints.is_none() && weights.is_none() && targets.is_empty() {
        return vec![[0, 0]; count];
    }
    (0..count)
        .map(|i| {
            let mut h = og_core::hash::StableHasher::new();
            if let Some(j) = joints.as_ref().and_then(|j| j.get(i)) {
                j.iter().for_each(|&x| h.write_u64(x as u64));
            }
            if let Some(w) = weights.as_ref().and_then(|w| w.get(i)) {
                w.iter().for_each(|&x| h.write_u64(x.to_bits() as u64));
            }
            for (p, n) in &targets {
                for v in p.get(i).into_iter().chain(n.get(i)) {
                    v.iter().for_each(|&x| h.write_u64(x.to_bits() as u64));
                }
            }
            let v = h.finish();
            [v as u32, (v >> 32) as u32 | 1]
        })
        .collect()
}

fn positions_adapter(pos: &[[f32; 3]]) -> meshopt::VertexDataAdapter<'_> {
    meshopt::VertexDataAdapter::new(bytemuck::cast_slice(pos), 12, 0).expect("12-byte positions")
}

/// Attribute-aware permissive edge collapses (meshoptimizer, MIT): seams
/// may collapse only within the combined position/normal/UV/colour error.
/// Never weld corners back by position: every index keeps its own source
/// attributes and deformation data. Callers that rebuild normals give
/// their normal weight as zero; UVs and colours have unit weight.
pub(crate) fn simplify_attributes(
    v: &[FVertex],
    idx: &[u32],
    target: usize,
    error: f32,
    options: meshopt::SimplifyOptions,
    locks: &[bool],
    normal_weight: f32,
) -> (Vec<u32>, f32) {
    let attributes: Vec<[f32; 9]> = v
        .iter()
        .map(|x| {
            [
                x.n[0], x.n[1], x.n[2], x.uv[0], x.uv[1], x.c[0], x.c[1], x.c[2], x.c[3],
            ]
        })
        .collect();
    let weights = [
        normal_weight,
        normal_weight,
        normal_weight,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
    ];
    let mut locked = vec![0u8; v.len()];
    if !locks.is_empty() {
        for (flag, &lock) in locked.iter_mut().zip(locks) {
            *flag = u8::from(lock);
        }
    }
    // Protect UV wedges so meshoptimizer collapses both sides together;
    // unlike a lock this still allows simplification along the seam.
    // Deformation discontinuities must remain immovable.
    let mut keys = std::collections::BTreeMap::new();
    let mut split = std::collections::BTreeMap::new();
    for &i in idx {
        let x = &v[i as usize];
        let p = x.p.map(|a| (a + 0.0).to_bits());
        let identity = (x.key, x.uv.map(|a| (a + 0.0).to_bits()));
        let first = *keys.entry(p).or_insert(identity);
        if first != identity {
            *split.entry(p).or_insert(0) |= if first.0 != identity.0 { 1 } else { 2 };
        }
    }
    for (i, x) in v.iter().enumerate() {
        locked[i] |= split
            .get(&x.p.map(|a| (a + 0.0).to_bits()))
            .copied()
            .unwrap_or(0);
    }
    // Flat normals are rebuilt by the caller. Their duplicated corners
    // are not additional UV wedges: share equivalent corners before
    // meshoptimizer classifies manifold edges and paired seam edges.
    // Sparse excludes the unused duplicates from that classification.
    let mut corners = std::collections::BTreeMap::new();
    let remapped: Vec<u32> = idx
        .iter()
        .map(|&i| {
            let x = &v[i as usize];
            let key = (
                x.p.map(|a| (a + 0.0).to_bits()),
                x.uv.map(|a| (a + 0.0).to_bits()),
                x.c.map(|a| (a + 0.0).to_bits()),
                if normal_weight == 0.0 {
                    [0; 3]
                } else {
                    x.n.map(|a| (a + 0.0).to_bits())
                },
                x.key,
            );
            let first = *corners.entry(key).or_insert(i);
            locked[first as usize] |= locked[i as usize];
            first
        })
        .collect();
    // LockBorder classifies simple border loops upstream. Generated meshes
    // also contain non-manifold junctions; permissive classification can
    // treat these as complex/interior vertices. Preserve their boundaries
    // rather than letting an ambiguous wedge collapse open a surface.
    if options.contains(meshopt::SimplifyOptions::LockBorder) {
        let mut edges = std::collections::BTreeMap::new();
        for tri in remapped.as_chunks::<3>().0 {
            let p = tri.map(|i| v[i as usize].p.map(|a| (a + 0.0).to_bits()));
            for k in 0..3 {
                let mut edge = [p[k], p[(k + 1) % 3]];
                let forward = edge[0] <= edge[1];
                edge.sort();
                let entry = edges.entry(edge).or_insert((0u32, 0i32));
                entry.0 += 1;
                entry.1 += if forward { 1 } else { -1 };
            }
        }
        let boundary: std::collections::BTreeSet<_> = edges
            .into_iter()
            .filter(|(_, (count, balance))| *count != 2 || *balance != 0)
            .flat_map(|(edge, _)| edge)
            .collect();
        // Keep the whole incident fan. Locking just the junction allows
        // an adjacent manifold corner to collapse into it and delete one
        // side of an ambiguous edge (meshoptimizer's link test assumes a
        // manifold neighbourhood).
        let mut protected = boundary.clone();
        for tri in remapped.as_chunks::<3>().0 {
            let p = tri.map(|i| v[i as usize].p.map(|a| (a + 0.0).to_bits()));
            if p.iter().any(|p| boundary.contains(p)) {
                protected.extend(p);
            }
        }
        for (i, vertex) in v.iter().enumerate() {
            if protected.contains(&vertex.p.map(|a| (a + 0.0).to_bits())) {
                locked[i] |= 1;
            }
        }
    }
    let adapter =
        meshopt::VertexDataAdapter::new(bytemuck::cast_slice(v), std::mem::size_of::<FVertex>(), 0)
            .expect("positions lead the vertex");
    let mut achieved_error = 0.0;
    let mut indices = vec![0; idx.len()];
    // SAFETY: output has room for every input index; positions, attributes
    // and byte flags have one entry per vertex. The Rust wrapper accepts
    // bool locks only and cannot express meshopt_SimplifyVertex_Protect (2).
    let count = unsafe {
        meshopt::ffi::meshopt_simplifyWithAttributes(
            indices.as_mut_ptr(),
            remapped.as_ptr(),
            idx.len(),
            adapter.pos_ptr(),
            v.len(),
            std::mem::size_of::<FVertex>(),
            attributes.as_ptr().cast(),
            std::mem::size_of::<[f32; 9]>(),
            weights.as_ptr(),
            weights.len(),
            locked.as_ptr(),
            target,
            error,
            (options | meshopt::SimplifyOptions::Permissive | meshopt::SimplifyOptions::Sparse)
                .bits(),
            &mut achieved_error,
        )
    };
    indices.truncate(count);
    (indices, achieved_error)
}

/// A meshopt vertex stream (codec v1) of `count` elements of `stride` bytes.
pub(crate) fn encode_vertices(
    packed: &[u8],
    count: usize,
    stride: usize,
) -> Result<Vec<u8>, String> {
    // SAFETY: `out` holds the bound meshopt asks for; `packed` holds
    // `count * stride` bytes (asserted).
    assert_eq!(packed.len(), count * stride);
    unsafe {
        let bound = meshopt::ffi::meshopt_encodeVertexBufferBound(count, stride);
        let mut out = vec![0u8; bound];
        let n = meshopt::ffi::meshopt_encodeVertexBufferLevel(
            out.as_mut_ptr(),
            out.len(),
            packed.as_ptr().cast(),
            count,
            stride,
            2,
            1,
        );
        if n == 0 {
            return Err("meshopt vertex encoding failed".into());
        }
        out.truncate(n);
        Ok(out)
    }
}

pub(crate) struct CookedPrimitive {
    pub(super) row: PrimitiveRow,
    pub(super) lods: Vec<LodRow>,
    pub(super) baked_levels: Vec<(usize, og_bake::transfer::Pages)>,
    /// Every level's clusters, `index_first` relative to the primitive's
    /// cluster indices; each [`LodRow`] names its range of them (relative here,
    /// made absolute when the blob's table is written).
    pub(super) clusters: Vec<ClusterRow>,
    pub(super) vertices: Vec<u8>,
    pub(super) indices: Vec<u8>,
    pub(super) cluster_indices: Vec<u8>,
    /// The raycast BVH over LOD 0 (empty with `raycast false`): its nodes,
    /// `first` of a leaf relative to `bvh_tris`.
    pub(super) bvh_nodes: Vec<BvhNodeRow>,
    pub(super) bvh_tris: Vec<u32>,
    /// The source (glTF accessor) vertex of every cooked vertex.
    pub sources: Vec<u32>,
    /// Every cooked vertex's foliage ([`read_foliage`]) and whether the
    /// primitive is leaf cards; `None` for a primitive without foliage.
    pub(super) foliage: Option<(Vec<[f32; 8]>, bool)>,
    /// What the art contract changed (`conform::mesh`), and with an
    /// `ArtLine` the outline hull's normal of every cooked vertex.
    pub(super) conformed: Option<super::conform::mesh::Changes>,
    pub(super) hull: Vec<[i16; 2]>,
}

/// zlib at the best level (the BVH sections: about 2.5 times smaller).
pub(super) fn zlib(bytes: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(bytes, 9)
}

/// The bytes the BVH sections take in the blob, compressed.
pub(super) fn w_len_of_bvh(nodes: &[BvhNodeRow], tris: &[u32]) -> usize {
    zlib(bytemuck::cast_slice(nodes)).len() + zlib(bytemuck::cast_slice(tris)).len()
}

/// The raycast BVH og-render builds over LOD 0 as the runtime decodes it
/// (`ordered` dequantised exactly as `runtime::dequantize` does), with its
/// bounds quantised back onto the same grid: they decode to the bounds the
/// build computed, bit for bit.
fn cook_bvh(
    ordered: &[QVertex],
    lod0: &[u32],
    lo: Vec3,
    scale: Vec3,
) -> (Vec<BvhNodeRow>, Vec<u32>) {
    let deq = |a: usize, q: u16| lo[a] + q as f32 / 65535.0 * scale[a];
    let mesh = og_render_data::MeshData {
        vertices: ordered
            .iter()
            .map(|v| og_render_data::Vertex {
                position: [0, 1, 2].map(|a| deq(a, v.p[a])),
                ..bytemuck::Zeroable::zeroed()
            })
            .collect(),
        indices: lod0.to_vec(),
    };
    let bvh = og_render_data::geometry::Bvh::build(&mesh);
    // The grid point of a bound; a value off the grid (never: every bound is
    // some corner's coordinate) takes the next point outwards.
    let quant = |a: usize, v: f32, up: bool| -> u32 {
        if scale[a] <= 0.0 {
            return 0;
        }
        let guess = ((v - lo[a]) / scale[a] * 65535.0).round() as i64;
        let near = (guess - 2..=guess + 2).map(|q| q.clamp(0, 65535) as u16);
        let exact = near.clone().find(|&q| deq(a, q) == v);
        exact.unwrap_or_else(|| {
            if up {
                near.filter(|&q| deq(a, q) >= v).min().unwrap_or(65535)
            } else {
                near.filter(|&q| deq(a, q) <= v).max().unwrap_or(0)
            }
        }) as u32
    };
    let nodes = bvh
        .node_list()
        .iter()
        .map(|n| BvhNodeRow {
            min_xy: quant(0, n.min.x, false) | quant(1, n.min.y, false) << 16,
            min_z_max_x: quant(2, n.min.z, false) | quant(0, n.max.x, true) << 16,
            max_yz: quant(1, n.max.y, true) | quant(2, n.max.z, true) << 16,
            first: n.first,
            count: n.count,
        })
        .collect();
    (nodes, bvh.order().to_vec())
}

/// The largest distance between a level's surface and the full mesh's,
/// both ways: every corner of the level against the full mesh's nearest
/// triangle (a moved vertex) and every corner of the full mesh against the
/// level's (a feature simplified away). The runtime draws the level while
/// this error projects to less than a pixel (og-render's `lod`).
pub(crate) fn level_error(pos: &[[f32; 3]], full: &[u32], level: &[u32]) -> f32 {
    let mesh = |indices: &[u32]| og_render_data::MeshData {
        vertices: pos
            .iter()
            .map(|&p| og_render_data::Vertex {
                position: p,
                ..bytemuck::Zeroable::zeroed()
            })
            .collect(),
        indices: indices.to_vec(),
    };
    let farthest = |corners: &[u32], bvh: &og_render_data::geometry::Bvh| {
        let mut seen = vec![false; pos.len()];
        let mut worst = 0.0f32;
        for &i in corners {
            if !std::mem::replace(&mut seen[i as usize], true)
                && let Some(d) = bvh.distance(Vec3::from(pos[i as usize]))
            {
                worst = worst.max(d);
            }
        }
        worst
    };
    let (full_bvh, level_bvh) = (
        og_render_data::geometry::Bvh::build(&mesh(full)),
        og_render_data::geometry::Bvh::build(&mesh(level)),
    );
    farthest(level, &full_bvh).max(farthest(full, &level_bvh))
}

/// One level's triangles cut into clusters of at most
/// `og_render_data::meshlet::CLUSTER_TRIANGLES` (meshoptimizer's
/// `buildMeshlets`, a quarter of the weight on keeping their normal cones
/// tight, as three.js's `MeshletGeometryUtils`): the level's indices in
/// cluster order, and each cluster's range of them (from 0) with its
/// bounding sphere and normal cone over `pos`.
pub(crate) fn clusterize(pos: &[[f32; 3]], level: &[u32]) -> (Vec<u32>, Vec<ClusterRow>) {
    use og_render_data::meshlet::{CLUSTER_TRIANGLES, CLUSTER_VERTICES};
    if level.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let adapter = positions_adapter(pos);
    let meshlets =
        meshopt::build_meshlets(level, &adapter, CLUSTER_VERTICES, CLUSTER_TRIANGLES, 0.25);
    let mut indices = Vec::with_capacity(level.len());
    let mut rows = Vec::with_capacity(meshlets.len());
    for m in meshlets.iter() {
        let b = meshopt::compute_meshlet_bounds(m, &adapter);
        // meshoptimizer's sphere is approximate (a vertex can lie a few
        // tenths of a percent outside it): widened to the farthest vertex,
        // so the culling never drops a cluster that reaches into view. A
        // wider sphere only makes the cone test more conservative.
        let center = Vec3::from(b.center);
        let radius = m
            .vertices
            .iter()
            .map(|&v| Vec3::from(pos[v as usize]).distance(center))
            .fold(b.radius, f32::max);
        rows.push(ClusterRow {
            index_first: indices.len() as u32,
            triangle_count: (m.triangles.len() / 3) as u32,
            center_x: b.center[0],
            center_y: b.center[1],
            center_z: b.center[2],
            radius,
            cone_x: b.cone_axis[0],
            cone_y: b.cone_axis[1],
            cone_z: b.cone_axis[2],
            cone_cutoff: b.cone_cutoff,
            ..Default::default()
        });
        indices.extend(m.triangles.iter().map(|&t| m.vertices[t as usize]));
    }
    (indices, rows)
}

/// Cooks a primitive's levels (MSFT_lod: the first is the full mesh).
/// `uv`: the `KHR_texture_transform` baked into its uvs ([`super::ext::uv_plan`]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn cook_primitive(
    src: &GltfSource,
    levels: &[gltf::Primitive],
    material: u32,
    normal_mapped: bool,
    opts: &MeshOptions,
    coarsest: bool,
    uv: Option<og_render_data::UvTransform>,
) -> Result<Option<CookedPrimitive>, String> {
    // The primitive's levels of detail as the glTF gives them (MSFT_lod: the
    // first is the full mesh), read one after the other into one vertex
    // list; `given` holds each level's range of `indices`.
    let prim = &levels[0];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let (mut normals, mut uvs, mut colors, mut tangents) = (
        Some(Vec::<[f32; 3]>::new()),
        Some(Vec::<[f32; 2]>::new()),
        Some(Vec::<[f32; 4]>::new()),
        Some(Vec::<[f32; 4]>::new()),
    );
    let mut indices: Vec<u32> = Vec::new();
    let mut keys: Vec<[u32; 2]> = Vec::new();
    let mut given: Vec<std::ops::Range<usize>> = Vec::new();
    // Each vertex's foliage (`read_foliage`), when the full mesh has it.
    let mut foliage: Option<Vec<[f32; 8]>> = None;
    let leaves = leaf_cards(prim);
    for (l, level) in levels.iter().enumerate() {
        let reader = level.reader(|b| Some(&src.buffers[b.index()].0[..]));
        let Some(p) = reader
            .read_positions()
            .map(|p| p.collect::<Vec<[f32; 3]>>())
        else {
            if l == 0 {
                return Ok(None);
            }
            continue;
        };
        if p.is_empty() {
            if l == 0 {
                return Ok(None);
            }
            continue;
        }
        let base = positions.len() as u32;
        let count = p.len();
        // An attribute the full mesh lacks is dropped from every level; one
        // only a coarser level lacks takes its default there.
        fn extend<T: Copy>(
            to: &mut Option<Vec<T>>,
            from: Option<Vec<T>>,
            first: bool,
            count: usize,
            default: T,
        ) {
            match (to.as_mut(), from) {
                (Some(t), Some(f)) => t.extend(
                    f.into_iter()
                        .take(count)
                        .chain(std::iter::repeat(default))
                        .take(count),
                ),
                (Some(_), None) if first => *to = None,
                (Some(t), None) => t.extend(std::iter::repeat_n(default, count)),
                (None, _) => {}
            }
        }
        let first = l == 0;
        extend(
            &mut normals,
            reader.read_normals().map(|n| n.collect()),
            first,
            count,
            [0.0, 0.0, 1.0],
        );
        extend(
            &mut uvs,
            reader.read_tex_coords(0).map(|u| u.into_f32().collect()),
            first,
            count,
            [0.0; 2],
        );
        extend(
            &mut colors,
            reader.read_colors(0).map(|c| c.into_rgba_f32().collect()),
            first,
            count,
            [1.0; 4],
        );
        extend(
            &mut tangents,
            reader.read_tangents().map(|t| t.collect()),
            first,
            count,
            [0.0; 4],
        );
        let mut idx: Vec<u32> = reader
            .read_indices()
            .map(|i| i.into_u32().collect())
            .unwrap_or_else(|| (0..count as u32).collect());
        idx.truncate(idx.len() / 3 * 3);
        if idx.iter().any(|&i| i as usize >= count) {
            return Err(if l == 0 {
                "index out of range".to_string()
            } else {
                format!("MSFT_lod level {l}: index out of range")
            });
        }
        let start = indices.len();
        indices.extend(idx.iter().map(|&i| i + base));
        given.push(start..indices.len());
        positions.extend_from_slice(&p);
        let mut level_keys = vertex_keys(src, level, count);
        let level_foliage = read_foliage(src, level, count)?;
        if l == 0 {
            foliage = level_foliage.as_ref().map(|_| Vec::new());
        }
        if let Some(f) = foliage.as_mut() {
            // A coarser level without its own takes the rigid default.
            let rigid = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
            let level_foliage = level_foliage.unwrap_or_else(|| vec![rigid; count]);
            for (k, fv) in level_keys.iter_mut().zip(&level_foliage) {
                let h = foliage_key(fv);
                *k = [k[0] ^ h[0], k[1] ^ h[1]];
            }
            f.extend(level_foliage);
        }
        keys.extend(level_keys);
    }
    let _ = prim;
    let mut verts: Vec<FVertex> = (0..positions.len())
        .map(|i| FVertex {
            p: positions[i],
            n: normals.as_ref().map_or([0.0, 0.0, 1.0], |n| n[i]),
            uv: uvs
                .as_ref()
                .map_or([0.0; 2], |u| uv.map_or(u[i], |t| t.apply(u[i]))),
            c: colors.as_ref().map_or([1.0; 4], |c| c[i]),
            t: tangents.as_ref().map_or([0.0; 4], |t| t[i]),
            key: keys[i],
        })
        .collect();
    let mut sources: Vec<u32> = (0..positions.len() as u32).collect();
    // The art contract (docs/art.md §2.3, `conform::mesh`): the vertices
    // made to meet it before they are welded.
    let conformed = match &opts.conform {
        Some(c) if c.applies(src) => {
            let ch = super::conform::mesh::primitive(
                c,
                src,
                prim,
                &mut verts,
                &mut indices,
                &mut sources,
            );
            given[0] = 0..indices.len();
            if ch.normals {
                normals = Some(Vec::new());
                tangents = None;
            }
            if ch.colors {
                colors = Some(Vec::new());
            }
            Some(ch)
        }
        _ => None,
    };
    // Leaf cards without levels of their own: thinned ones, each with the
    // error `thin_leaves` measured and its leaf scale.
    let mut thinned: Vec<(f32, f32)> = Vec::new();
    if leaves
        && given.len() == 1
        && let Some(f) = &foliage
    {
        let full = given[0].clone();
        for (range, error, scale) in thin_leaves(
            &mut verts,
            &mut indices,
            &mut sources,
            f,
            full,
            &opts.fractions(),
            coarsest,
        ) {
            given.push(range);
            thinned.push((error, scale));
        }
    }
    let want_tangents = opts.tangents && normal_mapped && uvs.is_some();
    let generate_tangents = want_tangents && tangents.is_none();
    if normals.is_none() || generate_tangents {
        // Unweld to face corners: flat normals (glTF: no normals means flat
        // shading) and mikktspace both work per corner.
        let mut corners: Vec<FVertex> = indices.iter().map(|&i| verts[i as usize]).collect();
        if normals.is_none() {
            for tri in corners.as_chunks_mut::<3>().0 {
                let (a, b, c) = (
                    Vec3::from(tri[0].p),
                    Vec3::from(tri[1].p),
                    Vec3::from(tri[2].p),
                );
                let n = (b - a).cross(c - a).normalize_or(Vec3::Z).to_array();
                tri.iter_mut().for_each(|v| v.n = n);
            }
        }
        if generate_tangents && !mikktspace::generate_tangents(&mut Tangents { v: &mut corners }) {
            return Err("mikktspace could not generate tangents".into());
        }
        let flat: Vec<u32> = (0..corners.len() as u32).collect();
        sources = indices.iter().map(|&i| sources[i as usize]).collect();
        (verts, indices) = weld(&corners, &flat, &mut sources);
    } else {
        (verts, indices) = weld(&verts, &indices, &mut sources);
    }
    if indices.is_empty() {
        return Ok(None);
    }
    // Quantise.
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    let (mut uvlo, mut uvhi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for v in &verts {
        lo = lo.min(Vec3::from(v.p));
        hi = hi.max(Vec3::from(v.p));
        for a in 0..2 {
            uvlo[a] = uvlo[a].min(v.uv[a]);
            uvhi[a] = uvhi[a].max(v.uv[a]);
        }
    }
    let scale = hi - lo;
    let mut uvscale = [uvhi[0] - uvlo[0], uvhi[1] - uvlo[1]];
    let mut attributes = attr::NORMAL;
    if uvs.is_some() {
        attributes |= attr::UV;
    }
    if colors.is_some() {
        attributes |= attr::COLOR;
    }
    if want_tangents {
        attributes |= attr::TANGENT;
    }
    let q: Vec<QVertex> = verts
        .iter()
        .map(|v| QVertex {
            p: [
                unorm16(v.p[0], lo.x, scale.x),
                unorm16(v.p[1], lo.y, scale.y),
                unorm16(v.p[2], lo.z, scale.z),
                0,
            ],
            n: oct_encode(Vec3::from(v.n)),
            uv: if attributes & attr::UV != 0 {
                [
                    unorm16(v.uv[0], uvlo[0], uvscale[0]),
                    unorm16(v.uv[1], uvlo[1], uvscale[1]),
                ]
            } else {
                [0; 2]
            },
            c: if attributes & attr::COLOR != 0 {
                v.c.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
            } else {
                // A recharted level may add a colour stream later. Its
                // white vertices must not turn uncoloured LOD0 black.
                [255; 4]
            },
            t: if attributes & attr::TANGENT != 0 {
                let o = oct_encode(Vec3::new(v.t[0], v.t[1], v.t[2]));
                [o[0], o[1], if v.t[3] < 0.0 { -32767 } else { 32767 }, 0]
            } else {
                [0; 4]
            },
            key: v.key,
        })
        .collect();
    let flat_source = normals.is_none();
    let mut quantized_sources: Vec<u32> = (0..verts.len() as u32).collect();
    let (mut q, welded) = weld(&q, &indices, &mut quantized_sources);
    let mut uv_sources = vec![None; q.len()];
    sources = quantized_sources
        .iter()
        .map(|&i| sources[i as usize])
        .collect();
    let pos: Vec<[f32; 3]> = q
        .iter()
        .map(|v| {
            [
                lo.x + v.p[0] as f32 / 65535.0 * scale.x,
                lo.y + v.p[1] as f32 / 65535.0 * scale.y,
                lo.z + v.p[2] as f32 / 65535.0 * scale.z,
            ]
        })
        .collect();
    let adapter = positions_adapter(&pos);
    // Welding keeps the triangles in order: each given level is its range.
    let mut lod0 = welded[given[0].clone()].to_vec();
    lod0 = meshopt::optimize_vertex_cache(&lod0, q.len());
    meshopt::optimize_overdraw_in_place(&mut lod0, &adapter, opts.overdraw_threshold);
    let mut levels: Vec<(Vec<u32>, f32)> = vec![(lod0.clone(), 0.0)];
    let mut baked_levels = Vec::new();
    let mut texture_errors = std::collections::BTreeMap::new();
    if given.len() > 1 {
        // MSFT_lod: the glTF's own levels, kept as they are (and leaf
        // cards' thinned ones).
        for r in &given[1..] {
            if r.len() >= 3 {
                levels.push((
                    meshopt::optimize_vertex_cache(&welded[r.clone()], q.len()),
                    0.0,
                ));
            }
        }
    } else {
        // Flat-shaded levels rebuild their face normals after simplifying.
        // Permissive collapses keep the source corners' UVs and colours.
        let flat = flat_source
            || q.len() as f64 >= 0.9 * lod0.len() as f64
            // Authored flat meshes can share corners within each plane
            // (the skinned walker does). Their new faces need new normals,
            // not a smooth-normal metric that pins every hard edge.
            || lod0.as_chunks::<3>().0.iter().all(|&[a, b, c]| {
                q[a as usize].n == q[b as usize].n && q[a as usize].n == q[c as usize].n
            });
        let mut lod_vertices = None;
        let mut refaceted: Vec<bool> = vec![false];
        // A foliage mesh's vertices by their welded index (`sources` maps
        // them back to the source's), and the ones its levels keep.
        let foliage_locked = foliage.as_ref().map(|f| {
            let at: Vec<[f32; 8]> = sources.iter().map(|&s| f[s as usize]).collect();
            foliage_locks(&lod0, &at)
        });
        // Leaf cards have no simplified levels: an edge collapse merges
        // cards and changes their coverage (`thin_leaves` drops whole
        // leaves instead, where there are enough of them).
        let fractions = if foliage.is_some() && leaves {
            Vec::new()
        } else {
            opts.fractions()
        };
        for fraction in fractions.into_iter().skip(1) {
            let target = ((lod0.len() as f64 * fraction as f64) as usize) / 3 * 3;
            if target < 3 {
                break;
            }
            let previous = levels.last().map_or(lod0.len(), |l| l.0.len());
            let stalled = |s: &[u32]| s.len() < 3 || s.len() as f64 > previous as f64 * 0.95;
            let mut error = 0.0f32;
            let mut simplified = match &foliage_locked {
                Some(locks) => meshopt::simplify_with_locks(
                    &lod0,
                    &adapter,
                    locks,
                    target,
                    opts.lod_max_error,
                    meshopt::SimplifyOptions::None,
                    Some(&mut error),
                ),
                None => meshopt::simplify(
                    &lod0,
                    &adapter,
                    target,
                    opts.lod_max_error,
                    meshopt::SimplifyOptions::None,
                    Some(&mut error),
                ),
            };
            let mut refacet = false;
            // A level that drops 9% still misses a requested 50% target.
            // Try seam-aware simplification against this level's target,
            // not only when it stalls against the previous level.
            if simplified.len() > target || simplified.len() < 3 {
                let vertices = lod_vertices.get_or_insert_with(|| {
                    quantized_sources
                        .iter()
                        .enumerate()
                        .map(|(i, &s)| FVertex {
                            p: pos[i],
                            ..verts[s as usize]
                        })
                        .collect::<Vec<_>>()
                });
                let (candidate, candidate_error) = simplify_attributes(
                    vertices,
                    &lod0,
                    target,
                    opts.lod_max_error,
                    meshopt::SimplifyOptions::None,
                    foliage_locked.as_deref().unwrap_or(&[]),
                    if flat { 0.0 } else { 0.5 },
                );
                if candidate.len() >= 3
                    && (simplified.len() < 3 || candidate.len() < simplified.len())
                {
                    simplified = candidate;
                    error = candidate_error;
                    refacet = flat;
                }
            }
            // Chart topology must not set a geometric LOD's cost. When
            // preserving it misses the target, this level gets new UVs
            // and its own surface transferred directly from LOD0.
            let bakeable = uvs.is_some()
                && !coarsest
                && og_render_data::gltf::physical_extensions(&src.doc, &prim.material(), |_, _| 0)
                    .present
                    == 0
                && src
                    .json
                    .pointer("/extensions/KHR_materials_variants")
                    .is_none()
                && !opts
                    .conform
                    .as_ref()
                    .is_some_and(|c| c.bakes() && c.applies(src));
            if simplified.len() > target && bakeable {
                let vertices = lod_vertices.as_ref().expect("target miss tried attributes");
                let reduced = super::lod_bake::simplify_geometry(
                    vertices,
                    &lod0,
                    target,
                    opts.lod_max_error,
                    foliage_locked.as_deref().unwrap_or(&[]),
                    if flat { 0.0 } else { 0.5 },
                );
                if reduced.len() >= 3 && reduced.len() <= target {
                    let measured = level_error(&pos, &lod0, &reduced);
                    let baked = super::lod_bake::bake(
                        src,
                        prim,
                        vertices,
                        &lod0,
                        &reduced,
                        measured,
                        want_tangents,
                    )?;
                    let old_lo = uvlo;
                    let old_scale = uvscale;
                    if !want_tangents {
                        for a in 0..2 {
                            uvlo[a] = uvlo[a].min(0.0);
                            uvhi[a] = uvhi[a].max(1.0);
                            uvscale[a] = uvhi[a] - uvlo[a];
                        }
                    }
                    if uvlo != old_lo || uvscale != old_scale {
                        for vertex in &mut q {
                            for a in 0..2 {
                                let u = old_lo[a] + vertex.uv[a] as f32 / 65535.0 * old_scale[a];
                                vertex.uv[a] = unorm16(u, uvlo[a], uvscale[a]);
                            }
                        }
                    }
                    let first = q.len() as u32;
                    for (v, &source) in baked.vertices.iter().zip(&baked.sources) {
                        let mut vertex = q[source as usize];
                        vertex.uv = [0, 1].map(|a| {
                            if want_tangents {
                                unorm16(v.uv[a], 0.0, 1.0)
                            } else {
                                unorm16(v.uv[a], uvlo[a], uvscale[a])
                            }
                        });
                        vertex.c = [255; 4];
                        vertex.n = oct_encode(Vec3::from(v.n));
                        let tangent = oct_encode(Vec3::new(v.t[0], v.t[1], v.t[2]));
                        vertex.t = [
                            tangent[0],
                            tangent[1],
                            if v.t[3] < 0.0 { -32767 } else { 32767 },
                            0,
                        ];
                        q.push(vertex);
                        uv_sources.push(want_tangents.then_some(source));
                        sources.push(sources[source as usize]);
                    }
                    simplified = baked.indices.iter().map(|&i| first + i).collect();
                    texture_errors.insert(levels.len(), baked.texel);
                    baked_levels.push((levels.len(), baked.pages));
                    attributes |= attr::UV | attr::TANGENT;
                    error = measured;
                    refacet = false;
                }
            }
            if stalled(&simplified) {
                break;
            }
            levels.push((simplified, error));
            refaceted.push(refacet);
        }
        // Flat levels keep their own corners with each new face's normal.
        for (l, r) in levels.iter_mut().zip(&refaceted) {
            if !*r {
                continue;
            }
            let mut out = Vec::with_capacity(l.0.len());
            for tri in l.0.as_chunks::<3>().0 {
                let [a, b, c] = tri.map(|i| Vec3::from(pos[i as usize]));
                let n = oct_encode((b - a).cross(c - a).normalize_or(Vec3::Y));
                for &i in tri {
                    let mut v = q[i as usize];
                    v.n = n;
                    out.push(q.len() as u32);
                    q.push(v);
                    uv_sources.push(None);
                    sources.push(sources[i as usize]);
                }
            }
            l.0 = out;
        }
        for l in levels.iter_mut().skip(1) {
            l.0 = meshopt::optimize_vertex_cache(&l.0, q.len());
        }
    }
    // Each level's error is measured against the full mesh over the
    // positions the runtime decodes (the simplifier's own estimate, kept
    // until here, is replaced; a glTF's MSFT_lod level gets one too).
    let deq = |v: &QVertex| {
        [
            lo.x + v.p[0] as f32 / 65535.0 * scale.x,
            lo.y + v.p[1] as f32 / 65535.0 * scale.y,
            lo.z + v.p[2] as f32 / 65535.0 * scale.z,
        ]
    };
    let pos_all: Vec<[f32; 3]> = q.iter().map(deq).collect();
    // A thinned level's is how far its leaves moved: the leaves it drops
    // are covered by the larger ones beside them, not by a nearby surface.
    for (k, l) in levels.iter_mut().enumerate().skip(1) {
        l.1 = match thinned.get(k - 1) {
            Some(&(moved, _)) => moved,
            None => level_error(&pos_all, &lod0, &l.0),
        }
        .max(texture_errors.get(&k).copied().unwrap_or(0.0));
    }
    // Each level's leaf scale (a thinned one's, `thin_leaves`) and where
    // it draws the vertices: the cluster bounds and the selection sphere
    // are of what is drawn.
    let mut leaf_scales: Vec<f32> = (0..levels.len())
        .map(|k| {
            k.checked_sub(1)
                .and_then(|k| thinned.get(k))
                .map_or(1.0, |t| t.1)
        })
        .collect();
    // A streaming proxy keeps only the coarsest level (and the vertices it
    // uses; `thin_leaves` copied a thinned one's where it draws them).
    if coarsest && let Some((last, _)) = levels.pop() {
        levels = vec![(last, 0.0)];
        leaf_scales = vec![1.0];
    }
    let drawn_at = |k: usize| -> std::borrow::Cow<'_, [[f32; 3]]> {
        let s = leaf_scales[k];
        match &foliage {
            Some(f) if s != 1.0 => pos_all
                .iter()
                .zip(&sources)
                .map(|(p, &src)| {
                    let a = Vec3::new(f[src as usize][4], f[src as usize][5], f[src as usize][6]);
                    (a + (Vec3::from(*p) - a) * s).to_array()
                })
                .collect(),
            _ => std::borrow::Cow::Borrowed(&pos_all[..]),
        }
    };
    let drawn: Vec<_> = (0..levels.len()).map(drawn_at).collect();
    // Keep the simulation mesh's triangle and vertex order independent of
    // meshlets: changing the order changes physics contacts and ray hits.
    // Only the draw stream is reordered into contiguous clusters.
    let mut all: Vec<u32> = Vec::new();
    let mut cluster_indices = Vec::new();
    let mut lods = Vec::new();
    let mut clusters = Vec::new();
    // Selection per cluster in the per-level chain (format's ClusterRow):
    // each level's error at least the one before, the next level's as the
    // parent's, all projected from the primitive's bounding sphere.
    let chain: Vec<f32> = levels
        .iter()
        .scan(0.0f32, |e, l| {
            *e = e.max(l.1);
            Some(*e)
        })
        .collect();
    let used = levels
        .iter()
        .zip(&drawn)
        .flat_map(|(l, at)| l.0.iter().map(move |&i| Vec3::from(at[i as usize])));
    let (blo, bhi) = used.clone().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    );
    let sphere_center = (blo + bhi) * 0.5;
    let sphere_radius = used
        .map(|p| p.distance(sphere_center))
        .fold(0.0f32, f32::max);
    for (k, (l, error)) in levels.iter().enumerate() {
        let (ordered, mut cut) = if opts.clusters {
            clusterize(&drawn[k], l)
        } else {
            (Vec::new(), Vec::new())
        };
        let first = all.len() as u32;
        for c in &mut cut {
            c.index_first += first;
            c.error = chain[k];
            c.parent_error = chain.get(k + 1).copied().unwrap_or(f32::MAX);
            [c.error_center_x, c.error_center_y, c.error_center_z] = sphere_center.to_array();
            c.error_radius = sphere_radius;
            c.group = k as u32;
        }
        lods.push(LodRow {
            index_first: first,
            // The level's own count: `ordered` is empty without clusters.
            index_count: l.len() as u32,
            error: *error,
            cluster_first: clusters.len() as u32,
            cluster_count: cut.len() as u32,
            material: NONE,
            leaf_scale: leaf_scales[k],
        });
        clusters.extend(cut);
        all.extend_from_slice(l);
        cluster_indices.extend_from_slice(&ordered);
    }
    // Reorder vertices for fetch locality (LOD 0 first decides the order).
    let mut remap = vec![0u32; q.len()];
    let vertex_count = unsafe {
        meshopt::ffi::meshopt_optimizeVertexFetchRemap(
            remap.as_mut_ptr(),
            all.as_ptr(),
            all.len(),
            q.len(),
        )
    };
    let mut ordered = vec![QVertex::default(); vertex_count];
    let mut ordered_sources = vec![0u32; vertex_count];
    let mut ordered_uv_sources = vec![None; vertex_count];
    for (old, &new) in remap.iter().enumerate() {
        if new != u32::MAX {
            ordered[new as usize] = q[old];
            ordered_sources[new as usize] = sources[old];
            ordered_uv_sources[new as usize] = uv_sources[old].map(|s| remap[s as usize]);
        }
    }
    // One full source stream; recharted corners add only source + UV.
    let base_count = ordered_uv_sources.iter().filter(|s| s.is_none()).count();
    let order: Vec<_> = (0..vertex_count)
        .filter(|&i| ordered_uv_sources[i].is_none())
        .chain((0..vertex_count).filter(|&i| ordered_uv_sources[i].is_some()))
        .collect();
    let mut partition = vec![0u32; vertex_count];
    for (new, &old) in order.iter().enumerate() {
        partition[old] = new as u32;
    }
    let mut uv_refs = Vec::<[u32; 2]>::new();
    for &old in &order[base_count..] {
        let source = partition[ordered_uv_sources[old].expect("atlas corner") as usize];
        assert!((source as usize) < base_count);
        let uv = ordered[old].uv;
        uv_refs.push([source, u32::from(uv[0]) | u32::from(uv[1]) << 16]);
    }
    ordered = order.iter().map(|&i| ordered[i]).collect();
    ordered_sources = order.iter().map(|&i| ordered_sources[i]).collect();
    for i in all.iter_mut().chain(&mut cluster_indices) {
        *i = partition[remap[*i as usize] as usize];
    }
    let mut stride = 12;
    for (bit, size) in [(attr::UV, 4), (attr::COLOR, 4), (attr::TANGENT, 8)] {
        if attributes & bit != 0 {
            stride += size;
        }
    }
    let mut packed = Vec::with_capacity(vertex_count * stride);
    for v in &ordered {
        let b = bytemuck::bytes_of(v);
        packed.extend_from_slice(&b[0..12]);
        if attributes & attr::UV != 0 {
            packed.extend_from_slice(&b[12..16]);
        }
        if attributes & attr::COLOR != 0 {
            packed.extend_from_slice(&b[16..20]);
        }
        if attributes & attr::TANGENT != 0 {
            packed.extend_from_slice(&b[20..28]);
        }
    }
    let (bvh_nodes, bvh_tris) = if opts.raycast {
        let l0 = &lods[0];
        let lod0 = &all[l0.index_first as usize..(l0.index_first + l0.index_count) as usize];
        cook_bvh(&ordered, lod0, lo, scale)
    } else {
        (Vec::new(), Vec::new())
    };
    let hull = match &opts.conform {
        Some(c) if c.line.is_some() && conformed.is_some() => {
            let deq: Vec<[f32; 3]> = ordered
                .iter()
                .map(|v| [0, 1, 2].map(|a| lo[a] + v.p[a] as f32 / 65535.0 * scale[a]))
                .collect();
            let lod0 = &all[..lods[0].index_count as usize];
            super::conform::mesh::hull_normals(&deq, lod0)
                .into_iter()
                .map(|n| oct_encode(Vec3::from(n)))
                .collect()
        }
        _ => Vec::new(),
    };
    let vertices = if uv_refs.is_empty() {
        encode_vertices(&packed, vertex_count, stride)?
    } else {
        let base = encode_vertices(&packed[..base_count * stride], base_count, stride)?;
        let refs = encode_vertices(bytemuck::cast_slice(&uv_refs), uv_refs.len(), 8)?;
        attributes |= attr::LOD_UV_REFS;
        let mut bytes = Vec::with_capacity(8 + base.len() + refs.len());
        bytes.extend((base_count as u32).to_le_bytes());
        bytes.extend((base.len() as u32).to_le_bytes());
        bytes.extend(base);
        bytes.extend(refs);
        bytes
    };
    let index_bytes =
        meshopt::encode_index_buffer(&all, vertex_count).map_err(|e| format!("meshopt: {e}"))?;
    // No stream at all without clusters (`cook { clusters false }`).
    let cluster_indices = if opts.clusters {
        meshopt::encode_index_buffer(&cluster_indices, vertex_count)
            .map_err(|e| format!("meshopt: {e}"))?
    } else {
        Vec::new()
    };
    Ok(Some(CookedPrimitive {
        row: PrimitiveRow {
            material,
            attributes,
            vertex_count: vertex_count as u32,
            vertex_stride: stride as u32,
            index_count: all.len() as u32,
            lod_count: lods.len() as u32,
            pos_offset_x: lo.x,
            pos_offset_y: lo.y,
            pos_offset_z: lo.z,
            pos_scale_x: scale.x,
            pos_scale_y: scale.y,
            pos_scale_z: scale.z,
            uv_offset_x: if attributes & attr::UV != 0 {
                uvlo[0]
            } else {
                0.0
            },
            uv_offset_y: if attributes & attr::UV != 0 {
                uvlo[1]
            } else {
                0.0
            },
            uv_scale_x: if attributes & attr::UV != 0 {
                uvscale[0]
            } else {
                0.0
            },
            uv_scale_y: if attributes & attr::UV != 0 {
                uvscale[1]
            } else {
                0.0
            },
            ..Default::default()
        },
        lods,
        baked_levels,
        clusters,
        vertices,
        indices: index_bytes,
        cluster_indices,
        bvh_nodes,
        bvh_tris,
        foliage: foliage.map(|f| {
            let at = ordered_sources.iter().map(|&s| f[s as usize]).collect();
            (at, leaves)
        }),
        sources: ordered_sources,
        conformed,
        hull,
    }))
}
