//! HLOD (docs/design.md §10.6): a scene's distant static models merged
//! into proxies, each one ordinary cooked model drawn in place of the
//! models it stands in for.
//!
//! ```text
//! type scene
//! version 1
//! ---
//! hlod {
//!   size 64        # a leaf's cell, metres along x and z (default 64)
//!   error 0.25     # how far a leaf's proxy may stand off its models, metres
//!                  # (default size / 256); each level above doubles both
//!   texels 1024    # the longest side a proxy's textures may have (default 1024)
//! }
//! entity "house-12" {
//!   model "models/house.glb"
//!   static true
//! }
//! ```
//!
//! **Members.** A scene's `static` model entities (the scene's own and its
//! prefabs'; a sub-scene's are its own scene's). A model keeps drawing
//! itself, and is counted in the cook report with the reason, when it is
//! skinned or morphed, has an impostor (already its distant stand-in), a
//! cut-out or translucent material (leaf cards merge into no surface), a
//! `fracture` block or GPU instances, sets `tint`, `flash`, `variant` or
//! `sway`, is hidden, or is larger than a leaf's cell.
//!
//! **The tree.** A quadtree over x and z of the members' centres: a leaf
//! is a `size` cell holding two members or more, and each level above is
//! the cell twice as wide. A node with one child and nothing else is that
//! child; a member alone in its cell goes to the first node above with
//! company, as a leaf of its own beside that node's children, so a node
//! with children holds no model itself and the leaves show every model.
//! Nodes are laid depth first, a node's members its children's and then
//! its own.
//!
//! **A proxy** is built per node, a leaf's from its members' source
//! geometry in the scene's space and a higher node's from its children's
//! proxies with the members of its own: welded by position, without the
//! parts that lie flush on a larger one (a window on its wall: no edge
//! collapse removes a part), simplified by meshoptimizer (parts smaller
//! than its limit pruned) with the limit lowered until the measured error
//! is within the node's,
//! given normals again (split at creases past 45°), charted by xatlas at
//! one texel per error, and painted from what it stands in for by
//! og-bake's `transfer` (base colour, roughness, metalness, emission).
//! Building a level from the one below keeps a node's work to the size of
//! what it draws, where merging a town's sources for its root would not
//! fit in memory; the error it inherits is added to its own.
//!
//! Its error is measured, never assumed: how far a vertex the
//! simplification removed stands off the proxy's surface (for a part that
//! was pruned whole, no farther than the part's own size), plus its
//! children's, or one texel on the surface, whichever is larger. The
//! runtime draws the proxy while that error projects under a pixel
//! (og-render's `lod`), the rule a model's own levels are drawn by.
//!
//! **Outputs**, beside the scene's blob and in the first group, since a
//! proxy draws while its members are away: `<scene>#hlod` (`og.Hlod`, the
//! tree and the members), and per node `<scene>#hlod-<n>` (an `og.Model`
//! of one level: the tree is its levels of detail) with its
//! textures `<scene>#hlod-<n>-base`, `-surface`, `-normal` and, when a
//! member emits light, `-emissive`. When `texels` passes the mip tail
//! (`og_bundle::runtime::MIP_TAIL`) each is streamed by level, as a model's
//! `stream_textures` are: the output holds its tail, which loads with the
//! scene, and `#mips` its whole chain, read only while its proxy is drawn
//! large enough to want it (og-runtime's `stream::mips`). A node's stamp is its members' models and where
//! they stand, so an edit elsewhere in the scene cooks no proxy again.
//!
//! One thread per node, each in a fixed order: the same scene gives the
//! same bytes.

use super::model::{self, GltfSource};
use super::texture::{self, Image};
use super::vox::Bin;
use super::{
    CookError, CookOptions, Cx, Exec, Job, MeshOptions, Out, PlanState, Planned, Source, mesh_for,
    prefab, stamp,
};
use crate::cook::Split;
use crate::format::{
    self, Area, BlobKind, HlodCellRow, HlodMemberRow, HlodMeta, HlodNodeRow, NONE, kind, row_hash,
};
use crate::hlod::{hlod_id, node_id};
use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
use og_bake::scene::{Image as Texels, Mesh};
use og_bake::transfer::{Mips, Source as Detailed, Surface, Target, TransferSettings, transfer};
use og_blob::BlobWriter;
use og_core::ogt::{Block, Document, Entry};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

/// In every HLOD stamp: a change to what a proxy is cooks them again.
const VERSION: u8 = 3;
/// The levels above the leaves a tree may have (a root 1024 leaves wide).
const MAX_LEVELS: u32 = 10;
/// The most triangles a proxy keeps; past it the simplification stops
/// keeping the surface's shape, and the measured error says by how much.
const MAX_TRIANGLES: usize = 60_000;
/// Faces meeting at more than this angle keep their own normals (cos 45°).
const CREASE_COS: f32 = 0.707;
/// The longest side a member's texture is read at (a proxy's texel is
/// centimetres to metres of surface; no proxy reads a finer level).
const IMAGE_SIDE: u32 = 512;
/// Points asked per texel along each side by the transfer.
const SAMPLES: u32 = 2;

pub(crate) const LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "size",
        "<4..4096 metres>",
        "64",
        "Leaf cell side; doubles at each level.",
        "",
    ),
    crate::grammar::Line::new(
        "error",
        "<metres > 0, <= size / 4>",
        "size / 256",
        "Maximum leaf proxy displacement; zero requests the derived default.",
        "",
    ),
    crate::grammar::Line::new(
        "texels",
        "<64..4096>",
        "1024",
        "Longest texture side in pixels.",
        "",
    ),
];

/// A scene's `hlod` block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub size: f32,
    pub error: f32,
    pub texels: u32,
}

impl Settings {
    fn parse(e: &Entry) -> Result<Settings, String> {
        let line = e.line;
        let empty = Block::new();
        if !e.values.is_empty() {
            return Err(format!(
                "line {line}: hlod takes a block: hlod {{ size 64 }}"
            ));
        }
        let b = e.block.as_ref().unwrap_or(&empty);
        let mut s = Settings {
            size: 64.0,
            error: 0.0,
            texels: 1024,
        };
        for f in &b.entries {
            if !LINES.iter().any(|v| v.key == f.key) {
                return Err(format!("line {}: unknown hlod entry '{}'", f.line, f.key));
            }
            let number = || {
                f.f32(0)
                    .ok()
                    .filter(|v| v.is_finite() && f.values.len() == 1 && f.block.is_none())
                    .ok_or_else(|| format!("line {}: hlod.{} takes one number", f.line, f.key))
            };
            match f.key.as_str() {
                "size" => s.size = number()?,
                "error" => s.error = number()?,
                "texels" => s.texels = number()? as u32,
                k => {
                    return Err(format!(
                        "line {}: '{k}' is not an hlod entry: size, error, texels",
                        f.line
                    ));
                }
            }
        }
        if !(4.0..=4096.0).contains(&s.size) {
            return Err(format!(
                "line {line}: hlod.size {} is a leaf cell's side in metres, 4 to 4096",
                s.size
            ));
        }
        if s.error == 0.0 {
            s.error = s.size / 256.0;
        }
        if !(s.error > 0.0 && s.error <= s.size * 0.25) {
            return Err(format!(
                "line {line}: hlod.error {} is how far a leaf's proxy may stand off its models in metres: above 0, at most a quarter of size ({})",
                s.error,
                s.size * 0.25
            ));
        }
        if !(64..=4096).contains(&s.texels) {
            return Err(format!(
                "line {line}: hlod.texels {} is the longest side of a proxy's textures, 64 to 4096",
                s.texels
            ));
        }
        Ok(s)
    }

    fn bytes(&self) -> Vec<u8> {
        [
            self.size.to_le_bytes(),
            self.error.to_le_bytes(),
            self.texels.to_le_bytes(),
        ]
        .concat()
    }
}

/// One member: a static model entity of the scene.
struct Member<'a> {
    /// Its index in the scene's entities (the scene cook's order).
    entity: u32,
    /// Its matrix in the scene's space.
    matrix: Mat4,
    source: &'a Source,
    /// Its box in the scene's space.
    lo: Vec3,
    hi: Vec3,
    emissive: bool,
}

/// What of a model the plan reads: its box in its own space and whether a
/// material of it emits light.
#[derive(Clone, Copy)]
struct Fit {
    lo: Vec3,
    hi: Vec3,
    emissive: bool,
}

