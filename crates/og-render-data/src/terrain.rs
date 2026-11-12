mod sampler;
pub use sampler::*;

use crate::scene::TextureHandle;
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// The most layers a [`Terrain`] has: up to four with textures of their
/// own, up to sixteen whose maps are layers of [`Terrain::layer_arrays`].
pub const MAX_TERRAIN_LAYERS: usize = 16;
/// The most layers with textures of their own (four albedo and four
/// normal bindings), and the most a pixel blends: each pixel blends its
/// four strongest layers.
pub const TERRAIN_TEXTURED_LAYERS: usize = 4;
/// The most clipmap levels a terrain draws.
pub const MAX_TERRAIN_LEVELS: usize = 16;

/// A terrain's layer weights: RGBA8, one channel per layer (linear), rows
/// from -z to +z, columns from -x to +x, stretched over the terrain. Past
/// four layers it holds more pages of four channels, one after the other
/// (`rgba` is `width × height × 4` bytes per page: page `p` the layers
/// `4p` to `4p + 3`); a layer past its pages reads 1.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainControl {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<[u8]>,
}

impl TerrainControl {
    /// Its pages of four channels (1 to 4).
    pub fn pages(&self) -> u32 {
        let page = (self.width as usize * self.height as usize * 4).max(1);
        (self.rgba.len() / page).max(1) as u32
    }

    /// Page `p` alone: a four-channel map.
    pub fn page(&self, p: u32) -> TerrainControl {
        let n = self.width as usize * self.height as usize * 4;
        let at = (p as usize * n).min(self.rgba.len());
        TerrainControl {
            width: self.width,
            height: self.height,
            rgba: Arc::from(&self.rgba[at..(at + n).min(self.rgba.len())]),
        }
    }
}

/// A terrain's geometry: `columns` × `rows` heights in metres over a
/// `size` rectangle centred on the terrain's origin, row-major from -z to
/// +z and -x to +x (sample `(c, r)` at `-size / 2 + (c, r) · cell`), an
/// optional control map and optional holes.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainData {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
    pub heights: Arc<[f32]>,
    pub control: Option<TerrainControl>,
    /// The cells cut out of the surface: `(columns - 1)` × `(rows - 1)`
    /// bytes, row-major like the heights, cell `(c, r)` the quad whose
    /// -x, -z corner is sample `(c, r)`; non-zero is a hole. Neither drawn
    /// nor collided with (`None`: no holes).
    pub holes: Option<Arc<[u8]>>,
}

impl TerrainData {
    /// A flat terrain of `columns` × `rows` samples at height 0.
    pub fn flat(size: Vec2, columns: u32, rows: u32) -> TerrainData {
        TerrainData {
            size,
            columns,
            rows,
            heights: Arc::from(vec![0.0; (columns * rows) as usize]),
            control: None,
            holes: None,
        }
    }

    /// Cells along x and z (the hole map's size).
    pub fn cells(&self) -> (u32, u32) {
        (self.columns.max(1) - 1, self.rows.max(1) - 1)
    }

    /// Whether the cell under `local` (x and z relative to the terrain's
    /// centre) is a hole; off the terrain, never.
    pub fn hole(&self, local: Vec2) -> bool {
        let Some(holes) = &self.holes else {
            return false;
        };
        TerrainSampler::of(self)
            .cell_of(local)
            .is_some_and(|(c, r)| holes[(r * self.cells().0 + c) as usize] != 0)
    }

    /// The distance between two samples along x and z.
    pub fn cell(&self) -> Vec2 {
        self.size
            / Vec2::new(
                self.columns.max(2) as f32 - 1.0,
                self.rows.max(2) as f32 - 1.0,
            )
    }

    /// Why the data cannot be drawn, if it cannot.
    pub fn check(&self) -> Result<(), String> {
        if self.columns < 2 || self.rows < 2 {
            return Err(format!(
                "a terrain needs at least 2 × 2 samples, got {} × {}",
                self.columns, self.rows
            ));
        }
        if self.heights.len() != (self.columns * self.rows) as usize {
            return Err(format!(
                "a {} × {} terrain needs {} heights, got {}",
                self.columns,
                self.rows,
                self.columns * self.rows,
                self.heights.len()
            ));
        }
        if !(self.size.x > 0.0 && self.size.y > 0.0 && self.size.is_finite()) {
            return Err(format!(
                "a terrain's size must be positive, got {}",
                self.size
            ));
        }
        if self.heights.iter().any(|h| !h.is_finite()) {
            return Err("a terrain's heights must be finite".into());
        }
        if let Some(h) = &self.holes {
            let (cx, cz) = self.cells();
            if h.len() != (cx * cz) as usize {
                return Err(format!(
                    "a {} × {} terrain has {} cells, its hole map {}",
                    self.columns,
                    self.rows,
                    cx * cz,
                    h.len()
                ));
            }
        }
        if let Some(c) = &self.control {
            let page = (c.width * c.height * 4) as usize;
            let pages = MAX_TERRAIN_LAYERS / 4;
            if c.width == 0
                || c.height == 0
                || c.rgba.len() % page != 0
                || !(1..=pages).contains(&(c.rgba.len() / page))
            {
                return Err(format!(
                    "a {} × {} control map needs {page} bytes per page of four layers, 1 to {pages} pages, got {}",
                    c.width,
                    c.height,
                    c.rgba.len()
                ));
            }
        }
        Ok(())
    }

