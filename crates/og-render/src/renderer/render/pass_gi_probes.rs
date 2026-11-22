//! The probe GI's pass (gi_probes.rs): the due probes' relighting, in
//! compute, into the light volumes' probe buffer.

use super::*;

pub(super) struct GiProbesPass;

impl Pass for GiProbesPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::GiProbes,
        name: "gi-probes",
        needs: Caps::COMPUTE,
    };
    type State = Option<crate::gi_probes::GiProbes>;
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    type BindIn<'f> = ();
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.gi_probes
    }

    /// The pass runs when probes are due.
    fn bind<'f>(r: &'f Renderer, _: &'f mut (), _: ()) -> Option<()> {
        Self::state(r)
            .as_ref()
            .is_some_and(|g| g.due())
            .then_some(())
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let g = st.as_ref().expect("planned with it");
        let ibl = r.lights.ibl.views();
        let frame = crate::volumetric::bind_at(
            &r.device,
            "gi probes frame",
            &g.frame_layout,
            &[
                (0, r.lights.frame_buffer.as_entire_binding()),
                (3, wgpu::BindingResource::TextureView(&ibl.irradiance)),
                (4, wgpu::BindingResource::TextureView(&ibl.prefiltered)),
                (6, wgpu::BindingResource::Sampler(&ibl.sampler)),
                (7, r.lights.light_tables.as_entire_binding()),
                (8, r.lights.lightmaps.probe_buffer.as_entire_binding()),
            ],
        );
        g.record(
            &r.device,
            cx.encoder,
            &frame,
            &r.lights.lightmaps.probe_buffer,
        );
    }
}

impl Renderer {
    /// The probe GI's frame (gi_probes.rs): off where the quality's GI
    /// switch or a 2D build says so; then the static lights' far field
    /// (static_far.rs) after its region. The probe buffer grows to hold
    /// both before the frame groups that bind it are made.
    pub(super) fn prepare_gi_probes(&mut self, scene: &RenderScene) {
        let desc = scene
            .look
            .post
            .gi_probes
            .filter(|_| scene.frame.quality.global_illumination)
            // A renderer made without the probe GI (RenderDesc::without).
            .filter(|_| !self.info.without.contains(LitFeatures::GI));
        if desc.is_none()
            && let Some(g) = &mut self.gi_probes
        {
            g.disable();
        }
        let region = crate::lightmap::PROBE_CAPACITY;
        let far_at = region + desc.as_ref().map_or(0, crate::gi_probes::region_vec4s);
        let far = crate::static_far::far_lights(scene, self.lights.clusters.skip_rects);
        let need = far_at
            + if far.is_empty() {
                0
            } else {
                crate::static_far::VEC4S
            };
        if self
            .lights
            .lightmaps
            .ensure_capacity(&self.device, &self.queue, need)
        {
            // The groups binding the buffer: the frame's and the effect
            // passes' (the screen GI reads the probes).
            self.lights.environment = None;
            self.graph.invalidate();
            if let Some(g) = &mut self.gi_probes {
                g.restart();
            }
            self.lights.static_far.reset();
        }
        if let Some(desc) = desc {
            let g = self
                .gi_probes
                .get_or_insert_with(|| crate::gi_probes::GiProbes::new(&self.device));
            g.prepare(&self.device, &self.queue, scene, desc, region);
        }
        let lightmaps = &mut self.lights.lightmaps;
        lightmaps.far_header = self.lights.static_far.prepare(
            &self.queue,
            &lightmaps.probe_buffer,
            scene,
            &far,
            far_at,
        );
    }
}
