//! G2-3: joints that break (og.Joint.breakForce, breakTorque, broken;
//! og.JointBroken) and contact hooks (og.ContactHook).
//!
//! Ways they could fail, and the test that catches each:
//! - a joint breaks under a load below its limit, never breaks, breaks
//!   but keeps holding, sends the event twice or with the wrong entities,
//!   or cannot be mended → `a_joint_breaks_past_its_force_and_is_mended`
//! - the torque limit is ignored or read as a force →
//!   `a_cantilever_breaks_past_its_torque`
//! - a broken joint keeps its bodies from colliding with each other, as
//!   it did while it held → `bodies_whose_joint_broke_collide_with_each_other_again`
//! - breaking depends on anything but the world: a restore before the
//!   break breaks it at another tick or force →
//!   `a_break_replays_from_a_restore`
//! - a one-way surface blocks from below, or lets through from above, in
//!   either profile → `a_one_way_platform_passes_from_below_and_holds_from_above`
//! - a conveyor's sign depends on which collider rapier calls first →
//!   `a_conveyor_carries_what_lies_on_it_whichever_was_spawned_first`
//! - friction, restitution or ignore never reach the solver →
//!   `friction_restitution_and_ignore_override_the_pair`
//! - hooks change a world after a restore (the hook flags or table lost)
//!   → `hooks_replay_from_a_restore`

use crate::support;

use og_core::List;
use og_core::glam::Vec3;
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// A world with physics and G2-3's lazy types registered (a game naming them).
fn world(settings: Physics, seed: u32) -> (World, PhysicsSystem) {
    let (mut w, p) = support::world(settings, seed);
    support::enable_lazy(&mut w);
    (w, p)
}

fn broken_events(w: &World) -> Vec<JointBroken> {
    w.pending_events::<JointBroken>().unwrap()
}

/// A `mass` kg ball hanging 1 m under a world point on a ball joint.
fn hang(w: &mut World, mass: f32, break_force: f32) -> Entity {
    let bob = spawn_body(
        w,
        at(0.0, 4.0, 0.0),
        RigidBody {
            mass,
            ..RigidBody::dynamic()
        },
        Collider::sphere(0.2),
    );
    w.insert(
        bob,
        Joint {
            kind: JOINT_BALL,
            anchor: Vec3::new(0.0, 1.0, 0.0),
            other_anchor: Vec3::new(0.0, 5.0, 0.0),
            break_force,
            ..Joint::default()
        },
    )
    .unwrap();
    bob
}

#[test]
fn a_joint_breaks_past_its_force_and_is_mended() {
    // 10 kg pulls about 98 N: a 200 N joint holds it for good.
    let (mut w, _p) = world(Physics::default(), 1);
    let light = hang(&mut w, 10.0, 200.0);
    for _ in 0..300 {
        w.step().unwrap();
        assert!(broken_events(&w).is_empty());
    }
    assert!((transform(&w, light).position.y - 4.0).abs() < 0.01);
    assert!(!w.get::<Joint>(light).unwrap().broken);

    // 30 kg pulls about 294 N: it breaks at once, once, and the ball falls.
    let (mut w, _p) = world(Physics::default(), 1);
    let heavy = hang(&mut w, 30.0, 200.0);
    let mut events = Vec::new();
    for _ in 0..60 {
        w.step().unwrap();
        events.extend(broken_events(&w));
    }
    assert_eq!(events.len(), 1, "{events:?}");
    let e = events[0];
    assert_eq!(
        (e.joint, e.body, e.other),
        (heavy.to_bits(), heavy.to_bits(), 0)
    );
    assert!(
        e.force > 200.0 && e.force < 2000.0,
        "broke at {} N",
        e.force
    );
    assert!(w.get::<Joint>(heavy).unwrap().broken);
    assert!(
        transform(&w, heavy).position.y < 0.0,
        "a broken joint still held"
    );

    // Mended at a lighter load, it holds again where the ball is now.
    let y = transform(&w, heavy).position.y;
    let mended = Joint {
        broken: false,
        break_force: 0.0,
        other_anchor: Vec3::new(0.0, y + 1.0, 0.0),
        ..w.get::<Joint>(heavy).unwrap()
    };
    w.insert(heavy, mended).unwrap();
    w.insert(heavy, Velocity::default()).unwrap();
    for _ in 0..60 {
        w.step().unwrap();
    }
    assert!((transform(&w, heavy).position.y - y).abs() < 0.05);
}

