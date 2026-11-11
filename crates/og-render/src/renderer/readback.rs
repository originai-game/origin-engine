//! Frame readback, pipeline warming and temporal state.

use super::*;

/// The frames [`Renderer::render_capture`] draws of one scene where GI or
/// screen-space reflections accumulate without TAA (under TAA it draws one
/// cycle of the jitter, 8 to 32 frames): what their history reaches on a
/// still frame.
pub const CAPTURE_WARMUP: u64 = 8;

impl Renderer {
    /// Reads the next frame's GPU culling counts back (`culled`,
    /// `occluded`, `triangles`, `shadow_casters` and the triangle counts
    /// in and out of view; see
    /// [`FrameStats::counts_lag`]), for one frame: a frame rendered
    /// without a request copies none, so the stats keep the newest
    /// requested frame's. Natively [`Renderer::finish_stats`] needs no
    /// request for the last frame rendered.
    pub fn request_culling_counts(&mut self) {
        self.visibility.culling_counts = true;
    }

    /// Makes one pipeline no frame has drawn with yet, the likeliest to be
    /// drawn soon first (the sprites and the overlay, the post passes, then
    /// sky, the environment's light, the plain lit and shadow pipelines,
    /// ribbons, particles, 2D lights, glass, water, terrain, metaballs,
    /// fluids);
    /// false when every pipeline is made. The renderer makes a pipeline when
    /// a pass first sets it (deferred.rs), so a frame compiles only what it
    /// draws; a host with idle time (the web page behind its play gate)
    /// calls this until
    /// it answers false, so a pass met later does not stop a frame. The
    /// frames are the same either way.
    pub fn warm(&mut self) -> bool {
        // Fork s60: asynchronous on the WebGPU tier while the host asks.
        frame_pipelines(|| self.warm_now())
    }

    fn warm_now(&mut self) -> bool {
        deferred::warm_one(&[
            &self.sprite_pipeline,
            &self.msdf_pipeline,
            &self.sprite_cover_pipeline,
            &self.sprite_untonemapped_pipeline,
            &self.sprite_additive_pipeline,
            &self.sprite_glow_pipeline,
        ]) || self.overlay.warm()
            || self.post.warm()
            || deferred::warm_one(&[&self.sky_pipeline, &self.sky_look_pipeline])
            || self.lights.ibl.warm()
            || self.warm_plain_lit()
            || deferred::warm_one(&[&self.ribbon_pipelines[0], &self.ribbon_pipelines[1]])
            || self.gpu_trails.as_ref().is_some_and(|g| {
                deferred::warm_one(&[&g.pipeline, &g.pipelines[0], &g.pipelines[1]])
            })
            || self.particles.gpu.warm()
            || self.lights_2d.warm()
            || self.glass.warm()
            || (WORLD_3D && self.water.warm())
            || (WORLD_3D && self.terrain.warm())
            || self.metaballs.warm()
            || (WORLD_3D && self.fluid3d.warm())
            || self.lines.warm()
    }

    /// The plain material's lit and shadow pipelines (front faces, opaque),
    /// which batches otherwise make when one first draws; true when this
    /// call made one.
    fn warm_plain_lit(&mut self) -> bool {
        // With the effect shader when the frame puts it on every batch:
        // the variant without it would be made for nothing.
        let key = (Side::Front, false, false, self.plain_effects);
        if !self.pbr_pipelines.contains_key(&key) {
            let p = pbr_pipeline(
                &self.device,
                (&self.pbr_layout, &self.pbr_skinned_layout),
                &self.pbr_shader,
                key,
                self.samples,
            );
            let declared = self.pbr_shader.declared;
            self.pbr_pipelines.insert(
                key,
                crate::pipeline_features::Variants::new(declared, move |_| {
                    let p = p.clone();
                    Deferred::new(move || p)
                }),
            );
            return true;
        }
        let key = (Side::Front, false, false);
        if !self.shadow_pipelines.contains_key(&key) {
            let p = shadow_pipeline(
                &self.device,
                &self.shadow_layouts,
                &self.shadow_shader,
                key,
                false,
                false,
            );
            self.shadow_pipelines.insert(key, Deferred::new(|| p));
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
        if !stats.gpu_culling {
            return false;
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
                .is_some_and(|(_, config)| !config.view_formats.is_empty()),
            RenderTarget::View(_) => false,
        }
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

    /// Forgets the TAA history: the next frame starts from its own samples
    /// (call it on a camera cut).
    pub fn reset_history(&mut self) {
        self.history.taa_history = None;
        self.gi_history = None;
        self.ssr.reset();
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
            || scene.post.gi.is_some_and(|g| g.intensity > 0.0)
        {
            CAPTURE_WARMUP
        } else {
            1
        };
        self.terrain.settled = true;
        self.capturing = true;
        let mut stats = self.render(scene, target);
        for _ in 1..warmup {
            stats = self.render(scene, target);
        }
        self.capturing = false;
        self.terrain.settled = false;
        self.put_frames(&kept);
        stats
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

#[cfg(test)]
mod tests {
    use crate::{
        AmbientOcclusion, AoMethod, Camera, RenderDesc, RenderScene, RenderTarget, Renderer, Side,
        StandardMaterial, shapes,
    };
    use glam::{Mat4, Vec3};

    /// [`Renderer::warm`] makes the plain lit pipeline that a plain
    /// material would draw with this frame: with screen-space AO every
    /// batch takes the effect shader (`prepare_frame_pipelines`), so the
    /// variant without it is never made.
    #[test]
    #[ignore = "requires GPU"]
    fn warming_makes_the_plain_lit_variant_the_frame_draws() {
        let desc = RenderDesc {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let mut renderer = Renderer::new_blocking(&desc).expect("renderer");
        let mut scene = RenderScene::new();
        scene.camera = Camera {
            position: Vec3::new(0.0, 0.0, 4.0),
            target: Vec3::ZERO,
            ..Default::default()
        };
        scene.post.ao = Some(AmbientOcclusion {
            method: AoMethod::Screen,
            ..Default::default()
        });
        let quad = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
        // The alpha cut keeps it off the lean pipelines: it draws a pbr one.
        let material = scene.add_standard_material(StandardMaterial {
            alphaTest: 0.5,
            ..Default::default()
        });
        scene.add_instance(quad, material, Mat4::IDENTITY);
        renderer
            .render(&scene, &RenderTarget::Offscreen)
            .expect("render");
        assert!(renderer.plain_effects, "the AO puts the effect shader on");
        while renderer.warm() {}
        let plain: Vec<bool> = renderer
            .pbr_pipelines
            .keys()
            .filter(|&&(side, blend, skinned, _)| side == Side::Front && !blend && !skinned)
            .map(|&(.., effects)| effects)
            .collect();
        assert_eq!(plain, [true], "the plain lit variants made");
    }
}
