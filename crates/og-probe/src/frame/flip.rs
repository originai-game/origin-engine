//! LDR-FLIP: NVIDIA's perceptual difference of two images as an observer
//! sees them flipped back and forth (Andersson et al., "FLIP: A Difference
//! Evaluator for Alternating Images", HPG 2020), ported from the reference
//! implementation's C++ (github.com/NVlabs/flip, `src/cpp/FLIP.h` at
//! d3ab02a, BSD-3-Clause, third_party/flip/LICENSE:
//! Copyright (c) 2020-2025, NVIDIA CORPORATION & AFFILIATES). The port
//! follows its CPU path step for step in f32: sRGB to linear to YCxCz, the
//! contrast sensitivity filters (separated as the reference separates them),
//! the Hunt-adjusted HyAB colour difference remapped to [0, 1], the edge
//! and point feature difference on the achromatic channel, and the error
//! `colour ^ (1 - feature)`, with og-core's deterministic `exp` and `pow`
//! for the host's libm (the same map on every host). Only LDR images are taken (the engine's frames
//! and references are 8-bit sRGB); the magma map is the reference's.
//!
//! The answer is a measurement per pixel in [0, 1] and its statistics; it
//! judges nothing (what error a game accepts belongs to its tests).

use super::{PixelRect, Rgba};
use og_core::math::{exp, pow};

/// The pixels per degree of the reference's default observer: 0.7 m from a
/// 0.7 m wide monitor of 3840 pixels.
pub const DEFAULT_PPD: f32 = 0.7 * (3840.0 / 0.7) * (std::f32::consts::PI / 180.0);

const GQC: f32 = 0.7;
const GPC: f32 = 0.4;
const GPT: f32 = 0.95;
const GW: f32 = 0.082;
const GQF: f32 = 0.5;

/// The contrast sensitivity Gaussians (the paper's a1, b1, a2, b2) for the
/// achromatic, red-green and blue-yellow channels.
const A1: [f32; 3] = [1.0, 1.0, 34.1];
const B1: [f32; 3] = [0.0047, 0.0053, 0.04];
const A2: [f32; 3] = [0.0, 0.0, 13.5];
const B2: [f32; 3] = [1.0e-5, 1.0e-5, 0.025];

const ILLUMINANT: [f32; 3] = [0.950_428_5, 1.0, 1.088_900_4];
const INV_ILLUMINANT: [f32; 3] = [1.052_156_9, 1.0, 0.918_357_7];

type C3 = [f32; 3];

/// A FLIP error map: one value in [0, 1] per pixel, rows top to bottom.
#[derive(Clone, Debug, PartialEq)]
pub struct FlipMap {
    pub width: u32,
    pub height: u32,
    pub error: Vec<f32>,
}

/// A FLIP map's statistics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlipStats {
    pub mean: f32,
    /// The error at the 50th, 95th and 99th percentile of the pixels.
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
    pub max: f32,
}

/// One of the worst regions of a map: its box and its error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlipRegion {
    pub rect: PixelRect,
    pub mean: f32,
    pub max: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        pow((c + 0.055) / 1.055, 2.4)
    }
}

fn linear_to_xyz(c: C3) -> C3 {
    let a11 = 10135552.0 / 24577794.0;
    let a12 = 8788810.0 / 24577794.0;
    let a13 = 4435075.0 / 24577794.0;
    let a21 = 2613072.0 / 12288897.0;
    let a22 = 8788810.0 / 12288897.0;
    let a23 = 887015.0 / 12288897.0;
    let a31 = 1425312.0 / 73733382.0;
    let a32 = 8788810.0 / 73733382.0;
    let a33 = 70074185.0 / 73733382.0;
    [
        a11 * c[0] + a12 * c[1] + a13 * c[2],
        a21 * c[0] + a22 * c[1] + a23 * c[2],
        a31 * c[0] + a32 * c[1] + a33 * c[2],
    ]
}

fn xyz_to_linear(c: C3) -> C3 {
    [
        3.241_003_3 * c[0] - 1.537_398_9 * c[1] - 0.498_615_87 * c[2],
        -0.969_224_3 * c[0] + 1.875_930_1 * c[1] + 0.041_554_224 * c[2],
        0.055_639_423 * c[0] - 0.204_011_2 * c[1] + 1.057_148_9 * c[2],
    ]
}

