//! glTF 2.0 → model blob: meshoptimizer-processed primitives with an LOD
//! chain, materials, the node graph and scenes. Textures become their own
//! blobs (see `texture.rs`); the model refers to them by asset id.

use super::{
    Action, CookError, Cx, Exec, Job, KindDef, MeshOptions, NODE_ROWS, Out, PlanState, Planned,
    Source, Split, anim, anim_settings, assets_root, conform_off, ext, fracture, hash_bytes,
    mesh_for, model_stream_textures, not_split, stamp, texture,
};
use crate::format::{self, *};
use glam::{Mat4, Quat, Vec3};
use og_blob::BlobWriter;
use std::path::Path;

mod blob;
pub(crate) mod ext;
mod impostor;
mod lod_bake;
mod operand;
pub(crate) mod orient;
pub mod plain;
mod primitive;
pub(crate) mod process;
mod shape;
pub(crate) mod skin_report;

use super::conform;
pub use blob::*;
pub(crate) use ext::uv_plan;
pub use impostor::{Baked, ImpostorBake};
pub(crate) use operand::geometry as operand_geometry;
pub(crate) use primitive::*;

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
pub(crate) fn cuts_out(m: &gltf::Material, vertex_alpha: bool) -> Option<u8> {
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
    /// Per material, the synthesized image index in [`Self::images`] when
    /// separate `metallicRoughnessTexture` and `occlusionTexture` maps were
    /// packed into one ORM (`R=AO, G=Roughness, B=Metallic`) texture at cook
    /// time.
    pub orm_overrides: Vec<Option<usize>>,
    pub warnings: Vec<String>,
    /// Import-time skin relaxation facts, reported even on incremental cooks.
    pub(crate) skin_relax_notes: Vec<String>,
    /// How the art contract's face budget is shared over its primitives
    /// (`conform::mesh`), made by the first output that needs it.
    pub(crate) conform_plan: std::sync::OnceLock<super::conform::mesh::Plan>,
    /// What the model says it is (`conform character`, `conform effect`,
    /// [`super::conform::model_role`]): the face budget the art contract
    /// holds it to.
    pub role: super::conform::ModelRole,
    /// A `type model` built again at fewer segments for its face budget
    /// (`model_text::to_glb_within`): the factor and the triangles it
    /// drew as written, for the cook report.
    pub(crate) segments: Option<(f64, usize)>,
    /// The model's impostor, baked by the first of its two atlases' outputs.
    pub(crate) impostor: std::sync::OnceLock<Result<Baked, String>>,
}

