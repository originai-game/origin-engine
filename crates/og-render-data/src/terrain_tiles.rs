//! Terrain tiles (phase 3): a terrain is drawn from a pyramid of tiles so
//! that only what the camera needs is on the GPU, and the rest can stream.
//!
//! **The pyramid.** Level `k` samples every `2^k`-th height of the field
//! (level 0 is the field itself); each level is cut into tiles of
//! [`TerrainLayout::tile`] quads, and the last level is one tile covering the
//! whole terrain, always resident: the low-resolution fallback for anything
//! still loading. A tile holds `(tile + 3)²` heights (its `tile + 1`
//! samples and a border of one on each side, clamped at the terrain's
//! edge), the normal map of its `tile + 1` samples with its mips, and,
//! with a control map, `(control + 2)²` texels of it (the map resampled to
//! `control` texels per tile at level 0 and halved per level) with mips.
//! Level 0's samples are the field's own; each coarser level's are its
//! finer level's filtered by a centred tent (weights 1, 2, 1 along each
//! axis over the three finer samples around it: [`HeightPyramid`]), so a
//! far ring reads the ground's mean height instead of whichever sample
//! decimation kept, and its silhouette does not shimmer as the camera
//! moves (Losasso & Hoppe 2004's prefiltered clipmap). Every clipmap
//! vertex reads the one height function at its point (the finest
//! resident level there), so neighbouring vertices agree whatever tile
//! each reads: no crack. Collision, `sample` and level 0 are untouched.
//!
//! **Residency.** [`terrain_tiles_wanted`] answers the tiles a frame needs,
//! best first: for each clipmap level, the tiles of the pyramid level whose
//! spacing matches its own under the level's square, their ancestors, and
//! the level-0 tiles within a detail distance (scatter); truncated to a
//! capacity, ancestors always before their children. The renderer and the
//! runtime's streamer call the same function, so what is decoded is what
//! is drawn. A vertex or pixel reads the finest resident level at its
//! point, so a tile that has not arrived shows its parent.

use crate::terrain::{TerrainControl, TerrainData};
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// Quads per tile side when nothing else is asked for.
pub const TERRAIN_TILE: u32 = 128;
/// The most tiles (all levels together) one terrain's table holds: the
/// shader's tile table is one 16 KiB uniform block of one f32 per tile.
pub const MAX_TERRAIN_TILES: u32 = 4096;
/// The most pyramid levels.
pub const MAX_TERRAIN_PYRAMID: u32 = 16;
/// The azimuths a tile's horizon is kept in (every 45° from +x towards +z
/// in the terrain's space; [`HeightPyramid::horizon`]).
pub const HORIZON_AZIMUTHS: usize = 8;
/// A tile's level samples between two of its horizon texels.
pub const HORIZON_SPACING: u32 = 8;
/// Bits per horizon angle: 12 (a 0.044° step; 8 bits' 0.7° stepped a low
/// sun's terminator into visible bands).
pub const HORIZON_BITS: u32 = 12;
/// Words per horizon texel: two angles to a word, so each word is an
/// integer an f32 holds exactly (`terrain.wgsl` reads them from the
/// R32Float heights texture as numbers, never as bits a GPU may flush or
/// canonicalise).
pub const HORIZON_WORDS: u32 = (HORIZON_AZIMUTHS as u32).div_ceil(2);

/// How a terrain is cut into tiles: shared by the cook (which stores them),
/// the runtime (which streams them) and the renderer (which draws them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainLayout {
    /// Extent along x and z in metres, centred on the terrain's origin.
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    /// Quads per tile side, at every level.
    pub tile: u32,
    /// Control-map texels per tile side (0: no control map).
    pub control: u32,
    /// Pyramid levels; the last is one tile.
    pub levels: u32,
    /// The terrain has holes: each tile height's lowest bit is the hole
    /// flag of the cell at that sample ([`mark_holes`]).
    pub holes: bool,
    /// The control map's pages of four channels (1 to 4; one per four
    /// layers): each tile's control texels hold them one after the other.
    pub pages: u32,
}

fn ceil_log2(n: u32) -> u32 {
    if n <= 1 {
        0
    } else {
        32 - (n - 1).leading_zeros()
    }
}

impl TerrainLayout {
    /// The layout of a `columns` × `rows` field over `size` cut into tiles
    /// of `tile` quads, with a control map of `control` texels when it has
    /// one.
    pub fn new(
        size: Vec2,
        columns: u32,
        rows: u32,
        tile: u32,
        control: Option<(u32, u32)>,
    ) -> Result<TerrainLayout, String> {
        if columns < 2 || rows < 2 {
            return Err(format!(
                "a terrain needs at least 2 × 2 samples, got {columns} × {rows}"
            ));
        }
        if !(16..=1024).contains(&tile) {
            return Err(format!("a terrain tile is 16 to 1024 quads, got {tile}"));
        }
        let quads = (columns - 1).max(rows - 1);
        let levels = 1 + ceil_log2(quads.div_ceil(tile));
        let control = match control {
            None => 0,
            Some((w, _)) => {
                ((w as f32 * tile as f32 / (columns - 1) as f32).round() as u32).clamp(1, 256)
            }
        };
        let layout = TerrainLayout {
            size,
            columns,
            rows,
            tile,
            control,
            levels: levels.min(MAX_TERRAIN_PYRAMID),
            holes: false,
            pages: 1,
        };
        if levels > MAX_TERRAIN_PYRAMID || layout.tile_count() > MAX_TERRAIN_TILES {
            return Err(format!(
                "a {columns} × {rows} terrain in tiles of {tile} makes {} tiles; at most {MAX_TERRAIN_TILES}: use a larger tile",
                layout.tile_count()
            ));
        }
        Ok(layout)
    }

    /// Field samples between two samples of `level`.
    pub fn step(&self, level: u32) -> u32 {
        1 << level
    }

    /// Samples along x and z at `level`: every `2^level`-th, and the last.
    pub fn samples(&self, level: u32) -> (u32, u32) {
        let s = self.step(level);
        (
            (self.columns - 1).div_ceil(s) + 1,
            (self.rows - 1).div_ceil(s) + 1,
        )
    }

    /// Tiles along x and z at `level`.
    pub fn tiles(&self, level: u32) -> (u32, u32) {
        let (sx, sz) = self.samples(level);
        (
            (sx - 1).div_ceil(self.tile).max(1),
            (sz - 1).div_ceil(self.tile).max(1),
        )
    }

    /// The first tile index of `level` (levels are numbered finest first).
    pub fn first(&self, level: u32) -> u32 {
        (0..level)
            .map(|k| {
                let (x, z) = self.tiles(k);
                x * z
            })
            .sum()
    }

    /// Every level's tiles.
    pub fn tile_count(&self) -> u32 {
        self.first(self.levels)
    }

    /// The index of tile (`x`, `z`) of `level`.
    pub fn index(&self, level: u32, x: u32, z: u32) -> u32 {
        self.first(level) + z * self.tiles(level).0 + x
    }

    /// The level and position of a tile index.
    pub fn locate(&self, index: u32) -> Option<(u32, u32, u32)> {
        let mut at = 0;
        for k in 0..self.levels {
            let (x, z) = self.tiles(k);
            if index < at + x * z {
                let i = index - at;
                return Some((k, i % x, i / x));
            }
            at += x * z;
        }
        None
    }

    /// The tile of the next level up covering this one (None at the top).
    pub fn parent(&self, index: u32) -> Option<u32> {
        let (k, x, z) = self.locate(index)?;
        (k + 1 < self.levels).then(|| self.index(k + 1, x / 2, z / 2))
    }

