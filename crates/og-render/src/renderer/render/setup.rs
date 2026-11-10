//! Setup stage.

use super::*;

impl Renderer {
    pub(super) fn prepare_frame_resources(&mut self, scene: &RenderScene) -> (u32, (u32, u32)) {
        self.apply_quality(&scene.quality);
        // The render origin moved (`origin.rs`): what the renderer keeps
        // from the frames before moves with it, so the move is invisible.
        if scene.origin != self.drawn_origin {
            let d = self.drawn_origin - scene.origin;
            self.prev_view_proj = self
                .prev_view_proj
                .map(|m| crate::origin::shift_matrix(m, d));
            self.taa_history = self.taa_history.map(|m| crate::origin::shift_matrix(m, d));
            self.particles.shift(d);
            self.trails.shift(d);
            if let Some(g) = &mut self.gpu_trails {
                g.shift(d);
            }
            self.drawn_origin = scene.origin;
        }
        // A look that switches the stylisation on or off (passes added or
        // removed, the pixel grid, the material style) is a cut: no ghost of
        // the old look in the first frame of the new one.
        // So is a camera cut (`RenderScene::camera_cut`): TAA keeps no
        // ghost of the previous shot. TAA is the only history the renderer
        // reprojects; motion vectors take the cut from `prev_view_proj`.
        let chain = scene.stylise.chain_bits();
        if scene.camera_cut || self.stylise_chain.is_some_and(|c| c != chain) {
            self.taa_history = None;
            self.gi_history = None;
            self.ssr.reset();
        }
        self.stylise_chain = Some(chain);
        let (grid, upscale, grid_offset) = self.pixel_grid(scene.stylise.pixel);
        // The render scale's frame (upscale 0, not a pixel grid) below the
        // output under TAA is resolved to the output by TAA itself (TAAU);
        // above it, it is supersampled and scaled down after TAA.
        let out = (self.out_width, self.out_height);
        let taau = upscale == 0 && grid != out && grid.0 <= out.0 && grid.1 <= out.1;
        self.scene_size(grid.0, grid.1, taau);
        self.warn_without(scene);
        self.grow_local_shadows(scene);
        self.fit_shelter_layer(scene);
        self.prepare_gi_probes(scene);
        // The reflections' targets (ssr.rs), before the frame groups that
        // bind them: new ones make the groups again.
        if let Some(s) = Self::frame_ssr(scene)
            && self
                .ssr
                .ensure_targets(&self.device, &s, (self.width, self.height))
        {
            self.environment = None;
        }
        self.ensure_environment(scene);
        self.write_sky_params(scene);
        (upscale, grid_offset)
    }

    /// The frame's screen-space reflections (ssr.rs): a perspective 3D
    /// camera's only, as an orthographic view reflects nothing it shows.
    pub(in crate::renderer) fn frame_ssr(
        scene: &RenderScene,
    ) -> Option<ssr::ScreenSpaceReflections> {
        let perspective = matches!(scene.camera.projection, Projection::Perspective { .. });
        scene
            .post
            .ssr
            .filter(|s| WORLD_3D && perspective && scene.quality.ssr && s.intensity > 0.0)
            .map(|s| ssr::ScreenSpaceReflections {
                steps: scene.quality.ssr_steps(s.steps),
                ..s
            })
    }

    /// Says once for each lit feature the renderer was made without
    /// (RenderDesc::without) that the scene uses it, so it is not drawn.
    fn warn_without(&mut self, scene: &RenderScene) {
        let off = self.info.without;
        if off.is_empty() {
            return;
        }
        let used = crate::pipeline_features::Features::of(scene);
        for (feature, name) in LitFeatures::NAMED {
            if off.contains(feature)
                && used.contains(feature.features())
                && !self.warned_without.contains(feature)
            {
                self.warned_without = self.warned_without | feature;
                log::warn!(
                    "og-render: the scene uses {name}, but the renderer was made without it \
                     (RenderOptions.without), so it is not drawn"
                );
            }
        }
    }

