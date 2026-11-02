//! Everything around the two networks, shared by the CPU and GPU paths.
//!
//! The steps of og-motion-train's `ogm/reference.py`, each reproducing an
//! NVIDIA function (NVlabs/motionbricks `54fc563`, `tools/onnx/models`):
//! [`prepare`] (input rotations to the standard T pose, the student's slot
//! features, the normalised pose condition), [`decoder_inputs`] (the root
//! path's local root values, `global_root_to_local_onnx_compatible`, the
//! dense pose condition and the codebook lookup) and [`finish`]
//! (`local_to_global`, the `from_height_constraint_violation` root choice,
//! `_reapply_initial_root_info`, the rotations back to the capture T pose).

use og_core::math::{atan2, sin_cos};

use crate::bundle::{Bundle, BundleError};
use crate::contract::{
    FRAMES, Inputs, JOINTS, MAX_TOKENS, MOTION_FEATURES, POSE_FEATURES, POSES_PER_TOKEN,
    SAMPLE_RATE, SLOTS, mat_mul, rotation_from_6d, rotation_to_6d, transpose,
};

pub const HEADS: usize = 5;
pub const CODES: usize = 16;
pub const CODE_DIM: usize = 32;
/// The student's per-slot features: global root 5, local root 4, pose 267, three masks.
pub const SLOT_FEATURES: usize = 5 + 4 + POSE_FEATURES + 3;
/// The decoder's per-frame pose condition: hips height and the pose features.
pub const TARGET_FEATURES: usize = 1 + POSE_FEATURES;
/// The decoder's normalised local motion features: local root 4, body 361.
pub const LOCAL_FEATURES: usize = 365;
const STATS: usize = 370;
const LOCAL_ROOT: usize = 5;
const BODY: usize = 9;
const BODY_ROTATIONS: std::ops::Range<usize> = 87..267;
const EPS: f32 = 1e-5;

/// The fixed numbers the glue reads from the bundle.
#[derive(Clone, Debug)]
pub struct Statics {
    mean: Vec<f32>,
    scale: Vec<f32>,
    student_mean: Vec<f32>,
    student_std: Vec<f32>,
    pub codebook: Vec<f32>,
    capture: Vec<[[f32; 3]; 3]>,
    standard: Vec<[[f32; 3]; 3]>,
}

fn matrices(data: &[f32]) -> Vec<[[f32; 3]; 3]> {
    data.as_chunks::<9>()
        .0
        .iter()
        .map(|m| [[m[0], m[1], m[2]], [m[3], m[4], m[5]], [m[6], m[7], m[8]]])
        .collect()
}

impl Statics {
    pub fn new(bundle: &Bundle) -> Result<Statics, BundleError> {
        let std = bundle.get("stats.std", &[STATS])?;
        Ok(Statics {
            mean: bundle.get("stats.mean", &[STATS])?.to_vec(),
            scale: std.iter().map(|s| (s * s + EPS).sqrt()).collect(),
            student_mean: bundle.get("student.mean", &[SLOT_FEATURES])?.to_vec(),
            student_std: bundle.get("student.std", &[SLOT_FEATURES])?.to_vec(),
            codebook: bundle.get("codebook", &[HEADS, CODES, CODE_DIM])?.to_vec(),
            capture: matrices(bundle.get("capture_offsets", &[JOINTS, 3, 3])?),
            standard: matrices(bundle.get("standard_offsets", &[JOINTS, 3, 3])?),
        })
    }

    fn norm(&self, x: f32, index: usize) -> f32 {
        (x - self.mean[index]) / self.scale[index]
    }

    fn unnorm(&self, x: f32, index: usize) -> f32 {
        x * self.scale[index] + self.mean[index]
    }

    /// 6D global rotations of the 30 joints → `R · from · toᵀ`, in place.
    fn change_t_pose(rotations: &mut [f32], from: &[[[f32; 3]; 3]], to: &[[[f32; 3]; 3]]) {
        for j in 0..JOINTS {
            let r = rotation_from_6d(&rotations[j * 6..j * 6 + 6]);
            let m = mat_mul(&mat_mul(&r, &from[j]), &transpose(&to[j]));
            rotations[j * 6..j * 6 + 6].copy_from_slice(&rotation_to_6d(&m));
        }
    }
}

/// What the student reads: its normalised slot features and allowed counts.
#[derive(Clone, Debug)]
pub struct StudentInputs {
    pub n: usize,
    /// `[n,8,279]`.
    pub slots: Vec<f32>,
    /// `[n,11]`.
    pub allowed: Vec<f32>,
}

/// The student's run-time output.
#[derive(Clone, Debug, PartialEq)]
pub struct StudentOutput {
    /// Chosen token counts, 6..=16.
    pub counts: Vec<u32>,
    /// `[n,64,5]` hips x, y, z, cos and sin of heading, aligned frame.
    pub root: Vec<f32>,
    /// `[n,16,5]` per-head code indices.
    pub tokens: Vec<u32>,
}

