use super::*;

/// The levels of one terrain for a camera, in whole grid units.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainLevel {
    /// The centre (grid units from the terrain's -x, -z corner).
    pub center: [f64; 2],
    /// Grid units per quad (2^level).
    pub step: f64,
    /// Which ring variant leaves the finer level out (None: the finest level).
    pub hole: Option<(u32, u32)>,
    pub morph: bool,
    /// Where its morph band starts: its quads from the camera (Chebyshev).
    pub morph_start: f32,
}

/// The clipmap levels around `camera` (grid units, the terrain's -x, -z
/// corner at 0) over a terrain `extent` grid units wide, with `ring` quads
/// per side, out to `far` grid units.
pub(crate) fn levels(camera: Vec2, extent: Vec2, ring: u32, far: f64) -> Vec<TerrainLevel> {
    let cam = [camera.x as f64, camera.y as f64];
    let ext = [extent.x as f64, extent.y as f64];
    let quarter = ring as f64 / 4.0;
    let half = ring as f64 / 2.0;
    let mut out: Vec<TerrainLevel> = Vec::new();
    let mut prev: Option<[f64; 2]> = None;
    for l in 0..MAX_TERRAIN_LEVELS {
        let step = (1u64 << l) as f64;
        let c = cam.map(|x| (x / (2.0 * step)).floor() * 2.0 * step);
        let hole = prev.map(|p| {
            let o = |i: usize| ((p[i] - c[i]) / step).round().clamp(0.0, 1.0) as u32;
            (o(0), o(1))
        });
        if let Some(p) = prev {
            // The finer level's square: past it nothing is left to draw.
            let r = quarter * step;
            let covers = (0..2).all(|i| p[i] - r <= 0.0 && p[i] + r >= ext[i]);
            if covers || r > far {
                break;
            }
        }
        let r = half * step;
        let touches = (0..2).all(|i| c[i] + r >= 0.0 && c[i] - r <= ext[i]);
        if touches {
            out.push(TerrainLevel {
                center: c,
                step,
                hole,
                morph: true,
                morph_start: 0.0,
            });
        }
        prev = Some(c);
    }
    if let Some(last) = out.last_mut() {
        last.morph = false;
    }
    out
}

/// Blocks per side of a level, each culled on its own (`BLOCKS`² per
/// level: Far Cry 5's per-patch culling on the CPU, with no readback).
pub(crate) const BLOCKS: u32 = 4;

/// One ring variant's index range and, within it, each block's (row by
/// row, `BLOCKS` to a row; a block the hole leaves empty is empty).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RingRange {
    pub all: std::ops::Range<u32>,
    pub blocks: [std::ops::Range<u32>; (BLOCKS * BLOCKS) as usize],
}

/// The index ranges of one ring size ([`ring_indices`]): the full grid and
/// the ring with its hole at each of the four offsets, and the skirt.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RingRanges {
    pub variants: [RingRange; 5],
    /// The zero-area triangles along the level's outer edge, each an odd
    /// corner between its two even neighbours: drawn with a level that
    /// morphs, they close the T-junctions where its odd corners meet the
    /// coarser level's edges (Losasso & Hoppe 2004's degenerate triangles).
    pub skirt: std::ops::Range<u32>,
}

/// Cell `(x, z)`'s four triangles in a level of `n` quads a side
/// (`terrain_vertex.wgsl` terrain_local_vertex's ids: the corners row by row,
/// then the centres): each from one edge to the cell's centre, wound as
/// the corners go round (-x -z, -x +z, +x +z, +x -z).
pub(crate) fn cell_fan(n: u32, x: u32, z: u32) -> [u32; 12] {
    let corner = |x: u32, z: u32| z * (n + 1) + x;
    let centre = (n + 1) * (n + 1) + z * n + x;
    let (a, b, c, d) = (
        corner(x, z),
        corner(x, z + 1),
        corner(x + 1, z + 1),
        corner(x + 1, z),
    );
    [a, b, centre, b, c, centre, c, d, centre, d, a, centre]
}

