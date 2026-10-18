//! Tilesets: an atlas image cut into a grid, per-tile collision and
//! animation, and auto-tiling terrains (`.ogt` of `type tileset`).

use og_core::ogt::{Block, Document, Entry, Number, Value};
use og_core::{Error, ErrorKind, Result};
use std::collections::BTreeMap;

/// How the atlas is sampled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    /// Pixel art: no smoothing (the default).
    #[default]
    Nearest,
    Linear,
}

/// A tile's collision shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Collision {
    /// The whole tile.
    Full,
    /// A rectangle in pixels from the tile's top-left corner (x, y down, w, h).
    Rect { x: f32, y: f32, w: f32, h: f32 },
}

/// An animated tile: these tile ids in turn, each for its duration.
#[derive(Clone, Debug, PartialEq)]
pub struct TileAnim {
    pub frames: Vec<u32>,
    /// Seconds per frame, one per frame.
    pub durations: Vec<f32>,
}

/// What a tileset says about one tile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileDef {
    pub collision: Option<Collision>,
    pub anim: Option<TileAnim>,
}

/// How a terrain picks a tile from its neighbours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainMode {
    /// Eight neighbours, corners counted only between two filled edges (47 masks).
    Blob47,
    /// Four edge neighbours (16 masks).
    Bitmask16,
}

/// An auto-tiling terrain.
#[derive(Clone, Debug, PartialEq)]
pub struct Terrain {
    pub name: String,
    pub mode: TerrainMode,
    /// Neighbour mask → tile id.
    pub rules: BTreeMap<u8, u32>,
    /// The tile for a mask no rule lists.
    pub default: u32,
    /// Cells outside the map count as filled (walls continue off the edge).
    pub edge_filled: bool,
}

/// Neighbour bits of the masks: edges for both modes (bitmask16 uses
/// N=1 E=2 S=4 W=8), the blob47 bits below.
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

/// A blob47 mask with every corner bit dropped unless both of its adjacent
/// edge bits are set: the 47 distinct values a terrain's rules name.
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
    /// The tile for a cell whose filled neighbours are `n`, `ne`, ... (in
    /// [`mask`] order: N, NE, E, SE, S, SW, W, NW).
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

/// A tileset.
#[derive(Clone, Debug, PartialEq)]
pub struct Tileset {
    /// Asset id of the atlas image (empty for a collision-only tileset).
    pub texture: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub margin: u32,
    pub spacing: u32,
    pub filter: Filter,
    /// Per tile id, what is declared about it (tiles not listed have nothing).
    pub tiles: BTreeMap<u32, TileDef>,
    pub terrains: Vec<Terrain>,
}

fn err(line: usize, msg: impl std::fmt::Display) -> Error {
    if line == 0 {
        Error::new(ErrorKind::Invalid, format!("tileset: {msg}"))
    } else {
        Error::new(ErrorKind::Parse, format!("tileset line {line}: {msg}"))
    }
}

pub(crate) fn uint(e: &Entry, i: usize) -> Result<u32> {
    let n = e.number(i).map_err(|_| {
        err(
            e.line,
            format!("'{}' needs a whole number at {}", e.key, i + 1),
        )
    })?;
    n.as_u64()
        .ok()
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| {
            err(
                e.line,
                format!("'{}': {} is not a whole number", e.key, n.text()),
            )
        })
}

pub(crate) fn num(e: &Entry, i: usize) -> Result<f32> {
    e.f32(i)
        .map_err(|_| err(e.line, format!("'{}' needs a number at {}", e.key, i + 1)))
}

impl Tileset {
    /// Tiles in the grid.
    pub fn count(&self) -> u32 {
        self.columns * self.rows
    }

    /// The terrain named `name`, by index.
    pub fn terrain(&self, name: &str) -> Option<u32> {
        self.terrains
            .iter()
            .position(|t| t.name == name)
            .map(|i| i as u32)
    }

    /// The tile's collision, if any.
    pub fn collision(&self, tile: u32) -> Option<Collision> {
        self.tiles.get(&tile).and_then(|d| d.collision)
    }

