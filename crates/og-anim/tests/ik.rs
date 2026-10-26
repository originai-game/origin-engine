//! IK chains after the clips: a reachable target is reached within 1e-3
//! (two-bone and FABRIK, model and world space), an unreachable one leaves
//! the chain straight towards it at full length and `reached` false; bone
//! lengths never change and the knee bends towards the pole.
//!
//! Failure modes covered: the law-of-cosines knee on the wrong side or at
//! the wrong height, local rotations derived in the wrong space (the tip
//! misses), the world target not taken into the skeleton's frame, an
//! unreachable target producing NaNs or a stretched bone, `reached` lying.

use crate::support;

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

// ---- joint limits and look-at constraints ----------------------------------

const HEAD: u32 = 3;
const J_SPINE: u32 = 1;

/// The angle (radians) between two rotations.
fn angle_between(a: Quat, b: Quat) -> f32 {
    2.0 * math::atan2(
        (a.conjugate() * b).xyz().length(),
        (a.conjugate() * b).w.abs(),
    )
}

#[test]
fn a_joint_limit_splits_swing_from_twist_and_clamps_each() {
    let limit = JointLimit {
        axis: Vec3::Y,
        swing: 0.5,
        twist_min: -0.3,
        twist_max: 0.3,
        ..JointLimit::default()
    };
    let swing = |a: f32| math::quat_from_axis_angle(Vec3::X, a);
    let twist = |a: f32| math::quat_from_axis_angle(Vec3::Y, a);
    // 0.8 rad of swing and 1.0 of twist: both held at their limits, the
    // swing about its own axis.
    let rest = math::quat_from_axis_angle(Vec3::Z, 0.4);
    let got = limit.clamp(rest, rest * swing(0.8) * twist(1.0));
    let want = rest * swing(0.5) * twist(0.3);
    assert!(angle_between(got, want) < 1e-5, "{got} vs {want}");
    // Twist the other way, swing within: only the twist moves.
    let got = limit.clamp(rest, rest * swing(0.2) * twist(-2.0));
    let want = rest * swing(0.2) * twist(-0.3);
    assert!(angle_between(got, want) < 1e-5, "{got} vs {want}");
    // Inside: untouched, bit for bit.
    let inside = rest * swing(0.4) * twist(0.1);
    assert_eq!(limit.clamp(rest, inside), inside);
    // The default limit holds nothing.
    let any = rest * swing(3.0) * twist(-3.0);
    assert_eq!(JointLimit::default().clamp(rest, any), any);
}

/// The shin's bend from rest about +X (positive: the foot swings back).
fn knee_bend(pose: &[Trs]) -> f32 {
    let q = pose[KNEE_L as usize].rotation;
    2.0 * math::atan2(q.x, q.w)
}

