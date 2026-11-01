//! MagicaVoxel chunks, palette and scene graph parsing.

use super::*;

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
    /// model count. With a playback range, at least its last frame plus
    /// one.
    pub frame_count: u32,
    /// The playback range of an animated file (`META`'s `_anim_range`,
    /// first and last frame played): the default clip's frames.
    pub playback: Option<(u32, u32)>,
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

pub(super) const IDENTITY: [(usize, i32); 3] = [(0, 1), (1, 1), (2, 1)];

/// The 4 x 4 identity, column-major.
pub(super) const IDENTITY_MAT4: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// The `_r` byte of an `nTRN` frame: bits 0-1 the input axis of output x,
/// bits 2-3 of output y (z is the remaining one), bits 4-6 the signs. A
/// byte naming one axis twice (or axis 3) is not a rotation: the identity.
pub(super) fn rotation_of(r: u8) -> [(usize, i32); 3] {
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
        /// `looped`: the node's `_loop`, its keys repeating from the
        /// first to the last on every frame outside them.
        Transform {
            child: u32,
            keys: Vec<Key>,
            looped: bool,
        },
        Group(Vec<u32>),
        /// Its models, each with its `_f` key frame when it has one, and
        /// its `_loop`.
        Shape(Vec<(u32, Option<u32>)>, bool),
    }
    // The scene's playback range (`META`'s `_anim_range`, both frames
    // played), when the file has one.
    let mut range: Option<(u32, u32)> = None;
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
                let attrs = dict(body, &mut p)?;
                let looped = looped(&attrs);
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
                nodes.insert(
                    node,
                    Node::Transform {
                        child,
                        keys,
                        looped,
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
                let looped = looped(&dict(body, &mut p)?);
                let n = u32_at(body, p)?;
                p += 4;
                let mut ms = Vec::new();
                for _ in 0..n {
                    let model = u32_at(body, p)?;
                    p += 4;
                    let attrs = dict(body, &mut p)?;
                    ms.push((model, attrs.get("_f").and_then(|f| f.parse().ok())));
                }
                nodes.insert(node, Node::Shape(ms, looped));
            }
            b"META" => {
                // `_anim_range "<start> <end>"` (MagicaVoxel 0.99.7.2): the
                // frames the editor plays; one that ends before it starts,
                // or is not two whole numbers, is ignored as ogt_vox does.
                let d = dict(body, &mut 0)?;
                if let Some(r) = d.get("_anim_range") {
                    let v: Vec<i64> = r.split(' ').filter_map(|s| s.parse().ok()).collect();
                    if let [a, b] = v[..]
                        && 0 <= a
                        && a <= b
                        && b <= u32::MAX as i64
                    {
                        range = Some((a as u32, b as u32));
                    }
                }
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
            Some(Node::Transform {
                child,
                keys,
                looped,
            }) => {
                // The key in force: the last at or before the frame (looped
                // into the keys' span when the node loops), the first before
                // any.
                let at = looped_frame(*looped, keys[0].0, keys[keys.len() - 1].0, frame);
                let k = keys.iter().rev().find(|k| k.0 <= at).unwrap_or(&keys[0]);
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
            Some(Node::Shape(ms, _)) if ms.len() == 1 => {
                out.push(Shown {
                    model: Some(ms[0].0 as usize).filter(|m| *m < models),
                    translation: t,
                    rotation: r,
                });
            }
            Some(Node::Shape(ms, looped)) => {
                // An animated shape: each model from its key frame (`_f`,
                // else its position in the list) to the next model's.
                let mut keyed: Vec<(u32, u32)> = ms
                    .iter()
                    .enumerate()
                    .map(|(i, (m, f))| (f.unwrap_or(i as u32), *m))
                    .collect();
                keyed.sort();
                keyed.dedup_by_key(|k| k.0);
                let at = looped_frame(*looped, keyed[0].0, keyed[keyed.len() - 1].0, frame);
                let model = keyed
                    .iter()
                    .rev()
                    .find(|k| k.0 <= at)
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
    // The frames a graph is walked over (0 without a graph).
    let mut walked = 0;
    if nodes.contains_key(&0) {
        // The frames: those of the playback range when the file has one
        // (its first frame to its last, frames before it walked too: they
        // are frames of the file, outside the default clip), else up to the
        // last key of any animated shape or keyed transform.
        let last_key = nodes
            .values()
            .filter_map(|n| match n {
                Node::Transform { keys, .. } if keys.len() > 1 => keys.last().map(|k| k.0),
                Node::Shape(ms, _) if ms.len() > 1 => ms
                    .iter()
                    .enumerate()
                    .map(|(i, (_, f))| f.unwrap_or(i as u32))
                    .max(),
                _ => None,
            })
            .max();
        let frames = match (range, last_key) {
            (_, None) => 1,
            (Some((_, end)), Some(_)) => end.saturating_add(1),
            (None, Some(k)) => k.saturating_add(1),
        };
        walked = frames;
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
        .map_or(0, |end| end.max(walked));
    Ok(VoxFile {
        models,
        palette,
        placements,
        frame_count,
        playback: range.filter(|_| frame_count > 0),
    })
}

/// A node's `_loop`: ogt_vox's reading, `1` first.
fn looped(attrs: &BTreeMap<String, String>) -> bool {
    attrs.get("_loop").is_some_and(|v| v.starts_with('1'))
}

/// The frame a node's keys are sampled at: `frame` itself, or for a node
/// that loops (`_loop`) `frame` wrapped into its keys' span `first..=last`,
/// both ways (a frame before the first key counts back from the last), as
/// MagicaVoxel and ogt_vox (`compute_looped_frame_index`) play it.
fn looped_frame(looped: bool, first: u32, last: u32, frame: u32) -> u32 {
    if !looped {
        return frame;
    }
    let len = last - first + 1;
    if frame >= first {
        first + (frame - first) % len
    } else {
        last - (first - frame - 1) % len
    }
}
