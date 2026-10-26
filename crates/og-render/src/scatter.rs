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
//! the fragment shader thins them out over the fade with a dither, at full
//! size (`shaders/scatter*.wgsl`). A streamed terrain's chunks are built
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
    /// The outer part of `distance` over which they fade out (0..1).
    pub fade: f32,
    /// The range of their uniform scale.
    pub scale: [f32; 2],
    /// How far they lean with the ground: 0 upright, 1 along its normal.
    pub align: f32,
    pub seed: u32,
    /// How much the [`crate::RenderScene::wind`] sways them: 1 as it
    /// blows, 0 still (stones). Not part of the placement.
    pub sway: f32,
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
}

impl Default for Wind {
    fn default() -> Self {
        Wind {
            strength: 0.2,
            direction: Vec3::X,
            speed: 3.0,
            gust: 0.6,
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
fn wind_phases(wind: &Wind, tick: u64, tick_rate: f32) -> [f32; 2] {
    let t = tick as f64 / f64::from(tick_rate.max(1e-3));
    let k = std::f64::consts::TAU / f64::from(WIND_WAVELENGTH);
    let first = k * f64::from(wind.speed) * t;
    [first, first * WIND_OCTAVE].map(|p| p.rem_euclid(std::f64::consts::TAU) as f32)
}

impl TerrainScatter {
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
    fn control(&self, p: Vec2) -> Option<[f32; 4]>;
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
    fn control(&self, p: Vec2) -> Option<[f32; 4]> {
        Some(self.control_at(p))
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
    fn height(&self, p: Vec2) -> Option<f32> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).height(p))
    }
    fn normal(&self, p: Vec2) -> Option<Vec3> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).normal(p))
    }
    fn control(&self, p: Vec2) -> Option<[f32; 4]> {
        let (t, x, z) = self.tile(p)?;
        let l = &self.layout;
        let Some(texels) = &t.control else {
            return Some([1.0; 4]);
        };
        let c = l.control as f32;
        let side = l.control_side() as i64;
        let s = (p + l.size * 0.5) / l.cell();
        // Texel coordinates within the tile (its border at 0).
        let g = (s - Vec2::new((x * l.tile) as f32, (z * l.tile) as f32)) / l.tile as f32 * c + 0.5;
        let i = g.floor();
        let f = g - i;
        let texel = |a: f32, b: f32, k: usize| {
            let a = (a as i64).clamp(0, side - 1);
            let b = (b as i64).clamp(0, side - 1);
            texels[((b * side + a) * 4) as usize + k] as f32 / 255.0
        };
        Some([0, 1, 2, 3].map(|k| {
            let lo = texel(i.x, i.y, k) + (texel(i.x + 1.0, i.y, k) - texel(i.x, i.y, k)) * f.x;
            let hi = texel(i.x, i.y + 1.0, k)
                + (texel(i.x + 1.0, i.y + 1.0, k) - texel(i.x, i.y + 1.0, k)) * f.x;
            lo + (hi - lo) * f.y
        }))
    }
}

impl TerrainData {
    /// The control map's RGBA at `local`, bilinear between texel centres
    /// (as the renderer's linear sampler reads it), 0..1; all 1 without one.
    pub fn control_at(&self, local: Vec2) -> [f32; 4] {
        let Some(c) = &self.control else {
            return [1.0; 4];
        };
        let uv = ((local + self.size * 0.5) / self.size).clamp(Vec2::ZERO, Vec2::ONE);
        let t = uv * Vec2::new(c.width as f32, c.height as f32) - 0.5;
        let i = t.floor();
        let f = t - i;
        let texel = |x: f32, y: f32| {
            let x = (x as i64).clamp(0, c.width as i64 - 1) as usize;
            let y = (y as i64).clamp(0, c.height as i64 - 1) as usize;
            let at = (y * c.width as usize + x) * 4;
            [0, 1, 2, 3].map(|k| c.rgba[at + k] as f32 / 255.0)
        };
        let (a, b) = (texel(i.x, i.y), texel(i.x + 1.0, i.y));
        let (d, e) = (texel(i.x, i.y + 1.0), texel(i.x + 1.0, i.y + 1.0));
        [0, 1, 2, 3].map(|k| {
            let lo = a[k] + (b[k] - a[k]) * f.x;
            let hi = d[k] + (e[k] - d[k]) * f.x;
            lo + (hi - lo) * f.y
        })
    }
}

impl Terrain {
    /// The layers' weights at `local` (summing to 1), as the renderer weighs
    /// them before the height blend: control map times the slope and
    /// altitude rules; the first layer where every weight is 0.
    pub fn layer_weights(&self, data: &TerrainData, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        self.weights_on(data, local).unwrap_or([1.0, 0.0, 0.0, 0.0])
    }

