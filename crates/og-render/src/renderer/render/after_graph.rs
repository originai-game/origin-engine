//! The passes over the finished frame, the graph's last nodes, each timed
//! as the graph's passes are and listed only in a frame where it drew
//! (`Timed::WhenDrawn`), so the UI's budget (`docs/ui.md` §6.3) reads from
//! `gpuPasses`: the viewport views over the finished frame, the UI layers,
//! the glass and the lines drawn among the overlay, the overlay itself,
//! and the HDR output's encode, in that order. Everything here is drawn
//! again each frame: a display frame may skip a draw here whose pipeline is
//! not ready (gpu/deferred.rs).

use super::*;

/// Where the frame's passes after the graph draw: the output, or a view of
/// it in its other encoding, and whether the frame goes through the glass
/// source on its way.
#[derive(Clone, Copy)]
pub(super) struct FinishedFrame<'f> {
    /// The frame goes through the glass source.
    pub routed: bool,
    /// The output's sRGB view the overlay blends in (`None`: the output).
    pub overlay_view: Option<&'f wgpu::TextureView>,
    /// Where the paint pass blends encoded values: the output or its
    /// plain view (`None`: the pass's layer, drawn over the output).
    pub paint_view: Option<&'f wgpu::TextureView>,
    pub output: &'f wgpu::TextureView,
    /// The output's texture when the renderer owns it (the offscreen
    /// target): what the paint pass's layer copies to blend over.
    pub output_texture: Option<&'f wgpu::Texture>,
}

/// A frame's viewport views (split-screen, views.rs).
pub(super) struct ViewportViewsPlan<'f> {
    pub scene: &'f RenderScene,
    pub view_plans: &'f [views::ViewPlan],
    pub viewport_format: wgpu::TextureFormat,
    pub to: FinishedFrame<'f>,
}

pub(super) struct ViewportViewsPass;

impl Pass for ViewportViewsPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::ViewportViews,
        name: "views",
        needs: Caps::NONE,
    };
    type State = views::Views;
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    type BindIn<'f> = ViewportViewsPlan<'f>;
    type Plan<'f> = ViewportViewsPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.views
    }

    fn bind<'f>(
        _: &'f Renderer,
        _: &'f mut (),
        plan: Self::BindIn<'f>,
    ) -> Option<ViewportViewsPlan<'f>> {
        Some(plan)
    }

    /// Viewport views over the finished frame, under the lines and the
    /// overlay.
    fn encode<'f>(
        st: &Self::State,
        plan: &ViewportViewsPlan<'f>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let r = cx.ctx.r;
        let FinishedFrame {
            routed,
            overlay_view,
            output,
            ..
        } = plan.to;
        for p in plan
            .view_plans
            .iter()
            .filter(|p| matches!(p.dest, views::Dest::Viewport(_)))
        {
            // Into the glass source when the frame is routed through it,
            // so the blit at the end keeps them.
            let routed_view =
                r.glass_targets
                    .as_ref()
                    .filter(|_| routed)
                    .and_then(|g| match overlay_view {
                        None => Some(&g.source_view),
                        Some(_) => g.source_linear_view.as_ref(),
                    });
            let dest = routed_view.unwrap_or(overlay_view.unwrap_or(output));
            st.resolve(
                &r.device,
                &r.queue,
                cx.encoder,
                p,
                plan.scene.look.post.tonemap,
                r.display
                    .as_ref()
                    .map_or([0.0; 4], |d| d.curve(plan.scene.look.range_limit)),
                dest,
                plan.viewport_format,
                false,
            );
            cx.stats.draw_calls += 1;
        }
    }
}

pub(super) struct UiLayersPass;

impl Pass for UiLayersPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::UiLayers,
        name: "ui-layers",
        needs: Caps::NONE,
    };
    type State = ();
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    /// The scene and its overlay's frame.
    type BindIn<'f> = (&'f RenderScene, Option<&'f crate::overlay::OverlayFrame>);
    /// The scene and its overlay's frame, which has layers.
    type Plan<'f> = (&'f RenderScene, &'f crate::overlay::OverlayFrame);

    fn state(_: &Renderer) -> &Self::State {
        &()
    }

    fn bind<'f>(
        _: &'f Renderer,
        _: &'f mut (),
        (scene, overlay): Self::BindIn<'f>,
    ) -> Option<Self::Plan<'f>> {
        overlay
            .filter(|of| !of.layers.is_empty())
            .map(|of| (scene, of))
    }

    /// UI layers (src/ui/pass.rs): each drawn into its own target before
    /// the overlay that composites it.
    fn encode<'f>(_: &(), &(scene, of): &Self::Plan<'f>, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        cx.stats.draw_calls += cx.ctx.r.draw_ui_layers(cx.encoder, scene, of);
    }
}

