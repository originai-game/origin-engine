//! The tonemap with grading, which adds the bloom, the god rays and the
//! lens flare: into the output, or into the first LDR target when passes
//! follow it (chain.rs). The bloom's passes are the same module's and
//! bind the same group (bloom.rs).

use super::*;

impl BindCx<'_> {
    /// A group of the post module: `src` under the exposure of `slot`,
    /// with the image it adds as bloom and the flare. Only the tonemap
    /// reads the flare: the bloom passes bind black there, so no texture
    /// they render into can be one they hold.
    pub fn post_group(
        &self,
        label: &str,
        src: &wgpu::TextureView,
        bloom: &wgpu::TextureView,
        slot: usize,
        flare: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.group(
            label,
            &self.passes.post_layout,
            &[
                tex(src),
                self.sampler(),
                self.passes.post_buffer.as_entire_binding(),
                tex(self.lum(slot)),
                tex(bloom),
                tex(self.lut),
                tex(self.view_of(FramePass::GodRaysResolve, "god.resolve")),
                tex(flare),
            ],
        )
    }
}

pub(super) struct Tonemap {
    groups: Pair,
}

impl Tonemap {
    pub fn bind(cx: &BindCx) -> Tonemap {
        let bloom = match cx.graph.bloom_spd() {
            Some(spd) if cx.has(FramePass::BloomUp(0)) => spd.output_view(),
            Some(spd) => spd.prefilter_view(),
            None => cx.view_of(FramePass::BloomPrefilter, "bloom.0"),
        };
        let flare = cx.view_of(FramePass::FlareBlurY, "flare");
        Tonemap {
            groups: cx.pair(|slot| cx.post_group("post", cx.source(slot), bloom, slot, flare)),
        }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        if pass != FramePass::Tonemap {
            return None;
        }
        let chained = !chain_passes(&cx.graph.passes).is_empty();
        let (target, pipeline) = match &cx.passes.tonemap_ldr {
            Some(to_ldr) if chained => (cx.graph.view(&ldr_name(0)), Some(to_ldr)),
            _ => (cx.output, cx.passes.tonemap.get()),
        };
        fullscreen_pass(
            encoder,
            "tonemap",
            target,
            CLEAR,
            pipeline,
            &self.groups[cx.slot],
        );
        Some(1)
    }
}