    /// Whether `light`'s kind is drawn: a renderer made without rect lights
    /// (RenderDesc::without) keeps none ([`clusters::ClusterBuilder::skip_rects`]).
    pub(crate) fn kept_kind(&self, light: &Light) -> bool {
        !(self.clusters.skip_rects && matches!(light.kind, LightKind::Rect { .. }))
    }

    /// The probe GI's frame (gi_probes.rs): off where the quality's GI
    /// switch or a 2D build says so. The probe buffer grows to hold its
    /// probes before the frame groups that bind it are made.
    fn prepare_gi_probes(&mut self, scene: &RenderScene) {
        let desc = scene
            .post
            .gi_probes
            .filter(|_| WORLD_3D && scene.quality.global_illumination)
            // A renderer made without the probe GI (RenderDesc::without).
            .filter(|_| !self.info.without.contains(LitFeatures::GI));
        let Some(desc) = desc else {
            if let Some(g) = &mut self.gi_probes {
                g.disable();
            }
            return;
        };
        let region = crate::lightmap::PROBE_CAPACITY;
        let g = self
            .gi_probes
            .get_or_insert_with(|| crate::gi_probes::GiProbes::new(&self.device));
        let need = region + crate::gi_probes::region_vec4s(&desc);
        if self
            .lightmaps
            .ensure_capacity(&self.device, &self.queue, need)
        {
            // The groups binding the buffer: the frame's and the effect
            // passes' (the screen GI reads the probes).
            self.environment = None;
            self.graph.invalidate();
            g.restart();
        }
        g.prepare(&self.device, &self.queue, scene, desc, region);
    }

    pub(super) fn prepare_frame_probes(
        &mut self,
        scene: &RenderScene,
    ) -> (probes::ProbeFrame, Vec<ReflectionProbe>, Vec<(Vec3, Vec3)>) {
        // The light volumes' probes, then the realtime volumes' due
        // relighting (probe_grid.rs), which the quality's GI switch stops.
        let realtime: &[crate::RealtimeLightVolume] = if scene.quality.global_illumination {
            &scene.light_volumes
        } else {
            &[]
        };
        let grid = self
            .lightmaps
            .prepare(&self.device, &self.queue, &scene.baked, realtime);
        // The probe GI's header rows, after the volumes' (gi_probes.rs).
        if let Some(g) = &mut self.gi_probes {
            g.write_header(&self.queue, &self.lightmaps.probe_buffer, false);
        }
        let probe_frame = self.probes.prepare(
            &self.device,
            &self.queue,
            &self.ibl,
            &scene.reflection_probes,
            scene.tick,
            self.samples,
            |h| {
                self.cache
                    .environment_maps
                    .get(h.index())
                    .is_some_and(Option::is_some)
            },
            grid,
        );
        // Reflection probes: the frame's active ones, whose sampled ones'
        // boxes key the instances' sort (probe_select.rs: a batch samples
        // one probe).
        let active_probes: Vec<ReflectionProbe> = probe_frame.active.iter().map(|a| a.0).collect();
        let probe_boxes = probe_select::boxes(&active_probes[..probe_frame.sampled]);
        self.sorted.set_probes(probe_boxes.clone());
        (probe_frame, active_probes, probe_boxes)
    }

    pub(super) fn prepare_full_meshes(&mut self, scene: &RenderScene, batches: &[Batch]) {
        // The f32 vertices of compact meshes drawn without their compact
        // ones: by a material that is not plain, skinned or scattered.
        let full: Vec<usize> = batches
            .iter()
            .filter(|b| !b.compact)
            .map(|b| b.mesh)
            .chain(scene.skinned_instances.iter().map(|s| s.mesh.index()))
            .chain(scene.scatters.iter().map(|s| s.mesh.index()))
            .collect();
        for mesh in full {
            self.ensure_full(scene, mesh);
        }
        // Water drawn over a mesh reads its f32 vertices.
        for m in scene.water.iter().filter_map(|w| w.mesh) {
            self.ensure_full(scene, m.index());
        }
    }
}
