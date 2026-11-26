//! Deterministic procedural sound effects synthesizer (shared by `og-cook`'s
//! `type sfx` pipeline and `og-runtime`'s `og.audio.synth` /
//! `og.audio.createSynthSound` APIs).
//!
//! Each layer is an oscillator (`sine`, `square`, `saw`, `triangle`, `noise`)
//! with PolyBLEP anti-aliasing on discontinuous waveforms, pitch slide /
//! frequency sweep, vibrato, arpeggio, optional FM modulation, click-free
//! attack/sustain/decay/release envelope with smoothed punch, Cytomic TPT SVF
//! / one-pole low-pass and high-pass filtering, soft-clip `drive`, and a final
//! DC blocker. Every transcendental uses [`og_core::math::double`] and every
//! random draw uses SplitMix64, so outputs are bit-identical across hosts.

use og_core::hash::splitmix64_next;
use og_core::math::double;
use std::f64::consts::{LN_2, LOG2_E, PI, TAU};

/// Default synthesis sample rate (Hz).
pub const RATE: u32 = 48_000;

/// Maximum rendered duration for a synthesised sound (seconds).
pub const MAX_DURATION: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Square,
    Saw,
    Sine,
    Triangle,
    Noise,
}

impl Wave {
    pub fn parse(name: &str) -> Option<Wave> {
        match name {
            "square" => Some(Wave::Square),
            "saw" | "sawtooth" => Some(Wave::Saw),
            "sine" | "sin" => Some(Wave::Sine),
            "triangle" | "tri" => Some(Wave::Triangle),
            "noise" => Some(Wave::Noise),
            _ => None,
        }
    }
}

/// An authored mono recording used instead of an oscillator.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub samples: Vec<f32>,
    pub rate: u32,
    /// Frequency at which the recording plays at its original speed.
    pub reference: f64,
    pub looping: bool,
}

/// One oscillator or sample layer, with the same envelope and filters.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub wave: Wave,
    pub sample: Option<Sample>,
    /// Start frequency, Hz. For `Wave::Noise`, `>= 24_000 Hz` (or `<= 0`) runs
    /// full-rate white noise; lower values run sample-and-hold at `freq` Hz.
    pub freq: f64,
    /// Optional target end frequency, Hz (overrides `slide` when set).
    pub freq_end: Option<f64>,
    /// Pitch slide, octaves per second.
    pub slide: f64,
    /// At `arp_at` seconds the frequency is multiplied by `arp` (0: never).
    pub arp_at: f64,
    pub arp: f64,
    pub vib_depth: f64,
    pub vib_rate: f64,
    /// Envelope, seconds.
    pub attack: f64,
    pub sustain: f64,
    pub decay: f64,
    /// Level held across `sustain` (default 1.0; < 1.0 decays from peak over `decay` before `sustain` and fades over `release`).
    pub sustain_level: f64,
    /// Release tail after sustain, seconds (0: unused; default uses attack/sustain/decay).
    pub release: f64,
    /// Extra level during the sustain, fading to none (0..1).
    pub punch: f64,
    pub duty: f64,
    /// Low-pass cutoff, Hz (0: none), and its slide in octaves per second.
    pub lowpass: f64,
    pub lp_slide: f64,
    /// Optional resonance Q for low-pass (> 0 uses a 2-pole SVF; 0 uses one-pole).
    pub q: f64,
    pub highpass: f64,
    /// Optional FM modulator frequency (Hz) and modulation index (radians; 0: off).
    pub fm_freq: f64,
    pub fm_index: f64,
    /// Optional soft-clip saturation drive (> 0 enables tanh waveshaping).
    pub drive: f64,
    pub gain: f64,
    /// Starts this many seconds in.
    pub delay: f64,
}

impl Default for Layer {
    fn default() -> Self {
        L
    }
}

pub const L: Layer = Layer {
    wave: Wave::Square,
    sample: None,
    freq: 440.0,
    freq_end: None,
    slide: 0.0,
    arp_at: 0.0,
    arp: 0.0,
    vib_depth: 0.0,
    vib_rate: 0.0,
    attack: 0.002,
    sustain: 0.05,
    decay: 0.15,
    sustain_level: 1.0,
    release: 0.0,
    punch: 0.0,
    duty: 0.5,
    lowpass: 0.0,
    lp_slide: 0.0,
    q: 0.0,
    highpass: 0.0,
    fm_freq: 0.0,
    fm_index: 0.0,
    drive: 0.0,
    gain: 0.5,
    delay: 0.0,
};

