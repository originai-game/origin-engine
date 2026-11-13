//! Colour grading: the lookup table, the frame's colour adjustment and the table the tonemap samples for it.

use super::*;

/// Colour grading through a 3D lookup table, applied to the display colour
/// (after the tonemap).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorGrading {
    pub lut: LutHandle,
    /// 0 leaves the image as tonemapped, 1 applies the LUT fully.
    pub intensity: f32,
}

/// A photo editor's colour adjustments of the display colour, after the
/// tone curve and [`ColorGrading`]'s table, in the order of the fields:
/// white balance, brightness and contrast, colour balance, hue,
/// saturation, vibrance, greyscale, invert and threshold. Every field at
/// zero leaves the image unchanged (`preserve_luminosity` acts only with a
/// colour balance). Each acts on the sRGB-encoded display colour (the
/// white balance on its linear light) and clamps it to 0..1.
///
/// They cost no pass: the renderer bakes them, after the grading table at
/// its intensity, into one 32³ table on the CPU ([`ColorAdjust::apply`] of
/// each of its 32,768 colours) whenever either changes, and the tonemap
/// samples that table instead. A frame whose adjustments hold still costs
/// what grading costs; one that changes them pays the bake: about 0.4 ms
/// with one adjustment on, 3.5 ms with all of them and 4.3 ms with a
/// grading table too (release build, one core of a 2.6 GHz Xeon).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorAdjust {
    /// White balance: towards warm (orange, positive) or cool (blue,
    /// negative), -1..1; Unity's Temperature divided by 100 (a von Kries
    /// scaling in LMS from a white point moved along the daylight locus).
    pub temperature: f32,
    /// White balance: towards magenta (positive) or green (negative),
    /// -1..1, as Lightroom's and Unity's Tint (divided by 100).
    pub tint: f32,
    /// Added to every channel, -1..1.
    pub brightness: f32,
    /// Around mid grey: positive spreads the tones by `1 / (1 - contrast)`,
    /// negative pulls them in by `1 + contrast` (-1 is flat grey).
    pub contrast: f32,
    /// The colour balance of the shadows (luminance below 0.5, fading out
    /// towards it): shifts along cyan-red (x), magenta-green (y) and
    /// yellow-blue (z), -1..1, added to red, green and blue.
    pub shadows: Vec3,
    /// The colour balance of the mid-tones (what neither of the others takes).
    pub midtones: Vec3,
    /// The colour balance of the highlights (luminance above 0.5, fading in from it).
    pub highlights: Vec3,
    /// Whether the colour balance keeps each pixel's luminance, so it moves
    /// only the hue (Photoshop's Preserve Luminosity).
    pub preserve_luminosity: bool,
    /// Hue rotation about the grey axis, in degrees.
    pub hue: f32,
    /// -1 grey, 0 unchanged, 1 twice the colour (the chroma around the
    /// luminance scaled by `1 + saturation`).
    pub saturation: f32,
    /// Saturation that spares colours already saturated and skin tones,
    /// -1..1.
    pub vibrance: f32,
    /// How far towards its luminance (grey) the colour goes, 0..1.
    pub grayscale: f32,
    /// How far towards its complement (`1 - c`) the colour goes, 0..1.
    pub invert: f32,
    /// Above 0, each pixel white where its luminance reaches this level
    /// and black below it (0..1); 0 is off. The table's 32 steps soften
    /// the edge over about 1/31 of the range.
    pub threshold: f32,
}

impl Default for ColorAdjust {
    fn default() -> Self {
        ColorAdjust {
            temperature: 0.0,
            tint: 0.0,
            brightness: 0.0,
            contrast: 0.0,
            shadows: Vec3::ZERO,
            midtones: Vec3::ZERO,
            highlights: Vec3::ZERO,
            preserve_luminosity: true,
            hue: 0.0,
            saturation: 0.0,
            vibrance: 0.0,
            grayscale: 0.0,
            invert: 0.0,
            threshold: 0.0,
        }
    }
}

/// Rec. 709 luma weights (`LUMA` in fullscreen.wgsl).
const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);

/// The sRGB transfer curves as tables of 4096 steps read with linear
/// interpolation: within 2e-5 of the curves (well inside an 8-bit step)
/// and much cheaper than `powf`, for the 32,768 colours of a bake.
struct SrgbTables {
    /// Linear light by the encoded value.
    decode: Vec<f32>,
    /// The encoded value by linear light.
    encode: Vec<f32>,
}

const SRGB_STEPS: usize = 4096;

fn srgb_tables() -> &'static SrgbTables {
    static TABLES: std::sync::OnceLock<SrgbTables> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let at = |i: usize| i as f32 / SRGB_STEPS as f32;
        SrgbTables {
            decode: (0..=SRGB_STEPS)
                .map(|i| {
                    let v = at(i);
                    if v <= 0.04045 {
                        v / 12.92
                    } else {
                        ((v + 0.055) / 1.055).powf(2.4)
                    }
                })
                .collect(),
            encode: (0..=SRGB_STEPS)
                .map(|i| {
                    let v = at(i);
                    if v <= 0.003_130_8 {
                        v * 12.92
                    } else {
                        1.055 * v.powf(1.0 / 2.4) - 0.055
                    }
                })
                .collect(),
        }
    })
}

