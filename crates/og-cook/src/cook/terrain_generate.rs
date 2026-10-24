//! A terrain that makes its height field (`generate { ... }` in a `type
//! terrain`, instead of `heightmap`): fractal gradient noise with a domain
//! warp, an island or ridge mask, hydraulic erosion by seeded drops,
//! thermal erosion, terraces, a sea level with its shore flattened, rivers
//! carved where the flow gathers, and biome rules writing the control map.
//! The field is normalised so its lowest sample is the terrain's `height`
//! low and its highest the high (before erosion, rivers and the shore),
//! so `height a b` and `size` keep their meaning: the samples are a
//! heightmap's, and none leaves that range. The grammar, every line with its default, is terrain.rs's
//! header (the one place it is written).
//!
//! **Determinism.** Every step uses only IEEE-exact operations (add,
//! subtract, multiply, divide, sqrt, floor, min, max, comparisons), integer
//! hashes and this module's own PCG32 for the drops; no libm call (the
//! arctangent and tangent the slope rules need are series of those
//! operations), no hash-map order, no threads. The same file makes the
//! same field bit for bit on every host (the test below holds its hash).

use og_core::ogt::{Block, Entry};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A `generate { }` block, read and checked.
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mask {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thermal {
    pub talus: f32,
    pub iterations: u32,
    pub rate: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rivers {
    pub threshold: u32,
    pub depth: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moisture {
    pub flow: f32,
    pub scale: f32,
    pub noise: f32,
    pub frequency: f32,
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
    /// The line of its block, for refusals made after parsing.
    pub line: usize,
}

const KEYS: [&str; 12] = [
    "samples", "seed", "noise", "mask", "erosion", "thermal", "terraces", "sea", "shore", "rivers",
    "moisture", "biome",
];
const NOISE_KEYS: [&str; 5] = ["octaves", "frequency", "lacunarity", "gain", "warp"];
const EROSION_KEYS: [&str; 9] = [
    "drops",
    "lifetime",
    "radius",
    "inertia",
    "capacity",
    "deposition",
    "erosion",
    "evaporation",
    "gravity",
];
const THERMAL_KEYS: [&str; 3] = ["talus", "iterations", "rate"];
const TERRACE_KEYS: [&str; 2] = ["step", "blend"];
const RIVER_KEYS: [&str; 3] = ["threshold", "depth", "width"];
const MOISTURE_KEYS: [&str; 4] = ["flow", "scale", "noise", "frequency"];
const BIOME_KEYS: [&str; 8] = [
    "layer",
    "weight",
    "altitude",
    "altitudeFade",
    "slope",
    "slopeFade",
    "moisture",
    "moistureFade",
];

/// The block of `key` inside `b`, its keys checked against `keys`.
fn sub<'a>(b: &'a Block, key: &str, keys: &[&str]) -> Result<Option<&'a Block>, String> {
    let Some(e) = b.get(key) else {
        return Ok(None);
    };
    let Some(inner) = &e.block else {
        return Err(format!(
            "line {}: '{key}' is a block: {key} {{ ... }}",
            e.line
        ));
    };
    check_keys(inner, keys, key)?;
    Ok(Some(inner))
}

fn check_keys(b: &Block, keys: &[&str], what: &str) -> Result<(), String> {
    for e in &b.entries {
        if !keys.contains(&e.key.as_str()) {
            return Err(format!(
                "line {}: '{}' is not a {what} entry: {}",
                e.line,
                e.key,
                keys.join(", ")
            ));
        }
    }
    Ok(())
}

/// The numbers of `key` (its line in errors), or `None`.
fn values(b: &Block, key: &str) -> Result<Option<(Vec<f32>, usize)>, String> {
    let Some(e) = b.get(key) else {
        return Ok(None);
    };
    let v: Option<Vec<f32>> = e
        .values
        .iter()
        .map(|v| v.as_number().map(|n| n.as_f32()))
        .collect();
    match v {
        Some(v) if v.iter().all(|x| x.is_finite()) => Ok(Some((v, e.line))),
        _ => Err(format!("line {}: '{key}' takes numbers", e.line)),
    }
}

/// One number in `lo..=hi` (default `default`).
fn num(b: &Block, key: &str, default: f32, lo: f32, hi: f32) -> Result<f32, String> {
    match values(b, key)? {
        None => Ok(default),
        Some((v, line)) => {
            if v.len() != 1 {
                return Err(format!("line {line}: '{key}' takes one number"));
            }
            if !(lo..=hi).contains(&v[0]) {
                return Err(format!(
                    "line {line}: '{key}' is from {lo} to {hi}, got {}",
                    v[0]
                ));
            }
            Ok(v[0])
        }
    }
}

/// One whole number in `lo..=hi`.
fn whole(b: &Block, key: &str, default: u32, lo: u32, hi: u32) -> Result<u32, String> {
    let v = num(b, key, default as f32, lo as f32, hi as f32)?;
    if v.fract() != 0.0 {
        let line = b.get(key).map_or(0, |e| e.line);
        return Err(format!("line {line}: '{key}' is a whole number, got {v}"));
    }
    Ok(v as u32)
}

/// Two numbers, the first no larger than the second.
fn range(b: &Block, key: &str, default: [f32; 2]) -> Result<[f32; 2], String> {
    match values(b, key)? {
        None => Ok(default),
        Some((v, _)) if v.len() == 2 && v[0] <= v[1] => Ok([v[0], v[1]]),
        Some((_, line)) => Err(format!(
            "line {line}: '{key}' takes two numbers, the low then the high"
        )),
    }
}

/// Reads a terrain's `generate` entry.
pub fn parse(entry: &Entry) -> Result<Generate, String> {
    let Some(b) = &entry.block else {
        return Err(format!(
            "line {}: generate is a block: generate {{ ... }}",
            entry.line
        ));
    };
    check_keys(b, &KEYS, "generate")?;
    let (columns, rows) = match values(b, "samples")? {
        None => (257, 257),
        Some((v, line)) => {
            let ok = v.len() == 2
                && v.iter()
                    .all(|x| x.fract() == 0.0 && (2.0..=4097.0).contains(x));
            if !ok {
                return Err(format!(
                    "line {line}: 'samples' takes two whole numbers from 2 to 4097 (columns and rows)"
                ));
            }
            (v[0] as u32, v[1] as u32)
        }
    };
    let seed = whole(b, "seed", 1, 0, u32::MAX)?;
    let empty = Block::default();
    let noise = sub(b, "noise", &NOISE_KEYS)?.unwrap_or(&empty);
    let mask = match b.get("mask") {
        None => Mask::None,
        Some(e) => {
            let word = e.values.first().and_then(|v| v.as_word());
            let amount = e
                .values
                .get(1)
                .and_then(|v| v.as_number())
                .map(|n| n.as_f32());
            match (word, amount, e.values.len()) {
                (Some("island"), a, n) if n <= 2 => {
                    let f = a.unwrap_or(0.4);
                    if !(f > 0.0 && f <= 1.0) {
                        return Err(format!(
                            "line {}: an island's falloff is a share of the half size, above 0 and at most 1, got {f}",
                            e.line
                        ));
                    }
                    Mask::Island(f)
                }
                (Some("ridge"), a, n) if n <= 2 => {
                    let s = a.unwrap_or(1.0);
                    if !(0.0..=1.0).contains(&s) {
                        return Err(format!(
                            "line {}: a ridge's strength is from 0 to 1, got {s}",
                            e.line
                        ));
                    }
                    Mask::Ridge(s)
                }
                _ => {
                    return Err(format!(
                        "line {}: mask is `mask island <falloff>` or `mask ridge <strength>`",
                        e.line
                    ));
                }
            }
        }
    };
    let erosion = match sub(b, "erosion", &EROSION_KEYS)? {
        None => None,
        Some(e) => Some(Hydraulic {
            drops: whole(e, "drops", 40000, 0, 10_000_000)?,
            lifetime: whole(e, "lifetime", 30, 1, 1000)?,
            radius: whole(e, "radius", 3, 1, 16)?,
            inertia: num(e, "inertia", 0.05, 0.0, 1.0)?,
            capacity: num(e, "capacity", 4.0, 0.0, 1000.0)?,
            deposition: num(e, "deposition", 0.3, 0.0, 1.0)?,
            erosion: num(e, "erosion", 0.3, 0.0, 1.0)?,
            evaporation: num(e, "evaporation", 0.01, 0.0, 1.0)?,
            gravity: num(e, "gravity", 4.0, 0.0, 1000.0)?,
        }),
    };
    let thermal = match sub(b, "thermal", &THERMAL_KEYS)? {
        None => None,
        Some(t) => Some(Thermal {
            talus: num(t, "talus", 35.0, 0.0, 89.0)?,
            iterations: whole(t, "iterations", 20, 0, 10000)?,
            rate: num(t, "rate", 0.5, 0.0, 1.0)?,
        }),
    };
    let terraces = match sub(b, "terraces", &TERRACE_KEYS)? {
        None => None,
        Some(t) => Some([
            num(t, "step", 10.0, 1.0e-3, 1.0e6)?,
            num(t, "blend", 0.4, 0.0, 1.0)?,
        ]),
    };
    let sea = match values(b, "sea")? {
        None => None,
        Some((v, _)) if v.len() == 1 => Some(v[0]),
        Some((_, line)) => {
            return Err(format!(
                "line {line}: 'sea' takes one number: the metres above the origin"
            ));
        }
    };
    if let (None, Some(e)) = (sea, b.get("shore")) {
        return Err(format!(
            "line {}: 'shore' flattens the land beside the sea: give the sea first (sea <metres>)",
            e.line
        ));
    }
    let rivers = match sub(b, "rivers", &RIVER_KEYS)? {
        None => None,
        Some(r) => Some(Rivers {
            threshold: whole(r, "threshold", 400, 1, u32::MAX)?,
            depth: num(r, "depth", 2.0, 0.0, 1.0e6)?,
            width: num(r, "width", 6.0, 0.0, 1.0e6)?,
        }),
    };
    let m = sub(b, "moisture", &MOISTURE_KEYS)?.unwrap_or(&empty);
    let moisture = Moisture {
        flow: num(m, "flow", 1.0, 0.0, 1.0e6)?,
        scale: num(m, "scale", 64.0, 1.0e-3, 1.0e9)?,
        noise: num(m, "noise", 0.5, 0.0, 1.0e6)?,
        frequency: num(m, "frequency", 6.0, 1.0e-3, 1.0e4)?,
    };
    let mut biomes = Vec::new();
    for e in b.get_all("biome") {
        let Some(r) = &e.block else {
            return Err(format!(
                "line {}: a biome is a block: biome {{ layer 0 ... }}",
                e.line
            ));
        };
        check_keys(r, &BIOME_KEYS, "biome")?;
        if r.get("layer").is_none() {
            return Err(format!(
                "line {}: a biome names the layer it writes: layer <0 to 3>",
                e.line
            ));
        }
        biomes.push(Biome {
            layer: whole(r, "layer", 0, 0, og_render::MAX_TERRAIN_LAYERS as u32 - 1)?,
            weight: num(r, "weight", 1.0, 0.0, 1.0)?,
            altitude: range(r, "altitude", [-1.0e6, 1.0e6])?,
            altitude_fade: num(r, "altitudeFade", 2.0, 0.0, 1.0e6)?,
            slope: range(r, "slope", [0.0, 90.0])?,
            slope_fade: num(r, "slopeFade", 4.0, 0.0, 90.0)?,
            moisture: range(r, "moisture", [0.0, 1.0])?,
            moisture_fade: num(r, "moistureFade", 0.1, 0.0, 1.0)?,
            line: e.line,
        });
    }
    Ok(Generate {
        columns,
        rows,
        seed,
        octaves: whole(noise, "octaves", 6, 1, 12)?,
        frequency: num(noise, "frequency", 3.0, 1.0e-3, 1.0e4)?,
        lacunarity: num(noise, "lacunarity", 2.0, 1.0, 8.0)?,
        gain: num(noise, "gain", 0.5, 0.0, 1.0)?,
        warp: num(noise, "warp", 0.0, 0.0, 100.0)?,
        mask,
        erosion,
        thermal,
        terraces,
        sea,
        shore: num(b, "shore", 2.0, 0.0, 1.0e6)?,
        rivers,
        moisture,
        biomes,
    })
}

/// A generated field.
#[derive(Clone, Debug)]
pub struct Generated {
    /// Metres above the origin, row-major from -z to +z and -x to +x.
    pub heights: Vec<f32>,
    /// The biomes' control map (`columns` × `rows` RGBA8), with biomes.
    pub control: Option<Vec<u8>>,
    /// The samples rivers run through, in index order (with rivers).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the tests follow a river to the sea")
    )]
    pub river: Vec<u32>,
    /// Each sample's downstream neighbour on the routed flow ([`NONE`]:
    /// an outlet, the sea or the edge), with rivers or biomes.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the tests follow a river to the sea")
    )]
    pub downstream: Vec<u32>,
    /// The rivers' length in metres (sample to downstream sample).
    pub river_length: f32,
}

