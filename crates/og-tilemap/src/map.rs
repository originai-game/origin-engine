//! Layered maps (`.ogt` of `type tilemap`, or imported by the cook from
//! Tiled and LDtk) and their coordinates.

use crate::tileset::{Tileset, num, uint};
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
    /// The collision layers (og.Collider.layers bits) of this layer's
    /// colliders; 0 takes the owner's (og.Tilemap.colliderLayers), so a
    /// layer of grates can sit on a one-way layer while the walls do not.
    pub collider_layers: u32,
    /// The layers (bits) this layer's colliders collide with
    /// (og.Collider.mask); 0 is every layer, og.Collider's default.
    pub collider_mask: u32,
    pub repeat_x: bool,
    pub repeat_y: bool,
    /// The sprite layer offset it draws at (the owner's base plus this).
    pub z: i32,
    /// Taken by 2D lighting when its sprite layer is in the lit range, as
    /// the sprites there are (true by default); `lit false` in a layer
    /// block, or a Tiled layer's bool property `lit` false, keeps it as
    /// authored (a sky, a far backdrop).
    pub lit: bool,
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
            collider_layers: 0,
            collider_mask: 0,
            repeat_x: false,
            repeat_y: false,
            z,
            lit: true,
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

/// How a map's cells are laid out. Cells always count (x, y) from the
/// top-left of the grid; the orientation places them in the world. `w` and
/// `h` below are a tile's size in world units ([`TilemapAsset::tile_size`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Rectangles in rows and columns.
    #[default]
    Orthogonal,
    /// Diamonds on a turned grid (Tiled's `isometric`): x runs down to the
    /// right and y down to the left, cell (0, 0) at the top.
    Isometric,
    /// Diamonds in rows (Tiled's `staggered`, odd rows): rows h/2 apart,
    /// every odd row shifted right w/2.
    Staggered,
    /// Hexagons in their tiles' boxes, corners at a quarter of the box
    /// (Tiled's `hexagonal`, odd stagger, sides half a tile). Pointy-top
    /// (`flat` false): rows 3h/4 apart, every odd row shifted right w/2.
    /// Flat-top: columns 3w/4 apart, every odd column shifted down h/2.
    Hexagonal { flat: bool },
}

const ORIENTATIONS: [Orientation; 5] = [
    Orientation::Orthogonal,
    Orientation::Isometric,
    Orientation::Hexagonal { flat: false },
    Orientation::Hexagonal { flat: true },
    Orientation::Staggered,
];

impl Orientation {
    /// Its name on the wire (TileOrientation in the API).
    pub fn name(self) -> &'static str {
        match self {
            Orientation::Orthogonal => "orthogonal",
            Orientation::Isometric => "isometric",
            Orientation::Staggered => "staggered",
            Orientation::Hexagonal { flat: false } => "hex",
            Orientation::Hexagonal { flat: true } => "hexFlat",
        }
    }

    /// The orientation [`Orientation::name`] names.
    pub fn named(name: &str) -> Option<Orientation> {
        ORIENTATIONS.into_iter().find(|o| o.name() == name)
    }

    /// Its code in the cooked blob and saves (0 orthogonal).
    pub fn code(self) -> u32 {
        ORIENTATIONS.iter().position(|o| *o == self).unwrap_or(0) as u32
    }

    /// The orientation of [`Orientation::code`].
    pub fn from_code(code: u32) -> Option<Orientation> {
        ORIENTATIONS.get(code as usize).copied()
    }

    /// Whether auto-tiling's eight square neighbours are the cells around
    /// a cell (orthogonal and isometric grids), so terrain layers work.
    pub fn square_grid(self) -> bool {
        matches!(self, Orientation::Orthogonal | Orientation::Isometric)
    }

    /// The axis whose odd rows (y) or columns (x) are shifted, if any: a
    /// resize keeps that shift even.
    pub fn stagger(self) -> Option<char> {
        match self {
            Orientation::Staggered | Orientation::Hexagonal { flat: false } => Some('y'),
            Orientation::Hexagonal { flat: true } => Some('x'),
            _ => None,
        }
    }
}

