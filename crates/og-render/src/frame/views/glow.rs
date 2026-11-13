//! A view's glow chain: its glowing sprites blurred and added to its colour.

use super::*;

/// A view's glow chain: half its size, as many levels as the frame's bloom
/// chain at the view's size (`post::bloom_levels`), its glowing sprites
/// drawn into level 0, the chain run over it as the frame's bloom runs and
/// level 0 added onto its HDR image.
pub(crate) struct ViewGlow {
    pub(super) size: (u32, u32),
    /// Each level as a target, level 0 the one the glow sprites draw into.
    pub levels: Vec<wgpu::TextureView>,
    /// Each level's `[intensity, radius, width, height]` (views_glow.wgsl's
    /// `GlowChain`), the size the level's own, not its view's
    /// textureDimensions.
    pub(super) uniforms: Vec<wgpu::Buffer>,
    /// Each level as the source of a pass (its group).
    groups: Vec<wgpu::BindGroup>,
}

/// The glow chain's pipelines (down, up, the add) and what its groups are
/// made with.
pub(super) struct GlowAdd {
    down: wgpu::RenderPipeline,
    up: wgpu::RenderPipeline,
    add: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

/// The glow chain's module: its passes and the frame's bloom filters.
const VIEWS_GLOW_WGSL: &str =
    wgsl_library!("fullscreen.wgsl", "views_glow.wgsl", "bloom_chain.wgsl");

/// The glow chain's module (`fs_down`, `fs_up`, `fs`).
pub(crate) fn glow_source() -> String {
    VIEWS_GLOW_WGSL.into()
}

impl Views {
    /// Runs `plan`'s glow chain (its level 0 drawn) and adds it onto its
    /// HDR image: the frame's bloom passes (`FramePass::BloomDown`, then
    /// `BloomUp` from the smallest level), then level 0 as the tonemap adds
    /// `bloom.0`. Returns the draw calls.
    pub fn add_glow(&self, encoder: &mut crate::Encoder, plan: &ViewPlan) -> u32 {
        let slot = self.slots[plan.index].as_ref().expect("made for the plan");
        let (Some(g), Some(add)) = (&slot.glow, &self.glow_add) else {
            return 0;
        };
        let pass = |encoder: &mut crate::Encoder,
                    target: &wgpu::TextureView,
                    load: wgpu::LoadOp<wgpu::Color>,
                    pipeline: &wgpu::RenderPipeline,
                    group: &wgpu::BindGroup| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("view glow"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        };
        let n = g.levels.len();
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        for i in 0..n - 1 {
            pass(encoder, &g.levels[i + 1], clear, &add.down, &g.groups[i]);
        }
        for i in (0..n - 1).rev() {
            pass(
                encoder,
                &g.levels[i],
                wgpu::LoadOp::Load,
                &add.up,
                &g.groups[i + 1],
            );
        }
        pass(
            encoder,
            &slot.hdr,
            wgpu::LoadOp::Load,
            &add.add,
            &g.groups[0],
        );
        2 * (n as u32 - 1) + 1
    }
}

pub(super) fn glow_add(device: &wgpu::Device) -> GlowAdd {
    let module = crate::shader(device, "view glow", glow_source());
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("view glow"),
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("view glow"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let add = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    // The down passes write their level; the up passes and the add blend
    // onto theirs, as the frame's bloom up passes do.
    let pipeline = |entry: &str, blend: bool| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("view glow"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: crate::HDR_FORMAT,
                    blend: blend.then_some(wgpu::BlendState {
                        color: add,
                        alpha: add,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    };
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("view glow"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    GlowAdd {
        down: pipeline("fs_down", false),
        up: pipeline("fs_up", true),
        add: pipeline("fs", true),
        layout,
        sampler,
    }
}

pub(super) fn view_glow(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    add: &GlowAdd,
    (w, h): (u32, u32),
    levels: u32,
) -> ViewGlow {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("view glow"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::HDR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let levels = crate::mips::level_views(&texture, "view glow level");
    let uniforms: Vec<wgpu::Buffer> = (0..levels.len() as u32)
        .map(|level| {
            crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some("view glow"),
                    contents: bytemuck::bytes_of(&[
                        0.0f32,
                        0.0,
                        (w >> level).max(1) as f32,
                        (h >> level).max(1) as f32,
                    ]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            )
        })
        .collect();
    let groups = levels
        .iter()
        .zip(&uniforms)
        .map(|(level, uniform)| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("view glow"),
                layout: &add.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(level),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&add.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.as_entire_binding(),
                    },
                ],
            })
        })
        .collect();
    ViewGlow {
        size: (w, h),
        levels,
        uniforms,
        groups,
    }
}
