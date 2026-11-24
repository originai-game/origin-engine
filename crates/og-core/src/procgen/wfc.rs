//! Wave function collapse on a grid of socketed tiles: incremental AC-4
//! constraint propagation, binary min-heap Shannon entropy selection with
//! deterministic tie-breaking, and bounded backtracking.

use super::weighted;
use crate::{Error, ErrorKind, Result, Rng, math};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A tile of a [`Wfc`]: its weight and the socket on each edge (up, right,
/// down, left); two tiles may sit side by side when the sockets they turn
/// to each other are equal. With `rotations` it also comes turned 90°,
/// 180° and 270° clockwise (its sockets turned with it).
#[derive(Clone, Debug, PartialEq)]
pub struct WfcTile {
    pub weight: f32,
    pub sockets: [u32; 4],
    pub rotations: bool,
}

/// What [`wfc`] is asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct Wfc {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<WfcTile>,
    /// Cells fixed before the first draw: `(x, y, tile, quarter_turns)`.
    pub fixed: Vec<(u32, u32, u32, u32)>,
    /// Edges wrap around (a tiling texture).
    pub wrap: bool,
    /// Tries before refusing, each on the seed's fork `attempt-<n>`.
    pub attempts: u32,
}

/// What [`wfc`] made: per cell (row-major from the top) its tile and rotation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WfcOut {
    pub tiles: Vec<u32>,
    pub rotations: Vec<u32>,
    /// The try that succeeded, from 1.
    pub attempt: u32,
}

#[derive(Clone, Copy)]
struct Variant {
    tile: u32,
    rot: u32,
    sockets: [u32; 4],
    weight: f64,
    w_log_w: f64,
}

/// Deterministic min-heap entry ordered by `(entropy_bits, cell_index)`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct HeapEntry {
    entropy_bits: u64,
    cell: usize,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for BinaryHeap (max-heap -> min-heap).
        other
            .entropy_bits
            .cmp(&self.entropy_bits)
            .then_with(|| other.cell.cmp(&self.cell))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[inline]
fn f64_order_bits(x: f64) -> u64 {
    let b = x.to_bits();
    if (b >> 63) != 0 { !b } else { b ^ (1u64 << 63) }
}

#[inline]
fn rotate_sockets(s: [u32; 4], r: u32) -> [u32; 4] {
    let r = (r & 3) as usize;
    [
        s[(4 - r) & 3],
        s[(5 - r) & 3],
        s[(6 - r) & 3],
        s[(7 - r) & 3],
    ]
}

