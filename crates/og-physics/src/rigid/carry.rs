//! Supporting-frame carry and finite-mass contact-point impulses.

use super::super::mover::Hit;
use super::*;

/// Carry is measured at the saved foot contact, not the capsule centre.
/// Keep only twist about up; a pitching deck must not tip a character over.
pub(crate) fn carried(before: rp::Pose, after: rp::Pose, point: Vec3, up: Vec3) -> (Vec3, Quat) {
    let relative = after * before.inverse();
    let delta = flat(unv(relative.transform_point(v(point))) - point);
    let (_, q) = unpose(&relative, Vec3::ZERO);
    let vector = Vec3::new(q.x, q.y, q.z);
    let twist = up * vector.dot(up);
    let yaw = Quat::from_xyzw(twist.x, twist.y, twist.z, q.w);
    (
        delta,
        if yaw.length_squared() > 1e-12 {
            yaw.normalize()
        } else {
            Quat::IDENTITY
        },
    )
}

/// Inelastic momentum exchange using point velocity and effective inverse
/// mass (including angular inertia and locked axes). Side pushes share the
/// force budget. Weight is a separate physical load at the support point.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push(
    w: &mut rp::PhysicsWorld,
    hits: &[Hit],
    ground: Option<Hit>,
    velocity: Vec3,
    mass: f32,
    strength: f32,
    gravity: Vec3,
    dt: f32,
    forced: &mut Vec<u64>,
) {
    if mass <= 0.0 || strength <= 0.0 {
        return;
    }
    let mut asks: Vec<(rp::RigidBodyHandle, Vec3, Vec3, f32)> = Vec::new();
    for hit in hits {
        let Some(parent) = w.colliders[hit.handle].parent() else {
            continue;
        };
        let rb = &w.bodies[parent];
        if !rb.is_dynamic() || ground.is_some_and(|g| g.handle == hit.handle) {
            continue;
        }
        let into = -hit.normal;
        let point = v(hit.point);
        let speed = (velocity - unv(rb.velocity_at_point(point)))
            .dot(into)
            .max(0.0);
        let n = v(into);
        let props = rb.mass_properties();
        let torque = angvel((hit.point - unv(rb.center_of_mass())).cross(into));
        let angular = unangvel(torque).dot(unangvel(props.effective_world_inv_inertia * torque));
        let inv = (props.effective_inv_mass * n).dot(n) + angular;
        let ask = speed / (mass.recip() + inv);
        match asks.iter_mut().find(|a| a.0 == parent) {
            Some(a) if a.3 < ask => *a = (parent, into, hit.point, ask),
            Some(_) => {}
            None => asks.push((parent, into, hit.point, ask)),
        }
    }
    let asked: f32 = asks.iter().map(|a| a.3).sum();
    let share = if asked > 0.0 {
        (strength * dt / asked).min(1.0)
    } else {
        0.0
    };
    for (body, into, point, ask) in asks {
        if ask * share > 0.0 {
            forced.push(w.bodies[body].user_data as u64);
            w.bodies[body].apply_impulse_at_point(v(flat(into * (ask * share))), v(point), true);
        }
    }
    if let Some(g) = ground
        && gravity != Vec3::ZERO
        && let Some(parent) = w.colliders[g.handle].parent()
        && w.bodies[parent].is_dynamic()
    {
        forced.push(w.bodies[parent].user_data as u64);
        w.bodies[parent].apply_impulse_at_point(v(flat(gravity * mass * dt)), v(g.point), true);
    }
}