    /// The field's sample spacing along x and z.
    pub fn cell(&self) -> Vec2 {
        self.size / Vec2::new((self.columns - 1) as f32, (self.rows - 1) as f32)
    }

    /// The field sample under sample `j` of `level` along an axis of `n`
    /// field samples.
    pub fn fine(&self, level: u32, j: i64, n: u32) -> u32 {
        (j.max(0) as u64 * self.step(level) as u64).min(n as u64 - 1) as u32
    }

    /// The rectangle a tile covers in the terrain's space (metres from its
    /// centre), clamped to the terrain.
    pub fn rect(&self, index: u32) -> (Vec2, Vec2) {
        let (k, x, z) = self.locate(index).unwrap_or((0, 0, 0));
        let s = (self.tile * self.step(k)) as f32;
        let cell = self.cell();
        let lo = Vec2::new(x as f32, z as f32) * s * cell - self.size * 0.5;
        let hi = (lo + Vec2::splat(s) * cell).min(self.size * 0.5);
        (lo, hi)
    }

    /// Heights per tile side (its samples and the border).
    pub fn heights_side(&self) -> u32 {
        self.tile + 3
    }

    /// Horizon texels per tile side: every [`HORIZON_SPACING`]-th of its
    /// samples and the last.
    pub fn horizon_side(&self) -> u32 {
        self.tile.div_ceil(HORIZON_SPACING) + 1
    }

    /// A tile's horizon words: [`HORIZON_WORDS`] per texel
    /// ([`HeightPyramid::horizon`]).
    pub fn horizon_words(&self) -> u32 {
        self.horizon_side() * self.horizon_side() * HORIZON_WORDS
    }

    /// Rows of a tile's height texture: its `heights_side()` rows of
    /// heights, then its horizon words (each as the f32 of its value,
    /// exact under 2^24) row after row.
    pub fn heights_rows(&self) -> u32 {
        self.heights_side() + self.horizon_words().div_ceil(self.heights_side())
    }

    /// Normal-map texels per tile side (its samples).
    pub fn normals_side(&self) -> u32 {
        self.tile + 1
    }

    /// A tile's control bytes per page: every mip of `control_side()`.
    pub fn control_page_bytes(&self) -> usize {
        if self.control == 0 {
            0
        } else {
            mip_texels(self.control_side()) as usize * 4
        }
    }

    /// Control texels per tile side with the border (0 without a map).
    pub fn control_side(&self) -> u32 {
        if self.control == 0 {
            0
        } else {
            self.control + 2
        }
    }

    /// The GPU bytes one resident tile takes (heights and horizon, normals
    /// and control with their mips).
    pub fn slot_bytes(&self) -> u64 {
        let h = self.heights_side() as u64;
        let rows = self.heights_rows() as u64;
        let n = mip_texels(self.normals_side());
        let c = if self.control == 0 {
            0
        } else {
            mip_texels(self.control_side()) * self.pages.max(1) as u64
        };
        h * rows * 4 + n * 4 + c * 4
    }
}

/// Texels of a square texture of side `side` with every mip.
fn mip_texels(side: u32) -> u64 {
    let mut n = 0;
    let mut s = side.max(1);
    loop {
        n += (s as u64) * (s as u64);
        if s == 1 {
            return n;
        }
        s = (s / 2).max(1);
    }
}

/// The mip sides of a square texture of side `side`, largest first.
pub fn mip_sides(side: u32) -> Vec<u32> {
    let mut out = vec![side.max(1)];
    while *out.last().expect("one") > 1 {
        out.push((out.last().expect("one") / 2).max(1));
    }
    out
}

/// A terrain's holes as its tiles carry them: a cell of pyramid level `k`
/// (the `2^k` × `2^k` field cells between its samples, fewer at the far
/// edges) is a hole when more than half of its field cells are, so a
/// coarse tile standing in for one that has not arrived keeps the larger
/// holes and drops the slivers. Counted with a summed-area table of the
/// field's hole cells.
#[derive(Clone, Debug, PartialEq)]
pub struct HoleCounts {
    /// Cells along x (the table is `(cells_x + 1)` wide).
    cells_x: u32,
    cells_z: u32,
    sums: Vec<u32>,
}

impl HoleCounts {
    /// The counts of a hole map of `cells_x` × `cells_z` cells (non-zero
    /// is a hole; [`TerrainData::holes`]).
    pub fn new(cells_x: u32, cells_z: u32, holes: &[u8]) -> HoleCounts {
        let w = cells_x as usize + 1;
        let mut sums = vec![0u32; w * (cells_z as usize + 1)];
        for r in 0..cells_z as usize {
            let mut row = 0;
            for c in 0..cells_x as usize {
                row += u32::from(holes.get(r * cells_x as usize + c).is_some_and(|h| *h != 0));
                sums[(r + 1) * w + c + 1] = sums[r * w + c + 1] + row;
            }
        }
        HoleCounts {
            cells_x,
            cells_z,
            sums,
        }
    }

    /// Hole cells among field cells `c0..c1` × `r0..r1`.
    fn count(&self, c0: u32, c1: u32, r0: u32, r1: u32) -> u32 {
        let w = self.cells_x as usize + 1;
        let at = |c: u32, r: u32| self.sums[r as usize * w + c as usize];
        at(c1, r1) + at(c0, r0) - at(c0, r1) - at(c1, r0)
    }

    /// Whether cell (`gx`, `gz`) of pyramid `level` is a hole.
    pub fn hole(&self, layout: &TerrainLayout, level: u32, gx: i64, gz: i64) -> bool {
        let span = |g: i64, n: u32| {
            let a = layout.fine(level, g, n + 1).min(n);
            let b = layout.fine(level, g + 1, n + 1).min(n);
            (a, b)
        };
        if gx < 0 || gz < 0 {
            return false;
        }
        let (c0, c1) = span(gx, self.cells_x);
        let (r0, r1) = span(gz, self.cells_z);
        let area = (c1 - c0) * (r1 - r0);
        area > 0 && self.count(c0, c1, r0, r1) * 2 > area
    }
}

/// `height` with its lowest bit set to `hole` (the flag the tile's height
/// texture carries for the shaders).
pub fn with_hole_bit(height: f32, hole: bool) -> f32 {
    f32::from_bits((height.to_bits() & !1) | u32::from(hole))
}

/// Whether a tile height carries the hole flag ([`with_hole_bit`]).
pub fn hole_bit(height: f32) -> bool {
    height.to_bits() & 1 != 0
}

/// Marks a tile's `(tile + 3)²` heights ([`HeightPyramid::tile_heights`]) with the hole
/// flags of the cells at their samples on its level.
pub fn mark_holes(layout: &TerrainLayout, index: u32, heights: &mut [f32], holes: &HoleCounts) {
    let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
    let t = layout.tile as i64;
    let side = layout.heights_side() as i64;
    for j in 0..side {
        let gz = tz as i64 * t + j - 1;
        for i in 0..side {
            let gx = tx as i64 * t + i - 1;
            let at = (j * side + i) as usize;
            heights[at] = with_hole_bit(heights[at], holes.hole(layout, k, gx, gz));
        }
    }
}

/// One tile ready to draw: its heights (metres, `(tile + 3)²`, the border
/// included), its normal map (`(tile + 1)²` and every mip after it, each
/// texel [`pack_normal`]'s RGB10A2), its control texels (per page,
/// `(control + 2)²` RGBA8 and its mips) and the range of its heights.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainTile {
    pub heights: Arc<[f32]>,
    /// Its horizon ([`HeightPyramid::horizon`]): `horizon_words()`.
    pub horizon: Arc<[u32]>,
    pub normals: Arc<[u32]>,
    pub control: Option<Arc<[u8]>>,
    pub range: (f32, f32),
    /// The height error of drawing its samples every `2^j`-th
    /// (`terrain_error.rs`): for each `j` from 1 while `2^j` is at most
    /// [`TerrainLayout::tile`], the largest error in each block of 8 × 8 of
    /// those cells, row by row (`ceil(tile / 2^j / 8)` blocks a side), the
    /// tile's level's metres. Built with the tile; never uploaded.
    pub error: Arc<[f32]>,
}

