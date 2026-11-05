//! Terrain tiles and environment map loaders.

use super::*;

// ── phase 2: terrain ─────────────────────────────────────────────────────

/// One layer of a cooked terrain: its textures (asset ids, loaded by the
/// host) and the rest of og-render's layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTerrainLayer {
    pub albedo: Option<String>,
    pub normal: Option<String>,
    /// The layer without its textures (`albedo` and `normal` are `None`).
    pub layer: og_render::TerrainLayer,
}

/// A cooked terrain (`og.Terrain`): og-render's tile layout, the range of
/// its heights, its layers and settings, and its tiles still compressed,
/// each decoded on its own ([`CookedTerrain::tile`], or inflated elsewhere
/// and finished by [`CookedTerrain::tile_from`]): a streamer decodes the
/// tiles a camera needs, physics the whole field ([`CookedTerrain::field`]).
/// Cloning shares the compressed tiles.
#[derive(Clone, Debug)]
pub struct CookedTerrain {
    /// The asset id.
    pub id: String,
    pub layout: og_render::TerrainLayout,
    /// The lowest and highest height (metres above the origin).
    pub range: (f32, f32),
    pub layers: Vec<CookedTerrainLayer>,
    /// The texture arrays the layers' albedos and normal maps come from
    /// (`layers`, `layerNormals`: asset ids, loaded by the host); each
    /// layer's `albedo_layer` and `normal_layer` name its layers of them.
    pub albedo_array: Option<String>,
    pub normal_array: Option<String>,
    pub height_blend: f32,
    pub lod: og_render::TerrainLod,
    pub cast_shadows: bool,
    /// Whether the game gets a height-field collider for it.
    pub collision: bool,
    /// A generated terrain's sea level (`generate { sea <m> }`), metres
    /// above its origin.
    pub sea_level: Option<f32>,
    tiles: std::sync::Arc<TerrainTiles>,
}

#[derive(Debug)]
struct TerrainTiles {
    rows: Vec<format::TerrainTileRow>,
    heights: Vec<u8>,
    control: Vec<u8>,
}

impl CookedTerrain {
    fn bad(&self, reason: impl Into<String>) -> LoadError {
        LoadError::Invalid {
            path: self.id.clone(),
            reason: reason.into(),
        }
    }

    /// The compressed bytes of a tile: its zlib'd 16-bit heights (then its
    /// horizon's words) and, with a control map, its zlib'd texels (for an
    /// inflater elsewhere: the web host's DecompressionStream).
    pub fn tile_zlib(&self, index: u32) -> Option<(&[u8], Option<&[u8]>)> {
        let t = self.tiles.rows.get(index as usize)?;
        let h = self
            .tiles
            .heights
            .get(t.offset as usize..(t.offset + t.bytes) as usize)?;
        let c =
            if self.layout.control > 0 {
                Some(self.tiles.control.get(
                    t.control_offset as usize..(t.control_offset + t.control_bytes) as usize,
                )?)
            } else {
                None
            };
        Some((h, c))
    }

    /// The bytes of a tile's inflated heights stream: its 16-bit heights,
    /// then its horizon's words.
    fn stream_bytes(&self) -> usize {
        let side = self.layout.heights_side() as usize;
        side * side * 2 + self.layout.horizon_words() as usize * 4
    }

