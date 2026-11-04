//! Grass: blades generated on the GPU every frame over a terrain layer, to
//! the horizon (R6, `shaders/grass*.wgsl`).
//!
//! A [`GrassDraw`] in [`crate::RenderScene::grass`] grows a field over one
//! of the frame's terrains where its layer shows. Nothing is stored per
//! blade: candidates are the points of a progressive blue-noise tile
//! ([`blue_noise_tile`]) copied over 16 m chunks, each copy offset by a
//! hash of its chunk and cell, and a candidate becomes a blade while its
//! rank is below the density there (the layer's weight times the distance
//! table, [`DistanceLod`]), so the field thins out evenly with distance
//! and the survivors widen to keep the ground covered. Blades take one of
//! three levels of detail (strips of 7, 3 and 1 rows), dithered over a 4 m
//! band at each boundary; past the blades the terrain takes the field's
//! colour ([`far_gpu`], terrain_lit.wgsl `grass_far`), so the far field is
//! the same colour as the near one and there is no seam.
//!
//! **Each frame**, the CPU lists the chunks in the camera's view (and, for
//! a field that casts, those whose blades' shadows reach into it) within
//! `distance`, nearest first, each cut into slices of [`SLICE`]
//! candidates by how many it may keep. Three compute passes follow
//! (grass_generate.wgsl): every candidate is evaluated at its own slot
//! (placement, layer weight, holes, the blade's frustum test, its level
//! and the wind's bend), a prefix sum of the slices' counts gives each
//! slice its place, and the blades are compacted, in candidate order, into
//! the instance buffer the three indexed indirect draws read. Every
//! blade's place follows from the candidates alone, so a frame lays out
//! the same instances on every run (bevy_grass appended with atomics).
//!
//! **Drawn** in the main camera's prepass and lit pass with pipelines and
//! groups of their own (group 1: the field's uniform and blades; no
//! binding is added to any other pipeline), and with `cast_shadows` into
//! the first shadow cascade. Wind is og.Wind at the tick (its travelling
//! wave, gust patches scrolled downwind and a sway per blade, all reduced
//! in f64 on the CPU): presentation only, and the previous frame's bend is
//! kept in each blade for the motion vectors.
//!
//! Ported from bevy_grass v0.20.3 (third_party/bevy-grass: src/mirror.rs,
//! src/shader_model.rs, src/settings.rs and the shaders). MIT License,
//! Copyright (c) 2026 Gediminas Mikelenas. Permission is hereby granted,
//! free of charge, to any person obtaining a copy of this software and
//! associated documentation files (the "Software"), to deal in the
//! Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or
//! sell copies of the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following conditions: The above
//! copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software. THE SOFTWARE IS
//! PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
//! INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
//! FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//! FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.

use crate::terrain::horizontal_scale;
use crate::wind::WindFieldGpu;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::sync::OnceLock;

/// A field of grass: the zero value is not usable; use `default()`, a
/// meadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grass {
    /// The terrain layer whose weight places the blades.
    pub layer: u32,
    /// Blades per square metre where the layer's weight is 1, near the camera.
    pub density: f32,
    /// How far from the camera blades are drawn (m); they thin out to
    /// nothing there.
    pub distance: f32,
    /// The terrain takes the field's average colour under its blades,
    /// where they thin out and past them.
    pub far_tint: bool,
    /// The tallest blade (m) and how much shorter blades are, 0..1.
    pub height: f32,
    pub height_variation: f32,
    /// A blade's width at its root (m).
    pub width: f32,
    /// How far blades droop along their length, 0..1.
    pub curve: f32,
    /// How far blades lean at rest along their face, as a share of height.
    pub lean: f32,
    /// The clumps' size (m), how far blades lean out from their clump's
    /// centre, and how far they are pulled in towards it (tufts), 0..1.
    pub clump_size: f32,
    pub clump_lean: f32,
    pub tufts: f32,
    /// Linear colours: roots, tips and dry grass.
    pub base_color: Vec3,
    pub tip_color: Vec3,
    pub dry_color: Vec3,
    /// How much blades vary in colour by clump, blade and patch, 0..1.
    pub color_variation: f32,
    /// The share of clumps that are dry, and how dry tips get, 0..1.
    pub dryness: f32,
    /// How much light passes through a blade lit from behind, 0..1.
    pub translucency: f32,
    /// The upper blade's gloss, 0..1, and the blade's roughness.
    pub sheen: f32,
    pub roughness: f32,
    /// How much og.Wind bends the blades (0: still), how stiff they are
    /// (0..1), and how much full-detail tips flutter.
    pub sway: f32,
    pub stiffness: f32,
    pub flutter: f32,
    /// Blades cast shadows into the first [`SHADOW_CASCADES`] cascades.
    pub cast_shadows: bool,
    pub seed: u32,
    /// Seconds trampled blades take to stand again (`trample.rs`).
    pub recovery: f32,
    /// Up to three more kinds of blade among the field's own (its height,
    /// width, curve and colours are the first): each clump grows one.
    pub types: [Option<GrassType>; 3],
    /// Where `types` grow (red, green and blue the shares of the first,
    /// second and third; the rest the field's own kind), over the
    /// terrain's extent as its control map; `None`: by their shares.
    pub type_map: Option<crate::TextureHandle>,
    /// Multiply the density and the height (their red channels), laid
    /// out as `type_map`.
    pub density_map: Option<crate::TextureHandle>,
    pub height_map: Option<crate::TextureHandle>,
}

/// Another kind of blade in a [`Grass`] field: its shape and colours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrassType {
    pub height: f32,
    pub width: f32,
    pub curve: f32,
    pub base_color: Vec3,
    pub tip_color: Vec3,
    /// Clumps of this kind per clump of the field's own, without a type map.
    pub share: f32,
}

impl Default for GrassType {
    fn default() -> Self {
        let g = Grass::default();
        GrassType {
            height: g.height,
            width: g.width,
            curve: g.curve,
            base_color: g.base_color,
            tip_color: g.tip_color,
            share: 1.0,
        }
    }
}

impl Default for Grass {
    fn default() -> Self {
        Grass {
            layer: 0,
            density: 300.0,
            distance: 150.0,
            far_tint: true,
            height: 0.45,
            height_variation: 0.4,
            width: 0.012,
            curve: 0.5,
            lean: 0.3,
            clump_size: 1.2,
            clump_lean: 0.4,
            tufts: 0.0,
            base_color: Vec3::new(0.04, 0.085, 0.024),
            tip_color: Vec3::new(0.235, 0.39, 0.11),
            dry_color: Vec3::new(0.45, 0.38, 0.17),
            color_variation: 0.3,
            dryness: 0.15,
            translucency: 0.6,
            sheen: 0.4,
            roughness: 0.6,
            sway: 1.0,
            stiffness: 0.5,
            flutter: 0.3,
            cast_shadows: true,
            seed: 1,
            recovery: 4.0,
            types: [None; 3],
            type_map: None,
            density_map: None,
            height_map: None,
        }
    }
}

/// A field of grass drawn this frame over terrain `terrain` (an index into
/// [`crate::RenderScene::terrains`], a drawn one: its data, placement and
/// layers place the blades).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrassDraw {
    pub terrain: usize,
    pub grass: Grass,
}

