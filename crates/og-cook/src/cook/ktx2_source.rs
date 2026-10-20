//! KTX2 files as sources (`assets/**/*.ktx2`, authored or from another
//! tool) and as `toktx`'s output: their `KTXorientation` and `KTXswizzle`
//! metadata honoured, so a cooked texture is always rows top to bottom,
//! columns left to right, channels as named.
//!
//! A KTX2 the runtime can take as it is (Basis ETC1S or UASTC, or RGBA8;
//! orientation `rd`, swizzle `rgba`, both or either absent; a colour or
//! linear kind) is passed through untouched. Any other is decoded to RGBA8
//! (Basis through the native transcoder), flipped to `rd`, swizzled
//! (`r g b a 0 1` per channel) and cooked again as an image of its kind;
//! the report says what was done.

use super::CookError;
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;

/// A KTX2's orientation as flips to `rd` (x right, y down).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orientation {
    pub flip_x: bool,
    pub flip_y: bool,
}

/// `KTXorientation` for 2D: `r`/`l` then `d`/`u` (absent: `rd`).
pub fn orientation(info: &ktx2::Info) -> Result<Orientation, String> {
    let o = info.orientation.as_deref().unwrap_or("rd");
    let b = o.as_bytes();
    match (b.first(), b.get(1)) {
        (Some(x @ (b'r' | b'l')), Some(y @ (b'd' | b'u'))) => Ok(Orientation {
            flip_x: *x == b'l',
            flip_y: *y == b'u',
        }),
        _ => Err(format!("KTXorientation '{o}' is not one of rd, ru, ld, lu")),
    }
}

/// `KTXswizzle`: per output channel, the source channel (0-3) or a
/// constant (4: 0, 5: 1); absent: `rgba`.
pub fn swizzle(info: &ktx2::Info) -> Result<[u8; 4], String> {
    let s = info.swizzle.as_deref().unwrap_or("rgba");
    let b = s.as_bytes();
    if b.len() != 4 {
        return Err(format!("KTXswizzle '{s}' is not four of r g b a 0 1"));
    }
    let mut out = [0u8; 4];
    for (i, c) in b.iter().enumerate() {
        out[i] = match c {
            b'r' => 0,
            b'g' => 1,
            b'b' => 2,
            b'a' => 3,
            b'0' => 4,
            b'1' => 5,
            _ => return Err(format!("KTXswizzle '{s}' is not four of r g b a 0 1")),
        };
    }
    Ok(out)
}

/// Whether the metadata leaves the texels as they are.
pub fn is_identity(o: Orientation, s: [u8; 4]) -> bool {
    !o.flip_x && !o.flip_y && s == [0, 1, 2, 3]
}

/// RGBA8 texels flipped to `rd` and swizzled.
pub fn apply(rgba: &[u8], w: u32, h: u32, o: Orientation, s: [u8; 4]) -> Vec<u8> {
    let mut out = vec![0u8; rgba.len()];
    for y in 0..h {
        let sy = if o.flip_y { h - 1 - y } else { y };
        for x in 0..w {
            let sx = if o.flip_x { w - 1 - x } else { x };
            let src = ((sy * w + sx) * 4) as usize;
            let dst = ((y * w + x) * 4) as usize;
            for c in 0..4 {
                out[dst + c] = match s[c] {
                    4 => 0,
                    5 => 255,
                    k => rgba[src + k as usize],
                };
            }
        }
    }
    out
}

/// What the file holds, as the cook's codec, colour space and kind.
fn shape(info: &ktx2::Info) -> Result<(u32, u32), String> {
    let srgb = info.transfer == 2 || info.vk_format == ktx2::VK_FORMAT_R8G8B8A8_SRGB;
    let codec = match (info.color_model, info.vk_format) {
        (163, 0) => codec::ETC1S,
        (166, 0) => codec::UASTC,
        (_, ktx2::VK_FORMAT_R8G8B8A8_UNORM | ktx2::VK_FORMAT_R8G8B8A8_SRGB) => codec::RGBA8,
        (m, f) => {
            return Err(format!(
                "a KTX2 of vkFormat {f} (colour model {m}): the cook takes Basis ETC1S or UASTC, or RGBA8"
            ));
        }
    };
    Ok((codec, if srgb { 0 } else { 1 }))
}

