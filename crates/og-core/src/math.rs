//! Math: `glam` 0.34 re-exported, plus the engine's own deterministic
//! transcendental functions in `f32`.
//!
//! [`sin`], [`cos`], [`sin_cos`], [`atan`], [`atan2`], [`exp`], [`ln`] and
//! [`pow`] give bit-identical results on every host (wasm and native, x86_64
//! and arm64), because they are built only from operations IEEE 754 defines
//! exactly: `+ - * /`, comparisons, `floor`, integer arithmetic and bit
//! casts. They never call the host's `libm` (`f32::sin` and friends do, and
//! their results differ between hosts), never use fused multiply-add, and
//! evaluate in `f64` before one final rounding to `f32`, so they are within
//! 1 ulp of the exact value (almost always correctly rounded).
//!
//! Simulation code uses these, never `f32::sin`/`exp`/`powf`/`powi`.
//!
//! glam is built with its `scalar-math` feature, set once in the workspace
//! `Cargo.toml`: every glam type (`Vec3A`, `Vec4`, `Quat`, `Mat3A`, `Mat4`,
//! `Affine3A`, ...) uses the same scalar code, in the same operation order, on
//! SSE2, NEON and wasm (with or without `simd128`), so their arithmetic
//! (`dot`, `length`, `normalize`, quaternion and matrix products, transforms)
//! is bit-identical across hosts. Without it, glam's SIMD backends sum lanes
//! in a different order per target. `tests/glam_scalar.rs` checks those
//! results against an explicit scalar reference. A consequence: `Vec4`,
//! `Quat` and `Mat4` have the alignment of `f32` (`Vec3A` and `Mat3A` keep
//! 16); GPU structs that need 16-byte alignment say so themselves.
//!
//! Rule: simulation transcendentals go only through this module. glam's own
//! trigonometry and exponentials (`Quat::from_axis_angle`, `from_euler`,
//! `slerp`, `Mat4::from_rotation_*`, `Vec2::from_angle`, `powf`, ...) call the
//! host `libm` even with `scalar-math`, so they are for rendering, which never
//! feeds back into the simulation; deterministic state builds rotations with
//! [`quat_from_axis_angle`] and uses [`sin`], [`cos`], [`atan2`], [`exp`],
//! [`ln`] and [`pow`]. `sqrt` (inside `length` and `normalize`) is exactly
//! rounded by IEEE 754 and is safe.

pub use glam::*;

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_6, PI};

// π/2 split for Cody–Waite reduction: A and B have 28 significant bits, so
// k·A and k·B are exact for |k| < 2^24; C is the rounded remainder (computed
// from π to 600 bits).
const PIO2_A: f64 = f64::from_bits(0x3ff9_21fb_5400_0000);
const PIO2_B: f64 = f64::from_bits(0x3e11_0b46_1000_0000);
const PIO2_C: f64 = f64::from_bits(0x3c5a_6263_3145_c06e);
const FRAC_2_PI: f64 = std::f64::consts::FRAC_2_PI;
const TWO_POW_M128: f64 = f64::from_bits((1023 - 128) << 52);

/// Bits of 2/π as a 256-bit fixed-point number whose first bit is the
/// integer bit (0): `floor(2/π · 2^255)`, most significant word first.
const TWO_OVER_PI: [u64; 4] = [
    0x517c_c1b7_2722_0a94,
    0xfe13_abe8_fa9a_6ee0,
    0x6db1_4acc_9e21_c820,
    0xff28_b1d5_ef5d_e2b0,
];

// ln 2 split (fdlibm): k·LN2_HI is exact for |k| < 2^21.
const LN2_HI: f64 = f64::from_bits(0x3fe6_2e42_fee0_0000);
const LN2_LO: f64 = f64::from_bits(0x3dea_39ef_3579_3c76);
const LOG2_E: f64 = std::f64::consts::LOG2_E;
const SQRT_2: f64 = std::f64::consts::SQRT_2;
const SQRT_3: f64 = 1.732_050_807_568_877_2;
const TAN_PI_12: f64 = 0.267_949_192_431_122_7;

/// Evaluates `c[0] + x·(c[1] + x·(c[2] + ...))`, innermost first. The
/// same operations in the same order on every host.
fn horner(c: &[f64], x: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, &k| acc * x + k)
}

/// `1/n!` for n = 0..=18, exact divisions of exactly representable integers.
const fn inverse_factorials() -> [f64; 19] {
    let mut out = [0.0; 19];
    let mut fact = 1.0;
    let mut n = 0;
    while n < 19 {
        if n > 0 {
            fact *= n as f64;
        }
        out[n] = 1.0 / fact;
        n += 1;
    }
    out
}

const INV_FACT: [f64; 19] = inverse_factorials();

/// Coefficients of z = r² in sin(r)/r - 1, to r^17 (truncation < 5e-17 on
/// |r| <= π/4).
const SIN_P: [f64; 8] = {
    let mut c = [0.0; 8];
    let mut k = 0;
    while k < 8 {
        let v = INV_FACT[2 * k + 3];
        c[k] = if k % 2 == 0 { -v } else { v };
        k += 1;
    }
    c
};

