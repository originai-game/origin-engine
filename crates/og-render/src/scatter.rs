//! Scatter: detail meshes (grass, flowers, stones) over a terrain where one
//! of its layers shows, near the camera (phase 2; drawn from cached chunks
//! in phase 3).
//!
//! Copies sit on a jittered grid fixed to the terrain: one candidate per
//! cell of `1 / sqrt(density)` metres, turned and sized by a hash of the
//! cell and the seed, kept with the probability of its layer's weight
//! there, so a copy keeps its place, size and turn as the camera moves.
//! [`Terrain::scatter`] answers them for a camera on the CPU (tests, tools).
//!
//! **Drawn from chunks.** A [`ScatterDraw`] in [`crate::RenderScene::scatters`]
//! has the renderer place its copies itself, 32 × 32 cells at a time: a
//! chunk's world transforms are built once (when it first comes within the
//! distance, and again only when its terrain data changes), uploaded into
//! an instance buffer of its own and kept while it is in use. Each frame
//! costs a cull of the chunks against every pass's volume and one instanced
//! draw per chunk; the vertex shader drops copies past the distance and
//! shrinks and thins them out over the fade: each copy shrinks to nothing
//! over its own part of the fade, chosen by a hash of its place, so the
//! field thins as grass does (`shaders/scatter*.wgsl`). No pixel of the
//! distance fade is dithered: a static dither left grain TAA cannot
//! resolve. Only the crossfade from a copy's mesh to its impostor
//! thresholds pixels, by the blue-noise tile turned each render frame
//! ([`crossfade_noise_offset`]), the same in the lit, prepass and shadow
//! passes. A streamed terrain's chunks are built
//! once the level-0 tiles under them have arrived.
//!
//! **Wind.** [`crate::RenderScene::wind`] sways the copies in the vertex
//! stage of every pass (lit, prepass and shadow): each vertex above the
//! copy's origin moves downwind by the copy's lean at the tick times its
//! height times its height over the mesh's (so the root stays put and the
//! tip moves furthest), lowered to keep its distance from the root. The
//! lean is `strength × (1 + gust × w)`, `w` a wave of two octaves that
//! travels downwind at `speed` over [`WIND_WAVELENGTH`] (a little offset
//! per copy, from its position), so gusts sweep across the field. Its
//! phases are reduced on the CPU in f64 from the tick, so a long session
//! sways as smoothly as its first minute, and the previous frame's are
//! kept for the motion vectors. A chunk's bounds grow by the furthest lean.

use crate::material::Side;
use crate::scene::{MaterialHandle, MeshHandle};
use crate::terrain::{MAX_TERRAIN_LAYERS, Terrain, TerrainData, TerrainSource, horizontal_scale};
use crate::terrain_tiles::{TerrainLayout, TerrainTile};
use glam::{Mat3, Mat4, Vec2, Vec3};
use og_core::IdMap;
use std::collections::HashMap;
use std::sync::Arc;

/// The most instances one [`TerrainScatter`] places in a frame on the CPU.
pub const MAX_SCATTER_INSTANCES: usize = 65_536;

/// Cells per chunk side.
const CHUNK_CELLS: u32 = 32;
/// Chunks a frame builds for a streamed terrain (all of them for a field in
/// memory or a settled frame).
const CHUNKS_PER_FRAME: usize = 16;
/// Frames a chunk out of use is kept.
const KEEP_FRAMES: u64 = 240;

/// How copies are scattered: the zero value is not usable; use `default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainScatter {
    /// The layer whose weight (control map, slope and altitude rules,
    /// normalised over the layers) places the instances.
    pub layer: u32,
    /// Instances per square metre where the layer's weight is 1.
    pub density: f32,
    /// How far from the camera (along x and z) instances are drawn.
    pub distance: f32,
    /// The outer part of `distance` over which they shrink and thin out (0..1).
    pub fade: f32,
    /// The range of their uniform scale.
    pub scale: [f32; 2],
    /// How far they lean with the ground: 0 upright, 1 along its normal.
    pub align: f32,
    pub seed: u32,
    /// How much the [`crate::RenderScene::wind`] sways them: 1 as it
    /// blows, 0 still (stones). Not part of the placement.
    pub sway: f32,
    /// Clusters (G4): copies grow only where a smooth noise of patches
    /// `noise_scale` metres across rises past `noise_threshold` (0..1; 0
    /// everywhere), their edges dithered over a short band.
    pub noise_scale: f32,
    pub noise_threshold: f32,
    /// How far (m, along x and z) this scatter's copies and those of the
    /// scatters placed before it on the same terrain keep apart: the larger
    /// of the two footprints. 0: none.
    pub footprint: f32,
    /// Placed before every scatter of lower priority on the same terrain
    /// (equal: the scene's order), whose copies keep clear of its footprints.
    pub priority: u32,
    /// How far above the scene's water ([`crate::RenderScene::water`])
    /// the ground must stand for a copy to grow (m); 0: anywhere.
    pub avoid_water: f32,
    /// Thins the copies where its red channel is below 1, over the
    /// terrain's extent as its control map. Drawn only (the GPU cull,
    /// `scatter_cull.rs`; no effect without compute); not part of the
    /// placement.
    pub density_map: Option<crate::TextureHandle>,
}

/// What places a scatter's copies besides its own rules
/// ([`Terrain::scatter_among`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScatterContext {
    /// The scatters over the same terrain placed before it (a higher
    /// priority, or the same and earlier in the scene), first placed
    /// first: its copies keep clear of their footprints.
    pub before: Vec<TerrainScatter>,
    /// The water surfaces `avoid_water` keeps clear of: each one's frame
    /// from the world (its surface local y = 0) and its half extent along
    /// local x and z (infinite for an endless sea).
    pub water: Vec<(Mat4, Vec2)>,
}

impl ScatterContext {
    /// The context of scatter `index` of `all` (each with its terrain's
    /// data index): the usable scatters over the same data placed before
    /// it, and `water` (world surfaces: their transform, size and whether
    /// endless).
    pub fn of(
        all: &[(usize, TerrainScatter)],
        index: usize,
        layers: usize,
        water: impl IntoIterator<Item = (Mat4, Vec2, bool)>,
    ) -> ScatterContext {
        let (data, me) = all[index];
        let mut before: Vec<(u32, usize, TerrainScatter)> = all
            .iter()
            .enumerate()
            .filter(|&(i, &(d, o))| {
                d == data
                    && i != index
                    && o.usable(layers)
                    && (o.priority > me.priority || (o.priority == me.priority && i < index))
            })
            .map(|(i, &(_, o))| (o.priority, i, o))
            .collect();
        og_core::sort::sort_by(&mut before, |a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        ScatterContext {
            before: before.into_iter().map(|b| b.2).collect(),
            water: water
                .into_iter()
                .map(|(m, size, endless)| {
                    let half = if endless { Vec2::INFINITY } else { size * 0.5 };
                    (m.inverse(), half)
                })
                .collect(),
        }
    }

    /// Whether any rule of `s` or the scatters before it reads the water.
    fn reads_water(&self, s: &TerrainScatter) -> bool {
        s.avoid_water > 0.0 || self.before.iter().any(|b| b.avoid_water > 0.0)
    }
}

impl Default for TerrainScatter {
    fn default() -> Self {
        TerrainScatter {
            layer: 0,
            density: 2.0,
            distance: 40.0,
            fade: 0.3,
            scale: [0.8, 1.2],
            align: 0.0,
            seed: 1,
            sway: 1.0,
            noise_scale: 8.0,
            noise_threshold: 0.0,
            footprint: 0.0,
            priority: 0,
            avoid_water: 0.0,
            density_map: None,
        }
    }
}

/// The wind the renderer sways vegetation with ([`crate::RenderScene::wind`]):
/// [`ScatterDraw`] copies lean downwind and sway in gusts that sweep
/// across the field (the module's documentation has the shape). Visual
/// only, a function of the tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    /// The mean lean: how far a copy's tip moves downwind per metre of its
    /// height (0.2: a 1 m stem's tip leans 20 cm); 0 is still.
    pub strength: f32,
    /// Where it blows: its horizontal part, normalised.
    pub direction: Vec3,
    /// How fast the gusts travel downwind (m/s).
    pub speed: f32,
    /// How much the lean varies in the gusts, 0..1: 0 a steady lean, 1
    /// from upright to twice the mean.
    pub gust: f32,
    /// How much the simulated field (`wind_field.rs`) swirls on its own,
    /// 0..1: curl noise of the gusts' size, at this share of the speed.
    pub turbulence: f32,
    /// The gust patches' size (m).
    pub gust_size: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Wind {
            strength: 0.2,
            direction: Vec3::X,
            speed: 3.0,
            gust: 0.6,
            turbulence: 0.3,
            gust_size: 14.0,
        }
    }
}

/// The length of the wind's gust wave (metres).
pub const WIND_WAVELENGTH: f32 = 10.0;

/// The wind's second octave: its wavenumber and time phase over the
/// first's (scatter_common.wgsl `scatter_sway`).
const WIND_OCTAVE: f64 = 2.3;

/// The wind's two time phases at `tick` (radians, reduced in f64): the
/// first octave's `k speed t` and the second's.
pub fn wind_phases(wind: &Wind, tick: u64, tick_rate: f32) -> [f32; 2] {
    let t = tick as f64 / f64::from(tick_rate.max(1e-3));
    let k = std::f64::consts::TAU / f64::from(WIND_WAVELENGTH);
    let first = k * f64::from(wind.speed) * t;
    [first, first * WIND_OCTAVE].map(|p| p.rem_euclid(std::f64::consts::TAU) as f32)
}

