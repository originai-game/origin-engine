//! G2-3 adds compound colliders, sleep, new queries, joint breaking,
//! contact hooks, a drivetrain and soft volumes, every one off unless a
//! world uses it. This pins that: the beyond scenario (`support/beyond.rs`:
//! bodies, statics, a character, joints, a car, ropes, cloths, water, a
//! ragdoll and fluids, in 3D and 2D) ends at the portable hash it had
//! before G2-3 (recorded at cb1d65bf's types and fields).
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
//! iteration order in the sync.

use crate::support;

use std::collections::BTreeMap;
use support::beyond::*;

/// The portable hash after TICKS ticks of `build(7, profile)`, as cb1d65bf
/// left it.
const PINNED: [(u32, u64); 2] = [(3, 0xaae516abb826b9be), (2, 0xd2b61993ef980a1c)];

/// What G2-3 added to the types the world registers: left out of the
/// table, as an older engine's trace leaves them.
const ADDED_TYPES: &[&str] = &[];
const ADDED_FIELDS: &[(&str, &str)] = &[];

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
