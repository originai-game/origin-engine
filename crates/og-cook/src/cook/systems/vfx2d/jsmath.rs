//! JavaScript's `Math` as the Forge's renderer calls it, in f64 and built
//! only from IEEE 754 operations, so the cook gives the same bytes on every
//! host (never the host's libm). Conversions and rounding follow
//! ECMAScript (`ToInt32`, `ToUint32`, `Math.round`, `Math.max`).

pub use libm::{asin, atan2, cbrt, cos, exp, pow, sin};

/// `Math.round`: halves round up (toward +∞).
pub fn round(x: f64) -> f64 {
    let r = x.floor();
    if x - r >= 0.5 { r + 1.0 } else { r }
}

/// `Math.max` of two: NaN when either is.
pub fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else {
        b
    }
}

/// `Math.min` of two: NaN when either is.
pub fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}

/// `Math.hypot(a, b)` as V8 computes it: scaled by the larger magnitude,
/// the squares summed with Kahan compensation.
pub fn hypot(a: f64, b: f64) -> f64 {
    let (a, b) = (a.abs(), b.abs());
    if a.is_infinite() || b.is_infinite() {
        return f64::INFINITY;
    }
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    let m = if a > b { a } else { b };
    if m == 0.0 {
        return 0.0;
    }
    let (mut sum, mut comp) = (0.0f64, 0.0f64);
    for v in [a, b] {
        let n = v / m;
        let s = n * n - comp;
        let p = sum + s;
        comp = (p - sum) - s;
        sum = p;
    }
    sum.sqrt() * m
}

/// ECMAScript `ToInt32`.
pub fn to_i32(x: f64) -> i32 {
    if !x.is_finite() {
        0
    } else if x.abs() < 2147483648.0 {
        x as i32
    } else {
        x.trunc().rem_euclid(4294967296.0) as u32 as i32
    }
}

/// ECMAScript `ToUint32` (`x >>> 0`).
pub fn to_u32(x: f64) -> u32 {
    to_i32(x) as u32
}

/// A JavaScript value's truth, for a number.
pub fn truthy(x: f64) -> bool {
    x != 0.0 && !x.is_nan()
}
