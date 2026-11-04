//! Prefracture: a model with a `fracture { }` block (at the top of a
//! `type model`, or in a glTF's import settings) is cut into chunks here,
//! after it became a glTF binary and before the rest of the cook sees it.
//! The model the cook then cooks has one node per chunk (the support
//! chunks drawn at rest, so the intact model looks as authored; their
//! children hidden, `KHR_node_visibility`), each at its centre of mass and
//! drawn with its outer faces' materials and the cut faces' own interior
//! material; beside it, `<model>#fracture` (`og.Fracture`) holds the
//! chunks, their hierarchy, the bonds of the support graph and the anchors
//! ([`og_destruct::FractureAsset`]).
//!
//! ```text
//! fracture {
//!   chunks 24            # support chunks (Voronoi cells), 2 to 256
//!   seed 1               # which pattern (the same seed, the same chunks)
//!   children 6           # each chunk's pieces when it shatters loose (0: none, up to 32)
//!   pattern volume       # volume (cells in 3D), slab (shards through the thinnest
//!                        # axis: a pane, a plank, a wall) or auto (slab when thin)
//!   impact 0 1 0.1 0.4   # gather half the seeds within 0.4 m of a point (model space)
//!   anchor bottom        # what holds it up: bottom (the faces on its lowest
//!                        # plane rest on the world), none
//!   interiorScale 2      # the cut faces' uvs: repeats a metre
//!   interior {           # the cut faces' material (`type model`'s material lines);
//!     color 0.5 0.45 0.4 #   default: the main material, darker and rough
//!     roughness 0.95
//!   }
//! }
//! ```

use super::model;
use super::vox::Bin;
use glam::{DMat4, DVec3};
use og_core::Rng;
use og_core::ogt::{Block, Document};
use og_destruct::{Bond, Chunk, FractureAsset, NONE};
use og_geom::Mesh;
use og_geom::fracture::{self, Piece};
use std::collections::BTreeMap;
use std::path::Path;

/// The `fracture` block's key, in a `type model` or import settings.
pub const BLOCK: &str = "fracture";

