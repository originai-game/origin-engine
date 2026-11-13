//! The meshlet path's pipelines and bind groups, made when first drawn.

use super::*;

impl MeshletGpu {
    /// The lit and prepass pipelines for the frame's clustered batches;
    /// `off`: the lit features the renderer is made without
    /// (pipeline_features.rs), out of the lit module.
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        planar: bool,
        off: crate::pipeline_features::Features,
        batches: &[Batch],
        prepass: bool,
        samples: u32,
    ) {
        if self.lit_module.as_ref().is_none_or(|(p, _)| *p != planar) {
            self.lit.clear();
            self.prepass.clear();
            self.vis_buffer.clear_pipelines();
            self.lit_module = Some((
                planar,
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("pbr meshlet"),
                    source: wgpu::ShaderSource::Wgsl(off.leave_out(lit_source(planar)).into()),
                }),
            ));
        }
        let module = &self.lit_module.as_ref().expect("made above").1;
        let soft: Vec<bool> = self
            .frame
            .iter()
            .filter(|d| d.soft)
            .map(|d| batches[d.batch as usize].effects)
            .collect();
        if !soft.is_empty() {
            self.vis_buffer
                .ensure_pipelines(device, layouts, module, soft, prepass, samples);
        }
        let camera = || self.frame.iter().filter(|d| d.lit);
        let missing = camera().any(|d| {
            let b = &batches[d.batch as usize];
            !self.lit.contains_key(&(b.side, b.effects))
                || (prepass && !self.prepass.contains_key(&b.side))
        });
        if !missing {
            return;
        }
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr meshlet"),
            bind_group_layouts: &[
                Some(layouts[0]),
                Some(layouts[1]),
                Some(layouts[2]),
                Some(&self.draw_layout),
            ],
            immediate_size: 0,
        });
        for d in self.frame.iter().filter(|d| d.lit) {
            let b = &batches[d.batch as usize];
            self.lit.entry((b.side, b.effects)).or_insert_with(|| {
                crate::hdr_pipeline_with(
                    device,
                    &[("EFFECTS", b.effects as u32 as f64)],
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "pbr meshlet",
                        layout: &layout,
                        module,
                        vs: "vs_meshlet",
                        fs: "fs",
                        buffers: &[],
                        cull: crate::cull_mode(b.side),
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples,
                    },
                )
            });
            if prepass {
                self.prepass.entry(b.side).or_insert_with(|| {
                    crate::prepass_pipeline(
                        device,
                        (&layout, &layout),
                        module,
                        b.side,
                        false,
                        None,
                        Some(("vs_prepass_meshlet", "fs_prepass", false)),
                        &[],
                    )
                });
            }
        }
    }

    /// The shadow casters' pipelines for the frame's shadow views: groups
    /// 0 the shadow pass's slot, 1 the material (alpha-tested and
    /// dissolving casters), 3 the meshlet draw group.
    pub fn ensure_shadow_pipelines(
        &mut self,
        device: &wgpu::Device,
        (pass, material): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
        batches: &[Batch],
    ) {
        let key = |d: &MeshletDraw| {
            let b = &batches[d.batch as usize];
            (b.side, b.dissolve)
        };
        let shadows = || self.frame.iter().filter(|d| !d.lit);
        if shadows().all(|d| self.shadow.contains_key(&key(d))) {
            return;
        }
        let module = self.shadow_module.get_or_insert_with(|| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shadow meshlet"),
                source: wgpu::ShaderSource::Wgsl(shadow_source().into()),
            })
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow meshlet"),
            bind_group_layouts: &[Some(pass), Some(material), None, Some(&self.draw_layout)],
            immediate_size: 0,
        });
        let keys: Vec<(Side, bool)> = self.frame.iter().filter(|d| !d.lit).map(key).collect();
        for k in keys {
            self.shadow.entry(k).or_insert_with(|| {
                crate::meshlet_shadow_pipeline(device, &layout, module, k.0, k.1)
            });
        }
    }

    /// The early phase's depth pipeline for `side` at `samples`.
    pub(super) fn depth_pipeline(
        &self,
        device: &wgpu::Device,
        side: Side,
        samples: u32,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("meshlet early depth"),
            layout: Some(&self.depth_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.depth_module,
                entry_point: Some("vs_depth"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: crate::cull_mode(side),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
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
    }

    /// Makes the draw groups of the frame's meshes: `instances` the
    /// sorted instances, `previous` their previous transforms at the same
    /// slots, `vertices(mesh)` a mesh's f32 vertex buffer.
    pub(super) fn ensure_groups<'a>(
        &mut self,
        device: &wgpu::Device,
        batches: &[Batch],
        (instances, previous): (&wgpu::Buffer, &wgpu::Buffer),
        vertices: impl Fn(usize) -> &'a wgpu::Buffer,
    ) {
        let Some(pools) = self.pools.as_ref() else {
            return;
        };
        for d in &self.frame {
            let mesh = batches[d.batch as usize].mesh;
            let v = vertices(mesh);
            let bound = vec![
                instances.clone(),
                self.items.clone(),
                pools.clusters.clone(),
                pools.indices.clone(),
                v.clone(),
                previous.clone(),
                pools.vertices.clone(),
            ];
            let bound = (bound, self.window);
            if self
                .mesh_groups
                .get(&mesh)
                .is_some_and(|(b, _)| *b == bound)
            {
                continue;
            }
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet draw"),
                layout: &self.draw_layout,
                entries: &[
                    entry(8, &self.draw_params),
                    entry(9, instances),
                    wgpu::BindGroupEntry {
                        binding: 10,
                        // A window from the batch's first item (the
                        // dynamic offset), within the buffer for every one.
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.items,
                            offset: 0,
                            size: wgpu::BufferSize::new(self.window),
                        }),
                    },
                    entry(11, &pools.clusters),
                    entry(12, &pools.indices),
                    entry(13, v),
                    entry(14, previous),
                    entry(17, &pools.vertices),
                    wgpu::BindGroupEntry {
                        binding: 18,
                        resource: wgpu::BindingResource::TextureView(&self.wind_field),
                    },
                ],
            });
            self.mesh_groups.insert(mesh, (bound, group));
        }
    }
}
