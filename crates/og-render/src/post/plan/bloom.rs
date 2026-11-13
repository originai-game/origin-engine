//! The bloom pyramid and the lens flare that reads its first level.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let has = |p| cx.has(p);
    let (lit, lum) = (cx.lit, cx.lum.as_str());
    let (auto, prepass) = (cx.auto, cx.prepass);
    let name = p.name();
    match p {
        FramePass::BloomPrefilter => {
            let mut reads = vec![lit];
            if auto {
                reads.push(lum);
            }
            if has(FramePass::Glow) {
                reads.push("glow");
            }
            g.pass(name, &reads, &["bloom.0"])
        }
        FramePass::BloomDown(i) => g.pass(name, &[&bloom_name(i)], &[&bloom_name(i + 1)]),
        FramePass::BloomUp(i) => g.pass(name, &[&bloom_name(i + 1)], &[&bloom_name(i)]),
        FramePass::FlareOcclusion => g.pass(name, &[lit, "lindepth"], &["flare.vis"]),
        // Declared before the upsamples, so the last of them (onto
        // `bloom.0`) waits for it.
        FramePass::Flare => {
            let mut reads = vec!["bloom.0"];
            if prepass {
                reads.push("lindepth");
            }
            if auto {
                reads.push(lum);
            }
            if has(FramePass::FlareOcclusion) {
                reads.push("flare.vis");
            }
            g.pass(name, &reads, &["flare.raw"])
        }
        FramePass::FlareBlurX => g.pass(name, &["flare.raw"], &["flare.x"]),
        FramePass::FlareBlurY => g.pass(name, &["flare.x"], &["flare"]),
        _ => unreachable!("{p:?} is not a bloom or flare pass"),
    };
}
