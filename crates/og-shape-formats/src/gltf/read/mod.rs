//! Import of glTF 2.0 (`.glb`, and `.gltf` with embedded or external buffers) into parts.

use std::collections::BTreeMap;

use og_shape::math::{DVec2, DVec3, Frame};
use og_shape::mesh::{Bone, Skin, Solid, Weights};
use og_shape::tools::export::{ColorSpace, Material};
use og_shape::tools::image::{ImageInfo, Texture};
use og_shape::tools::import::*;

use crate::import::unique;

use crate::draco::read as draco;
use crate::json::read::{self as jsonread, Value};
use crate::meshopt::read as meshopt;

mod animation;
mod buffers;
mod material;
mod mesh;
mod nodes;
mod skin;
pub(crate) use animation::*;
pub(crate) use buffers::*;
pub(crate) use skin::*;

pub(crate) const SUPPORTED_EXTENSIONS: [&str; 13] = [
    "KHR_materials_clearcoat",
    "KHR_materials_sheen",
    "KHR_materials_transmission",
    "KHR_materials_iridescence",
    "KHR_draco_mesh_compression",
    "EXT_meshopt_compression",
    "KHR_meshopt_compression",
    "KHR_mesh_quantization",
    "KHR_texture_transform",
    "KHR_materials_emissive_strength",
    "KHR_texture_basisu",
    "EXT_texture_webp",
    "KHR_materials_unlit",
];

/// A skin read, and the placed vertices and the scale taken out of its
/// bones, if one was.
pub(crate) type ReadSkin = (Skin, Option<(Vec<DVec3>, SkinScale)>);

/// Per vertex, its `(joint, weight)` pairs as the file has them.
pub(crate) type RawWeights = Vec<Vec<(u32, f64)>>;

pub(crate) struct Doc<'a> {
    pub(crate) json: Value,
    pub(crate) buffers: Vec<Vec<u8>>,
    /// The meshopt-compressed buffer views, decoded.
    pub(crate) decoded: BTreeMap<usize, Vec<u8>>,
    pub(crate) resolve: Resolve<'a>,
}

