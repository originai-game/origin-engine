//! displace: a solid's surface moved along its normals by noise, for
//! organic forms (rocks, asteroids, a moon's cratered limb).
//!
//! The noise is a function of the point alone: gradient noise on an
//! integer lattice, hashed with the seed. Its arithmetic is `+ - * /`,
//! `floor` and one square root per normal, each correctly rounded, in a
//! fixed order, so the same solid and desc give the same bytes on every
//! host. It moves vertices and adds none: subdivide the part first
//! (a `sphere` or a `box` with `subdivide`) until its edges are shorter
//! than about a quarter of `scale`.

use super::*;

/// The noise a [`Displace`] moves the surface by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Noise {
    /// Fractal gradient noise: soft lumps (weathered rock, an asteroid).
    #[default]
    Fbm,
    /// Ridged fractal noise: sharp crests and creases (broken stone, ice).
    Ridged,
    /// Bowls with raised rims scattered over the surface (a moon, a
    /// pocked asteroid); `amount` is a bowl's depth.
    Crater,
}

/// How to displace. `scale` and `amount` are in the program's units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Displace {
    /// Which noise.
    pub noise: Noise,
    /// The size of the largest feature: the first octave's wavelength, or
    /// the crater lattice's cell (a crater's radius is 0.2 to 0.45 of it).
    pub scale: f64,
    /// How far the surface moves at most, outwards for positive values
    /// (a crater's depth below the surface).
    pub amount: f64,
    /// Octaves of fbm and ridged noise, each half the wavelength and half
    /// the height of the one before; 0 is four. At most [`MAX_OCTAVES`].
    pub octaves: u32,
    /// Picks another field of the same noise.
    pub seed: u32,
    /// The share of crater cells that hold a crater, 0..1; 0 is a half.
    pub density: f64,
}

/// The most octaves [`displace`] takes.
pub const MAX_OCTAVES: u32 = 8;

/// Moves every vertex along its area-weighted normal by the noise at its
/// place (the noises' values are about −1..1, ridged's crests 1, a
/// crater's floor −1). Faces and names are kept; the result's own crossing
/// pairs are reported, as [`dent`]'s are: too large an `amount` for the
/// mesh's detail folds the surface through itself.
pub fn displace_report(s: &Solid, d: &Displace) -> Result<(Solid, EditReport), EditError> {
    let out = displace(s, d)?;
    let report = EditReport {
        crossing_pairs: crossing_pairs(&out),
    };
    Ok((out, report))
}

/// [`displace_report`] without the report.
pub fn displace(s: &Solid, d: &Displace) -> Result<Solid, EditError> {
    if crate::mesh::rides(s) {
        let mut out = displace(&crate::mesh::bare(s), d)?;
        let _ = crate::mesh::carry(&[s], &mut out);
        return Ok(out);
    }
    if !positive(d.scale) {
        return Err(EditError::NotPositive("scale"));
    }
    if !d.amount.is_finite() || d.amount == 0.0 {
        return Err(EditError::Unsupported(
            "displace: amount must be a finite distance other than 0",
        ));
    }
    if d.octaves > MAX_OCTAVES {
        return Err(EditError::Unsupported("displace takes at most 8 octaves"));
    }
    if !(0.0..=1.0).contains(&d.density) {
        return Err(EditError::Unsupported("displace: density is a share, 0..1"));
    }
    let mut normals = vec![DVec3::ZERO; s.positions.len()];
    for t in &s.triangles {
        let [a, b, c] = t.map(|v| s.positions[v as usize]);
        let n = (b - a).cross(c - a);
        for &v in t {
            normals[v as usize] += n;
        }
    }
    let mut out = s.clone();
    for (v, p) in s.positions.iter().enumerate() {
        let h = height(d, *p / d.scale);
        out.positions[v] = *p + normals[v].normalize_or_zero() * (d.amount * h);
    }
    out.canonicalize()
        .map_err(|_| EditError::Unsupported("displace: the surface collapsed"))?;
    Ok(out)
}

/// The noise's value at `p`, in lattice units (the point over `scale`).
fn height(d: &Displace, p: DVec3) -> f64 {
    let octaves = if d.octaves == 0 { 4 } else { d.octaves };
    match d.noise {
        Noise::Fbm | Noise::Ridged => {
            let (mut sum, mut norm, mut f, mut w) = (0.0, 0.0, 1.0, 1.0);
            for o in 0..octaves {
                let seed = d.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
                let n = gradient(p * f, seed);
                sum += w * match d.noise {
                    Noise::Ridged => {
                        let r = 1.0 - n.abs();
                        r * r
                    }
                    _ => n,
                };
                norm += w;
                f *= 2.0;
                w *= 0.5;
            }
            let v = sum / norm;
            match d.noise {
                // Crests at 1, the creases between at −1.
                Noise::Ridged => 2.0 * v - 1.0,
                _ => v,
            }
        }
        Noise::Crater => craters(p, d.seed, if d.density == 0.0 { 0.5 } else { d.density }),
    }
}

