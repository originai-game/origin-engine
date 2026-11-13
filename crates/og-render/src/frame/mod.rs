//! Per-frame math: camera matrices (with the TAA jitter), the local
//! lights' shadow views, frustum culling,
//! and the frame uniform the lit shaders read (`shaders/frame.wgsl`).

pub(crate) mod auto_quality;
pub(crate) mod fx;
pub mod graph;
pub(crate) mod origin;
pub(crate) mod quality;
pub(crate) mod timing;
pub(crate) mod views;

use crate::scene::{Camera, Light, LightKind, Projection};
use crate::shadows::{LOCAL_SHADOW_LAYERS, PARABOLOID_SCALE, PointShadows};
use glam::{Mat4, Vec2, Vec3, Vec4};

/// The most cascades of the directional light's shadow map (the uniform's array size).
pub(crate) const CASCADES: usize = 4;

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
    /// xyz: the point the cascades are nested round (`shadow_fit.rs`), w
    /// unused.
    pub cascade_anchor: [f32; 4],
    /// The light clusters' screen and depth mapping (`clusters.rs`).
    pub cluster: [f32; 4],
    pub local_view_proj: [[[f32; 4]; 4]; LOCAL_SHADOW_LAYERS],
    /// Fog, the drawn sky's look and the clock (`atmosphere.rs`).
    pub atmosphere: crate::atmosphere::AtmosphereGpu,
    /// The water's caustics: (height, strength, scale, fade), then the
    /// surface's xz rectangle, then the frame's origin in a lake's ripple
    /// tile (a sea: its curvature grid, 1 in the next row's w); then the
    /// shore a sea's surf wets (its level, the band's height, how wet) and
    /// its seabed's xz rectangle (`water.rs`).
    pub caustics: [[f32; 4]; 6],
    /// x: the sun's shadow softness (0: hard PCF), y: 1 when point lights'
    /// shadows are paraboloid hemispheres, z, w: unused.
    pub shadow: [f32; 4],
    /// Stylisation (`stylise.rs`, `Retro3d`): xy: the screen grid vertices
    /// snap to, in cells across and down (0: none), z: affine texturing
    /// (0 perspective-correct, 1 affine), w: depth bits (0: full precision).
    pub stylise: [f32; 4],
    /// `MaterialStyle`: rgb: the linear colour shadowed light moves
    /// towards, w: how far (0: off).
    pub shadow_tint: [f32; 4],
    /// `MaterialStyle`: x: the distance at which outlines are their width
    /// (0: constant), y: the shadow darkness (0: off), zw: unused.
    pub outline_style: [f32; 4],
    /// `MaterialStyle`: rgb: the outline's colour, w: its width in pixels
    /// (0: none), for the surfaces no material carries it to (the terrain).
    pub style_line: [f32; 4],
    /// The reflection probe of the frame group a draw binds (`probes.rs`):
    /// the box's min corner (w: 1 with a probe), its max corner (w: the
    /// probe's intensity), the capture point (w: the blend distance).
    /// Zero in the frame's own group.
    pub probe: [[f32; 4]; 3],
    /// The camera's right and up in the world (w unused): billboards
    /// (`RenderScene::sprite_facing`) face the camera of the group they
    /// are drawn with, a view's or a probe face's included.
    pub view_right: [f32; 4],
    pub view_up: [f32; 4],
    /// The hemisphere light (`RenderScene::hemisphere`): rgb sky colour,
    /// w intensity over pi over the IBL intensity (the shaders add it to
    /// the irradiance the IBL scale multiplies); rgb ground colour.
    pub hemi_sky: [f32; 4],
    pub hemi_ground: [f32; 4],
    /// `MaterialStyle`'s surface (`MaterialStyle::surface_uniform`): x the
    /// surface (0 none, 1 clay, 2 paper, 3 plastic), y its strength, z its
    /// scale in metres, w unused.
    pub material_surface: [f32; 4],
    /// xyz: where the zero of the frame's coordinates is in the world
    /// (`RenderScene::origin`): the shaders that read a world position for
    /// its look (layer uvs, normal maps, caustics, wind) add it back.
    pub origin: [f32; 4],
    /// The grounded dome the environment is drawn on and reflected from
    /// (`GroundProjection::gpu`; zero: at infinity).
    pub env_dome: [[f32; 4]; 2],
    /// Current and previous drawn foliage winds, packed as seven vec4s each.
    pub foliage_wind: [crate::foliage::FoliageWind; 2],
    /// `MaterialStyle`'s toon model: x the band edges' softness in pixels
    /// (`toon_softness`, 0: hard steps), y how smooth a toon material's
    /// ambient light stays (`toon_ambient`), zw unused.
    pub toon: [f32; 4],
    /// Screen-space reflections (`ssr.rs`): x 1 when the frame group's
    /// `ssr_tex` holds this camera's reflections (the main camera's; 0 for
    /// views, probe faces and planar reflections).
    pub ssr: [f32; 4],
    /// x: the frames the renderer drew before this one (its frame index as
    /// u32, which counts on while the clock is paused), which a material's
    /// shader parameters compare their stamp with (frame.wgsl
    /// `params_moved`); yzw unused.
    pub drawn: [u32; 4],
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
        forward: (cam.target - cam.position).normalize_or(og_core::facing::CAMERA_FORWARD),
        orthographic,
    }
}