    /// The height at `local` (x and z relative to the terrain's centre) on
    /// the triangles the renderer draws at its finest level and physics
    /// collides with: each cell uses the diagonal with the smaller corner
    /// height difference, ties keeping (-x, -z) to (+x, +z); clamped to the edge.
    pub fn height(&self, local: Vec2) -> f32 {
        TerrainSampler::of(self).height(local)
    }

    /// The surface normal at `local`: the samples' normals (central
    /// differences, as the renderer's normal map) interpolated bilinearly.
    pub fn normal(&self, local: Vec2) -> Vec3 {
        TerrainSampler::of(self).normal(local)
    }

    /// The lowest and highest height.
    pub fn range(&self) -> (f32, f32) {
        self.heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)))
    }
}

/// One material layer of a [`Terrain`]. The zero value is a plain grey
/// layer everywhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainLayer {
    /// The colour texture (sRGB; its alpha is the layer's height for
    /// `Terrain::height_blend`). `None`: `color` alone.
    pub albedo: Option<TextureHandle>,
    /// A tangent-space normal map (linear).
    pub normal: Option<TextureHandle>,
    /// Multiplies the albedo (linear).
    pub color: Vec3,
    pub roughness: f32,
    /// Metres per repeat of the textures.
    pub tile: f32,
    /// Sample by three projections blended by the normal (cliffs), unless
    /// `project_slope` decides.
    pub triplanar: bool,
    pub normal_strength: f32,
    /// The slopes (degrees from flat) where the layer shows, and how far
    /// beyond them it fades out.
    pub slope: [f32; 2],
    pub slope_fade: f32,
    /// The heights above the terrain's origin where it shows, and the fade.
    pub altitude: [f32; 2],
    pub altitude_fade: f32,
    /// With [`Terrain::layer_arrays`]: the layer of the albedo array this
    /// layer's colour (and height, in alpha) is (`None`: `color` alone).
    /// Past the array's last layer: the last.
    pub albedo_layer: Option<u32>,
    /// With [`Terrain::layer_arrays`]: the layer of the normal array this
    /// layer's normal map is (`None`: none).
    pub normal_layer: Option<u32>,
    /// Breaks the texture's repeat (0 off, 1 full): each sample blends
    /// three hexagonal tiles of the texture, each turned and moved by its
    /// own hash (Mikkelsen's hex-tiling, `terrain_lit.wgsl`), so no period
    /// shows across a field.
    pub anti_tiling: f32,
    /// How sharp the height blend is at this layer's edges, 0 to 1 (0: the
    /// terrain's window of 0.15, 1: a tenth of it).
    pub height_contrast: f32,
    /// The slope in degrees past which the layer is sampled by the two
    /// projections facing the surface most (biplanar) instead of from
    /// above, faded over 10° around it; 0 leaves `triplanar` to decide.
    pub project_slope: f32,
    /// Low-frequency brightness and hue drift over the ground, and the
    /// texture at a quarter of its frequency blended in with distance, 0 to 1.
    pub macro_variation: f32,
    /// Metres the drawn ground moves up and down by the layer's height
    /// (its albedo's alpha, about its middle) within
    /// [`Terrain::displacement_distance`] of the camera (0: flat).
    /// Only drawn: collision and og.terrain.sample keep the field.
    pub displacement: f32,
    /// How much each cell of the layer's pattern takes its own tone, 0
    /// to 1: its normal map's alpha names the cell (opaque: none), and
    /// each cell's brightness (up to ±25 %) and warmth (±20 %) move by it
    /// (`cell_tone`, terrain_layers.wgsl).
    pub cell_variation: f32,
}

