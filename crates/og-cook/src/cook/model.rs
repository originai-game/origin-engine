//! glTF 2.0 → model blob: meshoptimizer-processed primitives with an LOD
//! chain, materials, the node graph and scenes. Textures become their own
//! blobs (see `texture.rs`); the model refers to them by asset id.

use super::{CookError, MeshOptions};
use crate::format::{self, *};
use glam::{Mat4, Quat, Vec3};
use og_blob::BlobWriter;
use std::path::Path;

mod blob;
mod ext;
mod impostor;
pub mod plain;
mod primitive;
pub(super) mod skin_report;

use super::conform;
pub use blob::*;
pub(super) use ext::uv_plan;
pub use impostor::{Baked, ImpostorBake};
pub(super) use primitive::*;

/// A texture the model needs: which image, cooked as which kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureUse {
    pub image: usize,
    pub kind: u32,
    /// The base colour of a material that tests its alpha (glTF
    /// `alphaMode` `MASK`): a cut-out, whose mips keep its coverage at the
    /// material's cut-off (texture.rs `cutout_mips` and `keep_coverage`),
    /// here the least texture alpha byte the material keeps ([`cuts_out`]);
    /// `None` for every other use. An image that a cut-out material and
    /// another material sample, or two cut-out materials whose cut-offs come
    /// to different bytes, is that many uses, each with its own mips.
    pub cutout: Option<u8>,
}

/// A material's base colour use's [`TextureUse::cutout`]: for an
/// alpha-tested material (`MASK`), the least texture alpha byte it keeps.
/// The renderer discards where the texture's alpha times `baseColorFactor`'s
/// alpha (and a vertex colour's) is below `alphaCutoff` (0.5 when it names
/// none), so the byte is 255 × cut-off / factor, rounded up. `None`, the
/// encoder's mips as for any texture, where no one byte is the test: a
/// cut-off of 0 or less keeps every texel, a factor under the cut-off (0
/// too) keeps none, and `vertex_alpha` (a vertex alpha under 1 on a
/// primitive drawn with the material, [`vertex_alphas`]) moves the test per
/// vertex.
pub(super) fn cuts_out(m: &gltf::Material, vertex_alpha: bool) -> Option<u8> {
    if m.alpha_mode() != gltf::material::AlphaMode::Mask || vertex_alpha {
        return None;
    }
    let cut = m.alpha_cutoff().unwrap_or(0.5);
    let factor = m.pbr_metallic_roughness().base_color_factor()[3];
    // cut / factor first: 1 exactly when the two are equal.
    let byte = (cut / factor * 255.0).ceil();
    (cut > 0.0 && byte <= 255.0).then_some(byte as u8)
}

/// Per material, whether a primitive drawn with it (as its material or
/// under a `KHR_materials_variants` variant) has a `COLOR_0` with an alpha
/// under 1 (any of its component types: float, normalised u8 or u16).
fn vertex_alphas(
    doc: &gltf::Document,
    json: &serde_json::Value,
    buffers: &[gltf::buffer::Data],
) -> Vec<bool> {
    let mut out = vec![false; doc.materials().count()];
    for mesh in doc.meshes() {
        for prim in mesh.primitives() {
            let below = prim
                .reader(|b| Some(&buffers[b.index()].0[..]))
                .read_colors(0)
                .is_some_and(|c| c.into_rgba_f32().any(|c| c[3] < 1.0));
            if !below {
                continue;
            }
            let mapped = ext::mappings(&json["meshes"][mesh.index()]["primitives"][prim.index()]);
            for m in prim
                .material()
                .index()
                .into_iter()
                .chain(mapped.into_iter().map(|(m, _)| m))
            {
                if let Some(v) = out.get_mut(m) {
                    *v = true;
                }
            }
        }
    }
    out
}

