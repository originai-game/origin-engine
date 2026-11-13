//! The model's tensor contract: world poses in, the graph's seven inputs,
//! the `[64,366]` motion out, and back to world poses.
//!
//! Ported from og-motion-train's `ogm/encode.py`, itself a port of the
//! MotionBricks runtime's Warp kernels (NVlabs/motionbricks `54fc563`,
//! `motionbricks/model/soma/kernels.py` and `motionbricks/model/kernels.py`).
//! Poses are world transforms of the SOMA skeleton's 30 joints in the joint
//! frames of its USD/BVH rest pose, Y up, metres, 30 Hz. Matrices are row
//! major, `m[row][col]`.

use og_core::math::{atan2, cos, sin, sin_cos};

pub type Vec3 = [f32; 3];
pub type Mat3 = [[f32; 3]; 3];

pub const SAMPLE_RATE: f32 = 30.0;
pub const POSES_PER_TOKEN: usize = 4;
/// Four context slots, then four target slots.
pub const SLOTS: usize = 8;
pub const MIN_TOKENS: usize = 6;
pub const MAX_TOKENS: usize = 16;
/// Token counts a generation may choose from: index `i` is `6 + i` tokens.
pub const COUNTS: usize = MAX_TOKENS - MIN_TOKENS + 1;
pub const FRAMES: usize = MAX_TOKENS * POSES_PER_TOKEN;
/// Frames dropped at each end of a generation (the runtime's
/// `MODEL_BOUNDARY_FRAMES`).
pub const BOUNDARY_FRAMES: usize = POSES_PER_TOKEN - 1;
/// The runtime's default token counts, 8 to 14.
pub const DEFAULT_TOKEN_COUNTS: std::ops::RangeInclusive<usize> = 8..=14;

pub const JOINTS: usize = 30;
pub const JOINT_NAMES: [&str; JOINTS] = [
    "Hips",
    "Spine1",
    "Spine2",
    "Chest",
    "Neck1",
    "Neck2",
    "Head",
    "Jaw",
    "LeftEye",
    "RightEye",
    "LeftShoulder",
    "LeftArm",
    "LeftForeArm",
    "LeftHand",
    "LeftHandThumbEnd",
    "LeftHandMiddleEnd",
    "RightShoulder",
    "RightArm",
    "RightForeArm",
    "RightHand",
    "RightHandThumbEnd",
    "RightHandMiddleEnd",
    "LeftLeg",
    "LeftShin",
    "LeftFoot",
    "LeftToeBase",
    "RightLeg",
    "RightShin",
    "RightFoot",
    "RightToeBase",
];
pub const RIGHT_LEG: usize = 26;
pub const LEFT_LEG: usize = 22;
/// Joints whose contact the motion's last four features give, in order.
pub const CONTACT_JOINTS: [usize; 4] = [24, 25, 28, 29];

/// Per-slot pose features: joint positions 1..29 relative to the hips'
/// ground point (87), then each joint's 6D rotation (180).
pub const POSE_FEATURES: usize = 3 * (JOINTS - 1) + 6 * JOINTS;
pub const MOTION_FEATURES: usize = 366;
/// Ranges of one motion frame's 366 features.
pub const MOTION_ROOT: std::ops::Range<usize> = 0..5;
pub const MOTION_POSITIONS: std::ops::Range<usize> = 5..92;
pub const MOTION_ROTATIONS: std::ops::Range<usize> = 92..272;
pub const MOTION_VELOCITIES: std::ops::Range<usize> = 272..362;
pub const MOTION_CONTACTS: std::ops::Range<usize> = 362..366;

/// One world pose of the 30 joints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub positions: [Vec3; JOINTS],
    pub rotations: [Mat3; JOINTS],
}

/// The seven graph inputs for `n` characters, row major, masks as 0 or 1.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inputs {
    pub n: usize,
    /// `[n,8,5]`: hips x, y, z, cos and sin of the heading.
    pub global_root_values: Vec<f32>,
    pub has_global_root_values: Vec<f32>,
    /// `[n,8,4]`: heading rate (rad/s), ground velocity x and z (m/s), hips height.
    pub local_root_values: Vec<f32>,
    pub has_local_root_values: Vec<f32>,
    /// `[n,8,267]`.
    pub local_poses: Vec<f32>,
    pub has_local_poses: Vec<f32>,
    /// `[n,11]`.
    pub allowed_pred_num_tokens: Vec<f32>,
}

