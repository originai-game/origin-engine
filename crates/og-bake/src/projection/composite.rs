//! Putting a model's painting back beside the texture already trusted
//! (trompe's `composite.ts`): known pixels stay exactly, the model's colour
//! drift along the seam is spread into the new region (push-pull) and fades
//! out, the new image fades in over a narrow band, and whatever the model
//! handed back still as grey clay is left out of the projection.
//!
//! Images are RGBA8, rows from the top. Masks are one byte a pixel.

/// Chebyshev dilation of a mask by `r` pixels.
pub fn dilate(m: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    if r == 0 {
        return m.to_vec();
    }
    let mut tmp = vec![0u8; m.len()];
    for y in 0..h {
        let mut last: i64 = -1_000_000;
        for x in 0..w {
            if m[y * w + x] != 0 {
                last = x as i64;
            }
            tmp[y * w + x] = (x as i64 - last <= r as i64) as u8;
        }
        last = 1_000_000;
        for x in (0..w).rev() {
            if m[y * w + x] != 0 {
                last = x as i64;
            }
            if last - x as i64 <= r as i64 {
                tmp[y * w + x] = 1;
            }
        }
    }
    let mut out = vec![0u8; m.len()];
    for x in 0..w {
        let mut last: i64 = -1_000_000;
        for y in 0..h {
            if tmp[y * w + x] != 0 {
                last = y as i64;
            }
            out[y * w + x] = (y as i64 - last <= r as i64) as u8;
        }
        last = 1_000_000;
        for y in (0..h).rev() {
            if tmp[y * w + x] != 0 {
                last = y as i64;
            }
            if last - y as i64 <= r as i64 {
                out[y * w + x] = 1;
            }
        }
    }
    out
}

/// Chamfer distance (1, √2) from every pixel to the nearest seed.
pub fn distance_to(seed: &[u8], w: usize, h: usize) -> Vec<f32> {
    const INF: f32 = 1e9;
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    let mut d: Vec<f32> = seed
        .iter()
        .map(|&s| if s != 0 { 0.0 } else { INF })
        .collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + a);
            }
            if y > 0 {
                v = v.min(d[i - w] + a);
                if x > 0 {
                    v = v.min(d[i - w - 1] + b);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + b);
                }
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + a);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + a);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + b);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + b);
                }
            }
            d[i] = v;
        }
    }
    d
}

/// Box blur (radius `r`) of the masked pixels only, normalised: RGB floats.
pub fn masked_blur(src: &[[u8; 4]], mask: &[u8], w: usize, h: usize, r: usize) -> Vec<[f32; 3]> {
    let mut num: Vec<[f32; 4]> = (0..w * h)
        .map(|i| {
            if mask[i] != 0 {
                let c = src[i];
                [c[0] as f32, c[1] as f32, c[2] as f32, 1.0]
            } else {
                [0.0; 4]
            }
        })
        .collect();
    let pass = |a: &[[f32; 4]], horizontal: bool| -> Vec<[f32; 4]> {
        let mut out = vec![[0.0f32; 4]; a.len()];
        let (n, lines) = if horizontal { (w, h) } else { (h, w) };
        for l in 0..lines {
            let idx = |k: usize| if horizontal { l * w + k } else { k * w + l };
            let mut acc = [0f64; 4];
            for k in 0..=r.min(n - 1) {
                for c in 0..4 {
                    acc[c] += a[idx(k)][c] as f64;
                }
            }
            for k in 0..n {
                out[idx(k)] = acc.map(|v| v as f32);
                if k + r + 1 < n {
                    for c in 0..4 {
                        acc[c] += a[idx(k + r + 1)][c] as f64;
                    }
                }
                if k >= r {
                    for c in 0..4 {
                        acc[c] -= a[idx(k - r)][c] as f64;
                    }
                }
            }
        }
        out
    };
    num = pass(&pass(&num, true), false);
    num.iter()
        .map(|v| {
            if v[3] > 1e-6 {
                [v[0] / v[3], v[1] / v[3], v[2] / v[3]]
            } else {
                [0.0; 3]
            }
        })
        .collect()
}

