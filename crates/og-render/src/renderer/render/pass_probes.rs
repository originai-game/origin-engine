//! Probes render pass.

use super::*;

pub(super) struct ProbesPassInputs<'a, F0, F1, F2, F3, F4, F5, F6> {
    pub(super) scene: &'a RenderScene,
    pub(super) probe_frame: &'a probes::ProbeFrame,
    pub(super) face_depth: bool,
    pub(super) face_water: &'a [Vec<water::WaterDraw>],
    pub(super) face_fluids: &'a [Option<fluid3d::Fluid3dFrame>],
    pub(super) probe_faces: &'a [[frame::CameraFrame; 6]],
    pub(super) active_probes: &'a [ReflectionProbe],
    pub(super) stats: &'a mut FrameStats,
    pub(super) depth_prepass: &'a F0,
    pub(super) encoder: &'a mut Encoder,
    pub(super) face_groups: &'a [wgpu::BindGroup],
    pub(super) fx_group: &'a wgpu::BindGroup,
    pub(super) probe_lists: &'a [DrawList],
    pub(super) probe_skinned: &'a [Vec<Batch>],
    pub(super) draw_list: &'a F1,
    pub(super) draw_skinned: &'a F2,
    pub(super) terrain_indices: Option<&'a wgpu::Buffer>,
    pub(super) terrain_draws: &'a [terrain::TerrainDraw],
    pub(super) scatter_batches: &'a scatter::ScatterBatches,
    pub(super) scatter_material: &'a F3,
    pub(super) scatter_mesh: &'a F4,
    pub(super) sky_group: &'a wgpu::BindGroup,
    pub(super) draw_blended: &'a F5,
    pub(super) probe_blended: &'a [DrawList],
    pub(super) face_boards: &'a [sprites::Billboards],
    pub(super) glow_fx_group: &'a wgpu::BindGroup,
    pub(super) face_particle_groups: &'a [Option<wgpu::BindGroup>],
    pub(super) particle_frame: &'a particles_gpu::ParticleFrame,
    pub(super) line_frame: &'a lines::LineFrame,
    pub(super) face_line_groups: &'a [wgpu::BindGroup],
    pub(super) draw_sprites: &'a F6,
    pub(super) face_sprites: &'a [Vec<sprites::SpriteRun>],
    /// Each pass's view of the meshlet dispatch (0: none), and the
    /// frame's batches it draws.
    pub(super) meshlet_views: &'a [u32],
    pub(super) batches: &'a [Batch],
}

impl Renderer {
    pub(super) fn encode_probes<'r>(
        &self,
        ProbesPassInputs {
            scene,
            probe_frame,
            face_depth,
            face_water,
            face_fluids,
            probe_faces,
            active_probes,
            stats,
            depth_prepass,
            encoder,
            face_groups,
            fx_group,
            probe_lists,
            probe_skinned,
            draw_list,
            draw_skinned,
            terrain_indices,
            terrain_draws,
            scatter_batches,
            scatter_material,
            scatter_mesh,
            sky_group,
            draw_blended,
            probe_blended,
            face_boards,
            glow_fx_group,
            face_particle_groups,
            particle_frame,
            line_frame,
            face_line_groups,
            draw_sprites,
            face_sprites,
            meshlet_views,
            batches,
        }: ProbesPassInputs<
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
            impl Fn(
                &mut wgpu::RenderPass,
                &[sprites::SpriteRun],
                &wgpu::BindGroup,
                &[wgpu::BindGroup],
                bool,
            ) -> u32,
        >,
    ) {
        // Each due probe's six faces from its position, then
        // its filtering (`probes.rs`).
        let clear = match scene.environment.background {
            Background::Color(c) => wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: 1.0,
            },
            Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
        };
        let sky = match scene.environment.background {
            Background::Sky => Some(&self.sky_pipeline),
            Background::Procedural { .. } => Some(&self.sky_look_pipeline),
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
                    stats.draw_calls += depth_prepass(
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
                    let views = t.fluid.as_ref().expect("made with the fluids").views(
                        &t.prepass.lindepth,
                        t.copy.as_ref().expect("made with the fluids"),
                    );
                    stats.draw_calls += self.fluid3d.encode(encoder, f, &face_groups[k], &views);
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
                draw_list(&mut pass, list, true, &mut None, None, None, Some(fx_group));
                stats.draw_calls += list.ranges.len() as u32;
                stats.draw_calls += self.draw_meshlets(&mut pass, batches, meshlet_views[k]);
                // Skinned meshes in their pose, terrain and
                // scattered copies, as the main pass draws them.
                let skinned: Vec<&wgpu::RenderPipeline> = probe_skinned[k]
                    .iter()
                    .map(|b| self.lit_pipeline_of(b, true, false))
                    .collect();
                stats.draw_calls += draw_skinned(
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
                let (n, _) = self.scatter.draw(
                    &mut pass,
                    scatter::ScatterPass::Lit,
                    scatter_batches,
                    scatter_material,
                    scatter_mesh,
                    scattered,
                    None,
                    Some(fx_group),
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
                        stats.draw_calls +=
                            self.water
                                .draw(&mut pass, water, &mut probe_select::Binder::new(None));
                    }
                    if let Some(f) = fluid {
                        stats.draw_calls += self.fluid3d.draw(&mut pass, f);
                    }
                    pass.set_bind_group(2, fx_group, &[]);
                }
                // Transparent meshes and the billboards among
                // them, back to front from the probe, over the
                // opaque capture.
                stats.draw_calls += draw_blended(
                    &mut pass,
                    &probe_blended[k],
                    (&face_boards[k].0, &face_boards[k].1),
                    (fx_group, glow_fx_group),
                    None,
                    None,
                    true,
                );
                stats.draw_calls += draw_skinned(
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
                    let white = &self.white_sprite_group;
                    stats.draw_calls += self.particles.draw(
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
                );
                if n > 0 {
                    stats.draw_calls += n;
                    pass.set_bind_group(0, &face_groups[k], &[]);
                }
                stats.draw_calls +=
                    draw_sprites(&mut pass, &face_sprites[k], glow_fx_group, &[], true);
            }
            // A light volume probe is projected onto its harmonics
            // (probe_grid.rs); a reflection probe filtered.
            match probe_frame.grid_capture(d) {
                Some(_) => self.lightmaps.projector.record(
                    &self.device,
                    encoder,
                    probe_frame.due[d].1 - probe_frame.sampled,
                    &s.capture,
                    &self.lightmaps.probe_buffer,
                ),
                None => self.ibl.record_probe_filter(encoder, &s.filter),
            }
        }
    }
}