impl Inputs {
    pub fn zeros(n: usize) -> Inputs {
        Inputs {
            n,
            global_root_values: vec![0.0; n * SLOTS * 5],
            has_global_root_values: vec![0.0; n * SLOTS],
            local_root_values: vec![0.0; n * SLOTS * 4],
            has_local_root_values: vec![0.0; n * SLOTS],
            local_poses: vec![0.0; n * SLOTS * POSE_FEATURES],
            has_local_poses: vec![0.0; n * SLOTS],
            allowed_pred_num_tokens: vec![0.0; n * COUNTS],
        }
    }

    /// Character `i`'s inputs alone (`n` is 1).
    pub fn character(&self, i: usize) -> Inputs {
        let take = |v: &[f32]| {
            let per = v.len() / self.n;
            v[i * per..(i + 1) * per].to_vec()
        };
        Inputs {
            n: 1,
            global_root_values: take(&self.global_root_values),
            has_global_root_values: take(&self.has_global_root_values),
            local_root_values: take(&self.local_root_values),
            has_local_root_values: take(&self.has_local_root_values),
            local_poses: take(&self.local_poses),
            has_local_poses: take(&self.has_local_poses),
            allowed_pred_num_tokens: take(&self.allowed_pred_num_tokens),
        }
    }
}

/// Where a character's aligned frame sits in the world: `world = rotation ·
/// aligned + ground`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alignment {
    pub ground: Vec3,
    pub rotation: Mat3,
}

/// One character's request: its four most recent poses, one to four
/// consecutive target poses and the token counts it may use.
#[derive(Clone, Debug)]
pub struct Request<'a> {
    pub context: &'a [Pose; 4],
    /// One to four target poses, consecutive frames at 30 Hz.
    pub targets: &'a [Pose],
    /// Allowed token counts (6..=16); empty means [`DEFAULT_TOKEN_COUNTS`].
    pub token_counts: &'a [usize],
}

pub fn mat_vec(m: &Mat3, v: Vec3) -> Vec3 {
    [0, 1, 2].map(|r| m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2])
}

pub fn mat_mul(a: &Mat3, b: &Mat3) -> Mat3 {
    [0, 1, 2].map(|r| [0, 1, 2].map(|c| a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c]))
}

pub fn transpose(m: &Mat3) -> Mat3 {
    [0, 1, 2].map(|r| [0, 1, 2].map(|c| m[c][r]))
}

/// Rotation about +Y by `angle`.
pub fn rot_y(angle: f32) -> Mat3 {
    let (s, c) = sin_cos(angle);
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

/// Heading from the leg joints' positions: forward = up × (right − left),
/// as the angle `atan2(forward.x, forward.z)`.
pub fn heading(right: Vec3, left: Vec3) -> f32 {
    let (ax, az) = (right[0] - left[0], right[2] - left[2]);
    // sqrt is exactly rounded on every host; hypot is the host libm's.
    let length = (ax * ax + az * az).sqrt();
    if length <= 1e-6 {
        return 0.0;
    }
    atan2(az / length, -ax / length)
}

/// Gram–Schmidt of a 6D rotation (its first two columns) into a matrix.
pub fn rotation_from_6d(d: &[f32]) -> Mat3 {
    let normalize = |v: Vec3| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / l, v[1] / l, v[2] / l]
    };
    let cross = |a: Vec3, b: Vec3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let x = normalize([d[0], d[1], d[2]]);
    let z = normalize(cross(x, [d[3], d[4], d[5]]));
    let y = cross(z, x);
    [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
}

/// The first two columns of a rotation.
pub fn rotation_to_6d(m: &Mat3) -> [f32; 6] {
    [m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1]]
}

