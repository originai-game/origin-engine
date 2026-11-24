//! Palette mapping (`ArtPalette`): every opaque texel onto a palette
//! colour, nearest by Oklab distance, mixed with the next nearest by the
//! palette's dither where its tone falls between them; per asset at most
//! `maxColors` colours and at most `rampSteps` shades per hue family.
//!
//! The dither thresholds are the frame's (stylise.wgsl `bayer`, `ign`, the
//! blue-noise tile), read in the texture's texels from each cell's corner,
//! so every frame of a sheet carries its pattern with it. A texel on its
//! palette colour (or between two at less than the smallest threshold)
//! never moves: clean art keeps every pixel.

use super::colour::{self, Lab};
use std::collections::HashMap;

/// `DitherPattern`, by variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dither {
    None,
    Bayer(u32),
    BlueNoise,
    Ign,
}

impl Dither {
    pub fn from_word(w: &str) -> Dither {
        match w {
            "bayer2" => Dither::Bayer(2),
            "bayer4" => Dither::Bayer(4),
            "bayer8" => Dither::Bayer(8),
            "blueNoise" => Dither::BlueNoise,
            "interleavedGradient" => Dither::Ign,
            _ => Dither::None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Dither::None => "none",
            Dither::Bayer(2) => "bayer2",
            Dither::Bayer(4) => "bayer4",
            Dither::Bayer(_) => "bayer8",
            Dither::BlueNoise => "blueNoise",
            Dither::Ign => "interleavedGradient",
        }
    }

    /// The threshold at texel (x, y) of a cell, in 0..1 (0.5 without one).
    pub fn at(self, x: u32, y: u32) -> f64 {
        match self {
            Dither::None => 0.5,
            Dither::Bayer(n) => bayer(x, y, n),
            Dither::BlueNoise => {
                let v = BLUE_NOISE[((y % 64) * 64 + x % 64) as usize];
                (f64::from(v) + 0.5) / 256.0
            }
            Dither::Ign => {
                let f = |v: f64| v - v.floor();
                f(52.982_918_9 * f(0.067_110_56 * f64::from(x) + 0.005_837_15 * f64::from(y)))
            }
        }
    }
}

/// The engine's void-and-cluster tile (og-render's `blue_noise.rs`).
static BLUE_NOISE: &[u8; 4096] = include_bytes!("../../../../../og-render/data/blue-noise-64.r8");

/// The ordered threshold of an n x n Bayer matrix, as stylise.wgsl's `bayer`.
pub fn bayer(x: u32, y: u32, n: u32) -> f64 {
    let (x, y) = (x % n, y % n);
    // The finest bit of the position first, so it ends the most
    // significant: M(2n) = 4 M(n) + M(2), a dispersed matrix.
    let (mut v, mut bits, mut size) = (0u32, 0u32, 1u32);
    while size < n {
        let bx = (x / size) & 1;
        let by = (y / size) & 1;
        v = (v << 2) | ((bx ^ by) << 1) | by;
        bits += 2;
        size <<= 1;
    }
    (f64::from(v) + 0.5) / f64::from(1u32 << bits)
}

/// A palette's colours with their Oklab.
#[derive(Clone, Debug)]
pub struct Palette {
    pub rgb: Vec<[u8; 3]>,
    pub lab: Vec<Lab>,
}

impl Palette {
    pub fn new(rgb: Vec<[u8; 3]>) -> Palette {
        let lab = rgb.iter().map(|&c| colour::lab(c)).collect();
        Palette { rgb, lab }
    }
}

/// The nearest and the next nearest of `allowed` (indices into `pal`) to
/// `c`, and how far `c` lies from the first towards the second (0..1).
fn two_nearest(pal: &Palette, allowed: &[usize], c: Lab) -> (usize, usize, f64) {
    let (mut a, mut da) = (allowed[0], i64::MAX);
    let (mut b, mut db) = (allowed[0], i64::MAX);
    for &i in allowed {
        let d = colour::dist2(c, pal.lab[i]);
        if d < da {
            (b, db) = (a, da);
            (a, da) = (i, d);
        } else if d < db {
            (b, db) = (i, d);
        }
    }
    if a == b || da == 0 {
        return (a, a, 0.0);
    }
    let (p, q) = (pal.lab[a], pal.lab[b]);
    let seg = colour::dist2(p, q) as f64;
    let dot: f64 = (0..3)
        .map(|k| f64::from(c[k] - p[k]) * f64::from(q[k] - p[k]))
        .sum();
    (a, b, (dot / seg).clamp(0.0, 1.0))
}

