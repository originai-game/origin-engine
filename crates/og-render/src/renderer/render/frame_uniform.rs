//! The frame uniform and its copies: the camera's, the shadow passes' slots
//! (cascades, local faces, the weather's shelter map), the live sky's and
//! the due probe faces', and each planar reflection's, written in that
//! order.

use super::*;

pub(super) struct WriteFrameUniformsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub light: &'a crate::DirectionalLight,
    pub to_light: Vec3,
    pub shadow_size: u32,
    pub taa: bool,
    pub frame_fog: &'a Option<atmosphere::Fog>,
    pub scene_water: &'a [Water],
    pub lights_gpu: &'a [frame::LightGpu],
    pub cascades: &'a [Cascade],
    pub local_shadows: &'a [frame::LocalShadow],
    pub shelter: &'a Option<(Mat4, Frustum, u64, bool)>,
    pub probe_frame: &'a probes::ProbeFrame,
    pub active_probes: &'a [probes::ReflectionProbe],
    pub probe_faces: &'a [[frame::CameraFrame; 6]],
    pub reflections: &'a [camera::PlanarReflection],
    pub ssr: Option<&'a ssr::ScreenSpaceReflections>,
}

pub(super) struct WriteFrameUniformsOutputs {
    pub uniform: frame::FrameUniform,
    /// The frame's clock row (`atmosphere.time`).
    pub time: [f32; 4],
}