impl GltfSource {
    /// A neutral, full-strength emissive map may contain accidental albedo.
    /// glTF emissive RGB is sRGB; luminance is measured in linear light.
    /// With a base-colour map of the same size beside it, albedo baked into
    /// emission is emission that repeats the base colour: the map is
    /// flagged when it glows on at least half its texels and repeats the
    /// base colour (within 8 %) on at least half of those. A painted glow
    /// map (lines, seams, veins over a surface of their own colour) is not.
    /// Without one, a mean linear luminance above 0.01 is flagged.
    pub(crate) fn emissive_warnings(&self) -> Result<Vec<String>, String> {
        let linear: [f64; 256] = std::array::from_fn(|v| {
            let s = v as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        });
        let lum = |p: &[u8; 4]| {
            0.2126 * linear[p[0] as usize]
                + 0.7152 * linear[p[1] as usize]
                + 0.0722 * linear[p[2] as usize]
        };
        let mut decoded = std::collections::BTreeMap::new();
        let mut image = |index: usize| -> Result<std::rc::Rc<super::texture::Image>, String> {
            if let Some(i) = decoded.get(&index) {
                return Ok(std::rc::Rc::clone(i));
            }
            let i = std::rc::Rc::new(super::texture::decode(&self.images[index])?);
            decoded.insert(index, std::rc::Rc::clone(&i));
            Ok(i)
        };
        let mut notes = Vec::new();
        for material in self.doc.materials() {
            if material.emissive_factor() != [1.0; 3] {
                continue;
            }
            let Some(texture) = material.emissive_texture() else {
                continue;
            };
            let index = texture.texture().source().index();
            let glow = image(index)?;
            let px = glow.rgba.as_chunks::<4>().0;
            let mean =
                px.iter().map(&lum).sum::<f64>() / (f64::from(glow.width) * f64::from(glow.height));
            let base = material
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|t| image(t.texture().source().index()))
                .transpose()?
                .filter(|b| b.width == glow.width && b.height == glow.height);
            let flagged = match &base {
                None => mean > 0.01,
                Some(b) => {
                    let lit: Vec<usize> = (0..px.len()).filter(|&i| lum(&px[i]) > 0.01).collect();
                    let bp = b.rgba.as_chunks::<4>().0;
                    let same = lit
                        .iter()
                        .filter(|&&i| (0..3).all(|c| px[i][c].abs_diff(bp[i][c]) <= 20))
                        .count();
                    lit.len() * 2 >= px.len() && same * 2 >= lit.len().max(1)
                }
            };
            if flagged {
                let why = if base.is_some() {
                    "it repeats the base colour over most of the map"
                } else {
                    ">0.01"
                };
                notes.push(format!("warning: emissive texture: material {} '{}': emissive factor [1, 1, 1], image {index} mean linear luminance {mean:.5} {why}; check for albedo baked into emission (no automatic change)", material.index().unwrap(), material.name().unwrap_or("")));
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
        if og_render_data::skin::valid_bind(bind) {
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
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
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
    let bytes = read(&path)?;
    external.push((path, bytes.clone()));
    Ok(bytes)
}

/// Parses a `.glb`/`.gltf`, resolving its buffers and image bytes, and lists
/// the textures its materials use in first-use order.
/// The JSON of a glTF (a `.glb`'s JSON chunk, or the file) as the cook
/// reads it: with [`pointer_visibility`] applied. `None` when it does not
/// parse.
pub(crate) fn source_json(bytes: &[u8]) -> Option<serde_json::Value> {
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
pub(crate) fn pointer_visibility(json: &mut serde_json::Value) -> Result<Vec<String>, String> {
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

/// Samples channel `ch` of `img` bilinearly at normalized `(u, v)` in `[0, 1]`.
fn sample_channel_bilinear(img: &texture::Image, u: f32, v: f32, ch: usize) -> u8 {
    let w = img.width.max(1);
    let h = img.height.max(1);
    let x = (u * w as f32 - 0.5).clamp(0.0, (w - 1) as f32);
    let y = (v * h as f32 - 0.5).clamp(0.0, (h - 1) as f32);
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let at = |px: u32, py: u32| img.rgba[(py * w + px) as usize * 4 + ch] as f32;
    let top = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
    let bot = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
    (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8
}

/// The image a material's separate occlusion and metallic-roughness maps
/// pack into (`R = AO, G = Roughness, B = Metallic, A = 255`), as the bytes
/// [`GltfSource::images`] holds for it: this mark, then the two encoded
/// source images, each after its length. Nothing is decoded to make it:
/// [`texture::decode`] packs the two when the texture cooks
/// ([`decode_orm`]), and its stamp and its key in the shared cache are the
/// two sources' own bytes, whatever writes an image file.
pub(crate) const ORM_PACK: &[u8; 8] = b"og-orm\0\x01";

fn orm_pack(mr: &[u8], ao: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ORM_PACK.len() + 16 + mr.len() + ao.len());
    out.extend_from_slice(ORM_PACK);
    for image in [mr, ao] {
        out.extend_from_slice(&(image.len() as u64).to_le_bytes());
        out.extend_from_slice(image);
    }
    out
}

/// An [`ORM_PACK`]'s two encoded images: metallic-roughness, occlusion.
fn orm_sources(bytes: &[u8]) -> Option<[&[u8]; 2]> {
    let mut rest = bytes.strip_prefix(ORM_PACK)?;
    let mut next = || {
        let (len, after) = rest.split_first_chunk::<8>()?;
        let (image, after) = after.split_at_checked(u64::from_le_bytes(*len) as usize)?;
        rest = after;
        Some(image)
    };
    Some([next()?, next()?])
}

/// An [`ORM_PACK`]'s width and height ([`decode_orm`]'s), from its
/// sources' headers.
pub(crate) fn orm_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let [mr, ao] = orm_sources(bytes)?;
    let (mr, ao) = (texture::size(mr)?, texture::size(ao)?);
    Some((mr.0.max(ao.0).max(1), mr.1.max(ao.1).max(1)))
}

/// Decodes an [`ORM_PACK`]: occlusion in R, roughness in G and metallic in
/// B, at `max(mr.w, ao.w) x max(mr.h, ao.h)`, the smaller map sampled
/// bilinearly.
pub(crate) fn decode_orm(bytes: &[u8]) -> Result<texture::Image, String> {
    let [mr, ao] = orm_sources(bytes).ok_or("a truncated occlusion-roughness-metallic pack")?;
    let mr = texture::decode(mr)?;
    let ao = texture::decode(ao)?;
    let w = mr.width.max(ao.width).max(1);
    let h = mr.height.max(ao.height).max(1);
    let mut rgba = vec![255u8; w as usize * h as usize * 4];
    let same_mr = mr.width == w && mr.height == h;
    let same_ao = ao.width == w && ao.height == h;
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let dst = (y * w + x) as usize * 4;
            rgba[dst] = if same_ao {
                ao.rgba[dst]
            } else {
                sample_channel_bilinear(&ao, u, v, 0)
            };
            if same_mr {
                rgba[dst + 1] = mr.rgba[dst + 1];
                rgba[dst + 2] = mr.rgba[dst + 2];
            } else {
                rgba[dst + 1] = sample_channel_bilinear(&mr, u, v, 1);
                rgba[dst + 2] = sample_channel_bilinear(&mr, u, v, 2);
            }
        }
    }
    Ok(texture::Image {
        width: w,
        height: h,
        rgba,
    })
}

pub fn load(bytes: &[u8], dir: &Path) -> Result<GltfSource, String> {
    load_with(bytes, &|file| {
        std::fs::read(dir.join(file)).map_err(|e| format!("{file}: {e}"))
    })
}

/// Read source dependencies through the cook's input graph, including virtual maps.
pub(crate) fn load_with(
    bytes: &[u8],
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<GltfSource, String> {
    let plain::Plain {
        json,
        buffers,
        external,
        warnings: plain_warnings,
    } = plain::plain_with(bytes, read)?;
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
            gltf::image::Source::Uri { uri, .. } => read_uri(uri, read, &mut external)?,
        });
    }
    // Track images used by any multi-channel slot so standalone occlusion maps
    // only receive 4 bpp `kind::SINGLE` when the image isn't shared with colour,
    // metallic-roughness, normal, emissive, or physical extension slots.
    let mut multi_channel_images = std::collections::BTreeSet::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        if let Some(t) = pbr.base_color_texture() {
            multi_channel_images.insert(t.texture().source().index());
        }
        if let Some(t) = pbr.metallic_roughness_texture() {
            multi_channel_images.insert(t.texture().source().index());
        }
        if let Some(t) = m.normal_texture() {
            multi_channel_images.insert(t.texture().source().index());
        }
        if let Some(t) = m.emissive_texture() {
            multi_channel_images.insert(t.texture().source().index());
        }
        og_render_data::gltf::physical_extensions(&doc, &m, |t, _| {
            multi_channel_images.insert(t.source().index());
        });
    }

    let mut textures: Vec<TextureUse> = Vec::new();
    let mut warnings = plain_warnings;
    let vertex_alpha = vertex_alphas(&doc, &json, &buffers);
    let mut cutouts = Vec::new();
    let mut orm_overrides = Vec::new();
    let mut orm_cache: std::collections::BTreeMap<(usize, usize), usize> =
        std::collections::BTreeMap::new();
    for m in doc.materials() {
        let pbr = m.pbr_metallic_roughness();
        let mat_idx = m.index().unwrap_or(0);
        let cutout = cuts_out(&m, m.index().is_some_and(|i| vertex_alpha[i]));
        cutouts.push(cutout);

        // Auto-pack separate metallicRoughnessTexture + occlusionTexture into
        // one ORM texture when both use TEXCOORD_0 and matching UV transforms.
        let mr_info = pbr
            .metallic_roughness_texture()
            .map(|t| (t.texture(), t.tex_coord()));
        let ao_info = m.occlusion_texture().map(|t| (t.texture(), t.tex_coord()));
        let mut orm_packed: Option<usize> = None;
        if let (Some((mr_tex, 0)), Some((ao_tex, 0))) = (&mr_info, &ao_info) {
            let mr_img = mr_tex.source().index();
            let ao_img = ao_tex.source().index();
            if mr_img != ao_img {
                let mat_json = &json["materials"][mat_idx];
                let mr_tf =
                    ext::transform(&mat_json["pbrMetallicRoughness"]["metallicRoughnessTexture"]).0;
                let ao_tf = ext::transform(&mat_json["occlusionTexture"]).0;
                if mr_tf == ao_tf {
                    let idx = *orm_cache.entry((mr_img, ao_img)).or_insert_with(|| {
                        images.push(orm_pack(&images[mr_img], &images[ao_img]));
                        images.len() - 1
                    });
                    orm_packed = Some(idx);
                    let u = TextureUse {
                        image: idx,
                        kind: kind::LINEAR,
                        cutout: None,
                    };
                    if !textures.contains(&u) {
                        textures.push(u);
                    }
                }
            }
        }
        orm_overrides.push(orm_packed);

        let ao_kind = match &ao_info {
            Some((t, _)) if !multi_channel_images.contains(&t.source().index()) => kind::SINGLE,
            _ => kind::LINEAR,
        };
        let slots = [
            (
                pbr.base_color_texture()
                    .map(|t| (t.texture(), t.tex_coord())),
                kind::COLOR,
                cutout,
            ),
            (
                if orm_packed.is_some() { None } else { mr_info },
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
                if orm_packed.is_some() { None } else { ao_info },
                ao_kind,
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
        orm_overrides,
        warnings,
        skin_relax_notes: Vec::new(),
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
pub(crate) fn msft_lod(
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
pub(crate) struct NodeOrder {
    /// The cooked index of every glTF node.
    pub new_index: Vec<u32>,
    /// In cooked order; names through the caller's string pool.
    pub rows: Vec<NodeRow>,
}

pub(crate) fn node_order(
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

pub(crate) struct Strings(pub Vec<u8>);
impl Strings {
    pub fn add(&mut self, s: &str) -> (u32, u32) {
        let at = self.0.len() as u32;
        self.0.extend_from_slice(s.as_bytes());
        (at, s.len() as u32)
    }
}

pub(crate) static MODEL: KindDef = KindDef {
    name: "model",
    extensions: &["glb", "gltf", "vox", "fbx", "obj"],
    types: &[super::model_text::MODEL_TYPE],
    by_content: false,
    streams: true,
    plan: plan_model,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&MODEL];

fn plan_model<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        options,
        group_of,
        tex_desc,
        mesh_desc,
        budget_desc,
        ..
    } = *cx;
    let PlanState {
        early_errors,
        parsed_refs,
        planned_textures,
        ..
    } = state;
    let g = s.gltf.as_ref().expect("loaded");
    // The art contract's part of each output's stamp: empty
    // (and left out) when it changes nothing here.
    let contract = options
        .mesh
        .conform
        .as_ref()
        .filter(|_| !conform_off(s, options));
    let model_desc = contract
        .map(|c| {
            let mut d = if c.applies(g) || c.density > 0.0 || c.max_size > 0 {
                c.model_desc()
            } else {
                String::new()
            };
            // An effect's mesh is held to no face budget.
            if !d.is_empty() && g.role == conform::ModelRole::Effect {
                d.push_str(" role effect");
            }
            // Its colour textures' kind is in the blob.
            if c.filter == conform::mesh::Filter::Nearest {
                d.push_str(" textures nearest");
            }
            d
        })
        .unwrap_or_default();
    let texture_desc = contract.map(|c| c.texture_desc()).unwrap_or_default();
    let with = |areas: &[Area], parts: &[&[u8]], desc: &str, extra: &[u8]| {
        let mut v: Vec<&[u8]> = parts.to_vec();
        if g.doc.skins().len() > 0 {
            v.push(b"scale-relative-skin-binds-v1");
        }
        if !desc.is_empty() {
            v.push(desc.as_bytes());
            v.push(extra);
        }
        stamp(areas, &v)
    };
    let stream_textures = match model_stream_textures(s) {
        Ok(b) => b,
        Err(e) => {
            early_errors.push((s.rel.clone(), "model", e));
            return;
        }
    };
    let mut ids = Vec::new();
    let grouped = group_of.contains_key(&s.rel);
    for (i, u) in g.textures.iter().enumerate() {
        let image_hash = hash_bytes(&g.images[u.image]).to_le_bytes();
        // One cooked texture per image and use: its kind, and
        // whether it streams its levels (a model's
        // `stream_textures` splits it into a tail and a chain).
        // A model's own unless an art contract makes it depend
        // on the model (a density is per metre of the model) or
        // it is in a streaming group's pack.
        let shared = !grouped && texture_desc.is_empty();
        let id = if shared {
            crate::runtime::gltf_texture_id(
                u64::from_le_bytes(image_hash),
                u.kind,
                stream_textures,
                u.cutout,
            )
        } else {
            format!("{}#tex{i}", s.rel)
        };
        parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .insert(id.clone());
        if shared && !planned_textures.insert(id.clone()) {
            ids.push(id);
            continue;
        }
        // A density is per metre of the model: its hash too.
        let kind = u.kind.to_le_bytes();
        let st = |word: &[u8]| {
            let mut parts: Vec<&[u8]> = vec![b"texture", tex_desc.as_bytes(), &image_hash, &kind];
            // A cut-out's mips are its own, kept at its cut-off
            // (texture.rs `cutout_mips`, `keep_coverage`).
            if let Some(c) = &u.cutout {
                parts.push(b"cutout");
                parts.push(std::slice::from_ref(c));
            }
            if !word.is_empty() {
                parts.push(word);
            }
            with(&[Area::Texture], &parts, &texture_desc, &hash)
        };
        let not_split =
            stream_textures.then(|| not_split(&g.images[u.image], u.kind, group_of.get(&s.rel)));
        let gltf_texture = |split| Box::new(GltfTextureJob(s, u.image, u.kind, u.cutout, split));
        match not_split {
            Some(None) => {
                let mips = st(b"mips");
                outs.push((
                    id.clone(),
                    format::Texture::NAME,
                    st(b"tail"),
                    gltf_texture(Split::Tail(mips)),
                ));
                outs.push((
                    crate::runtime::mips_id(&id),
                    format::Texture::NAME,
                    mips,
                    gltf_texture(Split::Mips),
                ));
            }
            why => outs.push((
                id.clone(),
                format::Texture::NAME,
                st(b""),
                gltf_texture(Split::Whole(why.flatten())),
            )),
        }
        ids.push(id);
    }
    let joined = ids.join("\n");
    // A model written as text is refused past the budgets, so
    // they are among its inputs.
    let st = if ext(&s.rel) == "ogt" {
        with(
            &[Area::Model],
            &[
                b"model",
                NODE_ROWS,
                mesh_desc.as_bytes(),
                &hash,
                joined.as_bytes(),
                budget_desc.as_bytes(),
            ],
            &model_desc,
            &[],
        )
    } else {
        with(
            &[Area::Model],
            &[
                b"model",
                NODE_ROWS,
                mesh_desc.as_bytes(),
                &hash,
                joined.as_bytes(),
            ],
            &model_desc,
            &[],
        )
    };
    // The impostor's card outline is the blob's, from the bake's
    // frames: a new outline rule cooks it again.
    let st = stamp(
        &[Area::Model],
        &[
            b"lod-surface-v6-etc1s-normal",
            &st.to_le_bytes(),
            &impostor::trim_stamp(),
        ],
    );
    // The LOD surfaces a cooked blob of this stamp names: the last cook's
    // in cooked/, else the shared action cache's (a fresh checkout).
    // Without one only the model's cook can name them, and they are
    // planned once it has cooked (`ModelJob::derived`).
    let cached = (|| {
        let bytes = cx
            .old_outputs
            .get(s.rel.as_str())
            .filter(|o| o.stamp == st)
            .and_then(|old| {
                std::fs::read(cx.cooked_dir.join(&old.file))
                    .ok()
                    .filter(|b| hash_bytes(b) == old.hash)
            })
            .or_else(|| {
                crate::cook::pipeline::action_cache::project_output(
                    cx.texture_cache,
                    &s.rel,
                    format::Model::NAME,
                    st,
                    options,
                    cx.project_key,
                )
            })?;
        let blob = og_blob::Blob::parse(&bytes, &s.rel).ok()?;
        if blob.header().schema_version != format::Model::schema_version() {
            return None;
        }
        let refs = blob
            .table::<TextureRefRow>("textures", row_hash::<TextureRefRow>())
            .ok()?;
        let strings = blob.raw("strings").ok()?;
        let prefix = format!("{}#lod", s.rel);
        refs.iter()
            .map(|t| {
                let start = t.id_offset as usize;
                std::str::from_utf8(strings.get(start..start + t.id_len as usize)?)
                    .ok()
                    .map(str::to_owned)
            })
            .collect::<Option<Vec<String>>>()
            .map(|ids| {
                ids.into_iter()
                    .filter(|id| id.starts_with(&prefix))
                    .collect::<Vec<_>>()
            })
    })();
    let prepared = std::sync::Arc::new(PreparedModel {
        source: s,
        ids,
        stamp: st,
        tex_desc,
        derives: cached.is_none(),
    });
    outs.insert(
        0,
        (
            s.rel.clone(),
            format::Model::NAME,
            st,
            Box::new(ModelJob(prepared.clone())),
        ),
    );
    if s.fracture.is_some() {
        outs.push((
            format!("{}#fracture", s.rel),
            format::Fracture::NAME,
            // The asset's layout too: a new og-physics-data fracture version
            // recooks what the runtime would refuse.
            stamp(
                &[Area::Fracture],
                &[
                    b"fracture",
                    &og_physics_data::fracture::ASSET_VERSION.to_le_bytes(),
                    &hash,
                ],
            ),
            Box::new(FractureJob(s)),
        ));
    }
    if group_of.contains_key(&s.rel) {
        let st = with(
            &[Area::Model],
            &[b"proxy", NODE_ROWS, mesh_desc.as_bytes(), &hash],
            &model_desc,
            &[],
        );
        outs.push((
            format!("{}#proxy", s.rel),
            format::Model::NAME,
            st,
            Box::new(ProxyJob(s)),
        ));
    }
    // The impostor's three atlases (`model { impostor true }`),
    // baked from the model's source.
    if let Some(b) = mesh_for(s, options)
        .ok()
        .and_then(|m| m.impostor)
        .map(|b| b.resolved(g))
    {
        let size = [
            b.grid.to_le_bytes(),
            b.frame.to_le_bytes(),
            u32::from(b.below).to_le_bytes(),
        ]
        .concat();
        for (kind, id) in impostor_ids(&s.rel).into_iter().enumerate() {
            // `6`: the atlas's layout version (2, the
            // hemi-octahedron; 3, 16 samples a texel, filtered
            // textures and the material atlas; 4, the leaves' dome
            // normals and the mean normal's length; 6, the sun's
            // share in the material's alpha, 5 an unlanded bake of
            // it), so an atlas baked by an older layout is baked
            // again.
            let st = with(
                &[Area::Model, Area::Texture],
                &[
                    b"impostor",
                    &[6, kind as u8],
                    &size,
                    tex_desc.as_bytes(),
                    &hash,
                ],
                &model_desc,
                &[],
            );
            outs.push((
                id,
                format::Texture::NAME,
                st,
                Box::new(ImpostorJob(s, b, kind as u8)),
            ));
        }
    }
    match anim_settings(s) {
        Ok(a) if anim::wanted(g) || !a.clips.is_empty() => {
            let deps = anim::dependencies(&assets_root(s), &a);
            let motion = contract
                .and_then(|c| c.motion.as_ref())
                .map(|m| m.describe())
                .unwrap_or_default();
            let st = with(
                &[Area::Model, Area::Anim],
                &[
                    b"animation",
                    NODE_ROWS,
                    mesh_desc.as_bytes(),
                    a.describe().as_bytes(),
                    &hash,
                    deps.as_bytes(),
                ],
                &format!("{model_desc}{motion}"),
                &[],
            );
            outs.insert(
                1,
                (
                    format!("{}#anim", s.rel),
                    format::Animation::NAME,
                    st,
                    Box::new(AnimationJob(prepared.clone(), a)),
                ),
            );
        }
        Ok(_) => {}
        Err(e) => early_errors.push((s.rel.clone(), "model", e)),
    }
    // Its LOD surfaces, after its other outputs: where a cook that had to
    // name them itself puts them (`plan::derive`), so the manifest reads
    // the same from either.
    for id in cached.into_iter().flatten() {
        parsed_refs
            .entry(s.rel.clone())
            .or_default()
            .insert(id.clone());
        outs.push(prepared.lod_surface(id));
    }
}

/// A model's one blob: its meshes, materials and skeleton.
struct ModelJob<'a>(std::sync::Arc<PreparedModel<'a>>);

struct PreparedModel<'a> {
    source: &'a Source,
    ids: Vec<String>,
    stamp: u64,
    /// The texture options and encoder, in its LOD surfaces' stamps.
    tex_desc: &'a str,
    /// No cooked blob named its LOD surfaces when it was planned: its
    /// cook does (`ModelJob::derived`).
    derives: bool,
}

/// A model's cook, kept on its source (`Source::model`).
pub(crate) struct ModelOutput {
    blob: Vec<u8>,
    notes: ModelNotes,
    textures: Vec<blob::LodTexture>,
}

impl<'a> PreparedModel<'a> {
    /// The LOD surface `id` of this model, as a planned texture output.
    fn lod_surface(self: &std::sync::Arc<Self>, id: String) -> Out<'a> {
        let stamp = stamp(
            &[Area::Model, Area::Texture],
            &[
                b"lod-surface-v6-etc1s-normal",
                &self.stamp.to_le_bytes(),
                self.tex_desc.as_bytes(),
                id.as_bytes(),
            ],
        );
        (
            id,
            format::Texture::NAME,
            stamp,
            Box::new(LodTextureJob(self.clone())),
        )
    }

    fn get(&self, options: &super::CookOptions) -> Result<&ModelOutput, CookError> {
        self.source
            .model
            .get_or_init(|| {
                let mesh = mesh_for(self.source, options)?;
                let src = self.source.gltf.as_ref().expect("loaded");
                let mut textures = Vec::new();
                let (blob, notes) = blob::cook(
                    src,
                    &self.ids,
                    &mesh,
                    self.stamp,
                    false,
                    &self.source.rel,
                    &mut textures,
                )?;
                Ok(ModelOutput {
                    blob,
                    notes,
                    textures,
                })
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}

impl Job for ModelJob<'_> {
    fn cook(&self, _: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let s = self.0.source;
        let data = self.0.get(ex.options)?;
        let prepared = &data.notes;
        let budgets = ex.budgets;
        let mut notes = prepared.notes.clone();
        // A model written as text past the project's budgets is cooked
        // and flagged: the budgets are reported numbers, never a reason
        // a model is not made (indices are u32; nothing breaks past them).
        if ext(&s.rel) == "ogt" {
            for (what, value, limit, key) in [
                (
                    "LOD 0 triangles",
                    prepared.triangles,
                    budgets.triangles,
                    "triangles",
                ),
                ("vertices", prepared.vertices, budgets.vertices, "vertices"),
            ] {
                if value as u64 > limit {
                    notes.push(format!(
                            "warning: over budget: {what}: {value} is over the {key} budget of {limit} (project.ogt's budgets.{key}); cooked as written"
                        ));
                }
            }
        }
        Ok((data.blob.clone(), notes))
    }

    /// Its LOD surfaces, when its plan had no cooked blob to read them
    /// from.
    fn derived<'a>(&self, ex: &Exec) -> Result<Vec<Out<'a>>, CookError>
    where
        Self: 'a,
    {
        if !self.0.derives {
            return Ok(Vec::new());
        }
        let data = self.0.get(ex.options)?;
        Ok(data
            .textures
            .iter()
            .map(|t| self.0.lod_surface(t.id.clone()))
            .collect())
    }
}

struct LodTextureJob<'a>(std::sync::Arc<PreparedModel<'a>>);

impl Job for LodTextureJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let data = self.0.get(ex.options)?;
        let output = data.textures.iter().find(|t| t.id == p.id).ok_or_else(|| {
            CookError(format!("{}: LOD output no longer in prepared model", p.id))
        })?;
        let mut options = ex.options.texture.clone();
        options.cutout = output.cutout;
        options.data_etc1s = true;
        texture::cook(&output.image, output.kind, &options, p.stamp).map(|bytes| {
            (
                bytes,
                vec![format!(
                    "{}: {}x{} LOD surface",
                    p.id, output.image.width, output.image.height
                )],
            )
        })
    }
}

