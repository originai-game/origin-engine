//! The voxeliser (feature `bake`, run by og-cook): a level's triangles into
//! the walkable [`Heightfield`] the navmesh is built from.
//!
//! 1. Every triangle is rasterised into the columns it covers as a solid
//!    span (its height range in that column); triangles no steeper than
//!    `max_slope` mark their span's top walkable.
//! 2. Filters: a non-walkable span just above a walkable one (within the
//!    climb) becomes walkable (kerbs, stairs); a walkable top next to a drop
//!    deeper than the climb, or whose neighbours' reachable floors differ by
//!    more than the climb, does not (ledges); one with less headroom than
//!    the agent's height does not.
//! 3. The walkable tops become floors with their headroom.
//! 4. The floors are eroded by the agent radius (a chamfer distance to the
//!    nearest unwalkable neighbour), so paths keep that far from walls.

use crate::mesh::{Heightfield, NavParams, Span};
use og_core::glam::Vec3;

#[derive(Clone, Copy, Debug)]
struct Solid {
    smin: u32,
    smax: u32,
    walkable: bool,
}

/// Voxelises `triangles` (world space, counter-clockwise front faces) inside
/// `bounds` (or their own bounds). Refuses an empty input or a grid of more
/// than 16 million columns, naming the fix.
pub fn bake(
    triangles: &[[Vec3; 3]],
    params: &NavParams,
    bounds: Option<(Vec3, Vec3)>,
) -> Result<Heightfield, String> {
    if triangles.is_empty() {
        return Err(
            "the level has no triangles: give the navmesh a scene with static colliders or models"
                .into(),
        );
    }
    let (mut min, mut max) = bounds.unwrap_or_else(|| {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for t in triangles {
            for v in t {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
        }
        (lo, hi)
    });
    max.y += params.agent_height;
    min.y -= params.cell_height;
    let cs = params.cell_size;
    let ch = params.cell_height;
    if !(cs > 0.0 && ch > 0.0) {
        return Err("cell size and cell height must be positive".into());
    }
    let width = ((max.x - min.x) / cs).ceil().max(1.0) as u32;
    let depth = ((max.z - min.z) / cs).ceil().max(1.0) as u32;
    if (width as u64) * (depth as u64) > 16_000_000 {
        return Err(format!(
            "the level is {width} × {depth} cells at cell size {cs}: raise cellSize or bake a smaller box"
        ));
    }
    let top = ((max.y - min.y) / ch).ceil() as u32 + 1;
    let climb = params.climb_cells();
    let walk_cos = og_core::math::cos(params.max_slope.to_radians());
    let mut cols: Vec<Vec<Solid>> = vec![Vec::new(); (width * depth) as usize];

    // 1. Rasterise.
    for t in triangles {
        let n = (t[1] - t[0]).cross(t[2] - t[0]);
        let len = n.length();
        if len <= 1e-12 {
            continue;
        }
        let walkable = n.y / len >= walk_cos;
        let lo = t[0].min(t[1]).min(t[2]);
        let hi = t[0].max(t[1]).max(t[2]);
        let z0 = (((lo.z - min.z) / cs).floor() as i32).clamp(0, depth as i32 - 1);
        let z1 = (((hi.z - min.z) / cs).floor() as i32).clamp(0, depth as i32 - 1);
        if hi.z < min.z || lo.z > max.z || hi.x < min.x || lo.x > max.x {
            continue;
        }
        let poly = vec![t[0], t[1], t[2]];
        for z in z0..=z1 {
            let cz = min.z + z as f32 * cs;
            let row = clip(&clip(&poly, 2, cz, true), 2, cz + cs, false);
            if row.len() < 3 {
                continue;
            }
            let rlo = row.iter().fold(f32::MAX, |m, v| m.min(v.x));
            let rhi = row.iter().fold(f32::MIN, |m, v| m.max(v.x));
            let x0 = (((rlo - min.x) / cs).floor() as i32).clamp(0, width as i32 - 1);
            let x1 = (((rhi - min.x) / cs).floor() as i32).clamp(0, width as i32 - 1);
            for x in x0..=x1 {
                let cx = min.x + x as f32 * cs;
                let cell = clip(&clip(&row, 0, cx, true), 0, cx + cs, false);
                if cell.len() < 3 {
                    continue;
                }
                let ylo = cell.iter().fold(f32::MAX, |m, v| m.min(v.y)) - min.y;
                let yhi = cell.iter().fold(f32::MIN, |m, v| m.max(v.y)) - min.y;
                if yhi < 0.0 {
                    continue;
                }
                let smin = ((ylo / ch).floor().max(0.0) as u32).min(top);
                let smax = ((yhi / ch).ceil().max(0.0) as u32).clamp(smin + 1, top);
                add_span(
                    &mut cols[(z as u32 * width + x as u32) as usize],
                    Solid {
                        smin,
                        smax,
                        walkable,
                    },
                    climb,
                );
            }
        }
    }

    let at = |x: i32, z: i32| -> Option<usize> {
        (x >= 0 && z >= 0 && x < width as i32 && z < depth as i32)
            .then(|| (z as u32 * width + x as u32) as usize)
    };
    // 2a. Low-hanging obstacles.
    for col in &mut cols {
        let mut prev: Option<Solid> = None;
        for s in col.iter_mut() {
            let was = s.walkable;
            if !s.walkable && prev.is_some_and(|p| p.walkable && s.smax.abs_diff(p.smax) <= climb) {
                s.walkable = true;
            }
            prev = Some(Solid {
                walkable: was,
                ..*s
            });
        }
    }
    // 2b. Ledges and 2c. headroom, against the spans as rasterised.
    let height = params.height_cells() as i64;
    let climb_i = climb as i64;
    const OPEN: i64 = 1 << 40;
    let mut walk: Vec<Vec<bool>> = cols
        .iter()
        .map(|c| c.iter().map(|s| s.walkable).collect())
        .collect();
    for z in 0..depth as i32 {
        for x in 0..width as i32 {
            let i = at(x, z).unwrap();
            for k in 0..cols[i].len() {
                if !walk[i][k] {
                    continue;
                }
                let bot = cols[i][k].smax as i64;
                let top_ = cols[i].get(k + 1).map_or(OPEN, |n| n.smin as i64);
                if top_ - bot < height {
                    walk[i][k] = false;
                    continue;
                }
                let mut minh = OPEN;
                let (mut asmin, mut asmax) = (bot, bot);
                for (dx, dz) in [(-1, 0), (0, 1), (1, 0), (0, -1)] {
                    let Some(j) = at(x + dx, z + dz) else {
                        minh = minh.min(-climb_i - bot);
                        continue;
                    };
                    let nc = &cols[j];
                    let nbot = -climb_i;
                    let ntop = nc.first().map_or(OPEN, |s| s.smin as i64);
                    if top_.min(ntop) - bot.max(nbot) > height {
                        minh = minh.min(nbot - bot);
                    }
                    for (m, ns) in nc.iter().enumerate() {
                        let nbot = ns.smax as i64;
                        let ntop = nc.get(m + 1).map_or(OPEN, |n| n.smin as i64);
                        if top_.min(ntop) - bot.max(nbot) > height {
                            minh = minh.min(nbot - bot);
                            if (nbot - bot).abs() <= climb_i {
                                asmin = asmin.min(nbot);
                                asmax = asmax.max(nbot);
                            }
                        }
                    }
                }
                if minh < -climb_i || asmax - asmin > climb_i {
                    walk[i][k] = false;
                }
            }
        }
    }
    // 3. Floors.
    let mut columns = Vec::with_capacity(cols.len());
    let mut spans = Vec::new();
    for (i, col) in cols.iter().enumerate() {
        let first = spans.len() as u32;
        for (k, s) in col.iter().enumerate() {
            if !walk[i][k] {
                continue;
            }
            let clear = col
                .get(k + 1)
                .map_or(u32::MAX / 4, |n| n.smin.saturating_sub(s.smax));
            spans.push(Span {
                floor: s.smax,
                clear,
            });
        }
        columns.push([first, spans.len() as u32 - first]);
    }
    let field = Heightfield {
        params: *params,
        origin: min,
        width,
        depth,
        columns,
        spans,
    };
    // 4. Erode.
    Ok(erode(field))
}

/// Sutherland–Hodgman against `v[axis] >= at` (`keep_above`) or `<= at`.
fn clip(poly: &[Vec3], axis: usize, at: f32, keep_above: bool) -> Vec<Vec3> {
    let side = |v: &Vec3| {
        if keep_above {
            v[axis] - at
        } else {
            at - v[axis]
        }
    };
    let mut out = Vec::with_capacity(poly.len() + 2);
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let (da, db) = (side(&a), side(&b));
        if da >= 0.0 {
            out.push(a);
        }
        if (da >= 0.0) != (db >= 0.0) {
            let t = da / (da - db);
            out.push(a + (b - a) * t);
        }
    }
    out
}

