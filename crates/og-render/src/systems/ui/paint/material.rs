//! Custom materials over retained subtrees, using the overlay material's
//! source and uniform ABI. Targets and pipelines survive unchanged frames.

use super::*;
use crate::ui::{UiMaterialShader, pass::LayerParams, ui_material_module_source};
use og_draw::paint::PaintMaterial;
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Materials {
    pipelines: HashMap<usize, (UiMaterialShader, wgpu::RenderPipeline)>,
    targets: Vec<Option<Target>>,
}

struct Target {
    size: (u32, u32),
    view: wgpu::TextureView,
    uniform: wgpu::Buffer,
}

impl Materials {
    pub(super) fn holds(&self, view: &wgpu::TextureView) -> bool {
        self.targets.iter().flatten().any(|t| t.view == *view)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn record(
        &mut self,
        encoder: &mut crate::Encoder,
        cx: &GroupCx<'_>,
        index: usize,
        binding: PaintMaterial,
        source: &wgpu::TextureView,
        size: (u32, u32),
        css_size: [f32; 2],
    ) -> Option<wgpu::TextureView> {
        let shader = cx.materials.get(binding.shader.index())?;
        let key = binding.shader.index();
        if self
            .pipelines
            .get(&key)
            .is_none_or(|(old, _)| old != shader)
        {
            let module = crate::gpu::shader_module(
                cx.device,
                wgpu::ShaderModuleDescriptor {
                    label: Some("retained UI material"),
                    source: wgpu::ShaderSource::Wgsl(ui_material_module_source(shader).into()),
                },
            );
            let texture = wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            };
            let group1 = cx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("retained UI material"),
                    entries: &[
                        texture,
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
            let layout = cx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("retained UI material"),
                    bind_group_layouts: &[None, Some(&group1)],
                    immediate_size: 0,
                });
            let pipeline = crate::traffic::create_render_pipeline(
                cx.device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some("retained UI material"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &module,
                        entry_point: Some("vs_paint_material"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &module,
                        entry_point: Some("fs_paint_material"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: super::super::pass::LAYER_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(cx.device).as_ref(),
                },
            );
            self.pipelines.insert(key, (shader.clone(), pipeline));
        }
        if self.targets.len() <= index {
            self.targets.resize_with(index + 1, || None);
        }
        if self.targets[index].as_ref().is_none_or(|t| t.size != size) {
            let texture = cx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("retained UI material"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: super::super::pass::LAYER_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            });
            self.targets[index] = Some(Target {
                size,
                view: texture.create_view(&Default::default()),
                uniform: crate::traffic::create_buffer(
                    cx.device,
                    &wgpu::BufferDescriptor {
                        label: Some("retained UI material parameters"),
                        size: std::mem::size_of::<LayerParams>() as u64,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    },
                ),
            });
        }
        let target = self.targets[index].as_ref()?;
        let params = LayerParams {
            rect: [0.0, 0.0, css_size[0], css_size[1]],
            params: binding.params,
            info: [-1.0, 0.0, cx.time, 0.0],
            sample_uv: [1.0, 1.0, 0.5 / size.0 as f32, 0.5 / size.1 as f32],
            ..Default::default()
        };
        crate::traffic::write_buffer(cx.queue, &target.uniform, 0, bytemuck::bytes_of(&params));
        let pipeline = &self.pipelines[&key].1;
        let group = cx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("retained UI material"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(cx.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: target.uniform.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("retained UI material"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                depth_slice: None,
                resolve_target: None,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(1, &group, &[]);
        pass.draw(0..3, 0..1);
        Some(target.view.clone())
    }
}
