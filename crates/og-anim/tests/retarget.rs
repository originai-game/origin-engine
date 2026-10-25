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

mod support;

use og_anim::*;
use og_core::glam::Vec3;
use og_ecs::{World, WorldDesc};
use support::tall;

fn clip(name: &str) -> ClipAsset {
    support::clips()
        .into_iter()
        .map(|(_, c)| c)
        .find(|c| c.name == name)
        .unwrap()
}

fn frame_pose(s: &SkeletonAsset, c: &ClipAsset, i: u32) -> Vec<Trs> {
    let mut p = Pose::rest(s);
    let b = bind(s, c);
    sample(s, c, &b, (i as f32 / c.rate).min(c.duration), NONE, &mut p);
    s.model_space(&p.nodes)
}

#[test]
fn the_schemes_are_detected_from_the_names() {
    let walker = support::skeleton();
    let m = HumanoidMap::detect(&walker);
    assert_eq!(m.scheme, "blender");
    assert_eq!(m.count(), 12, "{:?}", m.bones(&walker));
    assert_eq!(m.node("leftUpperArm"), walker.node("arm.L"));
    assert_eq!(m.node("rightFoot"), walker.node("foot.R"));
    let t = tall();
    let m = HumanoidMap::detect(&t);
    assert_eq!(m.scheme, "mixamo");
    assert_eq!(m.count(), 17, "{:?}", m.bones(&t));
    assert_eq!(m.node("chest"), t.node("mixamorig:Spine1"));
    assert_eq!(m.node("leftToes"), t.node("mixamorig:LeftToeBase"));

    // A rig file: a forced scheme, a bone named outright, one taken away.
    let rig = RigAsset::parse(
        "type rig\nversion 1\n---\nscheme mixamo\nhead \"mixamorig:Neck\"\nleftToes none\n",
    )
    .unwrap();
    let m = rig.map(&t).unwrap();
    assert_eq!(m.node("head"), t.node("mixamorig:Neck"));
    assert_eq!(m.node("leftToes"), None);
    assert_eq!(RigAsset::parse(&rig.to_text()).unwrap().map(&t).unwrap(), m);
    // Mistakes name their line.
    let e = RigAsset::parse("type rig\nversion 1\n---\nhips \"a\"\nelbow \"b\"\n").unwrap_err();
    assert!(
        e.reason().contains("line 5") && e.reason().contains("elbow"),
        "{e}"
    );
    let e = RigAsset::parse("type rig\nversion 1\n---\nhips \"nowhere\"\n")
        .unwrap()
        .map(&t)
        .unwrap_err();
    assert!(
        e.reason().contains("line 4") && e.reason().contains("nowhere"),
        "{e}"
    );
}

/// Foot heights (model y) of the leg's foot node over the clip.
fn foot_track(s: &SkeletonAsset, c: &ClipAsset, foot: u32) -> Vec<Vec3> {
    (0..c.frames)
        .map(|i| frame_pose(s, c, i)[foot as usize].translation)
        .collect()
}

/// The largest height error of the target's foot over the frames where the
/// source's foot is planted ([`FootPlants`]: within 5 mm of its lowest): its height above
/// the target's ground (the source's ground scaled onto the target) against
/// the source foot's height above its own, scaled; and the largest slide
/// (xz travel between planted frames against the source's, scaled).
fn contact_error(walk: &ClipAsset, out: &Retargeted, t: &SkeletonAsset) -> (f32, f32, usize) {
    let s = support::skeleton();
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(t));
    let (mut height, mut slide, mut planted) = (0f32, 0f32, 0usize);
    for foot in ["leftFoot", "rightFoot"] {
        let src = foot_track(&s, walk, sm.node(foot).unwrap());
        let plants = FootPlants::new(&s, walk, sm.node(foot).unwrap());
        let dst = foot_track(t, &out.clip, tm.node(foot).unwrap());
        let s_rest = s.model_space(&s.rest_pose())[sm.node(foot).unwrap() as usize].translation;
        let t_rest = t.model_space(&t.rest_pose())[tm.node(foot).unwrap() as usize].translation;
        let ground_s = plants.lowest;
        let ground_t = t_rest.y + (ground_s - s_rest.y) * out.scale;
        let mut last: Option<usize> = None;
        for i in 0..src.len() {
            if !plants.planted(i) {
                last = None;
                continue;
            }
            planted += 1;
            // On the target's ground as the source's foot is on its own:
            // the same height above it, scaled (at most 5 mm × the ratio).
            let expected = ground_t + (src[i].y - ground_s) * out.scale;
            height = height.max((dst[i].y - expected).abs());
            if let Some(j) = last {
                let ds = (src[i] - src[j]) * out.scale;
                let dt = dst[i] - dst[j];
                slide = slide.max(((dt.x - ds.x).powi(2) + (dt.z - ds.z).powi(2)).sqrt());
            }
            last = Some(i);
        }
    }
    (height, slide, planted)
}

