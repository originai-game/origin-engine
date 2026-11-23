//! Procedural sounds (`type sfx` .ogt): a library of game sound presets and
//! a declarative multi-layer synth DSL, synthesised at cook time into ordinary
//! sound blobs. Each layer is an oscillator (`sine`, `square`, `saw`,
//! `triangle`, `noise`) with pitch slide / frequency sweep, vibrato, arpeggio,
//! optional FM modulation, an attack/sustain/decay/release envelope with punch,
//! low/high-pass filtering (optional resonance `q`), and soft-clip `drive`;
//! `seed` varies it slightly, `pitch`, `length` and `volume` scale it.
//! Everything is f64 arithmetic and a seeded xorshift, so a sound renders the
//! same samples on every machine.
//!
//! ```text
//! type sfx
//! version 1
//! ---
//! preset "coin"
//! seed 3          # a variation (0: the preset as designed)
//! pitch 1.2       # frequency multiplier
//! length 1        # duration multiplier
//! volume 0.8
//! ```
//!
//! Or define custom layers directly (standalone or layered on a `preset`):
//!
//! ```text
//! type sfx
//! version 1
//! ---
//! volume 0.9
//! layer {
//!   wave sine
//!   freq 440
//!   freq_end 110
//!   attack 0.002
//!   sustain 0.04
//!   decay 0.2
//!   lowpass 2400
//!   gain 0.7
//! }
//! ```

use super::{
    Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, ext, stamp,
};
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;
use og_core::ogt::{Document, Entry};
use std::f64::consts::TAU;

/// The `.ogt` type.
pub const SFX_TYPE: &str = "sfx";

/// Bumped when the synthesiser or a preset changes, so cooked presets re-cook.
pub const SYNTH_VERSION: &str = "1";

/// The rate presets render at.
pub const RATE: u32 = 48_000;

/// Maximum rendered duration for a custom `type sfx` sound (seconds).
const MAX_DURATION: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Square,
    Saw,
    Sine,
    Triangle,
    Noise,
}

