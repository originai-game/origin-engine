//! Procedural terrain heightfield and biome control-map generator:
//! fractal gradient noise with domain warp, island or ridge mask,
//! hydraulic erosion by seeded drops, CFL-stable mass-conserving thermal
//! erosion, terraces, a sea level with its shore flattened, rivers carved
//! where flow gathers, and biome rules writing the control map.
//!
//! **Determinism.** Every step uses only IEEE-exact arithmetic (`+`, `-`,
//! `*`, `/`, `sqrt`, `floor`, `min`, `max`, comparisons), integer hashes
//! and PCG32 for drops — no platform `libm` calls, no hash-map iteration
//! order, no threads. The same inputs make the same heightfield bit for
//! bit on every host.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::terrain::MAX_TERRAIN_LAYERS;

/// A terrain generation specification (shared by `og-cook`'s `generate {}`
/// block and `og-runtime`'s `og.procgen.heightmap` API).
#[derive(Clone, Debug, PartialEq)]
pub struct Generate {
    pub columns: u32,
    pub rows: u32,
    pub seed: u32,
    pub octaves: u32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub warp: f32,
    pub mask: Mask,
    pub erosion: Option<Hydraulic>,
    pub thermal: Option<Thermal>,
    pub terraces: Option<[f32; 2]>,
    pub sea: Option<f32>,
    pub shore: f32,
    pub rivers: Option<Rivers>,
    pub moisture: Moisture,
    pub biomes: Vec<Biome>,
}

