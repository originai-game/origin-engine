//! Lens flare ([`crate::LensFlare`], `lens_flare.wgsl`): the light sources
//! a frame's flare draws and their rows, and its passes' pipelines.
//!
//! Passes (after TAA, at the post frame's size; between the bloom
//! prefilter and the last bloom upsample, which adds onto the `bloom.0`
//! they read): `FlareOcclusion` (the prepass depth into `flare.vis`, one
//! texel per light), `Flare` (Filament's ghosts and halo from `bloom.0`
//! and the lights' discs into `flare.raw`, a quarter of the frame), and
//! `FlareBlurX` and `FlareBlurY` (Filament's 9-tap Gaussian into
//! `flare.x`, then `flare`), which the tonemap adds (post.wgsl, binding 7).
//!
//! Binding budget: the flare layout is one group of one uniform, one
//! filtering sampler and four sampled 2D textures (source, view depth,
//! exposure, visibility), against WebGPU's baseline of 12 uniform buffers,
//! 16 samplers and 16 sampled textures per stage. The post layout's
//! tonemap gains one sampled texture (binding 7: six of 16). FxPost and
//! the effect layout are unchanged.

use super::*;
use crate::{Light, LightKind};
use glam::{Mat4, Vec2, Vec3};

/// The light sources a frame's flare draws (`lens_flare.wgsl`
/// `FLARE_LIGHTS`): the sun, then the most important point and spot
/// lights.
pub(crate) const FLARE_LIGHTS: usize = 8;
/// A point or spot light's source radius in metres: its disc on screen,
/// the radius its occlusion is searched over, and how far in front of it
/// a surface may stand and not hide it (its own bulb or shade).
pub(crate) const FLARE_SOURCE_RADIUS: f32 = 0.2;
/// The sun's angular radius in degrees under a sky that draws no disc:
/// the real sun's.
const SUN_RADIUS_DEGREES: f32 = 0.27;
/// The bloom taps masked out round a source where they show it, in its
/// disc's radii (its image and the bloom chain's spread of it): the sky
/// round the sun, a light's own bulb.
const SUN_MASK_RADII: f32 = 3.0;
const LIGHT_MASK_RADII: f32 = 1.5;
/// A source ghost's least radius, as a fraction of the frame's height: a
/// soft aperture, not a sharp copy of a small source (`lens_flare.wgsl`
/// FLARE_APERTURE_TEXELS keeps it three flare texels in frames under 150
/// pixels high).
const GHOST_APERTURE: f32 = 0.08;
/// A source ghost's share of its exposed radiance, the same for the sun and
/// every light (art direction: a ghost 5 to 15 % of its source's halo).
const GHOST_GAIN: f32 = 0.007;
/// The view depth a local light must lie past to flare (metres).
const MIN_LIGHT_DEPTH: f32 = 0.05;

/// The flare target's format (also `flare.raw` and `flare.x`).
pub(crate) const FLARE_FORMAT: wgpu::TextureFormat = HDR_FORMAT;
/// `flare.vis`: one light's visible fraction per texel.
pub(crate) const FLARE_VIS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

/// `struct Flare` in lens_flare.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FlareUniform {
    pub size: [f32; 4],
    pub ghosts: [f32; 4],
    pub halo: [f32; 4],
    pub exposure: [f32; 4],
    pub meter: [f32; 4],
    pub mask: [f32; 4],
    pub lights: [[f32; 4]; FLARE_LIGHTS],
    pub colors: [[f32; 4]; FLARE_LIGHTS],
}

/// What places the sources on screen: the main camera without TAA's
/// jitter, so neither the ghosts nor the occlusion move with it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlareCamera {
    pub view: Mat4,
    pub view_proj: Mat4,
    pub eye: Vec3,
    pub forward: Vec3,
    /// The projection's `[1][1]`: 1 / tan(half the vertical field of view).
    pub focal: f32,
    pub far: f32,
}