#[test]
fn a_walk_on_longer_legs_keeps_its_planted_feet_on_the_ground() {
    let (s, t) = (support::skeleton(), tall());
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(&t));
    let walk = clip("walk");
    let out = retarget(&s, &sm, &t, &tm, &walk, &RetargetOptions::default()).unwrap();
    assert!(
        (out.scale - 1.14 / 0.9).abs() < 1e-4,
        "leg ratio {}",
        out.scale
    );
    assert_eq!((out.bones, out.legs), (12, 2));
    assert_eq!(out.clip.frames, walk.frames);
    assert_eq!(out.clip.events, walk.events);

    let (height, slide, planted) = contact_error(&walk, &out, &t);
    eprintln!("foot IK: {planted} planted frames, height error {height:.5} m, slide {slide:.5} m");
    assert!(planted >= 4, "the walk plants its feet ({planted} frames)");
    assert!(
        height <= 0.002,
        "planted feet within 2 mm of the ground: {height}"
    );
    assert!(slide <= 0.002, "planted feet do not slide: {slide}");

    // Without the legs solved, the longer thigh puts the planted feet off
    // the ground: what foot IK is for.
    let fk = retarget(&s, &sm, &t, &tm, &walk, &RetargetOptions { foot_ik: false }).unwrap();
    let (fk_height, _, _) = contact_error(&walk, &fk, &t);
    eprintln!("forward kinematics only: height error {fk_height:.5} m");
    assert!(
        fk_height > 0.01,
        "FK alone misses by more than 1 cm: {fk_height}"
    );

    // Root motion: the hips travel the walker's 1.2 m stride, scaled.
    let hips = tm.node("hips").unwrap() as usize;
    let first = frame_pose(&t, &out.clip, 0)[hips].translation;
    let last = frame_pose(&t, &out.clip, out.clip.frames - 1)[hips].translation;
    assert!(
        ((last.z - first.z) - 1.2 * out.scale).abs() < 1e-3,
        "hips travel {} m",
        last.z - first.z
    );
    // ... at the target's hip height (1.3 m at rest, bobbing ±3 cm scaled).
    assert!(
        (first.y - (1.3 + 0.03 * out.scale)).abs() < 1e-3,
        "hips at {}",
        first.y
    );
}

#[test]
fn a_wave_turns_the_a_posed_arm_as_the_walker_turns_its_t_posed_one() {
    let (s, t) = (support::skeleton(), tall());
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(&t));
    let wave = clip("wave");
    let out = retarget(&s, &sm, &t, &tm, &wave, &RetargetOptions::default()).unwrap();
    // Only the arm is animated: the legs are not solved, nothing else gets a channel.
    assert_eq!(out.legs, 0);
    let animated: Vec<&str> = out
        .clip
        .channels
        .iter()
        .map(|c| c.node_name.as_str())
        .collect();
    assert_eq!(animated, ["mixamorig:LeftArm"]);
    let (su, sl) = (
        sm.node("leftUpperArm").unwrap() as usize,
        sm.node("leftLowerArm").unwrap() as usize,
    );
    let (tu, tl) = (
        tm.node("leftUpperArm").unwrap() as usize,
        tm.node("leftLowerArm").unwrap() as usize,
    );
    let mut worst = 0f32;
    for i in 0..wave.frames {
        let ps = frame_pose(&s, &wave, i);
        let pt = frame_pose(&t, &out.clip, i);
        let ds = (ps[sl].translation - ps[su].translation).normalize();
        let dt = (pt[tl].translation - pt[tu].translation).normalize();
        worst = worst.max((ds - dt).length());
    }
    assert!(worst < 1e-3, "the arms point alike (worst {worst})");
}

