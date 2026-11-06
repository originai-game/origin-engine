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
//! [`quat_from_axis_angle`] and [`quat_slerp`] and uses [`sin`], [`cos`],
//! [`atan2`], [`acos`], [`exp`], [`ln`] and [`pow`]. `sqrt` (inside `length`
//! and `normalize`) is exactly rounded by IEEE 754 and is safe. The root
//! `clippy.toml` holds the rule: its `disallowed-methods` refuses those host
//! calls in every crate that does not opt out in its own `clippy.toml` (the
//! renderer, og-ui, audio, the cook, the bake, the og tool, the engine tests).
//! Render-only code in a simulation crate takes a scoped `#[allow]`.

pub use glam::*;

/// Split a height-field cell along h00--h11 when that diagonal's height
/// difference is no greater than h10--h01's. Ties keep the original split.
pub fn heightfield_zigzag(h00: f32, h10: f32, h01: f32, h11: f32) -> bool {
    (h00 - h11).abs() <= (h10 - h01).abs()
}

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
    if negative { -r } else { r }
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
    if x < 0.0 && odd { -r } else { r }
}

/// The rotation of `angle` radians about the unit vector `axis`, built with
/// the deterministic [`sin_cos`] (glam's `Quat::from_axis_angle` uses the
/// host `libm`).
pub fn quat_from_axis_angle(axis: Vec3, angle: f32) -> Quat {
    let (s, c) = sin_cos(angle * 0.5);
    Quat::from_xyzw(axis.x * s, axis.y * s, axis.z * s, c)
}

/// `p + offset`, and `p` bit for bit while `offset` is zero: a sum would
/// turn a -0.0 lane into 0.0, so a frame offset that og.WorldOrigin never
/// moved (R12) would change a world's bytes. (`p - offset` needs no such
/// care: subtracting 0.0 keeps every bit.)
pub fn offset(p: Vec3, offset: Vec3) -> Vec3 {
    if offset == Vec3::ZERO { p } else { p + offset }
}

/// The arc cosine of `x` in `[0, pi]`, from [`atan2`] and `sqrt` (which IEEE
/// 754 rounds exactly): `atan2(sqrt(1 - x^2), x)` in `f64`. NaN outside
/// `[-1, 1]`, as `f32::acos`.
pub fn acos(x: f32) -> f32 {
    if !(-1.0..=1.0).contains(&x) {
        return f32::NAN;
    }
    let x = f64::from(x);
    let y = ((1.0 - x) * (1.0 + x)).sqrt();
    let a = atan64(y / x.abs());
    (if x >= 0.0 { a } else { PI - a }) as f32
}

/// The spherical interpolation from `a` to `b` at `t` (0 gives `a`, 1 `b`),
/// along the shorter arc, built from the deterministic [`acos`] and [`sin`]:
/// glam's `Quat::slerp` uses the host `libm`, so its results differ by an ulp
/// between hosts and must not reach simulation state. Nearly equal rotations
/// fall back to a normalised lerp, as glam's do.
pub fn quat_slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let mut end = b;
    let mut dot = a.dot(b);
    if dot < 0.0 {
        end = -end;
        dot = -dot;
    }
    if dot > 1.0 - f32::EPSILON {
        return a.lerp(end, t);
    }
    let theta = acos(dot);
    let s1 = sin(theta * (1.0 - t));
    let s2 = sin(theta * t);
    (a * s1 + end * s2) * (1.0 / sin(theta))
}

/// A transform as `og.Transform` holds it: position, rotation and scale
/// (three.js Object3D's `position`, `quaternion` and `scale`).
pub type Pose = (Vec3, Quat, Vec3);

/// The world pose of `node`, whose own transform is `local`: its parents'
/// transforms applied down the parent chain (`parent_of`), as the renderer
/// composes `og.MatrixWorld` from `og.Transform` down `og.Parent`. One
/// helper for every simulation system that places a child (physics' soft
/// bodies, audio's sources, listener and zones), so they agree bit for bit:
/// plain products in a fixed order, no transcendental.
///
/// A parent's scale scales the child's offset and multiplies its scale
/// (exact for uniform scales; a matrix with shear has no pose). A parent
/// without a transform (`pose_of` gives `None`) ends the chain there, and
/// so does a chain longer than 256 (a cycle).
pub fn world_pose<K: Copy>(
    node: K,
    local: Pose,
    parent_of: impl Fn(K) -> Option<K>,
    pose_of: impl Fn(K) -> Option<Pose>,
) -> Pose {
    let (mut p, mut q, mut s) = local;
    let mut cur = node;
    for _ in 0..256 {
        let Some(parent) = parent_of(cur) else {
            break;
        };
        let Some((position, quaternion, scale)) = pose_of(parent) else {
            break;
        };
        p = position + quaternion * (scale * p);
        q = (quaternion * q).normalize();
        s *= scale;
        cur = parent;
    }
    (p, q, s)
}

