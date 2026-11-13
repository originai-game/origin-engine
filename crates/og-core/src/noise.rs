//! Gradient noise on a seed: Perlin and simplex in two and three
//! dimensions, and their fractal sum ([`Fbm`]). The one owner of CPU
//! gradient noise.
//!
//! Determinism: a seed's gradient table is drawn from an [`Rng`] on it in
//! a fixed order, and the arithmetic is `+ - * /` and `floor` on f32, so
//! the same seed gives the same field on every host.

use crate::Rng;
use crate::procgen::permutation;
use std::cell::RefCell;
use std::sync::Arc;

/// A seed's gradient table: 0..255 shuffled by its stream, twice over.
struct Table([u8; 512]);

impl Table {
    fn new(seed: u32) -> Table {
        let p = permutation(&mut Rng::new(seed), 256);
        let mut t = [0u8; 512];
        for i in 0..512 {
            t[i] = p[i & 255] as u8;
        }
        Table(t)
    }
    #[inline]
    fn at(&self, i: i32) -> usize {
        self.0[(i & 511) as usize] as usize
    }
}

thread_local! {
    /// The last few seeds' tables (building one is 255 draws).
    static TABLES: RefCell<Vec<(u32, Arc<Table>)>> = const { RefCell::new(Vec::new()) };
}

