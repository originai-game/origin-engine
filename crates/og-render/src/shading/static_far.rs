//! Static lights' far field ([`crate::SceneLighting::static_reach`],
//! RenderSettings.staticLightReach): a light that never moves has its
//! diffuse light shaded per pixel only near itself; farther out that light
//! comes from a grid of irradiance probes round the camera, worked out
//! here on the CPU when the grid moves a cell or a static light changes.
//! Its highlight stays per pixel over its whole range on glossy surfaces,
//! which is where a far light shows (the wet street's streaks); on a rough
//! one past the reach a rect light is not drawn at all (area.wgsl).
//!
//! The split is UE's stationary lights' (direct specular per pixel, diffuse
//! from the precomputed light) and Unity's mixed lighting's distance
//! split; the field is analytic (the lights have no shadow), so nothing
//! is baked or downloaded.
//!
//! - Which lights: [`far_lights`], static ones without a shadow or a
//!   cookie, of a kind the renderer draws.
//! - Their records keep their range, so the clusters, the fog and the
//!   passes that shade every record (a reflection probe's capture, the
//!   probe GI's trace) draw them as before; `source.y` is the reach, and
//!   `static_near` (frame.wgsl) fades the per-pixel diffuse light out over
//!   its last quarter. Every probe holds what that fade leaves ([`near`]).
//! - The grid: [`COUNTS`] probes [`spacing`] apart (half the reach), on the
//!   world's cells of that spacing round the camera, each the irradiance
//!   over π of the far lights as L1 spherical harmonics (`[c1, c0]` per
//!   channel, as a light volume's). A point or spot light is a point; a
//!   rect light 4 × 4 patches, each a point with its area and cosine.
//! - The shader (static_far.wgsl): one trilinear gather per lit fragment,
//!   added to its diffuse light around it (surface_shade.wgsl
//!   `surface_irradiance`), in a pipeline only while a frame draws it
//!   (`OG_STATIC_FAR`, pipeline_features.rs `STATIC_FAR`): with the reach
//!   at 0 the lit pipelines are those made without it. A leaf's light from
//!   behind and a toon material's flat ambient do not take it.
//! - Its probes live in the light volumes' buffer (frame binding 10) after
//!   the probe GI's region, described by [`HEADER_ROWS`] rows after the
//!   probe GI's header; none when the reach is 0.

use crate::scene::{LightDesc, LightKind, RenderScene};
use glam::{IVec3, Vec3};
use og_render_data::light::{LightMobility, range_window};
use std::f32::consts::PI;

/// The grid's probes along x, y and z (`FAR_COUNTS`).
pub(crate) const COUNTS: [u32; 3] = [32, 12, 32];
/// Its probes, and their vec4 in the buffer (three a probe).
pub(crate) const PROBES: usize = (COUNTS[0] * COUNTS[1] * COUNTS[2]) as usize;
pub(crate) const VEC4S: usize = 3 * PROBES;
/// The header rows after the probe GI's (`FAR_HEADER_AT`): the grid's
/// first probe's position and its first vec4 (-1: none), then its spacing.
pub(crate) const HEADER_AT: usize = crate::lightmap::VOLUME_HEADER + crate::gi_probes::HEADER_ROWS;
pub(crate) const HEADER_ROWS: usize = 2;
const _: () = assert!(HEADER_AT == 33, "FAR_HEADER_AT in static_far.wgsl");
/// The grid's cells below the camera's (the rest above: a camera near the
/// ground sees more above it).
const BELOW: i32 = 4;
/// Where a static light's per-pixel light starts to fade, in its reach.
const FADE_START: f32 = 0.75;
/// A rect light's patches a side.
const PATCHES: u32 = 4;

/// The header of no far field.
pub(crate) const OFF: [[f32; 4]; HEADER_ROWS] = [[0.0, 0.0, 0.0, -1.0], [0.0; 4]];

