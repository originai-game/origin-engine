//! Retargeting the walker's clips (Blender names, T-pose, legs 0.45 + 0.45 m,
//! identity rest rotations) onto a taller rig with Mixamo names, an
//! armature scaled by 0.01, Mixamo-style rest rotations (each bone's +Y
//! along the bone), arms in an A-pose and other leg proportions (thigh 0.70,
//! shin 0.44 m).
//!
//! Ways retargeting could fail, and what here catches each:
//! - a name scheme read wrongly or not at all (the walker's or the Mixamo
//!   rig's bones unmapped): the detection assertions;
//! - rest orientations not compensated (Mixamo's bone frames, the A-pose):
//!   the waving arm would point elsewhere than the walker's;
//! - the armature's scale or the hips' parent ignored: the hips' travel
//!   would be off by 100 or not scaled by the leg ratio;
//! - planted feet floating or sinking on a rig with other proportions
//!   (forward kinematics alone): the contact check, which fails without
//!   foot IK (`foot_ik: false` is asserted to miss by centimetres);
//! - a planted foot sliding: its travel between contact frames must match
//!   the source's, scaled;
//! - the result not playable by the system (bad channels, wrong node
//!   names): the clip is played by og.anim on the tall rig and its pose
//!   compared with the retargeted samples.

use crate::support;

use og_anim::*;
use og_core::glam::Vec3;

fn frame_pose(s: &SkeletonAsset, c: &ClipAsset, i: u32) -> Vec<Trs> {
    let mut p = Pose::rest(s);
    let b = bind(s, c);
    sample(s, c, &b, (i as f32 / c.rate).min(c.duration), NONE, &mut p);
    s.model_space(&p.nodes)
}

// ---- hands, fingers, twist bones and foot heights ----

use og_core::glam::Quat;
use support::{NodeDef, rz, skeleton_from_model};

/// A T-posed source arm in Blender (Rigify-style finger) names: the arm
/// along +x, an index finger and a thumb on the hand.
fn hand_source() -> SkeletonAsset {
    let i = Quat::IDENTITY;
    let defs: Vec<NodeDef> = vec![
        ("Rig", NONE, Vec3::ZERO, i),
        ("hips", 0, Vec3::new(0.0, 1.0, 0.0), i),
        ("spine", 1, Vec3::new(0.0, 1.3, 0.0), i),
        ("upper_arm.L", 2, Vec3::new(0.2, 1.5, 0.0), i),
        ("forearm.L", 3, Vec3::new(0.5, 1.5, 0.0), i),
        ("hand.L", 4, Vec3::new(0.75, 1.5, 0.0), i),
        ("f_index.01.L", 5, Vec3::new(0.83, 1.5, 0.0), i),
        ("f_index.02.L", 6, Vec3::new(0.87, 1.5, 0.0), i),
        ("f_index.03.L", 7, Vec3::new(0.90, 1.5, 0.0), i),
        ("thumb.01.L", 5, Vec3::new(0.77, 1.5, 0.03), i),
        ("thumb.02.L", 9, Vec3::new(0.80, 1.5, 0.05), i),
        ("thumb.03.L", 10, Vec3::new(0.82, 1.5, 0.07), i),
    ];
    skeleton_from_model(&defs, 1.0)
}

