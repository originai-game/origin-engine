//! The bloom: a prefilter of the lit image (with the 2D glow), a chain
//! of downsamples, and the upsamples that add each level back onto the
//! one above. The tonemap adds `bloom.0` (tonemap.rs).

use super::*;

pub(super) struct Bloom {
    prefilter: Option<Pair>,
    down: Vec<wgpu::BindGroup>,
    up: Vec<wgpu::BindGroup>,
}

impl Bloom {
    pub fn bind(cx: &BindCx) -> Bloom {
        if !cx.has(FramePass::BloomPrefilter) {
            return Bloom {
                prefilter: None,
                down: Vec::new(),
                up: Vec::new(),
            };
        }
        let black = &cx.passes.black;
        let levels = cx
            .frame
            .iter()
            .filter(|p| matches!(p, FramePass::BloomDown(_)))
            .count() as u32
            + 1;
        let glow = cx.view_of(FramePass::Glow, "glow");
        let level =
            |label: &str, i: u32| cx.post_group(label, cx.view(&bloom_name(i)), black, 0, black);
        Bloom {
            prefilter: Some(
                cx.pair(|slot| {
                    cx.post_group("bloom prefilter", cx.source(slot), glow, slot, black)
                }),
            ),
            down: (0..levels - 1).map(|i| level("bloom down", i)).collect(),
            up: (1..levels).map(|i| level("bloom up", i)).collect(),
        }
    }

    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        let passes = cx.passes;
        match pass {
            FramePass::BloomPrefilter => fullscreen_pass(
                encoder,
                "bloom prefilter",
                cx.graph.view("bloom.0"),
                CLEAR,
                &passes.prefilter,
                &self.prefilter.as_ref().expect("bloom bound")[cx.slot],
            ),
            FramePass::BloomDown(i) => fullscreen_pass(
                encoder,
                "bloom down",
                cx.graph.view(&bloom_name(i + 1)),
                CLEAR,
                &passes.down,
                &self.down[i as usize],
            ),
            FramePass::BloomUp(i) => fullscreen_pass(
                encoder,
                "bloom up",
                cx.graph.view(&bloom_name(i)),
                wgpu::LoadOp::Load,
                &passes.up,
                &self.up[i as usize],
            ),
            _ => return None,
        }
        Some(1)
    }
}