/// Lowbias32 (Chris Wellons): a 32-bit integer hash.
pub(crate) fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// The lattice point (x, y, z)'s hash under `seed`.
pub(crate) fn corner(x: i64, y: i64, z: i64, seed: u32) -> u32 {
    let h = mix(seed ^ (x as u32));
    let h = mix(h ^ (y as u32).wrapping_mul(0x27d4_eb2d));
    mix(h ^ (z as u32).wrapping_mul(0x1656_67b1))
}

/// `h`'s share of 2³², in [0, 1).
pub(crate) fn unit(h: u32) -> f64 {
    f64::from(h) / 4_294_967_296.0
}

/// The twelve edge directions of a cube: Perlin's improved gradients.
const GRADIENTS: [[f64; 3]; 12] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
];

/// Quintic fade, 6t⁵ − 15t⁴ + 10t³.
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Gradient noise at `p`, about −1..1, 0 on every lattice point.
pub(crate) fn gradient(p: DVec3, seed: u32) -> f64 {
    let (fx, fy, fz) = (p.x.floor(), p.y.floor(), p.z.floor());
    let (x, y, z) = (fx as i64, fy as i64, fz as i64);
    let (dx, dy, dz) = (p.x - fx, p.y - fy, p.z - fz);
    let dot = |i: i64, j: i64, k: i64| {
        let g = GRADIENTS[(corner(x + i, y + j, z + k, seed) % 12) as usize];
        g[0] * (dx - i as f64) + g[1] * (dy - j as f64) + g[2] * (dz - k as f64)
    };
    let (u, v, w) = (fade(dx), fade(dy), fade(dz));
    let x0 = lerp(
        lerp(dot(0, 0, 0), dot(1, 0, 0), u),
        lerp(dot(0, 1, 0), dot(1, 1, 0), u),
        v,
    );
    let x1 = lerp(
        lerp(dot(0, 0, 1), dot(1, 0, 1), u),
        lerp(dot(0, 1, 1), dot(1, 1, 1), u),
        v,
    );
    lerp(x0, x1, w)
}

/// The craters' height at `p`: per lattice cell a crater with chance
/// `density`, its centre and radius from the cell's hash; each is a bowl
/// (−1 at its centre) with a rim rising to 0.3 at its edge and falling to
/// 0 at 1.6 radii. Overlapping craters add, the deepest bowl winning
/// inside it.
pub(crate) fn craters(p: DVec3, seed: u32, density: f64) -> f64 {
    let (x, y, z) = (p.x.floor() as i64, p.y.floor() as i64, p.z.floor() as i64);
    let (mut bowl, mut rim) = (0.0f64, 0.0f64);
    for k in -1..=1 {
        for j in -1..=1 {
            for i in -1..=1 {
                let (cx, cy, cz) = (x + i, y + j, z + k);
                let h = corner(cx, cy, cz, seed);
                if unit(h) >= density {
                    continue;
                }
                let at = DVec3::new(
                    cx as f64 + unit(mix(h ^ 1)),
                    cy as f64 + unit(mix(h ^ 2)),
                    cz as f64 + unit(mix(h ^ 3)),
                );
                let radius = 0.2 + 0.25 * unit(mix(h ^ 4));
                let t = (p - at).length() / radius;
                if t < 1.0 {
                    let t2 = t * t;
                    bowl = bowl.min(t2 - 1.0 + 0.3 * t2 * t2 * t2);
                } else if t < 1.6 {
                    let f = 1.0 - (t - 1.0) / 0.6;
                    rim += 0.3 * f * f;
                }
            }
        }
    }
    if bowl < 0.0 { bowl } else { rim.min(0.6) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_noise_is_zero_on_the_lattice_and_bounded() {
        for s in 0..4 {
            assert_eq!(gradient(DVec3::new(3.0, -2.0, 7.0), s), 0.0);
        }
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        for i in 0..4000 {
            let f = i as f64;
            let v = gradient(DVec3::new(f * 0.137, f * 0.071 - 9.0, f * 0.053), 5);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        assert!(lo > -1.1 && hi < 1.1 && lo < -0.3 && hi > 0.3, "{lo} {hi}");
    }

    #[test]
    fn craters_are_bowls_with_rims() {
        let d = Displace {
            noise: Noise::Crater,
            scale: 1.0,
            amount: 1.0,
            density: 1.0,
            ..Default::default()
        };
        // Every cell holds a crater: the cell's own centre is a bowl floor.
        let h = corner(0, 0, 0, 0);
        let at = DVec3::new(unit(mix(h ^ 1)), unit(mix(h ^ 2)), unit(mix(h ^ 3)));
        assert!(height(&d, at) <= -0.99);
    }
}