/// Wave function collapse, the simple tiled model: every cell starts as every
/// variant (a tile and a unique rotation); the cell with lowest Shannon
/// entropy (ties broken deterministically by per-cell noise) collapses to one
/// drawn by weight, and the choice propagates incrementally via AC-4 support
/// counts. Contradictions backtrack before restarting on a fork of the seed,
/// up to `attempts` tries.
pub fn wfc(desc: &Wfc) -> Result<WfcOut> {
    let cells_u64 = u64::from(desc.width).saturating_mul(u64::from(desc.height));
    if cells_u64 == 0 || cells_u64 > (1 << 20) {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "WFC: the grid is {} x {}; it must have 1 to 1,048,576 cells",
                desc.width, desc.height
            ),
        ));
    }
    let (w, h) = (desc.width as usize, desc.height as usize);
    if desc.tiles.is_empty() {
        return Err(Error::new(ErrorKind::Invalid, "WFC: no tiles"));
    }

    // Build deduplicated variants per tile while preserving total tile weight
    // across symmetric rotations and mapping every (tile, rot) to its exact
    // rotated socket variant.
    let mut variants: Vec<Variant> = Vec::new();
    let mut tile_rot_map: Vec<[Option<usize>; 4]> = vec![[None; 4]; desc.tiles.len()];
    for (i, t) in desc.tiles.iter().enumerate() {
        let turns = if t.rotations { 4 } else { 1 };
        let raw_w = if t.weight.is_finite() && t.weight > 0.0 {
            f64::from(t.weight)
        } else {
            0.0
        };
        let mut unique: Vec<(u32, [u32; 4])> = Vec::new();
        for r in 0..turns {
            let turned = rotate_sockets(t.sockets, r);
            if !unique.iter().any(|&(_, s)| s == turned) {
                unique.push((r, turned));
            }
        }
        let share = if unique.is_empty() {
            0.0
        } else {
            raw_w / unique.len() as f64
        };
        let w_log_w = if share > 0.0 {
            share * f64::from(math::ln(share as f32))
        } else {
            0.0
        };
        let base = variants.len();
        for (r, turned) in &unique {
            variants.push(Variant {
                tile: i as u32,
                rot: *r,
                sockets: *turned,
                weight: share,
                w_log_w,
            });
        }
        for r in 0..turns {
            let turned = rotate_sockets(t.sockets, r);
            if let Some(off) = unique.iter().position(|&(_, s)| s == turned) {
                tile_rot_map[i][r as usize] = Some(base + off);
            }
        }
    }

    let n = variants.len();
    if n > 1024 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("WFC: {n} tile variants; at most 1,024"),
        ));
    }

    // Precompute compatibility lists: compat[d][a] = all variants b that may
    // sit in direction d (0=up, 1=right, 2=down, 3=left) of variant a.
    let mut compat: [Vec<Vec<u16>>; 4] = [
        vec![Vec::new(); n],
        vec![Vec::new(); n],
        vec![Vec::new(); n],
        vec![Vec::new(); n],
    ];
    for d in 0..4 {
        let opp = (d + 2) & 3;
        for a in 0..n {
            for b in 0..n {
                if variants[a].sockets[d] == variants[b].sockets[opp] {
                    compat[d][a].push(b as u16);
                }
            }
        }
    }

    // Validate fixed cells and map each to its exact socket-matched variant.
    let mut fixed: Vec<(usize, usize, u32)> = Vec::new();
    for &(x, y, tile, rot) in &desc.fixed {
        if x >= desc.width || y >= desc.height {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("WFC: fixed cell ({x}, {y}) is outside the grid"),
            ));
        }
        let ti = tile as usize;
        if ti >= desc.tiles.len() {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("WFC: fixed cell ({x}, {y}) names tile {tile}, past the last"),
            ));
        }
        let r_mod = rot & 3;
        let v = tile_rot_map[ti][r_mod as usize]
            .or(tile_rot_map[ti][0])
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    format!("WFC: fixed cell ({x}, {y}) names tile {tile}, past the last"),
                )
            })?;
        fixed.push((y as usize * w + x as usize, v, r_mod));
    }

    let root = Rng::new(desc.seed);
    let mut last = (0, 0);
    for attempt in 1..=desc.attempts.max(1) {
        let mut rng = if attempt == 1 {
            root
        } else {
            root.fork(&format!("attempt-{attempt}"))
        };
        match solve(&mut rng, w, h, desc.wrap, &variants, &compat, &fixed) {
            Ok(cells) => {
                let mut tiles = Vec::with_capacity(cells.len());
                let mut rotations = Vec::with_capacity(cells.len());
                for &v in &cells {
                    tiles.push(variants[v].tile);
                    rotations.push(variants[v].rot);
                }
                // Preserve exact requested quarter turns on symmetric fixed cells.
                for &(idx, _, rot) in &fixed {
                    if desc.tiles[tiles[idx] as usize].rotations {
                        rotations[idx] = rot;
                    }
                }
                return Ok(WfcOut {
                    tiles,
                    rotations,
                    attempt,
                });
            }
            Err(cell) => last = (cell % w, cell / w),
        }
    }
    Err(Error::new(
        ErrorKind::Invalid,
        format!(
            "WFC: no tiling in {} tries; the last ran out of tiles at cell ({}, {}): the sockets may not fit",
            desc.attempts.max(1),
            last.0,
            last.1
        ),
    ))
}

