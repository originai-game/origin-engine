//! glTF 2.0 → model blob: meshoptimizer-processed primitives with an LOD
//! chain, materials, the node graph and scenes. Textures become their own
//! blobs (see `texture.rs`); the model refers to them by asset id.

use super::{CookError, MeshOptions};
use crate::format::{self, *};
use glam::{Mat4, Quat, Vec3};
use og_blob::BlobWriter;
use std::path::Path;

/// A texture the model needs: which image, cooked as which kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureUse {
    pub image: usize,
    pub kind: u32,
}

/// A parsed glTF: the document, its buffers and the encoded image bytes.
pub struct GltfSource {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    /// Encoded image bytes (PNG/JPEG) per glTF image.
    pub images: Vec<Vec<u8>>,
    /// Bytes of every external file the glTF referenced (part of its source hash).
    pub external: Vec<(String, Vec<u8>)>,
    pub textures: Vec<TextureUse>,
    pub warnings: Vec<String>,
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

fn read_uri(
    uri: &str,
    base: &Path,
    external: &mut Vec<(String, Vec<u8>)>,
) -> Result<Vec<u8>, String> {
    if let Some(rest) = uri.strip_prefix("data:") {
        let (_, data) = rest
            .split_once(";base64,")
            .ok_or("only base64 data URIs are supported")?;
        return base64_decode(data).ok_or_else(|| "invalid base64 data URI".into());
    }
    let mut path = String::new();
    let b = uri.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(&uri[i + 1..i + 3], 16)
        {
            path.push(v as char);
            i += 3;
            continue;
        }
        path.push(b[i] as char);
        i += 1;
    }
    let full = base.join(&path);
    let bytes = std::fs::read(&full).map_err(|e| format!("{}: {e}", full.display()))?;
    external.push((path, bytes.clone()));
    Ok(bytes)
}

/// Parses a `.glb`/`.gltf`, resolving its buffers and image bytes, and lists
/// the textures its materials use in first-use order.
pub fn load(bytes: &[u8], dir: &Path) -> Result<GltfSource, String> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| format!("glTF: {e}"))?;
    let mut blob = gltf.blob;
    let doc = gltf.document;
    let mut external = Vec::new();
    let mut buffers = Vec::new();
    for b in doc.buffers() {
        let data = match b.source() {
            gltf::buffer::Source::Bin => blob.take().ok_or("glTF: the BIN chunk is missing")?,
            gltf::buffer::Source::Uri(uri) => read_uri(uri, dir, &mut external)?,
        };
        if data.len() < b.length() {
            return Err(format!(
                "glTF: buffer {} is shorter than declared",
                b.index()
            ));
        }
        buffers.push(gltf::buffer::Data(data));
    }
    let mut images = Vec::new();
    for img in doc.images() {
        images.push(match img.source() {
            gltf::image::Source::View { view, .. } => {
                let buf = &buffers[view.buffer().index()].0;
                buf.get(view.offset()..view.offset() + view.length())
                    .ok_or("glTF: image view out of range")?
                    .to_vec()
            }
            gltf::image::Source::Uri { uri, .. } => read_uri(uri, dir, &mut external)?,
        });
    }
    let mut textures: Vec<TextureUse> = Vec::new();
    let mut warnings = Vec::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let slots = [
            (
                pbr.base_color_texture()
                    .map(|t| (t.texture(), t.tex_coord())),
                kind::COLOR,
            ),
            (
                pbr.metallic_roughness_texture()
                    .map(|t| (t.texture(), t.tex_coord())),
                kind::LINEAR,
            ),
            (
                m.normal_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::NORMAL,
            ),
            (
                m.emissive_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::COLOR,
            ),
            (
                m.occlusion_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::LINEAR,
            ),
        ];
        // The physical extensions' textures (`KHR_materials_*`).
        og_render::gltf::physical_extensions(&doc, &m, |t, k| {
            let u = TextureUse {
                image: t.source().index(),
                kind: texture_kind(k),
            };
            if !textures.contains(&u) {
                textures.push(u);
            }
        });
        for (slot, k) in slots {
            if let Some((t, coord)) = slot {
                if coord != 0 {
                    warnings.push(format!(
                        "material '{}' samples TEXCOORD_{coord}; only TEXCOORD_0 is cooked",
                        m.name().unwrap_or("")
                    ));
                }
                let u = TextureUse {
                    image: t.source().index(),
                    kind: k,
                };
                if !textures.contains(&u) {
                    textures.push(u);
                }
            }
        }
    }
    Ok(GltfSource {
        doc,
        buffers,
        images,
        external,
        textures,
        warnings,
    })
}

