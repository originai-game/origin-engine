//! The f64 counterparts of this module's maths, for code that computes in
//! f64 (the modelling kernel, og-shape): the same kernels, `+ - * /`, `sqrt`
//! and `floor` only, so every host gives the same bits. Not correctly
//! rounded: within a few ulps of the true value.
//!
//! An angle reduces by a three-part π/2 (fdlibm's split, exact for a
//! quotient below 2^20); a larger angle is first taken modulo the f64 2π
//! (`%`, which IEEE 754 defines exactly), so it stays deterministic but no
//! longer tracks the true sine. A TypeScript twin of [`sin_cos`]
//! (`sinCos64`, packages/engine's math.ts) gives the same bits.

use super::{
    FRAC_2_PI, INV_FACT, LN2_HI, LN2_LO, LOG2_E, atan64, cos_kernel, horner, ln64, sin_kernel,
};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

/// The circle constant π.
pub const PI: f64 = std::f64::consts::PI;
/// 2π, one full turn in radians.
pub const TAU: f64 = std::f64::consts::TAU;

/// Degrees to radians.
pub fn to_radians(degrees: f64) -> f64 {
    degrees * (PI / 180.0)
}

/// Radians to degrees.
pub fn to_degrees(radians: f64) -> f64 {
    radians * (180.0 / PI)
}

// π/2 in 33-bit pieces (fdlibm e_rem_pio2.c): k·P1, k·P2 and k·P3 are
// exact for k < 2^20.
const P1: f64 = f64::from_bits(0x3ff9_21fb_5440_0000);
const P2: f64 = f64::from_bits(0x3dd0_b461_1a60_0000);
const P3: f64 = f64::from_bits(0x3ba3_198a_2e00_0000);
const P3T: f64 = f64::from_bits(0x397b_839a_2520_49c1);
/// 2^20·π/2: below it [`reduce`]'s quotient is exact.
const MEDIUM: f64 = 1_647_099.0;

/// A finite `a >= 0` as `r` in about [-π/4, π/4] and its quadrant mod 4.
fn reduce(a: f64) -> (f64, u32) {
    let a = if a < MEDIUM { a } else { a % TAU };
    let k = (a * FRAC_2_PI + 0.5).floor();
    let r = (((a - k * P1) - k * P2) - k * P3) - k * P3T;
    (r, (k as u64 & 3) as u32)
}

/// `(sin x, cos x)`. NaN or infinite input gives NaN for both.
pub fn sin_cos(x: f64) -> (f64, f64) {
    if !x.is_finite() {
        return (f64::NAN, f64::NAN);
    }
    let (r, q) = reduce(x.abs());
    let (s, c) = (sin_kernel(r), cos_kernel(r));
    let (s, c) = match q {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    };
    (if x.is_sign_negative() { -s } else { s }, c)
}

/// Sine of `x` radians.
pub fn sin(x: f64) -> f64 {
    sin_cos(x).0
}

/// Cosine of `x` radians.
pub fn cos(x: f64) -> f64 {
    sin_cos(x).1
}

/// Tangent of `x` radians.
pub fn tan(x: f64) -> f64 {
    let (s, c) = sin_cos(x);
    s / c
}

/// Arctangent, in [-π/2, π/2].
pub fn atan(x: f64) -> f64 {
    atan64(x)
}

/// Angle of the point `(x, y)` from the +x axis, in [-π, π]. Signed zeros
/// and infinities follow C99 `atan2`.
pub fn atan2(y: f64, x: f64) -> f64 {
    if y.is_nan() || x.is_nan() {
        return f64::NAN;
    }
    let sign = |v: f64| if y.is_sign_negative() { -v } else { v };
    let toward_negative_x = x < 0.0 || (x == 0.0 && x.is_sign_negative());
    if y == 0.0 {
        return if toward_negative_x { sign(PI) } else { y };
    }
    if x.is_infinite() {
        return match (y.is_infinite(), x > 0.0) {
            (true, true) => sign(FRAC_PI_4),
            (true, false) => sign(3.0 * FRAC_PI_4),
            (false, true) => sign(0.0),
            (false, false) => sign(PI),
        };
    }
    if x == 0.0 || y.is_infinite() {
        return sign(FRAC_PI_2);
    }
    // The smaller over the larger, so the quotient never overflows or
    // underflows to zero where the angle is not.
    let (ay, ax) = (y.abs(), x.abs());
    let a = if ay <= ax {
        atan64(ay / ax)
    } else {
        FRAC_PI_2 - atan64(ax / ay)
    };
    sign(if x > 0.0 { a } else { PI - a })
}

