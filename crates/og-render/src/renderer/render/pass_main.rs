//! Main render pass.

use super::*;

pub(super) struct MainPassInputs<'a, F0, F1, F2, F3, F4, F5, F6> {
    pub(super) scene: &'a RenderScene,
    pub(super) water_draws: &'a [water::WaterDraw],
    pub(super) fluid_frame: &'a Option<fluid3d::Fluid3dFrame>,
    pub(super) transmission_view: Option<(&'a wgpu::TextureView, &'a mips::MipChain)>,
    pub(super) taa: bool,
    pub(super) encoder: &'a mut Encoder,
    pub(super) frame_bind_group: &'a wgpu::BindGroup,
    pub(super) fx_group: &'a wgpu::BindGroup,
    pub(super) draw_list: &'a F0,
    pub(super) main_list: &'a DrawList,
    pub(super) batches: &'a [Batch],
    pub(super) physical_fx: Option<(&'a wgpu::BindGroup, &'a wgpu::BindGroup)>,
    pub(super) probe_binding: Option<probe_select::ProbeGroups<'a>>,
    pub(super) stats: &'a mut FrameStats,
    pub(super) draw_skinned: &'a F1,
    pub(super) skinned_visible: &'a [Batch],
    pub(super) lit_skinned: &'a [&'a wgpu::RenderPipeline],
    pub(super) terrain_indices: Option<&'a wgpu::Buffer>,
    pub(super) terrain_draws: &'a [terrain::TerrainDraw],
    pub(super) view_frustum: &'a Frustum,
    pub(super) scatter_batches: &'a scatter::ScatterBatches,
    pub(super) scatter_material: &'a F2,
    pub(super) scatter_mesh: &'a F3,
    pub(super) grass_frame: &'a grass::GrassFrame,
    pub(super) sky_group: &'a wgpu::BindGroup,
    pub(super) fluid_bounds: Option<(Vec3, f32)>,
    pub(super) draw_blended: &'a F4,
    pub(super) blended_list: &'a DrawList,
    pub(super) sprite_frame: &'a sprites::SpriteFrame,
    pub(super) sprite_fx_group: &'a wgpu::BindGroup,
    pub(super) particle_group: &'a Option<wgpu::BindGroup>,
    pub(super) particle_frame: &'a particles_gpu::ParticleFrame,
    pub(super) draw_strips: &'a F5,
    pub(super) ribbon_frame: &'a ribbons::RibbonFrame,
    pub(super) line_frame: &'a lines::LineFrame,
    pub(super) draw_sprites: &'a F6,
    pub(super) metaball_groups: &'a [wgpu::BindGroup],
    pub(super) volumetric_fog: Option<VolumetricFog>,
}

impl Renderer {
    /// The clustered batches lit view `view` of the meshlet dispatch
    /// draws (meshlet_gpu.rs; 0: none) into a reflection's, a view's or a
    /// probe face's lit pass, whose groups 0 and 2 are bound. Returns the
    /// draws made.
    pub(super) fn draw_meshlets(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        batches: &[Batch],
        view: u32,
    ) -> u32 {
        match self.meshlet.as_ref() {
            Some(m) if view > 0 => m.draw(
                pass,
                batches,
                view,
                false,
                |b| &self.cache.materials[b.material].bind_group,
                |_, _| {},
            ),
            _ => 0,
        }
    }

