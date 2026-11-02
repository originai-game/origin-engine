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

struct Tangents<'a> {
    v: &'a mut [FVertex],
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
fn weld<T: bytemuck::Pod>(
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
fn vertex_keys(src: &GltfSource, prim: &gltf::Primitive, count: usize) -> Vec<[u32; 2]> {
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
    /// Every level's clusters, `index_first` relative to the primitive's
    /// indices; each [`LodRow`] names its range of them (relative here,
    /// made absolute when the blob's table is written).
    pub(super) clusters: Vec<ClusterRow>,
    pub(super) vertices: Vec<u8>,
    pub(super) indices: Vec<u8>,
    /// The raycast BVH over LOD 0 (empty with `raycast false`): its nodes,
    /// `first` of a leaf relative to `bvh_tris`.
    pub(super) bvh_nodes: Vec<BvhNodeRow>,
    pub(super) bvh_tris: Vec<u32>,
    /// The source (glTF accessor) vertex of every cooked vertex.
    pub sources: Vec<u32>,
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
    let mesh = og_render::MeshData {
        vertices: ordered
            .iter()
            .map(|v| og_render::Vertex {
                position: [0, 1, 2].map(|a| deq(a, v.p[a])),
                ..bytemuck::Zeroable::zeroed()
            })
            .collect(),
        indices: lod0.to_vec(),
    };
    let bvh = og_render::geometry::Bvh::build(&mesh);
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
    let mesh = |indices: &[u32]| og_render::MeshData {
        vertices: pos
            .iter()
            .map(|&p| og_render::Vertex {
                position: p,
                ..bytemuck::Zeroable::zeroed()
            })
            .collect(),
        indices: indices.to_vec(),
    };
    let farthest = |corners: &[u32], bvh: &og_render::geometry::Bvh| {
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
        og_render::geometry::Bvh::build(&mesh(full)),
        og_render::geometry::Bvh::build(&mesh(level)),
    );
    farthest(level, &full_bvh).max(farthest(full, &level_bvh))
}

/// One level's triangles cut into clusters of at most
/// `og_render::meshlet::CLUSTER_TRIANGLES` (meshoptimizer's
/// `buildMeshlets`, a quarter of the weight on keeping their normal cones
/// tight, as three.js's `MeshletGeometryUtils`): the level's indices in
/// cluster order, and each cluster's range of them (from 0) with its
/// bounding sphere and normal cone over `pos`.
pub(crate) fn clusterize(pos: &[[f32; 3]], level: &[u32]) -> (Vec<u32>, Vec<ClusterRow>) {
    use og_render::meshlet::{CLUSTER_TRIANGLES, CLUSTER_VERTICES};
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
    uv: Option<og_render::UvTransform>,
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
        keys.extend(vertex_keys(src, level, count));
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
        sources = indices.clone();
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
    let uvscale = [uvhi[0] - uvlo[0], uvhi[1] - uvlo[1]];
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
                [0; 4]
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
    let (mut q, welded) = weld(&q, &indices, &mut sources);
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
    if given.len() > 1 {
        // MSFT_lod: the glTF's own levels, kept as they are.
        for r in &given[1..] {
            if r.len() >= 3 {
                levels.push((
                    meshopt::optimize_vertex_cache(&welded[r.clone()], q.len()),
                    0.0,
                ));
            }
        }
    } else {
        // A mesh whose corners are not shared (flat shading: no normals, or
        // every face its own vertices) has every edge on a border, which
        // meshoptimizer will not move: such a level is simplified over the
        // vertices welded by position alone (a shadow index buffer) and
        // given its own corners with face normals below.
        let flat = flat_source || q.len() as f64 >= 0.9 * lod0.len() as f64;
        // (the welded index buffer, each vertex's welded index, the welded positions)
        type Welded = (Vec<u32>, Vec<u32>, Vec<[f32; 3]>);
        let mut shadow: Option<Welded> = None;
        let mut refaceted: Vec<bool> = vec![false];
        for fraction in opts.fractions().into_iter().skip(1) {
            let target = ((lod0.len() as f64 * fraction as f64) as usize) / 3 * 3;
            if target < 3 {
                break;
            }
            let previous = levels.last().map_or(lod0.len(), |l| l.0.len());
            let stalled = |s: &[u32]| s.len() < 3 || s.len() as f64 > previous as f64 * 0.95;
            let mut error = 0.0f32;
            let mut simplified = meshopt::simplify(
                &lod0,
                &adapter,
                target,
                opts.lod_max_error,
                meshopt::SimplifyOptions::None,
                Some(&mut error),
            );
            let mut by_position = false;
            if stalled(&simplified) {
                // Over the distinct positions alone: vertices that share a
                // position but no triangle would count as seams and lock it.
                let (compact, back, compact_pos) = shadow.get_or_insert_with(|| {
                    let sh = meshopt::generate_shadow_indices(&lod0, &adapter);
                    let mut at = vec![u32::MAX; pos.len()];
                    let (mut back, mut compact_pos) = (Vec::new(), Vec::new());
                    let compact: Vec<u32> = sh
                        .iter()
                        .map(|&i| {
                            if at[i as usize] == u32::MAX {
                                at[i as usize] = back.len() as u32;
                                back.push(i);
                                compact_pos.push(pos[i as usize]);
                            }
                            at[i as usize]
                        })
                        .collect();
                    (compact, back, compact_pos)
                });
                simplified = meshopt::simplify(
                    compact,
                    &positions_adapter(compact_pos),
                    target,
                    opts.lod_max_error,
                    meshopt::SimplifyOptions::None,
                    Some(&mut error),
                )
                .into_iter()
                .map(|i| back[i as usize])
                .collect();
                by_position = flat;
            }
            if stalled(&simplified) {
                break;
            }
            levels.push((simplified, error));
            refaceted.push(by_position);
        }
        // The levels simplified by position over a flat mesh: their own
        // corners, each with its face's normal (the welded vertex's normal
        // is some other face's).
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
    for l in levels.iter_mut().skip(1) {
        l.1 = level_error(&pos_all, &lod0, &l.0);
    }
    // A streaming proxy keeps only the coarsest level (and the vertices it uses).
    if coarsest && let Some((last, _)) = levels.pop() {
        levels = vec![(last, 0.0)];
    }
    // Every level cut into clusters (og-render's `meshlet`): its indices
    // reordered so each cluster is a range of them.
    let mut all: Vec<u32> = Vec::new();
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
    let used = levels.iter().flat_map(|l| l.0.iter());
    let (blo, bhi) = used.clone().fold(
        (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
        |(lo, hi), &i| {
            (
                lo.min(pos_all[i as usize].into()),
                hi.max(pos_all[i as usize].into()),
            )
        },
    );
    let sphere_center = (blo + bhi) * 0.5;
    let sphere_radius = used
        .map(|&i| Vec3::from(pos_all[i as usize]).distance(sphere_center))
        .fold(0.0f32, f32::max);
    for (k, (l, error)) in levels.iter().enumerate() {
        let (ordered, mut cut) = clusterize(&pos_all, l);
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
            index_count: ordered.len() as u32,
            error: *error,
            cluster_first: clusters.len() as u32,
            cluster_count: cut.len() as u32,
        });
        clusters.extend(cut);
        all.extend_from_slice(&ordered);
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
    for (old, &new) in remap.iter().enumerate() {
        if new != u32::MAX {
            ordered[new as usize] = q[old];
            ordered_sources[new as usize] = sources[old];
        }
    }
    for i in all.iter_mut() {
        *i = remap[*i as usize];
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
    let vertices = encode_vertices(&packed, vertex_count, stride)?;
    let index_bytes =
        meshopt::encode_index_buffer(&all, vertex_count).map_err(|e| format!("meshopt: {e}"))?;
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
        clusters,
        vertices,
        indices: index_bytes,
        bvh_nodes,
        bvh_tris,
        sources: ordered_sources,
        conformed,
        hull,
    }))
}