/// Whether a model can be merged, else why it draws itself.
fn fit(s: &Source, options: &CookOptions) -> Result<Fit, &'static str> {
    let Some(g) = s.gltf.as_ref() else {
        return Err("not a model");
    };
    let doc = &g.doc;
    if doc.skins().next().is_some()
        || doc
            .meshes()
            .any(|m| m.primitives().any(|p| p.morph_targets().next().is_some()))
    {
        return Err("skinned or morphed");
    }
    if s.fracture.is_some() {
        return Err("destructible");
    }
    if mesh_for(s, options).is_ok_and(|m| m.impostor.is_some()) {
        return Err("with an impostor");
    }
    if g.json
        .get("extensionsUsed")
        .and_then(|v| v.as_array())
        .is_some_and(|a| a.iter().any(|e| e == "EXT_mesh_gpu_instancing"))
    {
        return Err("GPU-instanced");
    }
    if doc.materials().any(|m| {
        m.alpha_mode() == gltf::material::AlphaMode::Mask
            || m.extension_value("KHR_materials_diffuse_transmission")
                .is_some()
    }) {
        return Err("cut-out or translucent");
    }
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for (p, world) in super::conform::mesh::scene_primitives(g) {
        if p.mode() != gltf::mesh::Mode::Triangles {
            continue;
        }
        let b = p.bounding_box();
        for k in 0..8 {
            let corner = Vec3::new(
                if k & 1 == 0 { b.min[0] } else { b.max[0] },
                if k & 2 == 0 { b.min[1] } else { b.max[1] },
                if k & 4 == 0 { b.min[2] } else { b.max[2] },
            );
            let c = world.transform_point3(corner);
            if c.is_finite() {
                lo = lo.min(c);
                hi = hi.max(c);
            }
        }
    }
    if lo.cmpgt(hi).any() {
        return Err("without triangles");
    }
    Ok(Fit {
        lo,
        hi,
        emissive: doc.materials().any(|m| m.emissive_factor() != [0.0; 3]),
    })
}

/// A `Transform` block's matrix, as the runtime composes it.
fn transform(b: &Block) -> Mat4 {
    let lanes = |key: &str, default: &[f32]| -> Vec<f32> {
        b.get(key)
            .map(|e| {
                e.values
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f32()))
                    .collect::<Vec<f32>>()
            })
            .filter(|v| v.len() == default.len())
            .unwrap_or_else(|| default.to_vec())
    };
    let (p, q, s) = (
        lanes("position", &[0.0; 3]),
        lanes("quaternion", &[0.0, 0.0, 0.0, 1.0]),
        lanes("scale", &[1.0; 3]),
    );
    Mat4::from_scale_rotation_translation(
        Vec3::from_slice(&s),
        Quat::from_slice(&q),
        Vec3::from_slice(&p),
    )
}

/// A cell entity of a scene: its index, its matrix in the scene's space
/// and its sub-scene.
type PlacedCell = (u32, Mat4, String);

