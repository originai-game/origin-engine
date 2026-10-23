//! The 2D light map: ambient plus every [`Light2d`], shadowed by
//! [`Occluder2d`]s with soft penumbrae (`shaders/light2d.wgsl`). Lit sprite
//! layers multiply their colour with it in the sprite shader.

use crate::HDR_FORMAT;
use crate::deferred::{AsModule, Deferred};
use crate::effects::{Light2d, MAX_LIGHTS_2D, MAX_OCCLUDERS_2D, Occluder2d};
use glam::{Mat4, Vec3};

pub(crate) const LIGHT2D_WGSL: &str = wgsl!("light2d.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Lights2dUniform {
    view_proj: [[f32; 4]; 4],
    info: [f32; 4],
    occluders: [[f32; 4]; 2 * MAX_OCCLUDERS_2D],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightInstance {
    pos_radius: [f32; 4],
    color_source: [f32; 4],
    flags: [f32; 4],
}

const LIGHT_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<LightInstance>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
};

pub(crate) struct Light2dRenderer {
    pipeline: Deferred<wgpu::RenderPipeline>,
    uniform: wgpu::Buffer,
    instances: wgpu::Buffer,
    group: wgpu::BindGroup,
    count: u32,
}

impl Light2dRenderer {
    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.pipeline])
    }

    pub fn new(device: &wgpu::Device) -> Light2dRenderer {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lights 2d"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d"),
            size: std::mem::size_of::<Lights2dUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d"),
            size: (std::mem::size_of::<LightInstance>() * MAX_LIGHTS_2D) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights 2d"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        // Made when a frame first draws 2D lights (deferred.rs).
        let module = crate::deferred::module(device, "lights 2d", LIGHT2D_WGSL.into());
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lights 2d"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = {
            let (device, pl) = (device.clone(), pl.clone());
            Deferred::new(move || {
                let module = module.module();
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("lights 2d"),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[Some(LIGHT_LAYOUT)],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some("fs"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Light2dRenderer {
            pipeline,
            uniform,
            instances,
            group,
            count: 0,
        }
    }

    /// Writes the frame's lights and occluders (beyond the limits they are
    /// dropped); returns (lights, occluders) drawn.
    pub fn write(
        &mut self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        lights: &[Light2d],
        occluders: &[Occluder2d],
    ) -> (u32, u32) {
        let mut u: Lights2dUniform = bytemuck::Zeroable::zeroed();
        u.view_proj = view_proj.to_cols_array_2d();
        let occ = occluders.len().min(MAX_OCCLUDERS_2D);
        u.info = [occ as f32, 0.0, 0.0, 0.0];
        for (i, o) in occluders.iter().take(occ).enumerate() {
            let (s, c) = o.rotation.sin_cos();
            u.occluders[i * 2] = [o.center.x, o.center.y, o.half_size.x, o.half_size.y];
            u.occluders[i * 2 + 1] = [c, s, 0.0, 0.0];
        }
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let instances: Vec<LightInstance> = lights
            .iter()
            .take(MAX_LIGHTS_2D)
            .map(|l| {
                let c: Vec3 = l.color * l.intensity;
                LightInstance {
                    pos_radius: [l.position.x, l.position.y, l.radius.max(1e-3), l.falloff],
                    color_source: c.extend(l.source_radius.max(0.0)).to_array(),
                    flags: [l.cast_shadows as u32 as f32, 0.0, 0.0, 0.0],
                }
            })
            .collect();
        if !instances.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
        }
        self.count = instances.len() as u32;
        (self.count, occ as u32)
    }

    /// Clears `target` to `ambient` and adds the lights.
    pub fn encode(&self, encoder: &mut crate::Encoder, target: &wgpu::TextureView, ambient: Vec3) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lights 2d"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: ambient.x as f64,
                        g: ambient.y as f64,
                        b: ambient.z as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}
