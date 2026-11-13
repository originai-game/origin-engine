//! The ambient occlusion (or the screen-space GI in its place) and the
//! contact shadows that share its target.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let has = |p| cx.has(p);
    let ao_out = cx.ao_out;
    let mut ao_reads = vec![if cx.ao_depth { "ao-depth" } else { "lindepth" }, "gbuffer"];
    // The GI reads the previous frame's lit image.
    if cx.cfg.gi {
        ao_reads.push(if cx.cfg.taa {
            "history-read"
        } else {
            "gi-read"
        });
    }
    let name = p.name();
    match p {
        FramePass::Ao => g.pass(name, &ao_reads, &["ao.raw"]),
        FramePass::AoBlur if has(FramePass::AoUpsample) => {
            g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &["ao.blur"])
        }
        FramePass::AoBlur => g.pass(name, &["ao.raw", "lindepth", "gbuffer"], &[ao_out]),
        FramePass::AoUpsample => g.pass(name, &["ao.blur", "lindepth"], &[ao_out]),
        FramePass::Contact if has(FramePass::AoBlur) => {
            g.pass(name, &["ao.pre", "lindepth", "gbuffer"], &["ao"])
        }
        FramePass::Contact => g.pass(name, &["lindepth", "gbuffer"], &["ao"]),
        _ => unreachable!("{p:?} is not an AO pass"),
    };
}