/// Push-pull: values known where `wt` > 0 spread smoothly everywhere else.
pub fn push_pull(val: &[[f32; 3]], wt: &[f32], w: usize, h: usize) -> Vec<[f32; 3]> {
    struct L {
        v: Vec<[f32; 3]>,
        a: Vec<f32>,
        w: usize,
        h: usize,
    }
    let mut levels = vec![L {
        v: val.to_vec(),
        a: wt.to_vec(),
        w,
        h,
    }];
    while levels.last().is_some_and(|l| l.w > 1 || l.h > 1) {
        let f = levels.last().unwrap_or_else(|| unreachable!());
        let (cw, ch) = (f.w.div_ceil(2).max(1), f.h.div_ceil(2).max(1));
        let mut v = vec![[0.0f32; 3]; cw * ch];
        let mut a = vec![0.0f32; cw * ch];
        for y in 0..ch {
            for x in 0..cw {
                let (mut s, mut sa) = ([0.0f32; 3], 0.0f32);
                for dy in 0..2 {
                    for dx in 0..2 {
                        let (fx, fy) = (2 * x + dx, 2 * y + dy);
                        if fx >= f.w || fy >= f.h {
                            continue;
                        }
                        let j = fy * f.w + fx;
                        for (sc, vc) in s.iter_mut().zip(f.v[j]) {
                            *sc += vc * f.a[j];
                        }
                        sa += f.a[j];
                    }
                }
                let k = y * cw + x;
                if sa > 0.0 {
                    v[k] = s.map(|c| c / sa);
                }
                a[k] = sa.min(1.0);
            }
        }
        levels.push(L { v, a, w: cw, h: ch });
    }
    for li in (0..levels.len() - 1).rev() {
        let (lo, hi) = levels.split_at_mut(li + 1);
        let (f, c) = (&mut lo[li], &hi[0]);
        for y in 0..f.h {
            let cy = ((y as f32 + 0.5) / 2.0 - 0.5).clamp(0.0, (c.h - 1) as f32);
            let (y0, fy) = (cy.floor() as usize, cy - cy.floor());
            let y1 = (y0 + 1).min(c.h - 1);
            for x in 0..f.w {
                let j = y * f.w + x;
                let a = f.a[j];
                if a >= 1.0 {
                    continue;
                }
                let cx = ((x as f32 + 0.5) / 2.0 - 0.5).clamp(0.0, (c.w - 1) as f32);
                let (x0, fx) = (cx.floor() as usize, cx - cx.floor());
                let x1 = (x0 + 1).min(c.w - 1);
                for k in 0..3 {
                    let top = c.v[y0 * c.w + x0][k] * (1.0 - fx) + c.v[y0 * c.w + x1][k] * fx;
                    let bot = c.v[y1 * c.w + x0][k] * (1.0 - fx) + c.v[y1 * c.w + x1][k] * fx;
                    f.v[j][k] = a * f.v[j][k] + (1.0 - a) * (top * (1.0 - fy) + bot * fy);
                }
                f.a[j] = 1.0;
            }
        }
    }
    levels.swap_remove(0).v
}

/// Masks of one view: geometry (the level is under the pixel) and known
/// (texture already good enough to keep).
pub struct ViewMasks {
    pub geometry: Vec<u8>,
    pub known: Vec<u8>,
}

impl ViewMasks {
    pub fn unknown(&self) -> Vec<u8> {
        self.geometry
            .iter()
            .zip(&self.known)
            .map(|(&g, &k)| (g != 0 && k == 0) as u8)
            .collect()
    }
}

/// Shift the model's output so its low frequencies continue the known
/// texture: the difference along the seam (both blurred over known pixels
/// only) spreads into the new region and fades out over 120 px (scaled
/// from a 1024 px view, like the 6 px seam band and the 8 px blur).
pub fn color_correct(
    state: &[[u8; 4]],
    painted: &[[u8; 4]],
    m: &ViewMasks,
    w: usize,
    h: usize,
) -> Vec<[u8; 4]> {
    let k = w as f32 / 1024.0;
    let (band, blur, falloff) = (6.0 * k, (8.0 * k).round() as usize, 120.0 * k);
    let n = w * h;
    let known: Vec<u8> = (0..n)
        .map(|i| (m.geometry[i] != 0 && m.known[i] != 0) as u8)
        .collect();
    let unknown = m.unknown();
    if known.iter().filter(|&&k| k != 0).count() < 64
        || unknown.iter().filter(|&&k| k != 0).count() < 64
    {
        return painted.to_vec();
    }
    let to_unknown = distance_to(&unknown, w, h);
    let to_known = distance_to(&known, w, h);
    let s = masked_blur(state, &known, w, h, blur);
    let g = masked_blur(painted, &known, w, h, blur);
    let mut diff = vec![[0.0f32; 3]; n];
    let mut wt = vec![0.0f32; n];
    for i in 0..n {
        if known[i] == 0 || to_unknown[i] > band {
            continue;
        }
        diff[i] = [0, 1, 2].map(|c| s[i][c] - g[i][c]);
        wt[i] = 1.0;
    }
    let field = push_pull(&diff, &wt, w, h);
    (0..n)
        .map(|i| {
            let f = if m.geometry[i] != 0 {
                (-to_known[i] / falloff).exp()
            } else {
                0.0
            };
            let c = painted[i];
            [
                (c[0] as f32 + field[i][0] * f).round().clamp(0.0, 255.0) as u8,
                (c[1] as f32 + field[i][1] * f).round().clamp(0.0, 255.0) as u8,
                (c[2] as f32 + field[i][2] * f).round().clamp(0.0, 255.0) as u8,
                255,
            ]
        })
        .collect()
}

