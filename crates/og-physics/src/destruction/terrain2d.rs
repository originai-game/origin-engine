//! 2D destruction without physics or rendering: a pixel terrain that is
//! carved and filled ([`PixelTerrain`], Worms and Noita style), the collision
//! outline of each of its chunks ([`PixelTerrain::chunk_chains`]), the solid
//! islands a carve cuts loose ([`PixelTerrain::detach`]) and the Voronoi
//! pieces a sprite breaks into ([`voronoi_pieces`]).
//!
//! The method is the one Noita's and Slow Rush's write-ups describe
//! (Purho, "Exploring the Tech and Design of Noita", GDC 2019; slowrush.dev,
//! "Bridging Physics Worlds"): the terrain is a grid of cells in chunks
//! with dirty flags; a dirty chunk's outline is traced along the edges
//! between solid and empty cells (marching squares on cell centres gives
//! the same loops), simplified with Douglas-Peucker and handed to the
//! physics as a polyline. Here the edges are taken per chunk but tested
//! against the whole grid, so an outline never runs along a chunk border
//! inside solid ground (no seam a body can catch on), and a chain that
//! crosses a border ends exactly on it in both chunks (the simplification
//! keeps its ends), so neighbouring chunks meet with no gap.
//!
//! Integers and f32 `+ - * /` and `abs` only, in fixed orders: the same bits
//! on every host, so a carve replays.

pub mod collider;
pub mod cut;
pub mod state;

/// A cell's material: 0 is empty; [`BEDROCK`] is never carved.
pub type Cell = u8;

/// The material carving leaves alone (map edges, unbreakable floors).
pub const BEDROCK: Cell = 255;

/// A grid of cells in square chunks, with the chunks a change touched
/// marked dirty until their outline is taken.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PixelTerrain {
    width: u32,
    height: u32,
    chunk: u32,
    cells: Vec<Cell>,
    dirty: Vec<bool>,
}

/// One chain of a chunk's outline in cell units (x right, y up, a cell
/// (x, y) covering x..x+1, y..y+1): solid on its right as it runs, closed
/// when its last point is its first.
pub type Chain = Vec<[f32; 2]>;

/// Solid cells cut loose from the anchored ground ([`PixelTerrain::detach`]):
/// their cells, row by row from `origin` (`width` wide), 0 where empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Island {
    pub origin: [u32; 2],
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
}

impl PixelTerrain {
    /// An empty terrain of `width` by `height` cells in chunks of `chunk`
    /// (at least 1) cells a side.
    pub fn new(width: u32, height: u32, chunk: u32) -> PixelTerrain {
        let chunk = chunk.max(1);
        let n = (width.div_ceil(chunk) * height.div_ceil(chunk)) as usize;
        PixelTerrain {
            width,
            height,
            chunk,
            cells: vec![0; (width * height) as usize],
            dirty: vec![true; n],
        }
    }

