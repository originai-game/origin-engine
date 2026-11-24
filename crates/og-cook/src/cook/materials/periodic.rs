//! A tileable image from one that is not: Moisan's periodic plus smooth
//! decomposition (L. Moisan, *Periodic plus Smooth Image Decomposition*,
//! J. Math. Imaging Vis. 39, 2011). Written here from the paper.
//!
//! An image `u` is the sum of a periodic part `p` and a smooth part `s`.
//! `s` solves the periodic discrete Poisson equation `Δs = v`, where `v`
//! is zero inside and holds the jump across each edge the image wraps
//! over; in Fourier space that is one division. `p = u - s` keeps every
//! detail of `u` and meets itself at the edges, with no blur and no
//! mirrored or blended band.
//!
//! An edge that need not wrap (a facade module repeats along a street, not
//! up it) adds no jump, so only the other pair is made to meet.

use og_core::math::double;

/// One channel, row by row.
pub(crate) struct Plane<'a> {
    pub width: usize,
    pub height: usize,
    pub data: &'a mut [f64],
}

/// Removes the smooth part of `p`: afterwards its left edge meets its
/// right (and, with `wrap_y`, its top meets its bottom). Both sides are
/// powers of two.
pub(crate) fn make_periodic(p: Plane<'_>, wrap_y: bool) {
    let (w, h) = (p.width, p.height);
    debug_assert!(w.is_power_of_two() && h.is_power_of_two());
    let u = |x: usize, y: usize| p.data[y * w + x];
    let mut v = vec![C::default(); w * h];
    for y in 0..h {
        let d = u(w - 1, y) - u(0, y);
        v[y * w].re += d;
        v[y * w + w - 1].re -= d;
    }
    if wrap_y {
        for x in 0..w {
            let d = u(x, h - 1) - u(x, 0);
            v[x].re += d;
            v[(h - 1) * w + x].re -= d;
        }
    }
    fft2(&mut v, w, h, false);
    let cx: Vec<f64> = (0..w)
        .map(|q| 2.0 * double::cos(std::f64::consts::TAU * q as f64 / w as f64))
        .collect();
    let cy: Vec<f64> = (0..h)
        .map(|r| 2.0 * double::cos(std::f64::consts::TAU * r as f64 / h as f64))
        .collect();
    for (r, cy) in cy.iter().enumerate() {
        for (q, cx) in cx.iter().enumerate() {
            let i = r * w + q;
            if i == 0 {
                v[0] = C::default();
            } else {
                let k = cx + cy - 4.0;
                v[i] = C {
                    re: v[i].re / k,
                    im: v[i].im / k,
                };
            }
        }
    }
    fft2(&mut v, w, h, true);
    let n = (w * h) as f64;
    for (o, s) in p.data.iter_mut().zip(&v) {
        *o -= s.re / n;
    }
}

#[derive(Clone, Copy, Default)]
struct C {
    re: f64,
    im: f64,
}

/// The 2D DFT in place (unscaled), rows then columns.
fn fft2(a: &mut [C], w: usize, h: usize, inverse: bool) {
    for row in a.chunks_mut(w) {
        fft(row, inverse);
    }
    let mut col = vec![C::default(); h];
    for x in 0..w {
        for y in 0..h {
            col[y] = a[y * w + x];
        }
        fft(&mut col, inverse);
        for y in 0..h {
            a[y * w + x] = col[y];
        }
    }
}

/// An iterative radix-2 FFT over a power-of-two length.
fn fft(a: &mut [C], inverse: bool) {
    let n = a.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            a.swap(i, j);
        }
    }
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let (s, c) = double::sin_cos(sign * std::f64::consts::TAU / len as f64);
        for start in (0..n).step_by(len) {
            let (mut wr, mut wi) = (1.0, 0.0);
            for k in 0..len / 2 {
                let x = a[start + k];
                let y = a[start + k + len / 2];
                let t = C {
                    re: y.re * wr - y.im * wi,
                    im: y.re * wi + y.im * wr,
                };
                a[start + k] = C {
                    re: x.re + t.re,
                    im: x.im + t.im,
                };
                a[start + k + len / 2] = C {
                    re: x.re - t.re,
                    im: x.im - t.im,
                };
                (wr, wi) = (wr * c - wi * s, wr * s + wi * c);
            }
        }
        len <<= 1;
    }
}
