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
