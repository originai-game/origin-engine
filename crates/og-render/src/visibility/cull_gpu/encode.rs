//! A frame's culling passes.

use super::*;

impl GpuCulling {
    /// The occluders' depth pipeline for `side`, from the compact vertices
    /// when `compact` (`compact.rs`).
    pub(super) fn depth_pipeline(
        &mut self,
        device: &wgpu::Device,
        side: Side,
        compact: bool,
        samples: u32,
    ) {
        self.depth_pipelines
            .entry((side, compact, samples))
            .or_insert_with(|| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("occluder depth"),
                    layout: Some(&self.depth_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.depth_module,
                        entry_point: Some(if compact { "vs_compact" } else { "vs" }),
                        compilation_options: Default::default(),
                        buffers: &[
                            Some(if compact {
                                crate::compact::POSITION_LAYOUT
                            } else {
                                crate::VERTEX_LAYOUT
                            }),
                            Some(crate::INSTANCE_LAYOUT),
                        ],
                    },
                    primitive: wgpu::PrimitiveState {
                        cull_mode: cull_mode(side),
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: samples,
                        ..Default::default()
                    },
                    fragment: None,
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                })
            });
    }

    /// Grows a buffer to at least `size` bytes (dropping its bind groups).
    /// Whether it was replaced (a new buffer starts zeroed).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn grow(
        device: &wgpu::Device,
        buf: &mut wgpu::Buffer,
        groups: &mut Option<[wgpu::BindGroup; 2]>,
        label: &str,
        size: u64,
        cap: u64,
        usage: wgpu::BufferUsages,
    ) -> bool {
        let grown = size > buf.size();
        if grown {
            // Bound whole: never past the binding limit.
            *buf = buffer(
                device,
                label,
                size.next_power_of_two().min(cap).max(size),
                usage,
            );
            *groups = None;
        }
        grown
    }

    /// The frame's culling: uploads the sorted `transforms` and the batch
    /// table, and encodes both stages, the occluders' depth and the
    /// pyramid. Without an `Occluders` pass (see [`Self::primed`]) the
    /// occluders' depth and the pyramid are skipped and the camera pass is
    /// tested against its frustum alone. With `copy_counts` the counts
    /// are copied to a free readback (`stats_given`: those the frame's
    /// stats carry in their place). With `previous` (the sorted
    /// instances' previous transforms, TAA's object motion) the camera
    /// pass compacts them into [`Self::previous`] beside its own. Foliage
    /// batches copy them in every pass, carrying their current sway too. Returns
    /// the commands to submit before the frame's own and the occluder
    /// depth pass's draws.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        transforms: &[crate::instances::GpuTransform],
        previous: Option<&[crate::instances::GpuTransform]>,
        batches: &[Batch],
        skinned: Option<&SkinnedCull<'_>>,
        meshes: &[GpuMesh],
        materials: &[GpuMaterial],
        passes: &[CullPass],
        view_proj: Mat4,
        (width, height, samples): (u32, u32, u32),
        frame: u64,
        stats_given: GpuCounts,
        copy_counts: bool,
    ) -> (wgpu::CommandBuffer, u32) {
        // The skinned instances' boxes after the sorted instances, their
        // batches after the instances' (each `first` moved past them).
        let base = transforms.len() as u32;
        let ns = skinned.map_or(0, |s| s.len());
        let combined: Vec<crate::instances::GpuTransform>;
        let all_batches: Vec<Batch>;
        let (transforms, batches) = match skinned.filter(|_| ns > 0) {
            Some(s) => {
                combined = transforms
                    .iter()
                    .copied()
                    .chain(
                        s.boxes[..ns as usize]
                            .iter()
                            .map(|&(lo, hi)| box_transform(lo, hi)),
                    )
                    .collect();
                all_batches = batches
                    .iter()
                    .map(|x| Batch { ..*x })
                    .chain(s.batches.iter().map(|x| Batch {
                        first: x.first + base,
                        ..*x
                    }))
                    .collect();
                (&combined[..], &all_batches[..])
            }
            None => (transforms, batches),
        };
        let skinned_from = base as usize;
        let skinned_batch = |i: usize| ns > 0 && batches[i].first >= base;
        let n = transforms.len() as u32;
        let b = batches.len() as u32;
        let p = passes.len();
        self.n = n;
        self.b = b;
        self.skinned_n = ns;
        self.skinned_base = base;
        self.skinned_base_batch = batches.iter().take_while(|x| x.first < base).count() as u32;
        let occlusion = passes.iter().any(|x| x.kind == PassKind::Occluders);
        if occlusion {
            self.ensure_pyramid(device, width, height, samples);
            let occluder_kinds: Vec<(Side, bool)> = {
                let mut s: Vec<(Side, bool)> = batches
                    .iter()
                    .filter(|x| x.first < base && materials[x.material].occluder)
                    .map(|x| (x.side, x.compact))
                    .collect();
                og_core::sort::sort(&mut s);
                s.dedup();
                s
            };
            for (side, compact) in occluder_kinds {
                self.depth_pipeline(device, side, compact, samples);
            }
        }

        // Buffers (grown as needed; new ones start zeroed: nothing was
        // visible last frame).
        let blocks = n.div_ceil(WORKGROUP);
        let args_words = CountLayout::words(p, b);
        let instances_replaced = Self::grow(
            device,
            &mut self.instances,
            &mut self.groups,
            "culled instances",
            (p as u64 + 1) * n as u64 * INSTANCE_STRIDE,
            self.max_binding,
            Self::instance_usage(),
        );
        if ns > 0 {
            Self::grow(
                device,
                &mut self.skinned,
                &mut self.groups,
                "culled skinned instances",
                (p as u64 + 1) * ns as u64 * crate::skin::SKINNED_INSTANCE_STRIDE,
                self.max_binding,
                Self::instance_usage(),
            );
            let records = &skinned.expect("ns > 0").records[..ns as usize];
            queue.write_buffer(&self.skinned, 0, bytemuck::cast_slice(records));
        }
        let previous_uploaded = if let Some(prev) = previous {
            debug_assert_eq!(prev.len(), base as usize);
            if Self::grow(
                device,
                &mut self.previous,
                &mut self.groups,
                "culled previous instances",
                (p as u64 + 1) * n as u64 * INSTANCE_STRIDE,
                self.max_binding,
                Self::instance_usage(),
            ) {
                self.previous_upload.invalidate();
            }
            let bytes: &[u8] = bytemuck::cast_slice(prev);
            self.previous_upload.write(
                queue,
                &self.previous,
                bytes,
                std::slice::from_ref(&(0..bytes.len())),
            )
        } else {
            // Not written this frame: what it holds is not known next time.
            self.previous_upload.invalidate();
            0
        };
        Self::grow(
            device,
            &mut self.batches,
            &mut self.groups,
            "cull batches",
            b as u64 * std::mem::size_of::<BatchGpu>() as u64,
            self.max_binding,
            Self::storage_usage(),
        );
        let scan_replaced = Self::grow(
            device,
            &mut self.scan,
            &mut self.groups,
            "cull scan",
            Self::scan_words(n, p) * 4,
            self.max_binding,
            Self::storage_usage(),
        );
        if scan_replaced {
            self.primed = false;
        }
        Self::grow(
            device,
            &mut self.args,
            &mut self.groups,
            "cull draws",
            args_words * 4,
            self.max_binding,
            Self::args_usage(),
        );
        let count_bytes = args_words * 4;
        let readback_size = count_bytes.next_power_of_two().max(256);
        // A readback too small for this frame's counts is replaced once it
        // is free; one in flight keeps its frame's counts (its layout says
        // how many words it holds) until they are read.
        self.readbacks
            .retain(|r| r.layout.is_some() || r.buffer.size() >= count_bytes);
        while self.readbacks.len() < READBACKS {
            self.readbacks.push(Readback {
                buffer: buffer(
                    device,
                    "cull counts",
                    readback_size,
                    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                ),
                mapped: Arc::new(Mutex::new(None)),
                layout: None,
                unmapped: false,
                applied: GpuCounts::default(),
            });
        }

        // Uploads: of the transforms, only the blocks that changed (the
        // compute writes its outputs after them, never over them).
        if instances_replaced {
            self.upload.invalidate();
        }
        // The compute reads every one.
        let bytes: &[u8] = bytemuck::cast_slice(transforms);
        self.uploaded = previous_uploaded
            + self.upload.write(
                queue,
                &self.instances,
                bytes,
                std::slice::from_ref(&(0..bytes.len())),
            );
        let table: Vec<BatchGpu> = batches
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let m = &materials[x.material];
                if skinned_batch(i) {
                    // Boxes: their instances' matrices take the unit box
                    // to them. Never an occluder (not in its depth), never
                    // transparent here (the CPU sorts no skinned instance:
                    // each pass keeps those in view, blended or not).
                    return BatchGpu {
                        first: x.first,
                        count: x.count,
                        index_count: meshes[x.mesh].index_count,
                        flags: SKINNED
                            | (x.shader.is_none() as u32 * OCCLUDABLE)
                            | (!x.cast as u32 * NO_CAST),
                        bounds: [0.0, 0.0, 0.0, 3f32.sqrt()],
                        extent: [1.0; 3],
                        layers: x.layers,
                    };
                }
                let (c, e) = meshes[x.mesh].bounds.unwrap_or_default();
                // A compact batch's instance matrices take the unit box to
                // the world (compact.rs): its bounds are the unit box's.
                let (c, e, r) = match &meshes[x.mesh].compact {
                    Some((_, dq)) if x.compact => dq.unit_bounds(c, e),
                    _ => (c, e, e.length()),
                };
                BatchGpu {
                    first: x.first,
                    count: x.count,
                    index_count: meshes[x.mesh].index_count,
                    flags: (x.transparent as u32 * TRANSPARENT)
                        | (m.occluder as u32 * OCCLUDER)
                        | (x.shader.is_none() as u32 * OCCLUDABLE)
                        | (x.foliage as u32 * FOLIAGE),
                    bounds: [c.x, c.y, c.z, r],
                    extent: [e.x, e.y, e.z],
                    layers: x.layers,
                }
            })
            .collect();
        queue.write_buffer(&self.batches, 0, bytemuck::cast_slice(&table));
        queue.write_buffer(
            &self.view,
            0,
            bytemuck::cast_slice(&view_proj.to_cols_array()),
        );
        self.occlusion = occlusion;
        let pyramid = self.pyramid.as_ref().filter(|_| occlusion);
        // Pyramid levels 0: the camera pass tests no occlusion.
        let levels = pyramid.map_or(0, |x| x.levels);
        let main: Vec<usize> = (0..p)
            .filter(|&g| passes[g].kind == PassKind::Main)
            .collect();
        let others: Vec<usize> = (0..p)
            .filter(|&g| passes[g].kind != PassKind::Main)
            .collect();
        // The pass table: the other passes' stage first, the camera's
        // after; each stage reads its own from `motion.y`.
        let table: Vec<PassGpu> = others
            .iter()
            .chain(&main)
            .map(|&g| {
                let pass = &passes[g];
                let planes = &pass.planes[..pass.planes.len().min(MAX_PLANES)];
                let mut gpu: PassGpu = bytemuck::Zeroable::zeroed();
                for (i, plane) in planes.iter().enumerate() {
                    gpu.planes[i] = plane.to_array();
                }
                gpu.info = [pass.kind.code(), planes.len() as u32, g as u32, pass.layers];
                gpu
            })
            .collect();
        Self::grow(
            device,
            &mut self.passes,
            &mut self.groups,
            "cull passes",
            std::mem::size_of_val(table.as_slice()) as u64,
            self.max_binding,
            Self::storage_usage(),
        );
        queue.write_buffer(&self.passes, 0, bytemuck::cast_slice(&table));
        for (buffer, (which, base)) in self
            .params
            .iter()
            .zip([(&others, 0), (&main, others.len())])
        {
            if !which.is_empty() {
                let params = Params {
                    counts: [n, b, which.len() as u32, p as u32],
                    sizes: [blocks, levels, width, height],
                    motion: [
                        previous.is_some() as u32,
                        base as u32,
                        skinned_from as u32,
                        ns,
                    ],
                    view_proj: view_proj.to_cols_array_2d(),
                };
                queue.write_buffer(buffer, 0, bytemuck::bytes_of(&params));
            }
        }
        if self.groups.is_none() {
            let group = |params: &wgpu::Buffer| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("cull"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: params.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.instances.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.batches.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.scan.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: self.args.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(
                                self.pyramid.as_ref().map_or(&self.no_pyramid, |x| &x.full),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: self.previous.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: self.passes.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: self.skinned.as_entire_binding(),
                        },
                    ],
                })
            };
            self.groups = Some([group(&self.params[0]), group(&self.params[1])]);
        }
        let groups = self.groups.as_ref().expect("made above");

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("cull"),
        });
        // The inside and hidden counters and the crowds' start at zero.
        encoder.clear_buffer(
            &self.args,
            p as u64 * b as u64 * 20,
            Some((p as u64 + b as u64 + CROWD_COUNT_WORDS) * 4),
        );
        let run = |pass: &mut wgpu::ComputePass, group: &wgpu::BindGroup, stage_passes: u32| {
            pass.set_bind_group(0, group, &[]);
            pass.set_pipeline(&self.visibility);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
            pass.set_pipeline(&self.scan_blocks);
            pass.dispatch_workgroups(1, stage_passes, 1);
            pass.set_pipeline(&self.prefix);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
            pass.set_pipeline(&self.scatter);
            pass.dispatch_workgroups(blocks, stage_passes, 1);
        };
        let mut occluder_draws = 0;
        if let Some(pyramid) = pyramid {
            if !others.is_empty() {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("cull: shadows, reflection, occluders"),
                    timestamp_writes: None,
                });
                run(&mut pass, &groups[0], others.len() as u32);
            }
            let occluders = passes.iter().position(|x| x.kind == PassKind::Occluders);
            if let Some(g) = occluders {
                Self::copy_per_draw(
                    (&mut self.per_draw, &self.args, b),
                    device,
                    &mut encoder,
                    p as u32,
                    g as u32..g as u32 + 1,
                );
            }
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("occluder depth"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &pyramid.depth,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                if let Some(g) = occluders {
                    pass.set_bind_group(0, &self.view_group, &[]);
                    let mut kind = None;
                    for (i, x) in batches.iter().enumerate() {
                        if skinned_batch(i) || !materials[x.material].occluder {
                            continue;
                        }
                        if kind != Some((x.side, x.compact)) {
                            pass.set_pipeline(&self.depth_pipelines[&(x.side, x.compact, samples)]);
                            kind = Some((x.side, x.compact));
                        }
                        let mesh = &meshes[x.mesh];
                        let start = self.region(g as u32, x.first) as u64 * INSTANCE_STRIDE;
                        pass.set_vertex_buffer(0, mesh.vertex_buffer(x.compact).slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.instances
                                .slice(start..start + x.count as u64 * INSTANCE_STRIDE),
                        );
                        pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
                        let (args, at) = self.draw_args(g as u32, i as u32);
                        pass.draw_indexed_indirect(args, at);
                        occluder_draws += 1;
                    }
                }
            }
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cull: depth pyramid, camera"),
                timestamp_writes: None,
            });
            self.build_pyramid(&mut pass, pyramid);
            if !main.is_empty() {
                run(&mut pass, &groups[1], main.len() as u32);
            }
        } else {
            // No occluders: both stages in one pass, the camera's against
            // its frustum alone.
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cull: every pass, frustum only"),
                timestamp_writes: None,
            });
            if !others.is_empty() {
                run(&mut pass, &groups[0], others.len() as u32);
            }
            if !main.is_empty() {
                run(&mut pass, &groups[1], main.len() as u32);
            }
        }
        Self::copy_per_draw(
            (&mut self.per_draw, &self.args, b),
            device,
            &mut encoder,
            p as u32,
            0..p as u32,
        );
        self.primed |= !main.is_empty();
        // The counts, when asked for and a readback is free.
        let layout = CountLayout {
            frame,
            batches: b,
            kinds: passes.iter().map(|x| x.kind).collect(),
            // A pass that counts some batches counts the skinned ones it
            // draws too (all of them: they are never in a static cache).
            counted: passes
                .iter()
                .map(|x| {
                    x.counted.clone().map(|mut c| {
                        c.extend(self.skinned_base_batch..b);
                        c
                    })
                })
                .collect(),
            triangles: batches
                .iter()
                .map(|x| (meshes[x.mesh].index_count / 3) as u64)
                .collect(),
            // The camera pass's opaque instances on its layers, and every
            // skinned one (the CPU path culls them all).
            opaque: batches
                .iter()
                .enumerate()
                .filter(|&(i, x)| {
                    (!x.transparent || skinned_batch(i))
                        && passes
                            .iter()
                            .find(|p| p.kind == PassKind::Main)
                            .is_some_and(|p| x.layers & p.layers != 0)
                })
                .map(|(_, x)| x.count)
                .sum(),
        };
        self.copy = copy_counts;
        self.last = Some((layout, stats_given));
        (encoder.finish(), occluder_draws)
    }
}
