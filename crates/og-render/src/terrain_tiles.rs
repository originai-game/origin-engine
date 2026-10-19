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
//! Every level's samples are the field's own (decimated, never averaged),
//! so two levels agree exactly where their grids meet: a clipmap level
//! reading a coarser tile than its neighbour leaves no crack.
//!
//! **Residency.** [`terrain_tiles_wanted`] answers the tiles a frame needs,
//! best first: for each clipmap level, the tiles of the pyramid level whose
//! spacing matches its own under the level's square, their ancestors, and
//! the level-0 tiles within a detail distance (scatter); truncated to a
//! capacity, ancestors always before their children. The renderer and the
//! runtime's streamer call the same function, so what is decoded is what
//! is drawn. A vertex or pixel reads the finest resident level at its
//! point, so a tile that has not arrived shows its parent.

use crate::terrain::{Terrain, TerrainControl, TerrainData, horizontal_scale};
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// Quads per tile side when nothing else is asked for.
pub const TERRAIN_TILE: u32 = 128;
/// The most tiles (all levels together) one terrain's table holds: the
/// shader's tile table is one 16 KiB uniform block of one f32 per tile.
pub const MAX_TERRAIN_TILES: u32 = 4096;
/// The most pyramid levels.
pub const MAX_TERRAIN_PYRAMID: u32 = 16;

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

    /// Normal-map texels per tile side (its samples).
    pub fn normals_side(&self) -> u32 {
        self.tile + 1
    }

    /// Control texels per tile side with the border (0 without a map).
    pub fn control_side(&self) -> u32 {
        if self.control == 0 {
            0
        } else {
            self.control + 2
        }
    }

    /// The GPU bytes one resident tile takes (heights, normals and control
    /// with their mips).
    pub fn slot_bytes(&self) -> u64 {
        let h = self.heights_side() as u64;
        let n = mip_texels(self.normals_side());
        let c = if self.control == 0 {
            0
        } else {
            mip_texels(self.control_side())
        };
        h * h * 4 + n * 4 + c * 4
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
pub(crate) fn mip_sides(side: u32) -> Vec<u32> {
    let mut out = vec![side.max(1)];
    while *out.last().expect("one") > 1 {
        out.push((out.last().expect("one") / 2).max(1));
    }
    out
}

/// One tile ready to draw: its heights (metres, `(tile + 3)²`, the border
/// included), its normal map's x and z as half floats (`(tile + 1)²` and
/// every mip after it), its control texels (`(control + 2)²` RGBA8 and its
/// mips) and the range of its heights.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainTile {
    pub heights: Arc<[f32]>,
    pub normals: Arc<[u16]>,
    pub control: Option<Arc<[u8]>>,
    pub range: (f32, f32),
}

/// An f32 as the nearest half float's bits.
pub(crate) fn half(v: f32) -> u16 {
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let mant = b & 0x7f_ffff;
    if exp >= 31 {
        return sign | 0x7c00;
    }
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = mant | 0x80_0000;
        let shift = (14 - exp) as u32;
        let h = m >> shift;
        let rem = m & ((1 << shift) - 1);
        let round = 1 << (shift - 1);
        let h = if rem > round || (rem == round && h & 1 == 1) {
            h + 1
        } else {
            h
        };
        return sign | h as u16;
    }
    let h = ((exp as u32) << 10) | (mant >> 13);
    let rem = mant & 0x1fff;
    let h = if rem > 0x1000 || (rem == 0x1000 && h & 1 == 1) {
        h + 1
    } else {
        h
    };
    sign | h as u16
}

