//! The depth pyramid occlusion tests against, built via multi-level conservative
//! LDS SPD passes (`shaders/hiz.wgsl`).

use super::*;

/// The conservative SPD depth pyramid shader (`hiz.wgsl`).
pub(crate) fn hiz_source() -> String {
    HIZ_WGSL.to_string()
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

fn exact_half((w, h): (u32, u32)) -> bool {
    (w <= 1 || w.is_multiple_of(2)) && (h <= 1 || h.is_multiple_of(2))
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

    /// Builds all pyramid levels via the precomputed conservative SPD batches.
    pub(super) fn build_pyramid(&self, pass: &mut wgpu::ComputePass<'_>, pyramid: &Pyramid) {
        let pipes = self.hiz_pipelines.wait();
        for p in &pyramid.passes {
            let pipe = match p.kind {
                HizKind::Base => &pipes[0],
                HizKind::Reduce => &pipes[1],
                HizKind::Tail => &pipes[2],
            };
            pass.set_pipeline(pipe);
            pass.set_bind_group(0, &p.group, &[]);
            pass.dispatch_workgroups(p.dispatch.0, p.dispatch.1, 1);
        }
    }

    pub(super) fn ensure_pyramid(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        _samples: u32,
    ) {
        // Occluder depth is always 1x non-MSAA (`encode.rs`).
        let samples = 1u32;
        if self
            .pyramid
            .as_ref()
            .is_some_and(|p| p.size == (width, height, samples))
        {
            return;
        }
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
                texture_binding_view_dimension: None,
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
            texture_binding_view_dimension: None,
        });
        let level_views = crate::mips::level_views(&texture, "depth pyramid level");
        let level_sizes: Vec<(u32, u32)> = (0..levels)
            .map(|l| ((width >> l).max(1), (height >> l).max(1)))
            .collect();

        let make_group = |src: &wgpu::TextureView, start: usize, count: usize| {
            let dst = |i: usize| {
                if i < count {
                    &level_views[start + i]
                } else {
                    &self.hiz_dummy
                }
            };
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("depth pyramid spd"),
                layout: &self.hiz_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.hiz_counts[count - 1].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(src),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(dst(0)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(dst(1)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(dst(2)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(dst(3)),
                    },
                ],
            })
        };

        let mut passes = Vec::new();
        // Batch 0: fused base copy (level 0) + up to 3 in-LDS 2x2 reductions
        // while each source level has even (or 1) dimensions so 2x2 is exact.
        let mut base_count = 1usize;
        while base_count < 4
            && base_count < levels as usize
            && exact_half(level_sizes[base_count - 1])
        {
            base_count += 1;
        }
        passes.push(HizPass {
            kind: HizKind::Base,
            group: make_group(&depth, 0, base_count),
            dispatch: (width.div_ceil(16), height.div_ceil(16)),
        });

        let mut next = base_count;
        while next < levels as usize {
            let src_size = level_sizes[next - 1];
            let rem = levels as usize - next;
            if src_size.0 <= 16 && src_size.1 <= 16 {
                let count = rem.min(4);
                passes.push(HizPass {
                    kind: HizKind::Tail,
                    group: make_group(&level_views[next - 1], next, count),
                    dispatch: (1, 1),
                });
                next += count;
            } else {
                let mut count = 1usize;
                while count < 4
                    && next + count < levels as usize
                    && exact_half(level_sizes[next + count - 1])
                {
                    count += 1;
                }
                let (w0, h0) = level_sizes[next];
                passes.push(HizPass {
                    kind: HizKind::Reduce,
                    group: make_group(&level_views[next - 1], next, count),
                    dispatch: (w0.div_ceil(8), h0.div_ceil(8)),
                });
                next += count;
            }
        }

        self.pyramid = Some(Pyramid {
            size: (width, height, samples),
            depth,
            full: texture.create_view(&Default::default()),
            levels,
            passes,
        });
        self.groups = None;
    }
}
