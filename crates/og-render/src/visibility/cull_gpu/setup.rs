//! GPU culling's buffers, layouts and pipelines.

use super::*;

impl GpuCulling {
    pub fn new(device: &wgpu::Device, max_binding: u64, per_draw: bool) -> GpuCulling {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cull"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, false),
                storage(2, true),
                storage(3, false),
                storage(4, false),
                texture(
                    5,
                    wgpu::TextureSampleType::Float { filterable: false },
                    false,
                ),
                storage(6, false),
                storage(7, true),
                storage(8, false),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cull"),
            source: wgpu::ShaderSource::Wgsl(CULL_WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cull"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let hiz_base_layout = [false, true].map(|multisampled| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("depth pyramid base"),
                entries: &[
                    level_out(1),
                    texture(2, wgpu::TextureSampleType::Depth, multisampled),
                ],
            })
        });
        let hiz_reduce_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth pyramid"),
            entries: &[
                texture(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    false,
                ),
                level_out(1),
            ],
        });
        let hiz_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("depth pyramid"),
            source: wgpu::ShaderSource::Wgsl(hiz_source(false).into()),
        });
        let reduce_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("depth pyramid"),
            bind_group_layouts: &[Some(&hiz_reduce_layout)],
            immediate_size: 0,
        });
        let view = buffer(
            device,
            "occluder view",
            64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("occluder view"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let view_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("occluder view"),
            layout: &view_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view.as_entire_binding(),
            }],
        });
        let params = std::array::from_fn(|_| {
            buffer(
                device,
                "cull params",
                std::mem::size_of::<Params>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            )
        });
        let readbacks = Vec::new();
        GpuCulling {
            stages: CullStages::new(device, &pipeline_layout, &module),
            hiz_reduce: compute_stage(device, &reduce_layout, &hiz_module, "reduce"),
            hiz_base: [None, None],
            hiz_base_layout,
            hiz_reduce_layout,
            depth_module: device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("occluder depth"),
                source: wgpu::ShaderSource::Wgsl(DEPTH_WGSL.into()),
            }),
            depth_pipeline_layout: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("occluder depth"),
                bind_group_layouts: &[Some(&view_layout)],
                immediate_size: 0,
            }),
            depth_pipelines: HashMap::new(),
            view,
            view_group,
            params,
            passes: buffer(device, "cull passes", 0, Self::storage_usage()),
            instances: buffer(device, "culled instances", 0, Self::instance_usage()),
            skinned: buffer(
                device,
                "culled skinned instances",
                0,
                Self::instance_usage(),
            ),
            skinned_n: 0,
            skinned_base: 0,
            upload: Default::default(),
            previous: buffer(
                device,
                "culled previous instances",
                0,
                Self::instance_usage(),
            ),
            previous_upload: Default::default(),
            uploaded: 0,
            batches: buffer(device, "cull batches", 0, Self::storage_usage()),
            scan: buffer(device, "cull scan", 0, Self::storage_usage()),
            args: buffer(device, "cull draws", 0, Self::args_usage()),
            layout,
            groups: None,
            pyramid: None,
            no_pyramid: device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("no depth pyramid"),
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default()),
            primed: false,
            occlusion: false,
            readbacks,
            latest: None,
            last: None,
            copy: false,
            max_binding,
            n: 0,
            b: 0,
            skinned_base_batch: 0,
            per_draw: per_draw.then(Default::default),
        }
    }

    pub(super) fn instance_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::COPY_DST
            | Self::debug_usage()
    }

    pub(super) fn storage_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | Self::debug_usage()
    }

    /// Read back by [`Self::debug_buffers`] in a dev build; a publish build
    /// never copies them out.
    pub(super) fn debug_usage() -> wgpu::BufferUsages {
        if cfg!(feature = "publish") {
            wgpu::BufferUsages::empty()
        } else {
            wgpu::BufferUsages::COPY_SRC
        }
    }

    /// Dev builds only: the cull's buffers by name, for a debug readback
    /// that compares one frame's culling between backends or browsers
    /// (og-tests-web's `og_gfx_cull_buffer`).
    #[cfg(not(feature = "publish"))]
    pub fn debug_buffers(&self) -> [(&'static str, &wgpu::Buffer); 7] {
        [
            ("passes", &self.passes),
            ("batches", &self.batches),
            ("scan", &self.scan),
            ("args", &self.args),
            ("instances", &self.instances),
            ("previous", &self.previous),
            ("skinned", &self.skinned),
        ]
    }

    pub(super) fn args_usage() -> wgpu::BufferUsages {
        wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST
    }
}
