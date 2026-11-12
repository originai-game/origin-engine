//! The cloud pass: the main camera's clouds, marched at half the frame's
//! size each way into `clouds` (sky.wgsl `fs_clouds_low`). The main pass's
//! sky draws them from there over the sky at the frame's own size
//! (`fs_sky_look_low`): a march is the sky's whole cost, and a cloud has no
//! edge a pixel wide. No depth test spares the march a covered pixel as the
//! sky's own draw's did, so in a frame with a prepass it reads the frame's view depth
//! and skips the texels only covered pixels read (`clouds_covered`). The live sky's faces, the probes, the mirrors and the
//! views march at their own pixels.

use super::*;

pub(super) struct CloudsPass;

impl Pass for CloudsPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Clouds,
        name: "clouds",
        needs: Caps::NONE,
    };
    type State = crate::clouds::CloudsLow;
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    type BindIn<'f> = &'f RenderScene;
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.sky.clouds_low
    }

    /// The pass runs while the sky is the per-pixel one and has clouds.
    fn bind<'f>(_: &'f Renderer, _: &'f mut (), scene: &'f RenderScene) -> Option<()> {
        let procedural = matches!(
            scene.lighting.environment.background,
            Background::Procedural { .. }
        );
        (procedural && scene.atmosphere.clouds.is_some()).then_some(())
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        // The prepass's view depth in a frame with one: the march skips what its surfaces cover.
        let depth = r.graph.try_view("lindepth").map(|depth| {
            r.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("clouds low depth"),
                layout: &st.depth,
                entries: &[wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth),
                }],
            })
        });
        let mut pass = cx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clouds"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: r.graph.view("clouds"),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Every pixel is drawn.
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(
            if depth.is_some() {
                &st.march_depth
            } else {
                &st.march
            }
            .get(),
        );
        pass.set_bind_group(0, cx.frame_bind_group, &[]);
        pass.set_bind_group(1, cx.sky_group, &[]);
        if let Some(depth) = &depth {
            pass.set_bind_group(2, depth, &[]);
        }
        pass.draw(0..3, 0..1);
        cx.stats.draw_calls += 1;
    }
}