/// Full-precision vertex before quantisation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct FVertex {
    p: [f32; 3],
    n: [f32; 3],
    uv: [f32; 2],
    c: [f32; 4],
    t: [f32; 4],
    /// A hash of the source vertex's skin and morph data (zero without
    /// either), so welding never merges vertices that skin or morph apart.
    key: [u32; 2],
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
pub(super) fn encode_vertices(
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

pub(super) struct CookedPrimitive {
    row: PrimitiveRow,
    lods: Vec<LodRow>,
    vertices: Vec<u8>,
    indices: Vec<u8>,
    /// The source (glTF accessor) vertex of every cooked vertex.
    pub sources: Vec<u32>,
}

pub(super) fn cook_primitive(
    src: &GltfSource,
    prim: &gltf::Primitive,
    material: u32,
    normal_mapped: bool,
    opts: &MeshOptions,
    coarsest: bool,
) -> Result<Option<CookedPrimitive>, String> {
    let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
    let Some(positions) = reader
        .read_positions()
        .map(|p| p.collect::<Vec<[f32; 3]>>())
    else {
        return Ok(None);
    };
    if positions.is_empty() {
        return Ok(None);
    }
    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|n| n.collect());
    let uvs: Option<Vec<[f32; 2]>> = reader.read_tex_coords(0).map(|u| u.into_f32().collect());
    let colors: Option<Vec<[f32; 4]>> = reader.read_colors(0).map(|c| c.into_rgba_f32().collect());
    let tangents: Option<Vec<[f32; 4]>> = reader.read_tangents().map(|t| t.collect());
    let mut indices: Vec<u32> = reader
        .read_indices()
        .map(|i| i.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());
    indices.truncate(indices.len() / 3 * 3);
    if indices.iter().any(|&i| i as usize >= positions.len()) {
        return Err("index out of range".into());
    }
    let keys = vertex_keys(src, prim, positions.len());
    let mut verts: Vec<FVertex> = (0..positions.len())
        .map(|i| FVertex {
            p: positions[i],
            n: normals.as_ref().map_or([0.0, 0.0, 1.0], |n| n[i]),
            uv: uvs.as_ref().map_or([0.0; 2], |u| u[i]),
            c: colors.as_ref().map_or([1.0; 4], |c| c[i]),
            t: tangents.as_ref().map_or([0.0; 4], |t| t[i]),
            key: keys[i],
        })
        .collect();
    let mut sources: Vec<u32> = (0..positions.len() as u32).collect();
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
    let (q, mut lod0) = weld(&q, &indices, &mut sources);
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
    lod0 = meshopt::optimize_vertex_cache(&lod0, q.len());
    meshopt::optimize_overdraw_in_place(&mut lod0, &adapter, opts.overdraw_threshold);
    let mut levels: Vec<(Vec<u32>, f32)> = vec![(lod0.clone(), 0.0)];
    for level in 1..opts.lod_levels.max(1) {
        let target =
            ((lod0.len() as f64 * (opts.lod_ratio as f64).powi(level as i32)) as usize) / 3 * 3;
        if target < 3 {
            break;
        }
        let mut error = 0.0f32;
        let simplified = meshopt::simplify(
            &lod0,
            &adapter,
            target,
            opts.lod_max_error,
            meshopt::SimplifyOptions::None,
            Some(&mut error),
        );
        let previous = levels.last().map_or(lod0.len(), |l| l.0.len());
        if simplified.len() < 3 || simplified.len() as f64 > previous as f64 * 0.95 {
            break;
        }
        levels.push((meshopt::optimize_vertex_cache(&simplified, q.len()), error));
    }
    // A streaming proxy keeps only the coarsest level (and the vertices it uses).
    if coarsest && let Some((last, _)) = levels.pop() {
        levels = vec![(last, 0.0)];
    }
    let mut all: Vec<u32> = Vec::new();
    let mut lods = Vec::new();
    for (l, error) in &levels {
        lods.push(LodRow {
            index_first: all.len() as u32,
            index_count: l.len() as u32,
            error: *error,
        });
        all.extend_from_slice(l);
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
        vertices,
        indices: index_bytes,
        sources: ordered_sources,
    }))
}

