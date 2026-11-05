//! Painted 2D: the look's `Stylise` paint fields read as texel space
//! (docs/art.md §2.3), so the stroke follows the sprite, not the screen.
//! The same steps as the frame's paint pass (stylise.wgsl `paint`), one
//! algorithm on both sides so the frame and the cook agree:
//!
//! 1. the structure tensor: Sobel derivatives of the linear colour, their
//!    outer products summed over the channels, smoothed by a Gaussian
//!    (σ [`TENSOR_SIGMA`], radius [`TENSOR_RADIUS`]); its eigenvectors give
//!    the local stroke direction and its anisotropy how coherent the
//!    structure is (Kyprianidis, Kang and Döllner, "Image and video
//!    abstraction by anisotropic Kuwahara filtering", 2009);
//! 2. the anisotropic Kuwahara filter with eight sectors and polynomial
//!    weights (Kyprianidis 2010), its ellipse stretched along the stroke
//!    and squeezed across it by the anisotropy: the port of Garrett
//!    Gunnell's (Acerola) `AnisotropicKuwahara.shader`, MIT, the notice in
//!    `third_party/acerola-postprocessing`. Coherent structure (an edge,
//!    a thin line, an eye's contour) gets a thin ellipse along itself and
//!    keeps its shape; incoherent texture (grass, noise) a round one and is
//!    painted over;
//! 3. pigment pooling at the edges of regions as a pigment density (MNPR,
//!    Montesdeoca et al. 2018: the colour raised to 1 + density), the
//!    density from the smoothed tensor's edge strength;
//! 4. colour bleeding from a ring two radii out, and the canvas's grain;
//! 5. the watercolour wash over the painted texels ([`Wash`], the frame's
//!    `watercolor` pass): a pigment density about 1 (Bousseau et al. 2006,
//!    C' = C(1 - (1 - C)(d - 1)) relative to the paper) from the paper's
//!    height about its mean (granulation) and a slow value noise
//!    (turbulence), MNPR's edge darkening in a rim where the painted
//!    regions meet, the dry brush on the paper's peaks, and the texels
//!    read displaced by the paper's relief and a slow waver (wobble).
//!
//! Radii are in the texture's texels, the canvas `paintCanvasScale`
//! canvas texels per texel from each cell's corner. A transparent texel
//! stays transparent and lends no colour: where the frame reads a
//! neighbour off the shape, this reads the texel itself.

use super::colour;

/// The structure tensor's Gaussian (texels), as stylise.wgsl's.
pub const TENSOR_SIGMA: f64 = 2.0;
/// The tensor's window radius (texels), as stylise.wgsl's.
pub const TENSOR_RADIUS: i64 = 3;
/// The sector weights' sharpness (Acerola's `_Q`).
pub const SHARPNESS: f64 = 8.0;
/// How hard the sectors' variances choose (Acerola's `_Hardness`).
pub const HARDNESS: f64 = 8.0;
/// Where the polynomial sector weights cross zero (Acerola's default).
pub const ZERO_CROSSING: f64 = 0.58;

/// The smallest α (`paintAnisotropy` 1): a coherent line's ellipse is a
/// fifth of the radius across, thin enough to keep a one-pixel line; below
/// it weakly oriented noise (anisotropy about 0.3) is drawn as needles
/// too and keeps its spread instead of being painted over.
pub const ALPHA_FLOOR: f64 = 0.25;

/// The ellipse's tuning α from `paintAnisotropy` (0 to 1): none is a
/// circle (`None`), 0.5 Kyprianidis' α = 1, 1 the thinnest ellipse along
/// coherent structure ([`ALPHA_FLOOR`]).
pub fn alpha(anisotropy: f64) -> Option<f64> {
    (anisotropy > 0.0).then(|| ((1.0 - anisotropy.min(1.0)) / anisotropy).max(ALPHA_FLOOR))
}

