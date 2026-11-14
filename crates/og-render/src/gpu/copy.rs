//! Copies of GPU textures and buffers, made and copied back on the GPU.

use super::texture::texture_bytes;

/// A copy of GPU textures (every level and layer) and buffers.
pub(crate) struct GpuCopy {
    textures: Vec<wgpu::Texture>,
    buffers: Vec<wgpu::Buffer>,
}

fn same_shape(a: &wgpu::Texture, b: &wgpu::Texture) -> bool {
    a.size() == b.size()
        && a.mip_level_count() == b.mip_level_count()
        && a.format() == b.format()
        && a.dimension() == b.dimension()
        && a.sample_count() == b.sample_count()
}

fn copy_texture(encoder: &mut wgpu::CommandEncoder, from: &wgpu::Texture, to: &wgpu::Texture) {
    for mip in 0..from.mip_level_count() {
        let size = from.size().mip_level_size(mip, from.dimension());
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: from,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: to,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
    }
}

impl GpuCopy {
    /// Copies `textures` and `buffers` (each made with `COPY_SRC` and
    /// `COPY_DST`) into new ones, recorded into `encoder`.
    pub(crate) fn of(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        textures: &[&wgpu::Texture],
        buffers: &[&wgpu::Buffer],
    ) -> GpuCopy {
        let textures = textures
            .iter()
            .map(|t| {
                let copy = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("presentation checkpoint"),
                    size: t.size(),
                    mip_level_count: t.mip_level_count(),
                    sample_count: t.sample_count(),
                    dimension: t.dimension(),
                    format: t.format(),
                    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                copy_texture(encoder, t, &copy);
                copy
            })
            .collect();
        let buffers = buffers
            .iter()
            .map(|b| {
                let copy = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("presentation checkpoint"),
                    size: b.size(),
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                encoder.copy_buffer_to_buffer(b, 0, &copy, 0, b.size());
                copy
            })
            .collect();
        GpuCopy { textures, buffers }
    }

    /// Copies back into `textures` and `buffers` when they are the ones
    /// copied (as many, of the same shapes); false, copying nothing,
    /// otherwise.
    pub(crate) fn back(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        textures: &[&wgpu::Texture],
        buffers: &[&wgpu::Buffer],
    ) -> bool {
        let fits = textures.len() == self.textures.len()
            && buffers.len() == self.buffers.len()
            && self
                .textures
                .iter()
                .zip(textures)
                .all(|(a, b)| same_shape(a, b))
            && self
                .buffers
                .iter()
                .zip(buffers)
                .all(|(a, b)| a.size() == b.size());
        if !fits {
            return false;
        }
        for (from, to) in self.textures.iter().zip(textures) {
            copy_texture(encoder, from, to);
        }
        for (from, to) in self.buffers.iter().zip(buffers) {
            encoder.copy_buffer_to_buffer(from, 0, to, 0, from.size());
        }
        true
    }

    pub(crate) fn bytes(&self) -> u64 {
        self.textures.iter().map(texture_bytes).sum::<u64>()
            + self.buffers.iter().map(wgpu::Buffer::size).sum::<u64>()
    }
}