    /// The atlas region of `tile`, `[u0, v0, u1, v1]` with v down, inset by
    /// 1/512 of a texel on each side so neighbours never bleed in.
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

    /// Parses `.ogt` text of `type tileset`; errors name the line.
    pub fn parse(text: &str) -> Result<Tileset> {
        let doc = Document::parse(text)?;
        if doc.ty != "tileset" {
            return Err(err(
                0,
                format!("the file is of type '{}', not 'tileset'", doc.ty),
            ));
        }
        if doc.version != 1 {
            return Err(err(
                0,
                format!("version {} is not supported (1 is)", doc.version),
            ));
        }
        let mut t = Tileset {
            texture: String::new(),
            tile_width: 0,
            tile_height: 0,
            columns: 0,
            rows: 0,
            margin: 0,
            spacing: 0,
            filter: Filter::Nearest,
            tiles: BTreeMap::new(),
            terrains: Vec::new(),
        };
        let mut grid = false;
        let mut texture = false;
        for e in &doc.body.entries {
            match e.key.as_str() {
                "texture" => {
                    t.texture = e
                        .string(0)
                        .map_err(|_| err(e.line, "texture needs a quoted asset path"))?
                        .to_string();
                    texture = true;
                }
                "grid" => {
                    t.tile_width = uint(e, 0)?;
                    t.tile_height = uint(e, 1)?;
                    t.columns = uint(e, 2)?;
                    t.rows = uint(e, 3)?;
                    grid = true;
                }
                "margin" => t.margin = uint(e, 0)?,
                "spacing" => t.spacing = uint(e, 0)?,
                "filter" => {
                    t.filter = match e.word(0).unwrap_or("") {
                        "nearest" => Filter::Nearest,
                        "linear" => Filter::Linear,
                        _ => return Err(err(e.line, "filter is nearest or linear")),
                    }
                }
                "solid" => {
                    for i in 0..e.values.len() {
                        let id = uint(e, i)?;
                        t.tiles.entry(id).or_default().collision = Some(Collision::Full);
                    }
                }
                "tile" => {
                    let id = uint(e, 0)?;
                    let block = e
                        .block
                        .as_ref()
                        .ok_or_else(|| err(e.line, "tile needs a { block }"))?;
                    let def = parse_tile(block, &t)?;
                    let slot = t.tiles.entry(id).or_default();
                    if def.collision.is_some() {
                        slot.collision = def.collision;
                    }
                    if def.anim.is_some() {
                        slot.anim = def.anim;
                    }
                }
                "terrain" => {
                    let name = e
                        .values
                        .first()
                        .and_then(|v| v.as_word().or(v.as_str()))
                        .ok_or_else(|| err(e.line, "terrain needs a name"))?
                        .to_string();
                    let block = e
                        .block
                        .as_ref()
                        .ok_or_else(|| err(e.line, "terrain needs a { block }"))?;
                    if t.terrains.iter().any(|x| x.name == name) {
                        return Err(err(e.line, format!("terrain '{name}' is declared twice")));
                    }
                    t.terrains.push(parse_terrain(name, block, e.line)?);
                }
                other => return Err(err(e.line, format!("unknown key '{other}'"))),
            }
        }
        if !texture {
            return Err(err(0, "no texture line"));
        }
        if !grid {
            return Err(err(0, "no grid line (tile width, height, columns, rows)"));
        }
        // Collide rects covering the whole tile are full tiles.
        let (tw, th) = (t.tile_width as f32, t.tile_height as f32);
        for d in t.tiles.values_mut() {
            if let Some(Collision::Rect { x, y, w, h }) = d.collision
                && x <= 0.0
                && y <= 0.0
                && x + w >= tw
                && y + h >= th
            {
                d.collision = Some(Collision::Full);
            }
        }
        t.validate()?;
        Ok(t)
    }

