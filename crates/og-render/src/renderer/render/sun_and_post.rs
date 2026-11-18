//! The sun on screen for the god rays, the lens flare's sources and
//! uniform, the post chain's uniform and the exposure's adaptation step, in
//! that order.

use super::*;

pub(super) struct WriteSunAndPostInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub aspect: f32,
    pub light: &'a crate::DirectionalLight,
    pub to_light: Vec3,
    pub uniform: &'a frame::FrameUniform,
    pub post_fx: &'a crate::PostEffects,
    pub god_rays: Option<crate::GodRays>,
    pub lens_flare: Option<crate::LensFlare>,
    pub flare_sources_on: bool,
    pub glow: bool,
    pub sprite_frame: &'a sprites::SpriteFrame,
}

pub(super) struct WriteSunAndPostOutputs {
    pub sun_ndc: Vec2,
    pub sun_fade: f32,
    pub god_visible: bool,
    /// The god rays' sun (`FxPostUniform::god_sun`).
    pub god_sun: [[f32; 4]; 2],
}

impl Renderer {
    /// Places the sun on screen and writes the flare's and post chain's uniforms.
    #[allow(clippy::too_many_lines)]
    pub(super) fn write_sun_and_post(
        &mut self,
        WriteSunAndPostInputs {
            scene,
            camera,
            cam,
            aspect,
            light,
            to_light,
            uniform,
            post_fx,
            god_rays,
            lens_flare,
            flare_sources_on,
            glow,
            sprite_frame,
        }: WriteSunAndPostInputs<'_>,
    ) -> WriteSunAndPostOutputs {
        // The sun on screen, for god rays.
        let sun_clip = cam.view_proj * (camera.position + to_light * camera.far * 0.5).extend(1.0);
        let sun_ndc = sun_clip.truncate().truncate() / sun_clip.w.max(1e-6);
        let sun_front = sun_clip.w > 0.0 && to_light.dot(cam.forward) > 0.0;
        let sun_fade = (2.0 - sun_ndc.abs().max_element()).clamp(0.0, 1.0);
        let god_visible = god_rays.is_some() && sun_front && sun_fade > 0.0;
        let god_sun = if god_visible {
            god_sun(
                &drawn_sun(scene, light, to_light, uniform, camera),
                cam.proj,
                self.height,
            )
        } else {
            [[0.0; 4]; 2]
        };
        // The lens flare's sources, placed with the unjittered camera, and
        // its passes' uniform; its starburst turns with the camera's
        // forward axis (Filament: the sum of the view matrix's third
        // column).
        let flare_turn = cam.view.z_axis.truncate().dot(Vec3::ONE);
        if let Some(f) = &lens_flare {
            let sources = if flare_sources_on {
                let sun = drawn_sun(scene, light, to_light, uniform, camera);
                post::flare_sources(
                    f,
                    &post::FlareCamera {
                        view: cam.view,
                        view_proj: cam.view_proj,
                        eye: camera.position,
                        forward: cam.forward,
                        focal: cam.proj.y_axis.y,
                        far: camera.far,
                    },
                    aspect,
                    // The fog's share of the sky towards the sun is not in
                    // the light; the clouds' is read off the drawn disc.
                    Some(post::FlareSun {
                        direction: sun.direction,
                        irradiance: sun.irradiance,
                        radius_degrees: sun.radius_degrees,
                        fog_clear: uniform
                            .atmosphere
                            .sky_fog_clear(camera.position, sun.direction),
                        disc: sun.disc,
                    }),
                    &scene.lighting.lights,
                    &|p| uniform.atmosphere.fog_clear(camera.position, p),
                )
            } else {
                post::FlareSources::default()
            };
            let u = post::flare_uniform(
                f,
                &sources,
                post::PostPasses::exposure_rows(camera.exposure),
                self.history_size(),
                camera.far,
            );
            self.queue
                .write_buffer(&self.post.flare.buffer, 0, bytemuck::bytes_of(&u));
        }
        self.post.write_uniform(
            &self.queue,
            camera.exposure,
            post_fx,
            glow,
            god_rays.is_some(),
            sprite_frame.untonemapped,
            scene.frame.tick,
            self.history_size().0,
            lens_flare.as_ref().map(|f| (f, flare_turn)),
            self.display
                .as_ref()
                .map_or([0.0; 4], |d| d.curve(scene.look.range_limit)),
        );
        if camera.exposure.adaptation().is_some() {
            self.post.advance_exposure(
                &self.queue,
                camera.exposure,
                scene.frame.tick,
                scene.frame.tick_rate,
                (self.frame_index % 2) as usize,
            );
        } else {
            self.post.forget_exposure();
        }
        WriteSunAndPostOutputs {
            sun_ndc,
            sun_fade,
            god_visible,
            god_sun,
        }
    }
}

