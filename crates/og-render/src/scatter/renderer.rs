use super::*;

impl ScatterRenderer {
    /// Draw the lit pass into a target of `samples` per pixel from now on:
    /// its pipelines are made again as they are next used.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        if self.samples != samples {
            self.samples = samples;
            self.pipelines.clear();
        }
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: ScatterPipelines,
    ) -> ScatterRenderer {
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scatter"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ScatterGpu>() as u64
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            crate::blue_noise::BLUE_NOISE.len() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                            crate::wind::WindFieldGpu,
                        >() as u64),
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
                },
            ],
        });
        let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let lit_layout = pl(
            "scatter",
            &[p.frame_layout, p.material_layout, &uniform_layout],
        );
        let lit_fx_layout = pl(
            "scatter lit",
            &[
                p.frame_layout,
                p.material_layout,
                &uniform_layout,
                p.fx_layout,
            ],
        );
        let shadow_layout = pl(
            "scatter shadow",
            &[p.shadow_layout, p.material_layout, &uniform_layout],
        );
        let lit_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter"),
            source: wgpu::ShaderSource::Wgsl((p.lit_wgsl)(&crate::hooks::default_hooks()).into()),
        });
        let shadow_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scatter shadow"),
            source: wgpu::ShaderSource::Wgsl(p.shadow_wgsl.into()),
        });
        let wind_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scatter wind"),
            size: std::mem::size_of::<crate::wind::WindFieldGpu>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let blue_noise = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("scatter blue noise"),
                contents: crate::blue_noise::BLUE_NOISE,
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let (uniform, group) = Self::slots(
            device,
            &uniform_layout,
            [&blue_noise, &wind_uniform],
            [&p.wind_field, &p.trample],
            4,
        );
        ScatterRenderer {
            lit_layout,
            lit_fx_layout,
            shadow_layout,
            lit_module,
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
            culled_chunks: Vec::new(),
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
