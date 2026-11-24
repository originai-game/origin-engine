//! The air on rigid bodies: og.RigidBody.airDrag carries a body towards
//! the air's velocity where it is ([`og_env::air::Air`], the world's wind
//! and its motors).

use super::*;

/// Carries every dynamic body with an `airDrag` towards the air at its
/// centre of mass inside each substep, via `add_force` so `LockedAxes` are
/// respected and sleeping bodies are not thrashed by negligible breezes.
pub(crate) fn air_drag(
    scene: &Scene,
    air: &og_env::air::Air,
    sub_dt: f32,
    sim: &mut Sim,
    handles: &super::super::sim::Handles<'_>,
    forced_bodies: &mut Vec<rp::RigidBodyHandle>,
) {
    if sub_dt <= 0.0 {
        return;
    }
    let inv_dt = 1.0 / sub_dt;
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
        let gain = flat(air.carry(unv(rb.center_of_mass()), velocity, b.body.air_drag, sub_dt));
        if gain == Vec3::ZERO {
            continue;
        }
        let accel = gain * inv_dt;
        // Do not wake a sleeping body for a tiny breeze below rest threshold.
        if rb.is_sleeping() && accel.length_squared() < 0.05 * 0.05 {
            continue;
        }
        let mass = rb.mass();
        if mass > 0.0 {
            rb.add_force(v(accel * mass), true);
            forced_bodies.push(h);
        }
    }
}