/// An A-posed target in Mixamo names and bone frames (+Y along each bone).
fn hand_target() -> SkeletonAsset {
    let (c, s) = (
        std::f32::consts::FRAC_1_SQRT_2,
        std::f32::consts::FRAC_1_SQRT_2,
    );
    let at = |d: f32| Vec3::new(0.2 + d * c, 1.6 - d * s, 0.0);
    let r = rz(-135.0);
    let defs: Vec<NodeDef> = vec![
        ("Armature", NONE, Vec3::ZERO, Quat::IDENTITY),
        (
            "mixamorig:Hips",
            0,
            Vec3::new(0.0, 1.1, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Spine",
            1,
            Vec3::new(0.0, 1.4, 0.0),
            Quat::IDENTITY,
        ),
        ("mixamorig:LeftArm", 2, at(0.0), r),
        ("mixamorig:LeftForeArm", 3, at(0.32), r),
        ("mixamorig:LeftHand", 4, at(0.6), r),
        ("mixamorig:LeftHandIndex1", 5, at(0.69), r),
        ("mixamorig:LeftHandIndex2", 6, at(0.74), r),
        ("mixamorig:LeftHandIndex3", 7, at(0.78), r),
        ("mixamorig:LeftHandIndex4", 8, at(0.81), r),
    ];
    skeleton_from_model(&defs, 1.0)
}

/// Angle between two directions.
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn angle(a: Vec3, b: Vec3) -> f32 {
    a.normalize().dot(b.normalize()).clamp(-1.0, 1.0).acos()
}

#[test]
fn fingers_are_humanoid_bones_and_a_curl_carries_over() {
    // Every scheme's finger names land on the finger bones.
    let s = hand_source();
    let sm = HumanoidMap::detect(&s);
    assert_eq!(sm.scheme, "blender");
    assert_eq!(sm.node("leftIndexProximal"), s.node("f_index.01.L"));
    assert_eq!(sm.node("leftIndexDistal"), s.node("f_index.03.L"));
    assert_eq!(sm.node("leftThumbMetacarpal"), s.node("thumb.01.L"));
    let t = hand_target();
    let tm = HumanoidMap::detect(&t);
    assert_eq!(tm.scheme, "mixamo");
    assert_eq!(
        tm.node("leftIndexProximal"),
        t.node("mixamorig:LeftHandIndex1")
    );
    assert_eq!(
        tm.node("leftIndexIntermediate"),
        t.node("mixamorig:LeftHandIndex2")
    );
    // Mixamo's fourth index joint is the finger's end, not a bone.
    assert_eq!(tm.count(), 8, "{:?}", tm.bones(&t));
    for (name, bone) in [
        ("J_Bip_L_Index1", "leftIndexProximal"),
        ("J_Bip_R_Thumb3", "rightThumbDistal"),
        ("J_Bip_L_Little2", "leftLittleIntermediate"),
        ("DEF-f_pinky.02.R", "rightLittleIntermediate"),
        ("DEF-thumb.01.L", "leftThumbMetacarpal"),
        ("index_03_l", "leftIndexDistal"),
        ("middle_01_r", "rightMiddleProximal"),
        ("mixamorig:RightHandRing2", "rightRingIntermediate"),
        ("LeftMiddleDistal", "leftMiddleDistal"),
    ] {
        let skel = SkeletonAsset {
            nodes: vec![SkeletonNode {
                name: name.into(),
                parent: NONE,
                rest: Trs::IDENTITY,
                mesh: NONE,
            }],
            skins: Vec::new(),
            ..Default::default()
        };
        assert_eq!(HumanoidMap::detect(&skel).node(bone), Some(0), "{name}");
    }

    // A curl: each index joint turns 0.5 rad down (about the source's z).
    let curl = |node: u32, name: &'static str| -> support::ChannelFn<'static> {
        (
            node,
            name,
            ChannelPath::Rotation,
            4,
            Box::new(move |t| {
                og_core::math::quat_from_axis_angle(Vec3::Z, -0.5 * t)
                    .to_array()
                    .to_vec()
            }),
        )
    };
    let fist = support::clip(
        "fist",
        1.0,
        &[],
        vec![
            curl(6, "f_index.01.L"),
            curl(7, "f_index.02.L"),
            curl(8, "f_index.03.L"),
        ],
    );
    let out = retarget(&s, &sm, &t, &tm, &fist, &RetargetOptions::default()).unwrap();
    let last = fist.frames - 1;
    let (src, dst) = (frame_pose(&s, &fist, last), frame_pose(&t, &out.clip, last));
    let bent = |m: &[Trs], hand: usize, a: usize, b: usize, c: usize| {
        (
            angle(
                m[a].translation - m[hand].translation,
                m[b].translation - m[a].translation,
            ),
            angle(
                m[b].translation - m[a].translation,
                m[c].translation - m[b].translation,
            ),
        )
    };
    let want = bent(&src, 5, 6, 7, 8);
    let got = bent(&dst, 5, 6, 7, 8);
    assert!(
        (want.0 - 0.5).abs() < 1e-4 && (want.1 - 0.5).abs() < 1e-4,
        "{want:?}"
    );
    assert!(
        (got.0 - want.0).abs() < 1e-3 && (got.1 - want.1).abs() < 1e-3,
        "the target's index bends {got:?}, the source's {want:?}"
    );
    // The fingers' bones keep the target's lengths.
    let rest = t.model_space(&t.rest_pose());
    for (a, b) in [(6, 7), (7, 8)] {
        let l = (dst[b].translation - dst[a].translation).length();
        let r = (rest[b].translation - rest[a].translation).length();
        assert!((l - r).abs() < 1e-5);
    }
}

/// The twist of `q` about unit `axis`, in radians.
// Test reference values: libm checks the result, never the tick.
#[allow(clippy::disallowed_methods)]
fn twist(q: Quat, axis: Vec3) -> f32 {
    let p = Vec3::new(q.x, q.y, q.z).dot(axis);
    let (p, w) = if q.w < 0.0 { (-p, -q.w) } else { (p, q.w) };
    2.0 * p.atan2(w)
}

#[test]
fn twist_bones_share_their_segments_twist() {
    // An Unreal-named arm with a twist bone halfway along each segment.
    let i = Quat::IDENTITY;
    let defs: Vec<NodeDef> = vec![
        ("root", NONE, Vec3::ZERO, i),
        ("pelvis", 0, Vec3::new(0.0, 1.0, 0.0), i),
        ("spine_01", 1, Vec3::new(0.0, 1.3, 0.0), i),
        ("upperarm_l", 2, Vec3::new(0.2, 1.5, 0.0), i),
        ("upperarm_twist_01_l", 3, Vec3::new(0.35, 1.5, 0.0), i),
        ("lowerarm_l", 3, Vec3::new(0.5, 1.5, 0.0), i),
        ("lowerarm_twist_01_l", 5, Vec3::new(0.625, 1.5, 0.0), i),
        ("hand_l", 5, Vec3::new(0.75, 1.5, 0.0), i),
    ];
    let t = skeleton_from_model(&defs, 1.0);
    let s = hand_source();
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(&t));
    assert_eq!(tm.scheme, "ue");
    // The upper arm rolls 0.6 rad about its length, the hand 1.2.
    let roll = |node: u32, name: &'static str, a: f32| -> support::ChannelFn<'static> {
        (
            node,
            name,
            ChannelPath::Rotation,
            4,
            Box::new(move |t| {
                og_core::math::quat_from_axis_angle(Vec3::X, a * t)
                    .to_array()
                    .to_vec()
            }),
        )
    };
    let c = support::clip(
        "roll",
        1.0,
        &[],
        vec![roll(3, "upper_arm.L", 0.6), roll(5, "hand.L", 1.2)],
    );
    let out = retarget(&s, &sm, &t, &tm, &c, &RetargetOptions::default()).unwrap();
    assert_eq!(out.twists, 2);
    let m = frame_pose(&t, &out.clip, c.frames - 1);
    let rest = t.model_space(&t.rest_pose());
    // Twist of node `n` relative to node `of`, from rest, about +x.
    let rel = |n: usize, of: usize| {
        let now = m[of].rotation.conjugate() * m[n].rotation;
        let then = rest[of].rotation.conjugate() * rest[n].rotation;
        twist(now * then.conjugate(), Vec3::X)
    };
    // The hand keeps its whole roll, the forearm's twist bone half of it.
    assert!((rel(7, 5) - 1.2).abs() < 1e-3, "hand {}", rel(7, 5));
    assert!(
        (rel(6, 5) - 0.6).abs() < 1e-3,
        "forearm twist {}",
        rel(6, 5)
    );
    // Halfway up the upper arm, half its roll against the spine.
    assert!((rel(3, 2) - 0.6).abs() < 1e-3, "upper arm {}", rel(3, 2));
    assert!(
        (rel(4, 2) - 0.3).abs() < 1e-3,
        "upper arm twist {}",
        rel(4, 2)
    );
    // The forearm, a child of the upper arm (not of its twist bone), keeps
    // its own pose.
    assert!(rel(5, 3).abs() < 1e-3, "forearm {}", rel(5, 3));
}