/// A normal-map texel (Rgb10a2Unorm, `terrain.wgsl` `tile_maps`): the
/// mean normal `m` of the samples under it, its direction's x and z in
/// red and green (from -1..1) and its length in blue (1 for one sample;
/// shorter where the normals under it disagree: Toksvig's measure of
/// their spread, which widens the specular lobe so a far slope of bumps
/// does not shine as a flat one).
pub fn pack_normal(m: Vec3) -> u32 {
    let len = m.length().clamp(1e-6, 1.0);
    let n = m / m.length().max(1e-12);
    let q = |v: f32| ((v.clamp(0.0, 1.0) * 1023.0).round() as u32) & 1023;
    q(n.x * 0.5 + 0.5) | (q(n.z * 0.5 + 0.5) << 10) | (q(len) << 20)
}

impl TerrainTile {
    /// Whether the tile's arrays have the lengths `layout` uploads: heights
    /// `heights_side()²`, normals one packed texel per texel of every mip of
    /// `normals_side()`, and a control map (when the layout has one and the
    /// tile carries it) four bytes per texel of every mip of `control_side()`.
    pub fn fits(&self, layout: &TerrainLayout) -> bool {
        let h = layout.heights_side() as u64;
        let control_ok = match &self.control {
            Some(c) if layout.control != 0 => {
                c.len() == layout.control_page_bytes() * layout.pages.max(1) as usize
            }
            _ => true,
        };
        self.heights.len() as u64 == h * h
            && self.horizon.len() == layout.horizon_words() as usize
            && self.normals.len() as u64 == mip_texels(layout.normals_side())
            && control_ok
    }

    /// A tile from its heights (`(tile + 3)²` at its level, the border
    /// clamped at the terrain's edge), horizon ([`HeightPyramid::horizon`])
    /// and control texels: builds the normal map (central differences,
    /// one-sided at the terrain's edge, as the collider's surface) and the
    /// mips.
    pub fn build(
        layout: &TerrainLayout,
        index: u32,
        heights: Vec<f32>,
        horizon: Vec<u32>,
        control: Option<Vec<u8>>,
    ) -> TerrainTile {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let side = layout.heights_side() as usize;
        let ns = layout.normals_side() as usize;
        let t = layout.tile as i64;
        let (sx, sz) = layout.samples(k);
        let cell = layout.cell();
        let h = |i: i64, j: i64| heights[(j + 1) as usize * side + (i + 1) as usize];
        let mut xz = vec![Vec3::Y; ns * ns];
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for j in 0..ns as i64 {
            for i in 0..ns as i64 {
                let (gx, gz) = (tx as i64 * t + i, tz as i64 * t + j);
                // The neighbours on this level, clamped to the terrain.
                let (x0, x1) = ((gx - 1).max(0), (gx + 1).min(sx as i64 - 1));
                let (z0, z1) = ((gz - 1).max(0), (gz + 1).min(sz as i64 - 1));
                let gx = gx.min(sx as i64 - 1);
                let gz = gz.min(sz as i64 - 1);
                let local = |g: i64, base: i64| (g - base).clamp(-1, t + 1);
                let (bx, bz) = (tx as i64 * t, tz as i64 * t);
                let span_x = (layout.fine(k, x1, layout.columns) as f32
                    - layout.fine(k, x0, layout.columns) as f32)
                    .max(1.0)
                    * cell.x;
                let span_z = (layout.fine(k, z1, layout.rows) as f32
                    - layout.fine(k, z0, layout.rows) as f32)
                    .max(1.0)
                    * cell.y;
                let hx =
                    (h(local(x1, bx), local(gz, bz)) - h(local(x0, bx), local(gz, bz))) / span_x;
                let hz =
                    (h(local(gx, bx), local(z1, bz)) - h(local(gx, bx), local(z0, bz))) / span_z;
                xz[j as usize * ns + i as usize] = Vec3::new(-hx, 1.0, -hz).normalize();
                let v = h(i, j);
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        // Each mip: the mean of the unit normals under it (not of their x
        // and z alone, whose reconstructed y flattened far slopes), kept
        // unnormalised so its length carries their spread.
        let mut normals = Vec::new();
        let mut level = xz;
        let mut s = ns;
        loop {
            normals.extend(level.iter().map(|m| pack_normal(*m)));
            if s == 1 {
                break;
            }
            let d = (s / 2).max(1);
            let mut next = vec![Vec3::ZERO; d * d];
            for y in 0..d {
                for x in 0..d {
                    let mut acc = Vec3::ZERO;
                    for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        acc += level[(2 * y + oy).min(s - 1) * s + (2 * x + ox).min(s - 1)] * 0.25;
                    }
                    next[y * d + x] = acc;
                }
            }
            level = next;
            s = d;
        }
        // Each page's texels and its mips, page after page.
        let control = control.map(|c| {
            let cs = layout.control_side() as usize;
            let mut out = Vec::with_capacity(layout.control_page_bytes() * layout.pages as usize);
            for page in c.chunks(cs * cs * 4) {
                out.extend_from_slice(page);
                let mut level = page.to_vec();
                let mut s = cs;
                while s > 1 {
                    let d = (s / 2).max(1);
                    let mut next = vec![0u8; d * d * 4];
                    for y in 0..d {
                        for x in 0..d {
                            for ch in 0..4 {
                                let mut sum = 0u32;
                                for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                                    sum += level[((2 * y + oy).min(s - 1) * s
                                        + (2 * x + ox).min(s - 1))
                                        * 4
                                        + ch] as u32;
                                }
                                next[(y * d + x) * 4 + ch] = ((sum + 2) / 4) as u8;
                            }
                        }
                    }
                    out.extend_from_slice(&next);
                    level = next;
                    s = d;
                }
            }
            Arc::from(out)
        });
        let t = layout.tile;
        let levels = 31 - t.max(2).leading_zeros();
        let error = crate::terrain_error::Decimation::new(t + 1, t + 1, levels, &|x, z| {
            heights[(z as usize + 1) * side + x as usize + 1]
        })
        .flat();
        TerrainTile {
            heights: Arc::from(heights),
            horizon: Arc::from(horizon),
            normals: Arc::from(normals),
            control,
            range: (lo, hi),
            error: Arc::from(error),
        }
    }
}

/// A terrain's control map resampled for its tiles: level 0 at
/// `control` texels per tile (bilinear between the map's texel centres),
/// each level after it halved (2 × 2 means, rounded); one such pyramid per
/// page of four channels.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlPyramid {
    /// Per page, (width, height, RGBA8) per level.
    pub pages: Vec<Vec<(u32, u32, Vec<u8>)>>,
}

