//! The air on rigid bodies: og.RigidBody.airDrag carries a body towards
//! the air's velocity where it is ([`og_env::air::Air`], the world's wind
//! and its motors).

use super::*;

/// Carries every dynamic body with an `airDrag` towards the air at its
/// centre of mass, before the step: in entity order, each alone.
pub(crate) fn air_drag(
    scene: &Scene,
    air: &og_env::air::Air,
    dt: f32,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
) {
    for b in &scene.bodies {
        if b.body.air_drag <= 0.0 {
            continue;
        }
        let Some(&h) = handles.get(&b.entity.to_bits()) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let velocity = unv(rb.linvel());
        let gain = flat(air.carry(unv(rb.center_of_mass()), velocity, b.body.air_drag, dt));
        if gain != Vec3::ZERO {
            rb.set_linvel(v(velocity + gain), true);
        }
    }
}