/// An outer triangle's material when its primitive names none (glTF's
/// default material).
const DEFAULT_MATERIAL: u32 = u32::MAX - 1;
/// A cut face's material.
const INTERIOR: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Pattern {
    Auto,
    Volume,
    Slab,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Anchor {
    Bottom,
    None,
}

#[derive(Clone, Debug)]
pub(super) struct Settings {
    chunks: u32,
    seed: u32,
    children: u32,
    pattern: Pattern,
    impact: Option<(DVec3, f64)>,
    anchor: Anchor,
    interior_scale: f64,
    interior: Option<Block>,
}

fn num(b: &Block, k: &str, default: f64) -> Result<f64, String> {
    match b.get(k) {
        None => Ok(default),
        Some(e) => e
            .values
            .first()
            .and_then(|v| v.as_number())
            .map(|n| n.as_f64())
            .filter(|x| x.is_finite())
            .ok_or_else(|| format!("line {}: fracture.{k} takes a number", e.line)),
    }
}

fn count(b: &Block, k: &str, default: u32, min: u32, max: u32) -> Result<u32, String> {
    let v = num(b, k, f64::from(default))?;
    if v.fract() != 0.0 || v < f64::from(min) || v > f64::from(max) {
        return Err(format!(
            "fracture.{k} is a whole number from {min} to {max}, got {v}"
        ));
    }
    Ok(v as u32)
}

impl Settings {
    pub(super) fn parse(b: &Block) -> Result<Settings, String> {
        const KEYS: [&str; 8] = [
            "chunks",
            "seed",
            "children",
            "pattern",
            "impact",
            "anchor",
            "interiorScale",
            "interior",
        ];
        for e in &b.entries {
            if !KEYS.contains(&e.key.as_str()) {
                return Err(format!(
                    "line {}: fracture has no '{}' (its keys: {})",
                    e.line,
                    e.key,
                    KEYS.join(", ")
                ));
            }
        }
        let word = |k: &str, default: &'static str| -> Result<String, String> {
            match b.get(k) {
                None => Ok(default.into()),
                Some(e) => e
                    .word(0)
                    .map(str::to_string)
                    .map_err(|_| format!("line {}: fracture.{k} takes a word", e.line)),
            }
        };
        let pattern = match word("pattern", "auto")?.as_str() {
            "auto" => Pattern::Auto,
            "volume" => Pattern::Volume,
            "slab" => Pattern::Slab,
            o => {
                return Err(format!(
                    "fracture.pattern is auto, volume or slab, not '{o}'"
                ));
            }
        };
        let anchor = match word("anchor", "bottom")?.as_str() {
            "bottom" => Anchor::Bottom,
            "none" => Anchor::None,
            o => return Err(format!("fracture.anchor is bottom or none, not '{o}'")),
        };
        let impact = match b.get("impact") {
            None => None,
            Some(e) => {
                let v: Vec<f64> = e
                    .values
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                if v.len() != 4 || v.iter().any(|x| !x.is_finite()) || v[3] <= 0.0 {
                    return Err(format!(
                        "line {}: fracture.impact takes a point and a radius above 0: `impact x y z r`",
                        e.line
                    ));
                }
                Some((DVec3::new(v[0], v[1], v[2]), v[3]))
            }
        };
        let interior_scale = num(b, "interiorScale", 1.0)?;
        if interior_scale <= 0.0 {
            return Err("fracture.interiorScale is above 0".into());
        }
        let interior = match b.get("interior") {
            None => None,
            Some(e) => Some(
                e.block
                    .clone()
                    .ok_or_else(|| format!("line {}: fracture.interior is a block", e.line))?,
            ),
        };
        Ok(Settings {
            chunks: count(b, "chunks", 16, 2, 256)?,
            seed: count(b, "seed", 1, 0, u32::MAX)?,
            children: count(b, "children", 0, 0, 32)?,
            pattern,
            impact,
            anchor,
            interior_scale,
            interior,
        })
    }

    /// A source's `fracture` block: the top of a `type model` text, else
    /// its import settings'.
    pub(super) fn of(
        text: Option<&str>,
        settings: Option<&Document>,
    ) -> Result<Option<Settings>, String> {
        if let Some(t) = text
            && let Ok(doc) = Document::parse(t)
            && let Some(e) = doc.body.get(BLOCK)
        {
            let b = e.block.as_ref().ok_or("fracture is a block")?;
            return Settings::parse(b).map(Some);
        }
        match settings.and_then(|d| d.body.get(BLOCK)) {
            Some(e) => {
                let b = e.block.as_ref().ok_or("fracture is a block")?;
                Settings::parse(b).map(Some)
            }
            None => Ok(None),
        }
    }
}

/// The default scene's triangles in model space, one mesh, with each
/// triangle's glTF material.
fn flatten(g: &model::GltfSource) -> Result<(Mesh, Vec<u32>), String> {
    if g.doc.skins().next().is_some() {
        return Err(
            "a skinned model does not fracture (its chunks would not follow the skin)".into(),
        );
    }
    let scene = g
        .doc
        .default_scene()
        .or_else(|| g.doc.scenes().next())
        .ok_or("the model has no scene")?;
    let mut mesh = Mesh::default();
    let mut material = Vec::new();
    fn walk(
        node: gltf::Node,
        parent: DMat4,
        g: &model::GltfSource,
        mesh: &mut Mesh,
        material: &mut Vec<u32>,
    ) -> Result<(), String> {
        let m = parent
            * DMat4::from_cols_array_2d(&node.transform().matrix().map(|c| c.map(f64::from)));
        if let Some(me) = node.mesh() {
            let normal_m = glam::DMat3::from_mat4(m).inverse().transpose();
            let flip = glam::DMat3::from_mat4(m).determinant() < 0.0;
            for prim in me.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| Some(&g.buffers[b.index()].0[..]));
                let Some(pos) = reader.read_positions() else {
                    continue;
                };
                let pos: Vec<[f32; 3]> = pos.collect();
                let nor: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|n| n.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; pos.len()]);
                let uv: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|t| t.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; pos.len()]);
                let idx: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..pos.len() as u32).collect());
                let base = mesh.positions.len() as u32;
                for (k, p) in pos.iter().enumerate() {
                    let wp = m.transform_point3(DVec3::from(p.map(f64::from)));
                    let wn = (normal_m * DVec3::from(nor[k].map(f64::from))).normalize_or_zero();
                    mesh.positions.push(wp.as_vec3().to_array());
                    mesh.normals.push(wn.as_vec3().to_array());
                    mesh.uvs.push(uv[k]);
                }
                let mat = prim
                    .material()
                    .index()
                    .map_or(DEFAULT_MATERIAL, |i| i as u32);
                for t in idx.as_chunks::<3>().0 {
                    if flip {
                        mesh.indices.extend([base + t[0], base + t[2], base + t[1]]);
                    } else {
                        mesh.indices.extend([base + t[0], base + t[1], base + t[2]]);
                    }
                    material.push(mat);
                }
            }
        }
        for c in node.children() {
            walk(c, m, g, mesh, material)?;
        }
        Ok(())
    }
    for n in scene.nodes() {
        walk(n, DMat4::IDENTITY, g, &mut mesh, &mut material)?;
    }
    if mesh.indices.is_empty() {
        return Err("the model has no triangles to fracture".into());
    }
    Ok((mesh, material))
}