fn xyz_to_lab(c: C3) -> C3 {
    let delta = 6.0f32 / 29.0;
    let delta_cube = delta * delta * delta;
    let factor = 1.0 / (3.0 * delta * delta);
    let term = 4.0 / 29.0;
    let f = |v: f32| {
        if v > delta_cube {
            pow(v, 1.0 / 3.0)
        } else {
            factor * v + term
        }
    };
    let (x, y, z) = (
        f(c[0] * INV_ILLUMINANT[0]),
        f(c[1] * INV_ILLUMINANT[1]),
        f(c[2] * INV_ILLUMINANT[2]),
    );
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

fn xyz_to_ycxcz(c: C3) -> C3 {
    let (x, y, z) = (
        c[0] * INV_ILLUMINANT[0],
        c[1] * INV_ILLUMINANT[1],
        c[2] * INV_ILLUMINANT[2],
    );
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

fn ycxcz_to_xyz(c: C3) -> C3 {
    let y = (c[0] + 16.0) / 116.0;
    let cx = c[1] / 500.0;
    let cz = c[2] / 200.0;
    [
        (y + cx) * ILLUMINANT[0],
        y * ILLUMINANT[1],
        (y - cz) * ILLUMINANT[2],
    ]
}

fn hunt(l: f32, c: f32) -> f32 {
    0.01 * l * c
}

fn hyab(a: C3, b: C3) -> f32 {
    let (da, db) = (a[1] - b[1], a[2] - b[2]);
    (a[0] - b[0]).abs() + (da * da + db * db).sqrt()
}

fn max_distance() -> f32 {
    let g = xyz_to_lab(linear_to_xyz([0.0, 1.0, 0.0]));
    let b = xyz_to_lab(linear_to_xyz([0.0, 0.0, 1.0]));
    let gh = [g[0], hunt(g[0], g[1]), hunt(g[0], g[2])];
    let bh = [b[0], hunt(b[0], b[1]), hunt(b[0], b[2])];
    pow(hyab(gh, bh), GQC)
}

fn gaussian_ab(x2: f32, a: f32, b: f32) -> f32 {
    use std::f32::consts::PI;
    a * (PI / b).sqrt() * exp(-PI * PI * x2 / b)
}

fn gaussian_sqrt(x2: f32, a: f32, b: f32) -> f32 {
    use std::f32::consts::PI;
    (a * (PI / b).sqrt()).sqrt() * exp(-PI * PI * x2 / b)
}

/// The separated spatial filters: (Y, Cx) and the blue-yellow channel's two
/// Gaussians, normalised as the reference does.
fn spatial_filters(ppd: f32) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    use std::f32::consts::PI;
    let largest = B1.iter().chain(B2.iter()).fold(0.0f32, |m, &b| m.max(b));
    let radius = (3.0 * (largest / (2.0 * PI * PI)).sqrt() * ppd).ceil() as i32;
    let dx = 1.0 / ppd;
    let mut ycx = Vec::new();
    let mut cz = Vec::new();
    let (mut sum_ycx, mut sum_cz) = ([0.0f32; 2], [0.0f32; 2]);
    for x in -radius..=radius {
        let ix = x as f32 * dx;
        let ix2 = ix * ix;
        let v = [
            gaussian_ab(ix2, A1[0], B1[0]),
            gaussian_ab(ix2, A1[1], B1[1]),
        ];
        let w = [
            gaussian_sqrt(ix2, A1[2], B1[2]),
            gaussian_sqrt(ix2, A2[2], B2[2]),
        ];
        sum_ycx = [sum_ycx[0] + v[0], sum_ycx[1] + v[1]];
        sum_cz = [sum_cz[0] + w[0], sum_cz[1] + w[1]];
        ycx.push(v);
        cz.push(w);
    }
    let norm_cz = 1.0 / (sum_cz[0] * sum_cz[0] + sum_cz[1] * sum_cz[1]).sqrt();
    for v in &mut ycx {
        *v = [v[0] / sum_ycx[0], v[1] / sum_ycx[1]];
    }
    for w in &mut cz {
        *w = [w[0] * norm_cz, w[1] * norm_cz];
    }
    (ycx, cz)
}

/// The feature filter: the Gaussian and its first and second derivatives,
/// normalised as the reference does.
fn feature_filter(ppd: f32) -> Vec<C3> {
    let sd = 0.5 * GW * ppd;
    let radius = (3.0 * sd).ceil() as i32;
    let mut f = Vec::new();
    let (mut g_sum, mut dg_neg, mut dg_pos, mut ddg_neg, mut ddg_pos) =
        (0.0f32, 0.0, 0.0, 0.0, 0.0);
    for x in -radius..=radius {
        let xf = x as f32;
        let g = exp(-(xf * xf) / (2.0 * sd * sd));
        g_sum += g;
        let dg = -xf * g;
        if dg > 0.0 {
            dg_pos += dg;
        } else {
            dg_neg -= dg;
        }
        let ddg = (xf * xf / (sd * sd) - 1.0) * g;
        if ddg > 0.0 {
            ddg_pos += ddg;
        } else {
            ddg_neg -= ddg;
        }
        f.push([g, dg, ddg]);
    }
    for p in &mut f {
        *p = [
            p[0] / g_sum,
            p[1] / if p[1] > 0.0 { dg_pos } else { dg_neg },
            p[2] / if p[2] > 0.0 { ddg_pos } else { ddg_neg },
        ];
    }
    f
}

/// The image's pixels in YCxCz (alpha ignored, as the reference reads an
/// 8-bit image).
fn ycxcz(image: &Rgba) -> Vec<C3> {
    let lut: Vec<f32> = (0..256).map(|v| srgb_to_linear(v as f32 / 255.0)).collect();
    image
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            xyz_to_ycxcz(linear_to_xyz([
                lut[usize::from(p[0])],
                lut[usize::from(p[1])],
                lut[usize::from(p[2])],
            ]))
        })
        .collect()
}