/// Which of `allowed` each texel takes: `None` where it is transparent.
/// `origin` gives each texel's cell corner, the dither's origin.
pub fn map(
    px: &[[u8; 4]],
    width: u32,
    origin: &dyn Fn(u32, u32) -> (u32, u32),
    pal: &Palette,
    allowed: &[usize],
    dither: Dither,
) -> Vec<Option<usize>> {
    let mut near: HashMap<[u8; 3], (usize, usize, f64)> = HashMap::new();
    px.iter()
        .enumerate()
        .map(|(i, p)| {
            if p[3] == 0 {
                return None;
            }
            let rgb = [p[0], p[1], p[2]];
            let (a, b, t) = *near
                .entry(rgb)
                .or_insert_with(|| two_nearest(pal, allowed, colour::lab(rgb)));
            let (x, y) = (i as u32 % width, i as u32 / width);
            let (ox, oy) = origin(x, y);
            Some(if t > 0.0 && t > dither.at(x - ox, y - oy) {
                b
            } else {
                a
            })
        })
        .collect()
}

/// How many texels take each palette colour.
pub fn usage(index: &[Option<usize>], n: usize) -> Vec<u64> {
    let mut u = vec![0u64; n];
    for i in index.iter().flatten() {
        u[*i] += 1;
    }
    u
}

/// Of the colours `used` counts, the ones kept under the limits: in each
/// hue family the `ramp` most used (0: all), then overall the `max` most
/// used (0: all); ties keep the lower index.
pub fn limit(pal: &Palette, used: &[u64], ramp: u32, max: u32) -> Vec<usize> {
    let mut order: Vec<usize> = (0..used.len()).filter(|&i| used[i] > 0).collect();
    order.sort_by(|&a, &b| used[b].cmp(&used[a]).then(a.cmp(&b)));
    let mut kept = Vec::new();
    let mut per_family: HashMap<u8, u32> = HashMap::new();
    for i in order {
        let f = colour::hue_family(pal.lab[i]);
        let n = per_family.entry(f).or_default();
        if ramp > 0 && *n >= ramp {
            continue;
        }
        if max > 0 && kept.len() as u32 >= max {
            break;
        }
        *n += 1;
        kept.push(i);
    }
    kept.sort_unstable();
    kept
}

/// An asset's own palette, from its opaque texels: its distinct colours
/// when there are at most `max` of them (none changes), else a variance
/// cut in Oklab to `max` ([`median_cut`]) with each texel weighed, given
/// `w` texels a row, by how much a change of colour would show there
/// ([`importance`]), or counted alike without; then each hue family merged
/// to `ramp` shades by joining the two nearest in lightness (weighted by
/// their texels). Sorted dark to light.
pub fn adaptive(px: &[[u8; 4]], w: Option<u32>, max: u32, ramp: u32) -> Palette {
    let weight = w.map_or_else(|| vec![1; px.len()], |w| importance(px, w));
    let mut counts: HashMap<[u8; 3], u64> = HashMap::new();
    for (p, wt) in px.iter().zip(&weight).filter(|(p, _)| p[3] > 0) {
        *counts.entry([p[0], p[1], p[2]]).or_default() += u64::from(*wt);
    }
    let mut entries: Vec<([u8; 3], Lab, u64)> = counts
        .into_iter()
        .map(|(c, n)| (c, colour::lab(c), n))
        .collect();
    entries.sort_by_key(|e| e.0);
    let max = if max == 0 { 256 } else { max.min(256) } as usize;
    // (colour, Oklab, texels): exact colours, or the cut's means.
    let mut colours: Vec<([u8; 3], Lab, u64)> = if entries.len() <= max {
        entries
    } else {
        median_cut(entries, max)
    };
    if ramp > 0 {
        colours = ramps(colours, ramp as usize);
    }
    colours.sort_by_key(|c| (c.1[0], c.1[1], c.1[2], c.0));
    colours.dedup_by_key(|c| c.0);
    Palette::new(colours.into_iter().map(|c| c.0).collect())
}