/// Level 0 of a KTX2 as RGBA8 (Basis through the native transcoder).
fn decode(
    bytes: &[u8],
    info: &ktx2::Info,
    codec: u32,
    color_space: u32,
) -> Result<Vec<u8>, String> {
    if codec == format::codec::RGBA8 {
        return ktx2::decode_rgba8(bytes, 0).map(|(_, _, px)| px);
    }
    let t = crate::runtime::CookedTexture {
        id: "ktx2".into(),
        meta: TextureMeta {
            width: info.width,
            height: info.height,
            levels: info.levels.len() as u32,
            kind: if color_space == 0 {
                format::kind::COLOR
            } else {
                format::kind::LINEAR
            },
            codec,
            color_space,
            has_alpha: info.has_alpha as u32,
            supercompression: info.supercompression,
        },
        ktx2: bytes.to_vec(),
    };
    t.to_texture_data()
        .map(|d| d.rgba)
        .map_err(|e| e.to_string())
}

/// `toktx`'s output with its metadata honoured: as written when it is
/// the identity, else decoded, fixed and handed to `encode` again.
pub fn honour(
    ktx: Vec<u8>,
    encode: impl FnOnce(&super::texture::Image) -> Result<Vec<u8>, String>,
) -> Result<Vec<u8>, String> {
    let info = ktx2::parse(&ktx)?;
    let (o, s) = (orientation(&info)?, swizzle(&info)?);
    if is_identity(o, s) {
        return Ok(ktx);
    }
    let (codec, cs) = shape(&info)?;
    let rgba = decode(&ktx, &info, codec, cs)?;
    encode(&super::texture::Image {
        width: info.width,
        height: info.height,
        rgba: apply(&rgba, info.width, info.height, o, s),
    })
}

/// Cooks a `.ktx2` source. `kind` is its import settings' (`texture {
/// kind }`), else what the file holds (sRGB: colour, else linear).
pub fn cook(
    bytes: &[u8],
    kind: Option<u32>,
    opts: &super::TextureOptions,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let info = ktx2::parse(bytes).map_err(CookError)?;
    let (o, s) = (
        orientation(&info).map_err(CookError)?,
        swizzle(&info).map_err(CookError)?,
    );
    let (codec, color_space) = shape(&info).map_err(CookError)?;
    let kind = kind.unwrap_or(if color_space == 0 {
        format::kind::COLOR
    } else {
        format::kind::LINEAR
    });
    let wants_srgb = kind == format::kind::COLOR || kind == format::kind::PIXEL;
    let takes = match codec {
        format::codec::RGBA8 => matches!(info.supercompression, ktx2::SS_NONE | ktx2::SS_ZLIB),
        _ => ktx2::to_basis(bytes).is_ok(),
    };
    let what = format!(
        "KTX2 {}x{} {} level(s), {}, orientation {}, swizzle {}",
        info.width,
        info.height,
        info.levels.len(),
        match codec {
            format::codec::ETC1S => "ETC1S",
            format::codec::UASTC => "UASTC",
            _ => "RGBA8",
        },
        info.orientation.as_deref().unwrap_or("rd (absent)"),
        info.swizzle.as_deref().unwrap_or("rgba (absent)"),
    );
    let pass = is_identity(o, s)
        && takes
        && (kind == format::kind::COLOR || kind == format::kind::LINEAR)
        && wants_srgb == (color_space == 0);
    if pass {
        let meta = TextureMeta {
            width: info.width,
            height: info.height,
            levels: info.levels.len() as u32,
            kind,
            codec,
            color_space,
            has_alpha: info.has_alpha as u32,
            supercompression: info.supercompression,
        };
        let mut w = BlobWriter::new(
            format::Texture::type_hash(),
            format::Texture::schema_version(),
            stamp,
        );
        w.table("meta", &[meta], row_hash::<TextureMeta>())
            .raw("ktx2", bytes.to_vec(), 16);
        return Ok((w.finish(), vec![format!("{what}: taken as it is")]));
    }
    let rgba = decode(bytes, &info, codec, color_space).map_err(CookError)?;
    let img = super::texture::Image {
        width: info.width,
        height: info.height,
        rgba: apply(&rgba, info.width, info.height, o, s),
    };
    let blob = super::texture::cook(&img, kind, opts, stamp)?;
    Ok((
        blob,
        vec![format!(
            "{what}: decoded, {}{}re-encoded as a {} texture",
            match (o.flip_x, o.flip_y) {
                (false, false) => String::new(),
                (x, y) => format!(
                    "flipped {}to rd, ",
                    match (x, y) {
                        (true, true) => "x and y ",
                        (true, false) => "x ",
                        _ => "y ",
                    }
                ),
            },
            if s == [0, 1, 2, 3] { "" } else { "swizzled, " },
            match kind {
                format::kind::NORMAL => "normal",
                format::kind::LINEAR => "linear",
                format::kind::PIXEL => "pixel",
                _ => "colour",
            }
        )],
    ))
}