/// `table` at `v` (0..1, clamped), interpolated.
fn srgb_lookup(table: &[f32], v: f32) -> f32 {
    let p = v.clamp(0.0, 1.0) * SRGB_STEPS as f32;
    let i = (p as usize).min(SRGB_STEPS - 1);
    table[i] + (table[i + 1] - table[i]) * (p - i as f32)
}

fn srgb_decode(c: Vec3) -> Vec3 {
    let t = &srgb_tables().decode;
    Vec3::new(
        srgb_lookup(t, c.x),
        srgb_lookup(t, c.y),
        srgb_lookup(t, c.z),
    )
}

fn srgb_encode(c: Vec3) -> Vec3 {
    let t = &srgb_tables().encode;
    Vec3::new(
        srgb_lookup(t, c.x),
        srgb_lookup(t, c.y),
        srgb_lookup(t, c.z),
    )
}

/// Linear sRGB to LMS cone response (Unity's `LIN_2_LMS_MAT`, CAT02 over
/// the sRGB primaries), in columns.
const LIN_TO_LMS: glam::Mat3 = glam::Mat3::from_cols(
    Vec3::new(3.90405e-1, 7.08416e-2, 2.31082e-2),
    Vec3::new(5.49941e-1, 9.63172e-1, 1.28021e-1),
    Vec3::new(8.92632e-3, 1.35775e-3, 9.36245e-1),
);

/// The LMS scale that maps the white of `temperature` and `tint` (-1..1)
/// to D65 (Unity's `ColorBalanceToLMSCoeffs`): the reference white moved
/// along x on the daylight locus (`StandardIlluminantY`), then along y.
fn white_balance(temperature: f32, tint: f32) -> Vec3 {
    let t1 = temperature * 100.0 / 60.0;
    let t2 = tint * 100.0 / 60.0;
    let x = 0.31271 - t1 * if t1 < 0.0 { 0.1 } else { 0.05 };
    let y = 2.87 * x - 3.0 * x * x - 0.275_095_07 + t2 * 0.05;
    let (cx, cz) = (x / y, (1.0 - x - y) / y);
    let lms = Vec3::new(
        0.7328 * cx + 0.4296 - 0.1624 * cz,
        -0.7036 * cx + 1.6975 + 0.0061 * cz,
        0.0030 * cx + 0.0136 + 0.9834 * cz,
    );
    Vec3::new(0.949_237, 1.035_42, 1.087_28) / lms
}

impl ColorAdjust {
    /// Whether it changes nothing (every adjustment at zero).
    pub fn is_identity(&self) -> bool {
        let ColorAdjust {
            temperature,
            tint,
            brightness,
            contrast,
            shadows,
            midtones,
            highlights,
            preserve_luminosity: _,
            hue,
            saturation,
            vibrance,
            grayscale,
            invert,
            threshold,
        } = *self;
        [
            temperature,
            tint,
            brightness,
            contrast,
            hue,
            saturation,
            vibrance,
            grayscale,
            invert,
            threshold,
        ]
        .iter()
        .all(|&v| v == 0.0)
            && shadows == Vec3::ZERO
            && midtones == Vec3::ZERO
            && highlights == Vec3::ZERO
    }

    /// The adjusted display colour of `c` (sRGB-encoded, 0..1).
    pub fn apply(&self, c: Vec3) -> Vec3 {
        self.prepare()(c)
    }