    /// Checks sizes and that every tile id it names is in the grid.
    pub fn validate(&self) -> Result<()> {
        if self.tile_width == 0 || self.tile_height == 0 || self.columns == 0 || self.rows == 0 {
            return Err(err(
                0,
                "the grid's tile size, columns and rows must be at least 1",
            ));
        }
        let n = self.count();
        let check = |id: u32, what: &str| {
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
                if a.durations.iter().any(|d| !(d.is_finite() && *d > 0.0)) {
                    return Err(err(
                        0,
                        format!("tile {id}: frame durations must be above 0"),
                    ));
                }
            }
        }
        for tr in &self.terrains {
            check(tr.default, &format!("terrain '{}' default", tr.name))?;
            for (&m, &tile) in &tr.rules {
                check(tile, &format!("terrain '{}' rule {m}", tr.name))?;
            }
        }
        Ok(())
    }

    /// The canonical `.ogt` text (what the cook stores); [`Tileset::parse`]
    /// reads it back to an equal tileset.
    pub fn to_text(&self) -> String {
        let mut d = Document::new("tileset", 1);
        let b = &mut d.body;
        let n = |v: f32| Value::Number(Number::from_f32(v).expect("finite"));
        let u = |v: u32| Value::Number(Number::from(v));
        b.push(Entry::new("texture").with(Value::String(self.texture.clone())));
        b.push(
            Entry::new("grid")
                .with(u(self.tile_width))
                .with(u(self.tile_height))
                .with(u(self.columns))
                .with(u(self.rows)),
        );
        if self.margin != 0 {
            b.push(Entry::new("margin").with(u(self.margin)));
        }
        if self.spacing != 0 {
            b.push(Entry::new("spacing").with(u(self.spacing)));
        }
        b.push(
            Entry::new("filter").with(Value::Word(
                match self.filter {
                    Filter::Nearest => "nearest",
                    Filter::Linear => "linear",
                }
                .into(),
            )),
        );
        let solid: Vec<u32> = self
            .tiles
            .iter()
            .filter(|(_, d)| d.collision == Some(Collision::Full))
            .map(|(&i, _)| i)
            .collect();
        if !solid.is_empty() {
            let mut e = Entry::new("solid");
            for id in solid {
                e = e.with(u(id));
            }
            b.push(e);
        }
        for (&id, def) in &self.tiles {
            let mut inner = Block::new();
            if let Some(Collision::Rect { x, y, w, h }) = def.collision {
                inner.push(
                    Entry::new("collide")
                        .with(n(x))
                        .with(n(y))
                        .with(n(w))
                        .with(n(h)),
                );
            }
            if let Some(a) = &def.anim {
                let mut f = Entry::new("frames");
                for &fr in &a.frames {
                    f = f.with(u(fr));
                }
                inner.push(f);
                let mut du = Entry::new("durations");
                for &s in &a.durations {
                    du = du.with(n(s));
                }
                inner.push(du);
            }
            if !inner.entries.is_empty() {
                b.push(Entry::new("tile").with(u(id)).with_block(inner));
            }
        }
        for tr in &self.terrains {
            let mut inner = Block::new();
            inner.push(
                Entry::new("mode").with(Value::Word(
                    match tr.mode {
                        TerrainMode::Blob47 => "blob47",
                        TerrainMode::Bitmask16 => "bitmask16",
                    }
                    .into(),
                )),
            );
            for (&m, &tile) in &tr.rules {
                inner.push(Entry::new("rule").with(u(u32::from(m))).with(u(tile)));
            }
            inner.push(Entry::new("default").with(u(tr.default)));
            inner.push(Entry::new("edge").with(Value::Word(
                if tr.edge_filled { "filled" } else { "empty" }.into(),
            )));
            b.push(
                Entry::new("terrain")
                    .with(Value::String(tr.name.clone()))
                    .with_block(inner),
            );
        }
        d.to_text().expect("tileset text is made of words")
    }
}

