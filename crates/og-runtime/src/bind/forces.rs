//! og.ForceField, a region's acceleration: read from the world in one
//! place as each tick starts ([`before_tick`]) and given to the physics
//! that moves bodies, ropes, cloths, fluids and characters, and kept for
//! the particle systems the tick simulates, so they cannot disagree about
//! where a field pulls. Extraction reads them for the renderer's particles
//! ([`frame_forces`]).

use crate::host::codec::Layout;
use crate::host::wire::{ForceFieldKind, FromJson, record};
use glam::{Quat, Vec3};
use og_ecs::World;
use og_env::force::{Field, FieldKind, Forces};
use og_env::volume::Placement;
use std::sync::{Arc, Mutex};

/// The force fields of one engine's world this tick, as [`before_tick`]
/// read them: shared with the systems the tick runs.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Arc<Mutex<Forces>>);

impl ModuleState {
    /// The fields this tick.
    pub(crate) fn get(&self) -> Forces {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

/// How a field accelerates, as og-env takes it.
fn kind_of(kind: ForceFieldKind) -> FieldKind {
    match kind {
        ForceFieldKind::Directional => FieldKind::Directional,
        ForceFieldKind::Radial => FieldKind::Radial,
        ForceFieldKind::Vortex => FieldKind::Vortex,
    }
}

/// One og.ForceField row at its entity's world pose with the og.Volume
/// beside it; `None` while it is disabled.
fn field(
    v: &serde_json::Value,
    (position, quaternion, scale): (Vec3, Quat, Vec3),
    volume: Option<og_env::volume::Volume>,
) -> Option<Field> {
    let v = record::ForceField::from_json(v).ok()?;
    if !v.enabled {
        return None;
    }
    Some(Field {
        at: Placement {
            position,
            quaternion,
            scale,
        },
        volume,
        kind: kind_of(v.kind),
        direction: (quaternion * v.direction).normalize_or_zero(),
        strength: v.strength,
    })
}

/// Runs right before `og.physics` (`og.physics.env`): every enabled
/// og.ForceField at its entity's world pose with the og.Volume beside it,
/// in entity order, kept for particle systems and given to physics in the
/// same tick.
pub(crate) fn sync_env(
    eng: &crate::engine::Engine,
    physics: Option<&og_physics::PhysicsSystem>,
    r: &crate::bind::physics_ext::Reader<'_>,
) {
    let (fields, volumes) = {
        let st = eng.state.borrow();
        (
            st.layouts.get("og.ForceField").cloned(),
            st.layouts.get("og.Volume").cloned(),
        )
    };
    let fields = r
        .rows(fields.as_ref())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(e, v)| {
            let e = og_ecs::Entity::from_bits(e);
            let volume = r.volume(volumes.as_ref(), e);
            field(&v, r.pose(e)?, volume)
        })
        .collect();
    let forces = Forces { fields };
    if let Some(p) = physics {
        p.set_forces(forces.clone());
    }
    *eng.modules
        .forces
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = forces;
}

/// The force fields a frame draws its particles in: every enabled
/// og.ForceField where the frame's hierarchy places its entity, in entity
/// order.
pub(crate) fn frame_forces(
    world: &World,
    fields: Option<&Layout>,
    volumes: Option<&Layout>,
    h: &mut crate::present::extract::Hierarchy,
) -> Forces {
    let fields = crate::present::fx::kept_rows(world, fields)
        .into_iter()
        .filter_map(|(e, v)| {
            let (scale, quaternion, position) = h.world_of(e).to_scale_rotation_translation();
            let volume =
                crate::world::region::volume_of(world, volumes, og_ecs::Entity::from_bits(e));
            field(&v, (position, quaternion, scale), volume)
        })
        .collect();
    Forces { fields }
}