fn add_span(col: &mut Vec<Solid>, mut s: Solid, merge_climb: u32) {
    let mut i = 0;
    while i < col.len() {
        let c = col[i];
        if c.smin > s.smax {
            break;
        }
        if c.smax < s.smin {
            i += 1;
            continue;
        }
        // Overlap: merge; a top within the climb keeps the walkable flag.
        if c.smax.abs_diff(s.smax) <= merge_climb {
            s.walkable = s.walkable || c.walkable;
        } else if c.smax > s.smax {
            s.walkable = c.walkable;
        }
        s.smin = s.smin.min(c.smin);
        s.smax = s.smax.max(c.smax);
        col.remove(i);
    }
    col.insert(i, s);
}

/// Removes floors closer than the agent radius to an edge of the walkable area.
fn erode(field: Heightfield) -> Heightfield {
    let r = field.params.radius_cells();
    if r == 0 {
        return field;
    }
    let (w, d) = (field.width as i32, field.depth as i32);
    let n = field.spans.len();
    let mut dist = vec![u32::MAX; n];
    let neighbour = |s: usize, x: i32, z: i32, dx: i32, dz: i32| -> Option<usize> {
        field
            .column(x + dx, z + dz)
            .find(|&t| field.connected(field.spans[s], field.spans[t]))
    };
    // Boundary floors: fewer than four connected neighbours.
    for z in 0..d {
        for x in 0..w {
            for s in field.column(x, z) {
                let all = [(-1, 0), (0, 1), (1, 0), (0, -1)]
                    .iter()
                    .all(|&(dx, dz)| neighbour(s, x, z, dx, dz).is_some());
                if !all {
                    dist[s] = 0;
                }
            }
        }
    }
    // Two chamfer passes: orthogonal 2, diagonal 3.
    let relax = |dist: &mut Vec<u32>, s: usize, x: i32, z: i32, dx: i32, dz: i32| {
        if let Some(a) = neighbour(s, x, z, dx, 0)
            && dist[a] != u32::MAX
        {
            dist[s] = dist[s].min(dist[a] + 2);
        }
        if let Some(a) = neighbour(s, x, z, 0, dz)
            && dist[a] != u32::MAX
        {
            dist[s] = dist[s].min(dist[a] + 2);
        }
        if let Some(a) = neighbour(s, x, z, dx, 0) {
            let (ax, az) = (x + dx, z);
            if let Some(b) = field
                .column(ax, az + dz)
                .find(|&t| field.connected(field.spans[a], field.spans[t]))
                && dist[b] != u32::MAX
            {
                dist[s] = dist[s].min(dist[b] + 3);
            }
        }
    };
    for z in 0..d {
        for x in 0..w {
            for s in field.column(x, z) {
                relax(&mut dist, s, x, z, -1, -1);
                relax(&mut dist, s, x, z, 1, -1);
            }
        }
    }
    for z in (0..d).rev() {
        for x in (0..w).rev() {
            for s in field.column(x, z) {
                relax(&mut dist, s, x, z, 1, 1);
                relax(&mut dist, s, x, z, -1, 1);
            }
        }
    }
    let limit = r * 2;
    field.without_spans(|s| dist[s] < limit)
}