/// `n` seeds inside `mesh` (Mitchell's best candidate: of 8 candidates
/// the one farthest from the seeds so far, so chunks come in even sizes
/// without a grid), on the plane through the centre across `slab` (an
/// axis) when given, gathered round `impact` for half of them.
fn seeds(
    mesh: &Mesh,
    n: u32,
    rng: &mut Rng,
    slab: Option<usize>,
    impact: Option<(DVec3, f64)>,
) -> Vec<DVec3> {
    let (lo, hi) = fracture::bounds(mesh);
    let centre = (lo + hi) * 0.5;
    let mut out: Vec<DVec3> = Vec::new();
    let mut f = || rng.next_f64();
    let mut tries = 0;
    while (out.len() as u32) < n && tries < n * 64 {
        tries += 1;
        let near = impact.is_some() && out.len().is_multiple_of(2);
        let mut best: Option<(f64, DVec3)> = None;
        for _ in 0..8 {
            let mut p = match (near, impact) {
                (true, Some((at, r))) => {
                    // Denser towards the point: r u² along a direction.
                    let d = DVec3::new(f() * 2.0 - 1.0, f() * 2.0 - 1.0, f() * 2.0 - 1.0);
                    let u = f();
                    at + d.normalize_or_zero() * (r * u * u)
                }
                _ => lo + (hi - lo) * DVec3::new(f(), f(), f()),
            };
            if let Some(a) = slab {
                p[a] = centre[a];
            }
            if !fracture::contains(mesh, p) {
                continue;
            }
            let d = out
                .iter()
                .map(|q| (*q - p).length_squared())
                .fold(f64::INFINITY, f64::min);
            if best.is_none_or(|(bd, _)| d > bd) {
                best = Some((d, p));
            }
        }
        if let Some((d, p)) = best
            && d > 0.0
        {
            out.push(p);
        }
    }
    out
}

/// The anchor faces of a piece: its outer faces facing down on the
/// model's lowest plane (area and centroid).
fn anchor_of(p: &Piece, floor: f64, tol: f64) -> (f64, DVec3) {
    let (mut area, mut sum) = (0.0, DVec3::ZERO);
    for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        if p.material[t].is_none() {
            continue;
        }
        let q = tri.map(|i| DVec3::from(p.mesh.positions[i as usize].map(f64::from)));
        if q.iter().any(|v| v.y > floor + tol) {
            continue;
        }
        let n = (q[1] - q[0]).cross(q[2] - q[0]);
        let a = n.length() * 0.5;
        if a <= 0.0 || n.y / (2.0 * a) > -0.9 {
            continue;
        }
        area += a;
        sum += (q[0] + q[1] + q[2]) / 3.0 * a;
    }
    (area, if area > 0.0 { sum / area } else { DVec3::ZERO })
}