#[test]
fn a_hinged_knee_never_bends_backwards_for_two_bone_ik() {
    // A target the leg reaches bent; the pole behind asks for the knee to
    // bend backwards.
    let target = Vec3::new(0.15, 0.35, 0.2);
    let pole = Vec3::new(0.15, 0.6, -2.0);
    let (free, pose) = solve(Transform::default(), leg(target, pole, IK_MODEL));
    assert!(free.reached);
    assert!(
        knee_bend(&pose) < -0.3,
        "unlimited: bent backwards {}",
        knee_bend(&pose)
    );

    let (mut w, anim, e) = world(Transform::default());
    w.spawn((JointLimit {
        skeleton: e.to_bits(),
        joint: J_SHIN_L,
        axis: Vec3::X,
        swing: 0.0,
        twist_min: 0.0,
        twist_max: 2.6,
    },))
        .unwrap();
    let c = w
        .spawn((IkChain {
            skeleton: e.to_bits(),
            ..leg(target, pole, IK_MODEL)
        },))
        .unwrap();
    w.step().unwrap();
    let pose = anim.pose(e).unwrap();
    let chain: IkChain = w.get(c).unwrap();
    let bend = knee_bend(&pose);
    assert!(
        bend >= -1e-6,
        "the hinge holds the knee straight or forward: {bend}"
    );
    assert!(!chain.reached && chain.error > 0.01, "{chain:?}");
    // A swing-free hinge: the shin turns about +X only.
    let q = pose[KNEE_L as usize].rotation;
    assert!(q.y.abs() < 1e-6 && q.z.abs() < 1e-6, "{q}");

    // With the pole in front the same hinge lets the leg reach.
    let (mut w, anim, e) = world(Transform::default());
    w.spawn((JointLimit {
        skeleton: e.to_bits(),
        joint: J_SHIN_L,
        axis: Vec3::X,
        swing: 0.0,
        twist_min: 0.0,
        twist_max: 2.6,
    },))
        .unwrap();
    let c = w
        .spawn((IkChain {
            skeleton: e.to_bits(),
            ..leg(target, Vec3::new(0.15, 0.6, 2.0), IK_MODEL)
        },))
        .unwrap();
    w.step().unwrap();
    let chain: IkChain = w.get(c).unwrap();
    assert!(chain.reached, "{chain:?}");
    assert!(knee_bend(&anim.pose(e).unwrap()) > 0.3);
}

#[test]
fn fabrik_keeps_a_limited_shoulder_inside_its_cone() {
    let target = Vec3::new(0.25, 2.0, 0.3);
    let arm = |limit: bool| {
        let (mut w, anim, e) = world(Transform::default());
        if limit {
            w.spawn((JointLimit {
                skeleton: e.to_bits(),
                joint: J_ARM_L,
                axis: Vec3::X,
                swing: 0.6,
                ..JointLimit::default()
            },))
                .unwrap();
        }
        let c = w
            .spawn((IkChain {
                skeleton: e.to_bits(),
                tip: J_HAND_L,
                bones: 2,
                solver: IK_FABRIK,
                target,
                space: IK_MODEL,
                iterations: 64,
                ..IkChain::default()
            },))
            .unwrap();
        w.step().unwrap();
        (w.get::<IkChain>(c).unwrap(), anim.pose(e).unwrap())
    };
    let (free, pose) = arm(false);
    let rest = skeleton().rest_pose();
    let swing_of = |pose: &[Trs]| {
        let dir = pose[SHOULDER_L as usize].rotation * Vec3::X;
        math::atan2(dir.cross(Vec3::X).length(), dir.dot(Vec3::X))
    };
    assert!(free.reached, "{free:?}");
    assert!(swing_of(&pose) > 0.9, "unlimited swing {}", swing_of(&pose));
    let (held, pose) = arm(true);
    assert!(
        swing_of(&pose) <= 0.6 + 1e-4,
        "the shoulder stays in its 0.6 rad cone: {}",
        swing_of(&pose)
    );
    assert!(!held.reached);
    // The bones keep their lengths, and the elbow still turns towards it.
    let (s, el, h) = (
        node_position(&pose, SHOULDER_L),
        node_position(&pose, 11),
        node_position(&pose, HAND_L),
    );
    assert!(((el - s).length() - 0.3).abs() < 1e-5 && ((h - el).length() - 0.3).abs() < 1e-5);
    assert_ne!(pose[11].rotation, rest[11].rotation);
}

fn look(at: Transform, desc: LookAt) -> (LookAt, Vec<Trs>, Vec<Trs>) {
    let (mut w, anim, e) = world(at);
    let l = w
        .spawn((LookAt {
            skeleton: e.to_bits(),
            ..desc
        },))
        .unwrap();
    w.step().unwrap();
    let pose = anim.pose(e).unwrap();
    let model = skeleton().model_space(&pose);
    (w.get(l).unwrap(), pose, model)
}

