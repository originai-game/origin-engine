//! MagicaVoxel `.vox` → glTF: a voxel model becomes a flat-shaded, greedy-
//! meshed glTF binary in memory, which the model cook then treats like any
//! `.glb` (meshoptimizer, LODs, the node graph), so a voxel asset is a
//! model asset with vertex colours and one material.
//!
//! What is read: `SIZE`/`XYZI` model pairs (every model of a `PACK`), the
//! `RGBA` palette (MagicaVoxel's default palette when the file has none),
//! and the scene graph (`nTRN` translations, `nGRP`, `nSHP`) so a multi-part
//! file keeps each part where the editor put it; a file without a graph
//! places its models side by side along x. Rotations in `nTRN` (the `_r`
//! frame byte) are applied as the signed axis permutation they encode; one
//! with a negative determinant mirrors the model, so its triangles' winding
//! is flipped to stay counter-clockwise from outside.
//!
//! Axes: MagicaVoxel is z-up with y away from the viewer; the mesh is y-up
//! (glTF): `(x, y, z)` → `(x, z, -y)`, the model's origin at its `nTRN`
//! translation (the editor's pivot is the model's centre, floored, as
//! MagicaVoxel exports it). The import settings' `model { voxScale
//! voxOrigin }` ([`VoxSettings`]) give the metres per voxel (default 1) and
//! the origin: `center` (the default) where the editor puts it, `bottom`
//! the whole file lifted so its lowest voxel stands on y = 0.
//!
//! Greedy meshing: per face direction, per slice, runs of voxels of one
//! colour merge into rectangles, so a 32³ block of one colour is 12
//! triangles. Each rectangle is four unwelded vertices with the face normal
//! and the palette colour (`COLOR_0`, linear), so the mesh is flat-shaded
//! whatever the material says. The one material is white, metalness 0,
//! roughness 0.9: `MaterialStyle` or the game's material replaces it.

use std::collections::BTreeMap;

/// Where a `.vox` file's origin is (`voxOrigin`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoxOrigin {
    /// The editor's: each model about its centre (floored), at its `nTRN`
    /// translation.
    #[default]
    Center,
    /// The editor's in x and z, the whole file lifted so its lowest voxel
    /// stands on y = 0 (a prop or a character on the ground).
    Bottom,
}

/// A `.vox` file's import settings: `model { voxScale 0.1 voxOrigin bottom }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxSettings {
    /// Metres per voxel (default 1).
    pub scale: f32,
    pub origin: VoxOrigin,
}

impl Default for VoxSettings {
    fn default() -> Self {
        VoxSettings {
            scale: 1.0,
            origin: VoxOrigin::Center,
        }
    }
}

impl VoxSettings {
    /// Reads the `model { … }` block of a `.vox`'s import settings.
    pub fn parse(block: Option<&og_core::ogt::Block>) -> Result<VoxSettings, String> {
        let mut v = VoxSettings::default();
        let Some(b) = block else {
            return Ok(v);
        };
        for e in &b.entries {
            match e.key.as_str() {
                "voxScale" => {
                    v.scale = e
                        .values
                        .first()
                        .and_then(|x| x.as_number())
                        .map(|n| n.as_f32())
                        .filter(|s| s.is_finite() && *s > 0.0)
                        .ok_or_else(|| {
                            format!(
                                "line {}: voxScale takes the metres per voxel, a number above 0 (default 1)",
                                e.line
                            )
                        })?;
                }
                "voxOrigin" => {
                    v.origin = match e.values.first().and_then(|x| x.as_word()) {
                        Some("center") => VoxOrigin::Center,
                        Some("bottom") => VoxOrigin::Bottom,
                        _ => {
                            return Err(format!(
                                "line {}: voxOrigin takes center (the editor's origin, the default) or bottom (the lowest voxel on y = 0)",
                                e.line
                            ));
                        }
                    };
                }
                k => {
                    return Err(format!(
                        "line {}: unknown model setting '{k}' for a .vox (voxScale, voxOrigin)",
                        e.line
                    ));
                }
            }
        }
        Ok(v)
    }
}

