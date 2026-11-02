//! G2-3: sleep (og.Physics.sleepTicks, og.RigidBody.canSleep and
//! sleeping, og.BodySlept and og.BodyWoke).
//!
//! Ways it could fail, and the test that catches each:
//! - nothing sleeps, a body sleeps alone while what it touches is awake
//!   (and what rests on it sinks through it), a sleeping body drifts, or
//!   the events are missing, repeated or out of entity order →
//!   `a_settled_stack_sleeps_whole_and_stays_put`
//! - sleep happens with sleepTicks 0, or canSleep false is ignored for the
//!   body or what touches it → `sleep_is_off_by_default_and_per_body`
//! - something that should wake a body does not (a body landing on it, a
//!   push, game code writing `sleeping` or its Transform, the floor going
//!   away, a kinematic body moving into it, sleep turned off), or the wake
//!   lets bodies pass through each other →
//!   `what_wakes_a_sleeping_body`
//! - the rest counters live outside the og.physics block, or sleep depends
//!   on an iteration order: two runs, or a run restored mid-sleep, diverge
//!   → `a_pen_sleeps_deterministically_and_restores_mid_sleep`

use crate::support;

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

const SLEEP: Physics = Physics {
    profile: PROFILE_3D,
    gravity: Vec3::new(0.0, -9.81, 0.0),
    substeps: 1,
    solver_iterations: 4,
    rest_speed: 0.02,
    sleep_ticks: 30,
    sleep_speed: 0.1,
};

fn sleeping(w: &World, e: Entity) -> bool {
    w.get::<RigidBody>(e).unwrap().sleeping
}

/// Steps one tick; the entities that fell asleep and woke during it.
fn step(w: &mut World) -> (Vec<u64>, Vec<u64>) {
    w.step().unwrap();
    let slept = w.pending_events::<BodySlept>().unwrap();
    let woke = w.pending_events::<BodyWoke>().unwrap();
    (
        slept.iter().map(|e| e.entity).collect(),
        woke.iter().map(|e| e.entity).collect(),
    )
}

/// A floor and three unit boxes stacked on it, the lowest spawned last
/// (so entity order is not stacking order).
fn stack(w: &mut World) -> [Entity; 3] {
    floor(w);
    let cube = || Collider::cuboid(Vec3::ONE);
    let top = spawn_body(w, at(0.0, 2.5, 0.0), RigidBody::dynamic(), cube());
    let mid = spawn_body(w, at(0.0, 1.5, 0.0), RigidBody::dynamic(), cube());
    let low = spawn_body(w, at(0.0, 0.5, 0.0), RigidBody::dynamic(), cube());
    [low, mid, top]
}

/// Steps until every body of `bodies` sleeps; the tick it happened and
/// every BodySlept entity sent until then.
fn until_asleep(w: &mut World, bodies: &[Entity], limit: u32) -> (u32, Vec<Vec<u64>>) {
    let mut sent = Vec::new();
    for tick in 1..=limit {
        let (slept, woke) = step(w);
        assert!(woke.is_empty(), "woke {woke:?} at tick {tick}");
        if !slept.is_empty() {
            sent.push(slept);
        }
        if bodies.iter().all(|&e| sleeping(w, e)) {
            return (tick, sent);
        }
    }
    panic!("not asleep after {limit} ticks");
}

#[test]
fn a_settled_stack_sleeps_whole_and_stays_put() {
    let (mut w, _p) = world(SLEEP, 1);
    let boxes = stack(&mut w);
    let (tick, sent) = until_asleep(&mut w, &boxes, 600);
    // The whole stack in one tick, in entity order: no box sleeps under or
    // over an awake one.
    let mut expected: Vec<u64> = boxes.iter().map(|e| e.to_bits()).collect();
    expected.sort_unstable();
    assert_eq!(sent, vec![expected], "BodySlept by tick {tick}");
    assert!(tick >= 30, "asleep after {tick} ticks, before sleepTicks");

    // Asleep: not a bit of drift, no velocity, no more events.
    let at: Vec<Transform> = boxes.iter().map(|&e| transform(&w, e)).collect();
    for _ in 0..300 {
        assert_eq!(step(&mut w), (vec![], vec![]));
    }
    for (k, &e) in boxes.iter().enumerate() {
        assert_eq!(transform(&w, e), at[k], "box {k} moved asleep");
        assert_eq!(velocity(&w, e), Velocity::default());
        assert!(sleeping(&w, e));
    }
    assert!(
        (at[2].position.y - 2.5).abs() < 0.01,
        "top at {}",
        at[2].position.y
    );
}