    fn weights_on(&self, g: &dyn Ground, local: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
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
            return Some([1.0, 0.0, 0.0, 0.0]);
        }
        Some(w.map(|x| x / sum))
    }

    /// The copy of cell (`x`, `z`) of `s`'s grid: `Some(None)` where none
    /// grows, `None` where the ground is not resident yet.
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
        if unit(h3) >= weight {
            return Some(None);
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
                if let Some(Some((key, m, p))) = self.copy_at(data, s, x, z)
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
    /// xy: the wind's horizontal direction (x, z), z: its strength times
    /// the scatter's sway (0: still), w: its gust.
    wind: [f32; 4],
    /// The wind's time phases (`wind_phases`): xy this frame's, zw the
    /// previous frame's.
    wind_phase: [f32; 4],
    /// x: the gust wave's wavenumber (rad/m), y: the mesh's top above its
    /// origin (m).
    wind_shape: [f32; 4],
}

const SLOT: u64 = 256;

/// What a chunk was built from: the scatter, its terrain's placement and
/// data, the mesh and material, and the chunk's place.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ChunkKey {
    data: usize,
    /// The data's revision, and for a streamed terrain the tiles under the chunk.
    source: (u64, Vec<usize>),
    params: [u32; 8],
    transform: [u32; 16],
    layers: Vec<u32>,
    mesh: usize,
    chunk: (u32, u32),
}

struct Chunk {
    buffer: Option<wgpu::Buffer>,
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
    /// How far the wind moves a copy at most: the chunks' bounds grow by it.
    sway: f32,
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
    uniform_layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    slots: u64,
    group: wgpu::BindGroup,
    chunks: HashMap<ChunkKey, Chunk>,
    frame: u64,
    /// Chunks built for the last frame.
    pub built: u32,
    /// The wind's phases the last frame drew with (the motion vectors'
    /// previous positions).
    wind_phase: Option<[f32; 2]>,
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

