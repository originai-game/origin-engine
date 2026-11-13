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
        r.frame_draw_strips(
            pass,
            runs,
            scene.frame.debug != DebugView::None,
            StripPart::All,
        )
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
}