/// The FLIP error map of `test` against `reference` (two images of one
/// size) for an observer at `ppd` pixels per degree ([`DEFAULT_PPD`]).
pub fn flip(reference: &Rgba, test: &Rgba, ppd: f32) -> Result<FlipMap, String> {
    if (reference.width, reference.height) != (test.width, test.height) {
        return Err(format!(
            "FLIP compares images of one size: the reference is {}x{}, the test {}x{}",
            reference.width, reference.height, test.width, test.height
        ));
    }
    if !(ppd.is_finite() && ppd > 0.0) {
        return Err(format!("pixels per degree must be above 0, not {ppd}"));
    }
    let (w, h) = (reference.width as usize, reference.height as usize);
    let r = ycxcz(reference);
    let t = ycxcz(test);
    let mut error = colour_difference(&r, &t, w, h, ppd);
    feature_difference(&r, &t, w, h, ppd, &mut error);
    Ok(FlipMap {
        width: reference.width,
        height: reference.height,
        error,
    })
}

fn colour_difference(r: &[C3], t: &[C3], w: usize, h: usize, ppd: f32) -> Vec<f32> {
    let cmax = max_distance();
    let pccmax = GPC * cmax;
    let (fy, fz) = spatial_filters(ppd);
    let half = (fy.len() / 2) as i64;
    // [Y, Cx, Cz1, Cz2] of both images, filtered along x.
    let mut mid_r = vec![[0.0f32; 4]; w * h];
    let mut mid_t = vec![[0.0f32; 4]; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut ar, mut at) = ([0.0f32; 4], [0.0f32; 4]);
            for i in -half..=half {
                let xx = (x as i64 + i).clamp(0, w as i64 - 1) as usize;
                let (wy, wz) = (fy[(i + half) as usize], fz[(i + half) as usize]);
                let (pr, pt) = (r[y * w + xx], t[y * w + xx]);
                ar[0] += wy[0] * pr[0];
                ar[1] += wy[1] * pr[1];
                ar[2] += wz[0] * pr[2];
                ar[3] += wz[1] * pr[2];
                at[0] += wy[0] * pt[0];
                at[1] += wy[1] * pt[1];
                at[2] += wz[0] * pt[2];
                at[3] += wz[1] * pt[2];
            }
            mid_r[y * w + x] = ar;
            mid_t[y * w + x] = at;
        }
    }
    let lab = |v: [f32; 4]| -> C3 {
        let ycxcz = [v[0], v[1], v[2] + v[3]];
        let rgb = xyz_to_linear(ycxcz_to_xyz(ycxcz)).map(|c| c.clamp(0.0, 1.0));
        let l = xyz_to_lab(linear_to_xyz(rgb));
        [l[0], hunt(l[0], l[1]), hunt(l[0], l[2])]
    };
    let mut out = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut ar, mut at) = ([0.0f32; 4], [0.0f32; 4]);
            for i in -half..=half {
                let yy = (y as i64 + i).clamp(0, h as i64 - 1) as usize;
                let (wy, wz) = (fy[(i + half) as usize], fz[(i + half) as usize]);
                let (pr, pt) = (mid_r[yy * w + x], mid_t[yy * w + x]);
                ar[0] += wy[0] * pr[0];
                ar[1] += wy[1] * pr[1];
                ar[2] += wz[0] * pr[2];
                ar[3] += wz[1] * pr[3];
                at[0] += wy[0] * pt[0];
                at[1] += wy[1] * pt[1];
                at[2] += wz[0] * pt[2];
                at[3] += wz[1] * pt[3];
            }
            let mut d = pow(hyab(lab(ar), lab(at)), GQC);
            d = if d < pccmax {
                d * GPT / pccmax
            } else {
                GPT + ((d - pccmax) / (cmax - pccmax)) * (1.0 - GPT)
            };
            out[y * w + x] = d;
        }
    }
    out
}