/// MagicaVoxel's default palette (`RGBA` chunk absent), as the format
/// specification lists it (index 0 is unused: a voxel's colour index is
/// 1-based).
const DEFAULT_PALETTE: [u32; 256] = [
    0x00000000, 0xffffffff, 0xffccffff, 0xff99ffff, 0xff66ffff, 0xff33ffff, 0xff00ffff, 0xffffccff,
    0xffccccff, 0xff99ccff, 0xff66ccff, 0xff33ccff, 0xff00ccff, 0xffff99ff, 0xffcc99ff, 0xff9999ff,
    0xff6699ff, 0xff3399ff, 0xff0099ff, 0xffff66ff, 0xffcc66ff, 0xff9966ff, 0xff6666ff, 0xff3366ff,
    0xff0066ff, 0xffff33ff, 0xffcc33ff, 0xff9933ff, 0xff6633ff, 0xff3333ff, 0xff0033ff, 0xffff00ff,
    0xffcc00ff, 0xff9900ff, 0xff6600ff, 0xff3300ff, 0xff0000ff, 0xffffffcc, 0xffccffcc, 0xff99ffcc,
    0xff66ffcc, 0xff33ffcc, 0xff00ffcc, 0xffffcccc, 0xffcccccc, 0xff99cccc, 0xff66cccc, 0xff33cccc,
    0xff00cccc, 0xffff99cc, 0xffcc99cc, 0xff9999cc, 0xff6699cc, 0xff3399cc, 0xff0099cc, 0xffff66cc,
    0xffcc66cc, 0xff9966cc, 0xff6666cc, 0xff3366cc, 0xff0066cc, 0xffff33cc, 0xffcc33cc, 0xff9933cc,
    0xff6633cc, 0xff3333cc, 0xff0033cc, 0xffff00cc, 0xffcc00cc, 0xff9900cc, 0xff6600cc, 0xff3300cc,
    0xff0000cc, 0xffffff99, 0xffccff99, 0xff99ff99, 0xff66ff99, 0xff33ff99, 0xff00ff99, 0xffffcc99,
    0xffcccc99, 0xff99cc99, 0xff66cc99, 0xff33cc99, 0xff00cc99, 0xffff9999, 0xffcc9999, 0xff999999,
    0xff669999, 0xff339999, 0xff009999, 0xffff6699, 0xffcc6699, 0xff996699, 0xff666699, 0xff336699,
    0xff006699, 0xffff3399, 0xffcc3399, 0xff993399, 0xff663399, 0xff333399, 0xff003399, 0xffff0099,
    0xffcc0099, 0xff990099, 0xff660099, 0xff330099, 0xff000099, 0xffffff66, 0xffccff66, 0xff99ff66,
    0xff66ff66, 0xff33ff66, 0xff00ff66, 0xffffcc66, 0xffcccc66, 0xff99cc66, 0xff66cc66, 0xff33cc66,
    0xff00cc66, 0xffff9966, 0xffcc9966, 0xff999966, 0xff669966, 0xff339966, 0xff009966, 0xffff6666,
    0xffcc6666, 0xff996666, 0xff666666, 0xff336666, 0xff006666, 0xffff3366, 0xffcc3366, 0xff993366,
    0xff663366, 0xff333366, 0xff003366, 0xffff0066, 0xffcc0066, 0xff990066, 0xff660066, 0xff330066,
    0xff000066, 0xffffff33, 0xffccff33, 0xff99ff33, 0xff66ff33, 0xff33ff33, 0xff00ff33, 0xffffcc33,
    0xffcccc33, 0xff99cc33, 0xff66cc33, 0xff33cc33, 0xff00cc33, 0xffff9933, 0xffcc9933, 0xff999933,
    0xff669933, 0xff339933, 0xff009933, 0xffff6633, 0xffcc6633, 0xff996633, 0xff666633, 0xff336633,
    0xff006633, 0xffff3333, 0xffcc3333, 0xff993333, 0xff663333, 0xff333333, 0xff003333, 0xffff0033,
    0xffcc0033, 0xff990033, 0xff660033, 0xff330033, 0xff000033, 0xffffff00, 0xffccff00, 0xff99ff00,
    0xff66ff00, 0xff33ff00, 0xff00ff00, 0xffffcc00, 0xffcccc00, 0xff99cc00, 0xff66cc00, 0xff33cc00,
    0xff00cc00, 0xffff9900, 0xffcc9900, 0xff999900, 0xff669900, 0xff339900, 0xff009900, 0xffff6600,
    0xffcc6600, 0xff996600, 0xff666600, 0xff336600, 0xff006600, 0xffff3300, 0xffcc3300, 0xff993300,
    0xff663300, 0xff333300, 0xff003300, 0xffff0000, 0xffcc0000, 0xff990000, 0xff660000, 0xff330000,
    0xff0000ee, 0xff0000dd, 0xff0000bb, 0xff0000aa, 0xff000088, 0xff000077, 0xff000055, 0xff000044,
    0xff000022, 0xff000011, 0xff00ee00, 0xff00dd00, 0xff00bb00, 0xff00aa00, 0xff008800, 0xff007700,
    0xff005500, 0xff004400, 0xff002200, 0xff001100, 0xffee0000, 0xffdd0000, 0xffbb0000, 0xffaa0000,
    0xff880000, 0xff770000, 0xff550000, 0xff440000, 0xff220000, 0xff110000, 0xffeeeeee, 0xffdddddd,
    0xffbbbbbb, 0xffaaaaaa, 0xff888888, 0xff777777, 0xff555555, 0xff444444, 0xff222222, 0xff111111,
];

/// One model: its size and its voxels as `(x, y, z, colour index)`.
#[derive(Clone, Debug, Default)]
pub struct VoxModel {
    pub size: [u32; 3],
    pub voxels: Vec<[u8; 4]>,
}

/// A placed model: which model, and its translation in voxel units after
/// the scene graph's rotations, in MagicaVoxel's frame.
#[derive(Clone, Debug)]
pub struct Placement {
    pub model: usize,
    pub translation: [i32; 3],
    /// A signed axis permutation: output axis `i` takes input axis
    /// `rotation[i].0` with sign `rotation[i].1`.
    pub rotation: [(usize, i32); 3],
}

/// A parsed `.vox`: its models, palette (256 RGBA, index 0 unused) and
/// placements.
#[derive(Clone, Debug)]
pub struct VoxFile {
    pub models: Vec<VoxModel>,
    pub palette: [[u8; 4]; 256],
    pub placements: Vec<Placement>,
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| "vox: truncated".to_string())
}

fn i32_at(b: &[u8], at: usize) -> Result<i32, String> {
    u32_at(b, at).map(|v| v as i32)
}

