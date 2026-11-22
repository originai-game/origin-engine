//! The source's look read at a point: its maps sampled, its factors applied, values as bytes.

use super::*;

/// Barycentric weights of `p` (on or near the triangle's plane) in `t`,
/// clamped to the triangle; a degenerate triangle gives its first corner.
pub(crate) fn bary3(p: DVec3, t: [DVec3; 3]) -> [f64; 3] {
    let (v0, v1, v2) = (t[1] - t[0], t[2] - t[0], p - t[0]);
    let (d00, d01, d11) = (v0.dot(v0), v0.dot(v1), v1.dot(v1));
    let (d20, d21) = (v2.dot(v0), v2.dot(v1));
    let den = d00 * d11 - d01 * d01;
    if den <= 0.0 {
        return [1.0, 0.0, 0.0];
    }
    let v = ((d11 * d20 - d01 * d21) / den).clamp(0.0, 1.0);
    let w = ((d00 * d21 - d01 * d20) / den).clamp(0.0, 1.0 - v);
    [1.0 - v - w, v, w]
}

/// A decoded map, sampled bilinear with repeat wrap; colour channels in
/// linear light.
pub(crate) struct Map {
    pub(crate) image: Image,
    pub(crate) color: bool,
}

impl Map {
    /// RGBA in 0..=1 at `uv` (colour linear when the map holds colour):
    /// the texel grid's centres at (i + ½) / width, v = 0 at the bottom
    /// row, weights `floor(256 × fraction)`, integer sums.
    pub(crate) fn sample(&self, lin: &Linear, uv: DVec2) -> [f64; 4] {
        let l = &self.image;
        let (w, h) = (i64::from(l.width), i64::from(l.height));
        let fin = |v: f64| {
            if v.is_finite() {
                v.clamp(-1e9, 1e9)
            } else {
                0.0
            }
        };
        let x = fin(uv.x) * w as f64 - 0.5;
        let y = (1.0 - fin(uv.y)) * h as f64 - 0.5;
        let (fx0, fy0) = (x.floor(), y.floor());
        let wx = ((x - fx0) * 256.0).floor() as u64;
        let wy = ((y - fy0) * 256.0).floor() as u64;
        let (x0, y0) = (fx0 as i64, fy0 as i64);
        let tex = |xx: i64, yy: i64| -> usize {
            let (xx, yy) = (xx.rem_euclid(w), yy.rem_euclid(h));
            ((yy * w + xx) * 4) as usize
        };
        let q = [
            (tex(x0, y0), (256 - wx) * (256 - wy)),
            (tex(x0 + 1, y0), wx * (256 - wy)),
            (tex(x0, y0 + 1), (256 - wx) * wy),
            (tex(x0 + 1, y0 + 1), wx * wy),
        ];
        let mut out = [0.0; 4];
        for (c, o) in out.iter_mut().enumerate() {
            let s: u64 = q
                .iter()
                .map(|&(i, wt)| {
                    let b = l.rgba[i + c];
                    let v = if self.color && c < 3 {
                        u64::from(lin.get(b))
                    } else {
                        u64::from(b) * 257
                    };
                    v * wt
                })
                .sum();
            *o = (s / 65536) as f64 / 65535.0;
        }
        out
    }
}

/// Linear light in 0..=1 to an sRGB byte (nearest by the table).
pub(crate) fn srgb_byte(lin: &Linear, v: f64) -> u8 {
    let v = if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    };
    lin.encode((v * 65535.0 + 0.5).floor() as u32)
}

/// 0..=1 to a byte as stored.
pub(crate) fn data_byte(v: f64) -> u8 {
    let v = if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    };
    (v * 255.0 + 0.5).floor() as u8
}

pub(crate) fn unit(v: DVec3) -> DVec3 {
    let l = v.length();
    if l > 0.0 { v / l } else { DVec3::ZERO }
}

/// A material's factors in linear light.
pub(crate) struct Factors {
    pub(crate) color: [f64; 4],
    pub(crate) metalness: f64,
    pub(crate) roughness: f64,
    pub(crate) emissive: [f64; 3],
    pub(crate) ao: f64,
    pub(crate) normal_scale: f64,
}

pub(crate) fn factors(m: &Material) -> Factors {
    let c = m.color.unwrap_or([1.0; 3]);
    let lin = |v: f64| match m.color_space {
        ColorSpace::Linear => v,
        ColorSpace::Srgb => to_linear(v.clamp(0.0, 1.0)),
    };
    let ei = m.emissive_intensity.unwrap_or(1.0);
    Factors {
        color: [lin(c[0]), lin(c[1]), lin(c[2]), m.opacity.unwrap_or(1.0)],
        metalness: m.metalness,
        roughness: m.roughness.unwrap_or(1.0),
        emissive: m.emissive.map(|e| lin(e) * ei),
        ao: m.ao_map_intensity.unwrap_or(1.0),
        normal_scale: m.normal_scale.unwrap_or(1.0),
    }
}

/// A material without its name and maps: what decides whether the
/// factors can stay on the material.
pub(crate) fn bare(m: &Material) -> Material {
    Material {
        name: String::new(),
        map: None,
        metalness_map: None,
        normal_map: None,
        emissive_map: None,
        ao_map: None,
        ..m.clone()
    }
}
