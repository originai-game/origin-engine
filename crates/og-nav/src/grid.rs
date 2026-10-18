//! Grid pathfinding: a cost grid (0 blocked, 1 to 255 a cell's cost),
//! A* and jump point search over it with eight-way moves that never cut a
//! blocked corner, and line-of-sight smoothing.
//!
//! Costs are integers: an orthogonal step is [`ORTHO`] times the entered
//! cell's cost, a diagonal step [`DIAG`] times it, the heuristic the
//! octile distance in the same units. Ties are broken by the heuristic and
//! then by insertion order, so every host expands the same nodes in the
//! same order and answers the same path.

use og_core::glam::Vec2;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// A cell nothing may enter.
pub const BLOCKED: u8 = 0;
/// The cost unit of an orthogonal step into a cost-1 cell.
pub const ORTHO: u32 = 1000;
/// A diagonal step into a cost-1 cell: 1000 · √2, rounded.
pub const DIAG: u32 = 1414;

/// The eight moves, orthogonal first: (dx, dy).
pub const DIRS: [(i32, i32); 8] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];

/// Which search a path query runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    /// A* over every cell.
    AStar,
    /// Jump point search: the same optimal paths on a uniform-cost grid,
    /// expanding only jump points. On a grid with weighted cells it runs
    /// A* instead (jump points assume uniform costs).
    Jps,
}

/// A cost grid over a plane: `width × height` square cells of `cell_size`,
/// cell (0, 0)'s lower corner at `origin`.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    costs: Vec<u8>,
    /// Cells whose cost is not 0 or 1; jump point search needs none.
    weighted: u32,
    /// Bumped by every change, so cached paths and fields know to rebuild.
    version: u64,
}

/// A path through cells, start and goal included.
#[derive(Clone, Debug, PartialEq)]
pub struct GridPath {
    pub cells: Vec<(u32, u32)>,
    /// The summed step costs, in [`ORTHO`] units.
    pub cost: u64,
    /// Nodes taken off the open list.
    pub expanded: u32,
    /// What actually ran (`Jps` on a weighted grid runs `AStar`).
    pub algorithm: Algorithm,
}

/// Search buffers reused across queries on grids of the same size.
#[derive(Clone, Debug, Default)]
pub struct Scratch {
    g: Vec<u64>,
    parent: Vec<u32>,
    stamp: Vec<u32>,
    closed: Vec<u32>,
    epoch: u32,
}

impl Scratch {
    fn begin(&mut self, n: usize) {
        if self.g.len() != n {
            self.g = vec![0; n];
            self.parent = vec![u32::MAX; n];
            self.stamp = vec![0; n];
            self.closed = vec![0; n];
            self.epoch = 0;
        }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.stamp.iter_mut().for_each(|s| *s = 0);
            self.closed.iter_mut().for_each(|s| *s = 0);
            self.epoch = 1;
        }
    }
    fn seen(&self, i: usize) -> bool {
        self.stamp[i] == self.epoch
    }
    fn is_closed(&self, i: usize) -> bool {
        self.closed[i] == self.epoch
    }
    fn g(&self, i: usize) -> u64 {
        if self.seen(i) { self.g[i] } else { u64::MAX }
    }
}

impl Grid {
    /// A grid of cost-1 cells.
    pub fn new(width: u32, height: u32, cell_size: f32, origin: Vec2) -> Grid {
        Grid {
            width,
            height,
            cell_size: if cell_size > 0.0 { cell_size } else { 1.0 },
            origin,
            costs: vec![1; (width as usize) * (height as usize)],
            weighted: 0,
            version: 1,
        }
    }

