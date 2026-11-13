//! The UI renderer's layouts, buffers and pipelines.

use super::*;

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        overlay: &OverlayRenderer,
        sprite_layout: &wgpu::BindGroupLayout,
    ) -> UiRenderer {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let uniform_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let backdrop_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui backdrop"),
            entries: &[tex(0), tex(1), sampler_entry(2), uniform_entry(3)],
        });
        let layer_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui layer"),
            entries: &[tex(0), sampler_entry(1), uniform_entry(2), tex(3)],
        });
        let shape_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui shapes"),
            bind_group_layouts: &[
                Some(&overlay.layout),
                Some(sprite_layout),
                Some(&backdrop_layout),
            ],
            immediate_size: 0,
        });
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui composite"),
            bind_group_layouts: &[
                Some(&overlay.layout),
                Some(&layer_layout),
                Some(&backdrop_layout),
            ],
            immediate_size: 0,
        });
        // A uniform and a texture only: group 0 already takes the four
        // storage buffers a stage may have, and a uniform holds 16 KiB.
        let mass_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui mass"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let mass_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui mass"),
            bind_group_layouts: &[
                Some(&overlay.layout),
                Some(sprite_layout),
                Some(&mass_layout),
            ],
            immediate_size: 0,
        });
        let mass_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui mass"),
            source: wgpu::ShaderSource::Wgsl(mass_source().into()),
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui"),
            source: wgpu::ShaderSource::Wgsl(ui_source().into()),
        });
        let builtin = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui builtin materials"),
            source: wgpu::ShaderSource::Wgsl(builtin_source().into()),
        });
        let pipelines = |format| {
            let p = |label, fs, pipe| {
                pipeline(
                    device,
                    label,
                    &shape_layout,
                    &module,
                    "vs_shape",
                    fs,
                    format,
                    blend_state(pipe),
                    false,
                )
            };
            UiPipelines {
                normal: p("ui shapes", "fs_shape", None),
                multiply: p("ui multiply", "fs_shape", Some(ShapePipe::Multiply)),
                screen: p("ui screen", "fs_shape", Some(ShapePipe::Screen)),
                add: p("ui add", "fs_shape", Some(ShapePipe::Add)),
                destination: p("ui blends", "fs_shape_dst", None),
                operators: OPERATORS.map(|o| p("ui operator", "fs_shape", Some(o))),
                composite_operators: OPERATORS.map(|o| {
                    pipeline(
                        device,
                        "ui composite operator",
                        &composite_layout,
                        &builtin,
                        "vs_shape",
                        "fs_layer",
                        format,
                        blend_state(Some(o)),
                        false,
                    )
                }),
                backdrop: p("ui backdrop", "fs_backdrop", None),
                vector: pipeline(
                    device,
                    "ui vectors",
                    &shape_layout,
                    &module,
                    "vs_vector",
                    "fs_vector",
                    format,
                    wgpu::BlendState::ALPHA_BLENDING,
                    true,
                ),
                composite: pipeline(
                    device,
                    "ui composite",
                    &composite_layout,
                    &builtin,
                    "vs_shape",
                    "fs_layer",
                    format,
                    wgpu::BlendState::ALPHA_BLENDING,
                    false,
                ),
                mass: mass_pipeline(device, &mass_pipeline_layout, &mass_module, format),
            }
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ui"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let white_tex = wgpu::util::DeviceExt::create_texture_with_data(
            device,
            queue,
            &wgpu::TextureDescriptor {
                label: Some("ui white"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255, 255, 255, 255],
        );
        let white = white_tex.create_view(&Default::default());
        let backdrop_params = uniform_buffer(device, "ui backdrop", 16);
        let dummy_backdrop = crate::bind_group(
            device,
            "ui backdrop (none)",
            &backdrop_layout,
            &[
                wgpu::BindingResource::TextureView(&white),
                wgpu::BindingResource::TextureView(&white),
                wgpu::BindingResource::Sampler(&sampler),
                backdrop_params.as_entire_binding(),
            ],
        );
        UiRenderer {
            output: pipelines(overlay.output_format),
            linear: overlay.linear_format.map(pipelines),
            layer: pipelines(LAYER_FORMAT),
            backdrop_layout,
            layer_layout,
            composite_layout,
            output_format: overlay.output_format,
            linear_format: overlay.linear_format,
            sampler,
            white,
            dummy_backdrop,
            backdrop_params,
            layer_behind: None,
            filters: super::super::filter::FilterRunner::new(device),
            layer_backdrop_params: uniform_buffer(device, "ui layer backdrop", 16),
            instances: storage_buffer(device, "ui instances", 256),
            gradients: storage_buffer(device, "ui gradients", 256),
            clips: storage_buffer(device, "ui clips", 256),
            points: storage_buffer(device, "ui points", 256),
            uniforms: Vec::new(),
            layer_params: Vec::new(),
            mass_camera: uniform_buffer(
                device,
                "ui mass camera",
                std::mem::size_of::<super::super::mass::MassCameraGpu>() as u64,
            ),
            mass_table: None,
            mass_instances: vertex_buffer(device, 4096),
            mass_layout,
            targets: Vec::new(),
            custom: HashMap::new(),
            vectors: HashMap::new(),
            frame: UiFrameGroups::default(),
        }
    }

    pub(super) fn pipelines(&self, kind: TargetKind) -> &UiPipelines {
        match kind {
            TargetKind::Output => &self.output,
            TargetKind::Linear => self.linear.as_ref().expect("an sRGB view's pipelines"),
            TargetKind::Layer => &self.layer,
        }
    }

    pub(super) fn format(&self, kind: TargetKind) -> wgpu::TextureFormat {
        match kind {
            TargetKind::Output => self.output_format,
            TargetKind::Linear => self.linear_format.expect("an sRGB view's format"),
            TargetKind::Layer => LAYER_FORMAT,
        }
    }
}
