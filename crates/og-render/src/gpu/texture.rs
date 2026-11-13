use crate::scene::ColorSpace;
pub use og_render_data::texture::*;

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
