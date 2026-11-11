//! FSR 3.1 in TAA's place (post/fsr.rs): its frame prepared where the
//! quality asks for it and the frame is not supersampled, else its
//! resources released.

use super::*;

pub(super) struct PrepareFsrInputs<'a> {
    pub scene: &'a RenderScene,
    pub cam: &'a frame::CameraFrame,
    pub taa: bool,
    pub slot: usize,
}

impl Renderer {
    /// Prepares or releases the frame's FSR upscale; returns whether it runs.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_fsr(
        &mut self,
        PrepareFsrInputs {
            scene,
            cam,
            taa,
            slot,
        }: PrepareFsrInputs<'_>,
    ) -> bool {
        // FSR 3.1 in TAA's place (post/fsr.rs), where the quality asks for
        // it and the frame is not supersampled.
        let post_size = self.history_size();
        let fsr = taa
            && self.info.caps.contains(Caps::COMPUTE)
            && scene.frame.quality.upscaler == crate::Upscaler::Fsr3
            && self.width <= post_size.0
            && self.height <= post_size.1;
        let no_reactive = self.post.no_reactive.clone();
        match self.graph.fsr_views(&no_reactive, &self.persistent, slot) {
            Some((color, depth, prepass_views, reveal, output, previous_output, sharpened))
                if fsr =>
            {
                let frame = post::fsr::FsrFrame {
                    render: (self.width, self.height),
                    upscale: post_size,
                    jitter: self.taa_jitter(),
                    phases: frame::jitter_phases((self.width, self.height), post_size),
                    inv_view_proj: cam.jittered_view_proj.inverse(),
                    view_proj: cam.view_proj,
                    prev_view_proj: self.history.taa_history.unwrap_or(cam.view_proj),
                    proj: cam.proj,
                    reset: self.history.taa_history.is_none(),
                    still: self
                        .history
                        .taa_history
                        .is_none_or(|p| still_camera(&p, &cam.view_proj)),
                    tick: scene.frame.tick,
                    tick_rate: scene.frame.tick_rate,
                    sharpness: self.post.taa_sharpen,
                    color,
                    depth,
                    prepass: prepass_views,
                    reveal,
                    output,
                    previous_output,
                    sharpened,
                };
                self.post.fsr.prepare(&self.queue, &frame);
            }
            _ => self.post.fsr.release(),
        }
        fsr
    }
}