/// A `DICT`: count, then (key, value) string pairs.
fn dict(b: &[u8], at: &mut usize) -> Result<BTreeMap<String, String>, String> {
    let n = u32_at(b, *at)? as usize;
    *at += 4;
    let mut out = BTreeMap::new();
    for _ in 0..n {
        let k = string(b, at)?;
        let v = string(b, at)?;
        out.insert(k, v);
    }
    Ok(out)
}

fn string(b: &[u8], at: &mut usize) -> Result<String, String> {
    let n = u32_at(b, *at)? as usize;
    *at += 4;
    let s = b.get(*at..*at + n).ok_or("vox: truncated string")?;
    *at += n;
    Ok(String::from_utf8_lossy(s).into_owned())
}

const IDENTITY: [(usize, i32); 3] = [(0, 1), (1, 1), (2, 1)];

/// The `_r` byte of an `nTRN` frame: bits 0-1 the input axis of output x,
/// bits 2-3 of output y (z is the remaining one), bits 4-6 the signs. A
/// byte naming one axis twice (or axis 3) is not a rotation: the identity.
fn rotation_of(r: u8) -> [(usize, i32); 3] {
    let ax = (r & 3) as usize;
    let ay = ((r >> 2) & 3) as usize;
    if ax > 2 || ay > 2 || ax == ay {
        return IDENTITY;
    }
    let az = 3 - ax - ay;
    let sign = |bit: u8| if r & (1 << bit) != 0 { -1 } else { 1 };
    [(ax, sign(4)), (ay, sign(5)), (az, sign(6))]
}

/// The determinant of a signed axis permutation: the permutation's parity
/// times its signs. -1 is a mirror (an odd number of axes flipped, or a
/// swap of two axes alone), which turns a face's winding over.
pub fn determinant(r: [(usize, i32); 3]) -> i32 {
    let [a, b, c] = [r[0].0, r[1].0, r[2].0];
    // Inversions of the permutation (a, b, c) of (0, 1, 2).
    let inversions = (a > b) as i32 + (a > c) as i32 + (b > c) as i32;
    let parity = if inversions % 2 == 0 { 1 } else { -1 };
    parity * r[0].1 * r[1].1 * r[2].1
}