#[inline]
fn neighbour(c: usize, d: usize, w: usize, h: usize, wrap: bool) -> Option<usize> {
    let x = c % w;
    let y = c / w;
    match d {
        0 => {
            if y > 0 {
                Some(c - w)
            } else if wrap {
                Some((h - 1) * w + x)
            } else {
                None
            }
        }
        1 => {
            if x + 1 < w {
                Some(c + 1)
            } else if wrap {
                Some(y * w)
            } else {
                None
            }
        }
        2 => {
            if y + 1 < h {
                Some(c + w)
            } else if wrap {
                Some(x)
            } else {
                None
            }
        }
        _ => {
            if x > 0 {
                Some(c - 1)
            } else if wrap {
                Some(y * w + (w - 1))
            } else {
                None
            }
        }
    }
}

struct Solver<'a> {
    w: usize,
    h: usize,
    wrap: bool,
    n: usize,
    words: usize,
    variants: &'a [Variant],
    compat: &'a [Vec<Vec<u16>>; 4],
    /// Bitset of remaining variants per cell (`cells * words`).
    wave: Vec<u64>,
    /// AC-4 support counter: `support[(cell * n + v) * 4 + d]` = remaining
    /// variants in neighbour `d` compatible with variant `v` at `cell`.
    support: Vec<u16>,
    count: Vec<u16>,
    pos_count: Vec<u16>,
    sum_w: Vec<f64>,
    sum_w_log_w: Vec<f64>,
    tie: Vec<f64>,
    stack: Vec<(usize, usize)>,
    heap: BinaryHeap<HeapEntry>,
    /// Trail of banished `(cell, variant)` pairs for fast O(delta) backtracking.
    trail: Vec<(usize, usize)>,
    /// Trail of `(cell, variant)` pairs whose neighbour AC-4 support counters
    /// were decremented, ensuring exact 1-to-1 restoration on undo.
    prop_trail: Vec<(usize, usize)>,
    /// Scratch buffer for zero-support candidates discovered during propagation.
    pending: Vec<(usize, usize)>,
}

impl<'a> Solver<'a> {
    fn new(
        rng: &mut Rng,
        w: usize,
        h: usize,
        wrap: bool,
        variants: &'a [Variant],
        compat: &'a [Vec<Vec<u16>>; 4],
    ) -> Self {
        let cells = w * h;
        let n = variants.len();
        let words = n.div_ceil(64);
        let mut full = vec![!0u64; words];
        if !n.is_multiple_of(64) {
            full[words - 1] = (1u64 << (n % 64)) - 1;
        }
        let mut wave = Vec::with_capacity(cells * words);
        for _ in 0..cells {
            wave.extend_from_slice(&full);
        }

        // Initial AC-4 support counts: every cell starts with all variants.
        let mut base_sup = vec![0u16; n * 4];
        for v in 0..n {
            for d in 0..4 {
                base_sup[v * 4 + d] = compat[d][v].len() as u16;
            }
        }
        let mut support = Vec::with_capacity(cells * n * 4);
        for _ in 0..cells {
            support.extend_from_slice(&base_sup);
        }

        let total_w: f64 = variants.iter().map(|v| v.weight).sum();
        let total_w_log_w: f64 = variants.iter().map(|v| v.w_log_w).sum();
        let total_pos = variants.iter().filter(|v| v.weight > 0.0).count() as u16;

        let mut tie = Vec::with_capacity(cells);
        for _ in 0..cells {
            tie.push(rng.next_f64() * 1e-6);
        }

        let mut s = Self {
            w,
            h,
            wrap,
            n,
            words,
            variants,
            compat,
            wave,
            support,
            count: vec![n as u16; cells],
            pos_count: vec![total_pos; cells],
            sum_w: vec![total_w; cells],
            sum_w_log_w: vec![total_w_log_w; cells],
            tie,
            stack: Vec::new(),
            heap: BinaryHeap::with_capacity(cells),
            trail: Vec::new(),
            prop_trail: Vec::new(),
            pending: Vec::new(),
        };

        // Push initial entropy for every undecided cell.
        if n > 1 {
            for c in 0..cells {
                let e = s.entropy(c);
                s.heap.push(HeapEntry {
                    entropy_bits: f64_order_bits(e),
                    cell: c,
                });
            }
        }
        s
    }

    #[inline]
    fn has(&self, c: usize, v: usize) -> bool {
        (self.wave[c * self.words + (v >> 6)] >> (v & 63)) & 1 == 1
    }