#[test]
fn a_cantilever_breaks_past_its_torque() {
    // A 10 kg block held level 1 m out by a fixed joint at the origin: its
    // weight is about 98 N·m about the anchor.
    let arm = |break_torque: f32| {
        let (mut w, _p) = world(Physics::default(), 1);
        let block = spawn_body(
            &mut w,
            at(1.0, 5.0, 0.0),
            RigidBody {
                mass: 10.0,
                ..RigidBody::dynamic()
            },
            Collider::cuboid(Vec3::splat(0.2)),
        );
        w.insert(
            block,
            Joint {
                kind: JOINT_FIXED,
                anchor: Vec3::new(-1.0, 0.0, 0.0),
                other_anchor: Vec3::new(0.0, 5.0, 0.0),
                break_torque,
                ..Joint::default()
            },
        )
        .unwrap();
        let mut events = Vec::new();
        for _ in 0..120 {
            w.step().unwrap();
            events.extend(broken_events(&w));
        }
        events
    };
    assert!(arm(200.0).is_empty(), "a 200 N·m joint broke");
    let events = arm(50.0);
    assert_eq!(events.len(), 1);
    assert!(
        events[0].torque > 50.0 && events[0].torque < 500.0,
        "{events:?}"
    );
}

#[test]
fn a_break_replays_from_a_restore() {
    // 10 kg on a 200 N joint under gravity growing by 2 m/s² a tick from
    // tick 40: a load held through each step, which breaks it.
    let run = |w: &mut World, bob: Entity, from: u32| {
        let mut events = Vec::new();
        for t in from..120 {
            if t >= 40 {
                w.resource_mut::<Physics>().unwrap().gravity.y = -9.81 - 2.0 * (t - 39) as f32;
            }
            w.step().unwrap();
            for e in broken_events(w) {
                assert_eq!(e.joint, bob.to_bits());
                events.push((t, e.force.to_bits(), e.torque.to_bits()));
            }
        }
        (events, w.world_hash())
    };
    let (mut a, _pa) = world(Physics::default(), 1);
    let bob = hang(&mut a, 10.0, 200.0);
    for _ in 0..30 {
        a.step().unwrap();
    }
    let saved = a.snapshot();
    let first = run(&mut a, bob, 30);
    assert_eq!(first.0.len(), 1, "{:?}", first.0);

    let (mut b, _pb) = world(Physics::default(), 1);
    hang(&mut b, 10.0, 200.0);
    b.restore(&saved).unwrap();
    assert_eq!(run(&mut b, bob, 30), first);
}

/// A 4 × 0.2 × 4 platform at y = 0 with `hook`, a floor 3 m below, and a
/// ball of radius 0.25 at `y` moving at `vy`.
fn platform(w: &mut World, hook: ContactHook, y: f32, vy: f32, two_d: bool) -> Entity {
    let size = if two_d {
        Vec3::new(4.0, 0.2, 1.0)
    } else {
        Vec3::new(4.0, 0.2, 4.0)
    };
    spawn_static(
        w,
        at(0.0, -3.0, 0.0),
        Collider::cuboid(Vec3::new(40.0, 1.0, 40.0)),
    );
    let p = spawn_static(w, at(0.0, 0.0, 0.0), Collider::cuboid(size));
    w.insert(p, hook).unwrap();
    let ball = spawn_body(
        w,
        at(0.0, y, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.25),
    );
    w.insert(
        ball,
        Velocity {
            linear: Vec3::new(0.0, vy, 0.0),
            angular: Vec3::ZERO,
        },
    )
    .unwrap();
    ball
}