/// A frame's overlay, and the glass and lines drawn among it.
pub(super) struct OverlayPlan<'f> {
    pub scene: &'f RenderScene,
    pub cam: &'f frame::CameraFrame,
    pub overlay_frame: Option<&'f crate::overlay::OverlayFrame>,
    pub line_frame: &'f crate::lines::LineFrame,
    pub glass_on: bool,
    pub color_filter: Option<crate::ColorFilter>,
    pub to: FinishedFrame<'f>,
}

pub(super) struct OverlayPass;

impl Pass for OverlayPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Overlay,
        name: "overlay",
        needs: Caps::NONE,
    };
    type State = ();
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    type BindIn<'f> = OverlayPlan<'f>;
    type Plan<'f> = OverlayPlan<'f>;

    fn state(_: &Renderer) -> &Self::State {
        &()
    }

    fn bind<'f>(_: &'f Renderer, _: &'f mut (), plan: Self::BindIn<'f>) -> Option<OverlayPlan<'f>> {
        Some(plan)
    }

    /// Glass over overlay content (src/glass.rs): when overlay runs come
    /// before a glass run (or a UI backdrop), the overlay draws into the
    /// glass source, the pyramid is rebuilt before each such run, and the
    /// source is copied to the output at the end. Otherwise the source is
    /// copied and blurred once, before the overlay, which draws into the
    /// output. A UI blend that reads what is under it always draws into
    /// the source, copied before it to what it reads, and so does a
    /// backdrop layer, which captures the source before its composite.
    fn encode<'f>(_: &(), plan: &OverlayPlan<'f>, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let (encoder, stats) = (&mut *cx.encoder, &mut *cx.stats);
        let &OverlayPlan {
            scene,
            cam,
            overlay_frame,
            line_frame,
            glass_on,
            color_filter,
            to:
                FinishedFrame {
                    routed,
                    overlay_view,
                    paint_view,
                    output,
                    output_texture,
                },
        } = plan;
        let runs: &[overlay::OverlayRun] = overlay_frame.map_or(&[], |f| f.main_runs());
        let captures: Vec<(usize, u32)> = overlay_frame.map_or(Vec::new(), |f| {
            runs.iter()
                .enumerate()
                .filter_map(|(i, r)| f.capture(r).map(|k| (i, k)))
                .collect()
        });
        let decode = !r.output_format.is_srgb() && r.output_format != HDR_FORMAT;
        let mut rebuilds = Vec::new();
        let mut copies = Vec::new();
        let mut layered = None;
        let mut paint_to = paint_view;
        if let (Some(g), true) = (&r.glass_targets, routed) {
            rebuilds = glass::rebuilds(
                &runs
                    .iter()
                    .map(|r| r.source.needs_pyramid())
                    .collect::<Vec<_>>(),
            );
            copies = glass::rebuilds(
                &runs
                    .iter()
                    .map(|r| r.source.needs_copy())
                    .collect::<Vec<_>>(),
            );
            // A colour filter keeps the whole overlay in the source too.
            let over_content = rebuilds.first().is_some_and(|&i| i > 0)
                || rebuilds.len() > 1
                || !copies.is_empty()
                || !captures.is_empty()
                || color_filter.is_some()
                || r.paint.splits();
            // The overlay draws into the source through the view it blends in.
            layered = match (over_content, overlay_view) {
                (false, _) => None,
                (true, None) => Some(&g.source_view),
                (true, Some(_)) => g.source_linear_view.as_ref(),
            };
            // The paint pass with it, through the view that keeps what its
            // shader encodes.
            if layered.is_some() {
                paint_to = match decode {
                    true => Some(&g.source_view),
                    false => g.source_encoded_view.as_ref(),
                };
            }
            if layered.is_none() {
                // The source as it is now to the output: filtered (only the
                // scene, where the overlay has no view into the source),
                // and the pyramid where glass reads it.
                stats.draw_calls += match &color_filter {
                    Some(f) => r.glass.filter(&r.queue, encoder, g, output, f, decode),
                    None => r.glass.blit(encoder, g, output),
                };
                if glass_on {
                    stats.draw_calls += r.glass.pyramid(&r.queue, encoder, g, decode);
                }
                rebuilds.clear();
            }
        }
        // Lines over the finished frame (debug draw, screen lines), under
        // the overlay, where the overlay draws (lines.rs).
        if line_frame.overlay.1 > 0 {
            let srgb_view = overlay_view.is_some();
            r.lines.set_overlay_uniform(
                &r.queue,
                &lines::LineUniform::new(
                    cam.view_proj,
                    cam.proj,
                    r.out_width,
                    r.out_height,
                    scene.frame.css_height,
                    !srgb_view && !r.output_format.is_srgb() && r.output_format != HDR_FORMAT,
                ),
            );
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            stats.draw_calls += r.lines.draw_overlay(encoder, target, srgb_view, line_frame);
        }
        // The overlay: over the finished frame, in the output (or its sRGB
        // view), or in the glass source when what it draws reads what is
        // under it.
        // og-ui's paint arena draws at the UI's layer: after the runs
        // under it, before a game's screen layers over it (ui/paint.rs),
        // blending encoded values: in the overlay's pass where that does
        // too, else in a pass of its own on `paint_to`, else in its layer,
        // drawn here over the target.
        let paint = r.paint.drawn() && !scene.ui.paint.is_empty();
        // A paint whose backdrops end its pass draws in passes of its own.
        let paint_shared = decode && overlay_view.is_none() && !r.paint.splits();
        fn paint_group(r: &Renderer, bind: ui::paint::PaintBind) -> &wgpu::BindGroup {
            match bind {
                ui::paint::PaintBind::Texture(t) => r.cache.sprite_groups[t.index()]
                    .as_ref()
                    .expect("made at upload"),
                // A glyph page and a group's result are the pass's own.
                ui::paint::PaintBind::White
                | ui::paint::PaintBind::Glyphs(..)
                | ui::paint::PaintBind::Group(_) => &r.sprite.white_sprite_group,
            }
        }
        let group1 = |bind| paint_group(r, bind);
        let cx = ui::paint::GroupCx {
            device: &r.device,
            queue: &r.queue,
            group1: &group1,
            layout: &r.sprite.sprite_layout,
            sampler: &r.sprite.sprite_sampler,
            materials: &plan.scene.ui.ui_materials,
            time: plan.scene.ui.paint.time,
        };
        // The groups' layers and their filters, before anything reads them.
        if paint {
            stats.draw_calls += r.paint.record_groups(encoder, &cx);
        }
        // What a backdrop reads where the paint draws in the target: the
        // glass source the frame goes through ([`PaintRenderer::reads_target`]).
        let paint_pass = |encoder: &mut crate::Encoder, view, load| {
            let under = match layered {
                Some(_) => ui::paint::Under::Target(view, (r.out_width, r.out_height)),
                None => ui::paint::Under::None,
            };
            r.paint.record_main(encoder, &cx, view, load, under)
        };
        // The pass's own layer, drawn where the paint lies among the runs.
        let paint_layer = match (paint, paint_to) {
            (true, None) => r.paint.layer(),
            _ => None,
        };
        let paint_at = overlay_frame.map_or(0, |f| f.ui_at).min(runs.len());
        let group0 = r.ui_main_group();
        if (!runs.is_empty() && group0.is_some()) || paint {
            let kind = match overlay_view {
                Some(_) => ui::pass::TargetKind::Linear,
                None => ui::pass::TargetKind::Output,
            };
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            // The texture under `target`, which the paint pass's layer
            // copies to blend over in encoded values.
            let behind = match layered {
                Some(_) => r.glass_targets.as_ref().map(|g| g.source()),
                None => output_texture,
            };
            // The runs between the pyramid's builds and the target's copies,
            // each in a pass of its own: before run `at`, what `events`
            // lists (true: a pyramid, false: a copy).
            let mut events: Vec<(usize, bool)> = rebuilds
                .iter()
                .map(|&i| (i, true))
                .chain(copies.iter().map(|&i| (i, false)))
                .collect();
            og_core::sort::sort_by_key(&mut events, |e| *e);
            let mut cuts: Vec<usize> = events
                .iter()
                .map(|e| e.0)
                .chain(captures.iter().map(|c| c.0))
                .collect();
            og_core::sort::sort_by_key(&mut cuts, |c| *c);
            if paint {
                cuts.push(paint_at);
            }
            og_core::sort::sort_by_key(&mut cuts, |c| *c);
            cuts.dedup();
            cuts.push(runs.len());
            let mut from = 0;
            let mut painted = !paint;
            for &to in &cuts {
                let paint_here = !painted && from == paint_at;
                painted |= paint_here;
                let paint_own = paint_to.filter(|_| paint_here && !paint_shared);
                if let Some(view) = paint_own {
                    stats.draw_calls += paint_pass(encoder, view, wgpu::LoadOp::Load);
                }
                let over = paint_here && paint_own.is_none() && !paint_shared;
                if let (true, Some(texture)) = (over, behind) {
                    stats.draw_calls += r.paint.copy_behind(encoder, texture);
                }
                if let (true, Some(layer)) = (paint_here, paint_layer) {
                    let clear = wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT);
                    let under = ui::paint::Under::Layer(over && behind.is_some());
                    stats.draw_calls += r.paint.record_main(encoder, &cx, layer, clear, under);
                }
                if to > from || (paint_here && paint_own.is_none()) {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("overlay"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    if paint_here && paint_shared {
                        stats.draw_calls += r.paint.draw(&mut pass, |bind| paint_group(r, bind));
                    } else if paint_here && paint_own.is_none() {
                        stats.draw_calls += r.paint.composite(&mut pass, behind.is_some());
                    }
                    if let (true, Some(group0)) = (to > from, group0) {
                        stats.draw_calls +=
                            r.draw_overlay_runs(&mut pass, scene, &runs[from..to], group0, kind);
                    }
                }
                // What the runs from `to` read, as the target stands now.
                if let Some(g) = r.glass_targets.as_ref() {
                    for &(_, pyramid) in events.iter().filter(|e| e.0 == to) {
                        stats.draw_calls += if pyramid {
                            r.glass.pyramid(&r.queue, encoder, g, decode)
                        } else {
                            r.glass.copy_behind(encoder, g)
                        };
                    }
                    if layered.is_some() {
                        for &(_, k) in captures.iter().filter(|c| c.0 == to) {
                            stats.draw_calls += r.record_capture(encoder, k);
                        }
                    }
                }
                from = to;
            }
        }
        if let (Some(g), Some(_)) = (&r.glass_targets, layered) {
            stats.draw_calls += match &color_filter {
                Some(f) => r.glass.filter(&r.queue, encoder, g, output, f, decode),
                None => r.glass.blit(encoder, g, output),
            };
        }
    }
}