fn table(seed: u32) -> Arc<Table> {
    TABLES.with(|t| {
        let mut t = t.borrow_mut();
        if let Some((_, tab)) = t.iter().find(|(s, _)| *s == seed) {
            return tab.clone();
        }
        let tab = Arc::new(Table::new(seed));
        if t.len() >= 8 {
            t.remove(0);
        }
        t.push((seed, tab.clone()));
        tab
    })
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn grad3(h: usize, x: f32, y: f32, z: f32) -> f32 {
    // Ken Perlin's improved noise: twelve edge gradients (sixteen slots).
    let h = h & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

#[inline]
fn grad2(h: usize, x: f32, y: f32) -> f32 {
    // Eight gradients: the axes and the diagonals.
    match h & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

/// Improved Perlin noise at (x, y), about -1 to 1, 0 on every lattice point.
pub fn perlin2(seed: u32, x: f32, y: f32) -> f32 {
    let t = table(seed);
    let (fx, fy) = (x.floor(), y.floor());
    let (xi, yi) = (fx as i32, fy as i32);
    let (x, y) = (x - fx, y - fy);
    let (u, v) = (fade(x), fade(y));
    let a = t.at(xi) as i32 + yi;
    let b = t.at(xi + 1) as i32 + yi;
    let (aa, ab, ba, bb) = (t.at(a), t.at(a + 1), t.at(b), t.at(b + 1));
    let n = lerp(
        lerp(grad2(aa, x, y), grad2(ba, x - 1.0, y), u),
        lerp(grad2(ab, x, y - 1.0), grad2(bb, x - 1.0, y - 1.0), u),
        v,
    );
    // The diagonal gradients reach about ±1.4 between lattice points.
    n * 0.7
}

/// Improved Perlin noise at (x, y, z), about -1 to 1.
pub fn perlin3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    let t = table(seed);
    let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (fx as i32, fy as i32, fz as i32);
    let (x, y, z) = (x - fx, y - fy, z - fz);
    let (u, v, w) = (fade(x), fade(y), fade(z));
    let a = t.at(xi) as i32 + yi;
    let aa = t.at(a) as i32 + zi;
    let ab = t.at(a + 1) as i32 + zi;
    let b = t.at(xi + 1) as i32 + yi;
    let ba = t.at(b) as i32 + zi;
    let bb = t.at(b + 1) as i32 + zi;
    lerp(
        lerp(
            lerp(grad3(t.at(aa), x, y, z), grad3(t.at(ba), x - 1.0, y, z), u),
            lerp(
                grad3(t.at(ab), x, y - 1.0, z),
                grad3(t.at(bb), x - 1.0, y - 1.0, z),
                u,
            ),
            v,
        ),
        lerp(
            lerp(
                grad3(t.at(aa + 1), x, y, z - 1.0),
                grad3(t.at(ba + 1), x - 1.0, y, z - 1.0),
                u,
            ),
            lerp(
                grad3(t.at(ab + 1), x, y - 1.0, z - 1.0),
                grad3(t.at(bb + 1), x - 1.0, y - 1.0, z - 1.0),
                u,
            ),
            v,
        ),
        w,
    )
}

/// Simplex noise at (x, y) (Gustavson's construction), about -1 to 1.
pub fn simplex2(seed: u32, x: f32, y: f32) -> f32 {
    // (sqrt(3) - 1) / 2 and (3 - sqrt(3)) / 6, as f32 literals.
    const F2: f32 = 0.366_025_42;
    const G2: f32 = 0.211_324_87;
    let t = table(seed);
    let s = (x + y) * F2;
    let (i, j) = ((x + s).floor(), (y + s).floor());
    let tt = (i + j) * G2;
    let (x0, y0) = (x - (i - tt), y - (j - tt));
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let (x1, y1) = (x0 - i1 as f32 + G2, y0 - j1 as f32 + G2);
    let (x2, y2) = (x0 - 1.0 + 2.0 * G2, y0 - 1.0 + 2.0 * G2);
    let (ii, jj) = (i as i32 & 255, j as i32 & 255);
    let corner = |dx: f32, dy: f32, h: usize| {
        let r = 0.5 - dx * dx - dy * dy;
        if r < 0.0 {
            0.0
        } else {
            let r2 = r * r;
            r2 * r2 * grad2(h, dx, dy)
        }
    };
    let n0 = corner(x0, y0, t.at(ii + t.at(jj) as i32));
    let n1 = corner(x1, y1, t.at(ii + i1 + t.at(jj + j1) as i32));
    let n2 = corner(x2, y2, t.at(ii + 1 + t.at(jj + 1) as i32));
    // Scaled so the range is about -1 to 1.
    45.23 * (n0 + n1 + n2)
}

/// Simplex noise at (x, y, z), about -1 to 1.
pub fn simplex3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;
    let t = table(seed);
    let s = (x + y + z) * F3;
    let (i, j, k) = ((x + s).floor(), (y + s).floor(), (z + s).floor());
    let tt = (i + j + k) * G3;
    let (x0, y0, z0) = (x - (i - tt), y - (j - tt), z - (k - tt));
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else if y0 < z0 {
        (0, 0, 1, 0, 1, 1)
    } else if x0 < z0 {
        (0, 1, 0, 0, 1, 1)
    } else {
        (0, 1, 0, 1, 1, 0)
    };
    let offs = [
        (0, 0, 0, 0.0),
        (i1, j1, k1, G3),
        (i2, j2, k2, 2.0 * G3),
        (1, 1, 1, 3.0 * G3),
    ];
    let (ii, jj, kk) = (i as i32 & 255, j as i32 & 255, k as i32 & 255);
    let mut n = 0.0;
    for (oi, oj, ok, g) in offs {
        let (dx, dy, dz) = (x0 - oi as f32 + g, y0 - oj as f32 + g, z0 - ok as f32 + g);
        let r = 0.6 - dx * dx - dy * dy - dz * dz;
        if r > 0.0 {
            let h = t.at(ii + oi + t.at(jj + oj + t.at(kk + ok) as i32) as i32);
            let r2 = r * r;
            n += r2 * r2 * grad3(h, dx, dy, dz);
        }
    }
    32.0 * n
}

/// Which noise an [`Fbm`] sums.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseKind {
    Perlin,
    Simplex,
}

/// Fractal Brownian motion: `octaves` layers of noise, each `lacunarity`
/// times the frequency and `gain` times the amplitude of the one before,
/// normalised by the amplitudes' sum (so about -1 to 1). Each octave reads
/// its own table (the seed plus its index), so octaves do not line up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fbm {
    pub seed: u32,
    pub kind: NoiseKind,
    pub octaves: u32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
}

