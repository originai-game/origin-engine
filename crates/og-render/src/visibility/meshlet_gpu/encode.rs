//! A frame's meshlet culling: the passes that pick, cull and compact clusters.

use super::*;

impl MeshletGpu {
    /// The work items `taken` adds to a frame's dispatch, rounded up to a
    /// batch's alignment (`None` before the pools exist): the renderer
    /// adds views while their sum fits [`Self::max_items`].
    /// With `soft`, as the camera's view with every batch sending its
    /// small clusters to the visibility buffer (the most it may take).
    pub fn items(&self, batches: &[Batch], taken: &[u32], soft: bool) -> Option<u64> {
        let pools = self.pools.as_ref()?;
        let (_, _, items, _, _) = plan(
            batches,
            [(taken, soft)],
            |m| pools.slots.get(&m).copied(),
            |_| false,
            |_| soft,
        );
        Some(items.div_ceil(ITEM_ALIGN) as u64 * ITEM_ALIGN as u64)
    }

    /// Whether its cull's pipelines are ready (asked for when not yet:
    /// gpu/deferred.rs).
    pub(crate) fn ready(&self) -> bool {
        self.cull.ready().is_some() & self.compact.pipelines.ready().is_some()
    }

    /// The most work items one frame's dispatch may write (the device's
    /// storage binding limit).
    pub fn max_items(&self) -> u64 {
        self.max_binding / 8
    }

