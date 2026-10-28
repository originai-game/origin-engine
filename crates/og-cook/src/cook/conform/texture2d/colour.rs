//! Colour for conformance: sRGB bytes, linear light and Oklab (Björn
//! Ottosson's, the matrices stylise.wgsl's `oklab` uses), with Oklab held
//! as fixed-point integers so every comparison the cook makes between two
//! colours is exact and the same on every machine.

use std::sync::OnceLock;

/// Oklab in fixed point: L, a, b times [`ONE`].
pub type Lab = [i32; 3];

/// The fixed-point scale of [`Lab`] (2^20).
pub const ONE: f64 = 1_048_576.0;

/// The linear value of each sRGB byte.
fn table() -> &'static [f64; 256] {
    static T: OnceLock<[f64; 256]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0.0; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let c = i as f64 / 255.0;
            *v = if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            };
        }
        t
    })
}

pub fn to_linear(c: u8) -> f64 {
    table()[c as usize]
}

/// The nearest sRGB byte of a linear value (clamped to 0..1).
pub fn to_srgb8(x: f64) -> u8 {
    let x = x.clamp(0.0, 1.0);
    let e = if x <= 0.003_130_8 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (e * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn linear_to_lab(c: [f64; 3]) -> Lab {
    let [r, g, b] = c;
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;
    let (l, m, s) = (l.max(0.0).cbrt(), m.max(0.0).cbrt(), s.max(0.0).cbrt());
    let q = |x: f64| (x * ONE).round() as i32;
    [
        q(0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s),
        q(1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s),
        q(0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s),
    ]
}

pub fn lab(c: [u8; 3]) -> Lab {
    linear_to_lab([to_linear(c[0]), to_linear(c[1]), to_linear(c[2])])
}

pub fn lab_to_linear(c: Lab) -> [f64; 3] {
    let (ll, a, b) = (c[0] as f64 / ONE, c[1] as f64 / ONE, c[2] as f64 / ONE);
    let l = ll + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m = ll - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s = ll - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l, m, s) = (l * l * l, m * m * m, s * s * s);
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    ]
}

pub fn lab_to_srgb8(c: Lab) -> [u8; 3] {
    lab_to_linear(c).map(to_srgb8)
}

/// The squared distance between two Oklab colours, in fixed point squared.
pub fn dist2(a: Lab, b: Lab) -> i64 {
    let d = |i: usize| (a[i] - b[i]) as i64;
    d(0) * d(0) + d(1) * d(1) + d(2) * d(2)
}

/// Chroma below this (Oklab units) is grey: no hue.
pub const GREY: f64 = 0.03;

/// The Oklch hues of the twelve colours of the wheel (sRGB at full
/// saturation every 30 degrees of HSV hue: red, orange, yellow, ...).
fn wheel() -> &'static [f64; 12] {
    static W: OnceLock<[f64; 12]> = OnceLock::new();
    W.get_or_init(|| {
        const RGB: [[u8; 3]; 12] = [
            [255, 0, 0],
            [255, 128, 0],
            [255, 255, 0],
            [128, 255, 0],
            [0, 255, 0],
            [0, 255, 128],
            [0, 255, 255],
            [0, 128, 255],
            [0, 0, 255],
            [128, 0, 255],
            [255, 0, 255],
            [255, 0, 128],
        ];
        RGB.map(|rgb| {
            let c = lab(rgb);
            (c[2] as f64)
                .atan2(c[1] as f64)
                .to_degrees()
                .rem_euclid(360.0)
        })
    })
}

/// The hue family of a colour for ramp limits: 0 for greys (chroma under
/// [`GREY`]), else 1 to 12, the nearest of the wheel's twelve hues by its
/// Oklch hue, so a primary sits in the middle of its family.
pub fn hue_family(c: Lab) -> u8 {
    let (a, b) = (c[1] as f64 / ONE, c[2] as f64 / ONE);
    if (a * a + b * b).sqrt() < GREY {
        return 0;
    }
    let deg = b.atan2(a).to_degrees().rem_euclid(360.0);
    let gap = |h: f64| {
        let d = (deg - h).abs();
        d.min(360.0 - d)
    };
    let (mut best, mut bd) = (0usize, f64::MAX);
    for (k, &h) in wheel().iter().enumerate() {
        if gap(h) < bd {
            (best, bd) = (k, gap(h));
        }
    }
    1 + best as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_bytes_and_orders_lightness() {
        for c in [[0, 0, 0], [255, 255, 255], [200, 30, 90], [12, 140, 250]] {
            assert_eq!(lab_to_srgb8(lab(c)), c, "{c:?}");
        }
        // White is L 1, black L 0, greys have no chroma.
        assert!((lab([255, 255, 255])[0] as f64 / ONE - 1.0).abs() < 1e-4);
        assert_eq!(lab([0, 0, 0])[0], 0);
        assert_eq!(hue_family(lab([128, 128, 128])), 0);
        // Red lies near 30 degrees, blue near 265: different families.
        assert_ne!(hue_family(lab([255, 0, 0])), hue_family(lab([0, 0, 255])));
    }
}