#[test]
fn the_retargeted_clip_plays_on_the_tall_rig() {
    let (s, t) = (support::skeleton(), tall());
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(&t));
    let walk = clip("walk");
    let out = retarget(&s, &sm, &t, &tm, &walk, &RetargetOptions::default()).unwrap();
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    anim.add_skeleton(1, t.clone()).unwrap();
    anim.add_clip(2, out.clip.clone()).unwrap();
    let e = w
        .spawn((
            Transform::default(),
            Skeleton {
                asset: 1,
                ..Default::default()
            },
            AnimationPlayer {
                clip: 2,
                ..Default::default()
            },
        ))
        .unwrap();
    // Tick 15 at 60 Hz is t = 0.25 s, frame 7.5 at 30 Hz: between samples.
    for _ in 0..15 {
        w.step().unwrap();
    }
    let pose = anim.pose(e).unwrap();
    let played = t.model_space(&pose);
    let a = frame_pose(&t, &out.clip, 7);
    let b = frame_pose(&t, &out.clip, 8);
    let foot = tm.node("leftFoot").unwrap() as usize;
    let mid = (a[foot].translation + b[foot].translation) * 0.5;
    assert!(
        (played[foot].translation - mid).length() < 0.01,
        "played {} between {} and {}",
        played[foot].translation,
        a[foot].translation,
        b[foot].translation
    );
}

// ---- mirroring ---------------------------------------------------------------

#[test]
fn sides_swap_in_every_schemes_names() {
    for (a, b) in [
        ("mixamorig:LeftUpLeg", "mixamorig:RightUpLeg"),
        ("leftHand", "rightHand"),
        ("J_Bip_L_UpperArm", "J_Bip_R_UpperArm"),
        ("DEF-thigh.L", "DEF-thigh.R"),
        ("calf_r", "calf_l"),
        ("upper_arm.left", "upper_arm.right"),
        ("Foot_L", "Foot_R"),
    ] {
        assert_eq!(mirror_name(a).as_deref(), Some(b), "{a}");
        assert_eq!(mirror_name(b).as_deref(), Some(a), "{b}");
    }
    for centre in [
        "hips",
        "mixamorig:Spine1",
        "J_Bip_C_Neck",
        "pelvis",
        "DEF-spine.006",
    ] {
        assert_eq!(mirror_name(centre), None, "{centre}");
    }
    let s = support::skeleton();
    let pairs = mirror_pairs(&s);
    assert_eq!(
        pairs[s.node("thigh.L").unwrap() as usize],
        s.node("thigh.R").unwrap()
    );
    assert_eq!(
        pairs[s.node("hips").unwrap() as usize],
        s.node("hips").unwrap()
    );
    // The walker has no right arm: the left one has no counterpart.
    assert_eq!(pairs[s.node("arm.L").unwrap() as usize], NONE);
}

/// The largest distance between `mirrored`'s node `a` and `source`'s node
/// `b` reflected in x = 0, over every frame.
fn mirror_error(
    s: &SkeletonAsset,
    source: &ClipAsset,
    mirrored: &ClipAsset,
    a: u32,
    b: u32,
) -> f32 {
    (0..source.frames)
        .map(|i| {
            let p = frame_pose(s, mirrored, i)[a as usize].translation;
            let q = frame_pose(s, source, i)[b as usize].translation;
            (p - Vec3::new(-q.x, q.y, q.z)).length()
        })
        .fold(0.0, f32::max)
}