/// Reads a glTF 2.0 file: binary (GLB) or JSON text. External buffers and
/// images are read through `resolve`; `data:` URIs are decoded.
///
/// Read: triangle primitives (also strips and fans), `POSITION`,
/// `TEXCOORD_0`, indices of every integer width, normalized and quantized
/// attributes (`KHR_mesh_quantization`), sparse accessors, the default
/// scene's node tree (matrices and TRS), and images' sizes. A node's
/// skin becomes the part's `Solid::skin`: its joints as bones, parents
/// first, with world rest frames and the file's inverse bind matrices,
/// and `JOINTS_0`/`WEIGHTS_0` as weights (`docs/research.md` §11.4). The
/// mesh is in its bind pose, at its node's origin (glTF does not use a
/// skinned mesh's node transform). Bones under one uniform scale (an
/// armature scaled 0.01) have it taken out and the mesh is placed where
/// the file's bind pose puts it; [`Imported::skin_scales`] records it. A
/// bone scaled unevenly or sheared, bones scaled differently, or a skin
/// that does not check (an unweighted vertex), is
/// [`ImportError::Unsupported`]. Skipped and listed in
/// [`Imported::skipped`]: points and lines, and bone weights with no
/// skin. Morph targets' `POSITION` offsets become [`Solid::morphs`]
/// (names from `extras.targetNames`, rest weights from the node or the
/// mesh), and a clip's `weights` channel on the mesh's node its
/// `SkinClip::morphs`; their normals and tangents are not read. `COLOR_0` (float or normalised integer, RGB or
/// RGBA, linear as glTF has it) becomes [`Solid::colors`]; a primitive
/// without it is white, as glTF draws it. The material is read as
/// [`ImportedPart::material`] says. Buffer views compressed with
/// `EXT_meshopt_compression` or `KHR_meshopt_compression` are decoded (`docs/research.md` §42). A
/// primitive under `KHR_draco_mesh_compression` is decoded too
/// (`docs/research.md` §44). A required extension the importer does not
/// read is [`ImportError::Unsupported`].
pub fn import_gltf(bytes: &[u8], resolve: Resolve) -> Result<Imported, ImportError> {
    let (json_bytes, bin) = if bytes.starts_with(b"glTF") {
        let u32_at = |i: usize| -> Result<u32, ImportError> {
            bytes
                .get(i..i + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .ok_or_else(|| fmt_err("GLB is cut short"))
        };
        if u32_at(4)? != 2 {
            return Err(ImportError::Unsupported(
                "a GLB container other than version 2".into(),
            ));
        }
        let total = (u32_at(8)? as usize).min(bytes.len());
        let mut at = 12;
        let (mut json, mut bin) = (None, None);
        while at + 8 <= total {
            let len = u32_at(at)? as usize;
            let kind = u32_at(at + 4)?;
            let body = bytes
                .get(at + 8..at + 8 + len)
                .ok_or_else(|| fmt_err("a GLB chunk runs past the end"))?;
            match kind {
                0x4E4F_534A if json.is_none() => json = Some(body),
                0x004E_4942 if bin.is_none() => bin = Some(body.to_vec()),
                _ => {}
            }
            at += 8 + len.div_ceil(4) * 4;
        }
        (json.ok_or_else(|| fmt_err("GLB has no JSON chunk"))?, bin)
    } else {
        (bytes, None)
    };
    let json = jsonread::parse(json_bytes).map_err(|e| fmt_err(format!("glTF JSON: {e}")))?;
    let version = json.get("asset").get("version").str().unwrap_or("");
    if !version.starts_with("2.") {
        return Err(ImportError::Unsupported(format!(
            "glTF version {version:?} (only 2.x)"
        )));
    }
    for e in json.get("extensionsRequired").items() {
        let name = e.str().unwrap_or("");
        if !SUPPORTED_EXTENSIONS.contains(&name) {
            return Err(ImportError::Unsupported(format!(
                "the required glTF extension {name}"
            )));
        }
    }
    let compressed = json
        .get("bufferViews")
        .items()
        .iter()
        .any(|v| !meshopt_of(v).is_null());
    let mut buffers = Vec::new();
    let mut bin = bin;
    for (i, b) in json.get("buffers").items().iter().enumerate() {
        let data = match b.get("uri").str() {
            _ if meshopt_of(b).get("fallback").bool() == Some(true) => {
                buffers.push(Vec::new());
                continue;
            }
            Some(uri) => fetch(uri, resolve)?,
            None => match bin.take().filter(|_| i == 0) {
                Some(bin) => bin,
                // The buffer a file names for its compressed views'
                // decoded bytes, and does not carry.
                None if compressed => {
                    buffers.push(Vec::new());
                    continue;
                }
                None if i == 0 => {
                    return Err(fmt_err("buffer 0 has no URI and there is no BIN chunk"));
                }
                None => return Err(fmt_err(format!("buffer {i} has no URI"))),
            },
        };
        let len = b.get("byteLength").index().unwrap_or(0);
        if data.len() < len {
            return Err(fmt_err(format!(
                "buffer {i} holds {} bytes, fewer than its byteLength {len}",
                data.len()
            )));
        }
        buffers.push(data);
    }
    let mut decoded = BTreeMap::new();
    for (i, v) in json.get("bufferViews").items().iter().enumerate() {
        let c = meshopt_of(v);
        if c.is_null() {
            continue;
        }
        let bad = |why: String| fmt_err(format!("bufferView {i}'s meshopt data: {why}"));
        let off = c.get("byteOffset").index().unwrap_or(0);
        let (Some(len), Some(stride), Some(count)) = (
            c.get("byteLength").index(),
            c.get("byteStride").index(),
            c.get("count").index(),
        ) else {
            return Err(bad("no byteLength, byteStride or count".into()));
        };
        let data = c
            .get("buffer")
            .index()
            .and_then(|b| buffers.get(b))
            .and_then(|b| b.get(off..off.checked_add(len)?))
            .ok_or_else(|| bad("it runs past its buffer".into()))?;
        let mode = match c.get("mode").str() {
            Some("ATTRIBUTES") => meshopt::Mode::Attributes,
            Some("TRIANGLES") => meshopt::Mode::Triangles,
            Some("INDICES") => meshopt::Mode::Indices,
            m => return Err(bad(format!("mode {m:?}"))),
        };
        let filter = match c.get("filter").str() {
            None | Some("NONE") => meshopt::Filter::None,
            Some("OCTAHEDRAL") => meshopt::Filter::Octahedral,
            Some("QUATERNION") => meshopt::Filter::Quaternion,
            Some("EXPONENTIAL") => meshopt::Filter::Exponential,
            Some("COLOR") => meshopt::Filter::Color,
            Some(f) => return Err(bad(format!("filter {f:?}"))),
        };
        if v.get("byteLength").index() != count.checked_mul(stride) {
            return Err(bad("its count and stride are not the view's length".into()));
        }
        decoded.insert(
            i,
            meshopt::decode(data, count, stride, mode, filter).map_err(bad)?,
        );
    }
    let doc = Doc {
        json,
        buffers,
        decoded,
        resolve,
    };
    doc.read()
}

impl Doc<'_> {
    pub(crate) fn read(&self) -> Result<Imported, ImportError> {
        let mut out = Imported::default();
        // Images: sizes, and the bytes for the parts' base-colour maps.
        let mut image_bytes: Vec<Option<Vec<u8>>> = Vec::new();
        for (i, img) in self.json.get("images").items().iter().enumerate() {
            let name = img
                .get("name")
                .str()
                .or(img.get("uri").str().filter(|u| !u.starts_with("data:")))
                .map_or_else(|| format!("image{i}"), str::to_string);
            let bytes = if let Some(uri) = img.get("uri").str() {
                fetch(uri, self.resolve).map_err(|e| e.to_string())
            } else if let Some(bv) = img.get("bufferView").index() {
                self.view(bv)
                    .map(|(b, _)| b.to_vec())
                    .map_err(|e| e.to_string())
            } else {
                Err("no URI or bufferView".to_string())
            };
            out.images.push(match &bytes {
                Ok(b) => ImageInfo::from_bytes(&name, b),
                Err(why) => ImageInfo::missing(&name, why),
            });
            image_bytes.push(bytes.ok());
        }
        // The scene's roots: the default scene, else the first, else every
        // node that is nobody's child.
        let nodes = self.json.get("nodes").items();
        let scene = self.json.get("scene").index().unwrap_or(0);
        let roots: Vec<usize> = match self.json.get("scenes").at(scene).get("nodes").nums() {
            Some(r) => r.iter().map(|&v| v as usize).collect(),
            None => {
                let mut child = vec![false; nodes.len()];
                for n in nodes {
                    for c in n.get("children").items() {
                        if let Some(c) = c.index().filter(|&c| c < nodes.len()) {
                            child[c] = true;
                        }
                    }
                }
                (0..nodes.len()).filter(|&i| !child[i]).collect()
            }
        };
        let mut used = BTreeMap::new();
        // The nodes a clip moves (translation, rotation or scale).
        let parents = self.parents();
        let mut animated = vec![false; self.json.get("nodes").items().len()];
        for anim in self.json.get("animations").items() {
            for ch in anim.get("channels").items() {
                let target = ch.get("target");
                if let (Some(node), Some("translation" | "rotation" | "scale")) =
                    (target.get("node").index(), target.get("path").str())
                    && let Some(slot) = animated.get_mut(node)
                {
                    *slot = true;
                }
            }
        }
        let mut pending = Vec::new();
        let mut stack: Vec<(usize, Affine, usize)> = roots
            .iter()
            .rev()
            .map(|&r| (r, Affine::IDENTITY, 0))
            .collect();
        while let Some((i, above, depth)) = stack.pop() {
            let n = self.json.get("nodes").at(i);
            if n.is_null() || depth > 256 {
                return Err(fmt_err(format!(
                    "node {i} does not exist or the tree loops"
                )));
            }
            let world = above.then(&self.local(n));
            for c in n.get("children").items().iter().rev() {
                let c = c
                    .index()
                    .ok_or_else(|| fmt_err("a child that is not an index"))?;
                stack.push((c, world, depth + 1));
            }
            let Some(m) = n.get("mesh").index() else {
                continue;
            };
            let mesh = self.json.get("meshes").at(m);
            let name = n
                .get("name")
                .str()
                .or(mesh.get("name").str())
                .map_or_else(|| format!("node{i}"), str::to_string);
            let name = unique(&name, &mut used);
            let images = out.images.clone();
            let (mut part, raw) =
                self.part(&name, mesh, &world, &mut out, (&images, &image_bytes))?;
            // A node's weights stand in for its mesh's.
            if let (Some(m), Some(w)) = (&mut part.solid.morphs, n.get("weights").nums()) {
                for (t, w) in m.targets.iter_mut().zip(w) {
                    t.weight = w;
                }
            }
            pending.push((i, world, name, part, raw));
        }
        let mut skinned = vec![false; self.json.get("skins").items().len()];
        for (i, world, name, mut part, raw) in pending {
            let n = self.json.get("nodes").at(i);
            if let Some(slot) = n.get("skin").index().and_then(|k| skinned.get_mut(k)) {
                *slot = true;
            }
            let morph_node =
                |part: &ImportedPart| part.solid.morphs.as_ref().map(|m| (i, m.targets.len()));
            let weighted = raw.iter().any(|r| r.iter().any(|p| p.1 > 0.0));
            match n.get("skin").index() {
                Some(k) => {
                    let ((skin, scaled), space) =
                        self.skin(k, &name, &raw, &part.solid.positions)?;
                    part.clips = self.clips(&space, morph_node(&part))?;
                    if let Some((placed, info)) = scaled {
                        // Where the file's bind pose puts the vertices,
                        // and their morph offsets with them.
                        if let (Some(m), Some(k)) =
                            (&mut part.solid.morphs, affine_inverse(&space.unbind))
                        {
                            for d in m.targets.iter_mut().flat_map(|t| &mut t.deltas) {
                                *d = k.rotate(*d);
                            }
                        }
                        part.solid.positions = placed;
                        out.skin_scales.push(info);
                    }
                    // glTF: a skinned mesh's own node transform is not used;
                    // its bones place it.
                    if part.transform != Affine::IDENTITY {
                        out.skipped.push(format!(
                            "{name}: skinned, so its node transform is not used (glTF)"
                        ));
                    }
                    part.transform = Affine::IDENTITY;
                    part.solid.skin = Some(skin);
                }
                None => {
                    if weighted {
                        out.skipped
                            .push(format!("{name}: bone weights with no skin on its node"));
                    }
                    // A mesh a clip moves by its node (or a node above
                    // it) is carried as a skin of one bone: its vertices
                    // go to where the node puts them at rest, the bone
                    // stands at the origin, and the clip's matrices are
                    // the node's motion from there.
                    let moved = {
                        let mut at = Some(i);
                        let mut steps = 0;
                        let mut found = false;
                        while let Some(k) = at {
                            if animated.get(k).copied().unwrap_or(false) {
                                found = true;
                                break;
                            }
                            at = parents.get(k).copied().flatten();
                            steps += 1;
                            if steps > parents.len() {
                                break;
                            }
                        }
                        found
                    };
                    let rest = affine_inverse(&world).filter(|_| moved);
                    let placed = match rest {
                        Some(_) => og_shape::tools::repair::apply_transform(&part).ok(),
                        None => None,
                    };
                    if let (Some(inverse), Some(placed)) = (rest, placed) {
                        let space = SkinSpace {
                            nodes: vec![i],
                            ibms: vec![inverse],
                            unbind: Affine::IDENTITY,
                        };
                        let clips = self.clips(&space, morph_node(&part))?;
                        if !clips.is_empty() {
                            let vertices = placed.positions.len();
                            part.solid = placed;
                            part.solid.skin = Some(Skin {
                                bones: vec![Bone::new(
                                    &format!("{name}.node"),
                                    None,
                                    Frame::IDENTITY,
                                )],
                                weights: vec![Weights::single(0); vertices],
                            });
                            part.transform = Affine::IDENTITY;
                            part.clips = clips;
                            out.skipped.push(format!(
                                "{name}: moved by its node's animation, so carried as a skin of one bone ({name}.node)"
                            ));
                        }
                    } else if part.solid.morphs.is_some() {
                        // No bones: the clips drive the targets alone.
                        let none = SkinSpace {
                            nodes: Vec::new(),
                            ibms: Vec::new(),
                            unbind: Affine::IDENTITY,
                        };
                        part.clips = self.clips(&none, morph_node(&part))?;
                    }
                }
            }
            out.parts.push(part);
        }
        for (k, used) in skinned.iter().enumerate() {
            if *used {
                continue;
            }
            let name = self
                .json
                .get("skins")
                .at(k)
                .get("name")
                .str()
                .map_or_else(|| format!("skin{k}"), str::to_string);
            let read = self
                .skin(k, &name, &[], &[])
                .and_then(|((skin, _), space)| Ok((skin, self.clips(&space, None)?)));
            match read {
                Ok((skin, clips)) => out.skeletons.push(ImportedSkeleton { name, skin, clips }),
                Err(e) => out
                    .skipped
                    .push(format!("{name}: a skin no mesh uses, not read: {e}")),
            }
        }
        Ok(out)
    }
}