    #[inline]
    fn entropy(&self, c: usize) -> f64 {
        let k = self.count[c];
        if k <= 1 {
            return -1.0;
        }
        let sw = self.sum_w[c];
        let h = if sw > 0.0 && self.pos_count[c] > 0 {
            f64::from(math::ln(sw as f32)) - self.sum_w_log_w[c] / sw
        } else {
            // All remaining variants have weight 0: fall back to log(count).
            f64::from(math::ln(f32::from(k)))
        };
        h + self.tie[c]
    }

    /// Banish variant `v` from cell `c`, recording the change on `trail`.
    fn banish(&mut self, c: usize, v: usize) -> std::result::Result<(), usize> {
        let word = &mut self.wave[c * self.words + (v >> 6)];
        let bit = 1u64 << (v & 63);
        if (*word & bit) == 0 {
            return Ok(());
        }
        *word &= !bit;
        self.count[c] -= 1;
        let var = self.variants[v];
        if var.weight > 0.0 {
            self.pos_count[c] -= 1;
            self.sum_w[c] = (self.sum_w[c] - var.weight).max(0.0);
            self.sum_w_log_w[c] -= var.w_log_w;
        }
        self.trail.push((c, v));
        self.stack.push((c, v));
        if self.count[c] == 0 {
            return Err(c);
        }
        if self.count[c] > 1 {
            let e = self.entropy(c);
            self.heap.push(HeapEntry {
                entropy_bits: f64_order_bits(e),
                cell: c,
            });
        }
        Ok(())
    }

    fn propagate(&mut self) -> std::result::Result<(), usize> {
        while let Some((c, v)) = self.stack.pop() {
            self.pending.clear();
            for d in 0..4 {
                let Some(nb) = neighbour(c, d, self.w, self.h, self.wrap) else {
                    continue;
                };
                let opp = (d + 2) & 3;
                let nb_base = nb * self.n * 4 + opp;
                for &u_u16 in &self.compat[d][v] {
                    let u = u_u16 as usize;
                    let idx = nb_base + u * 4;
                    let sup = &mut self.support[idx];
                    *sup -= 1;
                    if *sup == 0 && (self.wave[nb * self.words + (u >> 6)] >> (u & 63)) & 1 == 1 {
                        self.pending.push((nb, u));
                    }
                }
            }
            // Record all 4 directions as decremented before any banish may error.
            self.prop_trail.push((c, v));
            for i in 0..self.pending.len() {
                let (nb, u) = self.pending[i];
                self.banish(nb, u)?;
            }
        }
        Ok(())
    }

    /// Undo all banishments and AC-4 decrements back to `(mark, prop_mark)`.
    fn undo_to(&mut self, mark: usize, prop_mark: usize) {
        self.stack.clear();
        self.pending.clear();
        while self.prop_trail.len() > prop_mark {
            let (c, v) = self.prop_trail.pop().unwrap();
            for d in 0..4 {
                let Some(nb) = neighbour(c, d, self.w, self.h, self.wrap) else {
                    continue;
                };
                let opp = (d + 2) & 3;
                let nb_base = nb * self.n * 4 + opp;
                for &u_u16 in &self.compat[d][v] {
                    let u = u_u16 as usize;
                    self.support[nb_base + u * 4] += 1;
                }
            }
        }
        while self.trail.len() > mark {
            let (c, v) = self.trail.pop().unwrap();
            let word = &mut self.wave[c * self.words + (v >> 6)];
            *word |= 1u64 << (v & 63);
            self.count[c] += 1;
            let var = self.variants[v];
            if var.weight > 0.0 {
                self.pos_count[c] += 1;
                self.sum_w[c] += var.weight;
                self.sum_w_log_w[c] += var.w_log_w;
            }
            if self.count[c] > 1 {
                let e = self.entropy(c);
                self.heap.push(HeapEntry {
                    entropy_bits: f64_order_bits(e),
                    cell: c,
                });
            }
        }
    }

