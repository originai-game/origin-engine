//! Prefracture: a model with a `fracture { }` block (at the top of a
//! `type model`, or in a glTF's import settings) is cut into chunks here,
//! after it became a glTF binary and before the rest of the cook sees it.
//! The model the cook then cooks has one node per chunk, hidden
//! (`KHR_node_visibility`), and last an `intact` node, the model as
//! authored, which is what draws at rest (the cut faces between intact
//! chunks never show, even through glass); a fractured prop draws its
//! chunks' nodes instead. Each chunk's node is at its centre of mass and
//! drawn with its outer faces' materials and the cut faces' own interior
//! material; beside it, `<model>#fracture` (`og.Fracture`) holds the
//! chunks, their hierarchy, the bonds of the support graph and the anchors
//! ([`og_destruct::FractureAsset`]).
//!
//! ```text
//! fracture {
//!   style voronoi        # how it breaks (below)
//!   chunks 24            # support chunks, 2 to 256
//!   seed 1               # which pattern (the same seed, the same chunks)
//!   children 6           # each chunk's pieces when it shatters loose (0: none, up to 32)
//!   pattern volume       # volume (cells in 3D), slab (shards through the thinnest
//!                        # axis: a pane, a plank, a wall) or auto (slab when thin)
//!   impact 0 1 0.1 0.4   # gather the pattern within 0.4 m of a point (model space)
//!   anchor bottom ends   # what holds it up: bottom (the faces on its lowest
//!                        # plane rest on the world), ends (the faces at both ends
//!                        # of its longest level axis are tied in), or none
//!   interiorScale 2      # the cut faces' uvs: repeats a metre
//!   interior {           # the cut faces' material (`type model`'s material lines,
//!     color 0.5 0.45 0.4 #   maps included); default: the main material, darker
//!     roughness 0.95     #   and rough
//!   }
//! }
//! ```
//!
//! The styles, one fracture profile a material:
//!
//! - `voronoi`: Voronoi cells of evenly spread seeds (Mitchell's best
//!   candidate), the general case.
//! - `concrete`: the same cells relaxed twice towards their centroids
//!   (Lloyd), so chunks come chunky and rounded rather than as slivers, at
//!   both levels.
//! - `brick`: a running bond of bricks `brick w h` (metres; along the
//!   longest level axis, up), laid through the thickness: the support
//!   chunks are blocks of the mortar grid in random sizes (a course's
//!   bricks cut where a block ends, as masonry breaks), their children the
//!   bricks within, so rubble is bricks and clusters of them.
//! - `wood`: cells stretched along `grain x|y|z` (default the longest axis)
//!   by three: long splinters along the grain.
//! - `glass`: radial cracks from `impact` (default the centre) crossed by
//!   `rings` concentric ones (1 to 8, default 2), a slab across the thinnest
//!   axis: the shards of a struck pane.

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
enum Style {
    Voronoi,
    Concrete,
    Brick,
    Wood,
    Glass,
}