impl ControlPyramid {
    /// `None` for a layout without a control map.
    pub fn new(layout: &TerrainLayout, map: &TerrainControl) -> Option<ControlPyramid> {
        if layout.control == 0 {
            return None;
        }
        let c = layout.control;
        let pages = (0..map.pages())
            .map(|p| {
                let map = map.page(p);
                let (tx, tz) = layout.tiles(0);
                let (w, h) = (c * tx, c * tz);
                let mut base = vec![0u8; (w * h * 4) as usize];
                for j in 0..h {
                    for i in 0..w {
                        let at = ((j * w + i) * 4) as usize;
                        base[at..at + 4].copy_from_slice(&Self::base(layout, &map, i, j));
                    }
                }
                let mut levels = vec![(w, h, base)];
                for k in 1..layout.levels {
                    let (pw, ph, prev) = levels.last().expect("level 0");
                    let (tx, tz) = layout.tiles(k);
                    let (w, h) = (c * tx, c * tz);
                    let mut next = vec![0u8; (w * h * 4) as usize];
                    for j in 0..h {
                        for i in 0..w {
                            let at = ((j * w + i) * 4) as usize;
                            next[at..at + 4].copy_from_slice(&Self::mean(*pw, *ph, prev, i, j));
                        }
                    }
                    levels.push((w, h, next));
                }
                levels
            })
            .collect();
        Some(ControlPyramid { pages })
    }

    /// Level 0's texel (`i`, `j`) of one page's map: bilinear between the
    /// map's texel centres.
    fn base(layout: &TerrainLayout, map: &TerrainControl, i: u32, j: u32) -> [u8; 4] {
        let c = layout.control;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let texel = |x: i64, y: i64, k: usize| {
            let x = x.clamp(0, map.width as i64 - 1) as usize;
            let y = y.clamp(0, map.height as i64 - 1) as usize;
            map.rgba[(y * map.width as usize + x) * 4 + k] as f32
        };
        // The texel's centre as a fraction of the terrain, then in the map's texels.
        let uv = Vec2::new(
            (i as f32 + 0.5) * layout.tile as f32 / c as f32,
            (j as f32 + 0.5) * layout.tile as f32 / c as f32,
        ) / quads;
        let t =
            uv.clamp(Vec2::ZERO, Vec2::ONE) * Vec2::new(map.width as f32, map.height as f32) - 0.5;
        let f0 = t.floor();
        let f = t - f0;
        let (x0, y0) = (f0.x as i64, f0.y as i64);
        std::array::from_fn(|k| {
            let a = texel(x0, y0, k) + (texel(x0 + 1, y0, k) - texel(x0, y0, k)) * f.x;
            let b = texel(x0, y0 + 1, k) + (texel(x0 + 1, y0 + 1, k) - texel(x0, y0 + 1, k)) * f.x;
            (a + (b - a) * f.y).round().clamp(0.0, 255.0) as u8
        })
    }

    /// A coarser level's texel (`i`, `j`): the mean of the 2 × 2 finer ones
    /// under it (`prev`, `pw` × `ph`), clamped, rounded.
    fn mean(pw: u32, ph: u32, prev: &[u8], i: u32, j: u32) -> [u8; 4] {
        std::array::from_fn(|ch| {
            let mut sum = 0u32;
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let x = (2 * i + ox).min(pw - 1);
                let y = (2 * j + oy).min(ph - 1);
                sum += prev[((y * pw + x) * 4 + ch as u32) as usize] as u32;
            }
            ((sum + 2) / 4) as u8
        })
    }

    /// Resamples what an edit of `map` (the edited map) inside `lo`..`hi`
    /// (the terrain's space) reaches on every level: the pyramid is then
    /// [`ControlPyramid::new`]'s of the edited map.
    pub fn update(&mut self, layout: &TerrainLayout, map: &TerrainControl, lo: Vec2, hi: Vec2) {
        // The edit in cells, widened by the two map texels a bilinear read
        // spans, then in level 0's texels (each `tile / control` cells).
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let texel = quads / Vec2::new(map.width as f32, map.height as f32);
        let to = |p: Vec2| (p + layout.size * 0.5) / layout.cell();
        let per = layout.tile as f32 / layout.control as f32;
        let a = ((to(lo) - 2.0 * texel) / per - 0.5).floor() - 1.0;
        let b = ((to(hi) + 2.0 * texel) / per - 0.5).ceil() + 1.0;
        for (p, levels) in self.pages.iter_mut().enumerate() {
            let map = map.page(p as u32);
            let (w, h, base) = &mut levels[0];
            let clamp = |v: f32, n: u32| (v.max(0.0) as u32).min(n - 1);
            let (mut x0, mut x1) = (clamp(a.x, *w), clamp(b.x, *w));
            let (mut y0, mut y1) = (clamp(a.y, *h), clamp(b.y, *h));
            for j in y0..=y1 {
                for i in x0..=x1 {
                    let at = ((j * *w + i) * 4) as usize;
                    base[at..at + 4].copy_from_slice(&Self::base(layout, &map, i, j));
                }
            }
            for k in 1..levels.len() {
                let (done, rest) = levels.split_at_mut(k);
                let (pw, ph, prev) = &done[k - 1];
                let (w, h, next) = &mut rest[0];
                // The texels past the finer level's last read its last.
                let up = |a: u32, n: u32, pn: u32| {
                    if a + 1 >= pn {
                        n - 1
                    } else {
                        (a / 2).min(n - 1)
                    }
                };
                (x0, x1, y0, y1) = (
                    (x0 / 2).min(*w - 1),
                    up(x1, *w, *pw),
                    (y0 / 2).min(*h - 1),
                    up(y1, *h, *ph),
                );
                for j in y0..=y1 {
                    for i in x0..=x1 {
                        let at = ((j * *w + i) * 4) as usize;
                        next[at..at + 4].copy_from_slice(&Self::mean(*pw, *ph, prev, i, j));
                    }
                }
            }
        }
    }

    /// The `(control + 2)²` texels of a tile (its texels and a border of
    /// one, clamped at the map's edge), page after page.
    pub fn tile(&self, layout: &TerrainLayout, index: u32) -> Vec<u8> {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let c = layout.control as i64;
        let side = layout.control_side() as usize;
        let mut out = Vec::with_capacity(side * side * 4 * self.pages.len());
        for page in &self.pages {
            let (w, h, px) = &page[k as usize];
            for j in -1..=c {
                for i in -1..=c {
                    let x = (tx as i64 * c + i).clamp(0, *w as i64 - 1) as usize;
                    let y = (tz as i64 * c + j).clamp(0, *h as i64 - 1) as usize;
                    let at = (y * *w as usize + x) * 4;
                    out.extend_from_slice(&px[at..at + 4]);
                }
            }
        }
        out
    }
}

impl TerrainData {
    /// This field's layout in tiles of `tile` quads.
    pub fn layout(&self, tile: u32) -> Result<TerrainLayout, String> {
        let mut layout = TerrainLayout::new(
            self.size,
            self.columns,
            self.rows,
            tile,
            self.control.as_ref().map(|c| (c.width, c.height)),
        )?;
        layout.holes = self.holes.is_some();
        layout.pages = self.control.as_ref().map_or(1, TerrainControl::pages);
        Ok(layout)
    }

    /// The hole counts its tiles are marked from (`None`: no holes).
    pub fn hole_counts(&self) -> Option<HoleCounts> {
        let (cx, cz) = self.cells();
        self.holes.as_ref().map(|h| HoleCounts::new(cx, cz, h))
    }

    /// Its coarser levels for `layout` ([`HeightPyramid`]).
    pub fn height_pyramid(&self, layout: &TerrainLayout) -> HeightPyramid {
        HeightPyramid::new(layout, |c, r| self.heights[(r * self.columns + c) as usize])
    }

    /// A tile's `(tile + 3)²` heights: its level's samples of `pyramid`
    /// (this field's: [`Self::height_pyramid`]), clamped at the edge.
    pub fn tile_heights(
        &self,
        layout: &TerrainLayout,
        index: u32,
        pyramid: &HeightPyramid,
    ) -> Vec<f32> {
        pyramid.tile_heights(layout, index, |c, r| {
            self.heights[(r * self.columns + c) as usize]
        })
    }

