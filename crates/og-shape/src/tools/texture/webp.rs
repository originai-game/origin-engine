//! WebP decoded to RGBA8 (image-webp).

use super::*;

/// A WebP image (lossy, lossless, with or without alpha; of an
/// animation, its first frame) by the `image-webp` crate.
pub(crate) fn decode_webp(bytes: &[u8]) -> Result<Image, TextureError> {
    let webp = |e: image_webp::DecodingError| bad(format!("WebP: {e}"));
    let mut d = image_webp::WebPDecoder::new(std::io::Cursor::new(bytes)).map_err(webp)?;
    let (width, height) = d.dimensions();
    if u64::from(width) * u64::from(height) > crate::tools::decode::MAX_PIXELS {
        return Err(TextureError::Unsupported(format!(
            "a {width} × {height} WebP is over {} pixels",
            crate::tools::decode::MAX_PIXELS
        )));
    }
    let size = d
        .output_buffer_size()
        .ok_or_else(|| bad("a WebP too large to hold"))?;
    let mut buf = vec![0u8; size];
    d.read_image(&mut buf).map_err(webp)?;
    let rgba = if d.has_alpha() {
        buf
    } else {
        buf.as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect()
    };
    Ok(Image {
        width,
        height,
        rgba,
    })
}
