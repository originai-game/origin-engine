//! The aurora's GPU side: the footprint and table texture, and the pass that draws them.

use super::*;

/// The footprint and table texture the sky pass reads (its group's
/// binding 6), and what draws it: a 1×1 placeholder until a frame has an
/// aurora.
pub(crate) struct AuroraMap {
    pub view: wgpu::TextureView,
    texture: wgpu::Texture,
    pass: Option<FootprintPass>,
    /// The frame's footprint, planned ([`AuroraMap::update`]) and not yet
    /// recorded ([`AuroraMap::encode`]).
    pending: bool,
}

struct FootprintPass {
    /// Compiling on a worker from the frame that makes it until its first
    /// footprint is recorded ([`AuroraMap::encode`]): the footprint, then
    /// the pass that sums its light on the clouds into [`LIGHT_ROW`].
    pipeline: crate::deferred::Deferred<(wgpu::ComputePipeline, wgpu::ComputePipeline)>,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    /// Each footprint workgroup's share of that light.
    partials: wgpu::Buffer,
    /// The table's columns ([`columns`]), into every frame's uniforms.
    columns: [[f32; 4]; 2 * TABLE_ROWS as usize],
    sampler: wgpu::Sampler,
    strip: wgpu::TextureView,
    seed: u32,
    group: wgpu::BindGroup,
}

fn map_texture(device: &wgpu::Device, width: u32, height: u32, storage: bool) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("aurora footprint"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: if storage {
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
        } else {
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        },
        view_formats: &[],
    })
}

fn write_rows(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    y: u32,
    width: u32,
    texels: &[[f32; 4]],
) {
    let bytes: Vec<u8> = texels
        .iter()
        .flat_map(|t| t.map(crate::lightmap::f16_bits))
        .flat_map(u16::to_le_bytes)
        .collect();
    let rows = texels.len() as u32 / width;
    crate::traffic::write_texture(
        queue,
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 8),
            rows_per_image: Some(rows),
        },
        wgpu::Extent3d {
            width,
            height: rows,
            depth_or_array_layers: 1,
        },
    );
}

impl AuroraMap {
    pub(crate) fn placeholder(device: &wgpu::Device, queue: &wgpu::Queue) -> AuroraMap {
        let texture = map_texture(device, 1, 1, false);
        write_rows(queue, &texture, 0, 1, &[[0.0; 4]]);
        AuroraMap {
            view: texture.create_view(&Default::default()),
            texture,
            pass: None,
            pending: false,
        }
    }

    /// Whether the full texture exists (the sky group must be made again
    /// the frame it does).
    pub(crate) fn made(&self) -> bool {
        self.pass.is_some()
    }

