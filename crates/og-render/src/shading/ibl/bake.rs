//! An uncooked equirectangular image prefiltered on the GPU.

use super::*;

impl IblGenerator {
    /// Prefilters an uncooked equirectangular image into a map the IBL
    /// passes read as they read a cooked one: the image resampled into a
    /// radiance cube of `image.size` (2x2 samples per texel, as the cook's
    /// bake), its mips box-filtered, and level `m` of the result
    /// GGX-filtered from them at roughness `m / 4` with the cook's 256
    /// samples (level 0 the radiance itself), every pass on the GPU.
    pub(crate) fn bake_equirect(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &crate::EnvironmentEquirect,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        const DOWN: usize = 1;
        const PREFILTER: usize = 3;
        const EQUIRECT: usize = 8;
        let extent = wgpu::Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        };
        let flat = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("environment equirect"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgb9e5Ufloat,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &flat,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&image.texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * image.width),
                rows_per_image: Some(image.height),
            },
            extent,
        );
        let flat_view = flat.create_view(&Default::default());
        let n = image.size;
        let mips = n.ilog2() + 1;
        let radiance = cube_texture(device, "environment radiance", n, mips);
        let out = cube_texture(device, "environment map", n, PREFILTER_MIPS);
        let count = 6 * (mips as usize + PREFILTER_MIPS as usize);
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("environment bake params"),
            size: SLOT * count as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = |source: &wgpu::TextureView, slot: usize| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("environment bake"),
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
                        resource: wgpu::BindingResource::TextureView(&flat_view),
                    },
                ],
            })
        };
        let base = Params {
            face: 0,
            roughness: 0.0,
            sample_count: 0,
            source_size: n as f32,
            sun_dir: [0.0; 4],
            sun_color: [0.0; 4],
            look: [[0.0; 4]; 5],
            env: [[0.0; 4]; 8],
            scatter: [[0.0; 4]; crate::sky_physical::SCATTER_VEC4S],
        };
        let mut bytes = vec![0u8; SLOT as usize * count];
        let mut draws: Vec<(wgpu::TextureView, usize, wgpu::BindGroup)> = Vec::with_capacity(count);
        let mut push = |target, pipeline, source: &wgpu::TextureView, p: Params| {
            let at = draws.len() * SLOT as usize;
            bytes[at..at + std::mem::size_of::<Params>()].copy_from_slice(bytemuck::bytes_of(&p));
            let g = group(source, draws.len());
            draws.push((target, pipeline, g));
        };
        // The equirect passes sample no cube: any will do.
        for face in 0..6 {
            push(
                face_view(&radiance, face, 0),
                EQUIRECT,
                &self.views.irradiance,
                Params { face, ..base },
            );
        }
        for mip in 1..mips {
            let above = cube_view_mips(&radiance, "environment above", mip - 1, 1);
            for face in 0..6 {
                push(
                    face_view(&radiance, face, mip),
                    DOWN,
                    &above,
                    Params { face, ..base },
                );
            }
        }
        let full = cube_view(&radiance, "environment radiance");
        for mip in 0..PREFILTER_MIPS {
            let roughness = mip as f32 / (PREFILTER_MIPS - 1) as f32;
            for face in 0..6 {
                push(
                    face_view(&out, face, mip),
                    PREFILTER,
                    &full,
                    Params {
                        face,
                        roughness,
                        // Level 0 is the radiance: its one sample is the texel's own direction.
                        sample_count: if mip == 0 { 1 } else { 256 },
                        ..base
                    },
                );
            }
        }
        queue.write_buffer(&params, 0, &bytes);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("environment bake"),
        });
        for (target, pipeline, group) in &draws {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("environment bake"),
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
        queue.submit(Some(encoder.finish()));
        let view = cube_view(&out, "environment map");
        (out, view)
    }
}
