use super::*;

impl ScatterRenderer {
    pub(super) fn slots(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        [blue_noise, wind]: [&wgpu::Buffer; 2],
        [wind_field, trample]: [&wgpu::TextureView; 2],
        n: u64,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scatter"),
            size: SLOT * n,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: blue_noise.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wind.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(wind_field),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(trample),
                },
            ],
        });
        (buffer, group)
    }

    pub(super) fn pipeline(
        &mut self,
        device: &wgpu::Device,
        shader: Option<usize>,
        side: Side,
        pass: ScatterPass,
        impostor: bool,
    ) {
        let kind = pass as u8 + if impostor { IMPOSTOR_KIND } else { 0 };
        // The shadow pass reads no material shader (copies cast as cut by the material's alpha).
        let shader = shader.filter(|_| pass != ScatterPass::Shadow);
        let key = if impostor {
            (None, Side::Double, kind)
        } else {
            (shader, side, kind)
        };
        if self.pipelines.contains_key(&key) || self.shadow_pipelines.contains_key(&key) {
            return;
        }
        if pass == ScatterPass::Shadow {
            let set = if impostor {
                crate::ShadowSet::new(|t| self.impostor_pipeline(device, pass, t))
            } else {
                // A local layer's pair only when a frame draws one (shadow_target.rs).
                let (device, layout, module) = (
                    device.clone(),
                    self.shadow_layout.clone(),
                    self.shadow_module.clone(),
                );
                crate::ShadowSet::lazy(move |t| {
                    Self::mesh_shadow_pipeline(&device, &layout, &module, side, t)
                })
            };
            self.shadow_pipelines.insert(key, set);
            return;
        }
        if impostor {
            let p = self.impostor_pipeline(device, pass, crate::ShadowTarget::Cascade);
            self.pipelines.insert(key, p);
            return;
        }
        let lit_module = match shader {
            Some(sh) => &self.hooked[&sh],
            None => &self.lit_module,
        };
        let buffers = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        let depth = |write| {
            Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(write),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            })
        };
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let p = match pass {
            ScatterPass::Lit => crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    // opaque copies write alpha 1 (every sample); alpha-to-coverage materials their coverage
                    alpha_to_coverage: true,
                    label: "scatter",
                    layout: &self.lit_fx_layout,
                    module: lit_module,
                    vs: "vs",
                    fs: "fs",
                    buffers: &buffers,
                    cull: crate::cull_mode(side),
                    depth_write: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    blend: None,
                    samples: self.samples,
                },
            ),
            ScatterPass::Prepass => {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("scatter prepass"),
                    layout: Some(&self.lit_layout),
                    vertex: wgpu::VertexState {
                        module: lit_module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &buffers,
                    },
                    primitive: wgpu::PrimitiveState {
                        cull_mode: crate::cull_mode(side),
                        ..Default::default()
                    },
                    depth_stencil: depth(true),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: lit_module,
                        entry_point: Some("fs_prepass"),
                        compilation_options: Default::default(),
                        targets: &[
                            target(crate::post::GBUFFER_FORMAT),
                            target(crate::post::LINDEPTH_FORMAT),
                            target(crate::post::OUTLINE_FORMAT),
                        ],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                })
            }
            ScatterPass::Shadow => unreachable!("made by mesh_shadow_pipeline"),
        };
        self.pipelines.insert(key, p);
    }

    /// A scattered mesh's shadow pipeline into `target`: as shadow.wgsl's
    /// casters (shadow_target.rs).
    fn mesh_shadow_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
        side: Side,
        target: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scatter shadow"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: target.cull(side),
                ..Default::default()
            },
            depth_stencil: Some(target.depth()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs_shadow"),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: crate::ShadowTarget::constants(side),
                    ..Default::default()
                },
                targets: &[Some(target.color())],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// An impostor pipeline (`impostor.wgsl`): the card has no vertex
    /// buffer but the instances' (slot 0), and both faces drawn.
    fn impostor_pipeline(
        &self,
        device: &wgpu::Device,
        pass: ScatterPass,
        // The shadow pass's (shadow_target.rs); the others ignore it.
        shadow: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(crate::INSTANCE_LAYOUT)];
        let depth = Some(if pass == ScatterPass::Shadow {
            shadow.depth()
        } else {
            wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let (layout, module, vs, fs, targets): (_, _, _, _, Vec<_>) = match pass {
            ScatterPass::Lit => {
                return crate::hdr_pipeline(
                    device,
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "impostor",
                        layout: &self.lit_fx_layout,
                        module: &self.lit_module,
                        vs: "vs_impostor",
                        fs: "fs_impostor",
                        buffers: &buffers,
                        cull: None,
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples: self.samples,
                    },
                );
            }
            ScatterPass::Prepass => (
                &self.lit_layout,
                &self.lit_module,
                "vs_impostor",
                "fs_impostor_prepass",
                vec![
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                ],
            ),
            ScatterPass::Shadow => (
                &self.shadow_layout,
                &self.shadow_module,
                "vs_impostor_shadow",
                "fs_impostor_shadow",
                vec![Some(shadow.color())],
            ),
        };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("impostor"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(vs),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: Default::default(),
            depth_stencil: depth,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fs),
                // Both faces cast (shadow_target.rs).
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: if pass == ScatterPass::Shadow {
                        crate::ShadowTarget::constants(Side::Double)
                    } else {
                        &[]
                    },
                    ..Default::default()
                },
                targets: &targets,
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }

    /// A crowd card pipeline (`crowd_card.wgsl`): instance records of
    /// [`CARD_LAYOUT`] (slot 0), indexed by the card quad, both faces.
    pub(super) fn card_pipeline(
        &self,
        device: &wgpu::Device,
        pass: ScatterPass,
        // The shadow pass's (shadow_target.rs); the others ignore it.
        shadow: crate::ShadowTarget,
    ) -> wgpu::RenderPipeline {
        let buffers = [Some(CARD_LAYOUT)];
        let depth = Some(if pass == ScatterPass::Shadow {
            shadow.depth()
        } else {
            wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let (layout, module, vs, fs, targets): (_, _, _, _, Vec<_>) = match pass {
            ScatterPass::Lit => {
                return crate::hdr_pipeline(
                    device,
                    crate::HdrPipeline {
                        alpha_to_coverage: true,
                        label: "crowd card",
                        layout: &self.lit_fx_layout,
                        module: &self.lit_module,
                        vs: "vs_crowd_card",
                        fs: "fs_crowd_card",
                        buffers: &buffers,
                        cull: None,
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples: self.samples,
                    },
                );
            }
            ScatterPass::Prepass => (
                &self.lit_layout,
                &self.lit_module,
                "vs_crowd_card",
                "fs_crowd_card_prepass",
                vec![
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                ],
            ),
            ScatterPass::Shadow => (
                &self.shadow_layout,
                &self.shadow_module,
                "vs_crowd_card_shadow",
                "fs_crowd_card_shadow",
                vec![Some(shadow.color())],
            ),
        };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("crowd card"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(vs),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            primitive: Default::default(),
            depth_stencil: depth,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(fs),
                // Both faces cast (shadow_target.rs).
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: if pass == ScatterPass::Shadow {
                        crate::ShadowTarget::constants(Side::Double)
                    } else {
                        &[]
                    },
                    ..Default::default()
                },
                targets: &targets,
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        })
    }
}