#[test]
fn a_mirrored_walk_moves_each_foot_as_the_other_did_reflected() {
    let s = support::skeleton();
    let walk = clip("walk");
    let m = mirror(&s, &walk).unwrap();
    assert_eq!((m.frames, &m.events), (walk.frames, &walk.events));
    let (fl, fr) = (s.node("foot.L").unwrap(), s.node("foot.R").unwrap());
    let err = mirror_error(&s, &walk, &m, fl, fr).max(mirror_error(&s, &walk, &m, fr, fl));
    assert!(err < 1e-5, "feet mirrored within {err}");
    // The hips sway the other way (their yaw flips) and travel as before.
    let hips = s.node("hips").unwrap();
    assert!(mirror_error(&s, &walk, &m, hips, hips) < 1e-5);
    let head = s.node("head").unwrap();
    assert!(mirror_error(&s, &walk, &m, head, head) < 1e-5);
    let i = 7;
    let (a, b) = (
        frame_pose(&s, &walk, i)[hips as usize].rotation,
        frame_pose(&s, &m, i)[hips as usize].rotation,
    );
    assert!(a.y.abs() > 0.01 && (a.y + b.y).abs() < 1e-6, "{a} {b}");
    // Twice is the clip back.
    let back = mirror(&s, &m).unwrap();
    for i in 0..walk.frames {
        let (p, q) = (frame_pose(&s, &walk, i), frame_pose(&s, &back, i));
        for (x, y) in p.iter().zip(&q) {
            assert!((x.translation - y.translation).length() < 1e-5);
        }
    }

    // On the tall Mixamo rig (bone frames along each bone, an armature
    // scaled by 0.01) the retargeted walk mirrors the same way.
    let t = tall();
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(&t));
    let out = retarget(&s, &sm, &t, &tm, &walk, &RetargetOptions::default()).unwrap();
    let m = mirror(&t, &out.clip).unwrap();
    let (fl, fr) = (tm.node("leftFoot").unwrap(), tm.node("rightFoot").unwrap());
    let err = mirror_error(&t, &out.clip, &m, fl, fr);
    assert!(err < 1e-4, "tall feet mirrored within {err}");
    let toe = (tm.node("leftToes").unwrap(), tm.node("rightToes").unwrap());
    assert!(mirror_error(&t, &out.clip, &m, toe.0, toe.1) < 1e-4);
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

/// An Unreal-named arm with a twist bone halfway along each segment, its
/// twist bones named `twist` (`…_twist_01_l`) or `roll`, which no scheme reads.
fn ue_arm(twist: bool) -> SkeletonAsset {
    let i = Quat::IDENTITY;
    let (upper, lower) = if twist {
        ("upperarm_twist_01_l", "lowerarm_twist_01_l")
    } else {
        ("upperarm_roll_01_l", "lowerarm_roll_01_l")
    };
    let defs: Vec<NodeDef> = vec![
        ("root", NONE, Vec3::ZERO, i),
        ("pelvis", 0, Vec3::new(0.0, 1.0, 0.0), i),
        ("spine_01", 1, Vec3::new(0.0, 1.3, 0.0), i),
        ("upperarm_l", 2, Vec3::new(0.2, 1.5, 0.0), i),
        (upper, 3, Vec3::new(0.35, 1.5, 0.0), i),
        ("lowerarm_l", 3, Vec3::new(0.5, 1.5, 0.0), i),
        (lower, 5, Vec3::new(0.625, 1.5, 0.0), i),
        ("hand_l", 5, Vec3::new(0.75, 1.5, 0.0), i),
    ];
    skeleton_from_model(&defs, 1.0)
}

