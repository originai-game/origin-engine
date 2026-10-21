//! MagicaVoxel `.vox` → glTF: a voxel model becomes a flat-shaded, greedy-
//! meshed glTF binary in memory, which the model cook then treats like any
//! `.glb` (meshoptimizer, LODs, the node graph), so a voxel asset is a
//! model asset with vertex colours and one material.
//!
//! What is read: `SIZE`/`XYZI` model pairs (every model of a `PACK`), the
//! `RGBA` palette (MagicaVoxel's default palette when the file has none),
//! and the scene graph (`nTRN` translations, `nGRP`, `nSHP`) so a multi-part
//! file keeps each part where the editor put it; a file without a graph
//! places each model at the origin about its pivot, exactly where a graph
//! of one identity transform puts it (ogt_vox's reading; MagicaVoxel
//! centres a lone model). Rotations in `nTRN` (the `_r`
//! frame byte) are applied as the signed axis permutation they encode; one
//! with a negative determinant mirrors the model, so its triangles' winding
//! is flipped to stay counter-clockwise from outside.
//!
//! Axes: MagicaVoxel is z-up with y away from the viewer; the mesh is y-up
//! (glTF): `(x, y, z)` → `(x, z, -y)`, the model's origin at its `nTRN`
//! translation. The pivot is MagicaVoxel's: `floor(size / 2)` (ogt_vox's
//! documented convention, which vengi's importer matches), so voxel `v` of a
//! model turned by `R` at translation `T` fills the cell whose lowest corner
//! is `floor(T + R (v + 1/2 - floor(size / 2)))`; with a whole-voxel pivot
//! and a signed axis permutation that is the box `T + R ([v, v + 1] -
//! floor(size / 2))` the mesh is built from, turned or mirrored, odd sizes
//! included (`tests/fixtures/vox/pivot.vox`: five turns and mirrors of a
//! 3 x 2 x 5 model, checked voxel by voxel). The import settings' `model { voxScale
//! voxOrigin }` ([`VoxSettings`]) give the metres per voxel (default 1) and
//! the origin: `center` (the default) where the editor puts it, `bottom`
//! the whole file lifted so its lowest voxel stands on y = 0.
//!
//! Animation: MagicaVoxel's frames are models swapped in time, an `nSHP`
//! listing several models each with its `_f` key frame (shown from that
//! frame to the next key), or, in a file without a graph, several models
//! (0.98's `PACK` animation: model `i` is frame `i`). Such a file cooks to
//! a skinned glTF: each animated model skinned whole to a joint of its
//! own, and per clip a STEP scale channel per joint (1 on the frames its
//! model shows, 0 elsewhere), so `og.AnimationPlayer` plays it as any
//! skinned clip. `model { voxFps 8 voxClip walk 0 3 }` gives the frame
//! rate (default 10) and named frame ranges (default one clip `frames` of
//! every frame). `nTRN` key frames (an `_f` per frame of a transform, each
//! held until the next, as ogt_vox samples them) move what is under the
//! transform: a model whose place changes across frames is skinned to a
//! joint of its own with a STEP translation channel as well, and a key
//! that turns it starts a new mesh (a turn is a different set of voxel
//! faces), swapped in like a frame's model.
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

/// A named range of an animated `.vox`'s frames (`voxClip walk 0 3`).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxClip {
    pub name: String,
    /// The first frame and the last, both shown.
    pub first: u32,
    pub last: u32,
}

/// A `.vox` file's import settings: `model { voxScale 0.1 voxOrigin bottom }`.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxSettings {
    /// Metres per voxel (default 1).
    pub scale: f32,
    pub origin: VoxOrigin,
    /// An animated file's frames per second (default 10).
    pub fps: f32,
    /// An animated file's clips; none: one clip `frames` of every frame.
    pub clips: Vec<VoxClip>,
}

/// The frame rate of an animated `.vox` without `voxFps`.
pub const DEFAULT_FPS: f32 = 10.0;

/// The clip of every frame of an animated `.vox` without `voxClip`.
pub const DEFAULT_CLIP: &str = "frames";