/// What the decoder reads.
#[derive(Clone, Debug)]
pub struct DecoderInputs {
    pub n: usize,
    /// `[n,160,16]`: codes per token, channels first.
    pub codes: Vec<f32>,
    /// `[n,16]`: 1 for tokens below the count.
    pub token_mask: Vec<f32>,
    /// `[n,64,2]`: normalised ground velocity of the root path.
    pub external: Vec<f32>,
    /// `[n,64,268]`: the normalised pose condition on conditioned frames.
    pub target: Vec<f32>,
    /// `[n,64]`: 1 on conditioned frames.
    pub has_target: Vec<f32>,
}

/// The student's inputs.
pub fn student_inputs(statics: &Statics, inputs: &Inputs) -> StudentInputs {
    let n = inputs.n;
    let mut slots = vec![0.0; n * SLOTS * SLOT_FEATURES];
    for row in 0..n * SLOTS {
        let s = &mut slots[row * SLOT_FEATURES..(row + 1) * SLOT_FEATURES];
        s[0..5].copy_from_slice(&inputs.global_root_values[row * 5..row * 5 + 5]);
        s[5..9].copy_from_slice(&inputs.local_root_values[row * 4..row * 4 + 4]);
        s[9..9 + POSE_FEATURES]
            .copy_from_slice(&inputs.local_poses[row * POSE_FEATURES..(row + 1) * POSE_FEATURES]);
        s[276] = inputs.has_global_root_values[row];
        s[277] = inputs.has_local_root_values[row];
        s[278] = inputs.has_local_poses[row];
        for (i, v) in s.iter_mut().enumerate() {
            *v = (*v - statics.student_mean[i]) / statics.student_std[i];
        }
    }
    StudentInputs {
        n,
        slots,
        allowed: inputs.allowed_pred_num_tokens.clone(),
    }
}

/// The root path recentred on the first context frame, as the teacher's
/// inference holds it.
fn recentred_root(inputs: &Inputs, student: &StudentOutput, i: usize) -> Vec<[f32; 5]> {
    let (ox, oz) = (
        inputs.global_root_values[i * SLOTS * 5],
        inputs.global_root_values[i * SLOTS * 5 + 2],
    );
    (0..FRAMES)
        .map(|f| {
            let r = &student.root[(i * FRAMES + f) * 5..(i * FRAMES + f) * 5 + 5];
            [r[0] - ox, r[1], r[2] - oz, r[3], r[4]]
        })
        .collect()
}

/// The decoder's inputs for the student's count, root path and tokens.
pub fn decoder_inputs(
    statics: &Statics,
    inputs: &Inputs,
    student: &StudentOutput,
) -> DecoderInputs {
    let n = inputs.n;
    let mut out = DecoderInputs {
        n,
        codes: vec![0.0; n * HEADS * CODE_DIM * MAX_TOKENS],
        token_mask: vec![0.0; n * MAX_TOKENS],
        external: vec![0.0; n * FRAMES * 2],
        target: vec![0.0; n * FRAMES * TARGET_FEATURES],
        has_target: vec![0.0; n * FRAMES],
    };
    for i in 0..n {
        let count = student.counts[i] as usize;
        let frames = count * POSES_PER_TOKEN;
        for t in 0..MAX_TOKENS {
            out.token_mask[i * MAX_TOKENS + t] = if t < count { 1.0 } else { 0.0 };
            for h in 0..HEADS {
                let code = student.tokens[(i * MAX_TOKENS + t) * HEADS + h] as usize;
                for d in 0..CODE_DIM {
                    let channel = h * CODE_DIM + d;
                    out.codes[(i * HEADS * CODE_DIM + channel) * MAX_TOKENS + t] =
                        statics.codebook[(h * CODES + code) * CODE_DIM + d];
                }
            }
        }

        // The root path's ground velocity by forward differences, the last frame
        // repeating the one before it (its heading rate and height are local root
        // values too, but the decoder reads only the velocity).
        let root = recentred_root(inputs, student, i);
        let mut velocity = vec![[0.0f32; 2]; FRAMES];
        for f in 0..FRAMES - 1 {
            velocity[f] = [
                SAMPLE_RATE * (root[f + 1][0] - root[f][0]),
                SAMPLE_RATE * (root[f + 1][2] - root[f][2]),
            ];
        }
        velocity[frames - 1] = velocity[frames - 2];
        let given = inputs.has_local_root_values[i * SLOTS + SLOTS - 1] > 0.0;
        for f in 0..FRAMES {
            let mut v = velocity[f];
            if given && f == frames - 1 {
                // The given final local root replaces the path's.
                let l = &inputs.local_root_values[(i * SLOTS + SLOTS - 1) * 4..];
                v = [l[1], l[2]];
            }
            out.external[(i * FRAMES + f) * 2] = statics.norm(v[0], LOCAL_ROOT + 1);
            out.external[(i * FRAMES + f) * 2 + 1] = statics.norm(v[1], LOCAL_ROOT + 2);
        }

        // Dense pose condition: context slots on frames 0-3, targets ending the motion.
        for s in 0..SLOTS {
            let frame = if s < 4 {
                s
            } else {
                frames - POSES_PER_TOKEN + (s - 4)
            };
            let row = i * SLOTS + s;
            if inputs.has_local_poses[row] > 0.0 {
                out.has_target[i * FRAMES + frame] = 1.0;
            }
            let mut pose =
                inputs.local_poses[row * POSE_FEATURES..(row + 1) * POSE_FEATURES].to_vec();
            Statics::change_t_pose(
                &mut pose[BODY_ROTATIONS],
                &statics.capture,
                &statics.standard,
            );
            let target = &mut out.target[(i * FRAMES + frame) * TARGET_FEATURES
                ..(i * FRAMES + frame + 1) * TARGET_FEATURES];
            // Feature indices of the local representation: hips height, then the pose.
            target[0] = statics.norm(inputs.global_root_values[row * 5 + 1], LOCAL_ROOT + 3);
            for (k, v) in pose.iter().enumerate() {
                target[1 + k] = statics.norm(*v, BODY + k);
            }
        }
    }
    out
}

