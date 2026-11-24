//! Uncooked HDR images for environments loaded at run time
//! (`og.load.environmentData`): a Radiance `.hdr` read a row at a time, so
//! a loader can spread the decode over frames on a host without threads,
//! and an OpenEXR file's size from its header, checked before a native
//! decoder reads it. Untrusted input: every length is checked and a broken
//! file is refused with a reason, never read past its end.

/// Whether `bytes` start as a Radiance `.hdr` does.
pub fn is_hdr(bytes: &[u8]) -> bool {
    bytes.starts_with(b"#?RADIANCE") || bytes.starts_with(b"#?RGBE")
}

/// Whether `bytes` start as an OpenEXR file does.
pub fn is_exr(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x76, 0x2f, 0x31, 0x01])
}

/// A Radiance `.hdr`'s width and height from its header, without
/// decoding it.
pub fn hdr_size(bytes: &[u8]) -> Result<(u32, u32), String> {
    header(bytes).map(|(w, h, _)| (w, h))
}

/// The header: its format (32-bit RLE RGBE), its size (the usual `-Y
/// height +X width`, rows from the top) and where the pixels start.
fn header(bytes: &[u8]) -> Result<(u32, u32, usize), String> {
    if !is_hdr(bytes) {
        return Err("not a Radiance .hdr (it starts with #?RADIANCE)".into());
    }
    let mut at = 0;
    let line = |at: &mut usize| -> Result<String, String> {
        let rest = &bytes[*at..];
        let end = rest
            .iter()
            .position(|&b| b == b'\n')
            .filter(|&e| e <= 4096)
            .ok_or("a .hdr whose header does not end (the file is cut short)")?;
        *at += end + 1;
        Ok(String::from_utf8_lossy(&rest[..end]).trim().to_string())
    };
    line(&mut at)?;
    loop {
        let l = line(&mut at)?;
        if l.is_empty() {
            break;
        }
        if let Some(f) = l.strip_prefix("FORMAT=")
            && f != "32-bit_rle_rgbe"
        {
            return Err(format!(
                "a .hdr in FORMAT={f}: only 32-bit_rle_rgbe (RGB) is read; save it as RGB"
            ));
        }
    }
    let size = line(&mut at)?;
    let parts: Vec<&str> = size.split_whitespace().collect();
    let (height, width) = match parts.as_slice() {
        ["-Y", h, "+X", w] => (h.parse::<u32>(), w.parse::<u32>()),
        _ => {
            return Err(format!(
                "a .hdr laid out as \"{size}\": only \"-Y <height> +X <width>\" (rows from the top) is read"
            ));
        }
    };
    let (Ok(height), Ok(width)) = (height, width) else {
        return Err(format!("a .hdr whose size line \"{size}\" does not parse"));
    };
    if width == 0 || height == 0 {
        return Err(format!("a .hdr of {width}x{height} pixels holds nothing"));
    }
    Ok((width, height, at))
}

/// A Radiance `.hdr` decoded a row at a time: rows from the top, linear
/// RGB.
pub struct HdrRows {
    bytes: Vec<u8>,
    at: usize,
    width: u32,
    height: u32,
    row: u32,
    line: Vec<[u8; 4]>,
}

