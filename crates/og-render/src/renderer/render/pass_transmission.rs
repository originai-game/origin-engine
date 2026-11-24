//! Capture the visible volumes' exit surfaces after the opaque scene.

use super::*;

impl Renderer {
    pub(super) fn encode_transmission_surfaces(
        &self,
        plan: &MainPlan<'_>,
        ctx: &DrawCtx<'_>,
        encoder: &mut crate::Encoder,
        stats: &mut FrameStats,
    ) {
        // The visible draw list already instances compatible neighbours.
        // Thin walls retain their scene copy but need no geometric exits.
        let mut ranges = plan
            .blended_list
            .ranges
            .iter()
            .copied()
            .filter(|&(b, _, _)| self.cache.materials[plan.batches[b as usize].material].volumetric)
            .peekable();
        if ranges.peek().is_none()
            && !plan
                .skinned_visible
                .iter()
                .any(|b| self.cache.materials[b.material].volumetric && b.crowd.is_none())
        {
            return;
        }
        let volume = self
            .transmission_volume
            .as_ref()
            .expect("visible transmission");
        volume.seed_depth(
            &self.device,
            encoder,
            self.graph.view("depth"),
            plan.frame_bind_group,
        );
        stats.draw_calls += 1;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("transmission exits"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &volume.exits,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &volume.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(frame::projection::DEPTH_CLEAR),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, plan.frame_bind_group, &[]);
        pass.set_bind_group(2, plan.backface_fx.expect("exit capture group"), &[]);
        pass.set_vertex_buffer(1, self.instances.instance_buffer.slice(..));
        for (b, first, count) in ranges {
            let batch = &plan.batches[b as usize];
            let key = (batch.shader, batch.faces, false, batch.effects);
            pass.set_pipeline(&volume.pipelines[&key]);
            pass.set_bind_group(
                1,
                self.cache.materials[batch.material]
                    .physical
                    .as_ref()
                    .expect("physical material"),
                &[],
            );
            let mesh = &self.cache.meshes[batch.mesh];
            pass.set_vertex_buffer(0, mesh.vertex_slice(false));
            pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..mesh.index_count, 0, first..first + count);
            stats.draw_calls += 1;
        }
        if let Some(palette) = ctx.palette_group {
            let indirect = plan
                .main_list
                .indirect
                .filter(|_| ctx.skinned_gpu)
                .zip(ctx.culled_gpu);
            for (k, batch) in
                plan.skinned_visible.iter().enumerate().filter(|(_, b)| {
                    self.cache.materials[b.material].volumetric && b.crowd.is_none()
                })
            {
                let key = (batch.shader, batch.faces, true, batch.effects);
                pass.set_pipeline(&volume.pipelines[&key]);
                pass.set_bind_group(
                    1,
                    self.cache.materials[batch.material]
                        .physical
                        .as_ref()
                        .expect("physical material"),
                    &[],
                );
                pass.set_bind_group(3, palette, &[]);
                let (records, first) = match indirect {
                    Some((g, gc)) => (&gc.skinned, gc.skinned_region(g, batch.first)),
                    None => (&self.instances.skinned_buffer, batch.first),
                };
                let start = first as u64 * skin::SKINNED_INSTANCE_STRIDE;
                let end = start + batch.count as u64 * skin::SKINNED_INSTANCE_STRIDE;
                pass.set_vertex_buffer(1, records.slice(start..end));
                let mesh = &self.cache.meshes[batch.mesh];
                pass.set_vertex_buffer(0, mesh.vertex_slice(false));
                pass.set_vertex_buffer(
                    2,
                    self.cache.skins[batch.mesh]
                        .as_ref()
                        .expect("skinned mesh")
                        .slice(..),
                );
                pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                match indirect {
                    Some((g, gc)) => {
                        let (args, at) = gc.skinned_draw_args(g, k as u32);
                        pass.draw_indexed_indirect(args, at);
                    }
                    None => pass.draw_indexed(0..mesh.index_count, 0, 0..batch.count),
                }
                stats.draw_calls += 1;
            }
        }
    }
}