    /// A tile ready to draw (`pyramid`: this field's height pyramid;
    /// `control`: its control pyramid; `holes`: its hole counts, which
    /// mark the heights).
    pub fn tile(
        &self,
        layout: &TerrainLayout,
        index: u32,
        pyramid: &HeightPyramid,
        control: Option<&ControlPyramid>,
        holes: Option<&HoleCounts>,
    ) -> TerrainTile {
        let mut heights = self.tile_heights(layout, index, pyramid);
        if let Some(h) = holes {
            mark_holes(layout, index, &mut heights, h);
        }
        TerrainTile::build(
            layout,
            index,
            heights,
            self.horizon(layout, index, pyramid),
            control.map(|p| p.tile(layout, index)),
        )
    }

    /// A tile's horizon over this field ([`HeightPyramid::horizon`];
    /// `pyramid`: its height pyramid).
    pub fn horizon(&self, layout: &TerrainLayout, index: u32, pyramid: &HeightPyramid) -> Vec<u32> {
        pyramid.horizon(layout, index, |c, r| {
            self.heights[(r * self.columns + c) as usize]
        })
    }
}

/// A field's coarser levels (1 up): each level's samples, the finer
/// level's under a centred tent filter, clamped at the field's edges; and
/// beside them the highest field height under each sample's footprint
/// (the same 3 × 3 of the finer level), which the horizon reads.
#[derive(Clone, Debug, PartialEq)]
pub struct HeightPyramid {
    /// Level k + 1's samples along x and z and its heights, row-major.
    pub levels: Vec<(u32, u32, Vec<f32>)>,
    /// Level k + 1's maxima, laid out as its heights.
    pub maxima: Vec<Vec<f32>>,
}

impl HeightPyramid {
    /// The pyramid of a field read as `field(column, row)`, for `layout`.
    pub fn new(layout: &TerrainLayout, field: impl Fn(u32, u32) -> f32) -> HeightPyramid {
        let mut levels: Vec<(u32, u32, Vec<f32>)> = Vec::new();
        let mut maxima: Vec<Vec<f32>> = Vec::new();
        let (mut fx, mut fz) = (layout.columns as i64, layout.rows as i64);
        for k in 1..layout.levels {
            let (sx, sz) = layout.samples(k);
            let samples = (0..sz as i64).flat_map(|z| (0..sx as i64).map(move |x| (x, z)));
            let out = {
                let at = Self::reader(&field, levels.last(), fx, fz);
                samples
                    .clone()
                    .map(|(x, z)| Self::tent(&at, fx, fz, x, z))
                    .collect()
            };
            let peaks = {
                let finer = levels
                    .last()
                    .map(|(w, h, _)| (*w, *h, maxima.last().expect("beside")));
                let at = Self::max_reader(&field, finer, fx, fz);
                samples
                    .map(|(x, z)| Self::peak(&at, fx, fz, x, z))
                    .collect()
            };
            levels.push((sx, sz, out));
            maxima.push(peaks);
            (fx, fz) = (sx as i64, sz as i64);
        }
        HeightPyramid { levels, maxima }
    }

