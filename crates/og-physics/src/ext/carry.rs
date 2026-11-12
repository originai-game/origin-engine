//! Moving a kinematic shape: what a platform carries and what a mover pushes.

use super::*;

/// Where a point on a body moved during the step (the body's motion from
/// its pre-step pose), for riding moving platforms.
pub(crate) fn carried(
    poses: &BodyPoses,
    w: &rp::PhysicsWorld,
    body: rp::RigidBodyHandle,
    point: Vec3,
) -> Vec3 {
    let Some(before) = poses.get(body) else {
        return Vec3::ZERO;
    };
    let after = *w.bodies[body].position();
    let moved = after * before.inverse();
    flat(unv(moved.transform_point(v(point)))) - flat(point)
}

/// The body a collider's entity is, by the tracked bodies.
pub(crate) fn body_of(sim: &Sim, entity: u64) -> Option<rp::RigidBodyHandle> {
    sim.bodies.get(&entity).map(|t| t.handle)
}

/// Pushes the dynamic bodies a controller walked into, one rule for both
/// controllers: each body is asked to take (most of) the controller's speed
/// into it along the contact normal, an impulse of its mass times that
/// speed; the bodies of one tick share at most `strength` (N) times `dt`,
/// each its share of what it asked for, so a light body is shoved at the
/// controller's speed and a heavy one only as far as the force beats its
/// friction (a tonne on the ground holds), and two bodies walked into at
/// once split the push instead of the first taking it all. A body hit more
/// than once in the tick asks once, its largest.
pub(crate) fn push(
    w: &mut rp::PhysicsWorld,
    hits: &[(rp::ColliderHandle, Vec3)],
    velocity: Vec3,
    strength: f32,
    dt: f32,
) {
    let budget = strength.max(0.0) * dt;
    if budget <= 0.0 {
        return;
    }
    // (body, direction, impulse asked), in hit order.
    let mut asks: Vec<(rp::RigidBodyHandle, Vec3, f32)> = Vec::new();
    for &(h, normal) in hits {
        let Some(parent) = w.colliders.get(h).and_then(|c| c.parent()) else {
            continue;
        };
        let rb = &w.bodies[parent];
        if !rb.is_dynamic() {
            continue;
        }
        let into = -normal;
        let speed = velocity.dot(into) - unv(rb.linvel()).dot(into);
        if speed <= 0.0 {
            continue;
        }
        let ask = speed * rb.mass() * 0.8;
        match asks.iter_mut().find(|a| a.0 == parent) {
            Some(a) if a.2 < ask => *a = (parent, into, ask),
            Some(_) => {}
            None => asks.push((parent, into, ask)),
        }
    }
    let asked: f32 = asks.iter().map(|a| a.2).sum();
    let share = if asked > budget { budget / asked } else { 1.0 };
    for (body, into, ask) in asks {
        w.bodies[body].apply_impulse(v(flat(into * (ask * share))), true);
    }
}
