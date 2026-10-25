//! Where og-physics's work runs on the lanes (og-lanes, the `physics`
//! site): from how many bodies the step goes to the pool for each thread
//! count, and the grains of the passes og-physics splits itself. Measured
//! with `examples/lanes_costs.rs` and `examples/soft_lanes_costs.rs`
//! (tests/evidence/P4/physics-lanes/, tests/evidence/P4/physics-lanes-2/
//! and tests/evidence/P4/physics-3/).

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
    /// The contact graph's edges of one task when a step on the pool reads
    /// its touching pairs back, and the pairs of one bucket's deal when it
    /// sorts them; they split from two grains, and never below the pool.
    pub pairs: usize,
    /// The bodies of one task when a step brings the rapier world in line
    /// with them before the step (what each body's change asks for) and
    /// reads them back after it (the rest rule, the values written back);
    /// they split from two grains. Measured not to pay to 4,000 bodies on
    /// any thread count (what stays serial, every rapier call in body
    /// order, outweighs it), so `usize::MAX`: never.
    pub bodies: usize,
    /// The rows of one task when the gather decodes a component's rows (an
    /// archetype's bodies' og.RigidBody and og.Collider); they split from
    /// two grains.
    pub rows: usize,
    /// The ragdolls of one task when a step plans their bones' targets
    /// before the step; they split from two grains. Measured not to pay to
    /// 1,024 ragdolls (0.2 ms in all, most of it the rapier calls made in
    /// ragdoll order), so `usize::MAX`: never.
    pub ragdolls: usize,
    /// The work of one task when ropes step side by side, in point passes
    /// (points × iterations); they split from two tasks' worth.
    pub rope_work: usize,
    /// The same for cloths (points × iterations × substeps).
    pub cloth_work: usize,
    /// The same for fluids, 2D and 3D, each alone or its group's set
    /// (particles × substeps).
    pub fluid_work: usize,
    /// The particles of one task in a 2D fluid's per-particle passes (the
    /// neighbour search, the collisions with solids); they split from two
    /// grains.
    pub particles_2d: usize,
    /// The same in a 3D fluid, whose relaxation (serial, Gauss-Seidel over
    /// the pairs) weighs more against the passes that split.
    pub particles_3d: usize,
}

impl Grains {
    /// The measured grains for a lanes pool of `threads` threads (M4 Pro;
    /// tests/evidence/P4/physics-lanes-2/ and tests/evidence/P4/physics-3/,
    /// which measured five and six threads apart).
    pub fn measured(threads: u32) -> Grains {
        Grains {
            pool_bodies: match threads {
                0 | 1 => usize::MAX,
                2..=4 => 500,
                5..=6 => 1000,
                _ => 1500,
            },
            pairs: 2048,
            bodies: usize::MAX,
            rows: 1024,
            ragdolls: usize::MAX,
            // Five threads split ropes from eight as four do, six from 16,
            // seven and up from about 24 (tests/evidence/P4/physics-3/).
            rope_work: match threads {
                0..=5 => 800,
                6 => 1600,
                _ => 2400,
            },
            cloth_work: if threads <= 4 { 1100 } else { 2000 },
            fluid_work: 500,
            particles_2d: 256,
            particles_3d: 512,
        }
    }

    /// Every pass split as soon as it has two items, every step on the
    /// pool: what a test of the parallel paths or a measurement of where
    /// they pay sets.
    pub fn always() -> Grains {
        Grains {
            pool_bodies: 1,
            pairs: 1,
            bodies: 1,
            rows: 1,
            ragdolls: 1,
            rope_work: 1,
            cloth_work: 1,
            fluid_work: 1,
            particles_2d: 1,
            particles_3d: 1,
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

/// The soft bodies that step side by side, each kind with its own
/// [`Gauge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Soft {
    Ropes,
    Cloths,
    Fluids,
    Fluids3d,
}

/// Whether the lanes pool still pays for one kind of soft body on this
/// machine now. The grains were measured on an idle machine; on a loaded
/// one the pool's threads wait to be scheduled while the engine thread
/// waits for them (The Kestrel Stair's five cloths: 0.93 ms a tick
/// serially, 2.9 to 4.2 ms on the pool at a load average of 20 to 25,
/// tests/evidence/P4/perf-games/kestrel-stair.md). A call that went to the
/// pool is measured: when its wall time on the calling thread is no less
/// than its jobs' own times summed (what running them one after the other
/// would have taken), the pool did not pay, and the next calls of that
/// kind run serially, 8 of them after the first loss, twice as many after
/// each further loss up to 512 (about 8.5 s of ticks), and the call after
/// them tries the pool again; a call that pays clears the backoff. Where
/// the work runs never changes the world (og-lanes), so neither does this.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Gauge {
    /// Calls left to run serially.
    serial_left: u32,
    /// How many the last loss sent serial (0: the pool paid last time).
    backoff: u32,
}

impl Gauge {
    const FIRST: u32 = 8;
    const MOST: u32 = 512;

    /// Whether this call may go to the pool (a call kept serial counts down).
    pub(crate) fn admit(&mut self) -> bool {
        if self.serial_left > 0 {
            self.serial_left -= 1;
            false
        } else {
            true
        }
    }

