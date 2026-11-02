//! Photoshop files (`.psd`, and `.psb` large documents) as texture
//! sources: the flattened composite the file stores after its layers
//! (Photoshop's "maximize compatibility", on by default), not the layers.
//!
//! Read: RGB, greyscale, CMYK, indexed and bitmap documents at 1, 8, 16
//! or 32 bits a channel; raw, PackBits (RLE) or zip compressed. The
//! composite's transparency is its first extra channel only when the
//! layer section says so (a negative layer count, Photoshop's flag); any
//! other extra channel is a saved selection or spot colour and is not
//! read. 16 bits keep their high byte; 32-bit documents are linear and
//! are encoded to sRGB; CMYK is converted without a colour profile.

use super::texture::Image;

pub(super) mod layers;
pub use layers::{LayerKind, layers};

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let s = self
            .b
            .get(self.at..self.at.checked_add(n).ok_or("PSD: a length overflows")?)
            .ok_or("PSD: the file is truncated")?;
        self.at += n;
        Ok(s)
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().expect("2")))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().expect("4")))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().expect("8")))
    }
}

/// PackBits: `n` 0..=127 copies `n + 1` bytes, -127..=-1 repeats the next
/// byte `1 - n` times, -128 is nothing.
fn packbits(src: &[u8], len: usize, out: &mut Vec<u8>) -> Result<(), String> {
    let start = out.len();
    let mut i = 0;
    while out.len() - start < len && i < src.len() {
        let n = src[i] as i8;
        i += 1;
        if n >= 0 {
            let k = n as usize + 1;
            out.extend_from_slice(
                src.get(i..i + k)
                    .ok_or("PSD: a PackBits run is truncated")?,
            );
            i += k;
        } else if n != -128 {
            let v = *src.get(i).ok_or("PSD: a PackBits run is truncated")?;
            out.extend(std::iter::repeat_n(v, (1 - n as isize) as usize));
            i += 1;
        }
    }
    out.truncate(start + len);
    if out.len() - start < len {
        return Err("PSD: a PackBits row is short".into());
    }
    Ok(())
}