    /// [`ColorAdjust::apply`] with what it computes once per table (the
    /// white balance and hue rotation as matrices) computed now.
    pub(crate) fn prepare(&self) -> impl Fn(Vec3) -> Vec3 + '_ {
        let unit = |c: Vec3| c.clamp(Vec3::ZERO, Vec3::ONE);
        let towards_luma = move |c: Vec3, k: f32| {
            let l = c.dot(LUMA);
            unit(Vec3::splat(l) + (c - l) * k)
        };
        // The white balance in linear light: to LMS, scaled, back.
        let balance = (self.temperature != 0.0 || self.tint != 0.0).then(|| {
            LIN_TO_LMS.inverse()
                * glam::Mat3::from_diagonal(white_balance(self.temperature, self.tint))
                * LIN_TO_LMS
        });
        let contrast = self.contrast.clamp(-1.0, 0.999);
        let slope = if contrast > 0.0 {
            1.0 / (1.0 - contrast)
        } else {
            1.0 + contrast
        };
        let shifts = self.shadows != Vec3::ZERO
            || self.midtones != Vec3::ZERO
            || self.highlights != Vec3::ZERO;
        // Rotation about the grey axis (RedGPU's hueSaturation, CSS
        // hue-rotate's axis): rows w, w.zxy, w.yzx.
        let hue = (self.hue != 0.0).then(|| {
            let (s, k) = self.hue.to_radians().sin_cos();
            let r3 = 3f32.sqrt();
            let w = (Vec3::new(2.0 * k, -r3 * s - k, r3 * s - k) + 1.0) / 3.0;
            glam::Mat3::from_cols(w, Vec3::new(w.z, w.x, w.y), Vec3::new(w.y, w.z, w.x)).transpose()
        });
        move |c: Vec3| {
            let mut c = unit(c);
            if let Some(m) = balance {
                c = srgb_encode(unit(m * srgb_decode(c)));
            }
            if self.contrast != 0.0 || self.brightness != 0.0 {
                c = unit((c - 0.5) * slope + 0.5 + self.brightness);
            }
            if shifts {
                let l = c.dot(LUMA);
                let smooth = |e0: f32, e1: f32, x: f32| {
                    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                };
                let shadow = 1.0 - smooth(0.0, 0.5, l);
                let highlight = smooth(0.5, 1.0, l);
                let mid = 1.0 - shadow - highlight;
                let shifted =
                    c + self.shadows * shadow + self.midtones * mid + self.highlights * highlight;
                let after = shifted.dot(LUMA);
                c = unit(if self.preserve_luminosity && after > 0.0 {
                    shifted * (l / after)
                } else {
                    shifted
                });
            }
            if let Some(m) = hue {
                c = unit(m * c);
            }
            if self.saturation != 0.0 {
                c = towards_luma(c, 1.0 + self.saturation.max(-1.0));
            }
            if self.vibrance != 0.0 {
                // RedGPU's vibrance: weaker where the colour is already
                // saturated (a logistic falloff past 0.6) and on skin
                // tones (red over green over blue).
                let spread = c.max_element() - c.min_element();
                let protect = 1.0 / (1.0 + (6.0 * (spread - 0.6)).exp());
                let skin = if c.x > c.y && c.y > c.z {
                    let t = (((c.x - c.z) / c.x.max(1e-3) - 0.3) / 0.5).clamp(0.0, 1.0);
                    1.0 - t * t * (3.0 - 2.0 * t) * 0.7
                } else {
                    1.0
                };
                c = towards_luma(c, 1.0 + self.vibrance * protect * skin);
            }
            if self.grayscale != 0.0 {
                c = c.lerp(Vec3::splat(c.dot(LUMA)), self.grayscale.clamp(0.0, 1.0));
            }
            if self.invert != 0.0 {
                c = c.lerp(Vec3::ONE - c, self.invert.clamp(0.0, 1.0));
            }
            if self.threshold > 0.0 {
                c = Vec3::splat(if c.dot(LUMA) >= self.threshold {
                    1.0
                } else {
                    0.0
                });
            }
            unit(c)
        }
    }
}

// ---------------------------------------------------------------------------
// Colour lookup tables.
// ---------------------------------------------------------------------------

pub use crate::scene::LutHandle;

/// The table the tonemap samples for the frame's colour adjustment.
pub(crate) trait AdjustedLut {
    /// The table the tonemap samples for `adjust` after `grade` (a
    /// [`LUT_SIZE`] table and its intensity, mixed in linear light as the
    /// tonemap mixes it). The grid points fall on the grading table's
    /// texels, so it is read without filtering.
    fn adjusted(adjust: &ColorAdjust, grade: Option<(&ColorLut, f32)>) -> ColorLut;
}

impl AdjustedLut for ColorLut {
    fn adjusted(adjust: &ColorAdjust, grade: Option<(&ColorLut, f32)>) -> ColorLut {
        let apply = adjust.prepare();
        let n = LUT_SIZE as usize;
        let decode = &srgb_tables().decode;
        // The grid's steps, encoded and in linear light, and the linear
        // light of each texel value.
        let steps: Vec<f32> = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
        let linear: Vec<f32> = steps.iter().map(|&v| srgb_lookup(decode, v)).collect();
        let texel: Vec<f32> = (0..=255)
            .map(|v| srgb_lookup(decode, v as f32 / 255.0))
            .collect();
        let grade = grade.map(|(lut, k)| {
            debug_assert_eq!(lut.size, LUT_SIZE);
            (lut, k.clamp(0.0, 1.0))
        });
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        let mut rgba = vec![0u8; n * n * n * 4];
        for (i, out) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (r, g, b) = (i % n, i / n % n, i / (n * n));
            let c = match grade {
                Some((lut, k)) => {
                    let t = &lut.rgba[i * 4..i * 4 + 3];
                    let graded = Vec3::new(
                        texel[t[0] as usize],
                        texel[t[1] as usize],
                        texel[t[2] as usize],
                    );
                    srgb_encode(Vec3::new(linear[r], linear[g], linear[b]).lerp(graded, k))
                }
                None => Vec3::new(steps[r], steps[g], steps[b]),
            };
            let c = apply(c);
            *out = [q(c.x), q(c.y), q(c.z), 255];
        }
        ColorLut {
            size: LUT_SIZE,
            rgba,
        }
    }
}
