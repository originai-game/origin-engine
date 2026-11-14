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
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    type BindIn<'f> = ();
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.sky.sky_capture
    }

    /// The pass runs while the scene's sky is live.
    fn bind<'f>(r: &'f Renderer, _: &'f mut (), _: ()) -> Option<()> {
        Self::state(r).is_some().then_some(())
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
                &r.frame_entries(FrameGroupParts {
                    uniform,
                    clusters: [&r.lights.light_buffers[1], &r.lights.light_buffers[2]],
                    cubes: [&ibl.irradiance, &ibl.prefiltered],
                    ssr: r.ssr.off_view(),
                }),
            )
        };
        c.record(cx.encoder, &r.lights.ibl, frame, cx.sky_group);
    }
}

impl LiveSkyPass {
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