/// Built-in preset catalogue: `(name, description)`.
pub const PRESETS: &[(&str, &str)] = &[
    ("coin", "a bright two-note pickup chime"),
    ("pickup", "a short rising blip for items"),
    ("powerup", "a rising vibrato sweep with an arpeggio"),
    ("jump", "a quick upward square chirp"),
    ("land", "a soft low thud"),
    ("footstep", "a short filtered noise tick"),
    ("hit", "a punchy noise crack with a low body"),
    ("hurt", "a falling buzzy saw"),
    ("explosion", "a long low-passed noise boom"),
    ("laser", "a falling square zap"),
    ("whoosh", "a band-passed noise swell"),
    ("door", "a low creak and thump"),
    ("death", "a long falling buzz"),
    ("click", "a tiny UI tick"),
    ("select", "a two-step UI confirm"),
    ("back", "a falling UI cancel"),
    ("error", "a low double buzz"),
    ("levelup", "a three-note rising fanfare"),
];

/// Full-rate white noise marker frequency for built-in presets (stays well
/// above Nyquist across typical pitch scales, eliminating the 1 kHz cliff).
const WHITE_NOISE_FREQ: f64 = 48_000.0;

/// Built-in preset layers by name.
pub fn preset_layers(name: &str) -> Option<Vec<Layer>> {
    use Wave::*;
    Some(match name {
        "coin" => vec![Layer {
            freq: 988.0,
            arp_at: 0.06,
            arp: 1.335,
            sustain: 0.05,
            decay: 0.22,
            punch: 0.4,
            duty: 0.25,
            gain: 0.35,
            ..L
        }],
        "pickup" => vec![Layer {
            freq: 660.0,
            slide: 2.5,
            sustain: 0.03,
            decay: 0.09,
            duty: 0.3,
            gain: 0.35,
            ..L
        }],
        "powerup" => vec![Layer {
            freq: 330.0,
            slide: 2.2,
            vib_depth: 0.04,
            vib_rate: 18.0,
            arp_at: 0.18,
            arp: 1.5,
            sustain: 0.25,
            decay: 0.25,
            duty: 0.4,
            gain: 0.3,
            ..L
        }],
        "jump" => vec![Layer {
            freq: 280.0,
            slide: 3.5,
            sustain: 0.05,
            decay: 0.12,
            duty: 0.35,
            lowpass: 5000.0,
            gain: 0.35,
            ..L
        }],
        "land" => vec![
            Layer {
                wave: Sine,
                freq: 120.0,
                slide: -3.0,
                sustain: 0.01,
                decay: 0.12,
                gain: 0.6,
                ..L
            },
            Layer {
                wave: Noise,
                freq: WHITE_NOISE_FREQ,
                sustain: 0.0,
                decay: 0.06,
                lowpass: 900.0,
                gain: 0.25,
                ..L
            },
        ],
        "footstep" => vec![Layer {
            wave: Noise,
            freq: WHITE_NOISE_FREQ,
            attack: 0.001,
            sustain: 0.0,
            decay: 0.05,
            lowpass: 2200.0,
            highpass: 300.0,
            gain: 0.45,
            ..L
        }],
        "hit" => vec![
            Layer {
                wave: Noise,
                freq: WHITE_NOISE_FREQ,
                attack: 0.0005,
                sustain: 0.02,
                decay: 0.12,
                punch: 0.6,
                lowpass: 6000.0,
                lp_slide: -4.0,
                gain: 0.32,
                ..L
            },
            Layer {
                wave: Sine,
                freq: 180.0,
                slide: -2.0,
                sustain: 0.02,
                decay: 0.12,
                punch: 0.5,
                gain: 0.36,
                ..L
            },
        ],
        "hurt" => vec![Layer {
            wave: Saw,
            freq: 440.0,
            slide: -2.8,
            sustain: 0.06,
            decay: 0.2,
            punch: 0.3,
            lowpass: 3500.0,
            gain: 0.35,
            ..L
        }],
        "explosion" => vec![
            Layer {
                wave: Noise,
                freq: WHITE_NOISE_FREQ,
                attack: 0.002,
                sustain: 0.1,
                decay: 0.9,
                punch: 0.5,
                lowpass: 2500.0,
                lp_slide: -2.0,
                gain: 0.45,
                ..L
            },
            Layer {
                wave: Sine,
                freq: 70.0,
                slide: -1.0,
                sustain: 0.1,
                decay: 0.5,
                gain: 0.4,
                ..L
            },
        ],
        "laser" => vec![Layer {
            freq: 1500.0,
            slide: -5.0,
            sustain: 0.04,
            decay: 0.16,
            duty: 0.2,
            gain: 0.3,
            ..L
        }],
        "whoosh" => vec![Layer {
            wave: Noise,
            freq: WHITE_NOISE_FREQ,
            attack: 0.15,
            sustain: 0.05,
            decay: 0.25,
            lowpass: 900.0,
            lp_slide: 2.5,
            highpass: 250.0,
            gain: 0.5,
            ..L
        }],
        "door" => vec![
            Layer {
                wave: Saw,
                freq: 95.0,
                slide: 0.6,
                vib_depth: 0.08,
                vib_rate: 11.0,
                attack: 0.05,
                sustain: 0.25,
                decay: 0.15,
                lowpass: 900.0,
                gain: 0.3,
                ..L
            },
            Layer {
                wave: Sine,
                freq: 90.0,
                slide: -2.0,
                sustain: 0.02,
                decay: 0.15,
                delay: 0.42,
                gain: 0.6,
                ..L
            },
        ],
        "death" => vec![Layer {
            freq: 520.0,
            slide: -1.6,
            vib_depth: 0.05,
            vib_rate: 9.0,
            sustain: 0.4,
            decay: 0.5,
            duty: 0.4,
            lowpass: 3000.0,
            gain: 0.3,
            ..L
        }],
        "click" => vec![Layer {
            wave: Triangle,
            freq: 1800.0,
            attack: 0.0005,
            sustain: 0.0,
            decay: 0.025,
            gain: 0.35,
            ..L
        }],
        "select" => vec![Layer {
            wave: Triangle,
            freq: 880.0,
            arp_at: 0.05,
            arp: 1.5,
            sustain: 0.05,
            decay: 0.08,
            gain: 0.35,
            ..L
        }],
        "back" => vec![Layer {
            wave: Triangle,
            freq: 880.0,
            arp_at: 0.05,
            arp: 0.75,
            sustain: 0.05,
            decay: 0.08,
            gain: 0.35,
            ..L
        }],
        "error" => vec![
            Layer {
                freq: 150.0,
                sustain: 0.08,
                decay: 0.03,
                duty: 0.5,
                lowpass: 1800.0,
                gain: 0.3,
                ..L
            },
            Layer {
                freq: 150.0,
                sustain: 0.08,
                decay: 0.05,
                duty: 0.5,
                lowpass: 1800.0,
                delay: 0.14,
                gain: 0.3,
                ..L
            },
        ],
        "levelup" => vec![
            Layer {
                freq: 523.25,
                sustain: 0.08,
                decay: 0.05,
                duty: 0.3,
                gain: 0.25,
                ..L
            },
            Layer {
                freq: 659.25,
                sustain: 0.08,
                decay: 0.05,
                duty: 0.3,
                delay: 0.12,
                gain: 0.25,
                ..L
            },
            Layer {
                freq: 783.99,
                sustain: 0.2,
                decay: 0.3,
                duty: 0.3,
                vib_depth: 0.01,
                vib_rate: 6.0,
                delay: 0.24,
                gain: 0.3,
                ..L
            },
        ],
        _ => return None,
    })
}

