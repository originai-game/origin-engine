//! The bloom: a prefilter of the lit image (with the 2D glow), a chain
//! of downsamples, and the upsamples that add each level back onto the
//! one above. The tonemap adds `bloom.0` (tonemap.rs).

use super::*;

struct ComputeBloom {
    down: Pair,
    tail: Option<wgpu::BindGroup>,
    ups: Vec<(wgpu::BindGroup, (u32, u32))>,
}

pub(super) struct Bloom {
    compute: Option<ComputeBloom>,
    prefilter: Option<Pair>,
    down: Vec<wgpu::BindGroup>,
    up: Vec<wgpu::BindGroup>,
}

impl Bloom {
    pub fn bind(cx: &BindCx) -> Bloom {
        if !cx.has(FramePass::BloomPrefilter) {
            return Bloom {
                compute: None,
                prefilter: None,
                down: Vec::new(),
                up: Vec::new(),
            };
        }
        let glow = cx.view_of(FramePass::Glow, "glow");
        if let Some(spd) = cx.graph.bloom_spd() {
            let down = cx.pair(|slot| {
                cx.passes.bloom_spd.bind_down(
                    cx.device,
                    &cx.passes.post_buffer,
                    &cx.passes.sampler,
                    cx.source(slot),
                    cx.lum(slot),
                    glow,
                    spd,
                )
            });
            let tail = cx
                .passes
                .bloom_spd
                .bind_tail(cx.device, &cx.passes.sampler, spd);
            let ups = if cx.has(FramePass::BloomUp(0)) {
                cx.passes.bloom_spd.bind_ups(
                    cx.device,
                    &cx.passes.post_buffer,
                    &cx.passes.sampler,
                    spd,
                )
            } else {
                Vec::new()
            };
            return Bloom {
                compute: Some(ComputeBloom { down, tail, ups }),
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
        let level =
            |label: &str, i: u32| cx.post_group(label, cx.view(&bloom_name(i)), black, 0, black);
        Bloom {
            compute: None,
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
        if let Some(compute) = &self.compute {
            let spd = cx.graph.bloom_spd().expect("bloom spd realized");
            return match pass {
                FramePass::BloomPrefilter => Some(passes.bloom_spd.encode_down(
                    encoder,
                    &compute.down[cx.slot],
                    compute.tail.as_ref(),
                    spd,
                )),
                FramePass::BloomUp(_) => Some(passes.bloom_spd.encode_up(encoder, &compute.ups)),
                _ => None,
            };
        }
        match pass {
            FramePass::BloomPrefilter => fullscreen_pass(
                encoder,
                "bloom prefilter",
                cx.graph.view("bloom.0"),
                CLEAR,
                passes.prefilter.get(),
                &self.prefilter.as_ref().expect("bloom bound")[cx.slot],
            ),
            FramePass::BloomDown(i) => fullscreen_pass(
                encoder,
                "bloom down",
                cx.graph.view(&bloom_name(i + 1)),
                CLEAR,
                passes.down.get(),
                &self.down[i as usize],
            ),
            FramePass::BloomUp(i) => fullscreen_pass(
                encoder,
                "bloom up",
                cx.graph.view(&bloom_name(i)),
                wgpu::LoadOp::Load,
                passes.up.get(),
                &self.up[i as usize],
            ),
            _ => return None,
        }
        Some(1)
    }
}
