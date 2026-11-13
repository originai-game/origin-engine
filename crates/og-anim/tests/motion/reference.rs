//! The CPU path against og-motion-train's `ogm/reference.py`, on the fixture
//! its `scripts/make_fixture.py` wrote: a random tiny bundle (no trained or
//! NVIDIA number) and three characters with one, two and four targets and
//! different allowed token counts.

use og_anim::motion::contract::{FRAMES, JOINTS, MOTION_FEATURES, POSE_FEATURES, Pose, SLOTS};
use og_anim::motion::{Bundle, Inputs, Model, Request, encode, valid_frames};

fn fixture(name: &str) -> Bundle {
    let path = format!(
        "{}/tests/motion/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    Bundle::from_safetensors(&std::fs::read(&path).unwrap()).unwrap()
}

fn tensor<'a>(case: &'a Bundle, name: &str) -> &'a [f32] {
    &case
        .tensors
        .get(name)
        .unwrap_or_else(|| panic!("{name}"))
        .data
}

fn poses(
    case: &Bundle,
    positions: &str,
    rotations: &str,
    character: usize,
    count: usize,
) -> Vec<Pose> {
    let p = tensor(case, positions);
    let r = tensor(case, rotations);
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

fn case_inputs(case: &Bundle) -> Inputs {
    let n = 3;
    Inputs {
        n,
        global_root_values: tensor(case, "input.global_root_values").to_vec(),
        has_global_root_values: tensor(case, "input.has_global_root_values").to_vec(),
        local_root_values: tensor(case, "input.local_root_values").to_vec(),
        has_local_root_values: tensor(case, "input.has_local_root_values").to_vec(),
        local_poses: tensor(case, "input.local_poses").to_vec(),
        has_local_poses: tensor(case, "input.has_local_poses").to_vec(),
        allowed_pred_num_tokens: tensor(case, "input.allowed_pred_num_tokens").to_vec(),
    }
}

fn max_diff(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

#[test]
fn encode_matches_the_python_encoding() {
    let case = fixture("case.safetensors");
    let targets_per = tensor(&case, "num_targets").to_vec();
    let contexts: Vec<[Pose; 4]> = (0..3)
        .map(|i| {
            poses(&case, "context_positions", "context_rotations", i, 4)
                .try_into()
                .unwrap()
        })
        .collect();
    let targets: Vec<Vec<Pose>> = (0..3)
        .map(|i| {
            poses(
                &case,
                "target_positions",
                "target_rotations",
                i,
                targets_per[i] as usize,
            )
        })
        .collect();
    let allowed = tensor(&case, "input.allowed_pred_num_tokens");
    let counts: Vec<Vec<usize>> = (0..3)
        .map(|i| {
            (0..11)
                .filter(|c| allowed[i * 11 + c] > 0.0)
                .map(|c| c + 6)
                .collect()
        })
        .collect();
    let requests: Vec<Request> = (0..3)
        .map(|i| Request {
            context: &contexts[i],
            targets: &targets[i],
            token_counts: &counts[i],
        })
        .collect();
    let (inputs, alignments) = encode(&requests);
    let expected = case_inputs(&case);
    // Joint positions are metres: 1e-4 is a tenth of a millimetre.
    assert!(max_diff(&inputs.global_root_values, &expected.global_root_values) < 1e-4);
    assert!(max_diff(&inputs.local_root_values, &expected.local_root_values) < 2e-3); // rates ×30
    assert!(max_diff(&inputs.local_poses, &expected.local_poses) < 1e-4);
    assert_eq!(
        inputs.has_global_root_values,
        expected.has_global_root_values
    );
    assert_eq!(inputs.has_local_root_values, expected.has_local_root_values);
    assert_eq!(inputs.has_local_poses, expected.has_local_poses);
    assert_eq!(
        inputs.allowed_pred_num_tokens,
        expected.allowed_pred_num_tokens
    );
    let ground = tensor(&case, "align_ground");
    for (i, a) in alignments.iter().enumerate() {
        assert!(max_diff(&a.ground, &ground[i * 3..i * 3 + 3]) < 1e-5);
    }
    assert_eq!(inputs.local_poses.len(), 3 * SLOTS * POSE_FEATURES);
}

#[test]
fn cpu_generation_matches_the_reference() {
    let case = fixture("case.safetensors");
    let model = Model::new(&fixture("tiny.safetensors")).unwrap();
    let generation = model.generate(&case_inputs(&case));
    let counts: Vec<u32> = tensor(&case, "expected.counts")
        .iter()
        .map(|v| *v as u32)
        .collect();
    assert_eq!(generation.counts, counts, "token counts");
    assert_eq!(
        counts,
        [8, 11, 16],
        "the fixture covers three different counts"
    );
    let tokens: Vec<u32> = tensor(&case, "expected.tokens")
        .iter()
        .map(|v| *v as u32)
        .collect();
    assert_eq!(generation.student.tokens, tokens, "tokens");
    assert!(
        max_diff(&generation.student.root, tensor(&case, "expected.root")) < 1e-4,
        "root path"
    );
    let expected = tensor(&case, "expected.motion");
    for (i, &count) in counts.iter().enumerate() {
        for f in valid_frames(count as usize) {
            let at = (i * FRAMES + f) * MOTION_FEATURES;
            let d = max_diff(
                &generation.motion[at..at + MOTION_FEATURES],
                &expected[at..at + MOTION_FEATURES],
            );
            assert!(d < 1e-3, "character {i} frame {f}: {d}");
        }
    }
}
