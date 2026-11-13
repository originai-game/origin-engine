//! Colour ramps, palettes and OKLab, ported from the Pixel VFX Forge's
//! `color.js` (third_party/scenario-pixel-vfx). Ramps run dark to light;
//! colours are packed `0xRRGGBB` as the Forge packs them.

use super::jsmath::{self as m, round};
use super::json::{Val, parse_float};
use std::collections::BTreeMap;
use std::f64::consts::PI;

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// `hexToRgb`: `#rgb`, `#rrggbb` (or without the `#`); what does not parse is black.
pub fn hex_to_rgb(hex: &str) -> [i32; 3] {
    let h = hex.replacen('#', "", 1);
    let mut h = h.trim().to_string();
    if h.chars().count() == 3 {
        h = h.chars().flat_map(|c| [c, c]).collect();
    }
    let six: String = h.chars().take(6).collect();
    let n = parse_int_hex(&six);
    [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}

/// `parseInt(s, 16) || 0`, as an int32 the shifts then see.
fn parse_int_hex(s: &str) -> i32 {
    let t = s.trim_start();
    let (neg, t) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let t = t
        .strip_prefix("0x")
        .or_else(|| t.strip_prefix("0X"))
        .unwrap_or(t);
    let digits: String = t.chars().take_while(char::is_ascii_hexdigit).collect();
    let Ok(v) = i64::from_str_radix(&digits, 16) else {
        return 0;
    };
    (if neg { -v } else { v }) as i32
}

pub fn pack([r, g, b]: [i32; 3]) -> i32 {
    (r << 16) | (g << 8) | b
}

pub fn unpack(c: i32) -> [i32; 3] {
    [(c >> 16) & 255, (c >> 8) & 255, c & 255]
}

fn to_lin(c: i32) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        m::pow((c + 0.055) / 1.055, 2.4)
    }
}

pub fn rgb_to_oklab([r, g, b]: [i32; 3]) -> [f64; 3] {
    let (r, g, b) = (to_lin(r), to_lin(g), to_lin(b));
    let l = m::cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    let mm = m::cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    let s = m::cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
    [
        0.2104542553 * l + 0.793617785 * mm - 0.0040720468 * s,
        1.9779984951 * l - 2.428592205 * mm + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * mm - 0.808675766 * s,
    ]
}

fn oklab_to_linear([ll, a, b]: [f64; 3]) -> [f64; 3] {
    let l = m::pow(ll + 0.3963377774 * a + 0.2158037573 * b, 3.0);
    let mm = m::pow(ll - 0.1055613458 * a - 0.0638541728 * b, 3.0);
    let s = m::pow(ll - 0.0894841775 * a - 1.291485548 * b, 3.0);
    [
        4.0767416621 * l - 3.3077115913 * mm + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * mm - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * mm + 1.707614701 * s,
    ]
}

fn in_gamut(lin: [f64; 3]) -> bool {
    lin.iter().all(|c| (-0.0005..=1.0005).contains(c))
}

fn lin_to_byte(c: f64) -> i32 {
    let c = clamp01(c);
    let s = if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * m::pow(c, 1.0 / 2.4) - 0.055
    };
    round(clamp01(s) * 255.0) as i32
}

