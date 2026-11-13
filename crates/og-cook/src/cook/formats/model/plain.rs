//! A glTF made plain before the gltf crate reads it: the forms commercial
//! and optimised files ship in, rewritten in the JSON and its buffers as
//! the core glTF 2.0 the rest of the cook reads.
//!
//! - `EXT_meshopt_compression` / `KHR_meshopt_compression`: each
//!   compressed buffer view is decoded (meshoptimizer's own decoder and
//!   filters) into its place in the uncompressed buffer it describes (a
//!   `fallback` buffer is made of zeros first, never read).
//! - `KHR_draco_mesh_compression`: each primitive's Draco stream is decoded
//!   (draco-core, pure Rust) into new accessors of the types the glTF
//!   declares, appended in a buffer of their own; the primitive points at
//!   them and loses the extension.
//! - `KHR_mesh_quantization`: positions, normals, tangents and texture
//!   coordinates (and morph targets') stored as integers become floats
//!   (normalised ones divided as glTF says), so every later reader sees
//!   floats; a gltfpack file's dequantisation in its node transforms and
//!   `KHR_texture_transform` is the file's own and stays.
//! - `KHR_texture_basisu`, `EXT_texture_webp`, `MSFT_texture_dds`: a
//!   texture whose image is only in the extension takes it as its source
//!   (one with a `source` keeps it: the fallback is the original's
//!   quality); the texture cook decodes KTX2, WebP and DDS.
//! - `KHR_animation_pointer` onto `KHR_node_visibility` ([`super::pointer_visibility`]).
//!
//! Nothing is renumbered: accessors, views and buffers are only appended,
//! so every index the file's other parts (and the animation cook, which
//! reads the source's JSON again) hold stays right. Then
//! `extensionsRequired` loses every extension the cook reads ([`READ`]);
//! one it does not read refuses the file, by name.

use serde_json::{Map, Value, json};
use std::path::Path;

/// The extensions the cook reads: a file that requires one is taken.
pub const READ: &[&str] = &[
    "KHR_draco_mesh_compression",
    "EXT_meshopt_compression",
    "KHR_meshopt_compression",
    "KHR_mesh_quantization",
    "KHR_texture_basisu",
    "EXT_texture_webp",
    "MSFT_texture_dds",
    "KHR_texture_transform",
    "KHR_materials_unlit",
    "KHR_lights_punctual",
    "EXT_mesh_gpu_instancing",
    "KHR_materials_variants",
    "KHR_materials_emissive_strength",
    "KHR_materials_clearcoat",
    "KHR_materials_sheen",
    "KHR_materials_transmission",
    "KHR_materials_volume",
    "KHR_materials_ior",
    "KHR_materials_iridescence",
    "KHR_materials_anisotropy",
    "KHR_node_visibility",
    "KHR_animation_pointer",
    "MSFT_lod",
];

/// The plain glTF: its JSON, every buffer's bytes (in `buffers` order),
/// the external files it read and what the rewrite noted.
pub struct Plain {
    pub json: Value,
    pub buffers: Vec<Vec<u8>>,
    pub external: Vec<(String, Vec<u8>)>,
    pub warnings: Vec<String>,
}