/// How a sound is varied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Variation {
    pub seed: u32,
    pub pitch: f64,
    pub length: f64,
    pub volume: f64,
}

impl Default for Variation {
    fn default() -> Self {
        Variation {
            seed: 0,
            pitch: 1.0,
            length: 1.0,
            volume: 1.0,
        }
    }
}

/// SplitMix64-backed deterministic PRNG (full 64-bit avalanche from draw 0).
struct SynthRng(u64);

impl SynthRng {
    fn new(seed: u64) -> Self {
        let mut s = seed;
        let _ = splitmix64_next(&mut s);
        SynthRng(s)
    }

    fn next(&mut self) -> f64 {
        (splitmix64_next(&mut self.0) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// ±`amount` around 1.
    fn around(&mut self, amount: f64) -> f64 {
        1.0 + (self.next() * 2.0 - 1.0) * amount
    }
}

/// Deterministic `tanh` built on [`og_core::math::double::exp`].
#[inline]
fn tanh_det(x: f64) -> f64 {
    let e = double::exp(-2.0 * x.abs());
    let t = (1.0 - e) / (1.0 + e);
    if x < 0.0 { -t } else { t }
}

/// Deterministic `2^x`.
#[inline]
fn exp2_det(x: f64) -> f64 {
    double::exp(x * LN_2)
}

/// Deterministic `log2(x)` for `x > 0`.
#[inline]
fn log2_det(x: f64) -> f64 {
    double::ln(x) * LOG2_E
}

/// Second-order polynomial band-limited step (PolyBLEP) for anti-aliased
/// sawtooth and pulse oscillators.
#[inline]
fn poly_blep(t: f64, dt_phase: f64) -> f64 {
    if dt_phase <= 0.0 || dt_phase >= 0.5 {
        return 0.0;
    }
    if t < dt_phase {
        let u = t / dt_phase;
        u + u - u * u - 1.0
    } else if t > 1.0 - dt_phase {
        let u = (t - 1.0) / dt_phase;
        u * u + u + u + 1.0
    } else {
        0.0
    }
}

/// A parsed or programmatic SFX description: optional preset plus layers and
/// variation parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct SfxDesc {
    pub preset: Option<String>,
    pub layers: Vec<Layer>,
    pub variation: Variation,
}

/// Renders a preset: mono 16-bit PCM at [`RATE`]. `None` for an unknown name.
pub fn render(name: &str, v: Variation) -> Option<Vec<i16>> {
    let layers = preset_layers(name)?;
    Some(render_layers(&layers, v))
}

/// Renders a parsed [`SfxDesc`]: mono 16-bit PCM at [`RATE`].
pub fn render_desc(desc: &SfxDesc) -> Vec<i16> {
    render_layers(&desc.layers, desc.variation)
}

/// Renders arbitrary oscillator layers into normalised mono `f32` PCM (`[-1, 1]`)
/// at [`RATE`].
pub fn render_layers_f32(base_layers: &[Layer], v: Variation) -> Vec<f32> {
    let mut layers = base_layers.to_vec();
    let mut rng = SynthRng::new(
        0x9e37_79b9_7f4a_7c15 ^ (u64::from(v.seed).wrapping_mul(0xbf58_476d_1ce4_e5b9)),
    );
    let pitch = if v.pitch.is_finite() {
        v.pitch.clamp(0.1, 10.0)
    } else {
        1.0
    };
    let len_scale = if v.length.is_finite() {
        v.length.clamp(0.1, 10.0)
    } else {
        1.0
    };
    for l in &mut layers {
        if l.sample.is_some() || l.wave != Wave::Noise || l.freq < f64::from(RATE) * 0.5 {
            l.freq *= pitch;
            if let Some(end) = l.freq_end {
                l.freq_end = Some(end * pitch);
            }
        }
        l.fm_freq *= pitch;
        l.attack *= len_scale;
        l.sustain *= len_scale;
        l.decay *= len_scale;
        l.release *= len_scale;
        l.delay *= len_scale;
        l.arp_at *= len_scale;
        // Preserve total pitch/filter sweep interval when stretching duration.
        l.slide /= len_scale;
        l.lp_slide /= len_scale;
        if v.seed != 0 {
            let f_jit = rng.around(0.06);
            if l.sample.is_some() || l.wave != Wave::Noise || l.freq < f64::from(RATE) * 0.5 {
                l.freq *= f_jit;
                if let Some(end) = l.freq_end {
                    l.freq_end = Some(end * f_jit);
                }
            }
            l.slide *= rng.around(0.2);
            l.decay *= rng.around(0.15);
            l.duty = (l.duty * rng.around(0.2)).clamp(0.05, 0.95);
        }
        let total_len = l.attack + l.sustain + l.decay + l.release;
        if let Some(end) = l.freq_end
            && total_len > 1e-6
            && l.freq > 0.0
            && end > 0.0
        {
            l.slide = log2_det(end / l.freq) / total_len;
        }
    }
    let tail_of = |l: &Layer| -> f64 {
        // Resonant low-pass filters ring slightly past envelope end; allow a
        // short tail (capped) so high-Q sweeps decay cleanly without clicks.
        if l.lowpass > 0.0 && l.q > 0.7 {
            (0.008 * l.q.clamp(0.5, 20.0)).min(0.12)
        } else {
            0.0
        }
    };
    let total = layers
        .iter()
        .map(|l| l.delay + l.attack + l.sustain + l.decay + l.release + tail_of(l))
        .fold(0.0, f64::max)
        .min(MAX_DURATION);
    let frames = ((total + 0.01) * f64::from(RATE)).ceil().max(1.0) as usize;
    let mut out = vec![0.0f64; frames];
    let dt = 1.0 / f64::from(RATE);
    let declick = 0.002; // 2 ms de-click edge smoothing
    for (k, l) in layers.iter().enumerate() {
        let mut noise = SynthRng::new(
            0x2545_f491_4f6c_dd1d
                ^ ((k as u64 + 1).wrapping_mul(0x94d0_49bb_1331_11eb))
                ^ (u64::from(v.seed) << 32),
        );
        let mut phase = 0.0f64;
        let mut fm_phase = 0.0f64;
        let mut cursor = 0.0f64;
        let mut held = 0.0f64;
        let (mut lp, mut hp_in, mut hp) = (0.0f64, 0.0f64, 0.0f64);
        let (mut svf_ic1, mut svf_ic2) = (0.0f64, 0.0f64);
        let start = (l.delay * f64::from(RATE)) as usize;
        let len = l.attack + l.sustain + l.decay + l.release;
        let tail = tail_of(l);
        let end_time = len + tail;
        let adsr_mode = l.release > 0.0 || (l.sustain_level - 1.0).abs() > 1e-12;
        let sustain_level = l.sustain_level.clamp(0.0, 1.0);
        let duty = l.duty.clamp(0.01, 0.99);
        let drive_norm = if l.drive > 0.0 {
            tanh_det(l.drive).max(1e-6)
        } else {
            1.0
        };
        let full_rate_noise =
            l.wave == Wave::Noise && (l.freq <= 0.0 || l.freq >= f64::from(RATE) * 0.5);
        for (i, o) in out.iter_mut().enumerate().skip(start) {
            let t = (i - start) as f64 * dt;
            if t > end_time {
                break;
            }
            let mut env = if t > len {
                0.0
            } else if !adsr_mode {
                if t < l.attack {
                    t / l.attack.max(1e-6)
                } else if t < l.attack + l.sustain {
                    let st = t - l.attack;
                    // Smooth punch onset over >= 1 ms so punch never clicks.
                    let punch_in = (st / 0.001).min(1.0);
                    1.0 + l.punch * punch_in * (1.0 - st / l.sustain.max(1e-6))
                } else {
                    let u = (1.0 - (t - l.attack - l.sustain) / l.decay.max(1e-6)).max(0.0);
                    u * u
                }
            } else if t < l.attack {
                t / l.attack.max(1e-6)
            } else if t < l.attack + l.decay {
                let u = (t - l.attack) / l.decay.max(1e-6);
                1.0 + (sustain_level - 1.0) * u
            } else if t < l.attack + l.decay + l.sustain {
                let st = t - l.attack - l.decay;
                let punch_in = (st / 0.001).min(1.0);
                sustain_level + l.punch * punch_in * (1.0 - st / l.sustain.max(1e-6))
            } else {
                let u = (1.0 - (t - l.attack - l.decay - l.sustain) / l.release.max(1e-6)).max(0.0);
                sustain_level * u * u
            };
            // Smooth de-click fade at the end of a layer's active window.
            if end_time > declick && t > end_time - declick {
                let fade = ((end_time - t) / declick).clamp(0.0, 1.0);
                env *= fade;
            }
            let mut f = l.freq * exp2_det(l.slide * t);
            if l.arp > 0.0 && t >= l.arp_at {
                f *= l.arp;
            }
            if l.vib_depth > 0.0 && l.vib_rate > 0.0 {
                f *= 1.0 + l.vib_depth * double::sin(TAU * l.vib_rate * t);
            }
            let f = if l.wave == Wave::Noise {
                f.clamp(20.0, f64::from(RATE))
            } else {
                f.clamp(20.0, 20_000.0)
            };
            let dt_phase = f * dt;
            let prev = phase;
            phase = (phase + dt_phase).fract();
            let mod_phase = if l.fm_freq > 0.0 && l.fm_index != 0.0 {
                fm_phase = (fm_phase + l.fm_freq.clamp(1.0, 20_000.0) * dt).fract();
                (phase + (l.fm_index / TAU) * double::sin(TAU * fm_phase)).rem_euclid(1.0)
            } else {
                phase
            };
            let s = if let Some(sample) = &l.sample {
                let count = sample.samples.len();
                let at = cursor.floor() as usize;
                let value = if count == 0 || (!sample.looping && at >= count) {
                    0.0
                } else {
                    let index = at % count;
                    let next = if sample.looping {
                        (index + 1) % count
                    } else {
                        (index + 1).min(count - 1)
                    };
                    let a = f64::from(sample.samples[index]);
                    a + (f64::from(sample.samples[next]) - a) * cursor.fract()
                };
                cursor += f64::from(sample.rate) * dt * f / sample.reference;
                value
            } else {
                match l.wave {
                    Wave::Square => {
                        let raw = if mod_phase < duty { 1.0 } else { -1.0 };
                        let shifted = (mod_phase + 1.0 - duty).fract();
                        // PolyBLEP at both rising (0) and falling (`duty`) edges,
                        // plus analytical DC removal for asymmetric pulse widths.
                        let blep = poly_blep(mod_phase, dt_phase) - poly_blep(shifted, dt_phase);
                        raw + blep - (2.0 * duty - 1.0)
                    }
                    Wave::Saw => {
                        let raw = mod_phase * 2.0 - 1.0;
                        raw - poly_blep(mod_phase, dt_phase)
                    }
                    Wave::Sine => double::sin(TAU * mod_phase),
                    Wave::Triangle => 1.0 - 4.0 * (mod_phase - 0.5).abs(),
                    Wave::Noise => {
                        if full_rate_noise {
                            noise.next() * 2.0 - 1.0
                        } else {
                            if phase < prev || i == start {
                                held = noise.next() * 2.0 - 1.0;
                            }
                            held
                        }
                    }
                }
            };
            let mut x = s * env;
            if l.drive > 0.0 {
                x = tanh_det(x * l.drive) / drive_norm;
            }
            if l.lowpass > 0.0 {
                let c = (l.lowpass * exp2_det(l.lp_slide * t)).clamp(20.0, 20_000.0);
                if l.q > 0.0 {
                    // Cytomic TPT 2-pole SVF (unconditionally stable up to Nyquist).
                    let g = double::tan(PI * c * dt);
                    let damp = 1.0 / l.q.clamp(0.5, 20.0);
                    let a1 = 1.0 / (1.0 + g * (g + damp));
                    let a2 = g * a1;
                    let a3 = g * a2;
                    let v3 = x - svf_ic2;
                    let v1 = a1 * svf_ic1 + a2 * v3;
                    let v2 = svf_ic2 + a2 * svf_ic1 + a3 * v3;
                    svf_ic1 = 2.0 * v1 - svf_ic1;
                    svf_ic2 = 2.0 * v2 - svf_ic2;
                    x = v2;
                } else {
                    let a = 1.0 - double::exp(-TAU * c * dt);
                    lp += a * (x - lp);
                    x = lp;
                }
            }
            if l.highpass > 0.0 {
                let a = 1.0 / (1.0 + TAU * l.highpass * dt);
                hp = a * (hp + x - hp_in);
                hp_in = x;
                x = hp;
            }
            if end_time > declick && t > end_time - declick {
                x *= ((end_time - t) / declick).clamp(0.0, 1.0);
            }
            *o += x * l.gain;
        }
    }
    // Final 1st-order DC blocker (~15 Hz corner at 48 kHz) so asymmetric
    // waveshaping or sub-cycle transients leave zero DC offset.
    let dc_r = 1.0 - TAU * 15.0 * dt;
    let (mut dc_x1, mut dc_y1) = (0.0f64, 0.0f64);
    let gain = if v.volume.is_finite() {
        v.volume.clamp(0.0, 4.0)
    } else {
        1.0
    };
    out.into_iter()
        .map(|x| {
            let y = x - dc_x1 + dc_r * dc_y1;
            dc_x1 = x;
            dc_y1 = y;
            (y * gain).clamp(-1.0, 1.0) as f32
        })
        .collect()
}

/// Renders arbitrary oscillator layers with `v`: mono 16-bit PCM at [`RATE`].
pub fn render_layers(base_layers: &[Layer], v: Variation) -> Vec<i16> {
    render_layers_f32(base_layers, v)
        .into_iter()
        .map(|s| (f64::from(s) * 32767.0).round() as i16)
        .collect()
}