    /// A grid from row-major costs (row 0 first); `costs.len()` must be `width × height`.
    pub fn from_costs(
        width: u32,
        height: u32,
        cell_size: f32,
        origin: Vec2,
        costs: Vec<u8>,
    ) -> Option<Grid> {
        if costs.len() != (width as usize) * (height as usize) {
            return None;
        }
        let weighted = costs.iter().filter(|&&c| c > 1).count() as u32;
        Some(Grid {
            width,
            height,
            cell_size: if cell_size > 0.0 { cell_size } else { 1.0 },
            origin,
            costs,
            weighted,
            version: 1,
        })
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Whether every walkable cell costs 1.
    pub fn uniform(&self) -> bool {
        self.weighted == 0
    }

    pub fn costs(&self) -> &[u8] {
        &self.costs
    }

    fn index(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// The cost of a cell; [`BLOCKED`] outside the grid.
    pub fn cost(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return BLOCKED;
        }
        self.costs[self.index(x as u32, y as u32)]
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.cost(x, y) != BLOCKED
    }

    /// Sets a cell's cost; false outside the grid.
    pub fn set(&mut self, x: u32, y: u32, cost: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let i = self.index(x, y);
        let old = self.costs[i];
        if old == cost {
            return true;
        }
        if old > 1 {
            self.weighted -= 1;
        }
        if cost > 1 {
            self.weighted += 1;
        }
        self.costs[i] = cost;
        self.version += 1;
        true
    }

    /// The cell a plane point falls in, if it is on the grid.
    pub fn cell_at(&self, p: Vec2) -> Option<(u32, u32)> {
        let local = (p - self.origin) / self.cell_size;
        let (x, y) = (local.x.floor(), local.y.floor());
        (x >= 0.0 && y >= 0.0 && x < self.width as f32 && y < self.height as f32)
            .then_some((x as u32, y as u32))
    }

    /// The nearest walkable cell to `p` within `radius` cells (ring by ring,
    /// then by distance to the point, then by index): where a query that
    /// starts or ends on a wall goes instead.
    pub fn nearest_walkable(&self, p: Vec2, radius: u32) -> Option<(u32, u32)> {
        let local = (p - self.origin) / self.cell_size;
        let cx = local.x.floor() as i32;
        let cy = local.y.floor() as i32;
        if self.walkable(cx, cy) {
            return Some((cx as u32, cy as u32));
        }
        for r in 1..=radius as i32 {
            let mut best: Option<(f32, i32, i32)> = None;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (x, y) = (cx + dx, cy + dy);
                    if !self.walkable(x, y) {
                        continue;
                    }
                    let c = self.center(x as u32, y as u32);
                    let d = c.distance_squared(p);
                    if best.is_none_or(|(bd, bx, by)| d < bd || (d == bd && (y, x) < (by, bx))) {
                        best = Some((d, x, y));
                    }
                }
            }
            if let Some((_, x, y)) = best {
                return Some((x as u32, y as u32));
            }
        }
        None
    }

    /// The plane centre of a cell.
    pub fn center(&self, x: u32, y: u32) -> Vec2 {
        self.origin + Vec2::new(x as f32 + 0.5, y as f32 + 0.5) * self.cell_size
    }

    /// Whether a move from (x, y) by (dx, dy) is allowed: into a walkable
    /// cell, and for a diagonal, past two walkable orthogonal cells.
    pub fn can_step(&self, x: i32, y: i32, dx: i32, dy: i32) -> bool {
        if !self.walkable(x + dx, y + dy) {
            return false;
        }
        dx == 0 || dy == 0 || (self.walkable(x + dx, y) && self.walkable(x, y + dy))
    }

    fn step_cost(&self, x: i32, y: i32, dx: i32, dy: i32) -> u64 {
        let base = if dx != 0 && dy != 0 { DIAG } else { ORTHO };
        base as u64 * self.cost(x + dx, y + dy) as u64
    }

    fn octile(ax: i32, ay: i32, bx: i32, by: i32) -> u64 {
        let dx = (ax - bx).unsigned_abs() as u64;
        let dy = (ay - by).unsigned_abs() as u64;
        let (lo, hi) = if dx < dy { (dx, dy) } else { (dy, dx) };
        DIAG as u64 * lo + ORTHO as u64 * (hi - lo)
    }

    /// The cheapest path from `start` to `goal`, or `None` when the goal is
    /// unreachable or either end is blocked.
    pub fn find_path(
        &self,
        start: (u32, u32),
        goal: (u32, u32),
        algorithm: Algorithm,
        scratch: &mut Scratch,
    ) -> Option<GridPath> {
        let (sx, sy) = (start.0 as i32, start.1 as i32);
        let (gx, gy) = (goal.0 as i32, goal.1 as i32);
        if !self.walkable(sx, sy) || !self.walkable(gx, gy) {
            return None;
        }
        let algorithm = if algorithm == Algorithm::Jps && !self.uniform() {
            Algorithm::AStar
        } else {
            algorithm
        };
        let n = self.costs.len();
        scratch.begin(n);
        let si = self.index(start.0, start.1);
        let gi = self.index(goal.0, goal.1);
        // (f, h, seq, node): the smallest f, then the smallest h, then the oldest.
        let mut open: BinaryHeap<Reverse<(u64, u64, u32, u32)>> = BinaryHeap::new();
        let mut seq = 0u32;
        scratch.stamp[si] = scratch.epoch;
        scratch.g[si] = 0;
        scratch.parent[si] = u32::MAX;
        let h0 = Self::octile(sx, sy, gx, gy);
        open.push(Reverse((h0, h0, seq, si as u32)));
        let mut expanded = 0u32;
        let w = self.width as i32;
        while let Some(Reverse((_, _, _, node))) = open.pop() {
            let ni = node as usize;
            if scratch.is_closed(ni) {
                continue;
            }
            scratch.closed[ni] = scratch.epoch;
            expanded += 1;
            if ni == gi {
                return Some(self.trace(scratch, si, gi, expanded, algorithm));
            }
            let x = (ni % self.width as usize) as i32;
            let y = (ni / self.width as usize) as i32;
            let g = scratch.g[ni];
            let mut relax =
                |nx: i32,
                 ny: i32,
                 cost: u64,
                 scratch: &mut Scratch,
                 open: &mut BinaryHeap<Reverse<(u64, u64, u32, u32)>>| {
                    let i = (ny * w + nx) as usize;
                    if scratch.is_closed(i) {
                        return;
                    }
                    let ng = g + cost;
                    if ng < scratch.g(i) {
                        scratch.stamp[i] = scratch.epoch;
                        scratch.g[i] = ng;
                        scratch.parent[i] = node;
                        let h = Self::octile(nx, ny, gx, gy);
                        seq += 1;
                        open.push(Reverse((ng + h, h, seq, i as u32)));
                    }
                };
            match algorithm {
                Algorithm::AStar => {
                    for (dx, dy) in DIRS {
                        if self.can_step(x, y, dx, dy) {
                            let c = self.step_cost(x, y, dx, dy);
                            relax(x + dx, y + dy, c, scratch, &mut open);
                        }
                    }
                }
                Algorithm::Jps => {
                    let parent = scratch.parent[ni];
                    let dirs = self.pruned(x, y, parent);
                    for (dx, dy) in dirs.iter().flatten().copied() {
                        if let Some((jx, jy)) = self.jump(x, y, dx, dy, gx, gy) {
                            let c = Self::octile(x, y, jx, jy);
                            relax(jx, jy, c, scratch, &mut open);
                        }
                    }
                }
            }
        }
        None
    }

    fn trace(
        &self,
        scratch: &Scratch,
        si: usize,
        gi: usize,
        expanded: u32,
        algorithm: Algorithm,
    ) -> GridPath {
        let mut points = vec![gi];
        let mut at = gi;
        while at != si {
            at = scratch.parent[at] as usize;
            points.push(at);
        }
        points.reverse();
        let w = self.width as usize;
        let mut cells = Vec::with_capacity(points.len());
        for (k, &p) in points.iter().enumerate() {
            let (x, y) = ((p % w) as i32, (p / w) as i32);
            if k == 0 {
                cells.push((x as u32, y as u32));
                continue;
            }
            // Jump points are joined by straight or diagonal runs: fill them in.
            let (px, py) = cells
                .last()
                .map(|&(a, b)| (a as i32, b as i32))
                .unwrap_or((x, y));
            let (dx, dy) = ((x - px).signum(), (y - py).signum());
            let (mut cx, mut cy) = (px, py);
            while (cx, cy) != (x, y) {
                cx += dx;
                cy += dy;
                cells.push((cx as u32, cy as u32));
            }
        }
        GridPath {
            cells,
            cost: scratch.g[gi],
            expanded,
            algorithm,
        }
    }

    /// The directions jump point search explores from (x, y), coming from
    /// `parent` (no corner cutting).
    fn pruned(&self, x: i32, y: i32, parent: u32) -> [Option<(i32, i32)>; 8] {
        let mut out = [None; 8];
        let mut k = 0;
        let mut push = |d: (i32, i32)| {
            out[k] = Some(d);
            k += 1;
        };
        if parent == u32::MAX {
            for (dx, dy) in DIRS {
                if self.can_step(x, y, dx, dy) {
                    push((dx, dy));
                }
            }
            return out;
        }
        let px = (parent % self.width) as i32;
        let py = (parent / self.width) as i32;
        let dx = (x - px).signum();
        let dy = (y - py).signum();
        if dx != 0 && dy != 0 {
            let v = self.walkable(x, y + dy);
            let h = self.walkable(x + dx, y);
            if v {
                push((0, dy));
            }
            if h {
                push((dx, 0));
            }
            if v && h && self.walkable(x + dx, y + dy) {
                push((dx, dy));
            }
        } else if dx != 0 {
            let next = self.walkable(x + dx, y);
            let up = self.walkable(x, y + 1);
            let down = self.walkable(x, y - 1);
            if next {
                push((dx, 0));
                if up && self.walkable(x + dx, y + 1) {
                    push((dx, 1));
                }
                if down && self.walkable(x + dx, y - 1) {
                    push((dx, -1));
                }
            }
            if up {
                push((0, 1));
            }
            if down {
                push((0, -1));
            }
        } else {
            let next = self.walkable(x, y + dy);
            let right = self.walkable(x + 1, y);
            let left = self.walkable(x - 1, y);
            if next {
                push((0, dy));
                if right && self.walkable(x + 1, y + dy) {
                    push((1, dy));
                }
                if left && self.walkable(x - 1, y + dy) {
                    push((-1, dy));
                }
            }
            if right {
                push((1, 0));
            }
            if left {
                push((-1, 0));
            }
        }
        out
    }

    /// The next jump point from (x, y) in direction (dx, dy), or `None`.
    fn jump(&self, x: i32, y: i32, dx: i32, dy: i32, gx: i32, gy: i32) -> Option<(i32, i32)> {
        let (mut cx, mut cy) = (x, y);
        loop {
            if !self.can_step(cx, cy, dx, dy) {
                return None;
            }
            cx += dx;
            cy += dy;
            if (cx, cy) == (gx, gy) {
                return Some((cx, cy));
            }
            if dx != 0 && dy != 0 {
                // A diagonal stops where a straight jump from it finds something.
                if self.jump_straight(cx, cy, dx, 0, gx, gy)
                    || self.jump_straight(cx, cy, 0, dy, gx, gy)
                {
                    return Some((cx, cy));
                }
            } else if self.forced(cx, cy, dx, dy) {
                return Some((cx, cy));
            }
        }
    }

    /// Whether a straight jump from (x, y) reaches a jump point.
    fn jump_straight(&self, x: i32, y: i32, dx: i32, dy: i32, gx: i32, gy: i32) -> bool {
        let (mut cx, mut cy) = (x, y);
        loop {
            if !self.walkable(cx + dx, cy + dy) {
                return false;
            }
            cx += dx;
            cy += dy;
            if (cx, cy) == (gx, gy) || self.forced(cx, cy, dx, dy) {
                return true;
            }
        }
    }

    /// A straight move's forced neighbour at (x, y): a side cell open here
    /// but closed one step back.
    fn forced(&self, x: i32, y: i32, dx: i32, dy: i32) -> bool {
        if dx != 0 {
            (self.walkable(x, y - 1) && !self.walkable(x - dx, y - 1))
                || (self.walkable(x, y + 1) && !self.walkable(x - dx, y + 1))
        } else {
            (self.walkable(x - 1, y) && !self.walkable(x - 1, y - dy))
                || (self.walkable(x + 1, y) && !self.walkable(x + 1, y - dy))
        }
    }

    /// Whether the segment between two plane points crosses only walkable
    /// cells, passing a corner only between two walkable cells: a straight
    /// move an agent of zero radius can make. `clearance` (in cells) also
    /// tests two lines offset to either side by it.
    pub fn line_of_sight(&self, a: Vec2, b: Vec2, clearance: f32) -> bool {
        let to_local = |p: Vec2| (p - self.origin) / self.cell_size;
        let (la, lb) = (to_local(a), to_local(b));
        if !self.ray(la, lb) {
            return false;
        }
        if clearance > 0.0 {
            let d = lb - la;
            let len = d.length();
            if len > 1e-6 {
                let n = Vec2::new(-d.y, d.x) / len * clearance;
                return self.ray(la + n, lb + n) && self.ray(la - n, lb - n);
            }
        }
        true
    }

    /// Amanatides–Woo traversal in cell units.
    fn ray(&self, a: Vec2, b: Vec2) -> bool {
        let (mut x, mut y) = (a.x.floor() as i32, a.y.floor() as i32);
        let (ex, ey) = (b.x.floor() as i32, b.y.floor() as i32);
        if !self.walkable(x, y) {
            return false;
        }
        let d = b - a;
        let sx = if d.x > 0.0 {
            1
        } else if d.x < 0.0 {
            -1
        } else {
            0
        };
        let sy = if d.y > 0.0 {
            1
        } else if d.y < 0.0 {
            -1
        } else {
            0
        };
        let inv = |v: f32| {
            if v != 0.0 {
                1.0 / v.abs()
            } else {
                f32::INFINITY
            }
        };
        let (dtx, dty) = (inv(d.x), inv(d.y));
        let mut tx = if sx > 0 {
            (x as f32 + 1.0 - a.x) * dtx
        } else if sx < 0 {
            (a.x - x as f32) * dtx
        } else {
            f32::INFINITY
        };
        let mut ty = if sy > 0 {
            (y as f32 + 1.0 - a.y) * dty
        } else if sy < 0 {
            (a.y - y as f32) * dty
        } else {
            f32::INFINITY
        };
        let mut guard = (self.width + self.height) as usize * 2 + 4;
        while (x, y) != (ex, ey) {
            guard -= 1;
            if guard == 0 {
                return false;
            }
            if tx.min(ty) > 1.0 {
                break;
            }
            if (tx - ty).abs() < 1e-6 {
                // Through a corner: both side cells must be open.
                if !self.walkable(x + sx, y) || !self.walkable(x, y + sy) {
                    return false;
                }
                x += sx;
                y += sy;
                tx += dtx;
                ty += dty;
            } else if tx < ty {
                x += sx;
                tx += dtx;
            } else {
                y += sy;
                ty += dty;
            }
            if !self.walkable(x, y) {
                return false;
            }
        }
        true
    }

    /// A cell path as plane points (cell centres), with the end points
    /// replaced by `from` and `to`, and, when `smooth`, every point that a
    /// straight line can skip removed (string pulling by line of sight).
    pub fn waypoints(
        &self,
        path: &GridPath,
        from: Vec2,
        to: Vec2,
        smooth: bool,
        clearance: f32,
    ) -> Vec<Vec2> {
        let mut pts: Vec<Vec2> = path.cells.iter().map(|&(x, y)| self.center(x, y)).collect();
        if pts.is_empty() {
            return pts;
        }
        pts[0] = from;
        let last = pts.len() - 1;
        pts[last] = to;
        // Drop collinear interior points first: a straight run is one segment.
        let mut out = vec![pts[0]];
        for i in 1..pts.len() - 1 {
            let (a, b, c) = (*out.last().unwrap(), pts[i], pts[i + 1]);
            let cross = (b - a).perp_dot(c - b);
            if cross.abs() > 1e-5 || (b - a).dot(c - b) < 0.0 {
                out.push(b);
            }
        }
        if pts.len() > 1 {
            out.push(pts[last]);
        }
        if !smooth || out.len() <= 2 {
            return out;
        }
        let mut pulled = vec![out[0]];
        let mut i = 0;
        while i < out.len() - 1 {
            // The farthest point visible from out[i].
            let mut j = out.len() - 1;
            while j > i + 1 && !self.line_of_sight(out[i], out[j], clearance) {
                j -= 1;
            }
            pulled.push(out[j]);
            i = j;
        }
        pulled
    }
}
