//! The aurora's Kelvin–Helmholtz strip: the curtain's fold pattern, simulated once.

use super::*;

/// The sheet's electron flux (the paper's fluid dye) at four moments of a
/// Kelvin–Helmholtz roll-up, one per channel (r: the smooth sheet, a: the
/// turbulent one), `STRIP_X` along (periodic) by `STRIP_Y` across.
///
/// The paper's 2D incompressible flow (Stam 1999) in its vorticity form:
/// the vorticity and the dye are carried by the flow (semi-Lagrangian,
/// with back-and-forth error compensation, Kim et al. 2005, so the thin
/// sheet is not smeared), and the flow is the curl of the stream function
/// solved exactly each step (a Fourier transform along the periodic strip,
/// a tridiagonal solve across it, walls at its edges), in place of the
/// paper's multigrid pressure projection.
pub(crate) fn kh_strip(seed: u32) -> Vec<[u8; 4]> {
    let (nx, ny) = (STRIP_X, STRIP_Y);
    let mut rng = SplitMix(u64::from(seed) ^ 0x6b68_5f73_7472_6970);
    let yc = ny as f32 * 0.5;
    // The sheet, displaced by a spectrum of small waves along it, and the
    // shear across it (velocity -1 to 1 over 1.5 cells).
    let modes: Vec<(f32, f32, f32)> = (2..28)
        .map(|m| {
            (
                m as f32,
                rng.unit() as f32 * std::f32::consts::TAU,
                0.3 + 0.7 * rng.unit() as f32,
            )
        })
        .collect();
    let shear = 1.5f32;
    let mut vort = vec![0.0f32; nx * ny];
    let mut dye = vec![0.0f32; nx * ny];
    for x in 0..nx {
        let mut eta = 0.0;
        for &(m, phase, a) in &modes {
            eta += a * (std::f32::consts::TAU * m * (x as f32 + 0.5) / nx as f32 + phase).sin()
                / m.sqrt();
        }
        eta *= 0.12;
        for y in 0..ny {
            let dy = (y as f32 + 0.5 - yc - eta) / shear;
            let sech = 1.0 / dy.cosh();
            vort[y * nx + x] = -sech * sech / shear;
            dye[y * nx + x] = (-dy * dy).exp();
        }
    }
    let dt = 0.6f32;
    let snapshots = [0usize, 45, 80, 140];
    let mut out = vec![[0u8; 4]; nx * ny];
    let mut step = 0;
    let mut flow = (vec![0.0f32; nx * ny], vec![0.0f32; nx * ny]);
    for (channel, &until) in snapshots.iter().enumerate() {
        while step < until {
            stream_flow(&vort, nx, ny, &mut flow);
            vort = advect_bfecc(&vort, &flow, nx, ny, dt);
            dye = advect_bfecc(&dye, &flow, nx, ny, dt);
            step += 1;
        }
        let peak = dye.iter().fold(1e-6f32, |m, &d| m.max(d));
        for (o, &d) in out.iter_mut().zip(&dye) {
            o[channel] = ((d / peak).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    out
}

/// The strip's mip levels (width, height, texels), 2×2 box averages down
/// to one texel, the first the strip itself. The footprint samples the
/// level of its texel's size: the sheet is far thinner than a footprint
/// texel away from the viewer (half a kilometre against a few), and read
/// at one point per texel it shows only where it crosses a texel's
/// centre, an even comb of false rays. The averages keep each texel's
/// flux, the sheet's column, at every distance.
pub(super) fn strip_mips(strip: &[[u8; 4]]) -> Vec<(usize, usize, Vec<[u8; 4]>)> {
    let mut levels = vec![(STRIP_X, STRIP_Y, strip.to_vec())];
    while levels.len() < STRIP_MIPS as usize {
        let (w, h, last) = levels.last().expect("the strip itself");
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![[0u8; 4]; nw * nh];
        for y in 0..nh {
            for x in 0..nw {
                let mut sum = [0u32; 4];
                let (ys, xs) = ((h / nh).max(1), w / nw);
                for dy in 0..ys {
                    for dx in 0..xs {
                        let t = last[(y * ys + dy) * w + x * xs + dx];
                        for c in 0..4 {
                            sum[c] += u32::from(t[c]);
                        }
                    }
                }
                let n = (ys * xs) as u32;
                next[y * nw + x] = sum.map(|v| ((v + n / 2) / n) as u8);
            }
        }
        levels.push((nw, nh, next));
    }
    levels
}

/// Bilinear sample of a strip field, periodic along x, clamped across y.
fn strip_sample(f: &[f32], nx: usize, ny: usize, x: f32, y: f32) -> f32 {
    let y = y.clamp(0.0, ny as f32 - 1.0);
    let x = x.rem_euclid(nx as f32);
    let (x0, y0) = (x.floor() as usize % nx, y.floor() as usize);
    let (x1, y1) = ((x0 + 1) % nx, (y0 + 1).min(ny - 1));
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let a = f[y0 * nx + x0] * (1.0 - fx) + f[y0 * nx + x1] * fx;
    let b = f[y1 * nx + x0] * (1.0 - fx) + f[y1 * nx + x1] * fx;
    a * (1.0 - fy) + b * fy
}

/// `f` carried by `flow` for `dt` (back-and-forth error compensation and
/// correction, the result limited to its source's neighbourhood).
fn advect_bfecc(f: &[f32], flow: &(Vec<f32>, Vec<f32>), nx: usize, ny: usize, dt: f32) -> Vec<f32> {
    let (u, v) = flow;
    let carry = |src: &[f32], sign: f32| -> Vec<f32> {
        (0..nx * ny)
            .map(|i| {
                let (x, y) = ((i % nx) as f32, (i / nx) as f32);
                strip_sample(src, nx, ny, x - sign * dt * u[i], y - sign * dt * v[i])
            })
            .collect()
    };
    let forward = carry(f, 1.0);
    let back = carry(&forward, -1.0);
    let corrected: Vec<f32> = f
        .iter()
        .zip(&back)
        .map(|(a, b)| a + 0.5 * (a - b))
        .collect();
    let mut result = carry(&corrected, 1.0);
    // Clamp to the values the plain step blended (no new extremes).
    for (i, r) in result.iter_mut().enumerate() {
        let (x, y) = ((i % nx) as f32 - dt * u[i], (i / nx) as f32 - dt * v[i]);
        let y = y.clamp(0.0, ny as f32 - 1.0);
        let (x0, y0) = (
            x.rem_euclid(nx as f32).floor() as usize % nx,
            y.floor() as usize,
        );
        let (x1, y1) = ((x0 + 1) % nx, (y0 + 1).min(ny - 1));
        let around = [
            f[y0 * nx + x0],
            f[y0 * nx + x1],
            f[y1 * nx + x0],
            f[y1 * nx + x1],
        ];
        let lo = around.iter().fold(f32::MAX, |m, &a| m.min(a));
        let hi = around.iter().fold(f32::MIN, |m, &a| m.max(a));
        *r = r.clamp(lo, hi);
    }
    result
}

/// The flow (u, v) of vorticity `vort`: the stream function ψ solves
/// ∇²ψ = -ω with ψ = 0 on the walls, by a Fourier transform along x and a
/// tridiagonal solve across y per wavenumber; u = ∂ψ/∂y, v = -∂ψ/∂x.
fn stream_flow(vort: &[f32], nx: usize, ny: usize, flow: &mut (Vec<f32>, Vec<f32>)) {
    // Rows to the frequency domain.
    let mut spectrum: Vec<Vec<(f32, f32)>> = (0..ny)
        .map(|y| {
            let mut row: Vec<(f32, f32)> = vort[y * nx..(y + 1) * nx]
                .iter()
                .map(|&w| (-w, 0.0))
                .collect();
            fft(&mut row, false);
            row
        })
        .collect();
    for k in 0..nx {
        // (ψ[y-1] - (2 + λ) ψ[y] + ψ[y+1]) = rhs[y], λ the discrete kx².
        let lambda = 2.0 - 2.0 * (std::f32::consts::TAU * k as f32 / nx as f32).cos();
        let diag = -(2.0 + lambda);
        let mut c = vec![0.0f32; ny];
        let mut d = vec![(0.0f32, 0.0f32); ny];
        for (y, row) in spectrum.iter().enumerate() {
            let rhs = row[k];
            let m = diag - if y > 0 { c[y - 1] } else { 0.0 };
            c[y] = 1.0 / m;
            let prev = if y > 0 { d[y - 1] } else { (0.0, 0.0) };
            d[y] = ((rhs.0 - prev.0) / m, (rhs.1 - prev.1) / m);
        }
        for y in (0..ny - 1).rev() {
            d[y] = (d[y].0 - c[y] * d[y + 1].0, d[y].1 - c[y] * d[y + 1].1);
        }
        for (row, v) in spectrum.iter_mut().zip(&d) {
            row[k] = *v;
        }
    }
    let psi: Vec<f32> = spectrum
        .iter_mut()
        .flat_map(|row| {
            fft(row, true);
            row.iter()
                .map(|&(re, _)| re / nx as f32)
                .collect::<Vec<f32>>()
        })
        .collect();
    let at = |x: usize, y: isize| -> f32 {
        if y < 0 || y >= ny as isize {
            0.0
        } else {
            psi[y as usize * nx + x]
        }
    };
    for y in 0..ny {
        for x in 0..nx {
            let (xl, xr) = ((x + nx - 1) % nx, (x + 1) % nx);
            let yi = y as isize;
            flow.0[y * nx + x] = 0.5 * (at(x, yi + 1) - at(x, yi - 1));
            flow.1[y * nx + x] = -0.5 * (at(xr, yi) - at(xl, yi));
        }
    }
}

/// An in-place radix-2 complex FFT (`inverse`: unscaled inverse).
fn fft(a: &mut [(f32, f32)], inverse: bool) {
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
    let mut len = 2;
    while len <= n {
        let angle = std::f32::consts::TAU / len as f32 * if inverse { 1.0 } else { -1.0 };
        for start in (0..n).step_by(len) {
            for k in 0..len / 2 {
                let (s, c) = (angle * k as f32).sin_cos();
                let (ur, ui) = a[start + k];
                let (vr0, vi0) = a[start + k + len / 2];
                let (vr, vi) = (vr0 * c - vi0 * s, vr0 * s + vi0 * c);
                a[start + k] = (ur + vr, ui + vi);
                a[start + k + len / 2] = (ur - vr, ui - vi);
            }
        }
        len <<= 1;
    }
}
