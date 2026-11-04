//! Prepass render pass.

use super::*;

pub(super) struct PrepassPassInputs<'a, F0, F1, F2> {
    pub(super) water_draws: &'a [water::WaterDraw],
    pub(super) encoder: &'a mut Encoder,
    pub(super) object_space: bool,
    pub(super) frame_bind_group: &'a wgpu::BindGroup,
    pub(super) fx_group: &'a wgpu::BindGroup,
    pub(super) main_list: &'a DrawList,
    pub(super) culled_gpu: Option<&'a cull_gpu::GpuCulling>,
    pub(super) object_motion: bool,
    pub(super) batches: &'a [Batch],
    pub(super) stats: &'a mut FrameStats,
    pub(super) draw_skinned: &'a F0,
    pub(super) skinned_visible: &'a [Batch],
    pub(super) prepass_skinned: &'a [&'a wgpu::RenderPipeline],
    pub(super) terrain_indices: Option<&'a wgpu::Buffer>,
    pub(super) terrain_draws: &'a [terrain::TerrainDraw],
    pub(super) view_frustum: &'a Frustum,
    pub(super) scatter_batches: &'a scatter::ScatterBatches,
    pub(super) scatter_material: &'a F1,
    pub(super) scatter_mesh: &'a F2,
    pub(super) grass_frame: &'a grass::GrassFrame,
}