/// The smoothed structure tensor (E, F, G: Sx·Sx, Sx·Sy, Sy·Sy) at a texel,
/// from `src`, which gives the linear colour at any texel.
pub fn tensor(x: i64, y: i64, src: &dyn Fn(i64, i64) -> [f64; 3]) -> [f64; 3] {
    let n = TENSOR_RADIUS + 1;
    let side = (2 * n + 1) as usize;
    let mut block = vec![[0.0f64; 3]; side * side];
    for j in -n..=n {
        for i in -n..=n {
            block[((j + n) as usize) * side + (i + n) as usize] = src(x + i, y + j);
        }
    }
    let at = |i: i64, j: i64| block[((j + n) as usize) * side + (i + n) as usize];
    let mut sum = [0.0f64; 3];
    let mut weight = 0.0;
    for j in -TENSOR_RADIUS..=TENSOR_RADIUS {
        for i in -TENSOR_RADIUS..=TENSOR_RADIUS {
            let g = (-((i * i + j * j) as f64) / (2.0 * TENSOR_SIGMA * TENSOR_SIGMA)).exp();
            let (mut e, mut f, mut gg) = (0.0, 0.0, 0.0);
            for k in 0..3 {
                let c = |a: i64, b: i64| at(i + a, j + b)[k];
                let sx = ((c(1, -1) + 2.0 * c(1, 0) + c(1, 1))
                    - (c(-1, -1) + 2.0 * c(-1, 0) + c(-1, 1)))
                    / 4.0;
                let sy = ((c(-1, 1) + 2.0 * c(0, 1) + c(1, 1))
                    - (c(-1, -1) + 2.0 * c(0, -1) + c(1, -1)))
                    / 4.0;
                e += sx * sx;
                f += sx * sy;
                gg += sy * sy;
            }
            sum[0] += e * g;
            sum[1] += f * g;
            sum[2] += gg * g;
            weight += g;
        }
    }
    sum.map(|v| v / weight)
}

/// The tensor's stroke frame: the direction along the structure (unit),
/// its anisotropy (0 isotropic to 1 a single direction) and the edge
/// strength (√λ1, the larger eigenvalue's root).
pub fn orientation(t: [f64; 3]) -> ([f64; 2], f64, f64) {
    let [e, f, g] = t;
    let root = ((e - g) * (e - g) + 4.0 * f * f).sqrt();
    let l1 = 0.5 * (e + g + root);
    let l2 = 0.5 * (e + g - root);
    let v = [l1 - e, -f];
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let dir = if len > 1e-12 {
        [v[0] / len, v[1] / len]
    } else {
        [0.0, 1.0]
    };
    let a = if l1 + l2 > 1e-12 {
        (l1 - l2) / (l1 + l2)
    } else {
        0.0
    };
    (dir, a, l1.max(0.0).sqrt())
}