impl CameraFrame {
    /// The camera's right and up in the world (`FrameUniform::view_right`,
    /// `view_up`): the view matrix's first two rows.
    pub fn axes(&self) -> ([f32; 4], [f32; 4]) {
        let t = self.view.transpose();
        (
            t.x_axis.truncate().extend(0.0).to_array(),
            t.y_axis.truncate().extend(0.0).to_array(),
        )
    }
}

/// How many frames the jitter cycles over for a scene drawn at `scene`
/// and resolved by TAA to `out`: 8 at one scene pixel per output pixel,
/// and 8 per output pixel a scene pixel covers below that (TAAU: each
/// output pixel is sampled about as often as without upscaling), at most
/// 32 (render scale 0.5).
pub(crate) fn jitter_phases(scene: (u32, u32), out: (u32, u32)) -> u64 {
    let area =
        f64::from(out.0) * f64::from(out.1) / (f64::from(scene.0) * f64::from(scene.1)).max(1.0);
    ((8.0 * area).round() as u64).clamp(8, 32)
}

/// The TAA jitter for frame `index`: Halton(2, 3) over `phases` frames
/// ([`jitter_phases`]), in pixels (-0.5..0.5).
pub(crate) fn halton_jitter(index: u64, phases: u64) -> Vec2 {
    fn halton(mut i: u64, base: u64) -> f32 {
        let (mut f, mut r) = (1.0f32, 0.0f32);
        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }
        r
    }
    let i = index % phases.max(1) + 1;
    Vec2::new(halton(i, 2) - 0.5, halton(i, 3) - 0.5)
}

/// SMAA T2x's image displacement and area layer, in pixels (y down).
/// Reference SMAA.cpp getJitter/JitteredMatrix, not the opposite-signed
/// sample positions in the HLSL table. MIT, third_party/smaa/LICENSE.txt.
pub(crate) fn smaa_sample(index: u64) -> (Vec2, f32) {
    if index.is_multiple_of(2) {
        (Vec2::splat(-0.25), 1.0)
    } else {
        (Vec2::splat(0.25), 2.0)
    }
}

/// The six planes of a view-projection with depth 0..1 (Gribb and
/// Hartmann), normalized, `n · p + d >= 0` inside.
pub(crate) fn frustum_planes(m: Mat4) -> [Vec4; 6] {
    let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
    [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|p| p / p.truncate().length().max(1e-20))
}

/// A convex volume as planes `(n, d)` with `n · p + d >= 0` inside.
#[derive(Clone)]
pub(crate) struct Frustum(pub(crate) Vec<Vec4>);

