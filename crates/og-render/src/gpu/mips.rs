//! Mip chains for uploaded textures, generated with render passes.

use crate::deferred::Deferred;
use std::collections::HashMap;

pub(crate) struct MipGenerator {
    shader: wgpu::ShaderModule,
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    /// One a format ([`MipGenerator::prepare`]), those of the uploaded
    /// textures started on the compile workers with the renderer
    /// ([`UPLOAD_FORMATS`]).
    pipelines: HashMap<wgpu::TextureFormat, Deferred<wgpu::RenderPipeline>>,
    spd_layout: wgpu::BindGroupLayout,
    spd_pipeline: Deferred<wgpu::ComputePipeline>,
    spd_dummy: wgpu::TextureView,
    spd_params: [wgpu::Buffer; 4],
}

/// The formats of the textures the renderer uploads with generated levels
/// (`upload_texture`): natively their pipelines start compiling when the
/// renderer is made, so a scene's first sync does not stop to make one
/// (about 60 ms on Metal, before any of the first frame's compiles began).
const UPLOAD_FORMATS: [wgpu::TextureFormat; 2] = [
    wgpu::TextureFormat::Rgba8UnormSrgb,
    wgpu::TextureFormat::Rgba8Unorm,
];

/// The number of levels down to 1x1.
pub(crate) fn mip_count(width: u32, height: u32) -> u32 {
    32 - width.max(height).max(1).leading_zeros()
}

/// A view of level `level` of `texture` alone: what a pass that fills one
/// level of a chain draws into or stores to, and what the pass after it
/// reads.
pub(crate) fn level_view(texture: &wgpu::Texture, label: &str, level: u32) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        base_mip_level: level,
        mip_level_count: Some(1),
        ..Default::default()
    })
}

/// [`level_view`] of every level of `texture`, from 0.
pub(crate) fn level_views(texture: &wgpu::Texture, label: &str) -> Vec<wgpu::TextureView> {
    (0..texture.mip_level_count())
        .map(|level| level_view(texture, label, level))
        .collect()
}

fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

fn static_u32x4(device: &wgpu::Device, label: &str, value: [u32; 4]) -> wgpu::Buffer {
    let buffer = crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some(label),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: true,
        },
    );
    buffer
        .slice(..)
        .get_mapped_range_mut()
        .expect("mapped at creation")
        .copy_from_slice(bytemuck::bytes_of(&value));
    buffer.unmap();
    buffer
}

