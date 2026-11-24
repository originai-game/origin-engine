//! Colour: the sRGB transfer functions, `#rrggbb` hex and Oklab, the
//! engine's one copy of each.
//!
//! The sRGB curve is IEC 61966-2-1's piecewise one (a linear toe below
//! 0.04045 encoded, 0.0031308 linear; a 2.4 power above), never a 2.2
//! power. Values are computed in `f64` with og-core's deterministic
//! [`math`](crate::math) and rounded once, so they are the same bits on
//! every host and within an ulp of the exact curve. Inputs outside 0..1
//! extend the curve (negative values follow the toe, HDR values the power);
//! the 8-bit encoder clamps.
//!
//! Oklab is Björn Ottosson's (2020), on linear sRGB.

use crate::math::{cbrt, pow64_positive};

/// One encoded sRGB channel (0..1) as linear light.
pub fn srgb_to_linear(c: f32) -> f32 {
    srgb_to_linear_f64(f64::from(c)) as f32
}

/// One linear channel as its sRGB encoding (0..1 for 0..1 in).
pub fn linear_to_srgb(c: f32) -> f32 {
    linear_to_srgb_f64(f64::from(c)) as f32
}

/// [`srgb_to_linear`] in `f64`.
pub fn srgb_to_linear_f64(c: f64) -> f64 {
    if c.is_infinite() {
        c
    } else if c > 0.04045 {
        pow64_positive((c + 0.055) / 1.055, 2.4)
    } else {
        c / 12.92
    }
}

/// [`linear_to_srgb`] in `f64`.
pub fn linear_to_srgb_f64(c: f64) -> f64 {
    if c.is_infinite() {
        c
    } else if c > 0.003_130_8 {
        1.055 * pow64_positive(c, 1.0 / 2.4) - 0.055
    } else {
        c * 12.92
    }
}

/// An 8-bit sRGB channel as linear light.
pub fn srgb8_to_linear(b: u8) -> f32 {
    srgb_to_linear(f32::from(b) / 255.0)
}

/// A linear channel as the 8-bit sRGB code an sRGB target stores: clamped
/// to 0..1, encoded, rounded to the nearest code (NaN gives 0).
pub fn linear_to_srgb8(c: f32) -> u8 {
    (linear_to_srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8
}

/// A hex colour, `#` optional: `rgb`, `rgba`, `rrggbb` or `rrggbbaa` as
/// sRGB bytes and alpha (255 when it has none). `None` for anything else.
pub fn parse_hex(text: &str) -> Option<[u8; 4]> {
    let h = text.strip_prefix('#').unwrap_or(text).as_bytes();
    let digit = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let d: Vec<u8> = h.iter().map(|&c| digit(c)).collect::<Option<_>>()?;
    let at = |i: usize| match d.len() {
        3 | 4 => d.get(i).map(|v| v * 17),
        _ => Some(d.get(2 * i)? * 16 + d.get(2 * i + 1)?),
    };
    match d.len() {
        3 | 6 => Some([at(0)?, at(1)?, at(2)?, 255]),
        4 | 8 => Some([at(0)?, at(1)?, at(2)?, at(3)?]),
        _ => None,
    }
}

/// `#rrggbb`, or `#rrggbbaa` when the alpha is not 255: lowercase, what
/// [`parse_hex`] reads back.
pub fn hex_string([r, g, b, a]: [u8; 4]) -> String {
    if a == 255 {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

/// The Oklab `[L, a, b]` of a linear sRGB colour.
pub fn oklab(rgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = rgb;
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l, m, s) = (cbrt(l), cbrt(m), cbrt(s));
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// The linear sRGB of an Oklab colour ([`oklab`]'s inverse).
pub fn oklab_to_linear(lab: [f32; 3]) -> [f32; 3] {
    let [l, a, b] = lab;
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}
