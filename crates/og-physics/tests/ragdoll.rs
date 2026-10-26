//! og.Ragdoll over a registered rig, in 3D and in 2D: the bones follow the
//! animation's targets while the weight is 0, fall as joined bodies while
//! it is 1, and the blend moves between them over blendTime both ways. In
//! 2D the bones are rapier2d bodies on hinges about z: they stay in their
//! plane at the targets' z, turn only about z, and the knee keeps to its
//! joint limit.
//!
//! Ways this could fail, each caught below: the bodies ignore the targets
//! (the ragdoll lags or stays at its rest pose); going limp does not make
//! them dynamic (nothing falls) or loses the joints (the bones scatter);
//! the blend jumps instead of ramping, or never returns; recovering leaves
//! the bodies dynamic (they do not follow the targets again); the bones
//! collide with each other (the chain explodes on creation); in 2D, the z
//! of the animation lost (bones drawn in the wrong layer), a 3D rotation
//! leaking in, or the hinges without limits (a knee folding backwards).

use crate::support;

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

/// A side-on 2D figure: pelvis, torso and head up, a thigh and shin down
/// (turned half a turn about z, so their +y points down), in the plane z = 0.3.
fn rig_2d() -> RagdollRig {
    let down = Quat::from_xyzw(0.0, 0.0, 1.0, 0.0);
    let bone = |parent: Option<u32>, at: Vec3, rotation: Quat, length: f32| RagdollBone {
        parent,
        rest: (at, rotation),
        length,
        axis: Vec3::Y,
    };
    RagdollRig {
        bones: vec![
            bone(None, Vec3::new(0.0, 1.0, 0.3), Quat::IDENTITY, 0.3),
            bone(Some(0), Vec3::new(0.0, 1.3, 0.3), Quat::IDENTITY, 0.5),
            bone(Some(1), Vec3::new(0.0, 1.8, 0.3), Quat::IDENTITY, 0.3),
            bone(Some(0), Vec3::new(0.1, 1.0, 0.3), down, 0.45),
            bone(Some(3), Vec3::new(0.1, 0.55, 0.3), down, 0.45),
        ],
    }
}

fn targets_2d(sway: f32) -> Vec<(Vec3, Quat)> {
    rig_2d()
        .bones
        .iter()
        .map(|b| (b.rest.0 + Vec3::new(sway * b.rest.0.y, 0.0, 0.0), b.rest.1))
        .collect()
}

/// The angle of a rotation about z (a 2D bone's rotation).
fn angle_z(q: Quat) -> f32 {
    2.0 * q.z.atan2(q.w)
}

#[test]
fn a_2d_ragdoll_follows_goes_limp_on_hinges_in_its_plane_and_recovers() {
    let (mut w, p) = world(
        Physics {
            profile: PROFILE_2D,
            ..Physics::default()
        },
        1,
    );
    floor(&mut w);
    let body = w
        .spawn((
            at(0.0, 0.0, 0.3),
            Ragdoll {
                impulse: Vec3::new(6.0, 2.0, 0.0),
                ..Ragdoll::default()
            },
        ))
        .unwrap();
    p.set_ragdoll_rig(body, rig_2d()).unwrap();
    // Following: the bones are where the animation puts them, at its z.
    for tick in 0..30 {
        let sway = 0.1 * (tick as f32 / 30.0);
        p.set_ragdoll_targets(body, targets_2d(sway));
        w.step().unwrap();
        w.step().unwrap();
        let pose = p.ragdoll_pose(body).unwrap();
        assert_eq!(pose.len(), 5);
        for (b, t) in pose.iter().zip(targets_2d(sway)) {
            assert!(
                (b.0 - t.0).length() < 1e-3,
                "bone at {} for target {}",
                b.0,
                t.0
            );
            assert!(
                (angle_z(b.1) - angle_z(t.1)).abs() < 1e-3,
                "bone turned {} for {}",
                angle_z(b.1),
                angle_z(t.1)
            );
        }
    }
    assert_eq!(ragdoll(&w, body).blend, 0.0);
    // Limp: the blend ramps over 0.25 s; the figure is knocked over and falls.
    set_weight(&mut w, body, 1.0);
    w.step().unwrap();
    let b1 = ragdoll(&w, body).blend;
    assert!(b1 > 0.05 && b1 < 0.1, "one tick into the blend: {b1}");
    run(&mut w, 15);
    assert_eq!(ragdoll(&w, body).blend, 1.0);
    let rig = rig_2d();
    let build = targets_2d(0.0);
    let mut worst_bend = 0f32;
    for _ in 0..150 {
        w.step().unwrap();
        let pose = p.ragdoll_pose(body).unwrap();
        // The knee stays within the joint limit of the pose it was made in.
        let bend =
            (angle_z(pose[4].1) - angle_z(pose[3].1)) - (angle_z(build[4].1) - angle_z(build[3].1));
        let bend =
            (bend + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        worst_bend = worst_bend.max(bend.abs());
    }
    // rapier's limits are stiff springs: landing on the floor under the
    // whole body's weight the knee overshoots by about 0.2 rad (without a
    // limit it folds to 3 rad).
    let limit = Ragdoll::default().joint_limit;
    assert!(
        worst_bend <= limit + 0.3,
        "the knee bent {worst_bend} rad past its {limit} limit"
    );
    let pose = p.ragdoll_pose(body).unwrap();
    assert!(pose[2].0.y < 1.0, "the head fell to {}", pose[2].0);
    assert!(
        pose[0].0.x > 0.3,
        "the hit pushed it along +x: {}",
        pose[0].0
    );
    for (i, b) in rig.bones.iter().enumerate() {
        // In the plane at its z, turned only about z.
        assert!(
            (pose[i].0.z - 0.3).abs() < 1e-6,
            "bone {i} left its plane: {}",
            pose[i].0
        );
        assert!(
            pose[i].1.x.abs() < 1e-6 && pose[i].1.y.abs() < 1e-6,
            "bone {i} tilted out of the plane: {:?}",
            pose[i].1
        );
        let Some(parent) = b.parent else { continue };
        let pb = &rig.bones[parent as usize];
        let (pp, pq) = pose[parent as usize];
        // The joint sits where the child's origin was relative to its parent when built.
        let built = build[parent as usize].1.inverse() * (build[i].0 - build[parent as usize].0);
        let gap = (pose[i].0 - (pp + pq * built)).length();
        assert!(
            gap < 0.05,
            "bone {i} came off its parent by {gap} ({} m long)",
            pb.length
        );
    }
    // Recover: the blend returns over 0.25 s, then the bones follow again.
    set_weight(&mut w, body, 0.0);
    run(&mut w, 8);
    let mid = ragdoll(&w, body).blend;
    assert!(mid > 0.3 && mid < 0.7, "half way back: {mid}");
    run(&mut w, 10);
    assert_eq!(ragdoll(&w, body).blend, 0.0);
    p.set_ragdoll_targets(body, targets_2d(0.0));
    run(&mut w, 2);
    let pose = p.ragdoll_pose(body).unwrap();
    for (b, t) in pose.iter().zip(targets_2d(0.0)) {
        assert!(
            (b.0 - t.0).length() < 1e-3,
            "recovered bone at {} for {}",
            b.0,
            t.0
        );
    }
}