/// Parses the chunks of a `.vox` file.
pub fn parse(bytes: &[u8]) -> Result<VoxFile, String> {
    if bytes.len() < 8 || &bytes[0..4] != b"VOX " {
        return Err("vox: not a MagicaVoxel file (no 'VOX ' header)".into());
    }
    let mut models: Vec<VoxModel> = Vec::new();
    let mut palette = [[0u8; 4]; 256];
    for (i, c) in DEFAULT_PALETTE.iter().enumerate() {
        palette[i] = [
            (c & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            ((c >> 16) & 0xff) as u8,
            ((c >> 24) & 0xff) as u8,
        ];
    }
    // The scene graph, by node id.
    enum Node {
        Transform {
            child: u32,
            translation: [i32; 3],
            rotation: [(usize, i32); 3],
        },
        Group(Vec<u32>),
        Shape(Vec<u32>),
    }
    let mut nodes: BTreeMap<u32, Node> = BTreeMap::new();
    let mut pending_size: Option<[u32; 3]> = None;
    let mut at = 8;
    // The MAIN chunk wraps the rest; walk every chunk flat.
    while at + 12 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let content = u32_at(bytes, at + 4)? as usize;
        let _children = u32_at(bytes, at + 8)?;
        let body_at = at + 12;
        let body = bytes
            .get(body_at..body_at + content)
            .ok_or("vox: truncated chunk")?;
        match id {
            b"MAIN" => {}
            b"SIZE" => {
                pending_size = Some([u32_at(body, 0)?, u32_at(body, 4)?, u32_at(body, 8)?]);
            }
            b"XYZI" => {
                let n = u32_at(body, 0)? as usize;
                let size = pending_size.take().ok_or("vox: XYZI without SIZE")?;
                let mut voxels = Vec::with_capacity(n);
                for i in 0..n {
                    let v = body
                        .get(4 + i * 4..8 + i * 4)
                        .ok_or("vox: truncated XYZI")?;
                    if (v[0] as u32) < size[0] && (v[1] as u32) < size[1] && (v[2] as u32) < size[2]
                    {
                        voxels.push([v[0], v[1], v[2], v[3]]);
                    }
                }
                models.push(VoxModel { size, voxels });
            }
            b"RGBA" => {
                for i in 0..255 {
                    let c = body.get(i * 4..i * 4 + 4).ok_or("vox: truncated RGBA")?;
                    palette[i + 1] = [c[0], c[1], c[2], c[3]];
                }
            }
            b"nTRN" => {
                let mut p = 0;
                let node = u32_at(body, p)?;
                p += 4;
                let _attrs = dict(body, &mut p)?;
                let child = u32_at(body, p)?;
                p += 4;
                let _reserved = i32_at(body, p)?;
                p += 4;
                let _layer = u32_at(body, p)?;
                p += 4;
                let frames = u32_at(body, p)?;
                p += 4;
                let mut translation = [0; 3];
                let mut rotation = IDENTITY;
                for f in 0..frames {
                    let d = dict(body, &mut p)?;
                    if f > 0 {
                        continue;
                    }
                    if let Some(t) = d.get("_t") {
                        let v: Vec<i32> = t.split(' ').filter_map(|s| s.parse().ok()).collect();
                        if v.len() == 3 {
                            translation = [v[0], v[1], v[2]];
                        }
                    }
                    if let Some(r) = d.get("_r")
                        && let Ok(r) = r.parse::<u8>()
                    {
                        rotation = rotation_of(r);
                    }
                }
                nodes.insert(
                    node,
                    Node::Transform {
                        child,
                        translation,
                        rotation,
                    },
                );
            }
            b"nGRP" => {
                let mut p = 0;
                let node = u32_at(body, p)?;
                p += 4;
                let _attrs = dict(body, &mut p)?;
                let n = u32_at(body, p)?;
                p += 4;
                let mut children = Vec::new();
                for _ in 0..n {
                    children.push(u32_at(body, p)?);
                    p += 4;
                }
                nodes.insert(node, Node::Group(children));
            }
            b"nSHP" => {
                let mut p = 0;
                let node = u32_at(body, p)?;
                p += 4;
                let _attrs = dict(body, &mut p)?;
                let n = u32_at(body, p)?;
                p += 4;
                let mut ms = Vec::new();
                for _ in 0..n {
                    ms.push(u32_at(body, p)?);
                    p += 4;
                    let _attrs = dict(body, &mut p)?;
                }
                nodes.insert(node, Node::Shape(ms));
            }
            _ => {}
        }
        // MAIN's content is empty and its children follow it, so every
        // chunk is visited in file order by skipping only the content.
        at = body_at + content;
    }
    if models.is_empty() {
        return Err("vox: no model (no SIZE/XYZI chunks)".into());
    }
    // Placements from the graph (root node 0), else side by side.
    let mut placements = Vec::new();
    fn walk(
        nodes: &BTreeMap<u32, Node>,
        id: u32,
        t: [i32; 3],
        r: [(usize, i32); 3],
        out: &mut Vec<Placement>,
        depth: u32,
        models: usize,
    ) {
        if depth > 64 {
            return;
        }
        match nodes.get(&id) {
            Some(Node::Transform {
                child,
                translation,
                rotation,
            }) => {
                // Compose: this frame's translation is in the parent's frame.
                let mut tt = t;
                for i in 0..3 {
                    tt[i] += translation[r[i].0] * r[i].1;
                }
                let rr = [
                    (rotation[r[0].0].0, rotation[r[0].0].1 * r[0].1),
                    (rotation[r[1].0].0, rotation[r[1].0].1 * r[1].1),
                    (rotation[r[2].0].0, rotation[r[2].0].1 * r[2].1),
                ];
                walk(nodes, *child, tt, rr, out, depth + 1, models);
            }
            Some(Node::Group(children)) => {
                for c in children {
                    walk(nodes, *c, t, r, out, depth + 1, models);
                }
            }
            Some(Node::Shape(ms)) => {
                for m in ms {
                    if (*m as usize) < models {
                        out.push(Placement {
                            model: *m as usize,
                            translation: t,
                            rotation: r,
                        });
                    }
                }
            }
            None => {}
        }
    }
    if nodes.contains_key(&0) {
        walk(
            &nodes,
            0,
            [0; 3],
            IDENTITY,
            &mut placements,
            0,
            models.len(),
        );
    }
    if placements.is_empty() {
        let mut x = 0i32;
        for (i, m) in models.iter().enumerate() {
            placements.push(Placement {
                model: i,
                translation: [
                    x + (m.size[0] / 2) as i32,
                    (m.size[1] / 2) as i32,
                    (m.size[2] / 2) as i32,
                ],
                rotation: IDENTITY,
            });
            x += m.size[0] as i32 + 1;
        }
    }
    Ok(VoxFile {
        models,
        palette,
        placements,
    })
}

/// A greedy-meshed quad: its four corners (voxel units, MagicaVoxel's
/// frame), its outward normal and its colour index.
struct Quad {
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
    color: u8,
}