/// The anisotropic Kuwahara filter of radius `radius` at a texel (Acerola's
/// fourth pass): `alpha` from [`alpha`] (`None`: round), `frame` from
/// [`orientation`], `tap` the linear colour of a texel that takes part
/// (`None`: left out: transparent, or across an object's edge).
pub fn kuwahara(
    radius: f64,
    alpha: Option<f64>,
    frame: ([f64; 2], f64),
    tap: &dyn Fn(i64, i64) -> Option<[f64; 3]>,
) -> Option<[f64; 3]> {
    let (dir, aniso) = frame;
    let (a, b) = match alpha {
        Some(al) => (
            radius * ((al + aniso) / al).clamp(0.1, 2.0),
            radius * (al / (al + aniso)).clamp(0.1, 2.0),
        ),
        None => (radius, radius),
    };
    // phi = -atan2(t.y, t.x): the rotation taking the stroke onto x.
    let (cos_phi, sin_phi) = (dir[0], -dir[1]);
    let max_x = (a * a * cos_phi * cos_phi + b * b * sin_phi * sin_phi).sqrt() as i64;
    let max_y = (a * a * sin_phi * sin_phi + b * b * cos_phi * cos_phi).sqrt() as i64;
    let zeta = 1.0 / radius.max(1.0);
    let sin_zc = ZERO_CROSSING.sin();
    let eta = (zeta + ZERO_CROSSING.cos()) / (sin_zc * sin_zc);
    let mut m = [[0.0f64; 4]; 8];
    let mut s = [[0.0f64; 3]; 8];
    for y in -max_y..=max_y {
        for x in -max_x..=max_x {
            let (fx, fy) = (x as f64, y as f64);
            // SR (x, y): rotated by phi, scaled to the unit ellipse / 2.
            let mut v = [
                (cos_phi * fx - sin_phi * fy) * 0.5 / a,
                (sin_phi * fx + cos_phi * fy) * 0.5 / b,
            ];
            if v[0] * v[0] + v[1] * v[1] > 0.25 {
                continue;
            }
            let Some(c) = tap(x, y) else { continue };
            let c = c.map(|v| v.clamp(0.0, 1.0));
            let mut w = [0.0f64; 8];
            let mut sum = 0.0;
            for half in 0..2 {
                let vxx = zeta - eta * v[0] * v[0];
                let vyy = zeta - eta * v[1] * v[1];
                let z = [
                    (v[1] + vxx).max(0.0),
                    (-v[0] + vyy).max(0.0),
                    (-v[1] + vxx).max(0.0),
                    (v[0] + vyy).max(0.0),
                ];
                for (q, z) in z.iter().enumerate() {
                    w[q * 2 + half] = z * z;
                    sum += z * z;
                }
                let r = std::f64::consts::FRAC_1_SQRT_2;
                v = [r * (v[0] - v[1]), r * (v[0] + v[1])];
            }
            let g = (-3.125 * (v[0] * v[0] + v[1] * v[1])).exp() / sum.max(1e-12);
            for k in 0..8 {
                let wk = w[k] * g;
                for i in 0..3 {
                    m[k][i] += c[i] * wk;
                    s[k][i] += c[i] * c[i] * wk;
                }
                m[k][3] += wk;
            }
        }
    }
    let mut out = [0.0f64; 4];
    for k in 0..8 {
        if m[k][3] <= 0.0 {
            continue;
        }
        let mean = [m[k][0] / m[k][3], m[k][1] / m[k][3], m[k][2] / m[k][3]];
        let sigma2: f64 = (0..3)
            .map(|i| (s[k][i] / m[k][3] - mean[i] * mean[i]).abs())
            .sum();
        let w = 1.0 / (1.0 + (HARDNESS * 1000.0 * sigma2).powf(0.5 * SHARPNESS));
        for i in 0..3 {
            out[i] += mean[i] * w;
        }
        out[3] += w;
    }
    (out[3] > 0.0).then(|| [out[0], out[1], out[2]].map(|v| (v / out[3]).clamp(0.0, 1.0)))
}

/// Pigment pooling at an edge of strength `edge` (√λ1): the colour raised
/// to one plus the density, so a region's rim dries darker and keeps its
/// hue (MNPR's pigment density), 0 leaving it.
pub fn pool(c: [f64; 3], darkening: f64, edge: f64) -> [f64; 3] {
    let density = darkening * (edge * 2.0).clamp(0.0, 1.0);
    c.map(|v| v.max(0.0).powf(1.0 + density))
}

/// Canvas texels per cell of the turbulence's noise, as stylise.wgsl's
/// `WASH_TURBULENCE_CELL`.
pub const WASH_TURBULENCE_CELL: f64 = 40.0;
/// Canvas texels per cell of the waver's noise (`WASH_WOBBLE_CELL`).
pub const WASH_WOBBLE_CELL: f64 = 14.0;
/// Density per unit of the paper's height about its mean (`WASH_GRAIN_GAIN`).
pub const WASH_GRAIN_GAIN: f64 = 2.5;
/// Waver per unit of the paper's height gradient (`WASH_RELIEF_GAIN`).
pub const WASH_RELIEF_GAIN: f64 = 4.0;
/// The height above the mean that is a peak the dry brush skips (`WASH_PEAK`).
pub const WASH_PEAK: f64 = 0.08;

/// The watercolour wash's fields (`Stylise.watercolor*`), at their frame
/// meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct Wash {
    pub edge: f64,
    pub granulation: f64,
    pub turbulence: f64,
    pub dry_brush: f64,
    pub wobble: f64,
    /// The paper's colour, linear.
    pub paper: [f64; 3],
}

impl Default for Wash {
    fn default() -> Self {
        Wash {
            edge: 0.0,
            granulation: 0.0,
            turbulence: 0.0,
            dry_brush: 0.0,
            wobble: 0.0,
            paper: [1.0; 3],
        }
    }
}

impl Wash {
    pub fn active(&self) -> bool {
        self.edge > 0.0
            || self.granulation > 0.0
            || self.turbulence > 0.0
            || self.dry_brush > 0.0
            || self.wobble > 0.0
    }

