//! The pixel-art downscale: a dominant-colour box filter, never a blend
//! of neighbours. Every source texel falls in exactly one target texel's
//! box (the one its centre lands in); the target takes the colour most of
//! its box's opaque texels share (texels within one step of 8 per channel
//! count as one colour, and the box takes their mean), and is opaque when
//! at least half of its box is. A box of one colour stays that colour
//! exactly, so pixel art drawn at a whole scale comes back as it was
//! drawn.

use super::colour;
use super::{Rect, Rgba};

/// Opaque from this alpha up (the 1-bit alpha's threshold).
pub const OPAQUE: u8 = 128;

/// The target texel each source column (or row) of `n` falls in at `t`.
fn bins(n: u32, t: u32) -> Vec<u32> {
    (0..n)
        .map(|x| (((2 * x + 1) as u64 * t as u64) / (2 * n as u64)) as u32)
        .collect()
}

/// The cell `from` of `src` scaled to `tw` x `th` by dominant colour; with
/// `one_bit`, alpha 0 or 255 (else the box's mean alpha).
pub fn downscale(src: &Rgba, from: Rect, tw: u32, th: u32, one_bit: bool) -> Vec<[u8; 4]> {
    let [x0, y0, w, h] = from;
    let (bx, by) = (bins(w, tw), bins(h, th));
    // Per target texel: its source texels, in scan order.
    let mut boxes: Vec<Vec<[u8; 4]>> = vec![Vec::new(); (tw * th) as usize];
    for y in 0..h {
        for x in 0..w {
            let t = (by[y as usize] * tw + bx[x as usize]) as usize;
            boxes[t].push(src.at(x0 + x, y0 + y));
        }
    }
    boxes.iter().map(|b| dominant(b, one_bit)).collect()
}

/// The colour a box of texels shows (see the module's doc).
pub fn dominant(texels: &[[u8; 4]], one_bit: bool) -> [u8; 4] {
    let opaque: Vec<&[u8; 4]> = texels.iter().filter(|p| p[3] >= OPAQUE).collect();
    if texels.is_empty() || opaque.len() * 2 < texels.len() {
        return [0, 0, 0, 0];
    }
    // Buckets of 8 per channel, in the order their first texel appears;
    // the most texels wins, the earliest on a tie.
    let mut keys: Vec<[u8; 3]> = Vec::new();
    let mut sums: Vec<([u32; 4], u32)> = Vec::new();
    for p in &opaque {
        let k = [p[0] >> 3, p[1] >> 3, p[2] >> 3];
        let i = match keys.iter().position(|q| *q == k) {
            Some(i) => i,
            None => {
                keys.push(k);
                sums.push(([0; 4], 0));
                keys.len() - 1
            }
        };
        for c in 0..4 {
            sums[i].0[c] += u32::from(p[c]);
        }
        sums[i].1 += 1;
    }
    let mut best = 0;
    for (i, s) in sums.iter().enumerate() {
        if s.1 > sums[best].1 {
            best = i;
        }
    }
    let (sum, n) = sums[best];
    let mean = |c: usize| ((sum[c] + n / 2) / n) as u8;
    let alpha = if one_bit {
        255
    } else {
        let all: u32 = texels.iter().map(|p| u32::from(p[3])).sum();
        let n = texels.len() as u32;
        ((all + n / 2) / n) as u8
    };
    [mean(0), mean(1), mean(2), alpha]
}

/// How far the cell drawn back at its size from `tw` x `th` texels is from
/// itself: the mean Oklab distance per source texel (an opaque texel shown
/// transparent or the other way counts 1).
pub fn error(src: &Rgba, from: Rect, tw: u32, th: u32) -> f64 {
    let small = downscale(src, from, tw, th, true);
    let [x0, y0, w, h] = from;
    let (bx, by) = (bins(w, tw), bins(h, th));
    let mut total = 0.0;
    let mut cache: std::collections::HashMap<[u8; 3], colour::Lab> = Default::default();
    let mut lab = |c: [u8; 3]| *cache.entry(c).or_insert_with(|| colour::lab(c));
    for y in 0..h {
        for x in 0..w {
            let s = src.at(x0 + x, y0 + y);
            let t = small[(by[y as usize] * tw + bx[x as usize]) as usize];
            total += match (s[3] >= OPAQUE, t[3] >= OPAQUE) {
                (false, false) => 0.0,
                (true, true) => {
                    (colour::dist2(lab([s[0], s[1], s[2]]), lab([t[0], t[1], t[2]])) as f64).sqrt()
                        / colour::ONE
                }
                _ => 1.0,
            };
        }
    }
    total / f64::from((w * h).max(1))
}