fn srgb(linear: f32) -> u8 {
    let l = linear.clamp(0.0, 1.0);
    let s = if l <= 0.003_130_8 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// The composite of a PSD or PSB as RGBA8.
pub fn composite(bytes: &[u8]) -> Result<Image, String> {
    let mut r = Reader { b: bytes, at: 0 };
    if r.take(4)? != b"8BPS" {
        return Err("PSD: not a Photoshop file".into());
    }
    let large = match r.u16()? {
        1 => false,
        2 => true,
        v => return Err(format!("PSD: version {v} is not PSD (1) or PSB (2)")),
    };
    r.take(6)?;
    let channels = r.u16()? as usize;
    let height = r.u32()? as usize;
    let width = r.u32()? as usize;
    let depth = r.u16()?;
    let mode = r.u16()?;
    if channels == 0 || width == 0 || height == 0 {
        return Err("PSD: an empty image".into());
    }
    let palette = {
        let n = r.u32()? as usize;
        r.take(n)?
    };
    let n = r.u32()? as usize;
    r.take(n)?;
    // The layer section: only its layer count's sign (the composite's transparency).
    let section = if large {
        r.u64()? as usize
    } else {
        r.u32()? as usize
    };
    let mut transparent = false;
    if section > 0 {
        let mut s = Reader {
            b: r.take(section)?,
            at: 0,
        };
        let info = if large {
            s.u64().unwrap_or(0)
        } else {
            s.u32().unwrap_or(0) as u64
        };
        if info >= 2 {
            transparent = (s.u16()? as i16) < 0;
        }
    }
    let compression = r.u16()?;
    let row_bytes = match depth {
        1 => width.div_ceil(8),
        8 => width,
        16 => width * 2,
        32 => width * 4,
        d => return Err(format!("PSD: {d} bits a channel is not read")),
    };
    // How many channels the conversion reads.
    let (needed, what) = match mode {
        0 => (1, "bitmap"),
        1 => (1 + transparent as usize, "greyscale"),
        2 => (1, "indexed"),
        3 => (3 + transparent as usize, "RGB"),
        4 => (4 + transparent as usize, "CMYK"),
        m => {
            let name = match m {
                7 => "multichannel",
                8 => "duotone",
                9 => "Lab",
                _ => "an unknown mode",
            };
            return Err(format!(
                "PSD: a {name} document; save it as RGB, greyscale, CMYK or indexed"
            ));
        }
    };
    if channels < needed {
        return Err(format!("PSD: a {what} document with {channels} channels"));
    }
    let rows = channels * height;
    let mut planes: Vec<u8> = Vec::with_capacity(needed * height * row_bytes);
    match compression {
        0 => planes.extend_from_slice(r.take(needed * height * row_bytes)?),
        1 => {
            let mut counts = Vec::with_capacity(rows);
            for _ in 0..rows {
                counts.push(if large {
                    r.u32()? as usize
                } else {
                    r.u16()? as usize
                });
            }
            for &c in counts.iter().take(needed * height) {
                packbits(r.take(c)?, row_bytes, &mut planes)?;
            }
        }
        2 | 3 => {
            let rest = &bytes[r.at..];
            let all = miniz_oxide::inflate::decompress_to_vec_zlib(rest)
                .map_err(|e| format!("PSD: zip: {e:?}"))?;
            let len = needed * height * row_bytes;
            planes.extend_from_slice(all.get(..len).ok_or("PSD: the zip data is short")?);
            if compression == 3 {
                // Prediction: each row a running sum of 8- or 16-bit deltas.
                match depth {
                    8 => planes.chunks_exact_mut(row_bytes).for_each(|row| {
                        for i in 1..row.len() {
                            row[i] = row[i].wrapping_add(row[i - 1]);
                        }
                    }),
                    16 => planes.chunks_exact_mut(row_bytes).for_each(|row| {
                        for i in 1..width {
                            let p = u16::from_be_bytes([row[2 * i - 2], row[2 * i - 1]]);
                            let v = u16::from_be_bytes([row[2 * i], row[2 * i + 1]]);
                            row[2 * i..2 * i + 2].copy_from_slice(&p.wrapping_add(v).to_be_bytes());
                        }
                    }),
                    _ => return Err("PSD: zip prediction at this depth is not read".into()),
                }
            }
        }
        c => return Err(format!("PSD: compression {c} is not read")),
    }
    let plane = height * row_bytes;
    // Channel `c` of pixel `p` as 0..=255.
    let sample = |c: usize, p: usize| -> u8 {
        let base = c * plane + (p / width) * row_bytes;
        let x = p % width;
        match depth {
            1 => {
                let bit = planes[base + x / 8] >> (7 - x % 8) & 1;
                if bit == 1 { 0 } else { 255 }
            }
            8 => planes[base + x],
            16 => planes[base + 2 * x],
            _ => {
                let v = f32::from_be_bytes(
                    planes[base + 4 * x..base + 4 * x + 4]
                        .try_into()
                        .expect("4"),
                );
                srgb(v)
            }
        }
    };
    let mut rgba = Vec::with_capacity(width * height * 4);
    for p in 0..width * height {
        let px = match mode {
            0 => {
                let v = sample(0, p);
                [v, v, v, 255]
            }
            1 => {
                let v = sample(0, p);
                [v, v, v, if transparent { sample(1, p) } else { 255 }]
            }
            2 => {
                let i = sample(0, p) as usize;
                let at = |k: usize| palette.get(k * 256 + i).copied().unwrap_or(0);
                [at(0), at(1), at(2), 255]
            }
            3 => [
                sample(0, p),
                sample(1, p),
                sample(2, p),
                if transparent { sample(3, p) } else { 255 },
            ],
            _ => {
                // Photoshop stores CMYK inverted: 255 is no ink.
                let k = sample(3, p) as u32;
                let c = |v: u8| (v as u32 * k / 255) as u8;
                [
                    c(sample(0, p)),
                    c(sample(1, p)),
                    c(sample(2, p)),
                    if transparent { sample(4, p) } else { 255 },
                ]
            }
        };
        rgba.extend_from_slice(&px);
    }
    Ok(Image {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 3 x 2 RGB document with transparency, written as Photoshop
    /// lays it out: planar channels, PackBits rows.
    fn psd(transparent: bool, rle: bool) -> Vec<u8> {
        let (w, h) = (3usize, 2usize);
        let planes: [[u8; 6]; 4] = [
            [10, 20, 30, 40, 50, 60],
            [1, 2, 3, 4, 5, 6],
            [200, 200, 200, 100, 100, 100],
            [255, 128, 0, 255, 255, 255],
        ];
        let mut b = Vec::new();
        b.extend_from_slice(b"8BPS");
        b.extend(1u16.to_be_bytes());
        b.extend([0; 6]);
        b.extend(4u16.to_be_bytes());
        b.extend((h as u32).to_be_bytes());
        b.extend((w as u32).to_be_bytes());
        b.extend(8u16.to_be_bytes());
        b.extend(3u16.to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(0u32.to_be_bytes());
        // Layer section: a layer info block holding only its count.
        let count: i16 = if transparent { -1 } else { 1 };
        b.extend(8u32.to_be_bytes());
        b.extend(2u32.to_be_bytes());
        b.extend(count.to_be_bytes());
        b.extend([0, 0]);
        b.extend((rle as u16).to_be_bytes());
        if rle {
            // Each row as one literal run: [n - 1, bytes...].
            for _ in 0..4 * h {
                b.extend(4u16.to_be_bytes());
            }
            for p in &planes {
                for row in p.chunks(w) {
                    b.push((w - 1) as u8);
                    b.extend_from_slice(row);
                }
            }
        } else {
            for p in &planes {
                b.extend_from_slice(p);
            }
        }
        b
    }

    #[test]
    fn the_composite_is_read_with_transparency_only_when_flagged() {
        for rle in [false, true] {
            let img = composite(&psd(true, rle)).unwrap();
            assert_eq!((img.width, img.height), (3, 2));
            assert_eq!(&img.rgba[..8], &[10, 1, 200, 255, 20, 2, 200, 128]);
            assert_eq!(&img.rgba[20..24], &[60, 6, 100, 255]);
            // A positive layer count: the fourth channel is a selection.
            let img = composite(&psd(false, rle)).unwrap();
            assert_eq!(&img.rgba[4..8], &[20, 2, 200, 255]);
        }
    }

    #[test]
    fn packbits_repeats_and_copies() {
        let mut out = Vec::new();
        packbits(&[0xFE, 7, 1, 1, 2, 0x80], 5, &mut out).unwrap();
        assert_eq!(out, [7, 7, 7, 1, 2]);
    }
}
