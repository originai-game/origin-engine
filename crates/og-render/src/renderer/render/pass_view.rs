//! View render pass.
//!
//! The views' lists: each view's transparent meshes on its layers back to
//! front from its eye, and its opaque meshes on its layers (under GPU culling
//! a pass of its own, drawn indirect; on the CPU path culled here).

use super::*;

pub(super) struct ViewPlan<'f> {
    pub(super) scene: &'f RenderScene,
    pub(super) view_plans: &'f [views::ViewPlan],
    pub(super) lights_2d: u32,
    pub(super) view_groups: &'f [(wgpu::BindGroup, wgpu::BindGroup, Option<wgpu::BindGroup>)],
    pub(super) view_lists: &'f [DrawList],
    pub(super) view_skinned: &'f [Vec<Batch>],
    pub(super) terrain_indices: Option<&'f wgpu::Buffer>,
    pub(super) terrain_draws: &'f [terrain::TerrainDraw],
    pub(super) view_blended: &'f [DrawList],
    pub(super) view_boards: &'f [sprites::Billboards],
    pub(super) view_sprite_fx: &'f [wgpu::BindGroup],
    pub(super) view_particle_groups: &'f [Option<wgpu::BindGroup>],
    pub(super) particle_frame: &'f particles_gpu::ParticleFrame,
    pub(super) view_ribbons: &'f [Vec<ribbons::RibbonRun>],
    pub(super) line_frame: &'f lines::LineFrame,
    pub(super) view_line_groups: &'f [wgpu::BindGroup],
    pub(super) view_sprites: &'f [Vec<sprites::SpriteRun>],
    pub(super) view_fields: &'f [Vec<wgpu::BindGroup>],
    pub(super) view_glow_runs: &'f [Vec<sprites::SpriteRun>],
    pub(super) post_fx: &'f PostEffects,
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'f [u32],
    pub(super) batches: &'f [Batch],
}