/// How much worse than the largest candidate a smaller one may draw the
/// cell back and still be taken (mean Oklab distance: about two just
/// noticeable differences).
pub const TOLERANCE: f64 = 0.02;

/// The cell size `cell` (w, h) takes: of the candidate longer sides not
/// past its own, the smallest that draws it back within [`TOLERANCE`] of
/// the largest (pixel art drawn at a whole scale finds its own grid;
/// detail no smaller size keeps takes the largest). `None`: no candidate
/// fits, or the cell keeps its size. `measure` is the reconstruction error
/// at a target size.
pub fn choose(
    cell: (u32, u32),
    candidates: &[u32],
    mut measure: impl FnMut(u32, u32) -> f64,
) -> Option<(u32, u32)> {
    let long = cell.0.max(cell.1);
    let mut fit: Vec<u32> = candidates
        .iter()
        .copied()
        .filter(|&c| c >= 1 && c <= long)
        .collect();
    fit.sort_unstable();
    fit.dedup();
    let size = |c: u32| {
        let f = |n: u32| {
            ((u64::from(n) * u64::from(c) + u64::from(long) / 2) / u64::from(long)).max(1) as u32
        };
        (f(cell.0), f(cell.1))
    };
    let largest = *fit.last()?;
    let target = if fit.len() == 1 {
        largest
    } else {
        let (lw, lh) = size(largest);
        let floor = measure(lw, lh);
        fit.iter()
            .copied()
            .find(|&c| {
                c == largest || {
                    let (w, h) = size(c);
                    measure(w, h) <= floor + TOLERANCE
                }
            })
            .unwrap_or(largest)
    };
    (target != long).then(|| size(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32, px: Vec<[u8; 4]>) -> Rgba {
        Rgba { w, h, px }
    }

    const R: [u8; 4] = [255, 0, 0, 255];
    const G: [u8; 4] = [0, 255, 0, 255];
    const B: [u8; 4] = [0, 0, 255, 255];
    const T: [u8; 4] = [0, 0, 0, 0];

    #[test]
    fn a_box_takes_its_dominant_colour_not_a_blend() {
        // 3 x 2 boxes of a 6 x 4 image: the left box is three reds and a
        // green (red), the middle two blues and two clear (opaque: a tie
        // keeps it), the right one red, three clear (clear).
        #[rustfmt::skip]
        let px = vec![
            R, R,  B, T,  R, T,
            R, G,  T, B,  T, T,
            G, G,  B, B,  B, B,
            G, R,  B, B,  B, G,
        ];
        let out = downscale(&img(6, 4, px), [0, 0, 6, 4], 3, 2, true);
        assert_eq!(out, vec![R, B, T, G, B, B]);
        // A near colour joins its bucket and the box takes their mean.
        let near = [250, 2, 1, 255];
        assert_eq!(dominant(&[R, near, G], true), [253, 1, 1, 255]);
        // Not one bit: the mean alpha of the box.
        assert_eq!(dominant(&[R, [255, 0, 0, 200], T, R], false)[3], 178);
    }

    #[test]
    fn finds_the_grid_pixel_art_was_drawn_on() {
        // A 4 x 2 sprite drawn at 3x: 12 x 6. Candidates 2, 4, 8 and 12:
        // 4 draws it back exactly; 2 loses detail, 8 and 12 only repeat it.
        #[rustfmt::skip]
        let small = [R, G, B, R,
                     T, B, G, T];
        let mut px = Vec::new();
        for y in 0..6 {
            for x in 0..12 {
                px.push(small[(y / 3) * 4 + x / 3]);
            }
        }
        let src = img(12, 6, px);
        let got = choose((12, 6), &[2, 4, 8, 12], |w, h| {
            error(&src, [0, 0, 12, 6], w, h)
        });
        assert_eq!(got, Some((4, 2)));
        assert_eq!(downscale(&src, [0, 0, 12, 6], 4, 2, true), small.to_vec());
        // No candidate at or under the cell's side: it keeps its size; a
        // cell already at the only one too.
        assert_eq!(choose((12, 6), &[16, 32], |_, _| 0.0), None);
        assert_eq!(choose((12, 6), &[12], |_, _| 0.0), None);
        // One candidate: taken without measuring.
        assert_eq!(
            choose((64, 32), &[16], |_, _| unreachable!()),
            Some((16, 8))
        );
    }
}
