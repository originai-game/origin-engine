//! Reverse-Z camera projections (`near -> 1.0`, `far -> 0.0`) and matching
//! depth-test constants for WebGPU's `[0, 1]` clip range.
//!
//! Mapping near to `1.0` and far to `0.0` pairs the hyperbolic `1 / z`
//! perspective curve with IEEE-754 float32's dense exponent distribution near
//! zero, eliminating far-plane z-fighting across large open worlds without
//! sacrificing near-plane precision.

use crate::scene::Projection;
use glam::{Mat4, Vec4};

/// Clear value for Reverse-Z camera depth attachments (far plane).
pub(crate) const DEPTH_CLEAR: f32 = 0.0;

/// Right-handed Reverse-Z perspective projection (`near -> 1`, `far -> 0`)
/// using deterministic [`og_core::math::sin_cos`].
pub(crate) fn perspective_focal(focal_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let denom = far - near;
    let a = near / denom;
    let b = (near * far) / denom;
    Mat4::from_cols(
        Vec4::new(focal_y / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, focal_y, 0.0, 0.0),
        Vec4::new(0.0, 0.0, a, -1.0),
        Vec4::new(0.0, 0.0, b, 0.0),
    )
}

/// Right-handed Reverse-Z orthographic projection (`near -> 1`, `far -> 0`).
pub(crate) fn orthographic(half_width: f32, half_height: f32, near: f32, far: f32) -> Mat4 {
    let r = 1.0 / (far - near);
    Mat4::from_cols(
        Vec4::new(1.0 / half_width, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0 / half_height, 0.0, 0.0),
        Vec4::new(0.0, 0.0, r, 0.0),
        Vec4::new(0.0, 0.0, far * r, 1.0),
    )
}

/// Right-handed Reverse-Z off-axis perspective projection (`near -> 1`,
/// `far -> 0`), for mirror / portal cameras.
pub(crate) fn off_axis(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
    let denom = f - n;
    Mat4::from_cols(
        Vec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
        Vec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
        Vec4::new((r + l) / (r - l), (t + b) / (t - b), n / denom, -1.0),
        Vec4::new(0.0, 0.0, n * f / denom, 0.0),
    )
}

/// Reverse-Z projection matrix for a [`Projection`].
pub(crate) fn matrix(projection: &Projection, aspect: f32, near: f32, far: f32) -> Mat4 {
    match *projection {
        Projection::Perspective { fov_y_degrees } => {
            let (s, c) = og_core::math::sin_cos(fov_y_degrees.to_radians() * 0.5);
            perspective_focal(c / s, aspect, near, far)
        }
        Projection::PerspectiveShifted {
            fov_y_degrees,
            shift,
        } => {
            let (s, c) = og_core::math::sin_cos(fov_y_degrees.to_radians() * 0.5);
            let mut m = perspective_focal(c / s, aspect, near, far);
            m.z_axis.x = 2.0 * shift.x;
            m.z_axis.y = 2.0 * shift.y;
            m
        }
        Projection::Orthographic { height } => {
            let h = height.abs().max(1e-6) * 0.5;
            orthographic(h * aspect, h, near, far)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec2, Vec3};

    #[test]
    fn perspective_maps_near_to_one_and_far_to_zero() {
        let p = matrix(
            &Projection::Perspective {
                fov_y_degrees: 60.0,
            },
            16.0 / 9.0,
            0.1,
            1000.0,
        );
        let near = p * Vec4::new(0.0, 0.0, -0.1, 1.0);
        let far = p * Vec4::new(0.0, 0.0, -1000.0, 1.0);
        let mid = p * Vec4::new(0.0, 0.0, -10.0, 1.0);
        assert!((near.z / near.w - 1.0).abs() < 1e-6);
        assert!((far.z / far.w - 0.0).abs() < 1e-6);
        assert!(mid.z / mid.w > 0.0 && mid.z / mid.w < 1.0);
    }

    #[test]
    fn orthographic_maps_near_to_one_and_far_to_zero() {
        let p = matrix(&Projection::Orthographic { height: 20.0 }, 1.5, 0.5, 500.0);
        let near = p * Vec4::new(0.0, 0.0, -0.5, 1.0);
        let far = p * Vec4::new(0.0, 0.0, -500.0, 1.0);
        assert!((near.z / near.w - 1.0).abs() < 1e-6);
        assert!((far.z / far.w - 0.0).abs() < 1e-6);
    }

    #[test]
    fn view_rays_point_forward_for_perspective_and_orthographic() {
        let view = glam::camera::rh::view::look_at_mat4(
            Vec3::new(0.0, 2.0, 5.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::Y,
        );
        for proj in [
            Projection::Perspective {
                fov_y_degrees: 50.0,
            },
            Projection::Orthographic { height: 10.0 },
        ] {
            let vp = matrix(&proj, 1.0, 0.1, 100.0) * view;
            let rays = crate::frame::view_rays(vp);
            let center = Vec3::new(rays[2][0], rays[2][1], rays[2][2]).normalize();
            assert!(
                center.dot(Vec3::NEG_Z) > 0.999,
                "expected forward -Z, got {center:?} for {proj:?}"
            );
        }
    }

    #[test]
    fn shifted_perspective_preserves_depth_mapping() {
        let p = matrix(
            &Projection::PerspectiveShifted {
                fov_y_degrees: 50.0,
                shift: Vec2::new(0.1, -0.05),
            },
            1.0,
            0.2,
            200.0,
        );
        let near = p * Vec4::new(0.0, 0.0, -0.2, 1.0);
        let far = p * Vec4::new(0.0, 0.0, -200.0, 1.0);
        assert!((near.z / near.w - 1.0).abs() < 1e-6);
        assert!((far.z / far.w - 0.0).abs() < 1e-6);
    }
}