impl Default for Generate {
    fn default() -> Self {
        Generate {
            columns: 257,
            rows: 257,
            seed: 1,
            octaves: 6,
            frequency: 3.0,
            lacunarity: 2.0,
            gain: 0.5,
            warp: 0.0,
            mask: Mask::None,
            erosion: None,
            thermal: None,
            terraces: None,
            sea: None,
            shore: 2.0,
            rivers: None,
            moisture: Moisture::default(),
            biomes: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Mask {
    #[default]
    None,
    /// The share of the half size over which the land fades to the low.
    Island(f32),
    /// How much of the noise is ridged (0 to 1).
    Ridge(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hydraulic {
    pub drops: u32,
    pub lifetime: u32,
    pub radius: u32,
    pub inertia: f32,
    pub capacity: f32,
    pub deposition: f32,
    pub erosion: f32,
    pub evaporation: f32,
    pub gravity: f32,
}

impl Default for Hydraulic {
    fn default() -> Self {
        Hydraulic {
            drops: 40_000,
            lifetime: 30,
            radius: 3,
            inertia: 0.05,
            capacity: 4.0,
            deposition: 0.3,
            erosion: 0.3,
            evaporation: 0.01,
            gravity: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thermal {
    pub talus: f32,
    pub iterations: u32,
    pub rate: f32,
}

impl Default for Thermal {
    fn default() -> Self {
        Thermal {
            talus: 35.0,
            iterations: 20,
            rate: 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rivers {
    pub threshold: u32,
    pub depth: f32,
    pub width: f32,
}

impl Default for Rivers {
    fn default() -> Self {
        Rivers {
            threshold: 400,
            depth: 2.0,
            width: 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moisture {
    pub flow: f32,
    pub scale: f32,
    pub noise: f32,
    pub frequency: f32,
}

impl Default for Moisture {
    fn default() -> Self {
        Moisture {
            flow: 1.0,
            scale: 64.0,
            noise: 0.5,
            frequency: 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Biome {
    pub layer: u32,
    pub weight: f32,
    pub altitude: [f32; 2],
    pub altitude_fade: f32,
    pub slope: [f32; 2],
    pub slope_fade: f32,
    pub moisture: [f32; 2],
    pub moisture_fade: f32,
    /// Optional source line for OGT cook error diagnostics.
    pub line: usize,
}

impl Default for Biome {
    fn default() -> Self {
        Biome {
            layer: 0,
            weight: 1.0,
            altitude: [-1.0e6, 1.0e6],
            altitude_fade: 2.0,
            slope: [0.0, 90.0],
            slope_fade: 4.0,
            moisture: [0.0, 1.0],
            moisture_fade: 0.1,
            line: 0,
        }
    }
}

/// A generated heightfield and optional biome control map.
#[derive(Clone, Debug)]
pub struct Generated {
    /// Metres above the origin, row-major from -z to +z and -x to +x.
    pub heights: Vec<f32>,
    /// The biomes' control map (`columns` × `rows` RGBA8 per page of four
    /// layers, enough pages for the highest layer a biome writes), with
    /// biomes.
    pub control: Option<Vec<u8>>,
    /// The rivers' length in metres (sample to downstream sample).
    pub river_length: f32,
}

pub const NONE: u32 = u32::MAX;

// ── deterministic arithmetic ─────────────────────────────────────────────

/// PCG32 (O'Neill's pcg32, XSH RR), seeded from the terrain's seed.
struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    fn new(seed: u64, stream: u64) -> Pcg32 {
        let mut p = Pcg32 {
            state: 0,
            inc: (stream << 1) | 1,
        };
        p.next();
        p.state = p.state.wrapping_add(seed);
        p.next();
        p
    }

    fn next(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// In 0..1 (24 bits, exact).
    fn unit(&mut self) -> f32 {
        (self.next() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}

/// An integer hash of a lattice point (murmur3's finaliser over a mix).
fn hash(ix: i32, iy: i32, seed: u32) -> u32 {
    let mut h = seed ^ 0x9e37_79b9;
    h ^= (ix as u32).wrapping_mul(0x85eb_ca6b);
    h = h.rotate_left(13).wrapping_mul(0xc2b2_ae35);
    h ^= (iy as u32).wrapping_mul(0x27d4_eb2f);
    h = h.rotate_left(15).wrapping_mul(0x1656_67b1);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

const D: f32 = std::f32::consts::FRAC_1_SQRT_2;
const GRADIENTS: [(f32, f32); 8] = [
    (1.0, 0.0),
    (-1.0, 0.0),
    (0.0, 1.0),
    (0.0, -1.0),
    (D, D),
    (-D, D),
    (D, -D),
    (-D, -D),
];

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 2D gradient noise, about -1..1.
fn perlin(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0 as i32, y0 as i32);
    let corner = |dx: i32, dy: i32| {
        let g = GRADIENTS[(hash(ix.wrapping_add(dx), iy.wrapping_add(dy), seed) & 7) as usize];
        g.0 * (fx - dx as f32) + g.1 * (fy - dy as f32)
    };
    let (u, v) = (fade(fx), fade(fy));
    let a = lerp(corner(0, 0), corner(1, 0), u);
    let b = lerp(corner(0, 1), corner(1, 1), u);
    lerp(a, b, v) * std::f32::consts::SQRT_2
}

/// Fractal noise in -1..1 (and, with `ridged`, ridged noise in 0..1).
fn fbm(g: &Generate, x: f32, y: f32, seed: u32, octaves: u32) -> (f32, f32) {
    let (mut v, mut r, mut amp, mut f, mut total) = (0.0, 0.0, 1.0, 1.0, 0.0);
    for o in 0..octaves {
        let n = perlin(x * f, y * f, seed.wrapping_add(o.wrapping_mul(1013)));
        v += n * amp;
        let k = 1.0 - n.abs().min(1.0);
        r += k * k * amp;
        total += amp;
        f *= g.lacunarity;
        amp *= g.gain;
    }
    if total > 0.0 {
        (v / total, r / total)
    } else {
        (0.0, 0.0)
    }
}

/// arctan(x) in degrees, x ≥ 0: two half-angle reductions to |x| ≤
/// tan(π/16), then its series (error under 1e-9 rad).
fn atan_degrees(x: f32) -> f32 {
    let (x, flip) = if x > 1.0 { (1.0 / x, true) } else { (x, false) };
    let x1 = x / (1.0 + (1.0 + x * x).sqrt());
    let x2 = x1 / (1.0 + (1.0 + x1 * x1).sqrt());
    let s = x2 * x2;
    let series = x2
        * (1.0 - s * (1.0 / 3.0 - s * (1.0 / 5.0 - s * (1.0 / 7.0 - s * (1.0 / 9.0 - s / 11.0)))));
    let a = 4.0 * series;
    let a = if flip {
        std::f32::consts::FRAC_PI_2 - a
    } else {
        a
    };
    a * (180.0 / std::f32::consts::PI)
}

/// tan of `degrees` (0 to 89) by the series of sine and cosine.
fn tan_degrees(degrees: f32) -> f32 {
    let x = degrees * (std::f32::consts::PI / 180.0);
    let x2 = x * x;
    let (mut sin, mut cos, mut ts, mut tc) = (0.0f32, 0.0f32, x, 1.0f32);
    for k in 0..9u32 {
        let (a, b) = ((2 * k + 2) as f32, (2 * k + 3) as f32);
        let (c, d) = ((2 * k + 1) as f32, (2 * k + 2) as f32);
        sin += ts;
        cos += tc;
        ts = -ts * x2 / (a * b);
        tc = -tc * x2 / (c * d);
    }
    sin / cos
}

/// Orders f32 as their values (for the flood's queue and sorts).
fn key(f: f32) -> u32 {
    let b = f.to_bits();
    if b >> 31 == 1 { !b } else { b | 0x8000_0000 }
}

/// A band with soft edges, as og-render's layer rules: 1 inside `[a, b]`,
/// fading to 0 over `fade` outside it.
fn band(v: f32, [a, b]: [f32; 2], fade: f32) -> f32 {
    if fade <= 0.0 {
        return if v >= a && v <= b { 1.0 } else { 0.0 };
    }
    smoothstep((v - (a - fade)) / fade) * (1.0 - smoothstep((v - b) / fade))
}

/// The eight neighbours, in a fixed order.
const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

const RIVER_EPS: f32 = 1.0e-3;

// ── stages ───────────────────────────────────────────────────────────────

/// Makes the field of `g` for a terrain of `size` metres whose samples
/// stand for `height` (`[low, high]` metres).
pub fn generate(g: &Generate, size: [f32; 2], height: [f32; 2]) -> Generated {
    let cols = g.columns.clamp(2, 4097) as usize;
    let rows = g.rows.clamp(2, 4097) as usize;
    let n = cols * rows;
    let sx = size[0].max(1e-3);
    let sz = size[1].max(1e-3);
    let cell = [sx / (cols - 1) as f32, sz / (rows - 1) as f32];
    let longer = sx.max(sz);
    let [lo, hi] = [height[0].min(height[1]), height[0].max(height[1])];

    // Noise and mask, normalised to 0..1.
    let mut v = vec![0.0f32; n];
    for r in 0..rows {
        for c in 0..cols {
            let (u, w) = (c as f32 / (cols - 1) as f32, r as f32 / (rows - 1) as f32);
            let mut x = u * sx / longer * g.frequency;
            let mut y = w * sz / longer * g.frequency;
            if g.warp > 0.0 {
                let o = g.octaves.clamp(1, 4);
                let (qx, _) = fbm(g, x + 5.2, y + 1.3, g.seed ^ 0x5bd1_e995, o);
                let (qy, _) = fbm(g, x + 1.7, y + 9.2, g.seed ^ 0x1b87_3593, o);
                x += g.warp * qx;
                y += g.warp * qy;
            }
            let (f, ridged) = fbm(g, x, y, g.seed, g.octaves.clamp(1, 12));
            let mut h = 0.5 + 0.5 * f;
            match g.mask {
                Mask::None => {}
                Mask::Ridge(s) => h = lerp(h, ridged, s.clamp(0.0, 1.0)),
                Mask::Island(falloff) => {
                    let (dx, dz) = (u * 2.0 - 1.0, w * 2.0 - 1.0);
                    let d = (dx * dx + dz * dz).sqrt();
                    h *= smoothstep((1.0 - d) / falloff.max(1e-4));
                }
            }
            v[r * cols + c] = h;
        }
    }
    let (vmin, vmax) = v
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    let span = vmax - vmin;
    let mut h: Vec<f32> = v
        .iter()
        .map(|&x| {
            let t = if span > 0.0 { (x - vmin) / span } else { 0.0 };
            lo + t * (hi - lo)
        })
        .collect();

    if let Some(e) = &g.erosion {
        hydraulic(&mut h, cols, rows, (cell[0] + cell[1]) * 0.5, e, g.seed);
    }
    if let Some(t) = &g.thermal {
        thermal(&mut h, cols, rows, cell, t);
    }
    if let Some([step, blend]) = g.terraces
        && step > 0.0
    {
        for x in &mut h {
            let s = (*x - lo) / step;
            let k = s.floor();
            let f = s - k;
            let ramp = if blend > 0.0 {
                smoothstep((f - (1.0 - blend)) / blend)
            } else {
                0.0
            };
            *x = lo + (k + ramp) * step;
        }
    }
    if let Some(sea) = g.sea
        && g.shore > 0.0
    {
        for x in &mut h {
            // d · (2t - t²), t = |d| / shore: flat at the sea level, the
            // land's own slope again at the band's edge, monotone between.
            let d = *x - sea;
            let t = d.abs() / g.shore;
            if t < 1.0 {
                let flat = sea + d * (t * (2.0 - t));
                // The sea stays below its level, however near it rounds.
                *x = if d < 0.0 {
                    flat.min(sea.next_down())
                } else {
                    flat
                };
            }
        }
    }

    // The flow: routed over the depressions filled (priority flood with ε)
    // to the sea and the edge, gathered from the highest sample down.
    let routed = g.rivers.is_some() || !g.biomes.is_empty();
    let (downstream, order, gathered) = if routed {
        flow(&h, cols, rows, cell, g.sea)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };
    let mut river_length = 0.0f32;
    if let Some(rv) = &g.rivers {
        let sea = g.sea.unwrap_or(f32::MIN);
        let threshold = rv.threshold.max(1);
        let is_river: Vec<bool> = (0..n)
            .map(|i| gathered[i] >= threshold && h[i] >= sea && downstream[i] != NONE)
            .collect();
        let full = |i: usize| {
            (gathered[i] as f32 / (threshold as f32 * 8.0))
                .min(1.0)
                .sqrt()
        };
        // Each river sample's bed, then lowered so it falls downstream.
        let mut bed: Vec<f32> = h.clone();
        for &i in &order {
            let i = i as usize;
            if is_river[i] {
                bed[i] = bed[i].min(h[i] - rv.depth * full(i));
                let d = downstream[i] as usize;
                bed[d] = bed[d].min((bed[i] - RIVER_EPS).min(bed[i].next_down()));
            }
        }
        // Carved across: a parabola from the bed to the ground at half the width.
        let ground = h.clone();
        for &i in &order {
            let i = i as usize;
            if !is_river[i] {
                continue;
            }
            let half = (rv.width * 0.5 * full(i)).max(cell[0].min(cell[1]) * 0.5);
            let (rc, rr) = (
                (half / cell[0]).floor() as i32,
                (half / cell[1]).floor() as i32,
            );
            let (c0, r0) = ((i % cols) as i32, (i / cols) as i32);
            for dr in -rr..=rr {
                for dc in -rc..=rc {
                    let (c, r) = (c0 + dc, r0 + dr);
                    if c < 0 || r < 0 || c >= cols as i32 || r >= rows as i32 {
                        continue;
                    }
                    let (x, z) = (dc as f32 * cell[0], dr as f32 * cell[1]);
                    let t = (x * x + z * z) / (half * half);
                    if t > 1.0 {
                        continue;
                    }
                    let q = r as usize * cols + c as usize;
                    let carved = bed[i] + (ground[q] - bed[i]).max(0.0) * t;
                    h[q] = h[q].min(carved);
                }
            }
        }
        // Every river sample above the next one down, to the sea.
        for &i in &order {
            let i = i as usize;
            if is_river[i] {
                let d = downstream[i] as usize;
                h[d] = h[d].min((h[i] - RIVER_EPS).min(h[i].next_down()));
            }
        }
        for i in 0..n {
            if is_river[i] {
                let d = downstream[i] as usize;
                let (dc, dr) = (
                    (d % cols) as f32 - (i % cols) as f32,
                    (d / cols) as f32 - (i / cols) as f32,
                );
                river_length += (dc * dc * cell[0] * cell[0] + dr * dr * cell[1] * cell[1]).sqrt();
            }
        }
    }

    // No sample leaves the range `height` names (erosion digs, rivers carve).
    for x in &mut h {
        *x = x.clamp(lo, hi);
    }
    let control =
        (!g.biomes.is_empty()).then(|| biomes(g, &h, &gathered, cols, rows, cell, longer));
    Generated {
        heights: h,
        control,
        river_length,
    }
}

/// Hydraulic erosion (after Beyer 2015 and Lague's drops): each seeded
/// drop runs down the bilinear gradient with inertia, eroding a brush of
/// samples around it while it can carry more and depositing where it
/// carries too much or climbs. Heights are worked in samples (metres over
/// the cell size) so the parameters do not depend on the terrain's scale.
fn hydraulic(h: &mut [f32], cols: usize, rows: usize, cell: f32, e: &Hydraulic, seed: u32) {
    if e.drops == 0 || e.lifetime == 0 || cols < 2 || rows < 2 {
        return;
    }
    // Scale brush radius and drop lifetime gently with grid resolution
    // relative to a 257-sample baseline so high-resolution terrains still
    // develop coherent gullies without changing drop count cost.
    let res_scale = (((cols.max(rows) - 1) as f32) / 256.0).clamp(0.5, 4.0);
    let r = (((e.radius.max(1) as f32) * res_scale).round() as i32).clamp(1, 24);
    let lifetime = (((e.lifetime as f32) * res_scale).round() as u32).clamp(1, 2000);
    for x in h.iter_mut() {
        *x /= cell;
    }
    let mut brush = Vec::new();
    let mut total = 0.0f32;
    for dy in -r..=r {
        for dx in -r..=r {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if d < r as f32 {
                let w = r as f32 - d;
                brush.push((dx, dy, w));
                total += w;
            }
        }
    }
    if total <= 0.0 {
        brush.push((0, 0, 1.0));
        total = 1.0;
    }
    for b in &mut brush {
        b.2 /= total;
    }
    let sample = |h: &[f32], x: f32, y: f32| -> (f32, f32, f32) {
        let (cx, cy) = (x as usize, y as usize);
        let (ox, oy) = (x - cx as f32, y - cy as f32);
        let i = cy * cols + cx;
        let (nw, ne, sw, se) = (h[i], h[i + 1], h[i + cols], h[i + cols + 1]);
        let gx = (ne - nw) * (1.0 - oy) + (se - sw) * oy;
        let gy = (sw - nw) * (1.0 - ox) + (se - ne) * ox;
        let height = nw * (1.0 - ox) * (1.0 - oy)
            + ne * ox * (1.0 - oy)
            + sw * (1.0 - ox) * oy
            + se * ox * oy;
        (height, gx, gy)
    };
    let (xmax, ymax) = ((cols - 1) as f32, (rows - 1) as f32);
    let mut rng = Pcg32::new(seed as u64, 0x6f67_7465_7272);
    for _ in 0..e.drops {
        let (mut x, mut y) = (rng.unit() * xmax, rng.unit() * ymax);
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        let (mut speed, mut water, mut sediment) = (1.0f32, 1.0f32, 0.0f32);
        for _ in 0..lifetime {
            if !(x >= 0.0 && y >= 0.0 && x < xmax && y < ymax) {
                break;
            }
            let (cx, cy) = (x as usize, y as usize);
            let (ox, oy) = (x - cx as f32, y - cy as f32);
            let (height, gx, gy) = sample(h, x, y);
            dx = dx * e.inertia - gx * (1.0 - e.inertia);
            dy = dy * e.inertia - gy * (1.0 - e.inertia);
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1.0e-6 {
                // On a flat plateau pick a deterministic random direction
                // instead of terminating on step 0.
                let g = GRADIENTS[(rng.next() & 7) as usize];
                dx = g.0;
                dy = g.1;
            } else {
                dx /= len;
                dy /= len;
            }
            x += dx;
            y += dy;
            if !(x >= 0.0 && y >= 0.0 && x < xmax && y < ymax) {
                break;
            }
            let dh = sample(h, x, y).0 - height;
            let capacity = (-dh * speed * water * e.capacity).max(0.01);
            let i = cy * cols + cx;
            if sediment > capacity || dh > 0.0 {
                let amount = if dh > 0.0 {
                    dh.min(sediment)
                } else {
                    (sediment - capacity) * e.deposition
                };
                sediment -= amount;
                h[i] += amount * (1.0 - ox) * (1.0 - oy);
                h[i + 1] += amount * ox * (1.0 - oy);
                h[i + cols] += amount * (1.0 - ox) * oy;
                h[i + cols + 1] += amount * ox * oy;
            } else {
                let amount = ((capacity - sediment) * e.erosion).min(-dh);
                for &(bx, by, w) in &brush {
                    let (qx, qy) = (cx as i32 + bx, cy as i32 + by);
                    if qx < 0 || qy < 0 || qx >= cols as i32 || qy >= rows as i32 {
                        continue;
                    }
                    let take = amount * w;
                    h[qy as usize * cols + qx as usize] -= take;
                    sediment += take;
                }
            }
            speed = (speed * speed - dh * e.gravity).max(0.0).sqrt();
            water *= 1.0 - e.evaporation;
        }
    }
    for x in h.iter_mut() {
        *x *= cell;
    }
}

/// CFL-stable, strictly mass-conserving thermal erosion: where a sample
/// stands above a neighbour by more than the talus slope allows, each
/// downhill link moves at most `(rate / 9) * excess` per Jacobi iteration
/// so neither peaks nor pits can ever overshoot or invert into spikes for
/// any `rate` in `[0, 1]`.
fn thermal(h: &mut [f32], cols: usize, rows: usize, cell: [f32; 2], t: &Thermal) {
    let rate = t.rate.clamp(0.0, 1.0);
    if t.iterations == 0 || rate <= 0.0 {
        return;
    }
    let tan = tan_degrees(t.talus.clamp(0.0, 89.0));
    let dist: Vec<f32> = NEIGHBOURS
        .iter()
        .map(|&(dc, dr)| {
            let (x, z) = (dc as f32 * cell[0], dr as f32 * cell[1]);
            (x * x + z * z).sqrt()
        })
        .collect();
    let k_cfl = rate * (1.0 / 9.0);
    let mut delta = vec![0.0f32; h.len()];
    for _ in 0..t.iterations {
        delta.fill(0.0);
        for r in 0..rows {
            for c in 0..cols {
                let i = r * cols + c;
                let hi = h[i];
                let mut out_total = 0.0f32;
                for (k, &(dc, dr)) in NEIGHBOURS.iter().enumerate() {
                    let (qc, qr) = (c as i32 + dc, r as i32 + dr);
                    if qc < 0 || qr < 0 || qc >= cols as i32 || qr >= rows as i32 {
                        continue;
                    }
                    let q = qr as usize * cols + qc as usize;
                    let over = hi - h[q] - tan * dist[k];
                    if over > 0.0 {
                        let flux = over * k_cfl;
                        delta[q] += flux;
                        out_total += flux;
                    }
                }
                if out_total > 0.0 {
                    delta[i] -= out_total;
                }
            }
        }
        for (x, d) in h.iter_mut().zip(&delta) {
            *x += d;
        }
    }
}

/// The routed flow: each sample's downstream neighbour (steepest descent
/// over the field with its depressions filled, priority flood with ε from
/// the edge and the sea), the samples from the highest filled height down,
/// and the samples gathered through each (itself included).
fn flow(
    h: &[f32],
    cols: usize,
    rows: usize,
    cell: [f32; 2],
    sea: Option<f32>,
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let n = h.len();
    let mut z = h.to_vec();
    let mut seen = vec![false; n];
    let mut outlet = vec![false; n];
    let mut queue = BinaryHeap::new();
    for i in 0..n {
        let (c, r) = (i % cols, i / cols);
        let edge = c == 0 || r == 0 || c == cols - 1 || r == rows - 1;
        if edge || sea.is_some_and(|s| h[i] < s) {
            outlet[i] = true;
            seen[i] = true;
            queue.push(Reverse((key(z[i]), i as u32)));
        }
    }
    while let Some(Reverse((_, i))) = queue.pop() {
        let i = i as usize;
        let (c, r) = ((i % cols) as i32, (i / cols) as i32);
        for &(dc, dr) in &NEIGHBOURS {
            let (qc, qr) = (c + dc, r + dr);
            if qc < 0 || qr < 0 || qc >= cols as i32 || qr >= rows as i32 {
                continue;
            }
            let q = qr as usize * cols + qc as usize;
            if seen[q] {
                continue;
            }
            seen[q] = true;
            if z[q] <= z[i] {
                z[q] = (z[i] + RIVER_EPS).max(z[i].next_up());
            }
            queue.push(Reverse((key(z[q]), q as u32)));
        }
    }
    let dist: Vec<f32> = NEIGHBOURS
        .iter()
        .map(|&(dc, dr)| {
            let (x, y) = (dc as f32 * cell[0], dr as f32 * cell[1]);
            (x * x + y * y).sqrt()
        })
        .collect();
    let mut downstream = vec![NONE; n];
    for i in 0..n {
        if outlet[i] {
            continue;
        }
        let (c, r) = ((i % cols) as i32, (i / cols) as i32);
        let mut best = 0.0f32;
        for (k, &(dc, dr)) in NEIGHBOURS.iter().enumerate() {
            let q = (r + dr) as usize * cols + (c + dc) as usize;
            let drop = (z[i] - z[q]) / dist[k];
            if drop > best {
                best = drop;
                downstream[i] = q as u32;
            }
        }
    }
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_by_key(|&i| (Reverse(key(z[i as usize])), i));
    let mut gathered = vec![1u32; n];
    for &i in &order {
        let d = downstream[i as usize];
        if d != NONE {
            gathered[d as usize] = gathered[d as usize].saturating_add(gathered[i as usize]);
        }
    }
    (downstream, order, gathered)
}

/// The biomes' control map: each layer's channel the most any of its
/// biomes gives a sample (its weight times its altitude, slope and
/// moisture bands).
fn biomes(
    g: &Generate,
    h: &[f32],
    gathered: &[u32],
    cols: usize,
    rows: usize,
    cell: [f32; 2],
    longer: f32,
) -> Vec<u8> {
    let m = &g.moisture;
    let channels = 4 * g
        .biomes
        .iter()
        .map(|b| (b.layer as usize).min(MAX_TERRAIN_LAYERS - 1) / 4 + 1)
        .max()
        .unwrap_or(1);
    let page = cols * rows * 4;
    let mut out = vec![0u8; page * channels / 4];
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let (c0, c1) = (c.saturating_sub(1), (c + 1).min(cols - 1));
            let (r0, r1) = (r.saturating_sub(1), (r + 1).min(rows - 1));
            let gx = (h[r * cols + c1] - h[r * cols + c0]) / ((c1 - c0) as f32 * cell[0]);
            let gz = (h[r1 * cols + c] - h[r0 * cols + c]) / ((r1 - r0) as f32 * cell[1]);
            let slope = atan_degrees((gx * gx + gz * gz).sqrt());
            let a = gathered[i] as f32;
            let flow = a / (a + m.scale.max(1e-3));
            let (x, y) = (
                c as f32 * cell[0] / longer * m.frequency,
                r as f32 * cell[1] / longer * m.frequency,
            );
            let noise = 0.5 + 0.5 * fbm(g, x, y, g.seed ^ 0x6d6f_6973, 4).0;
            let weights = m.flow + m.noise;
            let moisture = if weights > 0.0 {
                ((m.flow * flow + m.noise * noise) / weights).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let mut ch = [0.0f32; MAX_TERRAIN_LAYERS];
            for b in &g.biomes {
                let w = b.weight
                    * band(h[i], b.altitude, b.altitude_fade)
                    * band(slope, b.slope, b.slope_fade)
                    * band(moisture, b.moisture, b.moisture_fade);
                let k = (b.layer as usize).min(MAX_TERRAIN_LAYERS - 1);
                ch[k] = ch[k].max(w);
            }
            for (k, v) in ch.iter().enumerate().take(channels) {
                out[(k / 4) * page + i * 4 + k % 4] =
                    (v * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}