/// Arcsine, in [-π/2, π/2]; NaN outside [-1, 1].
pub fn asin(x: f64) -> f64 {
    if !(-1.0..=1.0).contains(&x) {
        return f64::NAN;
    }
    atan2(x, ((1.0 - x) * (1.0 + x)).sqrt())
}

/// Arccosine, in [0, π]; NaN outside [-1, 1].
pub fn acos(x: f64) -> f64 {
    if !(-1.0..=1.0).contains(&x) {
        return f64::NAN;
    }
    atan2(((1.0 - x) * (1.0 + x)).sqrt(), x)
}

/// e^x: infinity above ln(f64::MAX), zero below the smallest
/// subnormal's log. Cody–Waite by ln 2 and Taylor to r^14, as the f32
/// `exp`; the power of two is applied in two halves, so neither overflows
/// on its own near the ends of the range.
pub fn exp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if x > 709.782_712_893_383_9 {
        return f64::INFINITY;
    }
    if x < -745.133_219_101_941_1 {
        return 0.0;
    }
    let k = (x * LOG2_E + 0.5).floor();
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let p = horner(&INV_FACT[..15], r);
    let k = k as i64;
    let half = k / 2;
    let two = |e: i64| f64::from_bits(((e + 1023) as u64) << 52);
    p * two(half) * two(k - half)
}

/// Natural logarithm: `ln(0) = -inf`, NaN for a negative `x`.
pub fn ln(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }
    if x.is_infinite() {
        return x;
    }
    if x < f64::MIN_POSITIVE {
        // A subnormal, scaled into the normals by 2^54.
        return ln64(x * f64::from_bits((1023 + 54) << 52)) - 54.0 * std::f64::consts::LN_2;
    }
    ln64(x)
}

fn is_integer(y: f64) -> bool {
    y.floor() == y
}

/// Odd integers only exist below 2^53 in f64.
fn is_odd_integer(y: f64) -> bool {
    is_integer(y) && y.abs() < 9_007_199_254_740_992.0 && (y as i64) & 1 == 1
}

/// x^y. Special cases follow C99 `pow`, as [`super::pow`] does.
pub fn pow(x: f64, y: f64) -> f64 {
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    let odd = is_odd_integer(y);
    if y.is_infinite() {
        let a = x.abs();
        return if a == 1.0 {
            1.0
        } else if (a < 1.0) == (y > 0.0) {
            0.0
        } else {
            f64::INFINITY
        };
    }
    if x == 0.0 || x.is_infinite() {
        let big = (x == 0.0) == (y < 0.0);
        let magnitude = if big { f64::INFINITY } else { 0.0 };
        return if odd && x.is_sign_negative() {
            -magnitude
        } else {
            magnitude
        };
    }
    if x < 0.0 && !is_integer(y) {
        return f64::NAN;
    }
    let r = exp(y * ln(x.abs()));
    if x < 0.0 && odd { -r } else { r }
}

/// The cube root (negative for a negative `x`): a first guess from the
/// exponent bits, then Newton steps.
pub fn cbrt(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    let a = x.abs();
    let (a, unscale) = if a < f64::MIN_POSITIVE {
        (
            a * f64::from_bits((1023 + 54) << 52),
            f64::from_bits((1023 - 18) << 52),
        )
    } else {
        (a, 1.0)
    };
    let mut y = f64::from_bits(a.to_bits() / 3 + (1023 << 52) / 3 * 2);
    for _ in 0..6 {
        y -= (y * y * y - a) / (3.0 * y * y);
    }
    let y = y * unscale;
    if x < 0.0 { -y } else { y }
}

/// `sqrt(x² + y²)`, scaled so it neither overflows nor underflows early.
pub fn hypot(x: f64, y: f64) -> f64 {
    if x.is_infinite() || y.is_infinite() {
        return f64::INFINITY;
    }
    let (a, b) = (x.abs().max(y.abs()), x.abs().min(y.abs()));
    if a == 0.0 || a.is_nan() || b.is_nan() {
        return a + b;
    }
    let r = b / a;
    a * (1.0 + r * r).sqrt()
}
