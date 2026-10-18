//! A placed map's current cells: terrain resolution, deterministic edits,
//! snapshots, collision rectangles and draw chunks.

use crate::map::{FLIP_X, FLIP_Y, TILE_MASK, TilemapAsset};
use crate::tileset::{Collision, Terrain, Tileset};
use glam::Vec2;
use og_core::{Error, ErrorKind, Result};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Chunks are this many cells square.
pub const CHUNK: u32 = 16;

struct LayerState {
    /// As set: tile layers' cell encoding, terrain layers 0/1.
    cells: Vec<u32>,
    /// What is drawn: (tile id + 1) | flips, 0 empty.
    resolved: Vec<u32>,
    terrain: Option<Terrain>,
    version: u64,
    chunk_versions: Vec<u64>,
}

/// The current cells of a placed map. Equal edit sequences from the same
/// asset give equal states and equal [`MapState::encode`] bytes.
pub struct MapState {
    asset: TilemapAsset,
    tilesets: Vec<Arc<Tileset>>,
    layers: Vec<LayerState>,
    chunks_x: u32,
}

impl PartialEq for MapState {
    /// Equal cells (versions are not state).
    fn eq(&self, other: &MapState) -> bool {
        self.asset == other.asset
            && self.layers.len() == other.layers.len()
            && self
                .layers
                .iter()
                .zip(&other.layers)
                .all(|(a, b)| a.cells == b.cells)
    }
}

impl std::fmt::Debug for MapState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState")
            .field("width", &self.asset.width)
            .field("height", &self.asset.height)
            .field("layers", &self.layers.len())
            .finish()
    }
}

fn invalid(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, msg)
}

/// Rectangle in map-local world units (y up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

/// One tile to draw: its bottom-left corner and size in map-local world
/// units, and its atlas region (flips applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileQuad {
    pub min: Vec2,
    pub size: Vec2,
    pub uv: [f32; 4],
}

/// An animated tile of a chunk: which quad, its frames' regions and durations.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimatedQuad {
    pub quad: u32,
    pub frames: Arc<[[f32; 4]]>,
    pub durations: Arc<[f32]>,
}

/// The tiles of one chunk.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkQuads {
    pub quads: Vec<TileQuad>,
    pub animated: Vec<AnimatedQuad>,
}

/// The frame shown at `time_seconds` of a looping animation.
pub fn animated_frame(durations: &[f32], time_seconds: f32) -> usize {
    let total: f32 = durations.iter().sum();
    if durations.is_empty() || total <= 0.0 {
        return 0;
    }
    let mut t = time_seconds.rem_euclid(total);
    for (i, &d) in durations.iter().enumerate() {
        if t < d {
            return i;
        }
        t -= d;
    }
    durations.len() - 1
}

impl MapState {
    /// The map as authored, terrain layers resolved. `tilesets` are the
    /// loaded tilesets in the order of `asset.tilesets` (an embedded one is
    /// passed as itself).
    pub fn new(asset: &TilemapAsset, tilesets: &[Arc<Tileset>]) -> Result<MapState> {
        asset.validate()?;
        if tilesets.len() != asset.tilesets.len() {
            return Err(invalid(format!(
                "the map has {} tileset(s), {} were given",
                asset.tilesets.len(),
                tilesets.len()
            )));
        }
        let chunks_x = asset.width.div_ceil(CHUNK);
        let chunks = (chunks_x * asset.height.div_ceil(CHUNK)) as usize;
        let mut layers = Vec::with_capacity(asset.layers.len());
        for l in &asset.layers {
            let ts = &tilesets[l.tileset as usize];
            let terrain = match &l.terrain {
                None => {
                    if let Some(c) = l.cells.iter().find(|&&c| c & TILE_MASK > ts.count()) {
                        return Err(invalid(format!(
                            "layer '{}' uses tile {}, its tileset has {}",
                            l.name,
                            (c & TILE_MASK) - 1,
                            ts.count()
                        )));
                    }
                    None
                }
                Some(name) => Some(
                    ts.terrains
                        .iter()
                        .find(|t| &t.name == name)
                        .cloned()
                        .ok_or_else(|| {
                            invalid(format!(
                                "layer '{}' names terrain '{name}', which its tileset ({}) does not declare",
                                l.name, ts.texture
                            ))
                        })?,
                ),
            };
            layers.push(LayerState {
                cells: l.cells.clone(),
                resolved: l.cells.clone(),
                terrain,
                version: 0,
                chunk_versions: vec![0; chunks],
            });
        }
        let mut s = MapState {
            asset: asset.clone(),
            tilesets: tilesets.to_vec(),
            layers,
            chunks_x,
        };
        for i in 0..s.layers.len() {
            if s.layers[i].terrain.is_some() {
                for y in 0..asset.height {
                    for x in 0..asset.width {
                        s.resolve(i, x, y);
                    }
                }
            }
        }
        Ok(s)
    }

