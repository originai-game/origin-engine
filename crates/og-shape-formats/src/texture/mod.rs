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
//! `og_shape::math::trig::pow`) is rounded to integers before use.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Image {
        let mut rgba = Vec::new();
        for y in 0..h {
            for x in 0..w {
                rgba.extend([(x * 255 / w) as u8, (y * 255 / h) as u8, 128, 255]);
            }
        }
        Image {
            width: w,
            height: h,
            rgba,
        }
    }

    #[test]
    fn png_round_trips_and_shrinks_by_area() {
        let img = gradient(10, 6);
        assert_eq!(decode(&crate::png::write::png(&img)).unwrap(), img);
        // Halving a 2×2 checker of 0 and 255: plain gives the mean byte,
        // colour the byte of the mean light (188), not 128.
        let checker = Image {
            width: 2,
            height: 2,
            rgba: [0u8, 255, 255, 0]
                .iter()
                .flat_map(|&v| [v, v, v, 255])
                .collect(),
        };
        let t = Texture::from_bytes("c.jpg", crate::png::write::png(&checker));
        for (color, want) in [(false, 128), (true, 188)] {
            let small = downsize(&t, &Downsize { max: 1, color }).unwrap().unwrap();
            assert_eq!(
                (small.name.as_str(), small.mime.as_str()),
                ("c.png", "image/png")
            );
            let d = decode(&small.bytes).unwrap();
            assert_eq!(d.rgba, vec![want, want, want, 255]);
        }
        // Within the limit: kept as it is.
        assert_eq!(
            downsize(
                &t,
                &Downsize {
                    max: 2,
                    color: true
                }
            )
            .unwrap(),
            None
        );
        // 10 × 6 to 5 × 3: the aspect kept, each texel the mean of four.
        let small = resize(&img, 5, 3, false);
        // Red 0 and 25 in each row: 12.5, rounded up.
        assert_eq!(small.rgba[0], 13);
    }

    #[test]
    fn png_bit_depths_and_palettes_decode() {
        // 1-bit grey: 0b1010_0000, 3 wide, 1 high.
        let mut f = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let chunk = |out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]| {
            out.extend((data.len() as u32).to_be_bytes());
            out.extend(kind);
            out.extend(data);
            out.extend([0, 0, 0, 0]);
        };
        chunk(&mut f, b"IHDR", &[0, 0, 0, 3, 0, 0, 0, 1, 1, 0, 0, 0, 0]);
        let z = miniz_oxide::deflate::compress_to_vec_zlib(&[0, 0b1010_0000], 6);
        chunk(&mut f, b"IDAT", &z);
        chunk(&mut f, b"IEND", &[]);
        let d = decode(&f).unwrap();
        assert_eq!(
            d.rgba,
            vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255]
        );
    }
}

#[cfg(test)]
mod webp_tests {
    use super::*;

    /// A WebP image decodes to the texels it was made from (a lossless
    /// one, with alpha and without); a broken file is refused with the
    /// reason.
    #[test]
    fn a_webp_image_is_decoded() {
        let (w, h) = (23u32, 17u32);
        let rgba: Vec<u8> = (0..w * h * 4)
            .map(|i| ((i * 37 + 11) % 256) as u8)
            .collect();
        let pack = |data: &[u8], kind| {
            let mut out = Vec::new();
            image_webp::WebPEncoder::new(&mut out)
                .encode(data, w, h, kind)
                .unwrap();
            out
        };
        let with_alpha = pack(&rgba, image_webp::ColorType::Rgba8);
        // The same file is kept for the gate.
        assert_eq!(
            with_alpha,
            include_bytes!("../../../og-shape/tests/data/meshes/pattern.webp")
        );
        let im = decode(&with_alpha).unwrap();
        assert_eq!((im.width, im.height), (w, h));
        assert_eq!(im.rgba, rgba);
        let rgb: Vec<u8> = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect();
        let opaque = decode(&pack(&rgb, image_webp::ColorType::Rgb8)).unwrap();
        for (p, q) in opaque
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(rgba.as_chunks::<4>().0)
        {
            assert_eq!(*p, [q[0], q[1], q[2], 255]);
        }
        for cut in [12, 20, with_alpha.len() / 2] {
            assert!(decode(&with_alpha[..cut]).is_err(), "cut at {cut}");
        }
    }

    /// A map written as WebP is every texel it was, in fewer bytes than
    /// its PNG; a WebP that is shrunk stays WebP; and an animation
    /// gives its first frame.
    #[test]
    fn a_webp_image_is_written() {
        let (w, h) = (64u32, 48u32);
        let rgba: Vec<u8> = (0..w * h)
            .flat_map(|i| {
                let (x, y) = (i % w, i / w);
                [
                    (x * 4) as u8,
                    (y * 5) as u8,
                    ((x / 8 + y / 8) % 2 * 200) as u8,
                    255,
                ]
            })
            .collect();
        let image = Image {
            width: w,
            height: h,
            rgba: rgba.clone(),
        };
        let png = Texture::from_bytes("map.png", crate::png::write::png(&image));
        let packed = to_webp(&png).unwrap().expect("smaller than its PNG");
        assert_eq!(
            (packed.name.as_str(), packed.mime.as_str()),
            ("map.webp", "image/webp")
        );
        assert!(packed.bytes.len() < png.bytes.len());
        assert_eq!(decode(&packed.bytes).unwrap(), image);
        assert!(to_webp(&packed).unwrap().is_none());
        let small = downsize(
            &packed,
            &Downsize {
                max: 16,
                color: true,
            },
        )
        .unwrap()
        .expect("64 is over 16");
        assert_eq!(small.mime, "image/webp");
        assert_eq!(decode(&small.bytes).unwrap().width, 16);
        // An animation of two frames, put together by hand from two
        // stills' VP8L chunks: the first frame is what is read.
        let still = |color: [u8; 4]| {
            let mut out = Vec::new();
            image_webp::WebPEncoder::new(&mut out)
                .encode(&color.repeat(8 * 8), 8, 8, image_webp::ColorType::Rgba8)
                .unwrap();
            out[12..].to_vec()
        };
        let u24 = |v: u32| v.to_le_bytes()[..3].to_vec();
        let chunk = |tag: &[u8; 4], body: Vec<u8>| {
            let mut out = tag.to_vec();
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&body);
            if body.len() % 2 == 1 {
                out.push(0);
            }
            out
        };
        let frame = |color: [u8; 4]| {
            let mut body = [u24(0), u24(0), u24(7), u24(7), u24(100)].concat();
            // Not blended with what was there.
            body.push(2);
            body.extend(still(color));
            chunk(b"ANMF", body)
        };
        let mut body = b"WEBP".to_vec();
        body.extend(chunk(
            b"VP8X",
            [vec![0x12, 0, 0, 0], u24(7), u24(7)].concat(),
        ));
        body.extend(chunk(b"ANIM", vec![0, 0, 0, 0, 0, 0]));
        body.extend(frame([200, 10, 20, 255]));
        body.extend(frame([0, 250, 0, 255]));
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(body.len() as u32).to_le_bytes());
        file.extend(body);
        let first = decode(&file).unwrap();
        assert_eq!((first.width, first.height), (8, 8));
        assert_eq!(&first.rgba[..4], &[200, 10, 20, 255]);
    }
}
