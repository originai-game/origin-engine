//! PNG decoded to RGBA8.

use og_shape::tools::Image;

use crate::texture::*;

pub(crate) fn be32(b: &[u8], at: usize) -> Result<u32, TextureError> {
    b.get(at..at + 4)
        .map(|x| u32::from_be_bytes([x[0], x[1], x[2], x[3]]))
        .ok_or_else(|| bad("PNG is cut short"))
}

pub(crate) fn decode_png(b: &[u8]) -> Result<Image, TextureError> {
    let mut at = 8;
    let mut header: Option<(u32, u32, u8, u8, u8)> = None;
    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut trns: Vec<u8> = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    while at + 8 <= b.len() {
        let len = be32(b, at)? as usize;
        let kind = &b[at + 4..at + 8];
        let body = b
            .get(at + 8..at + 8 + len)
            .ok_or_else(|| bad("a PNG chunk runs past the end"))?;
        match kind {
            b"IHDR" if body.len() >= 13 => {
                if body[10] != 0 || body[11] != 0 {
                    return Err(bad("PNG compression or filter method"));
                }
                header = Some((be32(body, 0)?, be32(body, 4)?, body[8], body[9], body[12]));
            }
            b"PLTE" => palette = body.as_chunks::<3>().0.to_vec(),
            b"tRNS" => trns = body.to_vec(),
            b"IDAT" => data.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + len;
    }
    let (w, h, depth, kind, interlace) = header.ok_or_else(|| bad("PNG has no IHDR"))?;
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 1 << 28 {
        return Err(bad("PNG size"));
    }
    let channels: usize = match kind {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return Err(bad("PNG colour type")),
    };
    let ok_depth = match kind {
        0 => [1, 2, 4, 8, 16].contains(&depth),
        3 => [1, 2, 4, 8].contains(&depth),
        _ => [8, 16].contains(&depth),
    };
    if !ok_depth {
        return Err(bad("PNG bit depth"));
    }
    let bits = channels * depth as usize;
    let bpp = bits.div_ceil(8).max(1);
    // Passes: (x0, y0, dx, dy); one for a plain image, Adam7's seven else.
    let passes: &[(usize, usize, usize, usize)] = if interlace == 0 {
        &[(0, 0, 1, 1)]
    } else {
        &[
            (0, 0, 8, 8),
            (4, 0, 8, 8),
            (0, 4, 4, 8),
            (2, 0, 4, 4),
            (0, 2, 2, 4),
            (1, 0, 2, 2),
            (0, 1, 1, 2),
        ]
    };
    let (w, h) = (w as usize, h as usize);
    let mut limit = 0usize;
    for &(x0, y0, dx, dy) in passes {
        let (pw, ph) = ((w + dx - 1 - x0) / dx, (h + dy - 1 - y0) / dy);
        if pw > 0 && ph > 0 {
            limit += ph * (1 + (pw * bits).div_ceil(8));
        }
    }
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(&data, limit)
        .map_err(|_| bad("PNG data does not inflate"))?;
    let mut rgba = vec![0u8; w * h * 4];
    let sample = |row: &[u8], x: usize, c: usize| -> u16 {
        let i = x * channels + c;
        match depth {
            16 => u16::from_be_bytes([row[i * 2], row[i * 2 + 1]]),
            8 => u16::from(row[i]),
            d => {
                let d = d as usize;
                let bit = i * d;
                let byte = row[bit / 8];
                let shift = 8 - d - bit % 8;
                u16::from((byte >> shift) & ((1u8 << d) - 1))
            }
        }
    };
    let to8 = |v: u16| -> u8 {
        match depth {
            16 => ((u32::from(v) * 255 + 32767) / 65535) as u8,
            8 => v as u8,
            d => (u32::from(v) * 255 / ((1 << d) - 1)) as u8,
        }
    };
    let key = |c: usize| -> Option<u16> {
        trns.get(c * 2..c * 2 + 2)
            .map(|x| u16::from_be_bytes([x[0], x[1]]))
    };
    let mut at = 0;
    for &(x0, y0, dx, dy) in passes {
        let (pw, ph) = ((w + dx - 1 - x0) / dx, (h + dy - 1 - y0) / dy);
        if pw == 0 || ph == 0 {
            continue;
        }
        let stride = (pw * bits).div_ceil(8);
        let mut prev = vec![0u8; stride];
        for py in 0..ph {
            let filter = *raw.get(at).ok_or_else(|| bad("PNG data is short"))?;
            let mut row = raw
                .get(at + 1..at + 1 + stride)
                .ok_or_else(|| bad("PNG data is short"))?
                .to_vec();
            at += 1 + stride;
            for i in 0..stride {
                let a = if i >= bpp { row[i - bpp] } else { 0 };
                let up = prev[i];
                let c = if i >= bpp { prev[i - bpp] } else { 0 };
                row[i] = row[i].wrapping_add(match filter {
                    0 => 0,
                    1 => a,
                    2 => up,
                    3 => ((u16::from(a) + u16::from(up)) / 2) as u8,
                    4 => {
                        let p = i16::from(a) + i16::from(up) - i16::from(c);
                        let (pa, pb, pc) = (
                            (p - i16::from(a)).abs(),
                            (p - i16::from(up)).abs(),
                            (p - i16::from(c)).abs(),
                        );
                        if pa <= pb && pa <= pc {
                            a
                        } else if pb <= pc {
                            up
                        } else {
                            c
                        }
                    }
                    _ => return Err(bad("PNG filter type")),
                });
            }
            let y = y0 + py * dy;
            for px in 0..pw {
                let x = x0 + px * dx;
                let o = (y * w + x) * 4;
                let px_rgba: [u8; 4] = match kind {
                    0 => {
                        let v = sample(&row, px, 0);
                        let g = to8(v);
                        [g, g, g, if key(0) == Some(v) { 0 } else { 255 }]
                    }
                    2 => {
                        let v = [0, 1, 2].map(|c| sample(&row, px, c));
                        let transparent = (0..3).all(|c| key(c) == Some(v[c]));
                        [
                            to8(v[0]),
                            to8(v[1]),
                            to8(v[2]),
                            if transparent { 0 } else { 255 },
                        ]
                    }
                    3 => {
                        let i = sample(&row, px, 0) as usize;
                        let p = palette.get(i).ok_or_else(|| bad("PNG palette index"))?;
                        [p[0], p[1], p[2], trns.get(i).copied().unwrap_or(255)]
                    }
                    4 => {
                        let g = to8(sample(&row, px, 0));
                        [g, g, g, to8(sample(&row, px, 1))]
                    }
                    _ => [0, 1, 2, 3].map(|c| to8(sample(&row, px, c))),
                };
                rgba[o..o + 4].copy_from_slice(&px_rgba);
            }
            prev = row;
        }
    }
    Ok(Image {
        width: w as u32,
        height: h as u32,
        rgba,
    })
}
