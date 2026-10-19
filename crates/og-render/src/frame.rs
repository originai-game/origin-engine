//! Per-frame math: camera matrices (with the TAA jitter), the directional
//! light's stable cascades, the local lights' shadow views, frustum culling,
//! and the frame uniform the lit shaders read (`shaders/frame.wgsl`).

use crate::scene::{Camera, Light, LightKind, Projection};
use glam::{Mat4, Vec2, Vec3, Vec4};

/// The most cascades of the directional light's shadow map (the uniform's array size).
pub(crate) const CASCADES: usize = 3;
/// Layers of the local shadow array: six cube faces per shadowed light.
pub(crate) const LOCAL_SHADOW_LAYERS: usize = 12;
/// How far the cascade splits lean towards a logarithmic distribution (0 =
/// uniform, 1 = logarithmic).
const SPLIT_LAMBDA: f32 = 0.8;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightGpu {
    pub position_range: [f32; 4],
    pub color_kind: [f32; 4],
    pub direction_slot: [f32; 4],
    pub cone: [f32; 4],
}

/// `struct Frame` in frame.wgsl: only vec4 and mat4 members, so the Rust and
/// WGSL layouts agree without padding.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FrameUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inv_view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 4],
    pub light_dir: [f32; 4],
    pub light_color: [f32; 4],
    pub params: [f32; 4],
    pub view_forward: [f32; 4],
    pub cascade_view_proj: [[[f32; 4]; 4]; CASCADES],
    pub cascade_bias: [[f32; 4]; CASCADES],
    /// The light clusters' screen and depth mapping (`clusters.rs`).
    pub cluster: [f32; 4],
    pub local_view_proj: [[[f32; 4]; 4]; LOCAL_SHADOW_LAYERS],
    /// Fog, the drawn sky's look and the clock (`atmosphere.rs`).
    pub atmosphere: crate::atmosphere::AtmosphereGpu,
    /// The water's caustics: (height, strength, scale, fade), then the
    /// surface's xz rectangle (`water.rs`).
    pub caustics: [[f32; 4]; 2],
}

/// The camera's matrices for one frame.
pub(crate) struct CameraFrame {
    pub view: Mat4,
    /// The projection alone (no jitter).
    pub proj: Mat4,
    /// Without jitter: culling, cascades, TAA reprojection.
    pub view_proj: Mat4,
    /// With the TAA jitter (equal to `view_proj` without TAA): rasterization.
    pub jittered_view_proj: Mat4,
    pub forward: Vec3,
    pub orthographic: bool,
}

pub(crate) fn camera_frame(cam: &Camera, aspect: f32, jitter_ndc: Vec2) -> CameraFrame {
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let (proj, orthographic) = match cam.projection {
        Projection::Perspective { fov_y_degrees } => (
            glam::camera::rh::proj::directx::perspective(
                fov_y_degrees.to_radians(),
                aspect,
                cam.near,
                cam.far,
            ),
            false,
        ),
        Projection::Orthographic { height } => {
            let h = height.abs().max(1e-6) * 0.5;
            let w = h * aspect;
            (
                glam::camera::rh::proj::directx::orthographic(-w, w, -h, h, cam.near, cam.far),
                true,
            )
        }
    };
    let view_proj = proj * view;
    let jittered_view_proj = if jitter_ndc == Vec2::ZERO {
        view_proj
    } else {
        // A clip-space translation: shifts every pixel by the same sub-pixel
        // amount for perspective and orthographic projections alike.
        Mat4::from_translation(jitter_ndc.extend(0.0)) * view_proj
    };
    CameraFrame {
        view,
        proj,
        view_proj,
        jittered_view_proj,
        forward: (cam.target - cam.position).normalize_or(Vec3::NEG_Z),
        orthographic,
    }
}

/// The TAA jitter for frame `index`: Halton(2, 3) over 8 frames, in pixels
/// (-0.5..0.5).
pub(crate) fn halton_jitter(index: u64) -> Vec2 {
    fn halton(mut i: u64, base: u64) -> f32 {
        let (mut f, mut r) = (1.0f32, 0.0f32);
        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }
        r
    }
    let i = index % 8 + 1;
    Vec2::new(halton(i, 2) - 0.5, halton(i, 3) - 0.5)
}

/// The six planes of a view-projection with depth 0..1 (Gribb and
/// Hartmann), normalized, `n · p + d >= 0` inside.
fn frustum_planes(m: Mat4) -> [Vec4; 6] {
    let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
    [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|p| p / p.truncate().length().max(1e-20))
}

/// A convex volume as planes `(n, d)` with `n · p + d >= 0` inside.
#[derive(Clone)]
pub(crate) struct Frustum(Vec<Vec4>);

impl Frustum {
    pub fn from_view_proj(m: Mat4) -> Frustum {
        Frustum(frustum_planes(m).to_vec())
    }

    /// Whether a sphere touches the volume (conservative).
    #[inline]
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.0
            .iter()
            .all(|p| p.truncate().dot(center) + p.w >= -radius)
    }
}

