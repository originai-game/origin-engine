//! Procedural noise on a seed: Perlin, simplex, Worley (cellular), analytic
//! derivatives, curl noise, periodic variants and fractal sums ([`Fbm`]).
//! The one owner of CPU procedural noise.
//!
//! Determinism: every lattice corner is hashed statelessly with
//! [`crate::hash::pcg`] (matching GPU `hash.wgsl` operation for operation) —
//! no 256-entry permutation table, no thread-local cache thrashing, no hidden
//! periodicity. Arithmetic uses only IEEE-754 `+ - * /`, `floor` and `sqrt` on
//! `f32`, so the same seed gives bit-identical fields on every host.

use crate::hash::{pcg, unit24};
use std::f32::consts::SQRT_2;

/// 16 evenly spaced unit-length 2D gradients on the circle (`cos(k·π/8), sin(k·π/8)`).
/// Unit length eliminates the 41% diagonal anisotropy of unnormalized 8-direction tables.
const GRAD2: [[f32; 2]; 16] = [
    [1.0, 0.0],
    [0.923_879_5, 0.382_683_43],
    [0.707_106_77, 0.707_106_77],
    [0.382_683_43, 0.923_879_5],
    [0.0, 1.0],
    [-0.382_683_43, 0.923_879_5],
    [-0.707_106_77, 0.707_106_77],
    [-0.923_879_5, 0.382_683_43],
    [-1.0, 0.0],
    [-0.923_879_5, -0.382_683_43],
    [-0.707_106_77, -0.707_106_77],
    [-0.382_683_43, -0.923_879_5],
    [0.0, -1.0],
    [0.382_683_43, -0.923_879_5],
    [0.707_106_77, -0.707_106_77],
    [0.923_879_5, -0.382_683_43],
];

