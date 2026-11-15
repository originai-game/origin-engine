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

#[cfg(test)]
mod tests {
    use super::*;

    /// 2^k, exactly, from its bits.
    fn two(k: i32) -> f32 {
        f32::from_bits(((127 + k) as u32) << 23)
    }

    #[test]
    fn every_half_round_trips_and_ties_go_to_even() {
        for h in 0..=u16::MAX {
            let x = f16_to_f32(h);
            if x.is_nan() {
                assert!(f16_to_f32(f32_to_f16(x)).is_nan(), "{h:#06x}");
            } else {
                assert_eq!(f32_to_f16(x), h, "{h:#06x}");
            }
        }
        assert_eq!(f16_to_f32(0x3555), 0.333_251_95);
        assert_eq!(f16_to_f32(0x0001), two(-24));
        assert_eq!(f16_to_f32(0x8000).to_bits(), (-0.0f32).to_bits());
        // 1 + 2^-11 lies halfway between 1 and 1 + 2^-10: to the even, 1;
        // 1 + 3·2^-11 halfway between odd and even codes: up, to the even.
        assert_eq!(f32_to_f16(1.0 + two(-11)), 0x3c00);
        assert_eq!(f32_to_f16(1.0 + 3.0 * two(-11)), 0x3c02);
        // Just above a tie rounds up; just below, down.
        assert_eq!(f32_to_f16(1.0 + two(-11) + two(-20)), 0x3c01);
        // Subnormal ties: 2^-25 (half the smallest) goes to even zero,
        // 3·2^-25 to the even code 2.
        assert_eq!(f32_to_f16(two(-25)), 0);
        assert_eq!(f32_to_f16(3.0 * two(-25)), 2);
        assert_eq!(f32_to_f16(65504.0), 0x7bff);
        assert_eq!(f32_to_f16(65520.0), 0x7c00);
        assert_eq!(f32_to_f16(1e6), 0x7c00);
        assert_eq!(f32_to_f16(-1e-10), 0x8000);
        assert_eq!(f32_to_f16(0.333_333_34), 0x3555);
        assert!(f16_to_f32(f32_to_f16(f32::NAN)).is_nan());
        assert_eq!(f32_to_f16(f32::NEG_INFINITY), 0xfc00);
    }
}