    fn next_cell(&mut self) -> Option<usize> {
        while let Some(top) = self.heap.pop() {
            let c = top.cell;
            if self.count[c] <= 1 {
                continue;
            }
            let current = f64_order_bits(self.entropy(c));
            if current == top.entropy_bits {
                return Some(c);
            }
            // Stale entry (entropy changed due to undo or update); re-push once.
            self.heap.push(HeapEntry {
                entropy_bits: current,
                cell: c,
            });
        }
        // Fallback linear scan in case a cell's heap entry was evicted across backtracks.
        let mut best: Option<(u64, usize)> = None;
        for c in 0..(self.w * self.h) {
            if self.count[c] > 1 {
                let bits = f64_order_bits(self.entropy(c));
                if best.is_none_or(|b| (bits, c) < b) {
                    best = Some((bits, c));
                }
            }
        }
        best.map(|(_, c)| c)
    }

    fn collapse_to(&mut self, c: usize, chosen: usize) -> std::result::Result<(), usize> {
        for v in 0..self.n {
            if v != chosen && self.has(c, v) {
                self.banish(c, v)?;
            }
        }
        self.propagate()
    }
}

/// One solve attempt with bounded chronological backtracking.
fn solve(
    rng: &mut Rng,
    w: usize,
    h: usize,
    wrap: bool,
    variants: &[Variant],
    compat: &[Vec<Vec<u16>>; 4],
    fixed: &[(usize, usize, u32)],
) -> std::result::Result<Vec<usize>, usize> {
    let mut s = Solver::new(rng, w, h, wrap, variants, compat);
    let cells = w * h;
    let n = variants.len();

    // Eliminate variants that have zero compatibility in any present neighbour direction.
    for c in 0..cells {
        for v in 0..n {
            if !s.has(c, v) {
                continue;
            }
            let mut dead = false;
            for d in 0..4 {
                if neighbour(c, d, w, h, wrap).is_some() && s.support[(c * n + v) * 4 + d] == 0 {
                    dead = true;
                    break;
                }
            }
            if dead {
                s.banish(c, v)?;
            }
        }
    }

    // Apply fixed cells before the first draw.
    for &(c, v, _) in fixed {
        if !s.has(c, v) {
            return Err(c);
        }
        for other in 0..n {
            if other != v && s.has(c, other) {
                s.banish(c, other)?;
            }
        }
    }
    s.propagate()?;

    // Decision stack for bounded backtracking: (trail_mark, prop_mark, cell, tried_variant).
    const MAX_BACKTRACKS: usize = 64;
    let mut backtracks = 0usize;
    let mut decisions: Vec<(usize, usize, usize, usize)> = Vec::new();
    let mut weights = vec![0.0f64; n];

    while let Some(c) = s.next_cell() {
        let has_pos = s.pos_count[c] > 0;
        for (v, wt) in weights.iter_mut().enumerate() {
            *wt = if s.has(c, v) {
                if has_pos { variants[v].weight } else { 1.0 }
            } else {
                0.0
            };
        }
        let chosen = weighted(rng, &weights).ok_or(c)?;
        let mark = s.trail.len();
        let prop_mark = s.prop_trail.len();
        match s.collapse_to(c, chosen) {
            Ok(()) => {
                decisions.push((mark, prop_mark, c, chosen));
            }
            Err(conflict) => {
                // Undo the failed collapse and banish `chosen` from `c`; if that
                // also contradicts, walk back up the decision stack.
                let mut err_cell = conflict;
                s.undo_to(mark, prop_mark);
                let mut cur = Some((c, chosen));
                loop {
                    if backtracks >= MAX_BACKTRACKS {
                        return Err(err_cell);
                    }
                    backtracks += 1;
                    if let Some((bc, bv)) = cur.take()
                        && s.banish(bc, bv).and_then(|()| s.propagate()).is_ok()
                    {
                        break;
                    }
                    let Some((prev_mark, prev_prop, prev_c, prev_v)) = decisions.pop() else {
                        return Err(err_cell);
                    };
                    err_cell = prev_c;
                    s.undo_to(prev_mark, prev_prop);
                    cur = Some((prev_c, prev_v));
                }
            }
        }
    }

    let mut result = Vec::with_capacity(cells);
    for c in 0..cells {
        let v = (0..n)
            .find(|&v| s.has(c, v))
            .expect("a decided cell has one variant");
        result.push(v);
    }
    Ok(result)
}
