//! Uncooked glTF at run time (`og.load.gltfData`): a glTF 2.0 file a player
//! or a generator made, read from bytes into a [`CookedModel`] without
//! `og cook`, so it joins the renderer through the same path a cooked model
//! does.
//!
//! [`UncookedGltf::read`] does everything that needs no decoding: the GLB
//! container or the `.gltf` JSON, the buffers, the triangle count, the
//! images' bytes, the materials, nodes and scenes. Each primitive is then
//! converted on its own ([`UncookedGltf::primitive`]), so the runtime can
//! spread the work over frames, and [`UncookedGltf::finish`] assembles the
//! model. Images are only located here: the runtime decodes them off the
//! engine's thread and hands them to [`CookedModel::add_to_with`] in
//! [`UncookedGltf::textures`] order.
//!
//! What the cook would turn into something else is refused by name rather
//! than drawn wrong: Draco and meshopt compression, Basis, WebP and AVIF
//! textures, sparse accessors, other files named by URI, and any other
//! required extension. Skins and animations are skipped and counted
//! ([`CookedModel::skipped_skins`], [`CookedModel::skipped_animations`]);
//! points and lines are skipped; no levels of detail or raycast BVH are
//! built (a raycast builds its BVH when first needed).

use super::*;
use serde_json::Value;

/// A glTF file read for loading, everything but its primitives and images
/// converted.
pub struct UncookedGltf {
    id: String,
    json: Value,
    buffers: Vec<Vec<u8>>,
    /// The triangle primitives to convert, as (mesh, primitive) in glTF
    /// order: [`CookedModel::primitives`]' order.
    prims: Vec<(usize, usize)>,
    /// Every primitive's triangles, as [`UncookedGltf::read`] counted them.
    pub triangles: u64,
    /// The images the materials sample, each once: a name for errors and
    /// its PNG or JPEG bytes.
    pub images: Vec<(String, Vec<u8>)>,
    /// The model's textures, in [`CookedModel::textures`] order, and the
    /// image (in [`UncookedGltf::images`]) each is made from.
    pub textures: Vec<(TextureRef, usize)>,
    materials: Vec<CookedMaterial>,
    meshes: Vec<CookedMesh>,
    nodes: Vec<CookedNode>,
    scenes: Vec<CookedScene>,
    default_scene: Option<usize>,
}

const GLB_MAGIC: u32 = 0x4654_6c67;
const CHUNK_JSON: u32 = 0x4e4f_534a;
const CHUNK_BIN: u32 = 0x004e_4942;

/// Required extensions read here.
const READ: &[&str] = &[
    "KHR_materials_unlit",
    "KHR_materials_emissive_strength",
    "KHR_mesh_quantization",
    "KHR_node_visibility",
];

fn refused_extension(e: &str) -> String {
    match e {
        "KHR_draco_mesh_compression" => "its meshes are Draco-compressed (KHR_draco_mesh_compression), which og.load.gltfData does not decode; export it without mesh compression".into(),
        "EXT_meshopt_compression" | "KHR_meshopt_compression" => format!("its buffers are meshopt-compressed ({e}), which og.load.gltfData does not decode; export it without compression"),
        "KHR_texture_basisu" => "its textures are Basis Universal (KHR_texture_basisu); export them as PNG or JPEG".into(),
        "EXT_texture_webp" | "EXT_texture_avif" => format!("its textures are {} ({e}); export them as PNG or JPEG", if e.ends_with("webp") { "WebP" } else { "AVIF" }),
        _ => format!("it requires the extension {e}, which og.load.gltfData does not read; export it without {e}"),
    }
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

/// A `data:` URI's bytes (base64 only).
fn data_uri(uri: &str, what: &str) -> Result<Vec<u8>, String> {
    let Some(rest) = uri.strip_prefix("data:") else {
        return Err(format!(
            "{what} names another file ({uri}); og.load.gltfData reads one file: pack the model as a .glb"
        ));
    };
    let (head, payload) = rest
        .split_once(',')
        .ok_or_else(|| format!("{what}: a data: URI without a comma"))?;
    if !head.ends_with(";base64") {
        return Err(format!("{what}: a data: URI that is not base64"));
    }
    base64(payload).ok_or_else(|| format!("{what}: its base64 does not decode"))
}

fn base64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
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

fn num(v: &Value, k: &str, default: f32) -> f32 {
    v.get(k)
        .and_then(Value::as_f64)
        .map_or(default, |x| x as f32)
}

fn floats<const N: usize>(v: &Value, k: &str, default: [f32; N]) -> [f32; N] {
    let mut out = default;
    if let Some(a) = v.get(k).and_then(Value::as_array) {
        for (o, x) in out.iter_mut().zip(a) {
            *o = x.as_f64().map_or(*o, |x| x as f32);
        }
    }
    out
}

fn index(v: &Value, k: &str) -> Option<usize> {
    v.get(k).and_then(Value::as_u64).map(|i| i as usize)
}

fn list<'a>(json: &'a Value, k: &str) -> &'a [Value] {
    json.get(k)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// A checked view of an accessor's elements.
