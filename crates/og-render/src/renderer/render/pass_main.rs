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
    pub(super) backface_fx: Option<&'f wgpu::BindGroup>,
    pub(super) probe_binding: Option<probe_select::ProbeGroups<'f>>,
    pub(super) skinned_visible: &'f [Batch],
    pub(super) lit_skinned: &'f [Option<&'f wgpu::RenderPipeline>],
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
    pub(super) volumetric_fog: Option<crate::volumetric::Medium>,
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
            backface_fx: _,
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
        let shared_depth = post::shares_prepass_depth(
            self.graph.order().contains(&FramePass::Prepass),
            self.samples,
        );
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
                            // The prepass's depth, when it is this pass's
                            // (post::shares_prepass_depth).
                            load: if first && !shared_depth {
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
        // A serial frame (`set_gpu_serial`) draws each class of draw in a
        // render pass of its own, each waited for, so the perf record
        // reads the main pass's cost by what it draws (`"main: scatter"`,
        // timing.rs `split_queries`). Every pass loads and stores what
        // the one before it left: the image is the same.
        let serial = super::settings::gpu_serial(self.frame_index);
        let mut pass = begin(
            encoder,
            true,
            serial || split || oit.is_some(),
            if serial { "main: opaque" } else { "main" },
        );
        pass.set_bind_group(0, frame_bind_group, &[]);
        pass.set_bind_group(2, fx_group, &[]);
        macro_rules! part {
            ($label:literal) => {
                if serial {
                    drop(pass);
                    self.serial_cut(encoder);
                    pass = begin(encoder, false, true, $label);
                    pass.set_bind_group(0, frame_bind_group, &[]);
                    pass.set_bind_group(2, fx_group, &[]);
                }
            };
        }
        // The opaque parts below draw the same commands frame after frame
        // while the scene stands still (GPU culling's counts are read by
        // the GPU), so each plays from its render bundle (gpu/bundle.rs).
        // A bundle starts with nothing bound: each part binds the pass's
        // groups 0 and 2 itself.
        let target = Target::of(
            &[Some(if msaa {
                self.graph.view("hdr-msaa")
            } else {
                hdr
            })],
            Some((self.graph.view("depth"), false)),
        );
        macro_rules! bundled {
            ($part:literal, |$d:ident| $draw:expr) => {{
                let (out, played) = self.bundles.play(
                    &self.device,
                    &mut pass,
                    ($part, 0),
                    &target,
                    self.frame_index,
                    |$d| {
                        $d.set_bind_group(0, frame_bind_group, &[]);
                        $d.set_bind_group(2, fx_group, &[]);
                        $draw
                    },
                );
                played.count(stats);
                out
            }};
        }
        bundled!("main: instances", |d| ctx.draw_list(
            d,
            main_list,
            true,
            &mut None,
            physical_fx,
            probe_binding,
            Some(fx_group),
        ));
        stats.draw_calls += main_list.ranges.len() as u32;
        // The clustered batches (meshlet_gpu.rs), each one draw of its
        // visible clusters.
        part!("main: meshlets");
        if let Some(m) = self
            .visibility
            .meshlet
            .as_ref()
            .filter(|m| !m.frame.is_empty())
        {
            stats.draw_calls += bundled!("main: meshlets", |d| {
                let mut binder = probe_select::Binder::new(probe_binding);
                let n = m.draw(
                    d,
                    batches,
                    0,
                    false,
                    |b| &self.cache.materials[b.material].bind_group,
                    |p, b| binder.bind(p, b.probe),
                );
                // The visibility buffer's pixels of each soft batch.
                n + m.draw_resolve(
                    d,
                    batches,
                    false,
                    |b| &self.cache.materials[b.material].bind_group,
                    |p, b| binder.bind(p, b.probe),
                )
            });
        }
        part!("main: skinned");
        stats.draw_calls += bundled!("main: skinned", |d| ctx.draw_skinned(
            d,
            skinned_visible,
            false,
            1,
            3,
            lit_skinned,
            physical_fx,
            probe_binding,
            main_list.indirect,
        ));
        part!("main: terrain");
        if let Some(indices) = terrain_indices {
            let (n, tris) = bundled!("main: terrain", |d| {
                d.set_bind_group(3, self.world.terrain_fx.group(true), &[]);
                terrain::draw_levels(
                    d,
                    terrain_draws,
                    indices,
                    terrain::TerrainPass::Lit,
                    view_frustum,
                    Some(&mut probe_select::Binder::new(probe_binding)),
                )
            });
            stats.draw_calls += n;
            stats.terrain_draws += n;
            stats.terrain_triangles += tris;
            stats.triangles += tris;
        }
        part!("main: scatter");
        // The cards take the surface the prepass found (scatter draw.rs).
        let card_depth = shared_depth
            .then(|| self.graph.try_view("lindepth"))
            .flatten()
            .and_then(|v| {
                self.world
                    .scatter
                    .get()
                    .and_then(|x| x.card_depth_group(&self.device, v))
            });
        if let Some(x) = self.world.scatter.get() {
            let (n, copies) = bundled!("main: scatter", |d| x.draw(
                d,
                scatter::ScatterPass::Lit,
                scatter_batches,
                |m| ctx.scatter_material(m),
                |m| ctx.scatter_mesh(m),
                |c, r| view_frustum.intersects_sphere(c, r),
                Some(&mut probe_select::Binder::new(probe_binding)),
                Some(fx_group),
                true,
                shared_depth,
                card_depth.as_ref(),
                cards,
                ShadowTarget::Cascade,
            ));
            stats.draw_calls += n;
            stats.scatter_draws += n;
            stats.scatter_copies += copies;
        }
        part!("main: grass");
        if let Some(x) = self.world.grass.get() {
            stats.draw_calls += bundled!("main: grass", |d| x.draw(
                d,
                grass::GrassPass::Lit,
                grass_frame,
                Some(fx_group),
                f32::INFINITY,
            ));
        }
        // What follows draws on the pass's groups 0 and 2, which a bundle
        // leaves unbound.
        pass.set_bind_group(0, frame_bind_group, &[]);
        pass.set_bind_group(2, fx_group, &[]);
        // The clouds the cloud pass marched at half the frame's size, when it ran (pass_clouds.rs): the sky draws
        // them from there.
        let clouds_low = self.graph.try_view("clouds").map(|clouds| {
            bind_group(
                &self.device,
                "clouds low",
                &self.sky.clouds_low.layout,
                &[wgpu::BindingResource::TextureView(clouds)],
            )
        });
        // A grounded ground that takes shadows (ground_heights.rs): its own
        // pipeline, its openness as group 2.
        let ground = self
            .sky
            .ground
            .maps
            .as_ref()
            .filter(|_| ground_heights::wanted(scene))
            .map(|m| &m.group);
        let sky = match scene.lighting.environment.background {
            Background::Sky if ground.is_some() => self.sky.ground.pipeline.get(),
            _ if clouds_low.is_some() => self.sky.clouds_low.sky.get(),
            Background::Sky => self.sky.sky_pipeline.get(),
            Background::Procedural { .. } => self.sky.sky_look_pipeline.get(),
            Background::Color(_) => None,
        }
        .filter(|_| scene.frame.plate.is_none());
        part!("main: sky");
        // A camera's background texture in the sky's place: the cover sets
        // every pixel's HDR alpha to 1, then the plate, at the far plane,
        // writes 0 where nothing covers it (sprite.wgsl `fs_plate`), so
        // the sprites after it draw no cover of their own
        // (`frame_draw_sprites`).
        if let Some(group) = self.plate_group(scene)
            && let (Some(cover), Some(plate)) = (
                self.sprite.sprite_cover_pipeline.get(),
                self.sprite.plate_pipeline.get(),
            )
        {
            pass.set_pipeline(cover);
            pass.draw(0..3, 0..1);
            pass.set_pipeline(plate);
            pass.set_bind_group(1, group, &[]);
            pass.draw(0..3, 0..1);
            stats.draw_calls += 2;
        }
        if let Some(sky) = sky {
            pass.set_pipeline(sky);
            pass.set_bind_group(1, sky_group, &[]);
            if let Some(ground) = ground {
                pass.set_bind_group(2, ground, &[]);
            } else if let Some(clouds_low) = &clouds_low {
                pass.set_bind_group(2, clouds_low, &[]);
            }
            pass.draw(0..3, 0..1);
            if clouds_low.is_some() || ground.is_some() {
                // The effects' back for what follows.
                pass.set_bind_group(2, fx_group, &[]);
            }
            stats.draw_calls += 1;
        }
        // The sky layer is behind opaque surfaces. Their finite cloud
        // transport belongs over their lit colour, before transparency.
        if CloudsPass::runs(scene)
            && let (Some(front), Some(depth)) = (
                self.sky.clouds_low.front.get(),
                self.graph.try_view("lindepth"),
            )
        {
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("clouds over opaque"),
                layout: &self.sky.clouds_low.temporal.depth,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.sky.clouds_low.temporal.uniform.as_entire_binding(),
                    },
                ],
            });
            part!("main: foreground clouds");
            pass.set_pipeline(front);
            pass.set_bind_group(1, sky_group, &[]);
            pass.set_bind_group(2, &group, &[]);
            pass.draw(0..3, 0..1);
            pass.set_bind_group(2, fx_group, &[]);
            stats.draw_calls += 1;
        }
        if split {
            drop(pass);
            if transmission_view.is_some() {
                self.encode_transmission_surfaces(plan, ctx, encoder, stats);
            }
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
            if serial {
                self.serial_cut(encoder);
            }
            pass = begin(
                encoder,
                false,
                serial || oit.is_some(),
                if serial {
                    "main: water"
                } else {
                    "main (after water)"
                },
            );
            pass.set_bind_group(0, frame_bind_group, &[]);
            // Each surface and the fluids with their reflection probe (probe_select.rs).
            let mut binder = probe_select::Binder::new(probe_binding);
            stats.draw_calls += self.world.water.get().map_or(Default::default(), |x| {
                x.draw(&mut pass, water_draws, &mut binder, self.frame_features)
            });
            if let Some(f) = fluid_frame {
                if let Some((c, r)) = fluid_bounds {
                    binder.at(&mut pass, c, r);
                }
                stats.draw_calls += self
                    .world
                    .fluid3d
                    .get()
                    .map_or(Default::default(), |x| x.draw(&mut pass, f));
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
                Some(view_frustum),
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
                        .map(|g| (g.vertex_slice(false), &g.indices, g.index_count))
                },
            )
        };
        let diagnostic = scene.frame.debug != DebugView::None;
        match oit {
            None => {
                // Transparent meshes and the billboards among them, back
                // to front over the opaque scene.
                part!("main: blended");
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
                // The passes below write the arena from their shaders:
                // declared ahead of each (wgpu-flash transitions nothing
                // inside a render pass, and leaves the arena readable after
                // one). A no-op in the browser.
                let write_arena = |encoder: &mut crate::Encoder| {
                    if let Some(a) = arena {
                        encoder.transition_resources(
                            std::iter::once(wgpu::BufferTransition {
                                buffer: &a.buffer,
                                state: wgpu::BufferUses::STORAGE_READ_WRITE,
                            }),
                            std::iter::empty(),
                        );
                    }
                };
                let (frame_group, probes) = match (mode.arena(), oit_frame_group) {
                    (true, Some(g)) => (g, oit_probe_binding),
                    _ => (frame_bind_group, probe_binding),
                };
                // The weighted mode's front layer: each pixel's nearest
                // fragment marked in the arena first (no colour), which
                // the OIT pass then keeps exact (oit_arena.wgsl).
                if mode == oit::Oit::Front {
                    let mark = oit::Oit::Mark;
                    write_arena(encoder);
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
                write_arena(encoder);
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
        // Volumetric fog over everything drawn (volumetric.rs), to the first
        // surface each pixel shows: the water's where it lies in front.
        if volumetric_fog.is_some() {
            let depth = self.graph.try_view("front-depth");
            self.volumetric.encode_apply(
                &self.device,
                encoder,
                frame_bind_group,
                self.graph.view("hdr"),
                depth.unwrap_or(self.graph.view("lindepth")),
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
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    /// What the frame's shared stages made for it: its lists, its
    /// world's draws and the frame's groups.
    type BindIn<'f> = MainPlan<'f>;
    type Plan<'f> = MainPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.lit
    }

    /// Every frame draws its main pass.
    fn bind<'f>(_: &'f Renderer, _: &'f mut (), plan: MainPlan<'f>) -> Option<MainPlan<'f>> {
        Some(plan)
    }

    /// Order-independent transparency: its accumulation targets, each
    /// blended its own way.
    fn needs(plan: &MainPlan<'_>) -> Caps {
        if plan.oit.is_some() {
            Caps::MRT | Caps::INDEPENDENT_BLEND
        } else {
            Caps::NONE
        }
    }

    fn encode<'f>(_: &Self::State, plan: &MainPlan<'f>, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        cx.ctx.r.encode_main(plan, cx);
    }
}
