//! The cloud pass: the main camera's clouds, marched at half the frame's
//! size each way into `clouds` (sky.wgsl `fs_clouds_low`). The main pass's
//! sky draws them from there over the sky at the frame's own size
//! (`fs_sky_look_low`): a march is the sky's whole cost, and a cloud has no
//! edge a pixel wide. The live sky's faces, the probes, the mirrors and the
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
        pass.set_pipeline(st.march.get());
        pass.set_bind_group(0, cx.frame_bind_group, &[]);
        pass.set_bind_group(1, cx.sky_group, &[]);
        pass.draw(0..3, 0..1);
        cx.stats.draw_calls += 1;
    }
}
