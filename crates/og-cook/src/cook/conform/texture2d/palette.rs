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
    let (mut v, mut bits, mut size) = (0u32, 0u32, n);
    while size > 1 {
        size >>= 1;
        let bx = (x / size) & 1;
        let by = (y / size) & 1;
        v = (v << 2) | ((bx ^ by) << 1) | by;
        bits += 2;
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
/// when there are at most `max` of them (none changes), else a median cut
/// in Oklab to `max`; then each hue family merged to `ramp` shades by
/// joining the two nearest in lightness (weighted by their texels). Sorted
/// dark to light.
pub fn adaptive(px: &[[u8; 4]], max: u32, ramp: u32) -> Palette {
    let mut counts: HashMap<[u8; 3], u64> = HashMap::new();
    for p in px.iter().filter(|p| p[3] > 0) {
        *counts.entry([p[0], p[1], p[2]]).or_default() += 1;
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

fn median_cut(entries: Vec<([u8; 3], Lab, u64)>, k: usize) -> Vec<([u8; 3], Lab, u64)> {
    let mut boxes = vec![entries];
    while boxes.len() < k {
        // The box with the widest channel (and more than one colour).
        let mut pick: Option<(usize, usize, i32)> = None;
        for (bi, b) in boxes.iter().enumerate() {
            if b.len() < 2 {
                continue;
            }
            for ch in 0..3 {
                let lo = b.iter().map(|e| e.1[ch]).min().unwrap_or(0);
                let hi = b.iter().map(|e| e.1[ch]).max().unwrap_or(0);
                if pick.is_none_or(|p| hi - lo > p.2) {
                    pick = Some((bi, ch, hi - lo));
                }
            }
        }
        let Some((bi, ch, _)) = pick else { break };
        let mut b = boxes.remove(bi);
        b.sort_by_key(|e| (e.1[ch], e.0));
        let total: u64 = b.iter().map(|e| e.2).sum();
        let (mut acc, mut cut) = (0u64, 1);
        for (i, e) in b.iter().enumerate() {
            acc += e.2;
            if acc * 2 >= total {
                cut = (i + 1).clamp(1, b.len() - 1);
                break;
            }
        }
        let hi = b.split_off(cut);
        boxes.insert(bi, hi);
        boxes.insert(bi, b);
    }
    boxes.iter().map(|b| mean(b)).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bayer_matches_the_frame() {
        // The 2 x 2 matrix: 0 2 / 3 1, as thresholds (v + 0.5) / 4.
        let m: Vec<f64> = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .map(|&(x, y)| bayer(x, y, 2))
            .collect();
        assert_eq!(m, [0.125, 0.625, 0.875, 0.375]);
        // Every level of the 4 x 4 once.
        let mut v: Vec<u32> = (0..16)
            .map(|i| (bayer(i % 4, i / 4, 4) * 16.0) as u32)
            .collect();
        v.sort();
        assert_eq!(v, (0..16).collect::<Vec<_>>());
    }

    #[test]
    fn maps_to_the_nearest_and_dithers_between() {
        let pal = Palette::new(vec![[0, 0, 0], [255, 255, 255], [255, 0, 0]]);
        let all = [0, 1, 2];
        let px = [
            [10, 10, 10, 255],
            [250, 240, 245, 255],
            [200, 20, 10, 255],
            [9, 9, 9, 0],
        ];
        let got = map(&px, 4, &|_, _| (0, 0), &pal, &all, Dither::None);
        assert_eq!(got, [Some(0), Some(1), Some(2), None]);
        // A grey between black and white under a 2 x 2 Bayer: sRGB 119 is
        // linear 0.1845, Oklab L its cube root 0.569, so it lies nearer
        // white and 0.431 of the way to black; black takes the cells whose
        // thresholds (0.125 0.625 / 0.875 0.375) are under that.
        let grey = [[119, 119, 119, 255]; 4];
        let got = map(&grey, 2, &|_, _| (0, 0), &pal, &[0, 1], Dither::Bayer(2));
        assert_eq!(got, [Some(0), Some(1), Some(1), Some(0)]);
        // The origin moves the pattern with the cell: both columns read
        // the matrix's first column.
        let got = map(&grey, 2, &|x, _| (x, 0), &pal, &[0, 1], Dither::Bayer(2));
        assert_eq!(got, [Some(0), Some(0), Some(1), Some(1)]);
    }

    #[test]
    fn limits_keep_the_most_used_per_family_then_overall() {
        // Three reds (one hue family), a grey and a blue.
        let pal = Palette::new(vec![
            [80, 0, 0],
            [160, 0, 0],
            [255, 40, 40],
            [128, 128, 128],
            [0, 0, 255],
        ]);
        let used = [5, 9, 1, 3, 2];
        assert_eq!(limit(&pal, &used, 2, 0), vec![0, 1, 3, 4]);
        assert_eq!(limit(&pal, &used, 0, 3), vec![0, 1, 3]);
        assert_eq!(limit(&pal, &used, 1, 2), vec![1, 3]);
    }

    #[test]
    fn an_assets_own_palette_keeps_few_colours_exactly() {
        let px = [
            [10, 20, 30, 255],
            [10, 20, 30, 255],
            [200, 100, 0, 255],
            [1, 2, 3, 0],
        ];
        let p = adaptive(&px, 4, 0);
        assert_eq!(p.rgb, vec![[10, 20, 30], [200, 100, 0]]);
        // Four greys over two shades: the two nearest in lightness join
        // (200 and 255, L 0.83 and 1), then the next (0 and 20).
        let px = [
            [0, 0, 0, 255],
            [20, 20, 20, 255],
            [200, 200, 200, 255],
            [255, 255, 255, 255],
        ];
        let p = adaptive(&px, 0, 2);
        assert_eq!(p.rgb.len(), 2);
        assert!(p.rgb[0][0] > 0 && p.rgb[0][0] < 20, "{:?}", p.rgb);
        assert!(p.rgb[1][0] > 200 && p.rgb[1][0] < 255, "{:?}", p.rgb);
    }
}
