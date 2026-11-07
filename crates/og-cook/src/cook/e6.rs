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
//! terrain "terrain/island.ogt" {  # a terrain's surface (phase 2), standing at
//!   position 0 0 0                #   its og.Terrain entity's position
//!   step 2                        #   every 2nd sample (default 1)
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

/// Parses a behaviour source, reading its `use` subtrees from the assets
/// root; `read` sees every subtree file (its path and bytes) as it is read.
fn parse_behaviour(
    s: &Source,
    text: &str,
    read: &mut dyn FnMut(&str, &[u8]),
) -> Result<og_ai::Behaviour, String> {
    let root = assets_root(s);
    let mut resolve = |p: &str| -> Result<String, String> {
        let id = crate::runtime::asset_id(p);
        let bytes = std::fs::read(root.join(id)).map_err(|e| format!("cannot read it: {e}"))?;
        read(id, &bytes);
        String::from_utf8(bytes).map_err(|_| "a behaviour is text (UTF-8)".to_string())
    };
    og_ai::Behaviour::parse_with(text, crate::runtime::asset_id(&s.rel), &mut resolve)
}

/// What a behaviour's stamp covers beyond its own bytes: every subtree
/// file its `use` nodes reach, by path and hash (so editing a subtree
/// re-validates the trees that use it).
pub(super) fn behaviour_dependencies(s: &Source) -> String {
    let mut deps = Vec::new();
    let text = String::from_utf8_lossy(&s.bytes);
    let _ = parse_behaviour(s, &text, &mut |p, b| {
        deps.push(format!("{p}={:016x}", super::hash_bytes(b)));
    });
    deps.join("\n")
}

pub(super) fn cook_behaviour(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a behaviour is text (UTF-8)".into()))?;
    let b = parse_behaviour(s, text, &mut |_, _| {}).map_err(CookError)?;
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
        vec![if b.subtrees.is_empty() {
            format!("behaviour '{}': {kind}, {} nodes", b.name, b.nodes.len())
        } else {
            format!(
                "behaviour '{}': {kind}, {} nodes, using {}",
                b.name,
                b.nodes.len(),
                b.subtrees.join(", ")
            )
        }],
    ))
}

