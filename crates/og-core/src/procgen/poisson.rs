//! Poisson-disk points (Bridson 2007).

use crate::{Error, ErrorKind, Result, Rng};

/// Points in `[0, width) × [0, height)`, none closer than `radius` to
/// another, every gap filled to within twice it (Bridson 2007): `attempts`
/// candidates around each active point, the active list taken from by a
/// draw, at most `max` points. Deterministic for a seed. Refused for an
/// area of more than 16,777,216 cells of `radius / √2` (a radius too small
/// for the area).
pub fn poisson_disk(
    seed: u32,
    width: f32,
    height: f32,
    radius: f32,
    attempts: u32,
    max: usize,
) -> Result<Vec<[f32; 2]>> {
    poisson_disk_ex(seed, width, height, radius, attempts, max, false)
}

/// Like [`poisson_disk`], with optional toroidal wrapping (`wrap = true`) so
/// the point set tiles seamlessly across the `[0, width) × [0, height)` edges.
pub fn poisson_disk_ex(
    seed: u32,
    width: f32,
    height: f32,
    radius: f32,
    attempts: u32,
    max: usize,
    wrap: bool,
) -> Result<Vec<[f32; 2]>> {
    if !(width > 0.0
        && height > 0.0
        && radius > 0.0
        && width.is_finite()
        && height.is_finite()
        && radius.is_finite())
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: width {width}, height {height} and radius {radius} must be finite and above 0"
            ),
        ));
    }
    const MAX_CELLS: u64 = 1 << 24;
    // Slightly below radius / sqrt(2) so each cell's diagonal is strictly < radius,
    // guaranteeing at most one valid point can ever inhabit a cell.
    let cell_f64 = (f64::from(radius) / std::f64::consts::SQRT_2) * (1.0 - 1e-6);
    if !(cell_f64 > 0.0 && cell_f64.is_finite()) {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("Poisson disk: radius {radius} is too small"),
        ));
    }
    let gw_f = (f64::from(width) / cell_f64).floor() + 1.0;
    let gh_f = (f64::from(height) / cell_f64).floor() + 1.0;
    if !gw_f.is_finite() || !gh_f.is_finite() || gw_f > MAX_CELLS as f64 || gh_f > MAX_CELLS as f64
    {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: radius {radius} over {width} x {height} exceeds 16,777,216 grid cells"
            ),
        ));
    }
    let (gw_u64, gh_u64) = (gw_f as u64, gh_f as u64);
    if gw_u64.saturating_mul(gh_u64) > MAX_CELLS {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: radius {radius} over {width} x {height} needs {gw_u64} x {gh_u64} grid cells; at most 16,777,216"
            ),
        ));
    }
    let (gw, gh) = (gw_u64 as usize, gh_u64 as usize);
    if max == 0 {
        return Ok(Vec::new());
    }
    let mut rng = Rng::new(seed);
    let mut grid: Vec<u32> = vec![u32::MAX; gw * gh];
    let at = |p: [f32; 2]| {
        (
            ((f64::from(p[0]) / cell_f64) as usize).min(gw - 1),
            ((f64::from(p[1]) / cell_f64) as usize).min(gh - 1),
        )
    };
    let mut points: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<u32> = Vec::new();
    let first = [rng.range_f32(0.0, width), rng.range_f32(0.0, height)];
    let (cx, cy) = at(first);
    grid[cy * gw + cx] = 0;
    points.push(first);
    active.push(0);
    let r2 = f64::from(radius) * f64::from(radius);
    let (w64, h64) = (f64::from(width), f64::from(height));
    while !active.is_empty() && points.len() < max {
        let k = rng.integer(0, active.len() as i64 - 1) as usize;
        let base = points[active[k] as usize];
        let mut placed = false;
        for _ in 0..attempts.max(1) {
            // Area-uniform point in the annulus [r, 2r): random unit direction
            // from rejection in the unit disk, radial distance r * sqrt(1 + 3u).
            let (mut dx, mut dy);
            loop {
                dx = rng.range_f32(-1.0, 1.0);
                dy = rng.range_f32(-1.0, 1.0);
                let l = dx * dx + dy * dy;
                if l > 1e-6 && l <= 1.0 {
                    let inv = 1.0 / l.sqrt();
                    dx *= inv;
                    dy *= inv;
                    break;
                }
            }
            let u = rng.range_f32(0.0, 1.0);
            let d = radius * (1.0 + 3.0 * u).sqrt();
            let mut px_f = base[0] + dx * d;
            let mut py_f = base[1] + dy * d;
            if wrap {
                px_f = px_f.rem_euclid(width);
                py_f = py_f.rem_euclid(height);
                if px_f >= width {
                    px_f = 0.0;
                }
                if py_f >= height {
                    py_f = 0.0;
                }
            } else if !(px_f >= 0.0 && px_f < width && py_f >= 0.0 && py_f < height) {
                continue;
            }
            let p = [px_f, py_f];
            let (px, py) = at(p);
            let mut clear = true;
            'near: for oy in -2i64..=2 {
                let gy = if wrap {
                    (py as i64 + oy).rem_euclid(gh as i64) as usize
                } else {
                    let y = py as i64 + oy;
                    if y < 0 || y >= gh as i64 {
                        continue;
                    }
                    y as usize
                };
                for ox in -2i64..=2 {
                    let gx = if wrap {
                        (px as i64 + ox).rem_euclid(gw as i64) as usize
                    } else {
                        let x = px as i64 + ox;
                        if x < 0 || x >= gw as i64 {
                            continue;
                        }
                        x as usize
                    };
                    let q = grid[gy * gw + gx];
                    if q != u32::MAX {
                        let o = points[q as usize];
                        let mut ex = (f64::from(o[0]) - f64::from(p[0])).abs();
                        let mut ey = (f64::from(o[1]) - f64::from(p[1])).abs();
                        if wrap {
                            if ex > 0.5 * w64 {
                                ex = w64 - ex;
                            }
                            if ey > 0.5 * h64 {
                                ey = h64 - ey;
                            }
                        }
                        if ex * ex + ey * ey < r2 {
                            clear = false;
                            break 'near;
                        }
                    }
                }
            }
            if clear {
                grid[py * gw + px] = points.len() as u32;
                active.push(points.len() as u32);
                points.push(p);
                placed = true;
                break;
            }
        }
        if !placed {
            active.swap_remove(k);
        }
    }
    Ok(points)
}