impl Wave {
    fn parse(name: &str) -> Option<Wave> {
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

/// One oscillator layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    pub wave: Wave,
    /// Start frequency, Hz.
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

const L: Layer = Layer {
    wave: Wave::Square,
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

/// The library: (name, what it is, layers).
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

fn layers(name: &str) -> Option<Vec<Layer>> {
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
                freq: 1000.0,
                sustain: 0.0,
                decay: 0.06,
                lowpass: 900.0,
                gain: 0.25,
                ..L
            },
        ],
        "footstep" => vec![Layer {
            wave: Noise,
            freq: 1000.0,
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
                freq: 1000.0,
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
                freq: 1000.0,
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
            freq: 1000.0,
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

/// How a preset is varied.
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

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    /// ±`amount` around 1.
    fn around(&mut self, amount: f64) -> f64 {
        1.0 + (self.next() * 2.0 - 1.0) * amount
    }
}

/// A parsed `type sfx` description: an optional preset plus custom layers and
/// variation parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct SfxDesc {
    pub preset: Option<String>,
    pub layers: Vec<Layer>,
    pub variation: Variation,
}

/// Renders a preset: mono 16-bit PCM at [`RATE`]. `None` for an unknown name.
pub fn render(name: &str, v: Variation) -> Option<Vec<i16>> {
    let layers = layers(name)?;
    Some(render_layers(&layers, v))
}

/// Renders a parsed [`SfxDesc`]: mono 16-bit PCM at [`RATE`].
pub fn render_desc(desc: &SfxDesc) -> Vec<i16> {
    render_layers(&desc.layers, desc.variation)
}

/// Renders arbitrary oscillator layers with `v`: mono 16-bit PCM at [`RATE`].
pub fn render_layers(base_layers: &[Layer], v: Variation) -> Vec<i16> {
    let mut layers = base_layers.to_vec();
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ (u64::from(v.seed) << 17 | 1));
    let pitch = v.pitch.clamp(0.1, 10.0);
    let len_scale = v.length.clamp(0.1, 10.0);
    for l in &mut layers {
        l.freq *= pitch;
        if let Some(end) = l.freq_end {
            l.freq_end = Some(end * pitch);
        }
        l.fm_freq *= pitch;
        l.attack *= len_scale;
        l.sustain *= len_scale;
        l.decay *= len_scale;
        l.release *= len_scale;
        l.delay *= len_scale;
        l.arp_at *= len_scale;
        if v.seed != 0 {
            let f_jit = rng.around(0.06);
            l.freq *= f_jit;
            if let Some(end) = l.freq_end {
                l.freq_end = Some(end * f_jit);
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
            l.slide = (end / l.freq).log2() / total_len;
        }
    }
    let total = layers
        .iter()
        .map(|l| l.delay + l.attack + l.sustain + l.decay + l.release)
        .fold(0.0, f64::max)
        .min(MAX_DURATION);
    let frames = ((total + 0.01) * f64::from(RATE)).ceil() as usize;
    let mut out = vec![0.0f64; frames];
    let dt = 1.0 / f64::from(RATE);
    for (k, l) in layers.iter().enumerate() {
        let mut noise = Rng(0x2545_f491_4f6c_dd1d ^ (k as u64 + 1) ^ (u64::from(v.seed) << 32));
        let mut phase = 0.0f64;
        let mut fm_phase = 0.0f64;
        let mut held = 0.0f64;
        let (mut lp, mut hp_in, mut hp) = (0.0f64, 0.0f64, 0.0f64);
        let (mut svf_ic1, mut svf_ic2) = (0.0f64, 0.0f64);
        let start = (l.delay * f64::from(RATE)) as usize;
        let len = l.attack + l.sustain + l.decay + l.release;
        let adsr_mode = l.release > 0.0 || (l.sustain_level - 1.0).abs() > 1e-12;
        let sustain_level = l.sustain_level.clamp(0.0, 1.0);
        let drive_norm = if l.drive > 0.0 {
            l.drive.tanh().max(1e-6)
        } else {
            1.0
        };
        for (i, o) in out.iter_mut().enumerate().skip(start) {
            let t = (i - start) as f64 * dt;
            if t > len {
                break;
            }
            let env = if !adsr_mode {
                if t < l.attack {
                    t / l.attack.max(1e-6)
                } else if t < l.attack + l.sustain {
                    1.0 + l.punch * (1.0 - (t - l.attack) / l.sustain.max(1e-6))
                } else {
                    (1.0 - (t - l.attack - l.sustain) / l.decay.max(1e-6))
                        .max(0.0)
                        .powi(2)
                }
            } else if t < l.attack {
                t / l.attack.max(1e-6)
            } else if t < l.attack + l.decay {
                let u = (t - l.attack) / l.decay.max(1e-6);
                1.0 + (sustain_level - 1.0) * u
            } else if t < l.attack + l.decay + l.sustain {
                sustain_level + l.punch * (1.0 - (t - l.attack - l.decay) / l.sustain.max(1e-6))
            } else {
                sustain_level
                    * (1.0 - (t - l.attack - l.decay - l.sustain) / l.release.max(1e-6))
                        .max(0.0)
                        .powi(2)
            };
            let mut f = l.freq * (l.slide * t).exp2();
            if l.arp > 0.0 && t >= l.arp_at {
                f *= l.arp;
            }
            f *= 1.0 + l.vib_depth * (TAU * l.vib_rate * t).sin();
            let f = f.clamp(20.0, 20_000.0);
            let prev = phase;
            phase = (phase + f * dt).fract();
            let mod_phase = if l.fm_freq > 0.0 && l.fm_index != 0.0 {
                fm_phase = (fm_phase + l.fm_freq.clamp(1.0, 20_000.0) * dt).fract();
                (phase + (l.fm_index / TAU) * (TAU * fm_phase).sin()).rem_euclid(1.0)
            } else {
                phase
            };
            let s = match l.wave {
                Wave::Square => {
                    if mod_phase < l.duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Saw => mod_phase * 2.0 - 1.0,
                Wave::Sine => (TAU * mod_phase).sin(),
                Wave::Triangle => 1.0 - 4.0 * (mod_phase - 0.5).abs(),
                Wave::Noise => {
                    // A new random value every cycle of `freq` (sample-and-hold).
                    if phase < prev || i == start {
                        held = noise.next() * 2.0 - 1.0;
                    }
                    if l.freq >= 1000.0 {
                        noise.next() * 2.0 - 1.0
                    } else {
                        held
                    }
                }
            };
            let mut x = s;
            if l.drive > 0.0 {
                x = (x * l.drive).tanh() / drive_norm;
            }
            if l.lowpass > 0.0 {
                let c = (l.lowpass * (l.lp_slide * t).exp2()).clamp(20.0, 20_000.0);
                if l.q > 0.0 {
                    // Cytomic TPT 2-pole SVF (unconditionally stable up to Nyquist).
                    let g = (std::f64::consts::PI * c * dt).tan();
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
                    let a = 1.0 - (-TAU * c * dt).exp();
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
            *o += x * env * l.gain;
        }
    }
    let gain = v.volume.clamp(0.0, 4.0);
    out.iter()
        .map(|s| ((s * gain).clamp(-1.0, 1.0) * 32767.0).round() as i16)
        .collect()
}

/// A preset's `.ogt` text.
pub fn preset_ogt(name: &str, v: Variation) -> String {
    format!(
        "type sfx\nversion 1\n---\n# {}: og.load.sound(world, \"<this file>\") plays it.\npreset \"{name}\"\nseed {}\npitch {}\nlength {}\nvolume {}\n",
        PRESETS.iter().find(|p| p.0 == name).map_or("", |p| p.1),
        v.seed,
        v.pitch,
        v.length,
        v.volume
    )
}

fn finite_num(e: &Entry) -> Result<f64, String> {
    let n = e
        .values
        .first()
        .and_then(|x| x.as_number())
        .map(|n| n.as_f64())
        .ok_or_else(|| format!("line {}: '{}' takes a number", e.line, e.key))?;
    if !n.is_finite() {
        return Err(format!(
            "line {}: '{}' must be a finite number",
            e.line, e.key
        ));
    }
    Ok(n)
}

fn non_neg(e: &Entry) -> Result<f64, String> {
    let n = finite_num(e)?;
    if n < 0.0 {
        return Err(format!("line {}: '{}' cannot be negative", e.line, e.key));
    }
    Ok(n)
}

fn parse_layer(block_entry: &Entry) -> Result<Layer, String> {
    let mut layer = L;
    if let Some(w) = block_entry
        .values
        .first()
        .and_then(|x| x.as_str().or(x.as_word()))
    {
        layer.wave = Wave::parse(w).ok_or_else(|| {
            format!(
                "line {}: unknown wave '{w}': sine, square, saw, triangle or noise",
                block_entry.line
            )
        })?;
    }
    let body = block_entry.block.as_ref().ok_or_else(|| {
        format!(
            "line {}: '{}' needs a `{{ ... }}` block",
            block_entry.line, block_entry.key
        )
    })?;
    for e in &body.entries {
        match e.key.as_str() {
            "wave" => {
                let w = e
                    .values
                    .first()
                    .and_then(|x| x.as_str().or(x.as_word()))
                    .ok_or_else(|| format!("line {}: 'wave' takes a waveform name", e.line))?;
                layer.wave = Wave::parse(w).ok_or_else(|| {
                    format!(
                        "line {}: unknown wave '{w}': sine, square, saw, triangle or noise",
                        e.line
                    )
                })?;
            }
            "freq" => {
                let f = finite_num(e)?;
                if f <= 0.0 {
                    return Err(format!("line {}: 'freq' must be greater than 0", e.line));
                }
                layer.freq = f;
            }
            "freq_end" | "freqEnd" => {
                let f = finite_num(e)?;
                if f <= 0.0 {
                    return Err(format!(
                        "line {}: '{}' must be greater than 0",
                        e.line, e.key
                    ));
                }
                layer.freq_end = Some(f);
            }
            "slide" => layer.slide = finite_num(e)?,
            "arp" => layer.arp = non_neg(e)?,
            "arp_at" | "arpAt" => layer.arp_at = non_neg(e)?,
            "vib_depth" | "vibDepth" => layer.vib_depth = non_neg(e)?,
            "vib_rate" | "vibRate" => layer.vib_rate = non_neg(e)?,
            "attack" => layer.attack = non_neg(e)?,
            "sustain" => layer.sustain = non_neg(e)?,
            "decay" => layer.decay = non_neg(e)?,
            "sustain_level" | "sustainLevel" => {
                let s = non_neg(e)?;
                if s > 1.0 {
                    return Err(format!("line {}: '{}' must be in 0..1", e.line, e.key));
                }
                layer.sustain_level = s;
            }
            "release" => layer.release = non_neg(e)?,
            "punch" => layer.punch = non_neg(e)?,
            "duty" => {
                let d = finite_num(e)?;
                if !(0.01..=0.99).contains(&d) {
                    return Err(format!("line {}: 'duty' must be in 0.01..0.99", e.line));
                }
                layer.duty = d;
            }
            "lowpass" => layer.lowpass = non_neg(e)?,
            "lp_slide" | "lpSlide" => layer.lp_slide = finite_num(e)?,
            "q" | "resonance" => layer.q = non_neg(e)?,
            "highpass" => layer.highpass = non_neg(e)?,
            "fm_freq" | "fmFreq" => layer.fm_freq = non_neg(e)?,
            "fm_index" | "fmIndex" => layer.fm_index = finite_num(e)?,
            "drive" => layer.drive = non_neg(e)?,
            "gain" => layer.gain = non_neg(e)?,
            "delay" => layer.delay = non_neg(e)?,
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a layer entry: wave, freq, freq_end, slide, arp, arp_at, vib_depth, vib_rate, attack, sustain, decay, sustain_level, release, punch, duty, lowpass, lp_slide, q, highpass, fm_freq, fm_index, drive, gain, delay",
                    e.line
                ));
            }
        }
    }
    Ok(layer)
}

/// Reads a `type sfx` document: an optional preset, custom synth layers, and
/// variation settings.
pub fn parse_desc(text: &str) -> Result<SfxDesc, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != SFX_TYPE {
        return Err(format!(
            "a sound preset is `type {SFX_TYPE}`, not `{}`",
            doc.ty
        ));
    }
    let mut v = Variation::default();
    let mut preset = None;
    let mut custom_layers = Vec::new();
    for e in &doc.body.entries {
        match e.key.as_str() {
            "preset" => {
                preset = e
                    .values
                    .first()
                    .and_then(|x| x.as_str().or(x.as_word()))
                    .map(str::to_string);
            }
            "seed" => v.seed = non_neg(e)? as u32,
            "pitch" => v.pitch = finite_num(e)?,
            "length" => v.length = finite_num(e)?,
            "volume" => v.volume = non_neg(e)?,
            "layer" | "synth" => custom_layers.push(parse_layer(e)?),
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a sound preset entry: preset, layer, synth, seed, pitch, length or volume",
                    e.line
                ));
            }
        }
    }
    let mut all_layers = Vec::new();
    if let Some(ref name) = preset {
        let base = layers(name).ok_or_else(|| {
            let names: Vec<&str> = PRESETS.iter().map(|p| p.0).collect();
            format!("no preset '{name}': {}", names.join(", "))
        })?;
        all_layers.extend(base);
    }
    all_layers.extend(custom_layers);
    if all_layers.is_empty() {
        return Err(
            "a `type sfx` file needs `preset \"<name>\"` or at least one `layer { ... }` block"
                .into(),
        );
    }
    let len_scale = v.length.clamp(0.1, 10.0);
    let max_end = all_layers
        .iter()
        .map(|l| (l.delay + l.attack + l.sustain + l.decay + l.release) * len_scale)
        .fold(0.0, f64::max);
    if max_end > MAX_DURATION {
        return Err(format!(
            "synthesised sound is {max_end:.2} s, above the {MAX_DURATION:.0} s limit"
        ));
    }
    Ok(SfxDesc {
        preset,
        layers: all_layers,
        variation: v,
    })
}

