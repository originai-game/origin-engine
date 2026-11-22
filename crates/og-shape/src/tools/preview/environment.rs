//! The light round the scene: a sky over a ground, or a picture, for reflections and the diffuse light it gathers.

use super::*;

/// The light a mirror facing straight down and straight up reflects
/// (linear): the ground's and the sky's.
pub(crate) const ENV_DOWN: f64 = 0.15;

pub(crate) const ENV_UP: f64 = 0.9;

/// The light all round a scene, from a picture of it: an
/// equirectangular image (its width the turn round the scene's up
/// axis, its top row straight up), in linear light. With one, a
/// surface reflects the picture where it would reflect the plain sky
/// over the ground, blurred as it roughens, and the even light from
/// all round takes the picture's colour from each side.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    /// The picture and its halvings, each (width, height, linear RGB a
    /// texel), down to 8 across.
    pub(crate) levels: Vec<(usize, usize, Vec<[f32; 3]>)>,
    /// The light a matte surface facing each way gathers, 32 × 16.
    pub(crate) gathered: Vec<[f32; 3]>,
}

pub(crate) const GATHERED: (usize, usize) = (32, 16);

impl Environment {
    /// From linear RGB texels, rows from the top. The whole is scaled
    /// so that its mean is the plain sky's (0.525): the picture gives
    /// the light its shape and colour, not the exposure. `None` for an
    /// image of no size, of the wrong length, or not finite.
    pub fn new(width: usize, height: usize, texels: &[[f32; 3]]) -> Option<Environment> {
        if width == 0 || height == 0 || texels.len() != width * height {
            return None;
        }
        if texels.iter().flatten().any(|v| !v.is_finite() || *v < 0.0) {
            return None;
        }
        // The mean over the sphere: a row counts by its ring's size.
        let ring = |y: usize, h: usize| trig::sin(trig::PI * (y as f64 + 0.5) / h as f64);
        let (mut sum, mut weight) = (0.0f64, 0.0f64);
        for y in 0..height {
            let w = ring(y, height);
            for p in &texels[y * width..(y + 1) * width] {
                sum += w * (f64::from(p[0]) + f64::from(p[1]) + f64::from(p[2])) / 3.0;
            }
            weight += w * width as f64;
        }
        let mean = sum / weight;
        if mean <= 0.0 || !mean.is_finite() {
            return None;
        }
        let k = ((ENV_DOWN + ENV_UP) / 2.0 / mean) as f32;
        let mut levels = vec![(
            width,
            height,
            texels.iter().map(|p| p.map(|c| c * k)).collect::<Vec<_>>(),
        )];
        while let Some((w, h, last)) = levels.last() {
            if *w <= 8 || *h <= 4 {
                break;
            }
            let (nw, nh) = (w / 2, h / 2);
            let mut next = Vec::with_capacity(nw * nh);
            for y in 0..nh {
                for x in 0..nw {
                    let at = |dx: usize, dy: usize| last[(2 * y + dy) * w + 2 * x + dx];
                    let (a, b, c, d) = (at(0, 0), at(1, 0), at(0, 1), at(1, 1));
                    next.push([0, 1, 2].map(|i| (a[i] + b[i] + c[i] + d[i]) / 4.0));
                }
            }
            levels.push((nw, nh, next));
        }
        // What a matte surface gathers: the picture times the cosine to
        // its normal, summed over a small level.
        let small = levels
            .iter()
            .find(|l| l.0 <= 64)
            .unwrap_or(&levels[levels.len() - 1]);
        let (sw, sh, texels) = (small.0, small.1, &small.2);
        let dirs: Vec<(DVec3, f64)> = (0..sw * sh)
            .map(|i| {
                let (x, y) = (i % sw, i / sw);
                (
                    Self::direction((x as f64 + 0.5) / sw as f64, (y as f64 + 0.5) / sh as f64),
                    ring(y, sh),
                )
            })
            .collect();
        let mut gathered = Vec::with_capacity(GATHERED.0 * GATHERED.1);
        for gy in 0..GATHERED.1 {
            for gx in 0..GATHERED.0 {
                let n = Self::direction(
                    (gx as f64 + 0.5) / GATHERED.0 as f64,
                    (gy as f64 + 0.5) / GATHERED.1 as f64,
                );
                let (mut sum, mut weight) = ([0.0f64; 3], 0.0f64);
                for ((d, ring), p) in dirs.iter().zip(texels) {
                    let w = ring * d.dot(n).max(0.0);
                    for c in 0..3 {
                        sum[c] += w * f64::from(p[c]);
                    }
                    weight += w;
                }
                gathered.push(sum.map(|s| (s / weight.max(1e-12)) as f32));
            }
        }
        Some(Environment { levels, gathered })
    }