    /// The asset this state started from.
    pub fn asset(&self) -> &TilemapAsset {
        &self.asset
    }

    /// The tileset of layer `layer`.
    pub fn tileset(&self, layer: usize) -> Option<&Arc<Tileset>> {
        let l = self.asset.layers.get(layer)?;
        self.tilesets.get(l.tileset as usize)
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        (x < self.asset.width && y < self.asset.height).then(|| (y * self.asset.width + x) as usize)
    }

    fn filled(&self, layer: usize, x: i64, y: i64, edge: bool) -> bool {
        if x < 0 || y < 0 || x >= self.asset.width as i64 || y >= self.asset.height as i64 {
            return edge;
        }
        self.layers[layer].cells[(y as u32 * self.asset.width + x as u32) as usize] != 0
    }

    /// Re-resolves one terrain cell; bumps its chunk when it changed.
    fn resolve(&mut self, layer: usize, x: u32, y: u32) {
        let Some(i) = self.index(x, y) else { return };
        let Some(t) = &self.layers[layer].terrain else {
            return;
        };
        let edge = t.edge_filled;
        let value = if self.layers[layer].cells[i] == 0 {
            0
        } else {
            let (x, y) = (x as i64, y as i64);
            let f = |dx: i64, dy: i64| self.filled(layer, x + dx, y + dy, edge);
            // N, NE, E, SE, S, SW, W, NW with y down.
            let n = [
                f(0, -1),
                f(1, -1),
                f(1, 0),
                f(1, 1),
                f(0, 1),
                f(-1, 1),
                f(-1, 0),
                f(-1, -1),
            ];
            t.resolve(n) + 1
        };
        if self.layers[layer].resolved[i] != value {
            self.layers[layer].resolved[i] = value;
            self.bump(layer, x, y);
        }
    }

    fn bump(&mut self, layer: usize, x: u32, y: u32) {
        let c = ((y / CHUNK) * self.chunks_x + x / CHUNK) as usize;
        let l = &mut self.layers[layer];
        l.chunk_versions[c] += 1;
        l.version += 1;
    }

    /// Tile layers: the tile id at (x, y) or -1 when empty; terrain layers:
    /// 1 filled, 0 empty. -1 outside the map or for an unknown layer.
    pub fn get(&self, layer: usize, x: u32, y: u32) -> i32 {
        let (Some(l), Some(i)) = (self.layers.get(layer), self.index(x, y)) else {
            return -1;
        };
        if l.terrain.is_some() {
            l.cells[i] as i32
        } else {
            (l.cells[i] & TILE_MASK) as i32 - 1
        }
    }

    /// The tile drawn at (x, y) with its flips (terrain layers resolved).
    pub fn tile(&self, layer: usize, x: u32, y: u32) -> Option<(u32, bool, bool)> {
        let c = self.layers.get(layer)?.resolved[self.index(x, y)?];
        (c & TILE_MASK != 0).then(|| ((c & TILE_MASK) - 1, c & FLIP_X != 0, c & FLIP_Y != 0))
    }

