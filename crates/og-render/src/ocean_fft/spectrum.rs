//! The FFT sea's spectrum, built on the CPU once per sea (its wind, swell
//! and choppiness), the GPU only evolving and transforming it.
//!
//! Ported from goldflag/blue-water-sea-and-sky's `ocean/src/spectrum.rs`
//! (MIT, Copyright (c) 2026 Bill Yang; `third_party/ocean/`): each cascade
//! is a periodic tile whose wavevector lattice k = 2π n / L holds seeded
//! Gaussian amplitudes drawn from JONSWAP (Hasselmann et al. 1973), its short
//! waves held to Phillips' saturation range, with the frequency-dependent
//! spreading of Hasselmann et al. 1980, the bands of the cascades disjoint,
//! every angular frequency a whole multiple of 2π / [`FOLD_PERIOD`] (so the
//! GPU takes the time modulo it without float32 phase drift), and the whole
//! rescaled to the significant height asked of it.
//!
//! What is og's: the sea is og_core::ocean's for the same og.Water (its
//! peak `0.877 g / U`, its significant height `0.21 U² / g` times `swell`),
//! and the waves the tick floats things on are drawn as they are (Gerstner,
//! in the vertex stage): the longest of og_core's waves, those longer than
//! [`SeaParameters::long_cut`]. The spectrum leaves that band out of the
//! first cascade and draws the height variance they do not carry, so the
//! drawn sea's total is the asked-for one.

use std::f64::consts::{LN_2, PI};

pub const GRAVITY: f64 = 9.81;
/// Every angular frequency is a whole multiple of 2π / FOLD_PERIOD: the sea
/// repeats exactly after this many seconds.
pub const FOLD_PERIOD: f64 = 4096.0;
/// The finer cascade takes over a band once its lattice has this many modes
/// across the cut radius.
const FINE_MODES_AT_CUT: f64 = 8.0;
/// Breaking caps short waves at α g² ω⁻⁵ however high the sea (Phillips
/// 1958), blended in between these multiples of the peak frequency.
const SATURATION: f64 = 0.02;
const SATURATION_ONSET: f64 = 1.5;
const SATURATION_FULL: f64 = 3.0;
/// The share of Cox and Munk's mean square slope the cascades and the long
/// waves draw (the short band raised to it); the rest is roughness.
const SLOPE_DRAWN: f64 = 0.7;
/// The short band raised to those slopes: waves past this many times the
/// peak's wavenumber (under a 25th of its length), which carry slope with
/// little height.
const SHORT_FROM: f64 = 25.0;
/// The sea's seed: one sea for every og.Water of the same wind.
pub const SEED: u32 = 1941;

/// A cascade's tile: its size (m) and its texels across.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cascade {
    pub size: f64,
    pub resolution: u32,
}

/// Wavenumbers |k| (rad/m) a cascade holds: lo ≤ |k| < hi.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveBand {
    pub lo: f64,
    pub hi: f64,
}

/// One cascade's spectrum.
#[derive(Clone, Debug)]
pub struct CascadeSpectrum {
    pub size: f64,
    pub resolution: u32,
    pub band: WaveBand,
    /// Per texel, row z then column x, frequency n = i < N/2 ? i : i − N on
    /// each axis: (Re a, Im a, ±m, 0). The texel's wave is a e^{∓iωt} with
    /// ω = 2π m / FOLD_PERIOD; its twin at −k holds the conjugate, so the
    /// transform is real.
    pub amplitudes: Vec<f32>,
    /// Height variance (m²) this cascade carries.
    pub variance: f64,
    /// Its slope variance (∂y/∂x)² + (∂y/∂z)².
    pub slope_variance: f64,
}

/// The sea the cascades draw.
#[derive(Clone, Debug)]
pub struct Spectrum {
    pub cascades: Vec<CascadeSpectrum>,
    /// Bound on |height| from the cascades: Σ|a|.
    pub max_height: f64,
    /// The horizontal displacement's scale λ (choppiness).
    pub choppiness: f64,
    /// Slope variance the drawn waves and the long ones fall short of Cox and
    /// Munk's total at this wind, left to the facets' roughness.
    pub tail_slope_variance: f64,
}

