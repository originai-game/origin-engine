//! The auto-exposure: the meter, its reduction to one texel and the
//! adaptation.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let lit = cx.lit;
    let metered = cx.metered.as_str();
    let name = p.name();
    match p {
        FramePass::Meter => g.pass(name, &[lit], &["luminance.0"]),
        FramePass::Reduce(i) => g.pass(name, &[&luminance_name(i)], &[&luminance_name(i + 1)]),
        FramePass::Adapt => g.pass(name, &[metered], &["exposure"]),
        _ => unreachable!("{p:?} is not an exposure pass"),
    };
}
