//! Frame readback, pipeline warming and temporal state.

use super::*;

/// The frames [`Renderer::render_capture`] draws of one scene where GI or
/// screen-space reflections accumulate without TAA (under TAA it draws one
/// cycle of the jitter, 8 to 32 frames): what their history reaches on a
/// still frame.
pub const CAPTURE_WARMUP: u64 = 8;

/// The systems a scene has, as [`Renderer::warm`] reads them: it makes the
/// pipelines of these only. Set by each [`Renderer::render`].
#[derive(Clone, Copy, Default)]
pub(crate) struct Reach {
    /// Meshes, skinned meshes or crowds: the lit and shadow pipelines, the
    /// sky and the environment's light.
    pub world: bool,
    pub sprites: bool,
    pub lights_2d: bool,
    pub ribbons: bool,
    pub particles: bool,
    pub water: bool,
    pub terrain: bool,
    pub scatter: bool,
    pub metaballs: bool,
    pub fluids: bool,
    pub lines: bool,
    /// Screen-space sprites, text or glass over the frame: the overlay's
    /// pipelines.
    pub overlay: bool,
    /// og-ui's old core's shapes, vectors, layers or batches: its UI
    /// pipelines (og-ui-next paints with the paint pass, made at boot).
    pub old_ui: bool,
}

impl Reach {
    pub(crate) fn of(scene: &RenderScene) -> Reach {
        let (o, w, s, e) = (&scene.objects, &scene.world, &scene.sprite, &scene.effects);
        Reach {
            world: !o.instances.is_empty()
                || !o.skinned_instances.is_empty()
                || !o.crowds.is_empty()
                || !w.terrains.is_empty(),
            sprites: !s.sprites.is_empty()
                || !s.texts.is_empty()
                || !s.tile_layers.is_empty()
                || !s.sprite_arrays.is_empty()
                || !s.sprite_meshes.is_empty(),
            lights_2d: !s.lights_2d.is_empty() || s.lighting_2d.is_some(),
            ribbons: !e.ribbons.is_empty() || !e.trails.is_empty() || !e.beams.is_empty(),
            particles: !e.particles.is_empty() || !e.hosted_particles.is_empty(),
            water: !w.water.is_empty(),
            terrain: !w.terrains.is_empty(),
            scatter: !w.scatters.is_empty() || !w.grass.is_empty(),
            metaballs: !s.metaballs.is_empty(),
            fluids: !w.fluids_3d.is_empty(),
            lines: !e.lines.is_empty() || !e.points.is_empty(),
            overlay: scene.ui.overlay.is_some(),
            old_ui: scene.ui.overlay.as_ref().is_some_and(|o| !o.ui.is_empty()),
        }
    }
}

impl Renderer {
    /// Reads the next frame's GPU counts back: the culling's (`culled`,
    /// `occluded`, `triangles`, `shadow_casters` and the triangle counts
    /// in and out of view; see [`FrameStats::counts_lag`]) and the
    /// scatter's ([`Renderer::scatter_counts`]), for one frame. A frame
    /// rendered without a request reads none back, so the counts stay on
    /// the GPU until someone asks. Natively [`Renderer::finish_stats`]
    /// needs no request: it reads the last frame's at once.
    pub fn request_culling_counts(&mut self) {
        self.visibility.culling_counts = true;
    }

    /// What the main camera's cull made of each drawable of the last frame
    /// drawn (`visibility/cull_record.rs`), written by the culls that
    /// decided: the GPU cull's camera pass (with occlusion), the CPU cull
    /// on the GL tier, the transparent sort, the skinned cull and the
    /// sprites' build. Natively it waits for the GPU's part; on the web it
    /// asks for the next frame's and answers the newest whole record.
    /// `None` in a publish build, or before a frame was drawn.
    pub fn cull_record(&mut self) -> Option<&crate::CullRecord> {
        let pending = self.visibility.records.as_ref()?.pending();
        if let (Some(frame), Some(gc)) = (pending, self.visibility.gpu_cull.as_mut()) {
            #[cfg(not(target_arch = "wasm32"))]
            gc.finish(&self.device, &self.queue, frame);
            #[cfg(target_arch = "wasm32")]
            {
                let _ = frame;
                self.visibility.culling_counts = true;
                gc.poll_counts();
            }
            if let Some((f, words)) = gc.take_seen()
                && let Some(records) = self.visibility.records.as_mut()
            {
                records.landed(f, &words);
            }
        }
        self.visibility.records.as_ref()?.newest()
    }