fn parse_tile(block: &Block, t: &Tileset) -> Result<TileDef> {
    let mut def = TileDef::default();
    let mut frames: Option<(Vec<u32>, usize)> = None;
    let mut fps: Option<(f32, usize)> = None;
    let mut durations: Option<(Vec<f32>, usize)> = None;
    let _ = t;
    for e in &block.entries {
        match e.key.as_str() {
            "collide" => {
                let (x, y, w, h) = (num(e, 0)?, num(e, 1)?, num(e, 2)?, num(e, 3)?);
                if !(w > 0.0 && h > 0.0) {
                    return Err(err(e.line, "collide needs a width and height above 0"));
                }
                def.collision = Some(Collision::Rect { x, y, w, h });
            }
            "solid" => def.collision = Some(Collision::Full),
            "frames" => {
                let mut f = Vec::new();
                for i in 0..e.values.len() {
                    f.push(uint(e, i)?);
                }
                if f.is_empty() {
                    return Err(err(e.line, "frames needs at least one tile id"));
                }
                frames = Some((f, e.line));
            }
            "fps" => {
                let v = num(e, 0)?;
                if !(v.is_finite() && v > 0.0) {
                    return Err(err(e.line, "fps must be above 0"));
                }
                fps = Some((v, e.line));
            }
            "durations" => {
                let mut d = Vec::new();
                for i in 0..e.values.len() {
                    let v = num(e, i)?;
                    if !(v.is_finite() && v > 0.0) {
                        return Err(err(e.line, "durations must be above 0"));
                    }
                    d.push(v);
                }
                durations = Some((d, e.line));
            }
            other => {
                return Err(err(
                    e.line,
                    format!("unknown key '{other}' in a tile block"),
                ));
            }
        }
    }
    if let Some((frames, line)) = frames {
        let durations = match (durations, fps) {
            (Some((d, l)), _) => {
                if d.len() != frames.len() {
                    return Err(err(
                        l,
                        format!("{} durations for {} frames", d.len(), frames.len()),
                    ));
                }
                d
            }
            (None, Some((f, _))) => vec![1.0 / f; frames.len()],
            (None, None) => return Err(err(line, "an animated tile needs fps or durations")),
        };
        def.anim = Some(TileAnim { frames, durations });
    } else if let Some((_, l)) = durations
        .as_ref()
        .map(|d| ((), d.1))
        .or(fps.map(|f| ((), f.1)))
    {
        return Err(err(l, "fps or durations without frames"));
    }
    Ok(def)
}

fn parse_terrain(name: String, block: &Block, line: usize) -> Result<Terrain> {
    let mut mode = None;
    let mut rules = BTreeMap::new();
    let mut default = None;
    let mut edge_filled = true;
    let mut pending: Vec<(u32, u32, usize)> = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "mode" => {
                mode = Some(match e.word(0).unwrap_or("") {
                    "blob47" => TerrainMode::Blob47,
                    "bitmask16" => TerrainMode::Bitmask16,
                    _ => return Err(err(e.line, "mode is blob47 or bitmask16")),
                })
            }
            "rule" => pending.push((uint(e, 0)?, uint(e, 1)?, e.line)),
            "default" => default = Some(uint(e, 0)?),
            "edge" => {
                edge_filled = match e.word(0).unwrap_or("") {
                    "filled" => true,
                    "empty" => false,
                    _ => return Err(err(e.line, "edge is filled or empty")),
                }
            }
            other => {
                return Err(err(
                    e.line,
                    format!("unknown key '{other}' in a terrain block"),
                ));
            }
        }
    }
    let mode = mode.ok_or_else(|| err(line, format!("terrain '{name}' needs a mode")))?;
    for (m, tile, l) in pending {
        let ok = match mode {
            TerrainMode::Blob47 => m <= 255 && reduce_blob(m as u8) == m as u8,
            TerrainMode::Bitmask16 => m <= 15,
        };
        if !ok {
            return Err(err(
                l,
                match mode {
                    TerrainMode::Blob47 => format!(
                        "mask {m} is not a reduced blob47 mask (a corner bit needs both of its edge bits; 0-255)"
                    ),
                    TerrainMode::Bitmask16 => format!("mask {m} is not a bitmask16 mask (0-15)"),
                },
            ));
        }
        if rules.insert(m as u8, tile).is_some() {
            return Err(err(l, format!("mask {m} has two rules")));
        }
    }
    let default =
        default.ok_or_else(|| err(line, format!("terrain '{name}' needs a default tile")))?;
    Ok(Terrain {
        name,
        mode,
        rules,
        default,
        edge_filled,
    })
}
