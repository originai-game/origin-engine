//! The determinism contract of `og.anim` (docs/design.md §2.4): 20 walkers
//! with state machines, an additive layer, cross-fading players, planar root
//! motion, blendshapes, 60 IK chains, 60 joint limits, 20 look-ats, 40
//! foot locks and 20 spring bones for
//! 3600 ticks give the same
//! `world_hash` twice, and a snapshot taken at tick 1800 and restored into a
//! fresh world continues to the same hash and the same palettes.
//!
//! What would fail it: iteration in hash-map order, a wall clock or host
//! `sin`/`cos` inside the tick, player or graph state kept outside the state
//! block (a restore then replays from the wrong place), a pose or phase not
//! saved, or a palette derived from something the snapshot does not hold.
//! The walk also has to *do* something: walkers travel by root motion, the
//! state machine visits both states, footstep markers fire, feet reach,
//! heads follow their targets and are held at their angles when the target
//! goes behind.

use crate::support;

use og_anim::*;
use og_ecs::{Entity, World};
use std::collections::BTreeSet;
use support::walk::{WALKERS, build};

struct Run {
    hashes: Vec<(u64, u64)>,
    snapshot_1800: Vec<u8>,
    states_seen: BTreeSet<u32>,
    footsteps: usize,
    /// Look-at evaluations held at their angles, and free.
    looks_clamped: usize,
    looks_free: usize,
    /// Foot-lock evaluations holding a foot.
    feet_locked: usize,
    /// The largest spring lag seen.
    spring_lag: f32,
}

fn skeletons(world: &World) -> Vec<Entity> {
    let mut out = Vec::new();
    for e in world.entities() {
        if world.has::<Skeleton>(e).unwrap_or(false) {
            out.push(e);
        }
    }
    out
}

fn run(world: &mut World, from: u64, to: u64) -> Run {
    let mut r = Run {
        hashes: Vec::new(),
        snapshot_1800: Vec::new(),
        states_seen: BTreeSet::new(),
        footsteps: 0,
        looks_clamped: 0,
        looks_free: 0,
        feet_locked: 0,
        spring_lag: 0.0,
    };
    let feet: Vec<Entity> = world
        .entities()
        .into_iter()
        .filter(|&e| world.has::<FootLock>(e).unwrap_or(false))
        .collect();
    let looks: Vec<Entity> = world
        .entities()
        .into_iter()
        .filter(|&e| world.has::<LookAt>(e).unwrap_or(false))
        .collect();
    let springs: Vec<Entity> = world
        .entities()
        .into_iter()
        .filter(|&e| world.has::<SpringBone>(e).unwrap_or(false))
        .collect();
    let sk = skeletons(world);
    for tick in from..to {
        world.step().unwrap();
        r.footsteps += world
            .pending_events::<AnimationEvent>()
            .unwrap()
            .iter()
            .filter(|e| e.name.as_str().starts_with("foot"))
            .count();
        for &e in &sk {
            if let Ok(g) = world.get::<AnimationGraph>(e) {
                r.states_seen.insert(g.state);
            }
        }
        for &f in &feet {
            r.feet_locked += world.get::<FootLock>(f).unwrap().locked as usize;
        }
        for &s in &springs {
            r.spring_lag = r.spring_lag.max(world.get::<SpringBone>(s).unwrap().lag);
        }
        for &l in &looks {
            if world.get::<LookAt>(l).unwrap().clamped {
                r.looks_clamped += 1;
            } else {
                r.looks_free += 1;
            }
        }
        if (tick + 1) % 600 == 0 {
            r.hashes.push((tick + 1, world.world_hash()));
        }
        if tick + 1 == 1800 {
            r.snapshot_1800 = world.snapshot();
        }
    }
    r
}

