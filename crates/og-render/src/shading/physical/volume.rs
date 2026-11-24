//! On-demand screen-space volume exits and opaque depth. An integer
//! buffer keeps depth at f32 precision and the instance identity exact.

use std::collections::HashMap;

use crate::{Faces, INSTANCE_LAYOUT, VERTEX_LAYOUT, skin};

pub(crate) type Key = (Option<usize>, Faces, bool, bool);
pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

pub(crate) struct Volume {
    pub exits: wgpu::TextureView,
    pub depth: wgpu::TextureView,
    pub samples: u32,
    pub pipelines: HashMap<Key, wgpu::RenderPipeline>,
    depth_layout: wgpu::BindGroupLayout,
    depth_pipeline: wgpu::RenderPipeline,
}

/// WGSL module that seeds the transmission exit buffer from opaque scene depth.
pub(crate) fn depth_source(msaa: bool) -> String {
    let depth_read = if msaa {
        "@group(1) @binding(0) var opaque_depth: texture_multisampled_2d<f32>;
         fn opaque_depth_at(px: vec2<i32>) -> f32 {
           var z = 1.0;
           for (var i = 0u; i < textureNumSamples(opaque_depth); i++) {
             z = min(z, textureLoad(opaque_depth, px, i32(i)).r);
           }
           return z;
         }"
    } else {
        // An unfilterable float texture: compatibility mode loads no
        // texture_depth_2d.
        "@group(1) @binding(0) var opaque_depth: texture_2d<f32>;
         fn opaque_depth_at(px: vec2<i32>) -> f32 {
           return textureLoad(opaque_depth, px, 0).r;
         }"
    };
    [
        crate::FRAME_WGSL,
        depth_read,
        wgsl!("transmission_depth.wgsl"),
    ]
    .concat()
}

impl Volume {
    pub fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        samples: u32,
    ) -> Self {
        let view = |label, format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                    texture_binding_view_dimension: None,
                })
                .create_view(&Default::default())
        };
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("transmission opaque depth"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: samples > 1,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("transmission opaque depth"),
            bind_group_layouts: &[Some(frame_layout), Some(&depth_layout)],
            immediate_size: 0,
        });
        let module = crate::shader(device, "transmission depth", depth_source(samples > 1));
        let depth_pipeline = crate::traffic::create_render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("transmission opaque depth"),
                layout: Some(&layout),
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
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            },
        );
        Self {
            exits: view("transmission exits", FORMAT),
            depth: view("transmission exit depth", crate::DEPTH_FORMAT),
            samples,
            pipelines: HashMap::new(),
            depth_layout,
            depth_pipeline,
        }
    }

    pub fn seed_depth(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        opaque: &wgpu::TextureView,
        frame: &wgpu::BindGroup,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("transmission opaque depth"),
            layout: &self.depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(opaque),
            }],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("transmission opaque depth"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.exits,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.depth_pipeline);
        pass.set_bind_group(0, frame, &[]);
        pass.set_bind_group(1, &group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn prepare_pipeline(
        &mut self,
        device: &wgpu::Device,
        layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
        module: &wgpu::ShaderModule,
        key: Key,
    ) {
        self.pipelines.entry(key).or_insert_with(|| {
            let (_, faces, skinned, effects) = key;
            let constants = [("EFFECTS", f64::from(effects))];
            let options = wgpu::PipelineCompilationOptions {
                constants: &constants,
                ..Default::default()
            };
            let plain = [Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
            let skin = [
                Some(VERTEX_LAYOUT),
                Some(skin::SKINNED_INSTANCE_LAYOUT),
                Some(skin::SKIN_VERTEX_LAYOUT),
            ];
            crate::traffic::create_render_pipeline(
                device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some("transmission exits"),
                    layout: Some(if skinned { layouts.1 } else { layouts.0 }),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some(if skinned {
                            "vs_physical_skinned"
                        } else {
                            "vs_physical"
                        }),
                        compilation_options: options.clone(),
                        buffers: if skinned { &skin } else { &plain },
                    },
                    primitive: wgpu::PrimitiveState {
                        front_face: faces.front(),
                        cull_mode: Some(wgpu::Face::Front),
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: crate::DEPTH_FORMAT,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some("fs_volume_exit"),
                        compilation_options: options,
                        targets: &[Some(wgpu::ColorTargetState {
                            format: FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::RED
                                | wgpu::ColorWrites::GREEN
                                | wgpu::ColorWrites::BLUE,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(device).as_ref(),
                },
            )
        });
    }
}