/// The sun as the flare sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FlareSun {
    /// Towards the drawn sun.
    pub direction: Vec3,
    /// Its light at the ground: the directional light's colour times its
    /// intensity (clouds and the air already in it, `Clouds::light_scene`).
    pub irradiance: Vec3,
    /// Its disc's angular radius in degrees (0: the sky draws none).
    pub radius_degrees: f32,
    /// What the fog leaves of the sky at its direction.
    pub fog_clear: f32,
}

/// The sources' rows of a frame (`FlareUniform`'s `lights` and `colors`
/// and their count).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FlareSources {
    pub count: usize,
    pub lights: [[f32; 4]; FLARE_LIGHTS],
    pub colors: [[f32; 4]; FLARE_LIGHTS],
}

/// `p` on screen (uv, y down) and its clip w.
fn project(view_proj: Mat4, p: Vec3) -> (Vec2, f32) {
    let clip = view_proj * p.extend(1.0);
    let ndc = clip.truncate().truncate() / clip.w.max(1e-6);
    (Vec2::new(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5), clip.w)
}

/// Whether a disc of `radius` (height units) at `uv` reaches the frame.
fn on_screen(uv: Vec2, radius: f32, aspect: f32) -> bool {
    let rx = radius / aspect.max(1e-6);
    uv.x > -rx && uv.x < 1.0 + rx && uv.y > -radius && uv.y < 1.0 + radius
}

