//! The load budget and an image's header, checked before anything is decoded.

use super::*;

/// What `LoadBudget` leaves unset.
pub(super) const BYTES: f64 = 32.0 * 1024.0 * 1024.0;

pub(super) const TEXTURE_SIZE: u32 = 4096;

pub(super) const FRAME_MS: f64 = 2.0;

/// The largest texture every WebGPU device takes (`maxTextureDimension2D`'s
/// guaranteed minimum): `textureSize` is capped here.
pub(super) const DEVICE_TEXTURE_SIZE: u32 = 8192;

/// Whether og.load.environmentData reads OpenEXR here
/// (RenderCapabilities.exrDecode): natively, through the `image` crate; the
/// web module carries no EXR decoder.
pub(crate) const EXR_DECODE: bool = cfg!(not(target_arch = "wasm32"));

/// The face sizes an uncooked environment's cube may have.
pub(super) const ENV_SIZES: std::ops::RangeInclusive<u32> =
    og_render::ENVIRONMENT_MAP_MIN_SIZE..=og_render::ENVIRONMENT_MAP_MAX_SIZE;

impl Budget {
    /// A desc's `budget` (each field its default when omitted).
    pub fn from_desc(desc: &Value) -> Result<Budget, String> {
        let b = &desc["budget"];
        let num = |k: &str, default: f64| -> Result<f64, String> {
            match &b[k] {
                Value::Null => Ok(default),
                Value::Number(n) => n
                    .as_f64()
                    .filter(|v| v.is_finite() && *v >= 0.0)
                    .ok_or_else(|| format!("LoadBudget.{k} is a number of at least 0, got {n}")),
                other => Err(format!("LoadBudget.{k} is a number, got {other}")),
            }
        };
        Ok(Budget {
            bytes: num("bytes", BYTES)?,
            triangles: num("triangles", 500_000.0)?.min(u32::MAX as f64) as u32,
            texture_size: (num("textureSize", TEXTURE_SIZE as f64)?.min(u32::MAX as f64) as u32)
                .min(DEVICE_TEXTURE_SIZE),
            frame_ms: num("frameMs", FRAME_MS)?,
        })
    }
}

/// An image's format and size, read from its header only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ImageHeader {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
}

/// The format and size of a PNG or JPEG from its header, without decoding
/// it; why not for anything else.
pub(crate) fn image_header(bytes: &[u8]) -> Result<ImageHeader, String> {
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if bytes.starts_with(PNG) {
        if bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
            return Err("a PNG without its IHDR header (the file is cut short)".into());
        }
        let be = |at: usize| {
            u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        return Ok(ImageHeader {
            format: "png",
            width: be(16),
            height: be(20),
        });
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        // Markers until a start of frame (SOF0 to SOF15 but DHT, JPG and DAC).
        let mut at = 2;
        while at + 4 <= bytes.len() {
            if bytes[at] != 0xff {
                return Err("a JPEG whose markers do not parse".into());
            }
            let marker = bytes[at + 1];
            if marker == 0xff {
                at += 1;
                continue;
            }
            if (0xd0..=0xd9).contains(&marker) || marker == 0x01 {
                at += 2;
                continue;
            }
            let len = u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]) as usize;
            if (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker) {
                if at + 9 > bytes.len() {
                    break;
                }
                let height = u16::from_be_bytes([bytes[at + 5], bytes[at + 6]]) as u32;
                let width = u16::from_be_bytes([bytes[at + 7], bytes[at + 8]]) as u32;
                return Ok(ImageHeader {
                    format: "jpeg",
                    width,
                    height,
                });
            }
            at += 2 + len;
        }
        return Err("a JPEG without a frame header (the file is cut short)".into());
    }
    Err("not a PNG or JPEG (uncooked loads read those two; convert other formats first)".into())
}

/// An image's header checked against a budget: the reasons name the limit,
/// the value and the fix.
pub(crate) fn check_image(
    name: &str,
    bytes: &[u8],
    budget: &Budget,
) -> Result<ImageHeader, String> {
    check_size(name, bytes, budget)?;
    let h = image_header(bytes).map_err(|e| format!("{name}: {e}"))?;
    if h.width == 0 || h.height == 0 {
        return Err(format!(
            "{name}: a {} of {}x{} texels holds nothing",
            h.format, h.width, h.height
        ));
    }
    if h.width.max(h.height) > budget.texture_size {
        return Err(format!(
            "{name}: {}x{} is over LoadBudget.textureSize ({}); scale the image down or raise the budget (at most {DEVICE_TEXTURE_SIZE}, the largest every device takes)",
            h.width, h.height, budget.texture_size
        ));
    }
    Ok(h)
}

/// The source's size against `LoadBudget.bytes`.
pub(super) fn check_size(name: &str, bytes: &[u8], budget: &Budget) -> Result<(), String> {
    if bytes.len() as f64 > budget.bytes {
        return Err(format!(
            "{name}: {} bytes is over LoadBudget.bytes ({}); make the file smaller or raise the budget",
            bytes.len(),
            budget.bytes
        ));
    }
    Ok(())
}