fn feature_difference(r: &[C3], t: &[C3], w: usize, h: usize, ppd: f32, error: &mut [f32]) {
    let f = feature_filter(ppd);
    let half = (f.len() / 2) as i64;
    let norm = |y: f32| y / 116.0 + 16.0 / 116.0;
    // [dx, ddx, gaussian] of both images along x.
    let mut mid_r = vec![[0.0f32; 3]; w * h];
    let mut mid_t = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        for x in 0..w {
            let (mut ar, mut at) = ([0.0f32; 3], [0.0f32; 3]);
            for i in -half..=half {
                let xx = (x as i64 + i).clamp(0, w as i64 - 1) as usize;
                let k = f[(i + half) as usize];
                let (yr, yt) = (norm(r[y * w + xx][0]), norm(t[y * w + xx][0]));
                ar[0] += k[1] * yr;
                ar[1] += k[2] * yr;
                ar[2] += k[0] * yr;
                at[0] += k[1] * yt;
                at[1] += k[2] * yt;
                at[2] += k[0] * yt;
            }
            mid_r[y * w + x] = ar;
            mid_t[y * w + x] = at;
        }
    }
    let scale = 1.0 / 2.0f32.sqrt();
    for y in 0..h {
        for x in 0..w {
            let (mut dxr, mut dxt, mut ddxr, mut ddxt) = (0.0f32, 0.0, 0.0, 0.0);
            let (mut dyr, mut dyt, mut ddyr, mut ddyt) = (0.0f32, 0.0, 0.0, 0.0);
            for i in -half..=half {
                let yy = (y as i64 + i).clamp(0, h as i64 - 1) as usize;
                let k = f[(i + half) as usize];
                let (pr, pt) = (mid_r[yy * w + x], mid_t[yy * w + x]);
                dxr += k[0] * pr[0];
                dxt += k[0] * pt[0];
                ddxr += k[0] * pr[1];
                ddxt += k[0] * pt[1];
                dyr += k[1] * pr[2];
                dyt += k[1] * pt[2];
                ddyr += k[2] * pr[2];
                ddyt += k[2] * pt[2];
            }
            let edge = ((dxr * dxr + dyr * dyr).sqrt() - (dxt * dxt + dyt * dyt).sqrt()).abs();
            let point =
                ((ddxr * ddxr + ddyr * ddyr).sqrt() - (ddxt * ddxt + ddyt * ddyt).sqrt()).abs();
            let feature = pow(scale * edge.max(point), GQF);
            let e = &mut error[y * w + x];
            *e = pow(*e, 1.0 - feature);
        }
    }
}

impl FlipMap {
    /// The mean, percentiles and maximum of the map.
    pub fn stats(&self) -> FlipStats {
        let n = self.error.len().max(1);
        let mut sorted = self.error.clone();
        sorted.sort_by(f32::total_cmp);
        let at = |q: f32| {
            sorted
                .get(((q * (n - 1) as f32).round() as usize).min(n - 1))
                .copied()
                .unwrap_or(0.0)
        };
        FlipStats {
            mean: (self.error.iter().map(|&e| f64::from(e)).sum::<f64>() / n as f64) as f32,
            p50: at(0.5),
            p95: at(0.95),
            p99: at(0.99),
            max: sorted.last().copied().unwrap_or(0.0),
        }
    }