impl Frustum {
    pub fn from_view_proj(m: Mat4) -> Frustum {
        Frustum(frustum_planes(m).to_vec())
    }

    /// Its planes (inside: `dot(xyz, p) + w >= 0`).
    pub fn planes(&self) -> &[Vec4] {
        &self.0
    }

    /// Whether a sphere touches the volume (conservative).
    #[inline]
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        self.0
            .iter()
            .all(|p| p.truncate().dot(center) + p.w >= -radius)
    }

    /// Whether a box (min, max corner) touches the volume (conservative:
    /// each plane against the box's corner farthest inside it).
    pub fn intersects_box(&self, lo: Vec3, hi: Vec3) -> bool {
        self.0.iter().all(|p| {
            let n = p.truncate();
            n.dot(Vec3::select(n.cmpge(Vec3::ZERO), hi, lo)) + p.w >= 0.0
        })
    }

    /// The CPU cull's tests: every marked `inside[k]` becomes whether
    /// `spheres[k]` touches the volume; an unmarked one stays false and
    /// its sphere is not read. Each flag is its own sphere's, so the
    /// result is the same on any number of threads. On the lanes
    /// (og-lanes' `cull` site, [`CULL_GRAIN`] bounds a task) from
    /// [`CULL_LANES_FROM`] bounds, serially below.
    pub fn test_spheres(&self, inside: &mut [bool], spheres: &[(Vec3, f32)]) {
        let grain = if inside.len() >= CULL_LANES_FROM {
            CULL_GRAIN
        } else {
            // One grain of everything: og-lanes runs it on this thread.
            inside.len()
        };
        og_lanes::for_each_indexed(og_lanes::Site::Cull, inside, grain, |k, v| {
            if *v {
                let (c, r) = spheres[k];
                *v = self.intersects_sphere(c, r);
            }
        });
    }
}

/// Bounds a lane task tests in [`Frustum::test_spheres`]: a test is about
/// 5 ns, and with the split this small a pool thread that wakes late
/// still finds work (coarser tasks measured slower at every count,
/// the former test sample
pub(crate) const CULL_GRAIN: usize = 1024;

/// Bounds from which [`Frustum::test_spheres`] splits across the lanes.
/// Waking the pool and joining it costs 50 to 600 µs a call on an idle
/// machine, milliseconds under load, against 5 ns a bound serially: the
/// split lost at every count to 65,536 bounds idle and at every count
/// loaded, and first won at every thread count to eight from 131,072
/// bounds idle (the former test sample
pub(crate) const CULL_LANES_FROM: usize = 131_072;

/// A shadowed local light: its first layer in the local shadow array and
/// one view per layer it renders (six cube faces or two paraboloid
/// hemispheres for a point light, one for a spot). A paraboloid layer's
/// matrix is unused (the projection is not linear); its volume culls.
pub(crate) struct LocalShadow {
    /// Its light's index in the scene's lights, and its record's in the
    /// frame's light records.
    pub light: usize,
    pub record: usize,
    pub layer: usize,
    pub position: Vec3,
    pub range: f32,
    pub paraboloid: bool,
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

/// What one paraboloid hemisphere sees: the light's box of `range` cut by
/// the plane through the light across `axis` (-y: below, +y: above), less
/// the margin past 90° the map holds.
fn hemisphere(position: Vec3, range: f32, axis: Vec3) -> Frustum {
    let margin = range * 0.1;
    let mut planes = vec![axis.extend(-axis.dot(position) + margin)];
    for n in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z, -axis] {
        planes.push(n.extend(-n.dot(position) + range));
    }
    Frustum(planes)
}

pub(crate) fn spot_cone(angle_degrees: f32, penumbra: f32) -> (f32, f32) {
    let outer = angle_degrees.clamp(0.1, 89.0).to_radians();
    let inner = outer * (1.0 - penumbra.clamp(0.0, 1.0));
    (outer.cos(), inner.cos())
}

/// The most shadowed lights a frame draws screen-space contact shadows for
/// (contact.wgsl `cs_contact`): one each in the AO texture's g, b and a.
pub(crate) const CONTACT_LIGHTS: usize = 3;