impl Renderer {
    pub(super) fn encode_view(&self, plan: &ViewPlan<'_>, i: u32, cx: &mut EncodeCx<'_, '_>) {
        let &ViewPlan {
            scene,
            view_plans,
            lights_2d,
            view_groups,
            view_lists,
            view_skinned,
            terrain_indices,
            terrain_draws,
            view_blended,
            view_boards,
            view_sprite_fx,
            view_particle_groups,
            particle_frame,
            view_ribbons,
            line_frame,
            view_line_groups,
            view_sprites,
            view_fields,
            view_glow_runs,
            post_fx,
            meshlet_views,
            batches,
        } = plan;
        let (ctx, encoder, stats, sky_group) =
            (cx.ctx, &mut *cx.encoder, &mut *cx.stats, cx.sky_group);
        // A secondary camera (views.rs): the main pass's draws
        // on its layers from its eye, then its resolve.
        let (i, clear) = (
            i as usize,
            background_clear(&scene.lighting.environment.background),
        );
        let p = &view_plans[i];
        let frustum = Frustum::from_view_proj(p.frame.view_proj);
        // Its 2D light map, as its camera sees the lights.
        if let (Some(l), Some(slot)) =
            (scene.sprite.lighting_2d, self.views.slots[p.index].as_ref())
            && let Some((size, map)) = slot.light_map.as_ref()
        {
            self.lights_2d
                .encode_view(encoder, map, slot.light_dir.as_ref(), *size, l.ambient, i);
            stats.draw_calls += (lights_2d > 0) as u32;
        }
        // Its 2D fluids' fields, the kernels through its camera.
        if let Some(slot) = self.views.slots[p.index].as_ref()
            && !slot.fields.is_empty()
        {
            let fields: Vec<&wgpu::TextureView> = slot.fields.iter().collect();
            self.world
                .metaballs
                .encode_field(encoder, &fields, &view_groups[i].0);
            stats.draw_calls += fields.len() as u32;
        }
        // Its linear depth, for a target's depth texture (the
        // resolve normalises it) and for its soft particles:
        // the view's opaque meshes through the prepass
        // pipelines (views.rs), before its pass reads it.
        if let Some(pre) = self.views.slots[p.index]
            .as_ref()
            .and_then(|s| s.prepass.as_ref())
        {
            stats.draw_calls += ctx.depth_prepass(
                encoder,
                "view depth",
                pre,
                (&view_groups[i].0, &view_groups[i].1),
                &view_lists[i],
                &view_skinned[i],
                &frustum,
                None,
            );
        }
        {
            let mut pass = self.views.begin(encoder, p, clear);
            pass.set_bind_group(0, &view_groups[i].0, &[]);
            pass.set_bind_group(2, &view_groups[i].1, &[]);
            ctx.draw_list(
                &mut pass,
                &view_lists[i],
                true,
                &mut None,
                None,
                None,
                Some(&view_groups[i].1),
            );
            stats.draw_calls += view_lists[i].ranges.len() as u32;
            stats.draw_calls += self.draw_meshlets(&mut pass, batches, meshlet_views[i]);
            let pipelines: Vec<&wgpu::RenderPipeline> = view_skinned[i]
                .iter()
                .map(|b| self.lit_pipeline_of(b, true, false))
                .collect();
            stats.draw_calls += ctx.draw_skinned(
                &mut pass,
                &view_skinned[i],
                false,
                1,
                3,
                &pipelines,
                None,
                None,
                None,
            );
            if let Some(indices) = terrain_indices {
                pass.set_bind_group(3, self.world.terrain.decals.group(false), &[]);
                stats.draw_calls += terrain::draw_levels(
                    &mut pass,
                    terrain_draws,
                    indices,
                    terrain::TerrainPass::Lit,
                    &frustum,
                    None,
                )
                .0;
            }
            let sky = match scene.lighting.environment.background {
                Background::Sky => Some(&self.sky.sky_pipeline),
                Background::Procedural { .. } => Some(&self.sky.sky_look_pipeline),
                Background::Color(_) => None,
            };
            if let (Some(sky), None) = (sky, p.clear) {
                pass.set_pipeline(sky);
                pass.set_bind_group(1, sky_group, &[]);
                pass.draw(0..3, 0..1);
                stats.draw_calls += 1;
            }
            // What the main pass draws over its opaque scene, on
            // the view's layers from the view's eye: the
            // transparent meshes back to front, the particles,
            // the scene lines and points, the world sprites and
            // text.
            pass.set_bind_group(2, &view_groups[i].1, &[]);
            stats.draw_calls += ctx.draw_blended(
                &mut pass,
                &view_blended[i],
                (&view_boards[i].0, &view_boards[i].1),
                (&view_groups[i].1, &view_sprite_fx[i]),
                None,
                None,
                true,
            );
            stats.draw_calls += ctx.draw_skinned(
                &mut pass,
                &view_skinned[i],
                true,
                1,
                3,
                &pipelines,
                None,
                None,
                None,
            );
            if let Some(group) = &view_particle_groups[i] {
                let groups = &self.cache.sprite_groups;
                let white = &self.sprite.white_sprite_group;
                stats.draw_calls += self.particles.gpu.draw(
                    &mut pass,
                    particle_frame,
                    self.samples,
                    group,
                    p.layers,
                    false,
                    particles_gpu::ParticlePart::All,
                    |t: Option<TextureHandle>| match t {
                        None => white,
                        Some(t) => groups[t.index()].as_ref().expect("made before the pass"),
                    },
                    |m: crate::MeshHandle| {
                        self.cache
                            .meshes
                            .get(m.index())
                            .filter(|g| g.full)
                            .map(|g| (&g.vertices, &g.indices, g.index_count))
                    },
                );
            }
            // Trails, ribbons and beams facing the view's eye.
            stats.draw_calls += ctx.draw_strips(&mut pass, &view_ribbons[i]);
            let n = self.lines.draw_scene_with(
                &mut pass,
                line_frame,
                &view_line_groups[i],
                p.layers,
                &|m| &self.cache.materials[m].bind_group,
            );
            if n > 0 {
                stats.draw_calls += n;
                pass.set_bind_group(0, &view_groups[i].0, &[]);
            }
            // Lit by the view's 2D light map.
            stats.draw_calls += ctx.draw_sprites(
                &mut pass,
                &view_sprites[i],
                &view_sprite_fx[i],
                &view_fields[i],
                true,
            );
        }
        // Its glowing sprites into its glow buffer, the chain
        // added onto its image (views.rs).
        if let Some(g) = self.views.slots[p.index]
            .as_ref()
            .and_then(|s| s.glow.as_ref())
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("view glow sprites"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &g.levels[0],
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &view_groups[i].0, &[]);
            stats.draw_calls += ctx.draw_glow(&mut pass, &view_glow_runs[i]);
            drop(pass);
            stats.draw_calls += self.views.add_glow(encoder, p);
        }
        if let views::Dest::Texture {
            color,
            format,
            depth,
        } = p.dest
        {
            let tonemap = post_fx.tonemap;
            let (device, queue) = (&self.device, &self.queue);
            let view = &self.cache.textures[color].view;
            self.views.resolve(
                device,
                queue,
                encoder,
                p,
                tonemap,
                [0.0; 4],
                view,
                format.wgpu(),
                false,
            );
            if let Some(d) = depth {
                let view = &self.cache.textures[d].view;
                let f = wgpu::TextureFormat::Rgba16Float;
                self.views
                    .resolve(device, queue, encoder, p, tonemap, [0.0; 4], view, f, true);
            }
            stats.draw_calls += 1 + depth.is_some() as u32;
        }
    }
}

