//! The rapier world persists between ticks, so a restore into a world that
//! has run on must replace the newer rapier state with the snapshot's.
//!
//! Ways this could fail, caught here: the restored world keeps the newer
//! rapier state and replays differently.

use crate::support;

use og_core::glam::Vec3;
use og_physics::*;
use support::*;

#[test]
fn a_restore_into_a_world_that_ran_on_continues_from_the_snapshot() {
    let (mut w, _p) = world(Physics::default(), 17);
    floor(&mut w);
    for i in 0..20 {
        spawn_body(
            &mut w,
            at((i % 4) as f32 * 0.7 - 1.0, 1.0 + i as f32 * 1.1, 0.0),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.6)),
        );
    }
    run(&mut w, 30);
    let snap = w.snapshot();
    run(&mut w, 90);
    let end = w.snapshot();
    w.restore(&snap).unwrap();
    assert_eq!(w.snapshot(), snap);
    run(&mut w, 90);
    assert_eq!(
        w.snapshot(),
        end,
        "the rewound world replays the same 90 ticks"
    );
}
