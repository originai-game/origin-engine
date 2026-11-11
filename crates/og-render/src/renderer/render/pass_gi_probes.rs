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
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.gi_probes
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
                (7, r.lights.light_buffers[0].as_entire_binding()),
                (10, r.lights.lightmaps.probe_buffer.as_entire_binding()),
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

impl GiProbesPass {
    /// The frame's plan: the pass runs when probes are due.
    pub(super) fn plan(st: &<Self as Pass>::State) -> Option<()> {
        st.as_ref().is_some_and(|g| g.due()).then_some(())
    }
}

impl Renderer {
    /// The probe GI's frame (gi_probes.rs): off where the quality's GI
    /// switch or a 2D build says so. The probe buffer grows to hold its
    /// probes before the frame groups that bind it are made.
    pub(super) fn prepare_gi_probes(&mut self, scene: &RenderScene) {
        let desc = scene
            .look
            .post
            .gi_probes
            .filter(|_| WORLD_3D && scene.frame.quality.global_illumination)
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
            .lights
            .lightmaps
            .ensure_capacity(&self.device, &self.queue, need)
        {
            // The groups binding the buffer: the frame's and the effect
            // passes' (the screen GI reads the probes).
            self.lights.environment = None;
            self.graph.invalidate();
            g.restart();
        }
        g.prepare(&self.device, &self.queue, scene, desc, region);
    }
}
