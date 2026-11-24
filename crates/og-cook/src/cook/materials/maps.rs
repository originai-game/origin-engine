//! The maps `og materials make` writes from one source image: its colour
//! made periodic, and a tangent-space normal map from its brightness.

use super::periodic::{Plane, make_periodic};

/// An image as three channels of 0 to 1 (sRGB, as stored), row by row.
pub(crate) struct Rgb {
    pub width: usize,
    pub height: usize,
    pub channels: [Vec<f64>; 3],
}

impl Rgb {
    pub(crate) fn from_rgba8(width: usize, height: usize, rgba: &[u8]) -> Rgb {
        let channel = |c: usize| {
            rgba.as_chunks::<4>()
                .0
                .iter()
                .map(|p| f64::from(p[c]) / 255.0)
                .collect()
        };
        Rgb {
            width,
            height,
            channels: [channel(0), channel(1), channel(2)],
        }
    }

    /// Each channel made periodic across (and, with `wrap_y`, up), then
    /// stored as 8 bits again.
    pub(crate) fn periodic(&mut self, wrap_y: bool) {
        for c in &mut self.channels {
            make_periodic(
                Plane {
                    width: self.width,
                    height: self.height,
                    data: c,
                },
                wrap_y,
            );
            for v in c.iter_mut() {
                *v = (v.clamp(0.0, 1.0) * 255.0).round() / 255.0;
            }
        }
    }

    /// The mean step between the edges the image wraps over (left to
    /// right, and with `wrap_y` top to bottom), 0 to 1: how far a tile's
    /// seam shows.
    pub(crate) fn seam(&self, wrap_y: bool) -> f64 {
        let (w, h) = (self.width, self.height);
        let (mut sum, mut n) = (0.0, 0usize);
        for c in &self.channels {
            for y in 0..h {
                sum += (c[y * w] - c[y * w + w - 1]).abs();
                n += 1;
            }
            if wrap_y {
                for x in 0..w {
                    sum += (c[x] - c[(h - 1) * w + x]).abs();
                    n += 1;
                }
            }
        }
        sum / n as f64
    }

    /// The mean step between neighbours inside the image, the same way:
    /// what a seam that does not show is measured against.
    pub(crate) fn interior(&self, wrap_y: bool) -> f64 {
        let (w, h) = (self.width, self.height);
        let (mut sum, mut n) = (0.0, 0usize);
        for c in &self.channels {
            for y in 0..h {
                for x in 0..w - 1 {
                    sum += (c[y * w + x] - c[y * w + x + 1]).abs();
                    n += 1;
                }
            }
            if wrap_y {
                for y in 0..h - 1 {
                    for x in 0..w {
                        sum += (c[y * w + x] - c[(y + 1) * w + x]).abs();
                        n += 1;
                    }
                }
            }
        }
        sum / n as f64
    }

    /// Linear luminance (Rec. 709) of each texel.
    pub(crate) fn luminance(&self) -> Vec<f64> {
        let lin = |v: f64| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                og_core::math::double::pow((v + 0.055) / 1.055, 2.4)
            }
        };
        let [r, g, b] = &self.channels;
        (0..self.width * self.height)
            .map(|i| 0.2126 * lin(r[i]) + 0.7152 * lin(g[i]) + 0.0722 * lin(b[i]))
            .collect()
    }

    pub(crate) fn to_rgb8(&self) -> Vec<u8> {
        (0..self.width * self.height)
            .flat_map(|i| {
                self.channels
                    .iter()
                    .map(move |c| (c[i] * 255.0).round() as u8)
            })
            .collect()
    }
}

/// A tangent-space normal map (glTF: red right, green up the image, blue
/// out) of the height `relief × luminance` metres (a negative relief
/// sinks the lighter), the image covering
/// `tile` metres across and up. Across always wraps; up wraps with
/// `wrap_y`, else the edge rows take one-sided steps.
pub(crate) fn normal_map(
    lum: &[f64],
    width: usize,
    height: usize,
    tile: [f64; 2],
    relief: f64,
    wrap_y: bool,
) -> Vec<u8> {
    let h = |x: usize, y: usize| lum[y * width + x] * relief;
    let (dx, dy) = (tile[0] / width as f64, tile[1] / height as f64);
    let mut out = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        // Rows run down the image; "up" is the row above.
        let (above, below, rows) = if wrap_y {
            ((y + height - 1) % height, (y + 1) % height, 2.0)
        } else {
            let a = y.saturating_sub(1);
            let b = (y + 1).min(height - 1);
            (a, b, (b - a) as f64)
        };
        for x in 0..width {
            let (l, r) = ((x + width - 1) % width, (x + 1) % width);
            let sx = (h(r, y) - h(l, y)) / (2.0 * dx);
            let sy = (h(x, above) - h(x, below)) / (rows * dy);
            let len = (sx * sx + sy * sy + 1.0).sqrt();
            for v in [-sx / len, -sy / len, 1.0 / len] {
                out.push(((v * 0.5 + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8);
            }
        }
    }
    out
}

/// A local cavity approximation, not measured AO: darken a texel below
/// the mean of eight samples on a ring, by the game's strength. Height
/// sign follows relief, so light mortar can be below dark brick. Radius
/// is in texels; nothing is inferred from the subject's name or taste.
pub(crate) fn occlusion_map(
    lum: &[f64],
    width: usize,
    height: usize,
    relief: f64,
    [strength, radius]: [f64; 2],
    wrap_y: bool,
) -> Vec<u8> {
    let radius = radius as isize;
    let at = |x: isize, y: isize| {
        let x = x.rem_euclid(width as isize) as usize;
        let y = if wrap_y {
            y.rem_euclid(height as isize)
        } else {
            y.clamp(0, height as isize - 1)
        } as usize;
        if relief < 0.0 {
            1.0 - lum[y * width + x]
        } else {
            lum[y * width + x]
        }
    };
    let mut out = Vec::with_capacity(width * height * 3);
    for y in 0..height as isize {
        for x in 0..width as isize {
            let mut surrounding = 0.0;
            for (dx, dy) in [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                surrounding += at(x + dx * radius, y + dy * radius);
            }
            let cavity = (surrounding / 8.0 - at(x, y)).max(0.0);
            let v = ((1.0 - strength * cavity) * 255.0).round() as u8;
            out.extend([v; 3]);
        }
    }
    out
}
