//! Chains generations through a course of target poses on the GPU and
//! writes the motion as world joint positions, for viewing.
//!
//!     cargo run --release -p og-motion --example course -- <bundle> <course.safetensors> <out.json>
//!
//! A course (og-motion-train's `scripts/demo_course.py`) holds one
//! character's four context poses (`context_positions` [4,30,3],
//! `context_rotations` [4,30,3,3]) and its targets in the world
//! (`target_positions` [S,4,30,3], `target_rotations` [S,4,30,3,3],
//! `num_targets` [S]). Each step generates from the last four frames of the
//! one before; the motion is the context and each step's frames after its
//! first (which repeats the previous step's last).

use std::time::Instant;

use og_motion::contract::{JOINTS, MOTION_CONTACTS, Pose};
use og_motion::gpu::Gpu;
use og_motion::{Bundle, Request, decode_frame, encode, read_safetensors, valid_frames};

fn poses(positions: &[f32], rotations: &[f32], first: usize, count: usize) -> Vec<Pose> {
    (first..first + count)
        .map(|f| Pose {
            positions: std::array::from_fn(|j| {
                std::array::from_fn(|k| positions[(f * JOINTS + j) * 3 + k])
            }),
            rotations: std::array::from_fn(|j| {
                std::array::from_fn(|a| {
                    std::array::from_fn(|b| rotations[((f * JOINTS + j) * 3 + a) * 3 + b])
                })
            }),
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bundle = Bundle::from_safetensors(&std::fs::read(&args[1]).unwrap()).unwrap();
    let (course, _) = read_safetensors(&std::fs::read(&args[2]).unwrap()).unwrap();
    let t = |name: &str| &course[name].data;
    let gpu = pollster::block_on(Gpu::request(&bundle)).unwrap();
    let mut context: [Pose; 4] = poses(t("context_positions"), t("context_rotations"), 0, 4)
        .try_into()
        .unwrap();
    let mut frames: Vec<Pose> = context.to_vec();
    let mut contacts: Vec<[f32; 4]> = vec![[1.0; 4]; 4];
    let mut steps = Vec::new();
    let num_targets = t("num_targets");
    let start = Instant::now();
    for (step, k) in num_targets.iter().enumerate() {
        let targets = poses(
            t("target_positions"),
            t("target_rotations"),
            step * 4,
            *k as usize,
        );
        let request = Request {
            context: &context,
            targets: &targets,
            token_counts: &[],
        };
        let (inputs, alignments) = encode(std::slice::from_ref(&request));
        let generation = pollster::block_on(gpu.generate(&inputs)).unwrap();
        let count = generation.counts[0] as usize;
        let valid = valid_frames(count);
        steps.push(serde_json::json!({ "count": count, "start": frames.len() }));
        for f in valid.start + 1..valid.end {
            let frame = generation.frame(0, f);
            frames.push(decode_frame(frame, &alignments[0]));
            contacts.push(std::array::from_fn(|c| frame[MOTION_CONTACTS.start + c]));
        }
        context = std::array::from_fn(|i| frames[frames.len() - 4 + i]);
    }
    println!(
        "{} steps, {} frames in {:.0} ms",
        steps.len(),
        frames.len(),
        start.elapsed().as_secs_f64() * 1e3
    );
    let positions: Vec<f32> = frames
        .iter()
        .flat_map(|p| p.positions.iter().flatten().copied())
        .collect();
    let out = serde_json::json!({
        "frames": frames.len(),
        "positions": positions,
        "contacts": contacts.iter().flatten().collect::<Vec<_>>(),
        "steps": steps,
    });
    std::fs::write(&args[3], out.to_string()).unwrap();
}
