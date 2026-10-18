//! Flow fields: one Dijkstra pass from a set of target cells gives every
//! cell its cost to the nearest target, and a direction towards it, so any
//! number of agents steer by a lookup instead of a path query each. Built
//! for crowds that share a goal (a horde running at the player, units to a
//! rally point).

use crate::grid::{DIRS, Grid};
use og_core::glam::Vec2;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// No direction: a target cell, or a cell no target can be reached from.
pub const NONE: u8 = 255;

/// Integration costs and directions over a grid.
#[derive(Clone, Debug, PartialEq)]
pub struct FlowField {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    /// Cost to the nearest target in the grid's units; `u64::MAX` unreachable.
    pub cost: Vec<u64>,
    /// Index into [`DIRS`] of the step towards the nearest target, or [`NONE`].
    pub dir: Vec<u8>,
    /// The grid version it was built from.
    pub grid_version: u64,
    /// Cells reached.
    pub reached: u32,
}

impl FlowField {
    /// The field towards `targets` (walkable target cells; blocked ones are skipped).
    pub fn build(grid: &Grid, targets: &[(u32, u32)]) -> FlowField {
        let (w, h) = (grid.width as i32, grid.height as i32);
        let n = (w * h) as usize;
        let mut cost = vec![u64::MAX; n];
        let mut open: BinaryHeap<Reverse<(u64, u32)>> = BinaryHeap::new();
        for &(x, y) in targets {
            if grid.walkable(x as i32, y as i32) {
                let i = (y as i32 * w + x as i32) as usize;
                if cost[i] != 0 {
                    cost[i] = 0;
                    open.push(Reverse((0, i as u32)));
                }
            }
        }
        let mut reached = 0u32;
        while let Some(Reverse((c, node))) = open.pop() {
            let i = node as usize;
            if c > cost[i] {
                continue;
            }
            reached += 1;
            let (x, y) = ((i as i32) % w, (i as i32) / w);
            // Reverse edges: stepping from the neighbour into this cell costs this cell's cost.
            let here = grid.cost(x, y) as u64;
            for (dx, dy) in DIRS {
                let (nx, ny) = (x + dx, y + dy);
                if !grid.walkable(nx, ny) || !grid.can_step(nx, ny, -dx, -dy) {
                    continue;
                }
                let base = if dx != 0 && dy != 0 {
                    crate::grid::DIAG
                } else {
                    crate::grid::ORTHO
                } as u64;
                let nc = c + base * here;
                let ni = (ny * w + nx) as usize;
                if nc < cost[ni] {
                    cost[ni] = nc;
                    open.push(Reverse((nc, ni as u32)));
                }
            }
        }
        let mut dir = vec![NONE; n];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if cost[i] == 0 || cost[i] == u64::MAX {
                    continue;
                }
                let mut best = (cost[i], NONE);
                for (k, (dx, dy)) in DIRS.iter().enumerate() {
                    if !grid.can_step(x, y, *dx, *dy) {
                        continue;
                    }
                    let c = cost[((y + dy) * w + x + dx) as usize];
                    if c < best.0 {
                        best = (c, k as u8);
                    }
                }
                dir[i] = best.1;
            }
        }
        FlowField {
            width: grid.width,
            height: grid.height,
            cell_size: grid.cell_size,
            origin: grid.origin,
            cost,
            dir,
            grid_version: grid.version(),
            reached,
        }
    }

    fn cell_dir(&self, x: i32, y: i32) -> Option<Vec2> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        let i = (y * self.width as i32 + x) as usize;
        if self.cost[i] == u64::MAX {
            return None;
        }
        Some(match self.dir[i] {
            NONE => Vec2::ZERO,
            k => {
                let (dx, dy) = DIRS[k as usize];
                Vec2::new(dx as f32, dy as f32).normalize()
            }
        })
    }

    /// The cost to the nearest target from the cell under `p` (`None` off the field or unreachable).
    pub fn cost_at(&self, p: Vec2) -> Option<u64> {
        let l = (p - self.origin) / self.cell_size;
        let (x, y) = (l.x.floor() as i32, l.y.floor() as i32);
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        let c = self.cost[(y * self.width as i32 + x) as usize];
        (c != u64::MAX).then_some(c)
    }

    /// The direction to move at plane point `p`: the four nearest cells'
    /// directions blended bilinearly (unreachable ones left out), unit
    /// length, or zero at a target or where nothing is reachable. The cell
    /// under `p` decides when the blend cancels out.
    pub fn sample(&self, p: Vec2) -> Vec2 {
        let l = (p - self.origin) / self.cell_size - Vec2::splat(0.5);
        let (x0, y0) = (l.x.floor() as i32, l.y.floor() as i32);
        let (fx, fy) = (l.x - x0 as f32, l.y - y0 as f32);
        let mut sum = Vec2::ZERO;
        for (dx, dy, wgt) in [
            (0, 0, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ] {
            if let Some(d) = self.cell_dir(x0 + dx, y0 + dy) {
                sum += d * wgt;
            }
        }
        if sum.length_squared() > 1e-6 {
            return sum.normalize();
        }
        let c = (p - self.origin) / self.cell_size;
        self.cell_dir(c.x.floor() as i32, c.y.floor() as i32)
            .unwrap_or(Vec2::ZERO)
    }
}