/// A model's physics proxy: its simplified collision mesh.
struct ProxyJob<'a>(&'a Source);

impl Job for ProxyJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let ProxyJob(s) = self;
        let Exec { options, .. } = ex;
        let g = s.gltf.as_ref().expect("loaded");
        let mesh = mesh_for(s, options)?;
        cook(g, &[], &mesh, p.stamp, true, &s.rel, &mut Vec::new()).map(|(b, mut n)| {
            n.notes.retain(|x| x.contains("triangles"));
            (
                b,
                n.notes.into_iter().map(|x| format!("proxy: {x}")).collect(),
            )
        })
    }
}

/// One of a model's impostor textures: albedo, normal and depth, or material.
struct ImpostorJob<'a>(&'a Source, ImpostorBake, u8);

impl Job for ImpostorJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let ImpostorJob(s, b, kind) = self;
        let Exec { options, .. } = ex;
        let g = s.gltf.as_ref().expect("loaded");
        let baked = g
            .impostor(*b)
            .map_err(|e| CookError(format!("{}: impostor: {e}", s.rel)))?;
        let (img, k, what) = match kind {
            0 => (&baked.albedo, format::kind::COLOR, "albedo"),
            1 => (&baked.normal, format::kind::LINEAR, "normal and depth"),
            _ => (
                &baked.material,
                format::kind::LINEAR,
                "roughness, translucency and metallic",
            ),
        };
        let note = format!(
            "{}: {}x{} impostor {what} ({}×{} frames)",
            p.id, img.width, img.height, b.grid, b.grid
        );
        texture::cook(img, k, &options.texture, p.stamp).map(|t| (t, vec![note]))
    }
}

