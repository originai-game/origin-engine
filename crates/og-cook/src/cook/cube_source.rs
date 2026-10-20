//! HDR cube maps an environment (`type environment`) takes instead of an
//! equirectangular image: a KTX2 cube map, decoded here to linear RGB
//! floats for og-render's `bake::bake_cube`.
//!
//! A KTX2 cube map has `faceCount` 6, square faces in the order +x −x +y
//! −y +z −z with rows from the top (KTX2's and WebGPU's cube layout, the
//! one og-render's `face_direction` draws), no array layers. Level 0 is
//! read (a file's own mip chain, prefiltered or not, is not: the bake makes
//! its own levels). Its texels may be:
//!
//! | `vkFormat` | name | decoded |
//! |---:|---|---|
//! | 97 | `R16G16B16A16_SFLOAT` | half floats, alpha ignored |
//! | 90 | `R16G16B16_SFLOAT` | half floats |
//! | 109 | `R32G32B32A32_SFLOAT` | floats, alpha ignored |
//! | 106 | `R32G32B32_SFLOAT` | floats |
//! | 123 | `E5B9G9R9_UFLOAT_PACK32` | RGB9E5, og-render's encoding |
//! | 122 | `B10G11R11_UFLOAT_PACK32` | packed 11/11/10-bit floats |
//! | 143, 144 | `BC6H_UFLOAT_BLOCK`, `BC6H_SFLOAT_BLOCK` | [`super::bc6h`], at cook time |
//!
//! with no supercompression, Zstandard (scheme 2) or zlib (scheme 3). A
//! negative texel (signed formats) is light that cannot be: it reads as 0;
//! a texel that is not a finite number is refused, naming its face and
//! position. A file whose `KTXorientation` is not `rd` (x right, y down)
//! is refused: a cube map's faces have one orientation.

use super::bc6h;

pub const VK_R16G16B16_SFLOAT: u32 = 90;
pub const VK_R16G16B16A16_SFLOAT: u32 = 97;
pub const VK_R32G32B32_SFLOAT: u32 = 106;
pub const VK_R32G32B32A32_SFLOAT: u32 = 109;
pub const VK_B10G11R11_UFLOAT_PACK32: u32 = 122;
pub const VK_E5B9G9R9_UFLOAT_PACK32: u32 = 123;
pub const VK_BC6H_UFLOAT_BLOCK: u32 = 143;
pub const VK_BC6H_SFLOAT_BLOCK: u32 = 144;

/// A decoded cube map: six `size` x `size` faces of linear RGB, face after
/// face in og-render's order.
#[derive(Clone, Debug, PartialEq)]
pub struct CubeImage {
    pub size: u32,
    /// `6 * size * size * 3` floats.
    pub rgb: Vec<f32>,
    /// The format's name, for the report.
    pub format: &'static str,
}

/// The name of a cube map format the cook takes (`None`: not one).
pub fn format_name(vk_format: u32) -> Option<&'static str> {
    Some(match vk_format {
        VK_R16G16B16_SFLOAT => "RGB16F",
        VK_R16G16B16A16_SFLOAT => "RGBA16F",
        VK_R32G32B32_SFLOAT => "RGB32F",
        VK_R32G32B32A32_SFLOAT => "RGBA32F",
        VK_B10G11R11_UFLOAT_PACK32 => "B10G11R11",
        VK_E5B9G9R9_UFLOAT_PACK32 => "RGB9E5",
        VK_BC6H_UFLOAT_BLOCK => "BC6H unsigned",
        VK_BC6H_SFLOAT_BLOCK => "BC6H signed",
        _ => return None,
    })
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes(s.try_into().expect("4 bytes")))
        .ok_or_else(|| format!("KTX2 truncated at byte {at}"))
}

fn u64_at(b: &[u8], at: usize) -> Result<usize, String> {
    b.get(at..at + 8)
        .map(|s| u64::from_le_bytes(s.try_into().expect("8 bytes")) as usize)
        .ok_or_else(|| format!("KTX2 truncated at byte {at}"))
}

