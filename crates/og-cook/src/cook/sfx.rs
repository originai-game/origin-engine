//! Procedural sound presets (`type sfx` .ogt): a library of game sounds
//! synthesised at cook time into ordinary sound blobs, so a game has coins,
//! jumps, hits and UI clicks without a single audio file. Each preset is a
//! few layers of an oscillator with a pitch slide, vibrato, an arpeggio
//! jump, an attack/sustain/decay envelope with punch and a resonance-free
//! low/high-pass; `seed` varies it slightly, `pitch`, `length` and `volume`
//! scale it. Everything is f64 arithmetic and a seeded xorshift, so a preset
//! renders the same samples on every machine.
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

use super::CookError;
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;
use og_core::ogt::Document;
use std::f64::consts::TAU;

/// The `.ogt` type.
pub const SFX_TYPE: &str = "sfx";

/// Bumped when the synthesiser or a preset changes, so cooked presets re-cook.
pub const SYNTH_VERSION: &str = "1";

/// The rate presets render at.
pub const RATE: u32 = 48_000;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Wave {
    Square,
    Saw,
    Sine,
    Triangle,
    Noise,
}

/// One oscillator layer.
#[derive(Clone, Copy, Debug)]
struct Layer {
    wave: Wave,
    /// Start frequency, Hz.
    freq: f64,
    /// Pitch slide, octaves per second.
    slide: f64,
    /// At `arp_at` seconds the frequency is multiplied by `arp` (0: never).
    arp_at: f64,
    arp: f64,
    vib_depth: f64,
    vib_rate: f64,
    /// Envelope, seconds.
    attack: f64,
    sustain: f64,
    decay: f64,
    /// Extra level during the sustain, fading to none (0..1).
    punch: f64,
    duty: f64,
    /// One-pole low-pass, Hz (0: none), and its slide in octaves per second.
    lowpass: f64,
    lp_slide: f64,
    highpass: f64,
    gain: f64,
    /// Starts this many seconds in.
    delay: f64,
}

const L: Layer = Layer {
    wave: Wave::Square,
    freq: 440.0,
    slide: 0.0,
    arp_at: 0.0,
    arp: 0.0,
    vib_depth: 0.0,
    vib_rate: 0.0,
    attack: 0.002,
    sustain: 0.05,
    decay: 0.15,
    punch: 0.0,
    duty: 0.5,
    lowpass: 0.0,
    lp_slide: 0.0,
    highpass: 0.0,
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

/// Renders a preset: mono 16-bit PCM at [`RATE`]. `None` for an unknown name.
pub fn render(name: &str, v: Variation) -> Option<Vec<i16>> {
    let mut layers = layers(name)?;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ (u64::from(v.seed) << 17 | 1));
    for l in &mut layers {
        l.freq *= v.pitch.clamp(0.1, 10.0);
        let len = v.length.clamp(0.1, 10.0);
        l.attack *= len;
        l.sustain *= len;
        l.decay *= len;
        l.delay *= len;
        l.arp_at *= len;
        if v.seed != 0 {
            l.freq *= rng.around(0.06);
            l.slide *= rng.around(0.2);
            l.decay *= rng.around(0.15);
            l.duty = (l.duty * rng.around(0.2)).clamp(0.05, 0.95);
        }
    }
    let total = layers
        .iter()
        .map(|l| l.delay + l.attack + l.sustain + l.decay)
        .fold(0.0, f64::max);
    let frames = ((total + 0.01) * f64::from(RATE)).ceil() as usize;
    let mut out = vec![0.0f64; frames];
    let dt = 1.0 / f64::from(RATE);
    for (k, l) in layers.iter().enumerate() {
        let mut noise = Rng(0x2545_f491_4f6c_dd1d ^ (k as u64 + 1) ^ (u64::from(v.seed) << 32));
        let mut phase = 0.0f64;
        let mut held = 0.0f64;
        let (mut lp, mut hp_in, mut hp) = (0.0f64, 0.0f64, 0.0f64);
        let start = (l.delay * f64::from(RATE)) as usize;
        let len = l.attack + l.sustain + l.decay;
        for (i, o) in out.iter_mut().enumerate().skip(start) {
            let t = (i - start) as f64 * dt;
            if t > len {
                break;
            }
            let env = if t < l.attack {
                t / l.attack.max(1e-6)
            } else if t < l.attack + l.sustain {
                1.0 + l.punch * (1.0 - (t - l.attack) / l.sustain.max(1e-6))
            } else {
                (1.0 - (t - l.attack - l.sustain) / l.decay.max(1e-6))
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
            let s = match l.wave {
                Wave::Square => {
                    if phase < l.duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Saw => phase * 2.0 - 1.0,
                Wave::Sine => (TAU * phase).sin(),
                Wave::Triangle => 1.0 - 4.0 * (phase - 0.5).abs(),
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
            if l.lowpass > 0.0 {
                let c = (l.lowpass * (l.lp_slide * t).exp2()).clamp(20.0, 20_000.0);
                let a = 1.0 - (-TAU * c * dt).exp();
                lp += a * (x - lp);
                x = lp;
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
    Some(
        out.iter()
            .map(|s| ((s * gain).clamp(-1.0, 1.0) * 32767.0).round() as i16)
            .collect(),
    )
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

/// Reads a `type sfx` document: the preset and its variation.
pub fn parse(text: &str) -> Result<(String, Variation), String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != SFX_TYPE {
        return Err(format!(
            "a sound preset is `type {SFX_TYPE}`, not `{}`",
            doc.ty
        ));
    }
    let mut v = Variation::default();
    let mut name = None;
    for e in &doc.body.entries {
        let num = || {
            e.values
                .first()
                .and_then(|x| x.as_number())
                .map(|n| n.as_f64())
                .ok_or_else(|| format!("line {}: '{}' takes a number", e.line, e.key))
        };
        match e.key.as_str() {
            "preset" => {
                name = e
                    .values
                    .first()
                    .and_then(|x| x.as_str().or(x.as_word()))
                    .map(str::to_string);
            }
            "seed" => v.seed = num()?.max(0.0) as u32,
            "pitch" => v.pitch = num()?,
            "length" => v.length = num()?,
            "volume" => v.volume = num()?,
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a sound preset entry: preset, seed, pitch, length or volume",
                    e.line
                ));
            }
        }
    }
    let name = name.ok_or("a sound preset needs `preset \"<name>\"`")?;
    if layers(&name).is_none() {
        let names: Vec<&str> = PRESETS.iter().map(|p| p.0).collect();
        return Err(format!("no preset '{name}': {}", names.join(", ")));
    }
    Ok((name, v))
}

/// Cooks a `type sfx` .ogt into a sound blob.
pub fn cook(bytes: &[u8], stamp: u64) -> Result<(Vec<u8>, String), CookError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CookError("a sound preset is text (UTF-8)".into()))?;
    let (name, v) = parse(text).map_err(CookError)?;
    let samples = render(&name, v).expect("parse checked the name");
    let qoa = qoa::encode(&samples, 1, RATE).map_err(CookError)?;
    let frames = samples.len() as u32;
    let meta = SoundMeta {
        sample_rate: RATE,
        channels: 1,
        frames,
    };
    let note = format!(
        "preset {name} (seed {}), {:.2} s synthesised to QOA",
        v.seed,
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
