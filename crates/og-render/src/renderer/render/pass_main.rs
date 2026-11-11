//! Main render pass.

use super::*;

pub(super) struct MainPlan<'f> {
    pub(super) scene: &'f RenderScene,
    pub(super) water_draws: &'f [water::WaterDraw],
    pub(super) fluid_frame: &'f Option<fluid3d::Fluid3dFrame>,
    pub(super) transmission_view: Option<(&'f wgpu::TextureView, &'f mips::MipChain)>,
    pub(super) taa: bool,
    pub(super) frame_bind_group: &'f wgpu::BindGroup,
    pub(super) fx_group: &'f wgpu::BindGroup,
    pub(super) main_list: &'f DrawList,
    pub(super) batches: &'f [Batch],
    pub(super) physical_fx: Option<(&'f wgpu::BindGroup, &'f wgpu::BindGroup)>,
    pub(super) probe_binding: Option<probe_select::ProbeGroups<'f>>,
    pub(super) skinned_visible: &'f [Batch],
    pub(super) lit_skinned: &'f [&'f wgpu::RenderPipeline],
    pub(super) terrain_indices: Option<&'f wgpu::Buffer>,
    pub(super) terrain_draws: &'f [terrain::TerrainDraw],
    pub(super) view_frustum: &'f Frustum,
    pub(super) scatter_batches: &'f scatter::ScatterBatches,
    /// Where the crowds' cards come from in this pass.
    pub(super) cards: scatter::CardSource<'f>,
    pub(super) grass_frame: &'f grass::GrassFrame,
    pub(super) fluid_bounds: Option<(Vec3, f32)>,
    pub(super) blended_list: &'f DrawList,
    pub(super) sprite_frame: &'f sprites::SpriteFrame,
    pub(super) sprite_fx_group: &'f wgpu::BindGroup,
    pub(super) particle_group: &'f Option<wgpu::BindGroup>,
    pub(super) particle_frame: &'f particles_gpu::ParticleFrame,
    pub(super) ribbon_frame: &'f ribbons::RibbonFrame,
    pub(super) line_frame: &'f lines::LineFrame,
    pub(super) metaball_groups: &'f [wgpu::BindGroup],
    pub(super) volumetric_fog: Option<VolumetricFog>,
    /// The blended surfaces' order-independent pass (oit.rs), or `None`:
    /// sorted back to front.
    pub(super) oit: Option<oit::Oit>,
    /// The linked lists' frame group (binding 12, the arena) and their
    /// reflection probes' groups likewise.
    pub(super) oit_frame_group: Option<&'f wgpu::BindGroup>,
    pub(super) oit_probe_binding: Option<probe_select::ProbeGroups<'f>>,
    /// The lit image copied for the next frame's GI (`gi-write`).
    pub(super) gi_copy: bool,
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
        match self.visibility.meshlet.as_ref() {
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

    pub(super) fn encode_main(&self, plan: &MainPlan<'_>, cx: &mut EncodeCx<'_, '_>) {
        let &MainPlan {
            scene,
            water_draws,
            fluid_frame,
            transmission_view,
            taa,
            frame_bind_group,
            fx_group,
            main_list,
            batches,
            physical_fx,
            probe_binding,
            skinned_visible,
            lit_skinned,
            terrain_indices,
            terrain_draws,
            view_frustum,
            scatter_batches,
            cards,
            grass_frame,
            fluid_bounds,
            blended_list,
            sprite_frame,
            sprite_fx_group,
            particle_group,
            particle_frame,
            ribbon_frame,
            line_frame,
            metaball_groups,
            volumetric_fog,
            oit,
            oit_frame_group,
            oit_probe_binding,
            gi_copy,
        } = plan;
        let (ctx, encoder, stats, sky_group) =
            (cx.ctx, &mut *cx.encoder, &mut *cx.stats, cx.sky_group);
        let clear = match scene.lighting.environment.background {
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
        // Each part of the pass stores what a later part loads again: the
        // part before the water's copy, and the parts before the OIT pass
        // and its resolve (`keep`).
        let begin = |encoder: &mut crate::Encoder, first: bool, keep: bool, label: &'static str| {
            let (view, resolve_target, store) = if msaa {
                (
                    self.graph.view("hdr-msaa"),
                    Some(hdr),
                    if keep {
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
                    label: Some(label),
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
                            // TAA and T2x reproject through the depth.
                            store: if taa || keep {
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
        let mut pass = begin(encoder, true, split || oit.is_some(), "main");
        pass.set_bind_group(0, frame_bind_group, &[]);
        pass.set_bind_group(2, fx_group, &[]);
        ctx.draw_list(
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
        if let Some(m) = self
            .visibility
            .meshlet
            .as_ref()
            .filter(|m| !m.frame.is_empty())
        {
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
        stats.draw_calls += ctx.draw_skinned(
            &mut pass,
            skinned_visible,
            false,
            1,
            3,
            lit_skinned,
            physical_fx,
            probe_binding,
            main_list.indirect,
        );
        if let Some(indices) = terrain_indices {
            pass.set_bind_group(3, self.world.terrain.decals.group(true), &[]);
            let (n, tris) = terrain::draw_levels(
                &mut pass,
                terrain_draws,
                indices,
                terrain::TerrainPass::Lit,
                view_frustum,
                Some(&mut probe_select::Binder::new(probe_binding)),
            );
            stats.draw_calls += n;
            stats.terrain_draws += n;
            stats.terrain_triangles += tris;
            stats.triangles += tris;
        }
        let (n, copies) = self.world.scatter.draw(
            &mut pass,
            scatter::ScatterPass::Lit,
            scatter_batches,
            |m| ctx.scatter_material(m),
            |m| ctx.scatter_mesh(m),
            |c, r| view_frustum.intersects_sphere(c, r),
            Some(&mut probe_select::Binder::new(probe_binding)),
            Some(fx_group),
            true,
            cards,
            ShadowTarget::Cascade,
        );
        if n > 0 {
            // The copies' group 2 is theirs: the effects' back for what follows.
            pass.set_bind_group(2, fx_group, &[]);
        }
        stats.draw_calls += n;
        stats.scatter_draws += n;
        stats.scatter_copies += copies;
        stats.draw_calls += self.world.grass.draw(
            &mut pass,
            grass::GrassPass::Lit,
            grass_frame,
            Some(fx_group),
            f32::INFINITY,
        );
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
            pass = begin(encoder, false, oit.is_some(), "main (after water)");
            pass.set_bind_group(0, frame_bind_group, &[]);
            // Each surface and the fluids with their reflection probe (probe_select.rs).
            let mut binder = probe_select::Binder::new(probe_binding);
            stats.draw_calls +=
                self.world
                    .water
                    .draw(&mut pass, water_draws, &mut binder, self.frame_features);
            if let Some(f) = fluid_frame {
                if let Some((c, r)) = fluid_bounds {
                    binder.at(&mut pass, c, r);
                }
                stats.draw_calls += self.world.fluid3d.draw(&mut pass, f);
                binder.reset(&mut pass);
            }
        }
        // Particles, far systems first.
        let particles = |pass: &mut wgpu::RenderPass, part: particles_gpu::ParticlePart| {
            let Some(group) = particle_group else {
                return 0;
            };
            let groups = &self.cache.sprite_groups;
            let white = &self.sprite.white_sprite_group;
            self.particles.gpu.draw(
                pass,
                particle_frame,
                self.samples,
                group,
                scene.frame.camera_layers,
                true,
                part,
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
            )
        };
        let diagnostic = scene.frame.debug != DebugView::None;
        match oit {
            None => {
                // Transparent meshes and the billboards among them, back
                // to front over the opaque scene.
                pass.set_bind_group(2, fx_group, &[]);
                stats.draw_calls += ctx.draw_blended(
                    &mut pass,
                    blended_list,
                    (&sprite_frame.billboards, &sprite_frame.billboard_shadows),
                    (fx_group, sprite_fx_group),
                    physical_fx,
                    probe_binding,
                    false,
                );
                stats.draw_calls += ctx.draw_skinned(
                    &mut pass,
                    skinned_visible,
                    true,
                    1,
                    3,
                    lit_skinned,
                    physical_fx,
                    probe_binding,
                    main_list.indirect,
                );
                stats.draw_calls += particles(&mut pass, particles_gpu::ParticlePart::All);
                // Trails, ribbons and beams (layered 2D trails go with the sprites).
                stats.draw_calls += ctx.draw_strips(&mut pass, &ribbon_frame.runs);
            }
            Some(mode) => {
                // The transmissive meshes (depth written, the scene copy
                // refracted) and the debris (depth written) first: the
                // blended surfaces test against them.
                pass.set_bind_group(2, fx_group, &[]);
                stats.draw_calls +=
                    ctx.draw_oit_part(&mut pass, BlendedPart::Transmissive, probe_binding);
                stats.draw_calls += particles(&mut pass, particles_gpu::ParticlePart::Before);
                drop(pass);
                // The blended surfaces in any order into the weighted
                // targets (and the arena's nodes or front layer), against
                // the depth.
                let arena = self.oit.arena.as_ref().filter(|_| mode.arena());
                if let Some(a) = arena {
                    a.clear(encoder);
                }
                let (frame_group, probes) = match (mode.arena(), oit_frame_group) {
                    (true, Some(g)) => (g, oit_probe_binding),
                    _ => (frame_bind_group, probe_binding),
                };
                // The weighted mode's front layer: each pixel's nearest
                // fragment marked in the arena first (no colour), which
                // the OIT pass then keeps exact (oit_arena.wgsl).
                if mode == oit::Oit::Front {
                    let mark = oit::Oit::Mark;
                    let mut mark_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main (oit front)"),
                        color_attachments: &[],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: self.graph.view("depth"),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    mark_pass.set_bind_group(0, frame_group, &[]);
                    mark_pass.set_bind_group(2, fx_group, &[]);
                    stats.draw_calls +=
                        ctx.draw_oit_part(&mut mark_pass, BlendedPart::Oit(mark), probes);
                    if let Some(p) = self.oit_pipeline(oit::Source::Particles, mark) {
                        stats.draw_calls +=
                            particles(&mut mark_pass, particles_gpu::ParticlePart::Oit(p));
                    }
                    stats.draw_calls += self.frame_draw_strips(
                        &mut mark_pass,
                        &ribbon_frame.runs,
                        diagnostic,
                        StripPart::Oit(mark),
                    );
                }
                let target = |name, clear: f64| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: self.graph.view(name),
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear,
                                g: clear,
                                b: clear,
                                a: clear,
                            }),
                            // Read by the resolve (and TAA), never
                            // resolved: each sample's own sums.
                            store: wgpu::StoreOp::Store,
                        },
                    })
                };
                let mut oit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("main (oit)"),
                    color_attachments: &[
                        target("oit.accum", 0.0),
                        target("oit.reveal", 1.0),
                        target("oit.front", 0.0),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: self.graph.view("depth"),
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                oit_pass.set_bind_group(0, frame_group, &[]);
                oit_pass.set_bind_group(2, fx_group, &[]);
                stats.draw_calls +=
                    ctx.draw_oit_part(&mut oit_pass, BlendedPart::Oit(mode), probes);
                if let Some(p) = self.oit_pipeline(oit::Source::Particles, mode) {
                    stats.draw_calls +=
                        particles(&mut oit_pass, particles_gpu::ParticlePart::Oit(p));
                }
                stats.draw_calls += self.frame_draw_strips(
                    &mut oit_pass,
                    &ribbon_frame.runs,
                    diagnostic,
                    StripPart::Oit(mode),
                );
                drop(oit_pass);
                if let Some(a) = arena.filter(|_| mode == oit::Oit::List) {
                    a.copy_count(encoder, self.frame_index);
                }
                stats.draw_calls += self.encode_oit_resolve(encoder, mode);
                // Over the resolved scene: the billboards the OIT pass
                // cannot take, back to front, then what adds light (it is
                // never weighted down) and what bends the scene.
                pass = begin(encoder, false, false, "main (after oit)");
                pass.set_bind_group(0, frame_bind_group, &[]);
                pass.set_bind_group(2, fx_group, &[]);
                stats.draw_calls += ctx.draw_oit_part(&mut pass, BlendedPart::Rest, probe_binding);
                stats.draw_calls += particles(&mut pass, particles_gpu::ParticlePart::After);
                stats.draw_calls += self.frame_draw_strips(
                    &mut pass,
                    &ribbon_frame.runs,
                    diagnostic,
                    StripPart::Additive,
                );
            }
        }
        // Lines, points and wireframes (lines.rs), with their own group 0.
        let n = self
            .lines
            .draw_scene(&mut pass, line_frame, scene.frame.camera_layers, &|m| {
                &self.cache.materials[m].bind_group
            });
        if n > 0 {
            stats.draw_calls += n;
            pass.set_bind_group(0, frame_bind_group, &[]);
        }
        // Sprites and text, blended over the scene in sorted order.
        stats.draw_calls += ctx.draw_sprites(
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
        // The lit image for the next frame's GI (`gi-write`).
        if let (true, Some((_, copies))) = (gi_copy, &self.persistent.gi) {
            encoder.copy_texture_to_texture(
                self.graph.view("hdr").texture().as_image_copy(),
                copies[cx.slot].texture().as_image_copy(),
                self.graph.view("hdr").texture().size(),
            );
        }
    }
}

pub(super) struct MainPass;

impl Pass for MainPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Main,
        name: "main",
        needs: Caps::NONE,
    };
    type State = LitPipelines;
    type Plan<'f> = MainPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.lit
    }

    /// Order-independent transparency: its accumulation targets.
    fn needs(plan: &MainPlan<'_>) -> Caps {
        if plan.oit.is_some() {
            Caps::MRT
        } else {
            Caps::NONE
        }
    }

    fn encode<'f>(_: &Self::State, plan: &MainPlan<'f>, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        cx.ctx.r.encode_main(plan, cx);
    }
}
