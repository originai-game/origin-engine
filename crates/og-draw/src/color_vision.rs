//! Colour vision filters over the presented frame (docs/ui.md §3.2): the
//! player's colour-blind mode, simulated or corrected. A filter is one or
//! two 3×3 matrices in linear sRGB and the plane that picks between them;
//! [`ColorFilter::apply`] is what the GPU pass (shaders/color_filter.wgsl)
//! does to every pixel of the frame, scene and overlay alike.
//!
//! Simulation is libDaltonLens's (public domain, `third_party/daltonlens`):
//! Viénot, Brettel and Mollon 1999 for protanopia and deuteranopia, one
//! matrix each; Brettel, Viénot and Mollon 1997 for tritanopia, two
//! half-planes, because Viénot 1999 is not accurate for it. Partial
//! strength blends the simulation with the input, as libDaltonLens's
//! severity does (anomalous trichromacy, the commoner condition).
//!
//! Correction is Fidaner, Lin and Ozguven's daltonisation (2005), with the
//! per-deficiency error matrices andrewwillmott/colour-blind-luts uses
//! (public domain, `third_party/daltonlens`): what the viewer loses,
//! `rgb − simulate(rgb)`, is moved into the channels they still tell
//! apart, `rgb + E·(rgb − simulate(rgb))`. That is linear in each
//! half-plane, so a correction is matrices of the same form.

/// A colour vision deficiency (UiDesc.colorVision).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorVision {
    /// No filter.
    #[default]
    Typical,
    /// No long-wavelength (red) cones.
    Protanopia,
    /// No medium-wavelength (green) cones.
    Deuteranopia,
    /// No short-wavelength (blue) cones.
    Tritanopia,
}

impl ColorVision {
    pub const ALL: [ColorVision; 4] = [
        ColorVision::Typical,
        ColorVision::Protanopia,
        ColorVision::Deuteranopia,
        ColorVision::Tritanopia,
    ];

    /// Its CSS keyword (`-og-color-vision`).
    pub fn name(self) -> &'static str {
        match self {
            ColorVision::Typical => "none",
            ColorVision::Protanopia => "protanopia",
            ColorVision::Deuteranopia => "deuteranopia",
            ColorVision::Tritanopia => "tritanopia",
        }
    }

    pub fn from_name(name: &str) -> Option<ColorVision> {
        ColorVision::ALL.into_iter().find(|v| v.name() == name)
    }
}

/// What a colour vision filter does to the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorVisionFilter {
    /// Recolours the frame so a viewer with the deficiency tells apart
    /// what it would otherwise confuse (the player's setting).
    #[default]
    Correct,
    /// Shows the frame as a viewer with the deficiency sees it (checking a
    /// screen during development).
    Simulate,
}

type Mat3 = [[f32; 3]; 3];

const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

// libDaltonLens.c, revision 1a01fc1bf8d8dd419af8343b80b05e98ba50a75d:
// `dl_vienot_protan_rgbCvd_from_rgb`, `dl_vienot_deutan_rgbCvd_from_rgb`
// and `brettel_tritan_params`, row-major, linear sRGB.
const VIENOT_PROTAN: Mat3 = [
    [0.11238, 0.88762, 0.00000],
    [0.11238, 0.88762, -0.00000],
    [0.00401, -0.00401, 1.00000],
];
const VIENOT_DEUTAN: Mat3 = [
    [0.29275, 0.70725, 0.00000],
    [0.29275, 0.70725, -0.00000],
    [-0.02234, 0.02234, 1.00000],
];
const BRETTEL_TRITAN_1: Mat3 = [
    [1.01277, 0.13548, -0.14826],
    [-0.01243, 0.86812, 0.14431],
    [0.07589, 0.80500, 0.11911],
];
const BRETTEL_TRITAN_2: Mat3 = [
    [0.93678, 0.18979, -0.12657],
    [0.06154, 0.81526, 0.12320],
    [-0.37562, 1.12767, 0.24796],
];
const BRETTEL_TRITAN_PLANE: [f32; 3] = [0.03901, -0.02788, -0.01113];