/// OKLCH to sRGB, chroma reduced until the colour fits the gamut.
pub fn oklch(l: f64, c: f64, h_deg: f64) -> [i32; 3] {
    let h = (h_deg * PI) / 180.0;
    let (ch, sh) = (m::cos(h), m::sin(h));
    let (mut lo, mut hi) = (0.0, c);
    let mut lin = oklab_to_linear([l, c * ch, c * sh]);
    if !in_gamut(lin) {
        for _ in 0..14 {
            let mid = (lo + hi) / 2.0;
            if in_gamut(oklab_to_linear([l, mid * ch, mid * sh])) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lin = oklab_to_linear([l, lo * ch, lo * sh]);
    }
    lin.map(lin_to_byte)
}

fn to_oklch(rgb: [i32; 3]) -> [f64; 3] {
    let [l, a, b] = rgb_to_oklab(rgb);
    [
        l,
        m::hypot(a, b),
        ((m::atan2(b, a) * 180.0) / PI + 360.0) % 360.0,
    ]
}

fn hue_toward(h: f64, target: f64, amt: f64) -> f64 {
    let d = ((target - h + 540.0) % 360.0) - 180.0;
    h + d * amt
}

/// A hue-shifted ramp of seven: cool shadows, warm highlights, the
/// brightest step nearly white.
pub fn ramp_from_hue(hue: f64) -> Vec<i32> {
    let n = 7;
    (0..n)
        .map(|i| {
            let t = f64::from(i) / f64::from(n - 1);
            let l = 0.2 + 0.77 * m::pow(t, 0.95);
            let mut c = 0.2 * m::sin(PI * (0.12 + 0.72 * t)) * (1.0 - 0.8 * m::pow(t, 5.0));
            let h = if t < 0.5 {
                hue_toward(hue, 285.0, 0.32 * (1.0 - t * 2.0))
            } else {
                hue_toward(hue, 95.0, 0.22 * (t * 2.0 - 1.0))
            };
            if t == 1.0 {
                c *= 0.4;
            }
            pack(oklch(l, c, h))
        })
        .collect()
}

fn shift_ramp(rgbs: Vec<[i32; 3]>, deg: f64, sat: f64) -> Vec<i32> {
    if !m::truthy(deg) && sat == 1.0 {
        return rgbs.into_iter().map(pack).collect();
    }
    rgbs.into_iter()
        .map(|rgb| {
            let [l, c, h] = to_oklch(rgb);
            pack(oklch(l, c * sat, h + deg))
        })
        .collect()
}

/// The hand-tuned element ramps, dark to light.
pub const RAMPS: &[(&str, &[&str])] = &[
    (
        "fire",
        &[
            "#2a0b1c", "#6b1328", "#b3261e", "#e8541c", "#f99a1b", "#ffd84a", "#fff6c9",
        ],
    ),
    (
        "lava",
        &[
            "#1c0606", "#4c0b0a", "#9c1d0b", "#e3470f", "#ff8f1f", "#ffd25c", "#fff3b8",
        ],
    ),
    (
        "ice",
        &[
            "#0d1638", "#1b347e", "#2a63c8", "#43a3ec", "#86e0ff", "#cdf7ff", "#ffffff",
        ],
    ),
    (
        "frost",
        &[
            "#1d2547", "#3b4f8a", "#6a8fc9", "#a2c9ec", "#d5ecfa", "#ffffff",
        ],
    ),
    (
        "lightning",
        &[
            "#100b36", "#24208c", "#3a56ec", "#5cb2ff", "#a7efff", "#ffffff",
        ],
    ),
    (
        "thunder",
        &[
            "#1e1206", "#6d3b0d", "#cf8212", "#ffcf3a", "#fff4a3", "#ffffff",
        ],
    ),
    (
        "poison",
        &[
            "#140c1e", "#2b2148", "#2f5a2a", "#4f9a24", "#93d82c", "#d8ff7a", "#f6ffd6",
        ],
    ),
    (
        "holy",
        &[
            "#33200e", "#7d4f12", "#c99322", "#f6cf4a", "#fff1a6", "#fffbe8", "#ffffff",
        ],
    ),
    (
        "dark",
        &[
            "#06030b", "#160a28", "#2d1150", "#4f1d88", "#8338cc", "#c482ff", "#f0dcff",
        ],
    ),
    (
        "void",
        &[
            "#000000", "#0c0517", "#200b3a", "#40166c", "#7c2fc4", "#d8a8ff",
        ],
    ),
    (
        "arcane",
        &[
            "#190731", "#42106c", "#7a1db6", "#c13ae6", "#ff78ee", "#ffd2f8", "#ffffff",
        ],
    ),
    (
        "nature",
        &[
            "#0b1d13", "#145526", "#2b8f35", "#64c845", "#b6ee72", "#efffcc",
        ],
    ),
    (
        "water",
        &[
            "#071833", "#0d376b", "#1563ad", "#259fdc", "#73d6f3", "#d9faff", "#ffffff",
        ],
    ),
    (
        "blood",
        &[
            "#12030a", "#420914", "#7f0f22", "#c11d2f", "#f24848", "#ff9a8a",
        ],
    ),
    (
        "earth",
        &[
            "#170f0a", "#38261a", "#644429", "#946a42", "#c39c68", "#ead6a6",
        ],
    ),
    (
        "smoke",
        &[
            "#0f0c14", "#26212d", "#433d4d", "#6a6274", "#9a92a3", "#cfc8d6",
        ],
    ),
    (
        "dust",
        &[
            "#1d1610", "#3f3226", "#665443", "#91806a", "#bdae95", "#e6dcc6",
        ],
    ),
    (
        "ash",
        &["#141216", "#2c2930", "#4a464f", "#716c76", "#a39da8"],
    ),
    (
        "wind",
        &[
            "#0c2627", "#195852", "#33a093", "#73dcc4", "#c9fbea", "#ffffff",
        ],
    ),
    (
        "plasma",
        &[
            "#1a0420", "#55095a", "#b81488", "#ff3caa", "#ff98d6", "#ffe6f6", "#ffffff",
        ],
    ),
    (
        "energy",
        &[
            "#041e29", "#0a4757", "#0f8798", "#2dd2cd", "#9cfff0", "#ffffff",
        ],
    ),
    (
        "gold",
        &[
            "#291505", "#6b3c09", "#b5720d", "#f0b31b", "#ffe16b", "#fffadf",
        ],
    ),
    (
        "solar",
        &[
            "#2a1204", "#7a2e05", "#d65d0a", "#ff9f1c", "#ffe066", "#ffffff",
        ],
    ),
    (
        "shadow",
        &["#000000", "#08070c", "#141019", "#211b29", "#352c40"],
    ),
    (
        "steel",
        &[
            "#1b2233", "#3a4a66", "#6d84a8", "#a9c1dd", "#e3eefa", "#ffffff",
        ],
    ),
    ("white", &["#5a6378", "#9aa4b8", "#d4dbe8", "#ffffff"]),
    (
        "rainbow",
        &[
            "#3a0ca3", "#4361ee", "#4cc9f0", "#80ed99", "#ffd166", "#ff7f50", "#ffffff",
        ],
    ),
];

/// The fixed palettes an effect can be locked to.
pub const PALETTES: &[(&str, &[&str])] = &[
    (
        "pico8",
        &[
            "#000000", "#1d2b53", "#7e2553", "#008751", "#ab5236", "#5f574f", "#c2c3c7", "#fff1e8",
            "#ff004d", "#ffa300", "#ffec27", "#00e436", "#29adff", "#83769c", "#ff77a8", "#ffccaa",
        ],
    ),
    (
        "sweetie16",
        &[
            "#1a1c2c", "#5d275d", "#b13e53", "#ef7d57", "#ffcd75", "#a7f070", "#38b764", "#257179",
            "#29366f", "#3b5dc9", "#41a6f6", "#73eff7", "#f4f4f4", "#94b0c2", "#566c86", "#333c57",
        ],
    ),
    (
        "endesga32",
        &[
            "#be4a2f", "#d77643", "#ead4aa", "#e4a672", "#b86f50", "#733e39", "#3e2731", "#a22633",
            "#e43b44", "#f77622", "#feae34", "#fee761", "#63c74d", "#3e8948", "#265c42", "#193c3e",
            "#124e89", "#0099db", "#2ce8f5", "#ffffff", "#c0cbdc", "#8b9bb4", "#5a6988", "#3a4466",
            "#262b44", "#181425", "#ff0044", "#68386c", "#b55088", "#f6757a", "#e8b796", "#c28569",
        ],
    ),
    ("gameboy", &["#0f380f", "#306230", "#8bac0f", "#9bbc0f"]),
    ("onebit", &["#0b0b10", "#f4f1e8"]),
];

pub fn named<'a>(table: &'a [(&str, &'a [&'a str])], name: &str) -> Option<&'a [&'a str]> {
    table.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}

