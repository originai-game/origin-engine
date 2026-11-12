//! The foam's passes on the GPU: their pipelines and each advance's bind
//! group ([`Foam`]; the schedule is [`foam`]).

use super::*;

impl Foam {
    pub(super) fn new(
        device: &wgpu::Device,
        fields: &wgpu::Texture,
        cascades: u32,
        uniform_entry: &wgpu::BindGroupLayoutEntry,
        pipeline: &dyn Fn(
            &'static str,
            &wgpu::BindGroupLayout,
            &wgpu::ShaderModule,
            &'static str,
        ) -> wgpu::ComputePipeline,
        module: &dyn Fn(&'static str, &str) -> wgpu::ShaderModule,
    ) -> Foam {
        let first = foam_layer(cascades);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ocean foam"),
            size: ring_stride::<foam::FoamGpu>() * foam::MAX_STEPS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let history = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ocean foam history"),
            size: wgpu::Extent3d {
                width: foam::RES,
                height: foam::RES,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = |label, dimension, base_mip_level, mips, base_array_layer, layers| {
            fields.create_view(&wgpu::TextureViewDescriptor {
                label: Some(label),
                dimension: Some(dimension),
                base_mip_level,
                mip_level_count: mips,
                base_array_layer,
                array_layer_count: Some(layers),
                ..Default::default()
            })
        };
        // The update samples the cascades' layers (every mip) while it writes
        // the foam's (mip 0): disjoint parts of the one array.
        let waves = view(
            "ocean foam waves",
            wgpu::TextureViewDimension::D2Array,
            0,
            None,
            0,
            first,
        );
        let out = view(
            "ocean foam out",
            wgpu::TextureViewDimension::D2Array,
            0,
            Some(1),
            first,
            2,
        );
        let detail_out = view(
            "ocean foam detail",
            wgpu::TextureViewDimension::D2,
            0,
            Some(1),
            first + 2,
            1,
        );
        let sampled = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };
        let update_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean foam"),
            entries: &[
                *uniform_entry,
                sampled(1),
                sampled(2),
                storage_texture(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The seabed (shore.wgsl), its texels read with textureLoad.
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let detail_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ocean foam detail"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
        // Tiled (the levels are toroidal), trilinear.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ocean foam"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let m = module("ocean foam", &foam_source());
        let update = pipeline("ocean foam", &update_layout, &m, "update");
        let detail = pipeline("ocean foam detail", &detail_layout, &m, "detail");
        let history_view = history.create_view(&wgpu::TextureViewDescriptor {
            label: Some("ocean foam history"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let detail_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean foam detail"),
            layout: &detail_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&detail_out),
            }],
        });
        Foam {
            uniform,
            history,
            update,
            update_layout,
            update_views: [waves, history_view, out],
            sampler,
            detail,
            detail_group,
            detail_made: false,
            last: None,
        }
    }
}

impl Foam {
    /// The update's bind group with the seabed `shore` (one texel of none).
    pub(super) fn group(
        &self,
        device: &wgpu::Device,
        shore: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        let [waves, history, out] = &self.update_views;
        let view = wgpu::BindingResource::TextureView;
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ocean foam"),
            layout: &self.update_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ring_binding::<foam::FoamGpu>(&self.uniform),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: view(waves),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: view(history),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: view(out),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: view(shore),
                },
            ],
        })
    }
}