    /// The map in the reference's magma colours: black is no difference,
    /// pale yellow the largest.
    pub fn heat_map(&self) -> Rgba {
        let mut pixels = Vec::with_capacity(self.error.len() * 4);
        for &e in &self.error {
            let c = MAGMA[(e.clamp(0.0, 1.0) * 255.0).round() as usize];
            pixels.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
        Rgba {
            width: self.width,
            height: self.height,
            pixels,
        }
    }

    /// Up to `count` regions where the error is, worst first. Each starts as
    /// the `side`×`side` square with the largest mean error that no listed
    /// region overlaps, then grows a quarter side at a time towards any
    /// side whose next strip's mean error is at least half the region's,
    /// to at most four sides each way: one change, one region, framed to
    /// its extent. A square whose mean error is under a tenth of the first
    /// region's (the halo the contrast filters spread around a change) is
    /// not listed, nor one without any error.
    pub fn worst(&self, count: usize, side: u32) -> Vec<FlipRegion> {
        let (w, h) = (self.width as usize, self.height as usize);
        let s = (side.max(1) as usize).min(w).min(h).max(1);
        // The summed-area table of the error.
        let mut sum = vec![0f64; (w + 1) * (h + 1)];
        for y in 0..h {
            let mut row = 0f64;
            for x in 0..w {
                row += f64::from(self.error[y * w + x]);
                sum[(y + 1) * (w + 1) + x + 1] = sum[y * (w + 1) + x + 1] + row;
            }
        }
        // The error summed over [x0, x1) × [y0, y1).
        let total = |x0: usize, y0: usize, x1: usize, y1: usize| {
            sum[y1 * (w + 1) + x1] - sum[y0 * (w + 1) + x1] - sum[y1 * (w + 1) + x0]
                + sum[y0 * (w + 1) + x0]
        };
        let stride = (s / 4).max(1);
        let mut candidates = Vec::new();
        let xs: Vec<usize> = (0..=w - s).step_by(stride).chain([w - s]).collect();
        let ys: Vec<usize> = (0..=h - s).step_by(stride).chain([h - s]).collect();
        for &y in &ys {
            for &x in &xs {
                candidates.push((total(x, y, x + s, y + s), x, y));
            }
        }
        // Largest first; ties to the top-left, so the answer is the same
        // on every host.
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0).then((a.2, a.1).cmp(&(b.2, b.1))));
        let mut boxes: Vec<(usize, usize, usize, usize)> = Vec::new();
        let floor = candidates.first().map_or(0.0, |c| c.0 / 10.0);
        for (t, x, y) in candidates {
            if boxes.len() >= count || t <= 0.0 || t < floor {
                break;
            }
            let overlaps =
                |b: &(usize, usize, usize, usize)| x < b.2 && b.0 < x + s && y < b.3 && b.1 < y + s;
            if boxes.iter().any(overlaps) {
                continue;
            }
            let (mut x0, mut y0, mut x1, mut y1) = (x, y, x + s, y + s);
            let limit = 4 * s;
            loop {
                let mean = total(x0, y0, x1, y1) / ((x1 - x0) * (y1 - y0)) as f64;
                let half = 0.5 * mean;
                let strip_mean = |a: (usize, usize, usize, usize)| {
                    let n = (a.2 - a.0) * (a.3 - a.1);
                    if n == 0 {
                        0.0
                    } else {
                        total(a.0, a.1, a.2, a.3) / n as f64
                    }
                };
                let free = |a: (usize, usize, usize, usize)| {
                    !boxes
                        .iter()
                        .any(|b| a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3)
                };
                let mut grew = false;
                if x1 - x0 + stride <= limit && x0 > 0 {
                    let a = (x0.saturating_sub(stride), y0, x0, y1);
                    if strip_mean(a) >= half && free(a) {
                        x0 = a.0;
                        grew = true;
                    }
                }
                if x1 - x0 + stride <= limit && x1 < w {
                    let a = (x1, y0, (x1 + stride).min(w), y1);
                    if strip_mean(a) >= half && free(a) {
                        x1 = a.2;
                        grew = true;
                    }
                }
                if y1 - y0 + stride <= limit && y0 > 0 {
                    let a = (x0, y0.saturating_sub(stride), x1, y0);
                    if strip_mean(a) >= half && free(a) {
                        y0 = a.1;
                        grew = true;
                    }
                }
                if y1 - y0 + stride <= limit && y1 < h {
                    let a = (x0, y1, x1, (y1 + stride).min(h));
                    if strip_mean(a) >= half && free(a) {
                        y1 = a.3;
                        grew = true;
                    }
                }
                if !grew {
                    break;
                }
            }
            boxes.push((x0, y0, x1, y1));
        }
        boxes
            .into_iter()
            .map(|(x0, y0, x1, y1)| {
                let mut max = 0.0f32;
                for yy in y0..y1 {
                    for xx in x0..x1 {
                        max = max.max(self.error[yy * w + xx]);
                    }
                }
                FlipRegion {
                    rect: PixelRect {
                        x: x0 as u32,
                        y: y0 as u32,
                        width: (x1 - x0) as u32,
                        height: (y1 - y0) as u32,
                    },
                    mean: (total(x0, y0, x1, y1) / ((x1 - x0) * (y1 - y0)) as f64) as f32,
                    max,
                }
            })
            .collect()
    }
}