impl Default for VoxSettings {
    fn default() -> Self {
        VoxSettings {
            scale: 1.0,
            origin: VoxOrigin::Center,
            fps: DEFAULT_FPS,
            clips: Vec::new(),
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
                "voxFps" => {
                    v.fps = e
                        .values
                        .first()
                        .and_then(|x| x.as_number())
                        .map(|n| n.as_f32())
                        .filter(|s| s.is_finite() && (1.0..=120.0).contains(s))
                        .ok_or_else(|| {
                            format!(
                                "line {}: voxFps takes an animated .vox's frames per second, 1 to 120 (default {DEFAULT_FPS})",
                                e.line
                            )
                        })?;
                }
                "voxClip" => {
                    let name = e.values.first().and_then(|x| {
                        x.as_word()
                            .map(str::to_string)
                            .or_else(|| x.as_str().map(str::to_string))
                    });
                    let frame = |i: usize| {
                        e.values
                            .get(i)
                            .and_then(|x| x.as_number())
                            .map(|n| n.as_f64())
                            .filter(|f| f.fract() == 0.0 && *f >= 0.0 && *f <= u32::MAX as f64)
                            .map(|f| f as u32)
                    };
                    match (name, frame(1), frame(2)) {
                        (Some(name), Some(first), Some(last))
                            if first <= last && e.values.len() == 3 =>
                        {
                            if v.clips.iter().any(|c| c.name == name) {
                                return Err(format!("line {}: a second voxClip '{name}'", e.line));
                            }
                            v.clips.push(VoxClip { name, first, last });
                        }
                        _ => {
                            return Err(format!(
                                "line {}: voxClip takes a name and an animated .vox's first and last frame (`voxClip walk 0 3`, both shown)",
                                e.line
                            ));
                        }
                    }
                }
                k => {
                    return Err(format!(
                        "line {}: unknown model setting '{k}' for a .vox (voxScale, voxOrigin, voxFps, voxClip)",
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
    /// The animation frames this model shows (`None`: every frame, a static
    /// part): one model of an animated shape (`nSHP` of several models,
    /// each from its `_f` key frame to the next), a run of frames over
    /// which a keyed transform (`nTRN` of several frames) keeps the model
    /// and its rotation, or one model of a file without a scene graph that
    /// holds several (MagicaVoxel 0.98's animation, a `PACK` of frames).
    pub frames: Option<std::ops::Range<u32>>,
    /// Where a keyed transform moves the model over `frames`: its
    /// translation on each of them (`translation` is the first), or empty
    /// when it stays put.
    pub track: Vec<[i32; 3]>,
}

/// A parsed `.vox`: its models, palette (256 RGBA, index 0 unused) and
/// placements.
#[derive(Clone, Debug)]
pub struct VoxFile {
    pub models: Vec<VoxModel>,
    pub palette: [[u8; 4]; 256],
    pub placements: Vec<Placement>,
    /// Animation frames (0: the file is static): the last key frame of any
    /// animated shape or keyed transform plus one, or a graph-less file's
    /// model count.
    pub frame_count: u32,
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

/// The 4 x 4 identity, column-major.
const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

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
    // A transform's key frame: (frame, translation, rotation).
    type Key = (u32, [i32; 3], [(usize, i32); 3]);
    // The scene graph, by node id.
    enum Node {
        /// Its key frames, sorted by frame and at least one: (frame,
        /// translation, rotation), each holding until the next.
        Transform {
            child: u32,
            keys: Vec<Key>,
        },
        Group(Vec<u32>),
        /// Its models, each with its `_f` key frame when it has one.
        Shape(Vec<(u32, Option<u32>)>),
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
                // Each frame a key: its `_f` (else its place in the list),
                // `_t` and `_r`; MagicaVoxel holds a key until the next.
                let mut keys = Vec::new();
                for f in 0..frames {
                    let d = dict(body, &mut p)?;
                    let mut translation = [0; 3];
                    let mut rotation = IDENTITY;
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
                    let key = d.get("_f").and_then(|k| k.parse().ok()).unwrap_or(f);
                    keys.push((key, translation, rotation));
                }
                // The first key listed for a frame wins.
                keys.sort_by_key(|k| k.0);
                keys.dedup_by_key(|k| k.0);
                if keys.is_empty() {
                    keys.push((0, [0; 3], IDENTITY));
                }
                nodes.insert(node, Node::Transform { child, keys });
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
                    let model = u32_at(body, p)?;
                    p += 4;
                    let attrs = dict(body, &mut p)?;
                    ms.push((model, attrs.get("_f").and_then(|f| f.parse().ok())));
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
    // What one shape shows at a frame: per model of the graph's walk (in
    // walk order, the same at every frame), the model shown (none before an
    // animated shape's first key) and where.
    struct Shown {
        model: Option<usize>,
        translation: [i32; 3],
        rotation: [(usize, i32); 3],
    }
    #[allow(clippy::too_many_arguments)]
    fn walk(
        nodes: &BTreeMap<u32, Node>,
        id: u32,
        t: [i32; 3],
        r: [(usize, i32); 3],
        frame: u32,
        out: &mut Vec<Shown>,
        depth: u32,
        models: usize,
    ) {
        if depth > 64 {
            return;
        }
        match nodes.get(&id) {
            Some(Node::Transform { child, keys }) => {
                // The key in force: the last at or before the frame, the
                // first before any.
                let k = keys.iter().rev().find(|k| k.0 <= frame).unwrap_or(&keys[0]);
                let (translation, rotation) = (k.1, k.2);
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
                walk(nodes, *child, tt, rr, frame, out, depth + 1, models);
            }
            Some(Node::Group(children)) => {
                for c in children {
                    walk(nodes, *c, t, r, frame, out, depth + 1, models);
                }
            }
            Some(Node::Shape(ms)) if ms.len() == 1 => {
                out.push(Shown {
                    model: Some(ms[0].0 as usize).filter(|m| *m < models),
                    translation: t,
                    rotation: r,
                });
            }
            Some(Node::Shape(ms)) => {
                // An animated shape: each model from its key frame (`_f`,
                // else its position in the list) to the next model's.
                let mut keyed: Vec<(u32, u32)> = ms
                    .iter()
                    .enumerate()
                    .map(|(i, (m, f))| (f.unwrap_or(i as u32), *m))
                    .collect();
                keyed.sort();
                keyed.dedup_by_key(|k| k.0);
                let model = keyed
                    .iter()
                    .rev()
                    .find(|k| k.0 <= frame)
                    .map(|k| k.1 as usize)
                    .filter(|m| *m < models);
                out.push(Shown {
                    model,
                    translation: t,
                    rotation: r,
                });
            }
            None => {}
        }
    }
    if nodes.contains_key(&0) {
        // The frames: up to the last key of any animated shape or keyed
        // transform.
        let last_key = nodes
            .values()
            .filter_map(|n| match n {
                Node::Transform { keys, .. } if keys.len() > 1 => keys.last().map(|k| k.0),
                Node::Shape(ms) if ms.len() > 1 => ms
                    .iter()
                    .enumerate()
                    .map(|(i, (_, f))| f.unwrap_or(i as u32))
                    .max(),
                _ => None,
            })
            .max();
        let frames = last_key.map_or(1, |k| k.saturating_add(1));
        let mut at: Vec<Vec<Shown>> = Vec::with_capacity(frames as usize);
        for f in 0..frames {
            let mut shown = Vec::new();
            walk(&nodes, 0, [0; 3], IDENTITY, f, &mut shown, 0, models.len());
            at.push(shown);
        }
        for i in 0..at[0].len() {
            let first = &at[0][i];
            let still = at.iter().all(|f| {
                let s = &f[i];
                s.model == first.model
                    && s.translation == first.translation
                    && s.rotation == first.rotation
            });
            if still {
                if let Some(model) = first.model {
                    placements.push(Placement {
                        model,
                        translation: first.translation,
                        rotation: first.rotation,
                        frames: None,
                        track: Vec::new(),
                    });
                }
                continue;
            }
            // Runs of frames that show one model at one rotation (a turn is
            // a mesh of its own); the translation may move within a run.
            let mut f = 0;
            while f < frames {
                let s = &at[f as usize][i];
                let Some(model) = s.model else {
                    f += 1;
                    continue;
                };
                let mut end = f + 1;
                while end < frames {
                    let n = &at[end as usize][i];
                    if n.model != Some(model) || n.rotation != s.rotation {
                        break;
                    }
                    end += 1;
                }
                let track: Vec<[i32; 3]> =
                    (f..end).map(|k| at[k as usize][i].translation).collect();
                let moves = track.iter().any(|t| *t != s.translation);
                placements.push(Placement {
                    model,
                    translation: s.translation,
                    rotation: s.rotation,
                    frames: Some(f..end),
                    track: if moves { track } else { Vec::new() },
                });
                f = end;
            }
        }
    }
    // Without a graph, every model sits at the origin about its pivot,
    // exactly as under a graph of one identity transform (ogt_vox's
    // reading, and MagicaVoxel's: a lone model is centred). Several models
    // without a graph are MagicaVoxel 0.98's animation (a `PACK` of
    // frames): model `i` is frame `i`.
    if placements.is_empty() {
        let animated = models.len() > 1;
        for i in 0..models.len() {
            placements.push(Placement {
                model: i,
                translation: [0; 3],
                rotation: IDENTITY,
                frames: animated.then_some(i as u32..i as u32 + 1),
                track: Vec::new(),
            });
        }
    }
    let frame_count = placements
        .iter()
        .filter_map(|p| p.frames.as_ref())
        .map(|f| f.end)
        .max()
        .unwrap_or(0);
    Ok(VoxFile {
        models,
        palette,
        placements,
        frame_count,
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

/// The glTF binary chunk being written: its bytes, buffer views and
/// accessors.
#[derive(Default)]
struct Bin {
    bytes: Vec<u8>,
    views: Vec<serde_json::Value>,
    accessors: Vec<serde_json::Value>,
}

impl Bin {
    /// Appends `data` (4-aligned) as a buffer view and an accessor over it
    /// (`accessor` gives everything but the view); answers the accessor.
    fn add(&mut self, data: &[u8], target: Option<u32>, mut accessor: serde_json::Value) -> usize {
        while !self.bytes.len().is_multiple_of(4) {
            self.bytes.push(0);
        }
        let mut view = serde_json::json!({
            "buffer": 0, "byteOffset": self.bytes.len(), "byteLength": data.len()
        });
        if let Some(t) = target {
            view["target"] = t.into();
        }
        self.bytes.extend_from_slice(data);
        self.views.push(view);
        accessor["bufferView"] = (self.views.len() - 1).into();
        self.accessors.push(accessor);
        self.accessors.len() - 1
    }
}

/// The clips of an animated file: (name, first frame, last frame).
fn clips_of(file: &VoxFile, settings: &VoxSettings) -> Result<Vec<(String, u32, u32)>, String> {
    let n = file.frame_count;
    if n == 0 {
        return match settings.clips.first() {
            Some(c) => Err(format!(
                "voxClip '{}': the file has no animation frames (an animated shape lists several models, each with its `_f` key frame)",
                c.name
            )),
            None => Ok(Vec::new()),
        };
    }
    if settings.clips.is_empty() {
        return Ok(vec![(DEFAULT_CLIP.to_string(), 0, n - 1)]);
    }
    settings
        .clips
        .iter()
        .map(|c| {
            if c.last >= n {
                Err(format!(
                    "voxClip '{}' ends at frame {}; the file has {n} frame(s), 0 to {}",
                    c.name,
                    c.last,
                    n - 1
                ))
            } else {
                Ok((c.name.clone(), c.first, c.last))
            }
        })
        .collect()
}

/// A `.vox` as a glTF binary: one mesh per placed model under a node at
/// its translation, each a flat-shaded greedy mesh with vertex colours and
/// the shared white material, `settings.scale` metres per voxel, its
/// origin where `settings.origin` says.
///
/// An animated file (frames: [`VoxFile::frame_count`]) is a skinned glTF
/// whose clips swap meshes: each model of an animated shape is skinned
/// whole to a joint of its own (at its translation, inverse bind the
/// identity), and each clip is a STEP scale channel per joint, 1 on the
/// frames the model shows and 0 elsewhere, at `settings.fps` (one key per
/// frame, and one at the clip's end holding its last frame). A model a
/// keyed transform moves ([`Placement::track`]) also has a STEP translation
/// channel on its joint, its place on each frame (held from its nearest
/// shown frame where it is folded away). The rest pose is frame 0, and the
/// mesh nodes carry the same scale, so the model drawn without a skeleton
/// shows frame 0 too.
pub fn to_glb(bytes: &[u8], settings: &VoxSettings) -> Result<Vec<u8>, String> {
    let file = parse(bytes)?;
    let clips = clips_of(&file, settings)?;
    let scale = settings.scale;
    let mut bin = Bin::default();
    let mut meshes = Vec::new();
    // Per placed model: its index, its mesh, its translation (glTF frame,
    // metres), the frames it shows (animated only) and its translation on
    // each of them (empty: it stays put).
    type Placed = (
        usize,
        usize,
        [f32; 3],
        Option<std::ops::Range<u32>>,
        Vec<[f32; 3]>,
    );
    let mut placed: Vec<Placed> = Vec::new();
    let mut joints = 0u16;
    let mut floor = f32::MAX;
    let meshed: Vec<Vec<Quad>> = file.models.iter().map(mesh_model).collect();
    for (pi, placement) in file.placements.iter().enumerate() {
        let model = &file.models[placement.model];
        let quads = &meshed[placement.model];
        if quads.is_empty() {
            continue;
        }
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
        let count = positions.len();
        let pos = bin.add(
            bytemuck::cast_slice(&positions),
            Some(34962),
            serde_json::json!({
                "componentType": 5126, "count": count, "type": "VEC3", "min": lo, "max": hi
            }),
        );
        let nrm = bin.add(
            bytemuck::cast_slice(&normals),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC3" }),
        );
        let col = bin.add(
            bytemuck::cast_slice(&colors),
            Some(34962),
            serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
        );
        let idx = bin.add(
            bytemuck::cast_slice(&indices),
            Some(34963),
            serde_json::json!({ "componentType": 5125, "count": indices.len(), "type": "SCALAR" }),
        );
        let mut attributes = serde_json::json!({ "POSITION": pos, "NORMAL": nrm, "COLOR_0": col });
        if placement.frames.is_some() {
            // Skinned whole to its own joint.
            let j: Vec<[u16; 4]> = vec![[joints, 0, 0, 0]; count];
            let w: Vec<[f32; 4]> = vec![[1.0, 0.0, 0.0, 0.0]; count];
            attributes["JOINTS_0"] = bin
                .add(
                    bytemuck::cast_slice(&j),
                    Some(34962),
                    serde_json::json!({ "componentType": 5123, "count": count, "type": "VEC4" }),
                )
                .into();
            attributes["WEIGHTS_0"] = bin
                .add(
                    bytemuck::cast_slice(&w),
                    Some(34962),
                    serde_json::json!({ "componentType": 5126, "count": count, "type": "VEC4" }),
                )
                .into();
            joints += 1;
        }
        meshes.push(serde_json::json!({
            "name": format!("vox{pi}"),
            "primitives": [{ "attributes": attributes, "indices": idx, "material": 0 }]
        }));
        // MagicaVoxel (x, y, z) z-up → glTF (x, z, -y) y-up, in metres.
        let to_gltf = |t: [i32; 3]| {
            [
                t[0] as f32 * scale,
                t[2] as f32 * scale,
                -(t[1] as f32) * scale,
            ]
        };
        let at = to_gltf(placement.translation);
        let track: Vec<[f32; 3]> = placement.track.iter().map(|t| to_gltf(*t)).collect();
        for p in std::iter::once(&at).chain(&track) {
            floor = floor.min(p[1] + lo[1]);
        }
        placed.push((pi, meshes.len() - 1, at, placement.frames.clone(), track));
    }
    if meshes.is_empty() {
        return Err("vox: every model is empty".into());
    }
    // `bottom`: every node lifted by the same height, the lowest voxel on 0.
    let lift = match settings.origin {
        VoxOrigin::Center => 0.0,
        VoxOrigin::Bottom => -floor,
    };
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    // Per joint: its node, the frames its model shows and its translation
    // on each (lifted; empty: it stays put).
    type Joint = (usize, std::ops::Range<u32>, Vec<[f32; 3]>);
    let mut joint_nodes: Vec<Joint> = Vec::new();
    for (pi, mesh, at, frames, track) in &placed {
        let at = [at[0], at[1] + lift, at[2]];
        match frames {
            None => nodes.push(serde_json::json!({
                "name": format!("vox{pi}"), "mesh": mesh, "translation": at
            })),
            Some(f) => {
                let s = if f.contains(&0) { 1.0 } else { 0.0 };
                nodes.push(serde_json::json!({
                    "name": format!("vox{pi}"), "mesh": mesh, "skin": 0,
                    "translation": at, "scale": [s, s, s]
                }));
                nodes.push(serde_json::json!({
                    "name": format!("vox{pi}-frames"), "translation": at, "scale": [s, s, s]
                }));
                let track = track.iter().map(|t| [t[0], t[1] + lift, t[2]]).collect();
                joint_nodes.push((nodes.len() - 1, f.clone(), track));
            }
        }
    }
    let mut json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "og-cook vox" },
        "scene": 0,
        "scenes": [{ "nodes": (0..nodes.len()).collect::<Vec<_>>() }],
        "nodes": nodes,
        "meshes": meshes,
        "materials": [{
            "name": "voxels",
            "pbrMetallicRoughness": { "baseColorFactor": [1.0, 1.0, 1.0, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.9 }
        }],
    });
    if !joint_nodes.is_empty() {
        let identity: Vec<f32> = (0..joint_nodes.len()).flat_map(|_| IDENTITY_MAT4).collect();
        let ibm = bin.add(
            bytemuck::cast_slice(&identity),
            None,
            serde_json::json!({ "componentType": 5126, "count": joint_nodes.len(), "type": "MAT4" }),
        );
        json["skins"] = serde_json::json!([{
            "name": "frames",
            "joints": joint_nodes.iter().map(|(n, ..)| *n).collect::<Vec<_>>(),
            "inverseBindMatrices": ibm
        }]);
        let mut animations = Vec::new();
        for (name, first, last) in &clips {
            // One key per frame, and one at the end holding the last.
            let len = last - first + 1;
            let times: Vec<f32> = (0..=len).map(|i| i as f32 / settings.fps).collect();
            let input = bin.add(
                bytemuck::cast_slice(&times),
                None,
                serde_json::json!({
                    "componentType": 5126, "count": times.len(), "type": "SCALAR",
                    "min": [0.0], "max": [times[times.len() - 1]]
                }),
            );
            let mut samplers = Vec::new();
            let mut channels = Vec::new();
            for (node, frames, track) in &joint_nodes {
                let scales: Vec<[f32; 3]> = (0..=len)
                    .map(|i| {
                        let shown = frames.contains(&(first + i.min(len - 1)));
                        [if shown { 1.0 } else { 0.0 }; 3]
                    })
                    .collect();
                let output = bin.add(
                    bytemuck::cast_slice(&scales),
                    None,
                    serde_json::json!({ "componentType": 5126, "count": scales.len(), "type": "VEC3" }),
                );
                channels.push(serde_json::json!({
                    "sampler": samplers.len(), "target": { "node": node, "path": "scale" }
                }));
                samplers.push(serde_json::json!({
                    "input": input, "output": output, "interpolation": "STEP"
                }));
                if track.is_empty() {
                    continue;
                }
                // Where the keyed transform puts the model on each frame;
                // on a frame it is folded away, where it was or will be
                // nearest (a fold is not a move).
                let places: Vec<[f32; 3]> = (0..=len)
                    .map(|i| {
                        let f = (first + i.min(len - 1)).clamp(frames.start, frames.end - 1);
                        track[(f - frames.start) as usize]
                    })
                    .collect();
                let output = bin.add(
                    bytemuck::cast_slice(&places),
                    None,
                    serde_json::json!({ "componentType": 5126, "count": places.len(), "type": "VEC3" }),
                );
                channels.push(serde_json::json!({
                    "sampler": samplers.len(), "target": { "node": node, "path": "translation" }
                }));
                samplers.push(serde_json::json!({
                    "input": input, "output": output, "interpolation": "STEP"
                }));
            }
            animations.push(serde_json::json!({
                "name": name, "samplers": samplers, "channels": channels
            }));
        }
        json["animations"] = animations.into();
    }
    while !bin.bytes.len().is_multiple_of(4) {
        bin.bytes.push(0);
    }
    json["accessors"] = std::mem::take(&mut bin.accessors).into();
    json["bufferViews"] = std::mem::take(&mut bin.views).into();
    json["buffers"] = serde_json::json!([{ "byteLength": bin.bytes.len() }]);
    let bin = bin.bytes;
    let mut json_bytes = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
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
        write_placed_with(size, voxels, None, placements)
    }

    /// [`write_placed`] with a palette (`RGBA`, entry 0 for index 1).
    fn write_placed_with(
        size: [u32; 3],
        voxels: &[[u8; 4]],
        palette: Option<&[[u8; 4]]>,
        placements: &[([i32; 3], Option<u8>)],
    ) -> Vec<u8> {
        let mut out = write(size, voxels, palette);
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

    /// The pivot fixture (`tests/fixtures/vox/pivot.vox`, made by
    /// [`pivot_fixture`]): one 3 x 2 x 5 model (odd in x and z) of five
    /// isolated voxels, each its own colour, placed five times: as is, turned
    /// 90 degrees about z (`_r` 17), mirrored in x (20), turned 180 degrees
    /// about y (84) and the axes cycled (9), each at its own translation.
    const PIVOT_VOXELS: [[u8; 4]; 5] = [
        [0, 0, 0, 1],
        [2, 0, 0, 2],
        [0, 1, 2, 3],
        [1, 0, 3, 4],
        [2, 1, 4, 5],
    ];
    const PIVOT_PLACEMENTS: [([i32; 3], Option<u8>); 5] = [
        ([0, 0, 0], None),
        ([10, 0, 0], Some(17)),
        ([0, 10, 0], Some(20)),
        ([0, 0, 10], Some(84)),
        ([-10, 3, -7], Some(9)),
    ];

    fn pivot_fixture() -> Vec<u8> {
        let palette: Vec<[u8; 4]> = (1..=5u8)
            .map(|i| [i * 40, 255 - i * 40, i * 20, 255])
            .collect();
        write_placed_with([3, 2, 5], &PIVOT_VOXELS, Some(&palette), &PIVOT_PLACEMENTS)
    }

    /// MagicaVoxel's placement of a voxel (ogt_vox's documented pivot, which
    /// vengi's importer matches): a model's pivot is `floor(size / 2)`, and
    /// voxel `v` of a model turned by `R` at translation `T` occupies the
    /// world cell whose lowest corner is `floor(T + R (v + 1/2 - pivot))`.
    /// The pivot is a whole voxel and `R` a signed axis permutation, so the
    /// centre lands on a half and the cell is `T + R (v + 1/2 - pivot) - 1/2`.
    fn magicavoxel_cell(size: [u32; 3], v: [u8; 3], t: [i32; 3], r: [(usize, i32); 3]) -> [i32; 3] {
        let centre = |i: usize| 2 * v[i] as i32 + 1 - 2 * (size[i] / 2) as i32;
        std::array::from_fn(|i| {
            let turned = centre(r[i].0) * r[i].1;
            (2 * t[i] + turned).div_euclid(2)
        })
    }

    /// Every voxel of every turned or mirrored placement of an odd-sized
    /// model lands where MagicaVoxel puts it: the committed fixture (made
    /// by the test writer) is meshed and each colour's box compared with
    /// [`magicavoxel_cell`] taken to the glTF frame.
    #[test]
    fn turned_and_mirrored_odd_models_keep_magicavoxels_pivot() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vox/pivot.vox");
        let made = pivot_fixture();
        if std::env::var_os("OG_WRITE_FIXTURES").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &made).unwrap();
        }
        let bytes = std::fs::read(&path).expect("the committed pivot fixture");
        assert_eq!(bytes, made, "the fixture is the test writer's");
        let file = parse(&bytes).unwrap();
        assert_eq!(file.placements.len(), PIVOT_PLACEMENTS.len());
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let nodes: Vec<_> = g.document.nodes().collect();
        assert_eq!(nodes.len(), PIVOT_PLACEMENTS.len());
        let palette = &file.palette;
        for (node, (placement, (t, r))) in nodes
            .iter()
            .zip(file.placements.iter().zip(PIVOT_PLACEMENTS))
        {
            let rot = r.map_or(IDENTITY, rotation_of);
            assert_eq!(placement.rotation, rot);
            let at = glam::Vec3::from(node.transform().decomposed().0);
            let prim = node.mesh().unwrap().primitives().next().unwrap();
            let reader = prim.reader(|_| Some(&blob[..]));
            let pos: Vec<glam::Vec3> = reader
                .read_positions()
                .unwrap()
                .map(glam::Vec3::from)
                .collect();
            let col: Vec<[f32; 4]> = reader.read_colors(0).unwrap().into_rgba_f32().collect();
            for v in PIVOT_VOXELS {
                let c = palette[v[3] as usize];
                let want = [linear(c[0]), linear(c[1]), linear(c[2])];
                let (mut lo, mut hi) = (glam::Vec3::MAX, glam::Vec3::MIN);
                for (p, k) in pos.iter().zip(&col) {
                    if (0..3).all(|i| (k[i] - want[i]).abs() < 1e-6) {
                        lo = lo.min(at + *p);
                        hi = hi.max(at + *p);
                    }
                }
                let m = magicavoxel_cell([3, 2, 5], [v[0], v[1], v[2]], t, rot);
                // MagicaVoxel's cell [m, m + 1] in glTF's frame: (x, z, -y).
                let cell_lo = glam::Vec3::new(m[0] as f32, m[2] as f32, -(m[1] as f32) - 1.0);
                assert_eq!(
                    (lo, hi),
                    (cell_lo, cell_lo + glam::Vec3::ONE),
                    "voxel {v:?} of the placement at {t:?} turned by _r {r:?}"
                );
            }
        }
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
            ..VoxSettings::default()
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
            ..VoxSettings::default()
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
                ..VoxSettings::default()
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
                origin: VoxOrigin::Bottom,
                ..VoxSettings::default()
            }
        );
        for (text, says) in [
            ("voxScale 0", "above 0"),
            ("voxScale -1", "above 0"),
            ("voxOrigin top", "center"),
            ("voxPivot 1", "unknown model setting 'voxPivot'"),
            ("voxFps 0", "1 to 120"),
            ("voxClip walk 3 1", "first and last frame"),
            ("voxClip walk 0", "first and last frame"),
            ("voxClip walk 0 1.5", "first and last frame"),
            (
                "voxClip walk 0 1\nvoxClip walk 2 3",
                "line 6: a second voxClip 'walk'",
            ),
        ] {
            let e = VoxSettings::parse(Some(&block(text))).unwrap_err();
            assert!(e.contains(says) && e.contains("line "), "{text}: {e}");
            if !says.starts_with("line") {
                assert!(e.contains("line 5"), "{text}: {e}");
            }
        }
    }

    #[test]
    fn frame_rate_and_clips_are_read() {
        let block = og_core::ogt::Document::parse(
            "type import\nversion 1\n---\nmodel {\nvoxFps 4\nvoxClip idle 0 1\nvoxClip \"jump high\" 2 5\n}\n",
        )
        .unwrap()
        .body
        .get("model")
        .unwrap()
        .block
        .clone()
        .unwrap();
        let v = VoxSettings::parse(Some(&block)).unwrap();
        assert_eq!(v.fps, 4.0);
        let clips: Vec<(&str, u32, u32)> = v
            .clips
            .iter()
            .map(|c| (c.name.as_str(), c.first, c.last))
            .collect();
        assert_eq!(clips, vec![("idle", 0, 1), ("jump high", 2, 5)]);
    }

    fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
        out.extend_from_slice(id);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(body);
    }

    fn dict_of(b: &mut Vec<u8>, pairs: &[(&str, String)]) {
        b.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
        for (k, v) in pairs {
            for s in [*k, v.as_str()] {
                b.extend_from_slice(&(s.len() as u32).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            }
        }
    }

    /// A `.vox` of several models (each `SIZE`/`XYZI`) and then `graph`'s
    /// chunks (none: no scene graph).
    fn write_models(models: &[([u32; 3], Vec<[u8; 4]>)], graph: &[u8]) -> Vec<u8> {
        let mut children = Vec::new();
        let mut pack = Vec::new();
        pack.extend_from_slice(&(models.len() as u32).to_le_bytes());
        chunk(&mut children, b"PACK", &pack);
        for (size, voxels) in models {
            let body: Vec<u8> = size.iter().flat_map(|s| s.to_le_bytes()).collect();
            chunk(&mut children, b"SIZE", &body);
            let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
            for v in voxels {
                xyzi.extend_from_slice(v);
            }
            chunk(&mut children, b"XYZI", &xyzi);
        }
        children.extend_from_slice(graph);
        let mut out = b"VOX ".to_vec();
        out.extend_from_slice(&150u32.to_le_bytes());
        out.extend_from_slice(b"MAIN");
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(children.len() as u32).to_le_bytes());
        out.extend_from_slice(&children);
        out
    }

    fn trn(out: &mut Vec<u8>, node: u32, child: u32, t: [i32; 3]) {
        let mut b = Vec::new();
        for v in [node, 0, child, u32::MAX, 0, 1] {
            if v == 0 && b.len() == 4 {
                dict_of(&mut b, &[]);
            } else {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        dict_of(&mut b, &[("_t", format!("{} {} {}", t[0], t[1], t[2]))]);
        chunk(out, b"nTRN", &b);
    }

    /// An `nTRN` of several frames: per key its `_f`, `_t` and `_r`.
    fn trn_keys(out: &mut Vec<u8>, node: u32, child: u32, keys: &[(u32, [i32; 3], Option<u8>)]) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, &[]);
        for v in [child, u32::MAX, 0, keys.len() as u32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for (f, t, r) in keys {
            let mut frame = vec![
                ("_f", f.to_string()),
                ("_t", format!("{} {} {}", t[0], t[1], t[2])),
            ];
            if let Some(r) = r {
                frame.push(("_r", r.to_string()));
            }
            dict_of(&mut b, &frame);
        }
        chunk(out, b"nTRN", &b);
    }

    fn grp(out: &mut Vec<u8>, node: u32, children: &[u32]) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, &[]);
        b.extend_from_slice(&(children.len() as u32).to_le_bytes());
        for c in children {
            b.extend_from_slice(&c.to_le_bytes());
        }
        chunk(out, b"nGRP", &b);
    }