/// The glTF's nodes in topological order (roots by index, then depth
/// first): the order both the model and the animation blob use.
pub(super) struct NodeOrder {
    /// The cooked index of every glTF node.
    pub new_index: Vec<u32>,
    /// In cooked order; names through the caller's string pool.
    pub rows: Vec<NodeRow>,
}

pub(super) fn node_order(
    doc: &gltf::Document,
    add: &mut dyn FnMut(&str) -> (u32, u32),
) -> Result<NodeOrder, CookError> {
    let node_count = doc.nodes().count();
    let mut parent = vec![NONE; node_count];
    for n in doc.nodes() {
        for c in n.children() {
            parent[c.index()] = n.index() as u32;
        }
    }
    let nodes_vec: Vec<gltf::Node> = doc.nodes().collect();
    let mut order = Vec::with_capacity(node_count);
    let mut stack: Vec<usize> = (0..node_count)
        .rev()
        .filter(|&i| parent[i] == NONE)
        .collect();
    while let Some(i) = stack.pop() {
        order.push(i);
        let children: Vec<usize> = nodes_vec[i].children().map(|c| c.index()).collect();
        stack.extend(children.into_iter().rev());
    }
    if order.len() != node_count {
        return Err(CookError("glTF: the node hierarchy has a cycle".into()));
    }
    let mut new_index = vec![0u32; node_count];
    for (new, &old) in order.iter().enumerate() {
        new_index[old] = new as u32;
    }
    let mut rows = Vec::with_capacity(node_count);
    for &old in &order {
        let n = &nodes_vec[old];
        let (t, r, s) = n.transform().decomposed();
        let (name_offset, name_len) = add(n.name().unwrap_or(""));
        rows.push(NodeRow {
            parent: if parent[old] == NONE {
                NONE
            } else {
                new_index[parent[old] as usize]
            },
            mesh: n.mesh().map_or(NONE, |m| m.index() as u32),
            name_offset,
            name_len,
            tx: t[0],
            ty: t[1],
            tz: t[2],
            rx: r[0],
            ry: r[1],
            rz: r[2],
            rw: r[3],
            sx: s[0],
            sy: s[1],
            sz: s[2],
        });
    }
    Ok(NodeOrder { new_index, rows })
}

pub(super) struct Strings(pub Vec<u8>);
impl Strings {
    pub fn add(&mut self, s: &str) -> (u32, u32) {
        let at = self.0.len() as u32;
        self.0.extend_from_slice(s.as_bytes());
        (at, s.len() as u32)
    }
}

/// The cooked texture kind of what a glTF material's texture holds.
fn texture_kind(k: og_render::gltf::TextureKind) -> u32 {
    match k {
        og_render::gltf::TextureKind::Color => kind::COLOR,
        og_render::gltf::TextureKind::Normal => kind::NORMAL,
        og_render::gltf::TextureKind::Linear => kind::LINEAR,
    }
}

/// What a model cook reports besides the blob.
pub struct ModelNotes {
    pub notes: Vec<String>,
}

