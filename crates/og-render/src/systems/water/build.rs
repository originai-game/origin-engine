use super::*;

impl WaterRenderer {
    // Made in the 3D world's engine module through its OnDemand closure
    // (renderer/resources.rs), never inlined into it.
    #[inline(never)]
    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        off: crate::pipeline_features::Features,
        samples: u32,
        compute: bool,
    ) -> WaterRenderer {
        let caustics_left_out = off.contains(crate::pipeline_features::Features::CAUSTICS);
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water"),
            entries: &[
                // (0, 1 and 7 are read by the height probe's compute too: `probe_sea`.)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // A height field's (a 1-layer rgba16float array, height in r) or the FFT sea's fields (ocean_fft:
                // three layers per cascade, mipped), sampled filtered.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                tex(2, true),
                tex(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(5, true),
                tex(6, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The frame's screen-space AO (white without): the reflection is darkened by it as
                // the scene behind already is (water.wgsl).
                tex(8, true),
                // The flow map (Water::flow_map; the fallback without).
                tex(9, true),
                // The seabed (Water::shore; one sample without), read by the vertices and the height probe too.
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // W3: the wakes of the bodies in the water (sea_wakes.rs; one calm texel without), read by a sea's
                // vertices and the height probe too.
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT
                        .union(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // A sea's displacement per clipmap grid point (`sea_grid_fill`; one calm texel without), read by its
                // vertices.
                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = crate::deferred::module(device, "water", water_source(off));
        let probe_module = module.clone();
        // The sea grid's fill (`sea_grid_fill`), where compute runs (the FFT sea, the one surface drawn from the grid,
        // needs it too).
        let grid = compute.then(|| {
            let grid_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sea grid"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: SEA_GRID_FORMAT,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                }],
            });
            let empty = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sea grid"),
                entries: &[],
            });
            let none = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sea grid"),
                layout: &empty,
                entries: &[],
            });
            let grid_pipeline = {
                let (device, module) = (device.clone(), module.clone());
                let layouts = [empty, layout.clone(), grid_layout.clone()];
                crate::deferred::Deferred::anywhere(move || {
                    use crate::deferred::AsModule;
                    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("sea grid"),
                        bind_group_layouts: &[
                            Some(&layouts[0]),
                            Some(&layouts[1]),
                            Some(&layouts[2]),
                        ],
                        immediate_size: 0,
                    });
                    crate::traffic::create_compute_pipeline(
                        &device,
                        &wgpu::ComputePipelineDescriptor {
                            label: Some("sea grid"),
                            layout: Some(&pl),
                            module: module.module(),
                            entry_point: Some("sea_grid_fill"),
                            compilation_options: Default::default(),
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        },
                    )
                })
            };
            (grid_layout, none, grid_pipeline)
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        // The surface's lit pipelines, as the variant for a frame's features
        // and the full one (pipeline_features.rs).
        let lit =
            |label: &'static str,
             vs: &'static str,
             buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
                let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
                let declared = module.declared;
                Variants::new(declared, move |features| {
                    let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
                    let constants = features.constants(declared);
                    crate::deferred::Sampled::anywhere(samples, move |samples| {
                        use crate::deferred::AsModule;
                        crate::hdr_pipeline_with(
                            &device,
                            &constants,
                            crate::HdrPipeline {
                                alpha_to_coverage: false,
                                label,
                                layout: &pl,
                                module: module.module(),
                                vs,
                                fs: "fs",
                                buffers,
                                cull: None,
                                depth_write: true,
                                depth_compare: wgpu::CompareFunction::Less,
                                blend: None,
                                samples,
                            },
                        )
                    })
                })
            };
        let mesh_pipeline = lit(
            "water mesh",
            "vs_mesh",
            &[Some(crate::POSITION_NORMAL_UV_LAYOUT)],
        );
        let pipeline = lit("water", "vs", &[]);
        let ao_depth_pipelines = {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            crate::deferred::Deferred::new(move || {
                use crate::deferred::AsModule;
                let m = module.module();
                [
                    ("water ao depth", "vs", &[][..]),
                    (
                        "water mesh ao depth",
                        "vs_mesh",
                        &[Some(crate::POSITION_NORMAL_UV_LAYOUT)][..],
                    ),
                ]
                .map(|(label, vs, buffers)| {
                    crate::traffic::create_render_pipeline(
                        &device,
                        &wgpu::RenderPipelineDescriptor {
                            label: Some(label),
                            layout: Some(&pl),
                            vertex: wgpu::VertexState {
                                module: m,
                                entry_point: Some(vs),
                                compilation_options: Default::default(),
                                buffers,
                            },
                            primitive: wgpu::PrimitiveState::default(),
                            depth_stencil: Some(wgpu::DepthStencilState {
                                format: crate::DEPTH_FORMAT,
                                depth_write_enabled: Some(false),
                                depth_compare: Some(wgpu::CompareFunction::Less),
                                stencil: Default::default(),
                                bias: Default::default(),
                            }),
                            multisample: wgpu::MultisampleState::default(),
                            fragment: Some(wgpu::FragmentState {
                                module: m,
                                entry_point: Some("fs_ao_depth"),
                                compilation_options: Default::default(),
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: crate::post::LINDEPTH_FORMAT,
                                    blend: None,
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                            }),
                            multiview_mask: None,
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        },
                    )
                })
            })
        };
        let spray_pipeline = crate::deferred_hdr(
            device,
            &pl,
            module.clone(),
            samples,
            move |layout, module| crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water spray",
                layout,
                module,
                vs: "vs_spray",
                fs: "fs_spray",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Less,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                samples,
            },
        );
        let waterline_pipeline = crate::deferred_hdr(
            device,
            &pl,
            module.clone(),
            samples,
            move |layout, module| crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water waterline",
                layout,
                module,
                vs: "vs_waterline",
                fs: "fs_waterline",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                samples,
            },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // The ripples' normal map: tiled, trilinear and 16x anisotropic (three.js water normals are RepeatWrapping
        // with mips): the clamped, mip-less water sampler blocked them up at grazing angles and seamed every tile.
        let ripple_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water ripples"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let flat = height_texture(device, 1).1;
        let no_wake = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("water no wake"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                // Bound as an array however many layers it has (WebGPU compatibility mode infers `2d` for one).
                texture_binding_view_dimension: Some(wgpu::TextureViewDimension::D2Array),
            })
            .create_view(&Default::default());
        let no_shore = {
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("water no shore"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            });
            t.create_view(&Default::default())
        };
        let no_grid = sea_grid_texture(device, 0, 1, false);
        WaterRenderer {
            no_shore,
            no_grid,
            grid,
            grid_fills: Vec::new(),
            shores: Vec::new(),
            ripple_sampler,
            layout,
            pipeline,
            mesh_pipeline,
            spray_pipeline,
            waterline_pipeline,
            ao_depth_pipelines,
            sampler,
            flat,
            slots: Vec::new(),
            ripple_pipelines: None,
            compute,
            fft: None,
            module: probe_module,
            last_sea: None,
            foam_steps: 0,
            wakes: None,
            wake: None,
            rivers: Vec::new(),
            no_wake,
            caustics: None,
            caustics_left_out,
        }
    }
}