/// The side [`FlipMap::worst`]'s regions start from for a `width`×`height`
/// map: a twentieth of the shorter side, at least 16 pixels.
pub fn region_side(width: u32, height: u32) -> u32 {
    (width.min(height) / 20).max(16)
}

/// The integer factor that enlarges a region of `side` pixels to about
/// `target` pixels (at least 1).
pub fn enlargement(side: u32, target: u32) -> u32 {
    (target / side.max(1)).max(1)
}

/// One region of a comparison as one image a reviewer reads at a glance:
/// the reference's pixels, the test's and the heat map's of `rect`, side by
/// side with a 4-pixel gap, each enlarged `factor` times (nearest
/// neighbour, so pixels stay pixels).
pub fn region_strip(
    reference: &Rgba,
    test: &Rgba,
    heat: &Rgba,
    rect: PixelRect,
    factor: u32,
) -> Rgba {
    const GAP: u32 = 4;
    let parts = [reference, test, heat].map(|i| i.crop(rect).enlarge(factor));
    let w = parts.iter().map(|p| p.width).sum::<u32>() + 2 * GAP;
    let h = parts.iter().map(|p| p.height).max().unwrap_or(0);
    let mut out = Rgba::filled(w, h, [18, 18, 22, 255]);
    let mut x = 0;
    for p in &parts {
        for row in 0..p.height {
            let src = (row * p.width * 4) as usize;
            let dst = ((row * w + x) * 4) as usize;
            out.pixels[dst..dst + (p.width * 4) as usize]
                .copy_from_slice(&p.pixels[src..src + (p.width * 4) as usize]);
        }
        x += p.width + GAP;
    }
    out
}

