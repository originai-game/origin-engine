//! The shadow layers and the environment a frame's scene asks for.

use super::*;

impl Renderer {
    /// Gives the cascade array one more layer, last, for the weather
    /// occlusion map while the scene's surface weather has shelter, and
    /// takes it away again when it has none, in front of it the frame's
    /// cascades ([`Self::frame_cascades`]); the frame groups are made again
    /// when the layer count changes.
    pub(crate) fn fit_shelter_layer(&mut self, scene: &RenderScene) {
        // Only the weather on surfaces reads it (RenderDesc::without).
        let shelter = !self.info.without.contains(LitFeatures::SURFACE_WEATHER)
            && scene
                .atmosphere
                .surface_weather
                .is_some_and(|w| w.shelter > 0.0);
        let wanted = self.frame_cascades(scene) + u32::from(shelter);
        if self.shadows.cascade_map.layers.len() as u32 == wanted {
            return;
        }
        self.shadows.cascade_map = ShadowMap::new(
            &self.device,
            "cascades",
            self.info.shadow_map_size,
            wanted,
            wgpu::TextureFormat::R32Float,
        );
        self.shadows.shelter_key = None;
        self.shadows.shadow_cache.forget_layers();
        self.lights.environment = None;
    }

    /// The cascades a frame of `scene` draws: its quality's count within
    /// the renderer's ([`crate::RenderInfo::shadow_cascades`]), at least one.
    pub(crate) fn frame_cascades(&self, scene: &RenderScene) -> u32 {
        scene
            .frame
            .quality
            .shadow_cascades
            .min(self.info.shadow_cascades)
            .max(1)
    }

