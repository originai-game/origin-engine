//! The world's force fields on rigid bodies: every dynamic body takes the
//! acceleration the fields have where it is ([`og_env::force::Forces`],
//! the og.ForceFields), whatever its mass, as it takes gravity.

use super::*;

/// Accelerates every dynamic body by the force fields at its centre of mass
/// inside each substep, via `add_force` so `LockedAxes` are respected.
pub(crate) fn force_fields(
    scene: &Scene,
    forces: &og_env::force::Forces,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    forced_bodies: &mut Vec<rp::RigidBodyHandle>,
) {
    if forces.is_empty() {
        return;
    }
    let bones = sim.ragdolls.values().flat_map(|s| s.bones.iter().copied());
    let all: Vec<_> = scene
        .bodies
        .iter()
        .filter_map(|b| handles.get(&b.entity.to_bits()).copied())
        .chain(bones)
        .collect();
    for h in all {
        let rb = &mut sim.world.bodies[h];
        if !rb.is_dynamic() {
            continue;
        }
        let accel = flat(forces.at(unv(rb.center_of_mass())));
        if accel == Vec3::ZERO {
            continue;
        }
        if rb.is_sleeping() && accel.length_squared() < 1e-4 {
            continue;
        }
        let mass = rb.mass();
        if mass > 0.0 {
            rb.add_force(v(accel * mass), true);
            forced_bodies.push(h);
        }
    }
}
