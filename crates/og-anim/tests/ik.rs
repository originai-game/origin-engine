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

const KNEE_L: u32 = 5;
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
