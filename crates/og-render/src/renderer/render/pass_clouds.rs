//! The cloud pass: the main camera's clouds, marched at half the frame's
//! size each way into `clouds` (sky.wgsl `fs_clouds_low`). A frame that has
//! the frame before's marches one texel of each two by two into
//! `clouds.quarter` and resolves the history reprojected round it into
//! `clouds` (`fs_clouds_resolve`, clouds.rs `CloudsTemporal`), then copies
//! `clouds` into the history for the next. The main pass's
//! sky draws them from there over the sky at the frame's own size
//! (`fs_sky_look_low`): a march is the sky's whole cost, and a cloud has no
//! edge a pixel wide. No depth test spares the march a covered pixel as the
//! sky's own draw's did, so in a frame with a prepass it reads the frame's view depth
//! and skips the texels only covered pixels read (`clouds_covered`). The live sky's faces, the probes, the mirrors and the
//! views march at their own pixels.

use super::*;

pub(super) struct CloudsPass;

impl CloudsPass {
    /// Clouds are a layer over any background, except when no sky draws
    /// (camera plate, flat colour) or a grounded sky draws clouds at full res.
    pub(super) fn runs(scene: &RenderScene) -> bool {
        scene.atmosphere.clouds.is_some()
            && scene.frame.plate.is_none()
            && !matches!(scene.lighting.environment.background, Background::Color(_))
            && !(matches!(scene.lighting.environment.background, Background::Sky)
                && ground_heights::wanted(scene))
    }
}

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

    fn bind<'f>(_: &'f Renderer, _: &'f mut (), scene: &'f RenderScene) -> Option<()> {
        CloudsPass::runs(scene).then_some(())
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let t = &st.temporal;
        let step = wgpu::BindGroupEntry {
            binding: 2,
            resource: t.uniform.as_entire_binding(),
        };
        // The prepass's view depth in a frame with one: the march skips what its surfaces cover.
        let depth = r.graph.try_view("lindepth");
        let march_group = match depth {
            Some(depth) => r.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("clouds low depth"),
                layout: &t.depth,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    step.clone(),
                ],
            }),
            None => r.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("clouds low march"),
                layout: &t.march,
                entries: std::slice::from_ref(&step),
            }),
        };
        let target = r.graph.view("clouds");
        let quarter = r.graph.view("clouds.quarter");
        let (history, history_view) = t.history();
        let draw = |cx: &mut EncodeCx<'_, 'f>,
                    view: &wgpu::TextureView,
                    pipeline: &wgpu::RenderPipeline,
                    group: &wgpu::BindGroup,
                    label: &str| {
            let mut pass = cx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, cx.frame_bind_group, &[]);
            pass.set_bind_group(1, cx.sky_group, &[]);
            pass.set_bind_group(2, group, &[]);
            pass.draw(0..3, 0..1);
            cx.stats.draw_calls += 1;
        };
        let (Some(march), Some(resolve)) = (
            if depth.is_some() {
                &st.march_depth
            } else {
                &st.march
            }
            .get(),
            st.resolve.get(),
        ) else {
            return;
        };
        if t.reproject {
            // One texel of each two by two marched, then the history reprojected round it.
            draw(cx, quarter, march, &march_group, "clouds");
            let resolve_group = r.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("clouds low resolve"),
                layout: &t.resolve,
                entries: &[
                    step,
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(quarter),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(history_view),
                    },
                ],
            });
            draw(cx, target, resolve, &resolve_group, "clouds resolve");
        } else {
            draw(cx, target, march, &march_group, "clouds");
        }
        // The next frame's history.
        cx.encoder.copy_texture_to_texture(
            target.texture().as_image_copy(),
            history.as_image_copy(),
            history.size(),
        );
    }
}
