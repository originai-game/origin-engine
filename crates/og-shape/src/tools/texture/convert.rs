//! A map written again as WebP or KTX2 (UASTC, with or without rate-distortion, or ETC1S).

use super::*;

/// `t` as a lossless WebP file (`EXT_texture_webp`): every texel as it
/// is, in fewer bytes than PNG on most maps; `None` when it is WebP or
/// KTX2 already, or when the WebP would not be smaller (a JPEG kept
/// lossless is several times its size).
pub fn to_webp(t: &Texture) -> Result<Option<Texture>, TextureError> {
    if t.mime == "image/webp" || t.mime == "image/ktx2" {
        return Ok(None);
    }
    let packed = webp_of(&t.name, &decode(&t.bytes)?);
    Ok((packed.bytes.len() < t.bytes.len()).then_some(packed))
}

/// `image` as a lossless WebP texture named after `name` (by the
/// `image-webp` crate's encoder): three channels where every texel is
/// opaque, four otherwise.
pub(crate) fn webp_of(name: &str, image: &Image) -> Texture {
    let opaque = image.rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
    let (data, color) = if opaque {
        (
            image
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect(),
            image_webp::ColorType::Rgb8,
        )
    } else {
        (image.rgba.clone(), image_webp::ColorType::Rgba8)
    };
    let mut bytes = Vec::new();
    // Writing to a vector does not fail, and the sizes are the image's.
    let _ =
        image_webp::WebPEncoder::new(&mut bytes).encode(&data, image.width, image.height, color);
    Texture {
        name: match name.rsplit_once('.') {
            Some((stem, _)) => format!("{stem}.webp"),
            None => name.to_string(),
        },
        mime: "image/webp".into(),
        bytes,
    }
}

/// `t` as a KTX2 file of UASTC blocks with its mip levels
/// (`KHR_texture_basisu`), compressed with Zstandard; `None` when it is
/// a KTX2 file already. `color` as in [`Downsize::color`]: the texels
/// are sRGB, and the levels are averaged in linear light.
pub fn to_ktx2(t: &Texture, color: bool) -> Result<Option<Texture>, TextureError> {
    ktx2_of(t, color, false, 0.0)
}

/// As [`to_ktx2`], trading quality for bytes: a block may be written
/// as a copy of one just before it or above it where that costs it at
/// most `rdo` in squared error a channel of a texel over its own best,
/// and the Zstandard pass then stores the copy as a reference. 0 is
/// [`to_ktx2`]; 1 to 4 are useful.
pub fn to_ktx2_rdo(t: &Texture, color: bool, rdo: f64) -> Result<Option<Texture>, TextureError> {
    if !(rdo >= 0.0 && rdo.is_finite()) {
        return Err(bad("the rate-distortion allowance must be 0 or more"));
    }
    ktx2_of(t, color, false, rdo)
}

/// As [`to_ktx2`] with ETC1S blocks under BasisLZ: the small mode of
/// `KHR_texture_basisu`, a fraction of UASTC's bytes at a lower
/// quality (one colour and one intensity table a block, from codebooks
/// the whole file shares).
pub fn to_ktx2_etc1s(t: &Texture, color: bool) -> Result<Option<Texture>, TextureError> {
    ktx2_of(t, color, true, 0.0)
}

pub(crate) fn ktx2_of(
    t: &Texture,
    color: bool,
    small: bool,
    rdo: f64,
) -> Result<Option<Texture>, TextureError> {
    if t.mime == "image/ktx2" {
        return Ok(None);
    }
    let image = decode(&t.bytes)?;
    let alpha = image.rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    let mut levels = Vec::new();
    let (mut w, mut h) = (image.width, image.height);
    while w > 1 || h > 1 {
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
        levels.push(resize(&image, w, h, color));
    }
    levels.insert(0, image);
    let name = match t.name.rsplit_once('.') {
        Some((stem, _)) => format!("{stem}.ktx2"),
        None => format!("{}.ktx2", t.name),
    };
    let bytes = if small {
        // Codebooks by how many blocks there are, as the reference
        // tool's default comes out on a 1024² image (about 2 000
        // endpoints and 1 300 selectors).
        let blocks: usize = levels
            .iter()
            .map(|l| (l.width as usize).div_ceil(4) * (l.height as usize).div_ceil(4))
            .sum::<usize>()
            * if alpha { 2 } else { 1 };
        crate::tools::ktx2::encode_etc1s(
            &levels,
            color,
            alpha,
            (blocks / 40).clamp(256, 8192),
            (blocks / 60).clamp(256, 8192),
        )
    } else {
        crate::tools::ktx2::encode(&levels, color, alpha, rdo)
    };
    Ok(Some(Texture {
        name,
        mime: "image/ktx2".into(),
        bytes,
    }))
}
