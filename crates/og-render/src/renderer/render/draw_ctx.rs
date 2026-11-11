//! What every pass of a frame draws its meshes, sprites and casters with:
//! the frame's batches, culling and pipeline lists, and the draw helpers
//! over them that the passes share.

use super::*;

/// A frame's shared drawing state, made once the frame's bind groups are
/// made and lent to every pass it encodes.
#[derive(Clone, Copy)]
pub(super) struct DrawCtx<'f> {
    pub r: &'f Renderer,
    pub scene: &'f RenderScene,
    pub batches: &'f [Batch],
    pub culled_gpu: Option<&'f cull_gpu::GpuCulling>,
    pub crowd_cull: Option<&'f crowd_gpu::CrowdCulling>,
    /// Each pass's effect group with its group without the planar
    /// reflection (`fx_lean_of`).
    pub fx_leans: &'f [(&'f wgpu::BindGroup, &'f wgpu::BindGroup)],
    pub palette_group: Option<&'f wgpu::BindGroup>,
    pub skinned_gpu: bool,
    pub sprite_frame: &'f sprites::SpriteFrame,
    pub blended_list: &'f DrawList,
    pub main_list: &'f DrawList,
    pub skinned_visible: &'f [Batch],
    pub lit_skinned: &'f [&'f wgpu::RenderPipeline],
    /// The effect groups of the standard and the physical pipelines, in
    /// the passes that draw physical materials with theirs.
    pub physical_fx: Option<(&'f wgpu::BindGroup, &'f wgpu::BindGroup)>,
    pub fx_group: &'f wgpu::BindGroup,
    pub sprite_fx_group: &'f wgpu::BindGroup,
    pub glow_fx_group: &'f wgpu::BindGroup,
    pub terrain_indices: Option<&'f wgpu::Buffer>,
    pub terrain_draws: &'f [terrain::TerrainDraw],
    pub scatter_batches: &'f scatter::ScatterBatches,
    pub grass_frame: &'f grass::GrassFrame,
    pub shadow_skinned: &'f [&'f ShadowSet],
    pub skinned_batches: &'f [Batch],
    pub terrain_casters: bool,
}