/// Yarn is compiled to engine IR; quests retain their data representation.
pub(super) fn cook_story(
    s: &Source,
    stamp: u64,
    quest: bool,
    functions: Option<&std::collections::BTreeMap<String, og_story::yarn::types::Function>>,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let what = if quest { "quest" } else { "dialogue" };
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError(format!("a {what} is text (UTF-8)")))?;
    let compiled;
    let text = if quest {
        text
    } else {
        compiled =
            og_story::yarn::compile_with_functions(text, functions.cloned().unwrap_or_default())
                .map_err(CookError)?
                .encode()
                .map_err(CookError)?;
        &compiled
    };
    let (mut w, note) = if quest {
        let q = og_story::Quest::parse(text).map_err(CookError)?;
        let w = BlobWriter::new(
            format::Quest::type_hash(),
            format::Quest::schema_version(),
            stamp,
        );
        let objectives: usize = q.stages.iter().map(|s| s.objectives.len()).sum();
        let note = format!(
            "quest '{}': {} stages, {objectives} objectives",
            q.id,
            q.stages.len()
        );
        (w, note)
    } else {
        let d = og_story::Dialogue::decode(text).map_err(CookError)?;
        for (key, text) in &d.strings {
            og_i18n::Message::parse(text)
                .map_err(|e| CookError(format!("Yarn base string {key}: {e}")))?;
        }
        let w = BlobWriter::new(
            format::Dialogue::type_hash(),
            format::Dialogue::schema_version(),
            stamp,
        );
        let choices: usize = d.nodes.iter().map(|n| n.choices.len()).sum();
        let note = format!(
            "dialogue '{}': {} nodes, {choices} choices, {} speakers",
            d.name,
            d.nodes.len(),
            d.speakers.len()
        );
        (w, note)
    };
    if quest {
        w.table(
            "meta",
            &[format::QuestMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::QuestMeta>(),
        );
    } else {
        w.table(
            "meta",
            &[format::DialogueMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::DialogueMeta>(),
        );
    }
    w.raw("text", text.as_bytes().to_vec(), 16);
    Ok((w.finish(), vec![note]))
}

/// The cook's check of dialogues and quests against the base string table:
/// every key they show is in it, and every argument its message takes is
/// passed. As (source, line) warnings; nothing without a string table.
pub(super) fn check_story_keys(
    tables: &[(&str, &[u8])],
    stories: &[(&str, &[u8])],
    functions: &std::collections::BTreeMap<String, og_story::yarn::types::Function>,
) -> Vec<(String, String)> {
    let parsed: Vec<og_i18n::StringTable> = tables
        .iter()
        .filter_map(|(_, b)| og_i18n::StringTable::parse(std::str::from_utf8(b).ok()?).ok())
        .collect();
    let Some(base) = og_i18n::base_table(&parsed) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (rel, bytes) in stories {
        let Ok(text) = std::str::from_utf8(bytes) else {
            continue;
        };
        let result = if rel.ends_with(".yarn") {
            og_story::yarn::compile_with_functions(text, functions.clone())
                .map(|d| (d.keys(), d.strings))
        } else {
            match og_core::ogt::Document::parse(text).map(|d| d.ty) {
                Ok(t) if t == og_story::QUEST_TYPE => {
                    og_story::Quest::parse(text).map(|q| (q.keys(), Default::default()))
                }
                _ => continue,
            }
        };
        let Ok((keys, authored)) = result else {
            continue;
        };
        for (key, args, line) in keys {
            let at = if line > 0 {
                format!("line {line}: ")
            } else {
                String::new()
            };
            match base.entries.get(&key) {
                None if authored.contains_key(&key) => {}
                None => out.push((
                    rel.to_string(),
                    format!(
                        "{at}the key '{key}' is not in the base string table ({}): add it there",
                        base.locale
                    ),
                )),
                Some((_, m)) => {
                    for a in m.arguments() {
                        if !args.contains(&a) {
                            out.push((
                                rel.to_string(),
                                format!(
                                    "{at}the message '{key}' takes {{{a}}}, which nothing passes: add 'arg {a} \"...\"'"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    out
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
/// scene name, its terrains and their heightmaps), with their hashes, for
/// the cook stamp.
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
    for t in doc.body.get_all("terrain") {
        if let Some(p) = t.values.first().and_then(|v| v.as_str()) {
            files.push(p.to_string());
            if let Ok(text) = std::fs::read_to_string(root.join(p)) {
                files.extend(super::terrain::terrain_files(p, &root, &text));
            }
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
    let at = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    // A model written as text (`type model`) is read as the glb it cooks from.
    let (doc, buffers, _) = if path.extension().is_some_and(|e| e == "ogt") {
        let text = std::fs::read_to_string(path).map_err(|e| at(&e))?;
        let glb = super::model_text::to_glb(&text).map_err(|e| at(&e))?;
        gltf::import_slice(glb).map_err(|e| at(&e))?
    } else {
        gltf::import(path).map_err(|e| at(&e))?
    };
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

/// A terrain's surface as triangles (every `step`th sample, each cell
/// split along the renderer's diagonal, its holes left out), standing at `position`, only the
/// cells reaching into `bounds` when given. Returns how many it added.
fn terrain_triangles(
    root: &Path,
    rel: &str,
    position: Vec3,
    step: usize,
    bounds: Option<(Vec3, Vec3)>,
    out: &mut Vec<[Vec3; 3]>,
) -> Result<usize, String> {
    let text = std::fs::read_to_string(root.join(rel))
        .map_err(|_| "does not exist under assets/".to_string())?;
    let doc = super::terrain::parse(rel, root, &text)?;
    let field = super::terrain::field(&doc, root)?;
    let heights = field.heights;
    let holes = field.holes;
    let (cols, rows) = (field.columns as usize, field.rows as usize);
    // A quad of `step` cells is cut out where most of its cells are holes.
    let cut = |c: usize, r: usize| {
        let Some(h) = &holes else {
            return false;
        };
        let (c1, r1) = ((c + step).min(cols - 1), (r + step).min(rows - 1));
        let n = (r..r1)
            .flat_map(|r| (c..c1).map(move |c| (c, r)))
            .filter(|&(c, r)| h[r * (cols - 1) + c] != 0)
            .count();
        n * 2 > (c1 - c) * (r1 - r)
    };
    let size = field.size;
    let cell = (size[0] / (cols - 1) as f32, size[1] / (rows - 1) as f32);
    let height = |c: usize, r: usize| heights[r.min(rows - 1) * cols + c.min(cols - 1)];
    let at = |c: usize, r: usize| {
        let (c, r) = (c.min(cols - 1), r.min(rows - 1));
        Vec3::new(
            position.x - size[0] * 0.5 + c as f32 * cell.0,
            position.y + height(c, r),
            position.z - size[1] * 0.5 + r as f32 * cell.1,
        )
    };
    let before = out.len();
    let mut r = 0;
    while r + 1 < rows {
        let mut c = 0;
        while c + 1 < cols {
            let (p00, p10) = (at(c, r), at(c + step, r));
            let (p01, p11) = (at(c, r + step), at(c + step, r + step));
            let inside = bounds.is_none_or(|(lo, hi)| {
                p11.x >= lo.x && p00.x <= hi.x && p11.z >= lo.z && p00.z <= hi.z
            });
            if inside && !cut(c, r) {
                // Keep the nav bake on the rendered and collided surface.
                if og_core::math::heightfield_zigzag(
                    height(c, r),
                    height(c + step, r),
                    height(c, r + step),
                    height(c + step, r + step),
                ) {
                    out.extend([[p00, p01, p11], [p00, p11, p10]]);
                } else {
                    out.extend([[p00, p01, p10], [p01, p11, p10]]);
                }
            }
            c += step;
        }
        r += step;
    }
    Ok(out.len() - before)
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
            "terrain",
            "bounds",
            "area",
            "link",
        ]
        .contains(&e.key.as_str())
        {
            return Err(CookError(format!(
                "line {}: '{}' is not a navmesh entry: scene, agent, cellSize, cellHeight, tileSize, maxError, box, model, terrain, bounds, area or link",
                e.line, e.key
            )));
        }
    }
    /// A navmesh's `area` entries, in order (a later one wins where they
    /// overlap): `id` (1 to 63) and a shape, a box by default (`center`,
    /// `size`, turned by `yaw` radians about +y), a cylinder with `radius`
    /// (standing on `center`, `height` tall), or a convex prism of three or
    /// more `point` lines from `bottom` to `top`.
    fn navmesh_areas(body: &Block) -> Result<Vec<(og_nav::Shape, u8)>, CookError> {
        let mut out = Vec::new();
        for e in body.get_all("area") {
            let b = e.block.as_ref();
            let id = f32_of(b, "id", -1.0);
            if !(1.0..=63.0).contains(&id) || id.fract() != 0.0 {
                return Err(CookError(format!(
                    "line {}: an area needs an id from 1 to 63 (0 is the ground)",
                    e.line
                )));
            }
            let center = vec3_of(b, "center", Vec3::ZERO);
            let points: Vec<og_core::glam::Vec2> = b
                .map(|b| b.get_all("point"))
                .into_iter()
                .flatten()
                .map(|p| {
                    let n = |i: usize| {
                        p.values
                            .get(i)
                            .and_then(|v| v.as_number())
                            .map(|n| n.as_f32())
                    };
                    og_core::glam::Vec2::new(n(0).unwrap_or(0.0), n(2).unwrap_or(0.0))
                })
                .collect();
            let shape = if b.and_then(|b| b.get("radius")).is_some() {
                og_nav::Shape::Cylinder {
                    base: center,
                    radius: f32_of(b, "radius", 0.0).max(0.0),
                    height: f32_of(b, "height", 2.0).max(0.0),
                }
            } else if !points.is_empty() {
                if points.len() < 3 {
                    return Err(CookError(format!(
                        "line {}: a convex area needs three or more points, it has {}",
                        e.line,
                        points.len()
                    )));
                }
                og_nav::Shape::Convex {
                    points,
                    bottom: f32_of(b, "bottom", -1.0),
                    top: f32_of(b, "top", 2.0),
                }
            } else {
                let half = vec3_of(b, "size", Vec3::ONE) * 0.5;
                let yaw = f32_of(b, "yaw", 0.0);
                if yaw == 0.0 {
                    og_nav::Shape::Box {
                        min: center - half,
                        max: center + half,
                    }
                } else {
                    og_nav::Shape::Oriented { center, half, yaw }
                }
            };
            out.push((shape, id as u8));
        }
        Ok(out)
    }
    /// A navmesh's `link` entries: `id` (1 or more, unique), `start`, `end`,
    /// `kind` (jump, ladder, door or teleport), `bidirectional`, `cost`,
    /// `duration`, `open` and `traverse game` (the game moves the agent).
    fn navmesh_links(body: &Block) -> Result<Vec<(u32, og_nav::Link)>, CookError> {
        let mut out: Vec<(u32, og_nav::Link)> = Vec::new();
        for e in body.get_all("link") {
            let b = e.block.as_ref();
            let id = f32_of(b, "id", 0.0);
            if id < 1.0 || id.fract() != 0.0 || id > u32::MAX as f32 {
                return Err(CookError(format!(
                    "line {}: a link needs an id, a whole number from 1",
                    e.line
                )));
            }
            let id = id as u32;
            if out.iter().any(|(k, _)| *k == id) {
                return Err(CookError(format!(
                    "line {}: link id {id} is used twice",
                    e.line
                )));
            }
            let word = |key: &str| {
                b.and_then(|b| b.get(key))
                    .and_then(|e| e.values.first())
                    .and_then(|v| v.as_word().or(v.as_str()))
                    .map(str::to_string)
            };
            let flag = |key: &str, default: bool| {
                b.and_then(|b| b.get(key))
                    .and_then(|e| e.values.first())
                    .and_then(|v| v.as_bool())
                    .unwrap_or(default)
            };
            let kind = match word("kind").as_deref().unwrap_or("jump") {
                "jump" => og_nav::LinkKind::Jump,
                "ladder" => og_nav::LinkKind::Ladder,
                "door" => og_nav::LinkKind::Door,
                "teleport" => og_nav::LinkKind::Teleport,
                k => {
                    return Err(CookError(format!(
                        "line {}: '{k}' is not a link kind: jump, ladder, door or teleport",
                        e.line
                    )));
                }
            };
            let game = match word("traverse").as_deref() {
                None | Some("engine") => false,
                Some("game") => true,
                Some(t) => {
                    return Err(CookError(format!(
                        "line {}: traverse is engine or game, not '{t}'",
                        e.line
                    )));
                }
            };
            if b.and_then(|b| b.get("start")).is_none() || b.and_then(|b| b.get("end")).is_none() {
                return Err(CookError(format!(
                    "line {}: a link needs a start and an end",
                    e.line
                )));
            }
            out.push((
                id,
                og_nav::Link {
                    start: vec3_of(b, "start", Vec3::ZERO),
                    end: vec3_of(b, "end", Vec3::ZERO),
                    kind,
                    bidirectional: flag("bidirectional", true),
                    cost: f32_of(b, "cost", 0.0).max(0.0),
                    open: flag("open", true),
                    duration: f32_of(b, "duration", 0.0).max(0.0),
                    game,
                },
            ));
        }
        Ok(out)
    }
    let areas = navmesh_areas(body)?;
    let links = navmesh_links(body)?;
    let root = assets_root(s);
    // The agent profiles (G2-4): `agent { }` or `agent <name> { }`, one
    // mesh each. The first is the unnamed one (else the first named): it
    // fills the blob's tables as a navmesh of one agent always has; every
    // other is a whole navmesh blob in a `profile.<name>` section.
    let mut profiles: Vec<(String, Option<&Block>, usize)> = Vec::new();
    for e in body.get_all("agent") {
        let name = match e.values.first() {
            None => String::new(),
            Some(v) => v
                .as_word()
                .or(v.as_str())
                .filter(|n| {
                    !n.is_empty()
                        && n.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
                .ok_or_else(|| {
                    CookError(format!(
                        "line {}: an agent profile's name is letters, digits, '_' and '-'",
                        e.line
                    ))
                })?
                .to_string(),
        };
        if let Some((_, _, line)) = profiles.iter().find(|(n, ..)| *n == name) {
            return Err(CookError(if name.is_empty() {
                format!(
                    "line {}: a second unnamed agent (the first is on line {line}); name the others: agent small {{ ... }}",
                    e.line
                )
            } else {
                format!(
                    "line {}: agent profile '{name}' is also on line {line}",
                    e.line
                )
            }));
        }
        profiles.push((name, e.block.as_ref(), e.line));
    }
    if let Some(i) = profiles.iter().position(|(n, ..)| n.is_empty()) {
        let unnamed = profiles.remove(i);
        profiles.insert(0, unnamed);
    }
    if profiles.is_empty() {
        profiles.push((String::new(), None, 0));
    }
    let params_of = |agent: Option<&Block>| og_nav::NavParams {
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
    for e in body.get_all("terrain") {
        let p = e
            .values
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| CookError(format!("line {}: terrain takes a path", e.line)))?;
        let tb = e.block.as_ref();
        let step = f32_of(tb, "step", 1.0).max(1.0) as usize;
        let n = terrain_triangles(
            &root,
            p,
            vec3_of(tb, "position", Vec3::ZERO),
            step,
            bounds,
            &mut tris,
        )
        .map_err(|e| CookError(format!("terrain '{p}': {e}")))?;
        notes.push(format!("terrain '{p}': {n} triangles"));
    }
    let mut baked = Vec::with_capacity(profiles.len());
    for (name, agent, _) in &profiles {
        let params = params_of(*agent);
        let field = og_nav::bake::bake(&tris, &params, bounds).map_err(CookError)?;
        let mut own = Vec::new();
        let (w, polys, tiles) = navmesh_blob(&field, &params, &areas, &links, stamp, &mut own)?;
        baked.push((name.clone(), field, w, polys, tiles, own));
    }
    let (_, field, mut w, polys, tiles, own) = baked.remove(0);
    notes.extend(own);
    if profiles.len() > 1 || !profiles[0].0.is_empty() {
        let mut names = profiles[0].0.clone();
        for (name, _, b, polys, tiles, own) in baked {
            notes.push(format!(
                "agent profile '{name}': {polys} polygons in {tiles} tiles"
            ));
            notes.extend(
                own.into_iter()
                    .map(|n| format!("agent profile '{name}': {n}")),
            );
            w.raw(&format!("profile.{name}"), b.finish(), 16);
            names.push('\n');
            names.push_str(&name);
        }
        w.raw("profiles", names.into_bytes(), 1);
    }
    notes.insert(
        0,
        format!(
            "navmesh: {} triangles into {} × {} cells, {polys} polygons in {tiles} tiles",
            tris.len(),
            field.width,
            field.depth,
        ),
    );
    Ok((w.finish(), notes))
}

/// One agent profile's navmesh blob (its tables, areas and links), and its
/// polygon and tile counts; its notes go into `notes`.
fn navmesh_blob(
    field: &og_nav::Heightfield,
    params: &og_nav::NavParams,
    areas: &[(og_nav::Shape, u8)],
    links: &[(u32, og_nav::Link)],
    stamp: u64,
    notes: &mut Vec<String>,
) -> Result<(BlobWriter, usize, usize), CookError> {
    let mesh = og_nav::NavMesh::build_with_areas(field.clone(), areas.to_vec());
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
    // G2-4's areas, only in a navmesh that has some (a blob without them
    // loads as all ground, so earlier cooks stay as they were).
    if !areas.is_empty() {
        let poly_areas: Vec<u8> = mesh
            .tiles
            .iter()
            .flat_map(|t| t.polys.iter().map(|p| p.area))
            .collect();
        let mut shapes = Vec::new();
        for (shape, area) in areas {
            shapes.push(*area);
            shape.write(&mut shapes);
        }
        w.raw("poly_areas", poly_areas, 1).raw("areas", shapes, 4);
        notes.push(format!(
            "areas: {} shapes, {} polygons off the ground area",
            areas.len(),
            mesh.tiles
                .iter()
                .flat_map(|t| &t.polys)
                .filter(|p| p.area != 0)
                .count()
        ));
    }
    // G2-4's links, only in a navmesh that has some.
    if !links.is_empty() {
        let linked = og_nav::NavMesh::from_tiles(field.clone(), mesh.tiles.clone())
            .map(|m| m.with_links(links.to_vec()));
        let mut bytes = Vec::new();
        for (id, l) in links {
            bytes.extend_from_slice(&id.to_le_bytes());
            l.write(&mut bytes);
        }
        w.raw("links", bytes, 4);
        let off: Vec<String> = links
            .iter()
            .filter(|(id, _)| linked.as_ref().is_none_or(|m| m.link_ends(*id).is_none()))
            .map(|(id, _)| id.to_string())
            .collect();
        notes.push(format!("links: {}", links.len()));
        if !off.is_empty() {
            notes.push(format!(
                "warning: links {} have an end more than 1 m from the mesh; agents never cross them",
                off.join(", ")
            ));
        }
    }
    Ok((w, mesh.poly_count(), mesh.tiles.len()))
}

/// Cooks a string table (`type strings`): validated by og-i18n, kept as text.
pub(super) fn cook_strings(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a string table is text (UTF-8)".into()))?;
    let t = og_i18n::StringTable::parse(text).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Strings::type_hash(),
        format::Strings::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::StringsMeta {
            text_bytes: text.len() as u32,
            strings: t.entries.len() as u32,
        }],
        format::row_hash::<format::StringsMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    Ok((
        w.finish(),
        vec![format!(
            "strings '{}' ({}): {} entries{}",
            t.locale,
            t.direction.as_str(),
            t.entries.len(),
            if t.fonts.is_empty() {
                String::new()
            } else {
                format!(", fonts {}", t.fonts.join(", "))
            }
        )],
    ))
}

/// The cook's check across a project's string tables: per table, every
/// key the base table has and it lacks, takes other arguments for, or
/// has that the base lacks (og-i18n's `check_tables`), as (source, line).
pub(super) fn check_strings(sources: &[(&str, &[u8])]) -> Vec<(String, String)> {
    let tables: Vec<(String, og_i18n::StringTable)> = sources
        .iter()
        .filter_map(|(rel, bytes)| {
            let t = og_i18n::StringTable::parse(std::str::from_utf8(bytes).ok()?).ok()?;
            Some((rel.to_string(), t))
        })
        .collect();
    let only: Vec<og_i18n::StringTable> = tables.iter().map(|(_, t)| t.clone()).collect();
    og_i18n::check_tables(&only)
        .into_iter()
        .filter_map(|g| {
            let rel = tables.iter().find(|(_, t)| t.locale == g.locale)?.0.clone();
            let what = match g.problem {
                "missing" => format!("missing key '{}' ({})", g.key, g.detail),
                "arguments" => format!("key '{}' takes other arguments ({})", g.key, g.detail),
                _ => format!("key '{}' is not in the base table ({})", g.key, g.detail),
            };
            Some((rel, what))
        })
        .collect()
}