    /// The last frame drawn, when its cull record's GPU words were copied
    /// for a later [`Self::cull_record`] (the web's): the caller keeps
    /// what it needs of that frame (which entity each drawable is) until
    /// the record answers it.
    pub fn cull_asked(&self) -> Option<u64> {
        self.visibility.records.as_ref()?.asked()
    }

    /// Makes one pipeline that the last scene rendered can reach and no
    /// frame has drawn with yet: of the systems that scene has
    /// ([`Reach`]), the variants a frame did not draw (the full lit
    /// variants a feature turned on later draws with, the plain lit and
    /// shadow pipelines, the sky's cloud passes, the water's and the
    /// terrain's full variants, particles' simulation), the UI's, the
    /// overlay's and the post passes' (TAA's after a cut, the flare) first. False when none is left. A system the scene does not
    /// have gets none: a pipeline a frame draws is made by that frame
    /// (deferred.rs), so this is only what a later frame may reach. A host
    /// with idle time (the web page behind its play gate, og.render.compile)
    /// calls it until it answers false. The frames are the same either way.
    pub fn warm(&mut self) -> bool {
        let r = self.reach;
        let world = r.world;
        (r.sprites
            && deferred::warm_one(&[
                &self.sprite.sprite_pipeline,
                &self.sprite.msdf_pipeline,
                &self.sprite.sprite_cover_pipeline,
                &self.sprite.plate_pipeline,
                &self.sprite.sprite_untonemapped_pipeline,
                &self.sprite.sprite_additive_pipeline,
                &self.sprite.sprite_glow_pipeline,
            ]))
            || (r.overlay && self.overlay.warm())
            || (r.old_ui && self.ui.warm())
            || self.glass.warm()
            || self.post.warm()
            || (world
                && deferred::warm_one(&[
                    &self.sky.sky_pipeline,
                    &self.sky.sky_look_pipeline,
                    &self.sky.clouds_low.sky,
                    &self.sky.clouds_low.march,
                    &self.sky.clouds_low.march_depth,
                    &self.sky.clouds_low.resolve,
                ]))
            || (world && self.lights.ibl.warm())
            || (world && self.warm_plain_lit())
            || self.warm_full_lit()
            || (r.ribbons
                && deferred::warm_one(&[
                    &self.strips.ribbon_pipelines[0],
                    &self.strips.ribbon_pipelines[1],
                ]))
            || (r.ribbons
                && self.strips.gpu_trails.as_ref().is_some_and(|g| {
                    deferred::warm_one(&[&g.pipeline, &g.pipelines[0], &g.pipelines[1]])
                }))
            || (r.particles && self.particles.gpu.warm())
            || self.particles.gpu.warm_loaded(&self.device, self.samples)
            || self.warm_loaded_oit()
            || self.warm_loaded_materials()
            || (r.lights_2d
                && self
                    .lights_2d
                    .warm(self.info.caps.contains(Caps::MRT | Caps::INDEPENDENT_BLEND)))
            || (r.water && self.world.water.get().is_some_and(|w| w.warm()))
            || (r.terrain && self.world.terrain.get().is_some_and(|t| t.warm()))
            || (r.scatter && self.world.scatter.get().is_some_and(|s| s.warm()))
            || (r.metaballs && self.world.metaballs.warm())
            || (r.fluids && self.world.fluid3d.get().is_some_and(|f| f.warm()))
            || (r.lines && self.lines.warm())
    }