/// A sea's clipmap grid ([`WaterRenderer::encode_fields`]): its displacement in xyz per grid point of `segments`
/// cells a side (segments + 1 points), a layer per level; written by `sea_grid_fill` and read by `sea_vertex`.
/// `storage`: written by the fill (the calm one bound without a sea is not).
pub(super) fn sea_grid_texture(
    device: &wgpu::Device,
    segments: u32,
    levels: u32,
    storage: bool,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("sea grid"),
            size: wgpu::Extent3d {
                width: segments + 1,
                height: segments + 1,
                depth_or_array_layers: levels,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SEA_GRID_FORMAT,
            usage: if storage {
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
            } else {
                wgpu::TextureUsages::TEXTURE_BINDING
            },
            view_formats: &[],
            texture_binding_view_dimension: None,
        })
        .create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
}

/// A height field's texture: `n` × `n` rgba16float, the height in r,
/// viewed as a 1-layer array (water.wgsl `heights`; rgba16float is
/// filterable, as the FFT sea's fields bound there are sampled, and a
/// storage format for a compute pass writing it).
pub(super) fn height_texture(device: &wgpu::Device, n: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("water heights"),
        size: wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HEIGHT_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
        // Bound as a 1-layer array (WebGPU compatibility mode infers `2d`).
        texture_binding_view_dimension: Some(wgpu::TextureViewDimension::D2Array),
    });
    let v = t.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    (t, v)
}
