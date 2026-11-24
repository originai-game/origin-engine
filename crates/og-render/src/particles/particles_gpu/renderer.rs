use super::*;

/// A blend that leaves the target's alpha as it is.
const KEEP_ALPHA: wgpu::BlendComponent = wgpu::BlendComponent {
    src_factor: wgpu::BlendFactor::Zero,
    dst_factor: wgpu::BlendFactor::One,
    operation: wgpu::BlendOperation::Add,
};

impl ParticleRenderer {
    /// Makes one compute pipeline not yet made; false when all are
    /// ([`crate::Renderer::warm`]).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.sim, &self.compact_pipeline, &self.sprites_pipeline])
    }

    /// Reads what the scene's loaded emitters and effect materials can draw
    /// ([`reachable_draws`]), again only when either count changed.
    pub(crate) fn note_loaded(&mut self, scene: &crate::RenderScene) {
        let counts = (scene.emitters().len(), scene.effect_materials().len());
        if (self.reachable.0, self.reachable.1) != counts {
            self.reachable = (counts.0, counts.1, reachable_draws(scene));
            self.effect_meshes.note_loaded(scene.effect_materials());
        }
    }

    /// Whether a loaded emitter draws in the order-independent pass
    /// ([`in_oit`]'s systems): its OIT pipeline is warmed too.
    pub(crate) fn loaded_oit(&self) -> bool {
        self.reachable.2.iter().any(|&(b, k, l)| {
            k == DrawKind::Billboard && b == ParticleBlend::Alpha && l.hook.is_none()
        })
    }

    /// Makes one draw pipeline a loaded emitter or effect material can
    /// reach and no frame has made yet; false when none is left.
    pub(crate) fn warm_loaded(&mut self, device: &wgpu::Device, samples: u32) -> bool {
        if !self.reachable.2.is_empty() && self.warm() {
            return true;
        }
        let next = self
            .reachable
            .2
            .iter()
            .copied()
            .find(|&(b, k, l)| !self.draw.contains_key(&(b, samples, k, l)));
        match next {
            Some((blend, kind, look)) => {
                self.make_draw(device, (blend, samples, kind, look), &[]);
                true
            }
            None => self
                .effect_meshes
                .warm_loaded(device, &self.draw_pl, samples),
        }
    }

    /// Carries every system across a move of the render origin (`origin.rs`).
    pub(crate) fn shift(&mut self, d: Vec3) {
        let moved = |m: &mut Option<Mat4>| {
            if let Some(m) = m {
                m.w_axis += d.extend(0.0);
            }
        };
        for s in self.systems.values_mut() {
            s.shift += d;
            moved(&mut s.cpu.prev_transform);
            for p in &mut s.path {
                p.1 += d;
            }
            if let Some(g) = &mut s.gpu {
                moved(&mut g.prev_transform);
            }
        }
    }

    pub fn new(
        device: &wgpu::Device,
        effects: crate::effect::EffectRenderer,
        frame_layout: &wgpu::BindGroupLayout,
        sprite_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        samples: u32,
    ) -> ParticleRenderer {
        let sim_frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim frame"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                uniform_entry(1, wgpu::ShaderStages::COMPUTE, false),
                depth_entry(2, wgpu::ShaderStages::COMPUTE),
                depth_entry(3, wgpu::ShaderStages::COMPUTE),
                storage_entry(4, true),
                storage_entry(5, true),
            ],
        });
        let sim_pool_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim pool"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, false),
                storage_entry(4, false),
                storage_entry(5, false),
            ],
        });
        let sim = {
            let (device, frame_l, pool_l) = (
                device.clone(),
                sim_frame_layout.clone(),
                sim_pool_layout.clone(),
            );
            Deferred::new(move || {
                let module = crate::gpu::shader_module(
                    &device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("particle sim"),
                        source: wgpu::ShaderSource::Wgsl(sim_source().into()),
                    },
                );
                let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("particle sim"),
                    bind_group_layouts: &[Some(&frame_l), Some(&pool_l)],
                    immediate_size: 0,
                });
                let pipeline = |entry: &str| {
                    crate::traffic::create_compute_pipeline(
                        &device,
                        &wgpu::ComputePipelineDescriptor {
                            label: Some("particle sim"),
                            layout: Some(&layout),
                            module: &module,
                            entry_point: Some(entry),
                            compilation_options: Default::default(),
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        },
                    )
                };
                (pipeline("simulate"), pipeline("clear"), pipeline("shift"))
            })
        };
        let compact_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle compact"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                storage_entry(2, true),
                storage_entry(3, false),
                storage_entry(4, false),
                storage_entry(5, false),
                uniform_entry(6, wgpu::ShaderStages::COMPUTE, false),
            ],
        });
        let compact_pipeline = {
            let (device, compact_layout) = (device.clone(), compact_layout.clone());
            Deferred::new(move || {
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("particle compact"),
                        layout: Some(&device.create_pipeline_layout(
                            &wgpu::PipelineLayoutDescriptor {
                                label: Some("particle compact"),
                                bind_group_layouts: &[Some(&compact_layout)],
                                immediate_size: 0,
                            },
                        )),
                        module: &crate::gpu::shader_module(
                            &device,
                            wgpu::ShaderModuleDescriptor {
                                label: Some("particle compact"),
                                source: wgpu::ShaderSource::Wgsl(compact_source().into()),
                            },
                        ),
                        entry_point: Some("compact"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let sprites_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sprites"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                storage_entry(1, true),
                uniform_entry(2, wgpu::ShaderStages::COMPUTE, true),
                storage_entry(3, false),
            ],
        });
        let sprites_pipeline = {
            let (device, sprites_layout) = (device.clone(), sprites_layout.clone());
            Deferred::new(move || {
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("particle sprites"),
                        layout: Some(&device.create_pipeline_layout(
                            &wgpu::PipelineLayoutDescriptor {
                                label: Some("particle sprites"),
                                bind_group_layouts: &[Some(&sprites_layout)],
                                immediate_size: 0,
                            },
                        )),
                        module: &crate::gpu::shader_module(
                            &device,
                            wgpu::ShaderModuleDescriptor {
                                label: Some("particle sprites"),
                                source: wgpu::ShaderSource::Wgsl(sprites_source().into()),
                            },
                        ),
                        entry_point: Some("expand"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let sprites_params = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("particle sprites"),
                size: SPRITE_PARAMS_STRIDE * 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle draw"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true),
                uniform_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, false),
                depth_entry(2, wgpu::ShaderStages::FRAGMENT),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let draw_module =
            crate::deferred::module(device, "particles", draw_source(frame_wgsl, None));
        let draw_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("particles"),
            bind_group_layouts: &[
                Some(frame_layout),
                Some(sprite_layout),
                Some(&draw_layout),
                Some(&effects.layout),
            ],
            immediate_size: 0,
        });
        let emitter_draw_buffer = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("emitters (draw)"),
                size: EMITTER_DRAW_STRIDE * 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let view_buffer = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("particle view"),
                size: std::mem::size_of::<ParticleViewGpu>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        ParticleRenderer {
            sprites_layout,
            sprites_pipeline,
            sprites_params,
            sprites_params_gen: 1,
            sim_frame_layout,
            sim_pool_layout,
            sim,
            compact_layout,
            compact_pipeline,
            draw_layout,
            draw: HashMap::new(),
            reachable: (0, 0, Vec::new()),
            draw_module,
            hook_modules: HashMap::new(),
            frame_wgsl: frame_wgsl.to_string(),
            draw_pl,
            half_draw: HashMap::new(),
            half_res: crate::particles::half_res::HalfResFx::new(device, samples),
            effect_meshes: crate::particles::effect_mesh::EffectMeshRenderer::new(device),
            effects,
            emitter_draw_buffer,
            emitter_draw_gen: 1,
            emitter_draw_bytes: Vec::new(),
            draw_entries: Vec::new(),
            pool: ParticlePool::new(device),
            draw_groups: Mutex::new(Vec::new()),
            cpu_vertices: cpu_vertex_buffer(device, PARTICLE_STRIDE * 256),
            cpu_bytes: Vec::new(),
            view_buffer,
            colliders: collider_buffer(device, 256),
            colliders_gen: 1,
            fields: collider_buffer(device, 256),
            fields_gen: 1,
            systems: HashMap::new(),
            scene_id: 0,
            points: HashMap::new(),
            frame: 0,
        }
    }
}

