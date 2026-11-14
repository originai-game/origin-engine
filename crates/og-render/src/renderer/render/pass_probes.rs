//! Probes render pass.
//!
//! Its plan step lists the reflection probes' meshes: each due capture's
//! six faces culled (a face with a depth prepass keeps its whole list),
//! and their transparent meshes back to front from the probe. Its bind
//! step makes each face's particle and line groups.

use super::*;

pub(super) struct ProbesPlan<'f> {
    pub(super) scene: &'f RenderScene,
    pub(super) probe_frame: &'f probes::ProbeFrame,
    pub(super) face_depth: bool,
    pub(super) face_water: &'f [Vec<water::WaterDraw>],
    pub(super) face_fluids: &'f [Option<fluid3d::Fluid3dFrame>],
    pub(super) probe_faces: &'f [[frame::CameraFrame; 6]],
    pub(super) active_probes: &'f [ReflectionProbe],
    pub(super) face_groups: &'f [wgpu::BindGroup],
    pub(super) fx_group: &'f wgpu::BindGroup,
    pub(super) probe_lists: &'f [DrawList],
    pub(super) probe_skinned: &'f [Vec<Batch>],
    pub(super) terrain_indices: Option<&'f wgpu::Buffer>,
    pub(super) terrain_draws: &'f [terrain::TerrainDraw],
    pub(super) scatter_batches: &'f scatter::ScatterBatches,
    pub(super) probe_blended: &'f [DrawList],
    pub(super) face_boards: &'f [sprites::Billboards],
    pub(super) glow_fx_group: &'f wgpu::BindGroup,
    pub(super) face_particle_groups: &'f [Option<wgpu::BindGroup>],
    pub(super) particle_frame: &'f particles_gpu::ParticleFrame,
    pub(super) line_frame: &'f lines::LineFrame,
    pub(super) face_line_groups: &'f [wgpu::BindGroup],
    pub(super) face_sprites: &'f [Vec<sprites::SpriteRun>],
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'f [u32],
    pub(super) batches: &'f [Batch],
}