pub const NONE: u32 = u32::MAX;

// ── the deterministic arithmetic ─────────────────────────────────────────

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

// ── the stages ───────────────────────────────────────────────────────────

/// Makes the field of `g` for a terrain of `size` metres whose samples
/// stand for `height` (`[low, high]` metres).
pub fn generate(g: &Generate, size: [f32; 2], height: [f32; 2]) -> Generated {
    let (cols, rows) = (g.columns as usize, g.rows as usize);
    let n = cols * rows;
    let cell = [size[0] / (cols - 1) as f32, size[1] / (rows - 1) as f32];
    let longer = size[0].max(size[1]);
    let [lo, hi] = height;

    // Noise and mask, normalised to 0..1.
    let mut v = vec![0.0f32; n];
    for r in 0..rows {
        for c in 0..cols {
            let (u, w) = (c as f32 / (cols - 1) as f32, r as f32 / (rows - 1) as f32);
            let mut x = u * size[0] / longer * g.frequency;
            let mut y = w * size[1] / longer * g.frequency;
            if g.warp > 0.0 {
                let o = g.octaves.min(4);
                let (qx, _) = fbm(g, x + 5.2, y + 1.3, g.seed ^ 0x5bd1_e995, o);
                let (qy, _) = fbm(g, x + 1.7, y + 9.2, g.seed ^ 0x1b87_3593, o);
                x += g.warp * qx;
                y += g.warp * qy;
            }
            let (f, ridged) = fbm(g, x, y, g.seed, g.octaves);
            let mut h = 0.5 + 0.5 * f;
            match g.mask {
                Mask::None => {}
                Mask::Ridge(s) => h = lerp(h, ridged, s),
                Mask::Island(falloff) => {
                    let (dx, dz) = (u * 2.0 - 1.0, w * 2.0 - 1.0);
                    let d = (dx * dx + dz * dz).sqrt();
                    h *= smoothstep((1.0 - d) / falloff);
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
    if let Some([step, blend]) = g.terraces {
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
    let mut river = Vec::new();
    let mut river_length = 0.0f32;
    if let Some(rv) = &g.rivers {
        let sea = g.sea.unwrap_or(f32::MIN);
        let is_river: Vec<bool> = (0..n)
            .map(|i| gathered[i] >= rv.threshold && h[i] >= sea && downstream[i] != NONE)
            .collect();
        let full = |i: usize| {
            (gathered[i] as f32 / (rv.threshold as f32 * 8.0))
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
                river.push(i as u32);
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
        river,
        downstream,
        river_length,
    }
}

/// Hydraulic erosion (after Beyer 2015 and Lague's drops): each seeded
/// drop runs down the bilinear gradient with inertia, eroding a brush of
/// samples around it while it can carry more and depositing where it
/// carries too much or climbs. Heights are worked in samples (metres over
/// the cell size) so the parameters do not depend on the terrain's scale.
fn hydraulic(h: &mut [f32], cols: usize, rows: usize, cell: f32, e: &Hydraulic, seed: u32) {
    for x in h.iter_mut() {
        *x /= cell;
    }
    let r = e.radius as i32;
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
        for _ in 0..e.lifetime {
            if !(x >= 0.0 && y >= 0.0 && x < xmax && y < ymax) {
                break;
            }
            let (cx, cy) = (x as usize, y as usize);
            let (ox, oy) = (x - cx as f32, y - cy as f32);
            let (height, gx, gy) = sample(h, x, y);
            dx = dx * e.inertia - gx * (1.0 - e.inertia);
            dy = dy * e.inertia - gy * (1.0 - e.inertia);
            let len = (dx * dx + dy * dy).sqrt();
            if len == 0.0 {
                break;
            }
            dx /= len;
            dy /= len;
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

/// Thermal erosion: where a sample stands above a neighbour by more than
/// the talus slope allows, a share of the excess slumps onto the lower
/// neighbours (every sample moved from the same iteration's heights).
fn thermal(h: &mut [f32], cols: usize, rows: usize, cell: [f32; 2], t: &Thermal) {
    let tan = tan_degrees(t.talus);
    let dist: Vec<f32> = NEIGHBOURS
        .iter()
        .map(|&(dc, dr)| {
            let (x, z) = (dc as f32 * cell[0], dr as f32 * cell[1]);
            (x * x + z * z).sqrt()
        })
        .collect();
    let mut delta = vec![0.0f32; h.len()];
    let mut excess = [0.0f32; 8];
    for _ in 0..t.iterations {
        delta.iter_mut().for_each(|d| *d = 0.0);
        for r in 0..rows {
            for c in 0..cols {
                let i = r * cols + c;
                let (mut sum, mut most) = (0.0f32, 0.0f32);
                for (k, &(dc, dr)) in NEIGHBOURS.iter().enumerate() {
                    excess[k] = 0.0;
                    let (qc, qr) = (c as i32 + dc, r as i32 + dr);
                    if qc < 0 || qr < 0 || qc >= cols as i32 || qr >= rows as i32 {
                        continue;
                    }
                    let q = qr as usize * cols + qc as usize;
                    let over = h[i] - h[q] - tan * dist[k];
                    if over > 0.0 {
                        excess[k] = over;
                        sum += over;
                        most = most.max(over);
                    }
                }
                if sum <= 0.0 {
                    continue;
                }
                let moved = t.rate * most * 0.5;
                delta[i] -= moved;
                for (k, &(dc, dr)) in NEIGHBOURS.iter().enumerate() {
                    if excess[k] > 0.0 {
                        let q = (r as i32 + dr) as usize * cols + (c as i32 + dc) as usize;
                        delta[q] += moved * excess[k] / sum;
                    }
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
    let mut out = vec![0u8; cols * rows * 4];
    for r in 0..rows {
        for c in 0..cols {
            let i = r * cols + c;
            let (c0, c1) = (c.saturating_sub(1), (c + 1).min(cols - 1));
            let (r0, r1) = (r.saturating_sub(1), (r + 1).min(rows - 1));
            let gx = (h[r * cols + c1] - h[r * cols + c0]) / ((c1 - c0) as f32 * cell[0]);
            let gz = (h[r1 * cols + c] - h[r0 * cols + c]) / ((r1 - r0) as f32 * cell[1]);
            let slope = atan_degrees((gx * gx + gz * gz).sqrt());
            let a = gathered[i] as f32;
            let flow = a / (a + m.scale);
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
            let mut ch = [0.0f32; 4];
            for b in &g.biomes {
                let w = b.weight
                    * band(h[i], b.altitude, b.altitude_fade)
                    * band(slope, b.slope, b.slope_fade)
                    * band(moisture, b.moisture, b.moisture_fade);
                let k = b.layer as usize;
                ch[k] = ch[k].max(w);
            }
            for k in 0..4 {
                out[i * 4 + k] = (ch[k] * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str) -> Generate {
        let doc = og_core::ogt::Document::parse(text).unwrap();
        parse(doc.body.get("generate").unwrap()).unwrap()
    }

    /// Every stage on, 256 × 256 samples over 512 m, heights -40 to 120.
    const EVERYTHING: &str = "type terrain\nversion 1\n---\ngenerate {
  samples 256 256
  seed 7
  noise {
    octaves 6
    frequency 3
    lacunarity 2.1
    gain 0.5
    warp 0.6
  }
  mask island 0.45
  erosion {
    drops 20000
  }
  thermal {
    talus 40
    iterations 8
  }
  terraces {
    step 12
    blend 0.7
  }
  sea 8
  shore 3
  rivers {
    threshold 300
    depth 2.5
    width 8
  }
  biome {
    layer 0
    altitude 11 90
    slope 0 30
  }
  biome {
    layer 1
    slope 30 90
  }
  biome {
    layer 2
    altitude -1000 11
  }
  biome {
    layer 3
    weight 0.8
    altitude 9 1000
    moisture 0.55 1
  }
}
";

    const SIZE: [f32; 2] = [512.0, 512.0];
    const HEIGHT: [f32; 2] = [-40.0, 120.0];

    fn fnv(h: &mut u64, bytes: &[u8]) {
        for b in bytes {
            *h ^= *b as u64;
            *h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    /// The generated field's hash, held here: the same on every host and
    /// build. A change to any stage's arithmetic moves it; so does a
    /// platform that rounds differently (the gate runs this on macOS
    /// aarch64 and Linux x86_64).
    #[test]
    fn a_generated_field_hashes_the_same_everywhere() {
        let g = read(EVERYTHING);
        let a = generate(&g, SIZE, HEIGHT);
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for x in &a.heights {
            fnv(&mut h, &x.to_bits().to_le_bytes());
        }
        fnv(&mut h, a.control.as_deref().unwrap());
        fnv(&mut h, &a.river_length.to_bits().to_le_bytes());
        assert_eq!(a.heights.len(), 256 * 256);
        assert!(!a.river.is_empty(), "rivers run");
        assert_eq!(h, 0x3545_dee3_68d7_5755, "the generated field's hash");
    }

    #[test]
    fn the_series_match_the_library_closely() {
        for d in [0.0f32, 5.0, 30.0, 45.0, 60.0, 80.0, 89.0] {
            let want = d.to_radians().tan();
            assert!(
                (tan_degrees(d) - want).abs() <= want * 1e-5 + 1e-6,
                "tan {d}"
            );
        }
        for t in [0.0f32, 0.1, 0.5, 1.0, 2.0, 10.0, 1000.0] {
            let want = t.atan().to_degrees();
            assert!((atan_degrees(t) - want).abs() < 1e-4, "atan {t}");
        }
    }

    /// The sea flattens the land beside it: every sample within the shore
    /// band is nearer the sea level than it would be without the sea, and
    /// the band's slopes are gentler.
    #[test]
    fn the_sea_level_flattens_the_shore() {
        let text = "type terrain\nversion 1\n---\ngenerate {\n  samples 257 257\n  seed 3\n  mask island 0.5\n  sea 10\n  shore 6\n}\n";
        let with = read(text);
        let without = Generate {
            sea: None,
            ..with.clone()
        };
        let a = generate(&with, [256.0, 256.0], [0.0, 30.0]);
        let b = generate(&without, [256.0, 256.0], [0.0, 30.0]);
        let (mut near, mut steep_a, mut steep_b) = (0, 0.0f32, 0.0f32);
        for i in 0..a.heights.len() {
            let d = b.heights[i] - 10.0;
            if d.abs() < 6.0 {
                assert!((a.heights[i] - 10.0).abs() <= d.abs(), "sample {i}");
            } else {
                assert_eq!(a.heights[i], b.heights[i], "outside the band, sample {i}");
            }
            // Within a third of the band of the sea the slope is 4t - 3t²
            // of the land's (5/9 on average).
            let (c, r) = (i % 257, i / 257);
            if d.abs() < 2.0 && c > 0 && c < 256 && r > 0 && r < 256 {
                near += 1;
                for (p, q) in [(i - 1, i + 1), (i - 257, i + 257)] {
                    steep_a += (a.heights[q] - a.heights[p]).abs();
                    steep_b += (b.heights[q] - b.heights[p]).abs();
                }
            }
        }
        assert!(near > 500, "the island has a shore ({near} samples)");
        assert!(
            steep_a < steep_b * 0.7,
            "the shore is flatter: {steep_a} against {steep_b}"
        );
        // Nothing crossed the sea level: land stays land, sea stays sea.
        for (x, y) in a.heights.iter().zip(&b.heights) {
            assert_eq!(*x >= 10.0, *y >= 10.0);
        }
    }

    /// A river runs from its source to the sea, down at every step.
    #[test]
    fn a_river_reaches_the_sea() {
        let g = read(EVERYTHING);
        let a = generate(&g, SIZE, HEIGHT);
        let sea = 8.0;
        let is_river: std::collections::BTreeSet<u32> = a.river.iter().copied().collect();
        // The longest: from the river sample with the most below it.
        let mut longest = 0;
        for &s in &a.river {
            let (mut i, mut steps) = (s as usize, 0);
            while a.heights[i] >= sea {
                assert!(
                    is_river.contains(&(i as u32)),
                    "{s}: the path left the river at {i}"
                );
                let d = a.downstream[i];
                assert_ne!(d, NONE, "{s}: the river stopped at {i} above the sea");
                let d = d as usize;
                assert!(a.heights[d] < a.heights[i], "{s}: the river climbs at {i}");
                i = d;
                steps += 1;
            }
            longest = longest.max(steps);
        }
        assert!(longest > 20, "a river runs {longest} samples");
        assert!(a.river_length > 500.0, "{} m of rivers", a.river_length);
        // The river is carved: its samples lie below the uncarved field.
        let dry = generate(
            &Generate {
                rivers: None,
                ..g.clone()
            },
            SIZE,
            HEIGHT,
        );
        let carved = a
            .river
            .iter()
            .filter(|&&i| a.heights[i as usize] < dry.heights[i as usize] - 0.5)
            .count();
        assert!(
            carved * 2 > a.river.len(),
            "{carved} of {} carved",
            a.river.len()
        );
    }

    /// A biome writes its layer's channel where its rules hold and leaves
    /// it empty where they do not.
    #[test]
    fn a_biome_writes_its_layer_where_its_rules_hold() {
        let g = read(EVERYTHING);
        let a = generate(&g, SIZE, HEIGHT);
        let control = a.control.as_deref().unwrap();
        let (cols, cell) = (256usize, 512.0 / 255.0);
        let (mut beach, mut high_flat, mut checked) = (0, 0, 0);
        for r in 1..255 {
            for c in 1..255 {
                let i = r * cols + c;
                let h = a.heights[i];
                let gx = (a.heights[i + 1] - a.heights[i - 1]) / (2.0 * cell);
                let gz = (a.heights[i + cols] - a.heights[i - cols]) / (2.0 * cell);
                let slope = (gx * gx + gz * gz).sqrt().atan().to_degrees();
                // Sand (layer 2) below 11 m, fading over 2 m: full under 11, none over 13.
                if h < 10.9 {
                    assert_eq!(control[i * 4 + 2], 255, "sand at {h} m");
                    beach += 1;
                } else if h > 13.1 {
                    assert_eq!(control[i * 4 + 2], 0, "no sand at {h} m");
                }
                // Grass (layer 0) on flats from 11 to 90 m.
                if (13.1..88.0).contains(&h) && slope < 25.9 {
                    assert_eq!(control[i * 4], 255, "grass at {h} m, {slope}°");
                    high_flat += 1;
                }
                if slope > 34.1 || h < 8.9 {
                    assert_eq!(control[i * 4], 0, "no grass at {h} m, {slope}°");
                }
                checked += 1;
            }
        }
        assert!(
            beach > 1000 && high_flat > 1000,
            "{beach} beach, {high_flat} meadow of {checked}"
        );
        // Layer 3 (moisture): its weight 0.8 at most, and some of it.
        assert!(control.chunks(4).all(|t| t[3] <= 204));
        assert!(control.chunks(4).any(|t| t[3] == 204));
    }

    #[test]
    fn refusals_name_the_line() {
        let refuse = |body: &str| {
            let text = format!("type terrain\nversion 1\n---\ngenerate {{\n{body}}}\n");
            let doc = og_core::ogt::Document::parse(&text).unwrap();
            parse(doc.body.get("generate").unwrap()).unwrap_err()
        };
        assert!(refuse("  shore 2\n").contains("line 5: 'shore'"));
        assert!(refuse("  octaves 3\n").contains("line 5: 'octaves' is not a generate entry"));
        assert!(refuse("  mask circle\n").contains("line 5: mask is"));
        assert!(
            refuse("  noise {\n    octaves 40\n  }\n")
                .contains("line 6: 'octaves' is from 1 to 12")
        );
        assert!(
            refuse("  biome {\n    slope 0 30\n  }\n").contains("line 5: a biome names the layer")
        );
        assert!(refuse("  samples 1 9\n").contains("line 5: 'samples'"));
    }

    #[test]
    fn two_seeds_two_terrains_one_seed_one() {
        let g = read(
            "type terrain\nversion 1\n---\ngenerate {\n  samples 65 65\n  erosion {\n    drops 500\n  }\n}\n",
        );
        let a = generate(&g, [64.0, 64.0], [0.0, 10.0]);
        let b = generate(&g, [64.0, 64.0], [0.0, 10.0]);
        let c = generate(
            &Generate {
                seed: 2,
                ..g.clone()
            },
            [64.0, 64.0],
            [0.0, 10.0],
        );
        assert_eq!(a.heights, b.heights);
        assert_ne!(a.heights, c.heights);
        // Normalised to the height range before erosion.
        let plain = generate(&Generate { erosion: None, ..g }, [64.0, 64.0], [0.0, 10.0]);
        let (lo, hi) = plain
            .heights
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        assert_eq!((lo, hi), (0.0, 10.0));
    }
}
