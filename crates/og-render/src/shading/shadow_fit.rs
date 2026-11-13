//! The directional light's shadow views (`docs/design.md` 2.6): cascades
//! nested round the camera, or one fixed shadow box.
//!
//! A cascade is a square of the light's space round the *anchor* (the
//! camera's position; an orthographic view's centre) that shades every
//! point within its radius of it, each cascade wider than the one before.
//! Its centre is the anchor snapped to a grid of [`GRID`] cells across the
//! map, so its view stays the same while the anchor stays in one cell: a
//! camera that turns keeps every cascade's view, and one that walks moves
//! a cascade once per cell it crosses (the wide ones seldom). A layer
//! drawn from a view holds what stood still until then
//! (`shadow_cache.rs`), where a cascade fitted to the camera's slice was
//! drawn again on every frame the camera moved. Unreal's virtual shadow
//! maps nest their clipmap levels round the camera this way, and
//! CryEngine's cached cascades recentre in steps.
//!
//! The lit shaders pick a point's cascade by its distance from the anchor
//! (lighting.wgsl `cascade_index`), which is the same in every view of
//! the frame: a probe's face or a reflection is shaded behind the camera
//! as before it.

use crate::frame::{CASCADES, CameraFrame, Frustum, frustum_planes};
use crate::scene::{Camera, Projection};
use glam::{Mat4, Vec3};

/// How far the cascade splits lean towards a logarithmic distribution (0 =
/// uniform, 1 = logarithmic).
const SPLIT_LAMBDA: f32 = 0.8;

/// The cells across a cascade's map that its centre snaps to. The map is
/// half a cell and [`EDGE`] wider each side than the radius it shades (its
/// texels a fifteenth coarser), and its view changes once per eighth of
/// that radius the camera travels.
const GRID: u32 = 16;

/// The texels a cascade's map keeps past what it shades on each side, for
/// the receiver's normal offset and the filter's taps.
const EDGE: u32 = 4;

/// The steps per octave the cascades' reach is rounded up to where the
/// scene ends before the shadow distance, so a camera walking through a
/// small scene resizes them seldom.
const REACH_STEPS: f32 = 8.0;

/// One cascade of the directional light.
pub(crate) struct Cascade {
    pub view_proj: Mat4,
    /// What the layer holds: its box. The casters a layer keeps are
    /// culled to it, whatever the camera sees.
    pub frustum: Frustum,
    /// What a frame's own casters are culled to: the box, and the camera's
    /// view of what the cascade shades swept towards the light.
    pub seen: Frustum,
    /// World-unit normal offset and the depth bias in the cascade's normalized depth.
    pub normal_offset: f32,
    pub depth_bias: f32,
    /// How far from the anchor it shades (m).
    pub split_far: f32,
    /// Where it starts to fade into the next cascade (from the anchor;
    /// equal to `split_far` without blending).
    pub blend_start: f32,
    /// The farthest from the camera a point it shades lies (m). Infinite
    /// for an orthographic view and a fixed box.
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

/// The scene's `origin` in light space past whole `step`s (f64): added to
/// a light-space position before it is snapped to `step` and taken off
/// after, the grid lies where it would at origin zero (`origin.rs`).
fn origin_rest(step: f32, rotation: Mat4, origin: Vec3) -> Vec3 {
    if origin == Vec3::ZERO {
        return Vec3::ZERO;
    }
    let o = rotation.as_dmat4().transform_point3(origin.as_dvec3());
    let t = f64::from(step);
    Vec3::new(
        o.x.rem_euclid(t) as f32,
        o.y.rem_euclid(t) as f32,
        o.z.rem_euclid(t) as f32,
    )
}

/// Snaps a shadow box's centre in light space to the texel grid through
/// the world's zero, so the texels do not swim.
fn snap_texel(c_ls: &mut Vec3, texel: f32, rotation: Mat4, origin: Vec3) {
    let rest = origin_rest(texel, rotation, origin);
    c_ls.x = ((c_ls.x + rest.x) / texel).floor() * texel - rest.x;
    c_ls.y = ((c_ls.y + rest.y) / texel).floor() * texel - rest.y;
}

/// The cell centre nearest `p_ls` on the grid of `cell` through the
/// world's zero, across the light and along it: at most half a cell from
/// `p_ls` on each axis.
fn snap_cell(p_ls: Vec3, cell: f32, rotation: Mat4, origin: Vec3) -> Vec3 {
    let rest = origin_rest(cell, rotation, origin);
    ((p_ls + rest) / cell + 0.5).floor() * cell - rest
}

/// `d` rounded up to one of [`REACH_STEPS`] steps of its octave.
fn round_up(d: f32) -> f32 {
    let step = 2f32.powi(d.log2().floor() as i32) / REACH_STEPS;
    (d / step).ceil() * step
}

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
        seen: Frustum(frustum_planes(view_proj).to_vec()),
        normal_offset: texel.max(normal_bias),
        depth_bias: texel.max(bias) / depth_range,
        split_far: 1.0e7,
        blend_start: 1.0e7,
        reach: f32::INFINITY,
        fixed_box: true,
    }]
}

