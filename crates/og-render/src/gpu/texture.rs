use crate::scene::ColorSpace;
pub use og_render_data::texture::*;
use wgpu::util::DeviceExt;

/// A [`TextureFormat`]'s form on the device.
pub(crate) trait TextureFormatGpu {
    /// The wgpu format for `color_space` (`None`: no sRGB variant exists).
    fn wgpu(self, color_space: ColorSpace) -> Option<wgpu::TextureFormat>;
    /// The device feature the format needs.
    fn feature(self) -> wgpu::Features;
}

impl TextureFormatGpu for TextureFormat {
    fn wgpu(self, color_space: ColorSpace) -> Option<wgpu::TextureFormat> {
        use wgpu::{AstcBlock, AstcChannel, TextureFormat as F};
        let srgb = color_space == ColorSpace::Srgb;
        Some(match (self, srgb) {
            (TextureFormat::Rgba8, true) => F::Rgba8UnormSrgb,
            (TextureFormat::Rgba8, false) => F::Rgba8Unorm,
            (TextureFormat::Bc7, true) => F::Bc7RgbaUnormSrgb,
            (TextureFormat::Bc7, false) => F::Bc7RgbaUnorm,
            (TextureFormat::Astc4x4, s) => F::Astc {
                block: AstcBlock::B4x4,
                channel: if s {
                    AstcChannel::UnormSrgb
                } else {
                    AstcChannel::Unorm
                },
            },
            (TextureFormat::Etc2Rgba8, true) => F::Etc2Rgba8UnormSrgb,
            (TextureFormat::Etc2Rgba8, false) => F::Etc2Rgba8Unorm,
            (TextureFormat::Bc5, false) => F::Bc5RgUnorm,
            (TextureFormat::EacRg11, false) => F::EacRg11Unorm,
            (TextureFormat::Bc5 | TextureFormat::EacRg11, true) => return None,
        })
    }

    fn feature(self) -> wgpu::Features {
        match self {
            TextureFormat::Rgba8 => wgpu::Features::empty(),
            TextureFormat::Bc7 | TextureFormat::Bc5 => wgpu::Features::TEXTURE_COMPRESSION_BC,
            TextureFormat::Astc4x4 => wgpu::Features::TEXTURE_COMPRESSION_ASTC,
            TextureFormat::Etc2Rgba8 | TextureFormat::EacRg11 => {
                wgpu::Features::TEXTURE_COMPRESSION_ETC2
            }
        }
    }
}

/// The renderer's choice of [`TextureFormats`] for a device.
pub(crate) trait TextureFormatsChoice {
    /// The features to request and the choice for `preference`, given what
    /// the adapter offers.
    fn choose(
        adapter: wgpu::Features,
        preference: TextureCompression,
    ) -> (wgpu::Features, TextureFormats);
}

impl TextureFormatsChoice for TextureFormats {
    fn choose(
        adapter: wgpu::Features,
        preference: TextureCompression,
    ) -> (wgpu::Features, TextureFormats) {
        let families = [
            wgpu::Features::TEXTURE_COMPRESSION_BC,
            wgpu::Features::TEXTURE_COMPRESSION_ASTC,
            wgpu::Features::TEXTURE_COMPRESSION_ETC2,
        ];
        // Off requests no compression feature at all: RGBA8 only.
        let available = families
            .into_iter()
            .filter(|f| adapter.contains(*f) && preference != TextureCompression::Off)
            .fold(wgpu::Features::empty(), |a, f| a | f);
        let order: &[wgpu::Features] = match preference {
            TextureCompression::Auto => &families,
            TextureCompression::PreferAstc => &[
                wgpu::Features::TEXTURE_COMPRESSION_ASTC,
                wgpu::Features::TEXTURE_COMPRESSION_ETC2,
                wgpu::Features::TEXTURE_COMPRESSION_BC,
            ],
            TextureCompression::PreferEtc2 => &[
                wgpu::Features::TEXTURE_COMPRESSION_ETC2,
                wgpu::Features::TEXTURE_COMPRESSION_ASTC,
                wgpu::Features::TEXTURE_COMPRESSION_BC,
            ],
            TextureCompression::Off => &[],
        };
        let (color, normal) = match order.iter().find(|f| available.contains(**f)) {
            Some(&f) if f == wgpu::Features::TEXTURE_COMPRESSION_BC => {
                (TextureFormat::Bc7, TextureFormat::Bc5)
            }
            Some(&f) if f == wgpu::Features::TEXTURE_COMPRESSION_ASTC => {
                (TextureFormat::Astc4x4, TextureFormat::Astc4x4)
            }
            Some(_) => (TextureFormat::Etc2Rgba8, TextureFormat::EacRg11),
            None => (TextureFormat::Rgba8, TextureFormat::Rgba8),
        };
        let supported = TextureFormat::ALL
            .into_iter()
            .filter(|f| available.contains(f.feature()))
            .collect();
        (
            available,
            TextureFormats {
                color,
                normal,
                data: color,
                supported,
            },
        )
    }
}

pub(crate) fn target_texture(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub(crate) fn solid_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba: [u8; 4],
) -> wgpu::TextureView {
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("default texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        )
        .create_view(&Default::default())
}

/// What `texture` allocates: every level of every layer, times its samples.
pub(crate) fn texture_bytes(texture: &wgpu::Texture) -> u64 {
    let size = texture.size();
    let format = texture.format();
    let layers = match texture.dimension() {
        wgpu::TextureDimension::D3 => 1,
        _ => size.depth_or_array_layers.max(1),
    };
    let mut bytes = 0;
    for level in 0..texture.mip_level_count() {
        let w = (size.width >> level).max(1);
        let h = (size.height >> level).max(1);
        let d = match texture.dimension() {
            wgpu::TextureDimension::D3 => (size.depth_or_array_layers >> level).max(1),
            _ => 1,
        };
        let (bw, bh) = format.block_dimensions();
        let block = format.block_copy_size(None).unwrap_or(match format {
            wgpu::TextureFormat::Depth16Unorm => 2,
            wgpu::TextureFormat::Stencil8 => 1,
            wgpu::TextureFormat::Depth32FloatStencil8 => 8,
            _ => 4,
        });
        let footprint =
            u64::from(w.div_ceil(bw)) * u64::from(h.div_ceil(bh)) * u64::from(d) * u64::from(block);
        bytes += footprint;
    }
    bytes * u64::from(layers) * u64::from(texture.sample_count().max(1))
}

/// What a view's texture allocates.
pub(crate) fn view_bytes(view: &wgpu::TextureView) -> u64 {
    texture_bytes(view.texture())
}