/// The lights that take the far field: those that never move (stationary
/// or static, [`og_render_data::light::LightMobility`]), cast no shadow and
/// have no cookie (their per-pixel light is what a field cannot hold), of
/// a kind the renderer draws. None while the reach is 0.
pub(crate) fn far_lights(scene: &RenderScene, skip_rects: bool) -> Vec<usize> {
    let l = &scene.lighting;
    if l.static_reach <= 0.0 {
        return Vec::new();
    }
    l.lights
        .iter()
        .enumerate()
        .filter(|&(i, light)| {
            light.mobility != LightMobility::Movable
                && !light.cast_shadows
                && !(skip_rects && matches!(light.kind, LightKind::Rect { .. }))
                && !l.light_cookies.iter().any(|c| c.light == i)
        })
        .map(|(i, _)| i)
        .collect()
}

/// The share of a static light shaded per pixel `d` metres from it
/// (frame.wgsl `static_near`): whole within three quarters of `reach`,
/// falling smoothly to none at it.
pub(crate) fn near(d: f32, reach: f32) -> f32 {
    let t = ((d - FADE_START * reach) / ((1.0 - FADE_START) * reach)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// The probes' spacing for `reach`.
pub(crate) fn spacing(reach: f32) -> f32 {
    (0.5 * reach).max(0.25)
}

/// The grid round `eye`: its first probe's world cell.
fn first_cell(eye: Vec3, spacing: f32) -> IVec3 {
    let c = (eye / spacing).floor().as_ivec3();
    c - IVec3::new(COUNTS[0] as i32 / 2, BELOW, COUNTS[2] as i32 / 2)
}

/// Adds irradiance `e` (on a surface facing it) arriving from `l` (unit,
/// towards the light) to a probe's harmonics: a clamped cosine's L1
/// projection, E (1/4 + 1/2 n·l), over π.
fn add(probe: &mut [[f32; 4]], e: Vec3, l: Vec3) {
    for (ch, c) in probe.iter_mut().zip(e.to_array()) {
        let k = c / (2.0 * PI);
        ch[0] += k * l.x;
        ch[1] += k * l.y;
        ch[2] += k * l.z;
        ch[3] += 0.5 * k;
    }
}

/// What the lights `far` send the point `p` past their near field, into
/// `probe` (three vec4, as the shader reads them).
fn light_at(light: &LightDesc, reach: f32, p: Vec3, probe: &mut [[f32; 4]]) {
    let to_p = p - light.position;
    let d2 = to_p.length_squared().max(1e-4);
    let range = light.range.max(1e-3);
    if d2 >= range * range {
        return;
    }
    let far = 1.0 - near(d2.sqrt(), reach);
    if far <= 0.0 {
        return;
    }
    let radiance = light.color * light.intensity.max(0.0) * range_window(d2, range) * far;
    match light.kind {
        LightKind::Rect {
            half_width,
            half_height,
            two_sided,
        } => {
            let side = half_width.cross(half_height);
            let area = 4.0 * side.length();
            let emits = -side.normalize_or_zero();
            if area <= 0.0 {
                return;
            }
            let a = area / (PATCHES * PATCHES) as f32;
            for i in 0..PATCHES {
                for j in 0..PATCHES {
                    let u = 2.0 * (i as f32 + 0.5) / PATCHES as f32 - 1.0;
                    let v = 2.0 * (j as f32 + 0.5) / PATCHES as f32 - 1.0;
                    let q = light.position + half_width * u + half_height * v;
                    let d = p - q;
                    let dd = d.length_squared().max(1e-4);
                    let dir = d / dd.sqrt();
                    let mut cos = emits.dot(dir);
                    if two_sided {
                        cos = cos.abs();
                    }
                    if cos <= 0.0 {
                        continue;
                    }
                    add(probe, radiance * (a * cos / dd), -dir);
                }
            }
        }
        LightKind::Point | LightKind::Spot { .. } => {
            let l = -to_p / d2.sqrt();
            let atten = light.cone(l) / (d2 + light.radius * light.radius);
            if atten > 0.0 {
                add(probe, radiance * atten, l);
            }
        }
    }
}

/// The grid's probes from its first probe at `min`, `spacing` apart: per
/// probe its red, green and blue harmonics.
pub(crate) fn field(
    lights: &[LightDesc],
    far: &[usize],
    reach: f32,
    min: Vec3,
    spacing: f32,
) -> Vec<[f32; 4]> {
    let mut out = vec![[0.0f32; 4]; VEC4S];
    let c = COUNTS.map(|n| n as i32);
    for &i in far {
        let light = &lights[i];
        // The probes inside its range.
        let r = light.range.max(1e-3);
        let lo = ((light.position - Vec3::splat(r) - min) / spacing)
            .ceil()
            .as_ivec3()
            .max(IVec3::ZERO);
        let hi = ((light.position + Vec3::splat(r) - min) / spacing)
            .floor()
            .as_ivec3()
            .min(IVec3::new(c[0] - 1, c[1] - 1, c[2] - 1));
        for z in lo.z..=hi.z {
            for y in lo.y..=hi.y {
                for x in lo.x..=hi.x {
                    let p = min + IVec3::new(x, y, z).as_vec3() * spacing;
                    let at = 3 * ((z * c[1] + y) * c[0] + x) as usize;
                    light_at(light, reach, p, &mut out[at..at + 3]);
                }
            }
        }
    }
    out
}

/// The far field on the GPU: what its probes were last worked out for.
#[derive(Default)]
pub(crate) struct StaticFar {
    key: Option<(IVec3, u32, u64, usize)>,
}

impl StaticFar {
    /// Forgets what the buffer holds (it was made again).
    pub fn reset(&mut self) {
        self.key = None;
    }

    /// Works the probes out again into `buffer` at vec4 `at` when the grid
    /// moved a cell or a far light changed. Answers the header rows (none
    /// without far lights).
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        scene: &RenderScene,
        far: &[usize],
        at: usize,
    ) -> [[f32; 4]; HEADER_ROWS] {
        let reach = scene.lighting.static_reach;
        if far.is_empty() {
            self.key = None;
            return OFF;
        }
        let s = spacing(reach);
        let cell = first_cell(scene.frame.camera.position, s);
        let key = (
            cell,
            s.to_bits(),
            lights_hash(&scene.lighting.lights, far, reach),
            at,
        );
        let min = cell.as_vec3() * s;
        if self.key != Some(key) {
            let data = field(&scene.lighting.lights, far, reach, min, s);
            crate::traffic::write_buffer(
                queue,
                buffer,
                (at * 16) as u64,
                bytemuck::cast_slice(&data),
            );
            self.key = Some(key);
        }
        [min.extend(at as f32).to_array(), [s, reach, 0.0, 0.0]]
    }
}

/// What the far field depends on of its lights.
fn lights_hash(lights: &[LightDesc], far: &[usize], reach: f32) -> u64 {
    let mut words = vec![u64::from(reach.to_bits())];
    let eat = |words: &mut Vec<u64>, x: f32| words.push(u64::from(x.to_bits()));
    eat(&mut words, reach);
    for &i in far {
        let l = &lights[i];
        eat(&mut words, i as f32);
        for v in [l.position, l.color] {
            v.to_array().into_iter().for_each(|x| eat(&mut words, x));
        }
        [l.intensity, l.range, l.radius]
            .into_iter()
            .for_each(|x| eat(&mut words, x));
        match l.kind {
            LightKind::Point => eat(&mut words, 0.0),
            LightKind::Spot {
                direction,
                angle_degrees,
                penumbra,
            } => {
                direction
                    .to_array()
                    .into_iter()
                    .for_each(|x| eat(&mut words, x));
                eat(&mut words, angle_degrees);
                eat(&mut words, penumbra);
            }
            LightKind::Rect {
                half_width,
                half_height,
                two_sided,
            } => {
                half_width
                    .to_array()
                    .into_iter()
                    .for_each(|x| eat(&mut words, x));
                half_height
                    .to_array()
                    .into_iter()
                    .for_each(|x| eat(&mut words, x));
                eat(&mut words, f32::from(u8::from(two_sided)));
            }
        }
    }
    og_core::hash::hash_words(words)
}