/// The material lines of the cut faces: the settings' `interior` block,
/// else the outer material most triangles use, darker and rough, without
/// its maps.
fn interior_material(
    s: &Settings,
    json: &serde_json::Value,
    main: Option<u32>,
) -> Result<serde_json::Value, String> {
    if let Some(b) = &s.interior {
        return super::model_text::material("interior", b);
    }
    let base = main
        .and_then(|i| json["materials"].get(i as usize))
        .and_then(|m| {
            m["pbrMetallicRoughness"]["baseColorFactor"]
                .as_array()
                .cloned()
        });
    let c: Vec<f64> = base
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .filter(|v: &Vec<f64>| v.len() == 4)
        .unwrap_or(vec![0.8, 0.8, 0.8, 1.0]);
    Ok(serde_json::json!({
        "name": "interior",
        "pbrMetallicRoughness": {
            "baseColorFactor": [c[0] * 0.62, c[1] * 0.6, c[2] * 0.58, 1.0],
            "metallicFactor": 0.0,
            "roughnessFactor": 0.95,
        },
    }))
}

/// One chunk's mesh, relative to its centroid, as a glTF mesh: one
/// primitive a material, in material order.
fn write_chunk(
    bin: &mut Bin,
    name: &str,
    p: &Piece,
    materials: &BTreeMap<u32, usize>,
) -> serde_json::Value {
    let c = p.centroid;
    let mut by: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        let m = p.material[t].unwrap_or(INTERIOR);
        by.entry(materials[&m]).or_default().extend(tri);
    }
    let rel: Vec<[f32; 3]> = p
        .mesh
        .positions
        .iter()
        .map(|q| (DVec3::from(q.map(f64::from)) - c).as_vec3().to_array())
        .collect();
    let mut primitives = Vec::new();
    for (mat, idx) in by {
        // Only the vertices this material's triangles use.
        let mut map: BTreeMap<u32, u32> = BTreeMap::new();
        let mut local = Mesh::default();
        let indices: Vec<u32> = idx
            .iter()
            .map(|&i| {
                *map.entry(i).or_insert_with(|| {
                    local.positions.push(rel[i as usize]);
                    local.normals.push(p.mesh.normals[i as usize]);
                    local.uvs.push(p.mesh.uvs[i as usize]);
                    (local.positions.len() - 1) as u32
                })
            })
            .collect();
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for q in &local.positions {
            for a in 0..3 {
                lo[a] = lo[a].min(q[a]);
                hi[a] = hi[a].max(q[a]);
            }
        }
        let count = local.positions.len();
        let pos = bin.add(
            bytemuck::cast_slice(&local.positions),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi }),
        );
        let nor = bin.add(
            bytemuck::cast_slice(&local.normals),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
        );
        let uv = bin.add(
            bytemuck::cast_slice(&local.uvs),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC2" }),
        );
        let ind = bin.add(
            bytemuck::cast_slice(&indices),
            Some(34963),
            serde_json::json!({ "componentType": 5125, "count": indices.len(), "type": "SCALAR" }),
        );
        primitives.push(serde_json::json!({
            "attributes": { "POSITION": pos, "NORMAL": nor, "TEXCOORD_0": uv },
            "indices": ind,
            "material": mat,
        }));
    }
    serde_json::json!({ "name": name, "primitives": primitives })
}

/// Its vertices relative to its centroid, each once (in first-use order).
fn hull_of(p: &Piece) -> Vec<glam::Vec3> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for q in &p.mesh.positions {
        if seen.insert(q.map(|x| (x + 0.0).to_bits())) {
            out.push((DVec3::from(q.map(f64::from)) - p.centroid).as_vec3());
        }
    }
    out
}

fn mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xff, 0xd8]) {
        "image/jpeg"
    } else if bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        "image/ktx2"
    }
}

