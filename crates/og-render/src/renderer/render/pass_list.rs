//! The frame's pass list (schedule.rs `frame_passes`): what each pass's
//! run depends on this frame, decided from the effects, the plans and
//! the stylise, and the post chain's targets made for it.

use super::*;

pub(super) struct PassListInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub effects: &'a FrameEffects,
    /// The crowds culled on the GPU, and GPU culling's counts copied.
    pub crowd_cull: bool,
    pub cull_counts: bool,
    pub live_sky: bool,
    pub gi_probes: bool,
    pub particles: bool,
    pub light_2d: bool,
    pub metaballs: bool,
    pub froxels: bool,
    pub cascades: u32,
    pub shelter_redraw: bool,
    pub local_shadows: &'a [frame::LocalShadow],
    pub probes: bool,
    pub reflections: u8,
    pub views: u32,
    pub glow: bool,
    pub taa: bool,
    pub smaa_t2x: bool,
    /// What is drawn over the finished frame (after_graph.rs).
    pub after: schedule::AfterGraph,
}

impl Renderer {
    /// Decides the frame's pass list and makes the post chain for it.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_pass_list(
        &mut self,
        PassListInputs {
            scene,
            camera,
            effects,
            crowd_cull,
            cull_counts,
            live_sky,
            gi_probes,
            particles,
            light_2d,
            metaballs,
            froxels,
            cascades,
            shelter_redraw,
            local_shadows,
            probes,
            reflections,
            views,
            glow,
            taa,
            smaa_t2x,
            after,
        }: PassListInputs<'_>,
    ) -> Vec<FramePass> {
        let post_fx = &scene.look.post;
        let st = &scene.look.stylise;
        let auto_exposure = matches!(camera.exposure, Exposure::Auto { .. });
        let bloom = post_fx.bloom.is_some_and(|b| b.intensity > 0.0) || glow;
        let lens = post_fx.lens_distortion.filter(|l| l.amount != 0.0);
        let passes = schedule::frame_passes(
            &schedule::PassDecisions {
                crowd_cull,
                cull_counts,
                live_sky,
                gi_probes,
                cascades,
                shelter: shelter_redraw,
                local_shadows,
                prepass: effects.prepass,
                ao: effects
                    .gi
                    .map(|g| g.resolution)
                    .or(effects.ao.map(|a| a.resolution)),
                contact: effects.contact,
                particles,
                light_2d,
                metaballs,
                fluid_3d: effects.fluids_3d,
                probes,
                reflections,
                views,
                froxels,
                ssr: effects.ssr.is_some(),
                depth_bands: effects.depth_bands.is_some(),
                glow,
                outline: effects.outline,
                outline_outside: scene.look.stylise.material.outline_place
                    != crate::OutlinePlace::Inside,
                taa,
                taa_sharpen: self.post.taa_sharpen > 0.0,
                dof: effects.dof.is_some(),
                motion_blur: effects.motion_blur.is_some(),
                screen_blur: effects.screen_blur.is_some(),
                custom_posts: effects.custom_posts.len() as u32,
                god_rays: effects.god_rays.is_some(),
                auto_exposure,
                adapt: camera.exposure.adaptation().is_some(),
                bloom,
                bloom_levels: {
                    let (pw, ph) = self.history_size();
                    post::bloom_levels(pw, ph)
                },
                lens_flare: effects.lens_flare.is_some(),
                flare_sources: effects.flare_sources_on,
                smaa_t2x,
                paint: st.paint.is_some_and(|p| p.paints()),
                watercolor: st.paint.is_some_and(|p| p.watercolor.active()),
                lines: st.hatching.is_some() || st.halftone.is_some() || st.ink.is_some(),
                lens: lens.is_some(),
                quantize: st.quantize.is_some(),
                fxaa: self.info.antialias == Antialias::Fxaa,
                smaa: self.info.antialias == Antialias::Smaa,
                // Under TAAU the frame after TAA is already the output's size.
                upscale: self.history_size() != (self.out_width, self.out_height),
                after,
            },
            self.info.caps,
        );
        self.post.ensure_chain(&self.device, &passes);
        passes
    }
}