#[cfg(test)]
mod math_contracts {
    //! Deterministic math on fixed inputs.
    //!
    //! The vectors are the correctly rounded f32 results, produced independently
    //! of this crate (CPython's `math` in f64, then rounded once to f32). Every
    //! host must reproduce them bit for bit; that is the cross-host contract.

    use crate::Rng;
    use crate::math::{
        Quat, Vec3, acos, atan, atan2, cos, exp, ln, pow, quat_from_axis_angle, quat_slerp, sin,
        sin_cos,
    };

    const SIN: &[(u32, u32)] = &[
        (0x00000000, 0x00000000), // sin(0.0)
        (0x0da24260, 0x0da24260), // sin(1.0000000031710769e-30)
        (0x3727c5ac, 0x3727c5ac), // sin(9.999999747378752e-06)
        (0x3f000000, 0x3ef57744), // sin(0.5)
        (0x3f800000, 0x3f576aa4), // sin(1.0)
        (0xbf800000, 0xbf576aa4), // sin(-1.0)
        (0x3f490fdb, 0x3f3504f3), // sin(0.7853981852531433)
        (0x3fc90fdb, 0x3f800000), // sin(1.5707963705062866)
        (0x40490fdb, 0xb3bbbd2e), // sin(3.1415927410125732)
        (0x40000000, 0x3f68c7b7), // sin(2.0)
        (0x40400000, 0x3e1081c3), // sin(3.0)
        (0xc0200000, 0xbf193578), // sin(-2.5)
        (0x41200000, 0xbf0b44f8), // sin(10.0)
        (0x42c80000, 0xbf01a12e), // sin(100.0)
        (0x447a0000, 0x3f53ae61), // sin(1000.0)
        (0x4640e6b6, 0xbf344b08), // sin(12345.677734375)
        (0x47c35000, 0x3d126d55), // sin(100000.0)
        (0x49742400, 0xbeb33259), // sin(1000000.0)
        (0x4affffff, 0x3f4fc8cd), // sin(8388607.5)
        (0x4b7fffff, 0xbf72bf60), // sin(16777215.0)
        (0x4b800000, 0xbf47917c), // sin(16777216.0)
        (0x4b189680, 0x3ed7520a), // sin(10000000.0)
        (0x4e26e49c, 0x3f5bcbf6), // sin(700000000.0)
        (0x501502f9, 0xbef99a64), // sin(10000000000.0)
        (0x60ad78ec, 0x3f281569), // sin(1.0000000200408773e+20)
        (0x7f61b1e6, 0x3f5ff9c4), // sin(3.0000000054977558e+38)
        (0xff61b1e6, 0xbf5ff9c4), // sin(-3.0000000054977558e+38)
    ];
    const COS: &[(u32, u32)] = &[
        (0x00000000, 0x3f800000), // cos(0.0)
        (0x0da24260, 0x3f800000), // cos(1.0000000031710769e-30)
        (0x3727c5ac, 0x3f800000), // cos(9.999999747378752e-06)
        (0x3f000000, 0x3f60a940), // cos(0.5)
        (0x3f800000, 0x3f0a5140), // cos(1.0)
        (0xbf800000, 0x3f0a5140), // cos(-1.0)
        (0x3f490fdb, 0x3f3504f3), // cos(0.7853981852531433)
        (0x3fc90fdb, 0xb33bbd2e), // cos(1.5707963705062866)
        (0x40490fdb, 0xbf800000), // cos(3.1415927410125732)
        (0x40000000, 0xbed51133), // cos(2.0)
        (0x40400000, 0xbf7d7026), // cos(3.0)
        (0xc0200000, 0xbf4d17bf), // cos(-2.5)
        (0x41200000, 0xbf56cd64), // cos(10.0)
        (0x42c80000, 0x3f5cc0ee), // cos(100.0)
        (0x447a0000, 0x3f0ff813), // cos(1000.0)
        (0x4640e6b6, 0x3f35be20), // cos(12345.677734375)
        (0x47c35000, 0xbf7fd61c), // cos(100000.0)
        (0x49742400, 0x3f6fcefd), // cos(1000000.0)
        (0x4affffff, 0xbf1589c3), // cos(8388607.5)
        (0x4b7fffff, 0xbea29962), // cos(16777215.0)
        (0x4b800000, 0x3f2056b4), // cos(16777216.0)
        (0x4b189680, 0xbf6842df), // cos(10000000.0)
        (0x4e26e49c, 0x3f033ede), // cos(700000000.0)
        (0x501502f9, 0x3f5f84c5), // cos(10000000000.0)
        (0x60ad78ec, 0x3f411723), // cos(1.0000000200408773e+20)
        (0x7f61b1e6, 0xbef7f57c), // cos(3.0000000054977558e+38)
        (0xff61b1e6, 0xbef7f57c), // cos(-3.0000000054977558e+38)
    ];
    const ATAN: &[(u32, u32)] = &[
        (0x00000000, 0x00000000), // atan(0.0)
        (0x1e3ce508, 0x1e3ce508), // atan(9.999999682655225e-21)
        (0x3dcccccd, 0x3dcc1f14), // atan(0.10000000149011612)
        (0x3e892a30, 0x3e86048d), // atan(0.2678999900817871)
        (0x3e99999a, 0x3e9539d4), // atan(0.30000001192092896)
        (0x3f000000, 0x3eed6338), // atan(0.5)
        (0x3f13cd36, 0x3f060a8f), // atan(0.5773500204086304)
        (0x3f800000, 0x3f490fdb), // atan(1.0)
        (0xbf800000, 0xbf490fdb), // atan(-1.0)
        (0x3fc00000, 0x3f7b985f), // atan(1.5)
        (0x40000000, 0x3f8db70d), // atan(2.0)
        (0x41200000, 0x3fbc4de9), // atan(10.0)
        (0xc2f6cccd, 0xbfc80651), // atan(-123.4000015258789)
        (0x501502f9, 0x3fc90fdb), // atan(10000000000.0)
        (0x7f61b1e6, 0x3fc90fdb), // atan(3.0000000054977558e+38)
    ];
    const ATAN2: &[(u32, u32, u32)] = &[
        (0x3f800000, 0x3f800000, 0x3f490fdb), // atan2(1.0, 1.0)
        (0x3f800000, 0xbf800000, 0x4016cbe4), // atan2(1.0, -1.0)
        (0xbf800000, 0xbf800000, 0xc016cbe4), // atan2(-1.0, -1.0)
        (0xbf800000, 0x3f800000, 0xbf490fdb), // atan2(-1.0, 1.0)
        (0x3f000000, 0x40000000, 0x3e7adbb0), // atan2(0.5, 2.0)
        (0x40400000, 0xbe800000, 0x3fd3b43a), // atan2(3.0, -0.25)
        (0x0da24260, 0x3f800000, 0x0da24260), // atan2(1.0000000031710769e-30, 1.0)
        (0x7149f2ca, 0x0da24260, 0x3fc90fdb), // atan2(1.0000000150474662e+30, 1.0000000031710769e-30)
        (0xc0000000, 0x3a83126f, 0xbfc8ff78), // atan2(-2.0, 0.0010000000474974513)
        (0x40e00000, 0x40e000d2, 0x3f490f63), // atan2(7.0, 7.000100135803223)
        (0xb8d1b717, 0xc0a00000, 0xc0490f87), // atan2(-9.999999747378752e-05, -5.0)
        (0x42f6e979, 0xc4454000, 0x403f20d8), // atan2(123.45600128173828, -789.0)
    ];
    const EXP: &[(u32, u32)] = &[
        (0x00000000, 0x3f800000), // exp(0.0)
        (0x322bcc77, 0x3f800000), // exp(9.99999993922529e-09)
        (0x3f000000, 0x3fd3094c), // exp(0.5)
        (0x3f800000, 0x402df854), // exp(1.0)
        (0xbf800000, 0x3ebc5ab2), // exp(-1.0)
        (0x40135d8d, 0x411ffffe), // exp(2.3025848865509033)
        (0x41200000, 0x46ac14ee), // exp(10.0)
        (0xc1200000, 0x383e6bce), // exp(-10.0)
        (0x42480000, 0x638c881f), // exp(50.0)
        (0x42b00000, 0x7ef882b7), // exp(88.0)
        (0x42b170a4, 0x7f7f4648), // exp(88.72000122070312)
        (0x42b20000, 0x7f800000), // exp(89.0)
        (0xc2ae0000, 0x00b33687), // exp(-87.0)
        (0xc2c80000, 0x0000001b), // exp(-100.0)
        (0xc2cfcccd, 0x00000001), // exp(-103.9000015258789)
        (0xc2dc0000, 0x00000000), // exp(-110.0)
    ];
    const LN: &[(u32, u32)] = &[
        (0x3f800000, 0x00000000), // ln(1.0)
        (0x40000000, 0x3f317218), // ln(2.0)
        (0x3f000000, 0xbf317218), // ln(0.5)
        (0x41200000, 0x40135d8e), // ln(10.0)
        (0x0da24260, 0xc28a27b5), // ln(1.0000000031710769e-30)
        (0x7f61b1e6, 0x42b13196), // ln(3.0000000054977558e+38)
        (0x3f800001, 0x33ffffff), // ln(1.0000001192092896)
        (0x3f7fffff, 0xb3800000), // ln(0.9999999403953552)
        (0x00000001, 0xc2ce8ed0), // ln(1.401298464324817e-45)
        (0x40e00000, 0x3ff91395), // ln(7.0)
        (0x3fb504f3, 0x3eb17217), // ln(1.4142135381698608)
        (0x47f12065, 0x413b940e), // ln(123456.7890625)
    ];
    const POW: &[(u32, u32, u32)] = &[
        (0x40000000, 0x41200000, 0x44800000), // pow(2.0, 10.0)
        (0x40000000, 0x3f000000, 0x3fb504f3), // pow(2.0, 0.5)
        (0x41200000, 0xc0400000, 0x3a83126f), // pow(10.0, -3.0)
        (0x3f000000, 0x40533333, 0x3dcfefc6), // pow(0.5, 3.299999952316284)
        (0x3f800347, 0x461c4000, 0x402dfd7e), // pow(1.000100016593933, 10000.0)
        (0xc0000000, 0x40400000, 0xc1000000), // pow(-2.0, 3.0)
        (0xc0000000, 0x40000000, 0x40800000), // pow(-2.0, 2.0)
        (0x40400000, 0x3eaaaaaa, 0x3fb89ba2), // pow(3.0, 0.3333333134651184)
        (0x501502f9, 0x40400000, 0x7149f2ca), // pow(10000000000.0, 3.0)
        (0x40e00000, 0xc2200000, 0x0750c653), // pow(7.0, -40.0)
        (0x3f666666, 0x42c90000, 0x37d360ea), // pow(0.8999999761581421, 100.5)
        (0x40000000, 0xc3150000, 0x00000001), // pow(2.0, -149.0)
        (0x3fc00000, 0xc0100000, 0x3ecd9ea2), // pow(1.5, -2.25)
        (0xbf000000, 0xc0400000, 0xc1000000), // pow(-0.5, -3.0)
    ];

