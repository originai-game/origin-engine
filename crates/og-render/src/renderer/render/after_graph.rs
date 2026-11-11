//! The passes after the graph's, each timed as the graph's passes are and
//! listed only in a frame where it drew: the viewport views over the
//! finished frame, the UI layers, the glass and the lines drawn among the
//! overlay, the overlay itself, and the HDR output's encode, in that order.
//! Everything here is drawn again each frame (FrameSpan).

use super::*;

pub(super) struct EncodeAfterGraphInputs<'a> {
    pub scene: &'a RenderScene,
    pub target: &'a RenderTarget<'a>,
    pub cam: &'a frame::CameraFrame,
    pub encoder: &'a mut Encoder,
    pub stats: &'a mut FrameStats,
    pub clock: Option<fn() -> f64>,
    pub pass_ms: Vec<PassTiming>,
    pub pass_queries: Vec<(String, std::ops::Range<u32>)>,
    pub view_plans: &'a [views::ViewPlan],
    pub viewport_format: wgpu::TextureFormat,
    pub overlay_frame: &'a Option<crate::overlay::OverlayFrame>,
    pub line_frame: &'a crate::lines::LineFrame,
    pub routed: bool,
    pub glass_on: bool,
    pub color_filter: Option<crate::ColorFilter>,
    pub overlay_view: Option<&'a wgpu::TextureView>,
    pub output: &'a wgpu::TextureView,
    pub destination: &'a wgpu::TextureView,
}

pub(super) struct EncodeAfterGraphOutputs {
    pub pass_ms: Vec<PassTiming>,
    pub pass_queries: Vec<(String, std::ops::Range<u32>)>,
}