impl HdrRows {
    /// Reads the header ([`hdr_size`]).
    pub fn new(bytes: Vec<u8>) -> Result<HdrRows, String> {
        let (width, height, at) = header(&bytes)?;
        Ok(HdrRows {
            bytes,
            at,
            width,
            height,
            row: 0,
            line: vec![[0; 4]; width as usize],
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// The rows read so far.
    pub fn row(&self) -> u32 {
        self.row
    }

    fn byte(&mut self) -> Result<u8, String> {
        let b = *self
            .bytes
            .get(self.at)
            .ok_or_else(|| format!("a .hdr cut short in row {}", self.row))?;
        self.at += 1;
        Ok(b)
    }

    /// The next row into `rgb` (`width * 3` floats); false once every row
    /// is read.
    pub fn next_row(&mut self, rgb: &mut Vec<f32>) -> Result<bool, String> {
        if self.row == self.height {
            return Ok(false);
        }
        let w = self.width as usize;
        let rest = &self.bytes[self.at.min(self.bytes.len())..];
        let new_rle = (8..0x8000).contains(&w)
            && rest.len() >= 4
            && rest[0] == 2
            && rest[1] == 2
            && ((rest[2] as usize) << 8 | rest[3] as usize) == w;
        if new_rle {
            self.at += 4;
            // Each channel's run-length bytes in turn.
            for c in 0..4 {
                let mut x = 0;
                while x < w {
                    let n = self.byte()? as usize;
                    if n > 128 {
                        let run = n - 128;
                        if x + run > w {
                            return Err(format!(
                                "a .hdr whose row {} runs past its width",
                                self.row
                            ));
                        }
                        let v = self.byte()?;
                        for p in &mut self.line[x..x + run] {
                            p[c] = v;
                        }
                        x += run;
                    } else {
                        if n == 0 || x + n > w {
                            return Err(format!(
                                "a .hdr whose row {} runs past its width",
                                self.row
                            ));
                        }
                        for k in 0..n {
                            let v = self.byte()?;
                            self.line[x + k][c] = v;
                        }
                        x += n;
                    }
                }
            }
        } else {
            // Flat pixels, with the old run-length (1, 1, 1, count) repeats.
            let mut x = 0;
            let mut shift = 0;
            while x < w {
                let p = [self.byte()?, self.byte()?, self.byte()?, self.byte()?];
                if p[..3] == [1, 1, 1] && x > 0 {
                    let run = (p[3] as usize) << shift;
                    if x + run > w {
                        return Err(format!("a .hdr whose row {} runs past its width", self.row));
                    }
                    let prev = self.line[x - 1];
                    self.line[x..x + run].fill(prev);
                    x += run;
                    shift += 8;
                } else {
                    self.line[x] = p;
                    x += 1;
                    shift = 0;
                }
            }
        }
        rgb.clear();
        rgb.reserve(w * 3);
        for p in &self.line {
            if p[3] == 0 {
                rgb.extend_from_slice(&[0.0; 3]);
            } else {
                let f = 2f32.powi(p[3] as i32 - 136);
                rgb.extend(p[..3].iter().map(|&v| (v as f32 + 0.5) * f));
            }
        }
        self.row += 1;
        Ok(true)
    }
}

/// An OpenEXR file's size from its header's `dataWindow`, without decoding
/// it.
pub fn exr_size(bytes: &[u8]) -> Result<(u32, u32), String> {
    if !is_exr(bytes) {
        return Err("not an OpenEXR file".into());
    }
    let mut at = 8;
    let cstr = |at: &mut usize| -> Result<&[u8], String> {
        let rest = bytes.get(*at..).unwrap_or(&[]);
        let end = rest
            .iter()
            .take(256)
            .position(|&b| b == 0)
            .ok_or("an .exr whose header does not parse (the file is cut short)")?;
        *at += end + 1;
        Ok(&rest[..end])
    };
    loop {
        let name = cstr(&mut at)?;
        if name.is_empty() {
            return Err("an .exr without a dataWindow in its header".into());
        }
        let kind = cstr(&mut at)?;
        let len = bytes
            .get(at..at + 4)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .filter(|&n| n >= 0)
            .ok_or("an .exr whose header does not parse (the file is cut short)")?
            as usize;
        at += 4;
        let value = bytes
            .get(at..at + len)
            .ok_or("an .exr whose header does not parse (the file is cut short)")?;
        at += len;
        if name == b"dataWindow" && kind == b"box2i" && len == 16 {
            let i = |k: usize| {
                i32::from_le_bytes([value[k], value[k + 1], value[k + 2], value[k + 3]]) as i64
            };
            let (w, h) = (i(8) - i(0) + 1, i(12) - i(4) + 1);
            if w <= 0 || h <= 0 || w > u32::MAX as i64 || h > u32::MAX as i64 {
                return Err(format!("an .exr whose dataWindow is {w}x{h}"));
            }
            return Ok((w as u32, h as u32));
        }
    }
}
