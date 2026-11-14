//! og.ForceField, a region's acceleration: read from the world in one
//! place as each tick starts ([`before_tick`]) and given to the physics
//! that moves bodies, ropes, cloths, fluids and characters, so they
//! cannot disagree about where a field pulls.

use crate::host::wire::{ForceFieldKind, FromJson, record};
use og_ecs::World;
use og_env::force::{Field, FieldKind, Forces};
use og_env::volume::Placement;

/// How a field accelerates, as og-env takes it.
fn kind_of(kind: ForceFieldKind) -> FieldKind {
    match kind {
        ForceFieldKind::Directional => FieldKind::Directional,
        ForceFieldKind::Radial => FieldKind::Radial,
        ForceFieldKind::Vortex => FieldKind::Vortex,
    }
}

/// As each tick starts: every enabled og.ForceField at its entity's world
/// pose with the og.Volume beside it, in entity order, given to physics.
pub(crate) fn before_tick(eng: &crate::engine::Engine, world: &World) {
    let (fields, volumes, physics) = {
        let st = eng.state.borrow();
        (
            st.layouts.get("og.ForceField").cloned(),
            st.layouts.get("og.Volume").cloned(),
            st.physics.clone(),
        )
    };
    let Some(physics) = physics else {
        return;
    };
    let r = crate::bind::physics_ext::Reader::World(world);
    let fields = r
        .rows(fields.as_ref())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(e, v)| {
            let v = record::ForceField::from_json(&v).ok()?;
            if !v.enabled {
                return None;
            }
            let e = og_ecs::Entity::from_bits(e);
            let (position, quaternion, scale) = r.pose(e)?;
            Some(Field {
                at: Placement {
                    position,
                    quaternion,
                    scale,
                },
                volume: crate::world::region::volume_of(world, volumes.as_ref(), e),
                kind: kind_of(v.kind),
                direction: (quaternion * v.direction).normalize_or_zero(),
                strength: v.strength,
            })
        })
        .collect();
    physics.set_forces(Forces { fields });
}
