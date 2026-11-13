//! The temporal resolves: TAA into its history and its sharpening (FSR
//! 3.1 in their place when the frame's quality asks for it, fsr.rs), and
//! the group SMAA T2x's resolve shares with them (chain.rs).

use super::*;

impl BindCx<'_> {
    /// A temporal resolve's group: this frame's image, the one before and
    /// the mask its history is trusted less under.
    pub fn temporal(
        &self,
        label: &str,
        current: &wgpu::TextureView,
        previous: &wgpu::TextureView,
        reactive: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.group(
            label,
            &self.passes.taa_layout,
            &[
                tex(current),
                tex(previous),
                tex(self.view("depth")),
                self.sampler(),
                self.passes.taa_buffer.as_entire_binding(),
                wgpu::BindingResource::Sampler(&self.passes.nearest_sampler),
                // The prepass's velocity and view depth TAA reprojects
                // moving objects with (placeholders without a prepass:
                // `TaaUniform::still.y` 0).
                tex(self.gbuffer()),
                tex(self.depth()),
                tex(reactive),
            ],
        )
    }
}

pub(super) struct Temporal {
    taa: Option<Pair>,
    sharpen: Option<Pair>,
}

impl Temporal {
    pub fn bind(cx: &BindCx) -> Temporal {
        let history = cx.history();
        // The OIT revealage TAA trusts its history less under (taa.wgsl);
        // white for the passes that do not weigh it.
        let reveal = cx
            .graph
            .try_view("oit.reveal")
            .filter(|_| cx.has(FramePass::Taa))
            .unwrap_or(&cx.passes.no_reactive);
        let taa = history.map(|h| {
            cx.pair(|slot| {
                cx.temporal(
                    "temporal resolve",
                    cx.view(cx.names.main),
                    &h[1 - slot],
                    reveal,
                )
            })
        });
        // The sharpening reads what TAA wrote this frame (as both images).
        let sharpen = history.filter(|_| cx.has(FramePass::TaaSharpen)).map(|h| {
            cx.pair(|slot| cx.temporal("taa sharpen", &h[slot], &h[slot], &cx.passes.no_reactive))
        });
        Temporal { taa, sharpen }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        let passes = cx.passes;
        Some(match pass {
            FramePass::Taa if passes.fsr.active() => passes.fsr.encode(encoder),
            FramePass::TaaSharpen if passes.fsr.active() => passes.fsr.encode_rcas(encoder),
            FramePass::Taa => match (&passes.taa, &cx.persistent.history, &self.taa) {
                (Some(pipeline), Some(history), Some(groups)) => {
                    fullscreen_pass(
                        encoder,
                        "taa",
                        &history[cx.slot],
                        CLEAR,
                        pipeline,
                        &groups[cx.slot],
                    );
                    1
                }
                _ => 0,
            },
            FramePass::TaaSharpen => match (&passes.taa_sharpen_pipeline, &self.sharpen) {
                (Some(pipeline), Some(groups)) => {
                    fullscreen_pass(
                        encoder,
                        "taa sharpen",
                        cx.graph.view("hdr.sharp"),
                        CLEAR,
                        pipeline,
                        &groups[cx.slot],
                    );
                    1
                }
                _ => 0,
            },
            _ => return None,
        })
    }
}