impl Renderer {
    pub(super) fn encode_probes(&self, plan: &ProbesPlan<'_>, cx: &mut EncodeCx<'_, '_>) {
        let &ProbesPlan {
            scene,
            probe_frame,
            face_depth,
            face_water,
            face_fluids,
            probe_faces,
            active_probes,
            face_groups,
            fx_group,
            probe_lists,
            probe_skinned,
            terrain_indices,
            terrain_draws,
            scatter_batches,
            probe_blended,
            face_boards,
            glow_fx_group,
            face_particle_groups,
            particle_frame,
            line_frame,
            face_line_groups,
            face_sprites,
            meshlet_views,
            batches,
        } = plan;
        let (ctx, stats, encoder, sky_group) =
            (cx.ctx, &mut *cx.stats, &mut *cx.encoder, cx.sky_group);
        // Each due probe's six faces from its position, then
        // its filtering (`probes.rs`).
        let clear = match scene.lighting.environment.background {
            Background::Color(c) => wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: 1.0,
            },
            Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
        };
        let sky = match scene.lighting.environment.background {
            Background::Sky => Some(&self.sky.sky_pipeline),
            Background::Procedural { .. } => Some(&self.sky.sky_look_pipeline),
            Background::Color(_) => None,
        };
        for (d, &(slot, _)) in probe_frame.due.iter().enumerate() {
            let s = &self.probes.slots[slot];
            // The faces' depth and copy (probes.rs `FaceTargets`),
            // and the water drawn through them.
            let targets = face_depth.then(|| self.probes.targets_of(slot));
            let water = face_water.get(d).filter(|w| !w.is_empty());
            let split = water.is_some() || !face_fluids.is_empty();
            for (face, cam) in probe_faces[d].iter().enumerate() {
                let k = 6 * d + face;
                let frustum = Frustum::from_view_proj(cam.view_proj);
                // Scattered copies near the box only: farther,
                // a blade is under a texel of the face.
                let near = probe_select::near_box(active_probes[probe_frame.due[d].1].bounds());
                let scattered = |c: Vec3, r: f32| near(c, r) && frustum.intersects_sphere(c, r);
                // Its linear depth, before its pass reads it: the
                // depth prepass a view draws (`depth_prepass`).
                if let Some(t) = targets {
                    stats.draw_calls += ctx.depth_prepass(
                        encoder,
                        "reflection probe face depth",
                        &t.prepass,
                        (&face_groups[k], fx_group),
                        &probe_lists[k],
                        &probe_skinned[k],
                        &frustum,
                        Some(&scattered),
                    );
                }
                // Its 3D fluids' depth, thickness and smoothing.
                let fluid = face_fluids.get(k).and_then(Option::as_ref);
                if let (Some(f), Some(t)) = (fluid, targets) {
                    let views = self.world.fluid3d.face(t.size).views(
                        &t.prepass.lindepth,
                        t.copy.as_ref().expect("made with the fluids"),
                    );
                    stats.draw_calls +=
                        self.world
                            .fluid3d
                            .encode(encoder, f, &face_groups[k], &views);
                }
                let (view, resolve_target) = match &s.msaa {
                    Some(m) => (m, Some(&s.faces[face])),
                    None => (&s.faces[face], None),
                };
                // The face's pass; with water it ends after the
                // sky (`first`), keeping its colour and depth,
                // and goes on over the face's copy.
                let begin = |encoder: &mut Encoder, first: bool| {
                    let keep = split && first;
                    encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("reflection probe face"),
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
                                    store: if resolve_target.is_none() || keep {
                                        wgpu::StoreOp::Store
                                    } else {
                                        wgpu::StoreOp::Discard
                                    },
                                },
                            })],
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: &s.depth,
                                    depth_ops: Some(wgpu::Operations {
                                        load: if first {
                                            wgpu::LoadOp::Clear(1.0)
                                        } else {
                                            wgpu::LoadOp::Load
                                        },
                                        store: if keep {
                                            wgpu::StoreOp::Store
                                        } else {
                                            wgpu::StoreOp::Discard
                                        },
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        })
                        .forget_lifetime()
                };
                let mut pass = begin(encoder, true);
                pass.set_bind_group(0, &face_groups[k], &[]);
                pass.set_bind_group(2, fx_group, &[]);
                let list = &probe_lists[k];
                ctx.draw_list(&mut pass, list, true, &mut None, None, None, Some(fx_group));
                stats.draw_calls += list.ranges.len() as u32;
                stats.draw_calls += self.draw_meshlets(&mut pass, batches, meshlet_views[k]);
                // Skinned meshes in their pose, terrain and
                // scattered copies, as the main pass draws them.
                let skinned: Vec<&wgpu::RenderPipeline> = probe_skinned[k]
                    .iter()
                    .map(|b| self.lit_pipeline_of(b, true, false))
                    .collect();
                stats.draw_calls += ctx.draw_skinned(
                    &mut pass,
                    &probe_skinned[k],
                    false,
                    1,
                    3,
                    &skinned,
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
                let (n, _) = self.world.scatter.draw(
                    &mut pass,
                    scatter::ScatterPass::Lit,
                    scatter_batches,
                    |m| ctx.scatter_material(m),
                    |m| ctx.scatter_mesh(m),
                    scattered,
                    None,
                    Some(fx_group),
                    false,
                    scatter::CardSource::Cpu,
                    ShadowTarget::Cascade,
                );
                if n > 0 {
                    pass.set_bind_group(2, fx_group, &[]);
                }
                stats.draw_calls += n;
                if let Some(sky) = sky {
                    pass.set_pipeline(sky);
                    pass.set_bind_group(1, sky_group, &[]);
                    pass.draw(0..3, 0..1);
                    stats.draw_calls += 1;
                }
                // Water and the 3D fluids over the opaque capture
                // and the sky, as the main pass draws them before
                // its transparents: through the copy of the face
                // drawn so far.
                if let (true, Some(t)) = (split, targets) {
                    drop(pass);
                    let copy = t.copy.as_ref().expect("made with the water");
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &s.capture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: face as u32,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        copy.texture().as_image_copy(),
                        wgpu::Extent3d {
                            width: s.size,
                            height: s.size,
                            depth_or_array_layers: 1,
                        },
                    );
                    pass = begin(encoder, false);
                    pass.set_bind_group(0, &face_groups[k], &[]);
                    if let Some(water) = water {
                        stats.draw_calls += self.world.water.draw(
                            &mut pass,
                            water,
                            &mut probe_select::Binder::new(None),
                            self.frame_features,
                        );
                    }
                    if let Some(f) = fluid {
                        stats.draw_calls += self.world.fluid3d.draw(&mut pass, f);
                    }
                    pass.set_bind_group(2, fx_group, &[]);
                }
                // Transparent meshes and the billboards among
                // them, back to front from the probe, over the
                // opaque capture.
                stats.draw_calls += ctx.draw_blended(
                    &mut pass,
                    &probe_blended[k],
                    (&face_boards[k].0, &face_boards[k].1),
                    (fx_group, glow_fx_group),
                    None,
                    None,
                    true,
                );
                stats.draw_calls += ctx.draw_skinned(
                    &mut pass,
                    &probe_skinned[k],
                    true,
                    1,
                    3,
                    &skinned,
                    None,
                    None,
                    None,
                );
                // Particles, lines and points, world sprites and
                // text, as the main pass draws them over its
                // scene (every layer; sprites unlit).
                if let Some(group) = &face_particle_groups[k] {
                    let groups = &self.cache.sprite_groups;
                    let white = &self.sprite.white_sprite_group;
                    stats.draw_calls += self.particles.gpu.draw(
                        &mut pass,
                        particle_frame,
                        self.samples,
                        group,
                        u32::MAX,
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
                let n = self.lines.draw_scene_with(
                    &mut pass,
                    line_frame,
                    &face_line_groups[k],
                    u32::MAX,
                    &|m| &self.cache.materials[m].bind_group,
                );
                if n > 0 {
                    stats.draw_calls += n;
                    pass.set_bind_group(0, &face_groups[k], &[]);
                }
                stats.draw_calls +=
                    ctx.draw_sprites(&mut pass, &face_sprites[k], glow_fx_group, &[], true);
            }
            // A light volume probe is projected onto its harmonics
            // (probe_grid.rs); a reflection probe filtered.
            match probe_frame.grid_capture(d) {
                Some(_) => self.lights.lightmaps.projector.record(
                    &self.device,
                    encoder,
                    probe_frame.due[d].1 - probe_frame.sampled,
                    &s.capture,
                    &self.lights.lightmaps.probe_buffer,
                ),
                None => self.lights.ibl.record_probe_filter(encoder, &s.filter),
            }
        }
    }
}