/// The material shader that sways a placed model in the wind (R6): its
/// `vertex_offset` hook moves each vertex above the model's origin as
/// `scatter_sway` moves a scattered copy's (the same lean, gusts and
/// wave), with its height in the model's units. Parameters, written each
/// frame by its user (og-runtime's `og.Model.sway`):
/// `wind` (xy the wind's horizontal direction, z its strength times the
/// sway, w its gust), `wave` (xy [`wind_phases`], z the wave's wavenumber,
/// w the model's top above its origin). A hooked material draws through
/// the lit, prepass and shadow pipelines of every other material shader,
/// static or skinned, so the shadows and the depth sway with it.
pub fn wind_shader() -> crate::MaterialShader {
    use crate::hooks::{ParamKind, ShaderParam};
    let param = |name: &str| ShaderParam {
        name: name.into(),
        kind: ParamKind::Vec4,
        default: [0.0; 4],
    };
    crate::MaterialShader {
        name: "og.wind".into(),
        code: WIND_HOOK.into(),
        params: vec![param("wind"), param("wave")],
        mode: Default::default(),
    }
}

/// `wind_shader`'s hook: `scatter_sway` (scatter_common.wgsl) over the
/// vertex's own place, the vertex's height its local y.
const WIND_HOOK: &str = r#"
fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  let lean = p.wind.z;
  let top = p.wave.w;
  let y = h.local.y;
  if (lean <= 0.0 || top <= 0.0 || y <= 0.0) {
    return vec3<f32>(0.0);
  }
  let dir = p.wind.xy;
  // The place in the world (the same bits wherever the render origin is).
  let at = h.world.xz + h.origin.xz;
  let x = p.wave.z * dot(at, dir);
  let wave = (sin(x - p.wave.x) + 0.5 * sin(2.3 * x + 1.7 - p.wave.y)) / 1.5;
  let d = lean * (1.0 + p.wind.w * wave) * y * min(y / top, 1.0);
  let drop = y - sqrt(max(y * y - d * d, 0.0));
  return vec3<f32>(dir.x * d, -drop, dir.y * d);
}
"#;

impl TerrainScatter {
    /// What places its copies, as bits (a chunk's key).
    fn placement_bits(&self) -> [u32; 11] {
        [
            self.density,
            self.scale[0],
            self.scale[1],
            self.align,
            self.layer as f32,
            self.seed as f32,
            self.noise_scale,
            self.noise_threshold,
            self.footprint,
            self.priority as f32,
            self.avoid_water,
        ]
        .map(f32::to_bits)
    }

    fn usable(&self, layers: usize) -> bool {
        (self.layer as usize) < layers.min(MAX_TERRAIN_LAYERS)
            && self.density > 0.0
            && self.distance > 0.0
            && self.density.is_finite()
            && self.distance.is_finite()
    }

    /// The grid's cell size in the terrain's space.
    fn cell(&self) -> f32 {
        1.0 / self.density.clamp(1e-4, 400.0).sqrt()
    }

    /// Where on screen the fade starts (world metres).
    pub fn fade_from(&self) -> f32 {
        self.distance * (1.0 - self.fade.clamp(0.0, 1.0))
    }
}

/// Copies of a mesh scattered over a terrain drawn this frame, placed and
/// drawn by the renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScatterDraw {
    /// The terrain: an index into [`crate::RenderScene::terrains`] (its
    /// data, transform and layers place the copies).
    pub terrain: usize,
    pub mesh: MeshHandle,
    /// Its StandardMaterial (opaque or cut out).
    pub material: MaterialHandle,
    pub scatter: TerrainScatter,
    /// Whether the copies are drawn into the shadow passes.
    pub cast_shadows: bool,
    /// The mesh's impostor (`impostor.rs`): copies past
    /// `impostor_distance` draw as it instead of the mesh, dithered across
    /// the last tenth before it.
    pub impostor: Option<crate::Impostor>,
    /// How far from the camera (along x and z) copies draw as their mesh;
    /// 0 (or no impostor): always. Useful below the scatter's `distance`.
    pub impostor_distance: f32,
}

/// Value noise over the lattice of `seed`, three octaves turned off the
/// axes, normalised to 0..1 (the clusters of `TerrainScatter::noise_threshold`).
fn cluster_noise(p: Vec2, seed: u32) -> f32 {
    let lattice = |q: Vec2, salt: u32| {
        let i = q.floor();
        let f = q - i;
        let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
        let at = |dx: i32, dz: i32| {
            let x = (i.x as i32).wrapping_add(dx) as u32;
            let z = (i.y as i32).wrapping_add(dz) as u32;
            unit(hash(
                hash(x ^ salt).wrapping_add(z.wrapping_mul(0x9e37_79b9)) ^ seed,
            ))
        };
        let a = at(0, 0) + (at(1, 0) - at(0, 0)) * u.x;
        let b = at(0, 1) + (at(1, 1) - at(0, 1)) * u.x;
        a + (b - a) * u.y
    };
    let turn = Mat3::from_rotation_z(0.6);
    let (mut q, mut amp, mut sum, mut total) = (p, 1.0, 0.0, 0.0);
    for octave in 0..3u32 {
        sum += amp * lattice(q, octave.wrapping_mul(0x632b_e5ab));
        total += amp;
        q = turn.transform_vector2(q) * 2.03;
        amp *= 0.5;
    }
    sum / total
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `band` in terrain_lit.wgsl.
fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
    let f = fade.max(1e-4);
    smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x))
}

/// A 32-bit integer hash (lowbias32).
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / 16_777_216.0
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

    /// The copy of cell (`x`, `z`) of `s`'s grid under every rule: its
    /// own ([`Self::copy_at`]), then clear of the water by `avoid_water`
    /// and of the copies of the scatters placed `before` it by the larger
    /// footprint. `Some(None)` where none grows, `None` where the ground is
    /// not resident yet.
    fn copy_ruled(
        &self,
        g: &dyn Ground,
        s: &TerrainScatter,
        before: &[TerrainScatter],
        water: &[(Mat4, Vec2)],
        x: u32,
        z: u32,
    ) -> Option<Option<(u32, Mat4, Vec2)>> {
        let Some(copy) = self.copy_at(g, s, x, z)? else {
            return Some(None);
        };
        if s.avoid_water > 0.0 {
            let at = copy.1.w_axis.truncate();
            for (to_local, half) in water {
                let l = to_local.transform_point3(at);
                if l.x.abs() <= half.x && l.z.abs() <= half.y && l.y < s.avoid_water {
                    return Some(None);
                }
            }
        }
        let along = horizontal_scale(self.transform).max(1e-6);
        let p = copy.2;
        let size = g.size();
        for (i, b) in before.iter().enumerate() {
            let reach = s.footprint.max(b.footprint) / along;
            if reach <= 0.0 {
                continue;
            }
            let cell = b.cell();
            let cells = (size / cell).ceil();
            let lo = ((p - reach + size * 0.5) / cell).floor().max(Vec2::ZERO);
            let hi = ((p + reach + size * 0.5) / cell).floor().min(cells - 1.0);
            for bz in lo.y as u32..=hi.y.max(0.0) as u32 {
                for bx in lo.x as u32..=hi.x.max(0.0) as u32 {
                    if let Some((_, _, q)) = self.copy_ruled(g, b, &before[..i], water, bx, bz)?
                        && (q - p).length() < reach
                    {
                        return Some(None);
                    }
                }
            }
        }
        Some(Some(copy))
    }

    /// The copy of cell (`x`, `z`) of `s`'s grid by its own rules (layer,
    /// slope, altitude, holes, clusters): `Some(None)` where none grows,
    /// `None` where the ground is not resident yet.
    fn copy_at(
        &self,
        g: &dyn Ground,
        s: &TerrainScatter,
        x: u32,
        z: u32,
    ) -> Option<Option<(u32, Mat4, Vec2)>> {
        let cell = s.cell();
        let size = g.size();
        let half = size * 0.5;
        let cells_x = (size.x / cell).ceil() as u32;
        let key = z.wrapping_mul(cells_x).wrapping_add(x);
        let seed = hash(s.seed ^ 0x9e37_79b9);
        let h0 = hash(key ^ seed);
        let (h1, h2, h3, h4) = (hash(h0), hash(h0 ^ 1), hash(h0 ^ 2), hash(h0 ^ 3));
        let p = Vec2::new(x as f32 + unit(h1), z as f32 + unit(h2)) * cell - half;
        if p.x > half.x || p.y > half.y {
            return Some(None);
        }
        let weight = self.weights_on(g, p)?[s.layer as usize];
        if unit(h3) >= weight || g.hole(p)? {
            return Some(None);
        }
        if s.noise_threshold > 0.0 {
            // Patches in the world's metres; a short band dithered by the
            // copy's own value, so a cluster's edge thins out.
            let along = horizontal_scale(self.transform);
            let n = cluster_noise(p * along / s.noise_scale.max(1e-3), seed);
            let edge = smoothstep(s.noise_threshold - 0.04, s.noise_threshold + 0.04, n);
            if unit(hash(h0 ^ 5)) >= edge {
                return Some(None);
            }
        }
        let model = self.transform;
        let n3 = Mat3::from_mat4(model).inverse().transpose();
        let size = s.scale[0] + (s.scale[1] - s.scale[0]) * unit(h4);
        let normal = (n3 * g.normal(p)?).normalize_or(Vec3::Y);
        let up = Vec3::Y
            .lerp(normal, s.align.clamp(0.0, 1.0))
            .normalize_or(Vec3::Y);
        let turn = glam::Quat::from_rotation_y(unit(hash(h0 ^ 4)) * std::f32::consts::TAU);
        let rotation = glam::Quat::from_rotation_arc(Vec3::Y, up) * turn;
        let at = model.transform_point3(Vec3::new(p.x, g.height(p)?, p.y));
        Some(Some((
            key,
            Mat4::from_scale_rotation_translation(Vec3::splat(size), rotation, at),
            p,
        )))
    }

    /// Places `s`'s copies for a camera at `camera` on the CPU: each a cell
    /// key (stable while the cell is in range) and its local-to-world
    /// transform, at full size within `distance` of the camera along x and
    /// z (the renderer fades the outer `fade` of it), at most
    /// [`MAX_SCATTER_INSTANCES`], nearest rows first. `data` is this
    /// terrain's data.
    pub fn scatter(
        &self,
        data: &TerrainData,
        s: &TerrainScatter,
        camera: Vec3,
    ) -> Vec<(u32, Mat4)> {
        self.scatter_among(data, s, &ScatterContext::default(), camera)
    }

    /// [`Self::scatter`] with the scatters placed before `s` and the
    /// water (`ctx`): the copies the renderer draws for it in that scene.
    pub fn scatter_among(
        &self,
        data: &TerrainData,
        s: &TerrainScatter,
        ctx: &ScatterContext,
        camera: Vec3,
    ) -> Vec<(u32, Mat4)> {
        let mut out = Vec::new();
        if !s.usable(self.layers.len()) {
            return out;
        }
        let cell = s.cell();
        let half = data.size * 0.5;
        // Placed in the terrain's own space; distances are the world's.
        let along = horizontal_scale(self.transform);
        let local_camera = self.transform.inverse().transform_point3(camera);
        let eye = Vec2::new(local_camera.x, local_camera.z);
        let reach = s.distance / along;
        let cells = (data.size / cell).ceil();
        let lo = ((eye - reach + half) / cell).floor().max(Vec2::ZERO);
        let hi = ((eye + reach + half) / cell).ceil().min(cells);
        if lo.x >= hi.x || lo.y >= hi.y {
            return out;
        }
        let (x0, x1, z0, z1) = (lo.x as u32, hi.x as u32, lo.y as u32, hi.y as u32);
        // Rows outwards from the camera's, so the cap drops the farthest.
        let centre =
            (((eye.y + half.y) / cell).floor() as i64).clamp(z0 as i64, z1 as i64 - 1) as u32;
        let rows = (0..(z1 - z0)).map(|k| {
            let step = (k as i64 + 1) / 2;
            if k % 2 == 0 {
                centre as i64 + step
            } else {
                centre as i64 - step
            }
        });
        for z in rows
            .filter(|&z| z >= z0 as i64 && z < z1 as i64)
            .map(|z| z as u32)
        {
            for x in x0..x1 {
                if let Some(Some((key, m, p))) =
                    self.copy_ruled(data, s, &ctx.before, &ctx.water, x, z)
                    && (p - eye).length() <= reach
                {
                    out.push((key, m));
                    if out.len() >= MAX_SCATTER_INSTANCES {
                        return out;
                    }
                }
            }
        }
        out
    }
}

