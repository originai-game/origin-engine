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
                tex(self
                    .persistent
                    .taa_depth
                    .as_ref()
                    .map_or(&self.passes.far_depth, |(_, view)| view)),
                tex(self
                    .graph
                    .try_view("motion-depth")
                    .unwrap_or(&self.passes.far_depth)),
            ],
        )
    }
}

pub(super) struct Temporal {
    taa: Option<Pair>,
}

impl Temporal {
    pub fn bind(cx: &BindCx) -> Temporal {
        let history = cx.history();
        // The OIT revealage TAA trusts its history less under (taa.wgsl);
        // white for the passes that do not weigh it.
        let reveal = cx
            .graph
            .try_view("oit.reveal")
            .or_else(|| cx.graph.try_view("fx.reactive"))
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
        Temporal { taa }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        let passes = cx.passes;
        Some(match pass {
            FramePass::Taa => {
                let draws = if passes.fsr.active() {
                    passes.fsr.encode(encoder)
                } else {
                    match (
                        if cx.taa_blend {
                            &passes.taa
                        } else {
                            &passes.taa_fresh
                        },
                        &cx.persistent.history,
                        &self.taa,
                    ) {
                        (Some(pipeline), Some(history), Some(groups)) => {
                            fullscreen_pass(
                                encoder,
                                "taa",
                                &history[cx.slot],
                                CLEAR,
                                pipeline.get(),
                                &groups[cx.slot],
                            );
                            1
                        }
                        _ => 0,
                    }
                };
                // Keep the depth of precisely the frame written into built-in
                // TAA's colour history. FSR 3.1 keeps its own dilated depth and
                // never reads `taa_depth`, so skip the Depth32Float copy under FSR.
                if !passes.fsr.active()
                    && let Some((_, previous_depth)) = cx.persistent.taa_depth.as_ref()
                {
                    let depth = cx.graph.view("depth").texture();
                    encoder.copy_texture_to_texture(
                        depth.as_image_copy(),
                        previous_depth.texture().as_image_copy(),
                        depth.size(),
                    );
                }
                draws
            }
            _ => return None,
        })
    }
}
