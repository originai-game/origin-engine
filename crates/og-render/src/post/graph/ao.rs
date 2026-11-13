//! The ambient occlusion's passes, and in their place the screen-space
//! GI's (ssgi.rs) or GTAO with its bent normals: the trace, its blur and
//! at half resolution its upsample.

use super::*;

pub(super) fn groups(cx: &BindCx, pass: FramePass) -> Option<Pair> {
    let black = &cx.passes.black;
    Some(match pass {
        // The AO reads the depth with the water's surfaces in it when there is one.
        // The GI reads the lit image the previous frame left: the
        // TAA history it wrote, or the main pass's copy.
        FramePass::Ao => {
            let d = cx.graph.try_view("ao-depth").unwrap_or(cx.depth());
            let previous = |slot: usize| match (cx.cfg.gi, cx.history(), &cx.persistent.gi) {
                (false, ..) => black,
                (true, Some(h), _) => &h[1 - slot],
                (true, None, Some((_, g))) => &g[1 - slot],
                (true, None, None) => black,
            };
            cx.pair(|slot| cx.fx_depth(previous(slot), d, black))
        }
        FramePass::AoBlur => cx.pair(|_| cx.fx(black, cx.view("ao.raw"))),
        FramePass::AoUpsample => cx.pair(|_| cx.fx(black, cx.view("ao.blur"))),
        _ => return None,
    })
}

pub(super) fn draw(cx: &PostCx, pass: FramePass) -> Option<fx::Draw> {
    // Screen-space GI in the AO passes' place (ssgi.rs).
    let gi = cx.graph.cfg.is_some_and(|c| c.gi);
    // GTAO with its bent normals: four channels blurred as the GI's.
    let bent = cx.graph.cfg.is_some_and(|c| c.bent);
    let half = cx.graph.try_view("ao.blur").is_some();
    // With contact shadows the AO passes end in `ao.pre` (plan.rs).
    let ao = ao_target_name(cx.graph.try_view("ao.pre").is_some());
    Some(match pass {
        FramePass::Ao if gi => ("fs_ssgi", "ao.raw"),
        FramePass::Ao if bent => ("fs_gtao_bent", "ao.raw"),
        FramePass::Ao => (if cx.gtao { "fs_gtao" } else { "fs_ssao" }, "ao.raw"),
        FramePass::AoBlur if gi || bent => ("fs_gi_blur", if half { "ao.blur" } else { ao }),
        // Half resolution: the blur's depth is at twice its texel, which
        // gathers do not read.
        FramePass::AoBlur if half => ("fs_ao_blur", "ao.blur"),
        FramePass::AoBlur => (AO_BLUR_GATHER, ao),
        FramePass::AoUpsample if gi || bent => ("fs_gi_upsample", ao),
        FramePass::AoUpsample => ("fs_ao_upsample", ao),
        _ => return None,
    })
}