/// `resolveRamp`: a ramp spec (a name, `"hue:<deg>"` or a list of hex
/// colours) to packed colours, hue-rotated and saturation-scaled.
pub fn resolve_ramp(spec: &Val, hue: f64, sat: f64) -> Vec<i32> {
    let rgbs: Vec<[i32; 3]> = match spec {
        Val::Arr(a) if !a.is_empty() => a.iter().map(|v| hex_to_rgb(&v.js_string())).collect(),
        Val::Str(s) if s.starts_with("hue:") => {
            let h = parse_float(&s[4..]);
            let h = if m::truthy(h) { h } else { 0.0 };
            ramp_from_hue(h).into_iter().map(unpack).collect()
        }
        v => {
            let key = v.js_string();
            named(RAMPS, &key)
                .unwrap_or(RAMPS[0].1)
                .iter()
                .map(|h| hex_to_rgb(h))
                .collect()
        }
    };
    shift_ramp(rgbs, hue, sat)
}

/// `nearestInPalette`: the palette colour nearest `c` in OKLab, lightness
/// weighted 1.4; memoised per colour.
pub struct Nearest {
    palette: Vec<i32>,
    labs: Vec<[f64; 3]>,
    memo: BTreeMap<i32, i32>,
}

impl Nearest {
    pub fn new(palette: Vec<i32>) -> Nearest {
        let labs = palette.iter().map(|&p| rgb_to_oklab(unpack(p))).collect();
        Nearest {
            palette,
            labs,
            memo: BTreeMap::new(),
        }
    }

    pub fn get(&mut self, c: i32) -> i32 {
        if let Some(&hit) = self.memo.get(&c) {
            return hit;
        }
        let lab = rgb_to_oklab(unpack(c));
        let (mut best, mut bd) = (0, f64::INFINITY);
        for (p, pl) in self.palette.iter().zip(&self.labs) {
            let d = m::pow(lab[0] - pl[0], 2.0) * 1.4
                + m::pow(lab[1] - pl[1], 2.0)
                + m::pow(lab[2] - pl[2], 2.0);
            if d < bd {
                bd = d;
                best = *p;
            }
        }
        self.memo.insert(c, best);
        best
    }
}

/// `darken`: each channel scaled by `k`, blue a little less.
pub fn darken(c: i32, k: f64) -> i32 {
    let [r, g, b] = unpack(c).map(f64::from);
    let bb = b * k * 1.08;
    pack([
        round(r * k) as i32,
        round(g * k) as i32,
        round(if bb > 255.0 { 255.0 } else { bb }) as i32,
    ])
}