    const ACOS: &[(u32, u32)] = &[
        (0x3f800000, 0x00000000), // acos(1.0)
        (0xbf800000, 0x40490fdb), // acos(-1.0)
        (0x00000000, 0x3fc90fdb), // acos(0.0)
        (0x3f000000, 0x3f860a92), // acos(0.5)
        (0xbf000000, 0x40060a92), // acos(-0.5)
        (0x3f7ffffe, 0x3a000000), // acos(0.9999998807907104)
        (0x3f3504f3, 0x3f490fdb), // acos(0.7071067690849304)
        (0xbe800000, 0x3fe967ae), // acos(-0.25)
        (0x3dcccccd, 0x3fbc3d91), // acos(0.10000000149011612)
        (0x3f733333, 0x3ea29749), // acos(0.949999988079071)
        (0x33d6bf95, 0x3fc90fda), // acos(1.0000000116860974e-07)
    ];

    fn f(bits: u32) -> f32 {
        f32::from_bits(bits)
    }

    fn check1(name: &str, table: &[(u32, u32)], fun: fn(f32) -> f32) {
        for &(x, want) in table {
            let got = fun(f(x)).to_bits();
            assert_eq!(
                got,
                want,
                "{name}({:e}) = {:e} ({got:#010x}), expected {:e} ({want:#010x})",
                f(x),
                f(got),
                f(want)
            );
        }
    }