impl<'f> DrawCtx<'f> {
    /// A scattered copy's material group, by material index.
    pub(super) fn scatter_material(&self, m: usize) -> Option<&'f wgpu::BindGroup> {
        let &DrawCtx { r, .. } = self;
        r.cache.materials.get(m).map(|g| &g.bind_group)
    }

    /// A scattered copy's mesh buffers and index count, by mesh index.
    #[allow(clippy::type_complexity)]
    pub(super) fn scatter_mesh(
        &self,
        m: usize,
    ) -> Option<(&'f wgpu::Buffer, &'f wgpu::Buffer, u32)> {
        let &DrawCtx { r, .. } = self;
        r.cache
            .meshes
            .get(m)
            .map(|g| (&g.vertices, &g.indices, g.index_count))
    }

    /// A pass's effect group without the planar reflection, for its
    /// lightmapped draws, by the pass's own group.
    pub(super) fn fx_lean_of(&self, fx: &wgpu::BindGroup) -> Option<&'f wgpu::BindGroup> {
        let &DrawCtx { fx_leans, .. } = self;
        fx_leans
            .iter()
            .find(|(f, _)| std::ptr::eq(*f, fx))
            .map(|(_, l)| *l)
    }

    /// Where a pass's crowd cards come from: GPU culling pass `g`'s row
    /// of the crowd cull, or the CPU path's records.
    pub(super) fn card_source(&self, g: Option<u32>) -> scatter::CardSource<'f> {
        match self.crowd_cull {
            Some(cc) => g
                .and_then(|g| cc.rows_of.get(g as usize).copied().flatten())
                .map_or(scatter::CardSource::None, |r| {
                    scatter::CardSource::Gpu(cc, r)
                }),
            None => scatter::CardSource::Cpu,
        }
    }

    /// The blended skinned batches' OIT pipelines (oit.rs), the rest as
    /// `lit_skinned` (not drawn by the OIT pass).
    pub(super) fn oit_skinned(&self, mode: oit::Oit) -> Vec<&'f wgpu::RenderPipeline> {
        let &DrawCtx {
            r,
            skinned_visible,
            lit_skinned,
            physical_fx,
            ..
        } = self;
        skinned_visible
            .iter()
            .zip(lit_skinned)
            .map(|(b, &sorted)| {
                let physical = b.physical && physical_fx.is_some();
                b.transparent
                    .then_some(())
                    .filter(|_| b.blend)
                    .and_then(|_| r.oit_pipeline(Renderer::oit_lit_source(b, true, physical), mode))
                    .unwrap_or(sorted)
            })
            .collect()
    }

    /// A part of the main camera's transparent surfaces under OIT
    /// (`BlendedPart`): the transmissive meshes, the blended surfaces
    /// into the OIT pass of its mode, or the billboards it cannot take
    /// (back to front). Returns the draws.
    pub(super) fn draw_oit_part(
        &self,
        pass: &mut wgpu::RenderPass,
        part: BlendedPart,
        probes: Option<probe_select::ProbeGroups>,
    ) -> u32 {
        let &DrawCtx {
            r,
            scene,
            batches,
            culled_gpu,
            palette_group,
            skinned_gpu,
            sprite_frame,
            blended_list,
            main_list,
            skinned_visible,
            lit_skinned,
            physical_fx,
            fx_group,
            sprite_fx_group,
            ..
        } = self;
        let billboard_runs = |oit: bool| -> Vec<sprites::SpriteRun> {
            let keep = |run: &sprites::SpriteRun| r.oit_billboard(run, scene).is_some() == oit;
            let boards: Vec<(f32, sprites::SpriteRun)> = sprite_frame
                .billboards
                .iter()
                .filter(|(_, r)| keep(r))
                .copied()
                .collect();
            sprite_frame
                .billboard_shadows
                .iter()
                .filter(|r| keep(r))
                .copied()
                .chain(sprites::draw_runs_of(&boards))
                .collect()
        };
        let mode = match part {
            BlendedPart::Transmissive => None,
            BlendedPart::Oit(m) => Some(m),
            BlendedPart::Rest => {
                let runs = billboard_runs(false);
                if runs.is_empty() {
                    return 0;
                }
                let n = r.frame_draw_sprites(pass, &runs, sprite_fx_group, &[], false, scene);
                pass.set_bind_group(2, fx_group, &[]);
                return n;
            }
        };
        let part_list = DrawList {
            ranges: blended_list
                .ranges
                .iter()
                .filter(|r| batches[r.0 as usize].blend == mode.is_some())
                .copied()
                .collect(),
            indirect: blended_list.indirect,
            ..DrawList::default()
        };
        r.frame_draw_list(DrawListInputs {
            pass,
            list: &part_list,
            lit: true,
            shadow: ShadowTarget::Cascade,
            current: &mut None,
            physical_fx,
            probes,
            fx: Some(fx_group),
            batches,
            culled_gpu,
            fx_lean_of: &|fx: &wgpu::BindGroup| self.fx_lean_of(fx),
            oit: mode,
        });
        let mut draws = part_list.ranges.len() as u32;
        draws += r.frame_draw_skinned(DrawSkinnedInputs {
            pass,
            list: skinned_visible,
            transparent: true,
            blended: Some(mode.is_some()),
            material_group: 1,
            palette_at: 3,
            pipelines: &match mode {
                Some(m) => self.oit_skinned(m),
                None => lit_skinned.to_vec(),
            },
            physical_fx,
            probes,
            palette_group,
            indirect: main_list
                .indirect
                .filter(|_| skinned_gpu)
                .zip(culled_gpu)
                .map(|(g, gc)| (gc, g)),
            // Crowds take this path only when every part is opaque.
            crowds: None,
        });
        if let Some(mode) = mode {
            draws += r.frame_draw_oit_billboards(
                pass,
                &billboard_runs(true),
                sprite_fx_group,
                mode,
                scene,
            );
            pass.set_bind_group(2, fx_group, &[]);
        }
        draws
    }

    /// Draws a list's ranges. Lit passes switch the pipeline when the
    /// batch's (side, blending, effects, shader, physical, lean) changes and
    /// bind each batch's material (and, between standard and physical,
    /// the effect group: `physical_fx`); shadow passes switch on the
    /// side and dissolve (with the material).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_list(
        &self,
        pass: &mut wgpu::RenderPass,
        list: &DrawList,
        lit: bool,
        current: &mut Option<LitDrawKey>,
        physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
        probes: Option<probe_select::ProbeGroups>,
        fx: Option<&wgpu::BindGroup>,
    ) {
        let &DrawCtx {
            r,
            batches,
            culled_gpu,
            ..
        } = self;
        r.frame_draw_list(DrawListInputs {
            pass,
            list,
            lit,
            shadow: ShadowTarget::Cascade,
            current,
            physical_fx,
            probes,
            fx,
            batches,
            culled_gpu,
            fx_lean_of: &|fx: &wgpu::BindGroup| self.fx_lean_of(fx),
            oit: None,
        })
    }

    /// The skinned batches (`transparent`: which of them) with the
    /// pipelines `pipeline` picks; materials at group `material_group`.
    /// `physical_fx` as in draw_list: the main pass's physical batches
    /// bind their physical groups (their pipelines are in `pipelines`).
    /// `cull_pass`: GPU culling's pass whose kept instances `list` (every
    /// skinned batch) draws; `None`: `list`'s own ranges.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_skinned(
        &self,
        pass: &mut wgpu::RenderPass,
        list: &[Batch],
        transparent: bool,
        material_group: u32,
        palette_at: u32,
        pipelines: &[&wgpu::RenderPipeline],
        physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
        probes: Option<probe_select::ProbeGroups>,
        cull_pass: Option<u32>,
    ) -> u32 {
        let &DrawCtx {
            r,
            culled_gpu,
            crowd_cull,
            palette_group,
            skinned_gpu,
            ..
        } = self;
        r.frame_draw_skinned(DrawSkinnedInputs {
            pass,
            list,
            transparent,
            material_group,
            palette_at,
            blended: None,
            pipelines,
            physical_fx,
            probes,
            palette_group,
            indirect: cull_pass
                .filter(|_| skinned_gpu)
                .zip(culled_gpu)
                .map(|(g, gc)| (gc, g)),
            crowds: cull_pass.zip(crowd_cull).and_then(|(g, cc)| {
                cc.rows_of
                    .get(g as usize)
                    .copied()
                    .flatten()
                    .map(|r| (cc, r))
            }),
        })
    }

    /// Glowing sprites into a glow buffer (group 0 set: the camera's).
    pub(super) fn draw_glow(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[sprites::SpriteRun],
    ) -> u32 {
        let &DrawCtx {
            r, glow_fx_group, ..
        } = self;
        r.frame_draw_glow(pass, runs, glow_fx_group)
    }

    /// Trails, ribbons and beams drawn with the 3D scene (layered 2D
    /// trails go with the sprites): the main camera's or a view's runs.
    pub(super) fn draw_strips(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[ribbons::RibbonRun],
    ) -> u32 {
        let &DrawCtx { r, scene, .. } = self;
        r.frame_draw_strips(pass, runs, scene.debug != DebugView::None, StripPart::All)
    }

    /// Sprites and text over a camera's scene, in sorted order: the main
    /// camera's runs with its effect group and metaball fields, or a
    /// view's (`view`: every run tone mapped, no HDR alpha cover).
    /// Answers the draws made.
    pub(super) fn draw_sprites(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[sprites::SpriteRun],
        fx: &wgpu::BindGroup,
        fields: &[wgpu::BindGroup],
        view: bool,
    ) -> u32 {
        self.r
            .frame_draw_sprites(pass, runs, fx, fields, view, self.scene)
    }

    /// The transparent meshes of `list` and the billboards sorted among
    /// them (`sprites::SpriteFrame::billboards`): the billboards' drop
    /// shadows first, then each billboard before the first run nearer
    /// than it (`DrawList::before`), the rest after the last. `fx`: the
    /// pass's effects group and its sprites' (each pass's group 2).
    /// Returns the draw calls.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub(super) fn draw_blended(
        &self,
        pass: &mut wgpu::RenderPass,
        list: &DrawList,
        (boards, shadows): (&[(f32, sprites::SpriteRun)], &[sprites::SpriteRun]),
        (fx, sprite_fx): (&wgpu::BindGroup, &wgpu::BindGroup),
        physical_fx: Option<(&wgpu::BindGroup, &wgpu::BindGroup)>,
        probes: Option<probe_select::ProbeGroups>,
        view: bool,
    ) -> u32 {
        if boards.is_empty() && shadows.is_empty() {
            self.draw_list(pass, list, true, &mut None, physical_fx, probes, Some(fx));
            return list.ranges.len() as u32;
        }
        let billboards = |pass: &mut wgpu::RenderPass, runs: &[sprites::SpriteRun]| {
            let n = self.draw_sprites(pass, runs, sprite_fx, &[], view);
            pass.set_bind_group(2, fx, &[]);
            n
        };
        let meshes = |pass: &mut wgpu::RenderPass, range: std::ops::Range<usize>| {
            let part = DrawList {
                ranges: list.ranges[range.clone()].to_vec(),
                ..DrawList::default()
            };
            self.draw_list(pass, &part, true, &mut None, physical_fx, probes, Some(fx));
            range.len() as u32
        };
        let mut draws = billboards(pass, shadows);
        let (mut drawn, mut start) = (0usize, 0usize);
        for r in 0..=list.ranges.len() {
            let before = list.before.get(r).map_or(boards.len(), |&b| b as usize);
            if before > drawn {
                draws += meshes(pass, start..r);
                draws += billboards(pass, &sprites::draw_runs_of(&boards[drawn..before]));
                (drawn, start) = (before, r);
            }
        }
        draws + meshes(pass, start..list.ranges.len())
    }

    /// The depth prepass of a camera other than the main one (a view's,
    /// a probe face's: views.rs, probes.rs): its opaque `list`, its
    /// opaque skinned batches, the terrain in `frustum` and the
    /// scattered copies `scatter` keeps, through the main prepass's
    /// pipelines (group 0 and 2: `groups`) into `pre`. Returns the draws.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn depth_prepass(
        &self,
        encoder: &mut crate::Encoder,
        label: &str,
        pre: &views::DepthPrepass,
        groups: (&wgpu::BindGroup, &wgpu::BindGroup),
        list: &DrawList,
        skinned: &[Batch],
        frustum: &Frustum,
        scatter: Option<&dyn Fn(Vec3, f32) -> bool>,
    ) -> u32 {
        let &DrawCtx {
            r,
            batches,
            culled_gpu,
            terrain_indices,
            terrain_draws,
            scatter_batches,
            ..
        } = self;
        let attachment = |view, clear: f64| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[
                attachment(&pre.gbuffer, 0.0),
                attachment(&pre.lindepth, post::FAR_DEPTH as f64),
                attachment(&pre.outline, 0.0),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &pre.depth,
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
        pass.set_bind_group(0, groups.0, &[]);
        pass.set_bind_group(2, groups.1, &[]);
        let mut draws = 0;
        for &(b, first, count) in &list.ranges {
            let batch = &batches[b as usize];
            let Some(pipeline) = r.prepass_pipeline_of(batch, false) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            let m = &r.cache.materials[batch.material];
            pass.set_bind_group(1, m.array.as_ref().unwrap_or(&m.bind_group), &[]);
            let mesh = &r.cache.meshes[batch.mesh];
            let start = first as u64 * INSTANCE_STRIDE;
            let range = start..start + count as u64 * INSTANCE_STRIDE;
            // A view's GPU-culled list: its pass's region and draws.
            let indirect = list.indirect.zip(culled_gpu);
            let instances = indirect.map_or(&r.instances.instance_buffer, |(_, gc)| &gc.instances);
            pass.set_vertex_buffer(0, mesh.vertex_buffer(batch.compact).slice(..));
            pass.set_vertex_buffer(1, instances.slice(range.clone()));
            pass.set_vertex_buffer(2, instances.slice(range));
            if let Some(surface) = &mesh.surface {
                pass.set_vertex_buffer(3, surface.previous.slice(..));
            }
            pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            match indirect {
                Some((g, gc)) => {
                    let (args, at) = gc.draw_args(g, b);
                    pass.draw_indexed_indirect(args, at)
                }
                None => pass.draw_indexed(0..mesh.index_count, 0, 0..count),
            }
            draws += 1;
        }
        let opaque: Vec<Batch> = skinned
            .iter()
            .filter(|b| !b.transparent && r.prepass_pipeline_of(b, true).is_some())
            .map(|b| Batch { ..*b })
            .collect();
        let pipelines: Vec<&wgpu::RenderPipeline> = opaque
            .iter()
            .filter_map(|b| r.prepass_pipeline_of(b, true))
            .collect();
        draws += self.draw_skinned(
            &mut pass, &opaque, false, 1, 3, &pipelines, None, None, None,
        );
        if let Some(indices) = terrain_indices {
            draws += terrain::draw_levels(
                &mut pass,
                terrain_draws,
                indices,
                terrain::TerrainPass::Prepass,
                frustum,
                None,
            )
            .0;
        }
        if let Some(keep) = scatter {
            draws += r
                .world
                .scatter
                .draw(
                    &mut pass,
                    scatter::ScatterPass::Prepass,
                    scatter_batches,
                    |m| self.scatter_material(m),
                    |m| self.scatter_mesh(m),
                    keep,
                    None,
                    None,
                    false,
                    self.card_source(None),
                    ShadowTarget::Cascade,
                )
                .0;
        }
        draws
    }

    /// One shadow layer: a cascade or a local light's face, as its plan
    /// (shadow_cache.rs) says: its static casters, the copy into or out
    /// of its cache texture, its dynamic casters over them. `target`: the
    /// live array, the layer's index in it and the depth buffer, which
    /// the cache copies. `all_casters`: false for the weather occlusion
    /// map, which takes only what stands still for its key
    /// (weather::shelter_key): no skinned casters, no scattered copies
    /// (grass must not keep the ground under it dry). `targets`: the
    /// draws of every caster, in order, each into its target
    /// (shadow_target.rs): a cascade's, or a local layer's far sides
    /// then near sides.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn shadow_pass(
        &self,
        encoder: &mut crate::Encoder,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        target: (&wgpu::Texture, u32, &wgpu::Texture),
        slot: usize,
        lists: &[DrawList; 2],
        plan: crate::shadow_cache::Plan,
        frustum: &Frustum,
        view: u32,
        stats: &mut FrameStats,
        all_casters: bool,
        targets: &[ShadowTarget],
    ) {
        let &DrawCtx {
            r,
            batches,
            culled_gpu,
            skinned_gpu,
            terrain_indices,
            terrain_draws,
            scatter_batches,
            grass_frame,
            shadow_skinned,
            skinned_batches,
            terrain_casters,
            ..
        } = self;
        stats.shadow_passes += 1;
        stats.shadow_cached += plan.cached() as u32;
        // What every pass of every caster draws besides the instances.
        let draw_rest =
            |pass: &mut wgpu::RenderPass, shadow: ShadowTarget, stats: &mut FrameStats| {
                // The clustered casters (meshlet_gpu.rs), each one draw of
                // the clusters the pass's view kept.
                if view > 0
                    && let Some(m) = r.visibility.meshlet.as_ref()
                {
                    stats.draw_calls += m.draw_shadow(pass, shadow, batches, view, |b| {
                        &r.cache.materials[b.material].bind_group
                    });
                }
                // Terrain levels that reach into the pass's volume.
                if let Some(indices) = terrain_indices
                    && terrain_casters
                {
                    let (n, _) = terrain::draw_levels(
                        pass,
                        terrain_draws,
                        indices,
                        terrain::TerrainPass::Shadow(shadow),
                        frustum,
                        None,
                    );
                    stats.draw_calls += n;
                    stats.terrain_shadow_draws += n;
                }
                if !all_casters {
                    return;
                }
                // Skinned casters: those GPU culling's pass for the layer
                // kept (its counts hold them), unculled on the CPU path.
                let g = lists[0].indirect.or(lists[1].indirect);
                let pipelines: Vec<&wgpu::RenderPipeline> =
                    shadow_skinned.iter().map(|s| s.get(shadow)).collect();
                let n = self.draw_skinned(
                    pass,
                    skinned_batches,
                    false,
                    1,
                    2,
                    &pipelines,
                    None,
                    None,
                    g,
                );
                stats.draw_calls += n;
                if !(skinned_gpu && g.is_some()) {
                    stats.shadow_casters += skinned_batches
                        .iter()
                        .filter(|b| !b.transparent && b.cast)
                        .map(|b| b.count)
                        .sum::<u32>();
                }
                let (n, _) = r.world.scatter.draw(
                    pass,
                    scatter::ScatterPass::Shadow,
                    scatter_batches,
                    |m| self.scatter_material(m),
                    |m| self.scatter_mesh(m),
                    |c, r| frustum.intersects_sphere(c, r),
                    None,
                    None,
                    false,
                    self.card_source(g),
                    shadow,
                );
                stats.draw_calls += n; // Grass casts into the first cascades only (grass.rs).
                if slot < grass::SHADOW_CASCADES {
                    stats.draw_calls +=
                        r.world
                            .grass
                            .draw(pass, grass::GrassPass::Shadow, grass_frame, None);
                }
            };
        let draw = |encoder: &mut crate::Encoder,
                    list: &DrawList,
                    rest: bool,
                    load: bool,
                    keep_depth: bool,
                    stats: &mut FrameStats| {
            let mut pass = shadow_target(encoder, color, depth, load, keep_depth);
            // No pipeline first: the draw list sets each batch's own, as
            // do the meshlet, skinned, terrain and scatter casters.
            pass.set_bind_group(
                0,
                &r.shadows.shadow_bind_group,
                &[(slot as u64 * SHADOW_SLOT_STRIDE) as u32],
            );
            for &shadow in targets {
                r.frame_draw_list(DrawListInputs {
                    pass: &mut pass,
                    list,
                    lit: false,
                    shadow,
                    current: &mut None,
                    physical_fx: None,
                    probes: None,
                    fx: None,
                    batches,
                    culled_gpu,
                    fx_lean_of: &|fx: &wgpu::BindGroup| self.fx_lean_of(fx),
                    oit: None,
                });
                stats.draw_calls += list.ranges.len() as u32;
                if rest {
                    draw_rest(&mut pass, shadow, stats);
                }
            }
            stats.shadow_casters += list.instances;
        };
        // The layer and its cache texture (and both depths), one way.
        let copy = |encoder: &mut crate::Encoder, into_cache: bool| {
            let cache = r.shadows.shadow_cache_maps[slot]
                .as_ref()
                .expect("made for its plan");
            let (array, layer, depth_texture) = target;
            let at = |texture, z| wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z },
                aspect: wgpu::TextureAspect::All,
            };
            let size = wgpu::Extent3d {
                width: cache.size,
                height: cache.size,
                depth_or_array_layers: 1,
            };
            let pairs = [
                (at(array, layer), at(&cache.color, 0)),
                (at(depth_texture, 0), at(&cache.depth, 0)),
            ];
            for (live, kept) in pairs {
                if into_cache {
                    encoder.copy_texture_to_texture(live, kept, size);
                } else {
                    encoder.copy_texture_to_texture(kept, live, size);
                }
            }
        };
        if plan.draw_static {
            // Every caster when it is one pass; the depth kept for the
            // dynamic casters when they follow.
            draw(encoder, &lists[0], plan.single(), false, plan.save, stats);
        }
        if plan.save {
            copy(encoder, true);
        }
        if plan.restore {
            copy(encoder, false);
        }
        if plan.overlay && !plan.single() {
            draw(encoder, &lists[1], true, true, false, stats);
        }
    }
}