impl Grass {
    /// The field's kinds of blade, its own first: (height, width, share).
    fn kinds(&self) -> [Option<(f32, f32, f32)>; 4] {
        let mut out = [Some((self.height, self.width, 1.0)), None, None, None];
        for (k, t) in self.types.iter().enumerate() {
            out[k + 1] = t.map(|t| (t.height.max(0.0), t.width.max(0.0), t.share.max(0.0)));
        }
        out
    }

    /// The tallest blade of any kind (m).
    pub fn tallest(&self) -> f32 {
        self.kinds().iter().flatten().fold(0.0, |a, k| a.max(k.0))
    }

    /// The widest root of any kind (m).
    pub fn widest(&self) -> f32 {
        self.kinds().iter().flatten().fold(0.0, |a, k| a.max(k.1))
    }

    pub(crate) fn usable(&self) -> bool {
        self.density > 0.0
            && self.distance > 0.0
            && self.height > 0.0
            && self.width > 0.0
            && self.density.is_finite()
            && self.distance.is_finite()
    }
}

// ── the CPU model (twins of grass_generate.wgsl) ───────────────────────────

/// A chunk's side (m in the world, divided by the terrain's horizontal scale).
pub const CHUNK: f32 = 16.0;
/// Points in the blue-noise tile.
pub const TILE_POINTS: u32 = 1024;
/// Candidates per slice: one workgroup's share of a chunk.
pub const SLICE: u32 = 512;
/// The most slices a field generates in a frame (the farthest are dropped).
pub const MAX_SLICES: u32 = 8192;
/// The most blades a field draws in a frame: past it, [`plan`] thins the
/// whole field evenly (its survivors widen) rather than dropping the far
/// chunks.
pub const MAX_BLADES: u32 = 1 << 21;
/// Rows of each level of detail's strip (15, 7 and 3 vertices).
pub const LOD_ROWS: [u32; 3] = [7, 3, 1];
/// How much a gust brightens the blades it presses down (and the ground
/// past them): grass_common.wgsl `grass_gust_sheen`.
const GUST_SHEEN: f32 = 1.5;
/// The narrowest a blade draws (px).
const MIN_WIDTH_PX: f32 = 1.0;
/// The sun's cascades blades cast into (G5): the near field's two, where
/// a blade's shadow is wider than a texel.
pub(crate) const SHADOW_CASCADES: usize = 2;
/// How much of a blade's width blocks a ray crossing the field's canopy
/// (the canopy's extinction is blades per square metre times their width
/// times this, per metre of the ray).
const CANOPY_BLOCKING: f32 = 0.5;
/// How far a blade seen edge-on turns towards the eye.
const TURN_TO_EYE: f32 = 0.4;

/// PCG hash (Jarzynski and Olano, 2020): wind_common.wgsl `wind_hash`.
pub fn hash(x: u32) -> u32 {
    let state = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

fn unit(h: u32) -> f32 {
    (h >> 8) as f32 / 16_777_216.0
}

/// The blue-noise tile: [`TILE_POINTS`] points in the unit square,
/// wrapping, in progressive order (every prefix is itself evenly spread,
/// so thinning by rank keeps the field even). Mitchell's best candidate
/// with a fixed seed: the same on every machine.
pub fn blue_noise_tile() -> &'static [Vec2] {
    static TILE: OnceLock<Vec<Vec2>> = OnceLock::new();
    TILE.get_or_init(|| {
        let mut points: Vec<Vec2> = Vec::with_capacity(TILE_POINTS as usize);
        let mut rng = 0x6b8a_3a00u32;
        let mut next = || {
            rng = hash(rng);
            unit(rng)
        };
        while points.len() < TILE_POINTS as usize {
            let (mut best, mut best_d) = (Vec2::ZERO, -1.0f32);
            for _ in 0..12 {
                let c = Vec2::new(next(), next());
                let d = points
                    .iter()
                    .map(|p| {
                        let d = (*p - c).abs();
                        d.min(Vec2::ONE - d).length_squared()
                    })
                    .fold(f32::INFINITY, f32::min);
                if d > best_d {
                    (best, best_d) = (c, d);
                }
            }
            points.push(best);
        }
        points
    })
}

/// How density and detail fall off with distance, for a field reaching
/// `radius` metres (bevy_grass's High tier, its shares of the radius).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceLod {
    pub radius: f32,
    /// Where LOD 1 and LOD 2 start (m).
    pub lod_start: [f32; 2],
    /// Where the last fade starts, a share of the radius.
    pub fade_start: f32,
    /// The band each level boundary dithers over (m).
    pub band: f32,
    /// The share of blades kept where LOD 2 starts and where the fade starts.
    pub density_at: [f32; 2],
    /// A further share kept from LOD 1 on (the survivors widen).
    pub mid_range: f32,
    /// The most a thinned blade widens.
    pub widest: f32,
}

impl DistanceLod {
    pub fn for_radius(radius: f32) -> DistanceLod {
        DistanceLod {
            radius,
            lod_start: [0.2 * radius, 0.5 * radius],
            fade_start: 0.8,
            band: 4.0,
            density_at: [0.5, 0.15],
            mid_range: 0.5,
            widest: 2.5,
        }
    }

    /// The share of blades drawn at `d` (grass_generate.wgsl `grass_falloff`).
    pub fn falloff(&self, d: f32) -> f32 {
        let into = ((d - (self.lod_start[0] - self.band * 0.5)) / self.band).clamp(0.0, 1.0);
        self.thinning(d) * (1.0 + (self.mid_range - 1.0) * into)
    }

    fn thinning(&self, d: f32) -> f32 {
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t.clamp(0.0, 1.0);
        let [lod1, lod2] = self.lod_start;
        let [at2, at_edge] = self.density_at;
        let edge = self.radius * self.fade_start;
        if d <= lod1 {
            1.0
        } else if d <= lod2 {
            lerp(1.0, at2, (d - lod1) / (lod2 - lod1).max(1e-4))
        } else if d <= edge {
            lerp(at2, at_edge, (d - lod2) / (edge - lod2).max(1e-4))
        } else {
            lerp(at_edge, 0.0, (d - edge) / (self.radius - edge).max(1e-4))
        }
    }

    /// The level of a blade at `d` with dither `dither` in [0, 1)
    /// (grass_generate.wgsl `grass_lod`).
    pub fn lod(&self, d: f32, dither: f32) -> u32 {
        self.lod_start
            .iter()
            .filter(|&&s| dither < (d - (s - self.band * 0.5)) / self.band)
            .count() as u32
    }
}

/// How the blue-noise tile covers a field's chunks, in the terrain's space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldLayout {
    /// A chunk's side.
    pub chunk: f32,
    pub tiles_per_side: u32,
    pub tile: f32,
    /// Candidates and wanted blades per square metre.
    pub candidates: f32,
    pub density: f32,
}

impl FieldLayout {
    /// The coarsest tiling with at least the field's density in
    /// candidates, for a terrain stretched `along` times along x and z.
    pub fn new(g: &Grass, along: f32) -> FieldLayout {
        let chunk = CHUNK / along.max(1e-4);
        let density = g.density.clamp(0.0, 4000.0) * along * along;
        let tiles = ((chunk * (density / TILE_POINTS as f32).sqrt()).ceil() as u32).clamp(1, 64);
        let tile = chunk / tiles as f32;
        FieldLayout {
            chunk,
            tiles_per_side: tiles,
            tile,
            candidates: TILE_POINTS as f32 / (tile * tile),
            density,
        }
    }
}

