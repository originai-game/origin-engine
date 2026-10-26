//! What og.CollisionStarted says about the hit (Rumpus Isle FINDINGS #9):
//! the solver's normal impulse (N·s) and `a`'s velocity relative to `b` at
//! the contact point as the step that made the contact began, in both
//! profiles. The event is read the tick after the step, when og.Velocity
//! already holds the solver's answer, so these are the only way a game
//! knows how hard two bodies met.
//!
//! Ways this could fail: the impulse is not the one the bodies took (each
//! side's momentum change differs from it), or one side's; the relative
//! velocity is post-solver (closing speed 0 or negative), signed b to a,
//! or not flipped with the normal when the lower id is collider 2; a
//! spinning body's point velocity leaves out ω × r; a static collider or a
//! kinematic body is taken as moving wrongly; a restored snapshot gives
//! other values.

use crate::support;

use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

fn weightless(profile: u32) -> Physics {
    Physics {
        profile,
        gravity: Vec3::ZERO,
        ..Physics::default()
    }
}

fn slick(c: Collider) -> Collider {
    Collider { friction: 0.0, ..c }
}

fn moving(w: &mut World, t: Transform, body: RigidBody, c: Collider, v: Velocity) -> Entity {
    w.spawn((t, body, c, v)).unwrap()
}

fn linear(x: f32, y: f32, z: f32) -> Velocity {
    Velocity {
        linear: Vec3::new(x, y, z),
        angular: Vec3::ZERO,
    }
}

/// Steps until a CollisionStarted is pending; returns it with the
/// velocities of `watch` before that tick and after it.
fn until_hit(
    w: &mut World,
    watch: &[Entity],
    limit: u32,
) -> (CollisionStarted, Vec<(Velocity, Velocity)>) {
    for _ in 0..limit {
        let before: Vec<Velocity> = watch
            .iter()
            .map(|&e| w.get::<Velocity>(e).unwrap())
            .collect();
        w.step().unwrap();
        let started = w.pending_events::<CollisionStarted>().unwrap();
        if let Some(&s) = started.first() {
            assert_eq!(started.len(), 1, "{started:?}");
            let after = watch.iter().map(|&e| velocity(w, e));
            return (s, before.into_iter().zip(after).collect());
        }
    }
    panic!("no CollisionStarted in {limit} ticks");
}

fn near(a: Vec3, b: Vec3, tol: f32) -> bool {
    (a - b).length() <= tol
}

/// A 1 kg ball at 6 m/s along +x into a 10 kg ball at rest, weightless and
/// frictionless; `light_first` spawns the light one first (the lower id, `a`).
/// `substeps` > 1 sums the impulse over them and keeps the first's velocity.
fn asymmetric(profile: u32, light_first: bool, substeps: u32) {
    let settings = Physics {
        substeps,
        ..weightless(profile)
    };
    let (mut w, _p) = world(settings, 1);
    let light = |w: &mut World| {
        moving(
            w,
            at(-2.0, 0.0, 0.0),
            RigidBody {
                mass: 1.0,
                ..RigidBody::dynamic()
            },
            slick(Collider::sphere(0.5)),
            linear(6.0, 0.0, 0.0),
        )
    };
    let heavy = |w: &mut World| {
        moving(
            w,
            at(0.0, 0.0, 0.0),
            RigidBody {
                mass: 10.0,
                ..RigidBody::dynamic()
            },
            slick(Collider::sphere(0.5)),
            Velocity::default(),
        )
    };
    let (l, h) = if light_first {
        let l = light(&mut w);
        (l, heavy(&mut w))
    } else {
        let h = heavy(&mut w);
        (light(&mut w), h)
    };
    let (s, v) = until_hit(&mut w, &[l, h], 60);
    let (a, b) = if light_first { (l, h) } else { (h, l) };
    assert_eq!((s.a, s.b), (a.to_bits(), b.to_bits()), "{s:?}");
    // The normal points from a to b: +x from the light ball, -x from the heavy.
    let x = if light_first { 1.0 } else { -1.0 };
    assert!(near(s.normal, Vec3::new(x, 0.0, 0.0), 1e-3), "{s:?}");
    // a relative to b before the solver: the light ball's 6 m/s, signed.
    assert!(
        near(s.relative_velocity, Vec3::new(6.0 * x, 0.0, 0.0), 1e-3),
        "{s:?}"
    );
    assert!(
        (s.relative_velocity.dot(s.normal) - 6.0).abs() < 1e-3,
        "closing speed {s:?}"
    );
    // Each side's momentum change is the impulse along the normal, opposite.
    let dp = |m: f32, (before, after): (Velocity, Velocity)| m * (after.linear - before.linear);
    let (dl, dh) = (dp(1.0, v[0]), dp(10.0, v[1]));
    // The light ball moves -x by the impulse, the heavy one +x.
    let j = Vec3::new(s.impulse, 0.0, 0.0);
    assert!(s.impulse > 1.0, "{s:?}");
    assert!(
        near(dl, -j, 0.02 * s.impulse),
        "light {dl} vs impulse {s:?}"
    );
    assert!(near(dh, j, 0.02 * s.impulse), "heavy {dh} vs impulse {s:?}");
    // Inelastic (restitution 0): the closing speed is taken out, 6 m/s at
    // the reduced mass of 10/11 kg.
    assert!((s.impulse - 6.0 * 10.0 / 11.0).abs() < 0.1, "{s:?}");
}

