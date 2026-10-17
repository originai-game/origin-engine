//! Solid-block texture encoders for the compressed-format tests: every 4x4
//! block of the texture is one colour, which BC7 (mode 6), BC5, ASTC 4x4
//! (void-extent blocks), ETC2 RGBA8 (individual mode with EAC alpha) and
//! EAC RG11 encode exactly or with a known quantization. [`representable`]
//! gives the colour a format really stores for a requested one, so a
//! reference RGBA8 texture with the same texels can be built.

use og_render::{ColorSpace, TextureDesc, TextureFormat, TextureUpload};

/// The colour `format` stores for a solid block of `c`.
pub fn representable(format: TextureFormat, c: [u8; 4]) -> [u8; 4] {
    match format {
        TextureFormat::Rgba8 | TextureFormat::Astc4x4 => c,
        // Mode 6 endpoints are 7 bits plus a shared p-bit (0 here).
        TextureFormat::Bc7 => c.map(|v| v & !1),
        TextureFormat::Bc5 => [c[0], c[1], 0, 255],
        // Individual mode: a 4-bit base expanded to 8 bits, plus 2 (table 0, index 0).
        TextureFormat::Etc2Rgba8 => {
            let rgb = |v: u8| ((v >> 4) * 17).saturating_add(2);
            [rgb(c[0]), rgb(c[1]), rgb(c[2]), c[3]]
        }
        // An 11-bit value base * 8 + 6 (multiplier 0, table 0, index 4), as 8 bits.
        TextureFormat::EacRg11 => {
            let v = |x: u8| (((x as u32) * 8 + 6) as f32 / 2047.0 * 255.0).round() as u8;
            [v(c[0]), v(c[1]), 0, 255]
        }
    }
}

/// One 16-byte (or for RGBA8 one texel's 4-byte) encoding of a solid block of `c`.
pub fn solid_block(format: TextureFormat, c: [u8; 4]) -> Vec<u8> {
    match format {
        TextureFormat::Rgba8 => c.to_vec(),
        TextureFormat::Bc7 => bc7_mode6(c),
        TextureFormat::Bc5 => {
            let mut b = vec![0u8; 16];
            b[0] = c[0];
            b[1] = c[0];
            b[8] = c[1];
            b[9] = c[1];
            b
        }
        TextureFormat::Astc4x4 => {
            // Void extent, LDR, no extent coordinates; then RGBA as UNORM16.
            let mut b = 0xFFFF_FFFF_FFFF_FDFCu64.to_le_bytes().to_vec();
            for v in c {
                b.extend_from_slice(&(v as u16 * 257).to_le_bytes());
            }
            b
        }
        TextureFormat::Etc2Rgba8 => {
            // EAC alpha: base = alpha, multiplier 0 (exact), indices 0.
            let mut b = vec![c[3], 0, 0, 0, 0, 0, 0, 0];
            // ETC1 individual mode: both sub-blocks the same 4-bit base,
            // table 0, no flip, every pixel index 0 (+2).
            let (r, g, bl) = (c[0] >> 4, c[1] >> 4, c[2] >> 4);
            b.extend_from_slice(&[r << 4 | r, g << 4 | g, bl << 4 | bl, 0, 0, 0, 0, 0]);
            b
        }
        TextureFormat::EacRg11 => {
            // Per channel: base, multiplier 0 (1/8), table 0, every index 4 (+2).
            let channel = |v: u8| {
                let indices: u64 = (0..16).fold(0, |acc, _| acc << 3 | 4);
                let word = (v as u64) << 56 | indices;
                word.to_be_bytes()
            };
            let mut b = channel(c[0]).to_vec();
            b.extend_from_slice(&channel(c[1]));
            b
        }
    }
}

/// BC7 mode 6 with both endpoints `c` (p-bits 0) and every index 0.
fn bc7_mode6(c: [u8; 4]) -> Vec<u8> {
    let mut bits: u128 = 1 << 6; // mode 6
    let mut at = 7;
    let mut put = |v: u128, n: u32| {
        bits |= v << at;
        at += n;
    };
    for ch in c {
        let e = (ch >> 1) as u128;
        put(e, 7);
        put(e, 7);
    }
    // p-bits and indices stay zero.
    bits.to_le_bytes().to_vec()
}

/// A texture whose 4x4 blocks are the colours `grid[by][bx]` at level 0,
/// with a full mip chain (each level's block the average of the blocks it
/// covers, made representable), encoded in `format`, and the same texels as
/// an RGBA8 upload: `(encoded, rgba8_reference)`.
pub fn block_texture(
    format: TextureFormat,
    color_space: ColorSpace,
    grid: &[Vec<[u8; 4]>],
) -> (TextureUpload, TextureUpload) {
    let blocks_y = grid.len() as u32;
    let blocks_x = grid[0].len() as u32;
    let (width, height) = (blocks_x * 4, blocks_y * 4);
    let levels = TextureDesc::full_chain(width, height);
    // Texel colours of the current level (representable), level by level.
    let mut texels: Vec<Vec<[u8; 4]>> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| representable(format, grid[(y / 4) as usize][(x / 4) as usize]))
                .collect()
        })
        .collect();
    let (mut encoded, mut reference) = (Vec::new(), Vec::new());
    for level in 0..levels {
        let (w, h) = ((width >> level).max(1), (height >> level).max(1));
        if level > 0 {
            // Box filter the previous level, then hold each block's colour.
            let prev = texels;
            let (pw, ph) = (prev[0].len() as u32, prev.len() as u32);
            let mut next = vec![vec![[0u8; 4]; w as usize]; h as usize];
            for (y, row) in next.iter_mut().enumerate() {
                for (x, px) in row.iter_mut().enumerate() {
                    let mut sum = [0u32; 4];
                    let mut n = 0;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (sx, sy) = (
                            (x as u32 * 2 + dx).min(pw - 1),
                            (y as u32 * 2 + dy).min(ph - 1),
                        );
                        let p = prev[sy as usize][sx as usize];
                        for c in 0..4 {
                            sum[c] += p[c] as u32;
                        }
                        n += 1;
                    }
                    *px = sum.map(|s| ((s + n / 2) / n) as u8);
                }
            }
            texels = next;
        }
        // Solid blocks: every block of this level takes its top-left texel's
        // colour, so the level is exactly representable.
        let d = format.block_dim().max(4);
        let (bx, by) = (w.div_ceil(d), h.div_ceil(d));
        let mut bytes = Vec::new();
        for y in 0..by {
            for x in 0..bx {
                let c = representable(format, texels[(y * d) as usize][(x * d) as usize]);
                for ty in 0..d {
                    for tx in 0..d {
                        let (px, py) = (x * d + tx, y * d + ty);
                        if px < w && py < h {
                            texels[py as usize][px as usize] = c;
                        }
                    }
                }
                bytes.extend(solid_block(format, c));
            }
        }
        if format == TextureFormat::Rgba8 {
            bytes = texels.iter().flatten().flatten().copied().collect();
        }
        encoded.push(bytes);
        reference.push(
            texels
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect::<Vec<u8>>(),
        );
    }
    let desc = TextureDesc {
        width,
        height,
        format,
        mip_levels: levels,
        color_space,
    };
    (
        TextureUpload {
            desc,
            levels: encoded,
        },
        TextureUpload {
            desc: TextureDesc {
                format: TextureFormat::Rgba8,
                ..desc
            },
            levels: reference,
        },
    )
}
