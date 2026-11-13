//! The particle fluid's neighbour search: a spatial hash of cells one
//! radius wide, the same in two and three dimensions.

use super::solve::Point;

fn bucket_of(c: [i32; 3], mask: u32) -> u32 {
    let h = (c[0] as u32).wrapping_mul(0x8da6_b343)
        ^ (c[1] as u32).wrapping_mul(0xd816_3841)
        ^ (c[2] as u32).wrapping_mul(0xcb1a_b31f);
    (h ^ (h >> 16)) & mask
}

/// The cell itself and the 13 of the 26 around it that come after it
/// (z, then y, then x first). The first five are the cell and the four
/// after it in its own plane: what a 2D fluid searches
/// ([`Point::FORWARD`]).
const FORWARD: [[i32; 3]; 14] = [
    [0, 0, 0],
    [1, 0, 0],
    [-1, 1, 0],
    [0, 1, 0],
    [1, 1, 0],
    [-1, -1, 1],
    [0, -1, 1],
    [1, -1, 1],
    [-1, 0, 1],
    [0, 0, 1],
    [1, 0, 1],
    [-1, 1, 1],
    [0, 1, 1],
    [1, 1, 1],
];

/// The neighbour pairs of one substep: for each particle `i`, the
/// particles `j > i` closer than the radius, in a fixed order
/// (`start[i]..start[i + 1]` into `pairs`).
pub(crate) struct Pairs {
    pub start: Vec<u32>,
    pub pairs: Vec<u32>,
}

impl Pairs {
    /// Builds the spatial hash (a bucket per cell of size `h`, at least
    /// twice as many buckets as particles) and finds the pairs: each
    /// particle against its own cell and the cells after it (half the
    /// cells around, so each pair is met once), the bucket's other cells
    /// (hash collisions) told apart by their coordinates. The search runs
    /// by ranges of particles on the lanes from two grains of them
    /// (`crate::lanes::ranges`), the ranges' pairs joined in particle
    /// order: the pairs a single thread finds, in its order.
    pub fn find<V: Point>(positions: &[V], h: f32) -> Pairs {
        let n = positions.len();
        let inv = 1.0 / h;
        let table = (2 * n).max(16).next_power_of_two();
        let mask = table as u32 - 1;
        let cells: Vec<[i32; 3]> = positions.iter().map(|p| p.cell(inv)).collect();
        // Counting sort: each bucket's particles in index order, their
        // cells and positions packed alongside.
        let mut first = vec![0u32; table + 1];
        for c in &cells {
            first[bucket_of(*c, mask) as usize + 1] += 1;
        }
        for k in 0..table {
            first[k + 1] += first[k];
        }
        let mut fill = first.clone();
        let mut sorted = vec![0u32; n];
        let mut packed = vec![(V::ZERO, [0i32; 3]); n];
        for (i, c) in cells.iter().enumerate() {
            let b = bucket_of(*c, mask) as usize;
            sorted[fill[b] as usize] = i as u32;
            packed[fill[b] as usize] = (positions[i], *c);
            fill[b] += 1;
        }
        let h2 = h * h;
        // Each pair once, as (lower, higher); then grouped by the lower.
        let search = |particles: std::ops::Range<usize>| {
            let mut found: Vec<(u32, u32)> = Vec::with_capacity(particles.len() * 24);
            for i in particles {
                let (p, c) = (positions[i], cells[i]);
                for &[dx, dy, dz] in &FORWARD[..V::FORWARD] {
                    let cell = [c[0] + dx, c[1] + dy, c[2] + dz];
                    let b = bucket_of(cell, mask) as usize;
                    let range = first[b] as usize..first[b + 1] as usize;
                    let own = dx == 0 && dy == 0 && dz == 0;
                    for (&j, (q, cj)) in sorted[range.clone()].iter().zip(&packed[range]) {
                        if *cj != cell || (own && j as usize <= i) {
                            continue;
                        }
                        if (*q - p).length_squared() < h2 {
                            let i = i as u32;
                            found.push(if i < j { (i, j) } else { (j, i) });
                        }
                    }
                }
            }
            found
        };
        let mut parts = crate::lanes::map_ranges(n, V::grain(), search);
        let found = if parts.len() == 1 {
            parts.pop().unwrap_or_default()
        } else {
            parts.concat()
        };
        let mut start = vec![0u32; n + 1];
        for &(i, _) in &found {
            start[i as usize + 1] += 1;
        }
        for k in 0..n {
            start[k + 1] += start[k];
        }
        let mut fill = start.clone();
        let mut pairs = vec![0u32; found.len()];
        for &(i, j) in &found {
            pairs[fill[i as usize] as usize] = j;
            fill[i as usize] += 1;
        }
        Pairs { start, pairs }
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.pairs[self.start[i] as usize..self.start[i + 1] as usize]
    }
}
