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
        // The lens flare's sources, placed with the unjittered camera, and
        // its passes' uniform; its starburst turns with the camera's
        // forward axis (Filament: the sum of the view matrix's third
        // column).
        let flare_turn = cam.view.z_axis.truncate().dot(Vec3::ONE);
        if let Some(f) = &lens_flare {
            let sources = if flare_sources_on {
                let look = scene
                    .lighting
                    .environment
                    .sky
                    .look
                    .as_ref()
                    .unwrap_or(&SkyLook::DEFAULT);
                // The drawn sun: the procedural sky's, the sky cube's when
                // it is the procedural one, else the light's direction
                // with no disc.
                let (direction, radius_degrees) = match scene.lighting.environment.background {
                    Background::Procedural { sun_direction, .. } => (sun_direction, look.sun_size),
                    Background::Sky if scene.lighting.environment_map.is_none() => {
                        (scene.lighting.environment.sky.sun_direction, look.sun_size)
                    }
                    _ => (to_light, 0.0),
                };
                let atmosphere = &uniform.atmosphere;
                // Clouds drawn over the procedural sky: the occlusion pass
                // reads the disc through them (lens_flare.wgsl), so the sun
                // is the one above them, as the sky draws its disc (its
                // colour and intensity, through the air under the physical
                // sky), undimmed by their mean cover (`Clouds::light`).
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
                    let [_, _, _, d, g] = look.gpu(true, Vec3::from_slice(&atmosphere.sky[4][1..]));
                    let through = look.physical.map_or(Vec3::ONE, |p| {
                        let up = direction.normalize_or(Vec3::Y).y;
                        p.sun_transmittance(
                            p.altitude + camera.position.y + scene.frame.origin.y,
                            up,
                        )
                    });
                    let glow = if look.physical.is_some() { 0.0 } else { d[3] };
                    disc = (Vec3::new(g[1], g[2], g[3]) * through * (d[2] + glow))
                        .dot(Vec3::new(0.2126, 0.7152, 0.0722));
                }
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
                        direction,
                        irradiance,
                        radius_degrees,
                        fog_clear: atmosphere.sky_fog_clear(camera.position, direction),
                        disc,
                    }),
                    &scene.lighting.lights,
                    &|p| atmosphere.fog_clear(camera.position, p),
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
        }
    }
}
