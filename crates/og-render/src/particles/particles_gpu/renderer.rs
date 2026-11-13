use super::*;

impl ParticleRenderer {
    /// Makes one compute pipeline not yet made; false when all are
    /// ([`crate::Renderer::warm`]).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.sim, &self.sprites_pipeline])
    }

    /// Carries every system across a move of the render origin (`origin.rs`;
    /// `d`, the old origin minus the new, is added to what is in world
    /// space): their last emitter transforms now, their world-space
    /// particles when they next simulate.
    pub(crate) fn shift(&mut self, d: Vec3) {
        let moved = |m: &mut Option<Mat4>| {
            if let Some(m) = m {
                m.w_axis += d.extend(0.0);
            }
        };
        for s in self.systems.values_mut() {
            s.shift += d;
            moved(&mut s.cpu.prev_transform);
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
    ) -> ParticleRenderer {
        let sim_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                uniform_entry(1, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(2, false),
                storage_entry(3, true),
                depth_entry(4, wgpu::ShaderStages::COMPUTE),
                depth_entry(5, wgpu::ShaderStages::COMPUTE),
                storage_entry(6, true),
            ],
        });
        let sim = {
            let (device, sim_layout) = (device.clone(), sim_layout.clone());
            Deferred::new(move || {
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("particle sim"),
                    source: wgpu::ShaderSource::Wgsl(sim_source().into()),
                });
                let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("particle sim"),
                    bind_group_layouts: &[Some(&sim_layout)],
                    immediate_size: 0,
                });
                let pipeline = |entry: &str| {
                    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("particle sim"),
                        layout: Some(&layout),
                        module: &module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    })
                };
                (pipeline("simulate"), pipeline("clear"), pipeline("shift"))
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
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("particle sprites"),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("particle sprites"),
                            bind_group_layouts: &[Some(&sprites_layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("particle sprites"),
                        source: wgpu::ShaderSource::Wgsl(sprites_source().into()),
                    }),
                    entry_point: Some("expand"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let sprites_params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle sprites"),
            size: SPRITE_PARAMS_STRIDE * 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle draw"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true),
                uniform_entry(1, wgpu::ShaderStages::VERTEX_FRAGMENT, false),
                depth_entry(2, wgpu::ShaderStages::FRAGMENT),
                depth_entry(3, wgpu::ShaderStages::FRAGMENT),
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
        let emitter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("emitters"),
            size: EMITTER_STRIDE * 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle view"),
            size: std::mem::size_of::<ParticleViewGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ParticleRenderer {
            sprites_layout,
            sprites_pipeline,
            sprites_params,
            sim_layout,
            sim,
            draw_layout,
            draw: HashMap::new(),
            draw_module,
            hook_modules: HashMap::new(),
            frame_wgsl: frame_wgsl.to_string(),
            draw_pl,
            effects,
            emitter_buffer,
            view_buffer,
            colliders: collider_buffer(device, 256),
            systems: HashMap::new(),
            scene_id: 0,
            points: HashMap::new(),
            frame: 0,
        }
    }
}