impl Heightfield {
    fn without_spans(&self, mut drop: impl FnMut(usize) -> bool) -> Heightfield {
        let mut columns = Vec::with_capacity(self.columns.len());
        let mut spans = Vec::new();
        for z in 0..self.depth {
            for x in 0..self.width {
                let first = spans.len() as u32;
                for s in self.column(x as i32, z as i32) {
                    if !drop(s) {
                        spans.push(self.spans[s]);
                    }
                }
                columns.push([first, spans.len() as u32 - first]);
            }
        }
        Heightfield {
            columns,
            spans,
            ..self.clone()
        }
    }
}

/// The twelve triangles of an axis-aligned box (outward, counter-clockwise).
pub fn box_triangles(center: Vec3, half: Vec3) -> Vec<[Vec3; 3]> {
    let c = |sx: f32, sy: f32, sz: f32| center + Vec3::new(sx * half.x, sy * half.y, sz * half.z);
    let quads = [
        [
            c(-1., 1., -1.),
            c(-1., 1., 1.),
            c(1., 1., 1.),
            c(1., 1., -1.),
        ],
        [
            c(-1., -1., -1.),
            c(1., -1., -1.),
            c(1., -1., 1.),
            c(-1., -1., 1.),
        ],
        [
            c(-1., -1., 1.),
            c(1., -1., 1.),
            c(1., 1., 1.),
            c(-1., 1., 1.),
        ],
        [
            c(1., -1., -1.),
            c(-1., -1., -1.),
            c(-1., 1., -1.),
            c(1., 1., -1.),
        ],
        [
            c(1., -1., 1.),
            c(1., -1., -1.),
            c(1., 1., -1.),
            c(1., 1., 1.),
        ],
        [
            c(-1., -1., -1.),
            c(-1., -1., 1.),
            c(-1., 1., 1.),
            c(-1., 1., -1.),
        ],
    ];
    quads
        .iter()
        .flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]])
        .collect()
}