    /// A finer level's maxima (the field's heights when `finer` is
    /// `None`), clamped at its edges.
    fn max_reader<'a>(
        field: &'a impl Fn(u32, u32) -> f32,
        finer: Option<(u32, u32, &'a Vec<f32>)>,
        fx: i64,
        fz: i64,
    ) -> impl Fn(i64, i64) -> f32 + 'a {
        move |i: i64, j: i64| {
            let (i, j) = (i.clamp(0, fx - 1), j.clamp(0, fz - 1));
            match finer {
                None => field(i as u32, j as u32),
                Some((w, _, h)) => h[(j * w as i64 + i) as usize],
            }
        }
    }

    /// The highest of the finer samples a coarser one's tent covers.
    fn peak(at: &impl Fn(i64, i64) -> f32, fx: i64, fz: i64, x: i64, z: i64) -> f32 {
        let ci = (2 * x).min(fx - 1);
        let cj = (2 * z).min(fz - 1);
        let mut v = f32::NEG_INFINITY;
        for b in -1..=1 {
            for a in -1..=1 {
                v = v.max(at(ci + a, cj + b));
            }
        }
        v
    }

    /// A finer level's heights (`fx` × `fz`; the field's when `finer` is
    /// `None`), clamped at its edges.
    fn reader<'a>(
        field: &'a impl Fn(u32, u32) -> f32,
        finer: Option<&'a (u32, u32, Vec<f32>)>,
        fx: i64,
        fz: i64,
    ) -> impl Fn(i64, i64) -> f32 + 'a {
        move |i: i64, j: i64| {
            let (i, j) = (i.clamp(0, fx - 1), j.clamp(0, fz - 1));
            match finer {
                None => field(i as u32, j as u32),
                Some((w, _, h)) => h[(j * *w as i64 + i) as usize],
            }
        }
    }

    /// A coarser sample (`x`, `z`): the finer level's (`at`, `fx` × `fz`)
    /// under it (every second, and the last at the last) under a centred
    /// tent.
    fn tent(at: &impl Fn(i64, i64) -> f32, fx: i64, fz: i64, x: i64, z: i64) -> f32 {
        const W: [f32; 3] = [0.25, 0.5, 0.25];
        let ci = (2 * x).min(fx - 1);
        let cj = (2 * z).min(fz - 1);
        let mut v = 0.0;
        for (b, wz) in W.iter().enumerate() {
            for (a, wx) in W.iter().enumerate() {
                v += wx * wz * at(ci + a as i64 - 1, cj + b as i64 - 1);
            }
        }
        v
    }

    /// Filters again what field samples `lo`..=`hi` (column, row) reach on
    /// every level after an edit inside them (`field`: the edited one):
    /// the pyramid is then [`HeightPyramid::new`]'s of the edited field.
    pub fn update(
        &mut self,
        layout: &TerrainLayout,
        field: impl Fn(u32, u32) -> f32,
        lo: (u32, u32),
        hi: (u32, u32),
    ) {
        let (mut lo, mut hi) = ((lo.0 as i64, lo.1 as i64), (hi.0 as i64, hi.1 as i64));
        let (mut fx, mut fz) = (layout.columns as i64, layout.rows as i64);
        for k in 1..layout.levels as usize {
            let (done, rest) = self.levels.split_at_mut(k - 1);
            let (sx, sz, out) = &mut rest[0];
            let (sx, sz) = (*sx as i64, *sz as i64);
            let at = Self::reader(&field, done.last(), fx, fz);
            let (peaks_done, peaks_rest) = self.maxima.split_at_mut(k - 1);
            let finer = done
                .last()
                .map(|(w, h, _)| (*w, *h, peaks_done.last().expect("beside")));
            let at_max = Self::max_reader(&field, finer, fx, fz);
            let peaks = &mut peaks_rest[0];
            // The samples whose tent (2x - 1 to 2x + 1 of the finer level,
            // the last one's clamped) reads a changed finer one.
            let reach = |a: i64, b: i64, n: i64| {
                (
                    (a - 1).div_euclid(2).clamp(0, n - 1),
                    ((b + 2) / 2).clamp(0, n - 1),
                )
            };
            let (x0, x1) = reach(lo.0, hi.0, sx);
            let (z0, z1) = reach(lo.1, hi.1, sz);
            for z in z0..=z1 {
                for x in x0..=x1 {
                    out[(z * sx + x) as usize] = Self::tent(&at, fx, fz, x, z);
                    peaks[(z * sx + x) as usize] = Self::peak(&at_max, fx, fz, x, z);
                }
            }
            (lo, hi, fx, fz) = ((x0, z0), (x1, z1), sx, sz);
        }
    }

    /// Level `k`'s height at its sample (`x`, `z`), clamped; level 0 is
    /// `field`'s.
    fn at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        x: i64,
        z: i64,
    ) -> f32 {
        if k == 0 {
            let c = x.clamp(0, layout.columns as i64 - 1) as u32;
            let r = z.clamp(0, layout.rows as i64 - 1) as u32;
            return field(c, r);
        }
        let (sx, sz, h) = &self.levels[k as usize - 1];
        let (x, z) = (x.clamp(0, *sx as i64 - 1), z.clamp(0, *sz as i64 - 1));
        h[(z * *sx as i64 + x) as usize]
    }

    /// Level `k`'s height under field coordinates `s` (in field cells from
    /// the -x, -z corner): bilinear between its samples (every 2^k-th and
    /// the last, whose cell may be shorter: `terrain.wgsl` `level_coord`).
    fn height_at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        s: Vec2,
    ) -> f32 {
        let step = layout.step(k) as f32;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let (sx, sz) = layout.samples(k);
        let n = Vec2::new(sx as f32, sz as f32);
        let last = (n - 2.0) * step;
        let span = (quads - last).max(Vec2::splat(1e-6));
        let u = Vec2::select(s.cmpgt(last), n - 2.0 + (s - last) / span, s / step)
            .clamp(Vec2::ZERO, n - 1.0);
        let i = u.floor();
        let f = u - i;
        let (x, z) = (i.x as i64, i.y as i64);
        let h = |x: i64, z: i64| self.at(field, layout, k, x, z);
        let a = h(x, z) + (h(x + 1, z) - h(x, z)) * f.x;
        let b = h(x, z + 1) + (h(x + 1, z + 1) - h(x, z + 1)) * f.x;
        a + (b - a) * f.y
    }

    /// The highest ground near field coordinates `s` on level `k`: the
    /// largest of the four samples around `s` (past level 0, of their
    /// maxima: each the highest of the field a sample's spacing either
    /// side of it), so a peak between two reads is never lowered.
    fn max_at(
        &self,
        field: &impl Fn(u32, u32) -> f32,
        layout: &TerrainLayout,
        k: u32,
        s: Vec2,
    ) -> f32 {
        if k == 0 {
            let (x, z) = (s.x.floor() as i64, s.y.floor() as i64);
            let at = |x: i64, z: i64| self.at(field, layout, 0, x, z);
            return at(x, z)
                .max(at(x + 1, z))
                .max(at(x, z + 1))
                .max(at(x + 1, z + 1));
        }
        let step = layout.step(k) as f32;
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let (sx, sz) = layout.samples(k);
        let n = Vec2::new(sx as f32, sz as f32);
        let last = (n - 2.0) * step;
        let span = (quads - last).max(Vec2::splat(1e-6));
        let u = Vec2::select(s.cmpgt(last), n - 2.0 + (s - last) / span, s / step)
            .clamp(Vec2::ZERO, n - 1.0);
        let (x, z) = (u.x.floor() as i64, u.y.floor() as i64);
        let peaks = &self.maxima[k as usize - 1];
        let at = |x: i64, z: i64| {
            let (x, z) = (x.clamp(0, sx as i64 - 1), z.clamp(0, sz as i64 - 1));
            peaks[(z * sx as i64 + x) as usize]
        };
        at(x, z)
            .max(at(x + 1, z))
            .max(at(x, z + 1))
            .max(at(x + 1, z + 1))
    }

    /// A tile's horizon: at each of its `horizon_side()²` texels (every
    /// [`HORIZON_SPACING`]-th sample of its level and the last), the
    /// highest elevation of the ground seen from that point in each of
    /// [`HORIZON_AZIMUTHS`] directions (k · 45° from +x towards +z), out to
    /// the terrain's edge: a horizon map (Max, "Horizon mapping", 1988),
    /// marched at distances growing by 1/32 per step, each read on the
    /// pyramid's maxima at the level that covers the gap to the next (the
    /// far-field reads of Timonen and Westerholm, "Scalable Height Field
    /// Self-Shadowing", 2010, without their sweep's shared hull). Each
    /// angle is [`HORIZON_BITS`] over -90° to 90° (0 a horizon straight
    /// down, all ones straight up), two to a word from its low bits
    /// ([`HORIZON_WORDS`] words per texel), texel by texel row-major. Seen in the terrain's
    /// own space (its metres, before its transform).
    pub fn horizon(
        &self,
        layout: &TerrainLayout,
        index: u32,
        field: impl Fn(u32, u32) -> f32,
    ) -> Vec<u32> {
        // The march's steps grow by 1/GROWTH of the distance.
        const GROWTH: f32 = 32.0;
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let side = layout.horizon_side();
        let t = layout.tile as f32;
        let step = layout.step(k) as f32;
        let cell = layout.cell();
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let reach = quads.length();
        let dirs: [Vec2; HORIZON_AZIMUTHS] = std::array::from_fn(|a| {
            let angle = a as f32 * std::f32::consts::TAU / HORIZON_AZIMUTHS as f32;
            Vec2::new(angle.cos(), angle.sin())
        });
        let quantise = |tan: f32| {
            let angle = tan.atan() / std::f32::consts::FRAC_PI_2;
            let top = ((1 << HORIZON_BITS) - 1) as f32;
            ((angle * 0.5 + 0.5) * top).round().clamp(0.0, top) as u32
        };
        let mut out = Vec::with_capacity(layout.horizon_words() as usize);
        for j in 0..side {
            for i in 0..side {
                // The texel's place on its level, then in field cells.
                let g = Vec2::new(
                    tx as f32 * t + i as f32 * t / (side - 1) as f32,
                    tz as f32 * t + j as f32 * t / (side - 1) as f32,
                );
                let s = (g * step).min(quads);
                let h0 = self.height_at(&field, layout, k, s);
                let mut bytes = [0u32; HORIZON_AZIMUTHS];
                for (a, dir) in dirs.iter().enumerate() {
                    let metres = (*dir * cell).length();
                    let mut best = f32::NEG_INFINITY;
                    let mut d = step;
                    while d <= reach {
                        let q = s + *dir * d;
                        if q.x < 0.0 || q.y < 0.0 || q.x > quads.x || q.y > quads.y {
                            break;
                        }
                        // The finest level whose maxima (each a sample's
                        // spacing either side) cover the gap to the next
                        // step: a peak is never stepped over, and is seen
                        // at most an eighth of `d` nearer than it stands
                        // (below), so the horizon errs a little high.
                        let lv = ((d / (2.0 * GROWTH)).log2().ceil().max(0.0) as u32)
                            .min(layout.levels - 1);
                        let h = self.max_at(&field, layout, lv, q);
                        // As if at the nearest the read may have found it
                        // (its four samples' footprint), so a peak is never
                        // seen lower than it stands.
                        let near = (d - 2.0 * layout.step(lv) as f32).max(step.min(d));
                        best = best.max((h - h0) / (near * metres));
                        d = (d * (1.0 + 1.0 / GROWTH)).max(d + step);
                    }
                    bytes[a] = quantise(best);
                }
                for word in bytes.chunks(2) {
                    out.push(word.iter().rev().fold(0, |w, b| w << HORIZON_BITS | b));
                }
            }
        }
        out
    }

    /// A tile's `(tile + 3)²` heights: its level's samples (the field's at
    /// level 0), the border clamped at the terrain's edge.
    pub fn tile_heights(
        &self,
        layout: &TerrainLayout,
        index: u32,
        field: impl Fn(u32, u32) -> f32,
    ) -> Vec<f32> {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let t = layout.tile as i64;
        let side = layout.heights_side() as i64;
        let mut out = Vec::with_capacity((side * side) as usize);
        for j in 0..side {
            for i in 0..side {
                let (gx, gz) = (tx as i64 * t + i - 1, tz as i64 * t + j - 1);
                out.push(if k == 0 {
                    let (sx, sz) = layout.samples(0);
                    let c = layout.fine(0, gx.clamp(0, sx as i64 - 1), layout.columns);
                    let r = layout.fine(0, gz.clamp(0, sz as i64 - 1), layout.rows);
                    field(c, r)
                } else {
                    self.at(&field, layout, k, gx, gz)
                });
            }
        }
        out
    }
}

