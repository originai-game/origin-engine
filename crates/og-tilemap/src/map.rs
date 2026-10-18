//! Layered maps (`.ogt` of `type tilemap`, or imported by the cook from
//! Tiled and LDtk) and their coordinates.

use crate::tileset::{num, uint, Tileset};
use glam::Vec2;
use og_core::ogt::{Document, Entry};
use og_core::{Error, ErrorKind, Result};

/// Cell bit: the tile is mirrored left to right (Tiled's bit).
pub const FLIP_X: u32 = 1 << 31;
/// Cell bit: the tile is mirrored top to bottom.
pub const FLIP_Y: u32 = 1 << 30;
/// Cell bit: Tiled's diagonal flip, which maps refuse.
pub const FLIP_DIAGONAL: u32 = 1 << 29;
/// The tile id + 1 of a cell.
pub const TILE_MASK: u32 = FLIP_DIAGONAL - 1;

/// A map's tileset: an asset id of a tileset `.ogt`, or one made by an
/// importer (from a Tiled or LDtk file).
#[derive(Clone, Debug, PartialEq)]
pub enum TilesetRef {
    Asset(String),
    Embedded(Tileset),
}

/// One layer of cells.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub name: String,
    /// Index into [`TilemapAsset::tilesets`].
    pub tileset: u32,
    /// `width * height` cells, row 0 the top row. Tile layers: 0 empty, else
    /// (tile id + 1) | [`FLIP_X`] | [`FLIP_Y`]. Terrain layers: 0 empty, 1 filled.
    pub cells: Vec<u32>,
    /// A terrain layer: the name of its tileset's terrain, which resolves
    /// filled cells to tiles.
    pub terrain: Option<String>,
    /// How far the layer moves with the camera (1, 1 normal; 0 fixed to it).
    pub parallax: Vec2,
    /// World units added to the layer's position (y up).
    pub offset: Vec2,
    pub opacity: f32,
    pub visible: bool,
    /// Its tiles' collision shapes become colliders.
    pub collision: bool,
    pub repeat_x: bool,
    pub repeat_y: bool,
    /// The sprite layer offset it draws at (the owner's base plus this).
    pub z: i32,
}

impl Layer {
    /// An empty, visible tile layer of a `width × height` map.
    pub fn new(name: impl Into<String>, width: u32, height: u32, z: i32) -> Layer {
        Layer {
            name: name.into(),
            tileset: 0,
            cells: vec![0; (width * height) as usize],
            terrain: None,
            parallax: Vec2::ONE,
            offset: Vec2::ZERO,
            opacity: 1.0,
            visible: true,
            collision: false,
            repeat_x: false,
            repeat_y: false,
            z,
        }
    }
}

/// A named rectangle or point of the map (spawns, doors, triggers).
#[derive(Clone, Debug, PartialEq)]
pub struct MapObject {
    pub name: String,
    pub kind: String,
    /// The top-left corner in map-local world units (y up from the map's
    /// top-left corner, so inside the map y is 0 or negative).
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub properties: Vec<(String, String)>,
}

/// A map: its grid, tilesets, layers and objects.
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapAsset {
    pub width: u32,
    pub height: u32,
    pub tile_width_px: u32,
    pub tile_height_px: u32,
    /// World units per tile width.
    pub unit: f32,
    pub tilesets: Vec<TilesetRef>,
    pub layers: Vec<Layer>,
    pub objects: Vec<MapObject>,
}

fn err(line: usize, msg: impl std::fmt::Display) -> Error {
    if line == 0 {
        Error::new(ErrorKind::Invalid, format!("tilemap: {msg}"))
    } else {
        Error::new(ErrorKind::Parse, format!("tilemap line {line}: {msg}"))
    }
}

impl TilemapAsset {
    /// One tile's size in world units: `unit` wide, as tall as the pixels say.
    pub fn tile_size(&self) -> Vec2 {
        let w = self.unit;
        Vec2::new(
            w,
            w * self.tile_height_px as f32 / self.tile_width_px.max(1) as f32,
        )
    }

    /// The map's size in world units.
    pub fn world_size(&self) -> Vec2 {
        self.tile_size() * Vec2::new(self.width as f32, self.height as f32)
    }

    /// The layer named `name`, by index.
    pub fn layer(&self, name: &str) -> Option<usize> {
        self.layers.iter().position(|l| l.name == name)
    }