/// What the chunks are planned for.
pub struct PlanView<'a> {
    /// The terrain's placement (frame coordinates), size and height range.
    pub model: Mat4,
    pub size: Vec2,
    pub range: (f32, f32),
    pub camera: Vec3,
    pub orthographic: bool,
    /// Whether a world box (min, max corner) is in the view (or casts into
    /// it).
    pub visible: &'a dyn Fn(Vec3, Vec3) -> bool,
}

/// What a field generates this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// x, y a chunk, z the slice's first candidate in it, w how many;
    /// nearest chunks first, at most [`MAX_SLICES`].
    pub slices: Vec<[u32; 4]>,
    /// The share of the distance table's blades kept, 1 unless the view
    /// would hold more than [`MAX_BLADES`] candidates: then every chunk
    /// keeps that share, so the field still reaches `distance`.
    pub density: f32,
}

/// The slices field `g` generates for view `v`.
pub fn plan(g: &Grass, v: &PlanView) -> Plan {
    let along = horizontal_scale(v.model);
    let layout = FieldLayout::new(g, along);
    let lod = DistanceLod::for_radius(g.distance);
    let local = v.model.inverse().transform_point3(v.camera);
    let half = v.size * 0.5;
    let eye = Vec2::new(local.x, local.z) + half;
    let reach = g.distance / along;
    let n = (v.size / layout.chunk).ceil();
    let lo = ((eye - reach) / layout.chunk).floor().max(Vec2::ZERO);
    let hi = ((eye + reach) / layout.chunk).ceil().min(n).max(Vec2::ZERO);
    let tall = g.tallest() * (1.0 + g.lean.abs() + g.clump_lean.abs());
    let cells = layout.tiles_per_side * layout.tiles_per_side;
    let mut chunks: Vec<(f32, u32, u32, f32)> = Vec::new();
    for cz in lo.y as u32..hi.y as u32 {
        for cx in lo.x as u32..hi.x as u32 {
            let a = Vec2::new(cx as f32, cz as f32) * layout.chunk;
            let b = (a + layout.chunk).min(v.size);
            let d = if v.orthographic {
                0.0
            } else {
                (eye.clamp(a, b) - eye).length() * along
            };
            if !v.orthographic && d >= g.distance {
                continue;
            }
            let (lo, hi) = world_box(
                v.model,
                Vec3::new(a.x - half.x, v.range.0, a.y - half.y),
                Vec3::new(b.x - half.x, v.range.1 + tall, b.y - half.y),
            );
            if !(v.visible)(lo, hi) {
                continue;
            }
            let falloff = if v.orthographic { 1.0 } else { lod.falloff(d) };
            let keep = (layout.density * falloff / layout.candidates).clamp(0.0, 1.0);
            if keep > 0.0 {
                chunks.push((d, cx, cz, keep));
            }
        }
    }
    og_core::sort::sort_by(&mut chunks, |a, b| {
        a.0.total_cmp(&b.0).then((a.2, a.1).cmp(&(b.2, b.1)))
    });
    let needed = |keep: f32| (keep * TILE_POINTS as f32).ceil() as u32 * cells;
    let total: u64 = chunks.iter().map(|c| u64::from(needed(c.3))).sum();
    let density = (MAX_BLADES as f64 / total.max(1) as f64).min(1.0) as f32;
    let mut slices = Vec::new();
    'chunks: for (_, cx, cz, keep) in chunks {
        let needed = needed(keep * density);
        for first in (0..needed).step_by(SLICE as usize) {
            if slices.len() >= MAX_SLICES as usize {
                break 'chunks;
            }
            slices.push([cx, cz, first, SLICE.min(needed - first)]);
        }
    }
    Plan { slices, density }
}

/// The world box (min, max corner) of box `lo`..`hi` placed by `model`.
fn world_box(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, Vec3) {
    (0..8).fold((Vec3::MAX, Vec3::MIN), |(a, b), i| {
        let p = model.transform_point3(Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        ));
        (a.min(p), b.max(p))
    })
}

/// `TerrainGrass` in terrain.wgsl: the far colour of a field.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FarGpu {
    color: [f32; 4],
    fade: [f32; 4],
    clump: [f32; 4],
    dry: [f32; 4],
    wind: WindFieldGpu,
}

/// A field's average colour (grass.wgsl `grass_far_color`): its blades'
/// colour a little over halfway up, darkened by their occlusion towards
/// the roots and the shade between them.
fn far_color(g: &Grass) -> Vec3 {
    g.base_color.lerp(g.tip_color, 0.55) * 0.7
}

/// The seed as the shaders read it: 24 bits, exact in an f32.
fn seed_bits(seed: u32) -> f32 {
    (hash(seed ^ 0x9e37_79b9) >> 8) as f32
}

/// The far grass terrain `terrain` of `scene` takes: its first field
/// with `far_tint` (none: colour w -1).
pub(crate) fn far_gpu(scene: &crate::RenderScene, terrain: usize) -> FarGpu {
    let mut gpu = FarGpu {
        color: [0.0, 0.0, 0.0, -1.0],
        fade: [0.0; 4],
        clump: [1.0, 0.0, 0.0, 0.0],
        dry: [0.0; 4],
        wind: WindFieldGpu::default(),
    };
    let Some(d) = scene
        .grass
        .iter()
        .find(|d| d.terrain == terrain && d.grass.far_tint && d.grass.usable())
    else {
        return gpu;
    };
    let g = &d.grass;
    let along = scene
        .terrains
        .get(terrain)
        .map_or(1.0, |t| horizontal_scale(t.transform));
    // The field's average: its blades' colour a little over halfway up
    // (grass_blade.wgsl grass_color), darkened by their occlusion towards
    // the roots and the shade between them, so the ground under and past
    // the blades reads as the same field.
    let c = far_color(g);
    gpu.color = c.extend(g.layer as f32).to_array();
    // Under every blade (gaps show the field's colour, not the layer's),
    // whole by the start of LOD 1; gusts brighten it as they do the blades.
    let lod = DistanceLod::for_radius(g.distance);
    gpu.fade = [0.0, lod.lod_start[0], GUST_SHEEN, 1.0];
    gpu.clump = [
        g.clump_size.max(0.05) / along,
        g.color_variation.clamp(0.0, 1.0),
        g.dryness.clamp(0.0, 1.0),
        seed_bits(g.seed),
    ];
    gpu.dry = g.dry_color.extend(0.0).to_array();
    // The far tint brightens in og.Wind's analytic gusts (the terrain
    // does not read the simulated field).
    gpu.wind = crate::wind::field_gpu(
        scene.wind,
        scene.tick,
        scene.tick,
        scene.tick_rate,
        scene.origin,
        crate::wind_field::NO_GRID,
    );
    gpu
}

// ── drawing ────────────────────────────────────────────────────────────────

/// `GrassGen` in grass_generate.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GenGpu {
    planes: [[f32; 4]; 6],
    eye: [f32; 4],
    origin: [f32; 4],
    lod: [f32; 4],
    lod_density: [f32; 4],
    field: [f32; 4],
    chunk: [f32; 4],
    shape: [f32; 4],
    clump: [f32; 4],
    shadow: [f32; 4],
    motion: [f32; 4],
    counts: [u32; 4],
    trample: [f32; 4],
    kinds: [[f32; 4]; 4],
    maps: [f32; 4],
    wind: WindFieldGpu,
}