/// The shadowed point and spot lights whose contact shadows a frame draws:
/// the [`CONTACT_LIGHTS`] most important at `eye` (radiance at the camera,
/// as the shadows are kept; ties in scene order). A rect light keeps its
/// map alone (its shadow is `rect_shadow`, not `local_shadow`).
pub(crate) fn contact_lights<'a>(
    shadows: &'a [LocalShadow],
    lights: &[Light],
    eye: Vec3,
) -> Vec<&'a LocalShadow> {
    let mut picked: Vec<&LocalShadow> = shadows
        .iter()
        .filter(|s| !matches!(lights[s.light].kind, LightKind::Rect { .. }))
        .collect();
    let score = |s: &LocalShadow| crate::clusters::importance(&lights[s.light], eye);
    og_core::sort::sort_by(&mut picked, |a, b| {
        score(b).total_cmp(&score(a)).then(a.light.cmp(&b.light))
    });
    picked.truncate(CONTACT_LIGHTS);
    picked
}

/// The near plane of a local light's shadow views.
fn local_near(range: f32) -> f32 {
    (range * 0.01).max(0.02)
}

/// The local shadows a frame draws: at most `max_shadowed` lights,
/// point lights' shadows stored as `point` says (cube or paraboloid).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LocalShadowBudget {
    pub max_shadowed: usize,
    /// Rect lights' shadows ([`crate::RenderQuality::rect_shadows`]):
    /// false leaves every rect light unshadowed.
    pub rect_shadows: bool,
    pub point: PointShadows,
    pub shadow_size: u32,
}