pub(super) struct HdrOutputPass;

impl Pass for HdrOutputPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::HdrOutput,
        name: "hdr-output",
        needs: Caps::NONE,
    };
    type State = Option<display::Output>;
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    /// The frame drawn (the offscreen target), where it goes and how
    /// bright it may be (the scene's `range_limit`).
    type BindIn<'f> = (
        &'f wgpu::TextureView,
        &'f wgpu::TextureView,
        og_draw::RangeLimit,
    );
    /// The frame drawn (the offscreen target), the surface's view and the
    /// limit.
    type Plan<'f> = (
        &'f wgpu::TextureView,
        &'f wgpu::TextureView,
        og_draw::RangeLimit,
    );

    fn state(r: &Renderer) -> &Self::State {
        &r.display
    }

    fn bind<'f>(r: &'f Renderer, _: &'f mut (), views: Self::BindIn<'f>) -> Option<Self::Plan<'f>> {
        r.display.as_ref().map(|_| views)
    }

    fn encode<'f>(
        st: &Self::State,
        &(output, destination, limit): &Self::Plan<'f>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let display = st.as_ref().expect("planned with it");
        let r = cx.ctx.r;
        display.encode(
            &r.device,
            &r.queue,
            cx.encoder,
            (output, destination),
            limit,
        );
        cx.stats.draw_calls += 1;
    }
}