// colour-blind-luts CBLuts.cpp (revision 12d87832134e3ab08757676fcde04bdefdc04461),
// `kDaltonErrorToDelta*`: Fidaner et al.'s error redistribution per deficiency.
const ERROR_PROTAN: Mat3 = [[0.0, 0.0, 0.0], [0.7, 1.0, 0.0], [0.7, 0.0, 1.0]];
const ERROR_DEUTAN: Mat3 = [[1.0, 0.7, 0.0], [0.0, 0.0, 0.0], [0.0, 0.7, 1.0]];
const ERROR_TRITAN: Mat3 = [[1.0, 0.0, 0.7], [0.0, 1.0, 0.7], [0.0, 0.0, 0.0]];

fn mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut m = [[0.0; 3]; 3];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn lerp(a: &Mat3, b: &Mat3, t: f32) -> Mat3 {
    let mut m = *a;
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = a[i][j] + (b[i][j] - a[i][j]) * t;
        }
    }
    m
}

/// `I + E·(I − S)`: the daltonisation of simulation `s` with error matrix `e`.
fn correct(s: &Mat3, e: &Mat3) -> Mat3 {
    let mut lost = IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            lost[i][j] -= s[i][j];
        }
    }
    let shift = mul(e, &lost);
    let mut m = IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] += shift[i][j];
        }
    }
    m
}

/// A colour vision filter: every pixel's linear sRGB `rgb` becomes `a·rgb`
/// where `plane·rgb ≥ 0`, else `b·rgb` (`a == b` for one matrix), clamped
/// at zero. Row-major.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorFilter {
    pub a: [[f32; 3]; 3],
    pub b: [[f32; 3]; 3],
    pub plane: [f32; 3],
}

impl ColorFilter {
    /// The filter for `vision` at `strength` (0 to 1: the deficiency's
    /// severity, 1 a dichromat's), or none: typical vision or no strength.
    pub fn new(
        vision: ColorVision,
        filter: ColorVisionFilter,
        strength: f32,
    ) -> Option<ColorFilter> {
        let t = if strength.is_finite() {
            strength.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (a, b, plane, e) = match vision {
            ColorVision::Typical => return None,
            _ if t == 0.0 => return None,
            ColorVision::Protanopia => (VIENOT_PROTAN, VIENOT_PROTAN, [0.0; 3], ERROR_PROTAN),
            ColorVision::Deuteranopia => (VIENOT_DEUTAN, VIENOT_DEUTAN, [0.0; 3], ERROR_DEUTAN),
            ColorVision::Tritanopia => (
                BRETTEL_TRITAN_1,
                BRETTEL_TRITAN_2,
                BRETTEL_TRITAN_PLANE,
                ERROR_TRITAN,
            ),
        };
        let (a, b) = (lerp(&IDENTITY, &a, t), lerp(&IDENTITY, &b, t));
        Some(match filter {
            ColorVisionFilter::Simulate => ColorFilter { a, b, plane },
            ColorVisionFilter::Correct => ColorFilter {
                a: correct(&a, &e),
                b: correct(&b, &e),
                plane,
            },
        })
    }

    /// What the pass makes of linear sRGB `rgb`.
    pub fn apply(&self, rgb: [f32; 3]) -> [f32; 3] {
        let d: f32 = (0..3).map(|i| self.plane[i] * rgb[i]).sum();
        let m = if d >= 0.0 { &self.a } else { &self.b };
        let mut o = [0.0; 3];
        for (i, v) in o.iter_mut().enumerate() {
            *v = (0..3).map(|k| m[i][k] * rgb[k]).sum::<f32>().max(0.0);
        }
        o
    }

    /// What the pass makes of an 8-bit sRGB pixel: decoded, filtered,
    /// encoded and rounded, as an sRGB target stores it.
    pub fn apply_srgb8(&self, rgb: [u8; 3]) -> [u8; 3] {
        use og_core::color::{linear_to_srgb8, srgb8_to_linear};
        self.apply(rgb.map(srgb8_to_linear)).map(linear_to_srgb8)
    }

    /// Its uniform (`FilterParams` in color_filter.wgsl): `a`'s rows, `b`'s
    /// rows, then the plane with `decode` in w (1: the frame holds
    /// sRGB-encoded values, decoded before and encoded after).
    pub fn params(&self, decode: bool) -> [[f32; 4]; 7] {
        let row = |r: [f32; 3]| [r[0], r[1], r[2], 0.0];
        [
            row(self.a[0]),
            row(self.a[1]),
            row(self.a[2]),
            row(self.b[0]),
            row(self.b[1]),
            row(self.b[2]),
            [
                self.plane[0],
                self.plane[1],
                self.plane[2],
                f32::from(u8::from(decode)),
            ],
        ]
    }
}
