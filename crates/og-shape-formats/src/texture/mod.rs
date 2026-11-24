//! A texture budget: a map whose longest side is over a limit is decoded,
//! shrunk by a box filter and written again as PNG.
//!
//! Entry: [`downsize`]. Decoding reads PNG (every colour type and bit
//! depth, interlaced or not) and JPEG, baseline, extended sequential
//! or progressive (Huffman, 8-bit, any sampling; a lossless, hierarchical
//! or arithmetic-coded JPEG is [`TextureError::Unsupported`]), and KTX2
//! of UASTC blocks ([`crate::ktx2`]). [`to_ktx2`] writes a map as one.
//! Everything after the file's bytes is
//! integer arithmetic: the JPEG's inverse DCT in fixed point, the box
//! filter with integer area weights, so the output is the same bytes on
//! every host. The one table computed in f64 (sRGB to linear, by
//! `og_core::math::double::pow`) is rounded to integers before use.
//!
//! The filter averages each output texel over the input area it covers
//! (exact rational weights). Colour maps (base colour, emissive) are
//! averaged in linear light and their alpha as it is; data maps
//! (metallic-roughness, normal, occlusion) channel by channel as stored,
//! so a normal map's averaged vectors are not renormalised.

use std::fmt;

use og_shape::tools::image::Texture;
use og_shape::tools::preview::Image;

mod budget;
mod convert;
pub mod decode;
pub(crate) use crate::jpeg::read::*;
pub(crate) use crate::png::read::*;
pub(crate) use crate::webp::read::*;
pub use budget::*;
pub use convert::*;
pub use decode::*;

/// How far to shrink. The zero value changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Downsize {
    /// The longest side allowed, in texels; 0 is no limit. A larger map is
    /// scaled so its longest side is exactly this, keeping its aspect.
    pub max: u32,
    /// The map holds colour (sRGB: averaged in linear light); otherwise
    /// data, averaged as stored.
    pub color: bool,
}

/// Why a texture could not be resized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureError {
    /// The image is broken or not an image (what).
    Format(String),
    /// A kind of image this decoder does not read (which).
    Unsupported(String),
}

impl fmt::Display for TextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureError::Format(w) => write!(f, "not a readable image: {w}"),
            TextureError::Unsupported(w) => write!(f, "not decoded: {w}"),
        }
    }
}

impl std::error::Error for TextureError {}

pub(crate) fn bad(what: impl Into<String>) -> TextureError {
    TextureError::Format(what.into())
}

/// Decodes a PNG, a JPEG, a KTX2 file (UASTC, or ETC1S under BasisLZ) or a
/// WebP image to RGBA8.
pub fn decode(bytes: &[u8]) -> Result<Image, TextureError> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        decode_jpeg(bytes)
    } else if bytes.starts_with(b"\xabKTX 20") {
        crate::ktx2::decode(bytes).map_err(TextureError::Unsupported)
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        decode_webp(bytes)
    } else {
        Err(TextureError::Unsupported(
            "only PNG, JPEG, KTX2 and WebP are decoded".into(),
        ))
    }
}

// ── Box filter ──────────────────────────────────────────────────────────────

// ── PNG ─────────────────────────────────────────────────────────────────────

// ── JPEG (baseline) ─────────────────────────────────────────────────────────