/// The scene's members, with how many static models draw themselves by
/// reason, and its cells (entities with a `scene` line and a StreamCell).
/// Entities are counted as the scene cook lays them: depth first, parents
/// first.
fn collect<'a>(
    cx: &Cx<'a>,
    flat: &[prefab::Node],
    settings: &Settings,
) -> (
    Vec<Member<'a>>,
    BTreeMap<&'static str, usize>,
    Vec<PlacedCell>,
) {
    let mut cells = Vec::new();
    let by_rel: BTreeMap<&str, &'a Source> =
        cx.sources.iter().map(|s| (s.rel.as_str(), s)).collect();
    let mut fits: BTreeMap<&str, Result<Fit, &'static str>> = BTreeMap::new();
    let mut members = Vec::new();
    let mut left: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut index = 0u32;
    let mut stack: Vec<(&prefab::Node, Mat4)> =
        flat.iter().rev().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((n, parent)) = stack.pop() {
        let entity = index;
        index += 1;
        let mut local = Mat4::IDENTITY;
        let mut model = None;
        let mut is_static = false;
        let mut own_look = None;
        let (mut sub, mut is_cell) = (None, false);
        let truth = |e: &Entry| e.values.first().and_then(|v| v.as_bool());
        for e in &n.entries {
            match (e.key.as_str(), &e.block) {
                ("scene", None) => {
                    sub = e
                        .string(0)
                        .ok()
                        .map(|s| crate::runtime::asset_id(s).to_string())
                }
                ("StreamCell" | "og.StreamCell", Some(_)) => is_cell = true,
                ("Transform" | "og.Transform", Some(b)) => local = transform(b),
                ("model", None) => {
                    model = e
                        .string(0)
                        .ok()
                        .map(|s| crate::runtime::asset_id(s).to_string())
                }
                ("static", None) => is_static |= truth(e) == Some(true),
                ("Model" | "og.Model", Some(b)) => {
                    for f in &b.entries {
                        match f.key.as_str() {
                            "static" => is_static |= truth(f) == Some(true),
                            "visible" if truth(f) == Some(false) => own_look = Some("hidden"),
                            "tint" | "flash" | "variant" | "sway" => {
                                own_look = Some("with a tint, flash, variant or sway of its own")
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        let world = parent * local;
        stack.extend(n.children.iter().rev().map(|c| (c, world)));
        if let (Some(sub), true) = (sub, is_cell) {
            cells.push((entity, world, sub));
        }
        let (Some(model), true) = (model, is_static) else {
            continue;
        };
        let Some((&rel, &source)) = by_rel.get_key_value(model.as_str()) else {
            continue;
        };
        let fit = *fits.entry(rel).or_insert_with(|| fit(source, cx.options));
        let fit = match (own_look, fit) {
            (Some(why), _) | (None, Err(why)) => {
                *left.entry(why).or_default() += 1;
                continue;
            }
            (None, Ok(f)) => f,
        };
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for k in 0..8 {
            let c = world.transform_point3(Vec3::new(
                if k & 1 == 0 { fit.lo.x } else { fit.hi.x },
                if k & 2 == 0 { fit.lo.y } else { fit.hi.y },
                if k & 4 == 0 { fit.lo.z } else { fit.hi.z },
            ));
            lo = lo.min(c);
            hi = hi.max(c);
        }
        if !(lo.is_finite() && hi.is_finite()) {
            *left.entry("placed nowhere finite").or_default() += 1;
            continue;
        }
        if (hi - lo).length() * 0.5 > settings.size {
            *left.entry("larger than a cell").or_default() += 1;
            continue;
        }
        members.push(Member {
            entity,
            matrix: world,
            source,
            lo,
            hi,
            emissive: fit.emissive,
        });
    }
    (members, left, cells)
}

/// A node as the tree is grown: its children, and the members of its own.
struct Grown {
    level: u32,
    children: Vec<Grown>,
    own: Vec<usize>,
}

/// The node over `members` (indices), all in one cell of `level`; `Err`:
/// too few for a node, handed up.
fn grow(
    level: u32,
    members: Vec<usize>,
    cell: &dyn Fn(usize, u32) -> (i32, i32),
    wrap: bool,
) -> Result<Grown, Vec<usize>> {
    if level == 0 {
        return if members.len() >= 2 {
            Ok(Grown {
                level,
                children: Vec::new(),
                own: members,
            })
        } else {
            Err(members)
        };
    }
    let mut cells: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
    for m in members {
        cells.entry(cell(m, level - 1)).or_default().push(m);
    }
    let (mut children, mut own) = (Vec::new(), Vec::new());
    for (_, m) in cells {
        match grow(level - 1, m, cell, wrap) {
            Ok(g) => children.push(g),
            Err(m) => own.extend(m),
        }
    }
    if children.len() == 1 && own.is_empty() {
        return Ok(children.pop().expect("one child"));
    }
    if children.is_empty() && own.len() < 2 {
        return Err(own);
    }
    own.sort_unstable();
    // Beside children each is a leaf of its own, so every model is in a
    // node without children: the finest proxies of a tree show all of it.
    if wrap && !children.is_empty() {
        children.extend(own.drain(..).map(|m| Grown {
            level: 0,
            children: Vec::new(),
            own: vec![m],
        }));
    }
    Ok(Grown {
        level,
        children,
        own,
    })
}

/// A node of the planned tree.
struct NodePlan {
    parent: Option<usize>,
    end: usize,
    /// Its members, its descendants' first: a range of the members.
    first: usize,
    count: usize,
    /// Of them, its own: the range's end.
    own: usize,
    level: u32,
    children: Vec<usize>,
    /// Over cells: the cells' roots it stands for (of `Shared::cells`).
    cells: Vec<usize>,
    center: Vec3,
    radius: f32,
    emissive: bool,
    stamp: u64,
}

fn lay(
    g: Grown,
    parent: Option<usize>,
    nodes: &mut Vec<NodePlan>,
    order: &mut Vec<usize>,
) -> usize {
    let me = nodes.len();
    nodes.push(NodePlan {
        parent,
        end: 0,
        first: order.len(),
        count: 0,
        own: g.own.len(),
        level: g.level,
        children: Vec::new(),
        cells: Vec::new(),
        center: Vec3::ZERO,
        radius: 0.0,
        emissive: false,
        stamp: 0,
    });
    let children: Vec<usize> = g
        .children
        .into_iter()
        .map(|c| lay(c, Some(me), nodes, order))
        .collect();
    order.extend(g.own);
    let end = nodes.len();
    let n = &mut nodes[me];
    n.children = children;
    n.end = end;
    n.count = order.len() - n.first;
    me
}

/// Lays a node over cells (`g.own` cell indices) and its children after
/// the scene's own nodes; it holds no member (`first`: past them all).
fn lay_cells(g: Grown, parent: Option<usize>, nodes: &mut Vec<NodePlan>, first: usize) -> usize {
    let me = nodes.len();
    nodes.push(NodePlan {
        parent,
        end: 0,
        first,
        count: 0,
        own: 0,
        level: g.level,
        children: Vec::new(),
        cells: g.own,
        center: Vec3::ZERO,
        radius: 0.0,
        emissive: false,
        stamp: 0,
    });
    let children: Vec<usize> = g
        .children
        .into_iter()
        .map(|c| lay_cells(c, Some(me), nodes, first))
        .collect();
    let end = nodes.len();
    nodes[me].children = children;
    nodes[me].end = end;
    me
}

/// A model's default scene as the transfer reads it, in the model's space.
struct Flat {
    meshes: Vec<Mesh>,
    /// Per glTF material, then the default one; a map is a glTF image.
    surfaces: Vec<Surface>,
    triangles: usize,
}

fn flatten(g: &GltfSource) -> Result<Flat, String> {
    let scene = g
        .doc
        .default_scene()
        .or_else(|| g.doc.scenes().next())
        .ok_or("the model has no scene")?;
    let image = |t: gltf::texture::Texture| t.source().index();
    // `KHR_texture_transform` on one of a material's maps (`texture`, its
    // JSON path under the material), as a matrix of uvs.
    let uv_of = |material: usize, texture: &str| -> Mat3 {
        let Some(t) = g.json.pointer(&format!(
            "/materials/{material}/{texture}/extensions/KHR_texture_transform"
        )) else {
            return Mat3::IDENTITY;
        };
        let pair = |key: &str, d: f32| {
            let lane = |i: usize| {
                t.get(key)
                    .and_then(|v| v.get(i))
                    .and_then(|v| v.as_f64())
                    .map_or(d, |x| x as f32)
            };
            Vec2::new(lane(0), lane(1))
        };
        let [r0, r1] = og_render_data::UvTransform {
            offset: pair("offset", 0.0),
            scale: pair("scale", 1.0),
            rotation: t.get("rotation").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        }
        .matrix();
        Mat3::from_cols(
            Vec3::new(r0[0], r1[0], 0.0),
            Vec3::new(r0[1], r1[1], 0.0),
            Vec3::new(r0[2], r1[2], 1.0),
        )
    };
    // The base colour's transform is applied to the uvs read through it;
    // the normal map's is then its own after that one's inverse.
    let base_uv = |material: usize| uv_of(material, "pbrMetallicRoughness/baseColorTexture");
    let mut surfaces: Vec<Surface> = g
        .doc
        .materials()
        .enumerate()
        .map(|(i, m)| {
            let pbr = m.pbr_metallic_roughness();
            let b = pbr.base_color_factor();
            let e = Vec3::from(m.emissive_factor()) * m.emissive_strength().unwrap_or(1.0);
            let base = base_uv(i);
            let mut uv_transforms = [Mat3::IDENTITY; 5];
            if base.determinant().abs() > 1e-12 {
                uv_transforms[2] = uv_of(i, "normalTexture") * base.inverse();
            }
            Surface {
                base_color: Vec3::new(b[0], b[1], b[2]),
                base_map: pbr.base_color_texture().map(|t| image(t.texture())),
                roughness: pbr.roughness_factor(),
                metallic: pbr.metallic_factor(),
                metallic_roughness_map: pbr
                    .metallic_roughness_texture()
                    .map(|t| image(t.texture())),
                emissive: e,
                emissive_map: m.emissive_texture().map(|t| image(t.texture())),
                normal_map: m.normal_texture().map(|t| image(t.texture())),
                normal_scale: m.normal_texture().map_or(1.0, |t| t.scale()),
                uv_transforms,
                ..Default::default()
            }
        })
        .collect();
    // glTF's default material, as the model cook draws it.
    let default = surfaces.len();
    surfaces.push(Surface {
        base_color: Vec3::splat(0.8),
        roughness: 0.6,
        ..Default::default()
    });
    let mut out = Flat {
        meshes: Vec::new(),
        surfaces,
        triangles: 0,
    };
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        // A node hidden at rest is not drawn, nor what hangs under it.
        if node
            .extension_value("KHR_node_visibility")
            .and_then(|v| v.get("visible")?.as_bool())
            == Some(false)
        {
            continue;
        }
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        stack.extend(node.children().map(|c| (c, world)));
        let Some(mesh) = node.mesh() else { continue };
        for prim in mesh.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = prim.reader(|b| Some(&g.buffers[b.index()].0[..]));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let positions: Vec<Vec3> = positions.map(Vec3::from).collect();
            let surface = prim.material().index().unwrap_or(default);
            let uv = prim.material().index().map(base_uv);
            let uvs: Vec<Vec2> = reader
                .read_tex_coords(0)
                .map(|t| {
                    t.into_f32()
                        .map(|t| {
                            let t = Vec2::from(t);
                            uv.map_or(t, |m| (m * t.extend(1.0)).truncate())
                        })
                        .collect()
                })
                .unwrap_or_default();
            let colors: Vec<Vec3> = reader
                .read_colors(0)
                .map(|c| c.into_rgb_f32().map(Vec3::from).collect())
                .unwrap_or_default();
            let normals: Vec<Vec3> = reader
                .read_normals()
                .map(|n| n.map(Vec3::from).collect())
                .unwrap_or_default();
            let tangents: Vec<Vec4> = reader
                .read_tangents()
                .map(|t| t.map(Vec4::from).collect())
                .unwrap_or_default();
            let read: Vec<u32> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());
            let mut indices = Vec::with_capacity(read.len());
            for t in read.as_chunks::<3>().0 {
                if t.iter().any(|&i| i as usize >= positions.len()) {
                    return Err(format!(
                        "mesh '{}': an index is past its vertices",
                        mesh.name().unwrap_or("")
                    ));
                }
                indices.push(*t);
            }
            let count = positions.len();
            let local = Mesh {
                positions,
                normals: if normals.len() == count {
                    normals
                } else {
                    Vec::new()
                },
                tangents: if tangents.len() == count {
                    tangents
                } else {
                    Vec::new()
                },
                uvs: if uvs.len() == count { uvs } else { Vec::new() },
                colors: if colors.len() == count {
                    colors
                } else {
                    Vec::new()
                },
                indices: indices.into_iter().flatten().collect(),
                material: surface,
                lightmapped: false,
                ..Default::default()
            };
            out.triangles += local.indices.len() / 3;
            out.meshes.push(placed(&local, world));
        }
    }
    Ok(out)
}

/// `mesh` placed by `m`: its positions moved, its normals and tangents
/// turned (a normal by the inverse transpose, as the renderer turns it),
/// and under a mirroring `m` its triangles' winding and its tangents'
/// handedness reversed, so it faces and is mapped as before.
fn placed(mesh: &Mesh, m: Mat4) -> Mesh {
    let mirrored = m.determinant() < 0.0;
    let linear = Mat3::from_mat4(m);
    let normal = linear.inverse().transpose();
    let mut indices = mesh.indices.clone();
    if mirrored {
        for t in indices.as_chunks_mut::<3>().0 {
            t.swap(1, 2);
        }
    }
    Mesh {
        positions: mesh
            .positions
            .iter()
            .map(|&p| m.transform_point3(p))
            .collect(),
        normals: mesh
            .normals
            .iter()
            .map(|&n| (normal * n).normalize_or_zero())
            .collect(),
        tangents: mesh
            .tangents
            .iter()
            .map(|t| {
                (linear * t.truncate())
                    .normalize_or_zero()
                    .extend(if mirrored { -t.w } else { t.w })
            })
            .collect(),
        indices,
        ..mesh.clone()
    }
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// RGBA8 as the transfer's texels: colour decoded from sRGB when `srgb`.
fn texels(width: u32, height: u32, rgba: &[u8], srgb: bool) -> Texels {
    let table: Vec<f32> = (0..256)
        .map(|i| {
            let v = i as f32 / 255.0;
            if srgb { srgb_to_linear(v) } else { v }
        })
        .collect();
    Texels {
        width,
        height,
        texels: rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| {
                [
                    table[p[0] as usize],
                    table[p[1] as usize],
                    table[p[2] as usize],
                    p[3] as f32 / 255.0,
                ]
            })
            .collect(),
    }
}

/// An image no longer than [`IMAGE_SIDE`] a side: box filtered down by a
/// whole factor.
fn bounded(img: Image) -> Image {
    let f = img.width.max(img.height).div_ceil(IMAGE_SIDE).max(1);
    if f == 1 {
        return img;
    }
    let (w, h) = ((img.width / f).max(1), (img.height / f).max(1));
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0u32; 4];
            let mut n = 0;
            for yy in (y * f)..((y + 1) * f).min(img.height) {
                for xx in (x * f)..((x + 1) * f).min(img.width) {
                    let at = ((yy * img.width + xx) * 4) as usize;
                    for k in 0..4 {
                        sum[k] += img.rgba[at + k] as u32;
                    }
                    n += 1;
                }
            }
            rgba.extend(sum.map(|s| (s / n.max(1)) as u8));
        }
    }
    Image {
        width: w,
        height: h,
        rgba,
    }
}