/// Known pixels kept exactly, except a `feather`-px band where the new
/// image fades in; unknown and sky pixels take the new image.
pub fn feather(
    state: &[[u8; 4]],
    painted: &[[u8; 4]],
    m: &ViewMasks,
    w: usize,
    h: usize,
    feather: f32,
) -> Vec<[u8; 4]> {
    let unknown = m.unknown();
    let dk = distance_to(&unknown, w, h);
    (0..w * h)
        .map(|i| {
            let t = if m.geometry[i] == 0 || unknown[i] != 0 {
                1.0
            } else if feather > 0.0 && dk[i] <= feather {
                1.0 - dk[i] / (feather + 1.0)
            } else {
                0.0
            };
            let (s, g) = (state[i], painted[i]);
            [0, 1, 2, 3].map(|c| {
                if c == 3 {
                    255
                } else {
                    (s[c] as f32 + (g[c] as f32 - s[c] as f32) * t).round() as u8
                }
            })
        })
        .collect()
}

/// Pixels of `area` the model handed back still as grey clay. A masked model
/// returns them as it got them (the mean over a box of the largest channel
/// difference under `tol`); a model that redraws the whole image returns
/// them as flat grey of another shade, so a box that is also colourless
/// (mean chroma under 12) and textureless (luma deviation under 3) counts
/// too, measured over a box of radius `4 r` and kept only where it covers
/// a whole `4 r`-radius patch (an opening), so a smooth spot of grey paving
/// is not mistaken for an unpainted object.
pub fn untouched(
    state: &[[u8; 4]],
    painted: &[[u8; 4]],
    area: &[u8],
    w: usize,
    h: usize,
    tol: f32,
    r: usize,
) -> Vec<u8> {
    let stat: Vec<[u8; 4]> = (0..w * h)
        .map(|i| {
            let (s, p) = (state[i], painted[i]);
            let diff = (0..3)
                .map(|c| (p[c] as i32 - s[c] as i32).unsigned_abs())
                .max()
                .unwrap_or(0);
            let (mx, mn) = (p[0].max(p[1]).max(p[2]), p[0].min(p[1]).min(p[2]));
            let luma = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) as u8;
            [diff.min(255) as u8, mx - mn, luma, 255]
        })
        .collect();
    let all = vec![1u8; w * h];
    let same = masked_blur(&stat, &all, w, h, r);
    let wide = 4 * r;
    let mean = masked_blur(&stat, &all, w, h, wide);
    // Luma squared over 255 keeps it in a byte; the variance is rebuilt in luma units.
    let sq: Vec<[u8; 4]> = stat
        .iter()
        .map(|s| [((s[2] as u32 * s[2] as u32) / 255) as u8, 0, 0, 255])
        .collect();
    let mean_sq = masked_blur(&sq, &all, w, h, wide);
    let flat: Vec<u8> = (0..w * h)
        .map(|i| {
            let var = (mean_sq[i][0] * 255.0 - mean[i][2] * mean[i][2]).max(0.0);
            (area[i] != 0 && mean[i][1] < 10.0 && var.sqrt() < 2.5) as u8
        })
        .collect();
    let not_flat: Vec<u8> = flat.iter().map(|&f| 1 - f).collect();
    let eroded: Vec<u8> = dilate(&not_flat, w, h, wide)
        .iter()
        .map(|&n| 1 - n)
        .collect();
    let opened = dilate(&eroded, w, h, wide);
    (0..w * h)
        .map(|i| (area[i] != 0 && (same[i][0] < tol || (flat[i] & opened[i]) != 0)) as u8)
        .collect()
}

/// What a finished view hands to the projection.
pub struct Finished {
    pub image: Vec<[u8; 4]>,
    /// Share of the area to paint the model left unpainted.
    pub left: f32,
}

/// The whole of trompe's paste: colour correction, feathering, then every
/// pixel outside the view's geometry or `keep` (and whatever the model left
/// grey) made transparent so the projection skips it. `painted` must be
/// `w × h` already.
pub fn finish(
    state: &[[u8; 4]],
    painted: &[[u8; 4]],
    m: &ViewMasks,
    keep: Option<&[u8]>,
    w: usize,
    h: usize,
) -> Finished {
    let k = w as f32 / 1024.0;
    let corrected = color_correct(state, painted, m, w, h);
    let mut comp = feather(state, &corrected, m, w, h, 10.0 * k);
    let unknown = m.unknown();
    let left_over = untouched(state, painted, &unknown, w, h, 6.0, 3);
    let n_unknown = unknown.iter().filter(|&&u| u != 0).count();
    let n_left = left_over.iter().filter(|&&u| u != 0).count();
    for i in 0..w * h {
        // Outside the view's geometry (sky, or surfaces a roof pass or a
        // ground view of a roofed level leaves out) nothing projects.
        let kept = m.geometry[i] != 0 && keep.is_none_or(|k| k[i] != 0) && left_over[i] == 0;
        if !kept {
            comp[i] = [0; 4];
        }
    }
    Finished {
        image: comp,
        left: if n_unknown > 0 {
            n_left as f32 / n_unknown as f32
        } else {
            0.0
        },
    }
}
