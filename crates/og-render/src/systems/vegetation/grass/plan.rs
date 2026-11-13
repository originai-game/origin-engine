//! Where a view's grass goes: the blue-noise tile, the distance table and the chunks a view draws.

use super::*;

/// PCG hash (Jarzynski and Olano, 2020): wind_common.wgsl `wind_hash`.
pub fn hash(x: u32) -> u32 {
    let state = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

pub(super) fn unit(h: u32) -> f32 {
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
pub(super) fn world_box(model: Mat4, lo: Vec3, hi: Vec3) -> (Vec3, Vec3) {
    (0..8).fold((Vec3::MAX, Vec3::MIN), |(a, b), i| {
        let p = model.transform_point3(Vec3::new(
            if i & 1 == 0 { lo.x } else { hi.x },
            if i & 2 == 0 { lo.y } else { hi.y },
            if i & 4 == 0 { lo.z } else { hi.z },
        ));
        (a.min(p), b.max(p))
    })
}