/// A built proxy: its mesh in the scene's space and its pages.
struct Proxy {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    /// MikkTSpace on the page, with the bitangent up the image.
    tangents: Vec<Vec4>,
    /// On the page, 0 to 1, v down.
    uvs: Vec<Vec2>,
    indices: Vec<u32>,
    width: u32,
    height: u32,
    /// sRGB base colour, RGBA8.
    base: Vec<u8>,
    /// glTF's metallic-roughness layout: roughness in green, metalness in
    /// blue.
    surface: Vec<u8>,
    /// Tangent-space normals: what its models' surfaces and normal maps
    /// show that its simpler surface does not.
    normal: Vec<u8>,
    /// sRGB emission over `emissive_scale`.
    emissive: Vec<u8>,
    emissive_scale: f32,
    /// How far its surface stands off its members' (its children's too).
    geometry: f32,
    /// The length of surface one texel covers.
    texel: f32,
    /// The triangles of the members it stands in for.
    source_triangles: usize,
}

impl Proxy {
    fn error(&self) -> f32 {
        self.geometry.max(self.texel)
    }
}

/// A node's proxy, built once by whichever job asks first.
type Built = OnceLock<Result<Arc<Proxy>, String>>;
type ProxyCell = Arc<Built>;

/// The proxy of the node whose stamp is `stamp`, as every plan of this
/// process shares it while one holds it.
fn registered(stamp: u64) -> ProxyCell {
    static BUILT: Mutex<BTreeMap<u64, std::sync::Weak<Built>>> = Mutex::new(BTreeMap::new());
    let mut built = BUILT.lock().expect("no poisoning");
    if let Some(cell) = built.get(&stamp).and_then(std::sync::Weak::upgrade) {
        return cell;
    }
    built.retain(|_, w| w.strong_count() > 0);
    let cell = ProxyCell::default();
    built.insert(stamp, Arc::downgrade(&cell));
    cell
}

/// A cell under a node over cells: the cell entity, its matrix in the
/// scene's space, its sub-scene's tree (an index of `Shared::subs`) and
/// the root of it the node stands in for.
#[derive(Clone, Copy)]
struct CellRef {
    entity: u32,
    matrix: Mat4,
    sub: usize,
    root: usize,
}

/// A model's flattened source, made once by whichever node asks first.
type FlatCell = Arc<OnceLock<Result<Arc<Flat>, String>>>;
/// A model's image by its asset, its index and whether it is sRGB.
type ImageKey<'a> = (&'a str, usize, bool);

/// What a scene's HLOD jobs share: the plan, and each node's proxy built
/// once by the first job that needs it.
struct Shared<'a> {
    scene: String,
    /// The scene: its streamed pages' whole chains are cached on it.
    source: &'a Source,
    settings: Settings,
    members: Vec<Member<'a>>,
    nodes: Vec<NodePlan>,
    left: BTreeMap<&'static str, usize>,
    /// Per node, its proxy: shared by stamp with every other plan that
    /// builds the same node ([`registered`]), so a cell's root is built
    /// once for its own scene and the scene over its cells.
    proxies: Vec<ProxyCell>,
    /// The cells its nodes over cells stand for, and their sub-scenes'
    /// trees (planned without outputs of their own).
    cells: Vec<CellRef>,
    subs: Vec<Arc<Shared<'a>>>,
    flats: Mutex<BTreeMap<&'a str, FlatCell>>,
    images: Mutex<BTreeMap<ImageKey<'a>, Arc<Mips>>>,
}

impl<'a> Shared<'a> {
    fn proxy(&self, n: usize) -> Result<Arc<Proxy>, String> {
        self.proxies[n]
            .get_or_init(|| {
                self.build(n)
                    .map(Arc::new)
                    .map_err(|e| format!("hlod node {n}: {e}"))
            })
            .clone()
    }

    fn flat(&self, s: &'a Source) -> Result<Arc<Flat>, String> {
        let cell = self
            .flats
            .lock()
            .expect("no poisoning")
            .entry(s.rel.as_str())
            .or_default()
            .clone();
        cell.get_or_init(|| {
            flatten(s.gltf.as_ref().ok_or("not a model")?)
                .map(Arc::new)
                .map_err(|e| format!("{}: {e}", s.rel))
        })
        .clone()
    }

    fn image(&self, s: &'a Source, image: usize, srgb: bool) -> Result<Arc<Mips>, String> {
        let mut images = self.images.lock().expect("no poisoning");
        if let Some(m) = images.get(&(s.rel.as_str(), image, srgb)) {
            return Ok(m.clone());
        }
        let g = s.gltf.as_ref().ok_or("not a model")?;
        let bytes = g
            .images
            .get(image)
            .ok_or_else(|| format!("{}: image {image} is not in the model", s.rel))?;
        let img =
            bounded(texture::decode(bytes).map_err(|e| format!("{}: image {image}: {e}", s.rel))?);
        let mips = Arc::new(Mips::new(texels(img.width, img.height, &img.rgba, srgb)));
        images.insert((s.rel.as_str(), image, srgb), mips.clone());
        Ok(mips)
    }

    /// The error node `n` is built within.
    fn budget(&self, n: usize) -> f32 {
        self.settings.error * (1u64 << self.nodes[n].level.min(40)) as f32
    }

