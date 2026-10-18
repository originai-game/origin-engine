//! The E6 cook steps: behaviours (`type behaviour`, validated by og-ai and
//! stored as text) and navmeshes (`type navmesh`, baked here: the level's
//! triangles voxelised by og-nav's `bake`, then the tiles built, so the
//! runtime loads polygons ready to link).
//!
//! A navmesh file names its level:
//!
//! ```text
//! type navmesh
//! version 1
//! ---
//! scene "scenes/main.ogt"       # its entities' static colliders and models
//! agent {
//!   radius 0.4
//!   height 1.8
//!   climb 0.4
//!   slope 45
//! }
//! cellSize 0.2
//! cellHeight 0.1
//! tileSize 32
//! box {                         # static boxes the scene does not hold (made in code)
//!   center 0 -0.1 0
//!   size 30 0.2 30
//! }
//! model "models/crate.glb" {    # models placed by code
//!   position 2 0 1
//! }
//! bounds {                      # optional: bake only inside
//!   min -15 -1 -15
//!   max 15 4 15
//! }
//! ```
//!
//! A scene entity counts when it has a `model`, or an `og.Collider` box,
//! sphere, capsule or cylinder (as its bounding box) on no dynamic body.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;
use og_core::glam::{Mat4, Quat, Vec3};
use og_core::ogt::{Block, Document, Entry};
use std::path::{Path, PathBuf};

pub(super) const BEHAVIOUR_TYPE: &str = "behaviour";
pub(super) const NAVMESH_TYPE: &str = "navmesh";

fn assets_root(s: &Source) -> PathBuf {
    let mut p = s.path.clone();
    for _ in s.rel.split('/') {
        p.pop();
    }
    p
}

pub(super) fn cook_behaviour(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a behaviour is text (UTF-8)".into()))?;
    let b = og_ai::Behaviour::parse(text).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Behaviour::type_hash(),
        format::Behaviour::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::BehaviourMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::BehaviourMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    let kind = match &b.root {
        og_ai::Root::Tree(_) => "a tree".to_string(),
        og_ai::Root::Machine { states, .. } => format!("a machine of {} states", states.len()),
    };
    Ok((
        w.finish(),
        vec![format!(
            "behaviour '{}': {kind}, {} nodes",
            b.name,
            b.nodes.len()
        )],
    ))
}

