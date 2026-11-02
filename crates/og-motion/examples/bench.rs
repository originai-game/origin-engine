//! Times one generation on the GPU (and the CPU path for the first case) and,
//! given a case file with expected outputs, checks the GPU against it.
//!
//!     cargo run --release -p og-motion --example bench -- <bundle.safetensors> [case.safetensors]
//!
//! A case file is og-motion-train's: `input.*` tensors (the seven graph inputs)
//! and `expected.counts`, `expected.tokens`, `expected.motion`.

use std::time::Instant;

use og_motion::contract::{FRAMES, MOTION_FEATURES};
use og_motion::glue::{Statics, decoder_inputs, finish, student_inputs};
use og_motion::gpu::Gpu;
use og_motion::{Bundle, Inputs, Model, valid_frames};

fn read(path: &str) -> Bundle {
    Bundle::from_safetensors(&std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}")))
        .unwrap()
}

fn repeat(inputs: &Inputs, n: usize) -> Inputs {
    let take = |v: &[f32]| {
        let per = v.len() / inputs.n;
        (0..n)
            .flat_map(|i| v[(i % inputs.n) * per..(i % inputs.n + 1) * per].to_vec())
            .collect()
    };
    Inputs {
        n,
        global_root_values: take(&inputs.global_root_values),
        has_global_root_values: take(&inputs.has_global_root_values),
        local_root_values: take(&inputs.local_root_values),
        has_local_root_values: take(&inputs.has_local_root_values),
        local_poses: take(&inputs.local_poses),
        has_local_poses: take(&inputs.has_local_poses),
        allowed_pred_num_tokens: take(&inputs.allowed_pred_num_tokens),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bundle = read(&args[1]);
    println!(
        "bundle: {} parameters, {:?}",
        bundle.parameters(),
        bundle.config.student
    );
    let statics = Statics::new(&bundle).unwrap();
    let start = Instant::now();
    let gpu = pollster::block_on(Gpu::request(&bundle)).unwrap();
    println!("gpu ready in {:.0} ms", start.elapsed().as_secs_f64() * 1e3);
    let case = args.get(2).map(|p| read(p));
    let base = match &case {
        Some(c) => {
            let t = |name: &str| c.tensors[&format!("input.{name}")].data.clone();
            let n = c.tensors["input.allowed_pred_num_tokens"].data.len() / 11;
            Inputs {
                n,
                global_root_values: t("global_root_values"),
                has_global_root_values: t("has_global_root_values"),
                local_root_values: t("local_root_values"),
                has_local_root_values: t("has_local_root_values"),
                local_poses: t("local_poses"),
                has_local_poses: t("has_local_poses"),
                allowed_pred_num_tokens: t("allowed_pred_num_tokens"),
            }
        }
        None => panic!("a case file is needed for inputs"),
    };
    let generate = |inputs: &Inputs| {
        let student = pollster::block_on(gpu.student(&student_inputs(&statics, inputs))).unwrap();
        let local =
            pollster::block_on(gpu.decoder(&decoder_inputs(&statics, inputs, &student))).unwrap();
        let motion = finish(&statics, inputs, &student, &local);
        (student, motion)
    };
    if let Some(c) = &case {
        let (student, motion) = generate(&base);
        let counts: Vec<u32> = c.tensors["expected.counts"]
            .data
            .iter()
            .map(|v| *v as u32)
            .collect();
        let tokens: Vec<u32> = c.tensors["expected.tokens"]
            .data
            .iter()
            .map(|v| *v as u32)
            .collect();
        let same = student
            .tokens
            .iter()
            .zip(&tokens)
            .filter(|(a, b)| a == b)
            .count();
        let expected = &c.tensors["expected.motion"].data;
        let mut worst = 0f32;
        for (i, &n) in counts.iter().enumerate() {
            for f in valid_frames(n as usize) {
                let at = (i * FRAMES + f) * MOTION_FEATURES;
                for k in at..at + MOTION_FEATURES {
                    worst = worst.max((motion[k] - expected[k]).abs());
                }
            }
        }
        println!(
            "against the reference: counts equal {}, tokens equal {same}/{}, max |motion diff| {worst:.2e}",
            student.counts == counts,
            tokens.len()
        );
        let model = Model::new(&bundle).unwrap();
        let one = repeat(&base, 1);
        let start = Instant::now();
        let _ = model.generate(&one);
        println!(
            "cpu, 1 character, one thread: {:.1} ms",
            start.elapsed().as_secs_f64() * 1e3
        );
    }
    for n in [1usize, 8, 32, 128, 256] {
        let inputs = repeat(&base, n);
        let _ = generate(&inputs); // warm: pipelines and buffers of this size
        let runs = 10;
        let start = Instant::now();
        for _ in 0..runs {
            let _ = generate(&inputs);
        }
        let ms = start.elapsed().as_secs_f64() * 1e3 / runs as f64;
        let start = Instant::now();
        let mut student = None;
        for _ in 0..runs {
            student =
                Some(pollster::block_on(gpu.student(&student_inputs(&statics, &inputs))).unwrap());
        }
        let student_ms = start.elapsed().as_secs_f64() * 1e3 / runs as f64;
        let decoder = decoder_inputs(&statics, &inputs, &student.unwrap());
        let start = Instant::now();
        for _ in 0..runs {
            let _ = pollster::block_on(gpu.decoder(&decoder)).unwrap();
        }
        let decoder_ms = start.elapsed().as_secs_f64() * 1e3 / runs as f64;
        println!(
            "gpu, {n:>3} characters: {ms:.1} ms per generation of the batch ({:.2} ms per character; student {student_ms:.1}, decoder {decoder_ms:.1})",
            ms / n as f64
        );
    }
}
