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

#[cfg(test)]
mod tests {
    use super::*;

    fn ground(w: u32, h: u32, top: u32, chunk: u32) -> PixelTerrain {
        let cells = (0..w * h).map(|i| u8::from(i / w < top)).collect();
        PixelTerrain::from_cells(w, h, chunk, cells).unwrap()
    }

    /// A flat ground's outline is its top edge, split at the chunk border
    /// with both pieces ending on it, and no edge runs down the border
    /// inside the ground.
    #[test]
    fn a_flat_ground_is_one_line_per_chunk_meeting_at_the_border() {
        let t = ground(16, 8, 4, 8);
        let left = t.chunk_chains(0, 0.5);
        let right = t.chunk_chains(1, 0.5);
        // The left chunk: from the border along the map's bottom, up its
        // left edge, then along the top back to the border.
        assert_eq!(
            left,
            vec![vec![[8.0, 0.0], [0.0, 0.0], [0.0, 4.0], [8.0, 4.0]]]
        );
        // The right: from the border along the top, down the map's right
        // edge and back along the bottom.
        assert_eq!(
            right,
            vec![vec![[8.0, 4.0], [16.0, 4.0], [16.0, 0.0], [8.0, 0.0]]]
        );
    }

    /// A carved crater dirties the chunks it touches and its neighbours'
    /// border cells only, leaves bedrock, and its outline dips into it.
    #[test]
    fn a_carve_dirties_its_chunks_spares_bedrock_and_dips_the_outline() {
        let mut t = ground(32, 16, 8, 8);
        t.take_dirty();
        t.set(20, 7, BEDROCK);
        let n = t.circle(20.5, 8.0, 3.0, 0);
        assert!(n > 10, "{n}");
        assert_eq!(t.get(20, 7), BEDROCK);
        assert_eq!(t.get(18, 7), 0);
        assert_eq!(t.take_dirty(), vec![2, 6]);
        let chains = t.chunk_chains(2, 0.0);
        // Above the map's bottom edge, the surface's lowest point.
        let lowest = chains
            .iter()
            .flatten()
            .map(|p| p[1])
            .filter(|&y| y > 0.0)
            .fold(f32::MAX, f32::min);
        assert!((5.0..8.0).contains(&lowest), "{chains:?}");
        assert!(t.take_dirty().is_empty());
    }

    /// Two solid cells touching only at a corner make two loops, not a
    /// figure eight; a hole inside the ground is a loop of its own, run
    /// the other way.
    #[test]
    fn diagonal_cells_make_two_loops_and_a_hole_runs_the_other_way() {
        let mut t = PixelTerrain::new(4, 4, 4);
        t.set(1, 1, 1);
        t.set(2, 2, 1);
        let loops = t.chunk_chains(0, 0.0);
        assert_eq!(loops.len(), 2, "{loops:?}");
        for l in &loops {
            assert_eq!(l.first(), l.last());
            // Solid on the right: clockwise in y up, a negative area.
            assert!(polygon_area(&l[..l.len() - 1]) < 0.0, "{l:?}");
        }
        let mut g = ground(8, 8, 8, 8);
        g.set(3, 4, 0);
        g.set(4, 4, 0);
        let chains = g.chunk_chains(0, 0.0);
        let hole = chains
            .iter()
            .find(|c| c.first() == c.last() && c.iter().all(|p| p[0] > 0.0 && p[1] > 0.0))
            .expect("the hole's loop");
        assert!(polygon_area(&hole[..hole.len() - 1]) > 0.0, "{hole:?}");
    }

    /// A staircase simplifies to its slope within the tolerance, and to
    /// every corner at 0.
    #[test]
    fn a_staircase_simplifies_to_its_slope() {
        let mut t = PixelTerrain::new(8, 8, 8);
        for x in 0..8 {
            for y in 0..=x {
                t.set(x, y, 1);
            }
        }
        let exact: usize = t.chunk_chains(0, 0.0).iter().map(Vec::len).sum();
        let smooth: usize = t.chunk_chains(0, 1.0).iter().map(Vec::len).sum();
        assert!(exact >= 16 && smooth <= 5, "{exact} {smooth}");
    }

    /// Cutting a bridge's pier drops the deck as one island, and the
    /// ground left behind keeps everything touching the bottom row.
    #[test]
    fn a_cut_pier_drops_the_deck_as_one_island() {
        let mut t = PixelTerrain::new(12, 10, 4);
        for x in 0..12 {
            t.set(x, 0, 1);
        }
        for y in 1..6 {
            t.set(5, y, 1);
        }
        for x in 2..10 {
            t.set(x, 6, 2);
        }
        assert!(t.detach().is_empty());
        t.set(5, 3, 0);
        let islands = t.detach();
        assert_eq!(islands.len(), 1);
        let deck = &islands[0];
        assert_eq!((deck.origin, deck.width, deck.height), ([2, 4], 8, 3));
        assert_eq!(deck.cells.iter().filter(|&&c| c != 0).count(), 8 + 2);
        assert_eq!(t.get(5, 6), 0);
        assert_eq!(t.get(5, 2), 1);
    }

    /// The pieces tile the rectangle: convex, counter-clockwise, their
    /// areas summing to its area, each holding its seed; a repeated seed
    /// gets none.
    #[test]
    fn voronoi_pieces_tile_the_rectangle() {
        let seeds = [[1.0, 1.0], [3.0, 0.5], [2.0, 2.5], [0.4, 2.8], [3.0, 0.5]];
        let pieces = voronoi_pieces(4.0, 3.0, &seeds);
        assert!(pieces[4].is_empty());
        let total: f32 = pieces.iter().map(|p| polygon_area(p)).sum();
        assert!((total - 12.0).abs() < 1e-4, "{total}");
        for (p, s) in pieces.iter().zip(&seeds).take(4) {
            assert!(polygon_area(p) > 0.0);
            // The seed is inside: left of every edge.
            for k in 0..p.len() {
                let (a, b) = (p[k], p[(k + 1) % p.len()]);
                let cross = (b[0] - a[0]) * (s[1] - a[1]) - (b[1] - a[1]) * (s[0] - a[0]);
                assert!(cross >= -1e-5, "{p:?} {s:?}");
            }
        }
    }
}