impl TerrainTile {
    /// A tile from its heights (`(tile + 3)²` at its level, the border
    /// clamped at the terrain's edge) and control texels: builds the normal
    /// map (central differences, one-sided at the terrain's edge, as the
    /// collider's surface) and the mips.
    pub fn build(
        layout: &TerrainLayout,
        index: u32,
        heights: Vec<f32>,
        control: Option<Vec<u8>>,
    ) -> TerrainTile {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let side = layout.heights_side() as usize;
        let ns = layout.normals_side() as usize;
        let t = layout.tile as i64;
        let (sx, sz) = layout.samples(k);
        let cell = layout.cell();
        let h = |i: i64, j: i64| heights[(j + 1) as usize * side + (i + 1) as usize];
        let mut xz = vec![[0.0f32; 2]; ns * ns];
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
                let n = Vec3::new(-hx, 1.0, -hz).normalize();
                xz[j as usize * ns + i as usize] = [n.x, n.z];
                let v = h(i, j);
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        let mut normals = Vec::new();
        let mut level = xz;
        let mut s = ns;
        loop {
            normals.extend(level.iter().flat_map(|p| [half(p[0]), half(p[1])]));
            if s == 1 {
                break;
            }
            let d = (s / 2).max(1);
            let mut next = vec![[0.0f32; 2]; d * d];
            for y in 0..d {
                for x in 0..d {
                    let mut acc = [0.0f32; 2];
                    for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let p = level[(2 * y + oy).min(s - 1) * s + (2 * x + ox).min(s - 1)];
                        acc[0] += p[0] * 0.25;
                        acc[1] += p[1] * 0.25;
                    }
                    next[y * d + x] = acc;
                }
            }
            level = next;
            s = d;
        }
        let control = control.map(|c| {
            let cs = layout.control_side() as usize;
            let mut out = c.clone();
            let mut level = c;
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
            Arc::from(out)
        });
        TerrainTile {
            heights: Arc::from(heights),
            normals: Arc::from(normals),
            control,
            range: (lo, hi),
        }
    }
}

/// A terrain's control map resampled for its tiles: level 0 at
/// `control` texels per tile (bilinear between the map's texel centres),
/// each level after it halved (2 × 2 means, rounded).
#[derive(Clone, Debug, PartialEq)]
pub struct ControlPyramid {
    /// (width, height, RGBA8) per level.
    pub levels: Vec<(u32, u32, Vec<u8>)>,
}

impl ControlPyramid {
    /// `None` for a layout without a control map.
    pub fn new(layout: &TerrainLayout, map: &TerrainControl) -> Option<ControlPyramid> {
        if layout.control == 0 {
            return None;
        }
        let c = layout.control;
        let (tx, tz) = layout.tiles(0);
        let (w, h) = (c * tx, c * tz);
        let quads = Vec2::new((layout.columns - 1) as f32, (layout.rows - 1) as f32);
        let texel = |x: i64, y: i64, k: usize| {
            let x = x.clamp(0, map.width as i64 - 1) as usize;
            let y = y.clamp(0, map.height as i64 - 1) as usize;
            map.rgba[(y * map.width as usize + x) * 4 + k] as f32
        };
        let mut base = vec![0u8; (w * h * 4) as usize];
        for j in 0..h {
            for i in 0..w {
                // The texel's centre as a fraction of the terrain, then in the map's texels.
                let uv = Vec2::new(
                    (i as f32 + 0.5) * layout.tile as f32 / c as f32,
                    (j as f32 + 0.5) * layout.tile as f32 / c as f32,
                ) / quads;
                let t = uv.clamp(Vec2::ZERO, Vec2::ONE)
                    * Vec2::new(map.width as f32, map.height as f32)
                    - 0.5;
                let f0 = t.floor();
                let f = t - f0;
                let (x0, y0) = (f0.x as i64, f0.y as i64);
                for k in 0..4 {
                    let a = texel(x0, y0, k) + (texel(x0 + 1, y0, k) - texel(x0, y0, k)) * f.x;
                    let b = texel(x0, y0 + 1, k)
                        + (texel(x0 + 1, y0 + 1, k) - texel(x0, y0 + 1, k)) * f.x;
                    base[((j * w + i) * 4) as usize + k] =
                        (a + (b - a) * f.y).round().clamp(0.0, 255.0) as u8;
                }
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
                    for ch in 0..4 {
                        let mut sum = 0u32;
                        for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let x = (2 * i + ox).min(pw - 1);
                            let y = (2 * j + oy).min(ph - 1);
                            sum += prev[((y * pw + x) * 4 + ch) as usize] as u32;
                        }
                        next[((j * w + i) * 4 + ch) as usize] = ((sum + 2) / 4) as u8;
                    }
                }
            }
            levels.push((w, h, next));
        }
        Some(ControlPyramid { levels })
    }

    /// The `(control + 2)²` texels of a tile (its texels and a border of
    /// one, clamped at the map's edge).
    pub fn tile(&self, layout: &TerrainLayout, index: u32) -> Vec<u8> {
        let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
        let (w, h, px) = &self.levels[k as usize];
        let c = layout.control as i64;
        let side = layout.control_side() as usize;
        let mut out = Vec::with_capacity(side * side * 4);
        for j in -1..=c {
            for i in -1..=c {
                let x = (tx as i64 * c + i).clamp(0, *w as i64 - 1) as usize;
                let y = (tz as i64 * c + j).clamp(0, *h as i64 - 1) as usize;
                let at = (y * *w as usize + x) * 4;
                out.extend_from_slice(&px[at..at + 4]);
            }
        }
        out
    }
}

