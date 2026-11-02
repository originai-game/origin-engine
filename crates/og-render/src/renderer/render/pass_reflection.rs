//! Reflection render pass.

use super::*;

pub(super) struct ReflectionPassInputs<'a, F0, F1> {
    pub(super) k: u8,
    pub(super) scene: &'a RenderScene,
    pub(super) encoder: &'a mut Encoder,
    pub(super) reflection_frame_group: &'a [wgpu::BindGroup],
    pub(super) reflection_fx_group: &'a [wgpu::BindGroup],
    pub(super) reflection_list: &'a [DrawList],
    pub(super) draw_list: &'a F0,
    pub(super) stats: &'a mut FrameStats,
    pub(super) draw_skinned: &'a F1,
    pub(super) skinned_visible: &'a [Batch],
    pub(super) lit_skinned_standard: &'a [&'a wgpu::RenderPipeline],
    pub(super) terrain_indices: Option<&'a wgpu::Buffer>,
    #[allow(clippy::type_complexity)]
    pub(super) reflections: &'a [(usize, Camera, frame::CameraFrame, Mat4, (u32, u32))],
    pub(super) terrain_draws: &'a [terrain::TerrainDraw],
    pub(super) sky_group: &'a wgpu::BindGroup,
    pub(super) reflection_blended: &'a [DrawList],
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'a [u32],
    pub(super) batches: &'a [Batch],
}

impl Renderer {
    pub(super) fn encode_reflection(
        &self,
        ReflectionPassInputs {
            k,
            scene,
            encoder,
            reflection_frame_group,
            reflection_fx_group,
            reflection_list,
            draw_list,
            stats,
            draw_skinned,
            skinned_visible,
            lit_skinned_standard,
            terrain_indices,
            reflections,
            terrain_draws,
            sky_group,
            reflection_blended,
            meshlet_views,
            batches,
        }: ReflectionPassInputs<
            '_,
            impl Fn(
                &mut wgpu::RenderPass,
                &DrawList,
                bool,
                &mut Option<LitDrawKey>,
                Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
                Option<probe_select::ProbeGroups>,
                Option<&wgpu::BindGroup>,
            ),
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
        >,
    ) {
        let k = k as usize;
        let clear = match scene.environment.background {
            Background::Color(c) => wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: 1.0,
            },
            Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
        };
        let target = self.graph.view(post::reflection_target(k as u8, ""));
        let (view, resolve_target) = if self.samples > 1 {
            (
                self.graph.view(post::reflection_target(k as u8, "-msaa")),
                Some(target),
            )
        } else {
            (target, None)
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("planar reflection"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: if resolve_target.is_some() {
                        wgpu::StoreOp::Discard
                    } else {
                        wgpu::StoreOp::Store
                    },
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.graph.view(post::reflection_target(k as u8, "-depth")),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pbr_pipelines[&(Side::Front, false, false, false)]);
        pass.set_bind_group(0, &reflection_frame_group[k], &[]);
        pass.set_bind_group(2, &reflection_fx_group[k], &[]);
        let list = &reflection_list[k];
        draw_list(
            &mut pass,
            list,
            true,
            &mut Some((
                Side::Front,
                false,
                false,
                None,
                false,
                false,
                false,
                false,
                false,
            )),
            None,
            None,
            Some(&reflection_fx_group[k]),
        );
        stats.draw_calls += list.ranges.len() as u32;
        stats.reflection_draws += list.ranges.len() as u32;
        let n = self.draw_meshlets(&mut pass, batches, meshlet_views[k]);
        stats.draw_calls += n;
        stats.reflection_draws += n;
        let n = draw_skinned(
            &mut pass,
            skinned_visible,
            false,
            1,
            3,
            lit_skinned_standard,
            None,
            None,
        );
        stats.draw_calls += n;
        stats.reflection_draws += n;
        if let (Some(indices), Some(r)) = (terrain_indices, reflections.get(k)) {
            let frustum = Frustum::from_view_proj(r.3);
            let (n, _) = terrain::draw_levels(
                &mut pass,
                terrain_draws,
                indices,
                terrain::TerrainPass::Lit,
                |c, rad| frustum.intersects_sphere(c, rad),
                None,
            );
            stats.draw_calls += n;
            stats.reflection_draws += n;
        }
        let sky = match scene.environment.background {
            Background::Sky => Some(&self.sky_pipeline),
            Background::Procedural { .. } => Some(&self.sky_look_pipeline),
            Background::Color(_) => None,
        };
        if let Some(sky) = sky {
            pass.set_pipeline(sky);
            pass.set_bind_group(1, sky_group, &[]);
            pass.draw(0..3, 0..1);
            stats.draw_calls += 1;
            stats.reflection_draws += 1;
        }
        // Transparent meshes over the mirrored scene and sky, back to front.
        pass.set_bind_group(0, &reflection_frame_group[k], &[]);
        pass.set_bind_group(2, &reflection_fx_group[k], &[]);
        let list = &reflection_blended[k];
        draw_list(
            &mut pass,
            list,
            true,
            &mut None,
            None,
            None,
            Some(&reflection_fx_group[k]),
        );
        stats.draw_calls += list.ranges.len() as u32;
        stats.reflection_draws += list.ranges.len() as u32;
    }
}
