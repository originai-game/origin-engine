//! BC6H (BPTC float) block decoding, for HDR cube maps the cook takes in
//! that format and decodes before the environment prefilter.
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