/// The skirt of a level of `n` quads a side ([`RingRanges::skirt`]).
pub(crate) fn skirt_indices(n: u32) -> Vec<u32> {
    let corner = |x: u32, z: u32| z * (n + 1) + x;
    let mut idx = Vec::with_capacity(2 * n as usize * 3);
    for i in (0..n).step_by(2) {
        idx.extend([corner(i, 0), corner(i + 1, 0), corner(i + 2, 0)]);
        idx.extend([corner(n, i), corner(n, i + 1), corner(n, i + 2)]);
        idx.extend([corner(i + 2, n), corner(i + 1, n), corner(i, n)]);
        idx.extend([corner(0, i + 2), corner(0, i + 1), corner(0, i)]);
    }
    idx
}

/// Whether ring variant `hole` leaves out cell `(x, z)` of a level of `n`
/// quads a side (the finer level's square).
pub(crate) fn in_hole(n: u32, hole: Option<(u32, u32)>, x: u32, z: u32) -> bool {
    hole.is_some_and(|(ox, oz)| {
        let (x0, z0) = (n / 4 + ox, n / 4 + oz);
        let (x1, z1) = (3 * n / 4 + ox, 3 * n / 4 + oz);
        x >= x0 && x < x1 && z >= z0 && z < z1
    })
}

impl RingRanges {
    /// These ranges `by` indices further on.
    pub(super) fn shifted(&self, by: u32) -> RingRanges {
        let s = |r: &std::ops::Range<u32>| r.start + by..r.end + by;
        RingRanges {
            variants: self.variants.clone().map(|v| RingRange {
                all: s(&v.all),
                blocks: v.blocks.map(|b| s(&b)),
            }),
            skirt: s(&self.skirt),
        }
    }
}

/// The index buffer of one ring size: the full grid, then the ring with
/// the hole at each of its four offsets, each laid out block by block
/// (`n / BLOCKS` quads a side) so a block's quads are one range, then the
/// skirt. Returns the indices and the ranges.
pub(super) fn ring_indices(n: u32) -> (Vec<u32>, RingRanges) {
    let mut idx = Vec::new();
    let mut ranges: Vec<RingRange> = Vec::new();
    let q = n / BLOCKS;
    for hole in [None, Some((0, 0)), Some((1, 0)), Some((0, 1)), Some((1, 1))] {
        let start = idx.len() as u32;
        let mut blocks: Vec<std::ops::Range<u32>> = Vec::new();
        for bz in 0..BLOCKS {
            for bx in 0..BLOCKS {
                let b = idx.len() as u32;
                for z in bz * q..(bz + 1) * q {
                    for x in bx * q..(bx + 1) * q {
                        if !in_hole(n, hole, x, z) {
                            idx.extend(cell_fan(n, x, z));
                        }
                    }
                }
                blocks.push(b..idx.len() as u32);
            }
        }
        ranges.push(RingRange {
            all: start..idx.len() as u32,
            blocks: blocks.try_into().expect("BLOCKS² blocks"),
        });
    }
    let start = idx.len() as u32;
    idx.extend(skirt_indices(n));
    let ranges = RingRanges {
        variants: ranges.try_into().expect("five ranges"),
        skirt: start..idx.len() as u32,
    };
    (idx, ranges)
}