/// Fractures glTF binary `glb` (read beside `dir`) by `s`: the chunked
/// model as a glTF binary, and its fracture asset.
pub(super) fn fracture_glb(
    glb: &[u8],
    dir: &Path,
    s: &Settings,
) -> Result<(Vec<u8>, FractureAsset), String> {
    let g = model::load(glb, dir)?;
    let (mesh, material) = flatten(&g)?;
    let (lo, hi) = fracture::bounds(&mesh);
    let ext = hi - lo;
    let size = ext.max_element();
    let thin = (0..3)
        .min_by(|&a, &b| ext[a].total_cmp(&ext[b]))
        .expect("three axes");
    let others = (0..3)
        .filter(|&a| a != thin)
        .map(|a| ext[a])
        .fold(f64::INFINITY, f64::min);
    let slab = match s.pattern {
        Pattern::Slab => Some(thin),
        Pattern::Volume => None,
        Pattern::Auto => (ext[thin] * 4.0 <= others).then_some(thin),
    };
    let mut rng = Rng::new(s.seed);
    let seeds_1 = seeds(&mesh, s.chunks, &mut rng, slab, s.impact);
    if seeds_1.len() < 2 {
        return Err(
            "fewer than 2 seeds fell inside the model: is it closed and facing out?".into(),
        );
    }
    let level1 = fracture::voronoi(&mesh, &material, &seeds_1, s.interior_scale)
        .map_err(|e| format!("fracture: {e}"))?;
    let support = level1.pieces.len();
    // The children of each support chunk, cut from it as it stands (its
    // cut faces stay interior).
    let mut children: Vec<Vec<Piece>> = Vec::new();
    for p in &level1.pieces {
        if s.children < 2 {
            children.push(Vec::new());
            continue;
        }
        let mats: Vec<u32> = p.material.iter().map(|m| m.unwrap_or(INTERIOR)).collect();
        let cs = seeds(&p.mesh, s.children, &mut rng, None, None);
        if cs.len() < 2 {
            children.push(Vec::new());
            continue;
        }
        let f = fracture::voronoi(&p.mesh, &mats, &cs, s.interior_scale)
            .map_err(|e| format!("fracture: a chunk's children: {e}"))?;
        children.push(
            f.pieces
                .into_iter()
                .map(|mut c| {
                    for m in &mut c.material {
                        if *m == Some(INTERIOR) {
                            *m = None;
                        }
                    }
                    c
                })
                .collect(),
        );
    }
    // Materials: the model's own, glTF's default when a primitive names
    // none, then the interior.
    let mut json = g.json.clone();
    let mut materials_json: Vec<serde_json::Value> =
        json["materials"].as_array().cloned().unwrap_or_default();
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for &m in &material {
        *counts.entry(m).or_default() += 1;
    }
    let main = counts
        .iter()
        .filter(|(m, _)| **m != DEFAULT_MATERIAL)
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(m, _)| *m);
    let mut index: BTreeMap<u32, usize> = (0..materials_json.len() as u32)
        .map(|i| (i, i as usize))
        .collect();
    if counts.contains_key(&DEFAULT_MATERIAL) {
        materials_json.push(serde_json::json!({ "name": "default" }));
        index.insert(DEFAULT_MATERIAL, materials_json.len() - 1);
    }
    let interior = interior_material(s, &json, main)?;
    materials_json.push(interior);
    index.insert(INTERIOR, materials_json.len() - 1);
    let mut bin = Bin::default();
    // The images, carried over into the new binary.
    if let Some(images) = json["images"].as_array_mut() {
        for (i, img) in images.iter_mut().enumerate() {
            let bytes = &g.images[i];
            let view = bin.view(bytes);
            *img = serde_json::json!({ "bufferView": view, "mimeType": mime(bytes) });
        }
    }
    let mut nodes = Vec::new();
    let mut meshes = Vec::new();
    let mut chunks: Vec<Chunk> = Vec::new();
    let floor = lo.y;
    let tol = size * 1e-3;
    let place = |p: &Piece,
                 name: String,
                 hidden: bool,
                 nodes: &mut Vec<serde_json::Value>,
                 meshes: &mut Vec<serde_json::Value>,
                 bin: &mut Bin| {
        meshes.push(write_chunk(bin, &name, p, &index));
        let mut n = serde_json::json!({
            "name": name,
            "mesh": meshes.len() - 1,
            "translation": p.centroid.as_vec3().to_array(),
        });
        if hidden {
            n["extensions"] = serde_json::json!({ "KHR_node_visibility": { "visible": false } });
        }
        nodes.push(n);
        (nodes.len() - 1) as u32
    };
    for (i, p) in level1.pieces.iter().enumerate() {
        let node = place(
            p,
            format!("chunk{i}"),
            false,
            &mut nodes,
            &mut meshes,
            &mut bin,
        );
        let (anchor_area, anchor_centroid) = match s.anchor {
            Anchor::Bottom => anchor_of(p, floor, tol),
            Anchor::None => (0.0, DVec3::ZERO),
        };
        chunks.push(Chunk {
            centroid: p.centroid.as_vec3(),
            volume: p.volume as f32,
            hull: hull_of(p),
            node,
            parent: NONE,
            children: (0, 0),
            anchor_area: anchor_area as f32,
            anchor_centroid: anchor_centroid.as_vec3(),
        });
    }
    for (i, cs) in children.iter().enumerate() {
        if cs.is_empty() {
            continue;
        }
        chunks[i].children = (chunks.len() as u32, cs.len() as u32);
        for (k, c) in cs.iter().enumerate() {
            let node = place(
                c,
                format!("chunk{i}.{k}"),
                true,
                &mut nodes,
                &mut meshes,
                &mut bin,
            );
            chunks.push(Chunk {
                centroid: c.centroid.as_vec3(),
                volume: c.volume as f32,
                hull: hull_of(c),
                node,
                parent: i as u32,
                children: (0, 0),
                anchor_area: 0.0,
                anchor_centroid: glam::Vec3::ZERO,
            });
        }
    }
    let bonds: Vec<Bond> = level1
        .bonds
        .iter()
        .map(|b| Bond {
            a: b.a,
            b: b.b,
            centroid: b.centroid.as_vec3(),
            normal: b.normal.as_vec3(),
            area: b.area as f32,
        })
        .collect();
    let asset = FractureAsset {
        chunks,
        bonds,
        support: support as u32,
        min: lo.as_vec3(),
        max: hi.as_vec3(),
        interior: index[&INTERIOR] as u32,
    };
    asset.check()?;
    let roots: Vec<usize> = (0..nodes.len()).collect();
    let mut used: Vec<serde_json::Value> = json["extensionsUsed"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if children.iter().any(|c| !c.is_empty()) && !used.iter().any(|e| e == "KHR_node_visibility") {
        used.push("KHR_node_visibility".into());
    }
    let mut out = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook fracture" },
        "scene": 0,
        "scenes": [{ "nodes": roots }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": materials_json,
    });
    for k in ["textures", "samplers", "images"] {
        if let Some(v) = json
            .get(k)
            .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
        {
            out[k] = v.clone();
        }
    }
    if !used.is_empty() {
        out["extensionsUsed"] = used.into();
    }
    if let Some(r) = json
        .get("extensionsRequired")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
    {
        out["extensionsRequired"] = r.clone();
    }
    let glb = bin.finish(out)?;
    Ok((glb, asset))
}

/// The `#fracture` blob of a fractured model.
pub(super) fn cook(s: &super::Source, stamp: u64) -> (Vec<u8>, Vec<String>) {
    use crate::format::{self, BlobKind};
    let a = s
        .fracture
        .as_ref()
        .expect("planned only for a fractured model");
    let bytes = a.to_bytes();
    let mut w = og_blob::BlobWriter::new(
        format::Fracture::type_hash(),
        format::Fracture::schema_version(),
        stamp,
    );
    let children = a.chunks.len() as u32 - a.support;
    w.table(
        "meta",
        &[format::FractureMeta {
            chunks: a.chunks.len() as u32,
            support: a.support,
            bonds: a.bonds.len() as u32,
            asset_bytes: bytes.len() as u32,
        }],
        format::row_hash::<format::FractureMeta>(),
    )
    .raw("asset", bytes, 16);
    let anchored = a.chunks.iter().filter(|c| c.anchor_area > 0.0).count();
    (
        w.finish(),
        vec![format!(
            "fracture: {} chunks ({anchored} anchored), {} bonds, {children} children",
            a.support,
            a.bonds.len()
        )],
    )
}