    /// A tile's heights in metres from its inflated 16-bit values (with
    /// holes, each one's lowest bit its cell's hole flag:
    /// `og_render::hole_bit`).
    pub fn heights_from(&self, index: u32, raw: &[u8]) -> Result<Vec<f32>, LoadError> {
        let side = self.layout.heights_side() as usize;
        if raw.len() != self.stream_bytes() {
            return Err(self.bad(format!("tile {index} holds the wrong number of heights")));
        }
        let raw = &raw[..side * side * 2];
        let (lo, hi) = self.range;
        let span = hi - lo;
        let holes = self.layout.holes;
        Ok(raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|v| {
                let q = u16::from_le_bytes(*v);
                let h = lo + q as f32 / 65535.0 * span;
                if holes {
                    og_render::with_hole_bit(h, q & 1 != 0)
                } else {
                    h
                }
            })
            .collect())
    }

    /// A tile ready to draw from its inflated bytes (normals and mips
    /// built here).
    pub fn tile_from(
        &self,
        index: u32,
        heights: &[u8],
        control: Option<&[u8]>,
    ) -> Result<og_render::TerrainTile, LoadError> {
        let h = self.heights_from(index, heights)?;
        let c = match (self.layout.control, control) {
            (0, _) => None,
            (_, Some(c)) => {
                let side = self.layout.control_side() as usize;
                if c.len() != side * side * 4 * self.layout.pages.max(1) as usize {
                    return Err(
                        self.bad(format!("tile {index}'s control texels are the wrong size"))
                    );
                }
                Some(c.to_vec())
            }
            (_, None) => return Err(self.bad(format!("tile {index} has no control texels"))),
        };
        let side = self.layout.heights_side() as usize;
        let horizon = heights[side * side * 2..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|w| u32::from_le_bytes(*w))
            .collect();
        Ok(og_render::TerrainTile::build(
            &self.layout,
            index,
            h,
            horizon,
            c,
        ))
    }

    fn inflate(&self, index: u32) -> Result<(Vec<u8>, Option<Vec<u8>>), LoadError> {
        let (h, c) = self
            .tile_zlib(index)
            .ok_or_else(|| self.bad(format!("tile {index} is out of range")))?;
        let h = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(h, self.stream_bytes())
            .map_err(|_| self.bad(format!("tile {index} does not inflate")))?;
        let c = match c {
            None => None,
            Some(c) => {
                let side = self.layout.control_side() as usize;
                let bytes = side * side * 4 * self.layout.pages.max(1) as usize;
                Some(
                    miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(c, bytes).map_err(
                        |_| self.bad(format!("tile {index}'s control does not inflate")),
                    )?,
                )
            }
        };
        Ok((h, c))
    }

    /// Inflates and decodes a tile.
    pub fn tile(&self, index: u32) -> Result<og_render::TerrainTile, LoadError> {
        let (h, c) = self.inflate(index)?;
        self.tile_from(index, &h, c.as_deref())
    }

    /// A tile's `(tile + 3)²` heights in metres (no normals, no control).
    pub fn tile_heights(&self, index: u32) -> Result<Vec<f32>, LoadError> {
        let (h, _) = self.inflate(index)?;
        self.heights_from(index, &h)
    }

    /// The whole field from its level-0 tiles: every height, and the
    /// control map at the tiles' resolution.
    pub fn field(&self) -> Result<og_render::TerrainData, LoadError> {
        let l = &self.layout;
        let (cols, rows) = (l.columns as usize, l.rows as usize);
        let t = l.tile as usize;
        let side = l.heights_side() as usize;
        let (tx, tz) = l.tiles(0);
        let mut heights = vec![0.0f32; cols * rows];
        let mut holes = vec![0u8; (cols - 1) * (rows - 1)];
        let c = l.control as usize;
        let cside = l.control_side() as usize;
        let (cw, ch) = (c * tx as usize, c * tz as usize);
        let pages = l.pages.max(1) as usize;
        let mut rgba = vec![0u8; cw * ch * 4 * pages];
        for z in 0..tz as usize {
            for x in 0..tx as usize {
                let index = l.index(0, x as u32, z as u32);
                let (h, cz) = self.inflate(index)?;
                let h = self.heights_from(index, &h)?;
                for j in 0..=t {
                    let r = z * t + j;
                    if r >= rows {
                        break;
                    }
                    for i in 0..=t {
                        let col = x * t + i;
                        if col >= cols {
                            break;
                        }
                        let v = h[(j + 1) * side + i + 1];
                        heights[r * cols + col] = v;
                        if l.holes && r + 1 < rows && col + 1 < cols {
                            holes[r * (cols - 1) + col] = u8::from(og_render::hole_bit(v));
                        }
                    }
                }
                if let Some(cz) = cz {
                    for p in 0..pages {
                        for j in 0..c {
                            for i in 0..c {
                                let src = p * cside * cside * 4 + ((j + 1) * cside + i + 1) * 4;
                                let dst = p * cw * ch * 4 + ((z * c + j) * cw + x * c + i) * 4;
                                rgba[dst..dst + 4].copy_from_slice(&cz[src..src + 4]);
                            }
                        }
                    }
                }
            }
        }
        // The tiles' control texels cover whole tiles, which may reach past
        // the terrain: the map is their texels over its own extent.
        let control = (c > 0).then(|| {
            let quads = ((l.columns - 1) as f32, (l.rows - 1) as f32);
            let per = c as f32 / t as f32;
            let w = ((quads.0 * per).round() as usize).max(1);
            let hgt = ((quads.1 * per).round() as usize).max(1);
            let texel = |p: usize, x: i64, y: i64, k: usize| {
                let x = x.clamp(0, cw as i64 - 1) as usize;
                let y = y.clamp(0, ch as i64 - 1) as usize;
                rgba[p * cw * ch * 4 + (y * cw + x) * 4 + k] as f32
            };
            let mut out = vec![0u8; w * hgt * 4 * pages];
            for p in 0..pages {
                for j in 0..hgt {
                    for i in 0..w {
                        let gx = (i as f32 + 0.5) / w as f32 * quads.0 * per - 0.5;
                        let gy = (j as f32 + 0.5) / hgt as f32 * quads.1 * per - 0.5;
                        let (x0, y0) = (gx.floor(), gy.floor());
                        let (fx, fy) = (gx - x0, gy - y0);
                        let (x0, y0) = (x0 as i64, y0 as i64);
                        for k in 0..4 {
                            let t = |x, y| texel(p, x, y, k);
                            let a = t(x0, y0) + (t(x0 + 1, y0) - t(x0, y0)) * fx;
                            let b = t(x0, y0 + 1) + (t(x0 + 1, y0 + 1) - t(x0, y0 + 1)) * fx;
                            out[p * w * hgt * 4 + (j * w + i) * 4 + k] =
                                (a + (b - a) * fy).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
            og_render::TerrainControl {
                width: w as u32,
                height: hgt as u32,
                rgba: out.into(),
            }
        });
        let data = og_render::TerrainData {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            heights: heights.into(),
            control,
            holes: l.holes.then(|| holes.into()),
        };
        data.check().map_err(|e| self.bad(e))?;
        Ok(data)
    }
}

impl Bundle {
    /// Reads a terrain blob: its layout, layers and settings, and its tiles
    /// still compressed (each checked to lie inside the blob).
    pub fn terrain(&self, id: &str, bytes: &[u8]) -> Result<CookedTerrain, LoadError> {
        let blob = self.blob::<format::Terrain>(id, bytes)?;
        let m = one::<format::TerrainMeta>(&blob)?;
        let tiles = table::<format::TerrainTileRow>(&blob, "tiles")?;
        let layers = table::<format::TerrainLayerRow>(&blob, "layers")?;
        let strings = blob.raw("strings")?;
        let heights = blob.raw("heights")?.to_vec();
        let control = blob.raw("control")?.to_vec();
        let control_size = (m.control_width > 0).then_some((m.control_width, m.control_height));
        let mut layout = og_render::TerrainLayout::new(
            glam::Vec2::new(m.size_x, m.size_z),
            m.columns,
            m.rows,
            m.tile,
            control_size,
        )
        .map_err(|e| invalid(&blob, e))?;
        layout.holes = m.holes > 0;
        layout.pages = m.control_pages.max(1);
        if layout.control != m.control || layout.levels != m.levels {
            return Err(invalid(&blob, "the tile layout does not match its cook"));
        }
        if tiles.len() != layout.tile_count() as usize {
            return Err(invalid(&blob, "the tiles do not cover the pyramid"));
        }
        for (i, t) in tiles.iter().enumerate() {
            let inside = (t.offset as usize + t.bytes as usize) <= heights.len()
                && (t.control_offset as usize + t.control_bytes as usize) <= control.len();
            if t.index != i as u32 || !inside {
                return Err(invalid(&blob, format!("tile {i} is out of place or range")));
            }
        }
        let text = |at: u32, len: u32| -> Result<Option<String>, LoadError> {
            if at == format::NONE {
                return Ok(None);
            }
            strings
                .get(at as usize..(at + len) as usize)
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(|s| Some(s.to_string()))
                .ok_or_else(|| invalid(&blob, "a layer's texture name is out of range"))
        };
        let mut out = Vec::with_capacity(layers.len());
        for l in &layers {
            out.push(CookedTerrainLayer {
                albedo: text(l.albedo_at, l.albedo_len)?,
                normal: text(l.normal_at, l.normal_len)?,
                layer: og_render::TerrainLayer {
                    albedo: None,
                    normal: None,
                    color: glam::Vec3::new(l.r, l.g, l.b),
                    roughness: l.roughness,
                    tile: l.tile,
                    triplanar: l.triplanar != 0,
                    normal_strength: l.normal_strength,
                    slope: [l.slope_min, l.slope_max],
                    slope_fade: l.slope_fade,
                    altitude: [l.altitude_min, l.altitude_max],
                    altitude_fade: l.altitude_fade,
                    albedo_layer: (l.albedo_layer != format::NONE).then_some(l.albedo_layer),
                    normal_layer: (l.normal_layer != format::NONE).then_some(l.normal_layer),
                    project_slope: l.project_slope,
                    anti_tiling: l.anti_tiling,
                    height_contrast: l.height_contrast,
                    macro_variation: l.macro_variation,
                    displacement: l.displacement,
                    cell_variation: l.cell_variation,
                },
            });
        }
        Ok(CookedTerrain {
            id: id.to_string(),
            layout,
            range: (m.height_min, m.height_max),
            layers: out,
            albedo_array: text(m.albedo_array_at, m.albedo_array_len)?,
            normal_array: text(m.normal_array_at, m.normal_array_len)?,
            height_blend: m.height_blend,
            lod: og_render::TerrainLod {
                spacing: m.lod_spacing,
                ring: m.lod_ring,
                budget: 0,
                error: 0.0,
            },
            cast_shadows: m.cast_shadows != 0,
            collision: m.collision != 0,
            sea_level: (m.sea != 0).then_some(m.sea_level),
            tiles: std::sync::Arc::new(TerrainTiles {
                rows: tiles,
                heights,
                control,
            }),
        })
    }

    /// Reads and decodes a terrain (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_terrain(&self, id: &str) -> Result<CookedTerrain, LoadError> {
        self.terrain(id, &self.read(id)?)
    }

    /// Decodes an environment map blob (`type environment`): the texels
    /// inflated, the map ready for `RenderScene::add_environment_map`.
    pub fn environment(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<og_render::EnvironmentMapData, LoadError> {
        let blob = self.blob::<format::EnvironmentMap>(id, bytes)?;
        let meta = one::<format::EnvironmentMeta>(&blob)?;
        if meta.levels != og_render::ENVIRONMENT_MAP_LEVELS {
            return Err(invalid(
                &blob,
                format!(
                    "{} levels; this engine's environment maps have {}",
                    meta.levels,
                    og_render::ENVIRONMENT_MAP_LEVELS
                ),
            ));
        }
        let want = og_render::EnvironmentMapData::texel_count(meta.size);
        if meta.texel_count as usize != want {
            return Err(invalid(
                &blob,
                format!(
                    "a {}² map holds {want} texels, not {}",
                    meta.size, meta.texel_count
                ),
            ));
        }
        let bytes =
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(blob.raw("texels")?, want * 4)
                .map_err(|e| invalid(&blob, format!("texels: {e:?}")))?;
        if bytes.len() != want * 4 {
            return Err(invalid(&blob, "texels: truncated"));
        }
        let texels = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_le_bytes(*b))
            .collect();
        let sh = table::<format::EnvironmentShRow>(&blob, "irradiance")?;
        if sh.len() != 9 {
            return Err(invalid(
                &blob,
                format!("{} irradiance rows, not 9", sh.len()),
            ));
        }
        let map = og_render::EnvironmentMapData {
            size: meta.size,
            texels,
            irradiance: std::array::from_fn(|k| [sh[k].r, sh[k].g, sh[k].b]),
        };
        map.validate().map_err(|e| invalid(&blob, e))?;
        Ok(map)
    }

    /// Reads and decodes an environment map (native).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_environment(&self, id: &str) -> Result<og_render::EnvironmentMapData, LoadError> {
        self.environment(id, &self.read(id)?)
    }
}
