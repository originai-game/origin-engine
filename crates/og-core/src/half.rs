//! IEEE 754 half precision (binary16): the bits of an `f16` texel or
//! tensor, the one copy. Integer arithmetic only, so the same bits on every
//! host.

/// The half nearest `x`: round to nearest, ties to even (what a GPU stores
/// in an `rgba16float` target); past 65504 is infinity, a NaN stays a
/// (quiet) NaN, and values below the smallest subnormal round to zero with
/// their sign.
pub fn f32_to_f16(x: f32) -> u16 {
    let b = x.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32;
    let mant = b & 0x7f_ffff;
    if exp == 0xff {
        // Infinity, or a NaN kept quiet with the top of its payload.
        return sign
            | 0x7c00
            | if mant != 0 {
                0x200 | (mant >> 13) as u16
            } else {
                0
            };
    }
    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }
    // The significand with its implicit bit, and how many of its low bits go.
    let (m, shift) = if e <= 0 {
        if e < -10 {
            return sign;
        }
        (mant | 0x80_0000, (14 - e) as u32)
    } else {
        (mant, 13)
    };
    let kept = m >> shift;
    let rest = m & ((1 << shift) - 1);
    let halfway = 1 << (shift - 1);
    let up = u32::from(rest > halfway || (rest == halfway && kept & 1 == 1));
    let body = if e <= 0 {
        kept
    } else {
        ((e as u32) << 10) | kept
    };
    // A carry out of the significand lands in the exponent: the right result.
    sign | (body + up) as u16
}

/// The value of the half `bits`, exactly (every half is an f32): signed
/// zeros, subnormals, infinities and NaN payloads kept.
pub fn f16_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x3ff);
    let value = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            // Subnormal: normalise the mantissa.
            let shift = mantissa.leading_zeros() - 21;
            let mantissa = (mantissa << shift) & 0x3ff;
            sign | ((113 - shift) << 23) | (mantissa << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(value)
}
