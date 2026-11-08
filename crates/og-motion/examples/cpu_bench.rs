//! Times the CPU path's generation (median of runs) and prints a hash of
//! its output, natively or under wasm32-wasip1:
//!
//!     cargo run --release -p og-motion --example cpu_bench -- <bundle.safetensors> <case.safetensors> [n...]

use std::time::Instant;

use og_motion::{Bundle, Inputs, Model};

fn read(path: &str) -> Bundle {
    Bundle::from_safetensors(&std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}")))
        .unwrap()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bundle = read(&args[1]);
    let case = read(&args[2]);
    let t = |name: &str| case.tensors[&format!("input.{name}")].data.clone();
    let base = Inputs {
        n: case.tensors["input.allowed_pred_num_tokens"].data.len() / 11,
        global_root_values: t("global_root_values"),
        has_global_root_values: t("has_global_root_values"),
        local_root_values: t("local_root_values"),
        has_local_root_values: t("has_local_root_values"),
        local_poses: t("local_poses"),
        has_local_poses: t("has_local_poses"),
        allowed_pred_num_tokens: t("allowed_pred_num_tokens"),
    };
    let model = Model::new(&bundle).unwrap();
    let ns: Vec<usize> = args[3..].iter().map(|a| a.parse().unwrap()).collect();
    for n in if ns.is_empty() { vec![1, 16] } else { ns } {
        let take = |v: &[f32]| {
            let per = v.len() / base.n;
            (0..n)
                .flat_map(|i| v[(i % base.n) * per..(i % base.n + 1) * per].to_vec())
                .collect()
        };
        let inputs = Inputs {
            n,
            global_root_values: take(&base.global_root_values),
            has_global_root_values: take(&base.has_global_root_values),
            local_root_values: take(&base.local_root_values),
            has_local_root_values: take(&base.has_local_root_values),
            local_poses: take(&base.local_poses),
            has_local_poses: take(&base.has_local_poses),
            allowed_pred_num_tokens: take(&base.allowed_pred_num_tokens),
        };
        let mut times = Vec::new();
        let mut hash = 0u64;
        for _ in 0..5 {
            let start = Instant::now();
            let g = model.generate(&inputs);
            times.push(start.elapsed().as_secs_f64() * 1e3);
            hash = g
                .motion
                .iter()
                .chain(&g.student.root)
                .fold(0xcbf29ce484222325, |h, v| {
                    (h ^ u64::from(v.to_bits())).wrapping_mul(0x100000001b3)
                });
        }
        // The same characters split over 8 threads, one generation each.
        if n >= 8 {
            let per = n / 8;
            let one = |k: usize| {
                let pick = |v: &[f32]| {
                    let w = v.len() / n;
                    v[k * per * w..(k + 1) * per * w].to_vec()
                };
                Inputs {
                    n: per,
                    global_root_values: pick(&inputs.global_root_values),
                    has_global_root_values: pick(&inputs.has_global_root_values),
                    local_root_values: pick(&inputs.local_root_values),
                    has_local_root_values: pick(&inputs.has_local_root_values),
                    local_poses: pick(&inputs.local_poses),
                    has_local_poses: pick(&inputs.has_local_poses),
                    allowed_pred_num_tokens: pick(&inputs.allowed_pred_num_tokens),
                }
            };
            let parts: Vec<Inputs> = (0..8).map(one).collect();
            let start = Instant::now();
            std::thread::scope(|s| {
                for p in &parts {
                    let m = &model;
                    s.spawn(move || m.generate(p));
                }
            });
            println!(
                "cpu, {n:>3} characters on 8 threads: {:.1} ms ({:.2} ms a character)",
                start.elapsed().as_secs_f64() * 1e3,
                start.elapsed().as_secs_f64() * 1e3 / n as f64
            );
        }
        times.sort_by(f64::total_cmp);

        println!(
            "cpu, {n:>3} characters, one thread: median {:.1} ms ({:.1} ms a character), output hash {hash:016x}",
            times[2],
            times[2] / n as f64
        );
    }
}
