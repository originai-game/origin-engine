//! The intent API and the generated skeleton: goals chained through
//! generations on the fixture bundle (a random tiny bundle, see
//! reference.rs), goals placed in the world, and feet planted from contacts
//! on a synthetic skeleton.

use og_anim::Trs;
use og_core::glam::{Quat, Vec3};
use og_motion::contract::{FRAMES, JOINTS, LEFT_LEG, MOTION_FEATURES, Pose, RIGHT_LEG, heading};
use og_motion::intent::{Goal, Motion};
use og_motion::soma::{RELEASE_FRAMES, Skeleton, plant_feet};
use og_motion::{Bundle, Model, Request, decode_frame, encode, valid_frames};

fn fixture(name: &str) -> Bundle {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
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
    let mut shown: Vec<Vec<Pose>> = Vec::new();
    for (i, &count) in [8usize, 11, 16].iter().enumerate() {
        let valid = valid_frames(count);
        assert_eq!(motion.queued(i), valid.len() - 1);
        let mut frames = Vec::new();
        for f in valid.start + 1..valid.end {
            let frame = motion.pop(i).unwrap();
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

#[test]
fn a_placed_goal_stands_where_and_faces_how_it_was_told() {
    let case = fixture("case.safetensors");
    let source = poses(&case, "target", 2, 4);
    let goal = Goal::placed(&source, [3.0, -2.0], 1.2);
    let first = &goal.targets[0];
    assert!((first.positions[0][0] - 3.0).abs() < 1e-5);
    assert!((first.positions[0][2] + 2.0).abs() < 1e-5);
    assert!((first.positions[0][1] - source[0].positions[0][1]).abs() < 1e-6);
    let h = heading(first.positions[RIGHT_LEG], first.positions[LEFT_LEG]);
    assert!((h - 1.2).abs() < 1e-4, "{h}");
    // The later frames keep their offsets from the first, turned with it.
    let step = |p: &[Pose], k: usize| {
        let a = p[k].positions[0];
        let b = p[0].positions[0];
        ((a[0] - b[0]).hypot(a[2] - b[2]), a[1] - b[1])
    };
    for k in 1..4 {
        let (d0, y0) = step(&source, k);
        let (d1, y1) = step(&goal.targets, k);
        assert!((d0 - d1).abs() < 1e-5 && (y0 - y1).abs() < 1e-6);
    }
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
