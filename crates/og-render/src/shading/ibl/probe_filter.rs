//! A reflection probe's filtering.

use super::*;

/// A reflection probe's filtering (`probes.rs`): its capture copied into
/// its radiance cube's level 0 (`fs_probe_copy`), the radiance's mips
/// downsampled, and each prefiltered level GGX-filtered from them, every
/// pass's parameters written once.
pub(crate) struct ProbeFilter {
    _params: wgpu::Buffer,
    draws: Vec<(wgpu::TextureView, usize, wgpu::BindGroup)>,
}

impl ProbeFilter {
    /// Its passes in order: six copies of the capture, six per lower
    /// radiance level, six of irradiance when it has them, then six per
    /// prefiltered level.
    pub(crate) fn len(&self) -> usize {
        self.draws.len()
    }
}

impl IblGenerator {
    /// The filtering of a probe of face size `size` whose six faces are
    /// drawn into `capture` (one level), into `radiance` (a full chain),
    /// `irradiance` (the live sky's: sky_capture.rs) and `prefiltered`
    /// ([`PREFILTER_MIPS`] levels).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn probe_filter(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capture: &wgpu::Texture,
        radiance: &wgpu::Texture,
        irradiance: Option<&wgpu::Texture>,
        prefiltered: &wgpu::Texture,
        size: u32,
    ) -> ProbeFilter {
        const DOWN: usize = 1;
        const IRRADIANCE: usize = 2;
        const PREFILTER: usize = 3;
        const COPY: usize = 7;
        let mips = radiance.mip_level_count();
        let count = 6 * (mips as usize + irradiance.is_some() as usize + PREFILTER_MIPS as usize);
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("probe filter params"),
            size: SLOT * count as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = |source: &wgpu::TextureView, slot: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("probe filter"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &params,
                            offset: slot as u64 * SLOT,
                            size: wgpu::BufferSize::new(std::mem::size_of::<Params>() as u64),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.views.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&self.flat),
                    },
                ],
            })
        };
        let mut bytes = vec![0u8; SLOT as usize * count];
        let mut draws = Vec::with_capacity(count);
        let mut push = |target, pipeline, source: &wgpu::TextureView, p: Params| {
            let at = draws.len() * SLOT as usize;
            bytes[at..at + std::mem::size_of::<Params>()].copy_from_slice(bytemuck::bytes_of(&p));
            let g = group(source, draws.len());
            draws.push((target, pipeline, g));
        };
        let base = Params {
            face: 0,
            roughness: 0.0,
            sample_count: 0,
            source_size: size as f32,
            sun_dir: [0.0; 4],
            sun_color: [0.0; 4],
            look: [[0.0; 4]; 5],
            env: [[0.0; 4]; 8],
            scatter: [[0.0; 4]; crate::sky_physical::SCATTER_VEC4S],
        };
        let captured = cube_view(capture, "probe capture");
        for face in 0..6 {
            push(
                face_view(radiance, face, 0),
                COPY,
                &captured,
                Params { face, ..base },
            );
        }
        for mip in 1..mips {
            let above = cube_view_mips(radiance, "probe above", mip - 1, 1);
            for face in 0..6 {
                push(
                    face_view(radiance, face, mip),
                    DOWN,
                    &above,
                    Params { face, ..base },
                );
            }
        }
        let full = cube_view(radiance, "probe radiance");
        if let Some(irradiance) = irradiance {
            for face in 0..6 {
                push(
                    face_view(irradiance, face, 0),
                    IRRADIANCE,
                    &full,
                    Params { face, ..base },
                );
            }
        }
        for mip in 0..PREFILTER_MIPS {
            let roughness = mip as f32 / (PREFILTER_MIPS - 1) as f32;
            for face in 0..6 {
                push(
                    face_view(prefiltered, face, mip),
                    PREFILTER,
                    &full,
                    Params {
                        face,
                        roughness,
                        sample_count: 64,
                        ..base
                    },
                );
            }
        }
        queue.write_buffer(&params, 0, &bytes);
        ProbeFilter {
            _params: params,
            draws,
        }
    }

    /// Records `filter`'s passes (after the capture's).
    pub(crate) fn record_probe_filter(&self, encoder: &mut crate::Encoder, filter: &ProbeFilter) {
        self.record_filter_range(encoder, filter, 0..filter.draws.len());
    }

    /// Records `filter`'s passes in `range` (the live sky spreads them over
    /// frames: sky_capture.rs).
    pub(crate) fn record_filter_range(
        &self,
        encoder: &mut crate::Encoder,
        filter: &ProbeFilter,
        range: std::ops::Range<usize>,
    ) {
        for (target, pipeline, group) in &filter.draws[range] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("probe filter"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines[*pipeline]);
            pass.set_bind_group(0, group, &[0]);
            pass.draw(0..3, 0..1);
        }
    }
}
