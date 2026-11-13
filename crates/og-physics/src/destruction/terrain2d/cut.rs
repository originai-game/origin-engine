//! Solid cells cut loose from a pixel terrain: their islands, their
//! Voronoi pieces and a blast's push on them.

use super::Cell;
use glam::Vec2;
use og_core::hash::splitmix64;

/// Solid cells cut from a grid: their cells row by row from `origin` (in
/// the grid's cells), `width` wide, 0 where empty.
pub struct Sub {
    pub origin: [i32; 2],
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
    pub count: u32,
}

impl Sub {
    /// Its centre in its own cells.
    pub fn centre(&self) -> Vec2 {
        let (mut sx, mut sy) = (0.0f32, 0.0f32);
        for y in 0..self.height {
            for x in 0..self.width {
                if self.cells[(y * self.width + x) as usize] != 0 {
                    sx += x as f32 + 0.5;
                    sy += y as f32 + 0.5;
                }
            }
        }
        let n = self.count.max(1) as f32;
        Vec2::new(sx / n, sy / n)
    }
}

/// The 4-connected islands of a set of cells, each a [`Sub`], in the order
/// of their lowest cell (bottom row first).
pub fn components(cells: &[(i32, i32, Cell)]) -> Vec<Sub> {
    if cells.is_empty() {
        return Vec::new();
    }
    let (mut lx, mut ly, mut hx, mut hy) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for &(x, y, _) in cells {
        (lx, ly, hx, hy) = (lx.min(x), ly.min(y), hx.max(x), hy.max(y));
    }
    let (w, h) = ((hx - lx + 1) as usize, (hy - ly + 1) as usize);
    let mut grid = vec![0 as Cell; w * h];
    for &(x, y, c) in cells {
        grid[(y - ly) as usize * w + (x - lx) as usize] = c;
    }
    let mut seen = vec![false; w * h];
    let mut out = Vec::new();
    for start in 0..w * h {
        if grid[start] == 0 || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut members = vec![start];
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            let next = [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ];
            for j in next.into_iter().flatten() {
                if grid[j] != 0 && !seen[j] {
                    seen[j] = true;
                    members.push(j);
                    stack.push(j);
                }
            }
        }
        let (mut ax, mut ay, mut bx, mut by) = (w, h, 0, 0);
        for &i in &members {
            let (x, y) = (i % w, i / w);
            (ax, ay, bx, by) = (ax.min(x), ay.min(y), bx.max(x), by.max(y));
        }
        let (sw, sh) = (bx - ax + 1, by - ay + 1);
        let mut sub = vec![0; sw * sh];
        for &i in &members {
            sub[(i / w - ay) * sw + (i % w - ax)] = grid[i];
        }
        out.push(Sub {
            origin: [lx + ax as i32, ly + ay as i32],
            width: sw as u32,
            height: sh as u32,
            cells: sub,
            count: members.len() as u32,
        });
    }
    out
}

/// Cells cut into the Voronoi regions of `n` seeds picked among them by
/// `seed` (each cell to its nearest, the earlier seed on a tie), each
/// region split into its 4-connected islands.
pub fn voronoi(cells: &[(i32, i32, Cell)], n: u32, seed: u64) -> Vec<Sub> {
    if cells.is_empty() {
        return Vec::new();
    }
    let n = (n as usize).clamp(1, cells.len());
    let mut seeds: Vec<(i64, i64)> = Vec::with_capacity(n);
    for i in 0..n as u64 {
        let c =
            cells[(splitmix64(seed ^ i.wrapping_mul(0x9e37_79b9)) % cells.len() as u64) as usize];
        if !seeds.contains(&(c.0 as i64, c.1 as i64)) {
            seeds.push((c.0 as i64, c.1 as i64));
        }
    }
    let mut regions: Vec<Vec<(i32, i32, Cell)>> = vec![Vec::new(); seeds.len()];
    for &c in cells {
        let (x, y) = (c.0 as i64, c.1 as i64);
        let near = (0..seeds.len())
            .min_by_key(|&s| {
                let (dx, dy) = (x - seeds[s].0, y - seeds[s].1);
                (dx * dx + dy * dy, s)
            })
            .expect("one seed at least");
        regions[near].push(c);
    }
    regions.iter().flat_map(|r| components(r)).collect()
}

/// The outward push of blasts (point, reach, speed) at a point.
pub fn pushed(blasts: &[([f32; 2], f32, f32)], at: Vec2) -> Vec2 {
    let mut v = Vec2::ZERO;
    for &(p, reach, speed) in blasts {
        let d = at - Vec2::from(p);
        let dist = d.length();
        if dist >= reach {
            continue;
        }
        let dir = if dist > 1e-6 { d / dist } else { Vec2::Y };
        v += dir * (speed * (1.0 - dist / reach));
    }
    v
}