/// The sea a spectrum is built for: og.Water's, as og_core::ocean reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeaParameters {
    /// Wind speed (m/s).
    pub wind_speed: f64,
    /// Direction the wind blows towards, radians from +X towards +Z.
    pub wind_direction: f64,
    /// og.Water.swell: the height's factor.
    pub swell: f64,
    /// og.Water.choppiness (0 to 1).
    pub choppiness: f64,
    /// Wavenumber under which the waves are the tick's (drawn as Gerstner):
    /// the spectrum draws none of them.
    pub long_cut: f64,
    /// Height variance (m²) the tick's long waves carry.
    pub long_variance: f64,
    /// Slope variance they carry.
    pub long_slope_variance: f64,
}

impl SeaParameters {
    /// og_core::ocean's peak angular frequency for this wind.
    pub fn peak(&self) -> f64 {
        0.877 * GRAVITY / self.wind_speed.max(0.1)
    }

    /// The peak wavelength (m).
    pub fn peak_wavelength(&self) -> f64 {
        2.0 * PI * GRAVITY / self.peak().powi(2)
    }

    /// The significant height og_core::ocean gives the whole sea (m).
    pub fn significant_height(&self) -> f64 {
        0.21 * self.wind_speed.powi(2) / GRAVITY * self.swell.max(0.0)
    }

    /// The tick's waves of `ocean` longer than half the peak's wavelength:
    /// the long band, drawn as they are, and the cut and the variance it
    /// leaves the spectrum.
    pub fn from_ocean(
        wind: glam::Vec2,
        swell: f32,
        choppiness: f32,
        ocean: Option<&og_core::ocean::Ocean>,
    ) -> SeaParameters {
        let mut p = SeaParameters {
            wind_speed: f64::from(wind.length()),
            wind_direction: f64::from(wind.y).atan2(f64::from(wind.x)),
            swell: f64::from(swell),
            choppiness: f64::from(choppiness.clamp(0.0, 1.0)),
            long_cut: 0.0,
            long_variance: 0.0,
            long_slope_variance: 0.0,
        };
        p.long_cut = 2.0 * PI / (0.5 * p.peak_wavelength());
        if let Some(o) = ocean {
            for w in o.waves.iter().filter(|w| f64::from(w.k) < p.long_cut) {
                let a = f64::from(w.amplitude);
                p.long_variance += a * a * 0.5;
                p.long_slope_variance += (f64::from(w.k) * a).powi(2) * 0.5;
            }
        }
        p
    }

    /// The horizontal displacement's scale: og.Water.choppiness 1 is the
    /// most that keeps all but a gale's steepest crests from folding,
    /// sharper as the wind rises (blue-water's calibration, `0.65 + 0.9`
    /// over 3 to 15 m/s, times the choppiness over og's default 0.6).
    pub fn lambda(&self) -> f64 {
        let wind = ((self.wind_speed - 3.0) / 12.0).clamp(0.0, 1.0);
        (self.choppiness / 0.6 * (0.65 + 0.9 * wind)).min(2.0)
    }
}

/// The cascades for `count` tiles (1 to 3), largest first: blue-water's
/// 1024 / 181 / 31 m, all scaled so the largest holds four peak wavelengths
/// (at least 256 m, at most 4 km).
pub fn cascades(count: usize, params: &SeaParameters) -> Vec<Cascade> {
    let first = (4.0 * params.peak_wavelength()).clamp(256.0, 4096.0);
    [1.0, 181.0 / 1024.0, 31.0 / 1024.0][..count.clamp(1, 3)]
        .iter()
        .map(|f| Cascade {
            size: first * f,
            resolution: 256,
        })
        .collect()
}