/// The sun the frame draws: towards it, its disc's angular radius
/// (degrees; 0: none drawn), its irradiance and, with clouds over the
/// procedural sky, the luminance its disc is drawn with at its centre,
/// unexposed (0: not drawn through clouds). The clouds hide the disc as
/// they hide the sun, so the irradiance is the sun's above them (the light
/// undimmed by their mean cover, `Clouds::light`), and what of it they let
/// through is read off the drawn disc: what the frame shows of it over
/// `disc` (the lens flare's occlusion, the god rays' mask).
struct DrawnSun {
    direction: Vec3,
    radius_degrees: f32,
    irradiance: Vec3,
    disc: f32,
}

fn drawn_sun(
    scene: &RenderScene,
    light: &crate::DirectionalLight,
    to_light: Vec3,
    uniform: &frame::FrameUniform,
    camera: &Camera,
) -> DrawnSun {
    let look = scene
        .lighting
        .environment
        .sky
        .look
        .as_ref()
        .unwrap_or(&SkyLook::DEFAULT);
    // The procedural sky's sun, the sky cube's when it is the procedural
    // one, else the light's direction with no disc.
    let (direction, radius_degrees) = match scene.lighting.environment.background {
        Background::Procedural { sun_direction, .. } => (sun_direction, look.sun_size),
        Background::Sky if scene.lighting.environment_map.is_none() => {
            (scene.lighting.environment.sky.sun_direction, look.sun_size)
        }
        _ => (to_light, 0.0),
    };
    let clouded = scene.atmosphere.clouds.filter(|_| {
        matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        )
    });
    let mut irradiance = light.color * light.intensity;
    let mut disc = 0.0;
    if let Some(c) = clouded {
        irradiance /= c.light().0;
        // As the sky draws its disc: its colour and intensity, through the
        // air under the physical sky.
        let [_, _, _, d, g] = look.gpu(true, Vec3::from_slice(&uniform.atmosphere.sky[4][1..]));
        let through = look.physical.map_or(Vec3::ONE, |p| {
            let up = direction.normalize_or(Vec3::Y).y;
            p.sun_transmittance(p.altitude + camera.position.y + scene.frame.origin.y, up)
        });
        let glow = if look.physical.is_some() { 0.0 } else { d[3] };
        disc = (Vec3::new(g[1], g[2], g[3]) * through * (d[2] + glow)).dot(LUMA);
    }
    DrawnSun {
        direction,
        radius_degrees,
        irradiance,
        disc,
    }
}

/// Rec. 709 luminance.
const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

/// The sun's mean angular radius (degrees).
const SUN_SEMI_DIAMETER: f32 = 0.2666;

/// The angular radius of the god rays' sun: the circumsolar region a
/// pyrheliometer takes in with the sun (WMO CIMO Guide, 2.5° either side),
/// the bright part of the sky the shafts are cut from.
const GOD_RAYS_SUN_RADIUS: f32 = 2.5;
/// The share of the sun's irradiance the god rays' sun carries, so
/// `GodRays::intensity` keeps the strength it was tuned to on the drawn
/// disc (1.8° across its radius, read at most 32 a texel): `god-rays`, at
/// its threshold of 10, carries about the light its shafts carried then.
const GOD_RAYS_SUN_SHARE: f32 = 1.0 / 60.0;

/// `FxPostUniform::god_sun` (fxpost.wgsl `fs_godrays_mask`) for `sun` seen
/// through `proj` on a frame `height` pixels high: row 0 the god rays' sun's
/// radiance at its middle (rgb, unexposed: its share of the irradiance
/// spread over its (1 - x²)² profile) and the drawn disc's luminance there
/// (w, 0: not read off the frame), row 1 its radius and the drawn disc's
/// (height units; y 0: no sun of its own, the mask reads the sky alone, as
/// for an environment map's sun or an orthographic camera). The rays come
/// from the sun's light, whatever size its disc is drawn: read off the
/// frame, a disc a few pixels wide under a cap of 32 a texel cast none.
fn god_sun(sun: &DrawnSun, proj: glam::Mat4, height: u32) -> [[f32; 4]; 2] {
    let focal = proj.y_axis.y;
    if sun.radius_degrees <= 0.0 || proj.w_axis.w != 0.0 || focal <= 0.0 {
        return [[0.0; 4]; 2];
    }
    let to_height = |degrees: f32| degrees.to_radians().tan() * focal * 0.5;
    let radius = GOD_RAYS_SUN_RADIUS.to_radians();
    let peak = sun.irradiance * GOD_RAYS_SUN_SHARE / (std::f32::consts::PI * radius * radius / 3.0);
    // A disc under a pixel is drawn a pixel wide, its light spread over it
    // (sky.wgsl sky_physical_look).
    let pixel = 2.0 / (focal * height.max(1) as f32);
    let disc = sun.radius_degrees.to_radians();
    let cover = (disc / disc.max(pixel)).powi(2);
    // The sun's own radiance, its irradiance over its own disc: what the
    // threshold judges it by.
    let own = SUN_SEMI_DIAMETER.to_radians();
    let radiance = sun.irradiance.dot(LUMA) / (std::f32::consts::PI * own * own);
    [
        peak.extend(sun.disc * cover).to_array(),
        [
            to_height(GOD_RAYS_SUN_RADIUS),
            to_height(sun.radius_degrees),
            radiance,
            0.0,
        ],
    ]
}