pub(super) struct ProbesPass;

/// A frame's due probe faces: each face's lists and meshlet view (0:
/// none), and its particle and line groups over its own camera's
/// uniforms.
#[derive(Default)]
pub(super) struct ProbesFrame {
    pub lists: Vec<DrawList>,
    pub blended: Vec<DrawList>,
    pub views: Vec<u32>,
    pub particle_groups: Vec<Option<wgpu::BindGroup>>,
    pub line_groups: Vec<wgpu::BindGroup>,
}

/// What the probe faces' plan reads: the frame's culling, the due faces'
/// cameras and their probes, whether a face draws a depth prepass, the
/// meshlet batches and each face's billboards.
pub(super) struct ProbesIn<'a> {
    pub cull: &'a mut FrameCulling,
    pub seen: CullFrame<'a>,
    pub probe_faces: &'a [[frame::CameraFrame; 6]],
    pub active_probes: &'a [probes::ReflectionProbe],
    pub probe_frame: &'a probes::ProbeFrame,
    pub face_depth: bool,
    pub meshlet_all: &'a [u32],
    pub face_boards: &'a [sprites::Billboards],
}

/// What the probe faces' bind reads beyond their frame.
pub(super) struct ProbesBindIn<'f> {
    pub scene: &'f RenderScene,
    pub batches: &'f [Batch],
    pub probe_frame: &'f probes::ProbeFrame,
    pub face_depth: bool,
    pub face_water: &'f [Vec<water::WaterDraw>],
    pub face_fluids: &'f [Option<fluid3d::Fluid3dFrame>],
    pub probe_faces: &'f [[frame::CameraFrame; 6]],
    pub active_probes: &'f [ReflectionProbe],
    /// The faces' frame groups (frame_binds.rs).
    pub face_groups: &'f [wgpu::BindGroup],
    pub fx_group: &'f wgpu::BindGroup,
    pub glow_fx_group: &'f wgpu::BindGroup,
    pub probe_skinned: &'f [Vec<Batch>],
    pub terrain_draws: &'f [terrain::TerrainDraw],
    pub scatter_batches: &'f scatter::ScatterBatches,
    pub face_boards: &'f [sprites::Billboards],
    pub face_sprites: &'f [Vec<sprites::SpriteRun>],
    pub particle_frame: &'f particles_gpu::ParticleFrame,
    pub line_frame: &'f lines::LineFrame,
}

