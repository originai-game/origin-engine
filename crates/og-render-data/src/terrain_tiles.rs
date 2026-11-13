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

mod control_pyramid;
mod height_pyramid;

pub use control_pyramid::ControlPyramid;
pub use height_pyramid::HeightPyramid;

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
mod tests;
