//! The 3D fluids' screen-space pass (fluid3d.rs): their particles' depth
//! and thickness, smoothed, which the main pass shades as a surface.

use super::*;

pub(super) struct Fluid3dPass;

impl Pass for Fluid3dPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Fluid3d,
        name: "fluid-3d",
        needs: Caps::NONE,
    };
    type State = fluid3d::Fluid3dRenderer;
    /// The fluids' frame, when there is one to draw.
    type Plan<'f> = Option<&'f fluid3d::Fluid3dFrame>;

    fn state(r: &Renderer) -> &Self::State {
        &r.world.fluid3d
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &Option<&'f fluid3d::Fluid3dFrame>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let r = cx.ctx.r;
        if let (Some(f), Some(views)) = (plan, fluid3d::views(&r.graph, cx.scene_depth)) {
            cx.stats.draw_calls += st.encode(cx.encoder, f, cx.frame_bind_group, &views);
        }
    }
}