/// `GrassUniform` in grass_blade.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawGpu {
    view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    forward: [f32; 4],
    origin: [f32; 4],
    limits: [f32; 4],
    sun: [f32; 4],
    shape: [f32; 4],
    base_color: [f32; 4],
    tip_color: [f32; 4],
    dry_color: [f32; 4],
    variation: [f32; 4],
    rows: [u32; 4],
    canopy: [f32; 4],
    kinds: [[f32; 4]; 8],
    wind: WindFieldGpu,
}

/// A field's kinds as `GrassGen.kinds` holds them: height, width, where
/// its share ends (summed in order over the kinds the field has) and
/// whether it has it.
fn kinds_gen(g: &Grass) -> [[f32; 4]; 4] {
    let kinds = g.kinds();
    let total: f32 = kinds.iter().flatten().map(|k| k.2).sum();
    let mut run = 0.0;
    std::array::from_fn(|i| match kinds[i] {
        Some((h, w, share)) => {
            run += share;
            let end = if total > 0.0 { run / total } else { 1.0 };
            [h, w, if i == 3 { 1.0 } else { end }, 1.0]
        }
        None => [0.0, 0.0, if total > 0.0 { run / total } else { 1.0 }, 0.0],
    })
}

/// A field's kinds as `GrassUniform.kinds` holds them: roots' colour and
/// curve, then tips' colour, the field's own first.
fn kinds_draw(g: &Grass) -> [[f32; 4]; 8] {
    let own = GrassType {
        height: g.height,
        width: g.width,
        curve: g.curve,
        base_color: g.base_color,
        tip_color: g.tip_color,
        share: 1.0,
    };
    let mut out = [[0.0; 4]; 8];
    for (k, t) in std::iter::once(Some(own)).chain(g.types).enumerate() {
        let t = t.unwrap_or(own);
        out[2 * k] = t.base_color.extend(t.curve.clamp(0.0, 1.0)).to_array();
        out[2 * k + 1] = t.tip_color.extend(0.0).to_array();
    }
    out
}

/// A field's look as `GrassUniform` holds it: `base_color`, `tip_color`,
/// `dry_color` and `variation`.
fn look_gpu(g: &Grass) -> [[f32; 4]; 4] {
    [
        g.base_color
            .extend(g.translucency.clamp(0.0, 1.0))
            .to_array(),
        g.tip_color.extend(g.sheen.clamp(0.0, 1.0)).to_array(),
        g.dry_color.extend(g.roughness.clamp(0.04, 1.0)).to_array(),
        [
            g.color_variation.clamp(0.0, 1.0),
            g.dryness.clamp(0.0, 1.0),
            g.clump_size.max(0.05) * 6.0,
            g.distance,
        ],
    ]
}

/// The height range blade roots are stored over, either side of the camera (m).
const HEIGHT_RANGE: f32 = 512.0;

/// The grass lit module: the frame, the light tables, the lighting, the
/// area lights, then the blades.
pub(crate) fn lit_source() -> String {
    format!(
        "{}{}{}{}{}{}{}{}",
        crate::FRAME_WGSL,
        crate::clusters::wgsl(),
        crate::LIGHTING_WGSL,
        crate::AREA_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::terrain::GRASS_COMMON_WGSL,
        wgsl3d!("grass_blade.wgsl"),
        wgsl3d!("grass.wgsl"),
    )
}

/// The grass shadow module.
pub(crate) fn shadow_source() -> String {
    format!(
        "{}{}{}{}{}",
        crate::SHADOW_PASS_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::terrain::GRASS_COMMON_WGSL,
        wgsl3d!("grass_blade.wgsl"),
        wgsl3d!("grass_shadow.wgsl"),
    )
}

/// The generation module over a terrain's group (`arrayed`: the
/// layer-array variant's).
pub(crate) fn generate_source(arrayed: bool) -> String {
    let s = format!(
        "{}{}{}{}{}",
        crate::wind::WIND_COMMON_WGSL,
        crate::wind::WIND_LEAN_WGSL,
        crate::terrain::GRASS_COMMON_WGSL,
        crate::terrain::TERRAIN_WGSL,
        wgsl3d!("grass_generate.wgsl"),
    );
    if arrayed {
        crate::terrain::terrain_array_declarations(&s)
    } else {
        s
    }
}

/// The three levels' strips, one index list after another, each with
/// vertex indices local to its level (row r: 2r and 2r + 1; the tip 2 rows).
fn lod_indices() -> Vec<u32> {
    let mut out = Vec::new();
    for rows in LOD_ROWS {
        for r in 0..rows - 1 {
            let (l, rt) = (2 * r, 2 * r + 1);
            out.extend_from_slice(&[l, rt, l + 2, rt, rt + 2, l + 2]);
        }
        out.extend_from_slice(&[2 * (rows - 1), 2 * (rows - 1) + 1, 2 * rows]);
    }
    out
}

/// One field's buffers.
struct FieldGpu {
    generator: wgpu::Buffer,
    draw: wgpu::Buffer,
    slices: wgpu::Buffer,
    draws: wgpu::Buffer,
    /// Slices and blades the buffers hold.
    slice_cap: u32,
    blade_cap: u32,
    gen_group: wgpu::BindGroup,
    draw_group: wgpu::BindGroup,
}

/// One field's work this frame: its buffers' slot, its terrain's draw (the
/// group generation binds), its slices and whether it casts.
pub(crate) struct GrassField {
    slot: usize,
    /// Its maps (generation group 2).
    maps: wgpu::BindGroup,
    terrain_draw: usize,
    arrayed: bool,
    slices: u32,
    cast: bool,
}

/// The fields to generate and draw this frame.
#[derive(Default)]
pub(crate) struct GrassFrame {
    fields: Vec<GrassField>,
}

impl GrassFrame {
    /// Whether a field of the frame casts shadows.
    pub(crate) fn casts(&self) -> bool {
        self.fields.iter().any(|f| f.cast)
    }
}

/// Which pass grass draws into.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrassPass {
    Lit,
    Prepass,
    Shadow,
}

/// What [`GrassRenderer::new`] builds its pipelines from.
pub(crate) struct GrassPipelines<'a> {
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub shadow_layout: &'a wgpu::BindGroupLayout,
    pub fx_layout: &'a wgpu::BindGroupLayout,
    pub samples: u32,
    /// The simulated wind field's lean (`wind_field.rs`), read by the generation.
    pub wind_field: wgpu::TextureView,
    /// The trample map (`trample.rs`), read by the generation.
    pub trample: wgpu::TextureView,
}

/// What a frame's grass is prepared for: the main camera.
pub(crate) struct GrassView<'a> {
    pub camera: Vec3,
    pub frame: &'a crate::frame::CameraFrame,
    pub prev_view_proj: Mat4,
    pub height: u32,
    /// The simulated wind field's grid this frame (`wind_field.rs`).
    pub wind_grid: crate::wind_field::WindGrid,
    /// The trample map this frame (`trample.rs` `Trample::grid`).
    pub trample: [f32; 4],
}

