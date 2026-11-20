//! A terrain's effect group: the lit passes' effect group (fx.wgsl: the
//! effects uniform, the screen AO, the decals' atlas, sampler, records and
//! clusters), then the trample map its lying snow and sand keep tracks by,
//! read-only storage (the stage's sixteen texture slots are taken), and
//! the map's grid.
use crate::frame::fx::FxUniform;

pub(crate) struct TerrainFx {
    pub layout: wgpu::BindGroupLayout,
    /// The effects uniform of the views without the main camera's decal
    /// clusters or screen AO (planar reflections, render-to-texture views,
    /// reflection probes): the main camera's, unclustered, without AO.
    unclustered: wgpu::Buffer,
    /// `trample_grid` in terrain_lit.wgsl (`Trample::grid`).
    trample: wgpu::Buffer,
    groups: Option<[wgpu::BindGroup; 2]>,
}

/// What a group of the terrain's effect layout binds: the effects uniform
/// and the AO texture (the main camera's, or the unclustered views' and a
/// white texture), and what every group shares.
pub(crate) struct TerrainFxBinds<'a> {
    pub fx: &'a wgpu::Buffer,
    pub ao: &'a wgpu::TextureView,
    pub white: &'a wgpu::TextureView,
    pub atlas: &'a wgpu::TextureView,
    pub sampler: &'a wgpu::Sampler,
    pub records: &'a crate::decals::DecalBuffers,
    pub trample: &'a wgpu::TextureView,
}

impl TerrainFx {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut entries = crate::fx_layout_entries(false);
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 7,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::ReadOnly,
                format: crate::trample::FORMAT,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        });
        entries.push(crate::uniform_entry(8, wgpu::ShaderStages::FRAGMENT));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain effects"),
            entries: &entries,
        });
        let uniform = |label, size| {
            crate::traffic::create_buffer(
                device,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            )
        };
        Self {
            layout,
            unclustered: uniform(
                "terrain effects (unclustered)",
                std::mem::size_of::<FxUniform>() as u64,
            ),
            trample: uniform("terrain trample grid", 16),
            groups: None,
        }
    }

    /// The unclustered views' effects uniform from the main camera's `fx`
    /// (its decal records, walked in order, and no screen AO), and the
    /// trample map's grid (`Trample::grid`).
    pub fn write(&self, queue: &wgpu::Queue, fx: &FxUniform, trample: [f32; 4]) {
        let mut u = *fx;
        u.ao = [0.0; 4];
        crate::fx::unclustered(&mut u);
        crate::traffic::write_buffer(queue, &self.unclustered, 0, bytemuck::bytes_of(&u));
        crate::traffic::write_buffer(queue, &self.trample, 0, bytemuck::bytes_of(&trample));
    }

    /// Makes the two groups: the main camera's (`b.fx` and `b.ao`), and
    /// the unclustered views' (their uniform and `b.white`).
    pub fn bind(&mut self, device: &wgpu::Device, b: TerrainFxBinds<'_>) {
        let group = |fx: &wgpu::Buffer, ao, label| {
            let mut entries = vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: fx.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(ao),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(b.atlas),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(b.sampler),
                },
            ];
            entries.extend(b.records.entries());
            entries.push(wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::TextureView(b.trample),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 8,
                resource: self.trample.as_entire_binding(),
            });
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &self.layout,
                entries: &entries,
            })
        };
        self.groups = Some([
            group(b.fx, b.ao, "terrain effects"),
            group(&self.unclustered, b.white, "terrain effects (unclustered)"),
        ]);
    }

    pub fn group(&self, clustered: bool) -> &wgpu::BindGroup {
        &self
            .groups
            .as_ref()
            .expect("effect groups made before drawing")[usize::from(!clustered)]
    }
}
