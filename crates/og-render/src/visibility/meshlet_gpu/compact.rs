//! Stable work-list construction after each cluster cull. The scratch
//! slots survive the early/late boundary; lists and counts are rebuilt.

use super::{WORKGROUPS_PER_ROW, buffer, entry, grow, storage, storage_usage};

pub(crate) const WGSL: &str = wgsl!("meshlet_compact.wgsl");
const WG: u32 = 256;

pub(super) struct Compact {
    pub selected: wgpu::Buffer,
    sums: wgpu::Buffer,
    block_sums: wgpu::Buffer,
    params: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    pipelines: [wgpu::ComputePipeline; 4],
    group: Option<(Vec<wgpu::Buffer>, [u32; 4], wgpu::BindGroup)>,
}

impl Compact {
    pub fn new(device: &wgpu::Device) -> Self {
        let c = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet compact"),
            entries: &[
                super::uniform(0, c),
                storage(1, true, c),
                storage(2, true, c),
                storage(3, true, c),
                storage(4, false, c),
                storage(5, false, c),
                storage(6, false, c),
                storage(7, false, c),
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet compact"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet compact"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipelines = ["reduce", "scan_blocks", "prefix", "scatter"].map(|name| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        });
        Self {
            selected: buffer(device, "meshlet selected", 0, storage_usage()),
            sums: buffer(device, "meshlet sums", 0, storage_usage()),
            block_sums: buffer(device, "meshlet block sums", 0, storage_usage()),
            params: buffer(
                device,
                "meshlet compact params",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            layout,
            pipelines,
            group: None,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: [u32; 4],
        buffers: [&wgpu::Buffer; 4], // batches, meshes, items, draws
    ) {
        let slots = params[0];
        grow(
            device,
            &mut self.selected,
            "meshlet selected",
            u64::from(slots) * 8,
            storage_usage(),
        );
        grow(
            device,
            &mut self.sums,
            "meshlet sums",
            u64::from(slots) * 8,
            storage_usage(),
        );
        grow(
            device,
            &mut self.block_sums,
            "meshlet block sums",
            u64::from(slots.div_ceil(WG) + 1) * 8,
            storage_usage(),
        );
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        let bound = vec![
            buffers[0].clone(),
            buffers[1].clone(),
            self.selected.clone(),
            self.sums.clone(),
            buffers[2].clone(),
            buffers[3].clone(),
            self.block_sums.clone(),
        ];
        if self
            .group
            .as_ref()
            .is_some_and(|(b, p, _)| *b == bound && *p == params)
        {
            return;
        }
        let mut entries = vec![entry(0, &self.params)];
        for (i, buf) in bound.iter().enumerate() {
            // Power-of-two buffer growth may exceed the binding limit;
            // bind only the portion this frame's scan and lists use.
            let bytes = match i {
                2 | 3 => u64::from(slots) * 8,
                4 => u64::from(params[1] + params[2] * 2).max(u64::from(slots)) * 8,
                _ => buf.size(),
            };
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32 + 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(bytes),
                }),
            });
        }
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meshlet compact"),
            layout: &self.layout,
            entries: &entries,
        });
        self.group = Some((bound, params, group));
    }

    pub fn encode(&self, pass: &mut wgpu::ComputePass<'_>, slots: u32) {
        let blocks = slots.div_ceil(WG);
        pass.set_bind_group(0, &self.group.as_ref().expect("prepared").2, &[]);
        for (i, pipeline) in self.pipelines.iter().enumerate() {
            pass.set_pipeline(pipeline);
            let n = if i == 1 { 1 } else { blocks };
            pass.dispatch_workgroups(n.min(WORKGROUPS_PER_ROW), n.div_ceil(WORKGROUPS_PER_ROW), 1);
        }
    }
}