impl Default for Fbm {
    fn default() -> Self {
        Fbm {
            seed: 0,
            kind: NoiseKind::Perlin,
            octaves: 5,
            frequency: 1.0,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

impl Fbm {
    /// The sum at (x, y, z); `flat` reads 2D noise and ignores z.
    pub fn at(&self, x: f32, y: f32, z: f32, flat: bool) -> f32 {
        let (mut sum, mut amp, mut norm, mut f) = (0.0f32, 1.0f32, 0.0f32, self.frequency);
        for o in 0..self.octaves.clamp(1, 16) {
            let s = self.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
            let n = match (self.kind, flat) {
                (NoiseKind::Perlin, true) => perlin2(s, x * f, y * f),
                (NoiseKind::Perlin, false) => perlin3(s, x * f, y * f, z * f),
                (NoiseKind::Simplex, true) => simplex2(s, x * f, y * f),
                (NoiseKind::Simplex, false) => simplex3(s, x * f, y * f, z * f),
            };
            sum += n * amp;
            norm += amp;
            amp *= self.gain;
            f *= self.lacunarity;
        }
        if norm > 0.0 { sum / norm } else { 0.0 }
    }

    /// A `width` × `height` grid, row-major from (x0, y0), `step` apart,
    /// at height z (2D noise when `flat`).
    #[allow(clippy::too_many_arguments)]
    pub fn grid(
        &self,
        width: u32,
        height: u32,
        x0: f32,
        y0: f32,
        step: f32,
        z: f32,
        flat: bool,
    ) -> Vec<f32> {
        let mut out = Vec::with_capacity((width * height) as usize);
        for j in 0..height {
            for i in 0..width {
                out.push(self.at(x0 + i as f32 * step, y0 + j as f32 * step, z, flat));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_zero_on_the_lattice_continuous_and_in_range() {
        for seed in [0, 1, 99] {
            assert_eq!(perlin2(seed, 3.0, -7.0), 0.0);
            assert_eq!(perlin3(seed, 3.0, -7.0, 2.0), 0.0);
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for i in 0..4000 {
                let (x, y) = (i as f32 * 0.137, i as f32 * 0.071 - 50.0);
                for n in [
                    perlin2(seed, x, y),
                    perlin3(seed, x, y, 0.3),
                    simplex2(seed, x, y),
                    simplex3(seed, x, y, 0.3),
                ] {
                    lo = lo.min(n);
                    hi = hi.max(n);
                }
                // Continuous: a tiny step moves it a tiny amount.
                let d = (simplex2(seed, x, y) - simplex2(seed, x + 1e-3, y)).abs();
                assert!(d < 0.02, "simplex jumps {d} at {x} {y}");
                let d = (perlin2(seed, x, y) - perlin2(seed, x + 1e-3, y)).abs();
                assert!(d < 0.02, "perlin jumps {d} at {x} {y}");
            }
            assert!(lo > -1.05 && hi < 1.05, "{lo} {hi}");
            assert!(lo < -0.5 && hi > 0.5, "it uses its range: {lo} {hi}");
        }
        assert_ne!(perlin2(1, 0.5, 0.5), perlin2(2, 0.5, 0.5));
    }

    #[test]
    fn fbm_sums_octaves_and_a_grid_reads_the_same_points() {
        let f = Fbm {
            seed: 4,
            octaves: 4,
            ..Fbm::default()
        };
        let g = f.grid(8, 4, 1.0, 2.0, 0.25, 0.0, true);
        assert_eq!(g.len(), 32);
        assert_eq!(g[9], f.at(1.25, 2.25, 0.0, true));
        let one = Fbm { octaves: 1, ..f };
        assert_eq!(one.at(0.3, 0.6, 0.0, true), perlin2(4, 0.3, 0.6));
    }
}
