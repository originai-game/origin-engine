//! The directional light's shadow views (`docs/design.md` 2.6): its
//! stable cascades over the camera's view, or one fixed shadow box.

use crate::frame::{CASCADES, CameraFrame, Frustum, frustum_planes};
use crate::scene::{Camera, Projection};
use glam::{Mat4, Vec3};

/// How far the cascade splits lean towards a logarithmic distribution (0 =
/// uniform, 1 = logarithmic).
const SPLIT_LAMBDA: f32 = 0.8;

/// One cascade of the directional light.
pub(crate) struct Cascade {
    pub view_proj: Mat4,
    pub frustum: Frustum,
    /// World-unit normal offset and the depth bias in the cascade's normalized depth.
    pub normal_offset: f32,
    pub depth_bias: f32,
    /// The far end of the camera slice it shades (view depth).
    pub split_far: f32,
    /// Where it starts to fade into the next cascade (view depth; equal to
    /// `split_far` without blending).
    pub blend_start: f32,
    /// The farthest from the camera a point it shades lies (m): the
    /// slice's far corners. Infinite for a fixed box.
    pub reach: f32,
    /// A fixed shadow box ([`fit_box`]): three.js's shadow camera, whose
    /// `depth_bias` is the game's (`DirectionalLight::shadow_bias`), not a
    /// texel, so every receiver is compared towards the light by it, as
    /// three.js compares, and none past its own depth (lighting.wgsl
    /// `shadow_closed`, a quarter of a cascade's one-texel bias).
    pub fixed_box: bool,
}

/// The camera's projection with its near and far planes moved to `n` and `f`.
fn slice_projection(cam: &Camera, aspect: f32, n: f32, f: f32) -> Mat4 {
    match cam.projection {
        Projection::Perspective { fov_y_degrees } => glam::camera::rh::proj::directx::perspective(
            fov_y_degrees.to_radians(),
            aspect,
            n.max(1e-4),
            f,
        ),
        Projection::Orthographic { height } => {
            let h = height.abs().max(1e-6) * 0.5;
            let w = h * aspect;
            glam::camera::rh::proj::directx::orthographic(-w, w, -h, h, n, f)
        }
    }
}

/// The smallest sphere around the camera frustum between view depths `n` and `f`.
fn slice_sphere(cam: &Camera, aspect: f32, forward: Vec3, n: f32, f: f32) -> (Vec3, f32) {
    match cam.projection {
        Projection::Perspective { fov_y_degrees } => {
            let t = (fov_y_degrees.to_radians() * 0.5).tan();
            // Squared ratio of a corner's distance from the axis to its depth.
            let k2 = t * t * (1.0 + aspect * aspect);
            let c = (n + f) * 0.5 * (1.0 + k2);
            let (c, r) = if c >= f {
                (f, f * k2.sqrt())
            } else {
                (c, ((f - c) * (f - c) + f * f * k2).sqrt())
            };
            (cam.position + forward * c, r)
        }
        Projection::Orthographic { height } => {
            let hh = height.abs() * 0.5;
            let hw = hh * aspect;
            let hd = (f - n) * 0.5;
            (
                cam.position + forward * (n + hd),
                (hh * hh + hw * hw + hd * hd).sqrt(),
            )
        }
    }
}

/// Snaps a cascade's centre in light space to the texel grid through the
/// world's zero: the scene's `origin` brought into light space (f64, whole
/// texels dropped) is added before the snap and taken off after, so the
/// texels lie where they would at origin zero (`origin.rs`).
fn snap_texel(c_ls: &mut Vec3, texel: f32, rotation: Mat4, origin: Vec3) {
    let (rx, ry) = if origin == Vec3::ZERO {
        (0.0, 0.0)
    } else {
        let o = rotation.as_dmat4().transform_point3(origin.as_dvec3());
        let t = f64::from(texel);
        (o.x.rem_euclid(t) as f32, o.y.rem_euclid(t) as f32)
    };
    c_ls.x = ((c_ls.x + rx) / texel).floor() * texel - rx;
    c_ls.y = ((c_ls.y + ry) / texel).floor() * texel - ry;
}

