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