    fn shp(out: &mut Vec<u8>, node: u32, models: &[(u32, Option<u32>)]) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, &[]);
        b.extend_from_slice(&(models.len() as u32).to_le_bytes());
        for (m, f) in models {
            b.extend_from_slice(&m.to_le_bytes());
            match f {
                Some(f) => dict_of(&mut b, &[("_f", f.to_string())]),
                None => dict_of(&mut b, &[]),
            }
        }
        chunk(out, b"nSHP", &b);
    }

    /// A solid box of `size` in colour `c`.
    fn solid(size: [u32; 3], c: u8) -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for z in 0..size[2] as u8 {
            for y in 0..size[1] as u8 {
                for x in 0..size[0] as u8 {
                    v.push([x, y, z, c]);
                }
            }
        }
        v
    }

    /// The world-space bounds of a glTF binary's default scene (node
    /// translations and scales applied, as the model cook computes them).
    fn world_bounds(glb: &[u8]) -> (glam::Vec3, glam::Vec3) {
        let g = gltf::Gltf::from_slice(glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let (mut lo, mut hi) = (glam::Vec3::MAX, glam::Vec3::MIN);
        for n in g.document.nodes() {
            let Some(mesh) = n.mesh() else { continue };
            let m = glam::Mat4::from_cols_array_2d(&n.transform().matrix());
            for p in mesh.primitives() {
                let r = p.reader(|_| Some(&blob[..]));
                for v in r.read_positions().unwrap() {
                    let w = m.transform_point3(glam::Vec3::from(v));
                    lo = lo.min(w);
                    hi = hi.max(w);
                }
            }
        }
        (lo, hi)
    }

    /// A file without a scene graph lands exactly where the same voxels
    /// under a graph of one transform at the origin land (MagicaVoxel
    /// centres a lone model; the old reading put its corner at the
    /// origin), even and odd sizes, with either origin.
    #[test]
    fn a_file_without_a_graph_lands_where_a_trivial_graph_puts_it() {
        for size in [[4, 2, 6], [3, 2, 5], [1, 1, 1]] {
            let voxels = solid(size, 9);
            let bare = write(size, &voxels, None);
            let graphed = write_placed(size, &voxels, &[([0, 0, 0], None)]);
            for origin in [VoxOrigin::Center, VoxOrigin::Bottom] {
                let settings = VoxSettings {
                    scale: 0.5,
                    origin,
                    ..VoxSettings::default()
                };
                let a = to_glb(&bare, &settings).unwrap();
                let b = to_glb(&graphed, &settings).unwrap();
                assert_eq!(world_bounds(&a), world_bounds(&b), "{size:?} {origin:?}");
                assert_eq!(a, b, "{size:?} {origin:?}: the same glTF");
            }
            // Centred: MagicaVoxel's pivot floor(size / 2) at the origin.
            let (lo, hi) = world_bounds(&to_glb(&bare, &VoxSettings::default()).unwrap());
            let p = size.map(|s| (s / 2) as f32);
            let want_lo = glam::Vec3::new(-p[0], -p[2], -(size[1] as f32 - p[1]));
            assert_eq!(lo, want_lo, "{size:?}");
            assert_eq!(
                hi - lo,
                glam::Vec3::new(size[0] as f32, size[2] as f32, size[1] as f32)
            );
        }
    }

    /// Several models without a graph are MagicaVoxel 0.98's animation: one
    /// frame each, all at the origin, cooked as a skinned glTF whose clip
    /// `frames` scales each frame's joint to 1 on its frame and 0 elsewhere.
    #[test]
    fn a_pack_without_a_graph_is_the_frames_of_one_clip() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 4], solid([2, 2, 4], 2)),
            ([4, 2, 2], solid([4, 2, 2], 3)),
        ];
        let bytes = write_models(&models, &[]);
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 3);
        let frames: Vec<_> = file.placements.iter().map(|p| p.frames.clone()).collect();
        assert_eq!(frames, vec![Some(0..1), Some(1..2), Some(2..3)]);
        assert!(file.placements.iter().all(|p| p.translation == [0; 3]));
        let settings = VoxSettings {
            fps: 5.0,
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let skin = g.document.skins().next().expect("a skin");
        assert_eq!(skin.joints().count(), 3);
        // Rest pose and mesh nodes: frame 0 shown, the others folded away.
        let scales: Vec<[f32; 3]> = g
            .document
            .nodes()
            .map(|n| n.transform().decomposed().2)
            .collect();
        assert_eq!(
            scales,
            vec![[1.0; 3], [1.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]]
        );
        assert!(
            g.document
                .nodes()
                .filter(|n| n.mesh().is_some())
                .all(|n| n.skin().is_some())
        );
        let anims: Vec<_> = g.document.animations().collect();
        assert_eq!(anims.len(), 1);
        assert_eq!(anims[0].name(), Some(DEFAULT_CLIP));
        let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
        for (k, ch) in anims[0].channels().enumerate() {
            assert_eq!(ch.target().node().index(), joints[k]);
            assert_eq!(ch.target().property(), gltf::animation::Property::Scale);
            assert_eq!(
                ch.sampler().interpolation(),
                gltf::animation::Interpolation::Step
            );
            let r = ch.reader(|_| Some(&blob[..]));
            let times: Vec<f32> = r.read_inputs().unwrap().collect();
            assert_eq!(times, vec![0.0, 0.2, 0.4, 0.6]);
            let Some(gltf::animation::util::ReadOutputs::Scales(s)) = r.read_outputs() else {
                panic!("scale outputs");
            };
            let shown: Vec<f32> = s.map(|v| v[0]).collect();
            // Frame k only; the end key holds the last frame.
            let mut want = vec![0.0; 4];
            want[k] = 1.0;
            want[3] = want[2];
            assert_eq!(shown, want, "joint {k}");
        }
        // Every frame meshes into its own primitive, faces out.
        assert_eq!(primitives(&glb).len(), 3);
        assert!(faces_out(&glb));
    }

    /// An animated shape (an `nSHP` of several models with `_f` key frames)
    /// shows each model from its key to the next, beside a static part;
    /// `voxClip` cuts the frames into named clips.
    #[test]
    fn an_animated_shape_keys_its_models_and_clips_cut_its_frames() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 3], solid([2, 2, 3], 2)),
            ([6, 6, 1], solid([6, 6, 1], 3)),
        ];
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4]);
        trn(&mut graph, 2, 3, [0, 0, 4]);
        shp(&mut graph, 3, &[(1, Some(4)), (0, Some(0))]);
        trn(&mut graph, 4, 5, [0, 0, 0]);
        shp(&mut graph, 5, &[(2, None)]);
        let bytes = write_models(&models, &graph);
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 5);
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| (p.model, p.translation, p.frames.clone()))
            .collect();
        assert_eq!(
            placed,
            vec![
                (0, [0, 0, 4], Some(0..4)),
                (1, [0, 0, 4], Some(4..5)),
                (2, [0, 0, 0], None)
            ]
        );
        let settings = VoxSettings {
            clips: vec![
                VoxClip {
                    name: "idle".into(),
                    first: 0,
                    last: 3,
                },
                VoxClip {
                    name: "pop".into(),
                    first: 3,
                    last: 4,
                },
            ],
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        // The static part is not skinned.
        let skinned: Vec<bool> = g
            .document
            .nodes()
            .filter(|n| n.mesh().is_some())
            .map(|n| n.skin().is_some())
            .collect();
        assert_eq!(skinned, vec![true, true, false]);
        let shown = |anim: &gltf::Animation| -> Vec<Vec<f32>> {
            anim.channels()
                .map(|ch| {
                    let r = ch.reader(|_| Some(&blob[..]));
                    match r.read_outputs() {
                        Some(gltf::animation::util::ReadOutputs::Scales(s)) => {
                            s.map(|v| v[0]).collect()
                        }
                        _ => panic!("scale outputs"),
                    }
                })
                .collect()
        };
        let anims: Vec<_> = g.document.animations().collect();
        assert_eq!(anims[0].name(), Some("idle"));
        assert_eq!(
            shown(&anims[0]),
            vec![vec![1.0; 5], vec![0.0; 5]],
            "idle: model 0 throughout"
        );
        assert_eq!(anims[1].name(), Some("pop"));
        assert_eq!(
            shown(&anims[1]),
            vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 1.0]]
        );
        // A clip past the last frame, or on a static file, is refused.
        let past = VoxSettings {
            clips: vec![VoxClip {
                name: "far".into(),
                first: 2,
                last: 5,
            }],
            ..VoxSettings::default()
        };
        let e = to_glb(&bytes, &past).unwrap_err();
        assert!(e.contains("'far'") && e.contains("5 frame(s)"), "{e}");
        let still = write([1, 1, 1], &[[0, 0, 0, 1]], None);
        let e = to_glb(&still, &past).unwrap_err();
        assert!(e.contains("no animation frames"), "{e}");
        // A static file stays unskinned and without clips.
        let g = gltf::Gltf::from_slice(&to_glb(&still, &VoxSettings::default()).unwrap()).unwrap();
        assert_eq!(
            (g.document.skins().count(), g.document.animations().count()),
            (0, 0)
        );
    }

    /// A file whose transforms carry key frames (owed row 63): a cube
    /// keyed to move between frames 0 and 2, a bar keyed to turn at frame 1
    /// and move at frame 2, and a static base. Before the keys were read,
    /// each transform's first frame placed its model throughout: one
    /// static file, no clip.
    fn keyed_file() -> Vec<u8> {
        let models = vec![
            ([6, 6, 1], solid([6, 6, 1], 3)),
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([3, 1, 1], solid([3, 1, 1], 2)),
        ];
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4, 6]);
        trn(&mut graph, 2, 3, [0, 0, 0]);
        shp(&mut graph, 3, &[(0, None)]);
        // Listed out of order: the keys sort by `_f`.
        trn_keys(
            &mut graph,
            4,
            5,
            &[(2, [2, 0, 4], None), (0, [-2, 0, 2], None)],
        );
        shp(&mut graph, 5, &[(1, None)]);
        trn_keys(
            &mut graph,
            6,
            7,
            &[
                (0, [0, 3, 1], None),
                (1, [0, 3, 1], Some(17)),
                (2, [1, 3, 1], Some(17)),
            ],
        );
        shp(&mut graph, 7, &[(2, None)]);
        write_models(&models, &graph)
    }

    #[test]
    fn keyed_transforms_move_and_turn_what_they_hold() {
        let bytes = keyed_file();
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 3, "the last key, frame 2, plus one");
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| {
                (
                    p.model,
                    p.translation,
                    p.rotation,
                    p.frames.clone(),
                    p.track.clone(),
                )
            })
            .collect();
        let turned = rotation_of(17);
        assert_eq!(
            placed,
            vec![
                (0, [0, 0, 0], IDENTITY, None, vec![]),
                (
                    1,
                    [-2, 0, 2],
                    IDENTITY,
                    Some(0..3),
                    vec![[-2, 0, 2], [-2, 0, 2], [2, 0, 4]]
                ),
                (2, [0, 3, 1], IDENTITY, Some(0..1), vec![]),
                (2, [0, 3, 1], turned, Some(1..3), vec![[0, 3, 1], [1, 3, 1]]),
            ],
            "the cube moves, the bar is a mesh per turn and moves once turned"
        );
        let settings = VoxSettings {
            clips: vec![
                VoxClip {
                    name: "all".into(),
                    first: 0,
                    last: 2,
                },
                VoxClip {
                    name: "late".into(),
                    first: 1,
                    last: 2,
                },
            ],
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let skin = g.document.skins().next().expect("a skin");
        let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
        assert_eq!(joints.len(), 3, "the cube, the bar and the turned bar");
        // Per channel: its joint, its path and its values (x scale, or the
        // translation).
        type Read = (usize, &'static str, Vec<Vec<f32>>);
        let read = |anim: &gltf::Animation| -> Vec<Read> {
            anim.channels()
                .map(|ch| {
                    assert_eq!(
                        ch.sampler().interpolation(),
                        gltf::animation::Interpolation::Step
                    );
                    let joint = joints
                        .iter()
                        .position(|j| *j == ch.target().node().index())
                        .unwrap();
                    let r = ch.reader(|_| Some(&blob[..]));
                    match r.read_outputs().unwrap() {
                        gltf::animation::util::ReadOutputs::Scales(s) => {
                            (joint, "scale", s.map(|v| vec![v[0]]).collect())
                        }
                        gltf::animation::util::ReadOutputs::Translations(t) => {
                            (joint, "translation", t.map(|v| v.to_vec()).collect())
                        }
                        _ => panic!("scale or translation"),
                    }
                })
                .collect()
        };
        let anims: Vec<_> = g.document.animations().collect();
        let v = |x: f32, y: f32, z: f32| vec![x, y, z];
        // glTF's frame: MagicaVoxel (x, y, z) is (x, z, -y).
        assert_eq!(
            read(&anims[0]),
            vec![
                (0, "scale", vec![vec![1.0]; 4]),
                (
                    0,
                    "translation",
                    vec![
                        v(-2.0, 2.0, 0.0),
                        v(-2.0, 2.0, 0.0),
                        v(2.0, 4.0, 0.0),
                        v(2.0, 4.0, 0.0)
                    ]
                ),
                (1, "scale", vec![vec![1.0], vec![0.0], vec![0.0], vec![0.0]]),
                (2, "scale", vec![vec![0.0], vec![1.0], vec![1.0], vec![1.0]]),
                // Folded on frame 0, it waits where frame 1 shows it.
                (
                    2,
                    "translation",
                    vec![
                        v(0.0, 1.0, -3.0),
                        v(0.0, 1.0, -3.0),
                        v(1.0, 1.0, -3.0),
                        v(1.0, 1.0, -3.0)
                    ]
                ),
            ]
        );
        assert_eq!(
            read(&anims[1]),
            vec![
                (0, "scale", vec![vec![1.0]; 3]),
                (
                    0,
                    "translation",
                    vec![v(-2.0, 2.0, 0.0), v(2.0, 4.0, 0.0), v(2.0, 4.0, 0.0)]
                ),
                (1, "scale", vec![vec![0.0]; 3]),
                (2, "scale", vec![vec![1.0]; 3]),
                (
                    2,
                    "translation",
                    vec![v(0.0, 1.0, -3.0), v(1.0, 1.0, -3.0), v(1.0, 1.0, -3.0)]
                ),
            ]
        );
        // The rest pose is frame 0: the cube's joint and mesh node where
        // its first key puts it; the base is not skinned.
        let nodes: Vec<_> = g.document.nodes().collect();
        assert!(nodes[0].skin().is_none() && nodes[0].mesh().is_some());
        assert_eq!(nodes[1].transform().decomposed().0, [-2.0, 2.0, 0.0]);
        assert_eq!(
            nodes[joints[0]].transform().decomposed().0,
            [-2.0, 2.0, 0.0]
        );
        assert!(faces_out(&glb));
        // `bottom` lifts by the lowest voxel over every frame and lifts the
        // track with the nodes: the base's underside, 0 here, stays on 0.
        let grounded = to_glb(
            &bytes,
            &VoxSettings {
                origin: VoxOrigin::Bottom,
                ..VoxSettings::default()
            },
        )
        .unwrap();
        assert_eq!(world_bounds(&grounded).0.y, 0.0);
    }

    /// Keys that change nothing (the same place on every frame) leave a
    /// static file: no skin, no clip, the same glTF as one key.
    #[test]
    fn keys_that_do_not_move_leave_the_file_static() {
        let models = vec![([2, 2, 2], solid([2, 2, 2], 1))];
        let graph = |keys: &[(u32, [i32; 3], Option<u8>)]| {
            let mut g = Vec::new();
            trn(&mut g, 0, 1, [0, 0, 0]);
            grp(&mut g, 1, &[2]);
            trn_keys(&mut g, 2, 3, keys);
            shp(&mut g, 3, &[(0, None)]);
            write_models(&models, &g)
        };
        let still = graph(&[(0, [1, 2, 3], None), (4, [1, 2, 3], None)]);
        let one = graph(&[(0, [1, 2, 3], None)]);
        let file = parse(&still).unwrap();
        assert_eq!(file.frame_count, 0);
        let a = to_glb(&still, &VoxSettings::default()).unwrap();
        assert_eq!(a, to_glb(&one, &VoxSettings::default()).unwrap());
    }

    #[test]
    fn a_file_without_the_header_is_refused() {
        assert!(parse(b"not a vox").is_err());
        assert!(parse(b"VOX \x96\0\0\0MAIN\0\0\0\0\0\0\0\0").is_err());
    }
}
