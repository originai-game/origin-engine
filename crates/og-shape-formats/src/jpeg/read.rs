//! JPEG decoded to RGBA8: baseline and progressive, Huffman, IDCT and chroma upsampling.

use og_shape::tools::Image;

use crate::texture::*;

pub(crate) const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

/// A Huffman table as the JPEG specification decodes it (F.2.2.3): per
/// code length, the largest code, and where its symbols start.
#[derive(Clone, Default)]
pub(crate) struct Huffman {
    pub(crate) maxcode: [i32; 17],
    pub(crate) valptr: [i32; 17],
    pub(crate) mincode: [i32; 17],
    pub(crate) symbols: Vec<u8>,
}

impl Huffman {
    pub(crate) fn new(counts: &[u8; 16], symbols: Vec<u8>) -> Huffman {
        let mut h = Huffman {
            maxcode: [-1; 17],
            symbols,
            ..Default::default()
        };
        let (mut code, mut k) = (0i32, 0i32);
        for l in 1..=16 {
            let n = i32::from(counts[l - 1]);
            if n > 0 {
                h.valptr[l] = k;
                h.mincode[l] = code;
                code += n;
                k += n;
                h.maxcode[l] = code - 1;
            }
            code <<= 1;
        }
        h
    }
}

/// Bits of one entropy-coded segment (stuffing removed).
pub(crate) struct Bits<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) at: usize,
    pub(crate) acc: u32,
    pub(crate) n: u32,
}

impl Bits<'_> {
    pub(crate) fn bit(&mut self) -> u32 {
        if self.n == 0 {
            // Past the end: zeros, as decoders pad.
            self.acc = u32::from(self.data.get(self.at).copied().unwrap_or(0));
            self.at += 1;
            self.n = 8;
        }
        self.n -= 1;
        (self.acc >> self.n) & 1
    }

    pub(crate) fn bits(&mut self, n: u32) -> u32 {
        (0..n).fold(0, |v, _| (v << 1) | self.bit())
    }

    pub(crate) fn decode(&mut self, h: &Huffman) -> Result<u8, TextureError> {
        let mut code = 0i32;
        for l in 1..=16 {
            code = (code << 1) | self.bit() as i32;
            if h.maxcode[l] >= 0 && code <= h.maxcode[l] {
                let i = h.valptr[l] + code - h.mincode[l];
                return h
                    .symbols
                    .get(i as usize)
                    .copied()
                    .ok_or_else(|| bad("JPEG Huffman code"));
            }
        }
        Err(bad("JPEG Huffman code"))
    }

    /// A coefficient of `s` bits, sign-extended (F.2.2.1).
    pub(crate) fn extend(&mut self, s: u32) -> i32 {
        if s == 0 {
            return 0;
        }
        let v = self.bits(s) as i32;
        if v < 1 << (s - 1) {
            v - (1 << s) + 1
        } else {
            v
        }
    }
}

pub(crate) struct Component {
    pub(crate) id: u8,
    pub(crate) h: usize,
    pub(crate) v: usize,
    pub(crate) tq: usize,
    /// Blocks across and down, padded to whole MCUs.
    pub(crate) bw: usize,
    pub(crate) bh: usize,
    /// Decoded samples, `bw·8` × `bh·8`.
    pub(crate) samples: Vec<u8>,
    /// A progressive frame's coefficients as its scans build them, 64 a
    /// block (not yet scaled by the quantization table); empty otherwise.
    pub(crate) coef: Vec<i32>,
}

/// The inverse DCT basis, × 2¹³: `T[x][u] = c(u)/2 · cos((2x+1)uπ/16)`.
pub(crate) fn idct_table() -> [[i64; 8]; 8] {
    let pi = core::f64::consts::PI;
    core::array::from_fn(|x| {
        core::array::from_fn(|u| {
            let c = if u == 0 {
                core::f64::consts::FRAC_1_SQRT_2
            } else {
                1.0
            };
            let v = c / 2.0 * og_shape::math::trig::cos((2 * x + 1) as f64 * u as f64 * pi / 16.0);
            (v * 8192.0).round() as i64
        })
    })
}

