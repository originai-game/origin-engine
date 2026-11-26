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

use super::{Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;
use og_core::ogt::{Document, Entry};

pub use og_audio::synth::{
    L, Layer, MAX_DURATION, PRESETS, RATE, SfxDesc, Variation, Wave, preset_layers as layers,
    render, render_desc, render_layers, render_layers_f32,
};

/// The `.ogt` type.
pub const SFX_TYPE: &str = "sfx";

/// Bumped when the synthesiser or a preset changes, so cooked presets re-cook.
pub const SYNTH_VERSION: &str = "3";

macro_rules! lines {
    ($($key:literal, $args:literal, $default:literal, $doc:literal);* $(;)?) => {
        &[$(crate::grammar::Line::new($key, $args, $default, $doc, "")),*]
    };
}

pub const LINES: &[crate::grammar::Line] = lines![
    "preset", "\"<name>\"", "", "Optional existing sound preset; custom layers need no preset.";
    "seed", "<u32>", "0", "Seeded variation.";
    "pitch", "<number>", "1", "Frequency multiplier.";
    "length", "<number>", "1", "Duration multiplier, clamped to 0.1..10.";
    "volume", "<number >= 0>", "1", "Master gain.";
    "layer", "[wave] { ... }", "", "Repeat oscillator layers; at least one layer or preset required.";
    "synth", "[wave] { ... }", "", "Accepted oscillator-layer spelling."
];

pub const LAYER_LINES: &[crate::grammar::Line] = lines![
    "sample", "{ ... }", "", "Recording source instead of the oscillator; envelope and filters still apply.";
    "wave", "sine | square | saw | triangle | noise", "square", "Oscillator waveform; overrides a waveform on the layer line.";
    "freq", "<Hz > 0>", "440", "Starting frequency.";
    "freq_end", "<Hz > 0>", "", "Optional ending frequency.";
    "freqEnd", "<Hz > 0>", "", "Accepted freq_end spelling.";
    "slide", "<number>", "0", "Pitch slide.";
    "arp", "<number >= 0>", "0", "Arpeggio interval.";
    "arp_at", "<seconds >= 0>", "0", "Arpeggio time.";
    "arpAt", "<seconds >= 0>", "0", "Accepted arp_at spelling.";
    "vib_depth", "<number >= 0>", "0", "Vibrato depth.";
    "vibDepth", "<number >= 0>", "0", "Accepted vib_depth spelling.";
    "vib_rate", "<Hz >= 0>", "0", "Vibrato rate.";
    "vibRate", "<Hz >= 0>", "0", "Accepted vib_rate spelling.";
    "attack", "<seconds >= 0>", "0.002", "Envelope attack.";
    "sustain", "<seconds >= 0>", "0.05", "Envelope sustain duration.";
    "decay", "<seconds >= 0>", "0.15", "Envelope decay.";
    "sustain_level", "<0..1>", "1", "Sustain level.";
    "sustainLevel", "<0..1>", "1", "Accepted sustain_level spelling.";
    "release", "<seconds >= 0>", "0", "Envelope release.";
    "punch", "<number >= 0>", "0", "Envelope punch.";
    "duty", "<0.01..0.99>", "0.5", "Square-wave duty cycle.";
    "lowpass", "<Hz >= 0>", "0", "Low-pass cutoff; zero disables.";
    "lp_slide", "<number>", "0", "Low-pass cutoff slide.";
    "lpSlide", "<number>", "0", "Accepted lp_slide spelling.";
    "q", "<number >= 0>", "0", "Filter resonance.";
    "highpass", "<Hz >= 0>", "0", "High-pass cutoff; zero disables.";
    "fm_freq", "<Hz >= 0>", "0", "FM modulation frequency.";
    "fmFreq", "<Hz >= 0>", "0", "Accepted fm_freq spelling.";
    "fm_index", "<number>", "0", "FM modulation index.";
    "fmIndex", "<number>", "0", "Accepted fm_index spelling.";
    "drive", "<number >= 0>", "0", "Soft-clip drive.";
    "gain", "<number >= 0>", "0.5", "Layer gain.";
    "delay", "<seconds >= 0>", "0", "Layer start delay."
];

pub const SAMPLE_LINES: &[crate::grammar::Line] = lines![
    "source", "\"<audio file>\"", "", "Audio file relative to this SFX file; mono or stereo (stereo averages to mono). Exclusive with samples/rate.";
    "samples", "<finite mono PCM -1..1...>", "", "Explicit mono samples instead of a source file.";
    "rate", "<integer 8000..192000>", "", "Sample rate of explicit PCM; the file's rate is retained for source.";
    "reference", "<Hz > 0>", "440", "Layer frequency that plays the recording at original speed.";
    "loop", "true | false", "false", "Wrap at the recording's end; envelope duration remains explicit."
];

fn sample(
    e: &Entry,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<og_audio::synth::Sample, String> {
    let b = e
        .block
        .as_ref()
        .filter(|_| e.values.is_empty())
        .ok_or("sample takes a block")?;
    let mut seen = std::collections::BTreeSet::new();
    for e in &b.entries {
        if !SAMPLE_LINES.iter().any(|f| f.key == e.key) || !seen.insert(&e.key) {
            return Err(format!(
                "line {}: unknown or duplicate sample field '{}'",
                e.line, e.key
            ));
        }
    }
    let (samples, rate) = if let Some(source) = b.get("source") {
        if b.get("samples").is_some()
            || b.get("rate").is_some()
            || source.values.len() != 1
            || source.block.is_some()
        {
            return Err(
                "sample.source is exclusive with samples/rate and takes one audio path".into(),
            );
        }
        let pcm = resolve(source.string(0).map_err(|e| e.to_string())?)?;
        let samples = pcm
            .samples
            .chunks_exact(pcm.channels as usize)
            .map(|frame| {
                frame.iter().map(|&v| f32::from(v) / 32768.0).sum::<f32>() / pcm.channels as f32
            })
            .collect();
        (samples, pcm.rate)
    } else {
        let data = b
            .get("samples")
            .ok_or("sample needs source or samples and rate")?;
        let samples = data
            .values
            .iter()
            .map(|v| {
                v.as_number()
                    .map(|n| n.as_f64() as f32)
                    .ok_or_else(|| "sample.samples takes finite mono numbers".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let rate = finite_num(b.get("rate").ok_or("sample.samples needs rate")?)?;
        if rate.fract() != 0.0 || !(8000.0..=192000.0).contains(&rate) {
            return Err("sample.rate must be an integer in 8000..192000".into());
        }
        (samples, rate as u32)
    };
    let reference = b
        .get("reference")
        .map(finite_num)
        .transpose()?
        .unwrap_or(440.0);
    let looping = b
        .get("loop")
        .map(|e| {
            e.values
                .first()
                .and_then(|v| v.as_bool())
                .ok_or("sample.loop takes true or false")
        })
        .transpose()?
        .unwrap_or(false);
    if samples.is_empty()
        || samples.iter().any(|v| !v.is_finite() || v.abs() > 1.0)
        || !(8000..=192000).contains(&rate)
        || reference <= 0.0
    {
        return Err(
            "sample needs finite mono PCM in -1..1, rate 8000..192000 and positive reference"
                .into(),
        );
    }
    Ok(og_audio::synth::Sample {
        samples,
        rate,
        reference,
        looping,
    })
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

fn parse_layer(
    block_entry: &Entry,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<Layer, String> {
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
        if crate::grammar::find(LAYER_LINES, &e.key, &[]).is_none() {
            return Err(format!(
                "line {}: unknown sfx layer field '{}'",
                e.line, e.key
            ));
        }
        match e.key.as_str() {
            "sample" => {
                if layer.sample.is_some() {
                    return Err("a layer declares sample once".into());
                }
                layer.sample = Some(sample(e, resolve)?);
            }
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
    if layer.sample.is_some() && (layer.fm_freq != 0.0 || layer.fm_index != 0.0) {
        return Err("sample layers reject FM; it is oscillator-only".into());
    }
    Ok(layer)
}

/// Reads a `type sfx` document: an optional preset, custom synth layers, and
/// variation settings.
pub fn parse_desc(text: &str) -> Result<SfxDesc, String> {
    parse_desc_with(text, &mut |_| {
        Err("sample.source is resolved by cook; inline PCM needs samples and rate".into())
    })
}

fn parse_desc_with(
    text: &str,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<SfxDesc, String> {
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
        if crate::grammar::find(LINES, &e.key, &[]).is_none() {
            return Err(format!("line {}: unknown sfx field '{}'", e.line, e.key));
        }
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
            "layer" | "synth" => custom_layers.push(parse_layer(e, resolve)?),
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
    cook_desc(&desc, stamp)
}

fn cook_desc(desc: &SfxDesc, stamp: u64) -> Result<(Vec<u8>, String), CookError> {
    let samples = render_desc(desc);
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
    let mut inputs = Vec::new();
    let desc = parse_desc_with(&String::from_utf8_lossy(&s.bytes), &mut |name| {
        let path = s
            .path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(name)
            .canonicalize()
            .map_err(|e| format!("{name}: {e}"))?;
        let root = super::assets_root(s)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(root) {
            return Err(format!("{name}: sample source must stay under assets/"));
        }
        let bytes = std::fs::read(&path).map_err(|e| format!("{name}: {e}"))?;
        inputs.push(bytes.clone());
        super::audio::decode(name, &bytes)
    });
    let mut parts = vec![b"sfx".as_slice(), SYNTH_VERSION.as_bytes(), &hash];
    parts.extend(inputs.iter().map(Vec::as_slice));
    let st = stamp(&[Area::Audio], &parts);
    outs.push((
        s.rel.clone(),
        format::Sound::NAME,
        st,
        Box::new(SfxJob(s, desc)),
    ));
}

/// A sound preset's one output: its synthesised sound.
struct SfxJob<'a>(&'a Source, Result<SfxDesc, String>);

impl Job for SfxJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_desc(self.1.as_ref().map_err(|e| CookError(e.clone()))?, p.stamp)
            .map(|(b, n)| (b, vec![n]))
    }

    fn action(&self) -> Action<'_> {
        Action::Input(self.0.bytes.as_slice(), false)
    }
}