/// Deep-water angular frequency √(g |k|), quantised to a whole multiple m of
/// 2π / FOLD_PERIOD (never 0 for a wave).
pub fn frequency_multiple(k: f64) -> f64 {
    ((GRAVITY * k).sqrt() * FOLD_PERIOD / (2.0 * PI))
        .round()
        .max(1.0)
}

/// Where each cascade's band starts and ends; disjoint, so no wavevector is
/// drawn twice. The first starts at `long_cut`.
pub fn cascade_bands(cascades: &[Cascade], long_cut: f64) -> Vec<WaveBand> {
    let nyquist = |c: &Cascade| PI * f64::from(c.resolution) / c.size;
    let cuts: Vec<f64> = cascades
        .iter()
        .skip(1)
        .enumerate()
        .map(|(i, fine)| {
            (FINE_MODES_AT_CUT * 2.0 * PI / fine.size)
                .min(nyquist(&cascades[i]) / 2.0)
                .max(long_cut)
        })
        .collect();
    cascades
        .iter()
        .enumerate()
        .map(|(i, cascade)| WaveBand {
            lo: if i > 0 { cuts[i - 1] } else { long_cut },
            hi: if i < cuts.len() {
                cuts[i]
            } else {
                nyquist(cascade)
            },
        })
        .collect()
}

/// JONSWAP energy per unit angular frequency, without α.
pub fn jonswap(omega: f64, peak: f64, gamma: f64) -> f64 {
    if omega.is_nan() || omega <= 0.0 {
        return 0.0;
    }
    let sigma = if omega <= peak { 0.07 } else { 0.09 };
    let r = (-((omega - peak).powi(2)) / (2.0 * sigma * sigma * peak * peak)).exp();
    GRAVITY * GRAVITY / omega.powi(5) * (-1.25 * (peak / omega).powi(4)).exp() * gamma.powf(r)
}

/// JONSWAP at `level`, held at or below the saturation range above the peak.
pub fn sea_spectrum(omega: f64, peak: f64, gamma: f64, level: f64) -> f64 {
    let free = level * jonswap(omega, peak, gamma);
    let t =
        ((omega / peak - SATURATION_ONSET) / (SATURATION_FULL - SATURATION_ONSET)).clamp(0.0, 1.0);
    free + (free.min(SATURATION * GRAVITY.powi(2) / omega.powi(5)) - free) * t * t * (3.0 - 2.0 * t)
}