    /// From an sRGB image (PNG, JPEG, WebP or KTX2, decoded).
    pub fn from_image(image: &Image) -> Option<Environment> {
        let texels: Vec<[f32; 3]> = image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [0, 1, 2].map(|c| to_linear(f64::from(p[c]) / 255.0) as f32))
            .collect();
        Self::new(image.width as usize, image.height as usize, &texels)
    }

    /// From a Radiance `.hdr` file (RGBE, its rows from the top, flat or
    /// run-length coded); or why not.
    pub fn from_hdr(bytes: &[u8]) -> Result<Environment, String> {
        let bad = || "not a Radiance .hdr picture this reads".to_string();
        if !bytes.starts_with(b"#?") {
            return Err(bad());
        }
        // The header ends at an empty line; the size line follows.
        let mut at = 0;
        let mut line = || -> Option<&[u8]> {
            let end = at + bytes[at..].iter().position(|b| *b == b'\n')?;
            let l = &bytes[at..end];
            at = end + 1;
            Some(l)
        };
        loop {
            if line().ok_or_else(bad)?.is_empty() {
                break;
            }
        }
        let size = String::from_utf8_lossy(line().ok_or_else(bad)?).to_string();
        let words: Vec<&str> = size.split_whitespace().collect();
        let (height, width) = match words.as_slice() {
            ["-Y", h, "+X", w] => (h.parse::<usize>().ok(), w.parse::<usize>().ok()),
            _ => return Err("an .hdr picture whose rows do not run from the top".into()),
        };
        let (Some(height), Some(width)) = (height, width) else {
            return Err(bad());
        };
        if width == 0 || height == 0 || width.saturating_mul(height) > 64_000_000 {
            return Err("an .hdr picture of no size, or over 64 megapixels".into());
        }
        let mut data = &bytes[at..];
        let mut texels = Vec::with_capacity(width * height);
        let mut row = vec![[0u8; 4]; width];
        for _ in 0..height {
            if (8..32768).contains(&width)
                && data.len() >= 4
                && data[0] == 2
                && data[1] == 2
                && usize::from(data[2]) << 8 | usize::from(data[3]) == width
            {
                // Run-length coded, a channel after a channel.
                data = &data[4..];
                for c in 0..4 {
                    let mut x = 0;
                    while x < width {
                        let (&n, rest) = data.split_first().ok_or_else(bad)?;
                        data = rest;
                        if n > 128 {
                            let (&v, rest) = data.split_first().ok_or_else(bad)?;
                            data = rest;
                            for _ in 0..n - 128 {
                                row.get_mut(x).ok_or_else(bad)?[c] = v;
                                x += 1;
                            }
                        } else {
                            let n = usize::from(n);
                            let run = data.get(..n).ok_or_else(bad)?;
                            data = &data[n..];
                            for &v in run {
                                row.get_mut(x).ok_or_else(bad)?[c] = v;
                                x += 1;
                            }
                        }
                    }
                }
            } else {
                let flat = data.get(..4 * width).ok_or_else(bad)?;
                data = &data[4 * width..];
                for (p, q) in row.iter_mut().zip(flat.as_chunks::<4>().0) {
                    *p = *q;
                }
            }
            for p in &row {
                // Mantissas times 2^(e − 136), by halving or doubling.
                let mut scale = 1.0f32;
                let e = i32::from(p[3]) - 136;
                for _ in 0..e.abs() {
                    scale *= if e > 0 { 2.0 } else { 0.5 };
                }
                texels.push(if p[3] == 0 {
                    [0.0; 3]
                } else {
                    [0, 1, 2].map(|c| f32::from(p[c]) * scale)
                });
            }
        }
        Self::new(width, height, &texels)
            .ok_or_else(|| "an .hdr picture with no light in it".to_string())
    }

    /// The way texel (u, v) of the picture lies, both in 0..1: u a
    /// turn round +Z from −X through −Y, v from straight up down.
    pub(crate) fn direction(u: f64, v: f64) -> DVec3 {
        let (st, ct) = trig::sin_cos(trig::PI * v);
        let (sp, cp) = trig::sin_cos(trig::TAU * (u - 0.5));
        DVec3::new(st * cp, st * sp, ct)
    }

    /// Where direction `d` (a unit vector) is in the picture.
    pub(crate) fn place(d: DVec3) -> (f64, f64) {
        (
            trig::atan2(d.y, d.x) / trig::TAU + 0.5,
            trig::acos(d.z.clamp(-1.0, 1.0)) / trig::PI,
        )
    }

    pub(crate) fn texel(w: usize, h: usize, texels: &[[f32; 3]], u: f64, v: f64) -> [f64; 3] {
        // Between the four nearest; round the turn, held at the poles.
        let (x, y) = (
            u * w as f64 - 0.5,
            (v * h as f64 - 0.5).clamp(0.0, h as f64 - 1.0),
        );
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |dx: i64, dy: i64| {
            let xi = (x0 as i64 + dx).rem_euclid(w as i64) as usize;
            let yi = ((y0 as i64 + dy).max(0) as usize).min(h - 1);
            texels[yi * w + xi]
        };
        let (a, b, c, d) = (at(0, 0), at(1, 0), at(0, 1), at(1, 1));
        [0, 1, 2].map(|i| {
            let top = f64::from(a[i]) * (1.0 - fx) + f64::from(b[i]) * fx;
            let low = f64::from(c[i]) * (1.0 - fx) + f64::from(d[i]) * fx;
            top * (1.0 - fy) + low * fy
        })
    }

    /// What a mirror of roughness `rough` facing `d` reflects: the
    /// picture there, from a smaller level the rougher it is.
    pub(crate) fn reflected(&self, d: DVec3, rough: f64) -> [f64; 3] {
        let (u, v) = Self::place(d);
        let level = rough.clamp(0.0, 1.0) * (self.levels.len() - 1) as f64;
        let (lo, part) = (level.floor() as usize, level - level.floor());
        let read = |k: usize| {
            let (w, h, texels) = &self.levels[k.min(self.levels.len() - 1)];
            Self::texel(*w, *h, texels, u, v)
        };
        let (a, b) = (read(lo), read(lo + 1));
        [0, 1, 2].map(|i| a[i] * (1.0 - part) + b[i] * part)
    }

    /// The light a matte surface with normal `n` gathers, over the
    /// picture's mean: 1 where it gathers as much as the plain light.
    pub(crate) fn gathered(&self, n: DVec3) -> [f64; 3] {
        let (u, v) = Self::place(n);
        let g = Self::texel(GATHERED.0, GATHERED.1, &self.gathered, u, v);
        g.map(|c| c / ((ENV_DOWN + ENV_UP) / 2.0))
    }
}