#[test]
fn a_one_way_platform_passes_from_below_and_holds_from_above() {
    for (profile, two_d) in [(Physics::default(), false), (Physics::profile_2d(), true)] {
        let up = ContactHook {
            one_way: Vec3::Y,
            ..ContactHook::default()
        };
        // Thrown up from under it: through, then lands on top.
        let (mut w, _p) = world(profile, 1);
        let ball = platform(&mut w, up, -1.0, 9.0, two_d);
        run(&mut w, 240);
        let y = transform(&w, ball).position.y;
        assert!(
            (y - 0.35).abs() < 0.05,
            "{}D: the ball ended at {y}",
            if two_d { 2 } else { 3 }
        );

        // Without the hook the platform stops it from below: it falls to the floor.
        let (mut w, _p) = world(profile, 1);
        let ball = platform(&mut w, ContactHook::default(), -1.0, 9.0, two_d);
        run(&mut w, 240);
        assert!(transform(&w, ball).position.y < -2.0);

        // Dropped on it from above, the one-way platform holds it.
        let (mut w, _p) = world(profile, 1);
        let ball = platform(&mut w, up, 2.0, 0.0, two_d);
        run(&mut w, 240);
        assert!((transform(&w, ball).position.y - 0.35).abs() < 0.05);
    }
}

#[test]
fn a_conveyor_carries_what_lies_on_it_whichever_was_spawned_first() {
    let belt = ContactHook {
        surface_velocity: Vec3::new(1.0, 0.0, 0.0),
        ..ContactHook::default()
    };
    let cube = || Collider::cuboid(Vec3::splat(0.5));
    for box_first in [false, true] {
        let (mut w, _p) = world(Physics::default(), 1);
        let early =
            box_first.then(|| spawn_body(&mut w, at(0.0, 0.25, 0.0), RigidBody::dynamic(), cube()));
        let f = floor(&mut w);
        w.insert(f, belt).unwrap();
        let block = early.unwrap_or_else(|| {
            spawn_body(&mut w, at(0.0, 0.25, 0.0), RigidBody::dynamic(), cube())
        });
        run(&mut w, 120);
        let v = velocity(&w, block).linear;
        assert!(
            (v.x - 1.0).abs() < 0.1 && v.z.abs() < 0.05,
            "box first {box_first}: {v}"
        );
    }
}

#[test]
fn friction_restitution_and_ignore_override_the_pair() {
    // A box slid at 4 m/s on a floor of friction 0 keeps going; on the
    // default floor it stops.
    let slide = |hook: Option<ContactHook>| {
        let (mut w, _p) = world(Physics::default(), 1);
        let f = floor(&mut w);
        if let Some(h) = hook {
            w.insert(f, h).unwrap();
        }
        let b = spawn_body(
            &mut w,
            at(0.0, 0.25, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.5)),
        );
        w.insert(
            b,
            Velocity {
                linear: Vec3::new(4.0, 0.0, 0.0),
                angular: Vec3::ZERO,
            },
        )
        .unwrap();
        run(&mut w, 60);
        velocity(&w, b).linear.x
    };
    assert!(slide(None) < 0.5);
    assert!(
        slide(Some(ContactHook {
            friction: 0.0,
            ..ContactHook::default()
        })) > 3.9
    );

    // A ball dropped from 2 m on a floor of restitution 1 comes back up most of the way.
    let bounce = |restitution: f32| {
        let (mut w, _p) = world(Physics::default(), 1);
        let f = floor(&mut w);
        w.insert(
            f,
            ContactHook {
                restitution,
                ..ContactHook::default()
            },
        )
        .unwrap();
        let b = spawn_body(
            &mut w,
            at(0.0, 2.0, 0.0),
            RigidBody::dynamic(),
            Collider::sphere(0.25),
        );
        let mut top: f32 = 0.0;
        for t in 0..120 {
            w.step().unwrap();
            if t > 50 {
                top = top.max(transform(&w, b).position.y);
            }
        }
        top
    };
    assert!(bounce(1.0) > 1.5);
    assert!(bounce(-1.0) < 0.5);

    // A ball whose hook ignores the floor falls through it.
    let (mut w, _p) = world(Physics::default(), 1);
    let f = floor(&mut w);
    let b = spawn_body(
        &mut w,
        at(0.0, 1.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.25),
    );
    let mut ignore = List::new();
    ignore.push(f.to_bits()).unwrap();
    w.insert(
        b,
        ContactHook {
            ignore,
            ..ContactHook::default()
        },
    )
    .unwrap();
    run(&mut w, 60);
    assert!(transform(&w, b).position.y < -1.0);
}

