//! Drawing the overlay's runs and the UI's layers.

use super::*;

impl Renderer {
    /// Draws `runs` (one target's) into `pass` with `group0` bound, in
    /// `kind`'s pipelines. Returns the draws.
    pub(crate) fn draw_overlay_runs<'p>(
        &'p self,
        pass: &mut wgpu::RenderPass<'p>,
        scene: &RenderScene,
        runs: &[OverlayRun],
        group0: &'p wgpu::BindGroup,
        kind: TargetKind,
    ) -> u32 {
        let overlay_pipes = match kind {
            TargetKind::Output => &self.overlay.output,
            TargetKind::Linear => self
                .overlay
                .linear
                .as_ref()
                .expect("an sRGB view's pipelines"),
            TargetKind::Layer => &self.overlay.layer,
        };
        let ui = self.ui.pipelines(kind);
        let materials = scene.ui.overlay.as_ref().map(|o| &o.ui.layers);
        pass.set_bind_group(0, group0, &[]);
        // A layer's blends read the copy of the layer drawn so far.
        let backdrop = match kind {
            TargetKind::Layer => self.ui.frame.layer_backdrop.as_ref(),
            _ => self.ui.frame.backdrop.as_ref(),
        }
        .unwrap_or(&self.ui.dummy_backdrop);
        let mut draws = 0;
        let sprite_group = |t: Option<crate::TextureHandle>| match t {
            None => &self.sprite.white_sprite_group,
            Some(t) => self.cache.sprite_groups[t.index()]
                .as_ref()
                .expect("made before the pass"),
        };
        for run in runs {
            let range = run.first..run.first + run.count;
            match run.source {
                OverlaySource::Sprite(_)
                | OverlaySource::Font(_)
                | OverlaySource::Glass
                | OverlaySource::Array(_) => {
                    let (pipeline, group) = match run.source {
                        OverlaySource::Sprite(t) => (&*overlay_pipes.sprite, sprite_group(t)),
                        OverlaySource::Font(f) => {
                            let font = &self.cache.fonts[f.index()];
                            // A colour page's glyphs are images.
                            let pipeline = if font.layout.color {
                                &*overlay_pipes.sprite
                            } else {
                                &*overlay_pipes.msdf
                            };
                            (pipeline, &font.bind_group)
                        }
                        OverlaySource::Glass => (
                            &*overlay_pipes.glass,
                            &self
                                .glass_targets
                                .as_ref()
                                .expect("made with the overlay's glass")
                                .overlay_group,
                        ),
                        OverlaySource::Array(a) => {
                            // A layer has no array pipelines: its arrays are the output's.
                            if kind == TargetKind::Layer {
                                continue;
                            }
                            (
                                self.sprite
                                    .sprite_arrays
                                    .as_ref()
                                    .expect("made before the pass")
                                    .overlay(kind == TargetKind::Linear),
                                &self.cache.sprite_array_groups[a.index()]
                                    .as_ref()
                                    .expect("made before the pass")
                                    .0,
                            )
                        }
                        _ => unreachable!("matched above"),
                    };
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[]);
                    let stride = crate::overlay::OVERLAY_STRIDE;
                    let start = run.first as u64 * stride;
                    pass.set_vertex_buffer(
                        0,
                        self.overlay
                            .buffer
                            .slice(start..start + run.count as u64 * stride),
                    );
                    pass.draw(0..6, 0..run.count);
                }
                OverlaySource::Ui(ShapeSource::Shape(pipe, t)) => {
                    pass.set_pipeline(match pipe {
                        ShapePipe::Normal => &ui.normal,
                        ShapePipe::Multiply => &ui.multiply,
                        ShapePipe::Screen => &ui.screen,
                        ShapePipe::Add => &ui.add,
                        ShapePipe::Destination => &ui.destination,
                        op => &ui.operators[operator(op)],
                    });
                    pass.set_bind_group(1, sprite_group(t), &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Ui(ShapeSource::Backdrop | ShapeSource::SolidBackdrop) => {
                    pass.set_pipeline(&ui.backdrop);
                    pass.set_bind_group(1, &self.sprite.white_sprite_group, &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Ui(ShapeSource::Composite(k, op)) => {
                    let Some((_, _, group)) = self.ui.frame.layers.get(&k) else {
                        continue;
                    };
                    let plain = match op {
                        // A blend reads the copy in the shader and lays the
                        // result over as the normal composite does.
                        ShapePipe::Normal | ShapePipe::Destination => &ui.composite,
                        op => &ui.composite_operators[operator(op)],
                    };
                    let pipeline = match materials
                        .and_then(|l| l.get(k as usize))
                        .map(|l| l.material)
                    {
                        Some(UiMaterial::Custom(h)) => {
                            match self.ui.custom_pipeline(scene.id(), h, kind) {
                                Some(p) => p,
                                // A material that failed (or is compiling) draws plain.
                                None => plain,
                            }
                        }
                        _ => plain,
                    };
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.draw(0..6, range);
                }
                OverlaySource::Mass(k) => {
                    let (Some(group), Some(batch)) = (
                        self.ui.frame.mass.as_ref(),
                        scene
                            .ui
                            .overlay
                            .as_ref()
                            .and_then(|o| o.ui.mass.get(k as usize)),
                    ) else {
                        continue;
                    };
                    let t = &batch.template;
                    let texture = match t.kind {
                        super::super::UiMassKind::Number { .. }
                        | super::super::UiMassKind::Text { .. } => {
                            t.font.map(|f| &self.cache.fonts[f.index()].bind_group)
                        }
                        _ => None,
                    }
                    .unwrap_or_else(|| sprite_group(t.texture));
                    let stride = std::mem::size_of::<super::super::UiMassInstance>() as u64;
                    pass.set_pipeline(&ui.mass);
                    pass.set_bind_group(1, texture, &[]);
                    pass.set_bind_group(2, group, &[]);
                    pass.set_vertex_buffer(
                        0,
                        self.ui.mass_instances.slice(
                            run.first as u64 * stride..(run.first + run.count) as u64 * stride,
                        ),
                    );
                    pass.draw(0..6 * t.quads(), 0..run.count);
                }
                OverlaySource::Vector(h) => {
                    let Some(mesh) = self.ui.vectors.get(&(scene.id(), h.index())) else {
                        continue;
                    };
                    pass.set_pipeline(&ui.vector);
                    pass.set_bind_group(1, &self.sprite.white_sprite_group, &[]);
                    pass.set_bind_group(2, backdrop, &[]);
                    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..mesh.count, 0, range);
                }
            }
            draws += 1;
        }
        draws
    }

    /// Draws the frame's UI layers into their targets, innermost first.
    pub(crate) fn draw_ui_layers(
        &self,
        encoder: &mut crate::Encoder,
        scene: &RenderScene,
        frame: &OverlayFrame,
    ) -> u32 {
        let mut draws = 0;
        for plan in &frame.layers {
            // A backdrop runs where its composite draws (`record_capture`).
            if frame.backdrops.contains(&plan.index) {
                continue;
            }
            let Some((group0, target, _)) = self.ui.frame.layers.get(&plan.index) else {
                continue;
            };
            let view = match plan.target {
                Some(t) => &self.cache.textures[t].view,
                None => &self.ui.targets[*target].view,
            };
            // The runs between the copies its blends read and its
            // backdrops' captures, each in a pass of its own (the first
            // clears the target).
            let runs = frame.layer_runs(plan.index);
            let mut cuts: Vec<usize> = (0..runs.len())
                .filter(|&i| runs[i].source.needs_copy() || frame.capture(&runs[i]).is_some())
                .collect();
            cuts.push(runs.len());
            let mut from = 0;
            let mut first = true;
            for to in cuts {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ui layer"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                first = false;
                draws += self.draw_overlay_runs(
                    &mut pass,
                    scene,
                    &runs[from..to],
                    group0,
                    TargetKind::Layer,
                );
                drop(pass);
                if let Some(k) = runs.get(to).and_then(|r| frame.capture(r)) {
                    draws += self.record_capture(encoder, k);
                }
                if runs.get(to).is_some_and(|r| r.source.needs_copy())
                    && let (Some((behind, _)), None) = (&self.ui.layer_behind, plan.target)
                {
                    encoder.copy_texture_to_texture(
                        self.ui.targets[*target].texture.as_image_copy(),
                        behind.as_image_copy(),
                        wgpu::Extent3d {
                            width: plan.size.0,
                            height: plan.size.1,
                            depth_or_array_layers: 1,
                        },
                    );
                    draws += 1;
                }
                from = to;
            }
            if let Some(p) = self.ui.frame.filters.get(&plan.index) {
                draws += self.ui.filters.record(encoder, p);
            }
            // A blur's levels, from what the layer drew.
            if plan.target.is_none()
                && let Some(chain) = &self.ui.targets[*target].chain
            {
                self.mips.record_chain(encoder, chain);
                draws += chain.level_count() - 1;
            }
        }
        draws
    }

    /// Records backdrop layer `k`'s capture of what is under it and its
    /// filter, before its composite draws. Returns the passes.
    pub(crate) fn record_capture(&self, encoder: &mut crate::Encoder, k: u32) -> u32 {
        self.ui
            .frame
            .filters
            .get(&k)
            .map_or(0, |p| self.ui.filters.record(encoder, p))
    }

    /// The frame's group 0 for the output.
    pub(crate) fn ui_main_group(&self) -> Option<&wgpu::BindGroup> {
        self.ui.frame.main.as_ref()
    }

    /// The custom UI materials ([`RenderScene::add_ui_material`]) that
    /// failed to compile on the device, with its message; they composite
    /// as the plain layer.
    pub fn ui_material_errors(
        &self,
        scene: &RenderScene,
    ) -> Vec<(super::super::UiMaterialHandle, String)> {
        self.ui.errors(scene.id())
    }
}
