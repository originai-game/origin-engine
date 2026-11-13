//! Particles: one emitter description ([`ParticleEmitter`], an asset added
//! with [`crate::RenderScene::add_emitter`]) and per-frame systems
//! ([`ParticleSystem`]) that place it in the world with a seed and a clock.
//!
//! **Deterministic, visual only.** A system's particles are a function of
//! its emitter, seed, start tick, transform and the ticks it was rendered
//! at: every tick is simulated once with a fixed step (`1 / tick_rate`), the
//! particles spawned in a tick and their random values come from the spawn
//! schedule and a hash of (seed, particle index), never from the frame rate
//! or a clock. A renderer that skipped frames catches up tick by tick; a
//! jump back in time or further than the emitter's longest lifetime
//! replays that lifetime from empty (which is exact while the emitter
//! does not move: older particles are dead either way). Particles never
//! feed back into the simulation; nothing reads them but the renderer.
//!
//! **Two paths, one description.** By default the simulation runs in a
//! compute shader (`shaders/particles_sim.wgsl`) into a storage buffer the
//! draw reads as instance data, with nothing read back: it is visual only.
//! Colliding particles meet the depth prepass or, explicitly
//! ([`CollisionTarget::Colliders`], and always in sprite mode), the scene's
//! colliders through `shaders/scene_colliders.wgsl`. With
//! [`ParticleSimulation::Cpu`], [`CpuParticles`] runs the same steps in
//! Rust and uploads the state. Sprite-mode emitters
//! ([`ParticleRender::Sprite`]) simulate in compute too and draw through
//! the sprite batch; one with a texture array ([`ParticleEmitter::array`])
//! draws each particle from a layer of it, the layers being the sheet's
//! frames.
//!
//! **Events and sub-emitters.** The CPU path reports what happens to its
//! particles ([`ParticleEvent`]: births, deaths, collisions) and spawns
//! each emitter's [`SubEmitter`]s at them. Its arithmetic is
//! deterministic on every host (og-core's transcendentals, never the host
//! libm), so a host can run the same steps in its tick and read the events
//! as gameplay ([`crate::RenderScene::hosted_particles`] then hands the
//! renderer the tick's particles to draw, so what is drawn is what the
//! game read). A system whose emitter has sub-emitters simulates on the
//! CPU; its children draw with their own emitter's look.

use crate::scene::MeshData;
use glam::{Mat4, Vec3, Vec4};
// The CPU path's transcendentals: bit-identical on every host, so a host's
// tick can run it (ParticleEvent, sub-emitters).
use og_core::math;

mod cpu;
pub(crate) mod effect;
mod noise;
pub(crate) mod particles_gpu;
pub(crate) mod ribbons;
pub(crate) mod ribbons_gpu;
pub(crate) mod scene_colliders;
mod schedule;
pub use cpu::*;
pub(crate) use noise::*;
pub use og_render_data::particles::*;
pub(crate) use schedule::*;

/// One running emitter this frame. The renderer keeps its particles by
/// `id` between frames (a system absent from a frame is dropped).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSystem {
    /// Stable identity (the entity's id): state is kept per id.
    pub id: u64,
    pub emitter: EmitterHandle,
    /// The emitter's local to world transform.
    pub transform: Mat4,
    pub seed: u64,
    /// The tick emission started.
    pub start_tick: u64,
    /// Emission stops at this tick (the particles live out); `None` keeps
    /// emitting while the emitter's cycle does.
    pub stop_tick: Option<u64>,
    /// The system's clock: the game's tick (hold it still for hit-stop).
    pub tick: u64,
    /// How fast its particles' time runs against the tick: 1 normal, 0.25 a
    /// quarter speed, 0 frozen (og.TimeGroup's group scale). Spawning,
    /// ageing and motion all run on it; the tick still says how far to step.
    pub time_scale: f32,
    /// Where the air moves at the system's place and how fast (world,
    /// m/s): og.Wind and the og.WindMotors reaching it. An emitter with
    /// `wind` above 0 is carried by it ([`ParticleEmitter::wind`]).
    pub air: Vec3,
}

impl ParticleSystem {
    pub fn new(id: u64, emitter: EmitterHandle, transform: Mat4, seed: u64, tick: u64) -> Self {
        ParticleSystem {
            id,
            emitter,
            transform,
            seed,
            start_tick: 0,
            stop_tick: None,
            tick,
            time_scale: 1.0,
            air: Vec3::ZERO,
        }
    }
}

/// A system's time against the tick, piecewise: from `tick` on, its time
/// runs from `time` seconds at `scale` (re-anchored where the scale
/// changes). At scale 1 from the start tick it is the tick's own time,
/// computed as before, so an unscaled system steps bit for bit as it did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleClock {
    pub tick: u64,
    pub time: f64,
    pub scale: f32,
}