/// Encodes requests into the graph inputs, with each character's alignment
/// for decoding its motion.
///
/// Poses are aligned to the last context pose: its hips' ground point at the
/// origin, facing +X. Unused target slots repeat the last target.
pub fn encode(requests: &[Request]) -> (Inputs, Vec<Alignment>) {
    let n = requests.len();
    let mut inputs = Inputs::zeros(n);
    let mut alignments = Vec::with_capacity(n);
    for (i, request) in requests.iter().enumerate() {
        let k = request.targets.len();
        assert!((1..=4).contains(&k), "one to four target poses");
        let slot_pose = |s: usize| {
            if s < 4 {
                &request.context[s]
            } else {
                &request.targets[(s - 4).min(k - 1)]
            }
        };

        let last = &request.context[3];
        let current = heading(last.positions[RIGHT_LEG], last.positions[LEFT_LEG]);
        let desired = atan2(1.0, 0.0);
        let ground = [last.positions[0][0], 0.0, last.positions[0][2]];
        let rotation = rot_y(current - desired);
        let inverse = transpose(&rotation);
        alignments.push(Alignment { ground, rotation });

        let mut positions = [[[0.0f32; 3]; JOINTS]; SLOTS];
        let mut rotations = [[[[0.0f32; 3]; 3]; JOINTS]; SLOTS];
        for s in 0..SLOTS {
            let pose = slot_pose(s);
            for j in 0..JOINTS {
                let p = pose.positions[j];
                positions[s][j] = mat_vec(
                    &inverse,
                    [p[0] - ground[0], p[1] - ground[1], p[2] - ground[2]],
                );
                rotations[s][j] = mat_mul(&inverse, &pose.rotations[j]);
            }
        }
        let headings: [f32; SLOTS] =
            std::array::from_fn(|s| heading(positions[s][RIGHT_LEG], positions[s][LEFT_LEG]));
        for s in 0..SLOTS {
            let next = if s == POSES_PER_TOKEN - 1 {
                s
            } else {
                (s + 1).min(SLOTS - 1)
            };
            let hips = positions[s][0];
            let next_hips = positions[next][0];
            let (ds, dc) = sin_cos(headings[next] - headings[s]);
            let delta = atan2(ds, dc);
            let row = i * SLOTS + s;
            inputs.global_root_values[row * 5..row * 5 + 5].copy_from_slice(&[
                hips[0],
                hips[1],
                hips[2],
                cos(headings[s]),
                sin(headings[s]),
            ]);
            inputs.local_root_values[row * 4..row * 4 + 4].copy_from_slice(&[
                delta * SAMPLE_RATE,
                (next_hips[0] - hips[0]) * SAMPLE_RATE,
                (next_hips[2] - hips[2]) * SAMPLE_RATE,
                hips[1],
            ]);
            let pose = &mut inputs.local_poses[row * POSE_FEATURES..(row + 1) * POSE_FEATURES];
            for j in 1..JOINTS {
                let p = positions[s][j];
                pose[(j - 1) * 3..(j - 1) * 3 + 3].copy_from_slice(&[
                    p[0] - hips[0],
                    p[1],
                    p[2] - hips[2],
                ]);
            }
            for j in 0..JOINTS {
                let at = 87 + j * 6;
                pose[at..at + 6].copy_from_slice(&rotation_to_6d(&rotations[s][j]));
            }
            let given = s < POSES_PER_TOKEN + k;
            let no_derivative = s == POSES_PER_TOKEN - 1 || s == POSES_PER_TOKEN + k - 1;
            let flag = |b: bool| if b { 1.0 } else { 0.0 };
            inputs.has_global_root_values[row] = flag(given);
            inputs.has_local_root_values[row] = flag(given && !no_derivative);
            inputs.has_local_poses[row] = flag(given);
        }
        let allowed = &mut inputs.allowed_pred_num_tokens[i * COUNTS..(i + 1) * COUNTS];
        let counts: Vec<usize> = if request.token_counts.is_empty() {
            DEFAULT_TOKEN_COUNTS.collect()
        } else {
            request.token_counts.to_vec()
        };
        for c in counts {
            assert!(
                (MIN_TOKENS..=MAX_TOKENS).contains(&c),
                "token counts are 6..=16"
            );
            allowed[c - MIN_TOKENS] = 1.0;
        }
    }
    (inputs, alignments)
}

/// The frames of a generation of `count` tokens that are motion: the first
/// and last [`BOUNDARY_FRAMES`] are the context and target overlap.
pub fn valid_frames(count: usize) -> std::ops::Range<usize> {
    BOUNDARY_FRAMES..count * POSES_PER_TOKEN - BOUNDARY_FRAMES
}

/// One frame (366 features) of a character's motion as a world pose.
pub fn decode_frame(frame: &[f32], alignment: &Alignment) -> Pose {
    let r = &alignment.rotation;
    let g = alignment.ground;
    let hips = [frame[0], frame[1], frame[2]];
    let world = |p: Vec3| {
        let v = mat_vec(r, p);
        [v[0] + g[0], v[1] + g[1], v[2] + g[2]]
    };
    let mut positions = [[0.0; 3]; JOINTS];
    positions[0] = world(hips);
    for j in 1..JOINTS {
        let at = MOTION_POSITIONS.start + (j - 1) * 3;
        positions[j] = world([frame[at] + hips[0], frame[at + 1], frame[at + 2] + hips[2]]);
    }
    let rotations = std::array::from_fn(|j| {
        let at = MOTION_ROTATIONS.start + j * 6;
        mat_mul(r, &rotation_from_6d(&frame[at..at + 6]))
    });
    Pose {
        positions,
        rotations,
    }
}
