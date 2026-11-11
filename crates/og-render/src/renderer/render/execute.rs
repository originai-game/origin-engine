//! The graph's executor: each pass of the compiled graph in order, its
//! capabilities checked against the device (`Encoder::expect`), encoded
//! from its plan (pass.rs) or by the post chain, and timed.

use super::*;

/// What the graph's passes are encoded with beyond their plans.
pub(super) struct GraphRun<'a, 'f> {
    pub ctx: &'a DrawCtx<'f>,
    pub plans: &'a FramePlans<'f>,
    pub encoder: &'a mut Encoder,
    pub stats: &'a mut FrameStats,
    pub frame_bind_group: &'f wgpu::BindGroup,
    pub scene_depth: &'f wgpu::TextureView,
    pub sky_group: &'f wgpu::BindGroup,
    pub slot: usize,
    /// FSR takes TAA's place (its passes need compute).
    pub fsr: bool,
    /// The frame goes through the glass source.
    pub routed: bool,
    pub output: &'a wgpu::TextureView,
    pub gtao: bool,
    pub pass_ms: &'a mut Vec<PassTiming>,
    pub pass_queries: &'a mut Vec<(String, std::ops::Range<u32>)>,
}

impl Renderer {
    /// Encodes the graph's passes in their order.
    pub(super) fn execute_graph(
        &self,
        GraphRun {
            ctx,
            plans,
            encoder,
            stats,
            frame_bind_group,
            scene_depth,
            sky_group,
            slot,
            fsr,
            routed,
            output,
            gtao,
            pass_ms,
            pass_queries,
        }: GraphRun<'_, '_>,
    ) {
        let clock = self.clock;
        for &pass in self.graph.order() {
            // What the pass needs here: its kind's, and what this frame's
            // instance of it adds (schedule.rs).
            encoder.expect(
                self.info.caps,
                pass.needs()
                    | plans.needs(pass)
                    | match pass {
                        FramePass::Taa | FramePass::TaaSharpen if fsr => Caps::COMPUTE,
                        _ => Caps::NONE,
                    },
            );
            let t0 = clock.map(|c| c());
            let q0 = encoder.used();
            let _span = pass.redrawn().then(FrameSpan::open);
            match pass {
                _ if plans.encode(
                    pass,
                    &mut EncodeCx {
                        ctx,
                        encoder,
                        stats,
                        frame_bind_group,
                        scene_depth,
                        sky_group,
                        slot,
                    },
                ) => {}
                post_pass => {
                    let post_output = match (&self.glass_targets, routed) {
                        (Some(g), true) => &g.source_view,
                        _ => output,
                    };
                    stats.draw_calls += self.graph.encode_post(
                        encoder,
                        &self.post,
                        &self.persistent,
                        post_pass,
                        slot,
                        post_output,
                        gtao,
                    );
                }
            }
            if let (Some(c), Some(t0)) = (clock, t0) {
                pass_ms.push(PassTiming {
                    name: pass.name(),
                    start: t0,
                    ms: (c() - t0).max(0.0),
                });
            }
            if encoder.timed() {
                pass_queries.push((pass.name(), q0..encoder.used()));
            }
        }
    }
}
