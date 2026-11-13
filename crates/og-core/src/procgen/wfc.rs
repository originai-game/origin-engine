//! Wave function collapse on a grid of socketed tiles.

use super::weighted;
use crate::{Error, ErrorKind, Result, Rng};

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
    /// Cells fixed before the first draw: (x, y, tile, rotation).
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

/// Wave function collapse, the simple tiled model: every cell starts as
/// every variant (a tile and a rotation); the cell with the fewest left
/// (ties: the lowest index) collapses to one drawn by weight, and the
/// choice propagates to every neighbour until nothing changes. A cell left
/// with none is a contradiction: the try restarts on a fork of the seed, up
/// to `attempts` tries, and then it is refused naming the cell.
pub fn wfc(desc: &Wfc) -> Result<WfcOut> {
    let (w, h) = (desc.width as usize, desc.height as usize);
    if w == 0 || h == 0 || w * h > 1 << 20 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "WFC: the grid is {} x {}; it must have 1 to 1,048,576 cells",
                desc.width, desc.height
            ),
        ));
    }
    if desc.tiles.is_empty() {
        return Err(Error::new(ErrorKind::Invalid, "WFC: no tiles"));
    }
    // Variants: (tile, rotation, sockets turned, weight).
    let mut variants: Vec<(u32, u32, [u32; 4], f64)> = Vec::new();
    for (i, t) in desc.tiles.iter().enumerate() {
        let turns = if t.rotations { 4 } else { 1 };
        let mut seen: Vec<[u32; 4]> = Vec::new();
        for r in 0..turns {
            // Turned r quarter turns clockwise: the up edge shows what the
            // left edge showed, and so on round.
            let s = t.sockets;
            let turned = [
                s[(4 - r as usize) % 4],
                s[(5 - r as usize) % 4],
                s[(6 - r as usize) % 4],
                s[(7 - r as usize) % 4],
            ];
            if seen.contains(&turned) {
                continue;
            }
            seen.push(turned);
            variants.push((i as u32, r, turned, f64::from(t.weight.max(0.0))));
        }
    }
    let n = variants.len();
    if n > 1024 {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!("WFC: {n} tile variants; at most 1,024"),
        ));
    }
    // compat[d][a] = the variants b may sit in direction d of a.
    let words = n.div_ceil(64);
    let mut compat = vec![vec![vec![0u64; words]; n]; 4];
    for (d, rows) in compat.iter_mut().enumerate() {
        for (a, row) in rows.iter_mut().enumerate() {
            for (b, vb) in variants.iter().enumerate() {
                if variants[a].2[d] == vb.2[(d + 2) % 4] {
                    row[b / 64] |= 1 << (b % 64);
                }
            }
        }
    }
    let mut fixed: Vec<(usize, usize)> = Vec::new();
    for &(x, y, tile, rot) in &desc.fixed {
        if x >= desc.width || y >= desc.height {
            return Err(Error::new(
                ErrorKind::Invalid,
                format!("WFC: fixed cell ({x}, {y}) is outside the grid"),
            ));
        }
        let v = variants
            .iter()
            .position(|v| v.0 == tile && v.1 == rot % 4)
            .or_else(|| variants.iter().position(|v| v.0 == tile))
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Invalid,
                    format!("WFC: fixed cell ({x}, {y}) names tile {tile}, past the last"),
                )
            })?;
        fixed.push((y as usize * w + x as usize, v));
    }
    let root = Rng::new(desc.seed);
    let mut last = (0, 0);
    for attempt in 1..=desc.attempts.max(1) {
        let mut rng = if attempt == 1 {
            root
        } else {
            root.fork(&format!("attempt-{attempt}"))
        };
        match collapse(&mut rng, w, h, desc.wrap, &variants, &compat, &fixed) {
            Ok(cells) => {
                return Ok(WfcOut {
                    tiles: cells.iter().map(|&v| variants[v].0).collect(),
                    rotations: cells.iter().map(|&v| variants[v].1).collect(),
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

/// One try; `Err(cell)` names the cell left with no variant.
fn collapse(
    rng: &mut Rng,
    w: usize,
    h: usize,
    wrap: bool,
    variants: &[(u32, u32, [u32; 4], f64)],
    compat: &[Vec<Vec<u64>>],
    fixed: &[(usize, usize)],
) -> std::result::Result<Vec<usize>, usize> {
    let n = variants.len();
    let words = n.div_ceil(64);
    let mut full = vec![!0u64; words];
    if !n.is_multiple_of(64) {
        full[words - 1] = (1u64 << (n % 64)) - 1;
    }
    let cells = w * h;
    let mut wave: Vec<u64> = Vec::with_capacity(cells * words);
    for _ in 0..cells {
        wave.extend_from_slice(&full);
    }
    let count = |wave: &[u64], c: usize| -> u32 {
        wave[c * words..(c + 1) * words]
            .iter()
            .map(|x| x.count_ones())
            .sum()
    };
    let neighbour = |c: usize, d: usize| -> Option<usize> {
        let (x, y) = ((c % w) as i64, (c / w) as i64);
        let (dx, dy) = [(0, -1), (1, 0), (0, 1), (-1, 0)][d];
        let (mut nx, mut ny) = (x + dx, y + dy);
        if wrap {
            nx = nx.rem_euclid(w as i64);
            ny = ny.rem_euclid(h as i64);
        } else if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
            return None;
        }
        Some(ny as usize * w + nx as usize)
    };
    let mut stack: Vec<usize> = Vec::new();
    let mut allowed = vec![0u64; words];
    let propagate = |wave: &mut Vec<u64>,
                     stack: &mut Vec<usize>,
                     allowed: &mut Vec<u64>|
     -> std::result::Result<(), usize> {
        while let Some(c) = stack.pop() {
            for (d, cd) in compat.iter().enumerate() {
                let Some(nb) = neighbour(c, d) else { continue };
                allowed.fill(0);
                for v in 0..n {
                    if wave[c * words + v / 64] >> (v % 64) & 1 == 1 {
                        for (a, b) in allowed.iter_mut().zip(&cd[v]) {
                            *a |= *b;
                        }
                    }
                }
                let mut changed = false;
                let mut any = false;
                for k in 0..words {
                    let old = wave[nb * words + k];
                    let new = old & allowed[k];
                    if new != old {
                        changed = true;
                    }
                    any |= new != 0;
                    wave[nb * words + k] = new;
                }
                if !any {
                    return Err(nb);
                }
                if changed {
                    stack.push(nb);
                }
            }
        }
        Ok(())
    };
    // Every cell's constraints on its neighbours apply before the first
    // draw: with a single variant every cell starts decided.
    stack.extend((0..cells).rev());
    for &(c, v) in fixed {
        for k in 0..words {
            wave[c * words + k] = 0;
        }
        wave[c * words + v / 64] |= 1 << (v % 64);
        stack.push(c);
    }
    propagate(&mut wave, &mut stack, &mut allowed)?;
    let mut weights = vec![0f64; n];
    loop {
        // The undecided cell with the fewest variants left, lowest index first.
        let mut best: Option<(u32, usize)> = None;
        for c in 0..cells {
            let k = count(&wave, c);
            if k > 1 && best.is_none_or(|b| k < b.0) {
                best = Some((k, c));
            }
        }
        let Some((_, c)) = best else { break };
        for (v, wt) in weights.iter_mut().enumerate() {
            *wt = if wave[c * words + v / 64] >> (v % 64) & 1 == 1 {
                // A zero weight still stands in when nothing else is left.
                variants[v].3.max(1e-9)
            } else {
                0.0
            };
        }
        let v = weighted(rng, &weights).ok_or(c)?;
        for k in 0..words {
            wave[c * words + k] = 0;
        }
        wave[c * words + v / 64] |= 1 << (v % 64);
        stack.push(c);
        propagate(&mut wave, &mut stack, &mut allowed)?;
    }
    Ok((0..cells)
        .map(|c| {
            (0..n)
                .find(|&v| wave[c * words + v / 64] >> (v % 64) & 1 == 1)
                .expect("a decided cell has one variant")
        })
        .collect())
}
