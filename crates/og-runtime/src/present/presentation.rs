//! Presentation state stepped by the tick (docs/design.md §2.4): what the
//! frames draw that depends on the ticks before the drawn one, kept here as
//! each tick starts, whether or not a frame is drawn, so a frame and a
//! capture are functions of the world and this state, never of how many
//! frames were drawn before them.
//!
//! Each particle system's ticks ([`og_render::ParticleHistory`]): the
//! renderer steps a system through the ticks it has not drawn from them
//! (where its emitter was and whether it emitted on each), not from the
//! frame's system alone.
//!
//! A probe snapshot keeps this state beside the world as a presentation
//! checkpoint ([`save`], [`restore`]), with the renderer's own (the adapted
//! auto exposure), outside `world_hash`; [`hash`] is the presentation hash
//! a snapshot reports, per host and backend.
//!
//! Presentation only: nothing here writes the world or its hash.

use crate::present::extract::RenderState;
use og_core::IdMap;
use og_ecs::World;
use std::collections::VecDeque;
use std::sync::Arc;

/// Checkpoints kept for the probe's snapshots (it keeps 16; sweeps and
/// `reload` take their own).
const KEPT: usize = 64;

/// The presentation state a world snapshot was taken with.
struct Checkpoint {
    particles: IdMap<u64, Arc<og_render::ParticleHistory>>,
    /// og.motion's drawn agents and the model's queued work.
    motion: crate::bind::motion::present::MotionCheckpoint,
    /// The renderer's ([`crate::RenderBackend::save_state`]): its frame
    /// history.
    renderer: u64,
    ui: Option<UiCheckpoint>,
    /// Render interpolation's: og.Camera's pose, the snaps and og-anim's
    /// poses as the snapshot's tick started, and the transforms then (the
    /// raycast record's).
    interp: crate::present::interpolate::InterpCheckpoint,
    record: crate::present::geometry::RecordCheckpoint,
    /// Per-object motion: what the frame before drew, which the next
    /// frame's motion vectors move from.
    objects: crate::present::fx::MotionCheckpoint,
    /// The streamed terrains' tiles: requested, decoded and held.
    terrain: crate::bind::terrain::TerrainCheckpoint,
}

/// Portable UI presentation state, carried beside the simulation snapshot.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct UiCheckpoint {
    state: Vec<u8>,
    world: Vec<u8>,
}