    /// The OIT pipeline of the alpha particles a loaded emitter draws
    /// ([`crate::particles::particles_gpu::ParticleRenderer::loaded_oit`]),
    /// in the mode a frame of this size runs: their first frame then
    /// draws without compiling it. True when this call asked for it.
    fn warm_loaded_oit(&mut self) -> bool {
        if !self.particles.gpu.loaded_oit()
            || !self.info.caps.contains(Caps::MRT | Caps::INDEPENDENT_BLEND)
        {
            return false;
        }
        let mode = self
            .oit
            .frame_mode(&self.device, &self.queue, self.width, self.height);
        let modes: &[crate::oit::Oit] = if mode == crate::oit::Oit::Front {
            &[crate::oit::Oit::Front, crate::oit::Oit::Mark]
        } else {
            &[mode]
        };
        let samples = self.samples;
        let src = crate::oit::Source::Particles(crate::oit::ParticleOit::Billboard, false, None);
        let missing = |r: &Self| {
            modes
                .iter()
                .any(|&m| !r.oit.pipelines.contains_key(&(src, m, samples)))
        };
        if !missing(self) {
            return false;
        }
        self.ensure_oit_pipelines(mode, &[], &[], &[], &[src], &[], &[]);
        true
    }

    /// The full variants of the lit, depth and OIT pipelines that a frame
    /// has drawn with a specialization only (pipeline_features.rs): made
    /// here, a feature that turns on later draws without another compile.
    fn warm_full_lit(&mut self) -> bool {
        use crate::deferred::Warm;
        let lit = &self.lit;
        lit.pbr_pipelines
            .values()
            .chain(lit.lean_pipelines.values())
            .chain(self.prepass.prepass_pipelines.values())
            .chain(self.prepass.lean_prepass_pipelines.values())
            .chain(self.oit.pipelines.values())
            .any(|v| v.full().warm())
    }

    /// The plain material's lit and shadow pipelines (front faces, opaque),
    /// which batches otherwise make when one first draws; true when this
    /// call made one.
    fn warm_plain_lit(&mut self) -> bool {
        // With the effect shader when the frame puts it on every batch:
        // the variant without it would be made for nothing.
        let key = (
            Side::Front.into(),
            false,
            false,
            self.lit.plain_effects,
            false,
        );
        if !self.lit.pbr_pipelines.contains_key(&key) {
            let p = pbr_pipeline(
                &self.device,
                (&self.lit.pbr_layout, &self.lit.pbr_skinned_layout),
                &self.lit.pbr_shader,
                key,
                self.samples,
            );
            let declared = self.lit.pbr_shader.declared;
            self.lit.pbr_pipelines.insert(
                key,
                crate::pipeline_features::Variants::new(declared, move |_| {
                    let p = p.clone();
                    Deferred::new(move || p.clone())
                }),
            );
            return true;
        }
        let key = (Side::Front.into(), false, false);
        if !self.shadows.shadow_pipelines.contains_key(&key) {
            let p = shadow_pipeline(
                &self.device,
                &self.shadows.shadow_layouts,
                &self.shadows.shadow_shader,
                key,
                false,
                false,
            );
            self.shadows.shadow_pipelines.insert(key, {
                let p = p.clone();
                Deferred::new(move || p.clone())
            });
            return true;
        }
        false
    }

