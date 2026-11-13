//! The intent API and the generated skeleton: goals chained through
//! generations on the fixture bundle (a random tiny bundle, see
//! reference.rs), goals placed in the world, and feet planted from contacts
//! on a synthetic skeleton.

use og_anim::Trs;
use og_anim::motion::contract::{FRAMES, JOINTS, MOTION_FEATURES, Pose};
use og_anim::motion::intent::{Goal, Motion};
use og_anim::motion::soma::{RELEASE_FRAMES, Skeleton, plant_feet};
use og_anim::motion::{Bundle, Model, Request, decode_frame, encode, valid_frames};
use og_core::glam::{Quat, Vec3};

fn fixture(name: &str) -> Bundle {
    let path = format!(
        "{}/tests/motion/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    Bundle::from_safetensors(&std::fs::read(&path).unwrap()).unwrap()
}

fn poses(case: &Bundle, which: &str, character: usize, count: usize) -> Vec<Pose> {
    let p = &case.tensors[&format!("{which}_positions")].data;
    let r = &case.tensors[&format!("{which}_rotations")].data;
    (0..count)
        .map(|s| {
            let base = (character * 4 + s) * JOINTS;
            Pose {
                positions: std::array::from_fn(|j| std::array::from_fn(|k| p[(base + j) * 3 + k])),
                rotations: std::array::from_fn(|j| {
                    std::array::from_fn(|a| {
                        std::array::from_fn(|b| r[((base + j) * 3 + a) * 3 + b])
                    })
                }),
            }
        })
        .collect()
}

#[test]
fn goals_chain_through_generations_from_the_last_four_frames() {
    let case = fixture("case.safetensors");
    let model = Model::new(&fixture("tiny.safetensors")).unwrap();
    let counts = [vec![8], vec![11], vec![16]];
    let num_targets = &case.tensors["num_targets"].data;
    let contexts: Vec<[Pose; 4]> = (0..3)
        .map(|i| poses(&case, "context", i, 4).try_into().unwrap())
        .collect();
    let goals: Vec<Goal> = (0..3)
        .map(|i| Goal {
            targets: poses(&case, "target", i, num_targets[i] as usize),
            token_counts: counts[i].clone(),
            tag: i as u32,
        })
        .collect();

    let mut motion = Motion::default();
    for (context, goal) in contexts.iter().zip(&goals) {
        let c = motion.add(*context);
        motion.push(c, goal.clone());
    }
    // A character with enough frames queued waits; none is due at lookahead 0.
    assert!(motion.batch(0).is_none());
    assert_eq!(motion.generate(&model, usize::MAX), 3);
    assert_eq!(motion.generate(&model, usize::MAX), 0, "every goal is done");

    // The same generation by hand: the queued frames are its motion after
    // the context's last frame.
    let requests: Vec<Request> = (0..3)
        .map(|i| Request {
            context: &contexts[i],
            targets: &goals[i].targets,
            token_counts: &goals[i].token_counts,
        })
        .collect();
    let (inputs, alignments) = encode(&requests);
    let expected = model.generate(&inputs);
    // A landed batch may be spread across frames, in any character order.
    // Accepting one character must not consume or expose another's frames.
    let mut spread = Motion::default();
    for (context, goal) in contexts.iter().zip(&goals) {
        let c = spread.add(*context);
        spread.push(c, goal.clone());
    }
    let batch = spread.batch(usize::MAX).unwrap();
    let per = FRAMES * MOTION_FEATURES;
    for i in [2, 0, 1] {
        spread.accept_character(
            &batch,
            i,
            expected.counts[i],
            &expected.motion[i * per..(i + 1) * per],
        );
        assert_eq!(spread.goals(i), 0);
    }
    let mut shown: Vec<Vec<Pose>> = Vec::new();
    for (i, &count) in [8usize, 11, 16].iter().enumerate() {
        let valid = valid_frames(count);
        assert_eq!(motion.queued(i), valid.len() - 1);
        let mut frames = Vec::new();
        for f in valid.start + 1..valid.end {
            let frame = motion.pop(i).unwrap();
            assert_eq!(
                spread.pop(i),
                Some(frame),
                "spread decoding changed character {i}"
            );
            assert_eq!(frame.tag, i as u32, "a frame carries its goal's tag");
            let at = (i * FRAMES + f) * MOTION_FEATURES;
            let want = decode_frame(&expected.motion[at..at + MOTION_FEATURES], &alignments[i]);
            assert_eq!(frame.pose, want, "character {i} frame {f}");
            assert_eq!(frame.contacts[..], expected.motion[at + 362..at + 366]);
            frames.push(frame.pose);
        }
        assert!(motion.pop(i).is_none());
        shown.push(frames);
    }

    // The next goal starts from each character's last four frames.
    for (i, goal) in goals.iter().enumerate() {
        motion.push(i, goal.clone());
    }
    let batch = motion.batch(usize::MAX).unwrap();
    let last: Vec<[Pose; 4]> = shown
        .iter()
        .map(|f| std::array::from_fn(|k| f[f.len() - 4 + k]))
        .collect();
    let again: Vec<Request> = (0..3)
        .map(|i| Request {
            context: &last[i],
            targets: &goals[i].targets,
            token_counts: &goals[i].token_counts,
        })
        .collect();
    assert_eq!(batch.inputs, encode(&again).0);
}

/// SOMA's joint tree with plain proportions: legs of 0.45 + 0.45 m.
fn skeleton() -> Skeleton {
    let parents = [
        -1, 0, 1, 2, 3, 4, 5, 6, 6, 6, 3, 10, 11, 12, 13, 13, 3, 16, 17, 18, 19, 19, 0, 22, 23, 24,
        0, 26, 27, 28,
    ];
    let offset = |j: usize| match j {
        0 => Vec3::new(0.0, 0.95, 0.0),
        22 => Vec3::new(0.1, -0.05, 0.0),
        26 => Vec3::new(-0.1, -0.05, 0.0),
        23 | 24 | 27 | 28 => Vec3::new(0.0, -0.45, 0.0),
        25 | 29 => Vec3::new(0.0, -0.05, 0.1),
        _ => Vec3::new(0.0, 0.1, 0.0),
    };
    Skeleton {
        parents: std::array::from_fn(|j| {
            if parents[j] < 0 {
                og_anim::NONE
            } else {
                parents[j] as u32
            }
        }),
        rest: std::array::from_fn(|j| Trs::new(offset(j), Quat::IDENTITY, Vec3::ONE)),
    }
}

#[test]
fn a_heel_on_the_ground_stays_put_and_lets_go_smoothly() {
    let asset = skeleton().asset();
    // The hips walk forward 2 cm a frame with both knees bent; the model
    // says the left heel is down for frames 0 to 5.
    let frames = 14;
    let bend = |a: f32| Quat::from_rotation_x(a);
    let generated: Vec<Vec<Trs>> = (0..frames)
        .map(|f| {
            let mut pose = asset.rest_pose();
            pose[0].translation = Vec3::new(0.0, 0.9, 0.02 * f as f32);
            for (thigh, shin, ankle) in [(22, 23, 24), (26, 27, 28)] {
                pose[thigh].rotation = bend(-0.3);
                pose[shin].rotation = bend(0.6);
                pose[ankle].rotation = bend(-0.3 + 0.01 * f as f32);
            }
            pose
        })
        .collect();
    let contacts: Vec<[f32; 4]> = (0..frames)
        .map(|f| [if f < 6 { 0.9 } else { 0.1 }, 0.0, 0.0, 0.0])
        .collect();
    let mut planted = generated.clone();
    plant_feet(&asset, &mut planted, &contacts);

    let model = |p: &[Trs]| asset.model_space(p);
    let anchor = model(&generated[0])[24].translation;
    for f in 0..frames {
        let (g, p) = (model(&generated[f]), model(&planted[f]));
        // The foot keeps its generated orientation, planted or not.
        assert!(
            g[24].rotation.dot(p[24].rotation).abs() > 1.0 - 1e-6,
            "frame {f}"
        );
        // The right leg had no contact: untouched.
        assert_eq!(planted[f][27], generated[f][27]);
        let at = p[24].translation;
        if f < 6 {
            assert!(
                (at - anchor).length() < 1e-4,
                "frame {f}: {at} against {anchor}"
            );
        } else if f < 6 + RELEASE_FRAMES {
            // Between where it stood and its generated path, closing in.
            let gap = (at - g[24].translation).length();
            let before = (model(&planted[f - 1])[24].translation - g[24].translation).length();
            assert!(
                gap > 1e-3 && gap < before,
                "frame {f}: {gap} after {before}"
            );
        } else {
            assert_eq!(planted[f], generated[f], "frame {f} is the generated one");
        }
    }
}

// The knee is a hinge. A planted heel the hips drop toward needs the knee
// to bend; when the generated leg is straight, its own knee names no bend
// plane, and the solve must keep the one it last bent in (forward, +Z
// here), not pick an arbitrary one (sideways, or backwards).
#[test]
fn a_straight_generated_leg_bends_its_knee_where_it_last_bent() {
    let asset = skeleton().asset();
    let bend = |a: f32| Quat::from_rotation_x(a);
    let generated: Vec<Vec<Trs>> = (0..10)
        .map(|f| {
            let mut pose = asset.rest_pose();
            // Bent knees standing, then straight legs with the hips 8 cm
            // lower: the planted ankle is 8 cm inside the straight leg.
            pose[0].translation = Vec3::new(0.0, if f < 4 { 0.9 } else { 0.82 }, 0.0);
            if f < 4 {
                for (thigh, shin) in [(22, 23), (26, 27)] {
                    pose[thigh].rotation = bend(-0.3);
                    pose[shin].rotation = bend(0.6);
                }
            }
            pose
        })
        .collect();
    let contacts = vec![[0.9, 0.0, 0.9, 0.0]; 10];
    let mut planted = generated.clone();
    plant_feet(&asset, &mut planted, &contacts);
    for (f, pose) in planted.iter().enumerate().skip(4) {
        let m = asset.model_space(pose);
        for (hip, knee, ankle) in [(22, 23, 24), (26, 27, 28)] {
            let (a, b, c) = (
                m[hip].translation,
                m[knee].translation,
                m[ankle].translation,
            );
            // Still on its anchor (the bent stance's ankle).
            let anchor = asset.model_space(&generated[0])[ankle].translation;
            assert!(
                (c - anchor).length() < 1e-3,
                "frame {f}: ankle {c} off {anchor}"
            );
            // The bent stance's ankle is at y 0.85 − 0.9·cos 0.3 = −0.0097,
            // so the hip (0.77 m) is 0.7797 m over it on two 0.45 m bones:
            // the knee is √(0.45² − 0.38985²) = 0.2248 m forward of that
            // line, and not aside.
            let off = (b - a) - (c - a).normalize() * (b - a).dot((c - a).normalize());
            assert!(
                (off.z - 0.2248).abs() < 0.002 && off.x.abs() < 1e-3,
                "frame {f}: knee off its line by {off}"
            );
        }
    }
}

// A planted heel the hips rise away from: the knee straightens as it nears
// straight-leg reach, never in one step (the knee pop of a hard two-bone
// solve, which straightens the last centimetres of bend in one frame).
#[test]
fn a_leg_reaching_full_extension_straightens_without_a_pop() {
    let asset = skeleton().asset();
    let bend = |a: f32| Quat::from_rotation_x(a);
    let frames = 20;
    let generated: Vec<Vec<Trs>> = (0..frames)
        .map(|f| {
            let mut pose = asset.rest_pose();
            // A knee bent 0.2 rad, the hips rising 4 mm a frame: the
            // planted ankle 0.8955 m from the hip at first, out of the
            // 0.9 m leg's reach by frame 2.
            pose[0].translation = Vec3::new(0.0, 0.9 + 0.004 * f as f32, 0.0);
            pose[22].rotation = bend(-0.1);
            pose[23].rotation = bend(0.2);
            pose[24].rotation = bend(-0.1);
            pose
        })
        .collect();
    let contacts = vec![[0.9, 0.0, 0.0, 0.0]; frames];
    let mut planted = generated.clone();
    plant_feet(&asset, &mut planted, &contacts);
    let knee = |p: &[Trs]| {
        let m = asset.model_space(p);
        let (a, b, c) = (m[22].translation, m[23].translation, m[24].translation);
        let along = (c - a).normalize();
        ((b - a) - along * (b - a).dot(along)).length()
    };
    let bends: Vec<f32> = planted.iter().map(|p| knee(p)).collect();
    // A hard solve takes the knee from 2.2 cm off the line to straight
    // between frames 1 and 2; soft reach eases it over many frames.
    for f in 1..frames {
        let step = (bends[f] - bends[f - 1]).abs();
        assert!(step < 0.012, "frame {f}: knee moved {step} m ({bends:?})");
    }
    // The ankle keeps to its anchor within the softened reach: short of it
    // by no more than the leg's soft room.
    let anchor = asset.model_space(&generated[0])[24].translation;
    for (f, p) in planted.iter().enumerate() {
        let hip = asset.model_space(p)[22].translation;
        let ankle = asset.model_space(p)[24].translation;
        let short = (anchor - hip).length().min(0.9) - (ankle - hip).length();
        assert!(
            short < 0.9 * og_anim::motion::soma::SOFT_REACH + 1e-4,
            "frame {f}: {short}"
        );
    }
}
