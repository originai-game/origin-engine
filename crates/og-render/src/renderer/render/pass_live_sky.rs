//! The live sky's pass (sky_capture.rs): the environment's due slice,
//! captured from the sky as it is drawn this frame.

use super::*;

pub(super) struct LiveSkyPass;

impl Pass for LiveSkyPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::LiveSky,
        name: "live-sky",
        needs: Caps::NONE,
    };
    type State = Option<crate::sky_capture::SkyCapture>;
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.sky_capture
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let c = st.as_ref().expect("planned with it");
        let ibl = r.lights.ibl.views();
        let frame = |uniform: wgpu::BindingResource| {
            bind_group(
                &r.device,
                "frame (live sky)",
                &r.frame_layout,
                &[
                    uniform,
                    wgpu::BindingResource::TextureView(&r.shadows.cascade_map.array_view),
                    wgpu::BindingResource::TextureView(&r.shadows.local_map.array_view),
                    wgpu::BindingResource::TextureView(&ibl.irradiance),
                    wgpu::BindingResource::TextureView(&ibl.prefiltered),
                    wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                    wgpu::BindingResource::Sampler(&ibl.sampler),
                    r.lights.light_buffers[0].as_entire_binding(),
                    r.lights.light_buffers[1].as_entire_binding(),
                    r.lights.light_buffers[2].as_entire_binding(),
                    r.lights.lightmaps.probe_buffer.as_entire_binding(),
                    r.lights.ltc.as_entire_binding(),
                    r.lights.caustic_grid.as_entire_binding(),
                    wgpu::BindingResource::TextureView(r.ssr.off_view()),
                ],
            )
        };
        c.record(cx.encoder, &r.lights.ibl, frame, cx.sky_group);
    }
}

impl LiveSkyPass {
    /// The frame's plan: the pass runs while the scene's sky is live.
    pub(super) fn plan(st: &<Self as Pass>::State) -> Option<()> {
        st.is_some().then_some(())
    }

    /// The due faces' uniforms: the frame's, seen from each face.
    pub(super) fn write(
        st: &<Self as Pass>::State,
        queue: &wgpu::Queue,
        uniform: &crate::frame::FrameUniform,
        camera: &Camera,
    ) {
        if let Some(c) = st {
            c.write(queue, uniform, camera);
        }
    }
}