struct Accessor<'a> {
    data: &'a [u8],
    stride: usize,
    count: usize,
    comps: usize,
    ctype: u64,
    normalized: bool,
}

impl Accessor<'_> {
    fn get(&self, i: usize, c: usize) -> f32 {
        let at = i * self.stride;
        let b = self.data;
        match self.ctype {
            5126 => f32::from_le_bytes([
                b[at + c * 4],
                b[at + c * 4 + 1],
                b[at + c * 4 + 2],
                b[at + c * 4 + 3],
            ]),
            5120 => {
                let v = b[at + c] as i8 as f32;
                if self.normalized {
                    (v / 127.0).max(-1.0)
                } else {
                    v
                }
            }
            5121 => {
                let v = b[at + c] as f32;
                if self.normalized { v / 255.0 } else { v }
            }
            5122 => {
                let v = i16::from_le_bytes([b[at + c * 2], b[at + c * 2 + 1]]) as f32;
                if self.normalized {
                    (v / 32767.0).max(-1.0)
                } else {
                    v
                }
            }
            5123 => {
                let v = u16::from_le_bytes([b[at + c * 2], b[at + c * 2 + 1]]) as f32;
                if self.normalized { v / 65535.0 } else { v }
            }
            _ => u32_at(b, at + c * 4) as f32,
        }
    }

    fn uint(&self, i: usize) -> u32 {
        let at = i * self.stride;
        match self.ctype {
            5121 => self.data[at] as u32,
            5123 => u16::from_le_bytes([self.data[at], self.data[at + 1]]) as u32,
            _ => u32_at(self.data, at),
        }
    }

    fn vec<const N: usize>(&self, i: usize) -> [f32; N] {
        let mut out = [0.0; N];
        for (c, o) in out.iter_mut().enumerate().take(self.comps) {
            *o = self.get(i, c);
        }
        out
    }
}