    /// Whether the wash reads the paper's grain.
    pub fn reads_paper(&self) -> bool {
        self.granulation > 0.0 || self.dry_brush > 0.0 || self.wobble > 0.0
    }
}

/// A hash of an integer cell, 0 to 1 (stylise.wgsl `wash_hash`).
fn wash_hash(c: [i64; 3], seed: u32) -> f64 {
    let [x, y, z] = c.map(|v| v as i32 as u32);
    let mut h = x.wrapping_mul(0x8da6_b343)
        ^ y.wrapping_mul(0xd816_3841)
        ^ z.wrapping_mul(0xcb1a_b31f)
        ^ seed.wrapping_mul(0x9e37_79b9);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    f64::from(h & 0xff_ffff) / 16_777_215.0
}

/// Smooth value noise, 0 to 1 about 0.5, one cell per unit (stylise.wgsl
/// `wash_noise`).
fn wash_noise(p: [f64; 3], seed: u32) -> f64 {
    let i = p.map(|v| v.floor() as i64);
    let u = p.map(|v| {
        let f = v - v.floor();
        f * f * (3.0 - 2.0 * f)
    });
    (0..8)
        .map(|k| {
            let o = [k & 1, (k >> 1) & 1, k >> 2];
            let w: f64 = (0..3)
                .map(|a| if o[a] == 1 { u[a] } else { 1.0 - u[a] })
                .product();
            wash_hash([i[0] + o[0], i[1] + o[1], i[2] + o[2]], seed) * w
        })
        .sum()
}

/// The paint fields, at their frame meaning.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Paint {
    pub radius: f64,
    pub anisotropy: f64,
    pub edge: f64,
    pub bleed: f64,
    /// The canvas's grain (its red channel, linear) and size.
    pub canvas: Option<(Vec<f64>, u32, u32)>,
    pub canvas_strength: f64,
    pub canvas_scale: f64,
    pub wash: Wash,
}

impl Paint {
    pub fn active(&self) -> bool {
        self.radius > 0.0
            || self.bleed > 0.0
            || self.edge > 0.0
            || (self.canvas.is_some() && self.canvas_strength > 0.0)
            || self.wash.active()
    }

    pub fn describe(&self) -> String {
        let mut v = Vec::new();
        if self.radius > 0.0 {
            v.push(format!(
                "anisotropic Kuwahara radius {} texels",
                self.radius
            ));
            if self.anisotropy > 0.0 {
                v.push(format!("anisotropy {}", self.anisotropy));
            }
        }
        if self.bleed > 0.0 {
            v.push(format!("bleed {}", self.bleed));
        }
        if self.edge > 0.0 {
            v.push(format!("pigment pooling {}", self.edge));
        }
        if self.canvas.is_some() && self.canvas_strength > 0.0 {
            v.push(format!("canvas grain {}", self.canvas_strength));
        }
        let w = &self.wash;
        for (name, value) in [
            ("wash edge darkening", w.edge),
            ("granulation", w.granulation),
            ("turbulence", w.turbulence),
            ("dry brush", w.dry_brush),
            ("wobble", w.wobble),
        ] {
            if value > 0.0 {
                v.push(format!("{name} {value}"));
            }
        }
        v.join(", ")
    }

