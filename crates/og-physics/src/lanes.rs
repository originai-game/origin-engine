//! Where og-physics's work runs on the lanes (og-lanes, the `physics`
//! site): from how many bodies the step goes to the pool for each thread
//! count, and the grains of the passes og-physics splits itself. Measured
//! with `examples/lanes_costs.rs` (tests/evidence/P4/physics-lanes/ and
//! tests/evidence/P4/physics-lanes-2/).

use std::sync::Mutex;

/// What the lanes split by ([`grains`]). The world is the same whatever
/// they are; only where its work runs, and so its time, changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grains {
    /// The bodies from which a step goes to the lanes pool (og-lanes'
    /// `install`): rapier's `parallel` build splits its broad phase, narrow
    /// phase and solver across every pool thread, which pays only once
    /// each has enough bodies, and later with more threads, since rapier's
    /// stages wait for their slowest worker. Below it the step runs on the
    /// calling thread and never wakes the pool. `usize::MAX`: never.
    pub pool_bodies: usize,
    /// The contact pairs of one task when a step on the pool reads them
    /// back.
    pub pairs: usize,
    /// The work of one task when ropes step side by side, in point passes
    /// (points × iterations); they split from two tasks' worth.
    pub rope_work: usize,
    /// The same for cloths (points × iterations × substeps).
    pub cloth_work: usize,
    /// The same for fluids, 2D and 3D, each alone or its group's set
    /// (particles × substeps).
    pub fluid_work: usize,
    /// The particles of one task in a fluid's per-particle passes (the
    /// neighbour search, the collisions with solids); they split from two
    /// grains.
    pub particles: usize,
}

impl Grains {
    /// The measured grains for a lanes pool of `threads` threads (M4 Pro;
    /// tests/evidence/P4/physics-lanes-2/).
    pub fn measured(threads: u32) -> Grains {
        Grains {
            pool_bodies: match threads {
                0 | 1 => usize::MAX,
                2..=4 => 500,
                5..=6 => 1000,
                _ => 1500,
            },
            pairs: 512,
            rope_work: 4096,
            cloth_work: 8192,
            fluid_work: 2048,
            particles: 1024,
        }
    }

    /// Every pass split as soon as it has two items, every step on the
    /// pool: what a test of the parallel paths or a measurement of where
    /// they pay sets.
    pub fn always() -> Grains {
        Grains {
            pool_bodies: 1,
            pairs: 1,
            rope_work: 1,
            cloth_work: 1,
            fluid_work: 1,
            particles: 1,
        }
    }
}

static FORCED: Mutex<Option<Grains>> = Mutex::new(None);

/// The grains in force for a pool of `threads` threads: the measured ones,
/// or those [`set_grains`] forced.
pub fn grains(threads: u32) -> Grains {
    match *FORCED.lock().unwrap_or_else(|e| e.into_inner()) {
        Some(g) => g,
        None => Grains::measured(threads),
    }
}

/// Forces the grains for every thread count (`None`: the measured ones
/// again): `examples/lanes_costs.rs` times the passes split with
/// [`Grains::always`], and the tests put small worlds on the parallel
/// paths with it. The process's, like og-lanes' switch.
pub fn set_grains(grains: Option<Grains>) {
    *FORCED.lock().unwrap_or_else(|e| e.into_inner()) = grains;
}

/// [`Grains::pool_bodies`] in force for `threads` threads.
pub fn pool_bodies(threads: u32) -> usize {
    grains(threads).pool_bodies
}

/// The grains for the pool lanes would use now.
pub(crate) fn now() -> Grains {
    grains(og_lanes::threads())
}

/// The grain (items a task) og-lanes takes for soft bodies of uneven size
/// stepped side by side, each alone: a body a task when their `work`
/// gives two tasks of `per_task`, otherwise every body in one task, so
/// og-lanes runs them serially on the calling thread.
pub(crate) fn soft_grain(items: usize, work: usize, per_task: usize) -> usize {
    if items >= 2 && work >= per_task.saturating_mul(2) {
        1
    } else {
        items.max(1)
    }
}

/// `0..n` as the ranges of a per-particle pass: [`Grains::particles`]
/// items each from two grains, one range below.
pub(crate) fn ranges(n: usize) -> Vec<std::ops::Range<usize>> {
    let grain = now().particles.max(1);
    if n < grain.saturating_mul(2) {
        return std::iter::once(0..n).collect();
    }
    (0..n)
        .step_by(grain)
        .map(|a| a..(a + grain).min(n))
        .collect()
}

/// `f` on every range of [`ranges`]`(n)`, side by side on the lanes when
/// there are several, the results in range order.
pub(crate) fn map_ranges<R: Send>(
    n: usize,
    f: impl Fn(std::ops::Range<usize>) -> R + Sync + Send,
) -> Vec<R> {
    og_lanes::map(og_lanes::Site::Physics, &ranges(n), 1, |r| f(r.clone()))
}

/// `f(first index, positions, velocities)` on the chunks of two parallel
/// slices that [`ranges`] cuts, side by side on the lanes when there are
/// several: a per-particle pass that writes only its own particles.
pub(crate) fn for_each_chunk<P: Send, V: Send>(
    positions: &mut [P],
    velocities: &mut [V],
    f: impl Fn(usize, &mut [P], &mut [V]) + Sync + Send,
) {
    debug_assert_eq!(positions.len(), velocities.len());
    let n = positions.len();
    let size = ranges(n).first().map_or(1, |r| r.len().max(1));
    let mut parts: Vec<(usize, &mut [P], &mut [V])> = positions
        .chunks_mut(size)
        .zip(velocities.chunks_mut(size))
        .enumerate()
        .map(|(k, (p, v))| (k * size, p, v))
        .collect();
    og_lanes::for_each_mut(og_lanes::Site::Physics, &mut parts, 1, |part| {
        let (at, p, v) = part;
        f(*at, p, v)
    });
}

/// [`for_each_chunk`] over one slice.
pub(crate) fn for_each_chunk_of<P: Send>(
    items: &mut [P],
    f: impl Fn(usize, &mut [P]) + Sync + Send,
) {
    let size = ranges(items.len()).first().map_or(1, |r| r.len().max(1));
    let mut parts: Vec<(usize, &mut [P])> = items
        .chunks_mut(size)
        .enumerate()
        .map(|(k, p)| (k * size, p))
        .collect();
    og_lanes::for_each_mut(og_lanes::Site::Physics, &mut parts, 1, |part| {
        let (at, p) = part;
        f(*at, p)
    });
}