    pub(super) fn encode_main<'r>(
        &self,
        MainPassInputs {
            scene,
            water_draws,
            fluid_frame,
            transmission_view,
            taa,
            encoder,
            frame_bind_group,
            fx_group,
            draw_list,
            main_list,
            batches,
            physical_fx,
            probe_binding,
            stats,
            draw_skinned,
            skinned_visible,
            lit_skinned,
            terrain_indices,
            terrain_draws,
            view_frustum,
            scatter_batches,
            scatter_material,
            scatter_mesh,
            grass_frame,
            sky_group,
            fluid_bounds,
            draw_blended,
            blended_list,
            sprite_frame,
            sprite_fx_group,
            particle_group,
            particle_frame,
            draw_strips,
            ribbon_frame,
            line_frame,
            draw_sprites,
            metaball_groups,
            volumetric_fog,
        }: MainPassInputs<
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
            impl Fn(usize) -> Option<&'r wgpu::BindGroup>,
            impl Fn(usize) -> Option<(&'r wgpu::Buffer, &'r wgpu::Buffer, u32)>,
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
        >,
    ) {
        let clear = match scene.environment.background {
            Background::Color(c) => wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: 1.0,
            },
            Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
        };
        let hdr = self.graph.view("hdr");
        let msaa = self.samples > 1;
        // With water or a 3D fluid the pass is split after the
        // sky: the scene so far is copied for them to refract
        // and reflect, then the pass goes on over it.
        // So is it for transmissive materials, which refract the
        // copy's mip chain (`physical.rs`).
        // And for particles that bend the scene behind them (heat haze),
        // which read the same copy.
        let split = !water_draws.is_empty()
            || fluid_frame.is_some()
            || transmission_view.is_some()
            || particle_frame.distortion;
        let begin = |encoder: &mut crate::Encoder, first: bool| {
            let (view, resolve_target, store) = if msaa {
                (
                    self.graph.view("hdr-msaa"),
                    Some(hdr),
                    if first && split {
                        wgpu::StoreOp::Store
                    } else {
                        wgpu::StoreOp::Discard
                    },
                )
            } else {
                (hdr, None, wgpu::StoreOp::Store)
            };
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(if first { "main" } else { "main (after water)" }),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        depth_slice: None,
                        resolve_target,
                        ops: wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(clear)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: self.graph.view("depth"),
                        depth_ops: Some(wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(1.0)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            // TAA reads the depth to reproject the history.
                            store: if taa || (first && split) {
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
                })
                .forget_lifetime()
        };
        let mut pass = begin(encoder, true);
        pass.set_bind_group(0, frame_bind_group, &[]);
        pass.set_bind_group(2, fx_group, &[]);
        draw_list(
            &mut pass,
            main_list,
            true,
            &mut None,
            physical_fx,
            probe_binding,
            Some(fx_group),
        );
        stats.draw_calls += main_list.ranges.len() as u32;
        // The clustered batches (meshlet_gpu.rs), each one draw of its
        // visible clusters.
        if let Some(m) = self.meshlet.as_ref().filter(|m| !m.frame.is_empty()) {
            let mut binder = probe_select::Binder::new(probe_binding);
            stats.draw_calls += m.draw(
                &mut pass,
                batches,
                0,
                false,
                |b| &self.cache.materials[b.material].bind_group,
                |p, b| binder.bind(p, b.probe),
            );
            // The visibility buffer's pixels of each soft batch.
            stats.draw_calls += m.draw_resolve(
                &mut pass,
                batches,
                false,
                |b| &self.cache.materials[b.material].bind_group,
                |p, b| binder.bind(p, b.probe),
            );
            binder.reset(&mut pass);
        }
        stats.draw_calls += draw_skinned(
            &mut pass,
            skinned_visible,
            false,
            1,
            3,
            lit_skinned,
            physical_fx,
            probe_binding,
        );
        if let Some(indices) = terrain_indices {
            let (n, tris) = terrain::draw_levels(
                &mut pass,
                terrain_draws,
                indices,
                terrain::TerrainPass::Lit,
                |c, r| view_frustum.intersects_sphere(c, r),
                Some(&mut probe_select::Binder::new(probe_binding)),
            );
            stats.draw_calls += n;
            stats.terrain_draws += n;
            stats.terrain_triangles += tris;
            stats.triangles += tris;
        }
        let (n, copies) = self.scatter.draw(
            &mut pass,
            scatter::ScatterPass::Lit,
            scatter_batches,
            scatter_material,
            scatter_mesh,
            |c, r| view_frustum.intersects_sphere(c, r),
            Some(&mut probe_select::Binder::new(probe_binding)),
            Some(fx_group),
        );
        if n > 0 {
            // The copies' group 2 is theirs: the effects' back for what follows.
            pass.set_bind_group(2, fx_group, &[]);
        }
        stats.draw_calls += n;
        stats.scatter_draws += n;
        stats.scatter_copies += copies;
        stats.draw_calls += self.grass.draw(
            &mut pass,
            grass::GrassPass::Lit,
            grass_frame,
            Some(fx_group),
        );
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
        }
        if split {
            drop(pass);
            let size = wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            };
            if let Some(copy) = self.graph.try_view("scene-copy") {
                encoder.copy_texture_to_texture(
                    hdr.texture().as_image_copy(),
                    copy.texture().as_image_copy(),
                    size,
                );
            }
            if let Some((view, chain)) = transmission_view {
                encoder.copy_texture_to_texture(
                    hdr.texture().as_image_copy(),
                    view.texture().as_image_copy(),
                    size,
                );
                self.mips.record_chain(encoder, chain);
            }
            pass = begin(encoder, false);
            pass.set_bind_group(0, frame_bind_group, &[]);
            // Each surface and the fluids with their reflection probe (probe_select.rs).
            let mut binder = probe_select::Binder::new(probe_binding);
            stats.draw_calls += self.water.draw(&mut pass, water_draws, &mut binder);
            if let Some(f) = fluid_frame {
                if let Some((c, r)) = fluid_bounds {
                    binder.at(&mut pass, c, r);
                }
                stats.draw_calls += self.fluid3d.draw(&mut pass, f);
                binder.reset(&mut pass);
            }
        }
        // Transparent meshes and the billboards among them, back
        // to front over the opaque scene.
        pass.set_bind_group(2, fx_group, &[]);
        stats.draw_calls += draw_blended(
            &mut pass,
            blended_list,
            (&sprite_frame.billboards, &sprite_frame.billboard_shadows),
            (fx_group, sprite_fx_group),
            physical_fx,
            probe_binding,
            false,
        );
        stats.draw_calls += draw_skinned(
            &mut pass,
            skinned_visible,
            true,
            1,
            3,
            lit_skinned,
            physical_fx,
            probe_binding,
        );
        // Particles, far systems first.
        if let Some(group) = particle_group {
            let groups = &self.cache.sprite_groups;
            let white = &self.white_sprite_group;
            stats.draw_calls += self.particles.draw(
                &mut pass,
                particle_frame,
                self.samples,
                group,
                scene.camera_layers,
                true,
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
        // Trails, ribbons and beams (layered 2D trails go with the sprites).
        stats.draw_calls += draw_strips(&mut pass, &ribbon_frame.runs);
        // Lines, points and wireframes (lines.rs), with their own group 0.
        let n = self
            .lines
            .draw_scene(&mut pass, line_frame, scene.camera_layers);
        if n > 0 {
            stats.draw_calls += n;
            pass.set_bind_group(0, frame_bind_group, &[]);
        }
        // Sprites and text, blended over the scene in sorted order.
        stats.draw_calls += draw_sprites(
            &mut pass,
            &sprite_frame.runs,
            sprite_fx_group,
            metaball_groups,
            false,
        );
        drop(pass);
        // Volumetric fog over everything drawn (volumetric.rs).
        if volumetric_fog.is_some() {
            self.volumetric.encode_apply(
                &self.device,
                encoder,
                frame_bind_group,
                self.graph.view("hdr"),
                self.graph.view("lindepth"),
            );
        }
    }
}