impl UiCheckpoint {
    pub(crate) fn save(ui: &og_ui::SharedUi) -> Self {
        let ui = ui.lock();
        Self {
            state: ui.save_state(),
            world: ui.save_world_state(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut ui = og_ui::Ui::default();
        ui.restore_state(&self.state)
            .map_err(|e| e.reason().to_owned())?;
        ui.restore_world_state(&self.world)
            .map_err(|e| e.reason().to_owned())
    }

    pub(crate) fn restore(&self, ui: &og_ui::SharedUi) -> Result<(), String> {
        self.validate_for(ui)?;
        let mut ui = ui.lock();
        ui.restore_state(&self.state)
            .map_err(|e| e.reason().to_owned())?;
        ui.restore_world_state(&self.world)
            .map_err(|e| e.reason().to_owned())
    }

    pub(crate) fn validate_for(&self, current: &og_ui::SharedUi) -> Result<(), String> {
        let mut ui = og_ui::Ui::default();
        ui.inherit_store_registrations(&current.lock());
        ui.restore_state(&self.state)
            .map_err(|e| e.reason().to_owned())?;
        ui.restore_world_state(&self.world)
            .map_err(|e| e.reason().to_owned())
    }
}

/// The checkpoints kept, newest last, by id (0 is none).
#[derive(Default)]
pub(crate) struct Checkpoints {
    next: u64,
    kept: VecDeque<(u64, Checkpoint)>,
}

/// The presentation state now, as a checkpoint for a snapshot: its id.
pub(crate) fn save(rs: &mut RenderState) -> u64 {
    let renderer = rs.backend.as_mut().map_or(0, |b| b.save_state());
    let terrain = crate::bind::terrain::checkpoint(rs);
    let c = &mut rs.checkpoints;
    c.next += 1;
    c.kept.push_back((
        c.next,
        Checkpoint {
            particles: rs.particle_history.clone(),
            motion: rs.motion.checkpoint(),
            renderer,
            ui: rs.ui.as_ref().map(UiCheckpoint::save),
            interp: rs.interp.checkpoint(),
            record: rs.geo.record_checkpoint(),
            objects: rs.fx.motion_checkpoint(),
            terrain,
        },
    ));
    while c.kept.len() > KEPT {
        c.kept.pop_front();
    }
    c.next
}

pub(crate) fn validate_restore(rs: &RenderState, id: u64) -> Result<(), String> {
    if let Some((_, checkpoint)) = rs.checkpoints.kept.iter().find(|(k, _)| *k == id)
        && let (Some(saved), Some(current)) = (&checkpoint.ui, &rs.ui)
    {
        saved.validate_for(current)?;
    }
    Ok(())
}

/// Brings back checkpoint `id` with the snapshot it was taken with, so the
/// frames after it are the uninterrupted run's; 0, or one no longer kept,
/// starts the presentation state afresh (the restored tick's display
/// frames then draw the tick: their blend reads the tick before's
/// transforms, which only a checkpoint holds).
pub(crate) fn restore(rs: &mut RenderState, id: u64) {
    let kept = rs.checkpoints.kept.iter().position(|(k, _)| *k == id);
    match kept {
        Some(at) => {
            let (k, c) = rs.checkpoints.kept.remove(at).expect("found");
            rs.particle_history = c.particles.clone();
            rs.motion.restore(&c.motion);
            if let Some(b) = rs.backend.as_mut() {
                b.restore_state(c.renderer);
            }
            if let (Some(saved), Some(current)) = (&c.ui, &rs.ui) {
                saved
                    .restore(current)
                    .expect("presentation checkpoint preflight passed");
            }
            rs.interp.restore(&c.interp);
            rs.geo.restore_record(&c.record);
            rs.fx.restore_motion(&c.objects);
            crate::bind::terrain::restore(rs, &c.terrain);
            rs.checkpoints.kept.insert(at, (k, c));
        }
        None => {
            rs.particle_history = Default::default();
            rs.motion.forget();
            if let Some(b) = rs.backend.as_mut() {
                b.restore_state(0);
            }
            rs.interp.forget();
            rs.fx.restore_motion(&Default::default());
        }
    }
}

/// The presentation hash: FNV-1a over the state a checkpoint keeps that the
/// CPU holds (og.motion's, then each particle system's ticks, in id order). Equal hashes on
/// one host and backend mean the same frames from the same world.
pub(crate) fn hash(rs: &RenderState) -> u64 {
    let mut h = rs.motion.hash();
    let mut put = |bytes: &[u8]| {
        for &b in bytes {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for (id, history) in rs.particle_history.iter() {
        put(&id.to_le_bytes());
        for t in history.ticks() {
            put(&t.tick.to_le_bytes());
            for v in t.transform.to_cols_array() {
                put(&v.to_bits().to_le_bytes());
            }
            put(&t.start_tick.to_le_bytes());
            put(&t.stop_tick.unwrap_or(u64::MAX).to_le_bytes());
            put(&t.time_scale.to_bits().to_le_bytes());
            put(&t.clock.tick.to_le_bytes());
            put(&t.clock.time.to_bits().to_le_bytes());
            put(&t.clock.scale.to_bits().to_le_bytes());
        }
    }
    if let Some(ui) = &rs.ui {
        let saved = UiCheckpoint::save(ui);
        put(&saved.state);
        put(&saved.world);
    }
    h
}

/// As a tick starts, the world the last one left: each particle system as
/// that tick left it joins its history. Only while a renderer draws: the
/// particles' tick is the one the systems were extracted for, as a frame
/// between ticks extracts them.
pub(crate) fn before_tick(rs: &mut RenderState, world: &World) {
    if rs.backend.is_none() {
        return;
    }
    let tick = world.time().tick;
    let mut h = std::mem::take(&mut rs.hierarchy);
    h.refresh(world);
    // The systems a frame at this tick would draw: og.ParticleSystem and
    // the composed effects' emitters (the next frame's extraction
    // rebuilds the lists these fill).
    crate::present::fx::components(world, rs, &mut h, tick);
    crate::present::vfx::extract(world, rs, &mut h, tick);
    rs.hierarchy = h;
    // The rate a frame's extraction gives the scene (`extract/frame.rs`).
    let dt = world.time().dt;
    let rate = if dt > 0.0 { 1.0 / dt } else { 60.0 };
    let RenderState {
        scene,
        particle_history,
        ..
    } = rs;
    let mut live = og_core::IdMap::new();
    for sys in &scene.effects.particles {
        let Some(e) = scene.emitter(sys.emitter) else {
            continue;
        };
        let mut history = particle_history.remove(&sys.id).unwrap_or_default();
        Arc::make_mut(&mut history).record(e, sys, rate);
        live.insert(sys.id, history);
    }
    *particle_history = live;
}
