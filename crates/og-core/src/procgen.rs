//! Procedural generation on the engine's seeded [`Rng`]: weighted picks and
//! permutations here, Poisson-disk points, BSP rooms and wave function
//! collapse in their files. Gradient noise is [`crate::noise`].
//!
//! Determinism: every generator takes a seed (idiomatically
//! `world.resource(og.Rng)`'s value `.fork("label").seed`) and draws only
//! from an [`Rng`] on it, in a fixed order, so the main stream never moves
//! and the same seed gives the same output on every host. The arithmetic is `+ - * /`, `floor` and
//! `sqrt` on f32 (correctly rounded on wasm and native alike), never a
//! transcendental function, and every loop walks a fixed order; ties are
//! broken by index.

use crate::Rng;

mod bsp;
mod poisson;
#[cfg(test)]
mod tests;
mod wfc;

pub use bsp::{BspDesc, BspLayout, CellRect, bsp_rooms};
pub use poisson::poisson_disk;
pub use wfc::{Wfc, WfcOut, WfcTile, wfc};

/// The index of a weight drawn in proportion to it (one draw), among the
/// positive finite weights; `None` when there is none.
pub fn weighted(rng: &mut Rng, weights: &[f64]) -> Option<usize> {
    let ok = |w: f64| w.is_finite() && w > 0.0;
    let total: f64 = weights.iter().copied().filter(|w| ok(*w)).sum();
    if total <= 0.0 {
        return None;
    }
    let mut at = rng.next_f64() * total;
    let mut last = None;
    for (i, &w) in weights.iter().enumerate() {
        if !ok(w) {
            continue;
        }
        last = Some(i);
        if at < w {
            return Some(i);
        }
        at -= w;
    }
    // Rounding left a sliver past the last weight: it is the last one's.
    last
}

/// `0..n` shuffled (Fisher-Yates from the end, `n - 1` draws): apply it to
/// a list to shuffle the list.
pub fn permutation(rng: &mut Rng, n: u32) -> Vec<u32> {
    let mut p: Vec<u32> = (0..n).collect();
    for i in (1..n as usize).rev() {
        let j = rng.integer(0, i as i64) as usize;
        p.swap(i, j);
    }
    p
}
