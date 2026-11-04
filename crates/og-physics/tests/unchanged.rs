//! G2-3 adds compound colliders, sleep, new queries, joint breaking,
//! contact hooks, a drivetrain and soft volumes, every one off unless a
//! world uses it. This pins that: the beyond scenario (`support/beyond.rs`:
//! bodies, statics, a character, joints, a car, ropes, cloths, water, a
//! ragdoll and fluids, in 3D and 2D) ends at the portable hash it had
//! before G2-3 (recorded at cb1d65bf's types and fields), except for the
//! intentional adaptive terrain-contact change documented at PINNED.
//!
//! The portable hash is the one `og replay compare` checks the golden
//! replays with (og-net's `Snapshot::portable_hash`): every type and field
//! the table names, by name, and every state block's bytes, the og.physics
//! block included. G2-3's new fields (og.Physics.sleepTicks, ...) and
//! types (og.BodySlept, ...) are outside the table, so they do not move
//! it; the world hash covers every registered type's layout and moves
//! with any of them, which is why it is not what is pinned here.
//!
//! Ways this could fail: a new field or section written into a world that
//! uses none of the features (the block or a pinned field's bytes move); a
//! static collider under a parent taken for a compound child; a changed
//! iteration order in the sync; sleep off still waking the kinematic
//! bodies rapier puts to sleep by itself, which moves the block's bytes
//! once something rests long enough (`resting`: game-2d's golden replay
//! moved at tick 205 this way).

use crate::support;

use std::collections::BTreeMap;
use support::beyond::*;

/// The portable hash after TICKS ticks of `build(7, profile)`. The 3D bowl
/// uses adaptive terrain diagonals: the first fluid contact difference is
/// at tick 26; moving the bowl out of contact leaves both splits identical.
/// The 2D pin remains the one cb1d65bf left.
const PINNED: [(u32, u64); 2] = [(3, 0xc534fc05b7536866), (2, 0xd2b61993ef980a1c)];

/// What G2-3 added to the types the world registers: left out of the
/// table, as an older engine's trace leaves them.
const ADDED_TYPES: &[&str] = &[
    "og.BodySlept",
    "og.BodyWoke",
    "og.JointBroken",
    "og.ContactHook",
    "og.VehicleWheel",
    "og.SoftBody",
];
const ADDED_FIELDS: &[(&str, &str)] = &[
    ("og.Physics", "sleepTicks"),
    ("og.Physics", "sleepSpeed"),
    ("og.RigidBody", "canSleep"),
    ("og.RigidBody", "sleeping"),
    ("og.Joint", "breakForce"),
    ("og.Joint", "breakTorque"),
    ("og.Joint", "broken"),
    ("og.Vehicle", "model"),
    ("og.Vehicle", "handbrake"),
    ("og.Vehicle", "handbrakeGrip"),
    ("og.Vehicle", "driftGrip"),
    ("og.Vehicle", "gears"),
    ("og.Vehicle", "reverseRatio"),
    ("og.Vehicle", "finalDrive"),
    ("og.Vehicle", "idleRpm"),
    ("og.Vehicle", "maxRpm"),
    ("og.Vehicle", "torqueCurve"),
    ("og.Vehicle", "automatic"),
    ("og.Vehicle", "shiftUpRpm"),
    ("og.Vehicle", "shiftDownRpm"),
    ("og.Vehicle", "clutchTime"),
    ("og.Vehicle", "gear"),
    ("og.Vehicle", "rpm"),
    // Later, and as much outside the table: the GPU visual simulation mode.
    ("og.Cloth", "simulation"),
    ("og.Garment", "simulation"),
];

fn table(snapshot: &og_net::Snapshot) -> BTreeMap<String, Vec<String>> {
    let mut table = snapshot.portable_table();
    for t in ADDED_TYPES {
        table.remove(*t);
    }
    for (t, f) in ADDED_FIELDS {
        if let Some(fields) = table.get_mut(*t) {
            fields.retain(|x| x != f);
        }
    }
    table
}

#[test]
fn a_world_using_no_g2_3_feature_hashes_as_before() {
    let mut wrong = Vec::new();
    for (profile, pinned) in PINNED {
        let (mut w, p, c) = build(7, profile);
        run(&mut w, &p, &c, 0, TICKS, profile);
        let snapshot = og_net::Snapshot::decode(&w.snapshot()).expect("a world snapshot");
        let hash = snapshot.portable_hash(&table(&snapshot));
        if hash != pinned {
            wrong.push(format!("{profile}D: {hash:#018x}, pinned {pinned:#018x}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("; "));
}
/// The portable hash after 400 ticks of [`resting`], as 583b9cd5 (the
/// commit before sleep) left it.
const RESTING: [(u32, u64); 2] = [(3, 0x1cf0140cfa5b2a37), (2, 0x3dbf5270645d9208)];

/// Bodies at rest for hundreds of ticks with sleep off: a dynamic box on
/// the floor, a kinematic platform standing still, and an idle character
/// (a platformer in 2D, as game-2d's player). Rapier puts the still
/// kinematic bodies to sleep by itself, which the og.physics block
/// records; sleep off must leave that alone (game-2d's golden replay moved
/// at tick 205 when it did not).
fn resting(profile: u32) -> og_ecs::World {
    use og_core::glam::Vec3;
    use og_physics::*;
    use support::*;
    let (mut w, _p) = world(
        Physics {
            profile: if profile == 2 { PROFILE_2D } else { PROFILE_3D },
            ..Physics::default()
        },
        7,
    );
    floor(&mut w);
    spawn_body(
        &mut w,
        at(0.0, 0.5, 0.0),
        RigidBody::dynamic(),
        Collider::cuboid(Vec3::ONE),
    );
    spawn_body(
        &mut w,
        at(4.0, 1.0, 0.0),
        RigidBody::kinematic(),
        Collider::cuboid(Vec3::new(2.0, 0.2, 2.0)),
    );
    let walker = w
        .spawn((at(-4.0, 1.0, 0.0), Collider::capsule(0.3, 1.0)))
        .unwrap();
    if profile == 2 {
        w.insert(walker, PlatformerController::default()).unwrap();
    } else {
        w.insert(walker, CharacterController::default()).unwrap();
    }
    run(&mut w, 400);
    w
}

#[test]
fn bodies_resting_for_hundreds_of_ticks_with_sleep_off_hash_as_before() {
    let mut wrong = Vec::new();
    for (profile, pinned) in RESTING {
        let w = resting(profile);
        let snapshot = og_net::Snapshot::decode(&w.snapshot()).expect("a world snapshot");
        let hash = snapshot.portable_hash(&table(&snapshot));
        if hash != pinned {
            wrong.push(format!("{profile}D: {hash:#018x}, pinned {pinned:#018x}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("; "));
}