    /// Checks sizes, cell counts, tileset indices and flip bits.
    pub fn validate(&self) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(err(0, "the map needs at least one cell"));
        }
        if self.tile_width_px == 0 || self.tile_height_px == 0 {
            return Err(err(0, "the tile size must be at least 1 pixel"));
        }
        if !(self.unit.is_finite() && self.unit > 0.0) {
            return Err(err(0, "unit must be above 0"));
        }
        let n = (self.width * self.height) as usize;
        for l in &self.layers {
            if l.cells.len() != n {
                return Err(err(
                    0,
                    format!(
                        "layer '{}' has {} cells, the map {n}",
                        l.name,
                        l.cells.len()
                    ),
                ));
            }
            if l.tileset as usize >= self.tilesets.len() {
                return Err(err(
                    0,
                    format!(
                        "layer '{}' names tileset {}, the map has {}",
                        l.name,
                        l.tileset,
                        self.tilesets.len()
                    ),
                ));
            }
            if l.terrain.is_some() {
                if let Some(c) = l.cells.iter().find(|&&c| c > 1) {
                    return Err(err(
                        0,
                        format!("terrain layer '{}' holds {c}; its cells are 0 or 1", l.name),
                    ));
                }
            } else if l.cells.iter().any(|c| c & FLIP_DIAGONAL != 0) {
                return Err(err(
                    0,
                    format!("layer '{}' has a diagonally flipped (rotated) tile, which maps do not support", l.name),
                ));
            }
        }
        Ok(())
    }

    /// Parses a hand-written `.ogt` of `type tilemap`; errors name the line.
    pub fn parse(text: &str) -> Result<TilemapAsset> {
        let doc = Document::parse(text)?;
        if doc.ty != "tilemap" {
            return Err(err(
                0,
                format!("the file is of type '{}', not 'tilemap'", doc.ty),
            ));
        }
        if doc.version != 1 {
            return Err(err(
                0,
                format!("version {} is not supported (1 is)", doc.version),
            ));
        }
        let mut m = TilemapAsset {
            width: 0,
            height: 0,
            tile_width_px: 0,
            tile_height_px: 0,
            unit: 1.0,
            tilesets: Vec::new(),
            layers: Vec::new(),
            objects: Vec::new(),
        };
        let mut layer_entries: Vec<&Entry> = Vec::new();
        let mut object_entries: Vec<&Entry> = Vec::new();
        for e in &doc.body.entries {
            match e.key.as_str() {
                "size" => {
                    m.width = uint(e, 0)?;
                    m.height = uint(e, 1)?;
                }
                "tile" => {
                    m.tile_width_px = uint(e, 0)?;
                    m.tile_height_px = uint(e, 1)?;
                }
                "unit" => m.unit = num(e, 0)?,
                "tileset" => m.tilesets.push(TilesetRef::Asset(
                    e.string(0)
                        .map_err(|_| err(e.line, "tileset needs a quoted asset path"))?
                        .to_string(),
                )),
                "layer" => layer_entries.push(e),
                "object" => object_entries.push(e),
                other => return Err(err(e.line, format!("unknown key '{other}'"))),
            }
        }
        if m.width == 0 || m.height == 0 {
            return Err(err(0, "no size line (cells wide, cells high)"));
        }
        if m.tile_width_px == 0 || m.tile_height_px == 0 {
            return Err(err(0, "no tile line (tile width and height in pixels)"));
        }
        if m.tilesets.is_empty() {
            return Err(err(0, "no tileset line"));
        }
        for (i, e) in layer_entries.into_iter().enumerate() {
            let l = parse_layer(e, &m, i as i32)?;
            if m.layers.iter().any(|x| x.name == l.name) {
                return Err(err(e.line, format!("layer '{}' is declared twice", l.name)));
            }
            m.layers.push(l);
        }
        let ts = m.tile_size();
        for e in object_entries {
            let name = name_of(e).ok_or_else(|| err(e.line, "object needs a name"))?;
            let mut o = MapObject {
                name,
                kind: String::new(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                properties: Vec::new(),
            };
            for p in e.block.iter().flat_map(|b| &b.entries) {
                match p.key.as_str() {
                    "kind" => {
                        o.kind =
                            string_or_word(p, 0).ok_or_else(|| err(p.line, "kind needs a name"))?
                    }
                    "rect" => {
                        o.x = num(p, 0)? * ts.x;
                        o.y = -num(p, 1)? * ts.y;
                        o.width = num(p, 2)? * ts.x;
                        o.height = num(p, 3)? * ts.y;
                    }
                    "at" => {
                        o.x = num(p, 0)? * ts.x;
                        o.y = -num(p, 1)? * ts.y;
                    }
                    "prop" => {
                        let k =
                            string_or_word(p, 0).ok_or_else(|| err(p.line, "prop needs a key"))?;
                        let v =
                            p.values
                                .get(1)
                                .map(|v| match v {
                                    og_core::ogt::Value::Number(n) => n.text().to_string(),
                                    og_core::ogt::Value::String(s)
                                    | og_core::ogt::Value::Word(s) => s.clone(),
                                })
                                .ok_or_else(|| err(p.line, "prop needs a value"))?;
                        o.properties.push((k, v));
                    }
                    other => {
                        return Err(err(
                            p.line,
                            format!("unknown key '{other}' in an object block"),
                        ))
                    }
                }
            }
            m.objects.push(o);
        }
        m.validate()?;
        Ok(m)
    }
}

fn string_or_word(e: &Entry, i: usize) -> Option<String> {
    e.values
        .get(i)
        .and_then(|v| v.as_str().or(v.as_word()))
        .map(str::to_string)
}

fn name_of(e: &Entry) -> Option<String> {
    string_or_word(e, 0)
}

