//! Poisson-disk points (Bridson 2007).

use crate::{Error, ErrorKind, Result, Rng};

/// Authored constraints on the same Bridson sampler. Grids cover the
/// rectangular domain with nearest-cell lookup (no hidden resampling).
#[derive(Default)]
pub struct PoissonInput {
    pub region: Vec<[f32; 2]>,
    pub holes: Vec<Vec<[f32; 2]>>,
    pub points: Vec<[f32; 2]>,
    pub columns: u32,
    pub rows: u32,
    pub density: Vec<f32>,
    pub radii: Vec<f32>,
}

impl PoissonInput {
    fn inside(&self, p: [f32; 2]) -> bool {
        fn polygon(p: [f32; 2], ring: &[[f32; 2]]) -> bool {
            let mut inside = false;
            for i in 0..ring.len() {
                let (a, b) = (ring[i], ring[(i + 1) % ring.len()]);
                if (a[1] > p[1]) != (b[1] > p[1])
                    && f64::from(p[0])
                        < (f64::from(b[0]) - f64::from(a[0])) * (f64::from(p[1]) - f64::from(a[1]))
                            / (f64::from(b[1]) - f64::from(a[1]))
                            + f64::from(a[0])
                {
                    inside = !inside;
                }
            }
            inside
        }
        (self.region.is_empty() || polygon(p, &self.region))
            && !self.holes.iter().any(|ring| polygon(p, ring))
    }

    fn sample(&self, grid: &[f32], p: [f32; 2], size: [f32; 2], default: f32) -> f32 {
        if grid.is_empty() {
            return default;
        }
        let x = ((p[0] / size[0] * self.columns as f32) as u32).min(self.columns - 1);
        let y = ((p[1] / size[1] * self.rows as f32) as u32).min(self.rows - 1);
        grid[(y * self.columns + x) as usize]
    }
}

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
    poisson_disk_in(
        seed,
        [width, height],
        radius,
        attempts,
        max,
        wrap,
        &PoissonInput::default(),
    )
}

/// Bridson points with explicit region, exclusion holes, retained points,
/// density probabilities and spatial minimum separation.
pub fn poisson_disk_in(
    seed: u32,
    size: [f32; 2],
    radius: f32,
    attempts: u32,
    max: usize,
    wrap: bool,
    input: &PoissonInput,
) -> Result<Vec<[f32; 2]>> {
    let [width, height] = size;
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
    let invalid = |reason| Error::new(ErrorKind::Invalid, reason);
    for ring in std::iter::once(&input.region)
        .filter(|r| !r.is_empty())
        .chain(input.holes.iter())
    {
        if ring.len() < 3 || ring.iter().flatten().any(|v| !v.is_finite()) {
            return Err(invalid(
                "Poisson region and holes need at least three finite points",
            ));
        }
    }
    for grid in [&input.density, &input.radii] {
        if !grid.is_empty()
            && (input.columns == 0
                || input.rows == 0
                || u64::from(input.columns) * u64::from(input.rows) != grid.len() as u64)
        {
            return Err(invalid("Poisson grids need columns × rows values"));
        }
    }
    if input
        .density
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        || input.radii.iter().any(|v| !v.is_finite() || *v < radius)
    {
        return Err(invalid(
            "Poisson density must be 0..1 and radii must be finite and at least radius",
        ));
    }
    if input.points.len() > max {
        return Err(invalid("Poisson retained points exceed maxPoints"));
    }
    let accepted = |p: [f32; 2]| {
        p.iter().all(|v| v.is_finite())
            && p[0] >= 0.0
            && p[1] >= 0.0
            && p[0] < width
            && p[1] < height
            && input.inside(p)
            && input.sample(&input.density, p, size, 1.0) > 0.0
    };
    let separation = |p| input.sample(&input.radii, p, size, radius);
    let max_radius = input.radii.iter().copied().fold(radius, f32::max);
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
    let (w64, h64) = (f64::from(width), f64::from(height));
    let distance2 = |p: [f32; 2], q: [f32; 2]| {
        let [mut x, mut y] = [
            (f64::from(p[0]) - f64::from(q[0])).abs(),
            (f64::from(p[1]) - f64::from(q[1])).abs(),
        ];
        if wrap {
            x = x.min(w64 - x);
            y = y.min(h64 - y);
        }
        x * x + y * y
    };
    for &p in &input.points {
        if !accepted(p)
            || points
                .iter()
                .any(|&q| distance2(p, q) < f64::from(separation(p).max(separation(q))).powi(2))
        {
            return Err(invalid(
                "Poisson retained points must lie in the allowed region and satisfy pairwise separation",
            ));
        }
        let (cx, cy) = at(p);
        grid[cy * gw + cx] = points.len() as u32;
        active.push(points.len() as u32);
        points.push(p);
    }
    if points.is_empty() {
        for _ in 0..4096 {
            let p = [rng.range_f32(0.0, width), rng.range_f32(0.0, height)];
            if accepted(p)
                && (input.density.is_empty()
                    || rng.range_f32(0.0, 1.0) < input.sample(&input.density, p, size, 1.0))
            {
                let (cx, cy) = at(p);
                grid[cy * gw + cx] = 0;
                points.push(p);
                active.push(0);
                break;
            }
        }
    }
    let reach = (f64::from(max_radius) / cell_f64)
        .ceil()
        .min(gw.max(gh) as f64) as i64;
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
            let d = separation(base) * (1.0 + 3.0 * u).sqrt();
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
            if !accepted(p)
                || (!input.density.is_empty()
                    && rng.range_f32(0.0, 1.0) >= input.sample(&input.density, p, size, 1.0))
            {
                continue;
            }
            let (px, py) = at(p);
            let mut clear = true;
            'near: for oy in -reach..=reach {
                let gy = if wrap {
                    (py as i64 + oy).rem_euclid(gh as i64) as usize
                } else {
                    let y = py as i64 + oy;
                    if y < 0 || y >= gh as i64 {
                        continue;
                    }
                    y as usize
                };
                for ox in -reach..=reach {
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
                        if distance2(p, o) < f64::from(separation(p).max(separation(o))).powi(2) {
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