/// Fits `count` (1 to 3) cascades over the camera's view up to the far side of `bounds`
/// (center, radius): stable splits (they depend only on the camera's
/// intrinsics and `bounds`), each covered by a bounding sphere so its size
/// does not change as the camera turns, with its centre snapped to the shadow
/// map's texel grid in light space so the texels do not swim. The depth range
/// reaches towards the light far enough to keep every caster inside `bounds`.
/// With `blend` (a fraction of each cascade's depth, 0 none) every cascade
/// after the first also covers that much of the slice before it, where the
/// lit shaders fade from that cascade into this one.
#[allow(clippy::too_many_arguments)]
/// One shadow map over a fixed box (DirectionalLight::shadow_box): `half`
/// metres either side across the light, centred at `focus`, its depth from
/// the scene's bounds on the light's side to the box's far side; snapped to
/// whole texels so it does not shimmer as the camera moves. Its slice reaches
/// every view depth: what lies outside the box reads as lit.
pub(crate) fn fit_box(
    to_light: Vec3,
    bounds: (Vec3, f32),
    focus: Vec3,
    half: f32,
    map_size: u32,
    bias: f32,
    normal_bias: f32,
    origin: Vec3,
) -> Vec<Cascade> {
    let up = if to_light.y.abs() > 0.99 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let rotation = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, -to_light, up);
    let bounds_ls = rotation.transform_point3(bounds.0);
    let radius = half.max(0.5);
    let texel = 2.0 * radius / map_size as f32;
    let mut c_ls = rotation.transform_point3(focus);
    snap_texel(&mut c_ls, texel, rotation, origin);
    let top = (bounds_ls.z + bounds.1).max(c_ls.z + radius) + texel;
    let bottom = (bounds_ls.z - bounds.1).min(c_ls.z - radius);
    let depth_range = top - bottom;
    let view = Mat4::from_translation(Vec3::new(-c_ls.x, -c_ls.y, -top)) * rotation;
    let proj = glam::camera::rh::proj::directx::orthographic(
        -radius,
        radius,
        -radius,
        radius,
        0.0,
        depth_range,
    );
    let view_proj = proj * view;
    vec![Cascade {
        view_proj,
        frustum: Frustum(frustum_planes(view_proj).to_vec()),
        normal_offset: texel.max(normal_bias),
        depth_bias: texel.max(bias) / depth_range,
        split_far: 1.0e7,
        blend_start: 1.0e7,
        reach: f32::INFINITY,
        fixed_box: true,
    }]
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn fit_cascades(
    cam: &Camera,
    frame: &CameraFrame,
    aspect: f32,
    to_light: Vec3,
    bounds: (Vec3, f32),
    max_distance: f32,
    map_size: u32,
    count: usize,
    blend: f32,
    origin: Vec3,
) -> Vec<Cascade> {
    let count = count.clamp(1, CASCADES);
    let forward = frame.forward;
    let far_side = (bounds.0 - cam.position).dot(forward) + bounds.1;
    let near = cam.near.max(0.0);
    let far = far_side.min(cam.far).min(max_distance);
    if far <= near + 1e-4 {
        return Vec::new();
    }
    let lambda = if frame.orthographic {
        0.0
    } else {
        SPLIT_LAMBDA
    };
    let log_near = near.max(far * 1e-4);
    let mut splits = [near; CASCADES + 1];
    for (i, split) in splits.iter_mut().enumerate().take(count + 1).skip(1) {
        let t = i as f32 / count as f32;
        let log = log_near * (far / log_near).powf(t);
        let uniform = near + (far - near) * t;
        *split = lambda * log + (1.0 - lambda) * uniform;
    }
    splits[count] = far;

    let up = if to_light.y.abs() > 0.99 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    // World to light space: the light looks along -to_light.
    let rotation = glam::camera::rh::view::look_at_mat4(Vec3::ZERO, -to_light, up);
    let bounds_ls = rotation.transform_point3(bounds.0);
    let mut cascades = Vec::with_capacity(count);
    let blend = blend.clamp(0.0, 1.0);
    // A far corner's distance from the camera over its view depth.
    let corner = match cam.projection {
        Projection::Perspective { fov_y_degrees } => {
            let t = (fov_y_degrees.to_radians() * 0.5).tan();
            (1.0 + t * t * (1.0 + aspect * aspect)).sqrt()
        }
        Projection::Orthographic { .. } => f32::INFINITY,
    };
    for i in 0..count {
        // The slice this cascade shades, reaching back over the blend band
        // of the one before it.
        let start = if i > 0 && blend > 0.0 {
            splits[i] - (splits[i] - splits[i - 1]) * blend
        } else {
            splits[i]
        };
        let (center, radius) = slice_sphere(cam, aspect, forward, start, splits[i + 1]);
        // Quantize the radius so float noise in the camera never resizes the cascade.
        let step = 2f32.powi(radius.log2().floor() as i32 - 6);
        let radius = (radius / step).ceil() * step;
        let texel = 2.0 * radius / map_size as f32;
        let mut c_ls = rotation.transform_point3(center);
        snap_texel(&mut c_ls, texel, rotation, origin);
        let top = (bounds_ls.z + bounds.1).max(c_ls.z + radius) + texel;
        let bottom = c_ls.z - radius;
        let depth_range = top - bottom;
        let view = Mat4::from_translation(Vec3::new(-c_ls.x, -c_ls.y, -top)) * rotation;
        let proj = glam::camera::rh::proj::directx::orthographic(
            -radius,
            radius,
            -radius,
            radius,
            0.0,
            depth_range,
        );
        let view_proj = proj * view;
        // Casters: inside the cascade's box, and inside the camera slice swept
        // towards the light. A point p is in the sweep when p - t * to_light
        // lies in the slice for some t >= 0; every slice plane whose inward
        // normal faces the light must hold at p itself (t only makes it
        // harder), the others can be met by moving far enough. Keeping just
        // those planes is a conservative sweep.
        let mut planes = frustum_planes(view_proj).to_vec();
        let slice = slice_projection(cam, aspect, start, splits[i + 1]) * frame.view;
        planes.extend(
            frustum_planes(slice)
                .into_iter()
                .filter(|p| p.truncate().dot(to_light) >= 0.0),
        );
        cascades.push(Cascade {
            view_proj,
            frustum: Frustum(planes),
            normal_offset: texel,
            depth_bias: texel / depth_range,
            split_far: splits[i + 1],
            blend_start: splits[i + 1] - (splits[i + 1] - splits[i]) * blend,
            reach: splits[i + 1] * corner,
            fixed_box: false,
        });
    }
    cascades
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::camera_frame;
    use glam::Vec2;

    /// Four cascades split the shadow distance in order, the last ending at
    /// it, each one's texels coarser than the one before (finer near
    /// shadows than three over the same distance), and every count from 1
    /// to [`CASCADES`] reaches the same far end.
    #[test]
    fn four_cascades_split_the_distance_finer_near_the_camera() {
        let cam = Camera {
            position: Vec3::new(0.0, 2.0, 10.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        };
        let aspect = 16.0 / 9.0;
        let frame = camera_frame(&cam, aspect, Vec2::ZERO);
        let to_light = Vec3::new(0.3, 1.0, 0.2).normalize();
        let fit = |count| {
            fit_cascades(
                &cam,
                &frame,
                aspect,
                to_light,
                (Vec3::ZERO, 500.0),
                150.0,
                2048,
                count,
                0.0,
                Vec3::ZERO,
            )
        };
        let four = fit(4);
        assert_eq!(four.len(), 4);
        for pair in four.windows(2) {
            assert!(pair[0].split_far < pair[1].split_far);
            assert!(pair[0].normal_offset < pair[1].normal_offset);
        }
        let three = fit(3);
        assert!(four[0].normal_offset < three[0].normal_offset);
        for count in 1..=CASCADES {
            let c = fit(count);
            assert_eq!(c.len(), count);
            assert_eq!(c.last().unwrap().split_far, 150.0);
        }
        assert_eq!(fit(CASCADES + 3).len(), CASCADES);
        assert!(four.iter().all(|c| !c.fixed_box));
        // A cascade's reach is its slice's far corners' distance from the
        // camera (the corner of the screen, at the split's view depth).
        let unproject = frame.view_proj.inverse();
        let ray = (unproject.project_point3(Vec3::new(1.0, 1.0, 0.5)) - cam.position).normalize();
        for c in &four {
            let corner = c.split_far / ray.dot(frame.forward);
            assert!(
                (c.reach - corner).abs() < 1e-3 * corner,
                "{} {corner}",
                c.reach
            );
        }
    }

    /// A fixed shadow box is one cascade whose depth bias is the game's
    /// (three.js's shadow.bias over its depth range), not a texel: it is
    /// marked, so lighting.wgsl compares every receiver towards the light by
    /// it instead of a closed one a quarter of it past its own depth.
    #[test]
    fn a_fixed_box_keeps_the_games_bias_and_is_marked() {
        let to_light = Vec3::new(-0.2, 0.95, 0.24).normalize();
        let fit = |bias| {
            fit_box(
                to_light,
                (Vec3::ZERO, 300.0),
                Vec3::new(6.0, 0.0, 74.0),
                22.0,
                4096,
                bias,
                0.05,
                Vec3::ZERO,
            )
        };
        let (b, texel_only) = (fit(0.18), fit(0.0));
        assert_eq!(b.len(), 1);
        assert!(b[0].fixed_box);
        // 0.18 m against the 44 m / 4096 texel the bias falls back to.
        let texel = 2.0 * 22.0 / 4096.0;
        let ratio = b[0].depth_bias / texel_only[0].depth_bias;
        assert!((ratio - 0.18 / texel).abs() < 0.01 * ratio, "{ratio}");
    }
}