    /// A terrain from cells row by row, bottom row first.
    pub fn from_cells(width: u32, height: u32, chunk: u32, cells: Vec<Cell>) -> Option<Self> {
        (cells.len() == (width * height) as usize).then(|| PixelTerrain {
            cells,
            ..PixelTerrain::new(width, height, chunk)
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn chunk_size(&self) -> u32 {
        self.chunk
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
    /// Chunks across and up.
    pub fn chunks(&self) -> (u32, u32) {
        (
            self.width.div_ceil(self.chunk),
            self.height.div_ceil(self.chunk),
        )
    }

    /// The cell at (x, y); outside the grid, empty.
    pub fn get(&self, x: i64, y: i64) -> Cell {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return 0;
        }
        self.cells[(y as u32 * self.width + x as u32) as usize]
    }

    /// Sets a cell (bedrock stays unless `value` is bedrock too) and marks
    /// its chunk and any neighbour whose outline it borders. Whether it
    /// changed.
    pub fn set(&mut self, x: u32, y: u32, value: Cell) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let i = (y * self.width + x) as usize;
        if self.cells[i] == value || (self.cells[i] == BEDROCK && value != BEDROCK) {
            return false;
        }
        let solid_changed = (self.cells[i] != 0) != (value != 0);
        self.cells[i] = value;
        if solid_changed {
            self.touch(x, y);
        }
        true
    }

    fn touch(&mut self, x: u32, y: u32) {
        let (cx, cy) = self.chunks();
        for (dx, dy) in [(0i64, 0i64), (-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (nx, ny) = (x as i64 + dx, y as i64 + dy);
            if nx < 0 || ny < 0 || nx >= self.width as i64 || ny >= self.height as i64 {
                continue;
            }
            let k = (ny as u32 / self.chunk) * cx + nx as u32 / self.chunk;
            debug_assert!(k < cx * cy);
            self.dirty[k as usize] = true;
        }
    }

    /// Every cell whose centre lies within `radius` of (x, y) set to
    /// `value` (0 carves, a material fills). How many changed.
    pub fn circle(&mut self, x: f32, y: f32, radius: f32, value: Cell) -> u32 {
        let r2 = radius * radius;
        let lo = |c: f32| ((c - radius).floor().max(0.0)) as u32;
        let (x0, y0) = (lo(x), lo(y));
        let x1 = ((x + radius).ceil().max(0.0) as u32).min(self.width);
        let y1 = ((y + radius).ceil().max(0.0) as u32).min(self.height);
        let mut n = 0;
        for cy in y0..y1 {
            for cx in x0..x1 {
                let (dx, dy) = (cx as f32 + 0.5 - x, cy as f32 + 0.5 - y);
                if dx * dx + dy * dy <= r2 && self.set(cx, cy, value) {
                    n += 1;
                }
            }
        }
        n
    }

    /// The dirty chunks by index (row by row), clearing their flags.
    pub fn take_dirty(&mut self) -> Vec<u32> {
        let out: Vec<u32> = (0..self.dirty.len() as u32)
            .filter(|&k| self.dirty[k as usize])
            .collect();
        self.dirty.iter_mut().for_each(|d| *d = false);
        out
    }

    /// Chunk `k`'s outline: the edges of its solid cells that face an
    /// empty cell (in any chunk, or outside the grid), linked into chains
    /// with solid on their right, each simplified within `tolerance` cells
    /// (0 keeps every corner) with its ends kept. Chains run in a fixed
    /// order (by their first edge, bottom row first).
    pub fn chunk_chains(&self, k: u32, tolerance: f32) -> Vec<Chain> {
        let (cx, _) = self.chunks();
        let (x0, y0) = ((k % cx) * self.chunk, (k / cx) * self.chunk);
        let x1 = (x0 + self.chunk).min(self.width);
        let y1 = (y0 + self.chunk).min(self.height);
        let solid = |x: i64, y: i64| self.get(x, y) != 0;
        // Directed unit edges between lattice points, solid on the right.
        let mut edges: Vec<([i64; 2], [i64; 2])> = Vec::new();
        for y in y0 as i64..y1 as i64 {
            for x in x0 as i64..x1 as i64 {
                if !solid(x, y) {
                    continue;
                }
                if !solid(x, y + 1) {
                    edges.push(([x, y + 1], [x + 1, y + 1])); // top, running right
                }
                if !solid(x + 1, y) {
                    edges.push(([x + 1, y + 1], [x + 1, y])); // right, running down
                }
                if !solid(x, y - 1) {
                    edges.push(([x + 1, y], [x, y])); // bottom, running left
                }
                if !solid(x - 1, y) {
                    edges.push(([x, y], [x, y + 1])); // left, running up
                }
            }
        }
        link(edges)
            .into_iter()
            .map(|c| simplify(&c, tolerance))
            .collect()
    }

    /// Takes out every solid island not connected (through its four
    /// neighbours) to a bedrock cell or to the grid's bottom row, and
    /// returns them, largest first (ties by their lowest cell): the pieces
    /// a carve cut loose, to fall as bodies.
    pub fn detach(&mut self) -> Vec<Island> {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut seen = vec![false; w * h];
        let mut stack = Vec::new();
        let solid = |c: &Vec<Cell>, i: usize| c[i] != 0;
        for i in 0..w * h {
            if solid(&self.cells, i) && (i < w || self.cells[i] == BEDROCK) && !seen[i] {
                seen[i] = true;
                stack.push(i);
            }
        }
        flood(&self.cells, w, h, &mut seen, &mut stack);
        let mut islands = Vec::new();
        for start in 0..w * h {
            if !solid(&self.cells, start) || seen[start] {
                continue;
            }
            seen[start] = true;
            let mut members = vec![start];
            stack.push(start);
            while let Some(i) = stack.pop() {
                let (x, y) = (i % w, i / w);
                for (nx, ny) in neighbours(x, y, w, h) {
                    let j = ny * w + nx;
                    if solid(&self.cells, j) && !seen[j] {
                        seen[j] = true;
                        members.push(j);
                        stack.push(j);
                    }
                }
            }
            let (mut lx, mut ly, mut hx, mut hy) = (w, h, 0, 0);
            for &i in &members {
                let (x, y) = (i % w, i / w);
                (lx, ly, hx, hy) = (lx.min(x), ly.min(y), hx.max(x), hy.max(y));
            }
            let (iw, ih) = (hx - lx + 1, hy - ly + 1);
            let mut cells = vec![0; iw * ih];
            for &i in &members {
                let (x, y) = (i % w, i / w);
                cells[(y - ly) * iw + (x - lx)] = self.cells[i];
            }
            for &i in &members {
                self.set((i % w) as u32, (i / w) as u32, 0);
            }
            islands.push((
                members.len(),
                start,
                Island {
                    origin: [lx as u32, ly as u32],
                    width: iw as u32,
                    height: ih as u32,
                    cells,
                },
            ));
        }
        islands.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        islands.into_iter().map(|(_, _, i)| i).collect()
    }
}

fn neighbours(x: usize, y: usize, w: usize, h: usize) -> impl Iterator<Item = (usize, usize)> {
    [
        (x.wrapping_sub(1), y),
        (x + 1, y),
        (x, y.wrapping_sub(1)),
        (x, y + 1),
    ]
    .into_iter()
    .filter(move |&(nx, ny)| nx < w && ny < h)
}

fn flood(cells: &[Cell], w: usize, h: usize, seen: &mut [bool], stack: &mut Vec<usize>) {
    while let Some(i) = stack.pop() {
        for (nx, ny) in neighbours(i % w, i / w, w, h) {
            let j = ny * w + nx;
            if cells[j] != 0 && !seen[j] {
                seen[j] = true;
                stack.push(j);
            }
        }
    }
}

/// Links directed unit edges into chains. Where two edges leave one point
/// (two solid cells touching at a corner), the turn to the right is taken,
/// so diagonal neighbours make separate loops.
fn link(edges: Vec<([i64; 2], [i64; 2])>) -> Vec<Vec<[i64; 2]>> {
    use std::collections::BTreeMap;
    let mut from: BTreeMap<[i64; 2], Vec<usize>> = BTreeMap::new();
    let mut into: BTreeMap<[i64; 2], usize> = BTreeMap::new();
    for (i, (a, b)) in edges.iter().enumerate() {
        from.entry(*a).or_default().push(i);
        *into.entry(*b).or_default() += 1;
    }
    let mut used = vec![false; edges.len()];
    let mut chains = Vec::new();
    let next = |at: [i64; 2], dir: [i64; 2], used: &[bool]| -> Option<usize> {
        let out = from.get(&at)?;
        let free: Vec<usize> = out.iter().copied().filter(|&e| !used[e]).collect();
        if free.len() <= 1 {
            return free.first().copied();
        }
        // The right turn: (dx, dy) -> (dy, -dx).
        let right = [dir[1], -dir[0]];
        free.iter()
            .copied()
            .find(|&e| {
                let (a, b) = edges[e];
                [b[0] - a[0], b[1] - a[1]] == right
            })
            .or(free.first().copied())
    };
    // Open chains first (they start where no edge arrives), then loops.
    let starts: Vec<usize> = (0..edges.len())
        .filter(|&i| !into.contains_key(&edges[i].0))
        .chain(0..edges.len())
        .collect();
    for s in starts {
        if used[s] {
            continue;
        }
        used[s] = true;
        let (a, mut b) = edges[s];
        let mut dir = [b[0] - a[0], b[1] - a[1]];
        let mut chain = vec![a, b];
        while let Some(e) = next(b, dir, &used) {
            used[e] = true;
            let (_, c) = edges[e];
            dir = [c[0] - b[0], c[1] - b[1]];
            b = c;
            chain.push(c);
            if c == a {
                break;
            }
        }
        chains.push(chain);
    }
    chains
}

/// Douglas-Peucker within `tolerance` with the ends kept (a closed chain
/// keeps its start, which is its end, and the point farthest from it), and
/// collinear lattice points always merged.
fn simplify(chain: &[[i64; 2]], tolerance: f32) -> Chain {
    let p: Vec<[f32; 2]> = chain.iter().map(|q| [q[0] as f32, q[1] as f32]).collect();
    let n = p.len();
    let closed = n > 2 && p[0] == p[n - 1];
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let tol = tolerance.max(1e-4);
    if closed {
        let far = (1..n - 1)
            .max_by(|&i, &j| d2(p[0], p[i]).total_cmp(&d2(p[0], p[j])).then(j.cmp(&i)))
            .unwrap_or(0);
        keep[far] = true;
        dp(&p, 0, far, tol, &mut keep);
        dp(&p, far, n - 1, tol, &mut keep);
    } else {
        dp(&p, 0, n - 1, tol, &mut keep);
    }
    p.into_iter()
        .zip(keep)
        .filter_map(|(q, k)| k.then_some(q))
        .collect()
}

fn d2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    dx * dx + dy * dy
}

fn dp(p: &[[f32; 2]], i: usize, j: usize, tol: f32, keep: &mut [bool]) {
    if j <= i + 1 {
        return;
    }
    let (a, b) = (p[i], p[j]);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    let mut best = (0.0f32, 0usize);
    for (k, q) in p.iter().enumerate().take(j).skip(i + 1) {
        // Distance squared to the segment (to the point when it has none).
        let d = if len2 == 0.0 {
            d2(a, *q)
        } else {
            let cross = dx * (q[1] - a[1]) - dy * (q[0] - a[0]);
            cross * cross / len2
        };
        if d > best.0 {
            best = (d, k);
        }
    }
    if best.0 > tol * tol {
        keep[best.1] = true;
        dp(p, i, best.1, tol, keep);
        dp(p, best.1, j, tol, keep);
    }
}

/// A `width` by `height` rectangle (corner at 0, 0) cut into the Voronoi
/// cells of `seeds`: one convex polygon per seed inside the rectangle,
/// counter-clockwise, in the seeds' order (an empty one for a seed outside
/// it or one that repeats an earlier one). Half-plane clipping, quadratic
/// in the seeds: meant for the tens of pieces a sprite breaks into.
pub fn voronoi_pieces(width: f32, height: f32, seeds: &[[f32; 2]]) -> Vec<Vec<[f32; 2]>> {
    let rect = vec![[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]];
    seeds
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            if s[0] < 0.0 || s[1] < 0.0 || s[0] > width || s[1] > height || seeds[..i].contains(&s)
            {
                return Vec::new();
            }
            let mut poly = rect.clone();
            for (j, &o) in seeds.iter().enumerate() {
                if j == i || o == s {
                    continue;
                }
                // Keep the side nearer s: dot(p - m, n) <= 0, n = o - s, m the midpoint.
                let n = [o[0] - s[0], o[1] - s[1]];
                let m = [(o[0] + s[0]) * 0.5, (o[1] + s[1]) * 0.5];
                poly = clip(&poly, n, m);
                if poly.is_empty() {
                    break;
                }
            }
            poly
        })
        .collect()
}

fn clip(poly: &[[f32; 2]], n: [f32; 2], m: [f32; 2]) -> Vec<[f32; 2]> {
    let side = |p: [f32; 2]| (p[0] - m[0]) * n[0] + (p[1] - m[1]) * n[1];
    let mut out = Vec::with_capacity(poly.len() + 1);
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        let (sa, sb) = (side(a), side(b));
        if sa <= 0.0 {
            out.push(a);
        }
        if (sa <= 0.0) != (sb <= 0.0) {
            let t = sa / (sa - sb);
            out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
        }
    }
    out
}

/// The area of a polygon, positive when counter-clockwise.
pub fn polygon_area(poly: &[[f32; 2]]) -> f32 {
    let mut s = 0.0;
    for k in 0..poly.len() {
        let (a, b) = (poly[k], poly[(k + 1) % poly.len()]);
        s += a[0] * b[1] - b[0] * a[1];
    }
    s * 0.5
}