pub(super) struct ViewPass;

impl Pass for ViewPass {
    /// Its own depth prepass, as the probes' faces.
    const DESC: PassDesc = PassDesc {
        pass: FramePass::View(0),
        name: "view",
        needs: Caps::MRT,
    };
    type State = views::Views;
    type Plan<'f> = ViewPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.views
    }

    fn encode<'f>(
        _: &Self::State,
        plan: &ViewPlan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::View(i) = pass else {
            unreachable!("a view pass")
        };
        cx.ctx.r.encode_view(plan, i, cx);
    }
}

pub(super) struct PlanViewListsInputs<'a> {
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
    pub view_plans: &'a [views::ViewPlan],
    pub view_boards: &'a [sprites::Billboards],
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub cull_n: u32,
}

pub(super) struct PlanViewListsOutputs {
    pub view_blended: Vec<DrawList>,
    pub view_views: Vec<u32>,
    pub view_lists: Vec<DrawList>,
}

impl Renderer {
    /// Sorts and culls each view's meshes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_view_lists(
        &self,
        PlanViewListsInputs {
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
            view_plans,
            view_boards,
            particle_frame,
            cull_n,
        }: PlanViewListsInputs<'_>,
    ) -> PlanViewListsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        // Each view's transparent meshes on its layers, back to front from
        // its eye.
        let view_blended: Vec<DrawList> = view_plans
            .iter()
            .zip(view_boards)
            .map(|(p, boards)| {
                self.sort_transparent(
                    batches,
                    spheres,
                    submission,
                    transforms,
                    origins,
                    &Frustum::from_view_proj(p.frame.view_proj),
                    p.position,
                    p.frame.forward,
                    p.layers,
                    &sprites::depths(&boards.0),
                )
                .0
            })
            .collect();
        // Each view's opaque meshes on its layers: under GPU culling a pass
        // of its own (its planes and layers, as the planar reflection's),
        // drawn indirect; on the CPU path culled here.
        let mut view_views: Vec<u32> = Vec::new();
        let view_lists: Vec<DrawList> = view_plans
            .iter()
            .map(|p| {
                // A view with a depth prepass (a depth texture, soft
                // particles: views.rs `write`) keeps its list.
                let depth = particle_frame.soft
                    || matches!(p.dest, views::Dest::Texture { depth: Some(_), .. });
                let view = if depth {
                    0
                } else {
                    vis.meshlet_view(
                        meshlet_gpu,
                        batches,
                        meshlet_gpu::MeshletView::seen(
                            Frustum::from_view_proj(p.frame.view_proj).planes(),
                            on_layers(p.layers),
                            p.position,
                            p.frame.view_proj,
                            p.size.1,
                        ),
                    )
                };
                view_views.push(view);
                let frustum = Frustum::from_view_proj(p.frame.view_proj);
                let include = |b: &Batch| {
                    !b.transparent && b.layers & p.layers != 0 && !(view > 0 && in_meshlet(b))
                };
                if gpu_cull {
                    let g = vis.gpu_pass(
                        cull_gpu::PassKind::Reflection,
                        &frustum,
                        None,
                        Some(p.layers),
                    );
                    return indirect_list(batches, cull_n, g, include);
                }
                self.cull(
                    batches,
                    spheres,
                    transforms,
                    origins,
                    cull_inside,
                    &frustum,
                    true,
                    include,
                )
                .0
            })
            .collect();
        PlanViewListsOutputs {
            view_blended,
            view_views,
            view_lists,
        }
    }
}