/// Generates and draws [`GrassDraw`]s.
pub(crate) struct GrassRenderer {
    gen_layout: wgpu::BindGroupLayout,
    /// The generation's group 2: a field's type, density and height maps
    /// and their sampler; `white` stands in for a map a field lacks.
    maps_layout: wgpu::BindGroupLayout,
    white: wgpu::TextureView,
    map_sampler: wgpu::Sampler,
    draw_layout: wgpu::BindGroupLayout,
    /// The simulated wind field's lean (`wind_field.rs`) and the trample map.
    wind_field: wgpu::TextureView,
    trample: wgpu::TextureView,
    lit_layout: wgpu::PipelineLayout,
    prepass_layout: wgpu::PipelineLayout,
    shadow_layout: wgpu::PipelineLayout,
    samples: u32,
    /// The lit, prepass and shadow pipelines, made when a field first draws.
    pipelines: Option<[wgpu::RenderPipeline; 3]>,
    /// The generation pipelines (evaluate, scan, compact) over each
    /// terrain layout, standard then layer arrays.
    generate: [Option<[wgpu::ComputePipeline; 3]>; 2],
    tile: wgpu::Buffer,
    indices: wgpu::Buffer,
    fields: Vec<FieldGpu>,
    /// The tick the last frame drew (the motion vectors' wind).
    last_tick: Option<u64>,
}