/// Greedy meshing of one model: per axis and direction, per slice, maximal
/// rectangles of one colour whose face is exposed.
fn mesh_model(m: &VoxModel) -> Vec<Quad> {
    let [sx, sy, sz] = m.size.map(|v| v as usize);
    let mut grid = vec![0u8; sx * sy * sz];
    let idx = |x: usize, y: usize, z: usize| x + sx * (y + sy * z);
    for v in &m.voxels {
        grid[idx(v[0] as usize, v[1] as usize, v[2] as usize)] = v[3];
    }
    let at = |x: i64, y: i64, z: i64| -> u8 {
        if x < 0 || y < 0 || z < 0 || x >= sx as i64 || y >= sy as i64 || z >= sz as i64 {
            0
        } else {
            grid[idx(x as usize, y as usize, z as usize)]
        }
    };
    let dims = [sx, sy, sz];
    let mut quads = Vec::new();
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for dir in [-1i64, 1] {
            for slice in 0..dims[axis] {
                // The mask of this slice: the colour whose face along `dir`
                // is exposed, 0 where none.
                let mut mask = vec![0u8; dims[u] * dims[v]];
                for j in 0..dims[v] {
                    for i in 0..dims[u] {
                        let mut p = [0i64; 3];
                        p[axis] = slice as i64;
                        p[u] = i as i64;
                        p[v] = j as i64;
                        let c = at(p[0], p[1], p[2]);
                        if c == 0 {
                            continue;
                        }
                        let mut q = p;
                        q[axis] += dir;
                        if at(q[0], q[1], q[2]) == 0 {
                            mask[i + dims[u] * j] = c;
                        }
                    }
                }
                let mut j = 0;
                while j < dims[v] {
                    let mut i = 0;
                    while i < dims[u] {
                        let c = mask[i + dims[u] * j];
                        if c == 0 {
                            i += 1;
                            continue;
                        }
                        let mut w = 1;
                        while i + w < dims[u] && mask[i + w + dims[u] * j] == c {
                            w += 1;
                        }
                        let mut h = 1;
                        'rows: while j + h < dims[v] {
                            for k in 0..w {
                                if mask[i + k + dims[u] * (j + h)] != c {
                                    break 'rows;
                                }
                            }
                            h += 1;
                        }
                        for jj in 0..h {
                            for ii in 0..w {
                                mask[i + ii + dims[u] * (j + jj)] = 0;
                            }
                        }
                        let face = if dir > 0 {
                            slice as f32 + 1.0
                        } else {
                            slice as f32
                        };
                        let corner = |du: f32, dv: f32| {
                            let mut p = [0f32; 3];
                            p[axis] = face;
                            p[u] = i as f32 + du;
                            p[v] = j as f32 + dv;
                            p
                        };
                        let mut n = [0f32; 3];
                        n[axis] = dir as f32;
                        let (wf, hf) = (w as f32, h as f32);
                        // Counter-clockwise seen from outside.
                        let corners = if dir > 0 {
                            [
                                corner(0.0, 0.0),
                                corner(wf, 0.0),
                                corner(wf, hf),
                                corner(0.0, hf),
                            ]
                        } else {
                            [
                                corner(0.0, 0.0),
                                corner(0.0, hf),
                                corner(wf, hf),
                                corner(wf, 0.0),
                            ]
                        };
                        quads.push(Quad {
                            corners,
                            normal: n,
                            color: c,
                        });
                        i += w;
                    }
                    j += 1;
                }
            }
        }
    }
    quads
}

/// sRGB byte to linear.
fn linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// A `.vox` as a glTF binary: one mesh per placed model under a node at
/// its translation, each a flat-shaded greedy mesh with vertex colours and
/// the shared white material, `settings.scale` metres per voxel, its
/// origin where `settings.origin` says.
pub fn to_glb(bytes: &[u8], settings: &VoxSettings) -> Result<Vec<u8>, String> {
    let file = parse(bytes)?;
    let scale = settings.scale;
    let mut bin: Vec<u8> = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut meshes = Vec::new();
    // Per placed model: its index, its mesh and its translation (glTF frame, metres).
    let mut placed: Vec<(usize, usize, [f32; 3])> = Vec::new();
    let mut floor = f32::MAX;
    let mut quads_total = 0usize;
    let meshed: Vec<Vec<Quad>> = file.models.iter().map(mesh_model).collect();
    for (pi, placement) in file.placements.iter().enumerate() {
        let model = &file.models[placement.model];
        let quads = &meshed[placement.model];
        if quads.is_empty() {
            continue;
        }
        quads_total += quads.len();
        // The model's pivot: its centre, floored, as the editor exports.
        let pivot = model.size.map(|s| (s / 2) as f32);
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(quads.len() * 4);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(quads.len() * 4);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(quads.len() * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(quads.len() * 6);
        let rot = placement.rotation;
        let mirrored = determinant(rot) < 0;
        let turn = |p: [f32; 3]| -> [f32; 3] {
            let mut o = [0f32; 3];
            for i in 0..3 {
                o[i] = p[rot[i].0] * rot[i].1 as f32;
            }
            o
        };
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for q in quads {
            let base = positions.len() as u32;
            let c = file.palette[q.color as usize];
            let col = [linear(c[0]), linear(c[1]), linear(c[2]), 1.0];
            let n = turn(q.normal);
            // MagicaVoxel (x, y, z) z-up → glTF (x, z, -y) y-up.
            let n = [n[0], n[2], -n[1]];
            for corner in q.corners {
                let p = turn([
                    corner[0] - pivot[0],
                    corner[1] - pivot[1],
                    corner[2] - pivot[2],
                ]);
                let p = [p[0] * scale, p[2] * scale, -p[1] * scale];
                for i in 0..3 {
                    lo[i] = lo[i].min(p[i]);
                    hi[i] = hi[i].max(p[i]);
                }
                positions.push(p);
                normals.push(n);
                colors.push(col);
            }
            // (x, y, z) → (x, z, -y) is a rotation, so the winding stays
            // counter-clockwise from outside, unless the frame mirrors.
            if mirrored {
                indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
            } else {
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        let mut view = |data: &[u8], target: u32| -> usize {
            while !bin.len().is_multiple_of(4) {
                bin.push(0);
            }
            let offset = bin.len();
            bin.extend_from_slice(data);
            buffer_views.push(serde_json::json!({
                "buffer": 0, "byteOffset": offset, "byteLength": data.len(), "target": target
            }));
            buffer_views.len() - 1
        };
        let pos_view = view(bytemuck::cast_slice(&positions), 34962);
        let nrm_view = view(bytemuck::cast_slice(&normals), 34962);
        let col_view = view(bytemuck::cast_slice(&colors), 34962);
        let idx_view = view(bytemuck::cast_slice(&indices), 34963);
        let count = positions.len();
        accessors.push(serde_json::json!({
            "bufferView": pos_view, "componentType": 5126, "count": count, "type": "VEC3",
            "min": lo, "max": hi
        }));
        accessors.push(serde_json::json!({
            "bufferView": nrm_view, "componentType": 5126, "count": count, "type": "VEC3"
        }));
        accessors.push(serde_json::json!({
            "bufferView": col_view, "componentType": 5126, "count": count, "type": "VEC4"
        }));
        accessors.push(serde_json::json!({
            "bufferView": idx_view, "componentType": 5125, "count": indices.len(), "type": "SCALAR"
        }));
        let a = accessors.len() - 4;
        meshes.push(serde_json::json!({
            "name": format!("vox{pi}"),
            "primitives": [{
                "attributes": { "POSITION": a, "NORMAL": a + 1, "COLOR_0": a + 2 },
                "indices": a + 3,
                "material": 0
            }]
        }));
        let t = placement.translation;
        let at = [
            t[0] as f32 * scale,
            t[2] as f32 * scale,
            -(t[1] as f32) * scale,
        ];
        floor = floor.min(at[1] + lo[1]);
        placed.push((pi, meshes.len() - 1, at));
    }
    if meshes.is_empty() {
        return Err("vox: every model is empty".into());
    }
    // `bottom`: every node lifted by the same height, the lowest voxel on 0.
    let lift = match settings.origin {
        VoxOrigin::Center => 0.0,
        VoxOrigin::Bottom => -floor,
    };
    let nodes: Vec<serde_json::Value> = placed
        .iter()
        .map(|(pi, mesh, at)| {
            serde_json::json!({
                "name": format!("vox{pi}"),
                "mesh": mesh,
                "translation": [at[0], at[1] + lift, at[2]]
            })
        })
        .collect();
    let _ = quads_total;
    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook vox" },
        "scene": 0,
        "scenes": [{ "nodes": (0..nodes.len()).collect::<Vec<_>>() }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": [{
            "name": "voxels",
            "pbrMetallicRoughness": { "baseColorFactor": [1.0, 1.0, 1.0, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.9 }
        }],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": bin.len() }]
    });
    let mut json_bytes = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json_bytes);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    Ok(out)
}

