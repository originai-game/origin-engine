//! The views' uniforms and targets (views.rs `write`): each view's frame
//! and effects uniforms, its light map and glow weight, then its 2D lights'
//! view.

use super::*;

pub(super) struct WriteViewUniformsInputs<'a> {
    pub scene: &'a RenderScene,
    pub target: &'a RenderTarget<'a>,
    pub view_plans: &'a [views::ViewPlan],
    pub uniform: &'a frame::FrameUniform,
    pub fx_uniform: &'a fx::FxUniform,
    pub lights_on: bool,
    pub light_slots: &'a crate::light2d::LightSlots,
    pub fluids: usize,
    pub particle_soft: bool,
    pub post_fx: &'a crate::PostEffects,
    pub view_glow_runs: &'a [Vec<sprites::SpriteRun>],
}

pub(super) struct WriteViewUniformsOutputs {
    pub viewport_format: wgpu::TextureFormat,
}

impl Renderer {
    /// Writes each view's uniforms and its 2D lights' view; returns the viewport format.
    #[allow(clippy::too_many_lines)]
    pub(super) fn write_view_uniforms(
        &mut self,
        WriteViewUniformsInputs {
            scene,
            target,
            view_plans,
            uniform,
            fx_uniform,
            lights_on,
            light_slots,
            fluids,
            particle_soft,
            post_fx,
            view_glow_runs,
        }: WriteViewUniformsInputs<'_>,
    ) -> WriteViewUniformsOutputs {
        let viewport_format = self.viewport_format(target);
        self.views.write(
            &self.device,
            &self.queue,
            self.samples,
            &self.lights.clusters.kept,
            scene,
            view_plans,
            uniform,
            fx_uniform,
            lights_on,
            viewport_format,
            scene
                .sprite
                .lighting_2d
                .map(|l| (l.resolution.clamp(0.25, 1.0), light_slots.count())),
            fluids,
            particle_soft,
            // The weight the frame's tonemap gives its glow (post.rs
            // `write_uniform`): the bloom's intensity when the lit image
            // blooms too, else the glow's own.
            post_fx
                .glow
                .filter(|_| view_glow_runs.iter().any(|r| !r.is_empty()))
                .map(|g| {
                    post_fx
                        .bloom
                        .filter(|b| b.intensity > 0.0)
                        .map_or(g.intensity, |b| b.intensity)
                }),
        );
        if scene.sprite.lighting_2d.is_some() {
            for (k, p) in view_plans.iter().enumerate() {
                self.lights_2d
                    .write_view(&self.device, &self.queue, k, p.frame.view_proj);
            }
        }
        WriteViewUniformsOutputs { viewport_format }
    }
}