/// A `.glb`'s JSON and BIN chunks, or a `.gltf`'s text.
fn chunks(bytes: &[u8]) -> Result<(&[u8], Option<Vec<u8>>), String> {
    if bytes.len() < 12 || &bytes[0..4] != b"glTF" {
        return Ok((bytes, None));
    }
    let word = |at: usize| -> Result<usize, String> {
        bytes
            .get(at..at + 4)
            .map(|w| u32::from_le_bytes(w.try_into().expect("4 bytes")) as usize)
            .ok_or_else(|| "glTF: the GLB is truncated".to_string())
    };
    let total = word(8)?.min(bytes.len());
    let (mut at, mut json, mut bin) = (12, None, None);
    while at + 8 <= total {
        let (len, kind) = (word(at)?, word(at + 4)?);
        let data = bytes
            .get(at + 8..at + 8 + len)
            .ok_or("glTF: a GLB chunk runs past the file")?;
        match kind {
            0x4E4F_534A if json.is_none() => json = Some(data),
            0x004E_4942 if bin.is_none() => bin = Some(data.to_vec()),
            _ => {}
        }
        at += 8 + len.next_multiple_of(4);
    }
    Ok((json.ok_or("glTF: the GLB has no JSON chunk")?, bin))
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn idx(v: &Value, key: &str) -> Option<usize> {
    v.get(key).and_then(Value::as_u64).map(|x| x as usize)
}

/// The meshopt extension of a buffer view or buffer, by either name.
fn meshopt_ext(v: &Value) -> Option<&Value> {
    let e = v.get("extensions")?;
    e.get("EXT_meshopt_compression")
        .or_else(|| e.get("KHR_meshopt_compression"))
}

fn remove_ext(v: &mut Value, names: &[&str]) {
    if let Some(e) = v.get_mut("extensions").and_then(Value::as_object_mut) {
        for n in names {
            e.remove(*n);
        }
        if e.is_empty()
            && let Some(o) = v.as_object_mut()
        {
            o.remove("extensions");
        }
    }
}

/// Reads `bytes` (a `.glb` or `.gltf`, relative files under `dir`) and
/// makes it plain.
pub fn plain(bytes: &[u8], dir: &Path) -> Result<Plain, String> {
    let (text, mut bin) = chunks(bytes)?;
    let mut json: Value = serde_json::from_slice(text).map_err(|e| format!("glTF: {e}"))?;
    let mut warnings = super::pointer_visibility(&mut json)?;
    let mut external = Vec::new();
    let mut buffers = Vec::new();
    for (i, b) in arr(&json, "buffers").iter().enumerate() {
        let len = idx(b, "byteLength").unwrap_or(0);
        let fallback = meshopt_ext(b)
            .and_then(|e| e.get("fallback"))
            .and_then(Value::as_bool)
            == Some(true);
        let data = match b.get("uri").and_then(Value::as_str) {
            // Only views with the extension point at it: their decoded bytes fill it.
            _ if fallback => vec![0; len],
            Some(uri) => super::read_uri(uri, dir, &mut external)?,
            None => bin.take().ok_or("glTF: the BIN chunk is missing")?,
        };
        if data.len() < len {
            return Err(format!("glTF: buffer {i} is shorter than declared"));
        }
        buffers.push(data);
    }
    let mut out = Out {
        json: &mut json,
        buffers: &mut buffers,
        extra: Vec::new(),
    };
    out.meshopt()?;
    out.draco()?;
    out.dequantize(&mut warnings)?;
    out.extra_buffer();
    texture_sources(&mut json);
    let required: Vec<String> = arr(&json, "extensionsRequired")
        .iter()
        .filter_map(|e| e.as_str())
        .filter(|e| !READ.contains(e))
        .map(str::to_string)
        .collect();
    if !required.is_empty() {
        return Err(format!(
            "glTF: the file requires {}, which the cook does not read (it reads {})",
            required.join(", "),
            READ.join(", ")
        ));
    }
    if let Some(o) = json.as_object_mut() {
        o.remove("extensionsRequired");
    }
    Ok(Plain {
        json,
        buffers,
        external,
        warnings,
    })
}

/// A texture's image from `KHR_texture_basisu`, `EXT_texture_webp` or
/// `MSFT_texture_dds` when it has no `source` of its own.
fn texture_sources(json: &mut Value) {
    const IMAGES: [&str; 3] = ["KHR_texture_basisu", "EXT_texture_webp", "MSFT_texture_dds"];
    let Some(textures) = json.get_mut("textures").and_then(Value::as_array_mut) else {
        return;
    };
    for t in textures {
        let from = IMAGES
            .iter()
            .find_map(|n| t.pointer(&format!("/extensions/{n}/source"))?.as_u64());
        if t.get("source").is_none()
            && let (Some(s), Some(o)) = (from, t.as_object_mut())
        {
            o.insert("source".into(), json!(s));
        }
        remove_ext(t, &IMAGES);
    }
}

/// Components per element of an accessor type.
fn components(ty: &str) -> Option<usize> {
    Some(match ty {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" | "MAT2" => 4,
        "MAT3" => 9,
        "MAT4" => 16,
        _ => return None,
    })
}

const FLOAT: u64 = 5126;
const UNSIGNED_INT: u64 = 5125;

/// Bytes per component of a glTF component type.
fn width(component: u64) -> Option<usize> {
    Some(match component {
        5120 | 5121 => 1,
        5122 | 5123 => 2,
        5125 | 5126 => 4,
        _ => return None,
    })
}

/// One component at `b` as f32: the integer's value, or with `normalized`
/// glTF's normalisation (`max(c / 127, -1)` and the like).
fn scalar(component: u64, normalized: bool, b: &[u8]) -> f32 {
    let (v, max) = match component {
        5120 => (b[0] as i8 as f32, 127.0),
        5121 => (b[0] as f32, 255.0),
        5122 => (i16::from_le_bytes([b[0], b[1]]) as f32, 32767.0),
        5123 => (u16::from_le_bytes([b[0], b[1]]) as f32, 65535.0),
        5125 => (u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32, 1.0),
        _ => return f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
    };
    if normalized && component != 5125 {
        (v / max).max(-1.0)
    } else {
        v
    }
}

/// An accessor's elements as f32 (`count * components` values; sparse
/// substitutions applied, normalised integers divided): what the cook
/// reads instancing attributes and quantised vertex data with.
pub fn read_accessor<B: std::ops::Deref<Target = [u8]>>(
    json: &Value,
    buffers: &[B],
    accessor: usize,
) -> Result<(Vec<f32>, usize), String> {
    let a = arr(json, "accessors")
        .get(accessor)
        .ok_or_else(|| format!("glTF: accessor {accessor} does not exist"))?;
    let bad = |what: &str| format!("glTF: accessor {accessor}: {what}");
    let count = idx(a, "count").ok_or_else(|| bad("no count"))?;
    let n = components(a.get("type").and_then(Value::as_str).unwrap_or(""))
        .ok_or_else(|| bad("an unknown type"))?;
    let component = a.get("componentType").and_then(Value::as_u64).unwrap_or(0);
    let w = width(component).ok_or_else(|| bad("an unknown component type"))?;
    let normalized = a.get("normalized").and_then(Value::as_bool) == Some(true);
    let mut out = vec![0.0f32; count * n];
    let view_bytes = |view: usize| -> Result<(&[u8], usize), String> {
        let v = arr(json, "bufferViews")
            .get(view)
            .ok_or_else(|| bad("its buffer view does not exist"))?;
        let b = buffers
            .get(idx(v, "buffer").unwrap_or(usize::MAX))
            .ok_or_else(|| bad("its buffer does not exist"))?;
        let off = idx(v, "byteOffset").unwrap_or(0);
        let len = idx(v, "byteLength").unwrap_or(0);
        let data = b
            .get(off..off + len)
            .ok_or_else(|| bad("its buffer view runs past its buffer"))?;
        Ok((data, idx(v, "byteStride").unwrap_or(0)))
    };
    if let Some(view) = idx(a, "bufferView") {
        let (data, stride) = view_bytes(view)?;
        let stride = if stride == 0 { n * w } else { stride };
        let base = idx(a, "byteOffset").unwrap_or(0);
        for e in 0..count {
            for c in 0..n {
                let at = base + e * stride + c * w;
                let b = data
                    .get(at..at + w)
                    .ok_or_else(|| bad("its elements run past its buffer view"))?;
                out[e * n + c] = scalar(component, normalized, b);
            }
        }
    }
    if let Some(s) = a.get("sparse") {
        let k = idx(s, "count").unwrap_or(0);
        let (ind, val) = (&s["indices"], &s["values"]);
        let (ib, _) = view_bytes(idx(ind, "bufferView").ok_or_else(|| bad("sparse indices"))?)?;
        let (vb, _) = view_bytes(idx(val, "bufferView").ok_or_else(|| bad("sparse values"))?)?;
        let ic = ind
            .get("componentType")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let iw = width(ic).ok_or_else(|| bad("sparse index type"))?;
        let (io, vo) = (
            idx(ind, "byteOffset").unwrap_or(0),
            idx(val, "byteOffset").unwrap_or(0),
        );
        for i in 0..k {
            let b = ib
                .get(io + i * iw..io + (i + 1) * iw)
                .ok_or_else(|| bad("sparse indices run past their view"))?;
            let e = scalar(ic, false, b) as usize;
            if e >= count {
                return Err(bad("a sparse index past its count"));
            }
            for c in 0..n {
                let at = vo + (i * n + c) * w;
                let b = vb
                    .get(at..at + w)
                    .ok_or_else(|| bad("sparse values run past their view"))?;
                out[e * n + c] = scalar(component, normalized, b);
            }
        }
    }
    Ok((out, n))
}

/// Values as little-endian components of `component` (integers rounded and
/// clamped to the type's range).
fn encode(values: &[f32], component: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * width(component).unwrap_or(4));
    for &v in values {
        match component {
            5120 => out.push(v.round().clamp(-128.0, 127.0) as i8 as u8),
            5121 => out.push(v.round().clamp(0.0, 255.0) as u8),
            5122 => out.extend((v.round().clamp(-32768.0, 32767.0) as i16).to_le_bytes()),
            5123 => out.extend((v.round().clamp(0.0, 65535.0) as u16).to_le_bytes()),
            5125 => out.extend((v.round().max(0.0) as u32).to_le_bytes()),
            _ => out.extend(v.to_le_bytes()),
        }
    }
    out
}