#[test]
fn hooks_replay_from_a_restore() {
    let build = || {
        let (mut w, p) = world(Physics::default(), 1);
        let f = floor(&mut w);
        w.insert(
            f,
            ContactHook {
                surface_velocity: Vec3::new(0.5, 0.0, 0.5),
                friction: 0.8,
                ..ContactHook::default()
            },
        )
        .unwrap();
        for k in 0..4 {
            spawn_body(
                &mut w,
                at(k as f32 * 0.6, 0.5 + k as f32, 0.0),
                RigidBody::dynamic(),
                Collider::sphere(0.25),
            );
        }
        (w, p)
    };
    let (mut a, _pa) = build();
    run(&mut a, 30);
    let saved = a.snapshot();
    let hashes = |w: &mut World| {
        (0..90)
            .map(|_| {
                w.step().unwrap();
                w.world_hash()
            })
            .collect::<Vec<_>>()
    };
    let first = hashes(&mut a);
    let (mut b, _pb) = build();
    b.restore(&saved).unwrap();
    assert_eq!(hashes(&mut b), first);
}

#[test]
fn a_world_naming_none_of_g2_3s_types_registers_none_and_still_sleeps_and_breaks() {
    // G2-3's events and og.ContactHook are lazy: unregistered until a game
    // names them, so an older world's type table and hash stay as they
    // were. The features that write components work without them.
    let (mut w, _p) = support::world(
        Physics {
            sleep_ticks: 30,
            ..Physics::default()
        },
        1,
    );
    floor(&mut w);
    let resting = spawn_body(
        &mut w,
        at(-3.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let heavy = hang(&mut w, 30.0, 200.0);
    run(&mut w, 240);
    for name in og_physics::LAZY {
        assert!(w.id_by_name(name).is_err(), "{name} was registered");
    }
    assert!(w.get::<RigidBody>(resting).unwrap().sleeping);
    assert!(w.get::<Joint>(heavy).unwrap().broken);
    // Named later, they are sent from then on, and hooks are read.
    support::enable_lazy(&mut w);
    w.insert(
        heavy,
        Joint {
            broken: false,
            ..w.get::<Joint>(heavy).unwrap()
        },
    )
    .unwrap();
    let mut sent = 0;
    for _ in 0..120 {
        w.step().unwrap();
        sent += broken_events(&w).len();
    }
    assert_eq!(sent, 1);
}

#[test]
fn bodies_whose_joint_broke_collide_with_each_other_again() {
    // A box on the floor and a second box jointed into it, overlapping by
    // 0.6 m, the joint not colliding (the default): held, the upper box
    // rests inside the lower. Its joint breaks under the first load; from
    // then on the pair collides, so the upper box is pushed out and comes
    // to rest stacked on the lower, not sunk through it to the floor.
    for profile in [PROFILE_3D, PROFILE_2D] {
        let (mut w, _p) = world(
            Physics {
                profile,
                ..Physics::default()
            },
            1,
        );
        floor(&mut w);
        let lower = spawn_body(
            &mut w,
            at(0.0, 0.5, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::ONE),
        );
        let upper = spawn_body(
            &mut w,
            at(0.0, 0.9, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::ONE),
        );
        w.insert(
            upper,
            Joint {
                kind: JOINT_FIXED,
                other: lower.to_bits(),
                other_anchor: Vec3::new(0.0, 0.4, 0.0),
                break_force: 1.0,
                ..Joint::default()
            },
        )
        .unwrap();
        let mut broke = 0;
        for _ in 0..180 {
            w.step().unwrap();
            broke += broken_events(&w).len();
        }
        assert_eq!(broke, 1, "profile {profile}");
        let gap = transform(&w, upper).position.y - transform(&w, lower).position.y;
        assert!(
            gap > 0.95,
            "profile {profile}: the freed box rests {gap} m above the other, inside it"
        );
    }
}