#[test]
fn a_light_body_into_a_heavy_one_reports_one_impulse_for_both_and_a_to_b_velocity() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        asymmetric(profile, true, 1);
        asymmetric(profile, false, 1);
        asymmetric(profile, true, 4);
    }
}

/// A 2 m rod spinning at 10 rad/s about z, its tip sweeping up into a
/// static slab: the contact's velocity is the spin's ω × r alone.
fn spinning(profile: u32) {
    let (mut w, _p) = world(weightless(profile), 2);
    let rod = moving(
        &mut w,
        at(0.0, 0.0, 0.0),
        RigidBody::dynamic(),
        slick(Collider::cuboid(Vec3::new(2.0, 0.2, 0.2))),
        Velocity {
            linear: Vec3::ZERO,
            angular: Vec3::new(0.0, 0.0, 10.0),
        },
    );
    spawn_static(
        &mut w,
        at(1.0, 0.8, 0.0),
        slick(Collider::cuboid(Vec3::new(1.0, 1.0, 1.0))),
    );
    let (s, _) = until_hit(&mut w, &[rod], 60);
    assert_eq!(s.a, rod.to_bits(), "the rod spawned first");
    // The slab's underside is y = 0.3: a downward normal from the rod.
    assert!(near(s.normal, Vec3::new(0.0, 1.0, 0.0), 0.05), "{s:?}");
    // ω × r about the rod's centre (at the origin), r the contact point.
    let expect = Vec3::new(0.0, 0.0, 10.0).cross(s.point);
    assert!(
        near(s.relative_velocity, expect, 0.05 * expect.length()),
        "{s:?}, ω × r = {expect}"
    );
    assert!(s.relative_velocity.dot(s.normal) > 5.0, "{s:?}");
    assert!(s.impulse > 0.0, "{s:?}");
}

#[test]
fn a_spinning_body_hitting_a_wall_counts_its_angular_point_velocity() {
    spinning(PROFILE_3D);
    spinning(PROFILE_2D);
}

/// A 2 kg box falling onto a static floor: the floor (spawned first) is
/// `a`, so a's velocity relative to the box is up, closing along +y.
fn on_static(profile: u32) {
    let (mut w, _p) = world(
        Physics {
            profile,
            ..Physics::default()
        },
        3,
    );
    floor(&mut w);
    let crate_ = moving(
        &mut w,
        at(0.0, 2.0, 0.0),
        RigidBody {
            mass: 2.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::ONE),
        Velocity::default(),
    );
    let (s, v) = until_hit(&mut w, &[crate_], 120);
    assert!(near(s.normal, Vec3::Y, 1e-3), "{s:?}");
    // The box's velocity as the step began, seen from the floor: up.
    let falling = v[0].0.linear;
    assert!(falling.y < -3.0, "{falling}");
    assert!(
        near(s.relative_velocity, -falling, 1e-3),
        "{s:?} vs {falling}"
    );
    // The floor stops it: the impulse is its momentum change plus what
    // gravity added during the tick (m g dt).
    let stopped = 2.0 * (v[0].1.linear.y - falling.y) + 2.0 * 9.81 / 60.0;
    assert!(
        (s.impulse - stopped).abs() < 0.02 * stopped,
        "{s:?}, Δp {stopped}"
    );
}

/// A 1 kg box at rest above a kinematic platform rising at 3 m/s,
/// weightless: the platform's velocity is the whole of the hit.
fn on_kinematic(profile: u32) {
    let (mut w, _p) = world(weightless(profile), 4);
    let platform = moving(
        &mut w,
        at(0.0, -0.5, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::new(4.0, 1.0, 4.0)),
        linear(0.0, 3.0, 0.0),
    );
    let crate_ = moving(
        &mut w,
        at(0.0, 1.0, 0.0),
        RigidBody {
            mass: 1.0,
            ..RigidBody::dynamic()
        },
        Collider::cuboid(Vec3::ONE),
        Velocity::default(),
    );
    let (s, v) = until_hit(&mut w, &[crate_], 60);
    assert_eq!((s.a, s.b), (platform.to_bits(), crate_.to_bits()));
    assert!(near(s.normal, Vec3::Y, 1e-3), "{s:?}");
    assert!(
        near(s.relative_velocity, Vec3::new(0.0, 3.0, 0.0), 1e-3),
        "{s:?}"
    );
    // The box takes the platform's 3 m/s: 3 N·s.
    assert!(v[0].1.linear.y > 2.9, "{:?}", v[0]);
    assert!((s.impulse - v[0].1.linear.y).abs() < 0.03, "{s:?}");
}

