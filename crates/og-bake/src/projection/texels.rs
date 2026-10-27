//! The texture's texels in world space: the level rasterised in uv space,
//! every covered texel its surface point, normal and triangle.

use super::{Level, par_for};
use glam::{Vec2, Vec3};
use std::io::{Read, Write};

/// Covered texels, sorted by their index in the `size × size` texture (rows
/// from the top, as the image is stored).
pub struct Texels {
    pub size: u32,
    pub index: Vec<u32>,
    pub position: Vec<Vec3>,
    pub normal: Vec<Vec3>,
    pub triangle: Vec<u32>,
}

const MAGIC: &[u8; 8] = b"OGTEXEL1";
/// A texel whose centre misses every triangle still takes the nearest one
/// within this many texels, so bilinear filtering at a chart's edge reads
/// painted texels.
const REACH: f32 = 0.75;

fn closest_on_triangle(p: Vec2, t: [Vec2; 3]) -> Vec2 {
    let seg = |a: Vec2, b: Vec2| {
        let ab = b - a;
        let s = ((p - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
        a + ab * s
    };
    [seg(t[0], t[1]), seg(t[1], t[2]), seg(t[2], t[0])]
        .into_iter()
        .min_by(|a, b| {
            (*a - p)
                .length_squared()
                .total_cmp(&(*b - p).length_squared())
        })
        .unwrap_or(p)
}

fn barycentric(q: Vec2, uv: [Vec2; 3], area: f32) -> Vec3 {
    let b1 = (q - uv[0]).perp_dot(uv[2] - uv[0]) / area;
    let b2 = (uv[1] - uv[0]).perp_dot(q - uv[0]) / area;
    Vec3::new(1.0 - b1 - b2, b1, b2)
}

impl Texels {
    /// Rasterises `level`'s uvs into a `size × size` texture. A texel whose
    /// centre lies inside a triangle belongs to it; one just outside every
    /// triangle takes the nearest within [`REACH`].
    pub fn rasterise(level: &Level, size: u32, threads: usize) -> Texels {
        let s = size as usize;
        // Per texel: triangle + 1, the top bit set when the centre is inside.
        const INSIDE: u32 = 1 << 31;
        let mut slots = vec![0u32; s * s];
        let uv_px = |t: usize| {
            level
                .corners(t)
                .map(|i| level.uvs[i as usize] * size as f32)
        };
        for t in 0..level.triangles() {
            let uv = uv_px(t);
            let area = (uv[1] - uv[0]).perp_dot(uv[2] - uv[0]);
            if area.abs() < 1e-9 {
                continue;
            }
            let lo = uv[0].min(uv[1]).min(uv[2]) - Vec2::splat(1.0);
            let hi = uv[0].max(uv[1]).max(uv[2]) + Vec2::splat(1.0);
            let (x0, y0) = (
                lo.x.floor().max(0.0) as usize,
                lo.y.floor().max(0.0) as usize,
            );
            let (x1, y1) = (
                (hi.x.ceil().max(0.0) as usize).min(s - 1),
                (hi.y.ceil().max(0.0) as usize).min(s - 1),
            );
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let c = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let inside = barycentric(c, uv, area).min_element() >= -1e-5;
                    if !inside && (closest_on_triangle(c, uv) - c).length() > REACH {
                        continue;
                    }
                    let slot = &mut slots[y * s + x];
                    if *slot & INSIDE != 0 || (*slot != 0 && !inside) {
                        continue;
                    }
                    *slot = (t as u32 + 1) | if inside { INSIDE } else { 0 };
                }
            }
        }
        let index: Vec<u32> = (0..s * s)
            .filter(|&i| slots[i] != 0)
            .map(|i| i as u32)
            .collect();
        let mut triangle: Vec<u32> = index
            .iter()
            .map(|&i| (slots[i as usize] & !INSIDE) - 1)
            .collect();
        drop(slots);
        let mut surface = vec![(Vec3::ZERO, Vec3::Y); index.len()];
        par_for(&mut surface, threads, |i, out| {
            let t = triangle[i] as usize;
            let uv = uv_px(t);
            let area = (uv[1] - uv[0]).perp_dot(uv[2] - uv[0]);
            let c = Vec2::new(
                (index[i] as usize % s) as f32 + 0.5,
                (index[i] as usize / s) as f32 + 0.5,
            );
            let mut w = barycentric(c, uv, area);
            if w.min_element() < 0.0 {
                // The nearest point of the triangle, nudged inside off the edge.
                let q = closest_on_triangle(c, uv);
                let centre = (uv[0] + uv[1] + uv[2]) / 3.0;
                w = barycentric(q + (centre - q) * 0.01, uv, area);
            }
            *out = (level.position_at(t, w), level.normal_at(t, w));
        });
        let (position, normal) = surface.into_iter().unzip();
        triangle.shrink_to_fit();
        Texels {
            size,
            index,
            position,
            normal,
            triangle,
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// The texel list entry of texture texel (x, y), if covered.
    pub fn find(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.size || y >= self.size {
            return None;
        }
        self.index.binary_search(&(y * self.size + x)).ok()
    }

    /// The texel list entry under a uv, if covered.
    pub fn at_uv(&self, uv: Vec2) -> Option<usize> {
        let p = uv * self.size as f32;
        if p.x < 0.0 || p.y < 0.0 {
            return None;
        }
        self.find(p.x as u32, p.y as u32)
    }

    pub fn write(&self, w: &mut impl Write) -> std::io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_all(&self.size.to_le_bytes())?;
        w.write_all(&(self.index.len() as u64).to_le_bytes())?;
        let mut buf = Vec::with_capacity(self.index.len() * 32);
        for i in 0..self.index.len() {
            buf.extend_from_slice(&self.index[i].to_le_bytes());
            buf.extend_from_slice(&self.triangle[i].to_le_bytes());
            for v in self.position[i]
                .to_array()
                .into_iter()
                .chain(self.normal[i].to_array())
            {
                buf.extend_from_slice(&v.to_le_bytes());
            }
            if buf.len() > 1 << 24 {
                w.write_all(&buf)?;
                buf.clear();
            }
        }
        w.write_all(&buf)
    }

    pub fn read(r: &mut impl Read) -> std::io::Result<Texels> {
        let bad = |m: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, m.to_string());
        let mut head = [0u8; 20];
        r.read_exact(&mut head)?;
        if &head[..8] != MAGIC {
            return Err(bad("not a texel cache"));
        }
        let size = u32::from_le_bytes(head[8..12].try_into().unwrap_or_default());
        let n = u64::from_le_bytes(head[12..20].try_into().unwrap_or_default()) as usize;
        let mut body = vec![0u8; n * 32];
        r.read_exact(&mut body)?;
        let f = |b: &[u8], k: usize| f32::from_le_bytes(b[k..k + 4].try_into().unwrap_or_default());
        let mut t = Texels {
            size,
            index: Vec::with_capacity(n),
            position: Vec::with_capacity(n),
            normal: Vec::with_capacity(n),
            triangle: Vec::with_capacity(n),
        };
        for b in body.as_chunks::<32>().0 {
            t.index
                .push(u32::from_le_bytes(b[0..4].try_into().unwrap_or_default()));
            t.triangle
                .push(u32::from_le_bytes(b[4..8].try_into().unwrap_or_default()));
            t.position.push(Vec3::new(f(b, 8), f(b, 12), f(b, 16)));
            t.normal.push(Vec3::new(f(b, 20), f(b, 24), f(b, 28)));
        }
        Ok(t)
    }
}
