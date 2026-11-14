//! The grass renderer: its pipelines and the buffers of each field.

use super::*;

/// What a blade takes of the shared shading: the weather, the effect
/// group at group 2 with the screen AO, and the decals.
const SHADING: crate::Shading = crate::Shading::lit().weather().effects(2).decals();

/// The grass lit module: the frame, the shared shading, then the blades.
pub(crate) fn lit_source() -> String {
    crate::assemble(
        &[
            crate::FRAME_WGSL,
            &SHADING.text(),
            crate::wind::WIND_COMMON_WGSL,
            crate::ground::GRASS_COMMON_WGSL,
            wgsl3d!("grass_blade.wgsl"),
            wgsl3d!("grass.wgsl"),
        ],
        &crate::Capability::OPAQUE_SURFACE,
    )
}

/// The grass shadow module.
pub(crate) fn shadow_source() -> String {
    format!(
        "{}{}{}{}{}{}",
        crate::LIBRARY_WGSL,
        crate::SHADOW_PASS_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::ground::GRASS_COMMON_WGSL,
        wgsl3d!("grass_blade.wgsl"),
        wgsl3d!("grass_shadow.wgsl"),
    )
}

/// The generation module over a terrain's group (`arrayed`: the
/// layer-array variant's).
pub(crate) fn generate_source(arrayed: bool) -> String {
    let s = format!(
        "{}{}{}{}{}{}{}",
        crate::LIBRARY_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::wind::WIND_LEAN_WGSL,
        crate::trample::READ_WGSL,
        crate::ground::GRASS_COMMON_WGSL,
        crate::ground::TERRAIN_WGSL,
        wgsl3d!("grass_generate.wgsl"),
    );
    if arrayed {
        crate::ground::terrain_array_declarations(&s)
    } else {
        s
    }
}

/// The three levels' strips, one index list after another, each with
/// vertex indices local to its level (row r: 2r and 2r + 1; the tip 2 rows).
pub(super) fn lod_indices() -> Vec<u32> {
    let mut out = Vec::new();
    for rows in LOD_ROWS {
        for r in 0..rows - 1 {
            let (l, rt) = (2 * r, 2 * r + 1);
            out.extend_from_slice(&[l, rt, l + 2, rt, rt + 2, l + 2]);
        }
        out.extend_from_slice(&[2 * (rows - 1), 2 * (rows - 1) + 1, 2 * rows]);
    }
    out
}

impl GrassRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: GrassPipelines,
    ) -> GrassRenderer {
        use wgpu::util::DeviceExt;
        let entry = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        };
        let buffer = |ty| wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let cs = wgpu::ShaderStages::COMPUTE;
        let rw = wgpu::BufferBindingType::Storage { read_only: false };
        let gen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass generate"),
            entries: &[
                entry(0, cs, buffer(wgpu::BufferBindingType::Uniform)),
                entry(1, cs, buffer(wgpu::BufferBindingType::Uniform)),
                entry(
                    2,
                    cs,
                    buffer(wgpu::BufferBindingType::Storage { read_only: true }),
                ),
                entry(3, cs, buffer(rw)),
                entry(4, cs, buffer(rw)),
                entry(5, cs, buffer(rw)),
                entry(6, cs, buffer(rw)),
                entry(
                    7,
                    cs,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    8,
                    cs,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let map = wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };
        let maps_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass maps"),
            entries: &[
                entry(0, cs, map),
                entry(1, cs, map),
                entry(2, cs, map),
                entry(
                    3,
                    cs,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });
        let white = device
            .create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    label: Some("grass no map"),
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
                &[255; 4],
            )
            .create_view(&Default::default());
        let map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("grass maps"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grass"),
            entries: &[
                entry(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    buffer(wgpu::BufferBindingType::Uniform),
                ),
                entry(
                    1,
                    wgpu::ShaderStages::VERTEX,
                    buffer(wgpu::BufferBindingType::Storage { read_only: true }),
                ),
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
        let tile: Vec<[f32; 2]> = blue_noise_tile().iter().map(|p| p.to_array()).collect();
        GrassRenderer {
            lit_layout: pl("grass lit", &[p.frame_layout, &draw_layout, p.fx_layout]),
            prepass_layout: pl("grass prepass", &[p.frame_layout, &draw_layout]),
            shadow_layout: pl("grass shadow", &[p.shadow_layout, &draw_layout]),
            gen_layout,
            maps_layout,
            white,
            map_sampler,
            draw_layout,
            wind_field: p.wind_field,
            trample: p.trample,
            samples: p.samples,
            off: p.off,
            pipelines: None,
            generate: [None, None],
            tile: crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("grass tile"),
                    contents: bytemuck::cast_slice(&tile),
                    usage: wgpu::BufferUsages::UNIFORM,
                },
            ),
            indices: crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("grass indices"),
                    contents: bytemuck::cast_slice(&lod_indices()),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ),
            fields: Vec::new(),
            last_tick: None,
        }
    }

    /// Draw the lit pass into a target of `samples` per pixel from now on.
    pub(crate) fn set_samples(&mut self, samples: u32) {
        if self.samples != samples {
            self.samples = samples;
            self.pipelines = None;
        }
    }

    pub(super) fn make_pipelines(&self, device: &wgpu::Device) -> [wgpu::RenderPipeline; 3] {
        let module = |label, src: String| {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(src.into()),
            })
        };
        let lit_module = module("grass", self.off.leave_out(lit_source()));
        let shadow_module = module("grass shadow", shadow_source());
        let lit = crate::hdr_pipeline(
            device,
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "grass",
                layout: &self.lit_layout,
                module: &lit_module,
                vs: "vs",
                fs: "fs",
                buffers: &[],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples: self.samples,
            },
        );
        let depth = Some(wgpu::DepthStencilState {
            format: crate::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        });
        let target = |format| {
            Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
        };
        let raster = |label, layout, module, vs, fs, targets: &[Option<wgpu::ColorTargetState>]| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: depth.clone(),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets,
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let prepass = raster(
            "grass prepass",
            &self.prepass_layout,
            &lit_module,
            "vs_prepass",
            "fs_prepass",
            &[
                target(crate::post::GBUFFER_FORMAT),
                target(crate::post::LINDEPTH_FORMAT),
                target(crate::post::OUTLINE_FORMAT),
            ],
        );
        let shadow = raster(
            "grass shadow",
            &self.shadow_layout,
            &shadow_module,
            "vs_shadow",
            "fs_shadow",
            &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        );
        [lit, prepass, shadow]
    }

    pub(super) fn make_generate(
        &self,
        device: &wgpu::Device,
        terrain_layout: &wgpu::BindGroupLayout,
        arrayed: bool,
    ) -> [wgpu::ComputePipeline; 3] {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("grass generate"),
            source: wgpu::ShaderSource::Wgsl(generate_source(arrayed).into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("grass generate"),
            bind_group_layouts: &[
                Some(&self.gen_layout),
                Some(terrain_layout),
                Some(&self.maps_layout),
            ],
            immediate_size: 0,
        });
        ["evaluate", "scan", "compact"].map(|entry| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("grass generate"),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        })
    }
}
