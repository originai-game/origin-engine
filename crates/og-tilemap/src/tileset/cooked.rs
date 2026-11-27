//! Bounds-checked native tilesets, shared by bundles and world snapshots.
use super::*;
use og_core::bytes::{ByteReader, ByteWriter};

impl Tileset {
    /// Native little-endian tileset. Call `validate` before encoding authored data.
    pub fn encode(&self, out: &mut Vec<u8>) {
        let mut w = ByteWriter::new(out);
        w.bytes(b"OGTS\x02\0\0\0");
        w.str(&self.texture);
        for n in [
            self.tile_width,
            self.tile_height,
            self.columns,
            self.rows,
            self.margin,
            self.spacing,
        ] {
            w.u32(n);
        }
        w.u8(match self.filter {
            Filter::Nearest => 0,
            Filter::Linear => 1,
        });
        w.length(self.tiles.len());
        for (&id, tile) in &self.tiles {
            w.u32(id);
            match tile.collision {
                None => w.u8(0),
                Some(Collision::Full) => w.u8(1),
                Some(Collision::Rect { x, y, w: width, h }) => {
                    w.u8(2);
                    for v in [x, y, width, h] {
                        w.f32(v);
                    }
                }
            }
            w.length(tile.polygons.len());
            for polygon in &tile.polygons {
                w.length(polygon.len());
                for &[x, y] in polygon {
                    w.f32(x);
                    w.f32(y);
                }
            }
            w.bool(tile.anim.is_some());
            if let Some(anim) = &tile.anim {
                w.length(anim.frames.len());
                for (&frame, &duration) in anim.frames.iter().zip(&anim.durations) {
                    w.u32(frame);
                    w.f32(duration);
                }
            }
        }
        w.length(self.terrains.len());
        for terrain in &self.terrains {
            w.str(&terrain.name);
            w.u8(match terrain.mode {
                TerrainMode::Blob47 => 0,
                TerrainMode::Bitmask16 => 1,
            });
            w.u32(terrain.default);
            w.bool(terrain.edge_filled);
            w.length(terrain.rules.len());
            for (&mask, &tile) in &terrain.rules {
                w.u8(mask);
                w.u32(tile);
            }
        }
    }

    /// Decodes native fields and validates every ID, shape, duration and rule.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = ByteReader::new(bytes, "a native tileset");
        if r.take(8)? != b"OGTS\x02\0\0\0" {
            return Err(r.error("unknown tileset format"));
        }
        let mut out = Self {
            texture: r.string()?,
            tile_width: r.u32()?,
            tile_height: r.u32()?,
            columns: r.u32()?,
            rows: r.u32()?,
            margin: r.u32()?,
            spacing: r.u32()?,
            filter: match r.u8()? {
                0 => Filter::Nearest,
                1 => Filter::Linear,
                _ => return Err(r.error("unknown filter")),
            },
            tiles: IdMap::new(),
            terrains: Vec::new(),
        };
        for _ in 0..r.count(10)? {
            let id = r.u32()?;
            let collision = match r.u8()? {
                0 => None,
                1 => Some(Collision::Full),
                2 => Some(Collision::Rect {
                    x: r.f32()?,
                    y: r.f32()?,
                    w: r.f32()?,
                    h: r.f32()?,
                }),
                _ => return Err(r.error("unknown collision shape")),
            };
            let mut polygons = Vec::new();
            for _ in 0..r.count(4)? {
                let mut polygon = Vec::new();
                for _ in 0..r.count(8)? {
                    polygon.push([r.f32()?, r.f32()?]);
                }
                polygons.push(polygon);
            }
            let anim = match r.u8()? {
                0 => None,
                1 => {
                    let mut frames = Vec::new();
                    let mut durations = Vec::new();
                    for _ in 0..r.count(8)? {
                        frames.push(r.u32()?);
                        durations.push(r.f32()?);
                    }
                    Some(TileAnim { frames, durations })
                }
                _ => return Err(r.error("invalid animation flag")),
            };
            if out
                .tiles
                .insert(
                    id,
                    TileDef {
                        collision,
                        polygons,
                        anim,
                    },
                )
                .is_some()
            {
                return Err(r.error("duplicate tile ID"));
            }
        }
        for _ in 0..r.count(14)? {
            let name = r.string()?;
            let mode = match r.u8()? {
                0 => TerrainMode::Blob47,
                1 => TerrainMode::Bitmask16,
                _ => return Err(r.error("unknown terrain mode")),
            };
            let default = r.u32()?;
            let edge_filled = match r.u8()? {
                0 => false,
                1 => true,
                _ => return Err(r.error("invalid terrain edge flag")),
            };
            let mut rules = BTreeMap::new();
            for _ in 0..r.count(5)? {
                if rules.insert(r.u8()?, r.u32()?).is_some() {
                    return Err(r.error("duplicate terrain mask"));
                }
            }
            out.terrains.push(Terrain {
                name,
                mode,
                rules,
                default,
                edge_filled,
            });
        }
        r.finish()?;
        out.validate()?;
        Ok(out)
    }
}
