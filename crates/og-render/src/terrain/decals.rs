//! A terrain's decal-only group: one texture slot, shared records and clusters.
pub(crate) struct TerrainDecals {
    pub layout: wgpu::BindGroupLayout,
    uniforms: [wgpu::Buffer; 2],
    groups: Option<[wgpu::BindGroup; 2]>,
}

impl TerrainDecals {
    pub fn new(device: &wgpu::Device) -> Self {
        let records = crate::fx::decal_layout_entries();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain decals"),
            entries: &[
                crate::uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                crate::tex2d(2),
                crate::sampler_entry(3, wgpu::SamplerBindingType::Filtering),
                records[0],
                records[1],
            ],
        });
        Self {
            layout,
            uniforms: std::array::from_fn(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("terrain decal info"),
                    size: 16,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            }),
            groups: None,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, info: [f32; 4]) {
        queue.write_buffer(&self.uniforms[0], 0, bytemuck::bytes_of(&info));
        let mut unclustered = info;
        unclustered[0] = -info[0].abs();
        queue.write_buffer(&self.uniforms[1], 0, bytemuck::bytes_of(&unclustered));
    }

    pub fn bind(
        &mut self,
        device: &wgpu::Device,
        atlas: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        records: &crate::fx::DecalBuffers,
    ) {
        self.groups = Some(std::array::from_fn(|i| {
            let mut entries = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniforms[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(atlas),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ];
            entries.extend(records.entries());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("terrain decals"),
                layout: &self.layout,
                entries: &entries,
            })
        }));
    }

    pub fn group(&self, clustered: bool) -> &wgpu::BindGroup {
        &self
            .groups
            .as_ref()
            .expect("effect groups made before drawing")[usize::from(!clustered)]
    }
}