    /// The watercolour wash over the painted texels `lin` (linear), the
    /// frame's `watercolor` pass in texel space: `opaque` says which texels
    /// belong to the shape, and a read off it takes the texel asked from.
    fn wash(
        &self,
        lin: &[[f64; 3]],
        w: u32,
        h: u32,
        opaque: &dyn Fn(i64, i64) -> bool,
    ) -> Vec<[f64; 3]> {
        let ws = &self.wash;
        let paper = ws.paper.map(|v| v.max(1e-3));
        let (mean, height) = match &self.canvas {
            Some((grain, gw, gh)) => {
                let mean = grain.iter().sum::<f64>() / grain.len().max(1) as f64;
                let scale = self.canvas_scale;
                let at = move |x: f64, y: f64| sample_repeat(grain, *gw, *gh, x * scale, y * scale);
                (mean, Some(at))
            }
            None => (0.5, None),
        };
        let height = |x: f64, y: f64| height.as_ref().map_or(0.5, |at| at(x, y));
        let texel = |x: i64, y: i64| lin[(y as u32 * w + x as u32) as usize];
        let load = |x: i64, y: i64, from: (i64, i64)| -> [f64; 3] {
            let (x, y) = if opaque(x, y) { (x, y) } else { from };
            texel(x, y)
        };
        // A 3 x 3 Sobel's gradient of the painted texels at (x, y).
        let sobel = |x: i64, y: i64, from: (i64, i64)| -> f64 {
            let t = |dx: i64, dy: i64| load(x + dx, y + dy, from);
            let mut s = 0.0;
            for k in 0..3 {
                let sx = (t(1, -1)[k] + 2.0 * t(1, 0)[k] + t(1, 1)[k])
                    - (t(-1, -1)[k] + 2.0 * t(-1, 0)[k] + t(-1, 1)[k]);
                let sy = (t(-1, 1)[k] + 2.0 * t(0, 1)[k] + t(1, 1)[k])
                    - (t(-1, -1)[k] + 2.0 * t(0, -1)[k] + t(1, -1)[k]);
                s += sx * sx + sy * sy;
            }
            s.sqrt() * 0.25
        };
        let smoothstep = |a: f64, b: f64, x: f64| {
            let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        };
        let mut out = lin.to_vec();
        for y in 0..i64::from(h) {
            for x in 0..i64::from(w) {
                if !opaque(x, y) {
                    continue;
                }
                let at = (x, y);
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                let q = [px * self.canvas_scale, py * self.canvas_scale, 0.0];
                let hgt = height(px, py);
                // Wobble: the texels read where the relief and the waver
                // displace them (bilinear, within the shape).
                let mut off = [0.0, 0.0];
                if ws.wobble > 0.0 {
                    let cell = q.map(|v| v / WASH_WOBBLE_CELL);
                    let wave = [
                        wash_noise(cell, 7) * 2.0 - 1.0,
                        wash_noise(cell, 11) * 2.0 - 1.0,
                    ];
                    let relief = [
                        (height(px + 1.0, py) - height(px - 1.0, py)) * 0.5,
                        (height(px, py + 1.0) - height(px, py - 1.0)) * 0.5,
                    ];
                    for a in 0..2 {
                        off[a] = ws.wobble
                            * (wave[a] * 2.0 + relief[a] * WASH_RELIEF_GAIN).clamp(-1.0, 1.0);
                    }
                }
                let (sx, sy) = (px + off[0] - 0.5, py + off[1] - 0.5);
                let (x0, y0) = (sx.floor(), sy.floor());
                let (tx, ty) = (sx - x0, sy - y0);
                let (x0, y0) = (x0 as i64, y0 as i64);
                let mut c = [0.0; 3];
                for (dx, dy, wgt) in [
                    (0, 0, (1.0 - tx) * (1.0 - ty)),
                    (1, 0, tx * (1.0 - ty)),
                    (0, 1, (1.0 - tx) * ty),
                    (1, 1, tx * ty),
                ] {
                    let t = load(x0 + dx, y0 + dy, at);
                    for k in 0..3 {
                        c[k] += t[k] * wgt;
                    }
                }
                // Edge darkening: the rim where the painted regions meet,
                // the gradient blurred over the texel's 3 x 3 (a binomial
                // weight) so the rim is a soft band, not a line.
                let mut rim = 0.0;
                if ws.edge > 0.0 {
                    let o = ((px + off[0]).floor() as i64, (py + off[1]).floor() as i64);
                    let mut g = 0.0;
                    for dy in -1..=1_i64 {
                        for dx in -1..=1_i64 {
                            let wgt = ((2 - dx.abs()) * (2 - dy.abs())) as f64;
                            g += sobel(o.0 + dx, o.1 + dy, at) * wgt;
                        }
                    }
                    rim = smoothstep(0.02, 0.22, g / 16.0);
                }
                // The pigment density: granulation and turbulence.
                let mut d = 1.0;
                if ws.granulation > 0.0 {
                    d += ws.granulation * ((mean - hgt) * WASH_GRAIN_GAIN).clamp(-1.0, 1.0);
                }
                if ws.turbulence > 0.0 {
                    let n = wash_noise(q.map(|v| v / WASH_TURBULENCE_CELL), 3) * 0.6
                        + wash_noise(q.map(|v| v / (WASH_TURBULENCE_CELL * 0.5)), 5) * 0.4;
                    d += ws.turbulence * ((n - 0.5) * 3.0).clamp(-1.0, 1.0);
                }
                let mut t2 = [0.0; 3];
                for k in 0..3 {
                    let t = (c[k] / paper[k]).clamp(0.0, 1.0);
                    t2[k] = (t * (1.0 - (1.0 - t) * (d - 1.0))).clamp(0.0, 1.0);
                }
                if ws.dry_brush > 0.0 {
                    let peak = smoothstep(0.0, WASH_PEAK, hgt - mean);
                    let thin = 0.2126 * t2[0] + 0.7152 * t2[1] + 0.0722 * t2[2];
                    let k = ws.dry_brush * peak * thin * thin;
                    t2 = t2.map(|v| v + (1.0 - v) * k);
                }
                for k in 0..3 {
                    c[k] = t2[k] * paper[k];
                    if rim > 0.0 {
                        c[k] = c[k].max(0.0).powf(1.0 + ws.edge * rim);
                    }
                }
                out[(y as u32 * w + x as u32) as usize] = c;
            }
        }
        out
    }

