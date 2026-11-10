use super::*;

impl ScatterRenderer {
    /// Places the chunks this frame built on the GPU, then culls this
    /// frame's chunks for the main camera on the GPU (before every pass
    /// that draws them; nothing without compute). Answers the queries each
    /// part's pass wrote (`timing.rs`), when timed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        view: &crate::scatter_cull::CullView,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
        terrain_draws: &[crate::terrain::TerrainDraw],
    ) -> Vec<(&'static str, std::ops::Range<u32>)> {
        let mut queries = Vec::new();
        let mut timed = |name, q0: u32, encoder: &crate::Encoder| {
            if encoder.timed() && encoder.used() > q0 {
                queries.push((name, q0..encoder.used()));
            }
        };
        if let Some(place) = &mut self.place
            && place.pending()
        {
            let q0 = encoder.used();
            let groups: Vec<&wgpu::BindGroup> = self
                .placed_chunks
                .iter()
                .filter_map(|k| Some(&self.chunks.get(k)?.placed.as_ref()?.group))
                .collect();
            place.encode(device, encoder, terrain_draws, &groups);
            timed("scatter: place", q0, encoder);
        }
        let Some(cull) = &self.cull else {
            return queries;
        };
        let groups: Vec<&wgpu::BindGroup> = self
            .culled_chunks
            .iter()
            .filter_map(|k| self.chunks.get(k)?.cull_group.as_ref())
            .collect();
        debug_assert_eq!(groups.len(), self.culled_chunks.len());
        let q0 = encoder.used();
        cull.encode(device, queue, encoder, view, &groups, textures);
        timed("cull: scatter", q0, encoder);
        queries
    }

    /// Draws `batches` into a pass (groups 0 and, for the lit passes, the
    /// frame bound by the caller): the chunks whose bounds `visible` keeps,
    /// with their copies' impostors, then the model impostors; `camera`:
    /// the main camera's pass, which draws what the GPU cull kept. Answers
    /// the draws and the copies in the chunks drawn.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw<'a>(
        &self,
        pass: &mut wgpu::RenderPass,
        kind: ScatterPass,
        all: &ScatterBatches,
        material_group: impl Fn(usize) -> Option<&'a wgpu::BindGroup>,
        mesh: impl Fn(usize) -> Option<(&'a wgpu::Buffer, &'a wgpu::Buffer, u32)>,
        visible: impl Fn(Vec3, f32) -> bool,
        mut probes: Option<&mut crate::probe_select::Binder>,
        fx_group: Option<&wgpu::BindGroup>,
        camera: bool,
        cards: CardSource<'_>,
        // What a shadow pass draws into (shadow_target.rs).
        target: crate::ShadowTarget,
    ) -> (u32, u32) {
        let pipeline = |key: &(Option<usize>, Side, u8)| match kind {
            ScatterPass::Shadow => self.shadow_pipelines.get(key).map(|s| s.get(target)),
            _ => self.pipelines.get(key),
        };
        // The main camera's prepass and lit pass draw the GPU cull's copies.
        let list = self
            .cull
            .as_ref()
            .filter(|_| camera && kind != ScatterPass::Shadow);
        let (mut draws, mut copies) = (0, 0);
        let bind_rest = |pass: &mut wgpu::RenderPass, group, offset| {
            pass.set_bind_group(1, group, &[]);
            pass.set_bind_group(2, &self.group, &[offset]);
            if kind == ScatterPass::Lit
                && let Some(fx) = fx_group
            {
                pass.set_bind_group(3, fx, &[]);
            }
        };
        let impostor_pipeline = pipeline(&(None, Side::Double, kind as u8 + IMPOSTOR_KIND));
        for b in &all.scatters {
            if kind == ScatterPass::Shadow && !b.cast_shadows {
                continue;
            }
            let shader = b.shader.filter(|_| kind != ScatterPass::Shadow);
            let (Some(group), Some((vertices, indices, count)), Some(p)) = (
                material_group(b.material),
                mesh(b.mesh),
                pipeline(&(shader, b.side, kind as u8)),
            ) else {
                continue;
            };
            let mut bound = false;
            for (n, k) in b.chunks.iter().enumerate() {
                let Some(c) = self.chunks.get(k) else {
                    continue;
                };
                let Some(buffer) = &c.buffer else { continue };
                if !visible(c.center, c.radius + b.sway) {
                    continue;
                }
                let culled = list.zip(b.culled.get(n).copied().flatten());
                if !bound {
                    pass.set_pipeline(p);
                    bind_rest(pass, group, b.offset);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    bound = true;
                }
                // Each chunk's reflection probe (probe_select.rs).
                if let Some(b) = probes.as_deref_mut() {
                    b.at(pass, c.center, c.radius);
                }
                match culled {
                    Some((cull, at)) => {
                        pass.set_vertex_buffer(1, cull.list.slice(at.list_offset()..));
                        let (args, offset) = cull.mesh_args(&at);
                        pass.draw_indexed_indirect(args, offset);
                    }
                    None => {
                        pass.set_vertex_buffer(1, buffer.slice(..));
                        match c.placed {
                            // Its job wrote its count into its draw.
                            Some(_) => {
                                pass.draw_indexed_indirect(buffer, crate::scatter_place::TAIL)
                            }
                            None => pass.draw_indexed(0..count, 0, 0..c.count),
                        }
                    }
                }
                draws += 1;
                copies += c.count;
            }
            // The same chunks' far copies as the impostor: the vertex
            // stages split each copy between the two by its distance.
            let (Some(m), Some(ip)) = (b.impostor, impostor_pipeline) else {
                continue;
            };
            let Some(group) = material_group(m) else {
                continue;
            };
            let mut bound = false;
            for (n, k) in b.chunks.iter().enumerate() {
                let Some(c) = self.chunks.get(k) else {
                    continue;
                };
                let Some(buffer) = &c.buffer else { continue };
                if !visible(c.center, c.radius) {
                    continue;
                }
                let culled = list.zip(b.culled.get(n).copied().flatten());
                if !bound {
                    pass.set_pipeline(ip);
                    bind_rest(pass, group, b.offset);
                    bound = true;
                }
                if let Some(b) = probes.as_deref_mut() {
                    b.at(pass, c.center, c.radius);
                }
                match culled {
                    Some((cull, at)) => {
                        pass.set_vertex_buffer(0, cull.list.slice(at.list_offset()..));
                        let (args, offset) = cull.card_args(&at);
                        pass.draw_indirect(args, offset);
                    }
                    None => {
                        pass.set_vertex_buffer(0, buffer.slice(..));
                        match c.placed {
                            Some(_) => pass.draw_indirect(buffer, crate::scatter_place::TAIL + 20),
                            None => pass.draw(0..6, 0..c.count),
                        }
                    }
                }
                draws += 1;
            }
        }
        // The model impostors, an instanced card per batch.
        if let (Some(ip), Some((buffer, _))) = (impostor_pipeline, &self.impostor_instances) {
            for b in &all.impostors {
                if kind == ScatterPass::Shadow && !b.cast_shadows {
                    continue;
                }
                let Some(group) = material_group(b.material) else {
                    continue;
                };
                if !visible(b.center, b.radius) {
                    continue;
                }
                pass.set_pipeline(ip);
                bind_rest(pass, group, b.offset);
                if let Some(p) = probes.as_deref_mut() {
                    p.at(pass, b.center, b.radius);
                }
                let stride = std::mem::size_of::<crate::instances::GpuTransform>() as u64;
                pass.set_vertex_buffer(0, buffer.slice(b.first as u64 * stride..));
                pass.draw(0..6, 0..b.count);
                draws += 1;
                copies += b.count;
            }
        }
        // The crowds' cards: the cull's row of them, or the CPU path's
        // records drawn whole (the depth test keeps what the cull would).
        let card_pipeline = pipeline(&(None, Side::Double, kind as u8 + CARD_KIND));
        if let Some(cp) = card_pipeline {
            for b in &all.cards {
                if kind == ScatterPass::Shadow && !b.input.cast_shadows {
                    continue;
                }
                let Some(group) = &b.group else { continue };
                let source = match (cards, b.input.cpu, &self.card_instances) {
                    (CardSource::Gpu(cc, row), _, _) => {
                        let at = cc.row_offset(row);
                        Some((
                            cc.records.slice(at..cc.row_offset(row + 1)),
                            Some((cc, row)),
                            0,
                        ))
                    }
                    (CardSource::Cpu, Some((first, count)), Some((buffer, _))) => {
                        let stride = crate::skin::SKINNED_INSTANCE_STRIDE;
                        Some((buffer.slice(first as u64 * stride..), None, count))
                    }
                    _ => None,
                };
                let Some((slice, gpu, count)) = source else {
                    continue;
                };
                pass.set_pipeline(cp);
                bind_rest(pass, group, b.offset);
                pass.set_vertex_buffer(0, slice);
                pass.set_index_buffer(self.card_quad.slice(..), wgpu::IndexFormat::Uint32);
                match gpu {
                    Some((cc, row)) => {
                        let (args, at) = cc.draw_args(row, b.input.draw);
                        pass.draw_indexed_indirect(args, at)
                    }
                    None => pass.draw_indexed(0..6, 0, 0..count),
                }
                draws += 1;
            }
        }
        if let Some(b) = probes {
            b.reset(pass);
        }
        (draws, copies)
    }
}