// ── drawing ─────────────────────────────────────────────────────────────────

/// `ScatterUniform` in scatter_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScatterGpu {
    camera: [f32; 4],
    fade: [f32; 4],
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    /// xy: og.Wind's horizontal direction (x, z), z: the scatter's sway
    /// (0: still); the lean is the wind field's (`wind_field.rs`).
    wind: [f32; 4],
    /// y: the mesh's top above its origin (m), zw: the scene's origin's x
    /// and z (`origin.rs`).
    wind_shape: [f32; 4],
    /// The impostor's sphere in the model's space (xyz centre, w radius),
    /// its grid and distance (x frames per side, y the distance past which
    /// copies draw as it: 0 never, z 1 when the lit pass has MSAA and so
    /// alpha to coverage, else its edge is the cut), and the direction to
    /// the sun (xyz).
    impostor: [f32; 4],
    impostor_grid: [f32; 4],
    impostor_light: [f32; 4],
    /// The trample map (`trample.rs` `Trample::grid`).
    trample: [f32; 4],
}

impl ScatterGpu {
    fn impostor(&mut self, i: &crate::Impostor, distance: f32, light: Vec3, msaa: bool) {
        self.impostor = i.center.extend(i.radius).to_array();
        self.impostor_grid = [
            i.grid as f32,
            distance,
            f32::from(u8::from(msaa)),
            f32::from(u8::from(i.material_atlas)),
        ];
        self.impostor_light = light.extend(0.0).to_array();
    }
}

/// How far the crossfade's blue-noise thresholds turn on render frame
/// `frame_index` (`ScatterUniform.fade.y`): the golden ratio's multiples,
/// so each pixel's thresholds over consecutive frames are a low-
/// discrepancy sequence TAA integrates, while every frame's pattern stays
/// the tile's blue noise (Wolfe, Morrical, Akenine-Möller, Ramamoorthi,
/// *Spatiotemporal Blue Noise Masks*, EGSR 2022, §2; Peters, *Free blue
/// noise textures*, 2016). Reduced in f64, so it does not drift.
fn crossfade_noise_offset(frame_index: u64) -> f32 {
    const GOLDEN: f64 = 0.618_033_988_749_894_8;
    ((frame_index as f64 * GOLDEN).fract()) as f32
}

const SLOT: u64 = 256;
/// The pipelines' kinds past a [`ScatterPass`]'s: its impostor's, and its
/// crowd cards'.
const IMPOSTOR_KIND: u8 = 3;
const CARD_KIND: u8 = 6;

/// A crowd card's record (crowd.rs `card_record`, `CardIn`): a skinned
/// record's stride, its transform's columns, then (its place, flash, flash
/// colour, tint).
const CARD_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: crate::skin::SKINNED_INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![4 => Float32x3, 5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32x4],
};

/// What a chunk was built from: the scatter, its terrain's placement and
/// data, the mesh and material, and the chunk's place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ChunkKey {
    data: usize,
    /// The data's revision, and for a streamed terrain the tiles under the chunk.
    source: (u64, Vec<usize>),
    /// The placement's parameters: the scatter's, the scatters' placed
    /// before it and, when a rule reads it, the water's (as bits).
    params: Vec<u32>,
    transform: [u32; 16],
    /// The scene's origin the chunk's transforms are relative to
    /// (`origin.rs`): `transform` is its terrain's placement in the world.
    origin: [u32; 3],
    layers: Vec<u32>,
    mesh: usize,
    chunk: (u32, u32),
}

struct Chunk {
    buffer: Option<wgpu::Buffer>,
    /// Its copies as the GPU cull reads them (`scatter_cull.rs`).
    cull_group: Option<wgpu::BindGroup>,
    count: u32,
    center: Vec3,
    radius: f32,
    used: u64,
}

/// One scatter's chunks to draw this frame.
pub(crate) struct ScatterBatch {
    pub mesh: usize,
    pub material: usize,
    /// Its material's shader (the scene's index): the lit and prepass
    /// pipelines run its hooks.
    pub shader: Option<usize>,
    pub cast_shadows: bool,
    pub side: Side,
    offset: u32,
    chunks: Vec<ChunkKey>,
    /// Per chunk, where the GPU cull put its copies for the main camera
    /// (`None`: drawn whole).
    culled: Vec<Option<crate::scatter_cull::Culled>>,
    /// How far the wind moves a copy at most: the chunks' bounds grow by it.
    sway: f32,
    /// The impostor's material: the chunks' copies past the impostor
    /// distance draw as it.
    impostor: Option<usize>,
}

/// The model impostors of one atlas drawn this frame
/// ([`crate::RenderScene::impostors`]): a range of the frame's impostor
/// instances, with their bounds.
pub(crate) struct ImpostorBatch {
    material: usize,
    cast_shadows: bool,
    offset: u32,
    first: u32,
    count: u32,
    center: Vec3,
    radius: f32,
}

/// A crowd's impostor cards this frame (crowd.rs **Impostors**,
/// `crowd_card.wgsl`): its sphere in the model's space, its atlas's views
/// and frames and the elevation it was baked at (radians), its draw in the
/// crowd cull's table (the GPU path), its records among the frame's CPU
/// card records (the CPU path) and whether it casts shadows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardInput {
    pub center: Vec3,
    pub radius: f32,
    pub views: u32,
    pub frames: u32,
    pub elevation: f32,
    pub draw: u32,
    pub cpu: Option<(u32, u32)>,
    pub cast_shadows: bool,
}

/// A crowd's cards as drawn: its uniform slot, and its material group (the
/// atlas and its parts' colours, `crowd_impostor.rs`) once baked.
pub(crate) struct CardBatch {
    pub input: CardInput,
    offset: u32,
    pub group: Option<wgpu::BindGroup>,
}

/// Where a pass's crowd cards come from: the crowd cull's row (its records
/// and draws), the CPU path's records, or none.
#[derive(Clone, Copy)]
pub(crate) enum CardSource<'a> {
    None,
    Gpu(&'a crate::crowd_gpu::CrowdCulling, u32),
    Cpu,
}

/// What [`ScatterRenderer::prepare`] answers: the scatters' batches, the
/// model impostors' and the crowds' cards.
#[derive(Default)]
pub(crate) struct ScatterBatches {
    pub scatters: Vec<ScatterBatch>,
    pub impostors: Vec<ImpostorBatch>,
    pub cards: Vec<CardBatch>,
}