/// One clipmap level's mesh as `terrain_vertex.wgsl` terrain_local_vertex makes
/// it, on the CPU (the id pass draws what the frame drew): its (n + 1)²
/// corners then its n² cells' centres, in the terrain's space. `height`
/// answers the field's height at field coordinates (the finest level, as
/// every clipmap level reads it); `None` when it does.
pub(crate) fn level_vertices(
    clipmap: &TerrainClipmap,
    level: &TerrainLevel,
    layout: TerrainLayout,
    height: &dyn Fn(Vec2) -> Option<f32>,
) -> Option<Vec<Vec3>> {
    let n = clipmap.ring;
    let unit = clipmap.unit;
    let s = level.step as f32;
    let center = Vec2::new(level.center[0] as f32, level.center[1] as f32);
    let extent = layout.size / unit;
    let cell = layout.cell();
    let lo = -layout.size * 0.5;
    let width = if level.morph {
        clipmap.morph_width
    } else {
        0.0
    };
    let clamp = |c: Vec2| c.clamp(Vec2::ZERO, extent);
    let at = |m: Vec2| height(m * unit / cell);
    let alpha = |c: Vec2| {
        if width <= 0.0 {
            return 0.0;
        }
        let d = (c - clipmap.camera).abs().max_element() / s;
        ((d - level.morph_start) / width).clamp(0.0, 1.0)
    };
    let zigzag = |h: [f32; 4]| (h[0] - h[3]).abs() > (h[1] - h[2]).abs();
    let coarse_cell = |c: Vec2| -> Option<(Vec2, Vec2, [f32; 4])> {
        let p = (c / (2.0 * s)).floor() * (2.0 * s);
        let (a, b) = (clamp(p), clamp(p + 2.0 * s));
        Some((
            a,
            b,
            [
                at(a)?,
                at(Vec2::new(b.x, a.y))?,
                at(Vec2::new(a.x, b.y))?,
                at(b)?,
            ],
        ))
    };
    let coarse_height = |(a, b, h): (Vec2, Vec2, [f32; 4]), m: Vec2| {
        let span = b - a;
        let f = Vec2::select(
            span.cmpgt(Vec2::ZERO),
            (m - a) / span.max(Vec2::splat(1e-6)),
            Vec2::ZERO,
        );
        split_height(h, f)
    };
    let corner = |g: UVec2| center + (g.as_vec2() - n as f32 * 0.5) * s;
    let morphed = |c: Vec2, m: Vec2, h: f32, k: Option<(Vec2, Vec2, [f32; 4])>| {
        let a = alpha(c);
        match k {
            Some(k) if a > 0.0 => h + (coarse_height(k, m) - h) * a,
            _ => h,
        }
    };
    let local = |m: Vec2, h: f32| {
        let xz = lo + m * unit;
        Vec3::new(xz.x, h, xz.y)
    };
    let mut out = Vec::with_capacity(((n + 1) * (n + 1) + n * n) as usize);
    for gz in 0..=n {
        for gx in 0..=n {
            let g = UVec2::new(gx, gz);
            let c = corner(g);
            let m = clamp(c);
            let mut h = at(m)?;
            if alpha(c) > 0.0 {
                h = morphed(c, m, h, coarse_cell(c));
            }
            out.push(local(m, h));
        }
    }
    for gz in 0..n {
        for gx in 0..n {
            let g = UVec2::new(gx, gz);
            let c00 = corner(g);
            let k = coarse_cell(c00 + 0.5 * s)?;
            let gs = [g, g + UVec2::X, g + UVec2::Y, g + UVec2::ONE];
            let mut own = [0.0; 4];
            let mut mh = [0.0; 4];
            for i in 0..4 {
                let c = corner(gs[i]);
                let m = clamp(c);
                own[i] = at(m)?;
                mh[i] = morphed(c, m, own[i], Some(k));
            }
            let main = (mh[0] + mh[3]) * 0.5;
            let cross = (mh[1] + mh[2]) * 0.5;
            let mine = if zigzag(own) { cross } else { main };
            let theirs = if zigzag(k.2) { cross } else { main };
            let h = mine + (theirs - mine) * alpha(c00 + 0.5 * s);
            let m = (clamp(c00) + clamp(c00 + Vec2::splat(s))) * 0.5;
            out.push(local(m, h));
        }
    }
    Some(out)
}

/// Each block of a clipmap level of `n` quads a side ([`ring_indices`]'s
/// order): its box in the terrain's space, its xz the block's square on
/// the terrain (the morph moves vertices only up and down), clamped to
/// the terrain, its y the height range.
pub(crate) fn level_blocks(
    level: &TerrainLevel,
    n: u32,
    unit: f32,
    size: Vec2,
    (hmin, hmax): (f32, f32),
) -> Vec<(Vec3, Vec3)> {
    let q = (n / BLOCKS) as f64;
    let lo = -size * 0.5;
    let mut out = Vec::with_capacity((BLOCKS * BLOCKS) as usize);
    for bz in 0..BLOCKS {
        for bx in 0..BLOCKS {
            let corner = |i: usize, b: u32| {
                let g = b as f64 * q - n as f64 * 0.5;
                let c = level.center[i] + g * level.step;
                let ext = if i == 0 { size.x } else { size.y } as f64;
                ((c * unit as f64).clamp(0.0, ext)) as f32
            };
            let a = Vec2::new(corner(0, bx), corner(1, bz));
            let b = Vec2::new(corner(0, bx + 1), corner(1, bz + 1));
            out.push((
                Vec3::new(lo.x + a.x, hmin, lo.y + a.y),
                Vec3::new(lo.x + b.x, hmax, lo.y + b.y),
            ));
        }
    }
    out
}