/// Coefficients of z = r² in cos(r) - 1, divided by z, to r^18.
const COS_P: [f64; 9] = {
    let mut c = [0.0; 9];
    let mut k = 0;
    while k < 9 {
        let v = INV_FACT[2 * k + 2];
        c[k] = if k % 2 == 0 { -v } else { v };
        k += 1;
    }
    c
};

/// sin(r) for |r| ≲ π/4.
fn sin_kernel(r: f64) -> f64 {
    let z = r * r;
    r + r * z * horner(&SIN_P, z)
}

/// cos(r) for |r| ≲ π/4.
fn cos_kernel(r: f64) -> f64 {
    let z = r * r;
    1.0 + z * horner(&COS_P, z)
}

/// 128 bits of [`TWO_OVER_PI`] starting at bit `start` (0 = the integer bit).
fn two_over_pi_window(start: u32) -> u128 {
    let word = (start / 64) as usize;
    let shift = start % 64;
    let hi = (u128::from(TWO_OVER_PI[word]) << 64) | u128::from(TWO_OVER_PI[word + 1]);
    if shift == 0 {
        hi
    } else {
        let next = TWO_OVER_PI.get(word + 2).copied().unwrap_or(0);
        (hi << shift) | u128::from(next >> (64 - shift))
    }
}

/// Reduces a finite `a >= 0` to `r` in about [-π/4, π/4] and the quadrant
/// `q` (mod 4) with `a = r + q·π/2 (mod 2π)`.
fn reduce(a: f32) -> (f64, u32) {
    if a < 16_777_216.0 {
        // Cody–Waite: k < 2^24, so k·A and k·B are exact.
        let x = f64::from(a);
        let k = (x * FRAC_2_PI + 0.5).floor();
        let r = ((x - k * PIO2_A) - k * PIO2_B) - k * PIO2_C;
        return (r, (k as u64 & 3) as u32);
    }
    // Payne–Hanek: a = m·2^e with a 24-bit integer m and e >= 1. Bits of
    // 2/π above position e-1 contribute multiples of 4 and are skipped; the
    // product of m with the next 128 bits, modulo 2^128, is (a·2/π mod 4)
    // in fixed point with 126 fraction bits.
    let bits = a.to_bits();
    let e = ((bits >> 23) & 0xff) as i32 - 150;
    let m = u128::from((bits & 0x7f_ffff) | 0x80_0000);
    let y = m.wrapping_mul(two_over_pi_window((e - 1) as u32));
    let mut q = (y >> 126) as u32;
    let mut f = (y << 2) as f64 * TWO_POW_M128;
    if f >= 0.5 {
        f -= 1.0;
        q += 1;
    }
    (f * FRAC_PI_2, q & 3)
}

/// Sine of `x` radians, deterministic on every host.
pub fn sin(x: f32) -> f32 {
    sin_cos(x).0
}

/// Cosine of `x` radians, deterministic on every host.
pub fn cos(x: f32) -> f32 {
    sin_cos(x).1
}

/// `(sin x, cos x)`, deterministic on every host. NaN or infinite input gives
/// NaN for both.
pub fn sin_cos(x: f32) -> (f32, f32) {
    if !x.is_finite() {
        return (f32::NAN, f32::NAN);
    }
    let (r, q) = reduce(x.abs());
    let (s, c) = (sin_kernel(r), cos_kernel(r));
    let (s, c) = match q {
        0 => (s, c),
        1 => (c, -s),
        2 => (-s, -c),
        _ => (-c, s),
    };
    let s = if x.is_sign_negative() { -s } else { s };
    (s as f32, c as f32)
}

/// Coefficients of z = t² in atan(t)/t - 1 divided by z: -1/3, 1/5, -1/7, ... to t^27.
const ATAN_P: [f64; 13] = {
    let mut c = [0.0; 13];
    let mut k = 0;
    while k < 13 {
        let v = 1.0 / (2 * k + 3) as f64;
        c[k] = if k % 2 == 0 { -v } else { v };
        k += 1;
    }
    c
};

/// atan in f64 for any input: reduce to |t| <= tan(π/12), then a Taylor
/// series to t^27.
fn atan64(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let negative = x.is_sign_negative();
    let mut t = x.abs();
    let inverted = t > 1.0;
    if inverted {
        t = 1.0 / t;
    }
    let mut base = 0.0;
    if t > TAN_PI_12 {
        t = (t * SQRT_3 - 1.0) / (SQRT_3 + t);
        base = FRAC_PI_6;
    }
    let z = t * t;
    let p = horner(&ATAN_P, z);
    let mut r = base + (t + t * z * p);
    if inverted {
        r = FRAC_PI_2 - r;
    }
    if negative {
        -r
    } else {
        r
    }
}

/// Arctangent of `x`, in [-π/2, π/2], deterministic on every host.
pub fn atan(x: f32) -> f32 {
    atan64(f64::from(x)) as f32
}