/// Fits `count` (1 to [`CASCADES`]) cascades round the camera, out to the
/// shadow distance `max_distance` or the far side of `bounds` (center,
/// radius) if that is nearer: the anchor they are nested round, then the
/// cascades. Their radii depend only on the camera's intrinsics, the
/// shadow distance and `bounds`; their centres are the anchor snapped to
/// each one's grid ([`GRID`]), in whole texels, so the texels do not swim
/// and a view holds while the anchor stays in its cell. The depth range
/// reaches towards the light far enough to keep every caster inside
/// `bounds`, in whole cells. With `blend` (a fraction of each cascade's
/// ring, 0 none) the lit shaders fade from a cascade into the next, which
/// holds all of it, over that much of its outer edge.
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
) -> (Vec3, Vec<Cascade>) {
    let count = count.clamp(1, CASCADES);
    let forward = frame.forward;
    let near_depth = cam.near.max(0.0);
    // The anchor, the nearest and farthest distance shaded from it, and
    // how the splits between them are spread.
    let (anchor, near, far, lambda) = match cam.projection {
        Projection::Perspective { fov_y_degrees } => {
            // A far corner's distance from the camera over its view depth.
            let t = (fov_y_degrees.to_radians() * 0.5).tan();
            let corner = (1.0 + t * t * (1.0 + aspect * aspect)).sqrt();
            let scene = (bounds.0 - cam.position).length() + bounds.1;
            let far = round_up(scene.min(cam.far * corner)).min(max_distance);
            (cam.position, near_depth, far, SPLIT_LAMBDA)
        }
        Projection::Orthographic { height } => {
            // The view's box up to the far side of the scene: its centre,
            // and the sphere round it.
            let far_depth = ((bounds.0 - cam.position).dot(forward) + bounds.1)
                .min(cam.far)
                .min(max_distance);
            let hd = (far_depth - near_depth) * 0.5;
            let hh = height.abs() * 0.5;
            let hw = hh * aspect;
            let radius = (hh * hh + hw * hw + hd * hd).sqrt();
            let anchor = cam.position + forward * (near_depth + hd);
            (
                anchor,
                0.0,
                if hd > 5e-5 { round_up(radius) } else { 0.0 },
                0.0,
            )
        }
    };
    if far <= near + 1e-4 {
        return (anchor, Vec::new());
    }
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
    let bounds_top = rotation.transform_point3(bounds.0).z + bounds.1;
    let anchor_ls = rotation.transform_point3(anchor);
    let anchor_depth = (anchor - cam.position).dot(forward);
    let blend = blend.clamp(0.0, 1.0);
    let cell_texels = (map_size / GRID).max(1);
    let mut cascades = Vec::with_capacity(count);
    for i in 0..count {
        let shaded = splits[i + 1];
        // Half a cell wider than what it shades (the anchor is that far
        // from the snapped centre at most), and the edge.
        let spare = (cell_texels + 2 * EDGE).min(map_size / 2);
        let radius = shaded / (1.0 - spare as f32 / map_size as f32);
        let texel = 2.0 * radius / map_size as f32;
        let cell = cell_texels as f32 * texel;
        let c_ls = snap_cell(anchor_ls, cell, rotation, origin);
        let above = (bounds_top - c_ls.z).max(radius);
        let top = c_ls.z + (above / cell).ceil() * cell + texel;
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
        let planes = frustum_planes(view_proj).to_vec();
        // A frame's own casters: inside the box, and inside what the
        // camera sees of the cascade swept towards the light. A point p is
        // in the sweep when p - t * to_light lies in the view for some
        // t >= 0; every plane of the view whose inward normal faces the
        // light must hold at p itself (t only makes it harder), the others
        // can be met by moving far enough. Keeping just those planes is a
        // conservative sweep.
        let slice = slice_projection(
            cam,
            aspect,
            (anchor_depth - shaded).max(near_depth),
            (anchor_depth + shaded).min(cam.far),
        ) * frame.view;
        let mut seen = planes.clone();
        seen.extend(
            frustum_planes(slice)
                .into_iter()
                .filter(|p| p.truncate().dot(to_light) >= 0.0),
        );
        cascades.push(Cascade {
            view_proj,
            frustum: Frustum(planes),
            seen: Frustum(seen),
            normal_offset: texel,
            depth_bias: texel / depth_range,
            split_far: shaded,
            blend_start: shaded - (shaded - splits[i]) * blend,
            reach: if frame.orthographic {
                f32::INFINITY
            } else {
                shaded
            },
            fixed_box: false,
        });
    }
    (anchor, cascades)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::camera_frame;
    use glam::Vec2;

    fn camera(position: Vec3, target: Vec3) -> (Camera, crate::frame::CameraFrame) {
        let cam = Camera {
            position,
            target,
            ..Default::default()
        };
        let frame = camera_frame(&cam, ASPECT, Vec2::ZERO);
        (cam, frame)
    }

    const ASPECT: f32 = 16.0 / 9.0;

    fn fit(at: Vec3, target: Vec3, count: usize) -> (Vec3, Vec<Cascade>) {
        let (cam, frame) = camera(at, target);
        fit_cascades(
            &cam,
            &frame,
            ASPECT,
            Vec3::new(0.3, 1.0, 0.2).normalize(),
            (Vec3::ZERO, 500.0),
            150.0,
            2048,
            count,
            0.0,
            Vec3::ZERO,
        )
    }

    /// Four cascades split the shadow distance in order, the last ending at
    /// it, each one's texels coarser than the one before (finer near
    /// shadows than three over the same distance), and every count from 1
    /// to [`CASCADES`] reaches the same far end.
    #[test]
    fn four_cascades_split_the_distance_finer_near_the_camera() {
        let at = Vec3::new(0.0, 2.0, 10.0);
        let fit = |count| fit(at, Vec3::ZERO, count);
        let (anchor, four) = fit(4);
        assert_eq!(anchor, at);
        assert_eq!(four.len(), 4);
        for pair in four.windows(2) {
            assert!(pair[0].split_far < pair[1].split_far);
            assert!(pair[0].normal_offset < pair[1].normal_offset);
        }
        let three = fit(3).1;
        assert!(four[0].normal_offset < three[0].normal_offset);
        for count in 1..=CASCADES {
            let c = fit(count).1;
            assert_eq!(c.len(), count);
            assert_eq!(c.last().unwrap().split_far, 150.0);
        }
        assert_eq!(fit(CASCADES + 3).1.len(), CASCADES);
        assert!(four.iter().all(|c| !c.fixed_box && c.reach == c.split_far));
    }

    /// A cascade's map holds every point it shades, wherever the camera
    /// stands in its cell and whichever way it looks, with room for the
    /// filter round it.
    #[test]
    fn a_cascade_holds_what_it_shades() {
        let dirs = [Vec3::X, Vec3::Y, Vec3::Z, Vec3::new(0.6, -0.5, 0.62)];
        for step in 0..40 {
            let at = Vec3::new(step as f32 * 0.37, 2.0 + step as f32 * 0.11, 10.0);
            let (anchor, cascades) = fit(at, at + Vec3::X, 4);
            for c in &cascades {
                for d in dirs.iter().flat_map(|d| [*d, -*d]) {
                    let p = anchor + d.normalize() * c.split_far;
                    let ndc = c.view_proj.project_point3(p);
                    let edge = 1.0 - 2.0 * (EDGE as f32 - 0.5) / 2048.0;
                    assert!(ndc.x.abs() <= edge && ndc.y.abs() <= edge, "{ndc}");
                    assert!((0.0..=1.0).contains(&ndc.z), "{ndc}");
                    assert!(c.frustum.intersects_sphere(p, 0.0));
                }
            }
        }
    }

    /// Turning the camera keeps every cascade's view; walking moves a
    /// cascade's view once per cell, by whole texels, the wide ones
    /// seldom.
    #[test]
    fn a_cascades_view_holds_while_the_camera_turns_and_steps_as_it_walks() {
        let at = Vec3::new(3.0, 2.0, 10.0);
        let (_, base) = fit(at, at + Vec3::X, 4);
        for target in [Vec3::Z, Vec3::NEG_X, Vec3::new(0.3, -0.8, 0.5)] {
            let (_, turned) = fit(at, at + target, 4);
            for (a, b) in base.iter().zip(&turned) {
                assert_eq!(a.view_proj, b.view_proj);
                assert_eq!(a.split_far, b.split_far);
            }
        }
        // 20 m in 2 cm steps: how often each cascade's view changes.
        let mut moves = [0u32; 4];
        let mut last: Vec<Mat4> = base.iter().map(|c| c.view_proj).collect();
        for i in 1..=1000 {
            let p = at + Vec3::new(0.02, 0.0, 0.0) * i as f32;
            let (_, now) = fit(p, p + Vec3::X, 4);
            for (k, (a, b)) in last.iter_mut().zip(&now).enumerate() {
                if *a != b.view_proj {
                    moves[k] += 1;
                    // The shift across the light, in texels of the map.
                    let o = b.view_proj.project_point3(Vec3::ZERO) - a.project_point3(Vec3::ZERO);
                    for t in [o.x * 1024.0, o.y * 1024.0] {
                        assert!((t - t.round()).abs() < 0.02, "{t}");
                    }
                    *a = b.view_proj;
                }
            }
        }
        assert!(moves[0] > moves[3], "{moves:?}");
        assert!(moves[0] < 100 && moves[3] < 6, "{moves:?}");
    }

    /// The casters a frame draws itself are culled to what the camera sees
    /// swept towards the light: one behind the camera is out unless its
    /// shadow falls into the view; the layer's own box keeps both.
    #[test]
    fn a_frames_casters_are_culled_to_the_view_swept_towards_the_light() {
        let at = Vec3::new(0.0, 2.0, 0.0);
        let (cam, frame) = camera(at, at + Vec3::NEG_Z);
        let to_light = Vec3::new(0.0, 1.0, 1.0).normalize();
        let bounds = (Vec3::ZERO, 500.0);
        let (_, cascades) = fit_cascades(
            &cam,
            &frame,
            ASPECT,
            to_light,
            bounds,
            150.0,
            2048,
            3,
            0.0,
            Vec3::ZERO,
        );
        let c = cascades.last().unwrap();
        // Behind the camera and down the light from it: its shadow falls
        // farther behind. Behind and up the light: its shadow falls ahead.
        let (behind, above) = (Vec3::new(0.0, 1.0, 30.0), Vec3::new(0.0, 40.0, 30.0));
        assert!(c.frustum.intersects_sphere(behind, 1.0));
        assert!(!c.seen.intersects_sphere(behind, 1.0));
        assert!(c.seen.intersects_sphere(above, 1.0));
        assert!(c.seen.intersects_sphere(Vec3::new(0.0, 1.0, -30.0), 1.0));
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