impl TerrainData {
    /// This field's layout in tiles of `tile` quads.
    pub fn layout(&self, tile: u32) -> Result<TerrainLayout, String> {
        TerrainLayout::new(
            self.size,
            self.columns,
            self.rows,
            tile,
            self.control.as_ref().map(|c| (c.width, c.height)),
        )
    }

    /// A tile's `(tile + 3)²` heights: the field's samples under its
    /// level's samples, clamped at the edge.
    pub fn tile_heights(&self, layout: &TerrainLayout, index: u32) -> Vec<f32> {
        tile_heights(layout, index, |c, r| {
            self.heights[(r * self.columns + c) as usize]
        })
    }

    /// A tile ready to draw (`control`: this field's control pyramid).
    pub fn tile(
        &self,
        layout: &TerrainLayout,
        index: u32,
        control: Option<&ControlPyramid>,
    ) -> TerrainTile {
        TerrainTile::build(
            layout,
            index,
            self.tile_heights(layout, index),
            control.map(|p| p.tile(layout, index)),
        )
    }
}

/// A tile's `(tile + 3)²` heights from a field read as `field(column, row)`.
pub fn tile_heights(
    layout: &TerrainLayout,
    index: u32,
    field: impl Fn(u32, u32) -> f32,
) -> Vec<f32> {
    let (k, tx, tz) = layout.locate(index).unwrap_or((0, 0, 0));
    let t = layout.tile as i64;
    let (sx, sz) = layout.samples(k);
    let side = layout.heights_side() as i64;
    let mut out = Vec::with_capacity((side * side) as usize);
    for j in 0..side {
        let gz = (tz as i64 * t + j - 1).clamp(0, sz as i64 - 1);
        let r = layout.fine(k, gz, layout.rows);
        for i in 0..side {
            let gx = (tx as i64 * t + i - 1).clamp(0, sx as i64 - 1);
            out.push(field(layout.fine(k, gx, layout.columns), r));
        }
    }
    out
}

/// Where a terrain is seen from, for [`terrain_tiles_wanted`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainView {
    /// The camera in the world.
    pub camera: Vec3,
    /// Its far plane (metres).
    pub far: f32,
    /// The angle one pixel spans at the centre of the view (radians): the
    /// vertical field of view over the target's height.
    pub pixel: f32,
    /// Metres from the camera within which the finest tiles are wanted
    /// whatever their size on screen (scattered copies stand on them).
    pub detail: f32,
}

impl TerrainView {
    /// `camera`'s view on a target `height` pixels tall (an orthographic
    /// camera's pixel taken at the distance to its target).
    pub fn of(camera: &crate::Camera, height: u32) -> TerrainView {
        match camera.projection {
            crate::Projection::Perspective { fov_y_degrees } => {
                TerrainView::perspective(camera.position, camera.far, fov_y_degrees, height)
            }
            crate::Projection::Orthographic { height: h } => TerrainView {
                camera: camera.position,
                far: camera.far,
                pixel: h / height.max(1) as f32 / camera.position.distance(camera.target).max(1.0),
                detail: 0.0,
            },
        }
    }

    /// A perspective camera's view on a target `height` pixels tall.
    pub fn perspective(camera: Vec3, far: f32, fov_y_degrees: f32, height: u32) -> TerrainView {
        TerrainView {
            camera,
            far,
            pixel: 2.0 * (fov_y_degrees.to_radians() * 0.5).tan() / height.max(1) as f32,
            detail: 0.0,
        }
    }
}