/// One cascade of the directional light.
pub(crate) struct Cascade {
    pub view_proj: Mat4,
    pub frustum: Frustum,
    /// World-unit normal offset and the depth bias in the cascade's normalized depth.
    pub normal_offset: f32,
    pub depth_bias: f32,
    /// The far end of the camera slice it shades (view depth).
    pub split_far: f32,
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

/// Fits `count` (1 to 3) cascades over the camera's view up to the far side of `bounds`
/// (center, radius): stable splits (they depend only on the camera's
/// intrinsics and `bounds`), each covered by a bounding sphere so its size
/// does not change as the camera turns, with its centre snapped to the shadow
/// map's texel grid in light space so the texels do not swim. The depth range
/// reaches towards the light far enough to keep every caster inside `bounds`.
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
    for i in 0..count {
        let (center, radius) = slice_sphere(cam, aspect, forward, splits[i], splits[i + 1]);
        // Quantize the radius so float noise in the camera never resizes the cascade.
        let step = 2f32.powi(radius.log2().floor() as i32 - 6);
        let radius = (radius / step).ceil() * step;
        let texel = 2.0 * radius / map_size as f32;
        let mut c_ls = rotation.transform_point3(center);
        c_ls.x = (c_ls.x / texel).floor() * texel;
        c_ls.y = (c_ls.y / texel).floor() * texel;
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
        let slice = slice_projection(cam, aspect, splits[i], splits[i + 1]) * frame.view;
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
        });
    }
    cascades
}

/// A shadowed local light: its slot in the local shadow array and one view
/// per face it renders (six for a point light, one for a spot).
pub(crate) struct LocalShadow {
    pub slot: usize,
    pub position: Vec3,
    pub range: f32,
    pub faces: Vec<(Mat4, Frustum)>,
}

const CUBE_FACES: [(Vec3, Vec3); 6] = [
    (Vec3::X, Vec3::NEG_Y),
    (Vec3::NEG_X, Vec3::NEG_Y),
    (Vec3::Y, Vec3::Z),
    (Vec3::NEG_Y, Vec3::NEG_Z),
    (Vec3::Z, Vec3::NEG_Y),
    (Vec3::NEG_Z, Vec3::NEG_Y),
];

fn spot_cone(angle_degrees: f32, penumbra: f32) -> (f32, f32) {
    let outer = angle_degrees.clamp(0.1, 89.0).to_radians();
    let inner = outer * (1.0 - penumbra.clamp(0.0, 1.0));
    (outer.cos(), inner.cos())
}

/// The near plane of a local light's shadow views.
fn local_near(range: f32) -> f32 {
    (range * 0.01).max(0.02)
}

/// The records of the lights drawn (`kept`: scene indices, from the cluster
/// build) and their shadow views: the first `MAX_SHADOWED_LIGHTS` of them
/// with `cast_shadows` get a slot.
pub(crate) fn local_lights(
    lights: &[Light],
    kept: &[usize],
    shadow_size: u32,
) -> (Vec<LightGpu>, Vec<LocalShadow>) {
    let mut gpu = Vec::with_capacity(kept.len());
    let mut shadows = Vec::new();
    for light in kept.iter().map(|&i| &lights[i]) {
        let range = light.range.max(1e-3);
        let radiance = light.color * light.intensity;
        let mut slot = -1.0;
        if light.cast_shadows && shadows.len() < crate::scene::MAX_SHADOWED_LIGHTS {
            slot = shadows.len() as f32;
            let near = local_near(range);
            let faces = match light.kind {
                LightKind::Point => {
                    let proj = glam::camera::rh::proj::directx::perspective(
                        std::f32::consts::FRAC_PI_2,
                        1.0,
                        near,
                        range,
                    );
                    CUBE_FACES
                        .iter()
                        .map(|&(dir, up)| {
                            let view = glam::camera::rh::view::look_at_mat4(
                                light.position,
                                light.position + dir,
                                up,
                            );
                            let vp = proj * view;
                            (vp, Frustum::from_view_proj(vp))
                        })
                        .collect()
                }
                LightKind::Spot {
                    direction,
                    angle_degrees,
                    ..
                } => {
                    let dir = direction.normalize_or(Vec3::NEG_Y);
                    let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
                    let fov = (angle_degrees.clamp(0.1, 89.0) * 2.0 + 4.0).min(176.0);
                    let proj = glam::camera::rh::proj::directx::perspective(
                        fov.to_radians(),
                        1.0,
                        near,
                        range,
                    );
                    let view = glam::camera::rh::view::look_at_mat4(
                        light.position,
                        light.position + dir,
                        up,
                    );
                    let vp = proj * view;
                    vec![(vp, Frustum::from_view_proj(vp))]
                }
            };
            shadows.push(LocalShadow {
                slot: shadows.len(),
                position: light.position,
                range,
                faces,
            });
        }
        let (kind, direction, cone) = match light.kind {
            LightKind::Point => (0.0, Vec3::NEG_Y, [-1.0, 1.0]),
            LightKind::Spot {
                direction,
                angle_degrees,
                penumbra,
            } => {
                let (cos_outer, cos_inner) = spot_cone(angle_degrees, penumbra);
                (
                    1.0,
                    direction.normalize_or(Vec3::NEG_Y),
                    [cos_outer, 1.0 / (cos_inner - cos_outer).max(1e-4)],
                )
            }
        };
        // Normal offset per metre of distance: ~1.5 texels of a 90° face; the
        // depth bias in distance / range.
        let normal_offset = 3.0 / shadow_size as f32;
        gpu.push(LightGpu {
            position_range: light.position.extend(range).to_array(),
            color_kind: radiance.extend(kind).to_array(),
            direction_slot: direction.extend(slot).to_array(),
            cone: [cone[0], cone[1], normal_offset, 0.002],
        });
    }
    (gpu, shadows)
}