    fn check2(name: &str, table: &[(u32, u32, u32)], fun: fn(f32, f32) -> f32) {
        for &(a, b, want) in table {
            let got = fun(f(a), f(b)).to_bits();
            assert_eq!(
                got,
                want,
                "{name}({:e}, {:e}) = {:e} ({got:#010x}), expected {:e} ({want:#010x})",
                f(a),
                f(b),
                f(got),
                f(want)
            );
        }
    }

    #[test]
    fn sin_and_cos_match_the_vectors() {
        check1("sin", SIN, sin);
        check1("cos", COS, cos);
    }

    #[test]
    fn atan_and_atan2_match_the_vectors() {
        check1("atan", ATAN, atan);
        check2("atan2", ATAN2, atan2);
        check1("acos", ACOS, acos);
        assert!(acos(1.5).is_nan() && acos(f32::NAN).is_nan());
    }

    #[test]
    fn exp_ln_pow_match_the_vectors() {
        check1("exp", EXP, exp);
        check1("ln", LN, ln);
        check2("pow", POW, pow);
    }

    fn bits(x: f32) -> u32 {
        x.to_bits()
    }

    #[test]
    fn special_values_follow_c99() {
        use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
        let (inf, nan) = (f32::INFINITY, f32::NAN);
        assert_eq!(bits(sin(-0.0)), bits(-0.0));
        assert_eq!(bits(cos(-0.0)), bits(1.0));
        assert!(sin(inf).is_nan() && cos(-inf).is_nan() && sin(nan).is_nan());
        assert_eq!(sin_cos(0.25), (sin(0.25), cos(0.25)));

        assert_eq!(bits(atan(-0.0)), bits(-0.0));
        assert_eq!(atan(inf), FRAC_PI_2);
        assert_eq!(atan(-inf), -FRAC_PI_2);
        assert_eq!(bits(atan2(0.0, 1.0)), bits(0.0));
        assert_eq!(bits(atan2(-0.0, 1.0)), bits(-0.0));
        assert_eq!(atan2(0.0, -1.0), PI);
        assert_eq!(atan2(-0.0, -1.0), -PI);
        assert_eq!(atan2(0.0, -0.0), PI);
        assert_eq!(bits(atan2(0.0, 0.0)), bits(0.0));
        assert_eq!(atan2(2.0, 0.0), FRAC_PI_2);
        assert_eq!(atan2(-2.0, -0.0), -FRAC_PI_2);
        assert_eq!(atan2(inf, inf), FRAC_PI_4);
        assert_eq!(atan2(-inf, -inf), -3.0 * FRAC_PI_4);
        assert_eq!(atan2(1.0, -inf), PI);
        assert_eq!(bits(atan2(-1.0, inf)), bits(-0.0));
        assert_eq!(atan2(inf, 5.0), FRAC_PI_2);
        assert!(atan2(nan, 1.0).is_nan() && atan2(1.0, nan).is_nan());

        assert_eq!(exp(-inf), 0.0);
        assert_eq!(exp(inf), inf);
        assert!(exp(nan).is_nan());
        assert_eq!(ln(0.0), -inf);
        assert_eq!(ln(inf), inf);
        assert!(ln(-1.0).is_nan() && ln(nan).is_nan());

        assert_eq!(pow(nan, 0.0), 1.0);
        assert_eq!(pow(1.0, nan), 1.0);
        assert!(pow(nan, 2.0).is_nan() && pow(2.0, nan).is_nan());
        assert!(pow(-8.0, 1.0 / 3.0).is_nan());
        assert_eq!(pow(-1.0, inf), 1.0);
        assert_eq!(pow(0.5, inf), 0.0);
        assert_eq!(pow(0.5, -inf), inf);
        assert_eq!(pow(2.0, inf), inf);
        assert_eq!(pow(2.0, -inf), 0.0);
        assert_eq!(pow(0.0, -1.0), inf);
        assert_eq!(pow(-0.0, -3.0), -inf);
        assert_eq!(pow(-0.0, -2.0), inf);
        assert_eq!(bits(pow(-0.0, 3.0)), bits(-0.0));
        assert_eq!(bits(pow(-0.0, 2.0)), bits(0.0));
        assert_eq!(pow(-inf, 3.0), -inf);
        assert_eq!(bits(pow(-inf, -3.0)), bits(-0.0));
        assert_eq!(pow(inf, -1.0), 0.0);
        assert_eq!(pow(-2.0, 3.0), -8.0);
        assert_eq!(pow(-2.0, 1e9), inf);
    }