/// Angle of the point `(x, y)` from the +x axis, in [-π, π], deterministic
/// on every host. Signed zeros and infinities follow C99 `atan2`.
pub fn atan2(y: f32, x: f32) -> f32 {
    if y.is_nan() || x.is_nan() {
        return f32::NAN;
    }
    let sign = |v: f64| if y.is_sign_negative() { -v } else { v };
    let toward_negative_x = x < 0.0 || (x == 0.0 && x.is_sign_negative());
    let r = if y == 0.0 {
        if toward_negative_x {
            sign(PI)
        } else {
            return y;
        }
    } else if x.is_infinite() {
        match (y.is_infinite(), x > 0.0) {
            (true, true) => sign(FRAC_PI_4),
            (true, false) => sign(3.0 * FRAC_PI_4),
            (false, true) => sign(0.0),
            (false, false) => sign(PI),
        }
    } else if x == 0.0 || y.is_infinite() {
        sign(FRAC_PI_2)
    } else {
        let a = atan64((f64::from(y) / f64::from(x)).abs());
        sign(if x > 0.0 { a } else { PI - a })
    };
    r as f32
}

/// e^x in f64 for |x| <= 200: Cody–Waite by ln 2, Taylor to r^14.
fn exp64(x: f64) -> f64 {
    let k = (x * LOG2_E + 0.5).floor();
    let r = (x - k * LN2_HI) - k * LN2_LO;
    let p = horner(&INV_FACT[..15], r);
    let scale = f64::from_bits(((k as i64 + 1023) as u64) << 52);
    p * scale
}

/// e^x, deterministic on every host.
pub fn exp(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }
    exp64(f64::from(x).clamp(-200.0, 200.0)) as f32
}

/// 1/3, 1/5, ... 1/23: the atanh series of ln, divided by s³.
const LN_P: [f64; 11] = {
    let mut c = [0.0; 11];
    let mut k = 0;
    while k < 11 {
        c[k] = 1.0 / (2 * k + 3) as f64;
        k += 1;
    }
    c
};

/// ln of a positive, finite, normal f64: x = m·2^e with m in [√½, √2], then
/// ln m = 2·atanh((m-1)/(m+1)) by its series to s^23.
fn ln64(x: f64) -> f64 {
    let bits = x.to_bits();
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | (1023 << 52));
    if m > SQRT_2 {
        m *= 0.5;
        e += 1;
    }
    let s = (m - 1.0) / (m + 1.0);
    let z = s * s;
    let p = horner(&LN_P, z);
    let ln_m = 2.0 * s + 2.0 * (s * z * p);
    let e = e as f64;
    e * LN2_HI + (ln_m + e * LN2_LO)
}

/// Natural logarithm, deterministic on every host. `ln(0) = -inf`,
/// negative input gives NaN.
pub fn ln(x: f32) -> f32 {
    if x.is_nan() || x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 {
        return f32::NEG_INFINITY;
    }
    if x.is_infinite() {
        return x;
    }
    ln64(f64::from(x)) as f32
}

fn is_integer(y: f32) -> bool {
    y.floor() == y
}

/// Odd integers only exist below 2^24 in f32.
fn is_odd_integer(y: f32) -> bool {
    is_integer(y) && y.abs() < 16_777_216.0 && (y as i64) & 1 == 1
}

/// x^y, deterministic on every host. Special cases follow C99 `pow`:
/// `pow(x, ±0) = 1`, `pow(1, y) = 1`, a negative base needs an integer
/// exponent (otherwise NaN), and zeros and infinities keep the sign of an
/// odd integer exponent.
pub fn pow(x: f32, y: f32) -> f32 {
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if x.is_nan() || y.is_nan() {
        return f32::NAN;
    }
    let odd = is_odd_integer(y);
    if y.is_infinite() {
        let a = x.abs();
        return if a == 1.0 {
            1.0
        } else if (a < 1.0) == (y > 0.0) {
            0.0
        } else {
            f32::INFINITY
        };
    }
    if x == 0.0 || x.is_infinite() {
        // |x|^y is 0 or inf; the sign survives only for odd integer y.
        let big = (x == 0.0) == (y < 0.0);
        let magnitude = if big { f32::INFINITY } else { 0.0 };
        return if odd && x.is_sign_negative() {
            -magnitude
        } else {
            magnitude
        };
    }
    if x < 0.0 && !is_integer(y) {
        return f32::NAN;
    }
    let t = (f64::from(y) * ln64(f64::from(x.abs()))).clamp(-200.0, 200.0);
    let r = exp64(t) as f32;
    if x < 0.0 && odd {
        -r
    } else {
        r
    }
}

/// The rotation of `angle` radians about the unit vector `axis`, built with
/// the deterministic [`sin_cos`] (glam's `Quat::from_axis_angle` uses the
/// host `libm`).
pub fn quat_from_axis_angle(axis: Vec3, angle: f32) -> Quat {
    let (s, c) = sin_cos(angle * 0.5);
    Quat::from_xyzw(axis.x * s, axis.y * s, axis.z * s, c)
}