    /// Natively: waits for the GPU culling counts of the frame `stats` came
    /// from and puts them in place of the older ones it was given
    /// ([`FrameStats::counts_lag`] becomes 0); for the last frame rendered
    /// they are copied now when it copied none. Whether it could: false on
    /// the CPU path, or for a frame that found no readback free and was
    /// not the last one rendered.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn finish_stats(&mut self, stats: &mut FrameStats) -> bool {
        if let Some(s) = self.world.scatter.get_mut() {
            s.finish_counts(&self.device, &self.queue);
        }
        if !stats.gpu_culling {
            return false;
        }
        // Already the frame's own (a second read of the same frame).
        if stats.counts_lag == 0 {
            return true;
        }
        let Some((exact, given)) = self
            .visibility
            .gpu_cull
            .as_mut()
            .and_then(|gc| gc.finish(&self.device, &self.queue, stats.frame))
        else {
            return false;
        };
        stats.culled = stats.culled - given.culled + exact.culled;
        stats.occluded = exact.occluded;
        stats.occluded_triangles = exact.occluded_triangles;
        stats.in_view_triangles = exact.triangles + exact.occluded_triangles;
        stats.triangles = stats.triangles - given.triangles + exact.triangles;
        stats.shadow_casters = stats.shadow_casters - given.shadow_casters + exact.shadow_casters;
        stats.counts_lag = 0;
        true
    }

    /// A copy of a particle system's slots after the last frame (the GPU
    /// path reads its buffer back): for tests and tools (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_particles(&self, id: u64) -> Option<Vec<ParticleState>> {
        self.particles.gpu.read(&self.device, &self.queue, id)
    }

    /// Whether `target`, not sRGB itself, has an sRGB view the overlay
    /// blends in ([`linear_view_format`]).
    pub(crate) fn has_linear_view(&self, target: &RenderTarget) -> bool {
        match target {
            RenderTarget::Offscreen => self.persistent.offscreen_linear.is_some(),
            RenderTarget::Surface => self
                .surface
                .as_ref()
                .is_some_and(|(_, config)| config.view_formats.iter().any(|f| f.is_srgb())),
            RenderTarget::View(_) => false,
        }
    }

    /// Whether the paint pass draws a layer of its own for `target`: the
    /// output holds no encoded values, and `target` or the glass source,
    /// which a frame may go through, has no plain view
    /// ([`encoded_view_format`]).
    pub(crate) fn paint_layered(&self, target: &RenderTarget) -> bool {
        let plain = self.offscreen_encoded.is_some()
            && match target {
                RenderTarget::Offscreen => true,
                RenderTarget::Surface => self
                    .surface
                    .as_ref()
                    .is_some_and(|(_, config)| !config.view_formats.is_empty()),
                RenderTarget::View(_) => false,
            };
        ui::paint::encoded_format(self.output_format) != self.output_format && !plain
    }

    /// Reads back the capture of reflection probe `id` (blocking): `None`
    /// before its first capture (or for a probe sampling a baked map),
    /// else its face size and [`ENVIRONMENT_MAP_LEVELS`] prefiltered
    /// levels, each six faces of linear RGB floats (`og bake probes`).
    pub fn read_reflection_probe(&self, id: u64) -> Option<Result<ProbeCapture, String>> {
        let slot = self
            .probes
            .slots
            .iter()
            .find(|s| s.id == id && s.captured.is_some())?;
        Some(probes::read_back(&self.device, &self.queue, slot).map(|l| (slot.size, l)))
    }

    /// The linked lists' fragment counts of the last frame (blocking;
    /// tests): `None` when the frame drew no linked lists
    /// ([`RenderInfo::transparency`], [`FrameStats::transparency_layers`]).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_transparency_counts(&self) -> Option<Result<crate::TransparencyCounts, String>> {
        let arena = self
            .oit
            .arena
            .as_ref()
            .filter(|a| a.size.nodes_per_pixel > 0)?;
        Some(arena.read_counts(&self.device, &self.queue))
    }

    /// The main camera's scattered copies, by the scene's scatter, as the
    /// GPU cull counted them a frame or two ago ([`crate::ScatterCounts`]):
    /// `None` without the GPU cull (no compute), or before its first
    /// counts land.
    pub fn scatter_counts(&self) -> Option<&crate::ScatterCounts> {
        self.world.scatter.get()?.counts()
    }

    /// Forgets the TAA history: the next frame starts from its own samples
    /// (call it on a camera cut).
    pub fn reset_history(&mut self) {
        self.history.taa_history = None;
        self.gi_history = None;
        self.ssr.reset();
        self.sky.clouds_low.temporal.reset();
    }

    /// Draws the capture of `scene` into `target`: a pure function of the
    /// scene (docs/design.md §2.4, presentation), whatever frames were drawn
    /// before it, so a capture is the same after every frame from tick 0,
    /// after ticks stepped without frames and after a restore. The frame
    /// history starts afresh (TAA's and the upscaler's history, the GI and
    /// reflections' accumulation, the previous frame's camera, time, wind
    /// and sway, the phase that turns the jitter, the shadow filters'
    /// spiral and the screen noise: [`Renderer::phase`]), auto exposure meters this
    /// frame at once, and frames of the same scene are drawn, the last kept:
    /// under TAA one cycle of its jitter, which converges a still frame;
    /// [`CAPTURE_WARMUP`] where GI or reflections accumulate without it;
    /// one otherwise. Afterwards the display's frame history is put back
    /// (`history.rs`), so the frames the display draws next go on from the
    /// frame before the capture, as if it had not been taken.
    pub fn render_capture(
        &mut self,
        scene: &RenderScene,
        target: &RenderTarget,
    ) -> Result<FrameStats, RenderError> {
        let kept = self.take_frames();
        self.forget_frames();
        self.phase_base = self.frame_index;
        self.post.forget_exposure();
        let warmup = if self.info.antialias.temporal() {
            frame::jitter_phases((self.width, self.height), self.history_size())
        } else if self.frame_ssr(scene).is_some()
            || scene.look.post.gi.is_some_and(|g| g.intensity > 0.0)
        {
            CAPTURE_WARMUP
        } else {
            1
        };
        if let Some(t) = self.world.terrain.get_mut() {
            t.settled = true;
        }
        self.capturing = true;
        let mut stats = self.render(scene, target);
        for _ in 1..warmup {
            stats = self.render(scene, target);
        }
        self.capturing = false;
        if let Some(t) = self.world.terrain.get_mut() {
            t.settled = false;
        }
        self.put_frames(&kept);
        stats
    }

    /// The probe's capture of `scene` at `width`×`height` (0×0: the
    /// output's size), drawn offscreen and its readback started: as the
    /// next frame after the display's, with its history and at the render
    /// scale the frames draw at, or as a reference
    /// ([`crate::SceneFrame::reference`]: [`Renderer::render_capture`] at
    /// scale 1). The output's size and the render scale are put back
    /// before it returns. Every texture it draws is uploaded, past the
    /// per-frame budget ([`crate::set_upload_budget`]): a renderer made
    /// again after a lost device would otherwise capture its first frames'
    /// stand-ins. The one capture every host takes: natively the
    /// host waits for the request ([`crate::CaptureRequest::wait`]), the
    /// web polls it on later calls ([`crate::CaptureRequest::try_finish`]).
    pub fn capture_frame(
        &mut self,
        scene: &RenderScene,
        width: u32,
        height: u32,
    ) -> Result<(FrameStats, crate::CaptureRequest), RenderError> {
        let target = (self.out_width, self.out_height);
        let size = if width == 0 || height == 0 {
            target
        } else {
            (width, height)
        };
        if size != target {
            self.resize(size.0, size.1);
        }
        let scale = self.render_scale;
        self.uploads_whole = true;
        let drawn = if scene.frame.reference {
            self.render_scale = 1.0;
            self.render_capture(scene, &RenderTarget::Offscreen)
        } else {
            self.render(scene, &RenderTarget::Offscreen)
        };
        self.render_scale = scale;
        self.uploads_whole = false;
        let captured = drawn.and_then(|stats| {
            self.wait_frames()?;
            Ok((stats, self.begin_capture()?))
        });
        if size != target {
            self.resize(target.0, target.1);
        }
        captured
    }

    /// The frames drawn since the last capture: what turns the TAA jitter,
    /// the shadow filters' spiral and the screen-space effects' noise from
    /// frame to frame. `frame_index` itself only grows (readbacks are
    /// stamped with it).
    pub(crate) fn phase(&self) -> u64 {
        self.frame_index - self.phase_base
    }

    /// Everything a frame carries from the frame before it, forgotten.
    pub(super) fn forget_frames(&mut self) {
        self.reset_history();
        self.history.prev_view_proj = None;
        self.history.previous_time = None;
        self.instances.previous_sway.clear();
        self.history.previous_wind = None;
    }
}