/// The sources the flare draws this frame: the sun in slot 0 when it is in
/// front of the camera, above the horizon and on screen, then up to
/// [`FLARE_LIGHTS`] - 1 point and spot lights in front and on screen,
/// the most important at the camera first ([`crate::clusters::importance`],
/// as contact shadows pick theirs; ties in scene order). A light's row
/// holds its disc's uv, the view depth a sample must reach to see it (the
/// sun: -1, the sky) and its radius in height units; its colour row the
/// disc's radiance (its light over the disc's solid angle, so a source's
/// ghosts do not shrink with distance), and its gain. A source of gain 0
/// keeps its slot: it is still masked out of the bloom taps, so it casts no
/// ghosts at all. `fog`: what the fog leaves of a light at a point.
pub(crate) fn flare_sources(
    flare: &crate::LensFlare,
    camera: &FlareCamera,
    aspect: f32,
    sun: Option<FlareSun>,
    lights: &[Light],
    fog: &dyn Fn(Vec3) -> f32,
) -> FlareSources {
    let mut out = FlareSources::default();
    if let Some(s) = sun {
        let dir = s.direction.normalize_or(Vec3::Y);
        let theta = if s.radius_degrees > 0.0 {
            s.radius_degrees.clamp(0.0, 20.0)
        } else {
            SUN_RADIUS_DEGREES
        }
        .to_radians();
        let (uv, w) = project(camera.view_proj, camera.eye + dir * camera.far * 0.5);
        let radius = theta.tan() * camera.focal * 0.5;
        // Fading out as it sets (Wicked skips a sun below the horizon).
        let horizon = smoothstep(-0.03, 0.03, dir.y);
        if w > 0.0
            && dir.dot(camera.forward) > 0.0
            && horizon > 0.0
            && on_screen(uv, radius, aspect)
        {
            let radiance = s.irradiance * s.fog_clear / (std::f32::consts::PI * theta * theta);
            out.lights[0] = [uv.x, uv.y, -1.0, radius];
            out.colors[0] = radiance.extend(flare.sun * horizon).to_array();
            out.count = 1;
        }
    }
    let mut picked: Vec<(usize, Vec2, f32, f32, f32)> = Vec::new();
    for (i, l) in lights.iter().enumerate() {
        if l.intensity <= 0.0 || matches!(l.kind, LightKind::Rect { .. }) {
            continue;
        }
        let depth = -(camera.view * l.position.extend(1.0)).z;
        if depth < MIN_LIGHT_DEPTH {
            continue;
        }
        // A spot is seen only from inside its cone.
        let cone = match l.kind {
            LightKind::Spot {
                direction,
                angle_degrees,
                penumbra,
            } => {
                let (outer, inner) = crate::frame::spot_cone(angle_degrees, penumbra);
                let to_eye = (camera.eye - l.position).normalize_or_zero();
                let cos = direction.normalize_or_zero().dot(to_eye);
                smoothstep(outer, inner.max(outer + 1e-4), cos)
            }
            _ => 1.0,
        };
        let (uv, _) = project(camera.view_proj, l.position);
        let radius = FLARE_SOURCE_RADIUS / depth * camera.focal * 0.5;
        if cone > 0.0 && on_screen(uv, radius, aspect) {
            picked.push((i, uv, depth, radius, cone));
        }
    }
    let score = |i: usize| crate::clusters::importance(&lights[i], camera.eye);
    og_core::sort::sort_by(&mut picked, |a, b| {
        score(b.0).total_cmp(&score(a.0)).then(a.0.cmp(&b.0))
    });
    for (i, uv, depth, radius, cone) in picked.into_iter().take(FLARE_LIGHTS - 1) {
        let l = &lights[i];
        let radiance = l.color * l.intensity * cone * fog(l.position)
            / (std::f32::consts::PI * FLARE_SOURCE_RADIUS * FLARE_SOURCE_RADIUS);
        out.lights[out.count] = [uv.x, uv.y, depth - FLARE_SOURCE_RADIUS, radius];
        out.colors[out.count] = radiance.extend(flare.lights).to_array();
        out.count += 1;
    }
    out
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The flare passes' uniform: `flare`'s parameters, `sources`, the post
/// uniform's exposure rows (`exposure`, and the compensation) and the frame
/// after TAA's size.
pub(crate) fn flare_uniform(
    flare: &crate::LensFlare,
    sources: &FlareSources,
    exposure: ([f32; 4], f32),
    post_size: (u32, u32),
    far: f32,
) -> FlareUniform {
    let (pw, ph) = (post_size.0.max(1) as f32, post_size.1.max(1) as f32);
    let (qw, qh) = flare_size(post_size);
    let aspect = pw / ph;
    FlareUniform {
        size: [qw as f32, qh as f32, aspect, 1.0 / aspect],
        ghosts: [
            flare.ghosts.min(16) as f32,
            flare.ghost_spacing.clamp(0.0, 1.0),
            flare.ghost_threshold.max(0.0),
            flare.chromatic_aberration.max(0.0),
        ],
        halo: [
            flare.halo_radius.clamp(0.0, 0.5),
            flare.halo_thickness.max(0.0),
            flare.halo_threshold.max(0.0),
            far,
        ],
        exposure: exposure.0,
        meter: [exposure.1, sources.count as f32, 0.0, 0.0],
        mask: [SUN_MASK_RADII, LIGHT_MASK_RADII, GHOST_APERTURE, GHOST_GAIN],
        lights: sources.lights,
        colors: sources.colors,
    }
}

/// The flare's targets' size: a quarter of the frame after TAA (half of
/// `bloom.0`'s, as Filament draws its flare at half its bloom's base).
pub(crate) fn flare_size(post_size: (u32, u32)) -> (u32, u32) {
    ((post_size.0 / 4).max(1), (post_size.1 / 4).max(1))
}

/// The flare passes' layout, pipelines and uniform.
pub(crate) struct FlarePasses {
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub occlusion: Deferred<wgpu::RenderPipeline>,
    pub flare: Deferred<wgpu::RenderPipeline>,
    pub blur_x: Deferred<wgpu::RenderPipeline>,
    pub blur_y: Deferred<wgpu::RenderPipeline>,
}

/// The flare module's body (`lens_flare.wgsl`).
pub(crate) const FLARE_WGSL: &str = wgsl!("lens_flare.wgsl");

impl FlarePasses {
    pub fn new(device: &wgpu::Device) -> FlarePasses {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lens flare"),
            entries: &[
                layout_entry(0, UNIFORM),
                layout_entry(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                layout_entry(2, float_texture(true)),
                // The view depth, the exposure (R32Float when adapted) and
                // the visibility, read with textureLoad.
                layout_entry(3, float_texture(false)),
                layout_entry(4, float_texture(false)),
                layout_entry(5, float_texture(false)),
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lens flare"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::deferred::module(device, "lens flare", fullscreen_source(FLARE_WGSL));
        let pipeline = |label: &'static str, entry: &'static str, format| {
            deferred_fullscreen(device, label, &module, entry, &pl, format, None)
        };
        FlarePasses {
            occlusion: pipeline("flare occlusion", "fs_flare_occlusion", FLARE_VIS_FORMAT),
            flare: pipeline("flare", "fs_flare", FLARE_FORMAT),
            blur_x: pipeline("flare blur x", "fs_flare_blur_x", FLARE_FORMAT),
            blur_y: pipeline("flare blur y", "fs_flare_blur_y", FLARE_FORMAT),
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("lens flare"),
                size: std::mem::size_of::<FlareUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> FlareCamera {
        let eye = Vec3::new(0.0, 1.0, 0.0);
        let target = Vec3::new(0.0, 1.0, -10.0);
        let view = glam::camera::rh::view::look_at_mat4(eye, target, Vec3::Y);
        let fov = 60f32.to_radians();
        let proj = glam::camera::rh::proj::directx::perspective(fov, 16.0 / 9.0, 0.1, 500.0);
        FlareCamera {
            view,
            view_proj: proj * view,
            eye,
            forward: (target - eye).normalize(),
            focal: 1.0 / (fov * 0.5).tan(),
            far: 500.0,
        }
    }

    fn point(position: Vec3, intensity: f32) -> Light {
        Light {
            position,
            intensity,
            ..Default::default()
        }
    }

    fn no_fog(_: Vec3) -> f32 {
        1.0
    }

    /// The sun ahead and up is slot 0, at its projected uv, its disc the
    /// angular radius's share of the frame's height, its radiance its light
    /// over that disc's solid angle; behind the camera or below the
    /// horizon it has no slot.
    #[test]
    fn the_sun_takes_slot_zero_only_when_it_can_be_seen() {
        let cam = camera();
        let flare = crate::LensFlare::default();
        let sun = FlareSun {
            direction: Vec3::new(0.0, 0.3, -1.0),
            irradiance: Vec3::splat(3.0),
            radius_degrees: 1.8,
            fog_clear: 0.5,
        };
        let s = flare_sources(&flare, &cam, 16.0 / 9.0, Some(sun), &[], &no_fog);
        assert_eq!(s.count, 1);
        let (u, v) = (s.lights[0][0], s.lights[0][1]);
        assert!((u - 0.5).abs() < 1e-4, "centred across: {u}");
        // 0.3 up over 1 ahead at 60 degrees: tan 16.7° / tan 30° of the half-height.
        let want_v = 0.5 - 0.5 * 0.3 / (30f32.to_radians().tan());
        assert!((v - want_v).abs() < 1e-3, "v {v} want {want_v}");
        assert_eq!(s.lights[0][2], -1.0, "the sun is seen where the sky is");
        let theta = 1.8f32.to_radians();
        let radius = theta.tan() / (30f32.to_radians().tan()) * 0.5;
        assert!((s.lights[0][3] - radius).abs() < 1e-5);
        let radiance = 3.0 * 0.5 / (std::f32::consts::PI * theta * theta);
        assert!((s.colors[0][0] - radiance).abs() / radiance < 1e-4);
        assert_eq!(s.colors[0][3], 1.0);
        let behind = FlareSun {
            direction: Vec3::new(0.0, 0.3, 1.0),
            ..sun
        };
        assert_eq!(
            flare_sources(&flare, &cam, 16.0 / 9.0, Some(behind), &[], &no_fog).count,
            0
        );
        let set = FlareSun {
            direction: Vec3::new(0.0, -0.05, -1.0),
            ..sun
        };
        assert_eq!(
            flare_sources(&flare, &cam, 16.0 / 9.0, Some(set), &[], &no_fog).count,
            0
        );
        // At gain 0 it keeps its slot (it is masked out of the bloom taps).
        let off = crate::LensFlare { sun: 0.0, ..flare };
        let s = flare_sources(&off, &cam, 16.0 / 9.0, Some(sun), &[], &no_fog);
        assert_eq!((s.count, s.colors[0][3]), (1, 0.0));
    }

    /// Lights fill the slots after the sun by importance at the camera,
    /// ties in scene order, at most seven; those behind the camera, off
    /// screen, of no intensity or rect lights take none; a spot seen from
    /// outside its cone takes none; the visibility depth stands the source
    /// radius in front of the light.
    #[test]
    fn lights_take_the_slots_by_importance_with_ties_in_scene_order() {
        let cam = camera();
        let flare = crate::LensFlare::default();
        let mut lights: Vec<Light> = (0..10)
            .map(|i| point(Vec3::new(i as f32 * 0.2 - 1.0, 1.0, -8.0), 10.0 + i as f32))
            .collect();
        // Light 9 mirrored: as important, and later in the scene, so after it.
        lights.push(point(Vec3::new(-0.8, 1.0, -8.0), 19.0));
        lights.push(point(Vec3::new(0.0, 1.0, 5.0), 1000.0)); // behind
        lights.push(point(Vec3::new(80.0, 1.0, -8.0), 1000.0)); // off screen
        lights.push(point(Vec3::new(0.0, 1.0, -8.0), 0.0)); // dark
        lights.push(Light {
            kind: LightKind::Spot {
                direction: Vec3::Z,
                angle_degrees: 30.0,
                penumbra: 0.0,
            },
            ..point(Vec3::new(0.0, 0.0, -6.0), 1000.0)
        });
        lights.push(Light {
            kind: LightKind::Spot {
                direction: Vec3::NEG_Z,
                angle_degrees: 30.0,
                penumbra: 0.0,
            },
            ..point(Vec3::new(0.0, 0.0, -6.0), 900.0)
        });
        let s = flare_sources(&flare, &cam, 16.0 / 9.0, None, &lights, &no_fog);
        assert_eq!(s.count, FLARE_LIGHTS - 1);
        // The spot facing the camera first (the one facing away, which
        // would score before the points, is not a candidate); then the
        // points 9, 10 (a tie, in scene order), 8, 7, 6 and 5.
        let first = s.lights[0];
        let (uv, _) = project(cam.view_proj, Vec3::new(0.0, 0.0, -6.0));
        assert!((first[0] - uv.x).abs() < 1e-5 && (first[1] - uv.y).abs() < 1e-5);
        assert!((first[2] - (6.0 - FLARE_SOURCE_RADIUS)).abs() < 1e-4);
        let order: Vec<f32> = (1..s.count).map(|k| s.colors[k][0]).collect();
        let radiance = |i: f32| i / (std::f32::consts::PI * FLARE_SOURCE_RADIUS.powi(2));
        let want: Vec<f32> = [19.0, 19.0, 18.0, 17.0, 16.0, 15.0]
            .into_iter()
            .map(radiance)
            .collect();
        for (got, want) in order.iter().zip(&want) {
            assert!((got - want).abs() / want < 1e-4, "{order:?} want {want:?}");
        }
        let (uv9, _) = project(cam.view_proj, lights[9].position);
        assert!(
            (s.lights[1][0] - uv9.x).abs() < 1e-5,
            "light 9 before its tie"
        );
        let off = crate::LensFlare {
            lights: 0.0,
            ..flare
        };
        let s = flare_sources(&off, &cam, 16.0 / 9.0, None, &lights, &no_fog);
        assert_eq!(s.count, FLARE_LIGHTS - 1);
        assert!((0..s.count).all(|k| s.colors[k][3] == 0.0));
    }
}
