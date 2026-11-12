//! Reflection render pass.
//!
//! The planar reflections' lists: each mirror's opaque meshes, the first
//! through the GPU cull's reflection pass, the rest culled on the CPU, and
//! its transparent meshes back to front from the mirrored eye.

use super::*;

pub(super) struct ReflectionPlan<'f> {
    pub(super) scene: &'f RenderScene,
    pub(super) reflection_frame_group: &'f [wgpu::BindGroup],
    pub(super) reflection_fx_group: &'f [wgpu::BindGroup],
    pub(super) reflection_list: &'f [DrawList],
    pub(super) skinned_visible: &'f [Batch],
    pub(super) lit_skinned_standard: &'f [&'f wgpu::RenderPipeline],
    pub(super) terrain_indices: Option<&'f wgpu::Buffer>,
    #[allow(clippy::type_complexity)]
    pub(super) reflections: &'f [(usize, Camera, frame::CameraFrame, Mat4, (u32, u32))],
    pub(super) terrain_draws: &'f [terrain::TerrainDraw],
    pub(super) reflection_blended: &'f [DrawList],
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'f [u32],
    pub(super) batches: &'f [Batch],
}

impl Renderer {
    pub(super) fn encode_reflection(
        &self,
        plan: &ReflectionPlan<'_>,
        k: u8,
        cx: &mut EncodeCx<'_, '_>,
    ) {
        let &ReflectionPlan {
            scene,
            reflection_frame_group,
            reflection_fx_group,
            reflection_list,
            skinned_visible,
            lit_skinned_standard,
            terrain_indices,
            reflections,
            terrain_draws,
            reflection_blended,
            meshlet_views,
            batches,
        } = plan;
        let (ctx, encoder, stats, sky_group) =
            (cx.ctx, &mut *cx.encoder, &mut *cx.stats, cx.sky_group);
        let k = k as usize;
        let clear = match scene.lighting.environment.background {
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
        pass.set_bind_group(0, &reflection_frame_group[k], &[]);
        pass.set_bind_group(2, &reflection_fx_group[k], &[]);
        let list = &reflection_list[k];
        ctx.draw_list(
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
        let n = self.draw_meshlets(&mut pass, batches, meshlet_views[k]);
        stats.draw_calls += n;
        stats.reflection_draws += n;
        let n = ctx.draw_skinned(
            &mut pass,
            skinned_visible,
            false,
            1,
            3,
            lit_skinned_standard,
            None,
            None,
            list.indirect,
        );
        stats.draw_calls += n;
        stats.reflection_draws += n;
        if let (Some(indices), Some(r)) = (terrain_indices, reflections.get(k)) {
            let frustum = Frustum::from_view_proj(r.3);
            pass.set_bind_group(3, self.world.terrain.decals.group(false), &[]);
            let (n, _) = terrain::draw_levels(
                &mut pass,
                terrain_draws,
                indices,
                terrain::TerrainPass::Lit,
                &frustum,
                None,
            );
            stats.draw_calls += n;
            stats.reflection_draws += n;
        }
        let sky = match scene.lighting.environment.background {
            Background::Sky => Some(&self.sky.sky_pipeline),
            Background::Procedural { .. } => Some(&self.sky.sky_look_pipeline),
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
        ctx.draw_list(
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

pub(super) struct ReflectionPass;

impl Pass for ReflectionPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Reflection(0),
        name: "planar-reflection",
        needs: Caps::NONE,
    };
    type State = Vec<wgpu::Buffer>;
    type Plan<'f> = ReflectionPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.reflections.reflection_buffer
    }

    fn encode<'f>(
        _: &Self::State,
        plan: &ReflectionPlan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::Reflection(k) = pass else {
            unreachable!("a planar-reflection pass")
        };
        cx.ctx.r.encode_reflection(plan, k, cx);
    }
}

pub(super) struct PlanReflectionListsInputs<'a> {
    pub batches: &'a [Batch],
    pub vis: &'a mut FrameVisibility,
    pub gpu_cull: bool,
    pub spheres: &'a [(Vec3, f32)],
    pub transforms: &'a mut Vec<instances::GpuTransform>,
    pub origins: &'a mut Vec<u32>,
    pub cull_inside: &'a mut CullScratch,
    pub submission: &'a [u32],
    pub in_meshlet: &'a dyn Fn(&Batch) -> bool,
    pub on_layers: &'a dyn Fn(u32) -> Vec<u32>,
    pub reflections: &'a [camera::PlanarReflection],
    pub mask: u32,
    pub cull_n: u32,
}

pub(super) struct PlanReflectionListsOutputs {
    pub reflection_list: Vec<DrawList>,
    pub reflection_meshlets: Vec<u32>,
    pub reflection_blended: Vec<DrawList>,
}

impl Renderer {
    /// Culls and sorts each planar reflection's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_reflection_lists(
        &self,
        PlanReflectionListsInputs {
            batches,
            vis,
            gpu_cull,
            spheres,
            transforms,
            origins,
            cull_inside,
            submission,
            in_meshlet,
            on_layers,
            reflections,
            mask,
            cull_n,
        }: PlanReflectionListsInputs<'_>,
    ) -> PlanReflectionListsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        // (The GPU culling has one reflection pass: the second slot is culled here.)
        let mut reflection_list = Vec::new();
        let mut reflection_meshlets: Vec<u32> = Vec::new();
        for (k, r) in reflections.iter().enumerate() {
            let frustum = Frustum::from_view_proj(r.3);
            let view = vis.meshlet_view(
                meshlet_gpu,
                batches,
                meshlet_gpu::MeshletView::seen(
                    frustum.planes(),
                    on_layers(mask),
                    r.1.position,
                    r.3,
                    r.4.1,
                ),
            );
            reflection_meshlets.push(view);
            let include =
                |b: &Batch| !b.transparent && b.layers & mask != 0 && !(view > 0 && in_meshlet(b));
            reflection_list.push(if gpu_cull && k == 0 {
                let g = vis.gpu_pass(cull_gpu::PassKind::Reflection, &frustum, None, None);
                indirect_list(batches, cull_n, g, include)
            } else {
                self.cull(
                    batches,
                    spheres,
                    transforms,
                    origins,
                    cull_inside,
                    &frustum,
                    // Under GPU culling the sorted instances are the
                    // culling's: append, never draw a batch whole (the
                    // shift below would wrap).
                    !gpu_cull,
                    include,
                )
                .0
            });
        }
        // The transparent meshes in each mirror (three.js's Reflector renders the whole scene: the sky dome's clouds,
        // the falls' curtains), back to front from the mirrored eye.
        let mut reflection_blended = Vec::new();
        for r in reflections {
            let frustum = Frustum::from_view_proj(r.3);
            reflection_blended.push(
                self.sort_transparent(
                    batches,
                    spheres,
                    submission,
                    transforms,
                    origins,
                    &frustum,
                    r.1.position,
                    r.2.forward,
                    mask,
                    &[],
                )
                .0,
            );
        }
        PlanReflectionListsOutputs {
            reflection_list,
            reflection_meshlets,
            reflection_blended,
        }
    }
}