/// The quads a `.vox` meshes to, for tests and reports: (quads, triangles).
pub fn quad_count(bytes: &[u8]) -> Result<(usize, usize), String> {
    let file = parse(bytes)?;
    let q: usize = file
        .placements
        .iter()
        .map(|p| mesh_model(&file.models[p.model]).len())
        .sum();
    Ok((q, q * 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `.vox` of one model: `size`, its voxels, an optional palette.
    pub fn write(size: [u32; 3], voxels: &[[u8; 4]], palette: Option<&[[u8; 4]]>) -> Vec<u8> {
        let mut children = Vec::new();
        let chunk = |out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]| {
            out.extend_from_slice(id);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(body);
        };
        let mut size_body = Vec::new();
        for s in size {
            size_body.extend_from_slice(&s.to_le_bytes());
        }
        chunk(&mut children, b"SIZE", &size_body);
        let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
        for v in voxels {
            xyzi.extend_from_slice(v);
        }
        chunk(&mut children, b"XYZI", &xyzi);
        if let Some(p) = palette {
            let mut body = Vec::new();
            for i in 0..256 {
                body.extend_from_slice(&p.get(i).copied().unwrap_or([0; 4]));
            }
            chunk(&mut children, b"RGBA", &body);
        }
        let mut out = b"VOX ".to_vec();
        out.extend_from_slice(&150u32.to_le_bytes());
        out.extend_from_slice(b"MAIN");
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(children.len() as u32).to_le_bytes());
        out.extend_from_slice(&children);
        out
    }

    #[test]
    fn a_solid_block_of_one_colour_is_six_quads() {
        let mut voxels = Vec::new();
        for z in 0..4u8 {
            for y in 0..3u8 {
                for x in 0..5u8 {
                    voxels.push([x, y, z, 7]);
                }
            }
        }
        let bytes = write([5, 3, 4], &voxels, None);
        assert_eq!(quad_count(&bytes).unwrap(), (6, 12));
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        assert_eq!(&glb[0..4], b"glTF");
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let mesh = g.document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();
        assert!(prim.get(&gltf::Semantic::Colors(0)).is_some());
        assert_eq!(prim.get(&gltf::Semantic::Positions).unwrap().count(), 24);
    }

    #[test]
    fn two_colours_split_a_face_and_hidden_faces_are_dropped() {
        // Two voxels side by side along x of different colours: 10 faces
        // (the shared face is hidden), the +z face is two quads.
        let bytes = write([2, 1, 1], &[[0, 0, 0, 1], [1, 0, 0, 2]], None);
        assert_eq!(quad_count(&bytes).unwrap().0, 10);
        // The same colour: one block, six quads.
        let bytes = write([2, 1, 1], &[[0, 0, 0, 1], [1, 0, 0, 1]], None);
        assert_eq!(quad_count(&bytes).unwrap().0, 6);
    }

    #[test]
    fn the_palette_colours_the_vertices_and_the_frame_is_y_up() {
        let mut palette = [[0u8; 4]; 256];
        palette[1] = [255, 0, 0, 255];
        let bytes = write(
            [1, 1, 3],
            &[[0, 0, 0, 1], [0, 0, 1, 1], [0, 0, 2, 1]],
            Some(&palette[1..]),
        );
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let mesh = g.document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();
        let reader = prim.reader(|_| Some(&blob[..]));
        let pos: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        // A column of three voxels stands along y (glTF up), 3 m tall.
        let (lo, hi) = pos
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p[1]), h.max(p[1])));
        assert_eq!((lo, hi), (-1.0, 2.0));
        let colors: Vec<[f32; 4]> = reader.read_colors(0).unwrap().into_rgba_f32().collect();
        assert!(
            colors
                .iter()
                .all(|c| c[0] > 0.99 && c[1] < 0.01 && c[2] < 0.01)
        );
    }

    /// A `.vox` of one model placed by a scene graph: root transform 0 →
    /// group 1 → per placement a transform (`_t`, `_r`) over a shape of
    /// model 0.
    fn write_placed(
        size: [u32; 3],
        voxels: &[[u8; 4]],
        placements: &[([i32; 3], Option<u8>)],
    ) -> Vec<u8> {
        let mut out = write(size, voxels, None);
        let mut extra = Vec::new();
        let chunk = |out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]| {
            out.extend_from_slice(id);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(body);
        };
        let string = |b: &mut Vec<u8>, s: &str| {
            b.extend_from_slice(&(s.len() as u32).to_le_bytes());
            b.extend_from_slice(s.as_bytes());
        };
        let transform = |node: u32, child: u32, frame: &[(&str, String)]| {
            let mut b = Vec::new();
            b.extend_from_slice(&node.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes()); // attributes
            b.extend_from_slice(&child.to_le_bytes());
            b.extend_from_slice(&(-1i32).to_le_bytes()); // reserved
            b.extend_from_slice(&0u32.to_le_bytes()); // layer
            b.extend_from_slice(&1u32.to_le_bytes()); // one frame
            b.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            for (k, v) in frame {
                string(&mut b, k);
                string(&mut b, v);
            }
            b
        };
        chunk(&mut extra, b"nTRN", &transform(0, 1, &[]));
        let mut group = Vec::new();
        group.extend_from_slice(&1u32.to_le_bytes());
        group.extend_from_slice(&0u32.to_le_bytes());
        group.extend_from_slice(&(placements.len() as u32).to_le_bytes());
        for i in 0..placements.len() as u32 {
            group.extend_from_slice(&(2 + 2 * i).to_le_bytes());
        }
        chunk(&mut extra, b"nGRP", &group);
        for (i, (t, r)) in placements.iter().enumerate() {
            let node = 2 + 2 * i as u32;
            let mut frame = vec![("_t", format!("{} {} {}", t[0], t[1], t[2]))];
            if let Some(r) = r {
                frame.push(("_r", r.to_string()));
            }
            chunk(&mut extra, b"nTRN", &transform(node, node + 1, &frame));
            let mut shape = Vec::new();
            shape.extend_from_slice(&(node + 1).to_le_bytes());
            shape.extend_from_slice(&0u32.to_le_bytes());
            shape.extend_from_slice(&1u32.to_le_bytes());
            shape.extend_from_slice(&0u32.to_le_bytes()); // model 0
            shape.extend_from_slice(&0u32.to_le_bytes()); // its attributes
            chunk(&mut extra, b"nSHP", &shape);
        }
        out.extend_from_slice(&extra);
        // MAIN's children size.
        let children = (out.len() - 20) as u32;
        out[16..20].copy_from_slice(&children.to_le_bytes());
        out
    }

    /// A primitive's positions, normals and triangles.
    type Primitive = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>);

    /// Per primitive of a glTF binary: its positions, normals and triangles.
    fn primitives(glb: &[u8]) -> Vec<Primitive> {
        let g = gltf::Gltf::from_slice(glb).unwrap();
        let blob = g.blob.clone().unwrap();
        g.document
            .meshes()
            .flat_map(|m| m.primitives().collect::<Vec<_>>())
            .map(|p| {
                let r = p.reader(|_| Some(&blob[..]));
                let pos: Vec<[f32; 3]> = r.read_positions().unwrap().collect();
                let nrm: Vec<[f32; 3]> = r.read_normals().unwrap().collect();
                let idx: Vec<u32> = r.read_indices().unwrap().into_u32().collect();
                let tris = idx.chunks(3).map(|t| [t[0], t[1], t[2]]).collect();
                (pos, nrm, tris)
            })
            .collect()
    }

    /// Every triangle's winding (counter-clockwise) faces along its normal.
    fn faces_out(glb: &[u8]) -> bool {
        primitives(glb).iter().all(|(pos, nrm, tris)| {
            tris.iter().all(|t| {
                let v = |i: u32| glam::Vec3::from(pos[i as usize]);
                let n = glam::Vec3::from(nrm[t[0] as usize]);
                (v(t[1]) - v(t[0])).cross(v(t[2]) - v(t[0])).dot(n) > 0.0
            })
        })
    }

    #[test]
    fn determinants_of_signed_axis_permutations() {
        // Identity, a swap of x and y, one sign, two signs, a cycle.
        assert_eq!(determinant(IDENTITY), 1);
        assert_eq!(determinant([(1, 1), (0, 1), (2, 1)]), -1);
        assert_eq!(determinant([(0, -1), (1, 1), (2, 1)]), -1);
        assert_eq!(determinant([(0, -1), (1, -1), (2, 1)]), 1);
        assert_eq!(determinant([(1, 1), (2, 1), (0, 1)]), 1);
        // The `_r` byte 4 is the identity; 20 flips x; 1 swaps x and y.
        assert_eq!(rotation_of(4), IDENTITY);
        assert_eq!(determinant(rotation_of(20)), -1);
        assert_eq!(rotation_of(1)[0..2], [(1, 1), (0, 1)]);
        assert_eq!(determinant(rotation_of(1)), -1);
        // Not a rotation: an axis twice.
        assert_eq!(rotation_of(0), IDENTITY);
    }

    /// Every one of the 48 signed axis permutations a frame's `_r` can
    /// hold keeps every face's winding counter-clockwise from outside: the
    /// 24 mirrors flip it back (without the flip, they faced inwards).
    #[test]
    fn mirrors_flip_the_winding_and_every_frame_faces_out() {
        let voxels = [[0, 0, 0, 1], [1, 0, 0, 2], [0, 1, 0, 3], [0, 0, 1, 4]];
        let mut mirrors = 0;
        let mut seen = std::collections::BTreeSet::new();
        for r in 0u8..128 {
            let rot = rotation_of(r);
            if r & 3 == (r >> 2) & 3 || r & 3 == 3 || (r >> 2) & 3 == 3 || !seen.insert(rot) {
                continue;
            }
            let bytes = write_placed([2, 2, 2], &voxels, &[([3, -2, 5], Some(r))]);
            let file = parse(&bytes).unwrap();
            assert_eq!(file.placements[0].rotation, rot);
            mirrors += (determinant(rot) < 0) as usize;
            let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
            assert!(faces_out(&glb), "_r {r} ({rot:?}) turns faces inwards");
        }
        assert_eq!((seen.len(), mirrors), (48, 24));
    }

    #[test]
    fn vox_scale_and_bottom_origin_place_the_mesh() {
        // A column of three voxels, placed 10 voxels up (MagicaVoxel z).
        let column = [[0, 0, 0, 1], [0, 0, 1, 1], [0, 0, 2, 1]];
        let bytes = write_placed([1, 1, 3], &column, &[([0, 0, 10], None)]);
        let node_y = |glb: &[u8]| {
            let g = gltf::Gltf::from_slice(glb).unwrap();
            let n = g.document.nodes().next().unwrap();
            n.transform().decomposed().0[1]
        };
        let height = |glb: &[u8]| {
            let p = &primitives(glb)[0].0;
            p.iter()
                .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v[1]), h.max(v[1])))
        };
        let editor = to_glb(&bytes, &VoxSettings::default()).unwrap();
        assert_eq!(
            height(&editor),
            (-1.0, 2.0),
            "one metre a voxel, the pivot floored"
        );
        assert_eq!(node_y(&editor), 10.0);
        let tenth = VoxSettings {
            scale: 0.1,
            origin: VoxOrigin::Center,
        };
        let small = to_glb(&bytes, &tenth).unwrap();
        let (lo, hi) = height(&small);
        assert!(
            (lo + 0.1).abs() < 1e-6 && (hi - 0.2).abs() < 1e-6,
            "{lo} {hi}"
        );
        assert!((node_y(&small) - 1.0).abs() < 1e-6);
        let bottom = VoxSettings {
            scale: 0.1,
            origin: VoxOrigin::Bottom,
        };
        let grounded = to_glb(&bytes, &bottom).unwrap();
        let (lo, hi) = height(&grounded);
        let y = node_y(&grounded);
        assert!(
            (y + lo).abs() < 1e-6,
            "the lowest voxel on y = 0: {}",
            y + lo
        );
        assert!((y + hi - 0.3).abs() < 1e-6, "0.3 m tall: {}", y + hi);
        // Two models: the lowest of the two decides, both lift together.
        let two = write_placed(
            [1, 1, 3],
            &column,
            &[([0, 0, 10], None), ([5, 0, -4], None)],
        );
        let g = to_glb(
            &two,
            &VoxSettings {
                scale: 1.0,
                origin: VoxOrigin::Bottom,
            },
        )
        .unwrap();
        let doc = gltf::Gltf::from_slice(&g).unwrap();
        let ys: Vec<f32> = doc
            .document
            .nodes()
            .map(|n| n.transform().decomposed().0[1])
            .collect();
        assert_eq!(ys, vec![15.0, 1.0], "-4 - 1 is the floor: both up by 5");
    }

    #[test]
    fn vox_settings_are_read_and_bad_ones_refused() {
        let block = |text: &str| {
            og_core::ogt::Document::parse(&format!(
                "type import\nversion 1\n---\nmodel {{\n{text}\n}}\n"
            ))
            .unwrap()
            .body
            .get("model")
            .unwrap()
            .block
            .clone()
            .unwrap()
        };
        assert_eq!(VoxSettings::parse(None).unwrap(), VoxSettings::default());
        let b = block("voxScale 0.0625\nvoxOrigin bottom");
        assert_eq!(
            VoxSettings::parse(Some(&b)).unwrap(),
            VoxSettings {
                scale: 0.0625,
                origin: VoxOrigin::Bottom
            }
        );
        for (text, says) in [
            ("voxScale 0", "above 0"),
            ("voxScale -1", "above 0"),
            ("voxOrigin top", "center"),
            ("voxPivot 1", "unknown model setting 'voxPivot'"),
        ] {
            let e = VoxSettings::parse(Some(&block(text))).unwrap_err();
            assert!(e.contains(says) && e.contains("line 5"), "{text}: {e}");
        }
    }

    #[test]
    fn a_file_without_the_header_is_refused() {
        assert!(parse(b"not a vox").is_err());
        assert!(parse(b"VOX \x96\0\0\0MAIN\0\0\0\0\0\0\0\0").is_err());
    }
}