impl ScatterBatches {
    /// Whether any copy, model impostor or crowd card this frame casts a shadow.
    pub(crate) fn casts(&self) -> bool {
        self.scatters.iter().any(|b| b.cast_shadows)
            || self.impostors.iter().any(|b| b.cast_shadows)
            || self.cards.iter().any(|b| b.input.cast_shadows)
    }
}

/// Which pass a scatter draws into.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScatterPass {
    Lit,
    Prepass,
    Shadow,
}

/// What [`ScatterRenderer::new`] builds its pipelines from.
pub(crate) struct ScatterPipelines<'a> {
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub material_layout: &'a wgpu::BindGroupLayout,
    pub shadow_layout: &'a wgpu::BindGroupLayout,
    /// The lit passes' effect group (the screen-space AO), group 3 of the lit pipelines.
    pub fx_layout: &'a wgpu::BindGroupLayout,
    /// The lit module's source with a material shader's hooks.
    pub lit_wgsl: Box<dyn Fn(&str) -> String + Send + Sync>,
    pub shadow_wgsl: String,
    pub samples: u32,
    /// The wind field's lean (`wind_field.rs`) the copies and cards sway by.
    pub wind_field: wgpu::TextureView,
    /// The trample map (`trample.rs`) they part by.
    pub trample: wgpu::TextureView,
    /// Whether compute runs: the main camera's copies are culled on the GPU.
    pub compute: bool,
}

/// Draws [`ScatterDraw`]s from cached chunks.
pub(crate) struct ScatterRenderer {
    lit_layout: wgpu::PipelineLayout,
    /// The lit pass's: `lit_layout` plus the effect group (SSAO).
    lit_fx_layout: wgpu::PipelineLayout,
    shadow_layout: wgpu::PipelineLayout,
    lit_module: wgpu::ShaderModule,
    shadow_module: wgpu::ShaderModule,
    /// The lit module with each material shader's hooks (the scene's
    /// shader index), made when a scatter's material first uses it.
    hooked: HashMap<usize, wgpu::ShaderModule>,
    lit_wgsl: Box<dyn Fn(&str) -> String + Send + Sync>,
    samples: u32,
    pipelines: HashMap<(Option<usize>, Side, u8), wgpu::RenderPipeline>,
    /// The shadow kinds' pipelines, one per target (shadow_target.rs).
    shadow_pipelines: HashMap<(Option<usize>, Side, u8), crate::ShadowSet>,
    uniform_layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    /// The blue-noise tile ([`crate::blue_noise`]) the mesh-to-impostor
    /// crossfade thresholds its pixels by, its bytes packed into a uniform
    /// array (group 2 binding 1): a texture there would take the lit pass's
    /// fragment stage past its 16 sampled textures.
    blue_noise: wgpu::Buffer,
    slots: u64,
    group: wgpu::BindGroup,
    chunks: HashMap<ChunkKey, Chunk>,
    frame: u64,
    /// Chunks built for the last frame.
    pub built: u32,
    /// The tick the last frame drew (the analytic wind's previous phases).
    last_tick: Option<u64>,
    /// The frame's og.Wind (`WindField`, group 2 binding 2).
    wind_uniform: wgpu::Buffer,
    /// The wind field's lean (`wind_field.rs`) and the trample map.
    wind_field: wgpu::TextureView,
    trample: wgpu::TextureView,
    /// The frame's model impostors' transforms, and how many it holds.
    impostor_instances: Option<(wgpu::Buffer, u64)>,
    /// A card's quad (6 indices) and the CPU path's card records, with
    /// their capacity in bytes.
    card_quad: wgpu::Buffer,
    card_instances: Option<(wgpu::Buffer, u64)>,
    /// The GPU cull (with compute) and this frame's culled chunks, in order.
    cull: Option<crate::scatter_cull::ScatterCull>,
    culled_chunks: Vec<ChunkKey>,
}

