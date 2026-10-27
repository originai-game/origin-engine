//! The texture being painted, one entry per covered texel: its sRGB colour
//! and the quality it was captured at (0: unpainted).

use super::{GBuffer, Texels, par_for};
use glam::Vec2;
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Texel {
    pub rgb: [u8; 3],
    pub quality: f32,
}

/// The painted texture, aligned with a [`Texels`] list.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub texels: Vec<Texel>,
}

/// What one projection did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Projected {
    /// Texels in front of the camera, in its frame and facing it.
    pub seen: usize,
    /// Texels that took this view's colour.
    pub written: usize,
}

const MAGIC: &[u8; 8] = b"OGPAINT1";

impl Paint {
    pub fn new(texels: &Texels) -> Paint {
        Paint {
            texels: vec![Texel::default(); texels.len()],
        }
    }

    /// The share of covered texels painted.
    pub fn coverage(&self) -> f32 {
        let n = self.texels.iter().filter(|t| t.quality > 0.0).count();
        n as f32 / self.texels.len().max(1) as f32
    }

    pub fn write(&self, w: &mut impl Write) -> std::io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_all(&(self.texels.len() as u64).to_le_bytes())?;
        let mut buf = Vec::with_capacity(self.texels.len() * 7);
        for t in &self.texels {
            buf.extend_from_slice(&t.rgb);
            buf.extend_from_slice(&t.quality.to_le_bytes());
        }
        w.write_all(&buf)
    }

    pub fn read(r: &mut impl Read) -> std::io::Result<Paint> {
        let mut head = [0u8; 16];
        r.read_exact(&mut head)?;
        if &head[..8] != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a paint file",
            ));
        }
        let n = u64::from_le_bytes(head[8..16].try_into().unwrap_or_default()) as usize;
        let mut body = vec![0u8; n * 7];
        r.read_exact(&mut body)?;
        Ok(Paint {
            texels: body
                .as_chunks::<7>()
                .0
                .iter()
                .map(|b| Texel {
                    rgb: [b[0], b[1], b[2]],
                    quality: f32::from_le_bytes(b[3..7].try_into().unwrap_or_default()),
                })
                .collect(),
        })
    }
}

/// Pixels within a silhouette band of `6 px at 1024` scaled to the view size:
/// a model's slight misalignment paints the wrong side onto an edge, so
/// those pixels are captured at reduced quality and a view that sees the
/// texel away from an edge replaces them.
pub fn silhouette(g: &GBuffer) -> Vec<u8> {
    g.silhouette(6.0 * g.size as f32 / 1024.0)
}