/// What holds a fractured model up.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Anchor {
    bottom: bool,
    ends: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Settings {
    chunks: u32,
    seed: u32,
    children: u32,
    pattern: Pattern,
    style: Style,
    brick: (f64, f64),
    grain: Option<usize>,
    rings: u32,
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
        const KEYS: [&str; 12] = [
            "style",
            "chunks",
            "seed",
            "children",
            "pattern",
            "brick",
            "grain",
            "rings",
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
        let style = match word("style", "voronoi")?.as_str() {
            "voronoi" => Style::Voronoi,
            "concrete" => Style::Concrete,
            "brick" => Style::Brick,
            "wood" => Style::Wood,
            "glass" => Style::Glass,
            o => {
                return Err(format!(
                    "fracture.style is voronoi, concrete, brick, wood or glass, not '{o}'"
                ));
            }
        };
        let grain = match b.get("grain") {
            None => None,
            Some(_) => Some(match word("grain", "x")?.as_str() {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                o => return Err(format!("fracture.grain is x, y or z, not '{o}'")),
            }),
        };
        let brick = match b.get("brick") {
            None => (0.25, 0.1),
            Some(e) => {
                let v: Vec<f64> = e
                    .values
                    .iter()
                    .filter_map(|v| v.as_number().map(|n| n.as_f64()))
                    .collect();
                if v.len() != 2 || v.iter().any(|x| !x.is_finite() || *x < 0.01) {
                    return Err(format!(
                        "line {}: fracture.brick takes a brick's length and height, each at least 0.01 m: `brick 0.25 0.1`",
                        e.line
                    ));
                }
                (v[0], v[1])
            }
        };
        let mut anchor = Anchor::default();
        match b.get("anchor") {
            None => anchor.bottom = true,
            Some(e) => {
                for v in &e.values {
                    match v.as_word() {
                        Some("bottom") => anchor.bottom = true,
                        Some("ends") => anchor.ends = true,
                        Some("none") if e.values.len() == 1 => {}
                        _ => {
                            return Err(format!(
                                "line {}: fracture.anchor is bottom, ends, both, or none",
                                e.line
                            ));
                        }
                    }
                }
            }
        }
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
            style,
            brick,
            grain,
            rings: count(b, "rings", 2, 1, 8)?,
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

/// Voronoi cells of `seeds` under `metric`, the seeds moved to their
/// cells' centroids `relax` times first (Lloyd's relaxation: rounder,
/// more even cells).
fn relaxed(
    mesh: &Mesh,
    material: &[u32],
    mut seeds: Vec<DVec3>,
    metric: DVec3,
    relax: u32,
    uv_scale: f64,
) -> Result<fracture::Fracture, String> {
    let mut f = fracture::voronoi(mesh, material, &seeds, metric, uv_scale)
        .map_err(|e| format!("fracture: {e}"))?;
    for _ in 0..relax {
        let mut acc = vec![(0.0, DVec3::ZERO); seeds.len()];
        for p in &f.pieces {
            let a = &mut acc[p.cell as usize];
            a.0 += p.volume;
            a.1 += p.centroid * p.volume;
        }
        for (i, (v, c)) in acc.into_iter().enumerate() {
            if v > 0.0 {
                seeds[i] = c / v;
            }
        }
        f = fracture::voronoi(mesh, material, &seeds, metric, uv_scale)
            .map_err(|e| format!("fracture: {e}"))?;
    }
    Ok(f)
}

/// Each support chunk's children, cut from it as it stands by `children`
/// Voronoi cells under `metric` (its cut faces stay interior).
fn cells_children(
    level1: &fracture::Fracture,
    s: &Settings,
    rng: &mut Rng,
    metric: DVec3,
    relax: u32,
) -> Result<Vec<Vec<Piece>>, String> {
    let mut out = Vec::new();
    for p in &level1.pieces {
        if s.children < 2 {
            out.push(Vec::new());
            continue;
        }
        let mats: Vec<u32> = p.material.iter().map(|m| m.unwrap_or(INTERIOR)).collect();
        let cs = seeds(&p.mesh, s.children, rng, None, None);
        if cs.len() < 2 {
            out.push(Vec::new());
            continue;
        }
        let f = relaxed(&p.mesh, &mats, cs, metric, relax, s.interior_scale)
            .map_err(|e| format!("a chunk's children: {e}"))?;
        out.push(
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
    Ok(out)
}

/// Glass: `rings` concentric cracks (polygons through the radial cracks'
/// points at each ring's radius) crossed by radial ones from the impact
/// point, in the plane across axis `thin`. Cell 0 is the centre; cell
/// `1 + s * rings + k` lies in sector s, ring k (outwards).
fn radial(mesh: &Mesh, s: &Settings, thin: usize) -> Vec<Vec<fracture::Plane>> {
    use fracture::Plane;
    let (lo, hi) = fracture::bounds(mesh);
    let mut rng = Rng::new(s.seed ^ 0x9e37_79b9);
    let mut f = || rng.next_f64();
    let (u_axis, v_axis) = match thin {
        0 => (2, 1),
        1 => (0, 2),
        _ => (0, 1),
    };
    let (mut u, mut v) = (DVec3::ZERO, DVec3::ZERO);
    u[u_axis] = 1.0;
    v[v_axis] = 1.0;
    let mut c = s.impact.map_or((lo + hi) * 0.5, |(p, _)| p);
    c[thin] = (lo[thin] + hi[thin]) * 0.5;
    // The far corner's distance in the plane.
    let reach = [lo, hi]
        .iter()
        .flat_map(|a| [lo, hi].map(|b| (a[u_axis], b[v_axis])))
        .map(|(x, y)| ((x - c[u_axis]).powi(2) + (y - c[v_axis]).powi(2)).sqrt())
        .fold(0.0, f64::max)
        .max(1e-3);
    let rings = s.rings as usize;
    let sectors = ((s.chunks as usize).saturating_sub(1) / rings).max(3);
    // Sector edges: even, jittered by up to a third of a sector.
    let step = std::f64::consts::TAU / sectors as f64;
    let start = f() * step;
    let angles: Vec<f64> = (0..sectors)
        .map(|k| start + step * (k as f64 + (f() - 0.5) * 0.66))
        .collect();
    // Ring radii: from a twelfth of the reach to half of it, spaced
    // geometrically (cracks crowd the impact), each jittered.
    let inner = s
        .impact
        .map_or(reach / 12.0, |(_, r)| (r * 0.35).min(reach / 4.0));
    let radii: Vec<f64> = (0..rings)
        .map(|k| {
            let t = if rings == 1 {
                0.0
            } else {
                k as f64 / (rings - 1) as f64
            };
            inner * (reach * 0.55 / inner).powf(t) * (0.9 + 0.2 * f())
        })
        .collect();
    let dir = |a: f64| u * a.cos() + v * a.sin();
    let perp = |a: f64| u * -a.sin() + v * a.cos();
    let idx = |sec: usize, k: usize| (1 + sec * rings + k) as u32;
    let mut cells = Vec::with_capacity(1 + sectors * rings);
    // The centre: inside every sector's first chord.
    let chord = |sec: usize, r: f64| {
        let (a0, a1) = (
            angles[sec],
            angles[(sec + 1) % sectors]
                + if sec + 1 == sectors {
                    std::f64::consts::TAU
                } else {
                    0.0
                },
        );
        let p0 = c + dir(a0) * r;
        let p1 = c + dir(a1) * r;
        let edge = p1 - p0;
        // Outward: across the edge, away from the centre.
        let mut n = edge.cross(u.cross(v)).normalize();
        if n.dot(p0 - c) < 0.0 {
            n = -n;
        }
        (p0, n)
    };
    cells.push(
        (0..sectors)
            .map(|sec| {
                let (p0, n) = chord(sec, radii[0]);
                Plane::through(p0, n, idx(sec, 0))
            })
            .collect(),
    );
    for sec in 0..sectors {
        let next = (sec + 1) % sectors;
        let prev = (sec + sectors - 1) % sectors;
        let a1 = angles[next];
        for k in 0..rings {
            let mut planes = vec![
                Plane::through(c, perp(a1), idx(next, k)),
                Plane::through(c, -perp(angles[sec]), idx(prev, k)),
            ];
            let (p0, n) = chord(sec, radii[k]);
            planes.push(Plane::through(
                p0,
                -n,
                if k == 0 { 0 } else { idx(sec, k - 1) },
            ));
            if k + 1 < rings {
                let (p0, n) = chord(sec, radii[k + 1]);
                planes.push(Plane::through(p0, n, idx(sec, k + 1)));
            }
            cells.push(planes);
        }
    }
    cells
}

/// Brick: a running bond of `s.brick` bricks along the longest level axis,
/// through the thickness. The model is cut into half-brick cells (one
/// neighbour across every face, so their bonds are exact), which are
/// joined into blocks of one to three whole bricks along a course (support
/// chunks: boxes, so their hulls are their shapes; laid in the bond, so
/// they interlock) and, within each block, into its bricks (the children).
fn brick(
    mesh: &Mesh,
    material: &[u32],
    s: &Settings,
    rng: &mut Rng,
) -> Result<(fracture::Fracture, Vec<Vec<Piece>>), String> {
    use fracture::Plane;
    let (lo, hi) = fracture::bounds(mesh);
    let ext = hi - lo;
    let along = if ext.x >= ext.z { 0 } else { 2 };
    let (bw, bh) = s.brick;
    let half = bw * 0.5;
    let cols = ((ext[along] / half).ceil() as usize).max(1);
    let rows = ((ext.y / bh).ceil() as usize).max(1);
    if cols * rows > 4096 {
        return Err(format!(
            "fracture.brick {bw} {bh}: {} half-bricks, past the 4096 a model cuts; use larger bricks",
            cols * rows
        ));
    }
    let id = |r: usize, c: usize| (r * cols + c) as u32;
    let mut axis = DVec3::ZERO;
    axis[along] = 1.0;
    let mut cells = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            let x0 = lo[along] + half * c as f64;
            let y0 = lo.y + bh * r as f64;
            let at = |x: f64, y: f64| {
                let mut p = DVec3::ZERO;
                p[along] = x;
                p.y = y;
                p
            };
            let mut planes = Vec::new();
            if c > 0 {
                planes.push(Plane::through(at(x0, y0), -axis, id(r, c - 1)));
            }
            if c + 1 < cols {
                planes.push(Plane::through(at(x0 + half, y0), axis, id(r, c + 1)));
            }
            if r > 0 {
                planes.push(Plane::through(at(x0, y0), DVec3::NEG_Y, id(r - 1, c)));
            }
            if r + 1 < rows {
                planes.push(Plane::through(at(x0, y0 + bh), DVec3::Y, id(r + 1, c)));
            }
            cells.push(planes);
        }
    }
    let f = fracture::cut(mesh, material, &cells, s.interior_scale)
        .map_err(|e| format!("fracture: {e}"))?;
    let mut pieces_of: Vec<Vec<u32>> = vec![Vec::new(); rows * cols];
    for (k, p) in f.pieces.iter().enumerate() {
        pieces_of[p.cell as usize].push(k as u32);
    }
    // The brick a half-cell belongs to: courses alternate by half a brick.
    let brick_of = |r: usize, c: usize| {
        (
            r,
            if r.is_multiple_of(2) {
                c / 2
            } else {
                c.div_ceil(2)
            },
        )
    };
    // Blocks: runs of whole bricks along each course, one to three long
    // (about the bricks a chunk takes, fewer near the impact). Never more
    // than a course high, so no joint runs straight up the wall: every
    // block bears on two below and carries two above, and a crack steps
    // along the courses, leaving toothed piers as brickwork does.
    let spans_of = |r: usize| -> Vec<(usize, usize)> {
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for c in 0..cols {
            match spans.last_mut() {
                Some((c0, w)) if brick_of(r, *c0) == brick_of(r, c) => *w += 1,
                _ => spans.push((c, 1)),
            }
        }
        spans
    };
    let bricks: usize = (0..rows).map(|r| spans_of(r).len()).sum();
    let target = (bricks as f64 / f64::from(s.chunks)).max(1.0);
    let mut blocks: Vec<(usize, usize, usize, usize)> = Vec::new();
    for r in 0..rows {
        let spans = spans_of(r);
        let mut i = 0;
        while i < spans.len() {
            let mut n = target * (0.55 + 0.9 * rng.next_f64());
            if let Some((p, rad)) = s.impact {
                let mut q = DVec3::ZERO;
                q[along] = lo[along] + half * (spans[i].0 as f64 + 0.5);
                q.y = lo.y + bh * (r as f64 + 0.5);
                let d = ((q[along] - p[along]).powi(2) + (q.y - p.y).powi(2)).sqrt();
                n *= (0.3 + 0.7 * (d / (rad * 2.5)).min(1.0)).max(0.3);
            }
            let n = (n.round() as usize).clamp(1, 3).min(spans.len() - i);
            let w = spans[i..i + n].iter().map(|s| s.1).sum();
            blocks.push((r, spans[i].0, 1, w));
            i += n;
        }
    }
    let mut support_groups: Vec<Vec<u32>> = Vec::new();
    let mut child_groups: Vec<Vec<Vec<u32>>> = Vec::new();
    for &(r0, c0, h, w) in &blocks {
        let mut members = Vec::new();
        let mut bricks: BTreeMap<(usize, usize), Vec<u32>> = BTreeMap::new();
        for r in r0..r0 + h {
            for c in c0..c0 + w {
                let ps = &pieces_of[r * cols + c];
                members.extend(ps);
                bricks.entry(brick_of(r, c)).or_default().extend(ps);
            }
        }
        if members.is_empty() {
            continue;
        }
        let mut kids: Vec<Vec<u32>> = bricks.into_values().filter(|b| !b.is_empty()).collect();
        // A block of one brick has no children; past 32 bricks, pairs.
        while kids.len() > 32 {
            kids = kids.chunks(2).map(|p| p.concat()).collect();
        }
        if kids.len() < 2 || s.children == 0 {
            kids.clear();
        }
        support_groups.push(members);
        child_groups.push(kids);
    }
    let level1 = fracture::merge(&f, &support_groups);
    let children = child_groups
        .iter()
        .map(|g| fracture::merge(&f, g).pieces)
        .collect();
    Ok((level1, children))
}

/// The anchor faces of a piece: its outer faces on the planes `anchor`
/// names (facing down on the model's lowest, facing out at either end of
/// its longest level axis), their area and centroid.
fn anchor_of(p: &Piece, lo: DVec3, hi: DVec3, anchor: Anchor, tol: f64) -> (f64, DVec3) {
    let along = if hi.x - lo.x >= hi.z - lo.z { 0 } else { 2 };
    let (mut area, mut sum) = (0.0, DVec3::ZERO);
    for (t, tri) in p.mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        if p.material[t].is_none() {
            continue;
        }
        let q = tri.map(|i| DVec3::from(p.mesh.positions[i as usize].map(f64::from)));
        let n = (q[1] - q[0]).cross(q[2] - q[0]);
        let a = n.length() * 0.5;
        if a <= 0.0 {
            continue;
        }
        let n = n / (2.0 * a);
        let on = |axis: usize, at: f64| q.iter().all(|v| (v[axis] - at).abs() <= tol);
        let bottom = anchor.bottom && n.y < -0.9 && on(1, lo.y);
        let ends = anchor.ends
            && ((n[along] < -0.9 && on(along, lo[along]))
                || (n[along] > 0.9 && on(along, hi[along])));
        if !(bottom || ends) {
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
    images: &mut Vec<String>,
) -> Result<serde_json::Value, String> {
    if let Some(b) = &s.interior {
        let first = json["textures"].as_array().map_or(0, Vec::len);
        return super::model_text::material("interior", b, images, first);
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

/// What [`fracture_glb`] makes.
pub(super) struct Fractured {
    /// The chunked model as a glTF binary, its images inside.
    pub glb: Vec<u8>,
    pub asset: FractureAsset,
    /// The files beside the model it read (the source's images and the
    /// interior's maps), by their path relative to the model.
    pub read: Vec<(String, Vec<u8>)>,
}

/// Fractures glTF binary `glb` (read beside `dir`) by `s`.
pub(super) fn fracture_glb(glb: &[u8], dir: &Path, s: &Settings) -> Result<Fractured, String> {
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
    let slab = match (s.style, s.pattern) {
        (Style::Glass, _) | (_, Pattern::Slab) => Some(thin),
        (_, Pattern::Volume) => None,
        (_, Pattern::Auto) => (ext[thin] * 4.0 <= others).then_some(thin),
    };
    let mut rng = Rng::new(s.seed);
    let (level1, children) = match s.style {
        Style::Brick => brick(&mesh, &material, s, &mut rng)?,
        Style::Glass => {
            let cells = radial(&mesh, s, thin);
            let f = fracture::cut(&mesh, &material, &cells, s.interior_scale)
                .map_err(|e| format!("fracture: {e}"))?;
            let ch = cells_children(&f, s, &mut rng, DVec3::ONE, 0)?;
            (f, ch)
        }
        style => {
            let (metric, relax) = match style {
                Style::Concrete => (DVec3::ONE, 2),
                Style::Wood => {
                    // Long cells along the grain: distance across it counts
                    // three times as much.
                    let along = s.grain.unwrap_or_else(|| {
                        (0..3)
                            .max_by(|&a, &b| ext[a].total_cmp(&ext[b]))
                            .expect("three axes")
                    });
                    let mut m = DVec3::ONE;
                    m[along] = 1.0 / 3.0;
                    (m, 0)
                }
                _ => (DVec3::ONE, 0),
            };
            let seeds_1 = seeds(&mesh, s.chunks, &mut rng, slab, s.impact);
            if seeds_1.len() < 2 {
                return Err(
                    "fewer than 2 seeds fell inside the model: is it closed and facing out?".into(),
                );
            }
            let f = relaxed(&mesh, &material, seeds_1, metric, relax, s.interior_scale)?;
            let ch = cells_children(&f, s, &mut rng, metric, relax)?;
            (f, ch)
        }
    };
    if level1.pieces.len() < 2 {
        return Err(
            "the fracture left fewer than 2 chunks: is the model closed and facing out?".into(),
        );
    }
    let support = level1.pieces.len();
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
    let mut maps = Vec::new();
    let interior = interior_material(s, &json, main, &mut maps)?;
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
    // The interior's own maps, read beside the model, each a repeating
    // texture after the model's.
    let mut read = g.external.clone();
    if !maps.is_empty() {
        let sampler = json["samplers"].as_array().map_or(0, Vec::len);
        let mut samplers = json["samplers"].as_array().cloned().unwrap_or_default();
        samplers.push(serde_json::json!({ "wrapS": 10497, "wrapT": 10497 }));
        let mut images = json["images"].as_array().cloned().unwrap_or_default();
        let mut textures = json["textures"].as_array().cloned().unwrap_or_default();
        for file in &maps {
            let full = dir.join(file);
            let bytes = std::fs::read(&full)
                .map_err(|e| format!("fracture.interior: {}: {e}", full.display()))?;
            let view = bin.view(&bytes);
            images.push(serde_json::json!({ "bufferView": view, "mimeType": mime(&bytes) }));
            textures.push(serde_json::json!({ "source": images.len() - 1, "sampler": sampler }));
            read.push((file.clone(), bytes));
        }
        json["images"] = images.into();
        json["textures"] = textures.into();
        json["samplers"] = samplers.into();
    }
    let mut nodes = Vec::new();
    let mut meshes = Vec::new();
    let mut chunks: Vec<Chunk> = Vec::new();
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
            true,
            &mut nodes,
            &mut meshes,
            &mut bin,
        );
        let (anchor_area, anchor_centroid) = anchor_of(p, lo, hi, s.anchor, tol);
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
    // The model whole, as it draws at rest.
    let whole = Piece {
        mesh: mesh.clone(),
        material: material.iter().map(|&m| Some(m)).collect(),
        ..Piece::default()
    };
    place(
        &whole,
        "intact".into(),
        false,
        &mut nodes,
        &mut meshes,
        &mut bin,
    );
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
    if !used.iter().any(|e| e == "KHR_node_visibility") {
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
    if let Some(r) = json
        .get("extensionsRequired")
        .filter(|v| v.as_array().is_some_and(|a| !a.is_empty()))
    {
        out["extensionsRequired"] = r.clone();
    }
    for m in out["materials"].as_array().into_iter().flatten() {
        for k in m["extensions"]
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
        {
            if !used.iter().any(|e| e == k) {
                used.push(k.clone().into());
            }
        }
    }
    if !used.is_empty() {
        out["extensionsUsed"] = used.into();
    }
    let glb = bin.finish(out)?;
    Ok(Fractured { glb, asset, read })
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
