//! The lens flare's passes (`flare.rs`): the sun's visibility from the
//! lit frame, the flare from the bloom, and its two blurs. The tonemap
//! adds the result.

use super::*;

pub(super) struct LensFlare {
    groups: HashMap<FramePass, Pair>,
}

impl LensFlare {
    pub fn bind(cx: &BindCx) -> LensFlare {
        let black = &cx.passes.black;
        // (source, view depth, exposure, visibility) per slot,
        // placeholders where a pass reads none.
        let group = |src: &wgpu::TextureView, vis: &wgpu::TextureView, slot: usize| {
            cx.group(
                "lens flare",
                &cx.passes.flare.layout,
                &[
                    cx.passes.flare.buffer.as_entire_binding(),
                    cx.sampler(),
                    tex(src),
                    tex(cx.depth()),
                    tex(cx.lum(slot)),
                    tex(vis),
                ],
            )
        };
        let mut groups = HashMap::new();
        // The occlusion pass writes the visibility the others read, from
        // the lit frame (the sun's disc as the sky drew it, under clouds).
        if cx.has(FramePass::FlareOcclusion) {
            groups.insert(
                FramePass::FlareOcclusion,
                cx.pair(|slot| group(cx.source(slot), black, slot)),
            );
        }
        for (p, src) in [
            (FramePass::Flare, "bloom.0"),
            (FramePass::FlareBlurX, "flare.raw"),
            (FramePass::FlareBlurY, "flare.x"),
        ] {
            if cx.has(p) {
                let (src, vis) = (
                    cx.view(src),
                    cx.graph.try_view("flare.vis").unwrap_or(black),
                );
                groups.insert(p, cx.pair(|slot| group(src, vis, slot)));
            }
        }
        LensFlare { groups }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        let flare = &cx.passes.flare;
        let (pipeline, target) = match pass {
            FramePass::FlareOcclusion => (&flare.occlusion, "flare.vis"),
            FramePass::Flare => (&flare.flare, "flare.raw"),
            FramePass::FlareBlurX => (&flare.blur_x, "flare.x"),
            FramePass::FlareBlurY => (&flare.blur_y, "flare"),
            _ => return None,
        };
        fullscreen_pass(
            encoder,
            &pass.name(),
            cx.graph.view(target),
            CLEAR,
            pipeline,
            &self.groups[&pass][cx.slot],
        );
        Some(1)
    }
}