fn parse_layer(e: &Entry, m: &TilemapAsset, index: i32) -> Result<Layer> {
    let name = name_of(e).ok_or_else(|| err(e.line, "layer needs a name"))?;
    let mut l = Layer::new(name, m.width, m.height, index);
    let block = e
        .block
        .as_ref()
        .ok_or_else(|| err(e.line, "layer needs a { block }"))?;
    let mut legend: Vec<(char, i64)> = Vec::new();
    let mut rows: Vec<&Entry> = Vec::new();
    for p in &block.entries {
        match p.key.as_str() {
            "tileset" => l.tileset = uint(p, 0)?,
            "z" => {
                l.z = p
                    .number(0)
                    .ok()
                    .and_then(|n| n.as_i64().ok())
                    .and_then(|v| i32::try_from(v).ok())
                    .ok_or_else(|| err(p.line, "z needs a whole number"))?
            }
            "parallax" => l.parallax = Vec2::new(num(p, 0)?, num(p, 1)?),
            "offset" => l.offset = Vec2::new(num(p, 0)?, num(p, 1)?),
            "opacity" => l.opacity = num(p, 0)?,
            "visible" => {
                l.visible = p
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| err(p.line, "visible is true or false"))?
            }
            "collision" => {
                l.collision = p
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| err(p.line, "collision is true or false"))?
            }
            "repeat" => {
                for v in &p.values {
                    match v.as_word() {
                        Some("x") => l.repeat_x = true,
                        Some("y") => l.repeat_y = true,
                        _ => return Err(err(p.line, "repeat takes x, y or both")),
                    }
                }
            }
            "terrain" => {
                l.terrain =
                    Some(string_or_word(p, 0).ok_or_else(|| err(p.line, "terrain needs a name"))?)
            }
            "legend" => {
                let s = p
                    .string(0)
                    .map_err(|_| err(p.line, "legend needs a quoted character"))?;
                let mut chars = s.chars();
                let (Some(c), None) = (chars.next(), chars.next()) else {
                    return Err(err(p.line, "legend's character is one character"));
                };
                let v = p
                    .number(1)
                    .ok()
                    .and_then(|n| n.as_i64().ok())
                    .ok_or_else(|| err(p.line, "legend needs a tile id (or -1 for empty)"))?;
                legend.push((c, v));
            }
            "row" => rows.push(p),
            other => {
                return Err(err(
                    p.line,
                    format!("unknown key '{other}' in a layer block"),
                ))
            }
        }
    }
    let terrain = l.terrain.is_some();
    if !rows.is_empty() && rows.len() != m.height as usize {
        return Err(err(
            e.line,
            format!(
                "layer '{}' has {} rows, the map {}",
                l.name,
                rows.len(),
                m.height
            ),
        ));
    }
    let cell = |v: i64, line: usize| -> Result<u32> {
        if terrain {
            match v {
                0 | -1 => Ok(0),
                1 => Ok(1),
                _ => Err(err(
                    line,
                    format!("a terrain cell is 1 (filled) or 0 (empty), not {v}"),
                )),
            }
        } else if v < 0 {
            Ok(0)
        } else if v >= TILE_MASK as i64 {
            Err(err(line, format!("tile id {v} is too large")))
        } else {
            Ok(v as u32 + 1)
        }
    };
    for (y, r) in rows.iter().enumerate() {
        let w = m.width as usize;
        let values: Vec<i64> = if let Some(s) = r.values.first().and_then(|v| v.as_str()) {
            let mut out = Vec::new();
            for c in s.chars() {
                let v = match legend.iter().find(|(lc, _)| *lc == c) {
                    Some(&(_, v)) => v,
                    None if c == '.' => -1,
                    None => return Err(err(r.line, format!("'{c}' is not in the layer's legend"))),
                };
                out.push(v);
            }
            out
        } else {
            let mut out = Vec::new();
            for i in 0..r.values.len() {
                out.push(
                    r.number(i)
                        .ok()
                        .and_then(|n| n.as_i64().ok())
                        .ok_or_else(|| err(r.line, "a row is a quoted string or tile ids"))?,
                );
            }
            out
        };
        if values.len() != w {
            return Err(err(
                r.line,
                format!("the row is {} cells wide, the map {w}", values.len()),
            ));
        }
        for (x, v) in values.into_iter().enumerate() {
            l.cells[y * w + x] = cell(v, r.line)?;
        }
    }
    Ok(l)
}

/// The cell at a map-local world point (y up from the top-left corner).
pub fn world_to_cell(asset: &TilemapAsset, local: Vec2) -> Option<(u32, u32)> {
    let ts = asset.tile_size();
    let x = (local.x / ts.x).floor();
    let y = (-local.y / ts.y).floor();
    (x >= 0.0 && y >= 0.0 && x < asset.width as f32 && y < asset.height as f32)
        .then_some((x as u32, y as u32))
}

/// The centre of cell (x, y) in map-local world units.
pub fn cell_center(asset: &TilemapAsset, x: u32, y: u32) -> Vec2 {
    let ts = asset.tile_size();
    Vec2::new((x as f32 + 0.5) * ts.x, -(y as f32 + 0.5) * ts.y)
}