#[test]
fn a_head_looks_at_a_world_target_with_its_up_towards_the_hint() {
    let at = Transform {
        position: Vec3::new(2.0, 0.0, -1.0),
        quaternion: math::quat_from_axis_angle(Vec3::Y, 0.7),
        scale: Vec3::ONE,
    };
    let target = Vec3::new(4.0, 2.5, 1.5);
    let (l, pose, model) = look(
        at,
        LookAt {
            joint: J_HEAD,
            target,
            ..LookAt::default()
        },
    );
    assert!(!l.clamped);
    let head = model[HEAD as usize];
    let world_pos = at.position + at.quaternion * head.translation;
    let forward = at.quaternion * (head.rotation * Vec3::Z);
    let want = (target - world_pos).normalize();
    assert!((forward - want).length() < 1e-5, "{forward} vs {want}");
    // Its +Y leans to world up: no roll.
    let up = at.quaternion * (head.rotation * Vec3::Y);
    assert!(
        up.dot(forward.cross(Vec3::Y)).abs() < 1e-5 && up.y > 0.0,
        "{up}"
    );
    // Only the head turned.
    let rest = skeleton().rest_pose();
    assert_eq!(&pose[..3], &rest[..3]);

    // Half weight turns it part of the way.
    let (_, _, half) = look(
        at,
        LookAt {
            joint: J_HEAD,
            target,
            weight: 0.5,
            ..LookAt::default()
        },
    );
    let f = at.quaternion * (half[HEAD as usize].rotation * Vec3::Z);
    let base = at.quaternion * Vec3::Z;
    assert!(f.dot(want) < 1.0 - 1e-3 && f.dot(want) > base.dot(want));
}

#[test]
fn a_look_outside_its_angles_is_clamped_and_shared_down_the_spine() {
    // Straight behind and up: past a 1 rad yaw and 0.3 rad pitch.
    let target = Vec3::new(-1.0, 3.0, -2.0);
    let (l, pose, model) = look(
        Transform::default(),
        LookAt {
            joint: J_HEAD,
            bones: 2,
            target,
            space: IK_MODEL,
            max_yaw: 1.0,
            max_pitch: 0.3,
            ..LookAt::default()
        },
    );
    assert!(l.clamped);
    let f = model[HEAD as usize].rotation * Vec3::Z;
    let yaw = math::atan2(f.x, f.z);
    let pitch = math::atan2(f.y, (f.x * f.x + f.z * f.z).sqrt());
    assert!((yaw.abs() - 1.0).abs() < 1e-3 && yaw < 0.0, "yaw {yaw}");
    assert!((pitch - 0.3).abs() < 1e-3, "pitch {pitch}");
    // Two joints share it: the spine turned too, about half.
    let spine = angle_between(pose[2].rotation, Quat::IDENTITY);
    let head = angle_between(pose[3].rotation, Quat::IDENTITY);
    assert!(spine > 0.3 && head > 0.3, "spine {spine}, head {head}");

    // A limit on the spine holds its share; the head turns the rest.
    let (mut w, anim, e) = world(Transform::default());
    w.spawn((JointLimit {
        skeleton: e.to_bits(),
        joint: J_SPINE,
        swing: 0.1,
        twist_min: -0.1,
        twist_max: 0.1,
        ..JointLimit::default()
    },))
        .unwrap();
    w.spawn((LookAt {
        skeleton: e.to_bits(),
        joint: J_HEAD,
        bones: 2,
        target: Vec3::new(1.0, 1.7, 0.4),
        space: IK_MODEL,
        ..LookAt::default()
    },))
        .unwrap();
    w.step().unwrap();
    let pose = anim.pose(e).unwrap();
    assert!(angle_between(pose[2].rotation, Quat::IDENTITY) <= 0.1 * 2.0_f32.sqrt() + 1e-4);
    let model = skeleton().model_space(&pose);
    let f = model[HEAD as usize].rotation * Vec3::Z;
    let want = (Vec3::new(1.0, 1.7, 0.4) - model[HEAD as usize].translation).normalize();
    assert!((f - want).length() < 1e-4, "{f} vs {want}");
}
