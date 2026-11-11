//! The frame's end: its pass timings kept, the presented frame kept for
//! capture, the debris, cull and frame command buffers submitted (apart
//! under the bisect knob), what waits on the submission told, the surface
//! presented, and the history the next frame reads (TAA's, GI's, the
//! previous camera, time and wind) advanced.

use super::*;

pub(super) struct SubmitFrameInputs<'a> {
    pub scene: &'a RenderScene,
    pub target: &'a RenderTarget<'a>,
    pub cam: &'a frame::CameraFrame,
    pub encoder: Encoder,
    pub surface_frame: Option<(
        wgpu::SurfaceTexture,
        wgpu::TextureView,
        Option<wgpu::TextureView>,
    )>,
    pub debris_commands: Option<wgpu::CommandBuffer>,
    pub cull_commands: Vec<wgpu::CommandBuffer>,
    pub pass_ms: Vec<PassTiming>,
    pub pass_queries: Vec<(String, std::ops::Range<u32>)>,
    pub oit_mode: Option<oit::Oit>,
    pub gpu_cull: bool,
    pub temporal: bool,
    pub gi_copy: bool,
    pub t: [f32; 4],
    pub stats: &'a mut FrameStats,
}

impl Renderer {
    /// Submits and presents the frame and advances the renderer's history.
    #[allow(clippy::too_many_lines)]
    pub(super) fn submit_frame(
        &mut self,
        SubmitFrameInputs {
            scene,
            target,
            cam,
            mut encoder,
            surface_frame,
            debris_commands,
            cull_commands,
            pass_ms,
            pass_queries,
            oit_mode,
            gpu_cull,
            temporal,
            gi_copy,
            t,
            stats,
        }: SubmitFrameInputs<'_>,
    ) {
        let gpu_queries = encoder.used();
        self.pass_ms = pass_ms;
        // HDR already rendered scene + overlay into the float offscreen target.
        // Keep that paper-white-linear frame for capture, not the encoded surface.
        if let Some((frame, ..)) = surface_frame
            .as_ref()
            .filter(|_| self.keep_frame && self.hdr.is_none())
        {
            self.keep_presented(&mut encoder, &frame.texture);
        }
        // A bisect's knob (dev builds): the debris and cull command buffers
        // in a submit of their own, before the frame's (settings.rs
        // `debug_cull_apart`).
        #[cfg(not(feature = "publish"))]
        let (debris_commands, cull_commands) =
            if super::settings::DEBUG_CULL_APART.load(std::sync::atomic::Ordering::Relaxed) {
                self.queue
                    .submit(debris_commands.into_iter().chain(cull_commands));
                (Vec::new(), Vec::new())
            } else {
                (
                    debris_commands.into_iter().collect::<Vec<_>>(),
                    cull_commands.into_iter().collect::<Vec<_>>(),
                )
            };
        let _submission = self.queue.submit(
            debris_commands
                .into_iter()
                .chain(cull_commands)
                .chain(Some(encoder.finish())),
        );
        // The web keeps no backlog: its surface paces the frames.
        #[cfg(target_arch = "wasm32")]
        let _ = target;
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(target, RenderTarget::Offscreen) {
            self.offscreen_submissions.push_back(_submission);
        }
        self.shadows.shadow_cache.submitted();
        if let Some(arena) = self
            .oit
            .arena
            .as_ref()
            .filter(|_| oit_mode == Some(oit::Oit::List))
        {
            arena.after_submit();
        }
        if let Some(gc) = self.visibility.gpu_cull.as_mut().filter(|_| gpu_cull) {
            gc.after_submit();
        }
        if let (Some(t), Some(c)) = (self.gpu_timer.as_mut(), self.clock) {
            t.after_submit(
                &self.queue,
                self.frame_index,
                pass_queries,
                gpu_queries,
                c(),
            );
        }
        if let Some((frame, ..)) = surface_frame {
            self.queue.present(frame);
        }
        if temporal {
            // A history reset since the last frame (a cut, a resize, a
            // look switch) starts the count again.
            self.history.taa_history_frames = match self.history.taa_history {
                Some(_) => self.history.taa_history_frames + 1,
                None => 1,
            };
            self.history.taa_history = Some(cam.view_proj);
        } else {
            self.history.taa_history_frames = 0;
        }
        // The main pass copied its lit image for the next frame's GI.
        self.gi_history = gi_copy.then_some(self.frame_index);
        self.history.prev_view_proj = Some(cam.view_proj);
        self.history.previous_time = Some(t[0]);
        let wind = scene.wind.unwrap_or(Wind {
            strength: 0.0,
            ..Wind::default()
        });
        self.instances.previous_sway.clear();
        self.instances
            .previous_sway
            .extend(scene.instances.iter().map(|i| i.sway));
        self.history.previous_wind = Some((wind, scene.tick, scene.tick_rate));
        self.frame_index += 1;
        // The first frame made the scene's pipelines: keep them now, not
        // only when the renderer is dropped (a killed process never is).
        if self.frame_index == 1
            && let Some(cache) = &mut self.pipeline_disk
        {
            cache.save();
        }
        // The full lit variants this frame asked for compile now
        // (pipeline_features.rs).
        crate::deferred::release_background();
        stats.compile_wait_us = crate::deferred::take_waited_us().min(u64::from(u32::MAX)) as u32;
    }
}