#[test]
fn a_3600_tick_walk_gives_the_same_hash_twice_and_after_a_restore_at_1800() {
    let (mut a, anim_a) = build(7);
    let start: Vec<Transform> = skeletons(&a)
        .iter()
        .map(|&e| a.get::<Transform>(e).unwrap())
        .collect();
    // The second run goes on its own thread (a world is Send); it is the
    // same simulation either way, in about half the wall time.
    let second = std::thread::spawn(|| {
        let (mut b, _) = build(7);
        run(&mut b, 0, 3600).hashes
    });
    let ra = run(&mut a, 0, 3600);
    let rb_hashes = second.join().unwrap();
    assert_eq!(ra.hashes, rb_hashes, "two runs from one seed diverge");

    // Restore the tick-1800 snapshot into a fresh world and continue.
    let (mut c, anim_c) = build(7);
    c.restore(&ra.snapshot_1800).unwrap();
    assert_eq!(
        c.world_hash(),
        ra.hashes.iter().find(|h| h.0 == 1800).unwrap().1,
        "the restored world is not the world at tick 1800"
    );
    let rc = run(&mut c, 1800, 3600);
    assert_eq!(
        rc.hashes,
        ra.hashes
            .iter()
            .copied()
            .filter(|h| h.0 > 1800)
            .collect::<Vec<_>>(),
        "the restored world continues differently"
    );
    let (sa, sc) = (skeletons(&a), skeletons(&c));
    assert_eq!(sa, sc);
    for (&ea, &ec) in sa.iter().zip(&sc) {
        let (oa, oc) = (anim_a.output(ea).unwrap(), anim_c.output(ec).unwrap());
        assert_eq!(oa, oc, "palettes differ after the restore");
        assert_eq!(oa.palette.len(), 12);
    }

    // The walk did something.
    assert_eq!(ra.states_seen, BTreeSet::from([0, 1]), "idle and move");
    assert!(
        ra.footsteps > 200,
        "footstep markers: {} in 3600 ticks",
        ra.footsteps
    );
    let mut travelled = Vec::new();
    for (i, &e) in sa.iter().enumerate() {
        let t = a.get::<Transform>(e).unwrap();
        travelled.push((t.position - start[i].position).length());
    }
    assert!(
        travelled.iter().all(|&d| d > 3.0),
        "root motion moved every walker: {travelled:?}"
    );
    let mut reached = 0;
    let mut chains = 0;
    for e in a.entities() {
        if let Ok(c) = a.get::<IkChain>(e) {
            chains += 1;
            reached += c.reached as usize;
        }
    }
    assert_eq!(chains, 3 * WALKERS as usize);
    assert!(
        reached >= 2 * WALKERS as usize,
        "{reached} of {chains} chains reached their targets"
    );
    assert!(
        ra.looks_clamped > 10_000 && ra.looks_free > 10_000,
        "heads held at their angles {} times, free {} times",
        ra.looks_clamped,
        ra.looks_free
    );
    assert!(ra.feet_locked > 1000, "feet held {} times", ra.feet_locked);
    assert!(
        ra.spring_lag > 0.02,
        "springs lag {} m at most",
        ra.spring_lag
    );

    // Evidence: the hashes and what the walk did.
    let dir = std::env::var_os("OG_EVIDENCE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    std::fs::create_dir_all(&dir).unwrap();
    let hashes: Vec<String> = ra
        .hashes
        .iter()
        .map(|(t, h)| format!("{{\"tick\":{t},\"world_hash\":\"{h:016x}\"}}"))
        .collect();
    let restored: Vec<String> = rc
        .hashes
        .iter()
        .map(|(t, h)| format!("{{\"tick\":{t},\"world_hash\":\"{h:016x}\"}}"))
        .collect();
    let report = format!(
        "{{\n  \"walkers\": {WALKERS},\n  \"ik_chains\": {chains},\n  \"ticks\": 3600,\n  \"profile\": \"{}\",\n  \"run_a\": [{}],\n  \"run_b_equal\": true,\n  \"restored_at_1800\": [{}],\n  \"graph_states_seen\": {:?},\n  \"footstep_events\": {},\n  \"chains_reached_at_3600\": {reached},\n  \"min_root_motion_travel_m\": {:.3}\n}}\n",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        hashes.join(", "),
        restored.join(", "),
        ra.states_seen,
        ra.footsteps,
        travelled.iter().cloned().fold(f32::MAX, f32::min),
    );
    std::fs::write(dir.join("anim-walk-determinism.json"), report).unwrap();
}
