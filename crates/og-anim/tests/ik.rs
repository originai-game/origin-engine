//! IK chains after the clips: a reachable target is reached within 1e-3
//! (two-bone and FABRIK, model and world space), an unreachable one leaves
//! the chain straight towards it at full length and `reached` false; bone
//! lengths never change and the knee bends towards the pole.
//!
//! Failure modes covered: the law-of-cosines knee on the wrong side or at
//! the wrong height, local rotations derived in the wrong space (the tip
//! misses), the world target not taken into the skeleton's frame, an
//! unreachable target producing NaNs or a stretched bone, `reached` lying.

mod support;

use og_anim::*;
use og_core::glam::{Quat, Vec3};
use og_core::math;
use og_ecs::{Entity, World, WorldDesc};
use support::*;

const HIP_L: u32 = 4;
const KNEE_L: u32 = 5;
const FOOT_L: u32 = 6;
const SHOULDER_L: u32 = 10;
const HAND_L: u32 = 12;

fn world(at: Transform) -> (World, AnimationSystem, Entity) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    register(&anim);
    let e = w
        .spawn((
            at,
            Skeleton {
                asset: SKELETON,
                ..Skeleton::default()
            },
        ))
        .unwrap();
    (w, anim, e)
}

fn solve(at: Transform, chain: IkChain) -> (IkChain, Vec<Trs>) {
    let (mut w, anim, e) = world(at);
    let c = w
        .spawn((IkChain {
            skeleton: e.to_bits(),
            ..chain
        },))
        .unwrap();
    w.step().unwrap();
    (w.get(c).unwrap(), anim.pose(e).unwrap())
}

fn leg(target: Vec3, pole: Vec3, space: u32) -> IkChain {
    IkChain {
        tip: J_FOOT_L,
        target,
        pole,
        space,
        ..IkChain::default()
    }
}

#[test]
fn two_bone_reaches_a_reachable_target_and_bends_to_the_pole() {
    let target = Vec3::new(0.3, 0.45, 0.35);
    let pole = Vec3::new(0.15, 0.6, 2.0);
    let (c, pose) = solve(Transform::default(), leg(target, pole, IK_MODEL));
    let foot = node_position(&pose, FOOT_L);
    assert!(
        (foot - target).length() < 1e-3,
        "foot at {foot}, target {target}"
    );
    assert!(c.reached && c.error < 1e-3, "{c:?}");
    let (hip, knee) = (node_position(&pose, HIP_L), node_position(&pose, KNEE_L));
    assert!(((knee - hip).length() - 0.45).abs() < 1e-5);
    assert!(((foot - knee).length() - 0.45).abs() < 1e-5);
    // The knee sits on the pole's side of the hip-foot line.
    let axis = (foot - hip).normalize();
    let off = |p: Vec3| (p - hip) - axis * (p - hip).dot(axis);
    assert!(off(knee).dot(off(pole)) > 0.0);
    // The other leg is untouched.
    let rest = skeleton().rest_pose();
    assert_eq!(pose[7], rest[7]);
}

#[test]
fn two_bone_clamps_an_unreachable_target() {
    let target = Vec3::new(0.15, -1.0, 0.5);
    let (c, pose) = solve(Transform::default(), leg(target, Vec3::ZERO, IK_MODEL));
    let (hip, foot) = (node_position(&pose, HIP_L), node_position(&pose, FOOT_L));
    let dir = (target - hip).normalize();
    assert!(!c.reached);
    assert!(((foot - hip).length() - 0.9).abs() < 1e-4, "full length");
    assert!(
        (foot - (hip + dir * (foot - hip).length())).length() < 1e-4,
        "straight at it"
    );
    assert!((c.error - ((target - hip).length() - 0.9)).abs() < 1e-4);
    assert!(pose.iter().all(|t| t.rotation.is_finite()));
}

#[test]
fn world_targets_are_taken_into_the_skeletons_frame() {
    let at = Transform {
        position: Vec3::new(4.0, 0.5, -2.0),
        quaternion: math::quat_from_axis_angle(Vec3::Y, 0.8),
        scale: Vec3::ONE,
    };
    let local = Vec3::new(0.3, 0.45, 0.35);
    let world_target = at.position + at.quaternion * local;
    let (c, pose) = solve(at, leg(world_target, Vec3::ZERO, IK_WORLD));
    assert!(c.reached, "{c:?}");
    let foot_world = at.position + at.quaternion * node_position(&pose, FOOT_L);
    assert!((foot_world - world_target).length() < 1e-3);
}

#[test]
fn fabrik_reaches_and_clamps() {
    // Three bones from the spine: spine → arm → forearm → hand.
    let target = Vec3::new(0.55, 1.95, 0.3);
    let (c, pose) = solve(
        Transform::default(),
        IkChain {
            tip: J_HAND_L,
            bones: 3,
            solver: IK_FABRIK,
            target,
            space: IK_MODEL,
            iterations: 32,
            ..IkChain::default()
        },
    );
    let hand = node_position(&pose, HAND_L);
    assert!(
        c.reached && (hand - target).length() < 1e-3,
        "{c:?} hand {hand}"
    );
    let shoulder = node_position(&pose, SHOULDER_L);
    assert!(((node_position(&pose, 11) - shoulder).length() - 0.3).abs() < 1e-5);

    let far = Vec3::new(3.0, 1.7, 0.0);
    let (c, pose) = solve(
        Transform::default(),
        IkChain {
            tip: J_HAND_L,
            bones: 2,
            solver: IK_FABRIK,
            target: far,
            space: IK_MODEL,
            ..IkChain::default()
        },
    );
    let (s, h) = (
        node_position(&pose, SHOULDER_L),
        node_position(&pose, HAND_L),
    );
    assert!(!c.reached);
    assert!(((h - s).length() - 0.6).abs() < 1e-4);
    assert!((h - (s + (far - s).normalize() * 0.6)).length() < 1e-4);
}

#[test]
fn half_weight_turns_halfway_and_a_bad_tip_is_an_error() {
    let target = Vec3::new(0.3, 0.45, 0.35);
    let (_, full) = solve(Transform::default(), leg(target, Vec3::ZERO, IK_MODEL));
    let (c, half) = solve(
        Transform::default(),
        IkChain {
            weight: 0.5,
            ..leg(target, Vec3::ZERO, IK_MODEL)
        },
    );
    assert!(!c.reached);
    let angle = |a: Quat, b: Quat| a.dot(b).abs();
    let rest = skeleton().rest_pose();
    assert!(
        angle(half[HIP_L as usize].rotation, rest[HIP_L as usize].rotation)
            > angle(full[HIP_L as usize].rotation, rest[HIP_L as usize].rotation)
    );

    let (mut w, _, e) = world(Transform::default());
    w.spawn((IkChain {
        skeleton: e.to_bits(),
        tip: 40,
        ..IkChain::default()
    },))
        .unwrap();
    let err = w.step().unwrap_err().to_string();
    assert!(err.contains("og.IkChain.tip 40"), "{err}");
}