impl ParticleRenderer {
    /// Records the frame's batched simulation waves and alive-compaction pass.
    pub fn simulate(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        depth: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
    ) {
        // Carry over any grown Mega-Pool buffers before touching them.
        let pending = {
            let mut cache = self.pool.cache.lock().expect("particle sim cache");
            std::mem::take(&mut cache.pending)
        };
        if pending.any() {
            self.pool.encode_copies(encoder, &pending);
        }

        if !frame.gpu_plan.waves.is_empty() {
            let (simulate, clear, shift) = self.sim.wait();
            let (frame_group, pool_group) = {
                let mut cache = self.pool.cache.lock().expect("particle sim cache");
                let fkey = (
                    depth.clone(),
                    gbuffer.clone(),
                    self.pool.sim_dispatch_gen,
                    self.colliders_gen,
                    self.fields_gen,
                );
                let fg = if let Some((k, g)) = &cache.sim_frame
                    && *k == fkey
                {
                    g.clone()
                } else {
                    let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("particle sim frame"),
                        layout: &self.sim_frame_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &self.pool.sim_dispatch,
                                    offset: 0,
                                    size: wgpu::BufferSize::new(
                                        std::mem::size_of::<SimDispatchGpu>() as u64,
                                    ),
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.view_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::TextureView(depth),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: wgpu::BindingResource::TextureView(gbuffer),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: self.colliders.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.fields.as_entire_binding(),
                            },
                        ],
                    });
                    cache.sim_frame = Some((fkey, g.clone()));
                    g
                };
                let pkey = [
                    self.pool.sim_entries_gen,
                    self.pool.sim_tiles_gen,
                    self.pool.points_gen,
                    self.pool.particles_gen,
                    self.pool.trails_gen,
                    self.pool.sub_counters_gen,
                ];
                let pg = if let Some((k, g)) = &cache.sim_pool
                    && *k == pkey
                {
                    g.clone()
                } else {
                    let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("particle sim pool"),
                        layout: &self.sim_pool_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.pool.sim_entries.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.pool.sim_tiles.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.pool.points.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.pool.particles.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: self.pool.trails.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.pool.sub_counters.as_entire_binding(),
                            },
                        ],
                    });
                    cache.sim_pool = Some((pkey, g.clone()));
                    g
                };
                (fg, pg)
            };
            // Each wave gets its own compute pass so storage-buffer hazards
            // between dependent waves (shift -> clear -> children -> parents)
            // synchronize cleanly across all backends including wgpu-flash.
            for wave in &frame.gpu_plan.waves {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("particle sim"),
                    timestamp_writes: None,
                });
                let pipeline = match wave.kind {
                    SimWaveKind::Shift => shift,
                    SimWaveKind::Clear => clear,
                    SimWaveKind::Simulate => simulate,
                };
                pass.set_pipeline(pipeline);
                pass.set_bind_group(
                    0,
                    &frame_group,
                    &[(u64::from(wave.dispatch_slot) * SIM_DISPATCH_STRIDE) as u32],
                );
                pass.set_bind_group(1, &pool_group, &[]);
                pass.dispatch_workgroups(wave.tile_count, 1, 1);
            }
        }

        if frame.gpu_plan.compact_tiles > 0 {
            let compact = self.compact_pipeline.wait();
            let group = {
                let key = [
                    self.pool.compact_tiles_gen,
                    self.pool.particles_gen,
                    self.pool.trails_gen,
                    self.pool.compact_particles_gen,
                    self.pool.compact_trails_gen,
                    self.pool.indirect_args_gen,
                ];
                let mut cache = self.pool.cache.lock().expect("particle sim cache");
                if let Some((k, g)) = &cache.compact
                    && *k == key
                {
                    g.clone()
                } else {
                    let g = device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("particle compact"),
                        layout: &self.compact_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.pool.compact_tiles.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.pool.particles.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.pool.trails.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.pool.compact_particles.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: self.pool.compact_trails.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self.pool.indirect_args.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 6,
                                resource: self.pool.compact_info.as_entire_binding(),
                            },
                        ],
                    });
                    cache.compact = Some((key, g.clone()));
                    g
                }
            };
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("particle compact"),
                timestamp_writes: None,
            });
            pass.set_pipeline(compact);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(frame.gpu_plan.compact_tiles, 1, 1);
        }
    }

    /// The draw bind group (emitter draw entries, view, scene depth): cached.
    pub fn draw_group(
        &self,
        device: &wgpu::Device,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.draw_group_of(device, &self.view_buffer, depth, scene_color)
    }

    /// Cached draw bind group for any camera's [`ParticleViewGpu`] buffer.
    pub fn draw_group_of(
        &self,
        device: &wgpu::Device,
        view: &wgpu::Buffer,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let key = (
            view.clone(),
            depth.clone(),
            scene_color.clone(),
            self.emitter_draw_gen,
        );
        let mut cache = self.draw_groups.lock().expect("particle draw groups");
        if let Some((_, g)) = cache.iter().find(|(k, _)| *k == key) {
            return g.clone();
        }
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle draw"),
            layout: &self.draw_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.emitter_draw_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<EmitterDrawGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(scene_color),
                },
            ],
        });
        if cache.len() >= 16 {
            cache.remove(0);
        }
        cache.push((key, group.clone()));
        group
    }

    /// Makes the draw pipelines the frame needs (by blend and sample count).
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        frame: &ParticleFrame,
        samples: u32,
        shaders: &[crate::MaterialShader],
    ) {
        let mut any_half = false;
        for s in frame.systems.iter().filter(|s| s.kind != DrawKind::Sprite) {
            let key = (s.blend, samples, s.kind, s.look);
            if !self.draw.contains_key(&key) {
                self.make_draw(device, key, shaders);
            }
            if s.half_res {
                any_half = true;
                if !self.half_draw.contains_key(&s.blend) {
                    self.make_half_draw(device, s.blend);
                }
            }
        }
        if any_half {
            self.half_res.start();
        }
        self.effect_meshes.ensure_pipelines(
            device,
            &self.draw_pl,
            &frame.effect_meshes,
            samples,
            shaders,
        );
    }

    /// Makes a half-resolution soft-particle pipeline for `blend` (sample_count
    /// 1, no depth attachment — occlusion and soft fade read `fx.half_depth`).
    fn make_half_draw(&mut self, device: &wgpu::Device, blend_of: ParticleBlend) {
        let blend = match blend_of {
            ParticleBlend::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: KEEP_ALPHA,
            },
            // Premultiplied radiance in .rgb + remaining background transmittance in .a.
            _ => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            },
        };
        let module = self.draw_module.get().clone();
        let (device, layout) = (device.clone(), self.draw_pl.clone());
        let p = crate::deferred::Deferred::anywhere(move || {
            crate::traffic::create_render_pipeline(
                &device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some("particles (half res)"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[Some(PARTICLE_LAYOUT)],
                    },
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("fs_half"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(blend),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                },
            )
        });
        crate::deferred::Start::start(&p);
        self.half_draw.insert(blend_of, p);
    }

    /// Returns (and caches) the particle draw shader module compiled with effect
    /// hook shader `h`, shared by both direct and OIT particle pipelines.
    pub(crate) fn hook_module(
        &mut self,
        device: &wgpu::Device,
        h: usize,
        shaders: &[crate::MaterialShader],
    ) -> wgpu::ShaderModule {
        if let Some(m) = self.hook_modules.get(&h) {
            return m.clone();
        }
        let source = draw_source(&self.frame_wgsl, shaders.get(h));
        let module = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("particles (effect hook)"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            },
        );
        self.hook_modules.insert(h, module.clone());
        module
    }

    /// Makes the draw pipeline for `key`, (blend, samples, kind, look),
    /// started on the pipeline cache.
    fn make_draw(
        &mut self,
        device: &wgpu::Device,
        key: (ParticleBlend, u32, DrawKind, Look),
        shaders: &[crate::MaterialShader],
    ) {
        let (blend_of, samples, kind_of, look) = key;
        let alpha_over = wgpu::BlendState {
            color: wgpu::BlendComponent::OVER,
            alpha: KEEP_ALPHA,
        };
        let blend = match blend_of {
            _ if kind_of == DrawKind::Distort => alpha_over,
            ParticleBlend::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: KEEP_ALPHA,
            },
            ParticleBlend::Alpha => alpha_over,
            ParticleBlend::Subtractive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: KEEP_ALPHA,
            },
            ParticleBlend::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: KEEP_ALPHA,
            },
        };
        let mesh = kind_of == DrawKind::Mesh;
        let trail = kind_of == DrawKind::Trail;
        let solid = mesh && !look.effect_mesh && blend_of == ParticleBlend::Alpha;
        let module = match look.hook {
            Some(h) => self.hook_module(device, h, shaders),
            None => self.draw_module.get().clone(),
        };
        let (device, layout) = (device.clone(), self.draw_pl.clone());
        let buffers = if mesh {
            vec![Some(PARTICLE_LAYOUT), Some(MESH_LAYOUT)]
        } else if trail {
            vec![Some(PARTICLE_LAYOUT), Some(TRAIL_LAYOUT)]
        } else {
            vec![Some(PARTICLE_LAYOUT)]
        };
        let (kind, double_sided) = (kind_of, look.double_sided);
        let p = crate::deferred::Deferred::anywhere(move || {
            crate::hdr_pipeline(
                &device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label: "particles",
                    layout: &layout,
                    module: &module,
                    vs: match kind {
                        DrawKind::Mesh => "vs_mesh",
                        DrawKind::Trail => "vs_trail",
                        _ => "vs",
                    },
                    fs: match kind {
                        DrawKind::Billboard | DrawKind::Trail => "fs",
                        DrawKind::Mesh => "fs_mesh",
                        DrawKind::Distort => "fs_distort",
                        DrawKind::Sprite => unreachable!("filtered above"),
                    },
                    buffers: &buffers,
                    cull: (mesh && !double_sided).then_some(wgpu::Face::Back),
                    depth_write: solid,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        });
        crate::deferred::Start::start(&p);
        self.draw.insert(key, p);
    }

    /// Draws the frame's 3D systems and instanced effect meshes on a camera's
    /// layers (`mask`) that `part` selects, far to near.
    #[allow(clippy::too_many_arguments)]
    pub fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &ParticleFrame,
        samples: u32,
        group: &wgpu::BindGroup,
        mask: u32,
        distort: bool,
        view: Option<&crate::frame::Frustum>,
        part: ParticlePart,
        oit_lookup: impl Fn(crate::oit::Source) -> Option<&'b wgpu::RenderPipeline>,
        texture_group: impl Fn(Option<crate::TextureHandle>) -> &'b wgpu::BindGroup,
        mesh: impl Fn(crate::MeshHandle) -> Option<(wgpu::BufferSlice<'b>, &'b wgpu::Buffer, u32)>,
    ) -> u32 {
        let mut draws = 0;
        for s in frame.systems.iter().filter(|s| {
            s.layers & mask != 0
                && s.kind != DrawKind::Sprite
                && (distort || s.kind != DrawKind::Distort)
                && part.draws(s)
                && !view
                    .zip(s.bound)
                    .is_some_and(|(f, (lo, hi))| !f.intersects_box(lo, hi))
        }) {
            let Some(pipeline) = (match part {
                ParticlePart::Oit => oit_source_of(s).and_then(&oit_lookup),
                ParticlePart::Half => self.half_draw.get(&s.blend).and_then(|p| p.get()),
                _ => self
                    .draw
                    .get(&(s.blend, samples, s.kind, s.look))
                    .and_then(|p| p.get()),
            }) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, texture_group(s.texture), &[]);
            pass.set_bind_group(
                2,
                group,
                &[(u64::from(s.draw_entry) * EMITTER_DRAW_STRIDE) as u32],
            );
            let (effect, offset) = self.effects.bind(s.effect);
            pass.set_bind_group(3, effect, &[offset]);

            if let VertexSource::Compact(c) = s.source {
                if s.kind == DrawKind::Trail {
                    pass.set_vertex_buffer(
                        0,
                        self.pool.compact_particles.slice(c.particles_offset..),
                    );
                    pass.set_vertex_buffer(1, self.pool.compact_trails.slice(c.trails_offset..));
                    pass.draw_indirect(&self.pool.indirect_args, c.indirect_offset);
                    draws += 1;
                    continue;
                }
                match s.mesh.map(&mesh) {
                    Some(Some((vertices, indices, count))) => {
                        pass.set_vertex_buffer(
                            0,
                            self.pool.compact_particles.slice(c.particles_offset..),
                        );
                        pass.set_vertex_buffer(1, vertices);
                        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                        if count == c.index_count {
                            pass.draw_indexed_indirect(&self.pool.indirect_args, c.indirect_offset);
                        } else {
                            let total: u32 = s.slots.iter().map(|r| r.end - r.start).sum();
                            pass.draw_indexed(0..count, 0, 0..total);
                        }
                        draws += 1;
                    }
                    Some(None) => continue,
                    None => {
                        pass.set_vertex_buffer(
                            0,
                            self.pool.compact_particles.slice(c.particles_offset..),
                        );
                        pass.draw_indirect(&self.pool.indirect_args, c.indirect_offset);
                        draws += 1;
                    }
                }
                continue;
            }

            let (buffer, trails, base_p, base_t) = match s.source {
                VertexSource::Cpu(_) => (&self.cpu_vertices, &self.cpu_vertices, 0u64, 0u64),
                VertexSource::Pool(ps) => (
                    &self.pool.particles,
                    &self.pool.trails,
                    ps.particles_offset,
                    ps.trails_offset,
                ),
                VertexSource::Compact(_) => unreachable!(),
            };
            let ranges = s.slots.iter().enumerate().filter(|(_, r)| !r.is_empty());
            let ranges = ranges.map(|(i, r)| match s.source {
                VertexSource::Cpu(p) => (p.particles[i], p.trails[i], 0..r.end - r.start),
                VertexSource::Pool(_) => (base_p, base_t, r.clone()),
                VertexSource::Compact(_) => unreachable!(),
            });
            if s.kind == DrawKind::Trail {
                for (at, trail_at, r) in ranges {
                    pass.set_vertex_buffer(0, buffer.slice(at..));
                    pass.set_vertex_buffer(1, trails.slice(trail_at..));
                    pass.draw(0..TRAIL_POINTS as u32 * 6, r);
                    draws += 1;
                }
                continue;
            }
            match s.mesh.map(&mesh) {
                Some(Some((vertices, indices, count))) => {
                    pass.set_vertex_buffer(1, vertices);
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    for (at, _, r) in ranges {
                        pass.set_vertex_buffer(0, buffer.slice(at..));
                        pass.draw_indexed(0..count, 0, r);
                        draws += 1;
                    }
                }
                Some(None) => continue,
                None => {
                    for (at, _, r) in ranges {
                        pass.set_vertex_buffer(0, buffer.slice(at..));
                        pass.draw(0..6, r);
                        draws += 1;
                    }
                }
            }
        }
        draws += self.effect_meshes.draw(
            pass,
            &frame.effect_meshes,
            samples,
            group,
            texture_group(None),
            &self.effects,
            mask,
            part,
            &oit_lookup,
            &mesh,
        );
        draws
    }

    /// A copy of a system's particle slots (read back from the Mega-Pool),
    /// for tests and tools (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
    ) -> Option<Vec<ParticleState>> {
        let state = self.systems.get(&id)?;
        let Some(g) = &state.gpu else {
            return Some(state.cpu.slots.clone());
        };
        let offset = u64::from(g.particle_base) * PARTICLE_STRIDE;
        let size = u64::from(state.max) * PARTICLE_STRIDE;
        let staging = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("particle readback"),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            },
        );
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.pool.particles, offset, &staging, 0, size);
        queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        crate::traffic::map_async(&slice, wgpu::MapMode::Read, |_| {});
        crate::traffic::poll(device, wgpu::PollType::wait_indefinitely()).ok()?;
        let data = slice.get_mapped_range().ok()?;
        let slots = bytemuck::cast_slice::<u8, ParticleState>(&data).to_vec();
        drop(data);
        staging.unmap();
        Some(slots)
    }
}