impl ParticleRenderer {
    /// Records the frame's compute dispatches (one per simulated tick).
    pub fn simulate(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        frame: &ParticleFrame,
        depth: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
    ) {
        let (simulate, clear, shift) = &*self.sim;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sim"),
            timestamp_writes: None,
        });
        for fs in frame.systems.iter().filter(|s| s.gpu) {
            let Some(g) = self.systems.get(&fs.id).and_then(|s| s.gpu.as_ref()) else {
                continue;
            };
            let max = self.systems[&fs.id].max;
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("particle sim"),
                layout: &self.sim_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.emitter_buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.view_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: g.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: g.points.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(gbuffer),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.colliders.as_entire_binding(),
                    },
                ],
            });
            let groups = max.div_ceil(64);
            if let Some(k) = fs.shift {
                pass.set_pipeline(shift);
                pass.set_bind_group(0, &group, &[(k as u64 * EMITTER_STRIDE) as u32]);
                pass.dispatch_workgroups(groups, 1, 1);
            }
            if fs.reset {
                pass.set_pipeline(clear);
                pass.set_bind_group(
                    0,
                    &group,
                    &[(fs.first_entry as u64 * EMITTER_STRIDE) as u32],
                );
                pass.dispatch_workgroups(groups, 1, 1);
            }
            pass.set_pipeline(simulate);
            for t in 0..fs.ticks {
                let offset = ((fs.first_entry + t) as u64 * EMITTER_STRIDE) as u32;
                pass.set_bind_group(0, &group, &[offset]);
                pass.dispatch_workgroups(groups, 1, 1);
            }
        }
    }

    /// The draw bind group (emitter entries, view, scene depth): the main
    /// camera's view buffer.
    pub fn draw_group(
        &self,
        device: &wgpu::Device,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.draw_group_of(device, &self.view_buffer, depth, scene_color)
    }

    /// The draw bind group of a camera whose [`ParticleViewGpu`] is in
    /// `view` (a secondary camera's, `views.rs`); `scene_color`: the scene
    /// copy distortion reads (a placeholder where the pass has none).
    pub fn draw_group_of(
        &self,
        device: &wgpu::Device,
        view: &wgpu::Buffer,
        depth: &wgpu::TextureView,
        scene_color: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle draw"),
            layout: &self.draw_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.emitter_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<EmitterGpu>() as u64),
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
        })
    }

    /// Makes the draw pipelines the frame needs (by blend and sample count).
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        frame: &ParticleFrame,
        samples: u32,
        shaders: &[crate::MaterialShader],
    ) {
        for s in frame.systems.iter().filter(|s| s.kind != DrawKind::Sprite) {
            let key = (s.blend, samples, s.kind, s.look);
            if self.draw.contains_key(&key) {
                continue;
            }
            if let Some(h) = s.look.hook
                && !self.hook_modules.contains_key(&h)
            {
                let source = draw_source(&self.frame_wgsl, shaders.get(h));
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("particles (effect hook)"),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });
                self.hook_modules.insert(h, module);
            }
            let blend = match s.blend {
                _ if s.kind == DrawKind::Distort => wgpu::BlendState::ALPHA_BLENDING,
                ParticleBlend::Additive => wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                },
                ParticleBlend::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            };
            let mesh = s.kind == DrawKind::Mesh;
            // An effect mesh is blended like a billboard: no depth written.
            let solid = mesh && !s.look.effect_mesh;
            let module = match s.look.hook {
                Some(h) => &self.hook_modules[&h],
                None => self.draw_module.get(),
            };
            let buffers = if mesh {
                vec![Some(PARTICLE_LAYOUT), Some(MESH_LAYOUT)]
            } else {
                vec![Some(PARTICLE_LAYOUT)]
            };
            let p = crate::hdr_pipeline(
                device,
                crate::HdrPipeline {
                    alpha_to_coverage: false,
                    label: "particles",
                    layout: &self.draw_pl,
                    module,
                    vs: if mesh { "vs_mesh" } else { "vs" },
                    fs: match s.kind {
                        DrawKind::Billboard => "fs",
                        DrawKind::Mesh => "fs_mesh",
                        DrawKind::Distort => "fs_distort",
                        DrawKind::Sprite => unreachable!("filtered above"),
                    },
                    buffers: &buffers,
                    cull: (mesh && !s.look.double_sided).then_some(wgpu::Face::Back),
                    // Debris occludes what is behind it, and itself.
                    depth_write: solid,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            );
            self.draw.insert(key, p);
        }
        let _ = HDR_FORMAT;
    }

    /// Draws the frame's 3D systems on a camera's layers (`mask`) that
    /// `part` selects, far to near. `distort`: the group carries the pass's scene copy, so its
    /// distortion systems draw (elsewhere they are left out). `mesh`: a
    /// mesh's f32 vertices, indices and index count. Returns the draws.
    #[allow(clippy::too_many_arguments)]
    pub fn draw<'b>(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        frame: &ParticleFrame,
        samples: u32,
        group: &wgpu::BindGroup,
        mask: u32,
        distort: bool,
        part: ParticlePart<'_>,
        texture_group: impl Fn(Option<crate::TextureHandle>) -> &'b wgpu::BindGroup,
        mesh: impl Fn(crate::MeshHandle) -> Option<(&'b wgpu::Buffer, &'b wgpu::Buffer, u32)>,
    ) -> u32 {
        let mut draws = 0;
        for s in frame.systems.iter().filter(|s| {
            s.layers & mask != 0
                && s.kind != DrawKind::Sprite
                && (distort || s.kind != DrawKind::Distort)
                && part.draws(s)
        }) {
            let Some(state) = self.systems.get(&s.id) else {
                continue;
            };
            let buffer = match s.child {
                Some(k) => state.children[k].as_ref().expect("uploaded"),
                None if s.gpu => &state.gpu.as_ref().expect("gpu system").buffer,
                None => state.upload.as_ref().expect("uploaded"),
            };
            let Some(pipeline) = (match part {
                ParticlePart::Oit(p) => Some(p),
                _ => self.draw.get(&(s.blend, samples, s.kind, s.look)),
            }) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, texture_group(s.texture), &[]);
            let draw_entry = s.first_entry + s.ticks;
            pass.set_bind_group(2, group, &[(draw_entry as u64 * EMITTER_STRIDE) as u32]);
            let (effect, offset) = self.effects.bind(s.effect);
            pass.set_bind_group(3, effect, &[offset]);
            pass.set_vertex_buffer(0, buffer.slice(..));
            match s.mesh.map(&mesh) {
                Some(Some((vertices, indices, count))) => {
                    pass.set_vertex_buffer(1, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..count, 0, 0..s.max);
                }
                Some(None) => continue,
                None => pass.draw(0..6, 0..s.max),
            }
            draws += 1;
        }
        draws
    }

    /// A copy of a system's particle slots (the GPU buffer is read back),
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
        let size = g.buffer.size();
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("particle readback"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&g.buffer, 0, &staging, 0, size);
        queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let data = slice.get_mapped_range().ok()?;
        let slots = bytemuck::cast_slice::<u8, ParticleState>(&data).to_vec();
        drop(data);
        staging.unmap();
        Some(slots)
    }
}