/// The records of the lights drawn (`kept`: scene indices, from the cluster
/// build) and their shadow views. Of the kept lights with `cast_shadows`,
/// the `max_shadowed` most important at `eye` (radiance at the camera, as
/// the clusters drop lights; ties in scene order) get layers, handed out in
/// scene order; the rest are counted in the third answer.
pub(crate) fn local_lights(
    lights: &[Light],
    kept: &[usize],
    eye: Vec3,
    budget: LocalShadowBudget,
) -> (Vec<LightGpu>, Vec<LocalShadow>, u32) {
    let shadowed: Vec<usize> = (0..kept.len())
        .filter(|&k| {
            let l = &lights[kept[k]];
            l.cast_shadows && (budget.rect_shadows || !matches!(l.kind, LightKind::Rect { .. }))
        })
        .collect();
    let mut chosen = vec![false; kept.len()];
    if shadowed.len() > budget.max_shadowed {
        let score = |k: usize| crate::clusters::importance(&lights[kept[k]], eye);
        let mut by_score = shadowed.clone();
        og_core::sort::sort_by(&mut by_score, |&a, &b| {
            score(b).total_cmp(&score(a)).then(a.cmp(&b))
        });
        for &k in &by_score[..budget.max_shadowed] {
            chosen[k] = true;
        }
    } else {
        for &k in &shadowed {
            chosen[k] = true;
        }
    }
    let dropped = shadowed.len().saturating_sub(budget.max_shadowed) as u32;
    let paraboloid = budget.point == PointShadows::Paraboloid;
    let shadow_size = budget.shadow_size;
    let mut gpu = Vec::with_capacity(kept.len());
    let mut shadows: Vec<LocalShadow> = Vec::new();
    let mut next_layer = 0usize;
    for (k, light) in kept.iter().map(|&i| &lights[i]).enumerate() {
        let range = light.range.max(1e-3);
        let radiance = light.color * light.intensity;
        let mut slot = -1.0;
        let mut point_paraboloid = false;
        // tan of half the view's field: a texel at its centre spans
        // 2 tan / size radians (1 for a cube face).
        let mut half_tan = 1.0f32;
        if chosen[k] {
            slot = next_layer as f32;
            let near = local_near(range);
            let faces: Vec<(Mat4, Frustum)> = match light.kind {
                LightKind::Point if paraboloid => {
                    point_paraboloid = true;
                    [Vec3::NEG_Y, Vec3::Y]
                        .into_iter()
                        .map(|axis| (Mat4::IDENTITY, hemisphere(light.position, range, axis)))
                        .collect()
                }
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
                    half_tan = (0.5 * fov).to_radians().tan();
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
                LightKind::Rect {
                    half_width,
                    half_height,
                    two_sided,
                } => {
                    // From the centre along the normal (and its back when
                    // two-sided), a RECT_SHADOW_FOV view (area.rs).
                    let normal = -half_width.cross(half_height).normalize_or(Vec3::NEG_Y);
                    let up = half_height.normalize_or(Vec3::Y);
                    let proj = glam::camera::rh::proj::directx::perspective(
                        crate::area::RECT_SHADOW_FOV.to_radians(),
                        1.0,
                        near,
                        range,
                    );
                    let sides: &[f32] = if two_sided { &[1.0, -1.0] } else { &[1.0] };
                    sides
                        .iter()
                        .map(|&side| {
                            let view = glam::camera::rh::view::look_at_mat4(
                                light.position,
                                light.position + normal * side,
                                up,
                            );
                            let vp = proj * view;
                            (vp, Frustum::from_view_proj(vp))
                        })
                        .collect()
                }
            };
            shadows.push(LocalShadow {
                light: kept[k],
                record: k,
                layer: next_layer,
                position: light.position,
                range,
                paraboloid: point_paraboloid,
                faces,
            });
            next_layer += shadows.last().map_or(0, |s| s.faces.len());
        }
        if let LightKind::Rect {
            half_width,
            half_height,
            two_sided,
        } = light.kind
        {
            gpu.push(crate::area::rect_record(
                light.position,
                range,
                radiance,
                half_width,
                half_height,
                two_sided,
                slot,
            ));
            continue;
        }
        let (kind, direction, cone) = match light.kind {
            LightKind::Rect { .. } => unreachable!("recorded above"),
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
        // Per metre of distance, in texels of the map (a view's texel spans
        // 2 tan(fov / 2) / size radians at its centre: 2 / size for a 90°
        // cube face, 3.1 / size for a 55° spot's 114° view; a paraboloid's
        // 4 / (scale × size), about 2.2 times a cube face's): the normal
        // offset at a grazing light, a quarter of a texel (lighting.wgsl
        // scales it down to a quarter of that facing the light), and the
        // depth bias, a tenth of a texel, as a fraction of the distance. A
        // local map stores its closed casters' far side, so a lit surface
        // never compares against itself and needs almost no bias; what bias
        // there is lifts the shadow off a caster's base, where the far side
        // meets the floor: a texel of normal offset and half a texel of
        // depth left the floor 1 to 4 cm beside a box 26 % lit under a
        // paraboloid's tent (5 % under a cube face's), these 3 % and 0.2 %
        // (a CPU model of local_shadows_touch_their_casters' scene).
        let texel = if point_paraboloid {
            4.0 / (PARABOLOID_SCALE * shadow_size as f32)
        } else {
            2.0 * half_tan / shadow_size as f32
        };
        let (normal_offset, depth_bias) = (0.25 * texel, 0.1 * texel);
        gpu.push(LightGpu {
            position_range: light.position.extend(range).to_array(),
            color_kind: radiance.extend(kind).to_array(),
            direction_slot: direction.extend(slot).to_array(),
            cone: [cone[0], cone[1], normal_offset, depth_bias],
        });
    }
    (gpu, shadows, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// frame.wgsl's `Frame` is laid out as [`FrameUniform`]: the same size,
    /// so a field the Rust side grows (more cascades, more local layers)
    /// cannot shift every field after it in the shaders.
    #[test]
    fn the_wgsl_frame_is_the_rust_frame() {
        // frame.wgsl with the probe GI's sampling its ambient light calls.
        let module = naga::front::wgsl::parse_str(crate::FRAME_WGSL).expect("frame.wgsl");
        let mut layouter = naga::proc::Layouter::default();
        layouter.update(module.to_ctx()).expect("layout");
        let (handle, ty) = module
            .types
            .iter()
            .find(|(_, t)| t.name.as_deref() == Some("Frame"))
            .expect("struct Frame");
        assert_eq!(
            layouter[handle].size as usize,
            std::mem::size_of::<FrameUniform>()
        );
        let naga::TypeInner::Struct { members, .. } = &ty.inner else {
            panic!("Frame must be a struct")
        };
        let wind = members
            .iter()
            .find(|m| m.name.as_deref() == Some("foliage_wind"))
            .unwrap();
        assert_eq!(
            wind.offset as usize,
            std::mem::offset_of!(FrameUniform, foliage_wind)
        );
        assert_eq!(
            layouter[wind.ty].size as usize,
            std::mem::size_of::<[crate::foliage::FoliageWind; 2]>()
        );
    }

    /// Contact shadows go to the shadowed point and spot lights brightest at
    /// the camera, at most three, never a rect light; a nearer, dimmer light
    /// can outrank a far, bright one.
    #[test]
    fn contact_shadows_go_to_the_brightest_shadowed_lights() {
        let at = |x: f32, intensity: f32, kind: LightKind| Light {
            kind,
            position: Vec3::new(x, 2.0, 0.0),
            intensity,
            cast_shadows: true,
            ..Default::default()
        };
        let rect = LightKind::Rect {
            half_width: Vec3::X,
            half_height: Vec3::Z,
            two_sided: false,
        };
        let spot = LightKind::Spot {
            direction: Vec3::NEG_Y,
            angle_degrees: 40.0,
            penumbra: 0.2,
        };
        let lights = vec![
            at(0.0, 10.0, LightKind::Point),
            at(30.0, 200.0, LightKind::Point),
            at(1.0, 500.0, rect),
            at(2.0, 24.0, spot),
            at(-1.0, 5.0, LightKind::Point),
            at(-2.0, 1.0, LightKind::Point),
        ];
        let kept: Vec<usize> = (0..lights.len()).collect();
        let budget = LocalShadowBudget {
            max_shadowed: 16,
            rect_shadows: true,
            point: PointShadows::Cube,
            shadow_size: 512,
        };
        let (_, shadows, _) = local_lights(&lights, &kept, Vec3::ZERO, budget);
        let picked: Vec<usize> = contact_lights(&shadows, &lights, Vec3::ZERO)
            .iter()
            .map(|s| s.light)
            .collect();
        // Radiance at the camera: 3 (spot, 24 / 8), 2.5 (point 0, 10 / 4 at
        // 2 m), 1 (5 / 5); the far point (200 / 904), the dim one (1 / 8)
        // and the rect lose.
        assert_eq!(picked, vec![3, 0, 4]);
        assert!(contact_lights(&[], &lights, Vec3::ZERO).is_empty());
    }

    /// The phases follow the area a scene pixel covers in the output, so
    /// an output pixel at render scale 0.5 is sampled near its centre as
    /// often as one at scale 1; scale 1 keeps the 8-frame sequence it
    /// always had (frame for frame), and supersampling never goes under 8.
    #[test]
    fn the_jitter_cycles_longer_as_the_scene_shrinks() {
        // SMAA.cpp getJitter + JitteredMatrix translates the image left/up
        // for area layer 1. The HLSL table describes sample positions,
        // which have the opposite sign from that image displacement.
        for n in [0, 2, 32, 1024] {
            assert_eq!(smaa_sample(n), (Vec2::new(-0.25, -0.25), 1.0));
            assert_eq!(smaa_sample(n + 1), (Vec2::new(0.25, 0.25), 2.0));
        }
        assert_eq!(jitter_phases((1920, 1080), (1920, 1080)), 8);
        assert_eq!(jitter_phases((960, 540), (1920, 1080)), 32);
        assert_eq!(jitter_phases((1344, 756), (1920, 1080)), 16);
        assert_eq!(jitter_phases((3840, 2160), (1920, 1080)), 8);
        assert_eq!(jitter_phases((1, 1), (8192, 8192)), 32);
        // Scale 1's sequence as before: Halton(2, 3) from index 1.
        assert_eq!(halton_jitter(0, 8), Vec2::new(0.0, 1.0 / 3.0 - 0.5));
        assert_eq!(halton_jitter(8, 8), halton_jitter(0, 8));
        // At 32 phases a scene pixel's four output pixels (its quadrants)
        // each get a quarter of the samples, give or take two, and the
        // sequence repeats after 32 frames.
        let mut quadrants = [0; 4];
        for i in 0..32 {
            let j = halton_jitter(i, 32);
            assert!(j.abs().max_element() < 0.5);
            quadrants[(j.x >= 0.0) as usize + 2 * (j.y >= 0.0) as usize] += 1;
            assert_eq!(j, halton_jitter(i + 32, 32));
        }
        assert!(
            quadrants.iter().all(|&n| (6..=10).contains(&n)),
            "{quadrants:?}"
        );
    }

    /// `n` bounds on a grid that the frustum of [`view`] cuts through (about
    /// half inside), every seventh one unmarked (it must stay false wherever
    /// its sphere is).
    fn bounds(n: usize) -> (Vec<(Vec3, f32)>, Vec<bool>) {
        let side = (n as f32).sqrt().ceil() as usize;
        let s = 400.0 / side as f32;
        let spheres = (0..n)
            .map(|i| {
                let (x, z) = ((i % side) as f32, (i / side) as f32);
                (
                    Vec3::new((x - side as f32 / 2.0) * s, 2.0, z * s - 100.0),
                    0.5 + (i % 5) as f32 * 0.4,
                )
            })
            .collect();
        (spheres, (0..n).map(|i| i % 7 != 3).collect())
    }

    fn view() -> Frustum {
        let view = glam::camera::rh::view::look_at_mat4(
            Vec3::new(0.0, 30.0, -10.0),
            Vec3::new(20.0, 0.0, 80.0),
            Vec3::Y,
        );
        let proj = glam::camera::rh::proj::directx::perspective(1.0, 16.0 / 9.0, 0.1, 400.0);
        Frustum::from_view_proj(proj * view)
    }

    /// The culled set is each marked bound's own test, the same with lanes
    /// on and off and on any number of threads (run with `OG_LANES=<n>`),
    /// and the call goes to the pool only from `CULL_LANES_FROM` bounds:
    /// from 2,048 to 131,071 a split lost to the serial loop everywhere it
    /// was measured (the former test sample
    #[test]
    fn the_cull_splits_only_from_its_threshold_and_keeps_the_same_bounds() {
        let frustum = view();
        let from = CULL_LANES_FROM;
        for n in [
            0,
            1,
            2_048,
            8_192,
            100_000,
            from - 1,
            from,
            from + CULL_GRAIN + 17,
            2 * from + 7,
        ] {
            let (spheres, marks) = bounds(n);
            let expected: Vec<bool> = (0..n)
                .map(|k| marks[k] && frustum.intersects_sphere(spheres[k].0, spheres[k].1))
                .collect();
            if n >= 2_048 {
                let kept = expected.iter().filter(|v| **v).count();
                assert!(
                    kept > n / 5 && kept < n * 4 / 5,
                    "{n} bounds: {kept} kept, not a partial view"
                );
            }
            for on in [true, false] {
                og_lanes::set_enabled(on);
                let mut inside = marks.clone();
                let calls = og_lanes::calls_here();
                frustum.test_spheres(&mut inside, &spheres);
                let split = og_lanes::calls_here() - calls;
                let pool = on && og_lanes::threads() > 1;
                assert_eq!(
                    split,
                    u64::from(pool && n >= from),
                    "{n} bounds, lanes {on}: parallel calls"
                );
                assert!(
                    inside == expected,
                    "{n} bounds, lanes {on}: another culled set"
                );
            }
            og_lanes::set_enabled(true);
        }
    }
}