impl GrassRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: GrassPipelines,
    ) -> GrassRenderer {
        use wgpu::util::DeviceExt;
        let entry = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        };
        let buffer = |ty| wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let cs = wgpu::ShaderStages::COMPUTE;
        let rw = wgpu::BufferBindingType::Storage { read_only: false };
        let gen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass generate"),
            entries: &[
                entry(0, cs, buffer(wgpu::BufferBindingType::Uniform)),
                entry(1, cs, buffer(wgpu::BufferBindingType::Uniform)),
                entry(
                    2,
                    cs,
                    buffer(wgpu::BufferBindingType::Storage { read_only: true }),
                ),
                entry(3, cs, buffer(rw)),
                entry(4, cs, buffer(rw)),
                entry(5, cs, buffer(rw)),
                entry(6, cs, buffer(rw)),
                entry(
                    7,
                    cs,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    8,
                    cs,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let map = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };
        let maps_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass maps"),
            entries: &[
                entry(0, cs, map),
                entry(1, cs, map),
                entry(2, cs, map),
                entry(
                    3,
                    cs,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });
        let white = device
            .create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    label: Some("grass no map"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &[255; 4],
            )
            .create_view(&Default::default());
        let map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("grass maps"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass"),
            entries: &[
                entry(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    buffer(wgpu::BufferBindingType::Uniform),
                ),
                entry(
                    1,
                    wgpu::ShaderStages::VERTEX,
                    buffer(wgpu::BufferBindingType::Storage { read_only: true }),
                ),
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
        let tile: Vec<[f32; 2]> = blue_noise_tile().iter().map(|p| p.to_array()).collect();
        GrassRenderer {
            lit_layout: pl("grass lit", &[p.frame_layout, &draw_layout, p.fx_layout]),
            prepass_layout: pl("grass prepass", &[p.frame_layout, &draw_layout]),
            shadow_layout: pl("grass shadow", &[p.shadow_layout, &draw_layout]),
            gen_layout,
            maps_layout,
            white,
            map_sampler,
            draw_layout,
            wind_field: p.wind_field,
            trample: p.trample,
            samples: p.samples,
            pipelines: None,
            generate: [None, None],
            tile: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("grass tile"),
                contents: bytemuck::cast_slice(&tile),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("grass indices"),
                contents: bytemuck::cast_slice(&lod_indices()),
                usage: wgpu::BufferUsages::INDEX,
            }),
            fields: Vec::new(),
            last_tick: None,
        }
    }

    /// Draw the lit pass into a target of `samples` per pixel from now on.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        if self.samples != samples {
            self.samples = samples;
            self.pipelines = None;
        }
    }

    fn make_pipelines(&self, device: &wgpu::Device) -> [wgpu::RenderPipeline; 3] {
        let module = |label, src: String| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            })
        };
        let lit_module = module("grass", lit_source());
        let shadow_module = module("grass shadow", shadow_source());
        let lit = crate::hdr_pipeline(
            device,
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "grass",
                layout: &self.lit_layout,
                module: &lit_module,
                vs: "vs",
                fs: "fs",
                buffers: &[],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples: self.samples,
            },
        );
        let depth = Some(wgpu::DepthStencilState {
            format: crate::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let raster = |label, layout, module, vs, fs, targets: &[Option<wgpu::ColorTargetState>]| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: depth.clone(),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets,
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let prepass = raster(
            "grass prepass",
            &self.prepass_layout,
            &lit_module,
            "vs",
            "fs_prepass",
            &[
                target(crate::post::GBUFFER_FORMAT),
                target(crate::post::LINDEPTH_FORMAT),
                target(crate::post::OUTLINE_FORMAT),
            ],
        );
        let shadow = raster(
            "grass shadow",
            &self.shadow_layout,
            &shadow_module,
            "vs_shadow",
            "fs_shadow",
            &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        );
        [lit, prepass, shadow]
    }

    fn make_generate(
        &self,
        device: &wgpu::Device,
        terrain_layout: &wgpu::BindGroupLayout,
        arrayed: bool,
    ) -> [wgpu::ComputePipeline; 3] {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grass generate"),
            source: wgpu::ShaderSource::Wgsl(generate_source(arrayed).into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grass generate"),
            bind_group_layouts: &[
                Some(&self.gen_layout),
                Some(terrain_layout),
                Some(&self.maps_layout),
            ],
            immediate_size: 0,
        });
        ["evaluate", "scan", "compact"].map(|entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("grass generate"),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        })
    }

    /// A field's buffers holding at least `slices` slices (made again,
    /// larger, when not).
    fn field_gpu(&mut self, device: &wgpu::Device, slot: usize, slices: u32) {
        if self.fields.get(slot).is_some_and(|f| f.slice_cap >= slices) {
            return;
        }
        let slice_cap = slices.next_power_of_two().clamp(64, MAX_SLICES);
        let blade_cap = (slice_cap * SLICE).min(MAX_BLADES);
        let make = |label, size: u64, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let uniform = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let generator = make(
            "grass generate",
            std::mem::size_of::<GenGpu>() as u64,
            uniform,
        );
        let draw = make("grass", std::mem::size_of::<DrawGpu>() as u64, uniform);
        let slices_buf = make("grass slices", slice_cap as u64 * 16, storage);
        let scratch = make(
            "grass scratch",
            slice_cap as u64 * SLICE as u64 * 16,
            wgpu::BufferUsages::STORAGE,
        );
        let counts = make(
            "grass counts",
            slice_cap as u64 * 12,
            wgpu::BufferUsages::STORAGE,
        );
        let instances = make(
            "grass blades",
            (blade_cap as u64 + 1) * 16,
            wgpu::BufferUsages::STORAGE,
        );
        let draws = make(
            "grass draws",
            60,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
        );
        let group = |label, layout, buffers: &[&wgpu::Buffer]| {
            let entries: Vec<_> = buffers
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &entries,
            })
        };
        let mut gen_entries: Vec<_> = [
            &generator,
            &self.tile,
            &slices_buf,
            &scratch,
            &counts,
            &instances,
            &draws,
        ]
        .iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: b.as_entire_binding(),
        })
        .collect();
        gen_entries.push(wgpu::BindGroupEntry {
            binding: 7,
            resource: wgpu::BindingResource::TextureView(&self.wind_field),
        });
        gen_entries.push(wgpu::BindGroupEntry {
            binding: 8,
            resource: wgpu::BindingResource::TextureView(&self.trample),
        });
        let gen_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grass generate"),
            layout: &self.gen_layout,
            entries: &gen_entries,
        });
        let draw_group = group("grass", &self.draw_layout, &[&draw, &instances]);
        let f = FieldGpu {
            generator,
            draw,
            slices: slices_buf,
            draws,
            slice_cap,
            blade_cap,
            gen_group,
            draw_group,
        };
        if slot < self.fields.len() {
            self.fields[slot] = f;
        } else {
            self.fields.push(f);
        }
    }

    /// Plans this frame's fields over the drawn terrains and writes their
    /// uniforms and slices.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &crate::RenderScene,
        terrain: &crate::terrain::TerrainRenderer,
        terrain_draws: &[crate::terrain::TerrainDraw],
        view: GrassView,
        frustum: &crate::frame::Frustum,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> GrassFrame {
        let mut frame = GrassFrame::default();
        let previous = self.last_tick.unwrap_or(scene.tick);
        self.last_tick = (!scene.grass.is_empty()).then_some(scene.tick);
        let wind = crate::wind::field_gpu(
            scene.wind,
            scene.tick,
            previous,
            scene.tick_rate,
            scene.origin,
            view.wind_grid,
        );
        let l = scene.light.direction_to_light.normalize_or(Vec3::Y);
        let sun_on = scene.light.intensity > 0.0;
        // Where a blade's shadow lands, per metre of its height.
        let reach = Vec3::new(-l.x, -l.y, -l.z) / l.y.max(0.2);
        let lod_rows = LOD_ROWS;
        for d in &scene.grass {
            let g = d.grass;
            let Some(t) = scene.terrains.get(d.terrain) else {
                continue;
            };
            let Some(k) = terrain_draws.iter().position(|td| td.terrain == d.terrain) else {
                continue;
            };
            let td = &terrain_draws[k];
            let Some((size, range)) = terrain.extent_of(t.terrain.index()) else {
                continue;
            };
            if !g.usable() || (g.layer as usize) >= t.layers.len().max(1) {
                continue;
            }
            let Some(terrain_layout) = terrain.group_layout(td.arrayed) else {
                continue;
            };
            let a = usize::from(td.arrayed);
            if self.generate[a].is_none() {
                self.generate[a] = Some(self.make_generate(device, terrain_layout, td.arrayed));
            }
            let cast = g.cast_shadows && scene.light.cast_shadows && sun_on;
            // With its shadow: the box swept along the sun's reach.
            let shadow = reach * g.tallest() * f32::from(u8::from(cast));
            let visible = |lo: Vec3, hi: Vec3| {
                frustum.intersects_box(lo.min(lo + shadow), hi.max(hi + shadow))
            };
            let Plan { slices, density } = plan(
                &g,
                &PlanView {
                    model: t.transform,
                    size,
                    range,
                    camera: view.camera,
                    orthographic: view.frame.orthographic,
                    visible: &visible,
                },
            );
            if slices.is_empty() {
                continue;
            }
            let slot = frame.fields.len();
            self.field_gpu(device, slot, slices.len() as u32);
            let f = &self.fields[slot];
            let along = horizontal_scale(t.transform);
            let layout = FieldLayout::new(&g, along);
            let lod = DistanceLod::for_radius(g.distance);
            let extent = g.distance + g.tallest() * 4.0 + CHUNK;
            let origin = view.camera.extend(extent).to_array();
            let planes = frustum.planes();
            let generator = GenGpu {
                planes: std::array::from_fn(|i| {
                    planes.get(i).copied().unwrap_or(Vec4::ZERO).to_array()
                }),
                eye: view
                    .camera
                    .extend(f32::from(u8::from(view.frame.orthographic)))
                    .to_array(),
                origin,
                lod: [lod.lod_start[0], lod.lod_start[1], lod.band, lod.radius],
                lod_density: [
                    lod.density_at[0],
                    lod.density_at[1],
                    lod.mid_range,
                    lod.fade_start,
                ],
                field: [
                    layout.candidates,
                    layout.density,
                    layout.tile,
                    layout.tiles_per_side as f32,
                ],
                chunk: [layout.chunk, g.layer as f32, seed_bits(g.seed), lod.widest],
                shape: [
                    g.tallest(),
                    g.height_variation.clamp(0.0, 1.0),
                    g.lean,
                    g.clump_lean,
                ],
                clump: [
                    g.clump_size.max(0.05) / along,
                    g.tufts.clamp(0.0, 1.0),
                    HEIGHT_RANGE,
                    g.widest(),
                ],
                shadow: reach.extend(f32::from(u8::from(cast))).to_array(),
                motion: [
                    g.sway.max(0.0),
                    g.stiffness.clamp(0.0, 1.0),
                    density,
                    g.recovery.max(0.0),
                ],
                trample: view.trample,
                kinds: kinds_gen(&g),
                maps: [
                    f32::from(u8::from(g.type_map.is_some())),
                    f32::from(u8::from(g.density_map.is_some())),
                    f32::from(u8::from(g.height_map.is_some())),
                    0.0,
                ],
                counts: [
                    slices.len() as u32,
                    f.blade_cap,
                    lod_rows[0],
                    lod_rows[1] | (lod_rows[2] << 8),
                ],
                wind,
            };
            let proj = view.frame.proj;
            let per_pixel = 2.0 / (proj.y_axis.y.abs().max(1e-6) * view.height.max(1) as f32);
            let [base_color, tip_color, dry_color, variation] = look_gpu(&g);
            let draw = DrawGpu {
                view_proj: view.frame.view_proj.to_cols_array_2d(),
                prev_view_proj: view.prev_view_proj.to_cols_array_2d(),
                eye: generator.eye,
                forward: view.frame.forward.extend(per_pixel).to_array(),
                origin,
                limits: [HEIGHT_RANGE, MIN_WIDTH_PX, TURN_TO_EYE, 0.0],
                sun: l.extend(f32::from(u8::from(sun_on))).to_array(),
                shape: [
                    g.tallest(),
                    g.widest(),
                    g.curve.clamp(0.0, 1.0),
                    g.flutter.max(0.0),
                ],
                base_color,
                tip_color,
                dry_color,
                variation,
                rows: [lod_rows[0], lod_rows[1], lod_rows[2], 0],
                // The canopy the sun crosses to reach a blade's lower part
                // (grass.wgsl grass_canopy): its extinction per metre and
                // its top, the field's mean blade height.
                canopy: [
                    g.density.max(0.0) * g.width.max(0.0) * CANOPY_BLOCKING,
                    g.height * (1.0 - 0.5 * g.height_variation.clamp(0.0, 1.0)),
                    0.0,
                    0.0,
                ],
                kinds: kinds_draw(&g),
                wind,
            };
            queue.write_buffer(&f.generator, 0, bytemuck::bytes_of(&generator));
            queue.write_buffer(&f.draw, 0, bytemuck::bytes_of(&draw));
            queue.write_buffer(&f.slices, 0, bytemuck::cast_slice(&slices));
            let view_of =
                |m: Option<crate::TextureHandle>| m.and_then(textures).unwrap_or(&self.white);
            let maps = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("grass maps"),
                layout: &self.maps_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view_of(g.type_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(view_of(g.density_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(view_of(g.height_map)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.map_sampler),
                    },
                ],
            });
            frame.fields.push(GrassField {
                slot,
                maps,
                terrain_draw: k,
                arrayed: td.arrayed,
                slices: slices.len() as u32,
                cast,
            });
        }
        if !frame.fields.is_empty() && self.pipelines.is_none() {
            self.pipelines = Some(self.make_pipelines(device));
        }
        frame
    }

    /// Generates this frame's blades (before every pass that draws them).
    pub(crate) fn encode(
        &self,
        encoder: &mut crate::Encoder,
        frame: &GrassFrame,
        terrain_draws: &[crate::terrain::TerrainDraw],
    ) {
        if frame.fields.is_empty() {
            return;
        }
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("grass"),
            timestamp_writes: None,
        });
        for field in &frame.fields {
            let Some([evaluate, scan, compact]) = &self.generate[usize::from(field.arrayed)] else {
                continue;
            };
            let f = &self.fields[field.slot];
            pass.set_bind_group(0, &f.gen_group, &[]);
            pass.set_bind_group(1, &terrain_draws[field.terrain_draw].group, &[0]);
            pass.set_bind_group(2, &field.maps, &[]);
            pass.set_pipeline(evaluate);
            pass.dispatch_workgroups(field.slices, 1, 1);
            pass.set_pipeline(scan);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(compact);
            pass.dispatch_workgroups(field.slices, 1, 1);
        }
    }

    /// Draws this frame's fields into a pass whose group 0 (and, lit, group
    /// 2: `fx`) the caller bound. Answers the draws made.
    pub(crate) fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        kind: GrassPass,
        frame: &GrassFrame,
        fx: Option<&wgpu::BindGroup>,
    ) -> u32 {
        let Some(pipelines) = &self.pipelines else {
            return 0;
        };
        let mut draws = 0;
        for field in &frame.fields {
            if kind == GrassPass::Shadow && !field.cast {
                continue;
            }
            let f = &self.fields[field.slot];
            pass.set_pipeline(&pipelines[kind as usize]);
            pass.set_bind_group(1, &f.draw_group, &[]);
            if kind == GrassPass::Lit
                && let Some(fx) = fx
            {
                pass.set_bind_group(2, fx, &[]);
            }
            pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
            for l in 0..3u64 {
                pass.draw_indexed_indirect(&f.draws, l * 20);
                draws += 1;
            }
        }
        draws
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// grass_common.wgsl GRASS_LOD_STRIDE: vertex indices between levels.
    const LOD_STRIDE: u32 = 64;

    /// Every prefix of the tile is spread evenly: its closest pair is
    /// never much closer than an even spread of that many points would put
    /// them (so thinning by rank does not clump).
    #[test]
    fn every_prefix_of_the_tile_is_even() {
        let tile = blue_noise_tile();
        assert_eq!(tile.len(), TILE_POINTS as usize);
        for n in [16usize, 64, 256, 1024] {
            let mut closest = f32::INFINITY;
            for i in 0..n {
                for j in 0..i {
                    let d = (tile[i] - tile[j]).abs();
                    closest = closest.min(d.min(Vec2::ONE - d).length());
                }
            }
            // A hexagonal packing of n points has spacing sqrt(2 / (sqrt(3) n));
            // best candidate keeps about half of it, white noise about 1/sqrt(n) of it.
            let even = (2.0 / (3f32.sqrt() * n as f32)).sqrt();
            assert!(
                closest > 0.35 * even,
                "{n} points: closest {closest}, even {even}"
            );
        }
        // And it is the same tile every time (a fixed seed).
        assert_eq!(tile[0], blue_noise_tile()[0]);
    }

    /// Full density to LOD 1, the table's shares where LOD 2 and the fade
    /// start, nothing at the radius; levels change inside their bands only.
    #[test]
    fn the_distance_table_thins_to_nothing_at_the_radius() {
        let l = DistanceLod::for_radius(150.0);
        assert_eq!(l.falloff(10.0), 1.0);
        // Past LOD 1's band, the mid-range share on top of the table.
        assert!((l.falloff(75.0) - 0.5 * 0.5).abs() < 1e-5);
        assert!((l.falloff(120.0) - 0.15 * 0.5).abs() < 1e-5);
        assert_eq!(l.falloff(150.0), 0.0);
        let mut last = 2.0;
        for d in 0..150 {
            let f = l.falloff(d as f32);
            assert!(f <= last, "rises at {d}");
            last = f;
        }
        for dither in [0.0, 0.5, 0.999] {
            assert_eq!(l.lod(27.9, dither), 0);
            assert_eq!(l.lod(32.1, dither), 1);
            assert_eq!(l.lod(77.1, dither), 2);
        }
        // Inside the band the dither picks.
        assert_eq!(l.lod(30.0, 0.2), 1);
        assert_eq!(l.lod(30.0, 0.8), 0);
    }

    /// The tiling never has fewer candidates than the field wants.
    #[test]
    fn a_layout_has_the_candidates_its_density_needs() {
        for density in [1.0, 45.0, 300.0, 1000.0] {
            let g = Grass {
                density,
                ..Default::default()
            };
            let l = FieldLayout::new(&g, 1.0);
            assert!(l.candidates >= density, "{density}: {l:?}");
            assert!((l.tile * l.tiles_per_side as f32 - CHUNK).abs() < 1e-3);
        }
    }

    fn view(camera: Vec3, visible: &dyn Fn(Vec3, Vec3) -> bool) -> PlanView<'_> {
        PlanView {
            model: Mat4::IDENTITY,
            size: Vec2::splat(1024.0),
            range: (0.0, 10.0),
            camera,
            orthographic: false,
            visible,
        }
    }

    /// Chunks come nearest first, each slice within its chunk's prefix of
    /// candidates; chunks past the distance or out of view are left out;
    /// the same frame plans the same slices.
    #[test]
    fn the_plan_is_nearest_first_and_repeatable() {
        let g = Grass::default();
        // A quarter of the field round the camera, as a wide view sees.
        let ahead = |_: Vec3, hi: Vec3| hi.z > -7.0 && hi.x > 3.0;
        let a = plan(&g, &view(Vec3::new(3.0, 2.0, -7.0), &ahead));
        let b = plan(&g, &view(Vec3::new(3.0, 2.0, -7.0), &ahead));
        assert_eq!(a, b);
        assert_eq!(a.density, 1.0, "a quarter of the field fits the budget");
        let a = a.slices;
        assert!(!a.is_empty() && a.len() <= MAX_SLICES as usize);
        let half = 512.0;
        let eye = Vec2::new(3.0 + half, -7.0 + half);
        let dist = |s: &[u32; 4]| {
            let lo = Vec2::new(s[0] as f32, s[1] as f32) * CHUNK;
            (eye.clamp(lo, lo + CHUNK) - eye).length()
        };
        let mut last = 0.0;
        for s in &a {
            assert!(s[3] > 0 && s[3] <= SLICE);
            let d = dist(s);
            assert!(d < g.distance);
            assert!(d + 1e-3 >= last, "a farther chunk first");
            last = d;
        }
        // The camera's own chunk keeps nearly every candidate.
        let layout = FieldLayout::new(&g, 1.0);
        let own: u32 = a.iter().filter(|s| dist(s) == 0.0).map(|s| s[3]).sum();
        let cells = layout.tiles_per_side.pow(2);
        let wanted = (g.density / layout.candidates * TILE_POINTS as f32).ceil() as u32 * cells;
        assert_eq!(own, wanted);
        // Looking all round keeps more.
        let all = |_: Vec3, _: Vec3| true;
        assert!(
            plan(&g, &view(Vec3::new(3.0, 2.0, -7.0), &all))
                .slices
                .len()
                > a.len()
        );
    }

    /// A view holding more of the field than [`MAX_BLADES`] (all round,
    /// dense) thins every chunk evenly instead of dropping the far ones:
    /// the field still reaches its distance, within the budget.
    #[test]
    fn an_over_budget_view_thins_the_field_and_keeps_its_edge() {
        let g = Grass {
            density: 600.0,
            ..Default::default()
        };
        let all = |_: Vec3, _: Vec3| true;
        let p = plan(&g, &view(Vec3::ZERO, &all));
        assert!(p.density < 1.0, "{}", p.density);
        let kept: u64 = p.slices.iter().map(|s| u64::from(s[3])).sum();
        let chunks = p
            .slices
            .iter()
            .map(|s| (s[0], s[1]))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            kept <= u64::from(MAX_BLADES) + chunks.len() as u64 * 81,
            "{kept}"
        );
        let eye = Vec2::splat(512.0);
        let farthest = p
            .slices
            .iter()
            .map(|s| {
                let lo = Vec2::new(s[0] as f32, s[1] as f32) * CHUNK;
                (eye.clamp(lo, lo + CHUNK) - eye).length()
            })
            .fold(0.0f32, f32::max);
        assert!(
            farthest > g.distance - CHUNK * 1.5,
            "the field ends at {farthest} m"
        );
    }

    /// A field's kinds split the clumps by their shares in order, the
    /// field's own first: a missing kind takes none and the last bound is
    /// always 1, so every clump grows a kind the field has.
    #[test]
    fn the_kinds_split_the_clumps_by_their_shares() {
        let reeds = GrassType {
            height: 1.6,
            width: 0.02,
            share: 3.0,
            ..Default::default()
        };
        let g = Grass {
            types: [
                Some(reeds),
                None,
                Some(GrassType {
                    share: 0.0,
                    ..reeds
                }),
            ],
            ..Default::default()
        };
        let k = kinds_gen(&g);
        let bounds: Vec<f32> = k.iter().map(|r| r[2]).collect();
        assert_eq!(bounds, vec![0.25, 1.0, 1.0, 1.0]);
        assert_eq!(k.map(|r| r[3]), [1.0, 1.0, 0.0, 1.0]);
        assert_eq!((k[1][0], k[1][1]), (1.6, 0.02));
        assert_eq!(g.tallest(), 1.6);
        assert_eq!(g.widest(), 0.02);
        let d = kinds_draw(&g);
        assert_eq!(d[2][3], reeds.curve);
        // A missing kind draws as the field's own, never black.
        assert_eq!(d[4], d[0]);
    }

    /// Each uniform the grass writes has the layout its shader reads
    /// (naga's offsets of the WGSL struct against the Rust one's), and
    /// the look's values land where the shader reads them: translucency
    /// in `base_color.w`, which grass.wgsl's backlit term multiplies.
    #[test]
    fn the_uniforms_reach_the_shaders() {
        use std::mem::{offset_of, size_of};
        fn members(src: &str, name: &str) -> (Vec<(String, u32)>, u32) {
            let module = naga::front::wgsl::parse_str(src).expect("parses");
            let mut layouter = naga::proc::Layouter::default();
            layouter.update(module.to_ctx()).expect("lays out");
            let (handle, ty) = module
                .types
                .iter()
                .find(|(_, t)| t.name.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("{name} in the shader"));
            let naga::TypeInner::Struct { members, span } = &ty.inner else {
                panic!("{name} is a struct");
            };
            assert_eq!(*span, layouter[handle].size);
            let list = members
                .iter()
                .map(|m| (m.name.clone().unwrap_or_default(), m.offset))
                .collect();
            (list, *span)
        }
        let check = |src: &str, name: &str, rust: &[(&str, usize)], size: usize| {
            let (wgsl, span) = members(src, name);
            let names: Vec<&str> = wgsl.iter().map(|(n, _)| n.as_str()).collect();
            let expected: Vec<&str> = rust.iter().map(|(n, _)| *n).collect();
            assert_eq!(names, expected, "{name}'s members");
            for ((n, at), (_, want)) in wgsl.iter().zip(rust) {
                assert_eq!(*at as usize, *want, "{name}.{n}");
            }
            assert_eq!(span as usize, size, "{name}'s size");
        };
        macro_rules! fields {
            ($t:ty: $($f:ident),*) => { &[$((stringify!($f), offset_of!($t, $f))),*] };
        }
        check(
            &lit_source(),
            "GrassUniform",
            fields!(DrawGpu: view_proj, prev_view_proj, eye, forward, origin, limits, sun, shape,
                base_color, tip_color, dry_color, variation, rows, canopy, kinds, wind),
            size_of::<DrawGpu>(),
        );
        check(
            &generate_source(false),
            "GrassGen",
            fields!(GenGpu: planes, eye, origin, lod, lod_density, field, chunk, shape, clump,
                shadow, motion, counts, trample, kinds, maps, wind),
            size_of::<GenGpu>(),
        );
        check(
            &lit_source(),
            "WindField",
            fields!(WindFieldGpu: dir, phase, offset, time, grid, grid_outer),
            size_of::<WindFieldGpu>(),
        );
        check(
            &generate_source(false),
            "TerrainGrass",
            fields!(FarGpu: color, fade, clump, dry, wind),
            size_of::<FarGpu>(),
        );
        // Translucency, sheen, roughness and the look's numbers land in the
        // members the shaders read them from.
        let g = Grass {
            translucency: 0.6,
            sheen: 0.3,
            roughness: 0.7,
            dryness: 0.2,
            ..Default::default()
        };
        let [base, tip, dry, variation] = look_gpu(&g);
        assert_eq!(base, g.base_color.extend(0.6).to_array());
        assert_eq!(tip[3], 0.3);
        assert_eq!(dry[3], 0.7);
        assert_eq!(variation[1], 0.2);
        let opaque = look_gpu(&Grass {
            translucency: 0.0,
            ..g
        });
        assert_eq!(opaque[0][3], 0.0);
    }

    /// The level strips: 13, 5 and 1 triangles, every index inside its
    /// level's strip (rows · 2 + 1 vertices, within the stride).
    #[test]
    fn the_level_strips_fit_their_stride() {
        let idx = lod_indices();
        let mut at = 0;
        for rows in LOD_ROWS {
            let n = 3 * (2 * rows - 1) as usize;
            assert!(
                idx[at..at + n]
                    .iter()
                    .all(|&i| i <= 2 * rows && i < LOD_STRIDE)
            );
            at += n;
        }
        assert_eq!(at, idx.len());
        assert_eq!(idx.len(), 3 * (13 + 5 + 1));
    }
}
