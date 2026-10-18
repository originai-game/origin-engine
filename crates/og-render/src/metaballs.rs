//! 2D fluid drawn as metaballs (the E5 round; `shaders/metaballs.wgsl`).
//! [`Metaballs`]: the particles of a fluid (og-physics' `og.Fluid2D`) are
//! added into one channel of a field target as soft kernels before the
//! main pass, then composited in the sprite order at the fluid's layer by a
//! threshold over that channel. Up to [`MAX_FLUIDS`] fluids a frame; more
//! are not drawn.

use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// The most fluids drawn in a frame (one field channel each).
pub const MAX_FLUIDS: usize = 4;

/// A 2D fluid drawn as metaballs. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Metaballs {
    /// Particle positions in world units.
    pub particles: Arc<[Vec2]>,
    /// The radius of each particle's kernel.
    pub radius: f32,
    /// Where the surface is: the field value (each kernel peaks at 1).
    pub threshold: f32,
    /// Linear rgba of the body.
    pub color: Vec4,
    /// Linear colour of the bright rim inside the edge, and its width in
    /// field units (0: none).
    pub rim: Vec3,
    pub rim_width: f32,
    /// The sprite layer it is drawn in.
    pub layer: i32,
    /// World z of the particles.
    pub z: f32,
}

impl Default for Metaballs {
    fn default() -> Self {
        Metaballs {
            particles: Arc::from(Vec::new()),
            radius: 0.3,
            threshold: 0.45,
            color: Vec4::new(0.12, 0.45, 0.85, 0.9),
            rim: Vec3::new(0.7, 0.92, 1.0),
            rim_width: 0.35,
            layer: 0,
            z: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Blob {
    position_radius: [f32; 4],
    channel: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FluidGpu {
    color: [f32; 4],
    rim: [f32; 4],
    channel: [f32; 4],
    params: [f32; 4],
}

const SLOT: u64 = 256;

const BLOB_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Blob>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
};

pub(crate) struct MetaballRenderer {
    field_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    blobs: wgpu::Buffer,
    params: wgpu::Buffer,
    count: u32,
}

impl MetaballRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        samples: u32,
    ) -> MetaballRenderer {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("metaballs"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{frame_wgsl}{}", include_str!("../shaders/metaballs.wgsl")).into(),
            ),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("metaballs"),
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
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<FluidGpu>() as u64
                        ),
                    },
                    count: None,
                },
            ],
        });
        let field_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("metaball field"),
            bind_group_layouts: &[Some(frame_layout)],
            immediate_size: 0,
        });
        let field_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("metaball field"),
            layout: Some(&field_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_field"),
                compilation_options: Default::default(),
                buffers: &[Some(BLOB_LAYOUT)],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_field"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: crate::HDR_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("metaball composite"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        let composite_pipeline = crate::hdr_pipeline(
            device,
            crate::HdrPipeline {
                label: "metaball composite",
                layout: &composite_layout,
                module: &module,
                vs: "vs_composite",
                fs: "fs_composite",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::Always,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                samples,
            },
        );
        let blobs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metaballs"),
            size: 4096,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metaball params"),
            size: SLOT * MAX_FLUIDS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        MetaballRenderer {
            field_pipeline,
            composite_pipeline,
            layout,
            blobs,
            params,
            count: 0,
        }
    }

    /// Writes the frame's kernels and fluid parameters. Returns the number
    /// of fluids drawn.
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fluids: &[Metaballs],
        screen: (u32, u32),
    ) -> usize {
        let n = fluids.len().min(MAX_FLUIDS);
        let mut blobs = Vec::new();
        for (i, f) in fluids[..n].iter().enumerate() {
            let mut channel = [0.0; 4];
            channel[i] = 1.0;
            let radius = f.radius.max(1e-4);
            blobs.extend(
                f.particles
                    .iter()
                    .filter(|p| p.is_finite())
                    .map(|p| Blob {
                        position_radius: [p.x, p.y, f.z, radius],
                        channel,
                    }),
            );
            let gpu = FluidGpu {
                color: f.color.to_array(),
                rim: f.rim.extend(f.rim_width.max(0.0)).to_array(),
                channel,
                params: [
                    f.threshold.max(1e-3),
                    1.0 / screen.0.max(1) as f32,
                    1.0 / screen.1.max(1) as f32,
                    0.0,
                ],
            };
            queue.write_buffer(&self.params, i as u64 * SLOT, bytemuck::bytes_of(&gpu));
        }
        let bytes: &[u8] = bytemuck::cast_slice(&blobs);
        if bytes.len() as u64 > self.blobs.size() {
            self.blobs = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("metaballs"),
                size: (bytes.len() as u64).next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.blobs, 0, bytes);
        }
        self.count = blobs.len() as u32;
        n
    }

    /// The field pass: every kernel into `field`.
    pub(crate) fn encode_field(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        field: &wgpu::TextureView,
        frame_group: &wgpu::BindGroup,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("metaball field"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: field,
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
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.field_pipeline);
        pass.set_bind_group(0, frame_group, &[]);
        pass.set_vertex_buffer(0, self.blobs.slice(..));
        pass.draw(0..6, 0..self.count);
    }

    /// The bind group the composite reads the field through.
    pub(crate) fn group(&self, device: &wgpu::Device, field: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("metaballs"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(field),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.params,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<FluidGpu>() as u64),
                    }),
                },
            ],
        })
    }

    /// Composites fluid `i` (group 0: the frame).
    pub(crate) fn composite(&self, pass: &mut wgpu::RenderPass, group: &wgpu::BindGroup, i: u8) {
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(1, group, &[(i as u64 * SLOT) as u32]);
        pass.draw(0..3, 0..1);
    }
}

/// Where fluid `i` sits in the sprite order.
pub(crate) fn layers(fluids: &[Metaballs]) -> impl Iterator<Item = (usize, i32)> + '_ {
    fluids
        .iter()
        .take(MAX_FLUIDS)
        .enumerate()
        .map(|(i, f)| (i, f.layer))
}