/// Calls `f` with every tile an edit of the field inside `lo`..`hi` (the
/// terrain's space) changes: on every level, those whose samples, border,
/// height filter or control texels reach into it. Their horizons are baked
/// again with them; a tile further off keeps the horizon it had, which an
/// edit moves by at most its height change over its distance (the horizon
/// shades only the far field past the sun's cascades) until it is built
/// again.
pub fn edited_tiles(layout: &TerrainLayout, lo: Vec2, hi: Vec2, mut f: impl FnMut(u32)) {
    // A level-k sample's tent reads 2^k - 1 field cells out, a tile's
    // heights one sample (2^k cells) before it and two after; its control
    // texels (each 2^k texels of `tile / control` cells at level 0) a
    // border of one, and level 0's a bilinear read of about one map texel
    // (as wide as a level-0 texel) out.
    let per = if layout.control == 0 {
        0.0
    } else {
        layout.tile as f32 / layout.control as f32
    };
    for k in 0..layout.levels {
        let step = layout.step(k) as f32;
        let margin = layout.cell() * (3.0 * step).max(step * per + 2.0 * per + 1.0);
        cover(layout, k, lo - margin, hi + margin, &mut f);
    }
}

impl TerrainLayout {
    /// The field samples (column, row) inside `lo`..`hi` (the terrain's
    /// space), rounded outwards and clamped to the field.
    pub fn samples_in(&self, lo: Vec2, hi: Vec2) -> ((u32, u32), (u32, u32)) {
        let to = |p: Vec2| (p + self.size * 0.5) / self.cell();
        let (a, b) = (to(lo).floor(), to(hi).ceil());
        let c = |v: f32, n: u32| (v.max(0.0) as u32).min(n - 1);
        (
            (c(a.x, self.columns), c(a.y, self.rows)),
            (c(b.x, self.columns), c(b.y, self.rows)),
        )
    }
}