/// A model's animation clips, with the skin and morph streams of the
/// primitives its model cook made.
struct AnimationJob<'a>(std::sync::Arc<PreparedModel<'a>>, anim::AnimSettings);

impl Job for AnimationJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let AnimationJob(prepared, a) = self;
        let Exec { options, .. } = ex;
        let s = prepared.source;
        let g = s.gltf.as_ref().expect("loaded");
        let mesh = mesh_for(s, options)?;
        let primitives = &prepared.get(options)?.notes.sources;
        anim::cook(g, &s.bytes, &mesh, a, p.stamp, &assets_root(s), primitives)
    }
}

/// One image a model embeds, cooked as a texture.
struct GltfTextureJob<'a>(&'a Source, usize, u32, Option<u8>, Split);

impl Job for GltfTextureJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let GltfTextureJob(s, image, k, cutout, split) = self;
        let Exec { options, .. } = ex;
        let id = p.id.strip_suffix("#mips").unwrap_or(&p.id);
        let cook = |stamp| {
            let g = s.gltf.as_ref().expect("loaded");
            let img = texture::decode(&g.images[*image])
                .map_err(|e| CookError(format!("{id}: image {image}: {e}")))?;
            let mut notes = vec![format!(
                "{id}: {}x{} image {image}{}",
                img.width,
                img.height,
                cutout
                    .map(|c| format!(
                        " (cut-out at alpha {c}: an alpha-tested material's base colour)"
                    ))
                    .unwrap_or_default()
            )];
            let mut t = options.texture.clone();
            t.cutout = *cutout;
            let (img, k) = match options
                .mesh
                .conform
                .as_ref()
                .filter(|_| !conform_off(s, options))
            {
                Some(c) => {
                    let mapping =
                        conform::mesh::TextureMapping::of(g, *image, img.width, img.height);
                    let (img, k, mips, note) =
                        conform::mesh::texture(c, Some(&mapping), img, *k, t.mipmaps);
                    t.mipmaps = mips;
                    notes.extend(note.map(|n| format!("{id}: {n}")));
                    (img, k)
                }
                None => (img, *k),
            };
            if t.cutout.is_some() && t.mipmaps && (t.force_toktx || !cfg!(feature = "basis")) {
                notes.push(format!(
                        "{id}: a cut-out with toktx's own mips (the basis-universal encoder, og-cook's `basis` \
                         feature, makes the ones that keep its coverage)"
                    ));
            }
            texture::cook(&img, k, &t, stamp).map(|b| (b, notes))
        };
        texture::split_texture(s, *image, split, p, cook)
    }

    fn action(&self) -> Action<'_> {
        let GltfTextureJob(s, image, ..) = self;
        Action::Input(
            s.gltf.as_ref().expect("parsed model").images[*image].as_slice(),
            true,
        )
    }
    fn cost(&self, options: &super::CookOptions) -> u64 {
        let GltfTextureJob(s, image, k, ..) = self;
        let bytes = &s.gltf.as_ref().expect("parsed model").images[*image];
        texture::encode_cost(bytes, *k, &options.texture)
    }
}

/// A model's pre-fractured pieces.
struct FractureJob<'a>(&'a Source);

impl Job for FractureJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let FractureJob(s) = self;
        Ok(fracture::cook(s, p.stamp))
    }
}