    /// A call on the pool took `wall_ms` on the calling thread for jobs
    /// that took `busy_ms` in all where they ran.
    pub(crate) fn record(&mut self, wall_ms: f64, busy_ms: f64) {
        if wall_ms >= busy_ms {
            self.backoff = self
                .backoff
                .saturating_mul(2)
                .clamp(Gauge::FIRST, Gauge::MOST);
            self.serial_left = self.backoff;
        } else {
            self.backoff = 0;
        }
    }
}

static GAUGES: Mutex<[Gauge; 4]> = Mutex::new(
    [Gauge {
        serial_left: 0,
        backoff: 0,
    }; 4],
);

/// `f` on every soft body job of `kind`, side by side on the lanes when
/// [`soft_grain`] splits them (`work` against `per_task`) and the kind's
/// [`Gauge`] admits the pool, else one after the other on the calling
/// thread. Forced grains ([`set_grains`]) always split as they say.
pub(crate) fn soft_each<T: Send>(
    kind: Soft,
    jobs: &mut [T],
    work: usize,
    per_task: usize,
    f: impl Fn(&mut T) + Sync + Send,
) {
    let site = og_lanes::Site::Physics;
    let grain = soft_grain(jobs.len(), work, per_task);
    let forced = FORCED.lock().unwrap_or_else(|e| e.into_inner()).is_some();
    if grain >= jobs.len() || forced {
        og_lanes::for_each_mut(site, jobs, grain, f);
        return;
    }
    let gauges = || GAUGES.lock().unwrap_or_else(|e| e.into_inner());
    if !gauges()[kind as usize].admit() {
        og_lanes::for_each_mut(site, jobs, jobs.len(), f);
        return;
    }
    let calls = og_lanes::calls_here();
    let busy = Mutex::new(0.0f64);
    let start = crate::stats::now_ms();
    og_lanes::for_each_mut(site, jobs, grain, |job| {
        let at = crate::stats::now_ms();
        f(job);
        let took = crate::stats::now_ms() - at;
        *busy.lock().unwrap_or_else(|e| e.into_inner()) += took;
    });
    let wall = crate::stats::now_ms() - start;
    let busy = busy.into_inner().unwrap_or_else(|e| e.into_inner());
    // Recorded only when the call went to the pool, and with a clock (the
    // wasm without one reads 0 everywhere).
    if og_lanes::calls_here() > calls && busy > 0.0 {
        gauges()[kind as usize].record(wall, busy);
    }
}

/// `0..n` as the ranges of a per-particle pass: `grain` items each from
/// two grains, one range below.
pub(crate) fn ranges(n: usize, grain: usize) -> Vec<std::ops::Range<usize>> {
    let grain = grain.max(1);
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
    grain: usize,
    f: impl Fn(std::ops::Range<usize>) -> R + Sync + Send,
) -> Vec<R> {
    og_lanes::map(og_lanes::Site::Physics, &ranges(n, grain), 1, |r| {
        f(r.clone())
    })
}

/// `f(first index, positions, velocities)` on the chunks of two parallel
/// slices that [`ranges`] cuts, side by side on the lanes when there are
/// several: a per-particle pass that writes only its own particles.
pub(crate) fn for_each_chunk<P: Send, V: Send>(
    positions: &mut [P],
    velocities: &mut [V],
    grain: usize,
    f: impl Fn(usize, &mut [P], &mut [V]) + Sync + Send,
) {
    debug_assert_eq!(positions.len(), velocities.len());
    let n = positions.len();
    let size = ranges(n, grain).first().map_or(1, |r| r.len().max(1));
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
    grain: usize,
    f: impl Fn(usize, &mut [P]) + Sync + Send,
) {
    let size = ranges(items.len(), grain)
        .first()
        .map_or(1, |r| r.len().max(1));
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

#[cfg(test)]
mod tests {
    use super::Gauge;

    #[test]
    fn a_pool_that_does_not_pay_is_left_for_longer_each_time_until_it_pays() {
        let mut g = Gauge::default();
        assert!(g.admit(), "the pool is tried first");
        g.record(1.0, 3.0);
        assert!(g.admit(), "a call that paid keeps the pool");
        // A loss: the next 8 calls run serially, then the pool is tried.
        g.record(3.0, 0.9);
        assert_eq!((0..8).filter(|_| g.admit()).count(), 0);
        assert!(g.admit());
        // A second loss in a row: 16, then 32, ... up to 512.
        g.record(3.0, 0.9);
        assert_eq!((0..16).filter(|_| g.admit()).count(), 0);
        assert!(g.admit());
        for _ in 0..10 {
            g.record(3.0, 0.9);
        }
        assert_eq!((0..512).filter(|_| g.admit()).count(), 0);
        assert!(g.admit());
        // A call that pays clears it: the next loss starts again at 8.
        g.record(0.5, 0.9);
        assert!(g.admit());
        g.record(0.9, 0.9);
        assert_eq!((0..8).filter(|_| g.admit()).count(), 0);
        assert!(g.admit());
    }
}