/// An unsigned float of `exp` exponent and `man` mantissa bits (bias 15,
/// as B10G11R11's).
fn small_float(bits: u32, man: u32) -> f32 {
    let m = (bits & ((1 << man) - 1)) as f32;
    let e = (bits >> man) & 31;
    match e {
        0 => m / (1u32 << man) as f32 * 2f32.powi(-14),
        31 if m == 0.0 => f32::INFINITY,
        31 => f32::NAN,
        e => (1.0 + m / (1u32 << man) as f32) * 2f32.powi(e as i32 - 15),
    }
}

/// Reads level 0 of a KTX2 cube map into linear RGB.
pub fn read_ktx2_cube(bytes: &[u8]) -> Result<CubeImage, String> {
    if bytes.len() < 80 || bytes[..12] != crate::ktx2::IDENTIFIER {
        return Err("not a KTX2 file".into());
    }
    let vk_format = u32_at(bytes, 12)?;
    let (width, height) = (u32_at(bytes, 20)?, u32_at(bytes, 24)?);
    let (depth, layers, faces) = (u32_at(bytes, 28)?, u32_at(bytes, 32)?, u32_at(bytes, 36)?);
    let supercompression = u32_at(bytes, 44)?;
    let format = format_name(vk_format).ok_or_else(|| {
        format!(
            "vkFormat {vk_format} is not an HDR cube map format the cook decodes (RGBA16F, RGB16F, RGBA32F, RGB32F, RGB9E5, B10G11R11 or BC6H)"
        )
    })?;
    if faces != 6 {
        return Err(format!(
            "a KTX2 of {faces} face(s): an environment's KTX2 is a cube map (6 faces)"
        ));
    }
    if depth > 1 || layers > 1 {
        return Err("a cube map array or a 3D texture: give one cube map".into());
    }
    if width != height || width == 0 {
        return Err(format!(
            "faces of {width} x {height}: a cube map's faces are square"
        ));
    }
    if let Some(o) = crate::ktx2::metadata(bytes)?
        .iter()
        .find(|(k, _)| k == "KTXorientation")
        .map(|(_, v)| String::from_utf8_lossy(v.strip_suffix(&[0]).unwrap_or(v)).into_owned())
        && !o.starts_with("rd")
    {
        return Err(format!(
            "KTXorientation \"{o}\": a cube map's faces are x right, y down (\"rd\")"
        ));
    }
    let (offset, length) = (u64_at(bytes, 80)?, u64_at(bytes, 88)?);
    let stored = offset
        .checked_add(length)
        .and_then(|end| bytes.get(offset..end))
        .ok_or("KTX2 level 0 runs past the end of the file")?;
    let data = match supercompression {
        crate::ktx2::SS_NONE => stored.to_vec(),
        crate::ktx2::SS_ZSTD => {
            let mut out = Vec::new();
            let mut decoder = ruzstd::decoding::StreamingDecoder::new(stored)
                .map_err(|e| format!("KTX2 zstd level 0: {e}"))?;
            std::io::Read::read_to_end(&mut decoder, &mut out)
                .map_err(|e| format!("KTX2 zstd level 0: {e}"))?;
            out
        }
        crate::ktx2::SS_ZLIB => miniz_oxide::inflate::decompress_to_vec_zlib(stored)
            .map_err(|e| format!("KTX2 zlib level 0: {e:?}"))?,
        s => {
            return Err(format!(
                "KTX2 supercompression {s}: an HDR cube map is stored plain, with Zstandard or with zlib"
            ));
        }
    };
    let n = width as usize;
    let texels = n * n;
    let mut rgb = vec![0f32; 6 * texels * 3];
    let face_bytes = match vk_format {
        VK_BC6H_UFLOAT_BLOCK | VK_BC6H_SFLOAT_BLOCK => n.div_ceil(4) * n.div_ceil(4) * 16,
        VK_R16G16B16_SFLOAT => texels * 6,
        VK_R16G16B16A16_SFLOAT => texels * 8,
        VK_R32G32B32_SFLOAT => texels * 12,
        VK_R32G32B32A32_SFLOAT => texels * 16,
        _ => texels * 4,
    };
    if data.len() < 6 * face_bytes {
        return Err(format!(
            "KTX2 level 0 holds {} bytes; six {n} x {n} {format} faces take {}",
            data.len(),
            6 * face_bytes
        ));
    }
    for face in 0..6 {
        let src = &data[face * face_bytes..(face + 1) * face_bytes];
        let dst = &mut rgb[face * texels * 3..(face + 1) * texels * 3];
        let h = |at: usize| bc6h::half(u16::from_le_bytes([src[at], src[at + 1]]));
        let f = |at: usize| f32::from_le_bytes(src[at..at + 4].try_into().expect("4 bytes"));
        let w = |at: usize| u32::from_le_bytes(src[at..at + 4].try_into().expect("4 bytes"));
        match vk_format {
            VK_BC6H_UFLOAT_BLOCK | VK_BC6H_SFLOAT_BLOCK => {
                let blocks = n.div_ceil(4);
                for by in 0..blocks {
                    for bx in 0..blocks {
                        let at = (by * blocks + bx) * 16;
                        let block: &[u8; 16] = src[at..at + 16].try_into().expect("16 bytes");
                        let t = bc6h::decode_block(block, vk_format == VK_BC6H_SFLOAT_BLOCK);
                        for (i, c) in t.iter().enumerate() {
                            let (x, y) = (bx * 4 + i % 4, by * 4 + i / 4);
                            if x < n && y < n {
                                dst[(y * n + x) * 3..(y * n + x) * 3 + 3].copy_from_slice(c);
                            }
                        }
                    }
                }
            }
            _ => {
                for i in 0..texels {
                    let c = match vk_format {
                        VK_R16G16B16_SFLOAT => [h(i * 6), h(i * 6 + 2), h(i * 6 + 4)],
                        VK_R16G16B16A16_SFLOAT => [h(i * 8), h(i * 8 + 2), h(i * 8 + 4)],
                        VK_R32G32B32_SFLOAT => [f(i * 12), f(i * 12 + 4), f(i * 12 + 8)],
                        VK_R32G32B32A32_SFLOAT => [f(i * 16), f(i * 16 + 4), f(i * 16 + 8)],
                        VK_E5B9G9R9_UFLOAT_PACK32 => og_render::rgb9e5_decode(w(i * 4)),
                        _ => {
                            let v = w(i * 4);
                            [
                                small_float(v & 0x7ff, 6),
                                small_float((v >> 11) & 0x7ff, 6),
                                small_float(v >> 22, 5),
                            ]
                        }
                    };
                    dst[i * 3..i * 3 + 3].copy_from_slice(&c);
                }
            }
        }
        if let Some(i) = dst.iter().position(|v| !v.is_finite()) {
            let t = i / 3;
            return Err(format!(
                "face {face} ({}) texel ({}, {}) is not a finite number",
                ["+x", "-x", "+y", "-y", "+z", "-z"][face],
                t % n,
                t / n
            ));
        }
    }
    for v in &mut rgb {
        *v = v.max(0.0);
    }
    Ok(CubeImage {
        size: width,
        rgb,
        format,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_floats_decode() {
        // 1.0 in 11 bits: exponent 15, mantissa 0; 0.5 in 10 bits: 14.
        assert_eq!(small_float(15 << 6, 6), 1.0);
        assert_eq!(small_float(14 << 5, 5), 0.5);
        assert_eq!(small_float(1, 6), 2f32.powi(-14) / 64.0);
        assert!(small_float(31 << 6, 6).is_infinite());
    }

    #[test]
    fn a_ktx2_that_is_not_a_cube_is_refused() {
        assert!(read_ktx2_cube(b"not a ktx2").is_err());
    }
}