impl Pass for ProbesPass {
    /// Each face's depth prepass: the three targets of the camera's; and
    /// the light volumes' grid captures, projected in compute
    /// (probe_grid.rs).
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Probes,
        name: "reflection-probes",
        needs: Caps::MRT.union(Caps::COMPUTE),
    };
    type State = probes::Probes;
    type Frame = ProbesFrame;
    type PlanIn<'a> = ProbesIn<'a>;
    type UploadIn<'a> = ListsMoved;
    type BindIn<'f> = ProbesBindIn<'f>;
    type Plan<'f> = ProbesPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.probes
    }

    /// Culls and sorts each due probe face's meshes.
    fn plan(
        r: &mut Renderer,
        ProbesIn {
            cull,
            seen,
            probe_faces,
            active_probes,
            probe_frame,
            face_depth,
            meshlet_all,
            face_boards,
        }: ProbesIn<'_>,
    ) -> ProbesFrame {
        let r = &*r;
        let batches = seen.batches;
        let meshlet_gpu = r.visibility.meshlet.as_ref();
        // Reflection probes: each due capture's faces culled, their
        // transparent meshes sorted back to front from the probe (probes.rs).
        let mut lists: Vec<DrawList> = Vec::new();
        let mut blended: Vec<DrawList> = Vec::new();
        let mut views: Vec<u32> = Vec::new();
        for (d, cams) in probe_faces.iter().enumerate() {
            let probe = &active_probes[probe_frame.due[d].1];
            let eye = probe.position;
            for (face, f) in cams.iter().enumerate() {
                let frustum = Frustum::from_view_proj(f.view_proj);
                // A face with a depth prepass keeps its list (the prepass
                // draws it).
                let view = if face_depth {
                    0
                } else {
                    cull.vis.meshlet_view(
                        meshlet_gpu,
                        batches,
                        meshlet_gpu::MeshletView::seen(
                            frustum.planes(),
                            meshlet_all.to_vec(),
                            eye,
                            f.view_proj,
                            probe.face_size(),
                        ),
                    )
                };
                views.push(view);
                lists.push(
                    r.cull(
                        batches,
                        seen.spheres,
                        &mut cull.transforms,
                        &mut cull.origins,
                        &mut cull.cull_inside,
                        &frustum,
                        !seen.gpu_cull,
                        |b| !b.transparent && !(view > 0 && (seen.in_meshlet)(b)),
                    )
                    .0,
                );
                blended.push(
                    r.sort_transparent(
                        batches,
                        seen.spheres,
                        seen.submission,
                        &mut cull.transforms,
                        &mut cull.origins,
                        &frustum,
                        eye,
                        f.forward,
                        u32::MAX,
                        &sprites::depths(&face_boards[6 * d + face].0),
                    )
                    .0,
                );
            }
        }
        ProbesFrame {
            lists,
            blended,
            views,
            ..Default::default()
        }
    }

    /// The faces' lists after the uploaded records' move.
    fn upload(_: &mut Renderer, frame: &mut ProbesFrame, moved: ListsMoved) {
        moved.apply(frame.lists.iter_mut().chain(frame.blended.iter_mut()));
    }

    /// Each due face's particle and line groups, then the pass's plan
    /// while a capture is due.
    fn bind<'f>(
        r: &'f Renderer,
        frame: &'f mut ProbesFrame,
        ProbesBindIn {
            scene,
            batches,
            probe_frame,
            face_depth,
            face_water,
            face_fluids,
            probe_faces,
            active_probes,
            face_groups,
            fx_group,
            glow_fx_group,
            probe_skinned,
            terrain_draws,
            scatter_batches,
            face_boards,
            face_sprites,
            particle_frame,
            line_frame,
        }: ProbesBindIn<'f>,
    ) -> Option<ProbesPlan<'f>> {
        // Each due probe face's particle and line groups over its own
        // camera's uniforms (probes.rs); soft particles fade against the
        // face's own depth prepass.
        (frame.particle_groups, frame.line_groups) = probe_faces
            .iter()
            .enumerate()
            .flat_map(|(d, cams)| {
                let (slot, i) = probe_frame.due[d];
                let size = active_probes[i].face_size();
                let position = active_probes[i].position;
                let depth = if face_depth {
                    &r.probes.targets_of(slot).prepass.lindepth
                } else {
                    &r.post.far_depth
                };
                cams.iter().map(move |f| (f, position, size, depth))
            })
            .map(|(f, position, size, depth)| {
                let (particles, lines) =
                    views::camera_gpu(f, position, (size, size), scene.frame.css_height);
                let buffer = |label, contents: &[u8]| {
                    crate::buffer_init(
                        &r.device,
                        &r.queue,
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(label),
                            contents,
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    )
                };
                let pb = buffer("probe face particles", bytemuck::bytes_of(&particles));
                let lb = buffer("probe face lines", bytemuck::bytes_of(&lines));
                (
                    (!particle_frame.systems.is_empty()).then(|| {
                        r.particles
                            .gpu
                            .draw_group_of(&r.device, &pb, depth, &r.post.white)
                    }),
                    r.lines.view_group(&r.device, &lb),
                )
            })
            .unzip();
        let frame = &*frame;
        (!probe_frame.due.is_empty()).then(|| ProbesPlan {
            meshlet_views: &frame.views,
            batches,
            scene,
            probe_frame,
            face_depth,
            face_water,
            face_fluids,
            probe_faces,
            active_probes,
            face_groups,
            fx_group,
            probe_lists: &frame.lists,
            probe_skinned,
            terrain_indices: r.world.terrain.index_buffer(),
            terrain_draws,
            scatter_batches,
            probe_blended: &frame.blended,
            face_boards,
            glow_fx_group,
            face_particle_groups: &frame.particle_groups,
            particle_frame,
            line_frame,
            face_line_groups: &frame.line_groups,
            face_sprites,
        })
    }

    fn encode<'f>(_: &Self::State, plan: &ProbesPlan<'f>, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        cx.ctx.r.encode_probes(plan, cx);
    }
}
