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
use og_core::glam::{Quat, Vec3};
use og_core::math;
use og_ecs::{World, WorldDesc};

/// (name, parent, model position in metres, model rotation)
type NodeDef = (&'static str, u32, Vec3, Quat);

fn rz(deg: f32) -> Quat {
    math::quat_from_axis_angle(Vec3::Z, deg.to_radians())
}

/// A skeleton from model-space rest transforms under a root scaled by `scale`.
fn skeleton_from_model(defs: &[NodeDef], scale: f32) -> SkeletonAsset {
    let mut model: Vec<Trs> = Vec::new();
    let mut nodes = Vec::new();
    for (i, &(name, parent, p, r)) in defs.iter().enumerate() {
        let m = if i == 0 {
            Trs::new(Vec3::ZERO, Quat::IDENTITY, Vec3::splat(scale))
        } else {
            Trs::new(p, r, Vec3::splat(scale))
        };
        let local = if parent == NONE {
            m
        } else {
            let pm = model[parent as usize];
            let l = pm.inverse().mul(&m);
            Trs::new(l.translation, l.rotation, Vec3::ONE)
        };
        model.push(m);
        nodes.push(SkeletonNode {
            name: name.into(),
            parent,
            rest: local,
            mesh: NONE,
        });
    }
    let mut s = SkeletonAsset {
        nodes,
        skins: Vec::new(),
    };
    let mm = s.model_matrices(&s.rest_pose());
    let joints: Vec<u32> = (1..s.nodes.len() as u32).collect();
    let inverse_bind = joints.iter().map(|&j| mm[j as usize].inverse()).collect();
    s.skins.push(Skin {
        name: "tall".into(),
        joints,
        inverse_bind,
    });
    s
}

/// Thigh 0.70, shin 0.44, hips at 1.30: a leg 1.14 m against the walker's 0.90.
fn tall() -> SkeletonAsset {
    let down = rz(180.0);
    let toe = rotation_to(Vec3::new(0.0, -0.12, 0.12));
    let defs: Vec<NodeDef> = vec![
        ("Armature", NONE, Vec3::ZERO, Quat::IDENTITY),
        (
            "mixamorig:Hips",
            0,
            Vec3::new(0.0, 1.3, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Spine",
            1,
            Vec3::new(0.0, 1.5, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Spine1",
            2,
            Vec3::new(0.0, 1.65, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Neck",
            3,
            Vec3::new(0.0, 1.85, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:Head",
            4,
            Vec3::new(0.0, 1.97, 0.0),
            Quat::IDENTITY,
        ),
        (
            "mixamorig:LeftShoulder",
            3,
            Vec3::new(0.08, 1.8, 0.0),
            rz(-90.0),
        ),
        ("mixamorig:LeftArm", 6, Vec3::new(0.2, 1.8, 0.0), rz(-135.0)),
        (
            "mixamorig:LeftForeArm",
            7,
            Vec3::new(
                0.2 + 0.3 * std::f32::consts::FRAC_1_SQRT_2,
                1.8 - 0.3 * std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            ),
            rz(-135.0),
        ),
        (
            "mixamorig:LeftHand",
            8,
            Vec3::new(
                0.2 + 0.55 * std::f32::consts::FRAC_1_SQRT_2,
                1.8 - 0.55 * std::f32::consts::FRAC_1_SQRT_2,
                0.0,
            ),
            rz(-135.0),
        ),
        ("mixamorig:LeftUpLeg", 1, Vec3::new(0.12, 1.3, 0.0), down),
        ("mixamorig:LeftLeg", 10, Vec3::new(0.12, 0.6, 0.0), down),
        ("mixamorig:LeftFoot", 11, Vec3::new(0.12, 0.16, 0.0), toe),
        (
            "mixamorig:LeftToeBase",
            12,
            Vec3::new(0.12, 0.04, 0.12),
            toe,
        ),
        ("mixamorig:RightUpLeg", 1, Vec3::new(-0.12, 1.3, 0.0), down),
        ("mixamorig:RightLeg", 14, Vec3::new(-0.12, 0.6, 0.0), down),
        ("mixamorig:RightFoot", 15, Vec3::new(-0.12, 0.16, 0.0), toe),
        (
            "mixamorig:RightToeBase",
            16,
            Vec3::new(-0.12, 0.04, 0.12),
            toe,
        ),
    ];
    skeleton_from_model(&defs, 0.01)
}

/// The rotation taking +Y to `dir`.
fn rotation_to(dir: Vec3) -> Quat {
    og_anim::rotation_arc(Vec3::Y, dir)
}

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
/// source's foot is planted (within 5 mm of its lowest): its height above
/// the target's ground (the source's ground scaled onto the target) against
/// the source foot's height above its own, scaled; and the largest slide
/// (xz travel between planted frames against the source's, scaled).
fn contact_error(walk: &ClipAsset, out: &Retargeted, t: &SkeletonAsset) -> (f32, f32, usize) {
    let s = support::skeleton();
    let (sm, tm) = (HumanoidMap::detect(&s), HumanoidMap::detect(t));
    let (mut height, mut slide, mut planted) = (0f32, 0f32, 0usize);
    for foot in ["leftFoot", "rightFoot"] {
        let src = foot_track(&s, walk, sm.node(foot).unwrap());
        let dst = foot_track(t, &out.clip, tm.node(foot).unwrap());
        let s_rest = s.model_space(&s.rest_pose())[sm.node(foot).unwrap() as usize].translation;
        let t_rest = t.model_space(&t.rest_pose())[tm.node(foot).unwrap() as usize].translation;
        let ground_s = src.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        let ground_t = t_rest.y + (ground_s - s_rest.y) * out.scale;
        let mut last: Option<usize> = None;
        for i in 0..src.len() {
            if src[i].y > ground_s + 0.005 {
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