impl Renderer {
    pub(super) fn encode_prepass<'r>(
        &self,
        PrepassPassInputs {
            water_draws,
            encoder,
            object_space,
            frame_bind_group,
            fx_group,
            main_list,
            culled_gpu,
            object_motion,
            batches,
            stats,
            draw_skinned,
            skinned_visible,
            prepass_skinned,
            terrain_indices,
            terrain_draws,
            view_frustum,
            scatter_batches,
            scatter_material,
            scatter_mesh,
            grass_frame,
        }: PrepassPassInputs<
            '_,
            impl Fn(
                &mut wgpu::RenderPass,
                &[Batch],
                bool,
                u32,
                u32,
                &[&wgpu::RenderPipeline],
                Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
                Option<probe_select::ProbeGroups>,
            ) -> u32,
            impl Fn(usize) -> Option<&'r wgpu::BindGroup>,
            impl Fn(usize) -> Option<(&'r wgpu::Buffer, &'r wgpu::Buffer, u32)>,
        >,
    ) {
        // The water's surfaces for the AO (water.rs `draw_ao_depth`), drawn after the prepass against it.
        let ao_depth = self
            .graph
            .try_view("ao-depth")
            .filter(|_| !water_draws.is_empty());
        let attachment = |view, clear: f64| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("prepass"),
            color_attachments: &[
                attachment(self.graph.view("gbuffer"), 0.0),
                attachment(self.graph.view("lindepth"), post::FAR_DEPTH as f64),
                attachment(self.graph.view("outline-id"), 0.0),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.graph.view("pre-depth"),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    // Stylise's object pass tests against it.
                    store: if object_space || ao_depth.is_some() {
                        wgpu::StoreOp::Store
                    } else {
                        wgpu::StoreOp::Discard
                    },
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        p.set_bind_group(0, frame_bind_group, &[]);
        p.set_bind_group(2, fx_group, &[]);
        let instances = match (main_list.indirect, culled_gpu) {
            (Some(_), Some(gc)) => &gc.instances,
            _ => &self.instance_buffer,
        };
        // Last frame's transforms laid out as `instances`: GPU culling's
        // own (its camera pass compacts them), or the CPU path's.
        let prev = match (object_motion, main_list.indirect, culled_gpu) {
            (false, ..) => instances,
            (true, Some(_), Some(gc)) => &gc.previous,
            (true, ..) => &self.prev_instance_buffer,
        };
        let mut current = None;
        for &(b, first, count) in &main_list.ranges {
            let batch = &batches[b as usize];
            let key = (
                batch.side,
                batch.shader,
                batch.lean,
                batch.derivatives,
                batch.compact,
                batch.array,
                self.cache.meshes[batch.mesh].surface.is_some(),
            );
            if current != Some(key) {
                p.set_pipeline(
                    self.prepass_pipeline_of(batch, false)
                        .expect("prepared prepass"),
                );
                current = Some(key);
            }
            let m = &self.cache.materials[batch.material];
            p.set_bind_group(1, m.array.as_ref().unwrap_or(&m.bind_group), &[]);
            let mesh = &self.cache.meshes[batch.mesh];
            let start = first as u64 * INSTANCE_STRIDE;
            let end = start + count as u64 * INSTANCE_STRIDE;
            p.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
            p.set_vertex_buffer(1, instances.slice(start..end));
            p.set_vertex_buffer(2, prev.slice(start..end));
            if let Some(surface) = &mesh.surface {
                p.set_vertex_buffer(3, surface.previous.slice(..));
            }
            p.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            match (main_list.indirect, culled_gpu) {
                (Some(g), Some(gc)) => p.draw_indexed_indirect(&gc.args, gc.draw_offset(g, b)),
                _ => p.draw_indexed(0..mesh.index_count, 0, 0..count),
            }
            stats.prepass_draws += 1;
        }
        if let Some(m) = self.meshlet.as_ref().filter(|m| !m.frame.is_empty()) {
            stats.prepass_draws += m.draw(
                &mut p,
                batches,
                0,
                true,
                |b| &self.cache.materials[b.material].bind_group,
                |_, _| {},
            );
            stats.prepass_draws += m.draw_resolve(
                &mut p,
                batches,
                true,
                |b| &self.cache.materials[b.material].bind_group,
                |_, _| {},
            );
        }
        stats.prepass_draws += draw_skinned(
            &mut p,
            skinned_visible,
            false,
            1,
            3,
            prepass_skinned,
            None,
            None,
        );
        if let Some(indices) = terrain_indices {
            let prepass = terrain::TerrainPass::Prepass;
            let (n, _) = terrain::draw_levels(
                &mut p,
                terrain_draws,
                indices,
                prepass,
                |c, r| view_frustum.intersects_sphere(c, r),
                None,
            );
            stats.prepass_draws += n;
        }
        let (n, _) = self.scatter.draw(
            &mut p,
            scatter::ScatterPass::Prepass,
            scatter_batches,
            scatter_material,
            scatter_mesh,
            |c, r| view_frustum.intersects_sphere(c, r),
            None,
            None,
        );
        stats.prepass_draws += n;
        stats.prepass_draws +=
            self.grass
                .draw(&mut p, grass::GrassPass::Prepass, grass_frame, None);
        stats.draw_calls += stats.prepass_draws;
        drop(p);
        if let Some(object) = self.stylise_object.as_ref().filter(|_| object_space) {
            // Stylise's object pass (stylise_object.rs): the
            // prepass's instances again, against its depth.
            let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("stylise object"),
                color_attachments: &[attachment(self.graph.view("object-space"), 0.0)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.graph.view("pre-depth"),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            p.set_bind_group(0, frame_bind_group, &[]);
            p.set_bind_group(2, fx_group, &[]);
            let mut side = None;
            for &(b, first, count) in &main_list.ranges {
                let batch = &batches[b as usize];
                if side != Some(batch.side) {
                    p.set_pipeline(object.pipeline(batch.side, false));
                    side = Some(batch.side);
                }
                p.set_bind_group(1, &self.cache.materials[batch.material].bind_group, &[]);
                let mesh = &self.cache.meshes[batch.mesh];
                let start = first as u64 * INSTANCE_STRIDE;
                let end = start + count as u64 * INSTANCE_STRIDE;
                p.set_vertex_buffer(0, mesh.vertices.slice(..));
                p.set_vertex_buffer(1, instances.slice(start..end));
                p.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                match (main_list.indirect, culled_gpu) {
                    (Some(g), Some(gc)) => p.draw_indexed_indirect(&gc.args, gc.draw_offset(g, b)),
                    _ => p.draw_indexed(0..mesh.index_count, 0, 0..count),
                }
                stats.draw_calls += 1;
            }
            let skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
                .iter()
                .map(|b| object.pipeline(b.side, true))
                .collect();
            stats.draw_calls +=
                draw_skinned(&mut p, skinned_visible, false, 1, 3, &skinned, None, None);
        }
        if let Some(target) = self.graph.try_view("ao-depth") {
            // The AO's depth: the prepass's, then each water surface's own in front of what it covers.
            let size = wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            };
            encoder.copy_texture_to_texture(
                self.graph.view("lindepth").texture().as_image_copy(),
                self.graph.view("ao-depth").texture().as_image_copy(),
                size,
            );
            if ao_depth.is_some() {
                let mut p = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("water ao depth"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: self.graph.view("pre-depth"),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                p.set_bind_group(0, frame_bind_group, &[]);
                stats.draw_calls += self.water.draw_ao_depth(&mut p, water_draws);
            }
        }
    }
}
