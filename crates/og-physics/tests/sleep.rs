//! G2-3: sleep (og.Physics.sleepTicks, og.RigidBody.canSleep and
//! sleeping, og.BodySlept and og.BodyWoke) is world state.
//!
//! Ways it could fail: the rest counters live outside the og.physics block,
//! or sleep depends on an iteration order: two runs, or a run restored
//! mid-sleep, diverge.

use crate::support;

use og_ecs::World;
use og_physics::*;

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
    // pile it touches awake, as it should). The seed is one whose pile
    // settles: with others, one body rolls on past tick 900, and a change
    // to any body's bits (seed 5 stopped settling when the hulls' inertia
    // frame moved with glamx's host-independent eigensolver,
    // third_party/glamx) means picking the seed again.
    let pen = || {
        let (mut w, p) = support::pile::pen(80, 3);
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
    // The pile falls asleep whole at tick 434: at 400 its bodies are
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
