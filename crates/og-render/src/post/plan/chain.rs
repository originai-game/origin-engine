//! The tonemap and the chain after it: SMAA, the painterly and line
//! filters, the lens, the quantize, FXAA and the upscale, each from one
//! `ldr` target into the next, the last into the output.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let (passes, cfg) = (cx.passes, cx.cfg);
    let has = |p| cx.has(p);
    let chain = &cx.chain;
    let (lit, lum) = (cx.lit, cx.lum.as_str());
    let (auto, bloom) = (cx.auto, cx.bloom);
    let (prepass, smaa_t2x) = (cx.prepass, cx.smaa_t2x);
    let name = p.name();
    match p {
        FramePass::Tonemap => {
            let mut reads = vec![lit];
            if auto {
                reads.push(lum);
            }
            if bloom {
                reads.push("bloom.0");
            }
            if has(FramePass::GodRaysBlur) {
                reads.push("god.blur");
            }
            if has(FramePass::FlareBlurY) {
                reads.push("flare");
            }
            if cfg.ui_depth && prepass {
                reads.push("lindepth");
            }
            let target = if chain.is_empty() {
                "output".to_string()
            } else {
                ldr_name(0)
            };
            g.pass(name, &reads, &[target.as_str()])
        }
        FramePass::SmaaEdges => {
            let i = chain
                .iter()
                .position(|&c| c == FramePass::SmaaBlend || c == FramePass::SmaaResolve)
                .expect("SMAA in chain");
            g.pass(name, &[&ldr_name(i)], &["smaa.edges"])
        }
        FramePass::SmaaWeights => g.pass(name, &["smaa.edges"], &["smaa.weights"]),
        FramePass::SmaaBlend if smaa_t2x => {
            g.pass(name, &["ldr.0", "smaa.weights"], &["smaa-write"])
        }
        FramePass::SmaaResolve => {
            let mut reads = vec!["smaa-write", "smaa-read", "depth"];
            if prepass {
                reads.extend(["gbuffer", "lindepth"]);
            }
            let write = if chain.len() == 1 { "output" } else { "ldr.1" };
            g.pass(name, &reads, &[write])
        }
        FramePass::Paint
        | FramePass::Watercolor
        | FramePass::Lines
        | FramePass::Lens
        | FramePass::Quantize
        | FramePass::Fxaa
        | FramePass::SmaaBlend
        | FramePass::Upscale => {
            let i = chain.iter().position(|&c| c == p).expect("in the chain");
            let read = ldr_name(i);
            let write = if i + 1 == chain.len() {
                "output".to_string()
            } else {
                ldr_name(i + 1)
            };
            // The paint and lines passes bind the prepass
            // (`Bindings::chain`).
            let mut reads = vec![read.as_str()];
            // The quantize and lines passes also read the scene before
            // exposure and the tone curve (conformed art keeps its colour).
            if matches!(p, FramePass::Quantize | FramePass::Lines) {
                reads.push(untoned(passes));
            }
            if p == FramePass::SmaaBlend {
                reads.push("smaa.weights");
            }
            if matches!(
                p,
                FramePass::Paint | FramePass::Watercolor | FramePass::Lines
            ) && prepass
            {
                reads.extend(["lindepth", "gbuffer", "outline-id"]);
                if cfg.object_space {
                    reads.push("object-space");
                }
            }
            g.pass(name, &reads, &[write.as_str()])
        }
        _ => unreachable!("{p:?} is not a tonemap or chain pass"),
    };
}
