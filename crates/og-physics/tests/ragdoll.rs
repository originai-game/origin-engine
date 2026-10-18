//! og.Ragdoll over a registered rig (3D): the bones follow the animation's
//! targets while the weight is 0, fall as joined bodies while it is 1, and
//! the blend moves between them over blendTime both ways.
//!
//! Ways this could fail, each caught below: the bodies ignore the targets
//! (the ragdoll lags or stays at its rest pose); going limp does not make
//! them dynamic (nothing falls) or loses the joints (the bones scatter);
//! the blend jumps instead of ramping, or never returns; recovering leaves
//! the bodies dynamic (they do not follow the targets again); the bones
//! collide with each other (the chain explodes on creation).

mod support;

use og_core::glam::{Quat, Vec3};
use og_ecs::{Entity, World};
use og_physics::*;
use support::*;

/// Hips at 1 m, a 0.5 m spine and a 0.3 m neck, straight up.
fn rig() -> RagdollRig {
    let bone = |parent: Option<u32>, y: f32, length: f32| RagdollBone {
        parent,
        rest: (Vec3::new(0.0, y, 0.0), Quat::IDENTITY),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, 1.0, 0.3),
            bone(Some(0), 1.3, 0.5),
            bone(Some(1), 1.8, 0.3),
        ],
    }
}

fn targets(sway: f32) -> Vec<(Vec3, Quat)> {
    rig()
        .bones
        .iter()
        .map(|b| {
            (
                b.rest.0 + Vec3::new(sway * b.rest.0.y, 0.0, 0.0),
                Quat::IDENTITY,
            )
        })
        .collect()
}

fn ragdoll(w: &World, e: Entity) -> Ragdoll {
    w.get::<Ragdoll>(e).unwrap()
}

fn set_weight(w: &mut World, e: Entity, weight: f32) {
    let mut r = ragdoll(w, e);
    r.weight = weight;
    w.insert(e, r).unwrap();
}

#[test]
fn a_ragdoll_follows_its_animation_goes_limp_and_recovers_with_a_blend() {
    let (mut w, p) = world(Physics::default(), 1);
    floor(&mut w);
    let body = w.spawn((at(0.0, 0.0, 0.0), Ragdoll::default())).unwrap();
    p.set_ragdoll_rig(body, rig()).unwrap();
    // Following: the bones are where the animation puts them.
    for tick in 0..30 {
        let sway = 0.1 * (tick as f32 / 30.0);
        p.set_ragdoll_targets(body, targets(sway));
        w.step().unwrap();
        w.step().unwrap();
        let pose = p.ragdoll_pose(body).unwrap();
        for (b, t) in pose.iter().zip(targets(sway)) {
            assert!(
                (b.0 - t.0).length() < 1e-3,
                "bone at {} for target {}",
                b.0,
                t.0
            );
        }
    }
    assert_eq!(ragdoll(&w, body).blend, 0.0);
    // Limp: the blend ramps over 0.25 s (15 ticks) and the bones fall, joined.
    set_weight(&mut w, body, 1.0);
    w.step().unwrap();
    let b1 = ragdoll(&w, body).blend;
    assert!(b1 > 0.05 && b1 < 0.1, "one tick into the blend: {b1}");
    run(&mut w, 15);
    assert_eq!(ragdoll(&w, body).blend, 1.0);
    run(&mut w, 120);
    let pose = p.ragdoll_pose(body).unwrap();
    assert!(pose[2].0.y < 1.0, "the head fell to {}", pose[2].0);
    for (i, b) in rig().bones.iter().enumerate().skip(1) {
        let parent = &pose[b.parent.unwrap() as usize];
        let tip = parent.0 + parent.1 * (Vec3::Y * rig().bones[b.parent.unwrap() as usize].length);
        let gap = (pose[i].0 - tip).length();
        assert!(gap < 0.05, "bone {i} came off its parent by {gap}");
    }
    // Recover: the blend returns over 0.25 s, then the bones follow again.
    set_weight(&mut w, body, 0.0);
    run(&mut w, 8);
    let mid = ragdoll(&w, body).blend;
    assert!(mid > 0.3 && mid < 0.7, "half way back: {mid}");
    run(&mut w, 10);
    assert_eq!(ragdoll(&w, body).blend, 0.0);
    p.set_ragdoll_targets(body, targets(0.0));
    run(&mut w, 2);
    let pose = p.ragdoll_pose(body).unwrap();
    for (b, t) in pose.iter().zip(targets(0.0)) {
        assert!(
            (b.0 - t.0).length() < 1e-3,
            "recovered bone at {} for {}",
            b.0,
            t.0
        );
    }
}
