//! Texture formats by tier, pre-encoded uploads ([`TextureUpload`]) and the
//! format choice reported in [`crate::RenderInfo::texture_formats`].
//!
//! The renderer does not transcode: the host (the cook's Basis Universal
//! transcoder natively, the web host's in the browser) asks
//! `RenderInfo::texture_formats` which encoding each kind of texture should
//! arrive in, transcodes every mip level to it and uploads the levels with
//! [`crate::RenderScene::add_texture_upload`]. RGBA8 [`crate::TextureData`]
//! keeps working everywhere, with mips generated on the GPU.

use crate::scene::ColorSpace;

/// A texel encoding a [`TextureUpload`] can carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextureFormat {
    /// Uncompressed 8-bit RGBA: every backend.
    Rgba8,
    /// BC7 RGBA (desktop: Metal on macOS, Vulkan, D3D, WebGPU/WebGL2 with
    /// `texture-compression-bc`).
    Bc7,
    /// BC5: two channels (normal map x and y; z is reconstructed). Linear only.
    Bc5,
    /// ASTC LDR with 4x4 blocks (mobile GPUs, Apple silicon).
    Astc4x4,
    /// ETC2 RGBA8 (EAC alpha): OpenGL ES 3 / WebGL2 baseline on mobile.
    Etc2Rgba8,
    /// EAC RG11: two channels (normal maps on the ETC2 tier). Linear only.
    EacRg11,
}

impl TextureFormat {
    /// Every format, in declaration order.
    pub const ALL: [TextureFormat; 6] = [
        TextureFormat::Rgba8,
        TextureFormat::Bc7,
        TextureFormat::Bc5,
        TextureFormat::Astc4x4,
        TextureFormat::Etc2Rgba8,
        TextureFormat::EacRg11,
    ];

    /// Texels per block side (1 for RGBA8, 4 for the block formats).
    pub fn block_dim(self) -> u32 {
        match self {
            TextureFormat::Rgba8 => 1,
            _ => 4,
        }
    }

    /// Bytes per block (per texel for RGBA8).
    pub fn block_bytes(self) -> u32 {
        match self {
            TextureFormat::Rgba8 => 4,
            _ => 16,
        }
    }

    /// Whether the format holds only red and green (normal maps' x and y).
    pub fn two_channel(self) -> bool {
        matches!(self, TextureFormat::Bc5 | TextureFormat::EacRg11)
    }

    /// The bytes of mip `level` of a `width` x `height` texture.
    pub fn level_bytes(self, width: u32, height: u32, level: u32) -> usize {
        let (w, h) = ((width >> level).max(1), (height >> level).max(1));
        let d = self.block_dim();
        (w.div_ceil(d) * h.div_ceil(d) * self.block_bytes()) as usize
    }

    /// The wgpu format for `color_space` (`None`: no sRGB variant exists).
    pub(crate) fn wgpu(self, color_space: ColorSpace) -> Option<wgpu::TextureFormat> {
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

/// Which compressed family the renderer picks when several are available
/// ([`crate::RenderDesc::texture_compression`]). Unavailable families are
/// skipped; RGBA8 is the last resort everywhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureCompression {
    /// BC, then ASTC, then ETC2: BC7/BC5 on desktop GPUs, ASTC or ETC2 on
    /// mobile ones (which lack BC).
    #[default]
    Auto,
    /// ASTC, then ETC2, then BC (the mobile preset; on desktop it exercises
    /// the mobile formats where the GPU has them).
    PreferAstc,
    /// ETC2, then ASTC, then BC.
    PreferEtc2,
    /// RGBA8 only.
    Off,
}

/// The encoding each kind of texture should be uploaded in on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureFormats {
    /// Base colour, emissive (sRGB) and other RGBA textures.
    pub color: TextureFormat,
    /// Normal maps (linear; two-channel formats reconstruct z).
    pub normal: TextureFormat,
    /// Linear data: metallic-roughness, occlusion.
    pub data: TextureFormat,
    /// Every format this device accepts, [`TextureFormat::Rgba8`] first.
    pub supported: Vec<TextureFormat>,
}

impl TextureFormats {
    /// The features to request and the choice for `preference`, given what
    /// the adapter offers.
    pub(crate) fn choose(
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

/// What a pre-encoded texture is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    /// Mip levels supplied, from level 0 (1 = no mips). A full chain is
    /// `32 - max(width, height).leading_zeros()`.
    pub mip_levels: u32,
    /// sRGB (decoded when sampled) or linear data. The two-channel formats
    /// are linear only.
    pub color_space: ColorSpace,
}

impl TextureDesc {
    /// The level count of a full chain down to 1x1.
    pub fn full_chain(width: u32, height: u32) -> u32 {
        32 - width.max(height).max(1).leading_zeros()
    }

    /// Checks `levels` against the description: one entry per mip level,
    /// each exactly [`TextureFormat::level_bytes`] long, and a level-0 size
    /// that is a whole number of blocks.
    pub fn validate(&self, levels: &[Vec<u8>]) -> Result<(), String> {
        let d = self.format.block_dim();
        if self.width == 0 || self.height == 0 {
            return Err(format!("{}x{} texture", self.width, self.height));
        }
        if !self.width.is_multiple_of(d) || !self.height.is_multiple_of(d) {
            return Err(format!(
                "{:?} needs a size in whole {d}x{d} blocks, got {}x{}",
                self.format, self.width, self.height
            ));
        }
        let max = Self::full_chain(self.width, self.height);
        if self.mip_levels == 0 || self.mip_levels > max {
            return Err(format!(
                "{} mip levels for {}x{} (1..={max})",
                self.mip_levels, self.width, self.height
            ));
        }
        if levels.len() != self.mip_levels as usize {
            return Err(format!(
                "{} levels supplied, the description says {}",
                levels.len(),
                self.mip_levels
            ));
        }
        for (i, level) in levels.iter().enumerate() {
            let expected = self.format.level_bytes(self.width, self.height, i as u32);
            if level.len() != expected {
                return Err(format!(
                    "level {i} has {} bytes, {:?} {}x{} level {i} needs {expected}",
                    level.len(),
                    self.format,
                    self.width,
                    self.height
                ));
            }
        }
        if self.format.wgpu(self.color_space).is_none() {
            return Err(format!("{:?} has no sRGB variant", self.format));
        }
        Ok(())
    }
}

/// A texture whose mip levels arrive already encoded (cooked mips,
/// transcoded to [`crate::RenderInfo::texture_formats`]). Levels are tightly
/// packed rows of blocks (texels for RGBA8), level 0 first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureUpload {
    pub desc: TextureDesc,
    pub levels: Vec<Vec<u8>>,
}
