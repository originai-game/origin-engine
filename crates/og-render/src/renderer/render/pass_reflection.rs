//! Reflection render pass.
//!
//! Its plan step lists the planar reflections' meshes: each mirror's
//! opaque meshes, the first through the GPU cull's reflection pass, the
//! rest culled on the CPU, and its transparent meshes back to front from
//! the mirrored eye. Its bind step makes each mirror's effect groups.

use super::*;

pub(super) struct ReflectionPlan<'f> {
    pub(super) scene: &'f RenderScene,
    pub(super) reflection_frame_group: &'f [wgpu::BindGroup],
    pub(super) reflection_fx_group: &'f [wgpu::BindGroup],
    /// Each mirror's effect group without the planar reflection
    /// (`fx_lean_group`), which the frame's draws find by its own group.
    pub(super) reflection_fx_lean: &'f [Option<wgpu::BindGroup>],
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
            reflection_fx_lean: _,
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
            pass.set_bind_group(3, self.world.terrain.fx.group(false), &[]);
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

/// A frame's planar reflections: each mirror's lists and meshlet view (0:
/// none), and its effect groups.
#[derive(Default)]
pub(super) struct ReflectionFrame {
    pub list: Vec<DrawList>,
    pub meshlets: Vec<u32>,
    pub blended: Vec<DrawList>,
    /// Each mirror's effect group: its decals, no AO.
    pub fx_group: Vec<wgpu::BindGroup>,
    /// Each one without the planar reflection, for its lightmapped draws
    /// (`fx_lean_group`).
    pub fx_lean: Vec<Option<wgpu::BindGroup>>,
}

/// What the planar reflections' plan reads: the frame's culling, the
/// mirrors and the layers they see.
pub(super) struct ReflectionIn<'a> {
    pub cull: &'a mut FrameCulling,
    pub seen: CullFrame<'a>,
    pub reflections: &'a [camera::PlanarReflection],
    pub mask: u32,
}

/// What the planar reflections' bind reads beyond their frame.
pub(super) struct ReflectionBindIn<'f> {
    pub scene: &'f RenderScene,
    pub reflections: &'f [camera::PlanarReflection],
    pub batches: &'f [Batch],
    pub skinned_visible: &'f [Batch],
    pub lit_skinned_standard: &'f [&'f wgpu::RenderPipeline],
    pub terrain_draws: &'f [terrain::TerrainDraw],
}

