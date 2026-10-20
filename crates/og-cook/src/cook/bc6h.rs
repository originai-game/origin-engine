//! BC6H (BPTC float) block decoding, for HDR cube maps the cook takes in
//! that format and decodes before it prefilters (`type environment`).
//!
//! A block is 16 bytes for 4 x 4 texels of RGB half floats, unsigned
//! (`VK_FORMAT_BC6H_UFLOAT_BLOCK`) or signed (`_SFLOAT_BLOCK`). Each of the
//! 14 modes is described below by its header layout in the notation of the
//! BC6H specification (the D3D11 functional specification's BC6H section,
//! Khronos Data Format §BC6H): `rw[9:0]` is bits 9 to 0 of the first red
//! endpoint, read from bit 0 up; `rw[10:15]`, as the 16-bit mode writes it,
//! is bits 10 to 15 read from bit 15 down. `w x y z` are the endpoints (a
//! one-region mode has `w` and `x`), `d` the partition. Then: sign
//! extension, the inverse delta transform, unquantisation, interpolation
//! with the 3- or 4-bit weights, and the final scale to a half float, as
//! the specification gives them.

/// A mode: its mode bits' value, its header after the mode bits, the
/// endpoint precision and the deltas' precisions (red, green, blue), whether
/// it has two regions and whether its endpoints are deltas from `w`.
struct Mode {
    bits: u32,
    layout: &'static str,
    precision: u32,
    delta: [u32; 3],
    two: bool,
    transformed: bool,
}

const MODES: [Mode; 14] = [
    Mode {
        bits: 0b00,
        layout: "gy[4] by[4] bz[4] rw[9:0] gw[9:0] bw[9:0] rx[4:0] gz[4] gy[3:0] gx[4:0] bz[0] gz[3:0] bx[4:0] bz[1] by[3:0] ry[4:0] bz[2] rz[4:0] bz[3] d[4:0]",
        precision: 10,
        delta: [5, 5, 5],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b01,
        layout: "gy[5] gz[4] gz[5] rw[6:0] bz[0] bz[1] by[4] gw[6:0] by[5] bz[2] gy[4] bw[6:0] bz[3] bz[5] bz[4] rx[5:0] gy[3:0] gx[5:0] gz[3:0] bx[5:0] by[3:0] ry[5:0] rz[5:0] d[4:0]",
        precision: 7,
        delta: [6, 6, 6],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b00010,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[4:0] rw[10] gy[3:0] gx[3:0] gw[10] bz[0] gz[3:0] bx[3:0] bw[10] bz[1] by[3:0] ry[4:0] bz[2] rz[4:0] bz[3] d[4:0]",
        precision: 11,
        delta: [5, 4, 4],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b00110,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[3:0] rw[10] gz[4] gy[3:0] gx[4:0] gw[10] gz[3:0] bx[3:0] bw[10] bz[1] by[3:0] ry[3:0] bz[0] bz[2] rz[3:0] gy[4] bz[3] d[4:0]",
        precision: 11,
        delta: [4, 5, 4],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b01010,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[3:0] rw[10] by[4] gy[3:0] gx[3:0] gw[10] bz[0] gz[3:0] bx[4:0] bw[10] by[3:0] ry[3:0] bz[1] bz[2] rz[3:0] bz[4] bz[3] d[4:0]",
        precision: 11,
        delta: [4, 4, 5],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b01110,
        layout: "rw[8:0] by[4] gw[8:0] gy[4] bw[8:0] bz[4] rx[4:0] gz[4] gy[3:0] gx[4:0] bz[0] gz[3:0] bx[4:0] bz[1] by[3:0] ry[4:0] bz[2] rz[4:0] bz[3] d[4:0]",
        precision: 9,
        delta: [5, 5, 5],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b10010,
        layout: "rw[7:0] gz[4] by[4] gw[7:0] bz[2] gy[4] bw[7:0] bz[3] bz[4] rx[5:0] gy[3:0] gx[4:0] bz[0] gz[3:0] bx[4:0] bz[1] by[3:0] ry[5:0] rz[5:0] d[4:0]",
        precision: 8,
        delta: [6, 5, 5],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b10110,
        layout: "rw[7:0] bz[0] by[4] gw[7:0] gy[5] gy[4] bw[7:0] gz[5] bz[4] rx[4:0] gz[4] gy[3:0] gx[5:0] gz[3:0] bx[4:0] bz[1] by[3:0] ry[4:0] bz[2] rz[4:0] bz[3] d[4:0]",
        precision: 8,
        delta: [5, 6, 5],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b11010,
        layout: "rw[7:0] bz[1] by[4] gw[7:0] by[5] gy[4] bw[7:0] bz[5] bz[4] rx[4:0] gz[4] gy[3:0] gx[4:0] bz[0] gz[3:0] bx[5:0] by[3:0] ry[4:0] bz[2] rz[4:0] bz[3] d[4:0]",
        precision: 8,
        delta: [5, 5, 6],
        two: true,
        transformed: true,
    },
    Mode {
        bits: 0b11110,
        layout: "rw[5:0] gz[4] bz[0] bz[1] by[4] gw[5:0] gy[5] by[5] bz[2] gy[4] bw[5:0] gz[5] bz[3] bz[5] bz[4] rx[5:0] gy[3:0] gx[5:0] gz[3:0] bx[5:0] by[3:0] ry[5:0] rz[5:0] d[4:0]",
        precision: 6,
        delta: [6, 6, 6],
        two: true,
        transformed: false,
    },
    Mode {
        bits: 0b00011,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[9:0] gx[9:0] bx[9:0]",
        precision: 10,
        delta: [10, 10, 10],
        two: false,
        transformed: false,
    },
    Mode {
        bits: 0b00111,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[8:0] rw[10] gx[8:0] gw[10] bx[8:0] bw[10]",
        precision: 11,
        delta: [9, 9, 9],
        two: false,
        transformed: true,
    },
    Mode {
        bits: 0b01011,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[7:0] rw[10:11] gx[7:0] gw[10:11] bx[7:0] bw[10:11]",
        precision: 12,
        delta: [8, 8, 8],
        two: false,
        transformed: true,
    },
    Mode {
        bits: 0b01111,
        layout: "rw[9:0] gw[9:0] bw[9:0] rx[3:0] rw[10:15] gx[3:0] gw[10:15] bx[3:0] bw[10:15]",
        precision: 16,
        delta: [4, 4, 4],
        two: false,
        transformed: true,
    },
];