impl UncookedGltf {
    /// Reads a binary glTF (`.glb`) or a `.gltf` whose buffers and images
    /// are `data:` URIs: everything but the primitives and the images'
    /// pixels. `id` names the model (its textures are `<id>#tex<n>`).
    pub fn read(id: &str, bytes: &[u8]) -> Result<UncookedGltf, String> {
        let (json, bin) = if bytes.len() >= 12 && u32_at(bytes, 0) == GLB_MAGIC {
            let version = u32_at(bytes, 4);
            if version != 2 {
                return Err(format!(
                    "a glTF {version} binary; og.load.gltfData reads glTF 2.0"
                ));
            }
            let length = (u32_at(bytes, 8) as usize).min(bytes.len());
            let mut at = 12;
            let (mut json, mut bin) = (None, None);
            while at + 8 <= length {
                let len = u32_at(bytes, at) as usize;
                let kind = u32_at(bytes, at + 4);
                let body = bytes
                    .get(at + 8..at + 8 + len)
                    .filter(|_| at + 8 + len <= length)
                    .ok_or("the .glb is cut short (a chunk runs past its end)")?;
                match kind {
                    CHUNK_JSON if json.is_none() => json = Some(body),
                    CHUNK_BIN if bin.is_none() => bin = Some(body),
                    _ => {}
                }
                at += 8 + len.next_multiple_of(4);
            }
            let json = json.ok_or("a .glb without its JSON chunk")?;
            (json, bin)
        } else {
            let start = bytes
                .iter()
                .position(|b| !b" \t\r\n\xef\xbb\xbf".contains(b));
            if start.is_none_or(|s| bytes[s] != b'{') {
                return Err("not a glTF (neither a .glb nor .gltf JSON)".into());
            }
            (bytes, None)
        };
        let json: Value =
            serde_json::from_slice(json.strip_prefix(b"\xef\xbb\xbf").unwrap_or(json))
                .map_err(|e| format!("its glTF JSON does not parse: {e}"))?;
        let version = json["asset"]["version"].as_str().unwrap_or("");
        if !version.starts_with("2.") {
            return Err(format!(
                "a glTF of asset.version {version:?}; og.load.gltfData reads glTF 2.0"
            ));
        }
        for e in list(&json, "extensionsRequired") {
            let e = e.as_str().unwrap_or("");
            if !READ.contains(&e) {
                return Err(refused_extension(e));
            }
        }
        let mut buffers = Vec::new();
        for (i, b) in list(&json, "buffers").iter().enumerate() {
            let what = format!("buffer {i}");
            let data = match b.get("uri").and_then(Value::as_str) {
                Some(uri) => data_uri(uri, &what)?,
                None if i == 0 => bin
                    .ok_or("buffer 0 has no uri and the file no binary chunk")?
                    .to_vec(),
                None => return Err(format!("{what} has no uri")),
            };
            let len = index(b, "byteLength").unwrap_or(0);
            if data.len() < len {
                return Err(format!(
                    "{what} holds {} bytes but says byteLength {len} (the file is cut short)",
                    data.len()
                ));
            }
            buffers.push(data);
        }
        let mut model = UncookedGltf {
            id: id.to_string(),
            json,
            buffers,
            prims: Vec::new(),
            triangles: 0,
            images: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            scenes: Vec::new(),
            default_scene: None,
        };
        model.read_meshes()?;
        model.read_materials()?;
        model.read_nodes()?;
        Ok(model)
    }

    /// The triangle primitives, their triangle count, and the meshes'
    /// primitive ranges.
    fn read_meshes(&mut self) -> Result<(), String> {
        let mut meshes = Vec::new();
        for (m, mesh) in list(&self.json, "meshes").iter().enumerate() {
            let first = self.prims.len();
            for (p, prim) in list(mesh, "primitives").iter().enumerate() {
                let mode = index(prim, "mode").unwrap_or(4);
                if !(4..=6).contains(&mode) {
                    continue;
                }
                let position = index(&prim["attributes"], "POSITION")
                    .ok_or_else(|| format!("mesh {m} primitive {p} has no POSITION"))?;
                let n = match index(prim, "indices") {
                    Some(i) => self.count(i)?,
                    None => self.count(position)?,
                } as u64;
                self.triangles += if mode == 4 {
                    n / 3
                } else {
                    n.saturating_sub(2)
                };
                self.prims.push((m, p));
            }
            meshes.push(CookedMesh {
                name: mesh["name"].as_str().unwrap_or("").to_string(),
                primitives: first..self.prims.len(),
            });
        }
        self.meshes = meshes;
        Ok(())
    }

    fn count(&self, accessor: usize) -> Result<usize, String> {
        index(&self.json["accessors"][accessor], "count")
            .ok_or_else(|| format!("accessor {accessor} is missing or has no count"))
    }

    /// The image a glTF texture samples: its index among [`Self::images`],
    /// read once.
    fn image_of(
        &mut self,
        texture: usize,
        seen: &mut BTreeMap<usize, usize>,
    ) -> Result<usize, String> {
        let t = &self.json["textures"][texture];
        let Some(source) = index(t, "source") else {
            let ext = t["extensions"]
                .as_object()
                .and_then(|o| o.keys().next().cloned())
                .unwrap_or_default();
            return Err(if ext.is_empty() {
                format!("texture {texture} has no image")
            } else {
                refused_extension(&ext)
            });
        };
        if let Some(&i) = seen.get(&source) {
            return Ok(i);
        }
        let what = format!("image {source}");
        let img = &self.json["images"][source];
        let bytes = if let Some(uri) = img.get("uri").and_then(Value::as_str) {
            data_uri(uri, &what)?
        } else if let Some(view) = index(img, "bufferView") {
            self.view(view)?.to_vec()
        } else {
            return Err(format!("{what} has neither a uri nor a bufferView"));
        };
        let name = match img["name"].as_str() {
            Some(n) if !n.is_empty() => format!("{what} ({n})"),
            _ => what,
        };
        self.images.push((name, bytes));
        seen.insert(source, self.images.len() - 1);
        Ok(self.images.len() - 1)
    }