impl Pass for ReflectionPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Reflection(0),
        name: "planar-reflection",
        needs: Caps::NONE,
    };
    type State = Vec<wgpu::Buffer>;
    type Frame = ReflectionFrame;
    type PlanIn<'a> = ReflectionIn<'a>;
    type UploadIn<'a> = ListsMoved;
    type BindIn<'f> = ReflectionBindIn<'f>;
    type Plan<'f> = ReflectionPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.reflections.reflection_buffer
    }

    /// Culls and sorts each planar reflection's meshes.
    fn plan(
        r: &mut Renderer,
        ReflectionIn {
            cull,
            seen,
            reflections,
            mask,
        }: ReflectionIn<'_>,
    ) -> ReflectionFrame {
        let r = &*r;
        let batches = seen.batches;
        let meshlet_gpu = r.visibility.meshlet.as_ref();
        // (The GPU culling has one reflection pass: the second slot is culled here.)
        let mut list = Vec::new();
        let mut meshlets: Vec<u32> = Vec::new();
        for (k, m) in reflections.iter().enumerate() {
            let frustum = Frustum::from_view_proj(m.3);
            let view = cull.vis.meshlet_view(
                meshlet_gpu,
                batches,
                meshlet_gpu::MeshletView::seen(
                    frustum.planes(),
                    (seen.on_layers)(mask),
                    m.1.position,
                    m.3,
                    m.4.1,
                ),
            );
            meshlets.push(view);
            let include = |b: &Batch| {
                !b.transparent && b.layers & mask != 0 && !(view > 0 && (seen.in_meshlet)(b))
            };
            list.push(if seen.gpu_cull && k == 0 {
                let g = cull
                    .vis
                    .gpu_pass(cull_gpu::PassKind::Reflection, &frustum, None, None);
                indirect_list(batches, seen.cull_n, g, include)
            } else {
                r.cull(
                    batches,
                    seen.spheres,
                    &mut cull.transforms,
                    &mut cull.origins,
                    &mut cull.cull_inside,
                    &frustum,
                    // Under GPU culling the sorted instances are the
                    // culling's: append, never draw a batch whole (the
                    // shift below would wrap).
                    !seen.gpu_cull,
                    include,
                )
                .0
            });
        }
        // The transparent meshes in each mirror (three.js's Reflector renders the whole scene: the sky dome's clouds,
        // the falls' curtains), back to front from the mirrored eye.
        let mut blended = Vec::new();
        for m in reflections {
            let frustum = Frustum::from_view_proj(m.3);
            blended.push(
                r.sort_transparent(
                    batches,
                    seen.spheres,
                    seen.submission,
                    &mut cull.transforms,
                    &mut cull.origins,
                    &frustum,
                    m.1.position,
                    m.2.forward,
                    mask,
                    &[],
                )
                .0,
            );
        }
        ReflectionFrame {
            list,
            meshlets,
            blended,
            ..Default::default()
        }
    }

    /// The CPU-culled lists after the uploaded records' move.
    fn upload(_: &mut Renderer, frame: &mut ReflectionFrame, moved: ListsMoved) {
        moved.apply(
            frame
                .list
                .iter_mut()
                .filter(|l| l.indirect.is_none())
                .chain(frame.blended.iter_mut()),
        );
    }

    /// Each mirror's effect groups, then its plan.
    fn bind<'f>(
        r: &'f Renderer,
        frame: &'f mut ReflectionFrame,
        ReflectionBindIn {
            scene,
            reflections,
            batches,
            skinned_visible,
            lit_skinned_standard,
            terrain_draws,
        }: ReflectionBindIn<'f>,
    ) -> Option<ReflectionPlan<'f>> {
        // The planar reflection's effect group: its decals, no AO.
        frame.fx_group = (0..reflections.len())
            .map(|k| {
                let atlas_view = scene
                    .effects
                    .decal_atlas
                    .map_or(&r.default_white, |t| &r.cache.textures[t.index()].view);
                fx::fx_group(
                    &r.device,
                    "fx (planar reflection)",
                    &r.fx_layout,
                    &[
                        r.reflections.reflection_fx_buffer[k].as_entire_binding(),
                        wgpu::BindingResource::TextureView(&r.post.white),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&r.trilinear_sampler),
                        // Not the reflection itself: this pass draws it.
                        wgpu::BindingResource::TextureView(&r.post.white),
                    ][..if r.planar_slot() { 5 } else { 4 }],
                    &r.lights.decal_buffers,
                )
            })
            .collect();
        // Each pass's effect group without the planar reflection, for its
        // lightmapped draws (`fx_lean_group`), by the pass's own group.
        frame.fx_lean = (0..reflections.len())
            .map(|k| {
                let atlas_view = scene
                    .effects
                    .decal_atlas
                    .map_or(&r.default_white, |t| &r.cache.textures[t.index()].view);
                r.fx_lean_group(
                    &r.reflections.reflection_fx_buffer[k],
                    &r.post.white,
                    atlas_view,
                )
            })
            .collect();
        let frame = &*frame;
        let (_, _, _, reflection_frame_group) = r
            .lights
            .environment
            .as_ref()
            .expect("environment generated above");
        (!reflections.is_empty()).then(|| ReflectionPlan {
            meshlet_views: &frame.meshlets,
            batches,
            scene,
            reflection_frame_group,
            reflection_fx_group: &frame.fx_group,
            reflection_fx_lean: &frame.fx_lean,
            reflection_list: &frame.list,
            skinned_visible,
            lit_skinned_standard,
            terrain_indices: r.world.terrain.index_buffer(),
            reflections,
            terrain_draws,
            reflection_blended: &frame.blended,
        })
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
