//! Decodes a texture's PNG, JPEG, KTX2 or WebP bytes to RGBA8, for the preview to
//! sample (`docs/research.md` §30, §33).
//!
//! The decoding is the texture budget's ([`crate::texture::decode`],
//! `docs/research.md` §29.1): PNG of every colour type and bit depth,
//! Adam7, a palette's `tRNS` (16-bit samples rounded to 8 bits); baseline,
//! extended sequential and progressive JPEG, one or three components,
//! integer inverse DCT. Colour-space chunks (`gAMA`, `iCCP`, `sRGB`) are ignored: the
//! bytes are taken as sRGB, as glTF says a base-colour image is.
//! Arithmetic-coded and lossless JPEG are refused with
//! the reason, as is an image over [`MAX_PIXELS`] (read from its header,
//! before anything is decoded).
//!
//! Integer arithmetic only.

use og_shape::tools::image::ImageInfo;
use og_shape::tools::preview::Image;

/// The most pixels an image may have: 64 Mi, a 8192² texture.
pub const MAX_PIXELS: u64 = 1 << 26;

/// The codecs the tools take ([`og_shape::tools::image::Codec`]): [`decode`]
/// and the PNG encoder.
pub const CODEC: og_shape::tools::image::Codec = og_shape::tools::image::Codec {
    decode,
    png: crate::png::write::png,
};

/// The image an image file's bytes hold, as RGBA8, rows top to bottom.
pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        && !bytes.starts_with(&[0xFF, 0xD8])
        && !bytes.starts_with(b"\xabKTX 20")
        && !(bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP")
    {
        return Err("not a PNG, JPEG, KTX2 or WebP image (the preview decodes those)".into());
    }
    if let (Some(w), Some(h)) = {
        let i = ImageInfo::from_bytes("", bytes);
        (i.width, i.height)
    } {
        too_big(w, h)?;
    }
    let im = crate::texture::decode(bytes).map_err(|e| e.to_string())?;
    too_big(im.width, im.height)?;
    Ok(im)
}

fn too_big(w: u32, h: u32) -> Result<(), String> {
    if w == 0 || h == 0 {
        return Err("an image with no pixels".into());
    }
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(format!(
            "a {w} × {h} image is over the preview's {MAX_PIXELS} pixels"
        ));
    }
    Ok(())
}
