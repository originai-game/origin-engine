//! The effects over the lit image: the depth bands before TAA; after it
//! the depth of field, the motion blur and the screen blur, each reading
//! what the one before left; and the god rays' mask and blur, which the
//! tonemap adds.

use super::*;

pub(super) fn groups(cx: &BindCx, pass: FramePass) -> Option<Pair> {
    let black = &cx.passes.black;
    let names = &cx.names;
    Some(match pass {
        FramePass::DepthBands => cx.pair(|_| cx.fx(cx.view("hdr"), black)),
        FramePass::DofCoc => cx.pair(|slot| cx.fx_after(cx.stage(names.aa, slot), black)),
        FramePass::DofBlur => cx.pair(|_| cx.fx_after(cx.view("dof.coc"), black)),
        FramePass::DofComposite => {
            cx.pair(|slot| cx.fx_after(cx.stage(names.aa, slot), cx.view("dof.blur")))
        }
        FramePass::MotionBlur => {
            cx.pair(|slot| cx.fx_after(cx.stage(names.after_dof, slot), black))
        }
        FramePass::ScreenBlur => cx.pair(|slot| cx.fx_after(cx.stage(names.after_mb, slot), black)),
        FramePass::GodRaysMask => cx.pair(|slot| cx.fx_after(cx.source(slot), black)),
        FramePass::GodRaysBlur => cx.pair(|_| cx.fx_after(black, cx.view("god.mask"))),
        _ => return None,
    })
}

pub(super) fn draw(pass: FramePass) -> Option<fx::Draw> {
    Some(match pass {
        FramePass::DepthBands => ("fs_depth_bands", "hdr.bands"),
        FramePass::DofCoc => ("fs_dof_coc", "dof.coc"),
        FramePass::DofBlur => ("fs_dof_blur", "dof.blur"),
        FramePass::DofComposite => ("fs_dof_composite", "hdr.dof"),
        FramePass::MotionBlur => ("fs_motion_blur", "hdr.mb"),
        FramePass::ScreenBlur => ("fs_screen_blur", "hdr.blur"),
        FramePass::GodRaysMask => ("fs_godrays_mask", "god.mask"),
        FramePass::GodRaysBlur => ("fs_godrays_blur", "god.blur"),
        _ => return None,
    })
}
