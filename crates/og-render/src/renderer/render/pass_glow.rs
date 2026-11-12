//! The glow pass: the glowing sprites' runs drawn into the glow target,
//! which bloom spreads.

use super::*;

pub(super) struct GlowPass;

impl Pass for GlowPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Glow,
        name: "glow",
        needs: Caps::NONE,
    };
    type State = ();
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    /// Whether the look glows, and the sprites' glowing runs.
    type BindIn<'f> = (bool, &'f [sprites::SpriteRun]);
    /// The glowing sprites' runs.
    type Plan<'f> = &'f [sprites::SpriteRun];

    fn state(_: &Renderer) -> &Self::State {
        &()
    }

    fn bind<'f>(
        _: &'f Renderer,
        _: &'f mut (),
        (glow, runs): Self::BindIn<'f>,
    ) -> Option<&'f [sprites::SpriteRun]> {
        glow.then_some(runs)
    }

    fn encode<'f>(
        _: &(),
        runs: &&'f [sprites::SpriteRun],
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let r = cx.ctx.r;
        let mut pass = cx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("glow"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: r.graph.view("glow"),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, cx.frame_bind_group, &[]);
        cx.stats.draw_calls += cx.ctx.draw_glow(&mut pass, runs);
    }
}