    /// Paints one `w` x `h` cell of RGBA texels in place.
    pub fn apply(&self, px: &mut [[u8; 4]], w: u32, h: u32) {
        let lin: Vec<[f64; 3]> = px
            .iter()
            .map(|p| {
                [
                    colour::to_linear(p[0]),
                    colour::to_linear(p[1]),
                    colour::to_linear(p[2]),
                ]
            })
            .collect();
        let opaque = |x: i64, y: i64| -> bool {
            x >= 0
                && y >= 0
                && x < i64::from(w)
                && y < i64::from(h)
                && px[(y as u32 * w + x as u32) as usize][3] > 0
        };
        let texel = |x: i64, y: i64| lin[(y as u32 * w + x as u32) as usize];
        // `load`: the texel, or the one asked from where the shape is not.
        let load = |x: i64, y: i64, from: (i64, i64)| -> [f64; 3] {
            let (x, y) = if opaque(x, y) { (x, y) } else { from };
            texel(x, y)
        };
        let alpha = alpha(self.anisotropy);
        let mut out = lin.clone();
        for y in 0..i64::from(h) {
            for x in 0..i64::from(w) {
                if !opaque(x, y) {
                    continue;
                }
                let at = (x, y);
                let src = |a: i64, b: i64| load(a, b, at);
                let mut c = texel(x, y);
                let needs_tensor = self.radius > 0.0 || self.edge > 0.0;
                let (dir, aniso, edge) = if needs_tensor {
                    orientation(tensor(x, y, &src))
                } else {
                    ([0.0, 1.0], 0.0, 0.0)
                };
                if self.radius > 0.0 {
                    let tap =
                        |dx: i64, dy: i64| opaque(x + dx, y + dy).then(|| texel(x + dx, y + dy));
                    if let Some(k) = kuwahara(self.radius, alpha, (dir, aniso), &tap) {
                        c = k;
                    }
                }
                if self.bleed > 0.0 {
                    let d = self.radius.max(1.0) as i64 * 2;
                    let ring = [src(x + d, y), src(x - d, y), src(x, y + d), src(x, y - d)];
                    for k in 0..3 {
                        let around = ring.iter().map(|r| r[k]).sum::<f64>() * 0.25;
                        c[k] += (around - c[k]) * self.bleed * 0.5;
                    }
                }
                if self.edge > 0.0 {
                    c = pool(c, self.edge, edge);
                }
                if let Some((grain, gw, gh)) = &self.canvas
                    && self.canvas_strength > 0.0
                {
                    let g = sample_repeat(
                        grain,
                        *gw,
                        *gh,
                        (x as f64 + 0.5) * self.canvas_scale,
                        (y as f64 + 0.5) * self.canvas_scale,
                    );
                    let k = 1.0 + (g * 0.5 + 0.5 - 1.0) * self.canvas_strength;
                    c = c.map(|v| v * k);
                }
                out[(y as u32 * w + x as u32) as usize] = c;
            }
        }
        if self.wash.active() {
            out = self.wash(&out, w, h, &opaque);
        }
        for (p, c) in px.iter_mut().zip(out) {
            if p[3] > 0 {
                p[0] = colour::to_srgb8(c[0]);
                p[1] = colour::to_srgb8(c[1]);
                p[2] = colour::to_srgb8(c[2]);
            }
        }
    }
}