impl MipGenerator {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let shader = crate::gpu::shader_module(
            device,
            wgpu::ShaderModuleDescriptor {
                label: Some("mipmap"),
                source: wgpu::ShaderSource::Wgsl(MIPMAP_WGSL.into()),
            },
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mipmap"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mipmap"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mipmap"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let spd_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mipmap spd"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                storage_entry(2),
                storage_entry(3),
                storage_entry(4),
                storage_entry(5),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let spd_pipeline = {
            let device = device.clone();
            let layout = spd_layout.clone();
            Deferred::anywhere(move || {
                let module = crate::gpu::shader_module(
                    &device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("mipmap spd"),
                        source: wgpu::ShaderSource::Wgsl(MIP_SPD_WGSL.into()),
                    },
                );
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("mipmap spd"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("mipmap spd"),
                        layout: Some(&pl),
                        module: &module,
                        entry_point: Some("cs_mip_spd"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        let spd_dummy = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("mipmap spd dummy"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            })
            .create_view(&Default::default());
        let spd_params =
            [1u32, 2, 3, 4].map(|n| static_u32x4(device, "mipmap spd params", [n, 0, 0, 0]));
        let mut mips = MipGenerator {
            shader,
            layout,
            pipeline_layout,
            sampler,
            pipelines: HashMap::new(),
            spd_layout,
            spd_pipeline,
            spd_dummy,
            spd_params,
        };
        for format in UPLOAD_FORMATS {
            mips.prepare(device, format);
            crate::deferred::Start::start(&mips.pipelines[&format]);
        }
        mips
    }

    /// Records passes filling levels 1.. of `texture` from level 0.
    pub(crate) fn generate(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        texture: &wgpu::Texture,
    ) {
        self.prepare(device, texture.format());
        self.record(device, encoder, texture);
    }

    /// Asks for the pipeline for `format` ([`MipGenerator::record`] needs
    /// it): made when a chain of the format is first recorded, and waited
    /// for there if it was started.
    pub(crate) fn prepare(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if format == wgpu::TextureFormat::Rgba16Float {
            crate::deferred::Start::start(&self.spd_pipeline);
        }
        if self.pipelines.contains_key(&format) {
            return;
        }
        let (device, layout, shader) = (
            device.clone(),
            self.pipeline_layout.clone(),
            self.shader.clone(),
        );
        self.pipelines.insert(
            format,
            Deferred::anywhere(move || pipeline(&device, &layout, &shader, format)),
        );
    }

    /// [`MipGenerator::generate`] for a format made with [`MipGenerator::prepare`].
    pub(crate) fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        texture: &wgpu::Texture,
    ) {
        self.record_chain(encoder, &self.chain(device, texture));
    }

    /// The views and bind groups that fill `texture`'s levels: made once
    /// for a texture whose levels are filled every frame
    /// ([`MipGenerator::record_chain`]). Making them each frame was 20
    /// views and 10 bind groups a frame for a 1280 x 720 copy, and a view
    /// of a texture the GPU is still reading waited for it on lavapipe.
    pub(crate) fn chain(&self, device: &wgpu::Device, texture: &wgpu::Texture) -> MipChain {
        let levels = level_views(texture, "mip");
        let use_spd = texture.format() == wgpu::TextureFormat::Rgba16Float
            && texture
                .usage()
                .contains(wgpu::TextureUsages::STORAGE_BINDING)
            && levels.len() > 1;
        let spd = use_spd.then(|| {
            let mut dispatches = Vec::new();
            let mut base = 0usize;
            while base + 1 < levels.len() {
                let count = (levels.len() - 1 - base).min(4);
                let dst0_w = (texture.width() >> (base + 1)).max(1);
                let dst0_h = (texture.height() >> (base + 1)).max(1);
                let dst0 = &levels[base + 1];
                let dst1 = levels.get(base + 2).unwrap_or(&self.spd_dummy);
                let dst2 = levels.get(base + 3).unwrap_or(&self.spd_dummy);
                let dst3 = levels.get(base + 4).unwrap_or(&self.spd_dummy);
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("mipmap spd"),
                    layout: &self.spd_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&levels[base]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(dst0),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(dst1),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(dst2),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(dst3),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: self.spd_params[count - 1].as_entire_binding(),
                        },
                    ],
                });
                dispatches.push(SpdDispatch {
                    group,
                    groups_x: dst0_w.div_ceil(8),
                    groups_y: dst0_h.div_ceil(8),
                });
                base += count;
            }
            dispatches
        });
        let groups = if spd.is_some() {
            Vec::new()
        } else {
            levels
                .iter()
                .take(levels.len().saturating_sub(1))
                .map(|source| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("mipmap"),
                        layout: &self.layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(source),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    })
                })
                .collect()
        };
        MipChain {
            format: texture.format(),
            levels,
            groups,
            spd,
        }
    }

    /// Records passes filling levels 1.. of `chain`'s texture from level 0.
    pub(crate) fn record_chain(&self, encoder: &mut crate::Encoder, chain: &MipChain) {
        if let Some(dispatches) = &chain.spd {
            let pipeline = self.spd_pipeline.wait();
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("mipmap spd"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            for d in dispatches {
                pass.set_bind_group(0, &d.group, &[]);
                pass.dispatch_workgroups(d.groups_x, d.groups_y, 1);
            }
            return;
        }
        let pipeline = self.pipelines[&chain.format].wait();
        for (target, bind_group) in chain.levels.iter().skip(1).zip(&chain.groups) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mipmap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::DontCare(unsafe { wgpu::LoadOpDontCare::enabled() }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

struct SpdDispatch {
    group: wgpu::BindGroup,
    groups_x: u32,
    groups_y: u32,
}

/// A texture's levels as views, and the bind group that reads each level
/// but the last ([`MipGenerator::chain`]).
pub(crate) struct MipChain {
    format: wgpu::TextureFormat,
    levels: Vec<wgpu::TextureView>,
    groups: Vec<wgpu::BindGroup>,
    spd: Option<Vec<SpdDispatch>>,
}

impl MipChain {
    /// The view of level `k` alone.
    pub(crate) fn level(&self, k: usize) -> &wgpu::TextureView {
        &self.levels[k]
    }

    /// Its levels, level 0 included.
    pub(crate) fn level_count(&self) -> u32 {
        self.levels.len() as u32
    }
}

/// The mip generator's pipeline into a target of `format`.
fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    crate::traffic::create_render_pipeline(
        device,
        &wgpu::RenderPipelineDescriptor {
            label: Some("mipmap"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        },
    )
}

/// The mip generator's module (`crate::shaders` validates it).
pub(crate) const MIPMAP_WGSL: &str = wgsl_library!("fullscreen.wgsl", "mipmap.wgsl");
/// The compute SPD mip generator's module (`crate::shaders` validates it).
pub(crate) const MIP_SPD_WGSL: &str = wgsl!("mip_spd.wgsl");