    /// Plans the frame's clustered batches in each of `views` (the
    /// camera's first, then the shadow passes') and encodes their culling
    /// after GPU culling's commands, reading the sorted instances it
    /// uploaded (`gc`), and makes the frame's draw groups (`previous`, the
    /// previous transforms at the same slots; `vertices(mesh)`, a mesh's
    /// f32 vertices). With GPU culling's pyramid this frame the camera's
    /// clusters are culled in two phases: those visible last frame first,
    /// drawn into the occluders' depth (the batches `occluder` names), the
    /// pyramid built again, then the rest tested against it. `view_proj`
    /// is the camera's. Leaves [`Self::frame`] empty, and encodes nothing,
    /// when the work would not fit the device.
    #[allow(clippy::too_many_arguments)]
    pub fn encode<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        batches: &[Batch],
        views: &[MeshletView],
        gc: &crate::cull_gpu::GpuCulling,
        view_proj: Mat4,
        (width, height): (u32, u32),
        previous: &wgpu::Buffer,
        levels: &[u8],
        winds: ([crate::Wind; 2], [crate::foliage::FoliageWind; 2], f32),
        vertices: impl Fn(usize) -> wgpu::BufferBinding<'a>,
        occluder: impl Fn(&Batch) -> bool,
    ) -> Option<wgpu::CommandBuffer> {
        self.frame.clear();
        let pools = self.pools.as_mut()?;
        // Direct-pull tile-classified VisBuffer uses 9 fragment storage buffers
        // in `fs_meshlet_resolve`; devices below 10 fall back cleanly to the
        // hardware meshlet path.
        let soft_on = scene.objects.visibility_buffer
            && device.limits().max_storage_buffers_in_fragment_stage >= 10
            && self.vis_buffer.usable()
            && matches!(
                scene.frame.camera.projection,
                crate::Projection::Perspective { .. }
                    | crate::Projection::PerspectiveShifted { .. }
            );
        let (rows, mut draws, items, workgroups, list) = plan(
            batches,
            views.iter().map(|v| (v.taken.as_slice(), v.lit)),
            |m| pools.slots.get(&m).copied(),
            |b| b.faces == Side::Front.into(),
            |b| soft_on && occluder(b) && !b.faces.mirrored,
        );
        if rows.is_empty() || items as u64 * 8 > self.max_binding {
            return None;
        }
        draws.retain(|d| {
            let b = &batches[d.batch as usize];
            b.foliage || b.reaches(tested(&views[d.view as usize].planes))
        });
        let slots = rows
            .last()
            .map_or(0, |r| r.first_item + r.instances * r.most);
        let max_chunks_raw: u64 = rows
            .iter()
            .map(|r| u64::from(r.instances) * u64::from(r.most.div_ceil(64)))
            .sum();
        let chunks_bytes = (max_chunks_raw.max(1) * 32).min(self.max_binding);
        let max_chunks = (chunks_bytes / 32) as u32;
        if grow(
            device,
            &mut self.batches,
            "meshlet batches",
            (rows.len() * 32) as u64,
            storage_usage(),
        ) {
            self.held.batches.invalidate();
        }
        let (window, items_buffer) = item_window(&rows, items);
        self.window = window;
        grow(
            device,
            &mut self.items,
            "meshlet items",
            items_buffer,
            storage_usage(),
        );
        // Per row 4 words DrawIndirect, then:
        //   +0..+4: [soft_count, triangles, 0, 0]
        //   +4..+8: Stage 2 cull_clusters DispatchWorkgroupsIndirect [dx, dy, dz, total_chunks]
        //   +8..+12: Soft raster DispatchWorkgroupsIndirect [dx, dy, dz, soft_count]
        grow(
            device,
            &mut self.draws,
            "meshlet draws",
            (rows.len() * 16 + 48) as u64,
            Self::draws_usage(),
        );
        grow(
            device,
            &mut self.chunks,
            "meshlet cull chunks",
            chunks_bytes,
            storage_usage(),
        );
        let (soft_first, soft_cap) = list.unwrap_or((0, 0));
        self.compact.prepare(
            device,
            queue,
            [slots, soft_first, soft_cap, rows.len() as u32],
            [&self.batches, &pools.meshes, &self.items, &self.draws],
        );
        let half = (items as u64).max(1);
        if half != self.vis_half || half * 8 > self.vis.size() {
            if half * 8 > self.vis.size() {
                self.vis = buffer(device, "meshlet visibility", half * 8, storage_usage());
            }
            self.vis_half = half;
            self.vis_clear = true;
        }
        self.vis_parity = !self.vis_parity;
        let (read, write) = if self.vis_parity {
            (half, 0)
        } else {
            (0, half)
        };
        let mut planes = vec![[0.0f32; 4]; views.len() * VIEW_ROWS];
        for (v, view) in views.iter().enumerate() {
            let rows = &mut planes[v * VIEW_ROWS..(v + 1) * VIEW_ROWS];
            for (row, p) in rows.iter_mut().zip(view.planes.iter().take(VIEW_PLANES)) {
                *row = p.to_array();
            }
            rows[VIEW_PLANES] = view.eye.extend(view.per_unit).to_array();
            rows[VIEW_PLANES + 1] = [view.perspective as u32 as f32, 0.0, 0.0, 0.0];
        }
        let level_row = planes.len();
        let picked = levels.iter().any(|&l| l != u8::MAX);
        if picked {
            planes.extend(level_rows(levels));
        }
        if grow(
            device,
            &mut self.views,
            "meshlet views",
            (planes.len() * 16) as u64,
            storage_usage(),
        ) {
            self.held.views.invalidate();
        }
        self.held
            .views
            .write_all(queue, &self.views, bytemuck::cast_slice(&planes));
        self.held
            .batches
            .write_all(queue, &self.batches, bytemuck::cast_slice(&rows));
        let draw_args: Vec<[u32; 4]> = rows
            .iter()
            .map(|_| [(CLUSTER_TRIANGLES * 3) as u32, 0, 0, 0])
            .chain([[0; 4]; 3])
            .collect();
        crate::traffic::write_buffer(queue, &self.draws, 0, bytemuck::cast_slice(&draw_args));
        let pyramid = gc.pyramid();
        let depth = gc.occluder_depth().filter(|_| pyramid.is_some());
        let soft_count_word = rows.len() as u32 * 4;
        let triangles_word = soft_count_word + 1;
        let chunk_indirect_word = soft_count_word + 4;
        let soft_dispatch_byte = (u64::from(soft_count_word) + 8) * 4;
        let chunk_indirect_byte = u64::from(chunk_indirect_word) * 4;
        let params = |phase: u32| Params {
            view_proj: view_proj.to_cols_array_2d(),
            lod: [
                scene.objects.lod_bias.max(0.0),
                level_row as f32,
                picked as u32 as f32,
                scene.frame.camera.near,
            ],
            sizes: [width, height, pyramid.map_or(0, |p| p.1), rows.len() as u32],
            phase: [phase, read as u32, write as u32, workgroups],
            soft: match list {
                Some((first, cap)) => [first, cap, soft_count_word, triangles_word],
                None => [0, 0, 0, triangles_word],
            },
            chunks: [chunk_indirect_word, max_chunks, 0, 0],
        };
        let (first, second) = if depth.is_some() { (1, 2) } else { (0, 0) };
        let mut both = vec![0u8; PARAMS_SLOT as usize + std::mem::size_of::<Params>()];
        both[..std::mem::size_of::<Params>()].copy_from_slice(bytemuck::bytes_of(&params(first)));
        both[PARAMS_SLOT as usize..].copy_from_slice(bytemuck::bytes_of(&params(second)));
        self.held.params.write_all(queue, &self.params, &both);
        self.held.draw_params.write_all(
            queue,
            &self.draw_params,
            bytemuck::bytes_of(&DrawParams {
                flags: [scene.objects.lod_debug as u32, 0, 0, 0],
                wind: winds.1,
                leaf_year: crate::foliage::LeafCalendar::gpu(scene.atmosphere.leaves),
            }),
        );
        for (slot, bounds, written) in &mut pools.foliage {
            let lean = |w: crate::Wind| {
                crate::wind_field::lean_cap(Some(w).filter(|w| w.strength > 0.0), false)
                    .max(winds.2)
            };
            let margin = bounds
                .deform_margin(lean(winds.0[0]))
                .max(bounds.deform_margin(lean(winds.0[1])));
            if margin != *written {
                *written = margin;
                let at = (*slot as usize * MESH_ROWS + 1) * 16 + 8;
                crate::traffic::write_buffer(
                    queue,
                    &pools.meshes,
                    at as u64,
                    bytemuck::bytes_of(&margin),
                );
            }
        }
        let selected_bytes = u64::from(slots) * 8;
        let bound = vec![
            gc.instances.clone(),
            self.views.clone(),
            self.batches.clone(),
            self.compact.selected.clone(),
            self.draws.clone(),
            pools.meshes.clone(),
            pools.clusters.clone(),
            self.vis.clone(),
            self.chunks.clone(),
        ];
        let view = pyramid.map(|p| p.0.clone());
        let sizes_key = (selected_bytes, chunks_bytes);
        if self
            .cull_groups
            .as_ref()
            .is_none_or(|(b, v, n, _)| *b != bound || *v != view || *n != sizes_key)
        {
            let params_entry = wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.params,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                }),
            };
            let pyr_view = view.as_ref().unwrap_or(&self.no_pyramid);
            let chunks_entry = wgpu::BindGroupEntry {
                binding: 10,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.chunks,
                    offset: 0,
                    size: wgpu::BufferSize::new(chunks_bytes),
                }),
            };
            let inst_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet cull instances"),
                layout: &self.inst_layout,
                entries: &[
                    params_entry.clone(),
                    entry(1, &gc.instances),
                    entry(2, &self.views),
                    entry(3, &self.batches),
                    entry(4, &pools.meshes),
                    entry(7, &self.draws),
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(pyr_view),
                    },
                    chunks_entry.clone(),
                ],
            });
            let cluster_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet cull clusters"),
                layout: &self.cluster_layout,
                entries: &[
                    params_entry,
                    entry(1, &gc.instances),
                    entry(2, &self.views),
                    entry(5, &pools.clusters),
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.compact.selected,
                            offset: 0,
                            size: wgpu::BufferSize::new(selected_bytes),
                        }),
                    },
                    entry(7, &self.draws),
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(pyr_view),
                    },
                    entry(9, &self.vis),
                    chunks_entry,
                ],
            });
            self.cull_groups = Some((bound, view, sizes_key, [inst_group, cluster_group]));
        }
        self.frame = draws;
        self.ensure_groups(device, batches, (&gc.instances, previous), vertices);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("meshlet cull"),
        });
        encoder.clear_buffer(&self.compact.selected, 0, Some(u64::from(slots) * 8));
        if std::mem::take(&mut self.vis_clear) {
            encoder.clear_buffer(&self.vis, 0, None);
        } else {
            encoder.clear_buffer(&self.vis, write * 4, Some(half * 4));
        }
        let [inst_group, cluster_group] = &self.cull_groups.as_ref().expect("made above").3;
        let inst_wgs = workgroups.div_ceil(64);
        let dispatch = |pass: &mut wgpu::ComputePass<'_>, slot: u64| {
            let [p_inst, p_fin, p_cluster] = self.cull.wait();
            pass.set_pipeline(p_inst);
            pass.set_bind_group(0, inst_group, &[slot as u32]);
            pass.dispatch_workgroups(
                inst_wgs.min(WORKGROUPS_PER_ROW),
                inst_wgs.div_ceil(WORKGROUPS_PER_ROW),
                1,
            );
            pass.set_pipeline(p_fin);
            pass.set_bind_group(0, inst_group, &[slot as u32]);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(p_cluster);
            pass.set_bind_group(0, cluster_group, &[slot as u32]);
            pass.dispatch_workgroups_indirect(&self.draws, chunk_indirect_byte);
            self.compact.encode(pass, slots);
        };
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("meshlet cull"),
                timestamp_writes: None,
            });
            dispatch(&mut pass, 0);
        }
        if let Some((depth, samples)) = depth {
            for d in self.frame.iter().filter(|d| d.view == 0) {
                let b = &batches[d.batch as usize];
                if occluder(b) {
                    let key = (b.faces, samples);
                    if !self.depth_pipelines.contains_key(&key) {
                        let pipeline = self.depth_pipeline(device, b.faces, samples);
                        self.depth_pipelines.insert(key, pipeline);
                    }
                }
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("meshlet early depth"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth,
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
                pass.set_bind_group(0, &self.depth_group, &[]);
                for d in self.frame.iter().filter(|d| d.view == 0) {
                    let b = &batches[d.batch as usize];
                    let (Some(pipeline), Some((_, group))) = (
                        self.depth_pipelines.get(&(b.faces, samples)),
                        self.mesh_groups.get(&b.mesh),
                    ) else {
                        continue;
                    };
                    if !occluder(b) {
                        continue;
                    }
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(1, group, &[d.first_item * 8]);
                    pass.draw_indirect(&self.draws, d.row as u64 * 16);
                }
            }
            // Reset Stage 2 chunk counter before late-phase instance cull.
            encoder.clear_buffer(&self.draws, chunk_indirect_byte, Some(16));
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("meshlet late"),
                timestamp_writes: None,
            });
            gc.rebuild_pyramid(&mut pass);
            dispatch(&mut pass, PARAMS_SLOT);
        }
        if let Some(word) = gc.meshlet_triangles_word() {
            encoder.copy_buffer_to_buffer(
                &self.draws,
                triangles_word as u64 * 4,
                &gc.args,
                word as u64 * 4,
                4,
            );
        }
        let soft_rows = self
            .frame
            .iter()
            .filter(|d| d.soft)
            .map(|d| d.row + 1)
            .max()
            .unwrap_or(0);
        if let (Some(list), Some(pools)) = (list, self.pools.as_ref())
            && soft_rows > 0
        {
            self.vis_buffer.encode(
                device,
                queue,
                &mut encoder,
                vis::VisFrame {
                    view_proj,
                    size: (width, height),
                    list,
                    soft_rows,
                    count: (&self.draws, u64::from(soft_count_word) * 4),
                    soft_dispatch: (&self.draws, soft_dispatch_byte),
                    draw_params: &self.draw_params,
                    instances: &gc.instances,
                    previous,
                    items: &self.items,
                    clusters: &pools.clusters,
                    indices: &pools.indices,
                    vertices: &pools.vertices,
                    stylise: crate::Retro3d::uniform(scene.look.stylise.retro, width, height),
                    wind: winds.1,
                    leaf_year: crate::foliage::LeafCalendar::gpu(scene.atmosphere.leaves),
                    max_binding: self.max_binding,
                },
            );
        }
        Some(encoder.finish())
    }
}