    fn ordered(x: f32) -> i64 {
        let b = x.to_bits();
        if b & 0x8000_0000 != 0 {
            -i64::from(b & 0x7fff_ffff)
        } else {
            i64::from(b)
        }
    }

    fn ulps(a: f32, b: f32) -> i64 {
        (ordered(a) - ordered(b)).abs()
    }

    /// Accuracy across ranges, against the host's f64 library rounded to f32:
    /// never more than 1 ulp, and correctly rounded in all but at most 1 in
    /// 10,000 results per function (the implementation measured 140,000 of
    /// 140,000).
    #[test]
    fn results_are_correctly_rounded_across_ranges() {
        let mut rng = Rng::new(2026);
        // Per function: (results, results not correctly rounded).
        let mut stats = std::collections::BTreeMap::<&str, (usize, usize)>::new();
        let mut tally = |d: i64, what: &'static str, x: f32, y: f32| {
            assert!(d <= 1, "{what}({x:e}, {y:e}) is {d} ulps off");
            let s = stats.entry(what).or_default();
            s.0 += 1;
            s.1 += usize::from(d != 0);
        };
        for &(lo, hi) in &[
            (-4.0, 4.0),
            (-100.0, 100.0),
            (-1e5, 1e5),
            (-3e7, 3e7),
            (-1e30, 1e30),
        ] {
            for _ in 0..4000 {
                let x = rng.range(lo, hi) as f32;
                let xd = f64::from(x);
                tally(ulps(sin(x), xd.sin() as f32), "sin", x, 0.0);
                tally(ulps(cos(x), xd.cos() as f32), "cos", x, 0.0);
                tally(ulps(atan(x), xd.atan() as f32), "atan", x, 0.0);
                let y = rng.range(lo, hi) as f32;
                tally(
                    ulps(atan2(y, x), f64::from(y).atan2(xd) as f32),
                    "atan2",
                    y,
                    x,
                );
            }
        }
        for _ in 0..20000 {
            let x = rng.range(-104.0, 89.0) as f32;
            tally(ulps(exp(x), f64::from(x).exp() as f32), "exp", x, 0.0);
            let p = f32::from_bits(rng.integer(1, 0x7f7f_ffff) as u32);
            tally(ulps(ln(p), f64::from(p).ln() as f32), "ln", p, 0.0);
            let base = rng.range(0.0, 50.0) as f32;
            let e = rng.range(-20.0, 20.0) as f32;
            tally(
                ulps(pow(base, e), f64::from(base).powf(f64::from(e)) as f32),
                "pow",
                base,
                e,
            );
        }
        for (what, (total, off)) in stats {
            assert!(
                off * 10_000 <= total,
                "{what}: {off} of {total} not correctly rounded"
            );
        }
    }

    #[test]
    fn deterministic_rotation_matches_its_definition() {
        let q = quat_from_axis_angle(Vec3::Y, 1.0);
        let (s, c) = sin_cos(0.5);
        assert_eq!((q.x, q.y, q.z, q.w), (0.0, s, 0.0, c));
    }

    /// `quat_slerp` turns along the arc by the share asked: from a turn of 0.3
    /// about +Y to one of 2.1, a quarter of the way is a turn of 0.75. Its bits
    /// are pinned, so a host whose results differ fails here (glam's `slerp`
    /// uses the host's `sinf` and did, by an ulp, on Windows).
    #[test]
    fn quat_slerp_turns_by_the_share_and_holds_its_bits() {
        let a = quat_from_axis_angle(Vec3::Y, 0.3);
        let b = quat_from_axis_angle(Vec3::Y, 2.1);
        let q = quat_slerp(a, b, 0.25);
        let want = quat_from_axis_angle(Vec3::Y, 0.75);
        assert!(same(q, want), "{q:?} vs {want:?}");
        let got = [q.x, q.y, q.z, q.w].map(f32::to_bits);
        assert_eq!(got, PINNED_SLERP, "{got:#010x?}");
        // The shorter arc: -b is the same rotation as b.
        assert!(same(quat_slerp(a, -b, 0.25), want));
        // The ends, and the lerp of nearly equal rotations.
        assert!(same(quat_slerp(a, b, 0.0), a));
        assert!(same(quat_slerp(a, b, 1.0), b));
        assert_eq!(
            quat_slerp(Quat::IDENTITY, Quat::IDENTITY, 0.5),
            Quat::IDENTITY
        );
    }

    const PINNED_SLERP: [u32; 4] = [0x00000000, 0x3ebb8813, 0x00000000, 0x3f6e35bf];

    /// The same rotation within 1e-6 per component (q and -q are one rotation).
    fn same(a: Quat, b: Quat) -> bool {
        (a - b).length().min((a + b).length()) < 1e-6
    }

    /// `world_pose` down a chain of three: a grandparent moved and doubled, a
    /// parent turned a quarter about +Y, a child offset along +X. A missing
    /// transform and a cycle end the chain.
    #[test]
    fn world_pose_composes_down_the_parent_chain() {
        use crate::math::{Pose, Quat, world_pose};
        let quarter = quat_from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
        // 0: the child, 1: its parent, 2: the grandparent, 3: no transform.
        let parents = [Some(1usize), Some(2), None, None];
        let poses: [Option<Pose>; 4] = [
            None,
            Some((Vec3::new(0.0, 1.0, 0.0), quarter, Vec3::ONE)),
            Some((Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY, Vec3::splat(2.0))),
            None,
        ];
        let local = (Vec3::X, Quat::IDENTITY, Vec3::ONE);
        let (p, q, s) = world_pose(0, local, |c| parents[c], |c| poses[c]);
        // The parent turns +X to -Z and lifts it 1; the grandparent doubles
        // that and moves it 10 along +X.
        assert!((p - Vec3::new(10.0, 2.0, -2.0)).length() < 1e-5, "{p}");
        assert!(q.dot(quarter).abs() > 1.0 - 1e-6, "{q}");
        assert_eq!(s, Vec3::splat(2.0));
        // A root is its own pose, bit for bit.
        let root = world_pose(2, local, |_| None::<usize>, |c| poses[c]);
        assert_eq!(root, local);
        // A parent without a transform ends the chain.
        let (p, _, _) = world_pose(0, local, |c| (c == 0).then_some(3), |c| poses[c]);
        assert_eq!(p, Vec3::X);
        // A cycle ends after 256 steps instead of looping.
        let unit = (Vec3::new(0.0, 0.0, 1.0), Quat::IDENTITY, Vec3::ONE);
        let (p, _, _) = world_pose(0usize, local, |c| Some(1 - c), |_| Some(unit));
        assert_eq!(p, Vec3::new(1.0, 0.0, 256.0));
    }
}

