//! Texture formats by device, pre-encoded uploads ([`TextureUpload`]) and the
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
    /// BC7 RGBA (desktop: Metal on macOS, Vulkan, D3D, WebGPU with
    /// `texture-compression-bc`). 8 bpp.
    Bc7,
    /// BC1 RGB opaque (desktop `texture-compression-bc`). 4 bpp.
    Bc1,
    /// BC4 single-channel R linear (AO, height, masks; desktop
    /// `texture-compression-bc`). 4 bpp.
    Bc4,
    /// BC5: two channels (normal map x and y; z is reconstructed). Linear only. 8 bpp.
    Bc5,
    /// ASTC LDR with 4x4 blocks (mobile GPUs, Apple silicon). 8 bpp.
    Astc4x4,
    /// ETC2 RGBA8 (EAC alpha): the mobile baseline (WebGPU's
    /// `texture-compression-etc2`). 8 bpp.
    Etc2Rgba8,
    /// ETC2 RGB8 opaque (WebGPU's `texture-compression-etc2`). 4 bpp.
    Etc2Rgb8,
    /// EAC R11 single-channel linear (AO, height, masks on ETC2 tier). 4 bpp.
    EacR11,
    /// EAC RG11: two channels (normal maps on the ETC2 tier). Linear only. 8 bpp.
    EacRg11,
}

impl TextureFormat {
    /// Every format, in declaration order.
    pub const ALL: [TextureFormat; 10] = [
        TextureFormat::Rgba8,
        TextureFormat::Bc7,
        TextureFormat::Bc1,
        TextureFormat::Bc4,
        TextureFormat::Bc5,
        TextureFormat::Astc4x4,
        TextureFormat::Etc2Rgba8,
        TextureFormat::Etc2Rgb8,
        TextureFormat::EacR11,
        TextureFormat::EacRg11,
    ];

    /// The format's short name, as hosts and transcoders name it: `rgba8`,
    /// `bc7`, `bc1`, `bc4`, `bc5`, `astc4x4`, `etc2`, `etc2rgb`, `eacr11`, `eacrg11`.
    pub fn name(self) -> &'static str {
        match self {
            TextureFormat::Rgba8 => "rgba8",
            TextureFormat::Bc7 => "bc7",
            TextureFormat::Bc1 => "bc1",
            TextureFormat::Bc4 => "bc4",
            TextureFormat::Bc5 => "bc5",
            TextureFormat::Astc4x4 => "astc4x4",
            TextureFormat::Etc2Rgba8 => "etc2",
            TextureFormat::Etc2Rgb8 => "etc2rgb",
            TextureFormat::EacR11 => "eacr11",
            TextureFormat::EacRg11 => "eacrg11",
        }
    }

    /// The format [`TextureFormat::name`] names.
    pub fn from_name(name: &str) -> Option<TextureFormat> {
        TextureFormat::ALL.into_iter().find(|f| f.name() == name)
    }

    /// Texels per block side (1 for RGBA8, 4 for the block formats).
    pub fn block_dim(self) -> u32 {
        match self {
            TextureFormat::Rgba8 => 1,
            _ => 4,
        }
    }

    /// Bytes per block (per texel for RGBA8; 8 for 4 bpp blocks, 16 for 8 bpp blocks).
    pub fn block_bytes(self) -> u32 {
        match self {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Bc1
            | TextureFormat::Bc4
            | TextureFormat::Etc2Rgb8
            | TextureFormat::EacR11 => 8,
            _ => 16,
        }
    }

    /// Whether the format holds only red and green (normal maps' x and y).
    pub fn two_channel(self) -> bool {
        matches!(self, TextureFormat::Bc5 | TextureFormat::EacRg11)
    }

    /// Whether the format holds only a single linear red channel (AO, height, masks).
    pub fn single_channel(self) -> bool {
        matches!(self, TextureFormat::Bc4 | TextureFormat::EacR11)
    }

    /// The bytes of mip `level` of a `width` x `height` texture.
    pub fn level_bytes(self, width: u32, height: u32, level: u32) -> usize {
        let (w, h) = ((width >> level).max(1), (height >> level).max(1));
        let d = self.block_dim();
        (w.div_ceil(d) * h.div_ceil(d) * self.block_bytes()) as usize
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
    /// Base colour with alpha, emissive (sRGB) and other RGBA textures.
    pub color: TextureFormat,
    /// Opaque ETC1S colour textures (4 bpp BC1 / ETC2 RGB8 where available).
    pub color_opaque: TextureFormat,
    /// Normal maps (linear; two-channel formats reconstruct z).
    pub normal: TextureFormat,
    /// Multi-channel linear data: metallic-roughness, ORM.
    pub data: TextureFormat,
    /// Single-channel linear masks (AO, height, dissolve; 4 bpp BC4 / EAC R11).
    pub single: TextureFormat,
    /// Every format this device accepts, [`TextureFormat::Rgba8`] first.
    pub supported: Vec<TextureFormat>,
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
    /// sRGB (decoded when sampled) or linear data. Single- and two-channel
    /// formats are linear only.
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
        if (self.format.two_channel() || self.format.single_channel())
            && self.color_space == ColorSpace::Srgb
        {
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