#[test]
fn sleep_is_off_by_default_and_per_body() {
    // sleepTicks 0: a settled stack never sleeps.
    let (mut w, _p) = world(Physics::default(), 1);
    let boxes = stack(&mut w);
    for _ in 0..400 {
        assert_eq!(step(&mut w), (vec![], vec![]));
    }
    assert!(boxes.iter().all(|&e| !sleeping(&w, e)));

    // canSleep false on the middle box keeps the whole stack awake, while a
    // box resting apart sleeps.
    let (mut w, _p) = world(SLEEP, 1);
    let boxes = stack(&mut w);
    w.insert(
        boxes[1],
        RigidBody {
            can_sleep: false,
            ..RigidBody::dynamic()
        },
    )
    .unwrap();
    let apart = spawn_body(
        &mut w,
        at(5.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    let (_, sent) = until_asleep(&mut w, &[apart], 600);
    assert_eq!(sent, vec![vec![apart.to_bits()]]);
    for _ in 0..300 {
        step(&mut w);
    }
    assert!(
        boxes.iter().all(|&e| !sleeping(&w, e)),
        "a stack with a canSleep false box slept"
    );
}

/// The stack asleep, `poke` done to it, then 120 ticks: the entities that
/// woke (in order, each tick's) and the world.
fn poke(f: impl Fn(&mut World, &PhysicsSystem, [Entity; 3])) -> (Vec<u64>, World, [Entity; 3]) {
    let (mut w, p) = world(SLEEP, 1);
    let boxes = stack(&mut w);
    until_asleep(&mut w, &boxes, 600);
    f(&mut w, &p, boxes);
    let mut woke = Vec::new();
    for _ in 0..120 {
        woke.extend(step(&mut w).1);
    }
    (woke, w, boxes)
}

fn all_woke(woke: &[u64], boxes: [Entity; 3]) -> bool {
    boxes.iter().all(|e| woke.contains(&e.to_bits()))
}

#[test]
fn what_wakes_a_sleeping_body() {
    // A ball dropped on the stack wakes it, and lands on it, not in it.
    let (woke, w, boxes) = poke(|w, _, _| {
        spawn_body(
            w,
            at(0.0, 6.0, 0.0),
            RigidBody::dynamic(),
            Collider::sphere(0.3),
        );
    });
    assert!(all_woke(&woke, boxes), "a landing ball woke {woke:?}");
    assert!(
        transform(&w, boxes[2]).position.y > 2.4,
        "the stack collapsed"
    );

    // An impulse on the lowest box wakes it (and the stack it holds up),
    // and moves it.
    let (woke, w, boxes) = poke(|_, p, b| {
        p.apply_impulse(b[0], Vec3::new(3.0, 0.0, 0.0), None)
            .unwrap()
    });
    assert!(all_woke(&woke, boxes), "an impulse woke {woke:?}");
    assert!(transform(&w, boxes[0]).position.x > 0.1);

    // Game code writing sleeping false wakes it.
    let (woke, _, boxes) = poke(|w, _, b| {
        let body = w.get::<RigidBody>(b[2]).unwrap();
        w.insert(
            b[2],
            RigidBody {
                sleeping: false,
                ..body
            },
        )
        .unwrap();
    });
    assert!(
        woke.contains(&boxes[2].to_bits()),
        "writing sleeping woke {woke:?}"
    );

    // A teleport wakes the moved box; it falls from where it was put.
    let (woke, w, boxes) = poke(|w, _, b| w.insert(b[2], at(3.0, 4.0, 0.0)).unwrap());
    assert!(
        woke.contains(&boxes[2].to_bits()),
        "a teleport woke {woke:?}"
    );
    assert!((transform(&w, boxes[2]).position.y - 0.5).abs() < 0.05);

    // The floor going away wakes the stack, which falls.
    let (woke, w, boxes) = poke(|w, _, _| {
        let floor = w
            .entities()
            .into_iter()
            .find(|&e| w.get::<Collider>(e).is_ok() && w.get::<RigidBody>(e).is_err())
            .unwrap();
        w.despawn(floor).unwrap();
    });
    assert!(all_woke(&woke, boxes), "the floor going woke {woke:?}");
    assert!(transform(&w, boxes[0]).position.y < -1.0);

    // A kinematic paddle moving into the stack wakes it.
    let (woke, _, boxes) = poke(|w, _, _| {
        let paddle = spawn_body(
            w,
            at(-3.0, 0.5, 0.0),
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::new(1.0, 1.0, 3.0)),
        );
        w.insert(
            paddle,
            Velocity {
                linear: Vec3::new(2.0, 0.0, 0.0),
                angular: Vec3::ZERO,
            },
        )
        .unwrap();
    });
    assert!(woke.contains(&boxes[0].to_bits()), "a paddle woke {woke:?}");

    // Turning sleep off wakes everything.
    let (woke, _, boxes) = poke(|w, _, _| {
        w.resource_mut::<Physics>().unwrap().sleep_ticks = 0;
    });
    assert!(all_woke(&woke, boxes), "sleep off woke {woke:?}");
}

/// Hashes and BodySlept/BodyWoke counts of the pen, ticks `from..to`.
fn pen_run(w: &mut World, from: u32, to: u32) -> (Vec<u64>, usize, usize) {
    let (mut hashes, mut slept, mut woke) = (Vec::new(), 0, 0);
    for _ in from..to {
        let (s, k) = step(w);
        slept += s.len();
        woke += k.len();
        hashes.push(w.world_hash());
    }
    (hashes, slept, woke)
}

fn asleep(w: &World) -> usize {
    w.entities()
        .into_iter()
        .filter(|&e| w.get::<RigidBody>(e).is_ok_and(|b| b.sleeping))
        .count()
}

#[test]
fn a_pen_sleeps_deterministically_and_restores_mid_sleep() {
    // 80 bodies dropped into a pen, damped so that rolling capsules and
    // spheres come to rest (undamped, one rolls for ever and keeps the
    // pile it touches awake, as it should).
    let pen = || {
        let (mut w, p) = support::pile::pen(80, 5);
        support::enable_lazy(&mut w);
        *w.resource_mut::<Physics>().unwrap() = Physics {
            sleep_ticks: 30,
            ..Physics::default()
        };
        for e in w.entities() {
            if let Ok(b) = w.get::<RigidBody>(e) {
                let damped = RigidBody {
                    linear_damping: 0.2,
                    angular_damping: 1.0,
                    ..b
                };
                w.insert(e, damped).unwrap();
            }
        }
        (w, p)
    };
    let (mut a, _pa) = pen();
    // The pile falls asleep whole at tick 412: at 400 its bodies are
    // counting their calm ticks, which a restore must carry.
    let first = pen_run(&mut a, 0, 400);
    let counting = a.snapshot();
    let second = pen_run(&mut a, 400, 600);
    let slept = a.snapshot();
    let third = pen_run(&mut a, 600, 900);
    assert!(
        second.1 >= 70,
        "{} BodySlept by tick 600",
        first.1 + second.1
    );
    assert!(asleep(&a) >= 70, "{} asleep at tick 900", asleep(&a));

    // A second run is the same, tick for tick.
    let (mut b, _pb) = pen();
    assert_eq!(pen_run(&mut b, 0, 400), first);
    assert_eq!(pen_run(&mut b, 400, 600), second);
    assert_eq!(pen_run(&mut b, 600, 900), third);

    // Restored while bodies count their rest (tick 400) and while the pile
    // sleeps (tick 600), a world goes on as the original did.
    let (mut c, _pc) = pen();
    c.restore(&counting).unwrap();
    assert_eq!(asleep(&c), 0);
    assert_eq!(pen_run(&mut c, 400, 600), second);
    let (mut d, _pd) = pen();
    d.restore(&slept).unwrap();
    assert_eq!(pen_run(&mut d, 600, 900), third);
}
