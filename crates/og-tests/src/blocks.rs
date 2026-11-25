//! Solid-block texture encoders for the compressed-format tests: every 4x4
//! block of the texture is one colour, which BC7 (mode 6), BC5, ASTC 4x4
//! (void-extent blocks), ETC2 RGBA8 (individual mode with EAC alpha) and
//! EAC RG11 encode exactly or with a known quantization. [`representable`]
//! gives the colour a format really stores for a requested one, so a
//! reference RGBA8 texture with the same texels can be built.

use og_render::TextureFormat;

/// The colour `format` stores for a solid block of `c`.
pub fn representable(format: TextureFormat, c: [u8; 4]) -> [u8; 4] {
    match format {
        TextureFormat::Rgba8 | TextureFormat::Astc4x4 => c,
        // Mode 6 endpoints are 7 bits plus a shared p-bit (0 here).
        TextureFormat::Bc7 => c.map(|v| v & !1),
        // RGB565 endpoints expanded to 8 bits, opaque alpha.
        TextureFormat::Bc1 => {
            let r = (c[0] >> 3) * 255 / 31;
            let g = (c[1] >> 2) * 255 / 63;
            let b = (c[2] >> 3) * 255 / 31;
            [r, g, b, 255]
        }
        TextureFormat::Bc4 => [c[0], 0, 0, 255],
        TextureFormat::Bc5 => [c[0], c[1], 0, 255],
        // Individual mode: a 4-bit base expanded to 8 bits, plus 2 (table 0, index 0).
        TextureFormat::Etc2Rgba8 => {
            let rgb = |v: u8| ((v >> 4) * 17).saturating_add(2);
            [rgb(c[0]), rgb(c[1]), rgb(c[2]), c[3]]
        }
        TextureFormat::Etc2Rgb8 => {
            let rgb = |v: u8| ((v >> 4) * 17).saturating_add(2);
            [rgb(c[0]), rgb(c[1]), rgb(c[2]), 255]
        }
        // An 11-bit value base * 8 + 6 (multiplier 0, table 0, index 4), as 8 bits.
        TextureFormat::EacR11 => {
            let v = |x: u8| (((x as u32) * 8 + 6) as f32 / 2047.0 * 255.0).round() as u8;
            [v(c[0]), 0, 0, 255]
        }
        TextureFormat::EacRg11 => {
            let v = |x: u8| (((x as u32) * 8 + 6) as f32 / 2047.0 * 255.0).round() as u8;
            [v(c[0]), v(c[1]), 0, 255]
        }
    }
}