/// A map: its grid, tilesets, layers and objects.
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapAsset {
    /// How the cells are laid out (orthogonal unless the map says).
    pub orientation: Orientation,
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

    /// The map's size in world units: the box around its cells.
    pub fn world_size(&self) -> Vec2 {
        let t = self.tile_size();
        let (w, h) = (self.width as f32, self.height as f32);
        match self.orientation {
            Orientation::Orthogonal => t * Vec2::new(w, h),
            Orientation::Isometric => t * 0.5 * (w + h),
            Orientation::Staggered => Vec2::new(
                t.x * (w + if self.height > 1 { 0.5 } else { 0.0 }),
                t.y * (0.5 * (h - 1.0) + 1.0),
            ),
            Orientation::Hexagonal { flat: false } => Vec2::new(
                t.x * (w + if self.height > 1 { 0.5 } else { 0.0 }),
                t.y * (0.75 * (h - 1.0) + 1.0),
            ),
            Orientation::Hexagonal { flat: true } => Vec2::new(
                t.x * (0.75 * (w - 1.0) + 1.0),
                t.y * (h + if self.width > 1 { 0.5 } else { 0.0 }),
            ),
        }
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
            if let Some(t) = &l.terrain
                && !self.orientation.square_grid()
            {
                return Err(err(
                    0,
                    format!(
                        "layer '{}' is a terrain layer ({t}) on a {} map: auto-tiling reads eight square neighbours, so terrain layers need an orthogonal or isometric map",
                        l.name,
                        self.orientation.name()
                    ),
                ));
            }
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
                    format!(
                        "layer '{}' has a diagonally flipped (rotated) tile, which maps do not support",
                        l.name
                    ),
                ));
            }
        }
        Ok(())
    }

    /// This map at `width` × `height`, its cells moved so that the old
    /// cell (0, 0) lands on (dx, dy) (negative crops the left or top), the
    /// cells past the new edges dropped and the new ones empty, its objects
    /// moved with the cells. On a staggered or hex map the shift along the
    /// staggered axis must be even, or every row (column) would change its
    /// side.
    pub fn resized(&self, width: u32, height: u32, dx: i32, dy: i32) -> Result<TilemapAsset> {
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 1 << 24 {
            return Err(err(
                0,
                format!("a map of {width} x {height} cells: 1 to 16,777,216 cells"),
            ));
        }
        let shift = match self.orientation.stagger() {
            Some('x') => Some(("x", dx)),
            Some(_) => Some(("y", dy)),
            None => None,
        };
        if let Some((axis, d)) = shift
            && d.rem_euclid(2) == 1
        {
            return Err(err(
                0,
                format!(
                    "a {} map moves its cells by an even {axis} (keeping its odd rows or columns shifted), not {d}",
                    self.orientation.name()
                ),
            ));
        }
        let mut m = self.clone();
        m.width = width;
        m.height = height;
        let n = (width as usize) * (height as usize);
        for (l, old) in m.layers.iter_mut().zip(&self.layers) {
            l.cells = vec![0; n];
            for y in 0..self.height {
                let ny = y as i64 + dy as i64;
                if ny < 0 || ny >= height as i64 {
                    continue;
                }
                for x in 0..self.width {
                    let nx = x as i64 + dx as i64;
                    if nx >= 0 && nx < width as i64 {
                        l.cells[ny as usize * width as usize + nx as usize] =
                            old.cells[(y * self.width + x) as usize];
                    }
                }
            }
        }
        let moved = center(&m, dx as i64, dy as i64) - center(self, 0, 0);
        for o in &mut m.objects {
            o.x += moved.x;
            o.y += moved.y;
        }
        m.validate()?;
        Ok(m)
    }

    /// The whole map as bytes (a save's copy of a map made or resized at
    /// run time), little-endian; [`TilemapAsset::decode`] reads it back.
    pub fn encode(&self, out: &mut Vec<u8>) {
        fn u(out: &mut Vec<u8>, v: u32) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        fn f(out: &mut Vec<u8>, v: f32) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        fn s(out: &mut Vec<u8>, v: &str) {
            u(out, v.len() as u32);
            out.extend_from_slice(v.as_bytes());
        }
        for v in [
            self.orientation.code(),
            self.width,
            self.height,
            self.tile_width_px,
            self.tile_height_px,
        ] {
            u(out, v);
        }
        f(out, self.unit);
        u(out, self.tilesets.len() as u32);
        for t in &self.tilesets {
            match t {
                TilesetRef::Asset(id) => {
                    u(out, 0);
                    s(out, id);
                }
                TilesetRef::Embedded(t) => {
                    u(out, 1);
                    s(out, &t.to_text());
                }
            }
        }
        u(out, self.layers.len() as u32);
        for l in &self.layers {
            s(out, &l.name);
            u(out, l.tileset);
            s(out, l.terrain.as_deref().unwrap_or(""));
            for v in [
                l.parallax.x,
                l.parallax.y,
                l.offset.x,
                l.offset.y,
                l.opacity,
            ] {
                f(out, v);
            }
            let flags = [
                l.visible,
                l.collision,
                l.repeat_x,
                l.repeat_y,
                l.lit,
                l.terrain.is_some(),
            ]
            .iter()
            .enumerate()
            .fold(0u32, |a, (i, &b)| a | (b as u32) << i);
            for v in [flags, l.collider_layers, l.collider_mask, l.z as u32] {
                u(out, v);
            }
            for &c in &l.cells {
                u(out, c);
            }
        }
        u(out, self.objects.len() as u32);
        for o in &self.objects {
            s(out, &o.name);
            s(out, &o.kind);
            for v in [o.x, o.y, o.width, o.height] {
                f(out, v);
            }
            u(out, o.properties.len() as u32);
            for (k, v) in &o.properties {
                s(out, k);
                s(out, v);
            }
        }
    }

    /// The map [`TilemapAsset::encode`] wrote, checked.
    pub fn decode(bytes: &[u8]) -> Result<TilemapAsset> {
        struct R<'a>(&'a [u8]);
        impl<'a> R<'a> {
            fn take(&mut self, n: usize) -> Result<&'a [u8]> {
                if self.0.len() < n {
                    return Err(Error::new(ErrorKind::Parse, "a saved tilemap is cut short"));
                }
                let (a, b) = self.0.split_at(n);
                self.0 = b;
                Ok(a)
            }
            fn u(&mut self) -> Result<u32> {
                let b = self.take(4)?;
                Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            }
            fn f(&mut self) -> Result<f32> {
                self.u().map(f32::from_bits)
            }
            fn s(&mut self) -> Result<String> {
                let n = self.u()? as usize;
                String::from_utf8(self.take(n)?.to_vec()).map_err(|_| {
                    Error::new(ErrorKind::Parse, "a saved tilemap's text is not UTF-8")
                })
            }
        }
        let mut r = R(bytes);
        let orientation = Orientation::from_code(r.u()?).ok_or_else(|| {
            Error::new(ErrorKind::Parse, "a saved tilemap's orientation is unknown")
        })?;
        let mut m = TilemapAsset {
            orientation,
            width: r.u()?,
            height: r.u()?,
            tile_width_px: r.u()?,
            tile_height_px: r.u()?,
            unit: r.f()?,
            tilesets: Vec::new(),
            layers: Vec::new(),
            objects: Vec::new(),
        };
        let n = (m.width as usize).saturating_mul(m.height as usize);
        if n > 1 << 24 {
            return Err(err(0, "a saved tilemap has more than 16,777,216 cells"));
        }
        for _ in 0..r.u()? {
            let embedded = r.u()? == 1;
            let text = r.s()?;
            m.tilesets.push(if embedded {
                TilesetRef::Embedded(Tileset::parse(&text)?)
            } else {
                TilesetRef::Asset(text)
            });
        }
        for _ in 0..r.u()? {
            let mut l = Layer::new(r.s()?, 0, 0, 0);
            l.tileset = r.u()?;
            let terrain = r.s()?;
            l.parallax = Vec2::new(r.f()?, r.f()?);
            l.offset = Vec2::new(r.f()?, r.f()?);
            l.opacity = r.f()?;
            let flags = r.u()?;
            let bit = |i: u32| flags >> i & 1 == 1;
            (l.visible, l.collision, l.repeat_x, l.repeat_y, l.lit) =
                (bit(0), bit(1), bit(2), bit(3), bit(4));
            l.terrain = bit(5).then_some(terrain);
            l.collider_layers = r.u()?;
            l.collider_mask = r.u()?;
            l.z = r.u()? as i32;
            let cells = r.take(n * 4)?;
            l.cells = cells
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            m.layers.push(l);
        }
        for _ in 0..r.u()? {
            let (name, kind) = (r.s()?, r.s()?);
            let (x, y, width, height) = (r.f()?, r.f()?, r.f()?, r.f()?);
            let mut properties = Vec::new();
            for _ in 0..r.u()? {
                properties.push((r.s()?, r.s()?));
            }
            m.objects.push(MapObject {
                name,
                kind,
                x,
                y,
                width,
                height,
                properties,
            });
        }
        if !r.0.is_empty() {
            return Err(Error::new(
                ErrorKind::Parse,
                "a saved tilemap has bytes past its end",
            ));
        }
        m.validate()?;
        Ok(m)
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
            orientation: Orientation::Orthogonal,
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
                "orientation" => {
                    let words: Vec<&str> = e
                        .values
                        .iter()
                        .map(|v| v.as_word().or(v.as_str()).unwrap_or(""))
                        .collect();
                    m.orientation = match words.as_slice() {
                        ["orthogonal"] => Orientation::Orthogonal,
                        ["isometric"] => Orientation::Isometric,
                        ["staggered"] => Orientation::Staggered,
                        ["hex"] | ["hex", "pointy"] => Orientation::Hexagonal { flat: false },
                        ["hex", "flat"] => Orientation::Hexagonal { flat: true },
                        _ => {
                            return Err(err(
                                e.line,
                                "orientation is orthogonal, isometric, staggered, hex (pointy-top) or hex flat",
                            ));
                        }
                    }
                }
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
                        ));
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
            "colliderLayers" => l.collider_layers = uint(p, 0)?,
            "colliderMask" => l.collider_mask = uint(p, 0)?,
            "lit" => {
                l.lit = p
                    .values
                    .first()
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| err(p.line, "lit is true or false"))?
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
                ));
            }
        }
    }
    if (l.collider_layers != 0 || l.collider_mask != 0) && !l.collision {
        return Err(err(
            e.line,
            format!(
                "layer '{}' names colliderLayers or colliderMask without `collision true`",
                l.name
            ),
        ));
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

/// The cell at a map-local world point (y up from the top-left corner of
/// the box around the cells), exact on every diamond and hexagon edge; a
/// point on an edge two cells share goes to the one of the lower row, then
/// column.
pub fn world_to_cell(asset: &TilemapAsset, local: Vec2) -> Option<(u32, u32)> {
    let ts = asset.tile_size();
    let inside = |x: f32, y: f32| {
        (x >= 0.0 && y >= 0.0 && x < asset.width as f32 && y < asset.height as f32)
            .then_some((x as u32, y as u32))
    };
    let (guess_x, guess_y) = match asset.orientation {
        Orientation::Orthogonal => {
            return inside((local.x / ts.x).floor(), (-local.y / ts.y).floor());
        }
        Orientation::Isometric => {
            let (u, v) = (
                (local.x - asset.height as f32 * ts.x * 0.5) / ts.x,
                -local.y / ts.y,
            );
            return inside((u + v).floor(), (v - u).floor());
        }
        Orientation::Staggered => ((local.x / ts.x).floor(), (-local.y / (0.5 * ts.y)).floor()),
        Orientation::Hexagonal { flat: false } => {
            ((local.x / ts.x).floor(), (-local.y / (0.75 * ts.y)).floor())
        }
        Orientation::Hexagonal { flat: true } => {
            ((local.x / (0.75 * ts.x)).floor(), (-local.y / ts.y).floor())
        }
    };
    // The cell whose shape's gauge (1 on its outline) is least, among the
    // guess and its neighbours: inside when it is at most 1.
    let (gx, gy) = (guess_x as i64, guess_y as i64);
    let mut best: Option<(f32, i64, i64)> = None;
    for y in gy - 1..=gy + 1 {
        for x in gx - 1..=gx + 1 {
            let d = local - center(asset, x, y);
            let g = gauge(asset.orientation, d.x.abs(), d.y.abs(), ts);
            if best.is_none_or(|(b, ..)| g < b) {
                best = Some((g, x, y));
            }
        }
    }
    let (g, x, y) = best?;
    (g <= 1.0).then(|| inside(x as f32, y as f32)).flatten()
}

/// A diamond's or hexagon's gauge at offset (dx, dy) ≥ 0 from its centre:
/// 1 on its outline, less inside.
fn gauge(o: Orientation, dx: f32, dy: f32, t: Vec2) -> f32 {
    let (hw, hh) = (t.x * 0.5, t.y * 0.5);
    match o {
        Orientation::Hexagonal { flat: false } => (dx / hw).max((dy + dx * hh / t.x) / hh),
        Orientation::Hexagonal { flat: true } => (dy / hh).max((dx + dy * hw / t.y) / hw),
        _ => dx / hw + dy / hh,
    }
}

/// The centre of cell (x, y), which may lie off the map.
fn center(asset: &TilemapAsset, x: i64, y: i64) -> Vec2 {
    let ts = asset.tile_size();
    let (fx, fy) = (x as f32, y as f32);
    let odd = |n: i64| if n.rem_euclid(2) == 1 { 0.5 } else { 0.0 };
    match asset.orientation {
        Orientation::Orthogonal => Vec2::new((fx + 0.5) * ts.x, -(fy + 0.5) * ts.y),
        Orientation::Isometric => Vec2::new(
            (fx - fy + asset.height as f32) * ts.x * 0.5,
            -(fx + fy + 1.0) * ts.y * 0.5,
        ),
        Orientation::Staggered => Vec2::new((fx + 0.5 + odd(y)) * ts.x, -(1.0 + fy) * ts.y * 0.5),
        Orientation::Hexagonal { flat: false } => {
            Vec2::new((fx + 0.5 + odd(y)) * ts.x, -(0.5 + fy * 0.75) * ts.y)
        }
        Orientation::Hexagonal { flat: true } => {
            Vec2::new((0.5 + fx * 0.75) * ts.x, -(fy + 0.5 + odd(x)) * ts.y)
        }
    }
}

/// The centre of cell (x, y) in map-local world units.
pub fn cell_center(asset: &TilemapAsset, x: u32, y: u32) -> Vec2 {
    center(asset, x as i64, y as i64)
}

/// The cells next to (x, y) on the map, clockwise from the top (or the
/// top right): the four sides of a square or a turned square
/// (orthogonal, isometric), the four sides of a staggered diamond, the six
/// of a hexagon.
pub fn neighbours(asset: &TilemapAsset, x: u32, y: u32) -> Vec<(u32, u32)> {
    let (x, y) = (x as i64, y as i64);
    let ox = (y.rem_euclid(2) == 1) as i64;
    let oy = (x.rem_euclid(2) == 1) as i64;
    let steps: Vec<(i64, i64)> = match asset.orientation {
        Orientation::Orthogonal | Orientation::Isometric => {
            vec![(0, -1), (1, 0), (0, 1), (-1, 0)]
        }
        Orientation::Staggered => vec![(ox, -1), (ox, 1), (ox - 1, 1), (ox - 1, -1)],
        Orientation::Hexagonal { flat: false } => vec![
            (ox, -1),
            (1, 0),
            (ox, 1),
            (ox - 1, 1),
            (-1, 0),
            (ox - 1, -1),
        ],
        Orientation::Hexagonal { flat: true } => vec![
            (0, -1),
            (1, oy - 1),
            (1, oy),
            (0, 1),
            (-1, oy),
            (-1, oy - 1),
        ],
    };
    steps
        .into_iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .filter(|&(nx, ny)| {
            nx >= 0 && ny >= 0 && nx < asset.width as i64 && ny < asset.height as i64
        })
        .map(|(nx, ny)| (nx as u32, ny as u32))
        .collect()
}