    fn build(&self, n: usize) -> Result<Proxy, String> {
        let node = &self.nodes[n];
        let budget = self.budget(n);
        // What the proxy stands in for: its children's proxies and its own
        // members' models.
        let mut meshes: Vec<Mesh> = Vec::new();
        let mut surfaces: Vec<Surface> = Vec::new();
        let mut images: Vec<Arc<Mips>> = Vec::new();
        let mut inherited = 0.0f32;
        let mut source_triangles = 0;
        // The children's proxies, then the roots of the cells it is over,
        // each at its cell.
        let below: Vec<(Arc<Proxy>, Mat4)> = node
            .children
            .iter()
            .map(|&c| Ok((self.proxy(c)?, Mat4::IDENTITY)))
            .chain(node.cells.iter().map(|&c| {
                let cell = &self.cells[c];
                Ok((self.subs[cell.sub].proxy(cell.root)?, cell.matrix))
            }))
            .collect::<Result<_, String>>()?;
        for (p, placement) in &below {
            inherited = inherited.max(p.geometry);
            source_triangles += p.source_triangles;
            let at = images.len();
            images.push(Arc::new(Mips::new(texels(
                p.width, p.height, &p.base, true,
            ))));
            images.push(Arc::new(Mips::new(texels(
                p.width, p.height, &p.surface, false,
            ))));
            images.push(Arc::new(Mips::new(texels(
                p.width, p.height, &p.normal, false,
            ))));
            let emits = p.emissive_scale > 0.0;
            if emits {
                images.push(Arc::new(Mips::new(texels(
                    p.width,
                    p.height,
                    &p.emissive,
                    true,
                ))));
            }
            surfaces.push(Surface {
                metallic: 1.0,
                metallic_roughness_map: Some(at + 1),
                base_map: Some(at),
                normal_map: Some(at + 2),
                emissive: Vec3::splat(p.emissive_scale),
                emissive_map: emits.then_some(at + 3),
                ..Default::default()
            });
            let child = Mesh {
                positions: p.positions.clone(),
                normals: p.normals.clone(),
                tangents: p.tangents.clone(),
                uvs: p.uvs.clone(),
                indices: p.indices.clone(),
                material: surfaces.len() - 1,
                ..Default::default()
            };
            meshes.push(if *placement == Mat4::IDENTITY {
                child
            } else {
                placed(&child, *placement)
            });
        }
        let own = node.first + node.count - node.own..node.first + node.count;
        let mut image_at: BTreeMap<(&str, usize, bool), usize> = BTreeMap::new();
        for m in &self.members[own] {
            let flat = self.flat(m.source)?;
            source_triangles += flat.triangles;
            let base = surfaces.len();
            for s in &flat.surfaces {
                let mut map = |image: Option<usize>, srgb: bool| -> Result<Option<usize>, String> {
                    let Some(image) = image else { return Ok(None) };
                    let key = (m.source.rel.as_str(), image, srgb);
                    if let Some(&at) = image_at.get(&key) {
                        return Ok(Some(at));
                    }
                    images.push(self.image(m.source, image, srgb)?);
                    image_at.insert(key, images.len() - 1);
                    Ok(Some(images.len() - 1))
                };
                surfaces.push(Surface {
                    base_map: map(s.base_map, true)?,
                    metallic_roughness_map: map(s.metallic_roughness_map, false)?,
                    emissive_map: map(s.emissive_map, true)?,
                    normal_map: map(s.normal_map, false)?,
                    ..s.clone()
                });
            }
            for fm in &flat.meshes {
                meshes.push(Mesh {
                    material: base + fm.material,
                    ..placed(fm, m.matrix)
                });
            }
        }

        // One mesh, welded by position: the proxy is charted and painted
        // anew, so no seam of its sources' uvs or normals is kept.
        let mut pos: Vec<[f32; 3]> = Vec::new();
        let mut all: Vec<u32> = Vec::new();
        for m in &meshes {
            let base = pos.len() as u32;
            pos.extend(m.positions.iter().map(|p| p.to_array()));
            all.extend(
                m.indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|t| t.iter().all(|&i| m.positions[i as usize].is_finite()))
                    .flat_map(|t| t.map(|i| i + base)),
            );
        }
        if all.is_empty() {
            return Err("its members have no triangles".into());
        }
        let (count, remap) = meshopt::generate_vertex_remap(&pos, Some(&all));
        let mut welded = vec![[0.0f32; 3]; count];
        for (old, &new) in remap.iter().enumerate() {
            if new != u32::MAX {
                welded[new as usize] = pos[old];
            }
        }
        let full: Vec<u32> = all
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| t.map(|i| remap[i as usize]))
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .flatten()
            .collect();
        if full.is_empty() {
            return Err("its members' triangles are all degenerate".into());
        }
        drop((pos, all));

        // Simplified within what is left of the budget after the children's.
        let within = (budget - inherited).max(budget * 0.25);
        // First without the parts lying flush on a larger one (a window's
        // frame on its wall, a door, a sign), within half of it: no edge
        // collapse removes a part, and each would be charted on its own.
        // What they showed is painted on the surface behind them.
        let outer = without_flush(&welded, &full, within * 0.5);

        // The simplifier's limit is its own measure (a part it prunes as
        // smaller than the limit can stand farther off what is left than
        // that), so the result is measured and the limit lowered until the
        // proxy stands within the node's error. Members all smaller than
        // the limit would be pruned to nothing, a hole where they stood:
        // lowered for that too, and past the last try they are kept as
        // they are (few triangles, or they would not be so small).
        let (simplified, measured) = {
            let adapter = meshopt::VertexDataAdapter::new(bytemuck::cast_slice(&welded), 12, 0)
                .map_err(|e| e.to_string())?;
            let options = meshopt::SimplifyOptions::ErrorAbsolute | meshopt::SimplifyOptions::Prune;
            let mut limit = within;
            let mut best: Option<(Vec<u32>, f32)> = None;
            for _ in 0..8 {
                let s = meshopt::simplify(&outer, &adapter, 0, limit, options, None);
                limit *= 0.6;
                if s.len() < 3 {
                    continue;
                }
                let e = removed_error(&welded, &full, &s);
                let fits = e <= within;
                if fits || best.as_ref().is_none_or(|b| e < b.1) {
                    best = Some((s, e));
                }
                if fits {
                    break;
                }
            }
            let (mut s, mut e) =
                best.unwrap_or_else(|| (outer.clone(), removed_error(&welded, &full, &outer)));
            if s.len() / 3 > MAX_TRIANGLES {
                let sloppy = meshopt::simplify_sloppy(&s, &adapter, MAX_TRIANGLES * 3, 1.0, None);
                if sloppy.len() >= 3 {
                    e = removed_error(&welded, &full, &sloppy);
                    s = sloppy;
                }
            }
            (s, e)
        };
        let geometry = inherited + measured;

        // The kept vertices, with normals split at creases.
        let (positions, normals, indices) = creased(&welded, &simplified);
        drop((welded, full, outer, simplified));

        // Charted at one texel per error, coarser when the page would pass
        // the largest side.
        let cap = self.settings.texels;
        let unwrap_mesh = og_bake::UnwrapMesh {
            positions: positions.iter().map(|p| p.to_array()).collect(),
            normals: normals.iter().map(|n| n.to_array()).collect(),
            indices: indices.clone(),
        };
        let mut density = 1.0 / budget;
        let mut atlas = None;
        for _ in 0..8 {
            let a = og_bake::unwrap(
                std::slice::from_ref(&unwrap_mesh),
                &og_bake::PackSettings {
                    texels_per_unit: density,
                    max_size: cap,
                    padding: 2,
                    ..Default::default()
                },
            )
            .map_err(|e| format!("charting: {e}"))?;
            if a.pages <= 1 && a.width <= cap && a.height <= cap {
                atlas = Some(a);
                break;
            }
            let over = if a.pages > 1 {
                (a.pages as f32).sqrt()
            } else {
                a.width.max(a.height) as f32 / cap as f32
            };
            density /= over.max(1.0) * 1.1;
        }
        let atlas = atlas.ok_or_else(|| {
            format!("its charts do not fit a {cap} texel page at any density tried")
        })?;
        let texel = 1.0 / density;
        let charted = &atlas.meshes[0];
        let (width, height) = (
            atlas.width.max(1).next_multiple_of(4),
            atlas.height.max(1).next_multiple_of(4),
        );
        // Given MikkTSpace tangents on the page, the frame its normal page
        // is drawn in and the renderer reads it in (the model cook keeps
        // them).
        let size = Vec2::new(width as f32, height as f32);
        let mut corners: Vec<model::FVertex> = charted
            .indices
            .iter()
            .map(|&i| {
                let v = &charted.vertices[i as usize];
                model::FVertex {
                    p: positions[v.xref as usize].to_array(),
                    n: normals[v.xref as usize].to_array(),
                    uv: (Vec2::from(v.uv) / size).to_array(),
                    c: [1.0; 4],
                    t: [0.0; 4],
                    key: [0; 2],
                }
            })
            .collect();
        if !mikktspace::generate_tangents(&mut model::Tangents { v: &mut corners }) {
            return Err("its page has no MikkTSpace tangent frame".into());
        }
        let flat: Vec<u32> = (0..corners.len() as u32).collect();
        let (vertices, indices) = model::weld(&corners, &flat, &mut flat.clone());
        let positions: Vec<Vec3> = vertices.iter().map(|v| Vec3::from(v.p)).collect();
        let normals: Vec<Vec3> = vertices.iter().map(|v| Vec3::from(v.n)).collect();
        let tangents: Vec<Vec4> = vertices.iter().map(|v| Vec4::from(v.t)).collect();
        let uvs: Vec<Vec2> = vertices.iter().map(|v| Vec2::from(v.uv)).collect();
        let page_uvs: Vec<Vec2> = uvs.iter().map(|&uv| uv * size).collect();

        // Painted from what it stands in for.
        let refs: Vec<&Mips> = images.iter().map(Arc::as_ref).collect();
        let pages = transfer(
            Detailed {
                meshes: &meshes,
                surfaces: &surfaces,
                images: &refs,
            },
            Target {
                positions: &positions,
                normals: &normals,
                tangents: &tangents,
                uvs: &page_uvs,
                indices: &indices,
            },
            &TransferSettings {
                width,
                height,
                reach: measured.max(texel * 0.5) * 1.25 + 1e-3,
                samples: SAMPLES,
                texel,
            },
        );
        let emissive_scale = if node.emissive {
            pages
                .emissive
                .iter()
                .flat_map(|t| &t[..3])
                .fold(0.0f32, |a, &b| a.max(b))
        } else {
            0.0
        };
        let scale = 1.0 / emissive_scale.max(1e-6);
        Ok(Proxy {
            uvs,
            positions,
            normals,
            tangents,
            indices,
            width,
            height,
            base: pages
                .base
                .iter()
                .flat_map(|t| {
                    [
                        byte(linear_to_srgb(t[0])),
                        byte(linear_to_srgb(t[1])),
                        byte(linear_to_srgb(t[2])),
                        255,
                    ]
                })
                .collect(),
            surface: pages
                .surface
                .iter()
                .flat_map(|t| [255, byte(t[0]), byte(t[1]), 255])
                .collect(),
            normal: pages
                .normal
                .iter()
                .flat_map(|t| [byte(t[0]), byte(t[1]), byte(t[2]), 255])
                .collect(),
            emissive: pages
                .emissive
                .iter()
                .flat_map(|t| {
                    [
                        byte(linear_to_srgb(t[0] * scale)),
                        byte(linear_to_srgb(t[1] * scale)),
                        byte(linear_to_srgb(t[2] * scale)),
                        255,
                    ]
                })
                .collect(),
            emissive_scale,
            geometry,
            texel,
            source_triangles,
        })
    }

    /// Node `n`'s proxy as a glTF binary: one mesh about the node's centre,
    /// one material over its pages.
    fn glb(&self, n: usize, p: &Proxy) -> Result<Vec<u8>, String> {
        let center = self.nodes[n].center;
        let local: Vec<[f32; 3]> = p
            .positions
            .iter()
            .map(|&v| (v - center).to_array())
            .collect();
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for v in &local {
            lo = lo.min(Vec3::from(*v));
            hi = hi.max(Vec3::from(*v));
        }
        let normals: Vec<[f32; 3]> = p.normals.iter().map(|n| n.to_array()).collect();
        let tangents: Vec<[f32; 4]> = p.tangents.iter().map(|t| t.to_array()).collect();
        let uvs: Vec<[f32; 2]> = p.uvs.iter().map(|u| u.to_array()).collect();
        let mut bin = Bin::default();
        let count = local.len();
        let position = bin.add(
            bytemuck::cast_slice(&local),
            Some(34962),
            serde_json::json!({
                "componentType": 5126, "count": count, "type": "VEC3",
                "min": lo.to_array(), "max": hi.to_array()
            }),
        );
        let normal = bin.add(
            bytemuck::cast_slice(&normals),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
        );
        let tangent = bin.add(
            bytemuck::cast_slice(&tangents),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
        );
        let uv = bin.add(
            bytemuck::cast_slice(&uvs),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC2" }),
        );
        let index = bin.add(
            bytemuck::cast_slice(&p.indices),
            Some(34963),
            serde_json::json!({ "componentType": 5125, "count": p.indices.len(), "type": "SCALAR" }),
        );
        let mut images = Vec::new();
        let mut png = |rgba: &[u8]| -> Result<(), String> {
            use image::ImageEncoder;
            let mut out = Vec::new();
            image::codecs::png::PngEncoder::new(&mut out)
                .write_image(rgba, p.width, p.height, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("its page: {e}"))?;
            let view = bin.view(&out);
            images.push(serde_json::json!({ "bufferView": view, "mimeType": "image/png" }));
            Ok(())
        };
        png(&p.base)?;
        png(&p.surface)?;
        png(&p.normal)?;
        let mut material = serde_json::json!({
            "name": "hlod",
            "pbrMetallicRoughness": {
                "baseColorTexture": { "index": 0 },
                "metallicRoughnessTexture": { "index": 1 }
            },
            "normalTexture": { "index": 2 }
        });
        let mut json = serde_json::json!({
            "asset": { "version": "2.0", "generator": "og-cook hlod" },
            "scene": 0,
            "scenes": [{ "nodes": [0] }],
            "nodes": [{ "name": "hlod", "mesh": 0, "translation": center.to_array() }],
            "meshes": [{ "primitives": [{
                "attributes": {
                    "POSITION": position, "NORMAL": normal, "TANGENT": tangent, "TEXCOORD_0": uv
                },
                "indices": index,
                "material": 0
            }] }],
        });
        if self.nodes[n].emissive {
            png(&p.emissive)?;
            material["emissiveTexture"] = serde_json::json!({ "index": 3 });
            material["emissiveFactor"] = serde_json::json!([1.0, 1.0, 1.0]);
            material["extensions"] = serde_json::json!({
                "KHR_materials_emissive_strength": { "emissiveStrength": p.emissive_scale }
            });
            json["extensionsUsed"] = serde_json::json!(["KHR_materials_emissive_strength"]);
        }
        json["textures"] = (0..images.len())
            .map(|i| serde_json::json!({ "source": i }))
            .collect::<Vec<_>>()
            .into();
        json["images"] = images.into();
        json["materials"] = serde_json::json!([material]);
        bin.finish(json)
    }
}