/// The motion `[n,64,366]` from the decoder's normalised local features `[n,64,365]`.
pub fn finish(
    statics: &Statics,
    inputs: &Inputs,
    student: &StudentOutput,
    local: &[f32],
) -> Vec<f32> {
    let n = inputs.n;
    let mut motion = vec![0.0; n * FRAMES * MOTION_FEATURES];
    for i in 0..n {
        let count = student.counts[i] as usize;
        let g0 = &inputs.global_root_values[i * SLOTS * 5..i * SLOTS * 5 + 5];
        let (ox, oz) = (g0[0], g0[2]);
        let reference_heading = atan2(g0[4], g0[3]);

        // Local to global: integrate heading rate and ground velocity from frame 0.
        let mut global_root = vec![[0.0f32; 5]; FRAMES];
        let (mut heading, mut x, mut z) = (0.0f32, 0.0f32, 0.0f32);
        for f in 0..FRAMES {
            let l = &local[(i * FRAMES + f) * LOCAL_FEATURES..];
            let raw: [f32; 4] = std::array::from_fn(|k| statics.unnorm(l[k], LOCAL_ROOT + k));
            let (hs, hc) = sin_cos(heading);
            global_root[f] = [x, raw[3], z, hc, hs];
            heading += raw[0] / SAMPLE_RATE;
            x += raw[1] / SAMPLE_RATE;
            z += raw[2] / SAMPLE_RATE;
        }

        // The root path's hips replace the decoder's when the decoder misses the
        // target heights by more than 5 cm and the path misses them by less.
        let root = recentred_root(inputs, student, i);
        let (mut from_pose, mut from_path, mut weight) = (0.0f32, 0.0f32, 0.0f32);
        for k in 0..POSES_PER_TOKEN {
            let slot = i * SLOTS + 4 + k;
            let has = inputs.has_global_root_values[slot];
            let goal = inputs.global_root_values[slot * 5 + 1];
            let frame = (count - 1) * POSES_PER_TOKEN + k;
            from_pose += (global_root[frame][1] - goal).abs() * has;
            from_path += (root[frame][1] - goal).abs() * has;
            weight += has;
        }
        let (from_pose, from_path) = (from_pose / weight, from_path / weight);
        if from_path < from_pose && from_pose > 0.05 {
            global_root.copy_from_slice(&root);
        }

        for f in 0..FRAMES {
            let out = &mut motion
                [(i * FRAMES + f) * MOTION_FEATURES..(i * FRAMES + f + 1) * MOTION_FEATURES];
            let r = global_root[f];
            // Back to the caller's frame: the first frame's heading and ground position.
            let (sa, ca) = sin_cos(atan2(r[4], r[3]) + reference_heading);
            out[0..5].copy_from_slice(&[r[0] + ox, r[1], r[2] + oz, ca, sa]);
            let l = &local
                [(i * FRAMES + f) * LOCAL_FEATURES + 4..(i * FRAMES + f + 1) * LOCAL_FEATURES];
            for (k, v) in l.iter().enumerate() {
                out[5 + k] = statics.unnorm(*v, BODY + k);
            }
            Statics::change_t_pose(
                &mut out[5 + BODY_ROTATIONS.start..5 + BODY_ROTATIONS.end],
                &statics.standard,
                &statics.capture,
            );
        }
    }
    motion
}
