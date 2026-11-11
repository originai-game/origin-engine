//! The renderer's uniform and storage buffers: the frame uniform, the
//! lights' tables, the LTC and caustic tables, the volumetric fog's, one set
//! per planar reflection, and the shadow passes' slots with their group.

use super::*;

pub(super) struct UniformBuffersInputs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub info: &'a RenderInfo,
    pub shadow_bind_layout: &'a wgpu::BindGroupLayout,
}

pub(super) struct UniformBuffersOutputs {
    pub frame_buffer: wgpu::Buffer,
    pub light_buffers: [wgpu::Buffer; 3],
    pub ltc: wgpu::Buffer,
    pub caustic_grid: wgpu::Buffer,
    pub volumetric: volumetric::Volumetric,
    pub reflection_light_buffers: Vec<[wgpu::Buffer; 3]>,
    pub reflection_buffer: Vec<wgpu::Buffer>,
    pub reflection_clusters: Vec<clusters::ClusterBuilder>,
    pub reflection_fx_buffer: Vec<wgpu::Buffer>,
    pub shadow_buffer: wgpu::Buffer,
    pub shadow_bind_group: wgpu::BindGroup,
}

impl Renderer {
    /// Makes the renderer's uniform buffers.
    #[allow(clippy::too_many_lines)]
    pub(super) fn uniform_buffers(
        UniformBuffersInputs {
            device,
            queue,
            info,
            shadow_bind_layout,
        }: UniformBuffersInputs<'_>,
    ) -> UniformBuffersOutputs {
        // Uniforms.
        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame"),
            size: std::mem::size_of::<frame::FrameUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffers = clusters::buffers(device);
        let ltc = area::ltc_buffer(device, queue);
        let caustic_grid = water::caustic_buffer(device);
        let volumetric =
            volumetric::Volumetric::new(device, info.volumetric, info.without.features());
        // One set per planar reflection the renderer draws (none without
        // the 3D world).
        let slots = 0..info.planar_reflections;
        let reflection_light_buffers: Vec<_> =
            slots.clone().map(|_| clusters::buffers(device)).collect();
        let reflection_buffer: Vec<_> = slots
            .clone()
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("frame (planar reflection)"),
                    size: std::mem::size_of::<frame::FrameUniform>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        let reflection_clusters: Vec<_> = slots
            .clone()
            .map(|_| clusters::ClusterBuilder::default())
            .collect();
        let reflection_fx_buffer: Vec<_> = slots
            .map(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("fx (planar reflection)"),
                    size: std::mem::size_of::<fx::FxUniform>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();
        let shadow_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow passes"),
            size: SHADOW_SLOT_STRIDE * SHADOW_SLOTS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: shadow_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &shadow_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<ShadowPassUniform>() as u64),
                }),
            }],
        });
        UniformBuffersOutputs {
            frame_buffer,
            light_buffers,
            ltc,
            caustic_grid,
            volumetric,
            reflection_light_buffers,
            reflection_buffer,
            reflection_clusters,
            reflection_fx_buffer,
            shadow_buffer,
            shadow_bind_group,
        }
    }
}
