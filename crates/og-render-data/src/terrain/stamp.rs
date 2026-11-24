//! Deterministic projection of a terrain stamp, shared by the cook and
//! runtime. Coordinates and depth are in the terrain's own metres.

use glam::{DVec2, Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub size: Vec2,
    pub columns: u32,
    pub rows: u32,
}

impl Grid {
    pub fn of(d: &super::TerrainData) -> Grid {
        Grid {
            size: d.size,
            columns: d.columns,
            rows: d.rows,
        }
    }

    fn cell(&self) -> Vec2 {
        super::TerrainData::flat(self.size, self.columns.max(2), self.rows.max(2)).cell()
    }
}

/// The lowest triangle over each grid sample (or cell centre). Uncovered
/// points are infinite. Barycentrics use f64, including shared edges.
pub fn underside(grid: Grid, triangles: &[[Vec3; 3]], cells: bool) -> Option<([u32; 4], Vec<f32>)> {
    let (cols, rows) = if cells {
        (grid.columns.max(1) - 1, grid.rows.max(1) - 1)
    } else {
        (grid.columns, grid.rows)
    };
    if cols == 0 || rows == 0 {
        return None;
    }
    let cell = grid.cell().as_dvec2();
    let half = grid.size.as_dvec2() * 0.5;
    let shift = if cells { 0.5 } else { 0.0 };
    // Grid coordinates of a point in the terrain's space, and back.
    let to_grid = |p: DVec2| (p + half) / cell - shift;
    let to_local = |c: f64, r: f64| (DVec2::new(c, r) + shift) * cell - half;
    let mut lowest = vec![f32::INFINITY; (cols * rows) as usize];
    let (mut lo, mut hi) = ([u32::MAX; 2], [0u32; 2]);
    for t in triangles {
        let p = t.map(|v| DVec2::new(v.x as f64, v.z as f64));
        let area = (p[1] - p[0]).perp_dot(p[2] - p[0]);
        if area.is_nan() || area.abs() <= 1e-12 {
            continue;
        }
        let g = p.map(to_grid);
        let min = g[0].min(g[1]).min(g[2]);
        let max = g[0].max(g[1]).max(g[2]);
        let c0 = min.x.ceil().max(0.0);
        let r0 = min.y.ceil().max(0.0);
        let c1 = max.x.floor().min(cols as f64 - 1.0);
        let r1 = max.y.floor().min(rows as f64 - 1.0);
        if c0 > c1 || r0 > r1 {
            continue;
        }
        let eps = -1e-9 * area.abs();
        for r in r0 as u32..=r1 as u32 {
            for c in c0 as u32..=c1 as u32 {
                let q = to_local(c as f64, r as f64);
                let w0 = (p[2] - p[1]).perp_dot(q - p[1]) * area.signum();
                let w1 = (p[0] - p[2]).perp_dot(q - p[2]) * area.signum();
                let w2 = (p[1] - p[0]).perp_dot(q - p[0]) * area.signum();
                if w0 < eps || w1 < eps || w2 < eps {
                    continue;
                }
                let y = (w0 * t[0].y as f64 + w1 * t[1].y as f64 + w2 * t[2].y as f64) / area.abs();
                let k = (r * cols + c) as usize;
                lowest[k] = lowest[k].min(y as f32);
                lo = [lo[0].min(c), lo[1].min(r)];
                hi = [hi[0].max(c), hi[1].max(r)];
            }
        }
    }
    if lo[0] > hi[0] {
        return None;
    }
    let (w, h) = (hi[0] - lo[0] + 1, hi[1] - lo[1] + 1);
    let mut out = Vec::with_capacity((w * h) as usize);
    for r in lo[1]..=hi[1] {
        let row = (r * cols) as usize;
        out.extend_from_slice(&lowest[row + lo[0] as usize..=row + hi[0] as usize]);
    }
    Some(([lo[0], lo[1], w, h], out))
}

/// Carve heights (`NaN`: uncovered), or covered cells as 0/1 for a hole.
/// Only interior samples go below the underside by `depth`; the footprint
/// edge meets the mesh itself, rather than exposing a trench beside it.
pub fn patch(
    grid: Grid,
    triangles: &[[Vec3; 3]],
    hole: bool,
    depth: f32,
) -> Option<([u32; 4], Vec<f32>)> {
    let (rect, lowest) = underside(grid, triangles, hole)?;
    let [_, _, w, h] = rect;
    let covered = |c: i64, r: i64| {
        c >= 0
            && r >= 0
            && c < w as i64
            && r < h as i64
            && lowest[(r * w as i64 + c) as usize].is_finite()
    };
    let values = lowest
        .iter()
        .enumerate()
        .map(|(k, y)| {
            let (c, r) = ((k as u32 % w) as i64, (k as u32 / w) as i64);
            let inner =
                covered(c - 1, r) && covered(c + 1, r) && covered(c, r - 1) && covered(c, r + 1);
            match (y.is_finite(), hole) {
                (false, true) => 0.0,
                (true, true) => 1.0,
                (true, false) if inner => y - depth,
                (true, false) => *y,
                (false, false) => f32::NAN,
            }
        })
        .collect();
    Some((rect, values))
}