    pub(crate) fn new(device: &wgpu::Device, p: ScatterPipelines) -> ScatterRenderer {
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter"),
            entries: &[wgpu::BindGroupLayoutEntry {
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
            }],
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
        let (uniform, group) = Self::slots(device, &uniform_layout, 4);
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
            uniform_layout,
            uniform,
            slots: 4,
            group,
            chunks: HashMap::new(),
            frame: 0,
            built: 0,
            wind_phase: None,
        }
    }

    fn slots(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
                }),
            }],
        });
        (buffer, group)
    }

    fn pipeline(
        &mut self,
        device: &wgpu::Device,
        shader: Option<usize>,
        side: Side,
        pass: ScatterPass,
    ) {
        let kind = pass as u8;
        // The shadow pass reads no material shader (copies cast as cut by the material's alpha).
        let shader = shader.filter(|_| pass != ScatterPass::Shadow);
        if self.pipelines.contains_key(&(shader, side, kind)) {
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
            ScatterPass::Shadow => device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scatter shadow"),
                layout: Some(&self.shadow_layout),
                vertex: wgpu::VertexState {
                    module: &self.shadow_module,
                    entry_point: Some("vs_shadow"),
                    compilation_options: Default::default(),
                    buffers: &buffers,
                },
                // As shadow.wgsl's casters: front-sided ones draw back faces.
                primitive: wgpu::PrimitiveState {
                    cull_mode: match side {
                        Side::Front => Some(wgpu::Face::Front),
                        Side::Back => Some(wgpu::Face::Back),
                        Side::Double => None,
                    },
                    ..Default::default()
                },
                depth_stencil: depth(true),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &self.shadow_module,
                    entry_point: Some("fs_shadow"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            }),
        };
        self.pipelines.insert((shader, side, kind), p);
    }

    /// Builds the chunks this frame needs and answers what to draw.
    /// `side_of` answers a material's side (`None`: not of this scene);
    /// `mesh_bounds` a mesh's radius about its origin and its top above it
    /// (`None`: no geometry).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        camera: Vec3,
        view_proj: Mat4,
        prev_view_proj: Mat4,
        side_of: impl Fn(usize) -> Option<Side>,
        mesh_bounds: impl Fn(usize) -> Option<(f32, f32)>,
        shader_of: impl Fn(usize) -> Option<usize>,
        hooks_of: impl Fn(usize) -> String,
    ) -> Vec<ScatterBatch> {
        use wgpu::util::DeviceExt;
        self.frame += 1;
        self.built = 0;
        let mut batches = Vec::new();
        let wind = scene.wind.filter(|w| w.strength > 0.0);
        let phase = wind.map(|w| wind_phases(&w, scene.tick, scene.tick_rate));
        let prev_phase = self.wind_phase.or(phase).unwrap_or_default();
        self.wind_phase = phase;
        if scene.scatters.is_empty() {
            self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
            return batches;
        }
        if self.slots < scene.scatters.len() as u64 {
            self.slots = (scene.scatters.len() as u64).next_power_of_two();
            let (u, g) = Self::slots(device, &self.uniform_layout, self.slots);
            self.uniform = u;
            self.group = g;
        }
        let (sources, revisions) = scene.terrain_sources();
        let mut budget = CHUNKS_PER_FRAME;
        for (si, d) in scene.scatters.iter().enumerate() {
            let Some(t) = scene.terrains.get(d.terrain) else {
                continue;
            };
            let di = t.terrain.index();
            let (Some(source), Some(side), Some((mesh_r, mesh_top))) = (
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
            let mut params = [0u32; 8];
            for (k, v) in [
                s.density,
                s.scale[0],
                s.scale[1],
                s.align,
                s.layer as f32,
                s.seed as f32,
            ]
            .iter()
            .enumerate()
            {
                params[k] = v.to_bits();
            }
            let transform = model.to_cols_array().map(f32::to_bits);
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
                TerrainSource::Data(_) => None,
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
                    params,
                    transform,
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
                    TerrainSource::Data(d) => d,
                    TerrainSource::Stream { tiles, .. } => {
                        tile_ground = TileGround { layout, tiles };
                        &tile_ground
                    }
                };
                'cells: for z in cz * CHUNK_CELLS..((cz + 1) * CHUNK_CELLS).min(cells.y as u32) {
                    for x in cx * CHUNK_CELLS..((cx + 1) * CHUNK_CELLS).min(cells.x as u32) {
                        match t.copy_at(ground, &s, x, z) {
                            None => {
                                complete = false;
                                break 'cells;
                            }
                            Some(Some((_, m, _))) => placed.push(m),
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
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("scatter chunk"),
                        contents: bytemuck::cast_slice(&transforms),
                        usage: wgpu::BufferUsages::VERTEX,
                    })
                });
                self.chunks.insert(
                    key.clone(),
                    Chunk {
                        buffer,
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
            let (wind_row, sway) = match wind {
                Some(w) if s.sway > 0.0 && mesh_top > 0.0 => {
                    let dir = Vec2::new(w.direction.x, w.direction.z).normalize_or(Vec2::X);
                    let lean = w.strength * s.sway;
                    let gust = w.gust.clamp(0.0, 1.0);
                    let tip = lean * (1.0 + gust) * mesh_top * s.scale[1].max(s.scale[0]);
                    ([dir.x, dir.y, lean, gust], tip)
                }
                _ => ([0.0; 4], 0.0),
            };
            let now = phase.unwrap_or_default();
            let gpu = ScatterGpu {
                camera: camera.extend(s.distance).to_array(),
                fade: [s.fade_from(), 0.0, 0.0, 0.0],
                view_proj: view_proj.to_cols_array_2d(),
                prev_view_proj: prev_view_proj.to_cols_array_2d(),
                wind: wind_row,
                wind_phase: [now[0], now[1], prev_phase[0], prev_phase[1]],
                wind_shape: [std::f32::consts::TAU / WIND_WAVELENGTH, mesh_top, 0.0, 0.0],
            };
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
                self.pipeline(device, shader, side, pass);
            }
            batches.push(ScatterBatch {
                mesh: d.mesh.index(),
                material: d.material.index(),
                shader,
                cast_shadows: d.cast_shadows,
                side,
                offset: (si as u64 * SLOT) as u32,
                chunks: keys,
                sway,
            });
        }
        self.chunks.retain(|_, c| self.frame - c.used < KEEP_FRAMES);
        batches
    }

    /// Draws `batches` into a pass (groups 0 and, for the lit passes, the
    /// frame bound by the caller): the chunks whose bounds `visible` keeps.
    /// Answers the draws and the copies drawn.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw<'a>(
        &self,
        pass: &mut wgpu::RenderPass,
        kind: ScatterPass,
        batches: &[ScatterBatch],
        material_group: impl Fn(usize) -> Option<&'a wgpu::BindGroup>,
        mesh: impl Fn(usize) -> Option<(&'a wgpu::Buffer, &'a wgpu::Buffer, u32)>,
        visible: impl Fn(Vec3, f32) -> bool,
        mut probes: Option<&mut crate::probe_select::Binder>,
        fx_group: Option<&wgpu::BindGroup>,
    ) -> (u32, u32) {
        let (mut draws, mut copies) = (0, 0);
        for b in batches {
            if kind == ScatterPass::Shadow && !b.cast_shadows {
                continue;
            }
            let shader = b.shader.filter(|_| kind != ScatterPass::Shadow);
            let (Some(group), Some((vertices, indices, count)), Some(p)) = (
                material_group(b.material),
                mesh(b.mesh),
                self.pipelines.get(&(shader, b.side, kind as u8)),
            ) else {
                continue;
            };
            let mut bound = false;
            for k in &b.chunks {
                let Some(c) = self.chunks.get(k) else {
                    continue;
                };
                let Some(buffer) = &c.buffer else { continue };
                if !visible(c.center, c.radius + b.sway) {
                    continue;
                }
                if !bound {
                    pass.set_pipeline(p);
                    pass.set_bind_group(1, group, &[]);
                    pass.set_bind_group(2, &self.group, &[b.offset]);
                    if kind == ScatterPass::Lit
                        && let Some(fx) = fx_group
                    {
                        pass.set_bind_group(3, fx, &[]);
                    }
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    bound = true;
                }
                // Each chunk's reflection probe (probe_select.rs).
                if let Some(b) = probes.as_deref_mut() {
                    b.at(pass, c.center, c.radius);
                }
                pass.set_vertex_buffer(1, buffer.slice(..));
                pass.draw_indexed(0..count, 0, 0..c.count);
                draws += 1;
                copies += c.count;
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
