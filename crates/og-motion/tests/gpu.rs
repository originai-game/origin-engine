//! The wgpu compute path against the CPU path, on the fixture of
//! `reference.rs` (needs a GPU adapter: the runners' Metal and DX12; an orb
//! runs no GPU test). Tokens and counts must be equal; the motion agrees
//! within float reordering.

use og_motion::contract::{FRAMES, MOTION_FEATURES};
use og_motion::glue::{Statics, decoder_inputs, finish, student_inputs};
use og_motion::gpu::Gpu;
use og_motion::{Bundle, Inputs, Model, valid_frames};

fn fixture(name: &str) -> Bundle {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    Bundle::from_safetensors(&std::fs::read(&path).unwrap()).unwrap()
}

fn case_inputs(case: &Bundle) -> Inputs {
    let t = |name: &str| case.tensors[&format!("input.{name}")].data.clone();
    Inputs {
        n: 3,
        global_root_values: t("global_root_values"),
        has_global_root_values: t("has_global_root_values"),
        local_root_values: t("local_root_values"),
        has_local_root_values: t("has_local_root_values"),
        local_poses: t("local_poses"),
        has_local_poses: t("has_local_poses"),
        allowed_pred_num_tokens: t("allowed_pred_num_tokens"),
    }
}

#[test]
fn gpu_generation_matches_the_cpu() {
    let bundle = fixture("tiny.safetensors");
    let inputs = case_inputs(&fixture("case.safetensors"));
    let cpu = Model::new(&bundle).unwrap().generate(&inputs);
    let mut gpu = pollster::block_on(Gpu::request(&bundle)).expect("a GPU adapter");
    let statics = Statics::new(&bundle).unwrap();
    let student = pollster::block_on(gpu.student(&student_inputs(&statics, &inputs))).unwrap();
    assert_eq!(student.counts, cpu.counts, "token counts");
    assert_eq!(student.tokens, cpu.student.tokens, "tokens");
    let root = student
        .root
        .iter()
        .zip(&cpu.student.root)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
    assert!(root < 1e-4, "root path: {root}");
    let local =
        pollster::block_on(gpu.decoder(&decoder_inputs(&statics, &inputs, &student))).unwrap();
    let motion = finish(&statics, &inputs, &student, &local);
    for (i, &count) in cpu.counts.iter().enumerate() {
        for f in valid_frames(count as usize) {
            let at = (i * FRAMES + f) * MOTION_FEATURES;
            let d = (at..at + MOTION_FEATURES)
                .map(|k| (motion[k] - cpu.motion[k]).abs())
                .fold(0.0, f32::max);
            assert!(d < 1e-3, "character {i} frame {f}: {d}");
        }
    }
}
