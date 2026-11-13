//! The passes after the tonemap ([`chain_passes`]): the stylisation
//! (paint, watercolour, lines, lens, quantise), FXAA or SMAA, and the
//! upscale. Each reads one LDR target and draws into the next; the last
//! draws into the output.

use super::*;

pub(super) struct Chain {
    groups: HashMap<FramePass, wgpu::BindGroup>,
    /// SMAA T2x's resolve of its two display frames.
    smaa_resolve: Option<Pair>,
}

/// A stylise pass's group over `source`.
fn stylise(
    cx: &BindCx,
    pass: FramePass,
    source: &wgpu::TextureView,
    untoned: &wgpu::TextureView,
) -> wgpu::BindGroup {
    let passes = cx.passes;
    // The paint, lines and quantise passes read the prepass (a canvas,
    // hatching, halftone or dither on surfaces, ink on depth edges)
    // when the frame has one; the other passes bind placeholders.
    let lines = matches!(
        pass,
        FramePass::Paint | FramePass::Watercolor | FramePass::Lines | FramePass::Quantize
    );
    let prepass = |name: &str, none| tex(cx.graph.try_view(name).filter(|_| lines).unwrap_or(none));
    // The scene before exposure and the tone curve, which the quantize
    // and lines passes test for conformed colours (the others bind their
    // source).
    let scene = if matches!(pass, FramePass::Quantize | FramePass::Lines) {
        untoned
    } else {
        source
    };
    cx.group(
        &pass.name(),
        &passes.stylise_layout,
        &[
            tex(source),
            cx.sampler(),
            passes.stylise_buffer.as_entire_binding(),
            tex(cx.stylise.0),
            tex(cx.stylise.1),
            wgpu::BindingResource::Sampler(&passes.repeat_sampler),
            tex(&passes.blue_noise),
            prepass("lindepth", &passes.far_depth),
            prepass("gbuffer", &passes.black),
            prepass("object-space", &passes.no_object),
            // The object ids the paint pass keeps its taps within.
            prepass("outline-id", &passes.black),
            tex(cx.stylise.2),
            tex(scene),
        ],
    )
}

/// SMAA's groups over `source`: its edges, their weights and the blend.
fn smaa(cx: &BindCx, source: &wgpu::TextureView) -> [(FramePass, wgpu::BindGroup); 3] {
    let passes = cx.passes;
    let smaa = passes.smaa.as_ref().expect("SMAA renderer");
    [
        (
            FramePass::SmaaEdges,
            cx.group(
                "smaa edges",
                &passes.source_layout,
                &[tex(source), cx.sampler()],
            ),
        ),
        (
            FramePass::SmaaWeights,
            cx.group(
                "smaa weights",
                &smaa.weights_layout,
                &[
                    tex(cx.view("smaa.edges")),
                    cx.sampler(),
                    tex(&smaa.area),
                    tex(&smaa.search),
                    passes.taa_buffer.as_entire_binding(),
                ],
            ),
        ),
        (
            FramePass::SmaaBlend,
            cx.group(
                "smaa blend",
                &smaa.blend_layout,
                &[tex(source), cx.sampler(), tex(cx.view("smaa.weights"))],
            ),
        ),
    ]
}

impl Chain {
    pub fn bind(cx: &BindCx) -> Chain {
        let passes = cx.passes;
        let untoned = cx.view(untoned(cx.frame));
        let mut groups = HashMap::new();
        for (i, p) in chain_passes(cx.frame).into_iter().enumerate() {
            let source = cx.view(&ldr_name(i));
            match p {
                FramePass::Fxaa => {
                    let g = cx.group("fxaa", &passes.source_layout, &[tex(source), cx.sampler()]);
                    groups.insert(p, g);
                }
                // The resolve binds the two frames the blend left
                // (`smaa_resolve`).
                FramePass::SmaaBlend | FramePass::SmaaResolve => groups.extend(smaa(cx, source)),
                _ => {
                    groups.insert(p, stylise(cx, p, source, untoned));
                }
            }
        }
        let smaa_resolve = cx.persistent.smaa_history.as_ref().map(|h| {
            cx.pair(|slot| {
                cx.temporal(
                    "temporal resolve",
                    &h[slot],
                    &h[1 - slot],
                    &passes.no_reactive,
                )
            })
        });
        Chain {
            groups,
            smaa_resolve,
        }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        let passes = cx.passes;
        Some(match pass {
            FramePass::SmaaEdges | FramePass::SmaaWeights => {
                let smaa = passes.smaa.as_ref().expect("SMAA renderer");
                let (target, pipeline) = if pass == FramePass::SmaaEdges {
                    ("smaa.edges", &smaa.edges)
                } else {
                    ("smaa.weights", &smaa.weights)
                };
                fullscreen_pass(
                    encoder,
                    &pass.name(),
                    cx.graph.view(target),
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    pipeline,
                    &self.groups[&pass],
                );
                1
            }
            // T2x: the blend into this frame's history, which the resolve
            // reads with the frame before.
            FramePass::SmaaBlend if cx.persistent.smaa_history.is_some() => {
                let temporal = passes
                    .smaa
                    .as_ref()
                    .and_then(|s| s.temporal.as_ref())
                    .expect("T2x renderer");
                fullscreen_pass(
                    encoder,
                    "smaa history",
                    &cx.persistent.smaa_history.as_ref().unwrap()[cx.slot],
                    CLEAR,
                    &temporal.blend,
                    &self.groups[&pass],
                );
                1
            }
            FramePass::Paint
            | FramePass::Watercolor
            | FramePass::Lines
            | FramePass::Lens
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::SmaaBlend
            | FramePass::SmaaResolve
            | FramePass::Upscale => {
                let chain = chain_passes(&cx.graph.passes);
                let i = chain.iter().position(|&p| p == pass).expect("in the chain");
                let last = i + 1 == chain.len();
                let group = if pass == FramePass::SmaaResolve {
                    self.smaa_resolve.as_ref().map(|g| &g[cx.slot])
                } else {
                    self.groups.get(&pass)
                };
                match (passes.chain.get(&pass), group) {
                    (Some((to_ldr, to_output)), Some(group)) => {
                        let (target, pipeline) = if last {
                            (cx.output, to_output)
                        } else {
                            (cx.graph.view(&ldr_name(i + 1)), to_ldr)
                        };
                        fullscreen_pass(encoder, &pass.name(), target, CLEAR, pipeline, group);
                        1
                    }
                    _ => 0,
                }
            }
            _ => return None,
        })
    }
}
