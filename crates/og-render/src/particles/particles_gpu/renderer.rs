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
        crate::deferred::warm_one(&[&self.sim, &self.sprites_pipeline])
    }

    /// Reads what the scene's loaded emitters and effect materials can draw
    /// ([`reachable_draws`]), again only when either count changed.
    pub(crate) fn note_loaded(&mut self, scene: &crate::RenderScene) {
        let counts = (scene.emitters().len(), scene.effect_materials().len());
        if (self.reachable.0, self.reachable.1) != counts {
            self.reachable = (counts.0, counts.1, reachable_draws(scene));
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
    /// reach and no frame has made yet; false when none is left
    /// ([`crate::Renderer::warm`]): a composed effect loaded at start then
    /// draws its first frame without compiling.
    pub(crate) fn warm_loaded(&mut self, device: &wgpu::Device, samples: u32) -> bool {
        // Their simulation too, which a frame without particles never made.
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
            None => false,
        }
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
    ) -> ParticleRenderer {
        let sim_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE, true),
                uniform_entry(1, wgpu::ShaderStages::COMPUTE, false),
                depth_entry(2, wgpu::ShaderStages::COMPUTE),
                depth_entry(3, wgpu::ShaderStages::COMPUTE),
                storage_entry(4, true),
                storage_entry(5, true),
            ],
        });
        let sim_system_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("particle sim system"),
            entries: &[
                storage_entry(0, false),
                storage_entry(1, true),
                storage_entry(2, false),
            ],
        });
        let sim = {
            let (device, sim_layout, system_layout) = (
                device.clone(),
                sim_layout.clone(),
                sim_system_layout.clone(),
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
                    bind_group_layouts: &[Some(&sim_layout), Some(&system_layout)],
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
        let emitter_buffer = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("emitters"),
                size: EMITTER_STRIDE * 16,
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
            sim_layout,
            sim_system_layout,
            sim,
            draw_layout,
            draw: HashMap::new(),
            reachable: (0, 0, Vec::new()),
            draw_module,
            hook_modules: HashMap::new(),
            frame_wgsl: frame_wgsl.to_string(),
            draw_pl,
            effects,
            emitter_buffer,
            emitter_bytes: Vec::new(),
            entries: Vec::new(),
            cpu_vertices: cpu_vertex_buffer(device, PARTICLE_STRIDE * 256),
            cpu_bytes: Vec::new(),
            view_buffer,
            colliders: collider_buffer(device, 256),
            fields: collider_buffer(device, 256),
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
        let (simulate, clear, shift) = self.sim.wait();
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("particle sim"),
            timestamp_writes: None,
        });
        // The pass runs only when a system simulates here
        // (`ParticleFrame::any_gpu`).
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
        for fs in frame.systems.iter().filter(|s| s.gpu) {
            let Some((g, max)) = self
                .systems
                .get(&fs.id)
                .and_then(|s| Some((s.gpu.as_ref()?, s.max)))
            else {
                continue;
            };
            pass.set_bind_group(1, &g.group, &[]);
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
            if !self.draw.contains_key(&key) {
                self.make_draw(device, key, shaders);
            }
        }
        let _ = HDR_FORMAT;
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
        if let Some(h) = look.hook
            && !self.hook_modules.contains_key(&h)
        {
            let source = draw_source(&self.frame_wgsl, shaders.get(h));
            let module = crate::gpu::shader_module(
                device,
                wgpu::ShaderModuleDescriptor {
                    label: Some("particles (effect hook)"),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                },
            );
            self.hook_modules.insert(h, module);
        }
        let blend = match blend_of {
            _ if kind_of == DrawKind::Distort => wgpu::BlendState::ALPHA_BLENDING,
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
            // What is behind less the colour at its alpha.
            ParticleBlend::Subtractive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: KEEP_ALPHA,
            },
            // What is behind times the colour (`blend_out` mixes it
            // towards white by its alpha).
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
        // An effect mesh is blended like a billboard: no depth written.
        let solid = mesh && !look.effect_mesh;
        let module = match look.hook {
            Some(h) => self.hook_modules[&h].clone(),
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
        // An entry of the pipeline cache (gpu/deferred.rs), asked for now.
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
                    // Debris occludes what is behind it, and itself.
                    depth_write: solid,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    blend: Some(blend),
                    samples,
                },
            )
        });
        crate::deferred::Start::start(&p);
        self.draw.insert(key, p);
    }

    /// Draws the frame's 3D systems on a camera's layers (`mask`) that
    /// `part` selects, far to near. `distort`: the group carries the pass's scene copy, so its
    /// distortion systems draw (elsewhere they are left out). `view`: the
    /// camera's frustum, which skips a system whose bound it does not
    /// touch (None: every system draws). `mesh`: a
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
        view: Option<&crate::frame::Frustum>,
        part: ParticlePart<'_>,
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
            // A CPU-path system's ranges each start where they were packed;
            // a compute system's are its ring's slots.
            let (buffer, trails) = match s.packed {
                Some(_) => (&self.cpu_vertices, &self.cpu_vertices),
                None => {
                    let Some(g) = self.systems.get(&s.id).and_then(|st| st.gpu.as_ref()) else {
                        continue;
                    };
                    (&g.buffer, &g.trail)
                }
            };
            let ranges = s.slots.iter().enumerate().filter(|(_, r)| !r.is_empty());
            let ranges = ranges.map(|(i, r)| match s.packed {
                Some(p) => (p.particles[i], p.trails[i], 0..r.end - r.start),
                None => (0, 0, r.clone()),
            });
            let Some(pipeline) = (match part {
                ParticlePart::Oit(p) => Some(p),
                _ => self
                    .draw
                    .get(&(s.blend, samples, s.kind, s.look))
                    .and_then(|p| p.get()),
            }) else {
                continue;
            };
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, texture_group(s.texture), &[]);
            let draw_entry = s.first_entry + s.ticks;
            pass.set_bind_group(2, group, &[(draw_entry as u64 * EMITTER_STRIDE) as u32]);
            let (effect, offset) = self.effects.bind(s.effect);
            pass.set_bind_group(3, effect, &[offset]);
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
        encoder.copy_buffer_to_buffer(&g.buffer, 0, &staging, 0, size);
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
