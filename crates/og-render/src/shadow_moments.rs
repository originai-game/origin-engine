//! The soft sun's moment pyramid (shadow_moments.wgsl): when the sun's
//! shadows are soft (`DirectionalLight::shadow_softness`, or a weather's
//! cover), each cascade's four depth moments averaged over blocks of
//! 4 x 4 to 128 x 128 texels, in layers of the cascade array after the
//! cascades (lighting.wgsl `pcss` reads them: moment soft shadow mapping,
//! Peters et al., JCGT 2017). The lit pipelines have no texture slot for a
//! map of their own, so the pyramid lives in the array they already bind.

/// Cascades one moments layer holds (three quarters at each level, the
/// fourth holding the next level: shadow_moments.wgsl).
const CASCADES_PER_LAYER: u32 = 3;
/// Depth texels per side one workgroup reduces (shadow_moments.wgsl).
const BLOCK: u32 = 128;

/// The moments layers `cascades` cascades need.
pub(crate) fn layers(cascades: u32) -> u32 {
    cascades.div_ceil(CASCADES_PER_LAYER)
}

/// The deepest level for a map of side `size` (lighting.wgsl `moment_top`).
fn levels(size: u32) -> u32 {
    size.max(1).ilog2().min(7)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    size: u32,
    levels: u32,
    pad: [u32; 2],
}

pub(crate) struct ShadowMoments {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    params: wgpu::Buffer,
}

impl ShadowMoments {
    pub(crate) fn new(device: &wgpu::Device) -> ShadowMoments {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow moments"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::R32Float,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow moments"),
            source: wgpu::ShaderSource::Wgsl(wgsl!("shadow_moments.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow moments"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("shadow moments"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("reduce"),
            compilation_options: Default::default(),
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow moments"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ShadowMoments {
            layout,
            pipeline,
            params,
        }
    }

    /// Reduces cascades `0..cascades` of `map` (side `size`) into its
    /// moments layers from `first` on.
    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        map: &wgpu::Texture,
        size: u32,
        cascades: u32,
        first: u32,
    ) {
        let params = Params {
            size,
            levels: levels(size),
            pad: [0; 2],
        };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        let view = |base: u32, count: u32| {
            map.create_view(&wgpu::TextureViewDescriptor {
                label: Some("shadow moments"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_array_layer: base,
                array_layer_count: Some(count),
                ..Default::default()
            })
        };
        let (depth, moments) = (view(0, cascades), view(first, layers(cascades)));
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow moments"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&moments),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("shadow moments"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        let groups = size.div_ceil(BLOCK);
        pass.dispatch_workgroups(groups, groups, cascades);
    }
}


#[cfg(test)]
mod scratch {
    fn check(src: &str) {
        let m = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&m)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
    }
    #[test]
    fn scratch_parse() {
        check(wgsl!("shadow_moments.wgsl"));
        let tables = crate::clusters::wgsl();
        check(&format!("{}{}{}", crate::FRAME_WGSL, tables, wgsl!("lighting.wgsl")));
    }
}