impl Renderer {
    /// Builds and writes the frame uniform, its shadow slots and its per-view copies.
    #[allow(clippy::too_many_lines)]
    pub(super) fn write_frame_uniforms(
        &mut self,
        WriteFrameUniformsInputs {
            scene,
            camera,
            cam,
            light,
            to_light,
            shadow_size,
            taa,
            frame_fog,
            scene_water,
            lights_gpu,
            cascades,
            local_shadows,
            shelter,
            probe_frame,
            active_probes,
            probe_faces,
            reflections,
            ssr,
        }: WriteFrameUniformsInputs<'_>,
    ) -> WriteFrameUniformsOutputs {
        // Uniforms.
        let mut uniform = frame::FrameUniform {
            view_proj: cam.jittered_view_proj.to_cols_array_2d(),
            inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
            view_pos: camera
                .position
                .extend(if cam.orthographic { 1.0 } else { 0.0 })
                .to_array(),
            light_dir: to_light.extend(cascades.len() as f32).to_array(),
            light_color: (light.color * light.intensity)
                .extend(scene.frame.debug.lit_code())
                .to_array(),
            params: [
                scene.lighting.environment.intensity,
                (ibl::PREFILTER_MIPS - 1) as f32,
                lights_gpu.len() as f32,
                1.0 / shadow_size as f32,
            ],
            view_forward: cam
                .forward
                .extend(1.0 / self.shadows.local_shadow_size as f32)
                .to_array(),
            cascade_view_proj: [[[0.0; 4]; 4]; CASCADES],
            cascade_bias: [[0.0; 4]; CASCADES],
            cluster: self.lights.clusters.params,
            local_view_proj: [[[0.0; 4]; 4]; LOCAL_SHADOW_LAYERS],
            atmosphere: atmosphere::AtmosphereGpu::new(
                frame_fog.as_ref(),
                match scene.lighting.environment.background {
                    Background::Procedural {
                        sun_direction,
                        sun_color,
                    } => Some((
                        scene
                            .lighting
                            .environment
                            .sky
                            .look
                            .as_ref()
                            .unwrap_or(&SkyLook::DEFAULT),
                        sun_direction,
                        sun_color,
                    )),
                    _ => None,
                },
                scene
                    .atmosphere
                    .clouds
                    .as_ref()
                    .map(|c| (c, scene.sky_captured())),
                scene.atmosphere.aurora.as_ref(),
                scene.atmosphere.surface_weather.as_ref(),
                scene.frame.tick,
                scene.frame.tick_rate,
            ),
            caustics: water::caustics(scene_water, scene.frame.origin, scene.frame.camera.position),
            shadow: {
                let (softness, fill) = light.shadow_softness_and_fill();
                [
                    if cascades.is_empty() { 0.0 } else { softness },
                    (self.info.point_shadows == PointShadows::Paraboloid) as u32 as f32,
                    fill,
                    // The shadow filters' spiral turns per drawn frame
                    // under TAA (lighting.wgsl shadow_turn).
                    if taa {
                        1.0 + (self.phase() % 16) as f32
                    } else {
                        0.0
                    },
                ]
            },
            stylise: crate::Retro3d::uniform(scene.look.stylise.retro, self.width, self.height),
            shadow_tint: {
                let m = &scene.look.stylise.material;
                let a = m.shadow_tint_amount.clamp(0.0, 1.0);
                [m.shadow_tint[0], m.shadow_tint[1], m.shadow_tint[2], a]
            },
            outline_style: [
                scene.look.stylise.material.outline_distance.max(0.0),
                scene.look.stylise.material.shadow_darkness.clamp(0.0, 1.0),
                // The terrain's toon (terrain_lit.wgsl): it has no material to carry it.
                f32::from(u8::from(scene.look.stylise.material.toon)),
                0.0,
            ],
            probe: [[0.0; 4]; 3],
            view_right: cam.axes().0,
            view_up: cam.axes().1,
            hemi_sky: {
                let (s, _, i) = scene.lighting.hemisphere;
                // Irradiance in the IBL's units (the lit shaders add it beside the environment's
                // irradiance, at its own strength: an environment at intensity 0 leaves it).
                [s[0], s[1], s[2], i.max(0.0) / std::f32::consts::PI]
            },
            hemi_ground: {
                let g = scene.lighting.hemisphere.1;
                [g[0], g[1], g[2], 0.0]
            },
            material_surface: scene.look.stylise.material.surface_uniform(),
            origin: scene.frame.origin.extend(0.0).to_array(),
            env_dome: crate::GroundProjection::gpu(scene.lighting.environment.ground),
            foliage_wind: crate::foliage::foliage_winds(
                scene,
                self.history.previous_wind,
                self.world.wind_field.grid(),
            )
            .1,
            toon: [
                scene.look.stylise.material.toon_softness.clamp(0.0, 4.0),
                scene.look.stylise.material.toon_ambient.clamp(0.0, 1.0),
                0.0,
                0.0,
            ],
            ssr: ssr::ScreenSpaceReflections::frame_row(ssr),
            // The stamp a material's shader parameter change carries in
            // this frame (`sync_assets`; engine s100).
            drawn: [self.frame_index as u32, 0, 0, 0],
        };
        uniform.atmosphere.time[3] = self
            .history
            .previous_time
            .unwrap_or(uniform.atmosphere.time[0]);
        // The shadow passes' clock, and the scene's origin for the hooks
        // (`h.origin`, `origin.rs`).
        let t = uniform.atmosphere.time;
        let shadow_time = [
            t[0],
            scene.frame.origin.x,
            scene.frame.origin.y,
            scene.frame.origin.z,
        ];
        let mut shadow_slots = vec![bytemuck::Zeroable::zeroed(); SHADOW_SLOTS];
        for (i, c) in cascades.iter().enumerate() {
            uniform.cascade_view_proj[i] = c.view_proj.to_cols_array_2d();
            // w < 0: a fixed shadow box (one cascade: w, the blend start, is unread).
            let blend = if c.fixed_box { -1.0 } else { c.blend_start };
            uniform.cascade_bias[i] = [c.normal_offset, c.depth_bias, c.split_far, blend];
            shadow_slots[i] = ShadowPassUniform {
                view_proj: c.view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                time: shadow_time,
                paraboloid: [0.0; 4],
                foliage_wind: uniform.foliage_wind[0],
            };
        }
        for ls in local_shadows {
            for (face, (vp, _)) in ls.faces.iter().enumerate() {
                let layer = ls.layer + face;
                uniform.local_view_proj[layer] = vp.to_cols_array_2d();
                shadow_slots[CASCADES + layer] = ShadowPassUniform {
                    view_proj: vp.to_cols_array_2d(),
                    light: ls.position.extend(ls.range).to_array(),
                    time: shadow_time,
                    paraboloid: local_paraboloid(
                        ls.paraboloid,
                        face,
                        self.shadows.local_shadow_size,
                    ),
                    foliage_wind: uniform.foliage_wind[0],
                };
            }
        }
        // The weather occlusion map: its view and its layer, the array's
        // last (never a constant: cascades or cached layers may come first).
        if let Some((view_proj, ..)) = &shelter {
            let side = self.info.shadow_map_size as f32;
            let w = scene
                .atmosphere
                .surface_weather
                .as_ref()
                .expect("filtered above");
            let texel = w.shelter.clamp(4.0, 1024.0) / side;
            uniform.atmosphere.shelter = [
                view_proj.x_axis.to_array(),
                view_proj.y_axis.to_array(),
                view_proj.z_axis.to_array(),
                view_proj.w_axis.to_array(),
                [
                    self.shadows.cascade_map.layers.len() as f32,
                    weather::DRIP_EDGE / texel,
                    side,
                    // 5 cm along the fall, in the map's depth.
                    0.05 / (2.0 * weather::SHELTER_REACH),
                ],
            ];
            shadow_slots[SHELTER_SLOT] = ShadowPassUniform {
                view_proj: view_proj.to_cols_array_2d(),
                light: [0.0; 4],
                time: shadow_time,
                paraboloid: [0.0; 4],
                foliage_wind: uniform.foliage_wind[0],
            };
        }
        self.queue
            .write_buffer(&self.lights.frame_buffer, 0, bytemuck::bytes_of(&uniform));
        // The live sky's faces due this frame see the sky as it is drawn.
        LiveSkyPass::write(&self.sky.sky_capture, &self.queue, &uniform, camera);
        let probe_positions: Vec<Vec3> = probe_frame
            .due
            .iter()
            .map(|&(_, i)| active_probes[i].position)
            .collect();
        self.probes.write(
            &self.queue,
            &uniform,
            probe_frame,
            probe_faces,
            &probe_positions,
        );
        for (k, (_, mirrored, f, view_proj, _)) in reflections.iter().enumerate() {
            let mut u = uniform;
            u.view_proj = view_proj.to_cols_array_2d();
            // The sky's view rays: the mirrored camera's own projection
            // (the oblique one tilts its far plane through the sky).
            u.inv_view_proj = f.view_proj.inverse().to_cols_array_2d();
            u.view_pos = mirrored.position.extend(0.0).to_array();
            u.view_forward = f.forward.extend(u.view_forward[3]).to_array();
            (u.view_right, u.view_up) = f.axes();
            u.caustics = [[0.0; 4]; 6];
            u.cluster = self.reflections.reflection_clusters[k].params;
            u.ssr = [0.0; 4];
            self.queue.write_buffer(
                &self.reflections.reflection_buffer[k],
                0,
                bytemuck::bytes_of(&u),
            );
        }
        self.queue.write_buffer(
            &self.shadows.shadow_buffer,
            0,
            bytemuck::cast_slice(&shadow_slots),
        );
        WriteFrameUniformsOutputs { uniform, time: t }
    }
}
