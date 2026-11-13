//! The resolve pass: a view's HDR colour and depth into its target.

use super::*;

const VIEWS_WGSL: &str = wgsl!("views.wgsl");

const VIEWS_COLOR_WGSL: &str = wgsl!("views_color.wgsl");

const VIEWS_DEPTH_WGSL: &str = wgsl!("views_depth.wgsl");

/// The colour resolve's source (`fs_color`, `fs_linear`).
pub(crate) fn resolve_color_source() -> String {
    let lpm = wgsl!("hdr_lpm.wgsl");
    format!("{VIEWS_WGSL}{lpm}{VIEWS_COLOR_WGSL}")
}

/// The depth resolve's source (`fs_depth`): the view's linear depth
/// target normalised from near to far.
pub(crate) fn resolve_depth_source() -> String {
    format!("{VIEWS_WGSL}{VIEWS_DEPTH_WGSL}")
}

/// `Resolve` in views.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ResolveUniform {
    /// x: exposure multiplier, y: tone curve (0 ACES, 1 none, 2 AgX), z: 1
    /// mirrored across x, w: 1 encode sRGB in the shader.
    pub params: [f32; 4],
    /// x, y: the rect's origin in destination pixels, z, w: source size
    /// over rect size.
    pub rect: [f32; 4],
    /// x: near, y: far, z: 1 orthographic, w: source width.
    pub depth: [f32; 4],
    /// Display mapping for viewport destinations only; texture views retain radiance.
    pub hdr: [f32; 4],
}

/// The resolve's entry point for a destination.
pub(crate) fn resolve_entry(format: wgpu::TextureFormat, depth: bool) -> &'static str {
    match (depth, format) {
        (true, _) => "fs_depth",
        (false, wgpu::TextureFormat::Rgba16Float) => "fs_linear",
        _ => "fs_color",
    }
}

impl Views {
    /// Makes the resolve pipeline for a destination `format` and entry.
    pub fn pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        entry: &'static str,
    ) {
        self.pipelines.entry((format, entry)).or_insert_with(|| {
            let depth = entry == "fs_depth";
            let module = crate::shader(
                device,
                "view resolve",
                if depth {
                    resolve_depth_source()
                } else {
                    resolve_color_source()
                },
            );
            // The linear depth (RG32F) is not filterable: its layout
            // says so (a derived one would ask for a filterable float).
            let layout = depth.then(|| {
                let group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("view resolve depth"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
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
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("view resolve depth"),
                    bind_group_layouts: &[Some(&group)],
                    immediate_size: 0,
                })
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("view resolve"),
                layout: layout.as_ref(),
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
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        });
    }

    /// Writes `plan`'s resolve: its HDR (or depth) into `dest` of `format`,
    /// within `rect` (pixels) when given.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        plan: &ViewPlan,
        tonemap: crate::Tonemap,
        hdr: [f32; 4],
        dest: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        depth: bool,
    ) {
        let viewport = matches!(plan.dest, Dest::Viewport(_));
        let entry = if viewport {
            "fs_color"
        } else {
            resolve_entry(format, depth)
        };
        let slot = self.slots[plan.index].as_ref().expect("made for the plan");
        let rect = match plan.dest {
            Dest::Viewport(r) => r,
            Dest::Texture { .. } => [0, 0, plan.size.0, plan.size.1],
        };
        let u = ResolveUniform {
            params: [
                plan.ev.exp2(),
                match tonemap {
                    crate::Tonemap::Aces => 0.0,
                    crate::Tonemap::None => 1.0,
                    crate::Tonemap::Agx => 2.0,
                    crate::Tonemap::Neutral => 3.0,
                },
                plan.flip as u32 as f32,
                (!format.is_srgb() && format != wgpu::TextureFormat::Rgba16Float) as u32 as f32,
            ],
            rect: [rect[0] as f32, rect[1] as f32, 1.0, 1.0],
            depth: [
                plan.near,
                plan.far,
                plan.frame.orthographic as u32 as f32,
                plan.size.0 as f32,
            ],
            hdr: if viewport { hdr } else { [0.0; 4] },
        };
        queue.write_buffer(&slot.resolve_buffer, 0, bytemuck::bytes_of(&u));
        let pipeline = &self.pipelines[&(format, entry)];
        let layout = pipeline.get_bind_group_layout(0);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view resolve"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(if depth {
                        &slot.prepass.as_ref().expect("made for its depth").lindepth
                    } else {
                        &slot.hdr
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: slot.resolve_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("view resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dest,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_viewport(
            rect[0] as f32,
            rect[1] as f32,
            rect[2] as f32,
            rect[3] as f32,
            0.0,
            1.0,
        );
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
}