/// The part of each vertex of a mesh of `count` vertices: the least
/// vertex joined to it by `indices`' triangles.
fn parts(count: usize, indices: &[u32]) -> Vec<u32> {
    let mut part: Vec<u32> = (0..count as u32).collect();
    fn find(part: &mut [u32], mut v: u32) -> u32 {
        while part[v as usize] != v {
            part[v as usize] = part[part[v as usize] as usize];
            v = part[v as usize];
        }
        v
    }
    for t in indices.as_chunks::<3>().0 {
        let mut a = find(&mut part, t[0]);
        for &v in &t[1..] {
            let b = find(&mut part, v);
            part[a.max(b) as usize] = a.min(b);
            a = a.min(b);
        }
    }
    for v in 0..count as u32 {
        part[v as usize] = find(&mut part, v);
    }
    part
}

/// A part is large, and what smaller ones may lie flush on, from this
/// many times the distance across.
const LARGE: f32 = 16.0;

/// `full` without its parts that lie flush on a large one: smaller than
/// [`LARGE`] × `within` across, with every vertex and every triangle's
/// centre within `within` of the large parts' surface.
fn without_flush(pos: &[[f32; 3]], full: &[u32], within: f32) -> Vec<u32> {
    let part = parts(pos.len(), full);
    let mut lo = vec![Vec3::splat(f32::MAX); pos.len()];
    let mut hi = vec![Vec3::splat(f32::MIN); pos.len()];
    for &i in full {
        let r = part[i as usize] as usize;
        lo[r] = lo[r].min(Vec3::from(pos[i as usize]));
        hi[r] = hi[r].max(Vec3::from(pos[i as usize]));
    }
    let large = |v: u32| {
        let r = part[v as usize] as usize;
        (hi[r] - lo[r]).length() >= LARGE * within
    };
    let tris = full.as_chunks::<3>().0;
    let corners = |t: &[u32; 3]| t.map(|i| Vec3::from(pos[i as usize]));
    let on: Vec<[Vec3; 3]> = tris.iter().filter(|t| large(t[0])).map(corners).collect();
    if on.is_empty() || on.len() == tris.len() {
        return full.to_vec();
    }
    let bvh = og_bake::bvh::Bvh::new(on);
    let near = |p: Vec3| bvh.nearest(p, within).is_some_and(|h| h.distance <= within);
    // Per part, whether a point of it stands off the large ones.
    let mut off = vec![false; pos.len()];
    for t in tris {
        let r = part[t[0] as usize] as usize;
        if off[r] || large(t[0]) {
            continue;
        }
        let [a, b, c] = corners(t);
        off[r] = ![a, b, c, (a + b + c) / 3.0].into_iter().all(near);
    }
    tris.iter()
        .filter(|t| large(t[0]) || off[part[t[0] as usize] as usize])
        .flatten()
        .copied()
        .collect()
}

/// How far the simplification moved the surface: the farthest a vertex of
/// `full` it removed stands off `kept`'s triangles. A part removed whole
/// (pruned: no vertex of it is kept) counts for no more than its own size,
/// since what it leaves is its absence, not a surface standing where the
/// nearest other part is.
fn removed_error(pos: &[[f32; 3]], full: &[u32], kept: &[u32]) -> f32 {
    let part = parts(pos.len(), full);
    let mut is_kept = vec![false; pos.len()];
    for &i in kept {
        is_kept[i as usize] = true;
    }
    let mut part_kept = vec![false; pos.len()];
    let mut part_lo = vec![Vec3::splat(f32::MAX); pos.len()];
    let mut part_hi = vec![Vec3::splat(f32::MIN); pos.len()];
    let mut used = vec![false; pos.len()];
    for &i in full {
        used[i as usize] = true;
    }
    for v in 0..pos.len() as u32 {
        if !used[v as usize] {
            continue;
        }
        let r = part[v as usize] as usize;
        part_kept[r] |= is_kept[v as usize];
        part_lo[r] = part_lo[r].min(Vec3::from(pos[v as usize]));
        part_hi[r] = part_hi[r].max(Vec3::from(pos[v as usize]));
    }
    let bvh = og_bake::bvh::Bvh::new(
        kept.as_chunks::<3>()
            .0
            .iter()
            .map(|t| t.map(|i| Vec3::from(pos[i as usize])))
            .collect(),
    );
    let mut worst = 0.0f32;
    for v in 0..pos.len() as u32 {
        if !used[v as usize] || is_kept[v as usize] {
            continue;
        }
        let r = part[v as usize] as usize;
        let limit = if part_kept[r] {
            f32::INFINITY
        } else {
            (part_hi[r] - part_lo[r]).length()
        };
        if limit <= worst {
            continue;
        }
        let d = bvh
            .nearest(Vec3::from(pos[v as usize]), limit)
            .map_or(limit, |h| h.distance);
        worst = worst.max(d.min(limit));
    }
    if worst.is_finite() { worst } else { 0.0 }
}

/// The vertices `indices` use, each with the normal of the faces about it
/// that turn less than the crease angle from the corner's own (by area):
/// a vertex on a crease is one vertex per side.
fn creased(pos: &[[f32; 3]], indices: &[u32]) -> (Vec<Vec3>, Vec<Vec3>, Vec<u32>) {
    let tris = indices.as_chunks::<3>().0;
    // Each face's normal, its length twice its area.
    let faces: Vec<Vec3> = tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| Vec3::from(pos[i as usize]));
            (b - a).cross(c - a)
        })
        .collect();
    let mut first = vec![0u32; pos.len() + 1];
    for t in tris {
        for &v in t {
            first[v as usize + 1] += 1;
        }
    }
    for i in 0..pos.len() {
        first[i + 1] += first[i];
    }
    let mut about = vec![0u32; indices.len()];
    let mut fill = first.clone();
    for (f, t) in tris.iter().enumerate() {
        for &v in t {
            about[fill[v as usize] as usize] = f as u32;
            fill[v as usize] += 1;
        }
    }
    let mut made: std::collections::HashMap<(u32, [i16; 3]), u32> =
        std::collections::HashMap::new();
    let (mut positions, mut normals) = (Vec::new(), Vec::new());
    let mut out = Vec::with_capacity(indices.len());
    for (f, t) in tris.iter().enumerate() {
        let own = faces[f].normalize_or(Vec3::Y);
        for &v in t {
            let mut sum = Vec3::ZERO;
            for &g in &about[first[v as usize] as usize..first[v as usize + 1] as usize] {
                let n = faces[g as usize];
                if n.normalize_or(own).dot(own) >= CREASE_COS {
                    sum += n;
                }
            }
            let normal = sum.normalize_or(own);
            let key = (v, normal.to_array().map(|x| (x * 32767.0) as i16));
            let at = *made.entry(key).or_insert_with(|| {
                positions.push(Vec3::from(pos[v as usize]));
                normals.push(normal);
                positions.len() as u32 - 1
            });
            out.push(at);
        }
    }
    (positions, normals, out)
}

