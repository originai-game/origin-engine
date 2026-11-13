//! The depth pyramid occlusion tests against.

use super::*;

/// The pyramid's base: the farthest of a pixel's samples.
pub(crate) fn hiz_source(multisampled: bool) -> String {
    let decl = if multisampled {
        "@group(0) @binding(2) var depth_in: texture_depth_multisampled_2d;\n\
         fn farthest_sample(xy: vec2<u32>) -> f32 {\n\
           var d = 0.0;\n\
           for (var s = 0u; s < textureNumSamples(depth_in); s++) { d = max(d, textureLoad(depth_in, xy, i32(s))); }\n\
           return d;\n\
         }\n"
    } else {
        "@group(0) @binding(2) var depth_in: texture_depth_2d;\n\
         fn farthest_sample(xy: vec2<u32>) -> f32 { return textureLoad(depth_in, xy, 0); }\n"
    };
    format!("{decl}{HIZ_WGSL}")
}

pub(super) fn level_out(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::R32Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

impl GpuCulling {
    /// Whether a frame can test its camera pass against occluders: an
    /// earlier frame left its visibility in `scan`. Until then a frame
    /// passes no `Occluders` pass, and `encode` skips the occluders'
    /// depth, the pyramid and the camera pass's occlusion test.
    pub fn primed(&self) -> bool {
        self.primed
    }

    /// The depth pyramid the last `encode` built and its levels (the
    /// meshlet path tests clusters against it); `None` when it had no
    /// occluders.
    pub fn pyramid(&self) -> Option<(&wgpu::TextureView, u32)> {
        self.pyramid
            .as_ref()
            .filter(|_| self.occlusion)
            .map(|p| (&p.full, p.levels))
    }

    /// The occluders' depth the last `encode` drew and its samples (the
    /// meshlet path's first phase draws its clusters into it before
    /// [`Self::rebuild_pyramid`]); `None` when it had no occluders.
    pub fn occluder_depth(&self) -> Option<(&wgpu::TextureView, u32)> {
        self.pyramid
            .as_ref()
            .filter(|_| self.occlusion)
            .map(|p| (&p.depth, p.size.2))
    }

    /// Builds the pyramid again from the occluders' depth as it is now
    /// (no-op without one this frame).
    pub fn rebuild_pyramid(&self, pass: &mut wgpu::ComputePass<'_>) {
        if let Some(pyramid) = self.pyramid.as_ref().filter(|_| self.occlusion) {
            self.build_pyramid(pass, pyramid);
        }
    }

    /// The pyramid's levels from the occluders' depth: the base (the
    /// farthest sample per texel), then each level from the one below.
    pub(super) fn build_pyramid(&self, pass: &mut wgpu::ComputePass<'_>, pyramid: &Pyramid) {
        let (width, height, samples) = pyramid.size;
        pass.set_pipeline(
            self.hiz_base[(samples > 1) as usize]
                .as_ref()
                .expect("made with the pyramid"),
        );
        pass.set_bind_group(0, &pyramid.base_group, &[]);
        pass.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
        pass.set_pipeline(&self.hiz_reduce);
        for (l, group) in pyramid.reduce_groups.iter().enumerate() {
            let (w, h) = pyramid.level_sizes[l + 1];
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
    }

    pub(super) fn ensure_pyramid(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        samples: u32,
    ) {
        if self
            .pyramid
            .as_ref()
            .is_some_and(|p| p.size == (width, height, samples))
        {
            return;
        }
        let multisampled = samples > 1;
        let depth = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("occluder depth"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let levels = crate::mips::mip_count(width, height);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth pyramid"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let level_views = crate::mips::level_views(&texture, "depth pyramid level");
        let base_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth pyramid base"),
            layout: &self.hiz_base_layout[multisampled as usize],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&level_views[0]),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&depth),
                },
            ],
        });
        let reduce_groups = (1..levels as usize)
            .map(|l| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("depth pyramid"),
                    layout: &self.hiz_reduce_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&level_views[l - 1]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&level_views[l]),
                        },
                    ],
                })
            })
            .collect();
        if self.hiz_base[multisampled as usize].is_none() {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("depth pyramid base"),
                source: wgpu::ShaderSource::Wgsl(hiz_source(multisampled).into()),
            });
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("depth pyramid base"),
                bind_group_layouts: &[Some(&self.hiz_base_layout[multisampled as usize])],
                immediate_size: 0,
            });
            self.hiz_base[multisampled as usize] = Some(device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: Some("depth pyramid base"),
                    layout: Some(&layout),
                    module: &module,
                    entry_point: Some("base"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(device).as_ref(),
                },
            ));
        }
        self.pyramid = Some(Pyramid {
            size: (width, height, samples),
            depth,
            full: texture.create_view(&Default::default()),
            levels,
            base_group,
            reduce_groups,
            level_sizes: (0..levels)
                .map(|l| ((width >> l).max(1), (height >> l).max(1)))
                .collect(),
        });
        self.groups = None;
    }
}
