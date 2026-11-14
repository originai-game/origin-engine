//! The ground read at a point on the CPU.

use super::*;
use og_core::IdMap;

pub(crate) fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `band` in terrain_lit.wgsl.
pub(crate) fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
    let f = fade.max(1e-4);
    smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x))
}

/// What placing reads at a point of the terrain's space: its height,
/// normal and control channels (`None`: not resident yet).
pub(crate) trait Ground {
    fn size(&self) -> Vec2;
    fn height(&self, p: Vec2) -> Option<f32>;
    fn normal(&self, p: Vec2) -> Option<Vec3>;
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]>;
    /// Whether `p` is over a hole (no copy grows there).
    fn hole(&self, p: Vec2) -> Option<bool>;
}

impl Ground for TerrainData {
    fn size(&self) -> Vec2 {
        self.size
    }
    fn height(&self, p: Vec2) -> Option<f32> {
        Some(TerrainData::height(self, p))
    }
    fn normal(&self, p: Vec2) -> Option<Vec3> {
        Some(TerrainData::normal(self, p))
    }
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        Some(self.control_at(p))
    }
    fn hole(&self, p: Vec2) -> Option<bool> {
        Some(TerrainData::hole(self, p))
    }
}

/// A streamed terrain's resident level-0 tiles as ground.
pub(crate) struct TileGround<'a> {
    pub layout: TerrainLayout,
    pub tiles: &'a IdMap<u32, Arc<TerrainTile>>,
}

impl TileGround<'_> {
    /// The level-0 tile under `p` and its place.
    fn tile(&self, p: Vec2) -> Option<(&TerrainTile, u32, u32)> {
        let l = &self.layout;
        let s = ((p + l.size * 0.5) / l.cell()).max(Vec2::ZERO);
        let (tx, tz) = l.tiles(0);
        let x = ((s.x as u32) / l.tile).min(tx - 1);
        let z = ((s.y as u32) / l.tile).min(tz - 1);
        Some((self.tiles.get(&l.index(0, x, z))?.as_ref(), x, z))
    }

    fn sampler<'b>(
        &self,
        t: &'b TerrainTile,
        x: u32,
        z: u32,
    ) -> crate::TerrainSampler<impl Fn(u32, u32) -> f32 + 'b> {
        let l = self.layout;
        let side = l.heights_side() as i64;
        let (bx, bz) = ((x * l.tile) as i64, (z * l.tile) as i64);
        crate::TerrainSampler {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            sample: move |c: u32, r: u32| {
                let i = (c as i64 - bx + 1).clamp(0, side - 1);
                let j = (r as i64 - bz + 1).clamp(0, side - 1);
                t.heights[(j * side + i) as usize]
            },
        }
    }
}

impl Ground for TileGround<'_> {
    fn size(&self) -> Vec2 {
        self.layout.size
    }
    fn hole(&self, p: Vec2) -> Option<bool> {
        if !self.layout.holes {
            return Some(false);
        }
        let (t, x, z) = self.tile(p)?;
        let s = self.sampler(t, x, z);
        Some(
            s.cell_of(p)
                .is_some_and(|(c, r)| crate::terrain_tiles::hole_bit((s.sample)(c, r))),
        )
    }
    fn height(&self, p: Vec2) -> Option<f32> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).height(p))
    }
    fn normal(&self, p: Vec2) -> Option<Vec3> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).normal(p))
    }
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        let (t, x, z) = self.tile(p)?;
        let l = &self.layout;
        let Some(texels) = &t.control else {
            return Some([1.0; MAX_TERRAIN_LAYERS]);
        };
        let page = l.control_page_bytes();
        let c = l.control as f32;
        let side = l.control_side() as i64;
        let s = (p + l.size * 0.5) / l.cell();
        // Texel coordinates within the tile (its border at 0).
        let g = (s - Vec2::new((x * l.tile) as f32, (z * l.tile) as f32)) / l.tile as f32 * c + 0.5;
        let i = g.floor();
        let f = g - i;
        // Channel k is page k / 4's (each page's first mip), 1 past the pages.
        let texel = |a: f32, b: f32, k: usize| {
            let a = (a as i64).clamp(0, side - 1);
            let b = (b as i64).clamp(0, side - 1);
            texels[(k / 4) * page + ((b * side + a) * 4) as usize + k % 4] as f32 / 255.0
        };
        let pages = l.pages.max(1) as usize;
        Some(std::array::from_fn(|k| {
            if k / 4 >= pages {
                return 1.0;
            }
            let lo = texel(i.x, i.y, k) + (texel(i.x + 1.0, i.y, k) - texel(i.x, i.y, k)) * f.x;
            let hi = texel(i.x, i.y + 1.0, k)
                + (texel(i.x + 1.0, i.y + 1.0, k) - texel(i.x, i.y + 1.0, k)) * f.x;
            lo + (hi - lo) * f.y
        }))
    }
}

/// Every weight on the first layer.
fn first_layer() -> [f32; MAX_TERRAIN_LAYERS] {
    std::array::from_fn(|k| if k == 0 { 1.0 } else { 0.0 })
}

impl Terrain {
    /// The layers' weights at `local` (summing to 1), as the renderer weighs
    /// them before the height blend: control map times the slope and
    /// altitude rules; the first layer where every weight is 0.
    pub fn layer_weights(&self, data: &TerrainData, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        self.weights_on(data, local).unwrap_or_else(first_layer)
    }

    pub(crate) fn weights_on(
        &self,
        g: &dyn Ground,
        local: Vec2,
    ) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        let c = g.control(local)?;
        let n = g.normal(local)?;
        let angle = n.y.clamp(-1.0, 1.0).acos().to_degrees();
        let altitude = g.height(local)?;
        let mut w = [0.0f32; MAX_TERRAIN_LAYERS];
        for (i, l) in self.layers.iter().take(MAX_TERRAIN_LAYERS).enumerate() {
            w[i] = c[i]
                * band(angle, l.slope[0], l.slope[1], l.slope_fade)
                * band(altitude, l.altitude[0], l.altitude[1], l.altitude_fade);
        }
        let sum: f32 = w.iter().sum();
        if sum < 1e-4 {
            return Some(first_layer());
        }
        Some(w.map(|x| x / sum))
    }
}
