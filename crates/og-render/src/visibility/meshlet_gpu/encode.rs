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
        vertices: impl Fn(usize) -> &'a wgpu::Buffer,
        occluder: impl Fn(&Batch) -> bool,
    ) -> Option<wgpu::CommandBuffer> {
        self.frame.clear();
        let pools = self.pools.as_mut()?;
        // The visibility buffer takes the camera's small clusters of
        // occluder batches (opaque, depth exactly their triangles') under
        // a perspective camera (retro 3D's vertices snap and its uv go
        // affine in the rasteriser too, and the material style's surface
        // takes its gradients from the records).
        // And only while the device makes the resolve's pipelines (vis.rs).
        let soft_on = scene.objects.visibility_buffer
            && self.vis_buffer.usable()
            && matches!(
                scene.frame.camera.projection,
                crate::Projection::Perspective { .. }
            );
        let (rows, draws, items, workgroups, list) = plan(
            batches,
            views.iter().map(|v| (v.taken.as_slice(), v.lit)),
            |m| pools.slots.get(&m).copied(),
            |b| b.side == Side::Front,
            |b| soft_on && occluder(b),
        );
        if rows.is_empty() || items as u64 * 8 > self.max_binding {
            return None;
        }
        let slots = rows
            .last()
            .map_or(0, |r| r.first_item + r.instances * r.most);
        grow(
            device,
            &mut self.batches,
            "meshlet batches",
            (rows.len() * 32) as u64,
            storage_usage(),
        );
        // A draw binds a window of `items` from its batch's first item,
        // as wide as the largest batch's items: the buffer holds the last
        // batch's whole window.
        let (window, items_buffer) = item_window(&rows, items);
        self.window = window;
        grow(
            device,
            &mut self.items,
            "meshlet items",
            items_buffer,
            storage_usage(),
        );
        grow(
            device,
            &mut self.draws,
            "meshlet draws",
            // And the soft list's count.
            (rows.len() * 16 + 16) as u64,
            Self::draws_usage(),
        );
        let (soft_first, soft_cap) = list.unwrap_or((0, 0));
        self.compact.prepare(
            device,
            queue,
            [slots, soft_first, soft_cap, rows.len() as u32],
            [&self.batches, &pools.meshes, &self.items, &self.draws],
        );
        // The visibility halves: a new size starts every slot hidden (the
        // late phase then finds what the early one missed).
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
        // The levels callers picked, after the views' rows.
        let level_row = planes.len();
        let picked = levels.iter().any(|&l| l != u8::MAX);
        if picked {
            planes.extend(level_rows(levels));
        }
        grow(
            device,
            &mut self.views,
            "meshlet views",
            (planes.len() * 16) as u64,
            storage_usage(),
        );
        queue.write_buffer(&self.views, 0, bytemuck::cast_slice(&planes));
        queue.write_buffer(&self.batches, 0, bytemuck::cast_slice(&rows));
        let draw_args: Vec<[u32; 4]> = rows
            .iter()
            .map(|_| [(CLUSTER_TRIANGLES * 3) as u32, 0, 0, 0])
            .chain(std::iter::once([0; 4]))
            .collect();
        queue.write_buffer(&self.draws, 0, bytemuck::cast_slice(&draw_args));
        // Two phases with the pyramid and its depth, one without.
        let pyramid = gc.pyramid();
        let depth = gc.occluder_depth().filter(|_| pyramid.is_some());
        let triangles_word = rows.len() as u32 * 4 + 1;
        let params = |phase: u32| Params {
            view_proj: view_proj.to_cols_array_2d(),
            lod: [
                scene.objects.lod_bias.max(0.0),
                level_row as f32,
                picked as u32 as f32,
                scene.frame.camera.near,
            ],
            sizes: [width, height, pyramid.map_or(0, |p| p.1), rows.len() as u32],
            phase: [phase, read as u32, write as u32, 0],
            // w: the word after the soft list's count that sums the
            // camera's drawn clusters' triangles (FrameStats::triangles).
            soft: match list {
                Some((first, cap)) => [first, cap, rows.len() as u32 * 4, triangles_word],
                None => [0, 0, 0, triangles_word],
            },
        };
        let (first, second) = if depth.is_some() { (1, 2) } else { (0, 0) };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params(first)));
        queue.write_buffer(
            &self.params,
            PARAMS_SLOT,
            bytemuck::bytes_of(&params(second)),
        );
        queue.write_buffer(
            &self.draw_params,
            0,
            bytemuck::bytes_of(&DrawParams {
                flags: [scene.objects.lod_debug as u32, 0, 0, 0],
                wind: winds.1,
            }),
        );
        // Each foliage mesh's deformation bound at this frame's wind and
        // the last drawn frame's (an instance's motion vector spans both),
        // into its mesh row (`meshlet_cull.wgsl` widens its spheres by it).
        for (slot, bounds, written) in &mut pools.foliage {
            // og.Wind's lean, or the wind field's this frame (`winds.2`,
            // driven or settling) where it leans further.
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
                queue.write_buffer(&pools.meshes, at as u64, bytemuck::bytes_of(&margin));
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
            self.vis.clone(),
        ];
        let view = pyramid.map(|p| p.0.clone());
        if self
            .cull_group
            .as_ref()
            .is_none_or(|(b, v, n, _)| *b != bound || *v != view || *n != selected_bytes)
        {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet cull"),
                layout: &self.cull_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.params,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                        }),
                    },
                    entry(1, &gc.instances),
                    entry(2, &self.views),
                    entry(3, &self.batches),
                    entry(4, &pools.meshes),
                    entry(5, &pools.clusters),
                    // Stable candidate slots, not append destinations.
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
                        resource: wgpu::BindingResource::TextureView(
                            view.as_ref().unwrap_or(&self.no_pyramid),
                        ),
                    },
                    entry(9, &self.vis),
                ],
            });
            self.cull_group = Some((bound, view, selected_bytes, group));
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
        let cull_group = &self.cull_group.as_ref().expect("made above").3;
        let dispatch = |pass: &mut wgpu::ComputePass<'_>, slot: u64| {
            pass.set_pipeline(&self.cull);
            pass.set_bind_group(0, cull_group, &[slot as u32]);
            pass.dispatch_workgroups(
                workgroups.min(WORKGROUPS_PER_ROW),
                workgroups.div_ceil(WORKGROUPS_PER_ROW),
                1,
            );
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
            // The early phase's camera clusters into the occluders' depth.
            for d in self.frame.iter().filter(|d| d.view == 0) {
                let b = &batches[d.batch as usize];
                if occluder(b) {
                    let key = (b.side, samples);
                    if !self.depth_pipelines.contains_key(&key) {
                        let pipeline = self.depth_pipeline(device, b.side, samples);
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
                        self.depth_pipelines.get(&(b.side, samples)),
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
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("meshlet late"),
                timestamp_writes: None,
            });
            gc.rebuild_pyramid(&mut pass);
            dispatch(&mut pass, PARAMS_SLOT);
        }
        // The camera's drawn triangles, into the word of GPU culling's
        // counts that takes them (`GpuCulling::meshlet_triangles_word`).
        if let Some(word) = gc.meshlet_triangles_word() {
            encoder.copy_buffer_to_buffer(
                &self.draws,
                triangles_word as u64 * 4,
                &gc.args,
                word as u64 * 4,
                4,
            );
        }
        // The soft list both phases appended, rasterised once.
        if let (Some(list), Some(pools)) = (list, self.pools.as_ref()) {
            self.vis_buffer.encode(
                device,
                queue,
                &mut encoder,
                vis::VisFrame {
                    view_proj,
                    size: (width, height),
                    list,
                    count: (&self.draws, rows.len() as u64 * 16),
                    instances: &gc.instances,
                    previous,
                    items: &self.items,
                    clusters: &pools.clusters,
                    indices: &pools.indices,
                    vertices: &pools.vertices,
                    lod_debug: scene.objects.lod_debug,
                    stylise: crate::Retro3d::uniform(scene.look.stylise.retro, width, height),
                    wind: winds.1,
                    extra: scene.look.stylise.material.surface_on() || !pools.foliage.is_empty(),
                    max_binding: self.max_binding,
                },
            );
        }
        Some(encoder.finish())
    }
}