/// Bilinear RGBA of a `size`² image (rows from the top) at continuous pixel
/// coordinates; premultiplied alpha averages only the pixels a view keeps.
fn sample(image: &[[u8; 4]], size: u32, p: Vec2) -> [f32; 4] {
    let s = size as i64;
    let (fx, fy) = (p.x - 0.5, p.y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (ax, ay) = (fx - x0, fy - y0);
    let at = |x: i64, y: i64| {
        let c = image[(y.clamp(0, s - 1) * s + x.clamp(0, s - 1)) as usize];
        let a = c[3] as f32 / 255.0;
        [
            c[0] as f32 * a,
            c[1] as f32 * a,
            c[2] as f32 * a,
            c[3] as f32,
        ]
    };
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (c00, c10, c01, c11) = (
        at(x0, y0),
        at(x0 + 1, y0),
        at(x0, y0 + 1),
        at(x0 + 1, y0 + 1),
    );
    std::array::from_fn(|k| {
        let top = c00[k] + (c10[k] - c00[k]) * ax;
        let bottom = c01[k] + (c11[k] - c01[k]) * ax;
        top + (bottom - top) * ay
    })
}

/// Writes `image` (this view's composited painting, RGBA rows from the top;
/// alpha 0 where the view must not project) into every texel this view sees
/// better than what `paint` holds: a stored texel is replaced when its
/// quality is under `ratio` × this view's.
///
/// A texel is seen when the surface under its pixel is the texel's own
/// (depth within 1 cm + 0.5 % and normals agreeing), so a texel just behind
/// the visible surface is never painted with it.
pub fn project(
    texels: &Texels,
    paint: &mut Paint,
    g: &GBuffer,
    edge: &[u8],
    image: &[[u8; 4]],
    ratio: f32,
    threads: usize,
) -> Projected {
    let size = g.size;
    let view = g.view;
    let (f, ..) = view.axes();
    let mut seen = vec![0u8; texels.len()];
    {
        let mut work: Vec<(&mut Texel, &mut u8)> =
            paint.texels.iter_mut().zip(seen.iter_mut()).collect();
        par_for(&mut work, threads, |i, (out, saw)| {
            let p = texels.position[i];
            let to_cam = view.position - p;
            let d = to_cam.length();
            if d < 1e-4 || (-to_cam).dot(f) <= 0.0 {
                return;
            }
            let cos = texels.normal[i].dot(to_cam / d);
            if cos < 0.08 {
                return;
            }
            let Some(px) = view.project(p, size) else {
                return;
            };
            if px.x < 0.0 || px.y < 0.0 || px.x >= size as f32 || px.y >= size as f32 {
                return;
            }
            let pi = (px.y as u32 * size + px.x as u32) as usize;
            let Some(h) = g.hits[pi] else { return };
            let same = h.triangle == texels.triangle[i]
                || ((h.t - d).abs() <= 0.01 + 0.005 * d && h.normal.dot(texels.normal[i]) > 0.7);
            if !same {
                return;
            }
            **saw = 1;
            let mut q = cos / d.max(0.5);
            if edge[pi] != 0 {
                q *= 0.3;
            }
            if out.quality > 0.0 && out.quality >= ratio * q {
                return;
            }
            let c = sample(image, size, px);
            if c[3] < 127.5 {
                return;
            }
            let k = 255.0 / c[3];
            out.rgb = [0, 1, 2].map(|j| (c[j] * k).round().clamp(0.0, 255.0) as u8);
            out.quality = q;
            **saw = 2;
        });
    }
    Projected {
        seen: seen.iter().filter(|&&s| s > 0).count(),
        written: seen.iter().filter(|&&s| s == 2).count(),
    }
}

/// The texture as an RGB image (rows from the top): painted texels, then
/// `rings` rings of gutter filled from their painted neighbours so bilinear
/// sampling never reads `fill`, which every other texel keeps.
pub fn padded(texels: &Texels, paint: &Paint, rings: u32, fill: [u8; 3]) -> Vec<[u8; 3]> {
    let s = texels.size as usize;
    let mut img = vec![fill; s * s];
    let mut have = vec![false; s * s];
    for (i, t) in paint.texels.iter().enumerate() {
        if t.quality > 0.0 {
            let k = texels.index[i] as usize;
            img[k] = t.rgb;
            have[k] = true;
        }
    }
    let mut frontier: Vec<usize> = (0..s * s).filter(|&k| have[k]).collect();
    for _ in 0..rings {
        let mut next = Vec::new();
        let mut add: Vec<(usize, [u8; 3])> = Vec::new();
        let mut queued = std::collections::HashSet::new();
        for &k in &frontier {
            let (x, y) = ((k % s) as i64, (k / s) as i64);
            for (dx, dy) in [
                (-1i64, 0i64),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
            ] {
                let (xx, yy) = (x + dx, y + dy);
                if xx < 0 || yy < 0 || xx >= s as i64 || yy >= s as i64 {
                    continue;
                }
                let j = yy as usize * s + xx as usize;
                if have[j] || !queued.insert(j) {
                    continue;
                }
                let (mut acc, mut n) = ([0u32; 3], 0u32);
                for (ex, ey) in [
                    (-1i64, 0i64),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (1, 1),
                    (-1, 1),
                    (1, -1),
                ] {
                    let (ax, ay) = (xx + ex, yy + ey);
                    if ax < 0 || ay < 0 || ax >= s as i64 || ay >= s as i64 {
                        continue;
                    }
                    let a = ay as usize * s + ax as usize;
                    if have[a] {
                        for c in 0..3 {
                            acc[c] += img[a][c] as u32;
                        }
                        n += 1;
                    }
                }
                add.push((j, acc.map(|v| (v / n.max(1)) as u8)));
            }
        }
        for (j, c) in add {
            img[j] = c;
            have[j] = true;
            next.push(j);
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    img
}