/// Calls `f` with every tile of `level` overlapping the rectangle `a`..`b`
/// (the terrain's space, metres from its centre).
pub fn cover(layout: &TerrainLayout, level: u32, a: Vec2, b: Vec2, mut f: impl FnMut(u32)) {
    let half = layout.size * 0.5;
    let size = layout.cell() * (layout.tile * layout.step(level)) as f32;
    let (tx, tz) = layout.tiles(level);
    let lo = ((a + half) / size).floor();
    let hi = ((b + half) / size).ceil();
    let x0 = (lo.x.max(0.0) as u32).min(tx);
    let z0 = (lo.y.max(0.0) as u32).min(tz);
    let x1 = (hi.x.max(0.0) as u32).min(tx);
    let z1 = (hi.y.max(0.0) as u32).min(tz);
    for z in z0..z1 {
        for x in x0..x1 {
            f(layout.index(level, x, z));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`pack_normal`]'s direction and length back.
    fn unpack_normal(t: u32) -> (Vec3, f32) {
        let f = |s: u32| ((t >> s) & 1023) as f32 / 1023.0;
        let (x, z) = (f(0) * 2.0 - 1.0, f(10) * 2.0 - 1.0);
        let y = (1.0 - x * x - z * z).max(0.0).sqrt();
        (Vec3::new(x, y, z), f(20))
    }

    /// A 257 × 257 field over 256 m in tiles of 16: five levels.
    fn layout() -> TerrainLayout {
        TerrainLayout::new(Vec2::splat(256.0), 257, 257, 16, None).expect("layout")
    }

    #[test]
    fn a_coarse_level_holds_its_finer_levels_mean_not_one_sample() {
        let l = layout();
        // A checker of ±1 one sample wide: decimation keeps +1 at every
        // even sample; the mean under each coarse sample is 0.
        let checker = |c: u32, r: u32| if (c + r).is_multiple_of(2) { 1.0 } else { -1.0 };
        let p = HeightPyramid::new(&l, checker);
        let (sx, sz, h) = &p.levels[0];
        assert_eq!((*sx, *sz), l.samples(1));
        for z in 1..sz - 1 {
            for x in 1..sx - 1 {
                assert_eq!(h[(z * sx + x) as usize], 0.0, "level 1 at ({x}, {z})");
            }
        }
        // A ramp keeps its height everywhere off the edges, at every level.
        let ramp = |c: u32, r: u32| 0.5 * c as f32 - 0.25 * r as f32;
        let p = HeightPyramid::new(&l, ramp);
        for k in 1..l.levels {
            let (sx, sz, h) = &p.levels[k as usize - 1];
            let step = l.step(k);
            for z in 1..sz - 1 {
                for x in 1..sx - 1 {
                    let want = ramp(x * step, z * step);
                    let got = h[(z * sx + x) as usize];
                    assert!(
                        (got - want).abs() < 1e-4,
                        "level {k} ({x}, {z}): {got}, not {want}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_coarse_tile_reads_its_levels_samples_with_their_border() {
        let l = layout();
        let field = |c: u32, r: u32| ((c * 7 + r * 13) % 11) as f32;
        let p = HeightPyramid::new(&l, field);
        let (sx, _, h1) = &p.levels[0];
        // Level 1's tile (1, 0): its samples from x 16 and z 0, its border
        // one sample further out (clamped at z -1).
        let tile = p.tile_heights(&l, l.index(1, 1, 0), field);
        let side = l.heights_side();
        for j in 0..side {
            for i in 0..side {
                let (gx, gz) = (16 + i as i64 - 1, (j as i64 - 1).max(0));
                let want = h1[(gz as u32 * sx + gx as u32) as usize];
                assert_eq!(tile[(j * side + i) as usize], want, "({i}, {j})");
            }
        }
        // Level 0 is the field's own.
        let t0 = p.tile_heights(&l, l.index(0, 2, 3), field);
        assert_eq!(t0[side as usize + 1], field(32, 48));
    }

    #[test]
    fn a_normal_mip_keeps_the_mean_direction_and_the_spread() {
        // Level 0's tile 0 over ridges 8 m apart (slopes to 57°) on a
        // gentle rise towards +x.
        let l = layout();
        let field = |c: u32, _: u32| -> f32 {
            let x = c as f32;
            2.0 * (x * std::f32::consts::FRAC_PI_4).sin() + 0.2 * x
        };
        let p = HeightPyramid::new(&l, field);
        let tile = TerrainTile::build(
            &l,
            0,
            p.tile_heights(&l, 0, field),
            vec![0; l.horizon_words() as usize],
            None,
        );
        let ns = l.normals_side() as usize;
        let (base, top) = (&tile.normals[..ns * ns], *tile.normals.last().unwrap());
        // The mean of level 0's unit normals the halvings reach (129 halves
        // to 64 over the first 128), straight from them.
        let m = ns - 1;
        let mean = (0..m * m)
            .map(|k| unpack_normal(base[(k / m) * ns + k % m]).0)
            .sum::<Vec3>()
            / (m * m) as f32;
        let (dir, len) = unpack_normal(top);
        assert!(
            dir.angle_between(mean.normalize()) < 0.01,
            "{dir} against {}",
            mean.normalize()
        );
        assert!(
            (len - mean.length()).abs() < 0.01,
            "length {len}, the mean's {}",
            mean.length()
        );
        assert!(len < 0.95, "the ridges' spread shows: {len}");
        // A plane's mips are its normal, at full length.
        let plane = |c: u32, r: u32| 0.3 * c as f32 + 0.1 * r as f32;
        let p = HeightPyramid::new(&l, plane);
        let flat = TerrainTile::build(
            &l,
            0,
            p.tile_heights(&l, 0, plane),
            vec![0; l.horizon_words() as usize],
            None,
        );
        let (d, len) = unpack_normal(*flat.normals.last().unwrap());
        assert!(len > 0.998 && d.angle_between(Vec3::new(-0.3, 1.0, -0.1).normalize()) < 0.01);
    }

    /// Behind a ridge 40 m high the horizon towards it (+x) stands at the
    /// ridge's elevation seen from the texel or a little over it, on level
    /// 0 and on a coarse level (read from the maxima, not the mean, which
    /// would flatten a one-sample ridge); away from it (-x)
    /// the flat ground's horizon is level.
    #[test]
    fn a_ridge_raises_the_horizon_towards_it() {
        let l = layout();
        let ridge = 200u32;
        let field = |c: u32, _: u32| if c == ridge { 40.0 } else { 0.0 };
        let p = HeightPyramid::new(&l, field);
        let angle = |words: &[u32], texel: usize, a: usize| {
            let top = (1u32 << HORIZON_BITS) - 1;
            let w = words[texel * HORIZON_WORDS as usize + a / 2];
            let b = (w >> (HORIZON_BITS * (a as u32 % 2))) & top;
            (b as f32 / top as f32 * 2.0 - 1.0) * 90.0
        };
        let step: f32 = 180.0 / 4095.0;
        for k in [0u32, 2] {
            let tile = l.index(k, 0, 0);
            let h = p.horizon(&l, tile, field);
            assert_eq!(h.len(), l.horizon_words() as usize);
            let side = l.horizon_side() as usize;
            // Texel (2, 2): field column 16 · 2^k (metres too: 1 m cells).
            let x = (2 * HORIZON_SPACING * l.step(k)) as f32;
            let texel = 2 * side + 2;
            let d = ridge as f32 - x;
            let want = (40.0 / d).atan().to_degrees();
            // Conservative: never under it, and over it by no more than
            // the ridge an eighth nearer.
            let most = (40.0 / (d * 0.875)).atan().to_degrees();
            let step = step.max(0.05);
            let got = angle(&h, texel, 0);
            assert!(
                got >= want - step && got <= most + step,
                "level {k}: towards the ridge {got}°, not {want}° to {most}°"
            );
            assert!(angle(&h, texel, 4).abs() <= step, "level {k}: away");
        }
    }

    /// An edit's pyramids, updated where it reaches, are the edited field's
    /// own; every tile the edit changes is one [`edited_tiles`] names, and
    /// those are a few per level, not the terrain. Edits inside the field,
    /// on a tile edge and at its far corner (where the levels' last
    /// samples clamp).
    #[test]
    fn an_edit_builds_again_only_the_tiles_it_reaches() {
        let (cols, rows) = (301, 237);
        let size = Vec2::new(300.0, 236.0);
        let control = |seed: u32| TerrainControl {
            width: 75,
            height: 59,
            rgba: (0..75 * 59 * 4)
                .map(|i| ((i as u32 * 37 + seed) % 251) as u8)
                .collect(),
        };
        let field = |c: u32, r: u32| ((c * 7 + r * 13) % 11) as f32 + 0.01 * c as f32;
        let before = TerrainData {
            size,
            columns: cols,
            rows,
            heights: (0..rows)
                .flat_map(|r| (0..cols).map(move |c| field(c, r)))
                .collect(),
            control: Some(control(0)),
            holes: Some(vec![0u8; ((cols - 1) * (rows - 1)) as usize].into()),
        };
        let l = before.layout(16).expect("layout");
        assert!(l.levels >= 5 && l.control > 0);
        let all = |d: &TerrainData| {
            let (h, c, o) = (
                d.height_pyramid(&l),
                ControlPyramid::new(&l, d.control.as_ref().unwrap()),
                d.hole_counts(),
            );
            // The surface only: a horizon sees the whole terrain, so an
            // edit moves every tile's a little ([`edited_tiles`]).
            (0..l.tile_count())
                .map(|t| TerrainTile {
                    horizon: Arc::from([]),
                    ..d.tile(&l, t, &h, c.as_ref(), o.as_ref())
                })
                .collect::<Vec<_>>()
        };
        let old = all(&before);
        let mut heights = before.height_pyramid(&l);
        let mut pyramid = ControlPyramid::new(&l, before.control.as_ref().unwrap()).unwrap();
        let mut data = before.clone();
        // A 3 m brush in the field, one on a level-0 tile edge (x 32 m)
        // and one at the far corner.
        for centre in [Vec2::new(-41.3, 17.8), Vec2::new(-118.0, -60.0), size * 0.5] {
            let (lo, hi) = (centre - 3.0, centre + 3.0);
            let ((c0, r0), (c1, r1)) = l.samples_in(lo, hi);
            let mut h = data.heights.to_vec();
            let mut holes = data.holes.as_ref().unwrap().to_vec();
            for r in r0..=r1 {
                for c in c0..=c1 {
                    h[(r * cols + c) as usize] += 5.0;
                    if c < cols - 1 && r < rows - 1 {
                        holes[(r * (cols - 1) + c) as usize] = 1;
                    }
                }
            }
            let mut map = data.control.clone().unwrap();
            let mut px = map.rgba.to_vec();
            let to = |p: Vec2| ((p + size * 0.5) / size * Vec2::new(75.0, 59.0)).floor();
            let (a, b) = (to(lo).max(Vec2::ZERO), to(hi).min(Vec2::new(74.0, 58.0)));
            for y in a.y as usize..=b.y as usize {
                for x in a.x as usize..=b.x as usize {
                    px[(y * 75 + x) * 4] = 255;
                }
            }
            map.rgba = px.into();
            data.heights = h.into();
            data.holes = Some(holes.into());
            data.control = Some(map);
            heights.update(
                &l,
                |c, r| data.heights[(r * cols + c) as usize],
                (c0, r0),
                (c1, r1),
            );
            pyramid.update(&l, data.control.as_ref().unwrap(), lo, hi);
            assert_eq!(heights, data.height_pyramid(&l), "the heights at {centre}");
            assert_eq!(
                pyramid,
                ControlPyramid::new(&l, data.control.as_ref().unwrap()).unwrap(),
                "the control map at {centre}"
            );
        }
        let new = all(&data);
        let mut named = std::collections::HashSet::new();
        for centre in [Vec2::new(-41.3, 17.8), Vec2::new(-118.0, -60.0), size * 0.5] {
            edited_tiles(&l, centre - 3.0, centre + 3.0, |t| {
                named.insert(t);
            });
        }
        let changed: Vec<u32> = (0..l.tile_count())
            .filter(|&t| old[t as usize] != new[t as usize])
            .collect();
        assert!(!changed.is_empty());
        for t in &changed {
            assert!(
                named.contains(t),
                "tile {t} ({:?}) changed unnamed",
                l.locate(*t)
            );
        }
        assert!(
            named.len() <= 3 * 4 * l.levels as usize && named.len() * 4 < l.tile_count() as usize,
            "{} of {} tiles ({} changed)",
            named.len(),
            l.tile_count(),
            changed.len()
        );
    }
}