fn string_of(b: &Block, key: &str) -> Option<String> {
    b.get(key)
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn vec3_of(b: Option<&Block>, key: &str, default: Vec3) -> Vec3 {
    let Some(e) = b.and_then(|b| b.get(key)) else {
        return default;
    };
    let n = |i: usize| {
        e.values
            .get(i)
            .and_then(|v| v.as_number())
            .map(|n| n.as_f32())
    };
    match (n(0), n(1), n(2)) {
        (Some(x), Some(y), Some(z)) => Vec3::new(x, y, z),
        _ => default,
    }
}

fn quat_of(b: Option<&Block>, key: &str) -> Quat {
    let Some(e) = b.and_then(|b| b.get(key)) else {
        return Quat::IDENTITY;
    };
    let n = |i: usize| {
        e.values
            .get(i)
            .and_then(|v| v.as_number())
            .map(|n| n.as_f32())
    };
    match (n(0), n(1), n(2), n(3)) {
        (Some(x), Some(y), Some(z), Some(w)) => Quat::from_xyzw(x, y, z, w).normalize(),
        _ => Quat::IDENTITY,
    }
}

fn f32_of(b: Option<&Block>, key: &str, default: f32) -> f32 {
    b.and_then(|b| b.get(key))
        .and_then(|e| e.values.first())
        .and_then(|v| v.as_number())
        .map_or(default, |n| n.as_f32())
}

/// The files a navmesh depends on (its scene and the models it and the
/// scene name), with their hashes, for the cook stamp.
pub(super) fn navmesh_dependencies(s: &Source) -> String {
    let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    let root = assets_root(s);
    let mut files: Vec<String> = Vec::new();
    if let Some(scene) = string_of(&doc.body, "scene") {
        files.push(scene.clone());
        if let Ok(text) = std::fs::read_to_string(root.join(&scene))
            && let Ok(sd) = Document::parse(&text)
        {
            collect_models(&sd.body, &mut files);
        }
    }
    for m in doc.body.get_all("model") {
        if let Some(p) = m.values.first().and_then(|v| v.as_str()) {
            files.push(p.to_string());
        }
    }
    files.sort();
    files.dedup();
    files
        .iter()
        .map(|f| {
            format!(
                "{f}={:016x}",
                std::fs::read(root.join(f))
                    .map(|b| super::hash_bytes(&b))
                    .unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_models(b: &Block, out: &mut Vec<String>) {
    for e in b.get_all("entity") {
        if let Some(block) = &e.block {
            if let Some(m) = string_of(block, "model") {
                out.push(m);
            }
            collect_models(block, out);
        }
    }
}

/// Every triangle of a glTF's default scene, in its own space times `to_world`.
fn gltf_triangles(path: &Path, to_world: Mat4, out: &mut Vec<[Vec3; 3]>) -> Result<(), String> {
    let (doc, buffers, _) = gltf::import(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let scene = doc
        .default_scene()
        .or_else(|| doc.scenes().next())
        .ok_or_else(|| format!("{}: no scene", path.display()))?;
    fn walk(
        node: gltf::Node,
        parent: Mat4,
        buffers: &[gltf::buffer::Data],
        out: &mut Vec<[Vec3; 3]>,
    ) {
        let m = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| Some(&buffers[b.index()]));
                let Some(pos) = reader.read_positions() else {
                    continue;
                };
                let pos: Vec<Vec3> = pos
                    .map(|p| m.transform_point3(Vec3::from_array(p)))
                    .collect();
                let idx: Vec<u32> = match reader.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..pos.len() as u32).collect(),
                };
                for &[a, b, c] in idx.as_chunks::<3>().0 {
                    let (a, b, c) = (a as usize, b as usize, c as usize);
                    if a < pos.len() && b < pos.len() && c < pos.len() {
                        out.push([pos[a], pos[b], pos[c]]);
                    }
                }
            }
        }
        for child in node.children() {
            walk(child, m, buffers, out);
        }
    }
    for n in scene.nodes() {
        walk(n, to_world, &buffers, out);
    }
    Ok(())
}

/// A scene's static geometry: models and the bounding boxes of static colliders.
fn scene_triangles(
    root: &Path,
    b: &Block,
    parent: Mat4,
    out: &mut Vec<[Vec3; 3]>,
    notes: &mut Vec<String>,
) -> Result<(), String> {
    for e in b.get_all("entity") {
        let Some(block) = &e.block else { continue };
        let t = block.get("Transform").and_then(|t| t.block.as_ref());
        let m = parent
            * Mat4::from_scale_rotation_translation(
                vec3_of(t, "scale", Vec3::ONE),
                quat_of(t, "quaternion"),
                vec3_of(t, "position", Vec3::ZERO),
            );
        let dynamic = block
            .get("RigidBody")
            .and_then(|r| r.block.as_ref())
            .is_some_and(|r| f32_of(Some(r), "kind", 0.0) == 0.0);
        if let Some(model) = string_of(block, "model")
            && !dynamic
        {
            gltf_triangles(&root.join(&model), m, out)?;
        }
        if let Some(c) = block.get("Collider").and_then(|c| c.block.as_ref())
            && !dynamic
        {
            let shape = f32_of(Some(c), "shape", 0.0) as u32;
            let half = match shape {
                0 => vec3_of(Some(c), "size", Vec3::ONE) * 0.5,
                1 => Vec3::splat(f32_of(Some(c), "radius", 0.5)),
                2 | 3 => {
                    let r = f32_of(Some(c), "radius", 0.5);
                    let h = f32_of(Some(c), "height", 1.0);
                    Vec3::new(r, h * 0.5 + if shape == 2 { r } else { 0.0 }, r)
                }
                _ => {
                    notes.push(format!(
                        "entity '{}': a convex or mesh collider is baked from its model only",
                        e.values.first().and_then(|v| v.as_str()).unwrap_or("")
                    ));
                    continue;
                }
            };
            let offset = vec3_of(Some(c), "offset", Vec3::ZERO);
            let local = og_nav::bake::box_triangles(offset, half);
            out.extend(local.into_iter().map(|t| t.map(|v| m.transform_point3(v))));
        }
        scene_triangles(root, block, m, out, notes)?;
    }
    Ok(())
}

fn entry_block(e: &Entry) -> Option<&Block> {
    e.block.as_ref()
}

pub(super) fn cook_navmesh(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a navmesh is text (UTF-8)".into()))?;
    let doc = Document::parse(text).map_err(|e| CookError(e.to_string()))?;
    let body = &doc.body;
    for e in &body.entries {
        if ![
            "scene",
            "agent",
            "cellSize",
            "cellHeight",
            "tileSize",
            "maxError",
            "box",
            "model",
            "bounds",
        ]
        .contains(&e.key.as_str())
        {
            return Err(CookError(format!(
                "line {}: '{}' is not a navmesh entry: scene, agent, cellSize, cellHeight, tileSize, maxError, box, model or bounds",
                e.line, e.key
            )));
        }
    }
    let root = assets_root(s);
    let agent = body.get("agent").and_then(entry_block);
    let params = og_nav::NavParams {
        cell_size: f32_of(Some(body), "cellSize", 0.2),
        cell_height: f32_of(Some(body), "cellHeight", 0.1),
        agent_radius: f32_of(agent, "radius", 0.4),
        agent_height: f32_of(agent, "height", 1.8),
        agent_climb: f32_of(agent, "climb", 0.4),
        max_slope: f32_of(agent, "slope", 45.0),
        tile_size: f32_of(Some(body), "tileSize", 32.0).clamp(8.0, 256.0) as u32,
        max_error: f32_of(Some(body), "maxError", 1.3),
    };
    let mut tris: Vec<[Vec3; 3]> = Vec::new();
    let mut notes = Vec::new();
    if let Some(scene) = string_of(body, "scene") {
        let path = root.join(&scene);
        let st = std::fs::read_to_string(&path)
            .map_err(|_| CookError(format!("the scene '{scene}' does not exist under assets/")))?;
        let sd = Document::parse(&st).map_err(|e| CookError(format!("{scene}: {e}")))?;
        scene_triangles(&root, &sd.body, Mat4::IDENTITY, &mut tris, &mut notes)
            .map_err(CookError)?;
    }
    for b in body.get_all("box") {
        let bb = b.block.as_ref();
        let size = vec3_of(bb, "size", Vec3::ONE);
        let m = Mat4::from_rotation_translation(
            quat_of(bb, "quaternion"),
            vec3_of(bb, "center", Vec3::ZERO),
        );
        tris.extend(
            og_nav::bake::box_triangles(Vec3::ZERO, size * 0.5)
                .into_iter()
                .map(|t| t.map(|v| m.transform_point3(v))),
        );
    }
    for e in body.get_all("model") {
        let p = e
            .values
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| CookError(format!("line {}: model takes a path", e.line)))?;
        let mb = e.block.as_ref();
        let m = Mat4::from_scale_rotation_translation(
            vec3_of(mb, "scale", Vec3::ONE),
            quat_of(mb, "quaternion"),
            vec3_of(mb, "position", Vec3::ZERO),
        );
        gltf_triangles(&root.join(p), m, &mut tris).map_err(CookError)?;
    }
    let bounds = body.get("bounds").and_then(entry_block).map(|b| {
        (
            vec3_of(Some(b), "min", Vec3::splat(-10.0)),
            vec3_of(Some(b), "max", Vec3::splat(10.0)),
        )
    });
    let field = og_nav::bake::bake(&tris, &params, bounds).map_err(CookError)?;
    let mesh = og_nav::NavMesh::build(field.clone());
    let meta = format::NavMeshMeta {
        cell_size: params.cell_size,
        cell_height: params.cell_height,
        agent_height: params.agent_height,
        agent_radius: params.agent_radius,
        agent_climb: params.agent_climb,
        max_slope: params.max_slope,
        tile_size: params.tile_size,
        max_error: params.max_error,
        origin_x: field.origin.x,
        origin_y: field.origin.y,
        origin_z: field.origin.z,
        width: field.width,
        depth: field.depth,
        tiles: mesh.tiles.len() as u32,
    };
    let columns: Vec<format::NavColumnRow> = field
        .columns
        .iter()
        .map(|c| format::NavColumnRow {
            first: c[0],
            count: c[1],
        })
        .collect();
    let spans: Vec<format::NavSpanRow> = field
        .spans
        .iter()
        .map(|s| format::NavSpanRow {
            floor: s.floor,
            clear: s.clear,
        })
        .collect();
    let (mut tiles, mut verts, mut polys, mut indices) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for t in &mesh.tiles {
        tiles.push(format::NavTileRow {
            vert_first: verts.len() as u32,
            vert_count: t.verts.len() as u32,
            poly_first: polys.len() as u32,
            poly_count: t.polys.len() as u32,
        });
        for (v, l) in t.verts.iter().zip(&t.lattice) {
            verts.push(format::NavVertRow {
                x: v.x,
                y: v.y,
                z: v.z,
                lx: l.0,
                lz: l.1,
            });
        }
        for p in &t.polys {
            polys.push(format::NavPolyRow {
                index_first: indices.len() as u32,
                index_count: p.verts.len() as u32,
                region: p.region,
            });
            indices.extend(p.verts.iter().map(|&i| i as u32));
        }
    }
    let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let mut w = BlobWriter::new(
        format::NavMesh::type_hash(),
        format::NavMesh::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], format::row_hash::<format::NavMeshMeta>())
        .table(
            "columns",
            &columns,
            format::row_hash::<format::NavColumnRow>(),
        )
        .table("spans", &spans, format::row_hash::<format::NavSpanRow>())
        .table("tiles", &tiles, format::row_hash::<format::NavTileRow>())
        .table("verts", &verts, format::row_hash::<format::NavVertRow>())
        .table("polys", &polys, format::row_hash::<format::NavPolyRow>())
        .raw("poly_verts", index_bytes, 16);
    notes.insert(
        0,
        format!(
            "navmesh: {} triangles into {} × {} cells, {} polygons in {} tiles",
            tris.len(),
            field.width,
            field.depth,
            mesh.poly_count(),
            mesh.tiles.len()
        ),
    );
    Ok((w.finish(), notes))
}
