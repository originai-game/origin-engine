//! The world's force fields on rigid bodies: every dynamic body takes the
//! acceleration the fields have where it is ([`og_env::force::Forces`],
//! the og.ForceFields), whatever its mass, as it takes gravity.

use super::*;

/// Accelerates every dynamic body by the fields at its centre of mass,
/// before the step: in entity order, each alone. A body a field reaches
/// is kept awake, as one afloat is, so a field switched on moves what
/// rested in it and a replay and a restore agree.
pub(crate) fn force_fields(
    scene: &Scene,
    forces: &og_env::force::Forces,
    dt: f32,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
) {
    if forces.is_empty() {
        return;
    }
    for b in &scene.bodies {
        let Some(&h) = handles.get(&b.entity.to_bits()) else {
            continue;
        };
        let rb = &mut sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let gain = flat(forces.at(unv(rb.center_of_mass())) * dt);
        if gain != Vec3::ZERO {
            rb.set_linvel(v(unv(rb.linvel()) + gain), true);
        }
    }
}