impl ScatterRenderer {
    /// Draw the lit pass into a target of `samples` per pixel from now on:
    /// its pipelines are made again as they are next used.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        if self.samples != samples {
            self.samples = samples;
            self.pipelines.clear();
        }
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: ScatterPipelines,
    ) -> ScatterRenderer {
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ScatterGpu>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            crate::blue_noise::BLUE_NOISE.len() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                            crate::wind::WindFieldGpu,
                        >() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let lit_layout = pl(
            "scatter",
            &[p.frame_layout, p.material_layout, &uniform_layout],
        );
        let lit_fx_layout = pl(
            "scatter lit",
            &[
                p.frame_layout,
                p.material_layout,
                &uniform_layout,
                p.fx_layout,
            ],
        );
        let shadow_layout = pl(
            "scatter shadow",
            &[p.shadow_layout, p.material_layout, &uniform_layout],
        );
        let lit_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter"),
            source: wgpu::ShaderSource::Wgsl((p.lit_wgsl)(&crate::hooks::default_hooks()).into()),
        });
        let shadow_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter shadow"),
            source: wgpu::ShaderSource::Wgsl(p.shadow_wgsl.into()),
        });
        let wind_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scatter wind"),
            size: std::mem::size_of::<crate::wind::WindFieldGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let blue_noise = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("scatter blue noise"),
                contents: crate::blue_noise::BLUE_NOISE,
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let (uniform, group) = Self::slots(
            device,
            &uniform_layout,
            [&blue_noise, &wind_uniform],
            [&p.wind_field, &p.trample],
            4,
        );
        ScatterRenderer {
            lit_layout,
            lit_fx_layout,
            shadow_layout,
            lit_module,
            shadow_module,
            hooked: HashMap::new(),
            lit_wgsl: p.lit_wgsl,
            samples: p.samples,
            pipelines: HashMap::new(),
            shadow_pipelines: HashMap::new(),
            uniform_layout,
            uniform,
            blue_noise,
            slots: 4,
            group,
            chunks: HashMap::new(),
            frame: 0,
            built: 0,
            last_tick: None,
            wind_uniform,
            wind_field: p.wind_field,
            trample: p.trample,
            impostor_instances: None,
            card_quad: crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("crowd card quad"),
                    contents: bytemuck::cast_slice(&[0u32, 1, 2, 3, 4, 5]),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ),
            card_instances: None,
            cull: p
                .compute
                .then(|| crate::scatter_cull::ScatterCull::new(device, queue)),
            culled_chunks: Vec::new(),
        }
    }

    fn slots(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        [blue_noise, wind]: [&wgpu::Buffer; 2],
        [wind_field, trample]: [&wgpu::TextureView; 2],
        n: u64,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scatter"),
            size: SLOT * n,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: blue_noise.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wind.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(wind_field),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(trample),
                },
            ],
        });
        (buffer, group)
    }

    fn pipeline(
        &mut self,
        device: &wgpu::Device,
        shader: Option<usize>,
        side: Side,
        pass: ScatterPass,
        impostor: bool,
    ) {
        let kind = pass as u8 + if impostor { IMPOSTOR_KIND } else { 0 };
        // The shadow pass reads no material shader (copies cast as cut by the material's alpha).
        let shader = shader.filter(|_| pass != ScatterPass::Shadow);
        let key = if impostor {
            (None, Side::Double, kind)
        } else {
            (shader, side, kind)
        };
        if self.pipelines.contains_key(&key) || self.shadow_pipelines.contains_key(&key) {
            return;
        }
        if pass == ScatterPass::Shadow {
            let set = crate::ShadowSet::new(|t| {
                if impostor {
                    self.impostor_pipeline(device, pass, t)
                } else {
                    self.mesh_shadow_pipeline(device, side, t)
                }
            });
            self.shadow_pipelines.insert(key, set);
            return;
        }
        if impostor {
            let p = self.impostor_pipeline(device, pass, crate::ShadowTarget::Cascade);
            self.pipelines.insert(key, p);
            return;
        }
        let lit_module = match shader {
            Some(sh) => &self.hooked[&sh],
            None => &self.lit_module,
        };
        let buffers = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        let depth = |write| {
            Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(write),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            })
        };
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let p = match pass {
            ScatterPass::Lit => crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    // opaque copies write alpha 1 (every sample); alpha-to-coverage materials their coverage
                    alpha_to_coverage: true,
                    label: "scatter",
                    layout: &self.lit_fx_layout,
                    module: lit_module,
                    vs: "vs",
                    fs: "fs",
                    buffers: &buffers,
                    cull: crate::cull_mode(side),
                    depth_write: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    blend: None,
                    samples: self.samples,
                },
            ),
            ScatterPass::Prepass => {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("scatter prepass"),
                    layout: Some(&self.lit_layout),
                    vertex: wgpu::VertexState {
                        module: lit_module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &buffers,
                    },
                    primitive: wgpu::PrimitiveState {
                        cull_mode: crate::cull_mode(side),
                        ..Default::default()
                    },
                    depth_stencil: depth(true),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: lit_module,
                        entry_point: Some("fs_prepass"),
                        compilation_options: Default::default(),
                        targets: &[
                            target(crate::post::GBUFFER_FORMAT),
                            target(crate::post::LINDEPTH_FORMAT),
                            target(crate::post::OUTLINE_FORMAT),
                        ],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                })
            }
            ScatterPass::Shadow => unreachable!("made by mesh_shadow_pipeline"),
        };
        self.pipelines.insert(key, p);
    }

    /// A scattered mesh's shadow pipeline into `target`: as shadow.wgsl's
    /// casters (shadow_target.rs).
    fn mesh_shadow_pipeline(
        &self,
        device: &wgpu::Device,
        side: Side,
        target: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scatter shadow"),
            layout: Some(&self.shadow_layout),
            vertex: wgpu::VertexState {
                module: &self.shadow_module,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: target.cull(side),
                ..Default::default()
            },
            depth_stencil: Some(target.depth()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &self.shadow_module,
                entry_point: Some("fs_shadow"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: crate::ShadowTarget::constants(side),
                    ..Default::default()
                },
                targets: &[Some(target.color())],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// An impostor pipeline (`impostor.wgsl`): the card has no vertex
    /// buffer but the instances' (slot 0), and both faces drawn.
    fn impostor_pipeline(
        &self,
        device: &wgpu::Device,
        pass: ScatterPass,
        // The shadow pass's (shadow_target.rs); the others ignore it.
        shadow: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(crate::INSTANCE_LAYOUT)];
        let depth = Some(if pass == ScatterPass::Shadow {
            shadow.depth()
        } else {
            wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let (layout, module, vs, fs, targets): (_, _, _, _, Vec<_>) = match pass {
            ScatterPass::Lit => {
                return crate::hdr_pipeline(
                    device,
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "impostor",
                        layout: &self.lit_fx_layout,
                        module: &self.lit_module,
                        vs: "vs_impostor",
                        fs: "fs_impostor",
                        buffers: &buffers,
                        cull: None,
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples: self.samples,
                    },
                );
            }
            ScatterPass::Prepass => (
                &self.lit_layout,
                &self.lit_module,
                "vs_impostor",
                "fs_impostor_prepass",
                vec![
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                ],
            ),
            ScatterPass::Shadow => (
                &self.shadow_layout,
                &self.shadow_module,
                "vs_impostor_shadow",
                "fs_impostor_shadow",
                vec![Some(shadow.color())],
            ),
        };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("impostor"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(vs),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: Default::default(),
            depth_stencil: depth,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fs),
                // Both faces cast (shadow_target.rs).
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: if pass == ScatterPass::Shadow {
                        crate::ShadowTarget::constants(Side::Double)
                    } else {
                        &[]
                    },
                    ..Default::default()
                },
                targets: &targets,
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// A crowd card pipeline (`crowd_card.wgsl`): instance records of
    /// [`CARD_LAYOUT`] (slot 0), indexed by the card quad, both faces.
    fn card_pipeline(
        &self,
        device: &wgpu::Device,
        pass: ScatterPass,
        // The shadow pass's (shadow_target.rs); the others ignore it.
        shadow: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(CARD_LAYOUT)];
        let depth = Some(if pass == ScatterPass::Shadow {
            shadow.depth()
        } else {
            wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let (layout, module, vs, fs, targets): (_, _, _, _, Vec<_>) = match pass {
            ScatterPass::Lit => {
                return crate::hdr_pipeline(
                    device,
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "crowd card",
                        layout: &self.lit_fx_layout,
                        module: &self.lit_module,
                        vs: "vs_crowd_card",
                        fs: "fs_crowd_card",
                        buffers: &buffers,
                        cull: None,
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples: self.samples,
                    },
                );
            }
            ScatterPass::Prepass => (
                &self.lit_layout,
                &self.lit_module,
                "vs_crowd_card",
                "fs_crowd_card_prepass",
                vec![
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                ],
            ),
            ScatterPass::Shadow => (
                &self.shadow_layout,
                &self.shadow_module,
                "vs_crowd_card_shadow",
                "fs_crowd_card_shadow",
                vec![Some(shadow.color())],
            ),
        };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("crowd card"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(vs),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: Default::default(),
            depth_stencil: depth,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fs),
                // Both faces cast (shadow_target.rs).
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: if pass == ScatterPass::Shadow {
                        crate::ShadowTarget::constants(Side::Double)
                    } else {
                        &[]
                    },
                    ..Default::default()
                },
                targets: &targets,
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// Groups the scene's model impostors by atlas, uploads their
    /// transforms and writes each group's uniform at the slots after the
    /// scatters'.
    #[allow(clippy::too_many_arguments)]
    fn prepare_impostors(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        base: ScatterGpu,
        groups: &[(crate::Impostor, Vec<usize>)],
        first_slot: usize,
    ) -> Vec<ImpostorBatch> {
        let total: usize = groups.iter().map(|g| g.1.len()).sum();
        if total == 0 {
            return Vec::new();
        }
        let size = (total * std::mem::size_of::<crate::instances::GpuTransform>()) as u64;
        if self.impostor_instances.as_ref().is_none_or(|b| b.1 < size) {
            let cap = size.next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("impostor instances"),
                size: cap,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.impostor_instances = Some((buffer, cap));
        }
        let mut transforms = Vec::with_capacity(total);
        let mut out = Vec::with_capacity(groups.len());
        let light = scene.light.direction_to_light.normalize_or(Vec3::Y);
        for (k, (imp, members)) in groups.iter().enumerate() {
            let first = transforms.len() as u32;
            let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
            let mut grow = 0.0f32;
            let mut cast = false;
            for &i in members {
                let d = &scene.impostors[i];
                let c = d.transform.transform_point3(imp.center);
                let s = d
                    .transform
                    .to_scale_rotation_translation()
                    .0
                    .abs()
                    .max_element();
                lo = lo.min(c);
                hi = hi.max(c);
                grow = grow.max(imp.radius * s);
                cast |= d.cast_shadows;
                transforms.push(crate::instances::gpu_transform(&d.transform));
            }
            let slot = first_slot + k;
            let mut gpu = base;
            gpu.impostor(imp, 0.0, light, self.samples > 1);
            queue.write_buffer(&self.uniform, slot as u64 * SLOT, bytemuck::bytes_of(&gpu));
            out.push(ImpostorBatch {
                material: imp.material.index(),
                cast_shadows: cast,
                offset: (slot as u64 * SLOT) as u32,
                first,
                count: members.len() as u32,
                center: (lo + hi) * 0.5,
                radius: (hi - lo).length() * 0.5 + grow,
            });
        }
        if let Some((buffer, _)) = &self.impostor_instances {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&transforms));
        }
        for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
            self.pipeline(device, None, Side::Double, pass, true);
        }
        out
    }

    /// Builds the chunks this frame needs and answers what to draw.
    /// `side_of` answers a material's side (`None`: not of this scene);
    /// `mesh_bounds` a mesh's radius about its origin, its top above it
    /// and its index count (`None`: no geometry).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        camera: Vec3,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        frame_index: u64,
        (wind_grid, lean_cap, trample): (crate::wind_field::WindGrid, f32, [f32; 4]),
        side_of: impl Fn(usize) -> Option<Side>,
        mesh_bounds: impl Fn(usize) -> Option<(f32, f32, u32)>,
        shader_of: impl Fn(usize) -> Option<usize>,
        hooks_of: impl Fn(usize) -> String,
        (card_inputs, cpu_cards): (&[CardInput], &[crate::skin::SkinnedInstanceGpu]),
    ) -> ScatterBatches {
        self.frame += 1;
        self.built = 0;
        let noise = crossfade_noise_offset(frame_index);
        let mut batches = Vec::new();
        let wind = scene.wind.filter(|w| w.strength > 0.0);
        let previous = self.last_tick.unwrap_or(scene.tick);
        self.last_tick = Some(scene.tick);
        queue.write_buffer(
            &self.wind_uniform,
            0,
            bytemuck::bytes_of(&crate::wind::field_gpu(
                wind,
                scene.tick,
                previous,
                scene.tick_rate,
                scene.origin,
                wind_grid,
            )),
        );
        // How long trampled copies take to stand again: the scene's first
        // grass field's (`trample.rs`), else its default.
        let recovery = scene
            .grass
            .first()
            .map_or(crate::Grass::default().recovery, |g| g.grass.recovery);
        // `lean_cap`: the most a copy leans this frame (wind_field.rs; its
        // chunks' bounds widen by its tip's reach).
        // The model impostors by atlas, in the order each first appears.
        let mut groups: Vec<(crate::Impostor, Vec<usize>)> = Vec::new();
        for (i, d) in scene.impostors.iter().enumerate() {
            match groups.iter_mut().find(|g| g.0 == d.impostor) {
                Some(g) => g.1.push(i),
                None => groups.push((d.impostor, vec![i])),
            }
        }
        let slots = (scene.scatters.len() + groups.len() + card_inputs.len()) as u64;
        if self.slots < slots {
            self.slots = slots.next_power_of_two();
            let (u, g) = Self::slots(
                device,
                &self.uniform_layout,
                [&self.blue_noise, &self.wind_uniform],
                [&self.wind_field, &self.trample],
                self.slots,
            );
            self.uniform = u;
            self.group = g;
        }
        // A model impostor is drawn at any distance (the level pick
        // decides), never faded or swayed.
        let base = ScatterGpu {
            camera: camera.extend(f32::MAX).to_array(),
            fade: [f32::MAX, noise, 0.0, 0.0],
            view_proj: view_proj.to_cols_array_2d(),
            prev_view_proj: prev_view_proj.to_cols_array_2d(),
            wind: [0.0; 4],
            wind_shape: [0.0, 0.0, scene.origin.x, scene.origin.z],
            impostor: [0.0; 4],
            impostor_grid: [0.0; 4],
            impostor_light: [0.0; 4],
            trample: [0.0; 4],
        };
        let impostors =
            self.prepare_impostors(device, queue, scene, base, &groups, scene.scatters.len());
        let light = scene.light.direction_to_light.normalize_or(Vec3::Y);
        let cards = self.prepare_cards(
            device,
            queue,
            base,
            light,
            card_inputs,
            cpu_cards,
            scene.scatters.len() + groups.len(),
        );
        self.culled_chunks.clear();
        let mut cull_items = Vec::new();
        // Per batch, per chunk, its item among `cull_items`.
        let mut planned: Vec<Vec<Option<usize>>> = Vec::new();
        let view = crate::frame::Frustum::from_view_proj(view_proj);
        if scene.scatters.is_empty() {
            if let Some(cull) = &mut self.cull {
                cull.prepare(device, queue, &[]);
            }
            self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
            return ScatterBatches {
                scatters: batches,
                impostors,
                cards,
            };
        }
        let (sources, revisions) = scene.terrain_sources();
        let mut budget = CHUNKS_PER_FRAME;
        // Each scatter's terrain draw and parameters: what places before it.
        let placing: Vec<(usize, TerrainScatter)> = scene
            .scatters
            .iter()
            .map(|d| (d.terrain, d.scatter))
            .collect();
        for (si, d) in scene.scatters.iter().enumerate() {
            let Some(t) = scene.terrains.get(d.terrain) else {
                continue;
            };
            let di = t.terrain.index();
            let (Some(source), Some(side), Some((mesh_r, mesh_top, index_count))) = (
                sources.get(di),
                side_of(d.material.index()),
                mesh_bounds(d.mesh.index()),
            ) else {
                continue;
            };
            let s = d.scatter;
            if !s.usable(t.layers.len()) {
                continue;
            }
            let layout = match source.layout() {
                Ok(l) => l,
                Err(_) => continue,
            };
            let model = t.transform;
            let along = horizontal_scale(model);
            let local = model.inverse().transform_point3(camera);
            let eye = Vec2::new(local.x, local.z);
            let reach = s.distance / along;
            let cell = s.cell();
            let span = cell * CHUNK_CELLS as f32;
            let half = layout.size * 0.5;
            let cells = (layout.size / cell).ceil();
            let (nx, nz) = (
                (cells.x as u32).div_ceil(CHUNK_CELLS),
                (cells.y as u32).div_ceil(CHUNK_CELLS),
            );
            let lo = ((eye - reach + half) / span).floor().max(Vec2::ZERO);
            let hi = ((eye + reach + half) / span).ceil();
            let (x0, z0) = (lo.x as u32, lo.y as u32);
            let (x1, z1) = (
                (hi.x.max(0.0) as u32).min(nx),
                (hi.y.max(0.0) as u32).min(nz),
            );
            let ctx = ScatterContext::of(
                &placing,
                si,
                t.layers.len(),
                scene.water.iter().map(|w| {
                    let mut m = w.transform;
                    m.w_axis += scene.origin.extend(0.0);
                    (m, w.size, w.endless)
                }),
            );
            let mut params: Vec<u32> = std::iter::once(&s)
                .chain(&ctx.before)
                .flat_map(|o| o.placement_bits())
                .collect();
            if ctx.reads_water(&s) {
                for (m, half) in &ctx.water {
                    params.extend(m.to_cols_array().map(f32::to_bits));
                    params.extend(half.to_array().map(f32::to_bits));
                }
            }
            // The copies are placed in the world (the terrain's placement
            // plus the scene's origin: exact, as the origin is on a coarse
            // grid) and then moved by the origin, so each copy's place, and
            // the wind's phase hashed from its bits, is the same bits
            // wherever the origin is (`origin.rs`).
            let world = scene.origin != Vec3::ZERO;
            let placed_on = if world {
                let mut w = t.clone();
                w.transform.w_axis += scene.origin.extend(0.0);
                std::borrow::Cow::Owned(w)
            } else {
                std::borrow::Cow::Borrowed(t)
            };
            let transform = placed_on.transform.to_cols_array().map(f32::to_bits);
            let origin = scene.origin.to_array().map(f32::to_bits);
            let layers: Vec<u32> = t
                .layers
                .iter()
                .flat_map(|l| {
                    [
                        l.slope[0],
                        l.slope[1],
                        l.slope_fade,
                        l.altitude[0],
                        l.altitude[1],
                        l.altitude_fade,
                    ]
                })
                .map(f32::to_bits)
                .collect();
            let settle = !t.fade_in;
            let streamed = matches!(source, TerrainSource::Stream { .. });
            let tiles = match source {
                TerrainSource::Stream { tiles, .. } => Some(tiles),
                TerrainSource::Data(..) => None,
            };
            let mut keys = Vec::new();
            // Nearest chunks first, so a budgeted frame builds those.
            let mut order: Vec<(u32, u32)> = (z0..z1)
                .flat_map(|z| (x0..x1).map(move |x| (x, z)))
                .collect();
            let centre = |(x, z): (u32, u32)| {
                (Vec2::new(x as f32 + 0.5, z as f32 + 0.5) * span - half - eye).length()
            };
            og_core::sort::sort_by(&mut order, |a, b| {
                centre(*a).total_cmp(&centre(*b)).then(a.cmp(b))
            });
            for (cx, cz) in order {
                // The chunk's rectangle in the terrain's space: past the reach, nothing.
                let a = Vec2::new(cx as f32, cz as f32) * span - half;
                let b = a + span;
                if (eye.clamp(a, b) - eye).length() > reach {
                    continue;
                }
                let under: Vec<usize> = match tiles {
                    None => Vec::new(),
                    Some(tiles) => level0_under(&layout, a, b)
                        .into_iter()
                        .map(|i| tiles.get(&i).map_or(0, |t| Arc::as_ptr(t) as usize))
                        .collect(),
                };
                if under.contains(&0) {
                    continue;
                }
                let key = ChunkKey {
                    data: di,
                    source: (revisions[di], under),
                    params: params.clone(),
                    transform,
                    origin,
                    layers: layers.clone(),
                    mesh: d.mesh.index(),
                    chunk: (cx, cz),
                };
                if let Some(c) = self.chunks.get_mut(&key) {
                    c.used = self.frame;
                    keys.push(key);
                    continue;
                }
                if streamed && !settle && budget == 0 {
                    continue;
                }
                let mut placed = Vec::new();
                let mut complete = true;
                let tile_ground;
                let ground: &dyn Ground = match source {
                    TerrainSource::Data(d, ..) => d,
                    TerrainSource::Stream { tiles, .. } => {
                        tile_ground = TileGround { layout, tiles };
                        &tile_ground
                    }
                };
                'cells: for z in cz * CHUNK_CELLS..((cz + 1) * CHUNK_CELLS).min(cells.y as u32) {
                    for x in cx * CHUNK_CELLS..((cx + 1) * CHUNK_CELLS).min(cells.x as u32) {
                        match placed_on.copy_ruled(ground, &s, &ctx.before, &ctx.water, x, z) {
                            None => {
                                complete = false;
                                break 'cells;
                            }
                            Some(Some((_, mut m, _))) => {
                                if world {
                                    m.w_axis -= scene.origin.extend(0.0);
                                }
                                placed.push(m)
                            }
                            Some(None) => {}
                        }
                    }
                }
                if !complete {
                    continue;
                }
                budget = budget.saturating_sub(1);
                self.built += 1;
                let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for m in &placed {
                    let p = m.w_axis.truncate();
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
                let grow = mesh_r * s.scale[1].max(s.scale[0]);
                let (center, radius) = if placed.is_empty() {
                    (Vec3::ZERO, 0.0)
                } else {
                    ((lo + hi) * 0.5, (hi - lo).length() * 0.5 + grow)
                };
                let transforms: Vec<crate::instances::GpuTransform> =
                    placed.iter().map(crate::instances::gpu_transform).collect();
                let buffer = (!transforms.is_empty()).then(|| {
                    crate::buffer_init(
                        device,
                        queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("scatter chunk"),
                            contents: bytemuck::cast_slice(&transforms),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                        },
                    )
                });
                let cull_group = self
                    .cull
                    .as_ref()
                    .zip(buffer.as_ref())
                    .map(|(cull, b)| cull.chunk_group(device, b));
                self.chunks.insert(
                    key.clone(),
                    Chunk {
                        buffer,
                        cull_group,
                        count: transforms.len() as u32,
                        center,
                        radius,
                        used: self.frame,
                    },
                );
                keys.push(key);
            }
            // The wind: its lean on this scatter, and the furthest a copy
            // moves (the tip of the tallest at the top of a gust).
            // A trample map parts the copies by up to 1.2 of their height
            // (scatter_common.wgsl scatter_sway), wind or not.
            let parting = if trample[3] > 0.0 { 1.2 } else { 0.0 };
            let moves = lean_cap > 0.0 || parting > 0.0;
            let (wind_row, sway) = if s.sway > 0.0 && mesh_top > 0.0 && moves {
                let d = wind.map_or(Vec3::X, |w| w.direction);
                let dir = Vec2::new(d.x, d.z).normalize_or(Vec2::X);
                let tip = (lean_cap * s.sway + parting) * mesh_top * s.scale[1].max(s.scale[0]);
                ([dir.x, dir.y, s.sway, recovery], tip)
            } else {
                ([0.0; 4], 0.0)
            };
            let mut gpu = ScatterGpu {
                camera: camera.extend(s.distance).to_array(),
                fade: [s.fade_from(), noise, 0.0, 0.0],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                wind: wind_row,
                wind_shape: [0.0, mesh_top, scene.origin.x, scene.origin.z],
                impostor: [0.0; 4],
                impostor_grid: [0.0; 4],
                impostor_light: [0.0; 4],
                trample,
            };
            let impostor = d.impostor.filter(|_| d.impostor_distance > 0.0);
            if let Some(i) = &impostor {
                gpu.impostor(i, d.impostor_distance, light, self.samples > 1);
                for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
                    self.pipeline(device, None, Side::Double, pass, true);
                }
            }
            queue.write_buffer(&self.uniform, si as u64 * SLOT, bytemuck::bytes_of(&gpu));
            let shader = shader_of(d.material.index());
            if let Some(sh) = shader
                && !self.hooked.contains_key(&sh)
            {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("scatter (material shader)"),
                    source: wgpu::ShaderSource::Wgsl((self.lit_wgsl)(&hooks_of(sh)).into()),
                });
                self.hooked.insert(sh, module);
            }
            for pass in [ScatterPass::Lit, ScatterPass::Prepass, ScatterPass::Shadow] {
                self.pipeline(device, shader, side, pass, false);
            }
            // The chunks the camera may see, culled copy by copy on the GPU.
            let culled = keys
                .iter()
                .map(|k| {
                    let c = self.chunks.get(k)?;
                    c.cull_group.as_ref()?;
                    if !view.intersects_sphere(c.center, c.radius + sway) {
                        return None;
                    }
                    self.culled_chunks.push(k.clone());
                    cull_items.push(crate::scatter_cull::CullItem {
                        to_terrain: t.transform.inverse(),
                        extent: layout.size,
                        copies: c.count,
                        index_count,
                        radius: mesh_r,
                        moves: sway,
                        distance: s.distance,
                        density_map: s.density_map,
                    });
                    Some(cull_items.len() - 1)
                })
                .collect::<Vec<_>>();
            planned.push(culled);
            batches.push(ScatterBatch {
                culled: Vec::new(),
                mesh: d.mesh.index(),
                material: d.material.index(),
                shader,
                cast_shadows: d.cast_shadows,
                side,
                offset: (si as u64 * SLOT) as u32,
                chunks: keys,
                sway,
                impostor: impostor.map(|i| i.material.index()),
            });
        }
        self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
        // Where each culled chunk draws from, in the order planned.
        if let Some(cull) = &mut self.cull {
            let places = cull.prepare(device, queue, &cull_items);
            for (b, items) in batches.iter_mut().zip(planned) {
                b.culled = items.into_iter().map(|i| i.map(|i| places[i])).collect();
            }
        }
        ScatterBatches {
            scatters: batches,
            impostors,
            cards,
        }
    }

    /// The crowds' cards: each its uniform at the slots from `first_slot`
    /// (its sphere, views, frames and elevation), the CPU path's records
    /// uploaded, and their pipelines made.
    #[allow(clippy::too_many_arguments)]
    fn prepare_cards(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base: ScatterGpu,
        light: Vec3,
        inputs: &[CardInput],
        cpu: &[crate::skin::SkinnedInstanceGpu],
        first_slot: usize,
    ) -> Vec<CardBatch> {
        if inputs.is_empty() {
            return Vec::new();
        }
        if !cpu.is_empty() {
            let size = std::mem::size_of_val(cpu) as u64;
            if self.card_instances.as_ref().is_none_or(|b| b.1 < size) {
                let cap = size.next_power_of_two();
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("crowd card records"),
                    size: cap,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.card_instances = Some((buffer, cap));
            }
            let (buffer, _) = self.card_instances.as_ref().expect("made above");
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(cpu));
        }
        let out = inputs
            .iter()
            .enumerate()
            .map(|(k, c)| {
                let slot = first_slot + k;
                let mut gpu = base;
                gpu.impostor = c.center.extend(c.radius).to_array();
                gpu.impostor_grid = [0.0, 0.0, f32::from(u8::from(self.samples > 1)), 0.0];
                gpu.fade[2] = c.views as f32;
                gpu.fade[3] = c.frames as f32;
                gpu.impostor_light = light.extend(c.elevation).to_array();
                queue.write_buffer(&self.uniform, slot as u64 * SLOT, bytemuck::bytes_of(&gpu));
                CardBatch {
                    input: *c,
                    offset: (slot as u64 * SLOT) as u32,
                    group: None,
                }
            })
            .collect();
        for pass in [ScatterPass::Lit, ScatterPass::Prepass] {
            let kind = pass as u8 + CARD_KIND;
            if !self.pipelines.contains_key(&(None, Side::Double, kind)) {
                let p = self.card_pipeline(device, pass, crate::ShadowTarget::Cascade);
                self.pipelines.insert((None, Side::Double, kind), p);
            }
        }
        let key = (None, Side::Double, ScatterPass::Shadow as u8 + CARD_KIND);
        if !self.shadow_pipelines.contains_key(&key) {
            let set = crate::ShadowSet::new(|t| self.card_pipeline(device, ScatterPass::Shadow, t));
            self.shadow_pipelines.insert(key, set);
        }
        out
    }

    /// Culls this frame's chunks for the main camera on the GPU (before
    /// its prepass and lit pass; nothing without compute).
    pub(crate) fn encode<'t>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        view: &crate::scatter_cull::CullView,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) {
        let Some(cull) = &self.cull else { return };
        let groups: Vec<&wgpu::BindGroup> = self
            .culled_chunks
            .iter()
            .filter_map(|k| self.chunks.get(k)?.cull_group.as_ref())
            .collect();
        debug_assert_eq!(groups.len(), self.culled_chunks.len());
        cull.encode(device, queue, encoder, view, &groups, textures);
    }

    /// Draws `batches` into a pass (groups 0 and, for the lit passes, the
    /// frame bound by the caller): the chunks whose bounds `visible` keeps,
    /// with their copies' impostors, then the model impostors; `camera`:
    /// the main camera's pass, which draws what the GPU cull kept. Answers
    /// the draws and the copies in the chunks drawn.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw<'a>(
        &self,
        pass: &mut wgpu::RenderPass,
        kind: ScatterPass,
        all: &ScatterBatches,
        material_group: impl Fn(usize) -> Option<&'a wgpu::BindGroup>,
        mesh: impl Fn(usize) -> Option<(&'a wgpu::Buffer, &'a wgpu::Buffer, u32)>,
        visible: impl Fn(Vec3, f32) -> bool,
        mut probes: Option<&mut crate::probe_select::Binder>,
        fx_group: Option<&wgpu::BindGroup>,
        camera: bool,
        cards: CardSource<'_>,
        // What a shadow pass draws into (shadow_target.rs).
        target: crate::ShadowTarget,
    ) -> (u32, u32) {
        let pipeline = |key: &(Option<usize>, Side, u8)| match kind {
            ScatterPass::Shadow => self.shadow_pipelines.get(key).map(|s| s.get(target)),
            _ => self.pipelines.get(key),
        };
        // The main camera's prepass and lit pass draw the GPU cull's copies.
        let list = self
            .cull
            .as_ref()
            .filter(|_| camera && kind != ScatterPass::Shadow);
        let (mut draws, mut copies) = (0, 0);
        let bind_rest = |pass: &mut wgpu::RenderPass, group, offset| {
            pass.set_bind_group(1, group, &[]);
            pass.set_bind_group(2, &self.group, &[offset]);
            if kind == ScatterPass::Lit
                && let Some(fx) = fx_group
            {
                pass.set_bind_group(3, fx, &[]);
            }
        };
        let impostor_pipeline = pipeline(&(None, Side::Double, kind as u8 + IMPOSTOR_KIND));
        for b in &all.scatters {
            if kind == ScatterPass::Shadow && !b.cast_shadows {
                continue;
            }
            let shader = b.shader.filter(|_| kind != ScatterPass::Shadow);
            let (Some(group), Some((vertices, indices, count)), Some(p)) = (
                material_group(b.material),
                mesh(b.mesh),
                pipeline(&(shader, b.side, kind as u8)),
            ) else {
                continue;
            };
            let mut bound = false;
            for (n, k) in b.chunks.iter().enumerate() {
                let Some(c) = self.chunks.get(k) else {
                    continue;
                };
                let Some(buffer) = &c.buffer else { continue };
                if !visible(c.center, c.radius + b.sway) {
                    continue;
                }
                let culled = list.zip(b.culled.get(n).copied().flatten());
                if !bound {
                    pass.set_pipeline(p);
                    bind_rest(pass, group, b.offset);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    bound = true;
                }
                // Each chunk's reflection probe (probe_select.rs).
                if let Some(b) = probes.as_deref_mut() {
                    b.at(pass, c.center, c.radius);
                }
                match culled {
                    Some((cull, at)) => {
                        pass.set_vertex_buffer(1, cull.list.slice(at.list_offset()..));
                        pass.draw_indexed_indirect(&cull.draws, at.mesh_draw());
                    }
                    None => {
                        pass.set_vertex_buffer(1, buffer.slice(..));
                        pass.draw_indexed(0..count, 0, 0..c.count);
                    }
                }
                draws += 1;
                copies += c.count;
            }
            // The same chunks' far copies as the impostor: the vertex
            // stages split each copy between the two by its distance.
            let (Some(m), Some(ip)) = (b.impostor, impostor_pipeline) else {
                continue;
            };
            let Some(group) = material_group(m) else {
                continue;
            };
            let mut bound = false;
            for (n, k) in b.chunks.iter().enumerate() {
                let Some(c) = self.chunks.get(k) else {
                    continue;
                };
                let Some(buffer) = &c.buffer else { continue };
                if !visible(c.center, c.radius) {
                    continue;
                }
                let culled = list.zip(b.culled.get(n).copied().flatten());
                if !bound {
                    pass.set_pipeline(ip);
                    bind_rest(pass, group, b.offset);
                    bound = true;
                }
                if let Some(b) = probes.as_deref_mut() {
                    b.at(pass, c.center, c.radius);
                }
                match culled {
                    Some((cull, at)) => {
                        pass.set_vertex_buffer(0, cull.list.slice(at.list_offset()..));
                        pass.draw_indirect(&cull.draws, at.card_draw());
                    }
                    None => {
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        pass.draw(0..6, 0..c.count);
                    }
                }
                draws += 1;
            }
        }
        // The model impostors, an instanced card per batch.
        if let (Some(ip), Some((buffer, _))) = (impostor_pipeline, &self.impostor_instances) {
            for b in &all.impostors {
                if kind == ScatterPass::Shadow && !b.cast_shadows {
                    continue;
                }
                let Some(group) = material_group(b.material) else {
                    continue;
                };
                if !visible(b.center, b.radius) {
                    continue;
                }
                pass.set_pipeline(ip);
                bind_rest(pass, group, b.offset);
                if let Some(p) = probes.as_deref_mut() {
                    p.at(pass, b.center, b.radius);
                }
                let stride = std::mem::size_of::<crate::instances::GpuTransform>() as u64;
                pass.set_vertex_buffer(0, buffer.slice(b.first as u64 * stride..));
                pass.draw(0..6, 0..b.count);
                draws += 1;
                copies += b.count;
            }
        }
        // The crowds' cards: the cull's row of them, or the CPU path's
        // records drawn whole (the depth test keeps what the cull would).
        let card_pipeline = pipeline(&(None, Side::Double, kind as u8 + CARD_KIND));
        if let Some(cp) = card_pipeline {
            for b in &all.cards {
                if kind == ScatterPass::Shadow && !b.input.cast_shadows {
                    continue;
                }
                let Some(group) = &b.group else { continue };
                let source = match (cards, b.input.cpu, &self.card_instances) {
                    (CardSource::Gpu(cc, row), _, _) => {
                        let at = cc.row_offset(row);
                        Some((
                            cc.records.slice(at..cc.row_offset(row + 1)),
                            Some((cc, row)),
                            0,
                        ))
                    }
                    (CardSource::Cpu, Some((first, count)), Some((buffer, _))) => {
                        let stride = crate::skin::SKINNED_INSTANCE_STRIDE;
                        Some((buffer.slice(first as u64 * stride..), None, count))
                    }
                    _ => None,
                };
                let Some((slice, gpu, count)) = source else {
                    continue;
                };
                pass.set_pipeline(cp);
                bind_rest(pass, group, b.offset);
                pass.set_vertex_buffer(0, slice);
                pass.set_index_buffer(self.card_quad.slice(..), wgpu::IndexFormat::Uint32);
                match gpu {
                    Some((cc, row)) => {
                        pass.draw_indexed_indirect(&cc.args, cc.draw_offset(row, b.input.draw))
                    }
                    None => pass.draw_indexed(0..6, 0, 0..count),
                }
                draws += 1;
            }
        }
        if let Some(b) = probes {
            b.reset(pass);
        }
        (draws, copies)
    }
}