    /// A buffer view's bytes.
    fn view(&self, view: usize) -> Result<&[u8], String> {
        let v = &self.json["bufferViews"][view];
        let buffer = index(v, "buffer").ok_or_else(|| format!("bufferView {view} is missing"))?;
        let offset = index(v, "byteOffset").unwrap_or(0);
        let len = index(v, "byteLength").unwrap_or(0);
        self.buffers
            .get(buffer)
            .and_then(|b| b.get(offset..offset.checked_add(len)?))
            .ok_or_else(|| format!("bufferView {view} runs past its buffer"))
    }

    fn read_materials(&mut self) -> Result<(), String> {
        let mut seen = BTreeMap::new();
        let mut textures: BTreeMap<(usize, u32), usize> = BTreeMap::new();
        let mats: Vec<Value> = list(&self.json, "materials").to_vec();
        for m in &mats {
            let pbr = &m["pbrMetallicRoughness"];
            let mut tex =
                |this: &mut Self, v: &Value, kind: u32| -> Result<Option<usize>, String> {
                    let Some(t) = index(v, "index") else {
                        return Ok(None);
                    };
                    let image = this.image_of(t, &mut seen)?;
                    // Nearest-magnified colour is pixel art: sampled nearest, as
                    // a cooked pixel texture is.
                    let sampler = index(&this.json["textures"][t], "sampler")
                        .map(|s| &this.json["samplers"][s]);
                    let kind = match sampler.and_then(|s| index(s, "magFilter")) {
                        Some(9728) if kind == format::kind::COLOR => format::kind::PIXEL,
                        _ => kind,
                    };
                    let n = this.textures.len();
                    let i = *textures.entry((image, kind)).or_insert(n);
                    if i == n {
                        let id = format!("{}#tex{n}", this.id);
                        this.textures.push((TextureRef { id, kind }, image));
                    }
                    Ok(Some(i))
                };
            use format::kind::{COLOR, LINEAR, NORMAL};
            let ext = &m["extensions"];
            let strength = num(
                &ext["KHR_materials_emissive_strength"],
                "emissiveStrength",
                1.0,
            );
            let emissive = floats(m, "emissiveFactor", [0.0; 3]).map(|c| c * strength);
            let material = CookedMaterial {
                name: m["name"].as_str().unwrap_or("").to_string(),
                base_color: floats(pbr, "baseColorFactor", [1.0; 4]),
                metallic: num(pbr, "metallicFactor", 1.0),
                roughness: num(pbr, "roughnessFactor", 1.0),
                normal_scale: num(&m["normalTexture"], "scale", 1.0),
                occlusion_strength: num(&m["occlusionTexture"], "strength", 1.0),
                emissive,
                alpha_mode: match m["alphaMode"].as_str() {
                    Some("MASK") => format::alpha::MASK,
                    Some("BLEND") => format::alpha::BLEND,
                    _ => format::alpha::OPAQUE,
                },
                alpha_cutoff: num(m, "alphaCutoff", 0.5),
                double_sided: m["doubleSided"].as_bool().unwrap_or(false),
                base_color_texture: tex(self, &pbr["baseColorTexture"], COLOR)?,
                metallic_roughness_texture: tex(self, &pbr["metallicRoughnessTexture"], LINEAR)?,
                normal_texture: tex(self, &m["normalTexture"], NORMAL)?,
                emissive_texture: tex(self, &m["emissiveTexture"], COLOR)?,
                occlusion_texture: tex(self, &m["occlusionTexture"], LINEAR)?,
                physical: PhysicalExtensions::default(),
                unlit: ext.get("KHR_materials_unlit").is_some(),
                uv_transforms: [og_render::UvTransform::IDENTITY; 5],
            };
            self.materials.push(material);
        }
        // glTF's default material, last (as the cook adds it).
        self.materials.push(CookedMaterial {
            name: "default".into(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.6,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            emissive: [0.0; 3],
            alpha_mode: format::alpha::OPAQUE,
            alpha_cutoff: 0.5,
            double_sided: false,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emissive_texture: None,
            occlusion_texture: None,
            physical: PhysicalExtensions::default(),
            unlit: false,
            uv_transforms: [og_render::UvTransform::IDENTITY; 5],
        });
        Ok(())
    }

    /// The nodes with parents before children, and the scenes.
    fn read_nodes(&mut self) -> Result<(), String> {
        let nodes = list(&self.json, "nodes");
        let mut parent = vec![None; nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            for c in list(n, "children") {
                let c = c.as_u64().map(|c| c as usize).filter(|&c| c < nodes.len());
                let c = c.ok_or_else(|| format!("node {i} has a child that is not a node"))?;
                if parent[c].is_some() || c == i {
                    return Err(format!("node {c} has more than one parent"));
                }
                parent[c] = Some(i);
            }
        }
        // Depth first from the parentless nodes, in glTF order.
        let mut order = Vec::with_capacity(nodes.len());
        let mut stack: Vec<usize> = (0..nodes.len())
            .rev()
            .filter(|&i| parent[i].is_none())
            .collect();
        while let Some(n) = stack.pop() {
            order.push(n);
            let children: Vec<usize> = list(&nodes[n], "children")
                .iter()
                .filter_map(|c| c.as_u64().map(|c| c as usize))
                .collect();
            stack.extend(children.into_iter().rev());
        }
        if order.len() != nodes.len() {
            return Err("the nodes' children form a cycle".into());
        }
        let mut at = vec![0; nodes.len()];
        for (new, &old) in order.iter().enumerate() {
            at[old] = new;
        }
        for &old in &order {
            let n = &nodes[old];
            let mesh = index(n, "mesh");
            if mesh.is_some_and(|m| m >= self.meshes.len()) {
                return Err(format!("node {old} names a mesh that is not there"));
            }
            let (scale, rotation, translation) = match n.get("matrix") {
                Some(_) => {
                    Mat4::from_cols_array(&floats(n, "matrix", Mat4::IDENTITY.to_cols_array()))
                        .to_scale_rotation_translation()
                }
                None => (
                    Vec3::from(floats(n, "scale", [1.0; 3])),
                    Quat::from_array(floats(n, "rotation", [0.0, 0.0, 0.0, 1.0])).normalize(),
                    Vec3::from(floats(n, "translation", [0.0; 3])),
                ),
            };
            self.nodes.push(CookedNode {
                name: n["name"].as_str().unwrap_or("").to_string(),
                parent: parent[old].map(|p| at[p]),
                mesh,
                translation,
                rotation,
                scale,
                visible: n["extensions"]["KHR_node_visibility"]["visible"]
                    .as_bool()
                    .unwrap_or(true),
            });
        }
        let root = |v: &Value| v.as_u64().map(|r| r as usize).filter(|&r| r < nodes.len());
        for (s, sc) in list(&self.json, "scenes").iter().enumerate() {
            let roots = list(sc, "nodes")
                .iter()
                .map(|r| {
                    root(r)
                        .map(|r| at[r])
                        .ok_or_else(|| format!("scene {s} names a node that is not there"))
                })
                .collect::<Result<_, String>>()?;
            self.scenes.push(CookedScene {
                name: sc["name"].as_str().unwrap_or("").to_string(),
                roots,
            });
        }
        // No scene: every parentless node is one.
        if self.scenes.is_empty() && !nodes.is_empty() {
            self.scenes.push(CookedScene {
                name: String::new(),
                roots: (0..self.nodes.len())
                    .filter(|&n| self.nodes[n].parent.is_none())
                    .collect(),
            });
        }
        self.default_scene = match index(&self.json, "scene") {
            Some(s) if s < self.scenes.len() => Some(s),
            Some(s) => return Err(format!("scene {s} is the default but is not there")),
            None => (!self.scenes.is_empty()).then_some(0),
        };
        Ok(())
    }

    /// How many primitives [`Self::primitive`] converts.
    pub fn primitive_count(&self) -> usize {
        self.prims.len()
    }

    fn accessor(&self, i: usize) -> Result<Accessor<'_>, String> {
        let a = &self.json["accessors"][i];
        if a.get("sparse").is_some() {
            return Err(format!(
                "accessor {i} is sparse, which og.load.gltfData does not read; export without sparse accessors"
            ));
        }
        let comps = match a["type"].as_str() {
            Some("SCALAR") => 1,
            Some("VEC2") => 2,
            Some("VEC3") => 3,
            Some("VEC4") => 4,
            t => return Err(format!("accessor {i} is of type {t:?}")),
        };
        let ctype = a["componentType"].as_u64().unwrap_or(0);
        let size = match ctype {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            t => return Err(format!("accessor {i} has componentType {t}")),
        };
        let count = index(a, "count").ok_or_else(|| format!("accessor {i} has no count"))?;
        let view = index(a, "bufferView").ok_or_else(|| {
            format!(
                "accessor {i} has no bufferView (all zeros), which og.load.gltfData does not read"
            )
        })?;
        let data = self.view(view)?;
        let elem = comps * size;
        let stride = index(&self.json["bufferViews"][view], "byteStride")
            .unwrap_or(elem)
            .max(elem);
        let offset = index(a, "byteOffset").unwrap_or(0);
        let need = match count {
            0 => offset,
            n => offset + stride * (n - 1) + elem,
        };
        let data = data
            .get(offset..need.max(offset))
            .filter(|_| need <= data.len())
            .ok_or_else(|| format!("accessor {i} runs past its bufferView"))?;
        Ok(Accessor {
            data,
            stride,
            count,
            comps,
            ctype,
            normalized: a["normalized"].as_bool().unwrap_or(false),
        })
    }

