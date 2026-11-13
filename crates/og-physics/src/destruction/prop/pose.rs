//! Poses as og.destruction places its chunks with them.

use glam::{Quat, Vec3};

/// og.Transform's position, rotation and scale.
pub type Pose = (Vec3, Quat, Vec3);

pub fn pose_of_frame(f: &[f32; 10]) -> Pose {
    (
        Vec3::new(f[0], f[1], f[2]),
        Quat::from_xyzw(f[3], f[4], f[5], f[6]),
        Vec3::new(f[7], f[8], f[9]),
    )
}

pub fn frame_of_pose((p, q, s): Pose) -> [f32; 10] {
    [p.x, p.y, p.z, q.x, q.y, q.z, q.w, s.x, s.y, s.z]
}

/// A uniform scale's factor (the mean of the axes' magnitudes).
pub fn uniform(s: Vec3) -> f32 {
    (s.x.abs() + s.y.abs() + s.z.abs()) / 3.0
}

/// `p` of model space, on a chunk whose model-space centre is `c`, placed
/// by the chunk's world pose.
pub fn place_on(pose: &Pose, c: Vec3, p: Vec3) -> Vec3 {
    pose.0 + pose.1 * (pose.2 * (p - c))
}

/// `child`'s pose relative to `root`'s, both in the world.
pub fn relative(root: &Pose, child: &Pose) -> Pose {
    let inv = root.1.inverse();
    (
        inv * (child.0 - root.0) / root.2,
        (inv * child.1).normalize(),
        child.2 / root.2,
    )
}