#[cfg(test)]
mod glam_scalar_contracts {
    //! glam's vector, quaternion and matrix arithmetic is the same on every host.
    //!
    //! The workspace builds glam with `scalar-math`, so `Vec3A`, `Vec4`, `Quat`,
    //! `Mat3A`, `Mat4` and `Affine3A` run one scalar implementation everywhere.
    //! Each result below is compared bit for bit with an explicit reference in
    //! plain `f32`, written out in a fixed operation order (IEEE 754 makes each
    //! `+ - * / sqrt` exact-rounded, so the reference is the same on every
    //! host). glam's SIMD backends (SSE2, NEON, wasm `simd128`) sum the lanes in
    //! another order and fail this test; these are the types that diverged
    //! across hosts before `scalar-math`.

    use crate::Rng;
    use crate::math::{Affine3A, Mat3, Mat3A, Mat4, Quat, Vec3, Vec3A, Vec4};

    type V3 = [f32; 3];
    type V4 = [f32; 4];
    type M4 = [V4; 4]; // columns

    fn dot3(a: V3, b: V3) -> f32 {
        ((a[0] * b[0]) + (a[1] * b[1])) + (a[2] * b[2])
    }

    fn dot4(a: V4, b: V4) -> f32 {
        (((a[0] * b[0]) + (a[1] * b[1])) + (a[2] * b[2])) + (a[3] * b[3])
    }