/// The level α at which `sea_spectrum` holds `variance` (m²) in total.
pub fn spectrum_level(peak: f64, gamma: f64, variance: f64) -> f64 {
    let total = |level: f64| {
        let mut sum = 0.0;
        for i in 0..=600 {
            let omega = peak * 0.3 * 100f64.powf(f64::from(i) / 600.0);
            let weight = if i == 0 || i == 600 { 0.5 } else { 1.0 };
            sum += sea_spectrum(omega, peak, gamma, level) * omega * weight;
        }
        sum * 100f64.ln() / 600.0
    };
    let (mut low, mut high) = (0.0, 1e-3);
    while total(high) < variance {
        high *= 2.0;
    }
    for _ in 0..60 {
        let middle = (low + high) / 2.0;
        if total(middle) < variance {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) / 2.0
}

/// Spreading exponent s of cos^{2s}(θ/2) (Hasselmann et al. 1980).
pub fn spreading_exponent(omega: f64, peak: f64, wind_speed: f64) -> f64 {
    let ratio = omega / peak;
    let age = (wind_speed * peak / GRAVITY).clamp(0.3, 3.0);
    let s = if ratio < 1.05 {
        6.97 * ratio.powf(4.06)
    } else {
        9.77 * ratio.powf(-2.33 - 1.45 * (age - 1.17))
    };
    s.max(0.0)
}

/// ln Γ(x) for x > 0 (Lanczos, g = 7).
fn log_gamma(x: f64) -> f64 {
    if x < 0.5 {
        return (PI / (PI * x).sin()).ln() - log_gamma(1.0 - x);
    }
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let x = x - 1.0;
    let mut sum = C[0];
    for (i, c) in C.iter().enumerate().skip(1) {
        sum += c / (x + i as f64);
    }
    let t = x + 7.5;
    0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// Normalisation of |cos(θ/2)|^{2s} over a full turn.
pub fn spreading_normalization(s: f64) -> f64 {
    ((2.0 * s - 1.0) * LN_2 + 2.0 * log_gamma(s + 1.0) - log_gamma(2.0 * s + 1.0)).exp() / PI
}

/// PCG output permutation (Jarzynski and Olano 2020).
pub fn pcg(v: u32) -> u32 {
    let state = v.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277_803_737);
    (word >> 22) ^ word
}

/// The random stream of one lattice pair: the seed, the cascade and both
/// signed indices.
pub fn mode_hash(seed: u32, cascade: u32, nx: i32, nz: i32) -> u32 {
    pcg((nz as u32).wrapping_add(pcg(
        (nx as u32).wrapping_add(pcg(cascade.wrapping_add(pcg(seed))))
    )))
}

fn unit(bits: u32) -> f64 {
    (f64::from(bits) + 0.5) / 4_294_967_296.0
}

/// The signed frequency of index `i` of `n`.
pub fn signed(i: usize, n: usize) -> i32 {
    if i < n / 2 {
        i as i32
    } else {
        i as i32 - n as i32
    }
}

/// Every cascade's amplitudes for `params`. Waves travel towards the wind.
pub fn build(cascades: &[Cascade], params: &SeaParameters) -> Spectrum {
    let bands = cascade_bands(cascades, params.long_cut);
    let height = params.significant_height();
    let calm = !(height.is_finite() && height > 0.0 && params.wind_speed > 0.1);
    let peak = params.peak();
    let gamma = 3.3;
    let level = if calm {
        0.0
    } else {
        spectrum_level(peak, gamma, (height / 4.0).powi(2))
    };
    let (sin_wind, cos_wind) = params.wind_direction.sin_cos();
    let mut drawn = 0.0;
    let mut built: Vec<CascadeSpectrum> = cascades
        .iter()
        .enumerate()
        .map(|(index, cascade)| {
            let n = cascade.resolution as usize;
            let half = (n / 2) as i32;
            let band = bands[index];
            let mut amplitudes = vec![0f32; n * n * 4];
            let dk = 2.0 * PI / cascade.size;
            if !calm {
                for z in 0..n {
                    for x in 0..n {
                        let (nx, nz) = (signed(x, n), signed(z, n));
                        // One draw per ± pair, from its canonical member.
                        if nz < 0 || (nz == 0 && nx <= 0) {
                            continue;
                        }
                        let kx = f64::from(nx) * dk;
                        let kz = f64::from(nz) * dk;
                        let k = kx.hypot(kz);
                        // The Nyquist column has no distinct twin.
                        if k < band.lo || k >= band.hi || nx == -half {
                            continue;
                        }
                        let omega = (GRAVITY * k).sqrt();
                        let s = spreading_exponent(omega, peak, params.wind_speed);
                        let cos_theta = (kx * cos_wind + kz * sin_wind) / k;
                        let toward = ((1.0 + cos_theta) / 2.0).max(0.0).sqrt().powf(2.0 * s);
                        let away = ((1.0 - cos_theta) / 2.0).max(0.0).sqrt().powf(2.0 * s);
                        let density = sea_spectrum(omega, peak, gamma, level) * GRAVITY
                            / (2.0 * omega)
                            * spreading_normalization(s)
                            / k
                            * dk
                            * dk;
                        let pair = density * (toward + away);
                        if pair.is_nan() || pair <= 0.0 {
                            continue;
                        }
                        let bits = mode_hash(SEED, index as u32, nx, nz);
                        let radius = (-2.0 * unit(pcg(bits.wrapping_add(1))).ln()).sqrt();
                        let angle = 2.0 * PI * unit(pcg(bits.wrapping_add(2)));
                        let scale = (pair / 4.0).sqrt() * radius;
                        let (re, im) = (scale * angle.cos(), scale * angle.sin());
                        let forward = unit(pcg(bits.wrapping_add(3))) * (toward + away) < toward;
                        let m = frequency_multiple(k);
                        let here = (z * n + x) * 4;
                        let twin = (((n - z) % n) * n + (n - x) % n) * 4;
                        let (at, other) = if forward { (here, twin) } else { (twin, here) };
                        amplitudes[at] = re as f32;
                        amplitudes[at + 1] = im as f32;
                        amplitudes[at + 2] = m as f32;
                        amplitudes[other] = re as f32;
                        amplitudes[other + 1] = (-im) as f32;
                        amplitudes[other + 2] = (-m) as f32;
                        drawn += 2.0 * (re * re + im * im);
                    }
                }
            }
            CascadeSpectrum {
                size: cascade.size,
                resolution: cascade.resolution,
                band,
                amplitudes,
                variance: 0.0,
                slope_variance: 0.0,
            }
        })
        .collect();
    // The cascades carry what the long waves do not.
    let want = ((height / 4.0).powi(2) - params.long_variance).max(0.0);
    let scale = if drawn > 0.0 && !calm {
        (want / drawn).sqrt()
    } else {
        0.0
    };
    // The waves under a 25th of the peak's length carry what the eye sees, the slopes; the spectrum's own (held to
    // Phillips' saturation) leave a sea of some 0.015 mean square slope at any wind, and the rest of Cox and Munk's
    // (0.003 + 0.00512 U) as unresolved roughness made a gale's surface a uniform matte grey. As og_core::ocean does
    // for its waves, the short band is raised until the drawn slopes hold SLOPE_DRAWN of Cox and Munk's, times the
    // swell, the longer band lowered to keep the height; the rest is the facets' roughness.
    let short_k = SHORT_FROM * params.peak().powi(2) / GRAVITY;
    let k_of = |texel: usize, n: usize, dk: f64| {
        let (fx, fz) = (
            f64::from(signed(texel % n, n)),
            f64::from(signed(texel / n, n)),
        );
        (fx * fx + fz * fz).sqrt() * dk
    };
    // Each band's height and slope variance as rescaled: the short band, and the cascades' band longer than it.
    let (mut long_h, mut short_h, mut long_s, mut short_s) = (0.0, 0.0, 0.0, 0.0);
    for cascade in &built {
        let n = cascade.resolution as usize;
        let dk = 2.0 * PI / cascade.size;
        for (texel, a) in cascade.amplitudes.as_chunks::<4>().0.iter().enumerate() {
            if a[2] == 0.0 {
                continue;
            }
            let k = k_of(texel, n, dk);
            let e = (f64::from(a[0]).powi(2) + f64::from(a[1]).powi(2)) * scale * scale;
            if k > short_k {
                (short_h, short_s) = (short_h + e, short_s + e * k * k);
            } else {
                (long_h, long_s) = (long_h + e, long_s + e * k * k);
            }
        }
    }
    let cox_munk = 0.003 + 0.00512 * params.wind_speed;
    let want_slopes = SLOPE_DRAWN * cox_munk * params.swell.min(1.0);
    // The short band's energy times y and the longer band's times x, so the height stays the sea's (x Hl + y Hs = W)
    // and the slopes reach the share (Ll + x Sl + y Ss = T); the short band only ever raised, the longer one kept to
    // at least half its energy (else the slopes fall short, the height whole).
    let (want_h, mut x, mut y) = (long_h + short_h, 1.0, 1.0);
    if !calm && short_s > 0.0 && want_slopes > params.long_slope_variance + long_s + short_s {
        let t = want_slopes - params.long_slope_variance;
        let det = long_h * short_s - short_h * long_s;
        if det.abs() > 1e-30 {
            x = (want_h * short_s - short_h * t) / det;
            y = (long_h * t - long_s * want_h) / det;
        }
        if !(x >= 0.5 && y >= 1.0) {
            x = 0.5;
            y = ((want_h - x * long_h) / short_h.max(1e-30)).max(1.0);
        }
    }
    let (long_f, lift) = (x.sqrt(), y.sqrt());
    let mut bound = 0.0;
    for cascade in &mut built {
        let n = cascade.resolution as usize;
        let dk = 2.0 * PI / cascade.size;
        let (mut variance, mut slope_variance) = (0.0, 0.0);
        for (texel, a) in cascade
            .amplitudes
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .enumerate()
        {
            if a[2] == 0.0 {
                continue;
            }
            let k = k_of(texel, n, dk);
            let f = if k > short_k {
                scale * lift
            } else {
                scale * long_f
            };
            a[0] = (f64::from(a[0]) * f) as f32;
            a[1] = (f64::from(a[1]) * f) as f32;
            let energy = f64::from(a[0]).powi(2) + f64::from(a[1]).powi(2);
            bound += energy.sqrt();
            variance += energy;
            slope_variance += energy * k * k;
        }
        cascade.variance = variance;
        cascade.slope_variance = slope_variance;
    }
    let slopes: f64 =
        built.iter().map(|c| c.slope_variance).sum::<f64>() + params.long_slope_variance;
    let tail_slope_variance = if calm {
        0.0
    } else {
        (cox_munk - slopes).max(0.0)
    };
    Spectrum {
        cascades: built,
        max_height: bound,
        choppiness: params.lambda(),
        tail_slope_variance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cascades and the tick's long waves together hold the sea's
    /// significant height, (Hs / 4)², for a breeze, a fresh wind and a gale,
    /// the three cascades drawing SLOPE_DRAWN of Cox and Munk's slopes;
    /// the cascades' bands are disjoint, the first starting where the long
    /// band ends; and a calm (no wind) has no waves.
    #[test]
    fn the_cascades_and_the_long_waves_hold_the_sea_s_height() {
        for (wind, swell) in [(6.0, 1.0), (9.0, 0.7), (20.0, 1.0)] {
            let w = glam::Vec2::new(0.6, -0.8) * wind;
            let ocean = og_core::ocean::Ocean::new(w, swell, 0.8).expect("a sea");
            let params = SeaParameters::from_ocean(w, swell, 0.8, Some(&ocean));
            assert!(params.long_variance > 0.0, "{wind} m/s: no long band");
            for count in 1..=3 {
                let tiles = cascades(count, &params);
                let sea = build(&tiles, &params);
                let total: f64 =
                    sea.cascades.iter().map(|c| c.variance).sum::<f64>() + params.long_variance;
                let want = (params.significant_height() / 4.0).powi(2);
                assert!(
                    (total - want).abs() < 1e-3 * want,
                    "{wind} m/s, {count} cascades: {total} m² against {want}"
                );
                let slopes = sea.cascades.iter().map(|c| c.slope_variance).sum::<f64>()
                    + params.long_slope_variance;
                let cox_munk = 0.003 + 0.00512 * f64::from(wind);
                if count == 3 {
                    assert!(
                        slopes > 0.69 * cox_munk * f64::from(swell.min(1.0)),
                        "{wind} m/s: slopes {slopes} of Cox and Munk's {cox_munk}"
                    );
                }
                assert_eq!(sea.cascades[0].band.lo, params.long_cut);
                for pair in sea.cascades.windows(2) {
                    assert_eq!(pair[0].band.hi, pair[1].band.lo);
                    assert!(pair[0].size > pair[1].size);
                }
                assert!(sea.choppiness > 0.0 && sea.choppiness <= 2.0);
            }
        }
        let calm = SeaParameters::from_ocean(glam::Vec2::ZERO, 1.0, 0.6, None);
        let sea = build(&cascades(3, &calm), &calm);
        assert!(sea.cascades.iter().all(|c| c.variance == 0.0));
    }
}
