//! Auto-exposure: the lit image metered into a luminance, reduced to one
//! texel, and adapted towards it over time.

use super::*;

pub(super) struct Exposure {
    meter: Option<Pair>,
    reduce: Vec<wgpu::BindGroup>,
    adapt: Option<Pair>,
}

impl Exposure {
    pub fn bind(cx: &BindCx) -> Exposure {
        let passes = cx.passes;
        let auto = cx.has(FramePass::Meter);
        let source = |view: &wgpu::TextureView, label: &str| {
            cx.group(label, &passes.source_layout, &[tex(view), cx.sampler()])
        };
        let meter = auto.then(|| cx.pair(|slot| source(cx.source(slot), "meter")));
        let reduce = if auto {
            (0..meter_levels() - 1)
                .map(|i| source(cx.view(&luminance_name(i)), "reduce"))
                .collect()
        } else {
            Vec::new()
        };
        let adapt = cx.has(FramePass::Adapt).then(|| {
            cx.pair(|slot| {
                cx.group(
                    "exposure adapt",
                    &passes.adapt_layout,
                    &[
                        tex(cx.metered()),
                        tex(&passes.exposure.views[1 - slot]),
                        passes.adapt_buffer.as_entire_binding(),
                    ],
                )
            })
        });
        Exposure {
            meter,
            reduce,
            adapt,
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
            FramePass::Meter => fullscreen_pass(
                encoder,
                "meter",
                cx.graph.view("luminance.0"),
                CLEAR,
                &passes.meter,
                &self.meter.as_ref().expect("metering bound")[cx.slot],
            ),
            FramePass::Reduce(i) => fullscreen_pass(
                encoder,
                "meter reduce",
                cx.graph.view(&luminance_name(i + 1)),
                CLEAR,
                &passes.reduce,
                &self.reduce[i as usize],
            ),
            FramePass::Adapt => fullscreen_pass(
                encoder,
                "exposure adapt",
                &passes.exposure.views[cx.slot],
                CLEAR,
                &passes.adapt,
                &self.adapt.as_ref().expect("adaptation bound")[cx.slot],
            ),
            _ => return None,
        }
        Some(1)
    }
}