/// The level-0 tiles under a rectangle of the terrain's space.
fn level0_under(layout: &TerrainLayout, a: Vec2, b: Vec2) -> Vec<u32> {
    let half = layout.size * 0.5;
    let size = layout.cell() * layout.tile as f32;
    let (tx, tz) = layout.tiles(0);
    // One sample around, for the normals at the rectangle's edge.
    let pad = layout.cell();
    let lo = ((a - pad + half) / size).floor().max(Vec2::ZERO);
    let hi = ((b + pad + half) / size).floor();
    let mut out = Vec::new();
    for z in (lo.y as u32)..=((hi.y.max(0.0) as u32).min(tz - 1)) {
        for x in (lo.x as u32)..=((hi.x.max(0.0) as u32).min(tx - 1)) {
            out.push(layout.index(0, x, z));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A 40 m terrain of 81 × 81 samples whose height rises along x from
    /// -10 m to +10 m.
    fn ramp() -> (Terrain, TerrainData) {
        let mut data = TerrainData::flat(Vec2::splat(40.0), 81, 81);
        data.heights = Arc::from(
            (0..81 * 81)
                .map(|i| (i % 81) as f32 / 80.0 * 20.0 - 10.0)
                .collect::<Vec<_>>(),
        );
        let mut scene = crate::RenderScene::new();
        let h = scene.add_terrain(data.clone());
        (Terrain::new(h), data)
    }

    fn horizontal(m: &Mat4) -> Vec2 {
        Vec2::new(m.w_axis.x, m.w_axis.z)
    }

    /// A scatter placed after another keeps every copy at least the larger
    /// footprint away from the other's copies (Horizon's placement order);
    /// without footprints copies of the two come closer than that.
    #[test]
    fn footprints_keep_the_later_scatter_clear_of_the_earlier() {
        let (t, data) = ramp();
        let rocks = TerrainScatter {
            density: 0.05,
            distance: 30.0,
            footprint: 1.5,
            priority: 1,
            seed: 3,
            ..Default::default()
        };
        let flowers = TerrainScatter {
            density: 4.0,
            distance: 30.0,
            footprint: 0.25,
            seed: 9,
            ..Default::default()
        };
        let all = [(0, flowers), (0, rocks)];
        let ctx = ScatterContext::of(&all, 0, 1, []);
        assert_eq!(ctx.before, vec![rocks], "the higher priority places first");
        let placed_rocks = t.scatter_among(
            &data,
            &rocks,
            &ScatterContext::of(&all, 1, 1, []),
            Vec3::ZERO,
        );
        let placed = t.scatter_among(&data, &flowers, &ctx, Vec3::ZERO);
        let free = t.scatter(&data, &flowers, Vec3::ZERO);
        let closest = |copies: &[(u32, Mat4)]| {
            copies
                .iter()
                .flat_map(|f| {
                    placed_rocks
                        .iter()
                        .map(move |r| (horizontal(&f.1) - horizontal(&r.1)).length())
                })
                .fold(f32::MAX, f32::min)
        };
        assert!(placed_rocks.len() > 40, "{} rocks", placed_rocks.len());
        assert!(placed.len() > 1000, "{} flowers", placed.len());
        assert!(
            closest(&free) < 0.5,
            "without footprints a flower stands {} from a rock",
            closest(&free)
        );
        assert!(
            closest(&placed) >= 1.5,
            "a flower stands {} from a rock",
            closest(&placed)
        );
        // Only the flowers near rocks go: most of the field stays.
        assert!(
            placed.len() * 2 > free.len(),
            "{} of {} flowers",
            placed.len(),
            free.len()
        );
    }

    /// `avoid_water 2` grows nothing whose ground stands less than 2 m
    /// above a water surface over it, and everything above that; a copy
    /// beside the water's rectangle is not affected.
    #[test]
    fn avoid_water_keeps_copies_above_the_water() {
        let (t, data) = ramp();
        let s = TerrainScatter {
            density: 2.0,
            distance: 30.0,
            avoid_water: 2.0,
            ..Default::default()
        };
        // A pool at height 0 over the -z half (z from -20 to 0).
        let pool = Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0));
        let ctx = ScatterContext::of(&[(0, s)], 0, 1, [(pool, Vec2::new(40.0, 20.0), false)]);
        let placed = t.scatter_among(&data, &s, &ctx, Vec3::ZERO);
        let free = t.scatter(&data, &s, Vec3::ZERO);
        let low = |c: &&(u32, Mat4)| c.1.w_axis.y < 2.0 && c.1.w_axis.z < 0.0;
        assert!(
            free.iter().filter(low).count() > 100,
            "the low ground grows copies without the rule"
        );
        assert_eq!(
            placed.iter().filter(low).count(),
            0,
            "copies within 2 m of the pool"
        );
        let kept = |copies: &[(u32, Mat4)], f: &dyn Fn(&Mat4) -> bool| {
            copies.iter().filter(|c| f(&c.1)).count()
        };
        let above = |m: &Mat4| m.w_axis.y >= 2.0 || m.w_axis.z >= 0.0;
        assert_eq!(
            kept(&placed, &above),
            kept(&free, &above),
            "copies away from the pool"
        );
        // An endless sea at height 0 clears both halves' low ground.
        let sea = ScatterContext::of(&[(0, s)], 0, 1, [(Mat4::IDENTITY, Vec2::ONE, true)]);
        let placed = t.scatter_among(&data, &s, &sea, Vec3::ZERO);
        assert!(placed.iter().all(|c| c.1.w_axis.y >= 2.0));
    }

    /// A noise threshold gathers the copies into patches: at 0.5 it keeps
    /// part of the field, the same part every time, and a copy's
    /// neighbours are kept far more often than the field's average.
    #[test]
    fn a_noise_threshold_gathers_copies_into_patches() {
        let (t, data) = ramp();
        let s = TerrainScatter {
            density: 4.0,
            distance: 30.0,
            noise_scale: 6.0,
            noise_threshold: 0.5,
            ..Default::default()
        };
        let free = t.scatter(
            &data,
            &TerrainScatter {
                noise_threshold: 0.0,
                ..s
            },
            Vec3::ZERO,
        );
        let placed = t.scatter(&data, &s, Vec3::ZERO);
        assert_eq!(placed, t.scatter(&data, &s, Vec3::ZERO));
        let share = placed.len() as f32 / free.len() as f32;
        assert!((0.15..0.85).contains(&share), "keeps {share} of the field");
        // Kept copies within 1 m of a kept copy, against the share.
        let near = placed
            .iter()
            .map(|a| {
                let around = |c: &[(u32, Mat4)]| {
                    c.iter()
                        .filter(|b| {
                            b.0 != a.0 && (horizontal(&a.1) - horizontal(&b.1)).length() < 1.0
                        })
                        .count() as f32
                };
                around(&placed) / around(&free).max(1.0)
            })
            .sum::<f32>()
            / placed.len() as f32;
        assert!(
            near > share + 0.25,
            "a kept copy's neighbours are kept {near} of the time, the field {share}"
        );
    }
}