/// The per-component bounds of `values` (`n` a element).
fn bounds(values: &[f32], n: usize) -> (Vec<f32>, Vec<f32>) {
    let mut lo = vec![f32::MAX; n];
    let mut hi = vec![f32::MIN; n];
    for e in values.chunks_exact(n) {
        for c in 0..n {
            lo[c] = lo[c].min(e[c]);
            hi[c] = hi[c].max(e[c]);
        }
    }
    if values.is_empty() {
        (vec![0.0; n], vec![0.0; n])
    } else {
        (lo, hi)
    }
}

struct Out<'a> {
    json: &'a mut Value,
    buffers: &'a mut Vec<Vec<u8>>,
    /// The bytes of the buffer appended for decoded accessors.
    extra: Vec<u8>,
}

impl Out<'_> {
    fn list(&mut self, key: &str) -> &mut Vec<Value> {
        let o = self
            .json
            .as_object_mut()
            .expect("the glTF root is an object");
        o.entry(key)
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("a glTF list")
    }

    /// A new buffer view over `bytes`, in the appended buffer.
    fn view(&mut self, bytes: &[u8]) -> usize {
        while !self.extra.len().is_multiple_of(4) {
            self.extra.push(0);
        }
        let buffer = self.buffers.len();
        let v =
            json!({ "buffer": buffer, "byteOffset": self.extra.len(), "byteLength": bytes.len() });
        self.extra.extend_from_slice(bytes);
        let views = self.list("bufferViews");
        views.push(v);
        views.len() - 1
    }

    /// An accessor like `like` (its type and normalisation) holding
    /// `values` as `component`, with bounds when it is a position: `into`
    /// (a Draco primitive's data-less accessor, filled in its place, so no
    /// accessor is left without data) or a new one.
    fn accessor(
        &mut self,
        like: &Value,
        values: &[f32],
        n: usize,
        component: u64,
        position: bool,
        into: Option<usize>,
    ) -> usize {
        let view = self.view(&encode(values, component));
        let mut a = Map::new();
        a.insert("bufferView".into(), json!(view));
        a.insert("componentType".into(), json!(component));
        a.insert("count".into(), json!(values.len() / n));
        a.insert(
            "type".into(),
            like.get("type").cloned().unwrap_or(json!("SCALAR")),
        );
        if component != FLOAT
            && let Some(norm) = like.get("normalized")
        {
            a.insert("normalized".into(), norm.clone());
        }
        if position {
            let (lo, hi) = bounds(values, n);
            a.insert("min".into(), json!(lo));
            a.insert("max".into(), json!(hi));
        }
        let accessors = self.list("accessors");
        match into {
            Some(i) => {
                accessors[i] = Value::Object(a);
                i
            }
            None => {
                accessors.push(Value::Object(a));
                accessors.len() - 1
            }
        }
    }

    /// The appended buffer, when anything was decoded into it.
    fn extra_buffer(&mut self) {
        if self.extra.is_empty() {
            return;
        }
        let bytes = std::mem::take(&mut self.extra);
        self.list("buffers")
            .push(json!({ "byteLength": bytes.len() }));
        self.buffers.push(bytes);
    }

    /// `EXT_meshopt_compression`: every compressed view decoded into its
    /// place in its (uncompressed) buffer.
    fn meshopt(&mut self) -> Result<(), String> {
        let mut decoded = Vec::new();
        for (i, v) in arr(self.json, "bufferViews").iter().enumerate() {
            let Some(e) = meshopt_ext(v) else {
                continue;
            };
            let bad = |what: &str| format!("glTF: buffer view {i}: meshopt: {what}");
            let count = idx(e, "count").ok_or_else(|| bad("no count"))?;
            let stride = idx(e, "byteStride").ok_or_else(|| bad("no byteStride"))?;
            let off = idx(e, "byteOffset").unwrap_or(0);
            let len = idx(e, "byteLength").ok_or_else(|| bad("no byteLength"))?;
            let src = self
                .buffers
                .get(idx(e, "buffer").unwrap_or(usize::MAX))
                .and_then(|b| b.get(off..off + len))
                .ok_or_else(|| bad("its compressed bytes are out of range"))?;
            let mode = e.get("mode").and_then(Value::as_str).unwrap_or("");
            let filter = e.get("filter").and_then(Value::as_str).unwrap_or("NONE");
            // The strides meshoptimizer's decoders and filters take.
            let fits = match (mode, filter) {
                ("ATTRIBUTES", "NONE") => stride > 0 && stride <= 256 && stride % 4 == 0,
                ("ATTRIBUTES", "OCTAHEDRAL" | "COLOR") => stride == 4 || stride == 8,
                ("ATTRIBUTES", "QUATERNION") => stride == 8,
                ("ATTRIBUTES", "EXPONENTIAL") => stride > 0 && stride <= 256 && stride % 4 == 0,
                ("TRIANGLES", "NONE") => (stride == 2 || stride == 4) && count % 3 == 0,
                ("INDICES", "NONE") => stride == 2 || stride == 4,
                _ => false,
            };
            if !fits {
                return Err(bad(&format!(
                    "mode {mode} with filter {filter} and byteStride {stride} is not one meshopt decodes"
                )));
            }
            let mut out = vec![0u8; count * stride];
            // SAFETY: `out` holds `count * stride` bytes, as each decoder
            // and filter writes; `src` is the compressed range they read;
            // `fits` holds every stride to what its decoder and filter take.
            let rc = unsafe {
                use meshopt::ffi::*;
                let (o, s) = (out.as_mut_ptr().cast(), src.as_ptr());
                match mode {
                    "ATTRIBUTES" => {
                        let rc = meshopt_decodeVertexBuffer(o, count, stride, s, len);
                        if rc == 0 {
                            match filter {
                                "NONE" => {}
                                "OCTAHEDRAL" => meshopt_decodeFilterOct(o, count, stride),
                                "QUATERNION" => meshopt_decodeFilterQuat(o, count, stride),
                                "EXPONENTIAL" => meshopt_decodeFilterExp(o, count, stride),
                                _ => meshopt_decodeFilterColor(o, count, stride),
                            }
                        }
                        rc
                    }
                    "TRIANGLES" => meshopt_decodeIndexBuffer(o, count, stride, s, len),
                    _ => meshopt_decodeIndexSequence(o, count, stride, s, len),
                }
            };
            if rc != 0 {
                return Err(bad(&format!("the stream does not decode ({rc})")));
            }
            let target = idx(v, "buffer").ok_or_else(|| bad("no buffer"))?;
            let at = idx(v, "byteOffset").unwrap_or(0);
            if idx(v, "byteLength").unwrap_or(0) < out.len() {
                return Err(bad("the decoded data is longer than the view"));
            }
            decoded.push((i, target, at, out));
        }
        for (i, target, at, out) in decoded {
            self.buffers
                .get_mut(target)
                .and_then(|b| b.get_mut(at..at + out.len()))
                .ok_or_else(|| format!("glTF: buffer view {i}: past its buffer"))?
                .copy_from_slice(&out);
            remove_ext(
                &mut self.list("bufferViews")[i],
                &["EXT_meshopt_compression", "KHR_meshopt_compression"],
            );
        }
        if let Some(bs) = self.json.get_mut("buffers").and_then(Value::as_array_mut) {
            for b in bs {
                remove_ext(b, &["EXT_meshopt_compression", "KHR_meshopt_compression"]);
            }
        }
        Ok(())
    }

    /// `KHR_draco_mesh_compression`: each primitive's stream decoded into
    /// new accessors.
    fn draco(&mut self) -> Result<(), String> {
        const EXT: &str = "KHR_draco_mesh_compression";
        // The data-less accessors already filled by a decoded primitive: a
        // second primitive naming one gets an accessor of its own.
        let mut filled = std::collections::BTreeSet::new();
        let mut work = Vec::new();
        for (m, mesh) in arr(self.json, "meshes").iter().enumerate() {
            for (p, prim) in arr(mesh, "primitives").iter().enumerate() {
                if let Some(e) = prim.pointer(&format!("/extensions/{EXT}")) {
                    work.push((m, p, e.clone(), prim.clone()));
                }
            }
        }
        for (m, p, e, prim) in work {
            let at = format!("glTF: mesh {m} primitive {p}: Draco");
            let view = idx(&e, "bufferView").ok_or_else(|| format!("{at}: no bufferView"))?;
            let bytes = {
                let v = arr(self.json, "bufferViews")
                    .get(view)
                    .ok_or_else(|| format!("{at}: buffer view {view} does not exist"))?;
                let off = idx(v, "byteOffset").unwrap_or(0);
                let len = idx(v, "byteLength").unwrap_or(0);
                self.buffers
                    .get(idx(v, "buffer").unwrap_or(usize::MAX))
                    .and_then(|b| b.get(off..off + len))
                    .ok_or_else(|| format!("{at}: its bytes are out of range"))?
                    .to_vec()
            };
            let mut mesh = draco_core::Mesh::new();
            draco_core::MeshDecoder::new()
                .decode(&mut draco_core::DecoderBuffer::new(&bytes), &mut mesh)
                .map_err(|err| format!("{at}: {err}"))?;
            let points = mesh.num_points();
            let mut attributes = prim.get("attributes").cloned().unwrap_or(json!({}));
            let ids = e
                .get("attributes")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            for (name, id) in &ids {
                let Some(old) = attributes.get(name).and_then(Value::as_u64) else {
                    continue;
                };
                let like = arr(self.json, "accessors")
                    .get(old as usize)
                    .cloned()
                    .ok_or_else(|| format!("{at}: {name}'s accessor does not exist"))?;
                let n = components(like.get("type").and_then(Value::as_str).unwrap_or(""))
                    .ok_or_else(|| format!("{at}: {name}'s accessor has an unknown type"))?;
                let id = id.as_u64().unwrap_or(u64::MAX) as u32;
                let attr = (0..mesh.num_attributes())
                    .map(|i| mesh.attribute(i))
                    .find(|a| a.unique_id() == id)
                    .ok_or_else(|| format!("{at}: no Draco attribute {id} for {name}"))?;
                let mut values = attr.read_f32s(points, n);
                let component = like
                    .get("componentType")
                    .and_then(Value::as_u64)
                    .unwrap_or(FLOAT);
                // A normalised Draco integer read into a float accessor.
                if component == FLOAT && attr.normalized() {
                    let max = match attr.data_type() {
                        draco_core::DataType::Int8 => 127.0,
                        draco_core::DataType::Uint8 => 255.0,
                        draco_core::DataType::Int16 => 32767.0,
                        draco_core::DataType::Uint16 => 65535.0,
                        _ => 1.0,
                    };
                    values.iter_mut().for_each(|v| *v = (*v / max).max(-1.0));
                }
                let into = (like.get("bufferView").is_none() && filled.insert(old))
                    .then_some(old as usize);
                let new = self.accessor(&like, &values, n, component, name == "POSITION", into);
                attributes[name] = json!(new);
            }
            let faces: Vec<f32> = mesh.faces().iter().flatten().map(|i| i.0 as f32).collect();
            // Indices as u32 (exact through f32 below 2^24 points, which
            // Draco's u32 point ids of a cookable mesh stay under).
            if points >= 1 << 24 {
                return Err(format!(
                    "{at}: {points} points; the cook takes fewer than 2^24"
                ));
            }
            let into = idx(&prim, "indices").filter(|&i| {
                arr(self.json, "accessors")
                    .get(i)
                    .is_some_and(|a| a.get("bufferView").is_none())
                    && filled.insert(i as u64)
            });
            let indices = self.accessor(
                &json!({ "type": "SCALAR" }),
                &faces,
                1,
                UNSIGNED_INT,
                false,
                into,
            );
            let prim = &mut self.list("meshes")[m]["primitives"][p];
            prim["attributes"] = attributes;
            prim["indices"] = json!(indices);
            remove_ext(prim, &[EXT]);
        }
        Ok(())
    }

    /// `KHR_mesh_quantization`: integer positions, normals, tangents and
    /// texture coordinates (of primitives and morph targets) as floats.
    fn dequantize(&mut self, notes: &mut Vec<String>) -> Result<(), String> {
        let quantized = |name: &str| {
            matches!(name, "POSITION" | "NORMAL" | "TANGENT") || name.starts_with("TEXCOORD_")
        };
        let mut done: std::collections::BTreeMap<usize, usize> = Default::default();
        let mut sites = Vec::new();
        for (m, mesh) in arr(self.json, "meshes").iter().enumerate() {
            for (p, prim) in arr(mesh, "primitives").iter().enumerate() {
                let mut maps = vec![(None, prim.get("attributes"))];
                maps.extend(
                    arr(prim, "targets")
                        .iter()
                        .enumerate()
                        .map(|(t, v)| (Some(t), Some(v))),
                );
                for (target, map) in maps {
                    for (name, a) in map.and_then(Value::as_object).into_iter().flatten() {
                        let Some(a) = a.as_u64().map(|a| a as usize) else {
                            continue;
                        };
                        let component = arr(self.json, "accessors")
                            .get(a)
                            .and_then(|x| x.get("componentType"))
                            .and_then(Value::as_u64);
                        if quantized(name) && component.is_some_and(|c| c != FLOAT) {
                            sites.push((m, p, target, name.clone(), a));
                        }
                    }
                }
            }
        }
        for (m, p, target, name, a) in &sites {
            let new = match done.get(a) {
                Some(&n) => n,
                None => {
                    let (values, n) = read_accessor(self.json, &self.buffers[..], *a)?;
                    let like = self.json["accessors"][*a].clone();
                    let new = self.accessor(&like, &values, n, FLOAT, name == "POSITION", None);
                    done.insert(*a, new);
                    new
                }
            };
            let prim = &mut self.list("meshes")[*m]["primitives"][*p];
            match target {
                None => prim["attributes"][name] = json!(new),
                Some(t) => prim["targets"][*t][name] = json!(new),
            }
        }
        if !done.is_empty() {
            notes.push(format!(
                "KHR_mesh_quantization: {} quantised accessor(s) read as floats",
                done.len()
            ));
        }
        Ok(())
    }
}