impl ParticleClock {
    /// Seconds of the system's time at the start of `tick`.
    fn at(&self, tick: u64, dt: f64) -> f64 {
        self.time + tick.saturating_sub(self.tick) as f64 * (dt * f64::from(self.scale))
    }
}

/// One tick of a particle system as that tick left it: what its spawns and
/// steps on that tick read ([`ParticleHistory`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleTick {
    pub tick: u64,
    pub transform: Mat4,
    pub start_tick: u64,
    pub stop_tick: Option<u64>,
    pub time_scale: f32,
    /// The system's clock as stepping one tick at a time leaves it.
    pub clock: ParticleClock,
}

/// A particle system's ticks, as many as its live particles can come from
/// (one lifetime at its time scale), recorded as each tick ends whether or
/// not a frame was drawn ([`RenderScene::particle_history`]). The renderer
/// steps a system through the ticks it has not drawn from these, not from
/// the frame's system alone, so its particles are a function of the ticks:
/// the same after a frame on every tick, after ticks stepped without frames
/// and after a restore that brought the history back (docs/design.md §2.4,
/// presentation). Presentation state: never the world's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleHistory {
    ticks: std::collections::VecDeque<ParticleTick>,
}

impl ParticleHistory {
    /// Records `sys` at its tick: its clock follows the tick before's as a
    /// tick-by-tick step would (re-anchored where the scale changes, from
    /// the start tick after a gap), and ticks no live particle can come
    /// from any more are dropped. The same tick again replaces it (a held
    /// clock).
    pub fn record(&mut self, e: &ParticleEmitter, sys: &ParticleSystem, tick_rate: f32) {
        let scale = time_scale(sys);
        let dt = 1.0 / tick_rate.max(1e-3) as f64;
        // The same tick again (a held clock), or ticks a rewind took back
        // (og.net's rollback): recorded again from the tick before.
        while self.ticks.back().is_some_and(|t| t.tick >= sys.tick) {
            self.ticks.pop_back();
        }
        let clock = match self.ticks.back() {
            Some(t) if t.tick + 1 == sys.tick => next_clock(t.clock, sys.tick, scale, dt),
            _ => {
                self.ticks.clear();
                ParticleClock {
                    tick: sys.start_tick,
                    time: 0.0,
                    scale,
                }
            }
        };
        self.ticks.push_back(ParticleTick {
            tick: sys.tick,
            transform: sys.transform,
            start_tick: sys.start_tick,
            stop_tick: sys.stop_tick,
            time_scale: scale,
            clock,
        });
        let window = window(e, scale, tick_rate);
        while self
            .ticks
            .front()
            .is_some_and(|t| t.tick + window <= sys.tick)
        {
            self.ticks.pop_front();
        }
    }

    /// Tick `tick`'s record.
    pub fn at(&self, tick: u64) -> Option<&ParticleTick> {
        let first = self.ticks.front()?.tick;
        self.ticks
            .get(usize::try_from(tick.checked_sub(first)?).ok()?)
            .filter(|t| t.tick == tick)
    }

    /// The ticks recorded, oldest first.
    pub fn ticks(&self) -> impl Iterator<Item = &ParticleTick> {
        self.ticks.iter()
    }
}

/// `sys`'s time scale (non-finite as 1, negative as 0).
fn time_scale(sys: &ParticleSystem) -> f32 {
    if sys.time_scale.is_finite() {
        sys.time_scale.max(0.0)
    } else {
        1.0
    }
}

/// The clock of tick `tick` after `clock` stepped the tick before: the same
/// clock, or re-anchored at `tick` where the scale changes.
fn next_clock(clock: ParticleClock, tick: u64, scale: f32, dt: f64) -> ParticleClock {
    if clock.scale == scale {
        clock
    } else {
        ParticleClock {
            tick,
            time: clock.at(tick, dt),
            scale,
        }
    }
}

/// A lifetime in ticks at `scale` (slowed particles live more ticks),
/// replayed at most 16 normal lifetimes back.
fn window(e: &ParticleEmitter, scale: f32, tick_rate: f32) -> u64 {
    let window = (e.max_lifetime() as f64 * tick_rate as f64).ceil() as u64 + 1;
    if scale >= 1.0 {
        window
    } else {
        (window as f64 / f64::from(scale.max(1.0 / 16.0))).ceil() as u64
    }
}

/// Where particle systems simulate ([`crate::RenderDesc::particles`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ParticleSimulation {
    /// Compute (sprite-mode emitters on the CPU).
    #[default]
    Auto,
    /// Always on the CPU, on any backend: the state is Rust's to read and
    /// collides with [`crate::RenderScene::colliders`] instead of
    /// the depth buffer.
    Cpu,
}

#[cfg(test)]
mod tests;