/// A proxy's textures, in the order of its glTF's images.
const PAGES: [&str; 4] = ["base", "surface", "normal", "emissive"];

fn page_id(scene: &str, n: usize, page: usize) -> String {
    format!("{}-{}", node_id(scene, n), PAGES[page])
}

/// A scene's tree as planned: its nodes, their members and, `with_cells`,
/// the nodes over its cells; with the tree's stamp. None when the scene
/// does not flatten (its own cook reports why).
fn planned<'a>(
    cx: &Cx<'a>,
    s: &'a Source,
    body: &Block,
    settings: Settings,
    with_cells: bool,
) -> Option<(Arc<Shared<'a>>, u64)> {
    // The scene's own cook reports what does not flatten.
    let texts = |id: &str| cx.prefab_texts.get(id).cloned();
    let flat = prefab::Flattener::new(&texts, cx.schemas)
        .scene(&s.rel, body)
        .ok()?;
    let (found, left, placed_cells) = collect(cx, &flat, &settings);
    let center = |m: usize| (found[m].lo + found[m].hi) * 0.5;
    let cell = |m: usize, level: u32| {
        let c = center(m);
        let side = |v: f32| ((v / settings.size).floor() as i32) >> level.min(30);
        (side(c.x), side(c.z))
    };
    let mut roots: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
    for m in 0..found.len() {
        roots.entry(cell(m, MAX_LEVELS)).or_default().push(m);
    }
    let mut nodes = Vec::new();
    let mut order = Vec::new();
    let mut alone = 0;
    for (_, m) in roots {
        match grow(MAX_LEVELS, m, &cell, true) {
            Ok(g) => {
                lay(g, None, &mut nodes, &mut order);
            }
            Err(m) => alone += m.len(),
        }
    }
    let mut left = left;
    if alone > 0 {
        left.insert("with no other model near", alone);
    }
    // The members in the tree's order.
    let mut found: Vec<Option<Member>> = found.into_iter().map(Some).collect();
    let members: Vec<Member> = order
        .iter()
        .map(|&m| found[m].take().expect("a member is in one node"))
        .collect();
    let settings_bytes = settings.bytes();
    for n in &mut nodes {
        let mine = &members[n.first..n.first + n.count];
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        let mut described = Vec::new();
        for m in mine {
            lo = lo.min(m.lo);
            hi = hi.max(m.hi);
            n.emissive |= m.emissive;
            described.extend_from_slice(m.source.rel.as_bytes());
            described.push(0);
            described.extend_from_slice(&m.source.hash.to_le_bytes());
            described.extend_from_slice(format!("{:?}", m.source.settings).as_bytes());
            described.extend_from_slice(bytemuck::cast_slice(&m.matrix.to_cols_array()));
        }
        n.center = (lo + hi) * 0.5;
        n.radius = ((hi - lo) * 0.5).length().max(1e-3);
        n.stamp = stamp(
            &[Area::Hlod],
            &[
                b"hlod-node",
                &[VERSION],
                &settings_bytes,
                &n.level.to_le_bytes(),
                &(n.own as u64).to_le_bytes(),
                cx.mesh_desc.as_bytes(),
                cx.tex_desc.as_bytes(),
                &described,
            ],
        );
    }
    // Over the cells: each cell's sub-scene's roots, at the cell, grouped
    // as members are, after the scene's own nodes.
    let (mut cells, mut subs) = (Vec::new(), Vec::new());
    if with_cells {
        let mut by_rel: BTreeMap<&str, usize> = BTreeMap::new();
        for (entity, matrix, sub) in &placed_cells {
            let at = match by_rel.get(sub.as_str()) {
                Some(&at) => Some(at),
                None => cx
                    .sources
                    .iter()
                    .find(|x| x.rel == *sub)
                    .and_then(|x| {
                        let text = String::from_utf8_lossy(&x.bytes);
                        let doc = Document::parse(&text).ok()?;
                        let e = doc.body.get_all("hlod").next()?;
                        let settings = Settings::parse(e).ok()?;
                        planned(cx, x, &doc.body, settings, false)
                    })
                    .map(|(tree, _)| {
                        subs.push(tree);
                        by_rel.insert(sub.as_str(), subs.len() - 1);
                        subs.len() - 1
                    }),
            };
            let Some(at) = at else { continue };
            let tree: &Shared = &subs[at];
            for root in (0..tree.nodes.len()).filter(|&r| tree.nodes[r].parent.is_none()) {
                cells.push(CellRef {
                    entity: *entity,
                    matrix: *matrix,
                    sub: at,
                    root,
                });
            }
        }
        let at = |c: usize| {
            cells[c]
                .matrix
                .transform_point3(subs[cells[c].sub].nodes[cells[c].root].center)
        };
        let cell_of = |c: usize, level: u32| {
            let p = at(c);
            let side = |v: f32| ((v / settings.size).floor() as i32) >> level.min(30);
            (side(p.x), side(p.z))
        };
        let mut groups: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
        for c in 0..cells.len() {
            groups.entry(cell_of(c, MAX_LEVELS)).or_default().push(c);
        }
        for (_, c) in groups {
            if let Ok(g) = grow(MAX_LEVELS, c, &cell_of, false) {
                lay_cells(g, None, &mut nodes, members.len());
            }
        }
        // Each over its cells' roots and its children: where they stand,
        // a level above the highest, and a stamp of what they are.
        let over: Vec<usize> = (0..nodes.len())
            .rev()
            .filter(|&n| !nodes[n].cells.is_empty() || nodes[n].count == 0)
            .collect();
        for n in over {
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            let (mut level, mut emissive, mut described) = (0, false, Vec::new());
            for &c in &nodes[n].cells {
                let cell = &cells[c];
                let root = &subs[cell.sub].nodes[cell.root];
                let centre = cell.matrix.transform_point3(root.center);
                let scale = cell
                    .matrix
                    .to_scale_rotation_translation()
                    .0
                    .abs()
                    .max_element();
                lo = lo.min(centre - root.radius * scale);
                hi = hi.max(centre + root.radius * scale);
                level = level.max(root.level + 1);
                emissive |= root.emissive;
                described.extend_from_slice(&root.stamp.to_le_bytes());
                described.extend_from_slice(&cell.entity.to_le_bytes());
                described.extend_from_slice(bytemuck::cast_slice(&cell.matrix.to_cols_array()));
            }
            for &c in &nodes[n].children.clone() {
                let child = &nodes[c];
                lo = lo.min(child.center - child.radius);
                hi = hi.max(child.center + child.radius);
                level = level.max(child.level + 1);
                emissive |= child.emissive;
                described.extend_from_slice(&child.stamp.to_le_bytes());
            }
            let node = &mut nodes[n];
            node.center = (lo + hi) * 0.5;
            node.radius = ((hi - lo) * 0.5).length().max(1e-3);
            node.level = level;
            node.emissive = emissive;
            node.stamp = stamp(
                &[Area::Hlod],
                &[
                    b"hlod-cells",
                    &[VERSION],
                    &settings_bytes,
                    &level.to_le_bytes(),
                    &described,
                ],
            );
        }
    }
    let node_stamps: Vec<u8> = nodes.iter().flat_map(|n| n.stamp.to_le_bytes()).collect();
    let entities: Vec<u8> = members
        .iter()
        .flat_map(|m| m.entity.to_le_bytes())
        .collect();
    let left_desc = format!("{left:?}");
    let tree_stamp = stamp(
        &[Area::Hlod],
        &[
            b"hlod",
            &[VERSION],
            &settings_bytes,
            &node_stamps,
            &entities,
            left_desc.as_bytes(),
        ],
    );
    Some((
        Arc::new(Shared {
            scene: s.rel.clone(),
            source: s,
            settings,
            proxies: nodes.iter().map(|n| registered(n.stamp)).collect(),
            cells,
            subs,
            members,
            nodes,
            left,
            flats: Mutex::default(),
            images: Mutex::default(),
        }),
        tree_stamp,
    ))
}