/// The texel activity (Oklab distance to its most different opaque
/// neighbour) at which a texel weighs half: about twice a just noticeable
/// difference.
const ACTIVITY: f64 = 0.04;

/// Each texel's weight in the cut, 1 to [`WEIGHT`]: full in a smooth field
/// (a sky, a cloud, a lawn), falling as 1 / (1 + (a / ACTIVITY)²) with
/// its activity `a` in busy detail (foliage, line work, a jpeg's noise),
/// where a colour error is masked by its neighbours (the activity masking
/// of perceptual quantisers, as libimagequant's importance map). A busy
/// backdrop's thousands of detail shades thus no longer outweigh its sky,
/// whose blue and clouds' white kept one merged colour under a plain
/// texel count.
const WEIGHT: f64 = 64.0;

fn importance(px: &[[u8; 4]], w: u32) -> Vec<u32> {
    let w = w.max(1) as usize;
    let lab: Vec<Lab> = px.iter().map(|p| colour::lab([p[0], p[1], p[2]])).collect();
    (0..px.len())
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let mut a2 = 0i64;
            let mut near = |j: usize| {
                if px[j][3] > 0 {
                    a2 = a2.max(colour::dist2(lab[i], lab[j]));
                }
            };
            if x > 0 {
                near(i - 1);
            }
            if x + 1 < w && i + 1 < px.len() {
                near(i + 1);
            }
            if y > 0 {
                near(i - w);
            }
            if i + w < px.len() {
                near(i + w);
            }
            let a = (a2 as f64).sqrt() / colour::ONE / ACTIVITY;
            (WEIGHT / (1.0 + a * a)).round().max(1.0) as u32
        })
        .collect()
}

fn mean(group: &[([u8; 3], Lab, u64)]) -> ([u8; 3], Lab, u64) {
    let n: u64 = group.iter().map(|e| e.2).sum();
    let mut s = [0i64; 3];
    for e in group {
        for (k, v) in s.iter_mut().enumerate() {
            *v += i64::from(e.1[k]) * e.2 as i64;
        }
    }
    let half = (n / 2) as i64;
    let q = |v: i64| {
        (if v >= 0 {
            (v + half) / n as i64
        } else {
            (v - half) / n as i64
        }) as i32
    };
    let lab = [q(s[0]), q(s[1]), q(s[2])];
    let rgb = colour::lab_to_srgb8(lab);
    (rgb, colour::lab(rgb), n)
}

/// How much the cut weighs Oklab's a and b against its lightness: hues
/// apart count double, so a few colours keep a scene's hues (a sky, the
/// roofs, the grass) instead of spending them on steps of lightness.
const CHROMA: [i128; 3] = [1, 2, 2];