/// Reads a preset-only `type sfx` document (kept for compatibility).
pub fn parse(text: &str) -> Result<(String, Variation), String> {
    let desc = parse_desc(text)?;
    let name = desc
        .preset
        .ok_or("a sound preset needs `preset \"<name>\"`")?;
    Ok((name, desc.variation))
}

/// Cooks a `type sfx` .ogt into a sound blob.
pub fn cook(bytes: &[u8], stamp: u64) -> Result<(Vec<u8>, String), CookError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CookError("a sound preset is text (UTF-8)".into()))?;
    let desc = parse_desc(text).map_err(CookError)?;
    let samples = render_desc(&desc);
    let qoa = qoa::encode(&samples, 1, RATE).map_err(CookError)?;
    let frames = samples.len() as u32;
    let meta = SoundMeta {
        sample_rate: RATE,
        channels: 1,
        frames,
        ..SoundMeta::default()
    };
    let label = match &desc.preset {
        Some(name) if desc.layers.len() == layers(name).map_or(0, |l| l.len()) => {
            format!("preset {name}")
        }
        Some(name) => format!("preset {name} + {} layer(s)", desc.layers.len()),
        None => format!("synth ({} layer(s))", desc.layers.len()),
    };
    let note = format!(
        "{label} (seed {}), {:.2} s synthesised to QOA",
        desc.variation.seed,
        f64::from(frames) / f64::from(RATE)
    );
    let mut w = BlobWriter::new(
        format::Sound::type_hash(),
        format::Sound::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<SoundMeta>())
        .raw("qoa", qoa, 16);
    Ok((w.finish(), note))
}

/// A sound preset (`type sfx`): synthesised, then cooked as a sound.
pub(crate) static SFX: KindDef = KindDef {
    name: "sound",
    extensions: &[],
    types: &[SFX_TYPE],
    by_content: true,
    streams: true,
    plan: plan_sfx,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&SFX];

fn plan_sfx<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // A preset's stamp moves with the synthesiser.
    let st = stamp(&[b"sfx", SYNTH_VERSION.as_bytes(), &hash]);
    outs.push((s.rel.clone(), format::Sound::NAME, st, Box::new(SfxJob(s))));
}

/// A sound preset's one output: its synthesised sound.
struct SfxJob<'a>(&'a Source);

impl Job for SfxJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook(&self.0.bytes, p.stamp).map(|(b, n)| (b, vec![n]))
    }

    fn action(&self) -> Action<'_> {
        let s = self.0;
        Action::Input(
            s.bytes.as_slice(),
            format!(
                "sound {:?} {} {:?}",
                None::<(u32, u32)>,
                ext(&s.rel),
                s.settings
            ),
            false,
        )
    }
}
