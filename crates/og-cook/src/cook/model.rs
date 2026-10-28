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
/// The JSON of a glTF (a `.glb`'s JSON chunk, or the file) as the cook
/// reads it: with [`pointer_visibility`] applied. `None` when it does not
/// parse.
pub(super) fn source_json(bytes: &[u8]) -> Option<serde_json::Value> {
    let mut json: serde_json::Value = if bytes.len() >= 20 && &bytes[0..4] == b"glTF" {
        let len = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as usize;
        serde_json::from_slice(bytes.get(20..20 + len)?).ok()?
    } else {
        serde_json::from_slice(bytes).ok()?
    };
    pointer_visibility(&mut json).ok()?;
    Some(json)
}

/// The pointer `KHR_animation_pointer` animates `KHR_node_visibility` with.
fn visibility_node(pointer: &str) -> Option<u64> {
    let n = pointer
        .strip_prefix("/nodes/")?
        .strip_suffix("/extensions/KHR_node_visibility/visible")?;
    (!n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .then(|| n.parse().ok())
        .flatten()
}

/// Rewrites glTF's own visibility animation, a `KHR_animation_pointer`
/// channel (`"path": "pointer"`) onto
/// `/nodes/<n>/extensions/KHR_node_visibility/visible`, as the engine's
/// `OG_animation_visibility` entry `{ sampler, node: n }` of the same
/// animation, the form the gltf crate (which refuses the `pointer` path)
/// loads and validates as any other file. A pointer channel onto anything
/// else is dropped with a warning, or refused when the file requires
/// `KHR_animation_pointer`; once none is left the extension leaves
/// `extensionsRequired`, as does `KHR_node_visibility`, which the cook
/// reads. Untouched (`Ok` and no warnings, nothing changed) without a
/// pointer channel.
pub(super) fn pointer_visibility(json: &mut serde_json::Value) -> Result<Vec<String>, String> {
    use serde_json::{Value, json};
    const POINTER: &str = "KHR_animation_pointer";
    let required = |json: &Value, ext: &str| {
        json.get("extensionsRequired")
            .and_then(|r| r.as_array())
            .is_some_and(|r| r.iter().any(|e| e == ext))
    };
    let pointer_required = required(json, POINTER);
    let mut warnings = Vec::new();
    let mut changed = false;
    let Some(anims) = json.get_mut("animations").and_then(|a| a.as_array_mut()) else {
        return Ok(warnings);
    };
    for (ai, anim) in anims.iter_mut().enumerate() {
        let Some(channels) = anim.get_mut("channels").and_then(|c| c.as_array_mut()) else {
            continue;
        };
        let mut moved = Vec::new();
        let mut kept = Vec::with_capacity(channels.len());
        for (ci, ch) in std::mem::take(channels).into_iter().enumerate() {
            if ch.pointer("/target/path").and_then(|p| p.as_str()) != Some("pointer") {
                kept.push(ch);
                continue;
            }
            changed = true;
            let at = format!("animation {ai} channel {ci}");
            let pointer = ch
                .pointer(&format!("/target/extensions/{POINTER}/pointer"))
                .and_then(|p| p.as_str())
                .ok_or_else(|| {
                    format!("glTF: {at}: a `pointer` path without its {POINTER} pointer")
                })?;
            let sampler = ch
                .get("sampler")
                .and_then(|s| s.as_u64())
                .ok_or_else(|| format!("glTF: {at}: no sampler"))?;
            match visibility_node(pointer) {
                Some(node) => moved.push(json!({ "sampler": sampler, "node": node })),
                None if pointer_required => {
                    return Err(format!(
                        "glTF: {at} animates {pointer} through the required {POINTER}; the engine animates only KHR_node_visibility's visible that way"
                    ));
                }
                None => warnings.push(format!(
                    "glTF: {at}: {POINTER} onto {pointer} is not animated (only KHR_node_visibility's visible is); dropped"
                )),
            }
        }
        *channels = kept;
        if !moved.is_empty() {
            let ext = anim
                .as_object_mut()
                .ok_or("glTF: an animation is not an object")?
                .entry("extensions")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or("glTF: an animation's extensions are not an object")?
                .entry(crate::cook::anim::VISIBILITY_EXTENSION)
                .or_insert_with(|| json!({ "channels": [] }));
            ext.get_mut("channels")
                .and_then(|c| c.as_array_mut())
                .ok_or_else(|| {
                    format!(
                        "glTF: animation {ai}: {}'s channels are not a list",
                        crate::cook::anim::VISIBILITY_EXTENSION
                    )
                })?
                .extend(moved);
        }
    }
    if changed
        && let Some(r) = json
            .get_mut("extensionsRequired")
            .and_then(|r| r.as_array_mut())
    {
        r.retain(|e| e != POINTER && e != "KHR_node_visibility");
    }
    Ok(warnings)
}

/// `bytes` with [`pointer_visibility`] applied (a `.glb` rebuilt around its
/// rewritten JSON chunk), borrowed as they are when it changes nothing.
fn rewrite_pointers(bytes: &[u8]) -> Result<(std::borrow::Cow<'_, [u8]>, Vec<String>), String> {
    use std::borrow::Cow;
    let glb = bytes.len() >= 20 && &bytes[0..4] == b"glTF";
    let (json_bytes, rest) = if glb {
        let len = u32::from_le_bytes(bytes[12..16].try_into().expect("4 bytes")) as usize;
        match bytes.get(20..20 + len) {
            Some(j) => (j, &bytes[20 + len..]),
            // Malformed: the gltf crate says how.
            None => return Ok((Cow::Borrowed(bytes), Vec::new())),
        }
    } else {
        (bytes, &[][..])
    };
    // Only a file that names the `pointer` path is parsed here.
    if !json_bytes.windows(9).any(|w| w == b"\"pointer\"") {
        return Ok((Cow::Borrowed(bytes), Vec::new()));
    }
    let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(json_bytes) else {
        return Ok((Cow::Borrowed(bytes), Vec::new()));
    };
    let warnings = pointer_visibility(&mut json)?;
    let mut text = serde_json::to_vec(&json).map_err(|e| format!("glTF: {e}"))?;
    if !glb {
        return Ok((Cow::Owned(text), warnings));
    }
    while text.len() % 4 != 0 {
        text.push(b' ');
    }
    let mut out = Vec::with_capacity(20 + text.len() + rest.len());
    out.extend_from_slice(&bytes[0..8]);
    out.extend_from_slice(&((20 + text.len() + rest.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(text.len() as u32).to_le_bytes());
    out.extend_from_slice(&bytes[16..20]);
    out.extend_from_slice(&text);
    out.extend_from_slice(rest);
    Ok((Cow::Owned(out), warnings))
}

pub fn load(bytes: &[u8], dir: &Path) -> Result<GltfSource, String> {
    let (bytes, pointer_warnings) = rewrite_pointers(bytes)?;
    let gltf = gltf::Gltf::from_slice(&bytes).map_err(|e| format!("glTF: {e}"))?;
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
    let mut warnings = pointer_warnings;
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
pub(super) struct FVertex {
    pub(super) p: [f32; 3],
    pub(super) n: [f32; 3],
    pub(super) uv: [f32; 2],
    pub(super) c: [f32; 4],
    pub(super) t: [f32; 4],
    /// A hash of the source vertex's skin and morph data (zero without
    /// either), so welding never merges vertices that skin or morph apart.
    pub(super) key: [u32; 2],
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
    /// The raycast BVH over LOD 0 (empty with `raycast false`): its nodes,
    /// `first` of a leaf relative to `bvh_tris`.
    bvh_nodes: Vec<BvhNodeRow>,
    bvh_tris: Vec<u32>,
    /// The source (glTF accessor) vertex of every cooked vertex.
    pub sources: Vec<u32>,
    /// What the art contract changed (`conform::mesh`), and with an
    /// `ArtLine` the outline hull's normal of every cooked vertex.
    conformed: Option<super::conform::mesh::Changes>,
    hull: Vec<[i16; 2]>,
}

/// zlib at the best level (the BVH sections: about 2.5 times smaller).
fn zlib(bytes: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(bytes, 9)
}

/// The bytes the BVH sections take in the blob, compressed.
fn w_len_of_bvh(nodes: &[BvhNodeRow], tris: &[u32]) -> usize {
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
pub(super) fn level_error(pos: &[[f32; 3]], full: &[u32], level: &[u32]) -> f32 {
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

pub(super) fn cook_primitive(
    src: &GltfSource,
    levels: &[gltf::Primitive],
    material: u32,
    normal_mapped: bool,
    opts: &MeshOptions,
    coarsest: bool,
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
            uv: uvs.as_ref().map_or([0.0; 2], |u| u[i]),
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
        vertices,
        indices: index_bytes,
        bvh_nodes,
        bvh_tris,
        sources: ordered_sources,
        conformed,
        hull,
    }))
}

/// `MSFT_lod`: per mesh, the meshes of its coarser levels in order, from
/// every node whose extension names other nodes (`ids`) with meshes. The
/// nodes it names are levels, not parts of the scene: a scene that also
/// draws them draws them twice, so the cook says so.
pub(super) fn msft_lod(
    doc: &gltf::Document,
    notes: &mut Vec<String>,
) -> std::collections::BTreeMap<usize, Vec<usize>> {
    let mut out = std::collections::BTreeMap::new();
    let nodes: Vec<gltf::Node> = doc.nodes().collect();
    for n in &nodes {
        let (Some(mesh), Some(ext)) = (n.mesh(), n.extension_value("MSFT_lod")) else {
            continue;
        };
        let ids: Vec<usize> = ext
            .get("ids")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|i| i.as_u64())
                    .map(|i| i as usize)
                    .collect()
            })
            .unwrap_or_default();
        let meshes: Vec<usize> = ids
            .iter()
            .filter_map(|&i| nodes.get(i)?.mesh().map(|m| m.index()))
            .collect();
        if meshes.len() != ids.len() {
            notes.push(format!(
                "node {}: MSFT_lod names {} node(s), {} with a mesh; the others are skipped",
                n.index(),
                ids.len(),
                meshes.len()
            ));
        }
        if !meshes.is_empty() {
            out.insert(mesh.index(), meshes);
        }
    }
    out
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
            visible: u32::from(
                n.extension_value("KHR_node_visibility")
                    .and_then(|v| v.get("visible")?.as_bool())
                    != Some(false),
            ),
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
    /// LOD 0 triangles and vertices of every primitive, summed.
    pub triangles: usize,
    pub vertices: usize,
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
    // An untextured style's colours are baked into the vertices
    // (`conform::mesh`): its materials draw white without a base texture.
    let baked = opts
        .conform
        .as_ref()
        .is_some_and(|c| c.bakes() && c.applies(src));
    let mut materials = Vec::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let e = m.emissive_factor();
        let strength = m.emissive_strength().unwrap_or(1.0);
        let (name_offset, name_len) = strings.add(m.name().unwrap_or(""));
        let mut b = pbr.base_color_factor();
        if baked {
            b = [1.0, 1.0, 1.0, b[3]];
        }
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
            base_color_texture: tex_of(
                pbr.base_color_texture()
                    .map(|t| t.texture())
                    .filter(|_| !baked),
                kind::COLOR,
            ),
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
    let grey = if baked { 1.0 } else { 0.8 };
    materials.push(MaterialRow {
        base_r: grey,
        base_g: grey,
        base_b: grey,
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
    let mut bvh_nodes: Vec<BvhNodeRow> = Vec::new();
    let mut bvh_tris: Vec<u32> = Vec::new();
    let lod_meshes = msft_lod(doc, &mut notes);
    let mut changes = Vec::new();
    let mut hulls: Vec<u8> = Vec::new();
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
            // MSFT_lod: the same primitive of each coarser mesh is a level.
            let mut levels = vec![prim.clone()];
            for &m in lod_meshes.get(&mesh.index()).into_iter().flatten() {
                match doc.meshes().nth(m).and_then(|lm| lm.primitives().nth(pi)) {
                    Some(lp) if lp.mode() == gltf::mesh::Mode::Triangles => levels.push(lp),
                    _ => {
                        notes.push(format!(
                            "{what}: MSFT_lod mesh {m} has no triangle primitive {pi}; its levels stop there"
                        ));
                        break;
                    }
                }
            }
            match cook_primitive(src, &levels, material, normal_mapped, opts, proxy)
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
                    c.row.bvh_node_first = bvh_nodes.len() as u32;
                    c.row.bvh_node_count = c.bvh_nodes.len() as u32;
                    c.row.bvh_tri_first = bvh_tris.len() as u32;
                    c.row.bvh_tri_count = c.bvh_tris.len() as u32;
                    bvh_nodes.extend_from_slice(&c.bvh_nodes);
                    bvh_tris.extend_from_slice(&c.bvh_tris);
                    vertices.extend_from_slice(&c.vertices);
                    indices.extend_from_slice(&c.indices);
                    lods.extend_from_slice(&c.lods);
                    changes.extend(c.conformed.take());
                    hulls.extend((c.hull.len() as u32).to_le_bytes());
                    hulls.extend(c.hull.iter().flatten().flat_map(|x| x.to_le_bytes()));
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
    // source positions: what og-render's glTF bridge computes, less the
    // nodes hidden at rest (`KHR_node_visibility` `visible: false`) and
    // what is under them, which the rest pose does not draw (an animated
    // `.vox`'s frames after 0).
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    if let Some(scene) = doc.default_scene().or_else(|| doc.scenes().next()) {
        let mut stack: Vec<(gltf::Node, Mat4)> =
            scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
        while let Some((node, parent)) = stack.pop() {
            if node
                .extension_value("KHR_node_visibility")
                .and_then(|v| v.get("visible")?.as_bool())
                == Some(false)
            {
                continue;
            }
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
        .raw("indices", indices, 16)
        .raw("bvh_nodes", zlib(bytemuck::cast_slice(&bvh_nodes)), 4)
        .raw("bvh_tris", zlib(bytemuck::cast_slice(&bvh_tris)), 4);
    // The art contract's outline hull (`conform::mesh` gives the layout).
    if let Some(c) = &opts.conform
        && let Some(l) = &c.line
        && !changes.is_empty()
    {
        let mut section = super::conform::mesh::hull_header(l, prims.len());
        section.extend(hulls);
        w.raw("outline", section, 4);
    }
    if let Some(c) = &opts.conform {
        notes.extend(super::conform::mesh::notes(c, src, &changes));
    }
    let bvh_bytes = w_len_of_bvh(&bvh_nodes, &bvh_tris);
    let tris: usize = prims
        .iter()
        .map(|p| lods[p.lod_first as usize].index_count as usize / 3)
        .sum();
    let tris_lod: Vec<usize> = (0..prims.iter().map(|p| p.lod_count).max().unwrap_or(1) as usize)
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
    notes.push(if opts.raycast {
        format!(
            "raycast BVH: {} nodes, {bvh_bytes} bytes (`model {{ raycast false }}` leaves it out)",
            bvh_nodes.len()
        )
    } else {
        "raycast false: no BVH cooked; the first og.render.raycast that reaches it builds one"
            .into()
    });
    let vertices = prims.iter().map(|p| p.vertex_count as usize).sum();
    Ok((
        w.finish(),
        ModelNotes {
            notes,
            triangles: tris,
            vertices,
        },
    ))
}
