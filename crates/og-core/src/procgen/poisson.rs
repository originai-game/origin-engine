//! Poisson-disk points (Bridson 2007).

use crate::{Error, ErrorKind, Result, Rng};

/// Points in [0, width) × [0, height), none closer than `radius` to
/// another, every gap filled to within twice it (Bridson 2007): `attempts`
/// candidates around each active point, the active list taken from by a
/// draw, at most `max` points. Deterministic for a seed. Refused for an
/// area of more than 16,777,216 cells of radius / √2 (a radius too small
/// for the area).
pub fn poisson_disk(
    seed: u32,
    width: f32,
    height: f32,
    radius: f32,
    attempts: u32,
    max: usize,
) -> Result<Vec<[f32; 2]>> {
    if !(width > 0.0 && height > 0.0 && radius > 0.0 && width.is_finite() && height.is_finite()) {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: width {width}, height {height} and radius {radius} must be finite and above 0"
            ),
        ));
    }
    let cell = radius / std::f32::consts::SQRT_2;
    let (gw, gh) = (
        (width / cell).floor() as usize + 1,
        (height / cell).floor() as usize + 1,
    );
    if gw.saturating_mul(gh) > 1 << 24 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "Poisson disk: radius {radius} over {width} x {height} needs {gw} x {gh} grid cells; at most 16,777,216"
            ),
        ));
    }
    if max == 0 {
        return Ok(Vec::new());
    }
    let mut rng = Rng::new(seed);
    let mut grid: Vec<u32> = vec![u32::MAX; gw * gh];
    let at = |p: [f32; 2]| {
        (
            ((p[0] / cell) as usize).min(gw - 1),
            ((p[1] / cell) as usize).min(gh - 1),
        )
    };
    let mut points: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<u32> = Vec::new();
    let first = [rng.range_f32(0.0, width), rng.range_f32(0.0, height)];
    let (cx, cy) = at(first);
    grid[cy * gw + cx] = 0;
    points.push(first);
    active.push(0);
    let r2 = radius * radius;
    while !active.is_empty() && points.len() < max {
        let k = rng.integer(0, active.len() as i64 - 1) as usize;
        let base = points[active[k] as usize];
        let mut placed = false;
        for _ in 0..attempts.max(1) {
            // A point in the annulus r..2r: a direction from a unit square
            // draw (rejected outside the disc), a distance from another.
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
            let d = radius * (1.0 + rng.range_f32(0.0, 1.0));
            let p = [base[0] + dx * d, base[1] + dy * d];
            if !(p[0] >= 0.0 && p[0] < width && p[1] >= 0.0 && p[1] < height) {
                continue;
            }
            let (px, py) = at(p);
            let mut clear = true;
            'near: for gy in py.saturating_sub(2)..(py + 3).min(gh) {
                for gx in px.saturating_sub(2)..(px + 3).min(gw) {
                    let q = grid[gy * gw + gx];
                    if q != u32::MAX {
                        let o = points[q as usize];
                        let (ex, ey) = (o[0] - p[0], o[1] - p[1]);
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