#[test]
fn a_dynamic_body_on_a_static_and_on_a_kinematic_body() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        on_static(profile);
        on_kinematic(profile);
    }
}

#[test]
fn a_snapshot_taken_before_the_hit_replays_the_same_event() {
    for profile in [PROFILE_3D, PROFILE_2D] {
        let (mut w, _p) = world(weightless(profile), 5);
        let rod = moving(
            &mut w,
            at(-3.0, 0.0, 0.0),
            RigidBody {
                mass: 1.5,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::new(2.0, 0.2, 0.2)),
            Velocity {
                linear: Vec3::new(5.0, 0.5, 0.0),
                angular: Vec3::new(0.0, 0.0, 4.0),
            },
        );
        let ball = moving(
            &mut w,
            at(0.5, 0.3, 0.0),
            RigidBody {
                mass: 4.0,
                ..RigidBody::dynamic()
            },
            Collider::sphere(0.5),
            linear(-1.0, 0.0, 0.0),
        );
        run(&mut w, 5);
        let saved = w.snapshot();
        let (first, _) = until_hit(&mut w, &[rod, ball], 120);
        let tick = w.time().tick;
        assert!(first.impulse > 0.0, "{first:?}");
        w.restore(&saved).unwrap();
        let (again, _) = until_hit(&mut w, &[rod, ball], 120);
        assert_eq!(w.time().tick, tick);
        assert_eq!(again, first, "the replayed hit differs");
    }
}

/// A top-down character controller (gravityScale 0) walking +x, faster
/// each tick (1 m/s and 0.25 m/s more a tick, so this tick's velocity is
/// not the last one's), under a 2 kg ball dropped on its head at 8 m/s, weightless: the ball
/// comes from above, so the walk never sees it in the way, and it lands
/// while the controller moves. The controller is a position-based
/// kinematic body: the contact meets it at the velocity carrying it from
/// its pose to the target its last move set, and only the ball's momentum
/// answers the impulse.
fn controller_hit(profile: u32) {
    let (mut w, _p) = world(weightless(profile), 6);
    let walker = w
        .spawn((
            at(0.0, 0.0, 0.0),
            slick(Collider::capsule(0.4, 1.0)),
            CharacterController {
                movement: Vec3::new(3.0, 0.0, 0.0),
                gravity_scale: 0.0,
                push_bodies: false,
                ..CharacterController::default()
            },
        ))
        .unwrap();
    // Its cap's top is 0.9 m up: 1.6 m (12 ticks) of fall, by when the
    // controller has walked about 0.5 m.
    let ball = moving(
        &mut w,
        at(0.5, 3.0, 0.0),
        RigidBody {
            mass: 2.0,
            ..RigidBody::dynamic()
        },
        slick(Collider::sphere(0.5)),
        linear(0.0, -8.0, 0.0),
    );
    // The controller's target for each step is where its move after the
    // step before put its Transform; it starts the step where the move
    // before that put it.
    let mut targets = vec![transform(&w, walker).position];
    for k in 0..60 {
        let mut c = w.get::<CharacterController>(walker).unwrap();
        c.movement = Vec3::new(1.0 + 0.25 * k as f32, 0.0, 0.0);
        w.insert(walker, c).unwrap();
        let before = velocity(&w, ball);
        w.step().unwrap();
        let started = w.pending_events::<CollisionStarted>().unwrap();
        if let Some(&s) = started.first() {
            let n = targets.len();
            assert!(n >= 3, "hit in the first ticks");
            let expect = (targets[n - 1] - targets[n - 2]) * 60.0;
            let last = (targets[n - 2] - targets[n.saturating_sub(3)]) * 60.0;
            assert!(
                expect.x > 2.0 && expect.x - last.x > 0.2,
                "the controller was walking, faster than the tick before: {expect} after {last}"
            );
            assert_eq!((s.a, s.b), (walker.to_bits(), ball.to_bits()));
            assert!(s.normal.y > 0.9, "the ball landed on its head: {s:?}");
            // a (the controller) relative to b (the ball) before the solver.
            let rel = expect - before.linear;
            assert!(
                near(s.relative_velocity, rel, 1e-2),
                "{s:?}, expected {rel}"
            );
            // The kinematic side takes nothing: the ball's momentum change
            // is the whole impulse, along +normal (b's side).
            let dp = 2.0 * (velocity(&w, ball).linear - before.linear);
            assert!(s.impulse > 1.0, "{s:?}");
            assert!(
                near(dp, s.normal * s.impulse, 0.02 * s.impulse),
                "Δp {dp} vs {s:?}"
            );
            return;
        }
        targets.push(transform(&w, walker).position);
    }
    panic!("the ball never met the controller");
}

#[test]
fn a_controller_hitting_a_dynamic_body_reports_its_interpolated_velocity_and_the_bodys_momentum() {
    controller_hit(PROFILE_3D);
    controller_hit(PROFILE_2D);
}
