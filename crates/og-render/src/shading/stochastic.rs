//! Histogram transforms made when a material's maps change. Samples stay in
//! their original compressed textures; a 12 KiB uniform carries forward and
//! inverse 1D transforms, so no texture slot or decompressed map copy is added.

pub(crate) const WGSL: &str = wgsl!("stochastic_prepare.wgsl");
const SLOT_BYTES: u64 = 128 * 16;
const LOOKUP_BYTES: u64 = 6 * SLOT_BYTES;

pub(crate) struct Prepare {
    pipelines: std::cell::OnceCell<Pipelines>,
    neutral: wgpu::Buffer,
}

struct Pipelines {
    layout: wgpu::BindGroupLayout,
    count: crate::deferred::Deferred<wgpu::ComputePipeline>,
    transform: crate::deferred::Deferred<wgpu::ComputePipeline>,
}

impl Prepare {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        Self {
            pipelines: Default::default(),
            neutral: crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("unused stochastic lookup"),
                    size: LOOKUP_BYTES,
                    usage: wgpu::BufferUsages::UNIFORM,
                    mapped_at_creation: false,
                },
            ),
        }
    }

    pub(crate) fn lookup(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        maps: Option<[&wgpu::TextureView; 6]>,
    ) -> wgpu::Buffer {
        let Some(maps) = maps else {
            return self.neutral.clone();
        };
        let pipelines = self.pipelines.get_or_init(|| Pipelines::new(device));
        // Upload-time work must finish before any frame reads the lookup.
        crate::deferred::start_all(&[&pipelines.count, &pipelines.transform]);
        let output = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("stochastic lookup"),
                size: LOOKUP_BYTES,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("stochastic transforms"),
        });
        for (slot, map) in maps.into_iter().enumerate() {
            // Normals blend in tangent space, not through colour histograms.
            if slot == 3 {
                continue;
            }
            let histogram = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("texture histogram"),
                    size: 384 * 4,
                    usage: wgpu::BufferUsages::STORAGE,
                    mapped_at_creation: false,
                },
            );
            let lookup = crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some("texture quantiles"),
                    size: SLOT_BYTES,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                },
            );
            let group = crate::bind_group(
                device,
                "stochastic histogram",
                &pipelines.layout,
                &[
                    wgpu::BindingResource::TextureView(map),
                    histogram.as_entire_binding(),
                    lookup.as_entire_binding(),
                ],
            );
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_bind_group(0, &group, &[]);
            pass.set_pipeline(pipelines.count.wait());
            pass.dispatch_workgroups(
                map.texture().width().div_ceil(8),
                map.texture().height().div_ceil(8),
                1,
            );
            pass.set_pipeline(pipelines.transform.wait());
            pass.dispatch_workgroups(1, 1, 1);
            drop(pass);
            encoder.copy_buffer_to_buffer(
                &lookup,
                0,
                &output,
                slot as u64 * SLOT_BYTES,
                SLOT_BYTES,
            );
        }
        queue.submit(Some(encoder.finish()));
        output
    }
}

impl Pipelines {
    fn new(device: &wgpu::Device) -> Self {
        let storage = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("stochastic histogram"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(1),
                storage(2),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("stochastic histogram"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::deferred::module(device, "stochastic histogram", WGSL.into());
        let pipeline = |name: &'static str| {
            let (device, pipeline_layout, module) =
                (device.clone(), pipeline_layout.clone(), module.clone());
            crate::deferred::Deferred::anywhere(move || {
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some(name),
                        layout: Some(&pipeline_layout),
                        module: &module,
                        entry_point: Some(name),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        Self {
            layout,
            count: pipeline("count"),
            transform: pipeline("transform"),
        }
    }
}
