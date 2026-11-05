//! View render pass.

use super::*;

pub(super) struct ViewPassInputs<'a, F0, F1, F2, F3, F4, F5, F6> {
    pub(super) i: u32,
    pub(super) scene: &'a RenderScene,
    pub(super) view_plans: &'a [views::ViewPlan],
    pub(super) encoder: &'a mut Encoder,
    pub(super) stats: &'a mut FrameStats,
    pub(super) lights_2d: u32,
    pub(super) view_groups: &'a [(wgpu::BindGroup, wgpu::BindGroup, Option<wgpu::BindGroup>)],
    pub(super) depth_prepass: &'a F0,
    pub(super) view_lists: &'a [DrawList],
    pub(super) view_skinned: &'a [Vec<Batch>],
    pub(super) draw_list: &'a F1,
    pub(super) draw_skinned: &'a F2,
    pub(super) terrain_indices: Option<&'a wgpu::Buffer>,
    pub(super) terrain_draws: &'a [terrain::TerrainDraw],
    pub(super) sky_group: &'a wgpu::BindGroup,
    pub(super) draw_blended: &'a F3,
    pub(super) view_blended: &'a [DrawList],
    pub(super) view_boards: &'a [sprites::Billboards],
    pub(super) view_sprite_fx: &'a [wgpu::BindGroup],
    pub(super) view_particle_groups: &'a [Option<wgpu::BindGroup>],
    pub(super) particle_frame: &'a particles_gpu::ParticleFrame,
    pub(super) draw_strips: &'a F4,
    pub(super) view_ribbons: &'a [Vec<ribbons::RibbonRun>],
    pub(super) line_frame: &'a lines::LineFrame,
    pub(super) view_line_groups: &'a [wgpu::BindGroup],
    pub(super) draw_sprites: &'a F5,
    pub(super) view_sprites: &'a [Vec<sprites::SpriteRun>],
    pub(super) view_fields: &'a [Vec<wgpu::BindGroup>],
    pub(super) draw_glow: &'a F6,
    pub(super) view_glow_runs: &'a [Vec<sprites::SpriteRun>],
    pub(super) post_fx: &'a PostEffects,
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'a [u32],
    pub(super) batches: &'a [Batch],
}

impl Renderer {
    pub(super) fn encode_view(
        &self,
        ViewPassInputs {
            i,
            scene,
            view_plans,
            encoder,
            stats,
            lights_2d,
            view_groups,
            depth_prepass,
            view_lists,
            view_skinned,
            draw_list,
            draw_skinned,
            terrain_indices,
            terrain_draws,
            sky_group,
            draw_blended,
            view_blended,
            view_boards,
            view_sprite_fx,
            view_particle_groups,
            particle_frame,
            draw_strips,
            view_ribbons,
            line_frame,
            view_line_groups,
            draw_sprites,
            view_sprites,
            view_fields,
            draw_glow,
            view_glow_runs,
            post_fx,
            meshlet_views,
            batches,
        }: ViewPassInputs<
            '_,
            impl Fn(
                &mut Encoder,
                &str,
                &views::DepthPrepass,
                (&wgpu::BindGroup, &wgpu::BindGroup),
                &DrawList,
                &[Batch],
                &Frustum,
                Option<&dyn Fn(Vec3, f32) -> bool>,
            ) -> u32,
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
                Option<u32>,
            ) -> u32,
            impl Fn(
                &mut wgpu::RenderPass,
                &DrawList,
                (&[(f32, sprites::SpriteRun)], &[sprites::SpriteRun]),
                (&wgpu::BindGroup, &wgpu::BindGroup),
                Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
                Option<probe_select::ProbeGroups>,
                bool,
            ) -> u32,
            impl Fn(&mut wgpu::RenderPass, &[ribbons::RibbonRun]) -> u32,
            impl Fn(
                &mut wgpu::RenderPass,
                &[sprites::SpriteRun],
                &wgpu::BindGroup,
                &[wgpu::BindGroup],
                bool,
            ) -> u32,
            impl Fn(&mut wgpu::RenderPass, &[sprites::SpriteRun]) -> u32,
        >,
    ) {
        // A secondary camera (views.rs): the main pass's draws
        // on its layers from its eye, then its resolve.
        let (i, clear) = (i as usize, background_clear(&scene.environment.background));
        let p = &view_plans[i];
        let frustum = Frustum::from_view_proj(p.frame.view_proj);
        // Its 2D light map, as its camera sees the lights.
        if let (Some(l), Some(slot)) = (scene.lighting_2d, self.views.slots[p.index].as_ref())
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
            self.metaballs
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
            stats.draw_calls += depth_prepass(
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
            draw_list(
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
            stats.draw_calls += draw_skinned(
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
            let sky = match scene.environment.background {
                Background::Sky => Some(&self.sky_pipeline),
                Background::Procedural { .. } => Some(&self.sky_look_pipeline),
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
            stats.draw_calls += draw_blended(
                &mut pass,
                &view_blended[i],
                (&view_boards[i].0, &view_boards[i].1),
                (&view_groups[i].1, &view_sprite_fx[i]),
                None,
                None,
                true,
            );
            stats.draw_calls += draw_skinned(
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
                let white = &self.white_sprite_group;
                stats.draw_calls += self.particles.draw(
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
            stats.draw_calls += draw_strips(&mut pass, &view_ribbons[i]);
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
            stats.draw_calls += draw_sprites(
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
            stats.draw_calls += draw_glow(&mut pass, &view_glow_runs[i]);
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
