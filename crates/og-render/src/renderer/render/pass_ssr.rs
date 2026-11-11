//! The screen-space reflections' pass (ssr.rs): the main camera's
//! reflections, traced in compute through the prepass's depth and normals
//! over the previous frame's lit image.

use super::*;

pub(super) struct SsrPass;

impl Pass for SsrPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Ssr,
        name: "ssr",
        needs: Caps::COMPUTE,
    };
    type State = ssr::SsrRenderer;
    type Plan<'f> = ssr::ScreenSpaceReflections;

    fn state(r: &Renderer) -> &Self::State {
        &r.ssr
    }

    fn encode<'f>(
        st: &Self::State,
        _: &ssr::ScreenSpaceReflections,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let r = cx.ctx.r;
        let (taa, slot) = (r.info.antialias == Antialias::Taa, cx.slot);
        // The previous frame's lit image: TAA's history, or
        // without TAA the main pass's copy (`gi-write`).
        let prev = match (&r.persistent.history, &r.persistent.gi) {
            (Some(h), _) if taa => &h[1 - slot],
            (_, Some((_, g))) if !taa => &g[1 - slot],
            _ => &r.post.black,
        };
        st.encode(
            &r.device,
            cx.encoder,
            r.graph.view("lindepth"),
            r.graph.view("gbuffer"),
            prev,
            r.phase(),
        );
    }
}

impl Renderer {
    /// The frame's screen-space reflections (ssr.rs): a perspective 3D
    /// camera's only, as an orthographic view reflects nothing it shows.
    pub(in crate::renderer) fn frame_ssr(
        &self,
        scene: &RenderScene,
    ) -> Option<ssr::ScreenSpaceReflections> {
        let perspective = matches!(scene.camera.projection, Projection::Perspective { .. });
        scene
            .post
            .ssr
            .filter(|s| {
                WORLD_3D
                    && self.runs(FramePass::Ssr)
                    && perspective
                    && scene.quality.ssr
                    && s.intensity > 0.0
            })
            .map(|s| ssr::ScreenSpaceReflections {
                steps: scene.quality.ssr_steps(s.steps),
                ..s
            })
    }

    /// The pass's uniforms for the frame's camera.
    pub(super) fn prepare_ssr(
        &mut self,
        s: &ssr::ScreenSpaceReflections,
        cam: &frame::CameraFrame,
        camera: &Camera,
        history_valid: bool,
    ) {
        self.ssr.prepare(
            &self.device,
            &self.queue,
            s,
            (self.width, self.height),
            &ssr::SsrCamera {
                view: cam.view,
                view_proj: cam.view_proj,
                near: camera.near,
                far: camera.far,
            },
            self.phase(),
            history_valid,
        );
    }
}
