//! Atlas grids, collision shapes, animated tiles and explicit autotile rules.
use og_core::IdMap;
use og_core::{Error, ErrorKind, Result};
use std::collections::{BTreeMap, BTreeSet};

mod cooked;

/// How the atlas is sampled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    /// Pixel art: no smoothing.
    #[default]
    Nearest,
    Linear,
}

/// A tile's collision shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Collision {
    Full,
    /// Pixels from the tile's top-left corner, y down.
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

/// Tile IDs in playback order, one strictly positive duration per frame.
#[derive(Clone, Debug, PartialEq)]
pub struct TileAnim {
    pub frames: Vec<u32>,
    pub durations: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileDef {
    pub collision: Option<Collision>,
    /// Closed polygons in pixels from the top-left. Cook tessellates curves.
    pub polygons: Vec<Vec<[f32; 2]>>,
    pub anim: Option<TileAnim>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainMode {
    /// Eight neighbours; corners count only between two filled edges.
    Blob47,
    /// Four edge neighbours, N=1 E=2 S=4 W=8.
    Bitmask16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub name: String,
    pub mode: TerrainMode,
    pub rules: BTreeMap<u8, u32>,
    pub default: u32,
    pub edge_filled: bool,
}

/// Neighbour bits for blob47; bitmask16 uses N=1 E=2 S=4 W=8.
pub mod mask {
    pub const N: u8 = 1;
    pub const NE: u8 = 2;
    pub const E: u8 = 4;
    pub const SE: u8 = 8;
    pub const S: u8 = 16;
    pub const SW: u8 = 32;
    pub const W: u8 = 64;
    pub const NW: u8 = 128;
}

/// Drop a corner unless both adjacent edges are filled (47 distinct masks).
pub fn reduce_blob(m: u8) -> u8 {
    use mask::*;
    let mut out = m & (N | E | S | W);
    for (corner, a, b) in [(NE, N, E), (SE, S, E), (SW, S, W), (NW, N, W)] {
        if m & corner != 0 && m & a != 0 && m & b != 0 {
            out |= corner;
        }
    }
    out
}

impl Terrain {
    /// Tile for N, NE, E, SE, S, SW, W, NW occupancy.
    pub fn resolve(&self, neighbours: [bool; 8]) -> u32 {
        let m = match self.mode {
            TerrainMode::Blob47 => {
                let mut m = 0u8;
                for (i, &f) in neighbours.iter().enumerate() {
                    if f {
                        m |= 1 << i;
                    }
                }
                reduce_blob(m)
            }
            TerrainMode::Bitmask16 => {
                let [n, _, e, _, s, _, w, _] = neighbours;
                (n as u8) | (e as u8) << 1 | (s as u8) << 2 | (w as u8) << 3
            }
        };
        self.rules.get(&m).copied().unwrap_or(self.default)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tileset {
    /// Atlas asset ID; empty for collision-only tilesets.
    pub texture: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub margin: u32,
    pub spacing: u32,
    pub filter: Filter,
    pub tiles: IdMap<u32, TileDef>,
    pub terrains: Vec<Terrain>,
}

fn err(line: usize, msg: impl std::fmt::Display) -> Error {
    if line == 0 {
        Error::new(ErrorKind::Invalid, format!("tileset: {msg}"))
    } else {
        Error::new(ErrorKind::Parse, format!("tileset line {line}: {msg}"))
    }
}

impl Tileset {
    /// Collision overhang in cells, including diagonal flips. Shared by
    /// union halos and edit invalidation so neighbouring colliders stay fresh.
    pub fn collision_halo(&self) -> u32 {
        let mut extent = self.tile_width.max(self.tile_height) as f32;
        for d in self.tiles.values() {
            for &[x, y] in d.polygons.iter().flatten() {
                extent = extent.max(x.abs()).max(y.abs());
            }
            if let Some(Collision::Rect { x, y, w, h }) = d.collision {
                for v in [x, y, x + w, y + h] {
                    extent = extent.max(v.abs());
                }
            }
        }
        (((extent * 2.0) / self.tile_width.min(self.tile_height).max(1) as f32).ceil() as u32)
            .saturating_add(1)
    }

    /// Tile count of a validated grid.
    pub fn count(&self) -> u32 {
        self.columns * self.rows
    }
    pub fn terrain(&self, name: &str) -> Option<u32> {
        self.terrains
            .iter()
            .position(|t| t.name == name)
            .map(|i| i as u32)
    }
    pub fn collision(&self, tile: u32) -> Option<Collision> {
        self.tiles.get(&tile).and_then(|d| d.collision)
    }

    /// Atlas UVs, v down, inset by 1/512 texel to prevent neighbour bleed.
    pub fn uv(&self, tile: u32, texture_w: u32, texture_h: u32) -> [f32; 4] {
        let cols = self.columns.max(1);
        let (col, row) = (tile % cols, tile / cols);
        let x0 = (self.margin + col * (self.tile_width + self.spacing)) as f32;
        let y0 = (self.margin + row * (self.tile_height + self.spacing)) as f32;
        let (w, h) = (texture_w.max(1) as f32, texture_h.max(1) as f32);
        let eps = 1.0 / 512.0;
        [
            (x0 + eps) / w,
            (y0 + eps) / h,
            (x0 + self.tile_width as f32 - eps) / w,
            (y0 + self.tile_height as f32 - eps) / h,
        ]
    }

    /// Domain checks shared by author cooking, external imports and native loading.
    pub fn validate(&self) -> Result<()> {
        if self.tile_width == 0 || self.tile_height == 0 || self.columns == 0 || self.rows == 0 {
            return Err(err(
                0,
                "the grid's tile size, columns and rows must be at least 1",
            ));
        }
        let n = self
            .columns
            .checked_mul(self.rows)
            .ok_or_else(|| err(0, "grid tile count overflows u32"))?;
        for (tile, count) in [
            (self.tile_width, self.columns),
            (self.tile_height, self.rows),
        ] {
            tile.checked_add(self.spacing)
                .and_then(|stride| stride.checked_mul(count))
                .and_then(|span| span.checked_add(self.margin))
                .ok_or_else(|| err(0, "atlas pixel extent overflows u32"))?;
        }
        let check = |id: u32, what: &str| -> Result<()> {
            if id >= n {
                Err(err(
                    0,
                    format!(
                        "{what} names tile {id}, but the grid has {n} tiles (0..{})",
                        n - 1
                    ),
                ))
            } else {
                Ok(())
            }
        };
        for (&id, d) in &self.tiles {
            check(id, "a tile entry")?;
            if let Some(Collision::Rect { x, y, w, h }) = d.collision
                && (![x, y, w, h, x + w, y + h].iter().all(|v| v.is_finite())
                    || w <= 0.0
                    || h <= 0.0)
            {
                return Err(err(
                    0,
                    format!("tile {id}: collision needs finite coordinates and positive size"),
                ));
            }
            if d.polygons
                .iter()
                .any(|p| p.len() < 3 || p.iter().flatten().any(|x| !x.is_finite()))
            {
                return Err(err(
                    0,
                    format!("tile {id}: a collision polygon needs at least three finite points"),
                ));
            }
            if let Some(a) = &d.anim {
                if a.frames.is_empty() || a.frames.len() != a.durations.len() {
                    return Err(err(
                        0,
                        format!("tile {id}: an animation needs frames, one duration each"),
                    ));
                }
                for &f in &a.frames {
                    check(f, &format!("tile {id}'s frames"))?;
                }
                if a.durations.iter().any(|d| !(d.is_finite() && *d > 0.0))
                    || !a.durations.iter().sum::<f32>().is_finite()
                {
                    return Err(err(
                        0,
                        format!("tile {id}: frame durations must be above 0 with a finite sum"),
                    ));
                }
            }
        }
        let mut names = BTreeSet::new();
        for tr in &self.terrains {
            if !names.insert(&tr.name) {
                return Err(err(0, format!("terrain '{}' is declared twice", tr.name)));
            }
            check(tr.default, &format!("terrain '{}' default", tr.name))?;
            for (&m, &tile) in &tr.rules {
                if match tr.mode {
                    TerrainMode::Blob47 => reduce_blob(m) != m,
                    TerrainMode::Bitmask16 => m > 15,
                } {
                    return Err(err(
                        0,
                        format!("terrain '{}': invalid {:?} mask {m}", tr.name, tr.mode),
                    ));
                }
                check(tile, &format!("terrain '{}' rule {m}", tr.name))?;
            }
        }
        Ok(())
    }
}
