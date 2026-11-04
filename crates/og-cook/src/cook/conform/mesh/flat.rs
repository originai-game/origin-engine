//! Flat colour regions for a flat look's linear mesh textures (docs/art.md
//! §2.3): a photograph held to `ArtPalette.maxColors` and `rampSteps` keeps
//! its high-frequency detail, so under the frame's bands and print it reads
//! as speckle. The texture is abstracted into flat shapes the way image
//! abstraction does it (Winnemöller, Olsen and Gooch 2006, "Real-time video
//! abstraction": edge-preserving smoothing, then quantisation; Kyprianidis
//! et al. 2009 for the Kuwahara filter):
//!
//! 1. two passes of the Kuwahara filter (four square sectors, the one of
//!    least variance), wrapping as the texture tiles, which keeps edges and
//!    flattens what lies between them;
//! 2. the asset's own palette (the 2D conform's Oklab variance cut and ramp
//!    merge, `texture2d::palette::adaptive`), every opaque texel on its
//!    nearest colour;
//! 3. colours nearer than [`MERGE`] in Oklab merged into one, weighted by
//!    their texels: shades no one tells apart at a glance become one field;
//! 4. every 4-connected region smaller than the texture's smallest shape
//!    merged into the neighbour it shares the longest border with (a
//!    region of at least that size where there is one), so no speckle is
//!    left and the edges between fields stay where the filter put them.
//!
//! Every step is integer or IEEE arithmetic in a fixed order, so a texture
//! cooks to the same texels on every machine. Alpha is untouched, and a
//! transparent texel lends no colour.

use super::super::texture2d::{colour, palette};
use crate::cook::texture::Image;
use std::collections::BTreeSet;

/// Colours nearer than this in Oklab merge (about five just noticeable
/// differences: two greens of one lawn, not the lawn and its dirt).
pub const MERGE: f64 = 0.05;

/// The Kuwahara radius and the smallest region (texels) for a `w` x `h`
/// texture: a 512² ground takes radius 4 and regions of 144 texels or
/// more, a 256² crate radius 2 and 36.
pub fn scale(w: u32, h: u32) -> (usize, usize) {
    let r = (w.min(h) / 128).clamp(1, 8) as usize;
    (r, 9 * r * r)
}

/// Flattens `img` in place to at most `max` colours and `ramp` shades per
/// hue family (0: no limit). Returns the colours before and after, or
/// `None` when no texel changed.
pub fn flatten(img: &mut Image, max: u32, ramp: u32) -> Option<(usize, usize, usize)> {
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 {
        return None;
    }
    let src: Vec<[u8; 4]> = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| if p[3] == 0 { [0; 4] } else { *p })
        .collect();
    let before = src
        .iter()
        .filter(|p| p[3] > 0)
        .map(|p| [p[0], p[1], p[2]])
        .collect::<BTreeSet<_>>()
        .len();
    let (radius, area) = scale(img.width, img.height);
    let mut px = kuwahara(&src, w, h, radius);
    px = kuwahara(&px, w, h, radius);
    let pal = palette::adaptive(&px, max, ramp);
    if pal.rgb.is_empty() {
        return None;
    }
    let all: Vec<usize> = (0..pal.rgb.len()).collect();
    let ix = palette::map(
        &px,
        img.width,
        &|_, _| (0, 0),
        &pal,
        &all,
        palette::Dither::None,
    );
    let (rgb, mut ix) = merge_colours(&pal, ix);
    merge_regions(&mut ix, w, h, area);
    let mut changed = false;
    for (p, i) in img.rgba.as_chunks_mut::<4>().0.iter_mut().zip(&ix) {
        if let Some(i) = i {
            changed |= p[..3] != rgb[*i];
            p[..3].copy_from_slice(&rgb[*i]);
        }
    }
    let after = ix.iter().flatten().collect::<BTreeSet<_>>().len();
    changed.then_some((before, after, area))
}

