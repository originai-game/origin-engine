//! Texture checks and uploads.

use super::*;

impl Renderer {
    /// Checks a texture before it is uploaded: its data, its format on this
    /// adapter and its size against the device's limit.
    pub(crate) fn check_texture(
        &self,
        index: usize,
        source: &TextureSource,
    ) -> Result<(), RenderError> {
        match source {
            TextureSource::Rgba(texture) => {
                let expected = texture.width as usize * texture.height as usize * 4;
                if texture.width == 0 || texture.height == 0 || texture.rgba.len() != expected {
                    return Err(RenderError::InvalidData(format!(
                        "texture {index}: {}x{} with {} bytes (expected {expected})",
                        texture.width,
                        texture.height,
                        texture.rgba.len()
                    )));
                }
            }
            TextureSource::Encoded(upload) => {
                let desc = &upload.desc;
                desc.validate(&upload.levels)
                    .map_err(|e| RenderError::InvalidData(format!("texture {index}: {e}")))?;
                if !self.info.texture_formats.supported.contains(&desc.format) {
                    return Err(RenderError::Unsupported(format!(
                        "texture {index}: {:?} on {} {:?} (supported: {:?}; transcode to RenderInfo::texture_formats)",
                        desc.format,
                        self.info.adapter,
                        self.info.backend,
                        self.info.texture_formats.supported
                    )));
                }
            }
            TextureSource::Target(_) => {}
        }
        let (width, height) = source.size();
        let max = self.device.limits().max_texture_dimension_2d;
        if width > max || height > max {
            return Err(RenderError::Unsupported(format!(
                "texture {index}: {width}x{height} exceeds this device's {max}x{max} (resize it, or cook a smaller level 0)"
            )));
        }
        Ok(())
    }

    /// Uploads a texture [`Renderer::check_texture`] accepted.
    /// Texture slot `index` on the GPU: its source uploaded, or for a
    /// removed slot the shared fallback (`default_white`, allocated once
    /// for the renderer): the slot's own texture is dropped and freed, and
    /// a stale material or a draw in flight still has something to sample.
    pub(crate) fn texture_for(
        &mut self,
        scene: &crate::RenderScene,
        index: usize,
        encoder: &mut crate::Encoder,
    ) -> Result<GpuTexture, RenderError> {
        if scene.texture_live(index) {
            return Ok(self.upload_source(scene.texture_source(index)?, encoder));
        }
        Ok(GpuTexture {
            view: self.default_white.clone(),
            format: TextureFormat::Rgba8,
            bytes: 0,
            target: false,
            alpha_bounds: FULL_UV,
        })
    }

    /// Fork s63: a deferred texture's 1x1 stand-in until a later frame
    /// uploads it: mid grey and transparent for colour, a flat normal for a
    /// two-channel map, white occlusion and roughness for another linear one.
    pub(crate) fn stand_in(
        &mut self,
        source: &TextureSource,
        encoder: &mut crate::Encoder,
    ) -> GpuTexture {
        let (srgb, two_channel) = match source {
            TextureSource::Rgba(t) => (t.color_space == ColorSpace::Srgb, false),
            TextureSource::Encoded(u) => (
                u.desc.color_space == ColorSpace::Srgb,
                u.desc.format.two_channel(),
            ),
            TextureSource::Target(_) => (false, false),
        };
        let rgba = if srgb {
            vec![128, 128, 128, 0]
        } else if two_channel {
            vec![128, 128, 255, 255]
        } else {
            vec![255, 255, 0, 255]
        };
        let data = TextureData {
            width: 1,
            height: 1,
            rgba,
            color_space: if srgb {
                ColorSpace::Srgb
            } else {
                ColorSpace::Linear
            },
            mips: Mips::Generate,
        };
        GpuTexture {
            view: self.upload_texture(&data, encoder),
            format: TextureFormat::Rgba8,
            bytes: 4,
            target: false,
            alpha_bounds: FULL_UV,
        }
    }

    pub(crate) fn upload_source(
        &mut self,
        source: &TextureSource,
        encoder: &mut crate::Encoder,
    ) -> GpuTexture {
        match source {
            TextureSource::Rgba(texture) => GpuTexture {
                view: self.upload_texture(texture, encoder),
                format: TextureFormat::Rgba8,
                bytes: rgba_texture_bytes(texture),
                target: false,
                alpha_bounds: rgba_alpha_bounds(texture),
            },
            TextureSource::Encoded(upload) => self.upload_encoded(upload),
            TextureSource::Target(t) => GpuTexture {
                view: targets::create(&self.device, *t).create_view(&Default::default()),
                format: TextureFormat::Rgba8,
                bytes: t.bytes(),
                target: true,
                alpha_bounds: FULL_UV,
            },
        }
    }

    fn upload_texture(
        &mut self,
        texture: &TextureData,
        encoder: &mut crate::Encoder,
    ) -> wgpu::TextureView {
        let format = match texture.color_space {
            ColorSpace::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            ColorSpace::Linear => wgpu::TextureFormat::Rgba8Unorm,
        };
        let mip_level_count = match texture.mips {
            Mips::Generate => mips::mip_count(texture.width, texture.height),
            Mips::None => 1,
        };
        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        if mip_level_count > 1 {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
            texture_binding_view_dimension: None,
        });
        crate::traffic::write_texture(
            &self.queue,
            wgpu::TexelCopyTextureInfo {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: Some(texture.height),
            },
            size,
        );
        if mip_level_count > 1 {
            self.mips.generate(&self.device, encoder, &tex);
        }
        tex.create_view(&Default::default())
    }

    /// A texture whose levels arrive encoded (no mip generation).
    fn upload_encoded(&self, upload: &TextureUpload) -> GpuTexture {
        use crate::texture::TextureFormatGpu;
        let desc = &upload.desc;
        let format = desc.format.wgpu(desc.color_space).expect("checked");
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("texture (encoded)"),
                size: wgpu::Extent3d {
                    width: desc.width,
                    height: desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: desc.mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
                texture_binding_view_dimension: None,
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &upload.levels.concat(),
        );
        GpuTexture {
            view: texture.create_view(&Default::default()),
            format: desc.format,
            bytes: upload.levels.iter().map(|l| l.len() as u64).sum(),
            target: false,
            alpha_bounds: FULL_UV,
        }
    }
}

const FULL_UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Computes the tight non-transparent UV box `[u0, v0, u1, v1]` of an RGBA8
/// texture, padded by one texel on each side so bilinear filtering outside the
/// box is guaranteed to sample zero alpha.
fn rgba_alpha_bounds(texture: &TextureData) -> [f32; 4] {
    let (w, h) = (texture.width, texture.height);
    if w < 4 || h < 4 || texture.rgba.len() < (w as usize) * (h as usize) * 4 {
        return FULL_UV;
    }
    let (mut min_x, mut min_y) = (w, h);
    let (mut max_x, mut max_y) = (0u32, 0u32);
    for y in 0..h {
        let row = (y as usize) * (w as usize) * 4;
        for x in 0..w {
            if texture.rgba[row + (x as usize) * 4 + 3] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x > max_x {
        return FULL_UV;
    }
    let u0 = min_x.saturating_sub(1) as f32 / w as f32;
    let v0 = min_y.saturating_sub(1) as f32 / h as f32;
    let u1 = (max_x + 2).min(w) as f32 / w as f32;
    let v1 = (max_y + 2).min(h) as f32 / h as f32;
    if (u1 - u0) * (v1 - v0) >= 0.98 {
        return FULL_UV;
    }
    [u0, v0, u1, v1]
}