impl Default for TerrainLayer {
    fn default() -> Self {
        TerrainLayer {
            albedo: None,
            normal: None,
            color: Vec3::splat(0.5),
            roughness: 0.9,
            tile: 4.0,
            triplanar: false,
            normal_strength: 1.0,
            slope: [0.0, 90.0],
            slope_fade: 4.0,
            altitude: [-1.0e6, 1.0e6],
            altitude_fade: 4.0,
            albedo_layer: None,
            normal_layer: None,
            anti_tiling: 0.0,
            height_contrast: 0.0,
            project_slope: 0.0,
            macro_variation: 0.0,
            displacement: 0.0,
            cell_variation: 0.0,
        }
    }
}

/// How finely a terrain is drawn. The zero value is the default.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainLod {
    /// The finest level's spacing in metres (0: the data's sample spacing).
    pub spacing: f32,
    /// Quads per side of every level (0: 64; rounded to a multiple of 4,
    /// 16 to 256). More quads push each level's detail further out.
    pub ring: u32,
    /// GPU bytes for its resident tiles (0: [`DEFAULT_TERRAIN_BUDGET`]):
    /// how many tiles it keeps, at least 8.
    pub budget: u64,
    /// The height error, in pixels, a coarser level may show before the
    /// finer one reaches further out (0: 1; `terrain_error.rs`). Larger
    /// draws fewer vertices.
    pub error: f32,
}

/// A terrain's tile budget when its lod gives none: 16 MiB.
pub const DEFAULT_TERRAIN_BUDGET: u64 = 16 << 20;

impl TerrainLod {
    /// Quads per side of every level, as drawn.
    pub fn ring_quads(&self) -> u32 {
        let r = if self.ring == 0 { 64 } else { self.ring };
        (r.clamp(16, 256) / 4) * 4
    }

    /// The pixels of height error a level may show, as drawn.
    pub fn error_pixels(&self) -> f32 {
        if self.error > 0.0 { self.error } else { 1.0 }
    }

    /// GPU bytes for resident tiles, as drawn.
    pub fn budget(&self) -> u64 {
        if self.budget == 0 {
            DEFAULT_TERRAIN_BUDGET
        } else {
            self.budget
        }
    }
}

impl TerrainData {
    /// The control map's channels at `local` (four per page; 1 past its
    /// pages), bilinear between texel centres (as the renderer's linear
    /// sampler reads it), 0..1; all 1 without one.
    pub fn control_at(&self, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        let Some(c) = &self.control else {
            return [1.0; MAX_TERRAIN_LAYERS];
        };
        let pages = c.pages() as usize;
        let page = (c.width * c.height * 4) as usize;
        let uv = ((local + self.size * 0.5) / self.size).clamp(Vec2::ZERO, Vec2::ONE);
        let t = uv * Vec2::new(c.width as f32, c.height as f32) - 0.5;
        let i = t.floor();
        let f = t - i;
        let texel = |x: f32, y: f32| -> [f32; MAX_TERRAIN_LAYERS] {
            let x = (x as i64).clamp(0, c.width as i64 - 1) as usize;
            let y = (y as i64).clamp(0, c.height as i64 - 1) as usize;
            let at = (y * c.width as usize + x) * 4;
            std::array::from_fn(|k| {
                if k / 4 < pages {
                    c.rgba[(k / 4) * page + at + k % 4] as f32 / 255.0
                } else {
                    1.0
                }
            })
        };
        let (a, b) = (texel(i.x, i.y), texel(i.x + 1.0, i.y));
        let (d, e) = (texel(i.x, i.y + 1.0), texel(i.x + 1.0, i.y + 1.0));
        std::array::from_fn(|k| {
            let lo = a[k] + (b[k] - a[k]) * f.x;
            let hi = d[k] + (e[k] - d[k]) * f.x;
            lo + (hi - lo) * f.y
        })
    }
}

/// The height over a cell of corner heights `h` (-x -z, +x -z, -x +z,
/// +x +z) at `f` (0 to 1 along x and z), on the triangle of its split
/// (og_core's heightfield_zigzag; `terrain_vertex.wgsl` split_height).
pub fn split_height(h: [f32; 4], f: Vec2) -> f32 {
    if (h[0] - h[3]).abs() > (h[1] - h[2]).abs() {
        if f.x + f.y <= 1.0 {
            h[0] + f.x * (h[1] - h[0]) + f.y * (h[2] - h[0])
        } else {
            h[3] + (1.0 - f.x) * (h[2] - h[3]) + (1.0 - f.y) * (h[1] - h[3])
        }
    } else if f.x >= f.y {
        h[0] + f.x * (h[1] - h[0]) + f.y * (h[3] - h[1])
    } else {
        h[0] + f.y * (h[2] - h[0]) + f.x * (h[3] - h[2])
    }
}
