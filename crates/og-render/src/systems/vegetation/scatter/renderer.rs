use super::*;

impl ScatterRenderer {
    /// After the frame's submit: maps the counts the cull copied.
    pub(crate) fn after_submit(&mut self) {
        if let Some(cull) = &mut self.cull {
            cull.after_submit();
        }
    }

    /// Reads the cull's counts that have landed (each frame, before its
    /// stats).
    pub(crate) fn poll_counts(&mut self) {
        if let Some(cull) = &mut self.cull {
            cull.poll_counts();
        }
    }

    /// Natively: the last frame culled's counts, read now (blocking).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn finish_counts(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(cull) = &mut self.cull {
            cull.finish(device, queue);
        }
    }

    /// The GPU cull's newest counts ([`crate::ScatterCounts`]).
    pub(crate) fn counts(&self) -> Option<&crate::ScatterCounts> {
        self.cull.as_ref()?.counts.latest.as_ref()
    }

    /// Draw the lit pass into a target of `samples` per pixel from now on:
    /// its pipelines are made again as they are next used.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        if self.samples != samples {
            self.samples = samples;
            self.pipelines.clear();
        }
    }

    /// Makes the full variant of one of its lit or prepass pipelines that
    /// is not made yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        use crate::deferred::Warm;
        self.pipelines.values().any(|v| v.full().warm())
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: ScatterPipelines,
    ) -> ScatterRenderer {
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter"),
            entries: &uniform_entries(),
        });
        // The main camera's scatter group where it goes on in the prepass's
        // own depth (single-sampled, with compute): the uniform group's
        // bindings and the prepass's view depth, which the cards' lit stage
        // reads (impostor.wgsl `imp_view_depth`; a storage texture, as the
        // fragment stage has no sampled texture left).
        let card_depth_layout = (p.compute && p.samples == 1).then(|| {
            let mut entries = uniform_entries().to_vec();
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadOnly,
                    format: crate::post::LINDEPTH_FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            });
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scatter (prepass depth)"),
                entries: &entries,
            })
        });
        let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let lit_fx_layout = pl(
            "scatter lit",
            &[
                p.frame_layout,
                p.material_layout,
                &uniform_layout,
                p.fx_layout,
            ],
        );
        let card_depth = card_depth_layout.map(|g| {
            let l = pl(
                "scatter lit (prepass depth)",
                &[p.frame_layout, p.material_layout, &g, p.fx_layout],
            );
            (g, l)
        });
        let shadow_layout = pl(
            "scatter shadow",
            &[p.shadow_layout, p.material_layout, &uniform_layout],
        );
        let lit_source = (p.lit_wgsl)(&crate::hooks::default_hooks());
        let declared = Features::declared(&lit_source);
        let lit_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter"),
            source: wgpu::ShaderSource::Wgsl(lit_source.into()),
        });
        let shadow_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter shadow"),
            source: wgpu::ShaderSource::Wgsl(p.shadow_wgsl.into()),
        });
        let wind_uniform = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("scatter wind"),
                size: std::mem::size_of::<crate::wind::WindFieldGpu>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let blue_noise = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("scatter blue noise"),
                contents: crate::blue_noise::BLUE_NOISE,
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let frames = ImpostorFrames::new(device, queue, &[]);
        let (uniform, group) = Self::slots(
            device,
            &uniform_layout,
            [&blue_noise, &wind_uniform],
            [&p.wind_field, &p.trample, &frames.view],
            4,
        );
        ScatterRenderer {
            lit_fx_layout,
            card_depth,
            shadow_layout,
            lit_module,
            declared,
            features: Features::ALL,
            shadow_module,
            hooked: HashMap::new(),
            lit_wgsl: p.lit_wgsl,
            samples: p.samples,
            pipelines: HashMap::new(),
            shadow_pipelines: HashMap::new(),
            uniform_layout,
            uniform,
            blue_noise,
            slots: 4,
            group,
            chunks: HashMap::new(),
            shared: Vec::new(),
            frame: 0,
            built: 0,
            last_tick: None,
            wind_uniform,
            wind_field: p.wind_field,
            trample: p.trample,
            impostor_instances: None,
            frames,
            card_quad: crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("crowd card quad"),
                    contents: bytemuck::cast_slice(&[0u32, 1, 2, 3, 4, 5]),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ),
            card_instances: None,
            cull: p
                .compute
                .then(|| crate::scatter_cull::ScatterCull::new(device, queue, p.indirect_per_draw)),
            place: p
                .compute
                .then(|| crate::scatter_place::ScatterPlace::new(device)),
            placed_chunks: Vec::new(),
        }
    }
}

/// The level-0 tiles under a rectangle of the terrain's space.
pub(super) fn level0_under(layout: &TerrainLayout, a: Vec2, b: Vec2) -> Vec<u32> {
    let half = layout.size * 0.5;
    let size = layout.cell() * layout.tile as f32;
    let (tx, tz) = layout.tiles(0);
    // One sample around, for the normals at the rectangle's edge.
    let pad = layout.cell();
    let lo = ((a - pad + half) / size).floor().max(Vec2::ZERO);
    let hi = ((b + pad + half) / size).floor();
    let mut out = Vec::new();
    for z in (lo.y as u32)..=((hi.y.max(0.0) as u32).min(tz - 1)) {
        for x in (lo.x as u32)..=((hi.x.max(0.0) as u32).min(tx - 1)) {
            out.push(layout.index(0, x, z));
        }
    }
    out
}

/// The scatter group's entries (scatter_common.wgsl group 2).
fn uniform_entries() -> [wgpu::BindGroupLayoutEntry; 6] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(crate::blue_noise::BLUE_NOISE.len() as u64),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<crate::wind::WindFieldGpu>() as u64,
                ),
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }, // The impostors' card rectangles (ImpostorFrames), read in the vertex
        // stage by textureLoad: a storage buffer there needs
        // VERTEX_STORAGE.
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
    ]
}