/// The cut to `k` colours (Wu's variance-minimising quantiser, 1991, on
/// Oklab with a and b weighted by [`CHROMA`]): the box whose colours lie farthest from their mean (the largest
/// squared error, weighted by texels) splits next, along the channel where
/// they spread most, at the point that leaves the two halves the least
/// error. A large field of one colour (a sky, a lawn) thus keeps a colour
/// of its own, where the widest range split at the median of the texels cut
/// mostly by lightness and averaged a blue sky into grey. Integer sums
/// (i128), so the cut is the same on every machine.
fn median_cut(entries: Vec<([u8; 3], Lab, u64)>, k: usize) -> Vec<([u8; 3], Lab, u64)> {
    // A box's squared error: Σ n·|c|² − |Σ n·c|² / Σ n.
    let error = |b: &[([u8; 3], Lab, u64)]| -> i128 {
        let (mut n, mut s, mut s2) = (0i128, [0i128; 3], 0i128);
        for e in b {
            let w = i128::from(e.2);
            n += w;
            for ch in 0..3 {
                let v = i128::from(e.1[ch]) * CHROMA[ch];
                s[ch] += w * v;
                s2 += w * v * v;
            }
        }
        if n == 0 {
            return 0;
        }
        s2 - (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]) / n
    };
    type Boxed = (Vec<([u8; 3], Lab, u64)>, i128);
    let mut boxes: Vec<Boxed> = {
        let e = error(&entries);
        vec![(entries, e)]
    };
    while boxes.len() < k {
        let Some(bi) = (0..boxes.len())
            .filter(|&i| boxes[i].0.len() > 1 && boxes[i].1 > 0)
            .max_by_key(|&i| (boxes[i].1, usize::MAX - i))
        else {
            break;
        };
        let (mut b, _) = boxes.remove(bi);
        // The channel of the largest weighted variance.
        let n: i128 = b.iter().map(|e| i128::from(e.2)).sum();
        let ch = (0..3)
            .max_by_key(|&ch| {
                let (mut s, mut s2) = (0i128, 0i128);
                for e in &b {
                    let (w, v) = (i128::from(e.2), i128::from(e.1[ch]) * CHROMA[ch]);
                    s += w * v;
                    s2 += w * v * v;
                }
                (s2 - s * s / n, 2 - ch)
            })
            .unwrap_or(0);
        b.sort_by_key(|e| (e.1[ch], e.0));
        // The split of least error: prefix sums over the sorted colours.
        let total = {
            let (mut n, mut s, mut s2) = (0i128, [0i128; 3], 0i128);
            for e in &b {
                let w = i128::from(e.2);
                n += w;
                for c in 0..3 {
                    let v = i128::from(e.1[c]) * CHROMA[c];
                    s[c] += w * v;
                    s2 += w * v * v;
                }
            }
            (n, s, s2)
        };
        let sse = |n: i128, s: [i128; 3], s2: i128| {
            if n == 0 {
                0
            } else {
                s2 - (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]) / n
            }
        };
        let (mut ln, mut ls, mut ls2) = (0i128, [0i128; 3], 0i128);
        let mut best = (i128::MAX, 1usize);
        for (i, e) in b.iter().enumerate().take(b.len() - 1) {
            let w = i128::from(e.2);
            ln += w;
            for c in 0..3 {
                let v = i128::from(e.1[c]) * CHROMA[c];
                ls[c] += w * v;
                ls2 += w * v * v;
            }
            let rs = [total.1[0] - ls[0], total.1[1] - ls[1], total.1[2] - ls[2]];
            let err = sse(ln, ls, ls2) + sse(total.0 - ln, rs, total.2 - ls2);
            if err < best.0 {
                best = (err, i + 1);
            }
        }
        let hi = b.split_off(best.1);
        let (eh, el) = (error(&hi), error(&b));
        boxes.insert(bi, (hi, eh));
        boxes.insert(bi, (b, el));
    }
    boxes.iter().map(|b| mean(&b.0)).collect()
}

fn ramps(colours: Vec<([u8; 3], Lab, u64)>, ramp: usize) -> Vec<([u8; 3], Lab, u64)> {
    let mut families: std::collections::BTreeMap<u8, Vec<([u8; 3], Lab, u64)>> = Default::default();
    for c in colours {
        families.entry(colour::hue_family(c.1)).or_default().push(c);
    }
    let mut out = Vec::new();
    for (_, mut f) in families {
        f.sort_by_key(|c| (c.1[0], c.0));
        while f.len() > ramp {
            let i = (0..f.len() - 1)
                .min_by_key(|&i| (f[i + 1].1[0] - f[i].1[0], i))
                .expect("two or more");
            let m = mean(&f[i..i + 2]);
            f.splice(i..i + 2, [m]);
            f.sort_by_key(|c| (c.1[0], c.0));
        }
        out.extend(f);
    }
    out
}

/// A colour with its own lightness and chroma and the hue of its nearest
/// palette colour (a grey where that one is grey): `strict false`, the
/// palette holding hues only.
pub fn hold_hue(pal: &Palette, c: [u8; 3]) -> [u8; 3] {
    let lab = colour::lab(c);
    let all: Vec<usize> = (0..pal.rgb.len()).collect();
    let (n, _, _) = two_nearest(pal, &all, lab);
    let q = pal.lab[n];
    let qc = ((q[1] as f64).powi(2) + (q[2] as f64).powi(2)).sqrt();
    let cc = ((lab[1] as f64).powi(2) + (lab[2] as f64).powi(2)).sqrt();
    if qc < colour::GREY * colour::ONE {
        return colour::lab_to_srgb8([lab[0], 0, 0]);
    }
    let s = cc / qc;
    colour::lab_to_srgb8([
        lab[0],
        (q[1] as f64 * s).round() as i32,
        (q[2] as f64 * s).round() as i32,
    ])
}