impl Renderer {
    /// Encodes the passes after the graph's; returns the frame's pass timings.
    #[allow(clippy::too_many_lines)]
    pub(super) fn encode_after_graph(
        &self,
        EncodeAfterGraphInputs {
            scene,
            target,
            cam,
            encoder,
            stats,
            clock,
            pass_ms,
            pass_queries,
            view_plans,
            viewport_format,
            overlay_frame,
            line_frame,
            routed,
            glass_on,
            color_filter,
            overlay_view,
            output,
            destination,
        }: EncodeAfterGraphInputs<'_>,
    ) -> EncodeAfterGraphOutputs {
        // The passes after the graph's are timed as its own are, each listed
        // only in a frame where it drew: the viewport views, the UI layers,
        // and the overlay with the glass and the lines drawn among it, so
        // the UI's budget (`docs/ui.md` §6.3) reads from `gpuPasses`.
        let mut timings = Timings {
            clock,
            pass_ms,
            pass_queries,
        };
        // Everything after the graph is drawn again each frame (FrameSpan).
        let tail = FrameSpan::open();
        let views_at = timings.begin(encoder, stats);
        // Viewport views (split-screen, views.rs) over the finished frame,
        // under the lines and the overlay.
        for p in view_plans
            .iter()
            .filter(|p| matches!(p.dest, views::Dest::Viewport(_)))
        {
            // Into the glass source when the frame is routed through it,
            // so the blit at the end keeps them.
            let routed_view = self
                .glass_targets
                .as_ref()
                .filter(|_| routed)
                .and_then(|g| match overlay_view {
                    None => Some(&g.source_view),
                    Some(_) => g.source_linear_view.as_ref(),
                });
            let dest = routed_view.unwrap_or(overlay_view.unwrap_or(output));
            self.views.resolve(
                &self.device,
                &self.queue,
                encoder,
                p,
                scene.look.post.tonemap,
                self.hdr.as_ref().map_or([0.0; 4], |h| h.curve),
                dest,
                viewport_format,
                false,
            );
            stats.draw_calls += 1;
        }
        timings.end("views", views_at, encoder, stats);
        let layers_at = timings.begin(encoder, stats);
        // UI layers (src/ui/pass.rs): each drawn into its own target before
        // the overlay that composites it.
        if let Some(of) = &overlay_frame
            && !of.layers.is_empty()
        {
            stats.draw_calls += self.draw_ui_layers(encoder, scene, of);
        }
        timings.end("ui-layers", layers_at, encoder, stats);
        let overlay_at = timings.begin(encoder, stats);
        // Glass over overlay content (src/glass.rs): when overlay runs come
        // before a glass run (or a UI backdrop), the overlay draws into the
        // glass source, the pyramid is rebuilt before each such run, and the
        // source is copied to the output at the end. Otherwise the source is
        // copied and blurred once, before the overlay, which draws into the
        // output. A UI blend that reads what is under it always draws into
        // the source, copied before it to what it reads, and so does a
        // backdrop layer, which captures the source before its composite.
        let runs: &[overlay::OverlayRun] = overlay_frame.as_ref().map_or(&[], |f| f.main_runs());
        let captures: Vec<(usize, u32)> = overlay_frame.as_ref().map_or(Vec::new(), |f| {
            runs.iter()
                .enumerate()
                .filter_map(|(i, r)| f.capture(r).map(|k| (i, k)))
                .collect()
        });
        let decode = !self.output_format.is_srgb() && self.output_format != HDR_FORMAT;
        let mut rebuilds = Vec::new();
        let mut copies = Vec::new();
        let mut layered = None;
        if let (Some(g), true) = (&self.glass_targets, routed) {
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
                || color_filter.is_some();
            // The overlay draws into the source through the view it blends in.
            layered = match (over_content, overlay_view) {
                (false, _) => None,
                (true, None) => Some(&g.source_view),
                (true, Some(_)) => g.source_linear_view.as_ref(),
            };
            if layered.is_none() {
                // The source as it is now to the output: filtered (only the
                // scene, where the overlay has no view into the source),
                // and the pyramid where glass reads it.
                stats.draw_calls += match &color_filter {
                    Some(f) => self
                        .glass
                        .filter(&self.queue, encoder, g, output, f, decode),
                    None => self.glass.blit(encoder, g, output),
                };
                if glass_on {
                    stats.draw_calls += self.glass.pyramid(&self.queue, encoder, g, decode);
                }
                rebuilds.clear();
            }
        }
        // Lines over the finished frame (debug draw, screen lines), under
        // the overlay, where the overlay draws (lines.rs).
        if line_frame.overlay.1 > 0 {
            let srgb_view = overlay_view.is_some();
            self.lines.set_overlay_uniform(
                &self.queue,
                &lines::LineUniform::new(
                    cam.view_proj,
                    cam.proj,
                    self.out_width,
                    self.out_height,
                    scene.frame.css_height,
                    !srgb_view && !self.output_format.is_srgb() && self.output_format != HDR_FORMAT,
                ),
            );
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
            stats.draw_calls += self
                .lines
                .draw_overlay(encoder, target, srgb_view, line_frame);
        }
        // The overlay: over the finished frame, in the output (or its sRGB
        // view), or in the glass source when what it draws reads what is
        // under it.
        if let (false, Some(group0)) = (runs.is_empty(), self.ui_main_group()) {
            let kind = match overlay_view {
                Some(_) => ui::pass::TargetKind::Linear,
                None => ui::pass::TargetKind::Output,
            };
            let target = layered.unwrap_or(overlay_view.unwrap_or(output));
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
            cuts.dedup();
            cuts.push(runs.len());
            let mut from = 0;
            for &to in &cuts {
                if to > from {
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
                    stats.draw_calls +=
                        self.draw_overlay_runs(&mut pass, scene, &runs[from..to], group0, kind);
                }
                // What the runs from `to` read, as the target stands now.
                if let Some(g) = self.glass_targets.as_ref() {
                    for &(_, pyramid) in events.iter().filter(|e| e.0 == to) {
                        stats.draw_calls += if pyramid {
                            self.glass.pyramid(&self.queue, encoder, g, decode)
                        } else {
                            self.glass.copy_behind(encoder, g)
                        };
                    }
                    if layered.is_some() {
                        for &(_, k) in captures.iter().filter(|c| c.0 == to) {
                            stats.draw_calls += self.record_capture(encoder, k);
                        }
                    }
                }
                from = to;
            }
        }
        if let (Some(g), Some(_)) = (&self.glass_targets, layered) {
            stats.draw_calls += match &color_filter {
                Some(f) => self
                    .glass
                    .filter(&self.queue, encoder, g, output, f, decode),
                None => self.glass.blit(encoder, g, output),
            };
        }
        timings.end("overlay", overlay_at, encoder, stats);
        if let Some(hdr) = &self.hdr
            && !matches!(target, RenderTarget::Offscreen)
        {
            let at = timings.begin(encoder, stats);
            hdr.encode(&self.device, encoder, output, destination);
            stats.draw_calls += 1;
            timings.end("hdr-output", at, encoder, stats);
        }
        drop(tail);
        let Timings {
            pass_ms,
            pass_queries,
            ..
        } = timings;
        EncodeAfterGraphOutputs {
            pass_ms,
            pass_queries,
        }
    }
}