    /// Makes the footprint texture, its table and the pass that draws it
    /// (once), and the strip again when the seed changed.
    fn ensure(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, seed: u32) {
        if self.pass.as_ref().is_some_and(|p| p.seed == seed) {
            return;
        }
        let strip_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aurora strip"),
            size: wgpu::Extent3d {
                width: STRIP_X as u32,
                height: STRIP_Y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: STRIP_MIPS,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for (level, (w, h, texels)) in strip_mips(&kh_strip(seed)).iter().enumerate() {
            crate::traffic::write_texture(
                queue,
                wgpu::TexelCopyTextureInfo {
                    texture: &strip_texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(texels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(*w as u32 * 4),
                    rows_per_image: Some(*h as u32),
                },
                wgpu::Extent3d {
                    width: *w as u32,
                    height: *h as u32,
                    depth_or_array_layers: 1,
                },
            );
        }
        let strip = strip_texture.create_view(&Default::default());
        if let Some(pass) = &mut self.pass {
            pass.group = footprint_group(
                device,
                &pass.layout,
                &pass.uniform,
                &strip,
                &pass.sampler,
                &self.view,
                &pass.partials,
            );
            pass.strip = strip;
            pass.seed = seed;
            return;
        }
        let texture = map_texture(device, FOOTPRINT, LIGHT_ROW + 1, true);
        let table = table(FOOTPRINT as usize);
        write_rows(queue, &texture, FOOTPRINT, FOOTPRINT, &table);
        let columns = columns(&table, FOOTPRINT as usize);
        let view = texture.create_view(&Default::default());
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("aurora footprint"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                entry(
                    3,
                    wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                ),
                entry(
                    4,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("aurora footprint"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = {
            let device = device.clone();
            crate::deferred::Deferred::anywhere(move || {
                crate::deferred::note(|| "aurora footprint".into());
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("aurora footprint"),
                    source: wgpu::ShaderSource::Wgsl(FOOTPRINT_WGSL.into()),
                });
                let compute = |entry| {
                    crate::traffic::create_compute_pipeline(
                        &device,
                        &wgpu::ComputePipelineDescriptor {
                            label: Some("aurora footprint"),
                            layout: Some(&pipeline_layout),
                            module: &module,
                            entry_point: Some(entry),
                            compilation_options: Default::default(),
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        },
                    )
                };
                (compute("main"), compute("light"))
            })
        };
        let groups = u64::from(FOOTPRINT / 8) * u64::from(FOOTPRINT / 8);
        let partials = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("aurora light"),
                size: groups * 32,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            },
        );
        crate::deferred::Start::start(&pipeline);
        let uniform = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("aurora footprint"),
                size: std::mem::size_of::<FootprintParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        // Along the curtain the strip repeats; across it, it ends.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("aurora strip"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let group = footprint_group(
            device, &layout, &uniform, &strip, &sampler, &view, &partials,
        );
        self.pass = Some(FootprintPass {
            pipeline,
            layout,
            uniform,
            partials,
            columns,
            sampler,
            strip,
            seed,
            group,
        });
        self.view = view;
        self.texture = texture;
    }

    /// Plans the footprint for `aurora` at `seconds` about the viewer at
    /// `viewer` (world metres) on a planet of `radius_km` and writes its
    /// uniforms, for [`AuroraMap::encode`] to record before every pass of
    /// the frame that reads it.
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        aurora: &Aurora,
        seconds: f64,
        viewer: Vec3,
        radius_km: f32,
    ) {
        self.ensure(device, queue, aurora.seed);
        let pass = self.pass.as_ref().expect("made above");
        let mut params = aurora.footprint(seconds, Vec2::new(viewer.x, viewer.z) * 0.001);
        params.sky = [radius_km, viewer.y.max(0.0) * 0.001, LIGHT_ROW as f32, 0.0];
        params.columns = pass.columns;
        crate::traffic::write_buffer(queue, &pass.uniform, 0, bytemuck::bytes_of(&params));
        self.pending = true;
    }

    /// Records the footprint [`AuroraMap::update`] planned: nothing without
    /// one. A frame planned and never recorded draws it at the next.
    pub(crate) fn encode(&mut self, encoder: &mut crate::Encoder) {
        let (true, Some(pass)) = (std::mem::take(&mut self.pending), &self.pass) else {
            return;
        };
        let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("aurora footprint"),
            timestamp_writes: None,
        });
        let (footprint, light) = pass.pipeline.get();
        compute.set_pipeline(footprint);
        compute.set_bind_group(0, &pass.group, &[]);
        compute.dispatch_workgroups(FOOTPRINT / 8, FOOTPRINT / 8, 1);
        compute.set_pipeline(light);
        compute.dispatch_workgroups(1, 1, 1);
    }
}

fn footprint_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    strip: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    out: &wgpu::TextureView,
    partials: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("aurora footprint"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(strip),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(out),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: partials.as_entire_binding(),
            },
        ],
    })
}

/// The footprint pass's compute shader.
pub(crate) const FOOTPRINT_WGSL: &str = wgsl_library!("aurora_footprint.wgsl");