/// A parsed glTF: the document, its buffers and the encoded image bytes.
pub struct GltfSource {
    pub doc: gltf::Document,
    /// The plain glTF's JSON ([`plain`]): the extensions the gltf crate
    /// does not model (texture transforms, lights, instancing, variants)
    /// are read from it.
    pub json: serde_json::Value,
    pub buffers: Vec<gltf::buffer::Data>,
    /// Encoded image bytes (PNG/JPEG) per glTF image.
    pub images: Vec<Vec<u8>>,
    /// Bytes of every external file the glTF referenced (part of its source hash).
    pub external: Vec<(String, Vec<u8>)>,
    pub textures: Vec<TextureUse>,
    /// Per material, its base colour use's [`TextureUse::cutout`]
    /// ([`cuts_out`]).
    pub cutouts: Vec<Option<u8>>,
    pub warnings: Vec<String>,
    /// How the art contract's face budget is shared over its primitives
    /// (`conform::mesh`), made by the first output that needs it.
    pub(super) conform_plan: std::sync::OnceLock<super::conform::mesh::Plan>,
    /// What the model says it is (`conform character`, `conform effect`,
    /// [`super::conform::model_role`]): the face budget the art contract
    /// holds it to.
    pub role: super::conform::ModelRole,
    /// A `type model` built again at fewer segments for its face budget
    /// (`model_text::to_glb_within`): the factor and the triangles it
    /// drew as written, for the cook report.
    pub(super) segments: Option<(f64, usize)>,
    /// The model's impostor, baked by the first of its two atlases' outputs.
    pub(super) impostor: std::sync::OnceLock<Result<Baked, String>>,
}

