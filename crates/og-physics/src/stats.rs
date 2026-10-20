//! What the last step cost, phase by phase ([`PhysicsStats`],
//! `og.physics.stats`): where physics time goes in a game, and which of it
//! went to the lanes pool. Timing reads a wall clock ([`set_clock`]) and
//! never feeds the simulation: the stats are not world state, not in the
//! state block and not in the world hash.

use std::sync::RwLock;

/// The phases of a step, in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Reading the physics components out of the world.
    Gather,
    /// Bringing rapier's bodies and static colliders in line with them.
    Sync,
    /// Joints and vehicles.
    Joints,
    /// Ragdolls: building their bones, kinematic targets, the limp switch.
    Ragdolls,
    /// Water pushing and floating the bodies in it, before the step.
    Water,
    /// rapier's step (every substep): broad phase, narrow phase, solver.
    Rapier,
    /// The touching pairs and the dynamic bodies read back, the rest rule.
    Readout,
    /// Character and platformer controllers moved against the world.
    Controllers,
    /// Ropes and cloths.
    Soft,
    /// 2D fluids.
    Fluids,
    /// 3D fluids.
    Fluids3d,
    /// Transforms, velocities, controllers' and joints' components and the
    /// events written back to the world.
    WriteBack,
}

impl Phase {
    /// How many phases there are.
    pub const COUNT: usize = 12;
    /// Every phase, in step order.
    pub const ALL: [Phase; Phase::COUNT] = [
        Phase::Gather,
        Phase::Sync,
        Phase::Joints,
        Phase::Ragdolls,
        Phase::Water,
        Phase::Rapier,
        Phase::Readout,
        Phase::Controllers,
        Phase::Soft,
        Phase::Fluids,
        Phase::Fluids3d,
        Phase::WriteBack,
    ];

    /// The phase's name in [`PhaseStats::name`] (camelCase, as the
    /// TypeScript API spells it).
    pub fn name(self) -> &'static str {
        match self {
            Phase::Gather => "gather",
            Phase::Sync => "sync",
            Phase::Joints => "joints",
            Phase::Ragdolls => "ragdolls",
            Phase::Water => "water",
            Phase::Rapier => "rapier",
            Phase::Readout => "readout",
            Phase::Controllers => "controllers",
            Phase::Soft => "soft",
            Phase::Fluids => "fluids",
            Phase::Fluids3d => "fluids3d",
            Phase::WriteBack => "writeBack",
        }
    }
}

/// One phase of the last step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PhaseStats {
    /// [`Phase::name`].
    pub name: &'static str,
    /// Wall-clock milliseconds it took (0 where the host gives no clock).
    pub ms: f64,
    /// Whether any of its work ran on the lanes pool: rapier's step when
    /// the step went to the pool, the other phases when they split by
    /// bodies, pairs, ragdolls, soft bodies or particles.
    pub split: bool,
}

/// What the last step of a world cost ([`crate::PhysicsSystem::stats`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhysicsStats {
    /// The tick the step ran (0 before the first).
    pub tick: u64,
    /// The bodies it stepped (with ragdoll bones and vehicle wheels).
    pub bodies: u32,
    /// The lanes pool's threads at the time (1: lanes off).
    pub threads: u32,
    /// Whether the step ran on the lanes pool (from
    /// [`crate::pool_bodies`] bodies), where rapier splits its stages.
    pub pool: bool,
    /// Wall-clock milliseconds of the whole system, gather to write-back.
    pub ms: f64,
    /// Every phase, in [`Phase::ALL`] order.
    pub phases: Vec<PhaseStats>,
}

impl PhysicsStats {
    /// The stats of `phase`.
    pub fn phase(&self, phase: Phase) -> PhaseStats {
        self.phases
            .get(phase as usize)
            .copied()
            .unwrap_or(PhaseStats {
                name: phase.name(),
                ..PhaseStats::default()
            })
    }
}

static CLOCK: RwLock<Option<fn() -> f64>> = RwLock::new(None);

/// Sets the wall clock (milliseconds, any origin, monotonic on each
/// thread) the stats are timed with. Natively the default is
/// `std::time::Instant`; the wasm has none, so the web host gives
/// `performance.now` (each Worker its own: a phase is timed on one thread).
pub fn set_clock(clock: fn() -> f64) {
    *CLOCK.write().unwrap_or_else(|e| e.into_inner()) = Some(clock);
}

pub(crate) fn now_ms() -> f64 {
    if let Some(clock) = *CLOCK.read().unwrap_or_else(|e| e.into_inner()) {
        return clock();
    }
    native_ms()
}

#[cfg(not(target_arch = "wasm32"))]
fn native_ms() -> f64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

#[cfg(target_arch = "wasm32")]
fn native_ms() -> f64 {
    0.0
}

/// Times the phases of one step on the thread that runs them: each
/// [`Laps::lap`] charges the time and the parallel calls since the last
/// one to a phase.
pub(crate) struct Laps {
    at: f64,
    calls: u64,
    pub(crate) phases: [PhaseStats; Phase::COUNT],
}

impl Laps {
    pub(crate) fn start() -> Laps {
        Laps {
            at: now_ms(),
            calls: og_lanes::calls_here(),
            phases: Phase::ALL.map(|p| PhaseStats {
                name: p.name(),
                ..PhaseStats::default()
            }),
        }
    }

    /// Charges what ran since the last lap to `phase` (added, so a phase
    /// may be timed in pieces).
    pub(crate) fn lap(&mut self, phase: Phase) {
        let (at, calls) = (now_ms(), og_lanes::calls_here());
        let p = &mut self.phases[phase as usize];
        p.ms += (at - self.at).max(0.0);
        p.split |= calls != self.calls;
        self.at = at;
        self.calls = calls;
    }

    /// Starts the next lap now, charging what ran since the last one to no
    /// phase (another thread's laps covered it).
    pub(crate) fn skip(&mut self) {
        self.at = now_ms();
        self.calls = og_lanes::calls_here();
    }

    /// Marks `phase` as split (rapier's own rayon stages, which og-lanes
    /// does not count).
    pub(crate) fn split(&mut self, phase: Phase) {
        self.phases[phase as usize].split = true;
    }

    /// Adds another thread's laps (the step's, timed where it ran).
    pub(crate) fn merge(&mut self, other: &Laps) {
        for (mine, theirs) in self.phases.iter_mut().zip(&other.phases) {
            mine.ms += theirs.ms;
            mine.split |= theirs.split;
        }
    }
}