    /// Grows the local shadow array when the scene's shadowed lights could
    /// need more layers than it has ([`shadows::grown_layers`]; the frame
    /// bind groups are made again with it). It never shrinks.
    pub(crate) fn grow_local_shadows(&mut self, scene: &RenderScene) {
        let capacity =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows);
        // Cookies take layers past every shadow layer there can be
        // (area.rs), so the array grows to the full capacity with them.
        let cookies = area::Cookies::layers(scene) as usize;
        if self.shadows.local_map.layers.len() >= capacity + cookies {
            return;
        }
        // Every shadowed light of the scene's, within the frame's budget.
        let lights = scene
            .frame
            .quality
            .shadowed_lights
            .min(self.info.shadowed_lights) as usize;
        let wanted: Vec<usize> = scene
            .lighting
            .lights
            .iter()
            .filter(|l| l.cast_shadows && self.kept_kind(l))
            .map(|l| match l.kind {
                LightKind::Point => self.info.point_shadows.layers(),
                LightKind::Spot { .. } => 1,
                LightKind::Rect { two_sided, .. } => 1 + two_sided as usize,
            })
            .collect();
        let needed = if cookies > 0 {
            capacity + cookies
        } else {
            shadows::grown_layers(&wanted, lights, self.info.point_shadows, capacity)
        };
        if needed <= self.shadows.local_map.layers.len() {
            return;
        }
        self.shadows.local_map = ShadowMap::new(
            &self.device,
            "local shadows",
            self.shadows.local_shadow_size,
            needed as u32,
            LOCAL_SHADOW_FORMAT,
        );
        self.shadows.shadow_cache.forget_layers();
        self.lights.environment = None;
    }

    /// (Re)generates the image-based lighting when the sky, or the
    /// environment map that replaces it, changed.
    /// The environment map a frame draws with, when it is uploaded, and the
    /// map it blends towards (its index and drawn weight) while that is
    /// uploaded too and the weight moves off the first
    /// (`EnvironmentBlend::drawn_weight`).
    fn drawn_environment(
        &self,
        scene: &RenderScene,
    ) -> Option<(crate::EnvironmentMap, Option<(usize, f32)>)> {
        let live = |i: usize| {
            self.cache
                .environment_maps
                .get(i)
                .is_some_and(Option::is_some)
        };
        let map = scene
            .lighting
            .environment_map
            .filter(|m| live(m.map.index()))?;
        let blend = map
            .blend
            .filter(|b| live(b.to.index()))
            .map(|b| (b.to.index(), b.drawn_weight()))
            .filter(|b| b.1 > 0.0);
        Some((map, blend))
    }

    /// The sky pass's uniform for this frame: the background's blurriness
    /// and the map it draws (`SkyParams`).
    pub(crate) fn write_sky_params(&self, scene: &RenderScene) {
        let map = self.drawn_environment(scene).map(|(m, blend)| SkyMap {
            rotation: m.rotation,
            scale: m.background_intensity.max(0.0),
            blend: blend.map_or(0.0, |b| b.1),
        });
        let params = SkyParams::new(&scene.lighting.environment, crate::ibl::PREFILTER_MIPS, map);
        self.queue
            .write_buffer(&self.sky.sky_params, 0, bytemuck::bytes_of(&params));
    }

    pub(crate) fn ensure_environment(&mut self, scene: &RenderScene) {
        let sky = &scene.lighting.environment.sky;
        let drawn = self.drawn_environment(scene);
        let map = drawn.map(|d| d.0);
        let blend = drawn.and_then(|d| d.1);
        // The per-pixel sky without an environment map is live: captured
        // as it is drawn, a slice a frame (sky_capture.rs).
        let live = map.is_none()
            && matches!(
                scene.lighting.environment.background,
                Background::Procedural { .. }
            );
        if !live {
            self.sky.sky_capture = None;
        }
        let key = match map {
            None if live => EnvironmentKey::Live,
            Some(m) => EnvironmentKey::Map {
                scene: scene.id(),
                map: m.map.index(),
                revision: self.cache.environment_map_revisions[m.map.index()],
                rotation: m.rotation.to_bits(),
                background: m.background_intensity.to_bits(),
                blend: blend
                    .map(|(i, w)| (i, self.cache.environment_map_revisions[i], w.to_bits())),
            },
            None => EnvironmentKey::Sky(*sky),
        };
        // The first frame with clouds draws their noise and makes the sky
        // group again with it.
        if scene.atmosphere.clouds.is_some() && !self.sky.cloud_noise.made {
            self.sky.cloud_noise.make(&self.device, &self.queue);
            self.lights.environment = None;
        }
        // The aurora's footprint follows the clock every frame; the first
        // frame with one makes its texture, and the sky group again. It is
        // drawn by compute: without it the sky keeps the empty footprint.
        if let Some(a) = &scene.atmosphere.aurora
            && self.info.caps.contains(crate::Caps::COMPUTE)
        {
            let made = self.sky.aurora_map.made();
            let viewer = scene.frame.camera.position + scene.frame.origin;
            let rate = if scene.frame.tick_rate > 0.0 {
                scene.frame.tick_rate
            } else {
                60.0
            };
            self.sky.aurora_map.update(
                &self.device,
                &self.queue,
                a,
                scene.frame.tick as f64 / f64::from(rate),
                glam::Vec2::new(viewer.x, viewer.z),
            );
            if !made {
                self.lights.environment = None;
            }
        }
        if self
            .lights
            .environment
            .as_ref()
            .is_some_and(|(k, ..)| *k == key)
        {
            return;
        }
        let view = |i: usize| {
            &self.cache.environment_maps[i]
                .as_ref()
                .expect("filtered above")
                .view
        };
        match map {
            Some(m) => {
                let first = &scene.environment_maps()[m.map.index()].irradiance;
                let irradiance = match blend {
                    Some((i, w)) => environment::mix_irradiance(
                        first,
                        &scene.environment_maps()[i].irradiance,
                        w,
                    ),
                    None => *first,
                };
                self.lights.ibl.generate_map(
                    &self.device,
                    &self.queue,
                    view(m.map.index()),
                    blend.map(|(i, w)| (view(i), w)),
                    &irradiance,
                    m.rotation,
                    m.background_intensity.max(0.0),
                )
            }
            None => self.lights.ibl.generate(
                &self.device,
                &self.queue,
                sky.sun_direction,
                sky.sun_color,
                sky.look.as_ref(),
            ),
        }
        // The bound cubes are the sky without its clouds again: the live
        // sky captures it whole the next frame.
        if let Some(c) = &mut self.sky.sky_capture {
            c.restart();
        }
        let ibl = self.lights.ibl.views();
        let entries = self.frame_entries(FrameGroupParts {
            uniform: self.lights.frame_buffer.as_entire_binding(),
            clusters: [&self.lights.light_buffers[1], &self.lights.light_buffers[2]],
            cubes: [&ibl.irradiance, &ibl.prefiltered],
            // The main camera's reflections (ssr.rs; `Frame.ssr` switches
            // them, off in the planar reflections' uniforms).
            ssr: self.ssr.frame_view(),
        });
        let group = bind_group(&self.device, "frame", &self.frame_layout, &entries);
        let reflection_group = (0..self.reflections.reflection_buffer.len()).map(|k| {
            let clusters = &self.reflections.reflection_light_buffers[k];
            let reflected = self.frame_entries(FrameGroupParts {
                uniform: self.reflections.reflection_buffer[k].as_entire_binding(),
                clusters: [&clusters[1], &clusters[2]],
                cubes: [&ibl.irradiance, &ibl.prefiltered],
                ssr: self.ssr.frame_view(),
            });
            bind_group(
                &self.device,
                "frame (planar reflection)",
                &self.frame_layout,
                &reflected,
            )
        });
        let reflection_group = reflection_group.collect();
        let sky_group = bind_group(
            &self.device,
            "sky",
            &self.sky.sky_layout,
            &[
                wgpu::BindingResource::TextureView(&ibl.sky),
                wgpu::BindingResource::TextureView(&self.sky.cloud_noise.view),
                wgpu::BindingResource::Sampler(&self.sky.cloud_noise.sampler),
                self.sky.sky_params.as_entire_binding(),
                wgpu::BindingResource::TextureView(map.map_or(&ibl.sky, |m| view(m.map.index()))),
                wgpu::BindingResource::TextureView(blend.map_or(&ibl.sky, |b| view(b.0))),
                wgpu::BindingResource::TextureView(&self.sky.aurora_map.view),
            ],
        );
        self.lights.environment = Some((key, group, sky_group, reflection_group));
        if live && self.sky.sky_capture.is_none() {
            self.sky.sky_capture = Some(crate::sky_capture::SkyCapture::new(
                &self.device,
                &self.queue,
                &self.lights.ibl,
                [&self.frame_layout, &self.sky.sky_layout],
                self.info.without.features(),
            ));
        }
    }
}