/// A bilinear, repeating sample of a grain texture at texel coordinates.
fn sample_repeat(g: &[f64], w: u32, h: u32, x: f64, y: f64) -> f64 {
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let at = |x: f64, y: f64| {
        let xi = (x as i64).rem_euclid(i64::from(w)) as u32;
        let yi = (y as i64).rem_euclid(i64::from(h)) as u32;
        g[(yi * w + xi) as usize]
    };
    let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
    let bottom = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
    top * (1.0 - ty) + bottom * ty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuwahara_keeps_an_edge_and_flattens_noise_beside_it() {
        // 6 x 1: three dark texels with one speck, then three light.
        let k = [40u8, 40, 40, 255];
        let mut px = vec![
            k,
            [60, 60, 60, 255],
            k,
            [220, 220, 220, 255],
            [220, 220, 220, 255],
            [220, 220, 220, 255],
        ];
        let p = Paint {
            radius: 2.0,
            ..Paint::default()
        };
        p.apply(&mut px, 6, 1);
        // The speck joins its side; the edge between 2 and 3 stays.
        assert!(px[1][0] < 60 && px[1][0] >= 40, "{:?}", px[1]);
        assert!(px[2][0] < 100, "{:?}", px[2]);
        assert!(px[3][0] > 200, "{:?}", px[3]);
    }

    /// A one-texel diagonal line through a flat field, painted at radius 4
    /// along its structure: the anisotropic ellipse lies along the line and
    /// keeps it (the four-sector box Kuwahara turned by an unsmoothed Sobel
    /// takes the flat sectors either side and erases it). Ways this could
    /// fail: an unsmoothed tensor (the line's own texels have no gradient,
    /// so no direction), the ellipse turned across the line instead of
    /// along it (phi's sign), or the minor axis not squeezed.
    #[test]
    fn a_thin_diagonal_line_survives_the_anisotropic_filter() {
        let (w, h) = (24u32, 24u32);
        let field = [200u8, 200, 200, 255];
        let ink = [30u8, 30, 30, 255];
        let mut px = vec![field; (w * h) as usize];
        for i in 0..w {
            px[(i * w + i) as usize] = ink;
        }
        let p = Paint {
            radius: 4.0,
            anisotropy: 1.0,
            ..Paint::default()
        };
        p.apply(&mut px, w, h);
        // Along the line's middle (away from the corners) the contrast to
        // the field stays at least 80 % of the original's.
        let lin = |v: u8| colour::to_linear(v);
        let full = lin(200) - lin(30);
        for i in 6..18u32 {
            let on = lin(px[(i * w + i) as usize][0]);
            let off = lin(px[(i * w + i + 4) as usize][0]);
            assert!(
                off - on >= 0.8 * full,
                "texel {i}: on {on:.3} off {off:.3} (full {full:.3})"
            );
        }
    }

    /// Incoherent texture (noise, grass) is painted over: its spread falls
    /// by more than half, where a thin line's structure keeps its shape.
    /// Broken by too thin an ellipse (α 0.05): weakly oriented noise was
    /// drawn as needles and kept two thirds of its spread.
    #[test]
    fn noise_is_painted_into_flat_strokes() {
        let (w, h) = (24u32, 24u32);
        let mut seed = 7u32;
        let mut px: Vec<[u8; 4]> = (0..w * h)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let v = 90 + (seed >> 24) as u8 / 4;
                [v, v, v, 255]
            })
            .collect();
        let spread = |px: &[[u8; 4]]| {
            let inner: Vec<f64> = (6..18)
                .flat_map(|y| (6..18).map(move |x| (x, y)))
                .map(|(x, y)| f64::from(px[(y * w + x) as usize][0]))
                .collect();
            let mean = inner.iter().sum::<f64>() / inner.len() as f64;
            (inner.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / inner.len() as f64).sqrt()
        };
        let before = spread(&px);
        Paint {
            radius: 4.0,
            anisotropy: 1.0,
            ..Paint::default()
        }
        .apply(&mut px, w, h);
        let after = spread(&px);
        assert!(after < before * 0.5, "spread {before:.2} -> {after:.2}");
    }

    #[test]
    fn pooling_darkens_the_edges_of_a_region_not_its_middle_or_the_clear() {
        let mut px = vec![[200, 200, 200, 255]; 13 * 13];
        px[84] = [20, 20, 20, 255];
        px[0] = [0, 0, 0, 0];
        let p = Paint {
            edge: 1.0,
            ..Paint::default()
        };
        p.apply(&mut px, 13, 13);
        // Next to the dark texel the light ones darken; far from it not;
        // the clear corner stays clear and lends no edge.
        assert!(px[83][0] < 200, "{:?}", px[83]);
        assert_eq!(px[12], [200, 200, 200, 255]);
        assert_eq!(px[0], [0, 0, 0, 0]);
        assert_eq!(px[1], [200, 200, 200, 255]);
    }

    /// A grain with pits and peaks about its mean: 4 x 4 texels, 0.3 to 0.7.
    fn grain() -> (Vec<f64>, u32, u32) {
        let g = (0..16i32)
            .map(|i| 0.5 + 0.2 * f64::from(i * 7 % 5 - 2) / 2.0)
            .collect();
        (g, 4, 4)
    }

    fn mean_grey(px: &[[u8; 4]]) -> f64 {
        px.iter().map(|p| colour::to_linear(p[0])).sum::<f64>() / px.len() as f64
    }

    /// The wash's granulation keeps a mid-grey field's mean (the density
    /// thins as much as it thickens) where the canvas multiply only
    /// darkens it, and a field with no wash set is left as it was.
    #[test]
    fn granulation_keeps_the_mean_the_canvas_multiply_lowers() {
        let (w, h) = (32u32, 32u32);
        let field = vec![[140u8, 140, 140, 255]; (w * h) as usize];
        let before = mean_grey(&field);
        let mut untouched = field.clone();
        Paint {
            canvas: Some(grain()),
            canvas_scale: 1.0,
            ..Paint::default()
        }
        .apply(&mut untouched, w, h);
        assert_eq!(untouched, field);
        let mut multiplied = field.clone();
        Paint {
            canvas: Some(grain()),
            canvas_strength: 0.6,
            canvas_scale: 1.0,
            ..Paint::default()
        }
        .apply(&mut multiplied, w, h);
        let mut washed = field.clone();
        Paint {
            canvas: Some(grain()),
            canvas_scale: 1.0,
            wash: Wash {
                granulation: 0.6,
                ..Wash::default()
            },
            ..Paint::default()
        }
        .apply(&mut washed, w, h);
        let (m, g) = (mean_grey(&multiplied), mean_grey(&washed));
        assert!(m < before * 0.92, "multiply {m:.3} before {before:.3}");
        assert!(
            (g - before).abs() < before * 0.03,
            "wash {g:.3} before {before:.3}"
        );
        // And it is a texture, not a flat shift: the grain's pits and
        // peaks differ.
        let spread =
            washed.iter().map(|p| p[0]).max().unwrap() - washed.iter().map(|p| p[0]).min().unwrap();
        assert!(spread >= 8, "spread {spread}");
    }

    /// The wash's rim darkens the two texels either side of a boundary
    /// between painted regions and nothing farther away, where the paint
    /// pass's pooling spreads over the source's structure.
    #[test]
    fn the_wash_rim_hugs_a_region_boundary() {
        let (w, h) = (16u32, 9u32);
        let mut px = vec![[200u8, 200, 200, 255]; (w * h) as usize];
        for y in 0..h {
            for x in 8..w {
                px[(y * w + x) as usize] = [90, 90, 90, 255];
            }
        }
        Paint {
            wash: Wash {
                edge: 1.0,
                ..Wash::default()
            },
            ..Paint::default()
        }
        .apply(&mut px, w, h);
        let row = |x: u32| px[(4 * w + x) as usize][0];
        assert!(row(7) < 200 && row(8) < 90, "rim {} {}", row(7), row(8));
        assert_eq!(row(3), 200);
        assert_eq!(row(12), 90);
    }
}