    fn cross(a: V3, b: V3) -> V3 {
        [
            a[1] * b[2] - b[1] * a[2],
            a[2] * b[0] - b[2] * a[0],
            a[0] * b[1] - b[0] * a[1],
        ]
    }

    fn scale3(a: V3, s: f32) -> V3 {
        [a[0] * s, a[1] * s, a[2] * s]
    }

    fn add3(a: V3, b: V3) -> V3 {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    fn scale4(a: V4, s: f32) -> V4 {
        [a[0] * s, a[1] * s, a[2] * s, a[3] * s]
    }

    fn add4(a: V4, b: V4) -> V4 {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
    }

    fn normalize4(a: V4) -> V4 {
        scale4(a, 1.0 / dot4(a, a).sqrt())
    }

    fn quat_mul(a: V4, b: V4) -> V4 {
        let [x0, y0, z0, w0] = a;
        let [x1, y1, z1, w1] = b;
        [
            ((w0 * x1 + x0 * w1) + y0 * z1) - z0 * y1,
            ((w0 * y1 - x0 * z1) + y0 * w1) + z0 * x1,
            ((w0 * z1 + x0 * y1) - y0 * x1) + z0 * w1,
            ((w0 * w1 - x0 * x1) - y0 * y1) - z0 * z1,
        ]
    }

    /// `v·(w² - b·b) + b·(2 v·b) + (b × v)·2w`, with `b` the vector part.
    fn quat_rotate(q: V4, v: V3) -> V3 {
        let b = [q[0], q[1], q[2]];
        let w = q[3];
        let t = scale3(v, w * w - dot3(b, b));
        let t = add3(t, scale3(b, dot3(v, b) * 2.0));
        add3(t, scale3(cross(b, v), w * 2.0))
    }

    fn mat4_mul_vec4(m: &M4, v: V4) -> V4 {
        let r = scale4(m[0], v[0]);
        let r = add4(r, scale4(m[1], v[1]));
        let r = add4(r, scale4(m[2], v[2]));
        add4(r, scale4(m[3], v[3]))
    }

    fn mat3_mul_vec3(m: &[V3; 3], v: V3) -> V3 {
        let r = scale3(m[0], v[0]);
        let r = add3(r, scale3(m[1], v[1]));
        add3(r, scale3(m[2], v[2]))
    }

    /// The rotation matrix of a unit quaternion, scaled per column, translated.
    fn srt(s: V3, q: V4, t: V3) -> M4 {
        let [x, y, z, w] = q;
        let (x2, y2, z2) = (x + x, y + y, z + z);
        let (xx, xy, xz) = (x * x2, x * y2, x * z2);
        let (yy, yz, zz) = (y * y2, y * z2, z * z2);
        let (wx, wy, wz) = (w * x2, w * y2, w * z2);
        [
            scale4([1.0 - (yy + zz), xy + wz, xz - wy, 0.0], s[0]),
            scale4([xy - wz, 1.0 - (xx + zz), yz + wx, 0.0], s[1]),
            scale4([xz + wy, yz - wx, 1.0 - (xx + yy), 0.0], s[2]),
            [t[0], t[1], t[2], 1.0],
        ]
    }

    fn b3(v: V3) -> [u32; 3] {
        v.map(f32::to_bits)
    }

    fn b4(v: V4) -> [u32; 4] {
        v.map(f32::to_bits)
    }

    fn m4bits(m: &M4) -> [[u32; 4]; 4] {
        m.map(b4)
    }

    struct Inputs(Rng);

    impl Inputs {
        fn f(&mut self) -> f32 {
            self.0.range_f32(-100.0, 100.0)
        }
        fn v3(&mut self) -> V3 {
            [self.f(), self.f(), self.f()]
        }
        fn v4(&mut self) -> V4 {
            [self.f(), self.f(), self.f(), self.f()]
        }
        fn unit_quat(&mut self) -> V4 {
            normalize4(self.v4())
        }
        fn m4(&mut self) -> M4 {
            [self.v4(), self.v4(), self.v4(), self.v4()]
        }
    }

    const SAMPLES: usize = 20_000;

    #[test]
    fn vec3a_and_vec4_sum_lanes_in_one_order() {
        let mut r = Inputs(Rng::new(0x5ca1a));
        for _ in 0..SAMPLES {
            let (a, b) = (r.v3(), r.v3());
            let (ga, gb) = (Vec3A::from_array(a), Vec3A::from_array(b));
            assert_eq!(
                ga.dot(gb).to_bits(),
                dot3(a, b).to_bits(),
                "Vec3A::dot {a:?} {b:?}"
            );
            assert_eq!(
                b3(ga.cross(gb).to_array()),
                b3(cross(a, b)),
                "Vec3A::cross {a:?} {b:?}"
            );
            assert_eq!(
                ga.length().to_bits(),
                dot3(a, a).sqrt().to_bits(),
                "Vec3A::length {a:?}"
            );

            let (c, d) = (r.v4(), r.v4());
            let (gc, gd) = (Vec4::from_array(c), Vec4::from_array(d));
            assert_eq!(
                gc.dot(gd).to_bits(),
                dot4(c, d).to_bits(),
                "Vec4::dot {c:?} {d:?}"
            );
            assert_eq!(
                gc.length().to_bits(),
                dot4(c, c).sqrt().to_bits(),
                "Vec4::length {c:?}"
            );
            assert_eq!(
                b4(gc.normalize().to_array()),
                b4(normalize4(c)),
                "Vec4::normalize {c:?}"
            );
        }
    }

    #[test]
    fn quat_products_and_rotations_match_the_scalar_reference() {
        let mut r = Inputs(Rng::new(0x9a7));
        for _ in 0..SAMPLES {
            let (raw, p, q, v) = (r.v4(), r.unit_quat(), r.unit_quat(), r.v3());
            let normalized = Quat::from_array(raw).normalize();
            assert_eq!(
                b4(normalized.to_array()),
                b4(normalize4(raw)),
                "Quat::normalize {raw:?}"
            );
            let (gp, gq) = (Quat::from_array(p), Quat::from_array(q));
            assert_eq!(
                b4((gp * gq).to_array()),
                b4(quat_mul(p, q)),
                "Quat * Quat {p:?} {q:?}"
            );
            assert_eq!(
                b3((gp * Vec3::from_array(v)).to_array()),
                b3(quat_rotate(p, v)),
                "Quat * Vec3 {p:?} {v:?}"
            );
            assert_eq!(
                b3((gp * Vec3A::from_array(v)).to_array()),
                b3(quat_rotate(p, v)),
                "Quat * Vec3A {p:?} {v:?}"
            );
        }
    }

    #[test]
    fn mat4_products_and_transforms_match_the_scalar_reference() {
        let mut r = Inputs(Rng::new(0x3a74));
        for _ in 0..SAMPLES / 4 {
            let (a, b, v) = (r.m4(), r.m4(), r.v4());
            let (ga, gb) = (Mat4::from_cols_array_2d(&a), Mat4::from_cols_array_2d(&b));
            assert_eq!(
                b4((ga * Vec4::from_array(v)).to_array()),
                b4(mat4_mul_vec4(&a, v)),
                "Mat4 * Vec4"
            );
            let product: M4 = [0, 1, 2, 3].map(|i| mat4_mul_vec4(&a, b[i]));
            assert_eq!(
                m4bits(&(ga * gb).to_cols_array_2d()),
                m4bits(&product),
                "Mat4 * Mat4"
            );

            let (s, q, t, p) = (r.v3(), r.unit_quat(), r.v3(), r.v3());
            let m = srt(s, q, t);
            let gm = Mat4::from_scale_rotation_translation(
                Vec3::from_array(s),
                Quat::from_array(q),
                Vec3::from_array(t),
            );
            assert_eq!(
                m4bits(&gm.to_cols_array_2d()),
                m4bits(&m),
                "Mat4::from_scale_rotation_translation"
            );
            let point = mat4_mul_vec4(&m, [p[0], p[1], p[2], 1.0]);
            assert_eq!(
                b3(gm.transform_point3(Vec3::from_array(p)).to_array()),
                b3([point[0], point[1], point[2]]),
                "Mat4::transform_point3"
            );
        }
    }

    #[test]
    fn mat3a_and_affine3a_match_the_scalar_reference() {
        let mut r = Inputs(Rng::new(0xaff));
        for _ in 0..SAMPLES / 4 {
            let (cols, v, t) = ([r.v3(), r.v3(), r.v3()], r.v3(), r.v3());
            let m = Mat3A::from_cols_array_2d(&cols);
            assert_eq!(
                b3((m * Vec3A::from_array(v)).to_array()),
                b3(mat3_mul_vec3(&cols, v)),
                "Mat3A * Vec3A"
            );
            let affine = Affine3A::from_mat3_translation(
                Mat3::from_cols_array_2d(&cols),
                Vec3::from_array(t),
            );
            assert_eq!(
                b3(affine.transform_point3(Vec3::from_array(v)).to_array()),
                b3(add3(mat3_mul_vec3(&cols, v), t)),
                "Affine3A::transform_point3"
            );
        }
    }
}