/// The Kuwahara filter: each opaque texel takes the mean of whichever
/// of its four (r+1)² corner squares varies least (sRGB-encoded, so the
/// variance is near perceived contrast), the squares wrapping round the
/// texture's edges and leaving transparent texels out.
fn kuwahara(px: &[[u8; 4]], w: usize, h: usize, r: usize) -> Vec<[u8; 4]> {
    let at = |x: isize, y: isize| {
        let x = x.rem_euclid(w as isize) as usize;
        let y = y.rem_euclid(h as isize) as usize;
        px[y * w + x]
    };
    let r = r as isize;
    let mut out = px.to_vec();
    for y in 0..h as isize {
        for x in 0..w as isize {
            if px[y as usize * w + x as usize][3] == 0 {
                continue;
            }
            let mut best = [0.0f64; 3];
            let mut best_var = f64::MAX;
            for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (mut sum, mut sum2, mut n) = ([0.0f64; 3], 0.0f64, 0.0f64);
                for oy in 0..=r {
                    for ox in 0..=r {
                        let p = at(x + ox * sx, y + oy * sy);
                        if p[3] == 0 {
                            continue;
                        }
                        for k in 0..3 {
                            let v = f64::from(p[k]);
                            sum[k] += v;
                            sum2 += v * v;
                        }
                        n += 1.0;
                    }
                }
                if n == 0.0 {
                    continue;
                }
                let mean = sum.map(|v| v / n);
                let var = sum2 / n - mean.iter().map(|m| m * m).sum::<f64>();
                if var < best_var {
                    best_var = var;
                    best = mean;
                }
            }
            let p = &mut out[y as usize * w + x as usize];
            for k in 0..3 {
                p[k] = best[k].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// The palette's colours with those nearer than [`MERGE`] merged (the
/// nearest pair first, into their texel-weighted Oklab mean), and each
/// texel's index into the merged colours.
fn merge_colours(
    pal: &palette::Palette,
    ix: Vec<Option<usize>>,
) -> (Vec<[u8; 3]>, Vec<Option<usize>>) {
    let n = pal.rgb.len();
    let mut count = vec![0f64; n];
    for i in ix.iter().flatten() {
        count[*i] += 1.0;
    }
    let mut lab: Vec<[f64; 3]> = pal
        .lab
        .iter()
        .map(|c| c.map(|v| f64::from(v) / colour::ONE))
        .collect();
    // Where each colour went (itself while alive).
    let mut into: Vec<usize> = (0..n).collect();
    let mut alive: Vec<usize> = (0..n).filter(|&i| count[i] > 0.0).collect();
    loop {
        let mut pair = None;
        let mut nearest = MERGE * MERGE;
        for (k, &a) in alive.iter().enumerate() {
            for &b in &alive[k + 1..] {
                let d: f64 = (0..3).map(|c| (lab[a][c] - lab[b][c]).powi(2)).sum();
                if d < nearest {
                    nearest = d;
                    pair = Some((a, b));
                }
            }
        }
        let Some((a, b)) = pair else { break };
        let (wa, wb) = (count[a], count[b]);
        let lb = lab[b];
        for (v, u) in lab[a].iter_mut().zip(lb) {
            *v = (*v * wa + u * wb) / (wa + wb);
        }
        count[a] += wb;
        into[b] = a;
        alive.retain(|&i| i != b);
    }
    let root = |mut i: usize| {
        while into[i] != i {
            i = into[i];
        }
        i
    };
    let rgb: Vec<[u8; 3]> = alive
        .iter()
        .map(|&i| colour::lab_to_srgb8(lab[i].map(|v| (v * colour::ONE).round() as i32)))
        .collect();
    let slot: Vec<usize> = (0..n)
        .map(|i| alive.iter().position(|&a| a == root(i)).unwrap_or(0))
        .collect();
    (rgb, ix.into_iter().map(|i| i.map(|i| slot[i])).collect())
}

/// Merges every 4-connected region (wrapping) of fewer than `area` texels
/// into the neighbouring region it shares the longest border with,
/// preferring regions of at least `area`, until none is left (or the
/// texture has no larger region to give).
fn merge_regions(ix: &mut [Option<usize>], w: usize, h: usize, area: usize) {
    let neighbours = |i: usize| {
        let (x, y) = (i % w, i / w);
        [
            y * w + (x + w - 1) % w,
            y * w + (x + 1) % w,
            ((y + h - 1) % h) * w + x,
            ((y + 1) % h) * w + x,
        ]
    };
    for _ in 0..16 {
        // Label the regions.
        let mut label = vec![usize::MAX; ix.len()];
        let mut sizes: Vec<usize> = Vec::new();
        let mut stack = Vec::new();
        for start in 0..ix.len() {
            if ix[start].is_none() || label[start] != usize::MAX {
                continue;
            }
            let id = sizes.len();
            let colour = ix[start];
            let mut size = 0;
            label[start] = id;
            stack.push(start);
            while let Some(i) = stack.pop() {
                size += 1;
                for j in neighbours(i) {
                    if label[j] == usize::MAX && ix[j] == colour {
                        label[j] = id;
                        stack.push(j);
                    }
                }
            }
            sizes.push(size);
        }
        if sizes.iter().all(|&s| s >= area) {
            return;
        }
        // Each small region's border with each neighbouring region.
        let mut border: Vec<std::collections::BTreeMap<usize, usize>> =
            vec![Default::default(); sizes.len()];
        for i in 0..ix.len() {
            let l = label[i];
            if l == usize::MAX || sizes[l] >= area {
                continue;
            }
            for j in neighbours(i) {
                let m = label[j];
                if m != usize::MAX && m != l {
                    *border[l].entry(m).or_default() += 1;
                }
            }
        }
        // The neighbour each small region joins: the longest border among
        // the large neighbours, else among all (the larger on a tie).
        let target: Vec<Option<usize>> = border
            .iter()
            .map(|b| {
                let pick = |large: bool| {
                    b.iter()
                        .filter(|(m, _)| !large || sizes[**m] >= area)
                        .max_by_key(|(m, n)| (**n, sizes[**m], usize::MAX - **m))
                        .map(|(m, _)| *m)
                };
                pick(true).or_else(|| pick(false))
            })
            .collect();
        let colour_of: Vec<Option<usize>> = {
            let mut c = vec![None; sizes.len()];
            for i in 0..ix.len() {
                if label[i] != usize::MAX {
                    c[label[i]] = ix[i];
                }
            }
            c
        };
        let mut moved = false;
        for i in 0..ix.len() {
            let l = label[i];
            if l == usize::MAX {
                continue;
            }
            if let Some(m) = target[l]
                && colour_of[m] != ix[i]
            {
                ix[i] = colour_of[m];
                moved = true;
            }
        }
        if !moved {
            return;
        }
    }
}