/// The tiles a frame needs for `terrain` (drawn with `layout`, its heights
/// within `range`) seen from `view`, best first, at most `capacity`. From
/// the top level down, the tile whose samples look largest on screen (its
/// spacing over its distance from the camera, in pixels) is taken and its
/// four children become candidates, until every candidate's samples are
/// under a pixel or the capacity is reached; the level-0 tiles within
/// `view.detail` come right after the top level. Every tile comes after its
/// parent, and the top level is always first.
pub fn terrain_tiles_wanted(
    layout: &TerrainLayout,
    terrain: &Terrain,
    range: (f32, f32),
    view: &TerrainView,
    capacity: usize,
) -> Vec<u32> {
    use std::collections::{BinaryHeap, HashSet};
    let model = terrain.transform;
    let along = horizontal_scale(model);
    let local = model.inverse().transform_point3(view.camera);
    let eye = Vec2::new(local.x, local.z);
    let cell = layout.cell().min_element();
    let far = view.far / along;
    let pixel = view.pixel.max(1e-6);
    let dy = local.y - local.y.clamp(range.0, range.1);
    // How large a tile's samples look: their spacing over the distance, in pixels.
    let error = |index: u32| -> f32 {
        let (k, ..) = layout.locate(index).unwrap_or((0, 0, 0));
        let (lo, hi) = layout.rect(index);
        let d_xz = (eye.clamp(lo, hi) - eye).length();
        let d = (d_xz * d_xz + dy * dy).sqrt();
        if d_xz > far {
            return 0.0;
        }
        cell * layout.step(k) as f32 / (d.max(cell * 0.5) * pixel)
    };
    #[derive(PartialEq)]
    struct Candidate(f32, u32);
    impl Eq for Candidate {}
    impl PartialOrd for Candidate {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Candidate {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.total_cmp(&other.0).then(other.1.cmp(&self.1))
        }
    }
    let capacity = capacity.max(1);
    let mut out = Vec::new();
    let mut taken = HashSet::new();
    let mut heap = BinaryHeap::new();
    let top = layout.levels - 1;
    let (tx, tz) = layout.tiles(top);
    for z in 0..tz {
        for x in 0..tx {
            let i = layout.index(top, x, z);
            out.push(i);
            taken.insert(i);
            heap.push(Candidate(error(i), i));
        }
    }
    // The finest tiles near the camera (and their ancestors), whatever their size on screen.
    if view.detail > 0.0 {
        let d = view.detail / along;
        let mut near = Vec::new();
        cover(layout, 0, eye - d, eye + d, |i| near.push(i));
        near.sort_by(|a, b| {
            let da = (eye.clamp(layout.rect(*a).0, layout.rect(*a).1) - eye).length();
            let db = (eye.clamp(layout.rect(*b).0, layout.rect(*b).1) - eye).length();
            da.total_cmp(&db).then(a.cmp(b))
        });
        for i in near {
            let mut chain = vec![i];
            while let Some(p) = layout.parent(*chain.last().expect("one")) {
                chain.push(p);
            }
            for t in chain.into_iter().rev() {
                if taken.insert(t) {
                    out.push(t);
                    heap.push(Candidate(error(t), t));
                }
            }
        }
    }
    out.truncate(capacity);
    taken = out.iter().copied().collect();
    // Refine where the samples look largest.
    let mut expanded = HashSet::new();
    while out.len() < capacity {
        let Some(Candidate(e, i)) = heap.pop() else {
            break;
        };
        if e <= 1.0 {
            break;
        }
        if !taken.contains(&i) || !expanded.insert(i) {
            continue;
        }
        let Some((k, x, z)) = layout.locate(i) else {
            continue;
        };
        if k == 0 {
            continue;
        }
        let (cx, cz) = layout.tiles(k - 1);
        let mut children: Vec<u32> = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .map(|(ox, oz)| (2 * x + ox, 2 * z + oz))
            .filter(|&(x, z)| x < cx && z < cz)
            .map(|(x, z)| layout.index(k - 1, x, z))
            .collect();
        children.sort_by(|a, b| error(*b).total_cmp(&error(*a)).then(a.cmp(b)));
        for c in children {
            if out.len() >= capacity {
                break;
            }
            if taken.insert(c) {
                out.push(c);
            }
            heap.push(Candidate(error(c), c));
        }
    }
    out
}

/// Calls `f` with every tile of `level` overlapping the rectangle `a`..`b`
/// (the terrain's space, metres from its centre).
fn cover(layout: &TerrainLayout, level: u32, a: Vec2, b: Vec2, mut f: impl FnMut(u32)) {
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