/// `skeleton` playing `clip` at its end in a world, after one tick: the
/// local and the model-space pose.
fn played(skeleton: &SkeletonAsset, clip: &ClipAsset) -> (Vec<Trs>, Vec<Trs>) {
    let mut w = World::new(&WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let anim = install(&mut w).unwrap();
    anim.add_skeleton(1, skeleton.clone()).unwrap();
    anim.add_clip(2, clip.clone()).unwrap();
    let e = w
        .spawn((
            Skeleton {
                asset: 1,
                ..Skeleton::default()
            },
            AnimationPlayer {
                clip: 2,
                time: clip.duration,
                speed: 0.0,
                looping: false,
                ..AnimationPlayer::default()
            },
        ))
        .unwrap();
    w.step().unwrap();
    let pose = anim.pose(e).unwrap();
    let model = skeleton.model_space(&pose);
    (pose, model)
}

#[test]
fn twist_bones_share_their_segments_twist_when_a_clip_for_the_rig_leaves_them_still() {
    // A clip authored for the arm itself (not retargeted): the upper arm
    // rolls 0.6 rad about its length, the hand 1.2; the twist bones have
    // no channels.
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
    let rolls = || vec![roll(3, "upperarm_l", 0.6), roll(7, "hand_l", 1.2)];
    let c = support::clip("roll", 1.0, &[], rolls());
    let t = ue_arm(true);
    let rest = t.model_space(&t.rest_pose());
    let rel = |m: &[Trs], n: usize, of: usize| {
        let now = m[of].rotation.conjugate() * m[n].rotation;
        let then = rest[of].rotation.conjugate() * rest[n].rotation;
        twist(now * then.conjugate(), Vec3::X)
    };
    let (_, m) = played(&t, &c);
    // The shares retargeting gives: the forearm's twist bone half the
    // hand's roll, the upper arm's half the upper arm's against the spine.
    assert!((rel(&m, 7, 5) - 1.2).abs() < 1e-3, "hand {}", rel(&m, 7, 5));
    assert!(
        (rel(&m, 6, 5) - 0.6).abs() < 1e-3,
        "forearm twist {}",
        rel(&m, 6, 5)
    );
    assert!(
        (rel(&m, 3, 2) - 0.6).abs() < 1e-3,
        "upper arm {}",
        rel(&m, 3, 2)
    );
    assert!(
        (rel(&m, 4, 2) - 0.3).abs() < 1e-3,
        "upper arm twist {}",
        rel(&m, 4, 2)
    );
    assert!(rel(&m, 5, 3).abs() < 1e-3, "forearm {}", rel(&m, 5, 3));
    // A clip that animates a twist bone (held at rest here) keeps it; the
    // other is still shared.
    let mut keyed = rolls();
    keyed.push(roll(6, "lowerarm_twist_01_l", 0.0));
    let (local, m) = played(&t, &support::clip("keyed", 1.0, &[], keyed));
    assert_eq!(local[6].rotation, Quat::IDENTITY, "the keyed twist bone");
    assert!(
        (rel(&m, 4, 2) - 0.3).abs() < 1e-3,
        "upper arm twist {}",
        rel(&m, 4, 2)
    );
    // A rig without twist bones (the same nodes named `roll`) plays the
    // clip as it is: its in-between nodes ride their parents at rest.
    let r = ue_arm(false);
    let (local, _) = played(&r, &c);
    let rest = r.rest_pose();
    for n in [4usize, 6] {
        assert_eq!(local[n], rest[n], "node {n}");
    }
}

#[test]
fn a_rigs_foot_height_puts_its_feet_on_the_ground_not_where_its_rest_pose_has_them() {
    // The walker's rig with its root lifted 0.2 m: its rest pose floats,
    // its feet (ankles 0.1 m above the soles) at 0.3.
    let s = support::skeleton();
    let mut t = support::skeleton();
    t.nodes[0].rest.translation.y = 0.2;
    let sm = HumanoidMap::detect(&s);
    let rig = RigAsset::parse("type rig\nversion 1\n---\nfootHeight 0.1\n").unwrap();
    assert_eq!(rig.foot_height, Some([0.1, 0.1]));
    assert_eq!(RigAsset::parse(&rig.to_text()).unwrap(), rig);
    let tm = rig.map(&t).unwrap();
    let walk = clip("walk");
    let lowest = |out: &Retargeted| {
        let foot = tm.node("leftFoot").unwrap();
        foot_track(&t, &out.clip, foot)
            .iter()
            .map(|p| p.y)
            .fold(f32::MAX, f32::min)
    };
    let src = FootPlants::new(&s, &walk, sm.node("leftFoot").unwrap()).lowest;
    let on = retarget(&s, &sm, &t, &tm, &walk, &RetargetOptions::default()).unwrap();
    let want = 0.1 + (src - 0.1) * on.scale;
    assert!(
        (lowest(&on) - want).abs() < 2e-3,
        "the planted foot at {} m, the ground's {want}",
        lowest(&on)
    );
    // From the rest pose, the feet stay 0.2 m up in the air.
    let off = retarget(
        &s,
        &sm,
        &t,
        &HumanoidMap::detect(&t),
        &walk,
        &RetargetOptions::default(),
    )
    .unwrap();
    assert!(lowest(&off) > 0.25, "{}", lowest(&off));
    // One height per side, and mistakes by line.
    let both = RigAsset::parse("type rig\nversion 1\n---\nfootHeight 0.08 0.12\n").unwrap();
    assert_eq!(both.foot_height, Some([0.08, 0.12]));
    assert_eq!(RigAsset::parse(&both.to_text()).unwrap(), both);
    let e = RigAsset::parse("type rig\nversion 1\n---\nfootHeight 1 2 3\n").unwrap_err();
    assert!(e.reason().contains("line 4"), "{e}");
}
