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
    ) -> GpuTexture {
        if scene.texture_live(index) {
            return self.upload_source(&scene.textures()[index], encoder);
        }
        GpuTexture {
            view: self.default_white.clone(),
            format: TextureFormat::Rgba8,
            bytes: 0,
            target: false,
        }
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
            },
            TextureSource::Encoded(upload) => self.upload_encoded(upload),
            TextureSource::Target(t) => GpuTexture {
                view: targets::create(&self.device, *t).create_view(&Default::default()),
                format: TextureFormat::Rgba8,
                bytes: t.bytes(),
                target: true,
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
        });
        self.queue.write_texture(
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
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &upload.levels.concat(),
        );
        GpuTexture {
            view: texture.create_view(&Default::default()),
            format: desc.format,
            bytes: upload.levels.iter().map(|l| l.len() as u64).sum(),
            target: false,
        }
    }
}