/// Matplotlib's magma, as the reference's `MapMagma`, in 8-bit sRGB.
#[rustfmt::skip]
const MAGMA: [[u8; 3]; 256] = [
    [0, 0, 4], [1, 0, 5], [1, 1, 6], [1, 1, 8], [2, 1, 9], [2, 2, 11], [2, 2, 13], [3, 3, 15],
    [3, 3, 18], [4, 4, 20], [5, 4, 22], [6, 5, 24], [6, 5, 26], [7, 6, 28], [8, 7, 30], [9, 7, 32],
    [10, 8, 34], [11, 9, 36], [12, 9, 38], [13, 10, 41], [14, 11, 43], [16, 11, 45], [17, 12, 47], [18, 13, 49],
    [19, 13, 52], [20, 14, 54], [21, 14, 56], [22, 15, 59], [24, 15, 61], [25, 16, 63], [26, 16, 66], [28, 16, 68],
    [29, 17, 71], [30, 17, 73], [32, 17, 75], [33, 17, 78], [34, 17, 80], [36, 18, 83], [37, 18, 85], [39, 18, 88],
    [41, 17, 90], [42, 17, 92], [44, 17, 95], [45, 17, 97], [47, 17, 99], [49, 17, 101], [51, 16, 103], [52, 16, 105],
    [54, 16, 107], [56, 16, 108], [57, 15, 110], [59, 15, 112], [61, 15, 113], [63, 15, 114], [64, 15, 116], [66, 15, 117],
    [68, 15, 118], [69, 16, 119], [71, 16, 120], [73, 16, 120], [74, 16, 121], [76, 17, 122], [78, 17, 123], [79, 18, 123],
    [81, 18, 124], [82, 19, 124], [84, 19, 125], [86, 20, 125], [87, 21, 126], [89, 21, 126], [90, 22, 126], [92, 22, 127],
    [93, 23, 127], [95, 24, 127], [96, 24, 128], [98, 25, 128], [100, 26, 128], [101, 26, 128], [103, 27, 128], [104, 28, 129],
    [106, 28, 129], [107, 29, 129], [109, 29, 129], [110, 30, 129], [112, 31, 129], [114, 31, 129], [115, 32, 129], [117, 33, 129],
    [118, 33, 129], [120, 34, 129], [121, 34, 130], [123, 35, 130], [124, 35, 130], [126, 36, 130], [128, 37, 130], [129, 37, 129],
    [131, 38, 129], [132, 38, 129], [134, 39, 129], [136, 39, 129], [137, 40, 129], [139, 41, 129], [140, 41, 129], [142, 42, 129],
    [144, 42, 129], [145, 43, 129], [147, 43, 128], [148, 44, 128], [150, 44, 128], [152, 45, 128], [153, 45, 128], [155, 46, 127],
    [156, 46, 127], [158, 47, 127], [160, 47, 127], [161, 48, 126], [163, 48, 126], [165, 49, 126], [166, 49, 125], [168, 50, 125],
    [170, 51, 125], [171, 51, 124], [173, 52, 124], [174, 52, 123], [176, 53, 123], [178, 53, 123], [179, 54, 122], [181, 54, 122],
    [183, 55, 121], [184, 55, 121], [186, 56, 120], [188, 57, 120], [189, 57, 119], [191, 58, 119], [192, 58, 118], [194, 59, 117],
    [196, 60, 117], [197, 60, 116], [199, 61, 115], [200, 62, 115], [202, 62, 114], [204, 63, 113], [205, 64, 113], [207, 64, 112],
    [208, 65, 111], [210, 66, 111], [211, 67, 110], [213, 68, 109], [214, 69, 108], [216, 69, 108], [217, 70, 107], [219, 71, 106],
    [220, 72, 105], [222, 73, 104], [223, 74, 104], [224, 76, 103], [226, 77, 102], [227, 78, 101], [228, 79, 100], [229, 80, 100],
    [231, 82, 99], [232, 83, 98], [233, 84, 98], [234, 86, 97], [235, 87, 96], [236, 88, 96], [237, 90, 95], [238, 91, 94],
    [239, 93, 94], [240, 95, 94], [241, 96, 93], [242, 98, 93], [242, 100, 92], [243, 101, 92], [244, 103, 92], [244, 105, 92],
    [245, 107, 92], [246, 108, 92], [246, 110, 92], [247, 112, 92], [247, 114, 92], [248, 116, 92], [248, 118, 92], [249, 120, 93],
    [249, 121, 93], [249, 123, 93], [250, 125, 94], [250, 127, 94], [250, 129, 95], [251, 131, 95], [251, 133, 96], [251, 135, 97],
    [252, 137, 97], [252, 138, 98], [252, 140, 99], [252, 142, 100], [252, 144, 101], [253, 146, 102], [253, 148, 103], [253, 150, 104],
    [253, 152, 105], [253, 154, 106], [253, 155, 107], [254, 157, 108], [254, 159, 109], [254, 161, 110], [254, 163, 111], [254, 165, 113],
    [254, 167, 114], [254, 169, 115], [254, 170, 116], [254, 172, 118], [254, 174, 119], [254, 176, 120], [254, 178, 122], [254, 180, 123],
    [254, 182, 124], [254, 183, 126], [254, 185, 127], [254, 187, 129], [254, 189, 130], [254, 191, 132], [254, 193, 133], [254, 194, 135],
    [254, 196, 136], [254, 198, 138], [254, 200, 140], [254, 202, 141], [254, 204, 143], [254, 205, 144], [254, 207, 146], [254, 209, 148],
    [254, 211, 149], [254, 213, 151], [254, 215, 153], [254, 216, 154], [253, 218, 156], [253, 220, 158], [253, 222, 160], [253, 224, 161],
    [253, 226, 163], [253, 227, 165], [253, 229, 167], [253, 231, 169], [253, 233, 170], [253, 235, 172], [252, 236, 174], [252, 238, 176],
    [252, 240, 178], [252, 242, 180], [252, 244, 182], [252, 246, 184], [252, 247, 185], [252, 249, 187], [252, 251, 189], [252, 253, 191],
];

#[cfg(test)]
mod tests {
    use super::*;

    fn grey(w: u32, h: u32, v: u8) -> Rgba {
        Rgba::filled(w, h, [v, v, v, 255])
    }

    #[test]
    fn identical_images_have_no_error() {
        let mut a = grey(24, 16, 90);
        a.fill(5, 5, 6, 3, [200, 30, 40, 255]);
        let m = flip(&a, &a, DEFAULT_PPD).unwrap();
        assert!(m.error.iter().all(|&e| e == 0.0), "{:?}", m.stats());
        assert!(m.worst(3, 8).is_empty());
    }