    /// Converts primitive `n` (in [`CookedModel::primitives`] order).
    pub fn primitive(&self, n: usize) -> Result<CookedPrimitive, String> {
        let (m, p) = self.prims[n];
        let prim = &self.json["meshes"][m]["primitives"][p];
        let what = format!("mesh {m} primitive {p}");
        let attrs = &prim["attributes"];
        let attr = |k: &str| index(attrs, k).map(|i| self.accessor(i)).transpose();
        let positions = attr("POSITION")?.ok_or_else(|| format!("{what} has no POSITION"))?;
        let count = positions.count;
        let same = |k: &str| -> Result<Option<Accessor<'_>>, String> {
            match attr(k)? {
                Some(a) if a.count != count => Err(format!(
                    "{what}: {k} has {} elements for {count} positions",
                    a.count
                )),
                a => Ok(a),
            }
        };
        let normals = same("NORMAL")?;
        let uvs = same("TEXCOORD_0")?;
        let colors = same("COLOR_0")?;
        let tangents = same("TANGENT")?;
        let strip: Vec<u32> = match index(prim, "indices") {
            Some(i) => {
                let a = self.accessor(i)?;
                if !matches!(a.ctype, 5121 | 5123 | 5125) || a.comps != 1 {
                    return Err(format!("{what}: its indices are not unsigned integers"));
                }
                (0..a.count).map(|i| a.uint(i)).collect()
            }
            None => (0..count as u32).collect(),
        };
        if let Some(bad) = strip.iter().find(|&&i| i as usize >= count) {
            return Err(format!("{what}: index {bad} is past its {count} vertices"));
        }
        let indices: Vec<u32> = match index(prim, "mode").unwrap_or(4) {
            4 => strip[..strip.len() / 3 * 3].to_vec(),
            5 => (0..strip.len().saturating_sub(2))
                .flat_map(|i| match i % 2 {
                    0 => [strip[i], strip[i + 1], strip[i + 2]],
                    _ => [strip[i + 1], strip[i], strip[i + 2]],
                })
                .collect(),
            _ => (0..strip.len().saturating_sub(2))
                .flat_map(|i| [strip[0], strip[i + 1], strip[i + 2]])
                .collect(),
        };
        let pos: Vec<Vec3> = (0..count)
            .map(|i| Vec3::from(positions.vec::<3>(i)))
            .collect();
        // Without normals, each vertex takes the area-weighted normal of
        // its triangles (flat where the vertices are not shared).
        let normal: Vec<Vec3> = match &normals {
            Some(a) => (0..count)
                .map(|i| Vec3::from(a.vec::<3>(i)).normalize_or_zero())
                .collect(),
            None => {
                let mut n = vec![Vec3::ZERO; count];
                for t in indices.as_chunks::<3>().0 {
                    let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
                    let f = (pos[b] - pos[a]).cross(pos[c] - pos[a]);
                    n[a] += f;
                    n[b] += f;
                    n[c] += f;
                }
                n.into_iter()
                    .map(|v| v.try_normalize().unwrap_or(Vec3::Y))
                    .collect()
            }
        };
        let mut tangent_list = Vec::new();
        let mut vertices = Vec::with_capacity(count);
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for i in 0..count {
            min = min.min(pos[i]);
            max = max.max(pos[i]);
            let color = match &colors {
                Some(a) if a.comps == 3 => {
                    let c = a.vec::<3>(i);
                    [c[0], c[1], c[2], 1.0]
                }
                Some(a) => a.vec::<4>(i),
                None => [1.0; 4],
            };
            let tangent = match &tangents {
                Some(a) => {
                    let t = a.vec::<4>(i);
                    tangent_list.push(t);
                    t
                }
                None => [0.0; 4],
            };
            vertices.push(Vertex {
                position: pos[i].to_array(),
                normal: normal[i].to_array(),
                uv: uvs.as_ref().map_or([0.0; 2], |a| a.vec::<2>(i)),
                color,
                tangent,
            });
        }
        if count == 0 {
            (min, max) = (Vec3::ZERO, Vec3::ZERO);
        }
        let material = match index(prim, "material") {
            Some(i) if i < self.materials.len() - 1 => i,
            Some(i) => return Err(format!("{what} names material {i}, which is not there")),
            None => self.materials.len() - 1,
        };
        Ok(CookedPrimitive {
            material,
            vertices,
            tangents: tangent_list,
            lods: vec![Lod {
                first: 0,
                count: indices.len(),
                error: 0.0,
            }],
            indices,
            min,
            max,
            bvh_parts: None,
        })
    }

    /// The model, from every primitive [`Self::primitive`] converted, in
    /// order.
    pub fn finish(self, primitives: Vec<CookedPrimitive>) -> CookedModel {
        let mut model = CookedModel {
            id: self.id,
            primitives,
            meshes: self.meshes,
            materials: self.materials,
            textures: self.textures.into_iter().map(|(t, _)| t).collect(),
            nodes: self.nodes,
            scenes: self.scenes,
            default_scene: self.default_scene,
            min: Vec3::ZERO,
            max: Vec3::ZERO,
            skipped_animations: list(&self.json, "animations").len() as u32,
            skipped_skins: list(&self.json, "skins").len() as u32,
            lights: Vec::new(),
            instances: BTreeMap::new(),
            variants: Vec::new(),
            variant_materials: Vec::new(),
        };
        // The default scene's bounds, with the nodes' transforms.
        let world = model.node_world();
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        if let Some(s) = model.default_scene {
            for n in model.scene_nodes(s) {
                let Some(m) = model.nodes[n].mesh else {
                    continue;
                };
                for p in model.meshes[m].primitives.clone() {
                    let (a, b) = (model.primitives[p].min, model.primitives[p].max);
                    for c in 0..8 {
                        let corner = Vec3::new(
                            if c & 1 == 0 { a.x } else { b.x },
                            if c & 2 == 0 { a.y } else { b.y },
                            if c & 4 == 0 { a.z } else { b.z },
                        );
                        let w = world[n].transform_point3(corner);
                        min = min.min(w);
                        max = max.max(w);
                    }
                }
            }
        }
        if min.x <= max.x {
            (model.min, model.max) = (min, max);
        }
        model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A .glb of one indexed triangle and one strip quad (two meshes), the
    /// second under a translated child node.
    fn glb(json: &str, bin: &[u8]) -> Vec<u8> {
        let mut j = json.as_bytes().to_vec();
        while !j.len().is_multiple_of(4) {
            j.push(b' ');
        }
        let mut b = bin.to_vec();
        while !b.len().is_multiple_of(4) {
            b.push(0);
        }
        let mut out = Vec::new();
        out.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&((12 + 8 + j.len() + 8 + b.len()) as u32).to_le_bytes());
        out.extend_from_slice(&(j.len() as u32).to_le_bytes());
        out.extend_from_slice(&CHUNK_JSON.to_le_bytes());
        out.extend_from_slice(&j);
        out.extend_from_slice(&(b.len() as u32).to_le_bytes());
        out.extend_from_slice(&CHUNK_BIN.to_le_bytes());
        out.extend_from_slice(&b);
        out
    }

    fn sample() -> Vec<u8> {
        let mut bin = Vec::new();
        // Four positions (48 bytes), then three u16 indices.
        for p in [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ] {
            for c in p {
                bin.extend_from_slice(&c.to_le_bytes());
            }
        }
        for i in [0u16, 1, 2] {
            bin.extend_from_slice(&i.to_le_bytes());
        }
        let json = r#"{"asset":{"version":"2.0"},"scene":0,
          "scenes":[{"nodes":[0]}],
          "nodes":[{"mesh":0,"children":[1]},{"mesh":1,"translation":[5,0,0]}],
          "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]},
                    {"primitives":[{"attributes":{"POSITION":0},"mode":5},{"attributes":{"POSITION":0},"mode":1}]}],
          "buffers":[{"byteLength":54}],
          "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":48},{"buffer":0,"byteOffset":48,"byteLength":6}],
          "accessors":[{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3"},
                       {"bufferView":1,"componentType":5123,"count":3,"type":"SCALAR"}]}"#;
        glb(json, &bin)
    }

    #[test]
    fn a_glb_reads_into_a_model_with_its_nodes_and_bounds() {
        let g = UncookedGltf::read("drawings/kite.glb", &sample()).unwrap();
        // One triangle, a strip of four vertices (two), the lines skipped.
        assert_eq!(g.triangles, 3);
        assert_eq!(g.primitive_count(), 2);
        let prims = (0..2).map(|p| g.primitive(p).unwrap()).collect::<Vec<_>>();
        assert_eq!(prims[0].indices, vec![0, 1, 2]);
        // The strip's second triangle is turned to keep the winding.
        assert_eq!(prims[1].indices, vec![0, 1, 2, 2, 1, 3]);
        // No normals in the file: the triangle faces +z.
        assert_eq!(prims[0].vertices[0].normal, [0.0, 0.0, 1.0]);
        let model = g.finish(prims);
        assert_eq!(model.meshes[1].primitives, 1..2);
        assert_eq!(model.nodes[1].parent, Some(0));
        // The child's quad is moved 5 along x: the bounds reach 6.
        assert_eq!(
            (model.min, model.max),
            (Vec3::ZERO, Vec3::new(6.0, 1.0, 0.0))
        );
        // The default material is last, and both primitives draw with it.
        assert_eq!(model.primitives[0].material, model.materials.len() - 1);
    }

    #[test]
    fn what_the_reader_does_not_decode_is_refused_by_name() {
        let with = |extra: &str| {
            format!(r#"{{"asset":{{"version":"2.0"}},{extra}"buffers":[]}}"#).into_bytes()
        };
        let err = |b: Vec<u8>| UncookedGltf::read("m", &b).err().unwrap();
        assert!(
            err(with(
                r#""extensionsRequired":["KHR_draco_mesh_compression"],"#
            ))
            .contains("Draco")
        );
        assert!(
            err(with(r#""extensionsRequired":["EXT_meshopt_compression"],"#)).contains("meshopt")
        );
        assert!(
            err(with(r#""extensionsRequired":["KHR_texture_basisu"],"#)).contains("PNG or JPEG")
        );
        let external =
            r#"{"asset":{"version":"2.0"},"buffers":[{"uri":"kite.bin","byteLength":4}]}"#;
        assert!(err(external.as_bytes().to_vec()).contains("pack the model as a .glb"));
        assert!(err(b"PK\x03\x04".to_vec()).contains("not a glTF"));
        // A sparse accessor is refused when its primitive is converted.
        let sparse = r#"{"asset":{"version":"2.0"},"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}],
          "buffers":[{"uri":"data:application/octet-stream;base64,AAAAAA==","byteLength":4}],
          "bufferViews":[{"buffer":0,"byteLength":4}],
          "accessors":[{"bufferView":0,"componentType":5126,"count":0,"type":"VEC3","sparse":{"count":1}}]}"#;
        let g = UncookedGltf::read("m", sparse.as_bytes()).unwrap();
        assert!(g.primitive(0).err().unwrap().contains("sparse"));
        // A cut-short buffer is refused before anything is read from it.
        let short = r#"{"asset":{"version":"2.0"},"buffers":[{"uri":"data:;base64,AAAA","byteLength":40}]}"#;
        assert!(err(short.as_bytes().to_vec()).contains("cut short"));
    }

    #[test]
    fn base64_decodes_padded_and_unpadded() {
        assert_eq!(base64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64("aGVsbG8").unwrap(), b"hello");
        assert_eq!(base64("AAEC").unwrap(), [0, 1, 2]);
        assert!(base64("a*b").is_none());
    }
}