/// Plans a scene's HLOD outputs: nothing without an `hlod` block.
pub(crate) fn plan<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    text: &str,
    outs: &mut Vec<Out<'a>>,
) {
    let Ok(doc) = Document::parse(text) else {
        return;
    };
    let mut blocks = doc.body.get_all("hlod");
    let Some(entry) = blocks.next() else {
        return;
    };
    let refused = |state: &mut PlanState, e: String| {
        state.early_errors.push((s.rel.clone(), "scene", e));
    };
    if let Some(second) = blocks.next() {
        return refused(
            state,
            format!("line {}: a scene has one hlod block", second.line),
        );
    }
    let settings = match Settings::parse(entry) {
        Ok(v) => v,
        Err(e) => return refused(state, e),
    };
    let Some((shared, tree_stamp)) = planned(cx, s, &doc.body, settings, true) else {
        return;
    };
    // The cook's workers take outputs in this order, and a proxy waits
    // for its children's: the proxies first, each node after everything
    // under it (a later index, laid depth first), so the workers start on
    // as many leaves; then the pages and the tree, which only read them.
    for (n, node) in shared.nodes.iter().enumerate().rev() {
        outs.push((
            node_id(&s.rel, n),
            format::Model::NAME,
            stamp(
                &[Area::Hlod, Area::Model],
                &[b"hlod-model", &node.stamp.to_le_bytes()],
            ),
            Box::new(ProxyJob(shared.clone(), n)),
        ));
    }
    for (n, node) in shared.nodes.iter().enumerate() {
        for page in 0..if node.emissive { 4 } else { 3 } {
            let st = |word: &[u8]| {
                stamp(
                    &[Area::Hlod, Area::Texture],
                    &[b"hlod-page", &[page as u8], &node.stamp.to_le_bytes(), word],
                )
            };
            let id = page_id(&s.rel, n, page);
            let job = |split| Box::new(PageJob(shared.clone(), n, page, split));
            // Streamed by level when a page may pass the mip tail: the
            // tail loads with the scene, the whole chain only when its
            // proxy is drawn large enough to want it.
            if settings.texels > crate::runtime::MIP_TAIL {
                let mips = st(b"mips");
                outs.push((
                    id.clone(),
                    format::Texture::NAME,
                    st(b"tail"),
                    job(Split::Tail(mips)),
                ));
                outs.push((
                    crate::runtime::mips_id(&id),
                    format::Texture::NAME,
                    mips,
                    job(Split::Mips),
                ));
            } else {
                outs.push((id, format::Texture::NAME, st(b""), job(Split::Whole(None))));
            }
        }
    }
    outs.push((
        hlod_id(&s.rel),
        format::Hlod::NAME,
        tree_stamp,
        Box::new(TreeJob(shared.clone())),
    ));
}

/// A scene's tree: `<scene>#hlod`.
struct TreeJob<'a>(Arc<Shared<'a>>);

impl Job for TreeJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let TreeJob(shared) = self;
        let mut rows = Vec::with_capacity(shared.nodes.len());
        let (mut triangles, mut texels, mut levels) = (0usize, 0u64, 0u32);
        let mut worst: Option<(usize, f32, f32)> = None;
        for (n, node) in shared.nodes.iter().enumerate() {
            let proxy = shared.proxy(n).map_err(CookError)?;
            triangles += proxy.indices.len() / 3;
            texels += proxy.width as u64 * proxy.height as u64;
            levels = levels.max(node.level + 1);
            let budget = shared.budget(n);
            if proxy.error() > budget * 1.001
                && worst.is_none_or(|w| proxy.error() / budget > w.1 / w.2)
            {
                worst = Some((n, proxy.error(), budget));
            }
            rows.push(HlodNodeRow {
                parent: node.parent.map_or(NONE, |p| p as u32),
                end: node.end as u32,
                member_first: node.first as u32,
                member_count: node.count as u32,
                level: node.level,
                center_x: node.center.x,
                center_y: node.center.y,
                center_z: node.center.z,
                radius: node.radius,
                error: proxy.error(),
            });
        }
        let members: Vec<HlodMemberRow> = shared
            .members
            .iter()
            .map(|m| {
                let c = m.matrix.to_cols_array_2d();
                HlodMemberRow {
                    entity: m.entity,
                    xx: c[0][0],
                    xy: c[1][0],
                    xz: c[2][0],
                    xt: c[3][0],
                    yx: c[0][1],
                    yy: c[1][1],
                    yz: c[2][1],
                    yt: c[3][1],
                    zx: c[0][2],
                    zy: c[1][2],
                    zz: c[2][2],
                    zt: c[3][2],
                }
            })
            .collect();
        // Each node over cells' cells: the cell entity and its sub-scene's
        // root, drawn at the cell.
        let cells: Vec<HlodCellRow> = shared
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(n, node)| {
                node.cells.iter().map(move |&c| {
                    let cell = &shared.cells[c];
                    let m = cell.matrix.to_cols_array_2d();
                    HlodCellRow {
                        node: n as u32,
                        entity: cell.entity,
                        root: cell.root as u32,
                        xx: m[0][0],
                        xy: m[1][0],
                        xz: m[2][0],
                        xt: m[3][0],
                        yx: m[0][1],
                        yy: m[1][1],
                        yz: m[2][1],
                        yt: m[3][1],
                        zx: m[0][2],
                        zy: m[1][2],
                        zz: m[2][2],
                        zt: m[3][2],
                    }
                })
            })
            .collect();
        let mut w = BlobWriter::new(
            format::Hlod::type_hash(),
            format::Hlod::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[HlodMeta {
                node_count: rows.len() as u32,
                member_count: members.len() as u32,
                size: shared.settings.size,
                error: shared.settings.error,
            }],
            row_hash::<HlodMeta>(),
        )
        .table("nodes", &rows, row_hash::<HlodNodeRow>())
        .table("members", &members, row_hash::<HlodMemberRow>())
        .table("cells", &cells, row_hash::<HlodCellRow>());
        let source: usize = shared
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.parent.is_none())
            .map(|(n, _)| shared.proxy(n).map_or(0, |p| p.source_triangles))
            .sum();
        let mut notes = vec![if rows.is_empty() {
            format!(
                "hlod: no proxy ({} static model(s) to merge; a node needs two within its cell)",
                shared.members.len()
            )
        } else {
            format!(
                "hlod: {} proxies over {} models in {levels} level(s) ({} m cells, {} m error): {source} triangles as {triangles}, {texels} texels a texture",
                rows.len(),
                members.len(),
                shared.settings.size,
                shared.settings.error,
            )
        }];
        if !shared.left.is_empty() {
            notes.push(format!(
                "hlod: {} static model(s) draw themselves: {}",
                shared.left.values().sum::<usize>(),
                shared
                    .left
                    .iter()
                    .map(|(why, n)| format!("{n} {why}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some((n, error, budget)) = worst {
            notes.push(format!(
                "hlod: node {n}'s proxy stands {error:.3} m off its models, over its {budget:.3} m; it is drawn from that much farther (a larger `texels`, or a model too intricate to simplify within the error)"
            ));
        }
        Ok((w.finish(), notes))
    }
}

/// A node's proxy model: `<scene>#hlod-<n>`.
struct ProxyJob<'a>(Arc<Shared<'a>>, usize);

impl Job for ProxyJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let ProxyJob(shared, n) = self;
        let proxy = shared.proxy(*n).map_err(CookError)?;
        let glb = shared.glb(*n, &proxy).map_err(CookError)?;
        let src = model::load(&glb, std::path::Path::new("."))
            .map_err(|e| CookError(format!("hlod node {n}: {e}")))?;
        let ids: Vec<String> = src
            .textures
            .iter()
            .map(|u| page_id(&shared.scene, *n, u.image.min(PAGES.len() - 1)))
            .collect();
        // A proxy is no ray's target and no collider, and is made to the
        // art contract's models already.
        let options = MeshOptions {
            raycast: false,
            collider: false,
            conform: None,
            impostor: None,
            // Its own, which its normal page was drawn in.
            tangents: true,
            // The tree is its levels of detail.
            lods: Some(vec![1.0]),
            ..ex.options.mesh.clone()
        };
        let mut derived = Vec::new();
        let (blob, _) = model::cook(&src, &ids, &options, p.stamp, false, &p.id, &mut derived)?;
        debug_assert!(derived.is_empty(), "an HLOD proxy has only LOD0");
        Ok((
            blob,
            vec![format!(
                "{}: {} models' {} triangles as {}, {}×{} texels of {:.3} m, {:.3} m off them",
                p.id,
                shared.nodes[*n].count,
                proxy.source_triangles,
                proxy.indices.len() / 3,
                proxy.width,
                proxy.height,
                proxy.texel,
                proxy.geometry,
            )],
        ))
    }
}

/// One of a proxy's textures: base colour, surface, normals or emission.
/// Whole, or its mip tail and its whole chain (`#mips`): one encode of
/// the chain serves both (`Source::whole_texture` on the scene).
struct PageJob<'a>(Arc<Shared<'a>>, usize, usize, Split);

impl Job for PageJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PageJob(shared, n, page, split) = self;
        let image = n * PAGES.len() + page;
        texture::split_texture(shared.source, image, split, p, |stamp| {
            self.whole(ex, stamp)
        })
    }
}

impl PageJob<'_> {
    fn whole(&self, ex: &Exec, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PageJob(shared, n, page, _) = self;
        let proxy = shared.proxy(*n).map_err(CookError)?;
        let (rgba, k) = match page {
            0 => (&proxy.base, kind::COLOR),
            1 => (&proxy.surface, kind::LINEAR),
            2 => (&proxy.normal, kind::NORMAL),
            _ => (&proxy.emissive, kind::COLOR),
        };
        let img = Image {
            width: proxy.width,
            height: proxy.height,
            rgba: rgba.clone(),
        };
        texture::cook(&img, k, &ex.options.texture, stamp).map(|t| (t, Vec::new()))
    }
}