    #[test]
    fn the_reference_s_constants_hold() {
        // The default observer: 67.02 pixels per degree.
        assert!((DEFAULT_PPD - 67.0206).abs() < 1e-3);
        // Black against white is FLIP's largest flat difference: the HyAB
        // colour term alone is 0.95 + (100^0.7 - pccmax)/(cmax - pccmax)·0.05,
        // computed here independently of colour_difference.
        let cmax = max_distance();
        let pccmax = GPC * cmax;
        let expected = GPT + ((pow(100.0, GQC) - pccmax) / (cmax - pccmax)) * (1.0 - GPT);
        let m = flip(&grey(64, 64, 0), &grey(64, 64, 255), DEFAULT_PPD).unwrap();
        // Far from any edge (the images are flat) the feature term is 0.
        let centre = m.error[32 * 64 + 32];
        assert!((centre - expected).abs() < 1e-3, "{centre} vs {expected}");
    }

    #[test]
    fn a_larger_change_scores_higher_and_the_worst_region_finds_it() {
        let base = grey(96, 64, 128);
        let mut small = base.clone();
        small.fill(60, 30, 8, 8, [140, 140, 140, 255]);
        let mut large = base.clone();
        large.fill(60, 30, 8, 8, [250, 250, 250, 255]);
        let s = flip(&base, &small, DEFAULT_PPD).unwrap();
        let l = flip(&base, &large, DEFAULT_PPD).unwrap();
        assert!(l.stats().mean > s.stats().mean);
        assert!(l.stats().max > s.stats().max);
        let worst = l.worst(2, 16);
        assert_eq!(worst.len(), 1, "one change, one region: {worst:?}");
        let r = worst[0].rect;
        // The region frames the changed square (60..68, 30..38) and its
        // halo, and not much more.
        assert!(
            r.x <= 60 && 68 <= r.x + r.width && r.y <= 30 && 38 <= r.y + r.height,
            "{r:?}"
        );
        assert!(r.width <= 40 && r.height <= 40, "{r:?}");
        // A large change and a small one elsewhere: the large one is one
        // region however many squares it spans, the small one the second.
        let mut two = base.clone();
        two.fill(40, 20, 50, 40, [250, 250, 250, 255]);
        two.fill(4, 4, 6, 6, [10, 10, 10, 255]);
        let worst = flip(&base, &two, DEFAULT_PPD).unwrap().worst(3, 16);
        let r = worst[0].rect;
        assert!(
            r.x <= 44 && 86 <= r.x + r.width && r.y <= 24 && 56 <= r.y + r.height,
            "{worst:?}"
        );
        let r = worst[1].rect;
        assert!(
            r.x <= 7 && 7 < r.x + r.width && r.y <= 7 && 7 < r.y + r.height,
            "{worst:?}"
        );
    }

    #[test]
    fn the_port_matches_the_reference_implementation() {
        // A 96x64 crop of a reference frame (crates/og-tests references,
        // ao-gtao) and the same crop with a block brightened and a square
        // inverted; the numbers are NVIDIA's flip-evaluator 1.6 (pip,
        // `flip.evaluate(ref, test, "LDR", applyMagma=False)`) on the pair.
        let read = |name: &str| {
            let path = format!("{}/src/frame/testdata/{name}", env!("CARGO_MANIFEST_DIR"));
            super::super::decode_png(&std::fs::read(path).unwrap()).unwrap()
        };
        let m = flip(
            &read("flip-reference.png"),
            &read("flip-test.png"),
            DEFAULT_PPD,
        )
        .unwrap();
        let s = m.stats();
        assert!((s.mean - 0.157_432_42).abs() < 1e-5, "mean {}", s.mean);
        assert!((s.max - 0.983_433_6).abs() < 1e-5, "max {}", s.max);
        for (x, y, e) in [
            (5, 5, 0.005_769_853),
            (20, 20, 0.961_263_06),
            (70, 50, 0.410_508_33),
            (95, 63, 0.394_666_55),
            (30, 40, 0.0),
        ] {
            let got = m.error[y * 96 + x];
            assert!((got - e).abs() < 1e-5, "({x}, {y}): {got} vs {e}");
        }
    }

    #[test]
    fn the_heat_map_runs_from_black_to_pale_yellow() {
        let m = FlipMap {
            width: 2,
            height: 1,
            error: vec![0.0, 1.0],
        };
        let h = m.heat_map();
        assert_eq!(&h.pixels[..3], &[0, 0, 4]);
        assert_eq!(&h.pixels[4..7], &[252, 253, 191]);
    }
}