impl GltfSource {
    /// A neutral, full-strength emissive map may contain accidental albedo.
    /// glTF emissive RGB is sRGB; luminance is measured in linear light.
    pub(super) fn emissive_warnings(&self) -> Result<Vec<String>, String> {
        let mut means = std::collections::BTreeMap::new();
        let mut notes = Vec::new();
        for material in self.doc.materials() {
            if material.emissive_factor() != [1.0; 3] {
                continue;
            }
            let Some(texture) = material.emissive_texture() else {
                continue;
            };
            let index = texture.texture().source().index();
            let mean = match means.entry(index) {
                std::collections::btree_map::Entry::Occupied(e) => *e.get(),
                std::collections::btree_map::Entry::Vacant(e) => {
                    let image = super::texture::decode(&self.images[index])?;
                    let linear: [f64; 256] = std::array::from_fn(|v| {
                        let s = v as f64 / 255.0;
                        if s <= 0.04045 {
                            s / 12.92
                        } else {
                            ((s + 0.055) / 1.055).powf(2.4)
                        }
                    });
                    let sum: f64 = image
                        .rgba
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|p| {
                            0.2126 * linear[p[0] as usize]
                                + 0.7152 * linear[p[1] as usize]
                                + 0.0722 * linear[p[2] as usize]
                        })
                        .sum();
                    *e.insert(sum / (f64::from(image.width) * f64::from(image.height)))
                }
            };
            if mean > 0.01 {
                notes.push(format!("warning: emissive texture: material {} '{}': emissive factor [1, 1, 1], image {index} mean linear luminance {mean:.5} >0.01; check for albedo baked into emission (no automatic change)", material.index().unwrap(), material.name().unwrap_or("")));
            }
        }
        Ok(notes)
    }

    /// The model's impostor baked as `b` asks (once for both atlases).
    pub fn impostor(&self, b: ImpostorBake) -> Result<&Baked, String> {
        self.impostor
            .get_or_init(|| {
                let instanced = ext::instances(self)?;
                let bounds = blob::scene_bounds(self, &instanced);
                impostor::bake(self, &instanced, bounds, b)
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

/// Every skinned mesh node's bind pose in model space, by glTF node index
/// (format's `NodeBindRow`): its skin's first joint's rest transform (the
/// joints' parents' transforms from the scene's root) times that joint's
/// inverse bind matrix (identity without them). glTF ignores a skinned
/// mesh node's own transform, so this, not the node's, places its
/// vertices at rest: for its bounds, its levels' errors and a draw
/// without the pose.
pub(crate) fn skin_binds(src: &GltfSource) -> std::collections::BTreeMap<usize, Mat4> {
    let doc = &src.doc;
    let n = doc.nodes().count();
    let mut parent = vec![usize::MAX; n];
    for node in doc.nodes() {
        for c in node.children() {
            parent[c.index()] = node.index();
        }
    }
    let local = |i: usize| {
        let (t, r, s) = doc
            .nodes()
            .nth(i)
            .expect("in range")
            .transform()
            .decomposed();
        Mat4::from_scale_rotation_translation(Vec3::from(s), Quat::from_array(r), Vec3::from(t))
    };
    let world = |mut i: usize| {
        let mut m = Mat4::IDENTITY;
        let mut steps = 0;
        while i != usize::MAX && steps <= n {
            m = local(i) * m;
            i = parent[i];
            steps += 1;
        }
        m
    };
    let mut out = std::collections::BTreeMap::new();
    for node in doc.nodes() {
        let (Some(skin), Some(_)) = (node.skin(), node.mesh()) else {
            continue;
        };
        let Some(joint) = skin.joints().next() else {
            continue;
        };
        let reader = skin.reader(|b| Some(&src.buffers[b.index()].0[..]));
        let ibm = reader
            .read_inverse_bind_matrices()
            .and_then(|mut m| m.next())
            .map_or(Mat4::IDENTITY, |m| Mat4::from_cols_array_2d(&m));
        let bind = world(joint.index()) * ibm;
        if bind.is_finite() && bind.determinant().abs() > 1e-12 {
            out.insert(node.index(), bind);
        }
    }
    out
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

pub fn load(bytes: &[u8], dir: &Path) -> Result<GltfSource, String> {
    let plain::Plain {
        json,
        buffers,
        external,
        warnings: plain_warnings,
    } = plain::plain(bytes, dir)?;
    let root: gltf::json::Root =
        serde_json::from_value(json.clone()).map_err(|e| format!("glTF: {e}"))?;
    let doc = gltf::Document::from_json(root).map_err(|e| format!("glTF: {e}"))?;
    let mut external = external;
    for (b, data) in doc.buffers().zip(&buffers) {
        if data.len() < b.length() {
            return Err(format!(
                "glTF: buffer {} is shorter than declared",
                b.index()
            ));
        }
    }
    let buffers: Vec<gltf::buffer::Data> = buffers.into_iter().map(gltf::buffer::Data).collect();
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
    let mut warnings = plain_warnings;
    let vertex_alpha = vertex_alphas(&doc, &json, &buffers);
    let mut cutouts = Vec::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let cutout = cuts_out(&m, m.index().is_some_and(|i| vertex_alpha[i]));
        cutouts.push(cutout);
        let slots = [
            (
                pbr.base_color_texture()
                    .map(|t| (t.texture(), t.tex_coord())),
                kind::COLOR,
                cutout,
            ),
            (
                pbr.metallic_roughness_texture()
                    .map(|t| (t.texture(), t.tex_coord())),
                kind::LINEAR,
                None,
            ),
            (
                m.normal_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::NORMAL,
                None,
            ),
            (
                m.emissive_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::COLOR,
                None,
            ),
            (
                m.occlusion_texture().map(|t| (t.texture(), t.tex_coord())),
                kind::LINEAR,
                None,
            ),
        ];
        // The physical extensions' textures (`KHR_materials_*`).
        og_render_data::gltf::physical_extensions(&doc, &m, |t, k| {
            let u = TextureUse {
                image: t.source().index(),
                kind: texture_kind(k),
                cutout: None,
            };
            if !textures.contains(&u) {
                textures.push(u);
            }
        });
        for (slot, k, cutout) in slots {
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
                    cutout,
                };
                if !textures.contains(&u) {
                    textures.push(u);
                }
            }
        }
    }
    Ok(GltfSource {
        doc,
        json,
        buffers,
        images,
        external,
        textures,
        cutouts,
        warnings,
        conform_plan: Default::default(),
        role: Default::default(),
        segments: None,
        impostor: Default::default(),
    })
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