/// Ken Perlin's 12 cube-edge 3D gradients (each of length √2), indexed
/// uniformly without the 4-slot tetrahedral bias of a 16-entry table.
const GRAD3: [[f32; 3]; 12] = [
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

#[inline]
fn hash2i(x: i32, y: i32, seed: u32) -> u32 {
    let h = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ seed.wrapping_mul(0x9e37_79b9);
    pcg(h)
}

#[inline]
fn hash3i(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let h = (x as u32).wrapping_mul(73_856_093)
        ^ (y as u32).wrapping_mul(19_349_663)
        ^ (z as u32).wrapping_mul(83_492_791)
        ^ seed.wrapping_mul(0x9e37_79b9);
    pcg(h)
}

#[inline]
fn wrap_coord(i: i32, period: u32) -> i32 {
    if period == 0 {
        i
    } else {
        i.rem_euclid(period as i32)
    }
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn dfade(t: f32) -> f32 {
    30.0 * t * t * (t * (t - 2.0) + 1.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn grad2_vec(h: u32) -> [f32; 2] {
    GRAD2[(h >> 28) as usize]
}

#[inline]
fn grad3_vec(h: u32) -> [f32; 3] {
    GRAD3[(h % 12) as usize]
}

#[inline]
fn dot2(g: [f32; 2], x: f32, y: f32) -> f32 {
    g[0] * x + g[1] * y
}

#[inline]
fn dot3(g: [f32; 3], x: f32, y: f32, z: f32) -> f32 {
    g[0] * x + g[1] * y + g[2] * z
}

/// Improved 2D Perlin noise at `(x, y)` in `[-1, 1]`, `0` on every lattice point.
pub fn perlin2(seed: u32, x: f32, y: f32) -> f32 {
    perlin2_periodic(seed, x, y, 0, 0)
}

/// Seamless tileable 2D Perlin noise with integer lattice periods `(px, py)`.
/// A period of `0` disables wrapping on that axis.
pub fn perlin2_periodic(seed: u32, x: f32, y: f32, px: u32, py: u32) -> f32 {
    let (fx, fy) = (x.floor(), y.floor());
    let (xi, yi) = (fx as i32, fy as i32);
    let (dx, dy) = (x - fx, y - fy);
    let (u, v) = (fade(dx), fade(dy));

    let x0 = wrap_coord(xi, px);
    let x1 = wrap_coord(xi.wrapping_add(1), px);
    let y0 = wrap_coord(yi, py);
    let y1 = wrap_coord(yi.wrapping_add(1), py);

    let g00 = grad2_vec(hash2i(x0, y0, seed));
    let g10 = grad2_vec(hash2i(x1, y0, seed));
    let g01 = grad2_vec(hash2i(x0, y1, seed));
    let g11 = grad2_vec(hash2i(x1, y1, seed));

    let n00 = dot2(g00, dx, dy);
    let n10 = dot2(g10, dx - 1.0, dy);
    let n01 = dot2(g01, dx, dy - 1.0);
    let n11 = dot2(g11, dx - 1.0, dy - 1.0);

    // Unit 2D gradients have theoretical peak 1 / sqrt(2) at cell centres.
    lerp(lerp(n00, n10, u), lerp(n01, n11, u), v) * SQRT_2
}

/// 2D Perlin noise with analytic partial derivatives: `[value, dv/dx, dv/dy]`.
pub fn perlin2_deriv(seed: u32, x: f32, y: f32) -> [f32; 3] {
    perlin2_deriv_periodic(seed, x, y, 0, 0)
}

/// Seamless tileable 2D Perlin noise with analytic derivatives `[value, dv/dx, dv/dy]`.
pub fn perlin2_deriv_periodic(seed: u32, x: f32, y: f32, px: u32, py: u32) -> [f32; 3] {
    let (fx, fy) = (x.floor(), y.floor());
    let (xi, yi) = (fx as i32, fy as i32);
    let (dx, dy) = (x - fx, y - fy);
    let (u, v) = (fade(dx), fade(dy));
    let (du, dv) = (dfade(dx), dfade(dy));

    let x0 = wrap_coord(xi, px);
    let x1 = wrap_coord(xi.wrapping_add(1), px);
    let y0 = wrap_coord(yi, py);
    let y1 = wrap_coord(yi.wrapping_add(1), py);

    let g00 = grad2_vec(hash2i(x0, y0, seed));
    let g10 = grad2_vec(hash2i(x1, y0, seed));
    let g01 = grad2_vec(hash2i(x0, y1, seed));
    let g11 = grad2_vec(hash2i(x1, y1, seed));

    let n00 = dot2(g00, dx, dy);
    let n10 = dot2(g10, dx - 1.0, dy);
    let n01 = dot2(g01, dx, dy - 1.0);
    let n11 = dot2(g11, dx - 1.0, dy - 1.0);

    let k0 = n00;
    let k1 = n10 - n00;
    let k2 = n01 - n00;
    let k3 = n00 - n10 - n01 + n11;

    let val = k0 + k1 * u + k2 * v + k3 * u * v;
    let gx0 = lerp(g00[0], g10[0], u);
    let gx1 = lerp(g01[0], g11[0], u);
    let gy0 = lerp(g00[1], g10[1], u);
    let gy1 = lerp(g01[1], g11[1], u);
    let dval_dx = lerp(gx0, gx1, v) + du * (k1 + k3 * v);
    let dval_dy = lerp(gy0, gy1, v) + dv * (k2 + k3 * u);

    [val * SQRT_2, dval_dx * SQRT_2, dval_dy * SQRT_2]
}

/// Improved 3D Perlin noise at `(x, y, z)` in `[-1, 1]`.
pub fn perlin3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    perlin3_periodic(seed, x, y, z, 0, 0, 0)
}

/// Seamless tileable 3D Perlin noise with integer periods `(px, py, pz)`.
/// A period of `0` disables wrapping on that axis.
pub fn perlin3_periodic(seed: u32, x: f32, y: f32, z: f32, px: u32, py: u32, pz: u32) -> f32 {
    let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (fx as i32, fy as i32, fz as i32);
    let (dx, dy, dz) = (x - fx, y - fy, z - fz);
    let (u, v, w) = (fade(dx), fade(dy), fade(dz));

    let x0 = wrap_coord(xi, px);
    let x1 = wrap_coord(xi.wrapping_add(1), px);
    let y0 = wrap_coord(yi, py);
    let y1 = wrap_coord(yi.wrapping_add(1), py);
    let z0 = wrap_coord(zi, pz);
    let z1 = wrap_coord(zi.wrapping_add(1), pz);

    let n000 = dot3(grad3_vec(hash3i(x0, y0, z0, seed)), dx, dy, dz);
    let n100 = dot3(grad3_vec(hash3i(x1, y0, z0, seed)), dx - 1.0, dy, dz);
    let n010 = dot3(grad3_vec(hash3i(x0, y1, z0, seed)), dx, dy - 1.0, dz);
    let n110 = dot3(grad3_vec(hash3i(x1, y1, z0, seed)), dx - 1.0, dy - 1.0, dz);
    let n001 = dot3(grad3_vec(hash3i(x0, y0, z1, seed)), dx, dy, dz - 1.0);
    let n101 = dot3(grad3_vec(hash3i(x1, y0, z1, seed)), dx - 1.0, dy, dz - 1.0);
    let n011 = dot3(grad3_vec(hash3i(x0, y1, z1, seed)), dx, dy - 1.0, dz - 1.0);
    let n111 = dot3(
        grad3_vec(hash3i(x1, y1, z1, seed)),
        dx - 1.0,
        dy - 1.0,
        dz - 1.0,
    );

    lerp(
        lerp(lerp(n000, n100, u), lerp(n010, n110, u), v),
        lerp(lerp(n001, n101, u), lerp(n011, n111, u), v),
        w,
    )
}

/// 3D Perlin noise with analytic partial derivatives: `[value, dv/dx, dv/dy, dv/dz]`.
pub fn perlin3_deriv(seed: u32, x: f32, y: f32, z: f32) -> [f32; 4] {
    perlin3_deriv_periodic(seed, x, y, z, 0, 0, 0)
}

/// Seamless tileable 3D Perlin noise with analytic derivatives `[value, dv/dx, dv/dy, dv/dz]`.
#[allow(clippy::too_many_arguments)]
pub fn perlin3_deriv_periodic(
    seed: u32,
    x: f32,
    y: f32,
    z: f32,
    px: u32,
    py: u32,
    pz: u32,
) -> [f32; 4] {
    let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (fx as i32, fy as i32, fz as i32);
    let (dx, dy, dz) = (x - fx, y - fy, z - fz);
    let (u, v, w) = (fade(dx), fade(dy), fade(dz));
    let (du, dv, dw) = (dfade(dx), dfade(dy), dfade(dz));

    let x0 = wrap_coord(xi, px);
    let x1 = wrap_coord(xi.wrapping_add(1), px);
    let y0 = wrap_coord(yi, py);
    let y1 = wrap_coord(yi.wrapping_add(1), py);
    let z0 = wrap_coord(zi, pz);
    let z1 = wrap_coord(zi.wrapping_add(1), pz);

    let g000 = grad3_vec(hash3i(x0, y0, z0, seed));
    let g100 = grad3_vec(hash3i(x1, y0, z0, seed));
    let g010 = grad3_vec(hash3i(x0, y1, z0, seed));
    let g110 = grad3_vec(hash3i(x1, y1, z0, seed));
    let g001 = grad3_vec(hash3i(x0, y0, z1, seed));
    let g101 = grad3_vec(hash3i(x1, y0, z1, seed));
    let g011 = grad3_vec(hash3i(x0, y1, z1, seed));
    let g111 = grad3_vec(hash3i(x1, y1, z1, seed));

    let a = dot3(g000, dx, dy, dz);
    let b = dot3(g100, dx - 1.0, dy, dz);
    let c = dot3(g010, dx, dy - 1.0, dz);
    let d = dot3(g110, dx - 1.0, dy - 1.0, dz);
    let e = dot3(g001, dx, dy, dz - 1.0);
    let f = dot3(g101, dx - 1.0, dy, dz - 1.0);
    let g = dot3(g011, dx, dy - 1.0, dz - 1.0);
    let h = dot3(g111, dx - 1.0, dy - 1.0, dz - 1.0);

    let k0 = a;
    let k1 = b - a;
    let k2 = c - a;
    let k3 = e - a;
    let k4 = a - b - c + d;
    let k5 = a - c - e + g;
    let k6 = a - b - e + f;
    let k7 = -a + b + c - d + e - f - g + h;

    let val = k0 + k1 * u + k2 * v + k3 * w + k4 * u * v + k5 * v * w + k6 * w * u + k7 * u * v * w;
    let interp_g = |axis: usize| {
        lerp(
            lerp(
                lerp(g000[axis], g100[axis], u),
                lerp(g010[axis], g110[axis], u),
                v,
            ),
            lerp(
                lerp(g001[axis], g101[axis], u),
                lerp(g011[axis], g111[axis], u),
                v,
            ),
            w,
        )
    };
    let dval_dx = interp_g(0) + du * (k1 + k4 * v + k6 * w + k7 * v * w);
    let dval_dy = interp_g(1) + dv * (k2 + k5 * w + k4 * u + k7 * w * u);
    let dval_dz = interp_g(2) + dw * (k3 + k6 * u + k5 * v + k7 * u * v);

    [val, dval_dx, dval_dy, dval_dz]
}

/// 2D Simplex noise at `(x, y)` (Gustavson's construction with isotropic unit
/// gradients), normalised to `[-1, 1]`.
pub fn simplex2(seed: u32, x: f32, y: f32) -> f32 {
    const F2: f32 = 0.366_025_42;
    const G2: f32 = 0.211_324_87;
    let s = (x + y) * F2;
    let (i, j) = ((x + s).floor(), (y + s).floor());
    let tt = (i + j) * G2;
    let (x0, y0) = (x - (i - tt), y - (j - tt));
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let (x1, y1) = (x0 - i1 as f32 + G2, y0 - j1 as f32 + G2);
    let (x2, y2) = (x0 - 1.0 + 2.0 * G2, y0 - 1.0 + 2.0 * G2);
    let (ii, jj) = (i as i32, j as i32);
    let corner = |dx: f32, dy: f32, cx: i32, cy: i32| {
        let r = 0.5 - dx * dx - dy * dy;
        if r <= 0.0 {
            0.0
        } else {
            let r2 = r * r;
            r2 * r2 * dot2(grad2_vec(hash2i(cx, cy, seed)), dx, dy)
        }
    };
    let n0 = corner(x0, y0, ii, jj);
    let n1 = corner(x1, y1, ii.wrapping_add(i1), jj.wrapping_add(j1));
    let n2 = corner(x2, y2, ii.wrapping_add(1), jj.wrapping_add(1));
    // Unit gradients with r = 0.5 reach peak ~0.0101; 99.0 normalises to [-1, 1].
    (99.0 * (n0 + n1 + n2)).clamp(-1.0, 1.0)
}

/// 3D Simplex noise at `(x, y, z)` with exact `r = 0.5` support radius
/// (eliminating the C0 face discontinuity of `r = 0.6`), normalised to `[-1, 1]`.
pub fn simplex3(seed: u32, x: f32, y: f32, z: f32) -> f32 {
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;
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
    let (ii, jj, kk) = (i as i32, j as i32, k as i32);
    let mut n = 0.0;
    for (oi, oj, ok, g) in offs {
        let (dx, dy, dz) = (x0 - oi as f32 + g, y0 - oj as f32 + g, z0 - ok as f32 + g);
        // Exact simplex height squared is 0.5, guaranteeing C2 continuity across faces.
        let r = 0.5 - dx * dx - dy * dy - dz * dz;
        if r > 0.0 {
            let h = hash3i(
                ii.wrapping_add(oi),
                jj.wrapping_add(oj),
                kk.wrapping_add(ok),
                seed,
            );
            let r2 = r * r;
            n += r2 * r2 * dot3(grad3_vec(h), dx, dy, dz);
        }
    }
    // With r = 0.5 and length-sqrt(2) edge gradients, peak is ~0.0131; 76.0 normalises to [-1, 1].
    (76.0 * n).clamp(-1.0, 1.0)
}

/// Cellular (Worley) noise sample: distances to the nearest (`f1`) and
/// second-nearest (`f2`) feature points, and the nearest cell's hash id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorleySample {
    pub f1: f32,
    pub f2: f32,
    pub cell: u32,
}

impl WorleySample {
    /// Voronoi cell border distance (`f2 - f1`): `0` along cell edges.
    #[inline]
    pub fn edge(self) -> f32 {
        self.f2 - self.f1
    }
}

/// 2D Worley (cellular) noise at `(x, y)`.
pub fn worley2(seed: u32, x: f32, y: f32) -> WorleySample {
    worley2_periodic(seed, x, y, 0, 0)
}

/// Seamless tileable 2D Worley noise with integer periods `(px, py)`.
pub fn worley2_periodic(seed: u32, x: f32, y: f32, px: u32, py: u32) -> WorleySample {
    let (fx, fy) = (x.floor(), y.floor());
    let (xi, yi) = (fx as i32, fy as i32);
    let (lx, ly) = (x - fx, y - fy);
    let mut d1 = f32::MAX;
    let mut d2 = f32::MAX;
    let mut best_cell = 0u32;
    for oy in -1..=1 {
        let cy = wrap_coord(yi.wrapping_add(oy), py);
        for ox in -1..=1 {
            let cx = wrap_coord(xi.wrapping_add(ox), px);
            let h0 = hash2i(cx, cy, seed);
            let h1 = pcg(h0 ^ 0x6c8e_9cf5);
            let px_off = ox as f32 + 0.075 + 0.85 * unit24(h0) - lx;
            let py_off = oy as f32 + 0.075 + 0.85 * unit24(h1) - ly;
            let dist2 = px_off * px_off + py_off * py_off;
            if dist2 < d1 {
                d2 = d1;
                d1 = dist2;
                best_cell = h0;
            } else if dist2 < d2 {
                d2 = dist2;
            }
        }
    }
    WorleySample {
        f1: d1.sqrt(),
        f2: d2.sqrt(),
        cell: best_cell,
    }
}

/// 3D Worley (cellular) noise at `(x, y, z)`.
pub fn worley3(seed: u32, x: f32, y: f32, z: f32) -> WorleySample {
    worley3_periodic(seed, x, y, z, 0, 0, 0)
}

/// Seamless tileable 3D Worley noise with integer periods `(px, py, pz)`.
pub fn worley3_periodic(
    seed: u32,
    x: f32,
    y: f32,
    z: f32,
    px: u32,
    py: u32,
    pz: u32,
) -> WorleySample {
    let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
    let (xi, yi, zi) = (fx as i32, fy as i32, fz as i32);
    let (lx, ly, lz) = (x - fx, y - fy, z - fz);
    let mut d1 = f32::MAX;
    let mut d2 = f32::MAX;
    let mut best_cell = 0u32;
    for oz in -1..=1 {
        let cz = wrap_coord(zi.wrapping_add(oz), pz);
        for oy in -1..=1 {
            let cy = wrap_coord(yi.wrapping_add(oy), py);
            for ox in -1..=1 {
                let cx = wrap_coord(xi.wrapping_add(ox), px);
                let h0 = hash3i(cx, cy, cz, seed);
                let h1 = pcg(h0 ^ 0x6c8e_9cf5);
                let h2 = pcg(h1 ^ 0x94d0_49bb);
                let px_off = ox as f32 + 0.075 + 0.85 * unit24(h0) - lx;
                let py_off = oy as f32 + 0.075 + 0.85 * unit24(h1) - ly;
                let pz_off = oz as f32 + 0.075 + 0.85 * unit24(h2) - lz;
                let dist2 = px_off * px_off + py_off * py_off + pz_off * pz_off;
                if dist2 < d1 {
                    d2 = d1;
                    d1 = dist2;
                    best_cell = h0;
                } else if dist2 < d2 {
                    d2 = dist2;
                }
            }
        }
    }
    WorleySample {
        f1: d1.sqrt(),
        f2: d2.sqrt(),
        cell: best_cell,
    }
}

/// Divergence-free 2D curl noise from analytic Perlin derivatives: `[dN/dy, -dN/dx]`.
pub fn curl2(seed: u32, x: f32, y: f32) -> [f32; 2] {
    let d = perlin2_deriv(seed, x, y);
    [d[2], -d[1]]
}

/// Divergence-free 3D curl noise (`∇ × ψ`) from three decorrelated analytic
/// 3D Perlin potential fields.
pub fn curl3(seed: u32, x: f32, y: f32, z: f32) -> [f32; 3] {
    let a = perlin3_deriv(seed, x, y, z);
    let b = perlin3_deriv(seed.wrapping_add(0x9e37_79b9), x, y, z);
    let c = perlin3_deriv(seed.wrapping_add(0x3c6e_f372), x, y, z);
    [c[2] - b[3], a[3] - c[1], b[1] - a[2]]
}

/// Which base noise an [`Fbm`] sums.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseKind {
    Perlin,
    Simplex,
    /// Inverted cellular distance (`1 - 2·f1`, mapped into about `-1..1`).
    Worley,
    /// Voronoi cell border (`2·(f2 - f1) - 1`, mapped into about `-1..1`).
    WorleyEdge,
    /// Ridged multifractal (`1 - 2·|simplex|`, sharp mountain crests).
    Ridged,
    /// Billow (`2·|simplex| - 1`, puffy cumulus lobes).
    Billow,
}

/// Fractal Brownian motion: `octaves` layers of noise, each `lacunarity`
/// times the frequency and `gain` times the amplitude of the one before,
/// normalised by the amplitudes' sum (so about `-1` to `1`). Each octave
/// hashes with its own derived seed, so octaves never align.
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
    #[inline]
    fn sample_one(&self, seed: u32, x: f32, y: f32, z: f32, flat: bool) -> f32 {
        match (self.kind, flat) {
            (NoiseKind::Perlin, true) => perlin2(seed, x, y),
            (NoiseKind::Perlin, false) => perlin3(seed, x, y, z),
            (NoiseKind::Simplex, true) => simplex2(seed, x, y),
            (NoiseKind::Simplex, false) => simplex3(seed, x, y, z),
            (NoiseKind::Worley, true) => (1.0 - 2.0 * worley2(seed, x, y).f1).clamp(-1.0, 1.0),
            (NoiseKind::Worley, false) => (1.0 - 2.0 * worley3(seed, x, y, z).f1).clamp(-1.0, 1.0),
            (NoiseKind::WorleyEdge, true) => {
                (2.0 * worley2(seed, x, y).edge() - 1.0).clamp(-1.0, 1.0)
            }
            (NoiseKind::WorleyEdge, false) => {
                (2.0 * worley3(seed, x, y, z).edge() - 1.0).clamp(-1.0, 1.0)
            }
            (NoiseKind::Ridged, true) => 1.0 - 2.0 * simplex2(seed, x, y).abs(),
            (NoiseKind::Ridged, false) => 1.0 - 2.0 * simplex3(seed, x, y, z).abs(),
            (NoiseKind::Billow, true) => 2.0 * simplex2(seed, x, y).abs() - 1.0,
            (NoiseKind::Billow, false) => 2.0 * simplex3(seed, x, y, z).abs() - 1.0,
        }
    }

    /// The fractal sum at `(x, y, z)`; `flat` reads 2D noise and ignores `z`.
    pub fn at(&self, x: f32, y: f32, z: f32, flat: bool) -> f32 {
        let (mut sum, mut amp, mut norm, mut f) = (0.0f32, 1.0f32, 0.0f32, self.frequency);
        for o in 0..self.octaves.clamp(1, 16) {
            let s = self.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
            let n = self.sample_one(s, x * f, y * f, z * f, flat);
            sum += n * amp;
            norm += amp;
            amp *= self.gain;
            f *= self.lacunarity;
        }
        if norm > 0.0 { sum / norm } else { 0.0 }
    }

    /// Domain-warped fractal sum: offsets the input coordinate by a secondary
    /// noise field scaled by `warp_strength` and `warp_scale` before evaluating.
    #[allow(clippy::too_many_arguments)]
    pub fn at_warped(
        &self,
        x: f32,
        y: f32,
        z: f32,
        flat: bool,
        warp_strength: f32,
        warp_scale: f32,
    ) -> f32 {
        if warp_strength == 0.0 {
            return self.at(x, y, z, flat);
        }
        let wf = self.frequency * warp_scale;
        let s1 = self.seed.wrapping_add(0x517c_c1b7);
        let s2 = self.seed.wrapping_add(0x27d4_eb2d);
        let s3 = self.seed.wrapping_add(0x1656_67b1);
        let wx = if flat {
            simplex2(s1, x * wf, y * wf)
        } else {
            simplex3(s1, x * wf, y * wf, z * wf)
        };
        let wy = if flat {
            simplex2(s2, x * wf, y * wf)
        } else {
            simplex3(s2, x * wf, y * wf, z * wf)
        };
        let wz = if flat {
            0.0
        } else {
            simplex3(s3, x * wf, y * wf, z * wf)
        };
        self.at(
            x + wx * warp_strength,
            y + wy * warp_strength,
            z + wz * warp_strength,
            flat,
        )
    }

    /// Fractal analytic gradient in 2D (`[value, dv/dx, dv/dy]`), using
    /// Perlin analytic derivatives across octaves.
    pub fn deriv2(&self, x: f32, y: f32) -> [f32; 3] {
        let (mut v, mut gx, mut gy) = (0.0f32, 0.0f32, 0.0f32);
        let (mut amp, mut norm, mut f) = (1.0f32, 0.0f32, self.frequency);
        for o in 0..self.octaves.clamp(1, 16) {
            let s = self.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
            let d = perlin2_deriv(s, x * f, y * f);
            v += d[0] * amp;
            gx += d[1] * amp * f;
            gy += d[2] * amp * f;
            norm += amp;
            amp *= self.gain;
            f *= self.lacunarity;
        }
        if norm > 0.0 {
            [v / norm, gx / norm, gy / norm]
        } else {
            [0.0; 3]
        }
    }

    /// Fractal analytic gradient in 3D (`[value, dv/dx, dv/dy, dv/dz]`).
    pub fn deriv3(&self, x: f32, y: f32, z: f32) -> [f32; 4] {
        let (mut v, mut gx, mut gy, mut gz) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let (mut amp, mut norm, mut f) = (1.0f32, 0.0f32, self.frequency);
        for o in 0..self.octaves.clamp(1, 16) {
            let s = self.seed.wrapping_add(o.wrapping_mul(0x9e37_79b9));
            let d = perlin3_deriv(s, x * f, y * f, z * f);
            v += d[0] * amp;
            gx += d[1] * amp * f;
            gy += d[2] * amp * f;
            gz += d[3] * amp * f;
            norm += amp;
            amp *= self.gain;
            f *= self.lacunarity;
        }
        if norm > 0.0 {
            [v / norm, gx / norm, gy / norm, gz / norm]
        } else {
            [0.0; 4]
        }
    }

    /// Fractal divergence-free 2D curl vector at `(x, y)`.
    pub fn curl2(&self, x: f32, y: f32) -> [f32; 2] {
        let d = self.deriv2(x, y);
        [d[2], -d[1]]
    }

    /// Fractal divergence-free 3D curl vector at `(x, y, z)`.
    pub fn curl3(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        let a = self.deriv3(x, y, z);
        let b = Fbm {
            seed: self.seed.wrapping_add(0x9e37_79b9),
            ..*self
        }
        .deriv3(x, y, z);
        let c = Fbm {
            seed: self.seed.wrapping_add(0x3c6e_f372),
            ..*self
        }
        .deriv3(x, y, z);
        [c[2] - b[3], a[3] - c[1], b[1] - a[2]]
    }

    /// A `width` × `height` grid, row-major from `(x0, y0)`, `step` apart,
    /// at height `z` (2D noise when `flat`).
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
        self.grid_warped(width, height, x0, y0, step, z, flat, 0.0, 1.0)
    }

    /// Domain-warped `width` × `height` grid, row-major from `(x0, y0)`.
    #[allow(clippy::too_many_arguments)]
    pub fn grid_warped(
        &self,
        width: u32,
        height: u32,
        x0: f32,
        y0: f32,
        step: f32,
        z: f32,
        flat: bool,
        warp_strength: f32,
        warp_scale: f32,
    ) -> Vec<f32> {
        let total = (u64::from(width) * u64::from(height)) as usize;
        let mut out = Vec::with_capacity(total);
        for j in 0..height {
            let py = y0 + j as f32 * step;
            for i in 0..width {
                let px = x0 + i as f32 * step;
                out.push(self.at_warped(px, py, z, flat, warp_strength, warp_scale));
            }
        }
        out
    }
}