    /// Sets a cell. Tile layers: a tile id, or -1 to empty it. Terrain
    /// layers: 1 fills, 0 clears, and the 3×3 neighbourhood is re-resolved.
    pub fn set(&mut self, layer: usize, x: u32, y: u32, value: i32) -> Result<()> {
        let name = self
            .asset
            .layers
            .get(layer)
            .map(|l| l.name.clone())
            .ok_or_else(|| invalid(format!("layer {layer}: the map has {}", self.layers.len())))?;
        let i = self.index(x, y).ok_or_else(|| {
            invalid(format!(
                "cell ({x}, {y}) is outside the {}×{} map",
                self.asset.width, self.asset.height
            ))
        })?;
        if self.layers[layer].terrain.is_some() {
            let v = match value {
                0 => 0,
                1 => 1,
                _ => {
                    return Err(invalid(format!(
                        "layer '{name}' is a terrain layer: set 1 to fill or 0 to clear, not {value}"
                    )));
                }
            };
            if self.layers[layer].cells[i] == v {
                return Ok(());
            }
            self.layers[layer].cells[i] = v;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx >= 0 && ny >= 0 {
                        self.resolve(layer, nx as u32, ny as u32);
                    }
                }
            }
            return Ok(());
        }
        let count = self.tilesets[self.asset.layers[layer].tileset as usize].count();
        let v = if value < 0 {
            0
        } else if (value as u32) < count {
            value as u32 + 1
        } else {
            return Err(invalid(format!(
                "tile {value} is not in layer '{name}''s tileset ({count} tiles)"
            )));
        };
        if self.layers[layer].cells[i] != v {
            self.layers[layer].cells[i] = v;
            self.layers[layer].resolved[i] = v;
            self.bump(layer, x, y);
        }
        Ok(())
    }

    /// How many times layer `layer`'s drawn tiles changed.
    pub fn layer_version(&self, layer: usize) -> u64 {
        self.layers.get(layer).map_or(0, |l| l.version)
    }

    /// How many times chunk (cx, cy) of `layer` changed.
    pub fn chunk_version(&self, layer: usize, cx: u32, cy: u32) -> u64 {
        if cx >= self.chunks_x {
            return 0;
        }
        self.layers
            .get(layer)
            .and_then(|l| l.chunk_versions.get((cy * self.chunks_x + cx) as usize))
            .copied()
            .unwrap_or(0)
    }

    /// The chunk grid's size.
    pub fn chunks(&self) -> (u32, u32) {
        (self.chunks_x, self.asset.height.div_ceil(CHUNK))
    }

    /// The cells that differ from the asset's, little-endian: width,
    /// height, layer count, edit count, then (layer, index, value) in order.
    pub fn encode(&self, out: &mut Vec<u8>) {
        let mut edits: Vec<(u32, u32, u32)> = Vec::new();
        for (li, (l, a)) in self.layers.iter().zip(&self.asset.layers).enumerate() {
            for (i, (&c, &o)) in l.cells.iter().zip(&a.cells).enumerate() {
                if c != o {
                    edits.push((li as u32, i as u32, c));
                }
            }
        }
        for v in [
            self.asset.width,
            self.asset.height,
            self.layers.len() as u32,
            edits.len() as u32,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for (l, i, v) in edits {
            out.extend_from_slice(&l.to_le_bytes());
            out.extend_from_slice(&i.to_le_bytes());
            out.extend_from_slice(&v.to_le_bytes());
        }
    }

    /// The state [`MapState::encode`] wrote, over the same asset and tilesets.
    pub fn decode(
        asset: &TilemapAsset,
        tilesets: &[Arc<Tileset>],
        bytes: &[u8],
    ) -> Result<MapState> {
        let bad = |m: &str| Error::new(ErrorKind::Parse, format!("tilemap state: {m}"));
        let word = |i: usize| -> Result<u32> {
            bytes
                .get(i * 4..i * 4 + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .ok_or_else(|| bad("cut short"))
        };
        let mut s = MapState::new(asset, tilesets)?;
        if word(0)? != asset.width
            || word(1)? != asset.height
            || word(2)? as usize != s.layers.len()
        {
            return Err(bad("it was saved for a map of another size or layer count"));
        }
        let n = word(3)? as usize;
        if bytes.len() != 16 + n * 12 {
            return Err(bad("its length does not match its edit count"));
        }
        let cells = (asset.width * asset.height) as usize;
        for k in 0..n {
            let (l, i, v) = (
                word(4 + k * 3)? as usize,
                word(5 + k * 3)? as usize,
                word(6 + k * 3)?,
            );
            if l >= s.layers.len() || i >= cells {
                return Err(bad("an edit is outside the map"));
            }
            s.layers[l].cells[i] = v;
            if s.layers[l].terrain.is_none() {
                s.layers[l].resolved[i] = v;
            }
        }
        for li in 0..s.layers.len() {
            if s.layers[li].terrain.is_some() {
                for y in 0..asset.height {
                    for x in 0..asset.width {
                        s.resolve(li, x, y);
                    }
                }
            }
        }
        Ok(s)
    }

    fn cell_rect(&self, layer: usize, x: u32, y: u32) -> Rect {
        let ts = self.asset.tile_size();
        let off = self.asset.layers[layer].offset;
        Rect {
            min: Vec2::new(x as f32 * ts.x, -((y + 1) as f32) * ts.y) + off,
            max: Vec2::new((x + 1) as f32 * ts.x, -(y as f32) * ts.y) + off,
        }
    }
}

/// The collision rectangles of `layer` in map-local world units (y up),
/// including the layer's offset: full-tile solids merged greedily into
/// rectangles (runs along rows, then equal runs of consecutive rows),
/// partial rectangles one per tile (identical full-width neighbours on a
/// row merged). Ordered top to bottom, then left to right.
pub fn collision_rects(state: &MapState, layer: usize) -> Vec<Rect> {
    let Some(ts) = state.tileset(layer).cloned() else {
        return Vec::new();
    };
    let (w, h) = (state.asset.width, state.asset.height);
    let size = state.asset.tile_size();
    let (sx, sy) = (
        size.x / ts.tile_width as f32,
        size.y / ts.tile_height as f32,
    );
    // (top row, x0) → rect, for ordering.
    let mut out: Vec<(u32, u32, Rect)> = Vec::new();
    // Open full-tile rects by (x0, x1): (first row, last row).
    let mut open: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    let close = |out: &mut Vec<(u32, u32, Rect)>, (x0, x1): (u32, u32), y0: u32, y1: u32| {
        let a = state.cell_rect(layer, x0, y1);
        let b = state.cell_rect(layer, x1, y0);
        out.push((
            y0,
            x0,
            Rect {
                min: a.min,
                max: b.max,
            },
        ));
    };
    for y in 0..h {
        let mut runs: Vec<(u32, u32)> = Vec::new();
        let mut x = 0;
        while x < w {
            let full = |x: u32| {
                state
                    .tile(layer, x, y)
                    .is_some_and(|(t, _, _)| ts.collision(t) == Some(Collision::Full))
            };
            if full(x) {
                let x0 = x;
                while x + 1 < w && full(x + 1) {
                    x += 1;
                }
                runs.push((x0, x));
            }
            x += 1;
        }
        let keep: Vec<(u32, u32)> = runs.clone();
        let ended: Vec<((u32, u32), u32)> = open
            .iter()
            .filter(|(k, _)| !keep.contains(k))
            .map(|(k, v)| (*k, *v))
            .collect();
        for (k, y0) in ended {
            open.remove(&k);
            close(&mut out, k, y0, y - 1);
        }
        for r in runs {
            open.entry(r).or_insert(y);
        }
        // Partial rects on this row.
        let partial = |x: u32| -> Option<(f32, f32, f32, f32)> {
            let (t, fx, fy) = state.tile(layer, x, y)?;
            let Some(Collision::Rect {
                x: rx,
                y: ry,
                w: rw,
                h: rh,
            }) = ts.collision(t)
            else {
                return None;
            };
            let (tw, th) = (ts.tile_width as f32, ts.tile_height as f32);
            Some((
                if fx { tw - rx - rw } else { rx },
                if fy { th - ry - rh } else { ry },
                rw,
                rh,
            ))
        };
        let spans = |r: (f32, f32, f32, f32)| r.0 <= 0.0 && r.2 >= ts.tile_width as f32;
        let mut x = 0;
        while x < w {
            if let Some(r) = partial(x) {
                let (rx, ry, rw, rh) = r;
                let mut x1 = x;
                if spans(r) {
                    while x1 + 1 < w
                        && partial(x1 + 1).is_some_and(|n| spans(n) && n.1 == ry && n.3 == rh)
                    {
                        x1 += 1;
                    }
                }
                let cell = state.cell_rect(layer, x, y);
                let min_x = cell.min.x + rx * sx;
                let top = cell.max.y - ry * sy;
                let max_x = if x1 > x {
                    state.cell_rect(layer, x1, y).max.x
                } else {
                    min_x + rw * sx
                };
                out.push((
                    y,
                    x,
                    Rect {
                        min: Vec2::new(min_x, top - rh * sy),
                        max: Vec2::new(max_x, top),
                    },
                ));
                x = x1;
            }
            x += 1;
        }
    }
    let rest: Vec<((u32, u32), u32)> = open.into_iter().collect();
    for (k, y0) in rest {
        close(&mut out, k, y0, h - 1);
    }
    out.sort_by(|a, b| {
        (a.0, a.1)
            .cmp(&(b.0, b.1))
            .then(a.2.min.y.total_cmp(&b.2.min.y).reverse())
    });
    out.into_iter().map(|(_, _, r)| r).collect()
}

/// [`collision_rects`] of every layer with `collision`, in layer order.
pub fn collision_rects_all(state: &MapState) -> Vec<Rect> {
    (0..state.asset.layers.len())
        .filter(|&l| state.asset.layers[l].collision)
        .flat_map(|l| collision_rects(state, l))
        .collect()
}

fn flipped(mut uv: [f32; 4], fx: bool, fy: bool) -> [f32; 4] {
    if fx {
        uv.swap(0, 2);
    }
    if fy {
        uv.swap(1, 3);
    }
    uv
}

/// The quads of chunk (cx, cy) of `layer` (map-local, without the layer's
/// offset or parallax, which the drawer adds), with flips applied to the
/// regions and the animated tiles' frames.
pub fn chunk_quads(
    state: &MapState,
    layer: usize,
    cx: u32,
    cy: u32,
    texture_w: u32,
    texture_h: u32,
) -> ChunkQuads {
    let mut out = ChunkQuads::default();
    let Some(ts) = state.tileset(layer).cloned() else {
        return out;
    };
    let size = state.asset.tile_size();
    type Frames = (Arc<[[f32; 4]]>, Arc<[f32]>);
    let mut anims: BTreeMap<(u32, bool, bool), Frames> = BTreeMap::new();
    let (w, h) = (state.asset.width, state.asset.height);
    for y in cy * CHUNK..((cy + 1) * CHUNK).min(h) {
        for x in cx * CHUNK..((cx + 1) * CHUNK).min(w) {
            let Some((t, fx, fy)) = state.tile(layer, x, y) else {
                continue;
            };
            let index = out.quads.len() as u32;
            out.quads.push(TileQuad {
                min: Vec2::new(x as f32 * size.x, -((y + 1) as f32) * size.y),
                size,
                uv: flipped(ts.uv(t, texture_w, texture_h), fx, fy),
            });
            if let Some(a) = ts.tiles.get(&t).and_then(|d| d.anim.as_ref()) {
                let (frames, durations) = anims
                    .entry((t, fx, fy))
                    .or_insert_with(|| {
                        (
                            a.frames
                                .iter()
                                .map(|&f| flipped(ts.uv(f, texture_w, texture_h), fx, fy))
                                .collect(),
                            a.durations.iter().copied().collect(),
                        )
                    })
                    .clone();
                out.animated.push(AnimatedQuad {
                    quad: index,
                    frames,
                    durations,
                });
            }
        }
    }
    out
}

/// The chunks of `layer` a map-local world rectangle (y up) touches, row by
/// row from the top.
pub fn chunks_in(
    state: &MapState,
    layer: usize,
    view_min: Vec2,
    view_max: Vec2,
) -> impl Iterator<Item = (u32, u32)> + use<> {
    let mut out = Vec::new();
    if layer < state.layers.len() {
        let size = state.asset.tile_size();
        let off = state.asset.layers[layer].offset;
        let (lo, hi) = (view_min - off, view_max - off);
        let x0 = (lo.x / size.x).floor().max(0.0);
        let x1 = (hi.x / size.x).floor().min(state.asset.width as f32 - 1.0);
        let y0 = (-hi.y / size.y).floor().max(0.0);
        let y1 = (-lo.y / size.y)
            .floor()
            .min(state.asset.height as f32 - 1.0);
        if x0 <= x1 && y0 <= y1 {
            let (cx0, cx1) = (x0 as u32 / CHUNK, x1 as u32 / CHUNK);
            let (cy0, cy1) = (y0 as u32 / CHUNK, y1 as u32 / CHUNK);
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    out.push((cx, cy));
                }
            }
        }
    }
    out.into_iter()
}