/// Cooks a parsed glTF into a model blob. `texture_ids` are the asset ids
/// of [`GltfSource::textures`], in order. A `proxy` (a streaming group's
/// stand-in, `#proxy`) keeps each primitive's coarsest LOD and no textures.
pub fn cook(
    src: &GltfSource,
    texture_ids: &[String],
    opts: &MeshOptions,
    stamp: u64,
    proxy: bool,
) -> Result<(Vec<u8>, ModelNotes), CookError> {
    let doc = &src.doc;
    let mut notes = src.warnings.clone();
    let mut strings = Strings(Vec::new());
    let tex_of = |t: Option<gltf::texture::Texture>, k: u32| -> u32 {
        t.filter(|_| !proxy)
            .and_then(|t| {
                src.textures
                    .iter()
                    .position(|u| u.image == t.source().index() && u.kind == k)
            })
            .map_or(NONE, |i| i as u32)
    };
    let mut materials = Vec::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let e = m.emissive_factor();
        let strength = m.emissive_strength().unwrap_or(1.0);
        let (name_offset, name_len) = strings.add(m.name().unwrap_or(""));
        let b = pbr.base_color_factor();
        let p =
            og_render::gltf::physical_extensions(doc, &m, |t, k| tex_of(Some(t), texture_kind(k)));
        let t = |x: Option<u32>| x.unwrap_or(NONE);
        materials.push(MaterialRow {
            physical: p.present,
            clearcoat: p.clearcoat,
            clearcoat_roughness: p.clearcoat_roughness,
            clearcoat_normal_scale: p.clearcoat_normal_scale,
            sheen_r: p.sheen_color[0],
            sheen_g: p.sheen_color[1],
            sheen_b: p.sheen_color[2],
            sheen_roughness: p.sheen_roughness,
            transmission: p.transmission,
            thickness: p.thickness,
            attenuation_distance: p.attenuation_distance,
            attenuation_r: p.attenuation_color[0],
            attenuation_g: p.attenuation_color[1],
            attenuation_b: p.attenuation_color[2],
            ior: p.ior,
            iridescence: p.iridescence,
            iridescence_ior: p.iridescence_ior,
            iridescence_min: p.iridescence_thickness[0],
            iridescence_max: p.iridescence_thickness[1],
            anisotropy: p.anisotropy,
            anisotropy_rotation: p.anisotropy_rotation,
            clearcoat_texture: t(p.clearcoat_texture),
            clearcoat_roughness_texture: t(p.clearcoat_roughness_texture),
            clearcoat_normal_texture: t(p.clearcoat_normal_texture),
            sheen_color_texture: t(p.sheen_color_texture),
            sheen_roughness_texture: t(p.sheen_roughness_texture),
            transmission_texture: t(p.transmission_texture),
            thickness_texture: t(p.thickness_texture),
            iridescence_texture: t(p.iridescence_texture),
            iridescence_thickness_texture: t(p.iridescence_thickness_texture),
            anisotropy_texture: t(p.anisotropy_texture),
            base_r: b[0],
            base_g: b[1],
            base_b: b[2],
            base_a: b[3],
            emissive_r: e[0] * strength,
            emissive_g: e[1] * strength,
            emissive_b: e[2] * strength,
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            normal_scale: m.normal_texture().map_or(1.0, |t| t.scale()),
            occlusion_strength: m.occlusion_texture().map_or(1.0, |t| t.strength()),
            alpha_mode: match m.alpha_mode() {
                gltf::material::AlphaMode::Opaque => alpha::OPAQUE,
                gltf::material::AlphaMode::Mask => alpha::MASK,
                gltf::material::AlphaMode::Blend => alpha::BLEND,
            },
            alpha_cutoff: m.alpha_cutoff().unwrap_or(0.5),
            double_sided: m.double_sided() as u32,
            base_color_texture: tex_of(pbr.base_color_texture().map(|t| t.texture()), kind::COLOR),
            metallic_roughness_texture: tex_of(
                pbr.metallic_roughness_texture().map(|t| t.texture()),
                kind::LINEAR,
            ),
            normal_texture: tex_of(m.normal_texture().map(|t| t.texture()), kind::NORMAL),
            emissive_texture: tex_of(m.emissive_texture().map(|t| t.texture()), kind::COLOR),
            occlusion_texture: tex_of(m.occlusion_texture().map(|t| t.texture()), kind::LINEAR),
            name_offset,
            name_len,
        });
    }
    // The default material (glTF's, matching og-render's gltf bridge).
    let default_material = materials.len() as u32;
    let (name_offset, name_len) = strings.add("default");
    materials.push(MaterialRow {
        base_r: 0.8,
        base_g: 0.8,
        base_b: 0.8,
        base_a: 1.0,
        roughness: 0.6,
        normal_scale: 1.0,
        occlusion_strength: 1.0,
        alpha_cutoff: 0.5,
        base_color_texture: NONE,
        metallic_roughness_texture: NONE,
        normal_texture: NONE,
        emissive_texture: NONE,
        occlusion_texture: NONE,
        clearcoat_texture: NONE,
        clearcoat_roughness_texture: NONE,
        clearcoat_normal_texture: NONE,
        sheen_color_texture: NONE,
        sheen_roughness_texture: NONE,
        transmission_texture: NONE,
        thickness_texture: NONE,
        iridescence_texture: NONE,
        iridescence_thickness_texture: NONE,
        anisotropy_texture: NONE,
        name_offset,
        name_len,
        ..Default::default()
    });

    let mut prims: Vec<PrimitiveRow> = Vec::new();
    let mut lods: Vec<LodRow> = Vec::new();
    let mut meshes: Vec<MeshRow> = Vec::new();
    let mut vertices: Vec<u8> = Vec::new();
    let mut indices: Vec<u8> = Vec::new();
    for mesh in doc.meshes() {
        let first = prims.len() as u32;
        for (pi, prim) in mesh.primitives().enumerate() {
            let what = format!("mesh '{}' primitive {pi}", mesh.name().unwrap_or(""));
            if prim.mode() != gltf::mesh::Mode::Triangles {
                notes.push(format!(
                    "{what}: mode {:?} is not cooked (triangles only)",
                    prim.mode()
                ));
                continue;
            }
            let material = prim
                .material()
                .index()
                .map_or(default_material, |i| i as u32);
            let normal_mapped = !proxy && prim.material().normal_texture().is_some();
            match cook_primitive(src, &prim, material, normal_mapped, opts, proxy)
                .map_err(|e| CookError(format!("{what}: {e}")))?
            {
                Some(mut c) => {
                    while !vertices.len().is_multiple_of(4) {
                        vertices.push(0);
                    }
                    while !indices.len().is_multiple_of(4) {
                        indices.push(0);
                    }
                    c.row.vertex_offset = vertices.len() as u32;
                    c.row.vertex_size = c.vertices.len() as u32;
                    c.row.index_offset = indices.len() as u32;
                    c.row.index_size = c.indices.len() as u32;
                    c.row.lod_first = lods.len() as u32;
                    vertices.extend_from_slice(&c.vertices);
                    indices.extend_from_slice(&c.indices);
                    lods.extend_from_slice(&c.lods);
                    prims.push(c.row);
                }
                None => notes.push(format!("{what}: no geometry")),
            }
        }
        let (name_offset, name_len) = strings.add(mesh.name().unwrap_or(""));
        meshes.push(MeshRow {
            primitive_first: first,
            primitive_count: prims.len() as u32 - first,
            name_offset,
            name_len,
        });
    }

    let NodeOrder { new_index, rows } = node_order(doc, &mut |s| strings.add(s))?;
    let nodes = rows;
    let mut scenes = Vec::new();
    let mut roots: Vec<u32> = Vec::new();
    for sc in doc.scenes() {
        let first = roots.len() as u32;
        roots.extend(sc.nodes().map(|n| new_index[n.index()]));
        let (name_offset, name_len) = strings.add(sc.name().unwrap_or(""));
        scenes.push(SceneRow {
            root_first: first,
            root_count: roots.len() as u32 - first,
            name_offset,
            name_len,
        });
    }
    let default_scene = doc
        .default_scene()
        .map(|s| s.index() as u32)
        .unwrap_or(if scenes.is_empty() { NONE } else { 0 });

    // Bounds of the default scene's vertices with node transforms, from the
    // source positions (exactly what og-render's glTF bridge computes).
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    if let Some(scene) = doc.default_scene().or_else(|| doc.scenes().next()) {
        let mut stack: Vec<(gltf::Node, Mat4)> =
            scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
        while let Some((node, parent)) = stack.pop() {
            let (t, r, s) = node.transform().decomposed();
            let world = parent
                * Mat4::from_scale_rotation_translation(
                    Vec3::from(s),
                    Quat::from_array(r),
                    Vec3::from(t),
                );
            if let Some(mesh) = node.mesh() {
                for prim in mesh
                    .primitives()
                    .filter(|p| p.mode() == gltf::mesh::Mode::Triangles)
                {
                    let reader = prim.reader(|b| Some(&src.buffers[b.index()].0[..]));
                    if let Some(ps) = reader.read_positions() {
                        for p in ps {
                            let w = world.transform_point3(Vec3::from(p));
                            lo = lo.min(w);
                            hi = hi.max(w);
                        }
                    }
                }
            }
            for c in node.children() {
                stack.push((c, world));
            }
        }
    }
    if lo.x > hi.x {
        (lo, hi) = (Vec3::ZERO, Vec3::ZERO);
    }
    let skipped_animations = doc.animations().count() as u32;
    let skipped_skins = doc.skins().count() as u32;
    if skipped_animations + skipped_skins > 0 {
        notes.push(format!(
            "{skipped_animations} animation(s) and {skipped_skins} skin(s): in the #anim blob; this blob holds the bind pose"
        ));
    }
    let mut texture_refs = Vec::new();
    for (i, u) in src.textures.iter().enumerate().filter(|_| !proxy) {
        let (id_offset, id_len) = strings.add(&texture_ids[i]);
        texture_refs.push(TextureRefRow {
            id_offset,
            id_len,
            kind: u.kind,
        });
    }
    let meta = ModelMeta {
        primitive_count: prims.len() as u32,
        mesh_count: meshes.len() as u32,
        material_count: materials.len() as u32,
        texture_count: texture_refs.len() as u32,
        node_count: nodes.len() as u32,
        scene_count: scenes.len() as u32,
        default_scene,
        skipped_animations,
        skipped_skins,
        min_x: lo.x,
        min_y: lo.y,
        min_z: lo.z,
        max_x: hi.x,
        max_y: hi.y,
        max_z: hi.z,
    };
    let mut w = BlobWriter::new(
        format::Model::type_hash(),
        format::Model::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<ModelMeta>())
        .table("primitives", &prims, row_hash::<PrimitiveRow>())
        .table("lods", &lods, row_hash::<LodRow>())
        .table("meshes", &meshes, row_hash::<MeshRow>())
        .table("materials", &materials, row_hash::<MaterialRow>())
        .table("textures", &texture_refs, row_hash::<TextureRefRow>())
        .table("nodes", &nodes, row_hash::<NodeRow>())
        .table("scenes", &scenes, row_hash::<SceneRow>())
        .table("roots", &roots, 0)
        .raw("strings", strings.0, 1)
        .raw("vertices", vertices, 16)
        .raw("indices", indices, 16);
    let tris: usize = prims
        .iter()
        .map(|p| lods[p.lod_first as usize].index_count as usize / 3)
        .sum();
    let tris_lod: Vec<usize> = (0..opts.lod_levels.max(1) as usize)
        .map(|l| {
            prims
                .iter()
                .map(|p| {
                    let k = (l as u32).min(p.lod_count - 1);
                    lods[(p.lod_first + k) as usize].index_count as usize / 3
                })
                .sum()
        })
        .collect();
    notes.push(format!(
        "{} primitives, {tris} triangles; LOD triangles {:?}; {} materials, {} textures, {} nodes",
        prims.len(),
        tris_lod,
        materials.len(),
        texture_refs.len(),
        nodes.len()
    ));
    Ok((w.finish(), ModelNotes { notes }))
}