/// The 8×8 block of dequantized coefficients (natural order) to samples.
pub(crate) fn idct(t: &[[i64; 8]; 8], f: &[i32; 64], out: &mut [u8], stride: usize) {
    // Rows: tmp[v][x] = Σu T[x][u]·F[v][u], kept × 2³.
    let mut tmp = [[0i64; 8]; 8];
    for v in 0..8 {
        let row = &f[v * 8..v * 8 + 8];
        if row.iter().all(|&c| c == 0) {
            continue;
        }
        for x in 0..8 {
            let s: i64 = (0..8).map(|u| t[x][u] * i64::from(row[u])).sum();
            tmp[v][x] = (s + (1 << 9)) >> 10;
        }
    }
    // Columns: out[y][x] = Σv T[y][v]·tmp[v][x], × 2¹⁶ in all.
    for y in 0..8 {
        for x in 0..8 {
            let s: i64 = (0..8).map(|v| t[y][v] * tmp[v][x]).sum();
            let p = ((s + (1 << 15)) >> 16) + 128;
            out[y * stride + x] = p.clamp(0, 255) as u8;
        }
    }
}

pub(crate) fn decode_jpeg(b: &[u8]) -> Result<Image, TextureError> {
    let be16 = |at: usize| -> Result<usize, TextureError> {
        b.get(at..at + 2)
            .map(|x| usize::from(u16::from_be_bytes([x[0], x[1]])))
            .ok_or_else(|| bad("JPEG is cut short"))
    };
    let mut quant = [[0i32; 64]; 4];
    let mut dc: Vec<Huffman> = vec![Huffman::default(); 4];
    let mut ac: Vec<Huffman> = vec![Huffman::default(); 4];
    let mut comps: Vec<Component> = Vec::new();
    let (mut width, mut height) = (0usize, 0usize);
    let (mut hmax, mut vmax) = (1usize, 1usize);
    let mut restart = 0usize;
    let mut progressive = false;
    let mut adobe: Option<u8> = None;
    let mut jfif = false;
    let mut at = 2;
    let table = idct_table();
    loop {
        // Next marker.
        while b.get(at) == Some(&0xFF) && b.get(at + 1) == Some(&0xFF) {
            at += 1;
        }
        if b.get(at) != Some(&0xFF) {
            return Err(bad("JPEG marker expected"));
        }
        let marker = *b.get(at + 1).ok_or_else(|| bad("JPEG is cut short"))?;
        at += 2;
        if marker == 0xD9 {
            break;
        }
        if (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            continue;
        }
        let len = be16(at)?;
        let seg = b
            .get(at + 2..at + len)
            .ok_or_else(|| bad("a JPEG segment runs past the end"))?;
        match marker {
            0xC0..=0xC2 => {
                progressive = marker == 0xC2;
                if seg.first() != Some(&8) {
                    return Err(TextureError::Unsupported(
                        "a JPEG of other than 8-bit precision".into(),
                    ));
                }
                height = usize::from(u16::from_be_bytes([seg[1], seg[2]]));
                width = usize::from(u16::from_be_bytes([seg[3], seg[4]]));
                let n = usize::from(seg[5]);
                if width == 0 || height == 0 || !(n == 1 || n == 3) || seg.len() < 6 + 3 * n {
                    return Err(TextureError::Unsupported(format!(
                        "a JPEG of {n} components or no height"
                    )));
                }
                for k in 0..n {
                    let c = &seg[6 + 3 * k..9 + 3 * k];
                    let (h, v) = (usize::from(c[1] >> 4), usize::from(c[1] & 15));
                    if !(1..=4).contains(&h) || !(1..=4).contains(&v) || c[2] > 3 {
                        return Err(bad("JPEG sampling factors"));
                    }
                    comps.push(Component {
                        id: c[0],
                        h,
                        v,
                        tq: usize::from(c[2]),
                        bw: 0,
                        bh: 0,
                        samples: Vec::new(),
                        coef: Vec::new(),
                    });
                }
                hmax = comps.iter().map(|c| c.h).max().unwrap_or(1);
                vmax = comps.iter().map(|c| c.v).max().unwrap_or(1);
                let (mx, my) = (width.div_ceil(8 * hmax), height.div_ceil(8 * vmax));
                for c in &mut comps {
                    c.bw = mx * c.h;
                    c.bh = my * c.v;
                    c.samples = vec![0; c.bw * 8 * c.bh * 8];
                    if progressive {
                        c.coef = vec![0; c.bw * c.bh * 64];
                    }
                }
            }
            0xC3..=0xCF if marker != 0xC4 && marker != 0xC8 && marker != 0xCC => {
                return Err(TextureError::Unsupported(
                    "a lossless, hierarchical or arithmetic-coded JPEG".into(),
                ));
            }
            0xC4 => {
                let mut i = 0;
                while i + 17 <= seg.len() {
                    let (class, id) = (seg[i] >> 4, usize::from(seg[i] & 15));
                    let counts: [u8; 16] = seg[i + 1..i + 17].try_into().unwrap_or([0; 16]);
                    let n: usize = counts.iter().map(|&c| usize::from(c)).sum();
                    let symbols = seg
                        .get(i + 17..i + 17 + n)
                        .ok_or_else(|| bad("JPEG Huffman table"))?
                        .to_vec();
                    if id > 3 {
                        return Err(bad("JPEG Huffman table id"));
                    }
                    let h = Huffman::new(&counts, symbols);
                    if class == 0 {
                        dc[id] = h;
                    } else {
                        ac[id] = h;
                    }
                    i += 17 + n;
                }
            }
            0xDB => {
                let mut i = 0;
                while i < seg.len() {
                    let (p, id) = (seg[i] >> 4, usize::from(seg[i] & 15));
                    if id > 3 {
                        return Err(bad("JPEG quantization table id"));
                    }
                    for k in 0..64 {
                        quant[id][ZIGZAG[k]] = if p == 0 {
                            i32::from(*seg.get(i + 1 + k).ok_or_else(|| bad("JPEG DQT"))?)
                        } else {
                            let at = i + 1 + 2 * k;
                            i32::from(u16::from_be_bytes([
                                *seg.get(at).ok_or_else(|| bad("JPEG DQT"))?,
                                *seg.get(at + 1).ok_or_else(|| bad("JPEG DQT"))?,
                            ]))
                        };
                    }
                    i += 1 + if p == 0 { 64 } else { 128 };
                }
            }
            0xDD => restart = usize::from(u16::from_be_bytes([seg[0], seg[1]])),
            0xE0 if seg.starts_with(b"JFIF\0") => jfif = true,
            0xEE if seg.starts_with(b"Adobe") && seg.len() >= 12 => adobe = Some(seg[11]),
            0xDA => {
                if comps.is_empty() {
                    return Err(bad("JPEG scan before its frame"));
                }
                let n = usize::from(seg[0]);
                let mut scan: Vec<(usize, usize, usize)> = Vec::new();
                for k in 0..n {
                    let id = seg[1 + 2 * k];
                    let t = seg[2 + 2 * k];
                    let ci = comps
                        .iter()
                        .position(|c| c.id == id)
                        .ok_or_else(|| bad("JPEG scan names no component"))?;
                    scan.push((ci, usize::from(t >> 4).min(3), usize::from(t & 15).min(3)));
                }
                // The entropy-coded data, split at restart markers, stuffing
                // removed.
                at += len;
                let mut chunks: Vec<Vec<u8>> = vec![Vec::new()];
                while at < b.len() {
                    if b[at] == 0xFF {
                        let next = *b.get(at + 1).unwrap_or(&0xD9);
                        if next == 0 {
                            if let Some(c) = chunks.last_mut() {
                                c.push(0xFF);
                            }
                            at += 2;
                            continue;
                        }
                        if (0xD0..=0xD7).contains(&next) {
                            chunks.push(Vec::new());
                            at += 2;
                            continue;
                        }
                        if next == 0xFF {
                            at += 1;
                            continue;
                        }
                        break;
                    }
                    if let Some(c) = chunks.last_mut() {
                        c.push(b[at]);
                    }
                    at += 1;
                }
                if progressive {
                    // The band of coefficients this scan carries, and
                    // which bit of them (G.1.1).
                    let band = seg
                        .get(1 + 2 * n..4 + 2 * n)
                        .ok_or_else(|| bad("JPEG scan header"))?;
                    scan_progressive(
                        &mut comps,
                        &scan,
                        &chunks,
                        restart,
                        &dc,
                        &ac,
                        (
                            usize::from(band[0]),
                            usize::from(band[1]),
                            u32::from(band[2] >> 4),
                            u32::from(band[2] & 15),
                        ),
                        (hmax, vmax, width, height),
                    )?;
                } else {
                    scan_data(
                        &mut comps,
                        &scan,
                        &chunks,
                        restart,
                        &quant,
                        &dc,
                        &ac,
                        &table,
                        (hmax, vmax, width, height),
                    )?;
                }
                continue;
            }
            _ => {}
        }
        at += len;
    }
    if comps.is_empty() {
        return Err(bad("JPEG has no frame"));
    }
    if progressive {
        // Every scan is in: scale each block by its table and transform it.
        let mut f = [0i32; 64];
        for c in &mut comps {
            let q = &quant[c.tq];
            let stride = c.bw * 8;
            for by in 0..c.bh {
                for bx in 0..c.bw {
                    let block = &c.coef[(by * c.bw + bx) * 64..][..64];
                    for i in 0..64 {
                        f[i] = block[i] * q[i];
                    }
                    idct(
                        &table,
                        &f,
                        &mut c.samples[by * 8 * stride + bx * 8..],
                        stride,
                    );
                }
            }
        }
    }
    let rgb_ids = comps.len() == 3 && comps.iter().map(|c| c.id).eq(*b"RGB");
    let ycc = comps.len() == 3 && !rgb_ids && (jfif || adobe != Some(0));
    let planes: Vec<Vec<i32>> = comps
        .iter()
        .map(|c| upsample(c, hmax, vmax, width, height))
        .collect();
    let mut rgba = vec![255u8; width * height * 4];
    for i in 0..width * height {
        let o = i * 4;
        if comps.len() == 1 {
            let g = planes[0][i] as u8;
            rgba[o..o + 3].copy_from_slice(&[g, g, g]);
        } else if ycc {
            let (yy, cb, cr) = (planes[0][i], planes[1][i] - 128, planes[2][i] - 128);
            let r = yy + ((91_881 * cr + 32_768) >> 16);
            let g = yy + ((-22_554 * cb - 46_802 * cr + 32_768) >> 16);
            let bl = yy + ((116_130 * cb + 32_768) >> 16);
            rgba[o..o + 3].copy_from_slice(&[r, g, bl].map(|v| v.clamp(0, 255) as u8));
        } else {
            rgba[o..o + 3].copy_from_slice(&[0, 1, 2].map(|k| planes[k][i] as u8));
        }
    }
    Ok(Image {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

/// A component's samples at the image's size. Half resolution in either
/// direction is filled by the triangle filter (each output sample ¾ the
/// nearer input sample and ¼ the next, clamped at the edges, the weights
/// of both directions multiplied); any other ratio by the nearest sample.
pub(crate) fn upsample(
    c: &Component,
    hmax: usize,
    vmax: usize,
    width: usize,
    height: usize,
) -> Vec<i32> {
    let stride = c.bw * 8;
    let (cw, ch) = (
        (width * c.h).div_ceil(hmax).max(1),
        (height * c.v).div_ceil(vmax).max(1),
    );
    let at = |x: usize, y: usize| i32::from(c.samples[y.min(ch - 1) * stride + x.min(cw - 1)]);
    // Per axis: for output i, the two input samples and their weights
    // out of 4 (one sample with weight 4 when not halved).
    let taps = |n: usize, m: usize, cn: usize| -> Vec<(usize, usize, i32)> {
        (0..n)
            .map(|i| {
                if m == 2 {
                    let s = i / 2;
                    let other = if i % 2 == 0 {
                        s.saturating_sub(1)
                    } else {
                        (s + 1).min(cn - 1)
                    };
                    (s, other, 3)
                } else {
                    let s = (i / m).min(cn - 1);
                    (s, s, 4)
                }
            })
            .collect()
    };
    let tx = taps(width, hmax / c.h, cw);
    let ty = taps(height, vmax / c.v, ch);
    let mut out = Vec::with_capacity(width * height);
    for &(y0, y1, wy) in &ty {
        for &(x0, x1, wx) in &tx {
            let v = wy * (wx * at(x0, y0) + (4 - wx) * at(x1, y0))
                + (4 - wy) * (wx * at(x0, y1) + (4 - wx) * at(x1, y1));
            out.push((v + 8) >> 4);
        }
    }
    out
}

/// Decodes one scan's blocks into the components' samples.
/// The blocks of a scan in coding order, as (scan component, block
/// across, block down), and how many of them a restart interval holds:
/// per MCU, each scan component's h × v blocks; a scan of one component
/// goes block by block over its own size (A.2.2).
pub(crate) fn block_order(
    comps: &[Component],
    scan: &[(usize, usize, usize)],
    restart: usize,
    (hmax, vmax, width, height): (usize, usize, usize, usize),
) -> (Vec<(usize, usize, usize)>, usize) {
    let mut order: Vec<(usize, usize, usize)> = Vec::new();
    if scan.len() == 1 {
        let c = &comps[scan[0].0];
        let (cw, ch) = (
            (width * c.h).div_ceil(hmax).div_ceil(8),
            (height * c.v).div_ceil(vmax).div_ceil(8),
        );
        for by in 0..ch {
            for bx in 0..cw {
                order.push((0, bx, by));
            }
        }
    } else {
        let (mx, my) = (width.div_ceil(8 * hmax), height.div_ceil(8 * vmax));
        for my_ in 0..my {
            for mx_ in 0..mx {
                for (si, &(ci, _, _)) in scan.iter().enumerate() {
                    let c = &comps[ci];
                    for v in 0..c.v {
                        for h in 0..c.h {
                            order.push((si, mx_ * c.h + h, my_ * c.v + v));
                        }
                    }
                }
            }
        }
    }
    let per_mcu = if scan.len() == 1 {
        1
    } else {
        scan.iter()
            .map(|&(ci, _, _)| comps[ci].h * comps[ci].v)
            .sum()
    };
    let interval = if restart > 0 {
        restart * per_mcu
    } else {
        usize::MAX
    };
    (order, interval)
}

/// One scan of a progressive frame (G.1.2): a band `ss..=se` of each
/// block's coefficients in zigzag order, either their bits from `al` up
/// (`ah` 0) or one more bit, `al`, of what earlier scans sent. The DC
/// coefficient has scans of its own, over any of the components; the
/// others come one component a scan, with runs of blocks that end early
/// counted across blocks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn scan_progressive(
    comps: &mut [Component],
    scan: &[(usize, usize, usize)],
    chunks: &[Vec<u8>],
    restart: usize,
    dc: &[Huffman],
    ac: &[Huffman],
    (ss, se, ah, al): (usize, usize, u32, u32),
    dims: (usize, usize, usize, usize),
) -> Result<(), TextureError> {
    if ss > se || se > 63 || al > 13 || (ss == 0 && se != 0) || (ss > 0 && scan.len() != 1) {
        return Err(bad("JPEG progressive scan"));
    }
    let (order, interval) = block_order(comps, scan, restart, dims);
    let mut pred = vec![0i32; scan.len()];
    let mut eobrun = 0u32;
    let mut chunk = 0;
    let mut bits = Bits {
        data: chunks.first().map_or(&[][..], |c| c.as_slice()),
        at: 0,
        acc: 0,
        n: 0,
    };
    let one = 1i32 << al;
    for (k, &(si, bx, by)) in order.iter().enumerate() {
        if k > 0 && k % interval == 0 {
            chunk += 1;
            bits = Bits {
                data: chunks.get(chunk).map_or(&[][..], |c| c.as_slice()),
                at: 0,
                acc: 0,
                n: 0,
            };
            pred.iter_mut().for_each(|p| *p = 0);
            eobrun = 0;
        }
        let (ci, td, ta) = scan[si];
        let c = &mut comps[ci];
        if bx >= c.bw || by >= c.bh {
            return Err(bad("JPEG block outside its component"));
        }
        let block = &mut c.coef[(by * c.bw + bx) * 64..][..64];
        // One more bit of a coefficient already sent, towards its sign.
        let refine = |v: &mut i32, bits: &mut Bits| {
            if bits.bit() == 1 && (*v & one) == 0 {
                *v += if *v >= 0 { one } else { -one };
            }
        };
        if ss == 0 {
            if ah == 0 {
                let s = bits.decode(&dc[td])?;
                pred[si] += bits.extend(u32::from(s));
                block[0] = pred[si] << al;
            } else if bits.bit() == 1 {
                block[0] |= one;
            }
            continue;
        }
        if ah == 0 {
            if eobrun > 0 {
                eobrun -= 1;
                continue;
            }
            let mut i = ss;
            while i <= se {
                let rs = bits.decode(&ac[ta])?;
                let (r, s) = (u32::from(rs >> 4), u32::from(rs & 15));
                if s == 0 {
                    if r < 15 {
                        // This block ends here, and 2^r − 1 + more after it.
                        eobrun = (1 << r) - 1;
                        if r > 0 {
                            eobrun += bits.bits(r);
                        }
                        break;
                    }
                    i += 16;
                    continue;
                }
                i += r as usize;
                if i > se {
                    return Err(bad("JPEG coefficient run past the band"));
                }
                block[ZIGZAG[i]] = bits.extend(s) << al;
                i += 1;
            }
            continue;
        }
        // Refinement of the band: new coefficients of size one, each
        // after a run of still-zero ones; every coefficient passed that
        // is not zero gets its next bit.
        let mut i = ss;
        if eobrun == 0 {
            while i <= se {
                let rs = bits.decode(&ac[ta])?;
                let (mut r, s) = (u32::from(rs >> 4), u32::from(rs & 15));
                let mut new = 0;
                match s {
                    0 if r < 15 => {
                        eobrun = 1 << r;
                        if r > 0 {
                            eobrun += bits.bits(r);
                        }
                        break;
                    }
                    0 => {}
                    1 => new = if bits.bit() == 1 { one } else { -one },
                    _ => return Err(bad("JPEG refinement of a size other than one")),
                }
                while i <= se {
                    let v = &mut block[ZIGZAG[i]];
                    if *v != 0 {
                        refine(v, &mut bits);
                    } else {
                        if r == 0 {
                            break;
                        }
                        r -= 1;
                    }
                    i += 1;
                }
                if new != 0 {
                    if i > se {
                        return Err(bad("JPEG coefficient run past the band"));
                    }
                    block[ZIGZAG[i]] = new;
                }
                i += 1;
            }
        }
        if eobrun > 0 {
            // The rest of the band has nothing new: only next bits.
            while i <= se {
                let v = &mut block[ZIGZAG[i]];
                if *v != 0 {
                    refine(v, &mut bits);
                }
                i += 1;
            }
            eobrun -= 1;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn scan_data(
    comps: &mut [Component],
    scan: &[(usize, usize, usize)],
    chunks: &[Vec<u8>],
    restart: usize,
    quant: &[[i32; 64]; 4],
    dc: &[Huffman],
    ac: &[Huffman],
    table: &[[i64; 8]; 8],
    (hmax, vmax, width, height): (usize, usize, usize, usize),
) -> Result<(), TextureError> {
    let (order, interval) = block_order(comps, scan, restart, (hmax, vmax, width, height));
    let mut pred = vec![0i32; scan.len()];
    let mut chunk = 0;
    let mut bits = Bits {
        data: chunks.first().map_or(&[][..], |c| c.as_slice()),
        at: 0,
        acc: 0,
        n: 0,
    };
    let mut f = [0i32; 64];
    for (k, &(si, bx, by)) in order.iter().enumerate() {
        if k > 0 && k % interval == 0 {
            chunk += 1;
            bits = Bits {
                data: chunks.get(chunk).map_or(&[][..], |c| c.as_slice()),
                at: 0,
                acc: 0,
                n: 0,
            };
            pred.iter_mut().for_each(|p| *p = 0);
        }
        let (ci, td, ta) = scan[si];
        let q = &quant[comps[ci].tq];
        f.fill(0);
        let s = bits.decode(&dc[td])?;
        pred[si] += bits.extend(u32::from(s));
        f[0] = pred[si] * q[0];
        let mut i = 1;
        while i < 64 {
            let rs = bits.decode(&ac[ta])?;
            let (r, s) = (usize::from(rs >> 4), u32::from(rs & 15));
            if s == 0 {
                if r == 15 {
                    i += 16;
                    continue;
                }
                break;
            }
            i += r;
            if i > 63 {
                return Err(bad("JPEG coefficient run past the block"));
            }
            f[ZIGZAG[i]] = bits.extend(s) * q[ZIGZAG[i]];
            i += 1;
        }
        let c = &mut comps[ci];
        if bx < c.bw && by < c.bh {
            let stride = c.bw * 8;
            idct(
                table,
                &f,
                &mut c.samples[by * 8 * stride + bx * 8..],
                stride,
            );
        }
    }
    Ok(())
}
