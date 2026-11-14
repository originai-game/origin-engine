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

pub(crate) mod effect;
pub(crate) mod particles_gpu;
pub(crate) mod ribbons;
pub(crate) mod ribbons_gpu;
pub use og_render_data::particles::*;

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