/// The two-region partitions: bit `i` is texel `i`'s region (texels in
/// rows from the top, left to right). BC7's two-subset table.
const PARTITIONS: [u16; 32] = [
    0xCCCC, 0x8888, 0xEEEE, 0xECC8, 0xC880, 0xFEEC, 0xFEC8, 0xEC80, 0xC800, 0xFFEC, 0xFE80, 0xE800,
    0xFFE8, 0xFF00, 0xFFF0, 0xF000, 0xF710, 0x008E, 0x7100, 0x08CE, 0x008C, 0x7310, 0x3100, 0x8CCE,
    0x088C, 0x3110, 0x6666, 0x366C, 0x17E8, 0x0FF0, 0x718E, 0x399C,
];

/// The second region's anchor texel per partition (the first's is texel
/// 0): its index is stored with one bit less.
const ANCHORS: [u8; 32] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 2, 8, 2, 2, 8, 8, 15, 2, 8,
    2, 2, 8, 8, 2, 2,
];

const WEIGHTS3: [i32; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
const WEIGHTS4: [i32; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

struct Bits(u128, u32);

impl Bits {
    fn take(&mut self, n: u32) -> u32 {
        let v = ((self.0 >> self.1) & ((1u128 << n) - 1)) as u32;
        self.1 += n;
        v
    }
}

fn extend(v: i32, bits: u32) -> i32 {
    (v << (32 - bits)) >> (32 - bits)
}

fn unquantize(v: i32, bits: u32, signed: bool) -> i32 {
    if !signed {
        if bits >= 15 {
            v
        } else if v == 0 {
            0
        } else if v == (1 << bits) - 1 {
            0xFFFF
        } else {
            ((v << 16) + 0x8000) >> bits
        }
    } else {
        if bits >= 16 {
            return v;
        }
        let (negative, m) = (v < 0, v.abs());
        let u = if m == 0 {
            0
        } else if m >= (1 << (bits - 1)) - 1 {
            0x7FFF
        } else {
            ((m << 15) + 0x4000) >> (bits - 1)
        };
        if negative { -u } else { u }
    }
}

/// The interpolated value as half-float bits.
fn finish(v: i32, signed: bool) -> u16 {
    if !signed {
        ((v * 31) >> 6) as u16
    } else {
        let m = if v < 0 {
            -(((-v) * 31) >> 5)
        } else {
            (v * 31) >> 5
        };
        if m < 0 {
            0x8000 | (-m) as u16
        } else {
            m as u16
        }
    }
}

/// A half float's value.
pub fn half(h: u16) -> f32 {
    let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = (h >> 10) & 0x1f;
    let man = f32::from(h & 0x3ff);
    sign * match exp {
        0 => man * 2f32.powi(-24),
        31 if man == 0.0 => f32::INFINITY,
        31 => f32::NAN,
        e => (1.0 + man / 1024.0) * 2f32.powi(i32::from(e) - 15),
    }
}

/// Decodes one block: its 16 texels (rows from the top) as half-float bits
/// of red, green and blue. A reserved mode decodes to zero, as the
/// specification asks.
pub fn decode_block_half(block: &[u8; 16], signed: bool) -> [[u16; 3]; 16] {
    let mut bits = Bits(u128::from_le_bytes(*block), 0);
    let mut m = bits.take(2);
    if m > 1 {
        m |= bits.take(3) << 2;
    }
    let Some(mode) = MODES.iter().find(|x| x.bits == m) else {
        return [[0; 3]; 16];
    };
    // Endpoints: [channel][w x y z].
    let mut e = [[0i32; 4]; 3];
    let mut partition = 0usize;
    for field in mode.layout.split(' ') {
        let (name, range) = field.split_once('[').expect("a field");
        let range = range.trim_end_matches(']');
        let (a, b) = match range.split_once(':') {
            Some((a, b)) => (a.parse::<u32>().unwrap(), b.parse::<u32>().unwrap()),
            None => {
                let a = range.parse::<u32>().unwrap();
                (a, a)
            }
        };
        // Read from the right-hand index towards the left-hand one.
        let order: Vec<u32> = if a >= b {
            (b..=a).collect()
        } else {
            (a..=b).rev().collect()
        };
        for bit in order {
            let v = bits.take(1) as i32;
            match name {
                "d" => partition |= (v as usize) << bit,
                _ => {
                    let c = match &name[..1] {
                        "r" => 0,
                        "g" => 1,
                        _ => 2,
                    };
                    let k = match &name[1..] {
                        "w" => 0,
                        "x" => 1,
                        "y" => 2,
                        _ => 3,
                    };
                    e[c][k] |= v << bit;
                }
            }
        }
    }
    let ends = if mode.two { 4 } else { 2 };
    for (c, ch) in e.iter_mut().enumerate() {
        if signed {
            ch[0] = extend(ch[0], mode.precision);
        }
        if mode.transformed || signed {
            for v in ch.iter_mut().take(ends).skip(1) {
                *v = extend(*v, mode.delta[c]);
            }
        }
        if mode.transformed {
            for k in 1..ends {
                let v = (ch[k] + ch[0]) & ((1 << mode.precision) - 1);
                ch[k] = if signed { extend(v, mode.precision) } else { v };
            }
        }
        for v in ch.iter_mut().take(ends) {
            *v = unquantize(*v, mode.precision, signed);
        }
    }
    let mut out = [[0u16; 3]; 16];
    for (i, texel) in out.iter_mut().enumerate() {
        let region = if mode.two {
            usize::from(PARTITIONS[partition] >> i & 1 == 1)
        } else {
            0
        };
        let anchor = i == 0 || (mode.two && i == ANCHORS[partition] as usize);
        let width = if mode.two { 3 } else { 4 } - u32::from(anchor);
        let index = bits.take(width) as usize;
        let w = if mode.two {
            WEIGHTS3[index]
        } else {
            WEIGHTS4[index]
        };
        for c in 0..3 {
            let (a, b) = (e[c][2 * region], e[c][2 * region + 1]);
            texel[c] = finish((a * (64 - w) + b * w + 32) >> 6, signed);
        }
    }
    out
}

/// [`decode_block_half`] as linear RGB floats.
pub fn decode_block(block: &[u8; 16], signed: bool) -> [[f32; 3]; 16] {
    decode_block_half(block, signed).map(|t| t.map(half))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mode 11 (one region, raw 10-bit endpoints) as an encoder writes it:
    /// endpoints, then 3 + 15 x 4 index bits.
    fn mode11(w: [u32; 3], x: [u32; 3], indices: [u32; 16]) -> [u8; 16] {
        let mut v: u128 = 0b00011;
        let mut at = 5;
        let mut put = |value: u32, n: u32| {
            v |= u128::from(value) << at;
            at += n;
        };
        for c in w {
            put(c, 10);
        }
        for c in x {
            put(c, 10);
        }
        for (i, ix) in indices.iter().enumerate() {
            put(*ix, if i == 0 { 3 } else { 4 });
        }
        v.to_le_bytes()
    }

    #[test]
    fn half_floats_decode() {
        assert_eq!(half(0x3C00), 1.0);
        assert_eq!(half(0xC000), -2.0);
        assert_eq!(half(0x7BFF), 65504.0);
        assert_eq!(half(0x0001), 2f32.powi(-24));
        assert!(half(0x7C00).is_infinite() && half(0x7E00).is_nan());
    }

    #[test]
    fn a_one_region_block_interpolates_its_endpoints() {
        // 0 to 1023 in red, the weights from 0 to 64 across the texels.
        let idx: [u32; 16] = std::array::from_fn(|i| i as u32 % 8);
        let b = mode11([0, 1023, 512], [1023, 0, 512], idx);
        let t = decode_block_half(&b, false);
        // Endpoint 1023 unquantises to 0xFFFF, which finishes to 0x7BFF (65504).
        assert_eq!(
            t[0],
            [
                0,
                0x7BFF,
                (((((512 << 16) + 0x8000) >> 10) * 31) >> 6) as u16
            ]
        );
        for (i, x) in t.iter().enumerate() {
            let w = WEIGHTS4[i % 8];
            assert_eq!(
                x[0],
                ((((65535 * w + 32) >> 6) * 31) >> 6) as u16,
                "texel {i}"
            );
        }
    }

    /// Blocks of every mode, decoded by a reference BC6H decoder (bcdec,
    /// unsigned and signed): the first, an anchor and the last texel.
    #[test]
    fn every_mode_decodes_as_the_reference_does() {
        for (block, signed, want) in REFERENCE {
            let got = decode_block_half(block, *signed);
            for (i, w) in want.iter() {
                assert_eq!(got[*i], *w, "{block:02x?} signed {signed} texel {i}");
            }
        }
    }

    /// Per mode, 256 pseudo-random blocks of it decode, unsigned and
    /// signed, to the texels a reference decoder (bcdec_rs 0.2.0) gives:
    /// FNV-1a 64 over every texel's half-float bits.
    #[test]
    fn every_mode_hashes_as_the_reference_decodes() {
        const HASHES: [[u64; 2]; 14] = [
            [0xc52fc318d366e56c, 0x19e5ef0dc14a8523],
            [0x02900a791c98dcab, 0x585ab98e11cc2c56],
            [0xa2a588f9e4931b1d, 0xc0f1c84181b86514],
            [0x0d6ba27af505ac08, 0x33e7f88b11530841],
            [0x627b27b327c731fd, 0x6a65cf3b626d9ff2],
            [0xacbe211815469144, 0x72b27af2f3de0cc2],
            [0xf2bc033a053b0add, 0x1626d145bf809b66],
            [0x3c44a83637169727, 0x2d462d7eeb324352],
            [0xe62e6c0e9f6c87c0, 0xd4970bbec27fe300],
            [0xdb29ecdb6530a97c, 0xb60979d14250b152],
            [0x99e0b54f98b7852d, 0x54a5d8bac0aeb7fd],
            [0x41e2a5d89725d414, 0x9cfe1abfe5b31037],
            [0x00ebe1117da928d3, 0xeb5a7024cef7a7af],
            [0x53858a2a02da81a7, 0xe0cf335da990d595],
        ];
        for (k, mode) in MODES.iter().enumerate() {
            for (signed, want) in [(false, HASHES[k][0]), (true, HASHES[k][1])] {
                let mut s: u64 = 0x2545F4914F6CDD1D ^ (k as u64).wrapping_mul(0x9E3779B97F4A7C15);
                let mut h = 0xcbf29ce484222325u64;
                for _ in 0..256 {
                    let mut next = || {
                        s ^= s << 13;
                        s ^= s >> 7;
                        s ^= s << 17;
                        s
                    };
                    let mut v = u128::from(next()) | (u128::from(next()) << 64);
                    let width = if mode.bits < 2 { 2 } else { 5 };
                    v = (v & !((1u128 << width) - 1)) | u128::from(mode.bits);
                    for t in decode_block_half(&v.to_le_bytes(), signed) {
                        for c in t {
                            for b in c.to_le_bytes() {
                                h ^= u64::from(b);
                                h = h.wrapping_mul(0x100000001b3);
                            }
                        }
                    }
                }
                assert_eq!(h, want, "mode {k} (bits {:05b}) signed {signed}", mode.bits);
            }
        }
    }

    /// Generated with bcdec_rs 0.2.0 (MIT; a port of bcdec) from
    /// pseudo-random blocks, the first of each mode and signedness, and of
    /// a reserved mode (all zero). This decoder agreed with it on every
    /// texel of 2,000,000 random blocks, unsigned and signed.
    /// A block, its signedness, and texels with their half-float bits.
    type Reference = (&'static [u8; 16], bool, &'static [(usize, [u16; 3])]);

    #[rustfmt::skip]
    const REFERENCE: &[Reference] = &[
    (&[236, 74, 197, 161, 224, 22, 235, 44, 121, 251, 123, 78, 206, 29, 16, 151], false, &[(0, [18549, 28031, 2434]), (2, [18460, 27946, 2278]), (15, [18512, 27958, 2370])]),
    (&[236, 74, 197, 161, 224, 22, 235, 44, 121, 251, 123, 78, 206, 29, 16, 151], true, &[(0, [59218, 40254, 4868]), (2, [59397, 40425, 4557]), (15, [59292, 40401, 4740])]),
    (&[173, 77, 243, 11, 174, 119, 27, 220, 118, 96, 110, 2, 185, 238, 240, 100], false, &[(0, [26005, 14852, 13919]), (2, [25916, 24180, 30628]), (15, [26005, 14852, 13919])]),
    (&[173, 77, 243, 11, 174, 119, 27, 220, 118, 96, 110, 2, 185, 238, 240, 100], true, &[(0, [44741, 40053, 844]), (2, [44920, 48392, 35496]), (15, [44741, 40053, 844])]),
    (&[130, 171, 93, 35, 206, 45, 93, 100, 85, 176, 171, 189, 117, 154, 138, 128], false, &[(0, [21243, 10811, 12140]), (8, [21406, 10911, 12102]), (15, [21211, 10779, 12103])]),
    (&[130, 171, 93, 35, 206, 45, 93, 100, 85, 176, 171, 189, 117, 154, 138, 128], true, &[(0, [53800, 21623, 24280]), (8, [53473, 21823, 24204]), (15, [53864, 21559, 24207])]),
    (&[227, 117, 1, 109, 173, 69, 10, 60, 200, 111, 170, 134, 169, 37, 236, 91], false, &[(0, [22973, 12392, 16802]), (1, [10424, 5277, 7349]), (15, [21497, 11555, 15690])]),
    (&[227, 117, 1, 109, 173, 69, 10, 60, 200, 111, 170, 134, 169, 37, 236, 91], true, &[(0, [33489, 54653, 45831]), (1, [7940, 35121, 1791]), (15, [297, 52355, 44083])]),
    (&[38, 90, 41, 56, 233, 99, 179, 243, 18, 65, 111, 73, 243, 78, 58, 46], false, &[(0, [27035, 1246, 2471]), (2, [26985, 1246, 2512]), (15, [27048, 1268, 2441])]),
    (&[38, 90, 41, 56, 233, 99, 179, 243, 18, 65, 111, 73, 243, 78, 58, 46], true, &[(0, [42215, 2492, 4943]), (2, [42315, 2492, 5024]), (15, [42189, 2536, 4882])]),
    (&[39, 180, 188, 38, 248, 37, 47, 134, 143, 36, 70, 166, 221, 223, 142, 96], false, &[(0, [7858, 4870, 14401]), (1, [8044, 4739, 14165]), (15, [7673, 5001, 14637])]),
    (&[39, 180, 188, 38, 248, 37, 47, 134, 143, 36, 70, 166, 221, 223, 142, 96], true, &[(0, [15717, 9740, 37709]), (1, [16088, 9479, 34212]), (15, [15347, 10002, 41206])]),
    (&[234, 162, 239, 217, 182, 49, 76, 7, 47, 135, 183, 53, 141, 104, 196, 190], false, &[(0, [20217, 15370, 13616]), (8, [20312, 15244, 13461]), (15, [20271, 15379, 13741])]),
    (&[234, 162, 239, 217, 182, 49, 76, 7, 47, 135, 183, 53, 141, 104, 196, 190], true, &[(0, [55852, 30741, 27232]), (8, [55661, 30488, 26923]), (15, [55744, 30758, 27483])]),
    (&[139, 226, 214, 176, 133, 78, 170, 221, 142, 141, 66, 73, 211, 169, 142, 143], false, &[(0, [21808, 19498, 21267]), (1, [21785, 19538, 21234]), (15, [21785, 19538, 21234])]),
    (&[139, 226, 214, 176, 133, 78, 170, 221, 142, 141, 66, 73, 211, 169, 142, 143], true, &[(0, [52654, 57274, 53736]), (1, [52701, 57195, 53803]), (15, [52701, 57195, 53803])]),
    (&[206, 173, 220, 173, 116, 19, 240, 90, 21, 167, 63, 232, 233, 247, 170, 47], false, &[(0, [23089, 27373, 5075]), (8, [23447, 27069, 5355]), (15, [22845, 27373, 5267])]),
    (&[206, 173, 220, 173, 116, 19, 240, 90, 21, 167, 63, 232, 233, 247, 170, 47], true, &[(0, [50201, 41634, 10150]), (8, [49484, 42240, 10710]), (15, [50689, 41634, 10534])]),
    (&[207, 242, 232, 214, 68, 225, 210, 154, 234, 112, 244, 26, 46, 121, 170, 217], false, &[(0, [8379, 18578, 25100]), (1, [8377, 18580, 25102]), (15, [8377, 18580, 25101])]),
    (&[207, 242, 232, 214, 68, 225, 210, 154, 234, 112, 244, 26, 46, 121, 170, 217], true, &[(0, [16758, 59099, 46054]), (1, [16754, 59094, 46051]), (15, [16754, 59095, 46052])]),
    (&[50, 215, 138, 158, 62, 121, 55, 168, 28, 78, 220, 135, 114, 188, 234, 80], false, &[(0, [21694, 2404, 9021]), (15, [24982, 4049, 8633])]),
    (&[50, 215, 138, 158, 62, 121, 55, 168, 28, 78, 220, 135, 114, 188, 234, 80], true, &[(0, [53115, 4808, 18042]), (15, [46539, 8098, 17267])]),
    (&[19, 246, 189, 205, 211, 179, 134, 220, 63, 105, 231, 105, 194, 68, 8, 109], false, &[(0, [0, 0, 0]), (1, [0, 0, 0]), (15, [0, 0, 0])]),
    (&[19, 246, 189, 205, 211, 179, 134, 220, 63, 105, 231, 105, 194, 68, 8, 109], true, &[(0, [0, 0, 0]), (1, [0, 0, 0]), (15, [0, 0, 0])]),
    (&[54, 97, 144, 229, 145, 206, 7, 123, 116, 204, 141, 54, 12, 5, 95, 48], false, &[(0, [13837, 3925, 29546]), (15, [434, 2914, 28458])]),
    (&[54, 97, 144, 229, 145, 206, 7, 123, 116, 204, 141, 54, 12, 5, 95, 48], true, &[(0, [786, 7850, 37410]), (15, [868, 5828, 39588])]),
    (&[26, 162, 10, 156, 213, 80, 26, 15, 222, 113, 101, 130, 32, 33, 174, 128], false, &[(0, [1941, 2421, 26129]), (15, [3487, 3483, 26652])]),
    (&[26, 162, 10, 156, 213, 80, 26, 15, 222, 113, 101, 130, 32, 33, 174, 128], true, &[(0, [3882, 4843, 44245]), (15, [6975, 6967, 43199])]),
    (&[94, 14, 210, 86, 136, 226, 28, 45, 194, 253, 179, 72, 237, 21, 42, 87], false, &[(0, [25048, 18104, 21576]), (2, [18429, 1999, 14593]), (15, [20444, 18522, 19204])]),
    (&[94, 14, 210, 86, 136, 226, 28, 45, 194, 253, 179, 72, 237, 21, 42, 87], true, &[(0, [47152, 61040, 54096]), (2, [60814, 3999, 59419]), (15, [38224, 60203, 40704])]),
    ];
}
