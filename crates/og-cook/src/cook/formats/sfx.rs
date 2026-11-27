//! Typed sound declarations feed the shared deterministic synthesizer directly.
//! The output remains an ordinary mono QOA sound; no author text is generated.

use super::{Action, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::cook::source_schemas::procedural::{SampleInput, SfxInput, SfxPreset, SynthLayer};
use crate::format::{self, *};
use crate::qoa;
use og_blob::BlobWriter;

pub use og_audio::synth::{
    L, Layer, MAX_DURATION, PRESETS, RATE, SfxDesc, Variation, Wave, preset_layers as layers,
    render, render_desc, render_layers, render_layers_f32,
};

/// Bumped when the synthesiser or a preset changes, so cooked presets re-cook.
pub const SYNTH_VERSION: &str = "3";

/// A standalone declaration for `og sfx`, using the same factory as hand authors.
pub fn preset_ts(name: &str, v: Variation) -> String {
    format!(
        "import {{ file }} from '@origingame/engine/author';\n\nexport default file.sfx({{\n  preset: {name:?},\n  seed: {},\n  pitch: {},\n  length: {},\n  volume: {},\n}});\n",
        v.seed, v.pitch, v.length, v.volume
    )
}

fn sample(
    input: SampleInput,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<og_audio::synth::Sample, String> {
    let (samples, rate, reference, looping) = match input {
        SampleInput::File {
            source,
            reference,
            looping,
        } => {
            let pcm = resolve(&source)?;
            let samples = pcm
                .samples
                .chunks_exact(pcm.channels as usize)
                .map(|frame| {
                    frame.iter().map(|&v| f32::from(v) / 32768.0).sum::<f32>() / pcm.channels as f32
                })
                .collect::<Vec<_>>();
            (samples, pcm.rate, reference, looping)
        }
        SampleInput::Pcm {
            samples,
            rate,
            reference,
            looping,
        } => (
            samples.into_iter().map(|v| v.0).collect(),
            rate,
            reference,
            looping,
        ),
    };
    let reference = reference.unwrap_or(440.0);
    if samples.is_empty()
        || samples.iter().any(|v| !v.is_finite() || v.abs() > 1.0)
        || !(8000..=192000).contains(&rate)
        || !reference.is_finite()
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
        looping: looping.unwrap_or(false),
    })
}

fn layer(
    input: SynthLayer,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<Layer, String> {
    use crate::cook::source_schemas::procedural::Wave as InputWave;
    let layer = Layer {
        wave: match input.wave {
            None => L.wave,
            Some(InputWave::Sine) => Wave::Sine,
            Some(InputWave::Square) => Wave::Square,
            Some(InputWave::Saw) => Wave::Saw,
            Some(InputWave::Triangle) => Wave::Triangle,
            Some(InputWave::Noise) => Wave::Noise,
        },
        sample: input.sample.map(|s| sample(s, resolve)).transpose()?,
        freq: input.freq.unwrap_or(L.freq),
        freq_end: input.freq_end,
        slide: input.slide.unwrap_or(L.slide),
        arp: input.arp.unwrap_or(L.arp),
        arp_at: input.arp_at.unwrap_or(L.arp_at),
        vib_depth: input.vib_depth.unwrap_or(L.vib_depth),
        vib_rate: input.vib_rate.unwrap_or(L.vib_rate),
        attack: input.attack.unwrap_or(L.attack),
        sustain: input.sustain.unwrap_or(L.sustain),
        decay: input.decay.unwrap_or(L.decay),
        sustain_level: input.sustain_level.unwrap_or(L.sustain_level),
        release: input.release.unwrap_or(L.release),
        punch: input.punch.unwrap_or(L.punch),
        duty: input.duty.unwrap_or(L.duty),
        lowpass: input.lowpass.unwrap_or(L.lowpass),
        lp_slide: input.lp_slide.unwrap_or(L.lp_slide),
        q: input.q.unwrap_or(L.q),
        highpass: input.highpass.unwrap_or(L.highpass),
        fm_freq: input.fm_freq.unwrap_or(L.fm_freq),
        fm_index: input.fm_index.unwrap_or(L.fm_index),
        drive: input.drive.unwrap_or(L.drive),
        gain: input.gain.unwrap_or(L.gain),
        delay: input.delay.unwrap_or(L.delay),
    };
    if layer.sample.is_some() && (layer.fm_freq != 0.0 || layer.fm_index != 0.0) {
        return Err("sample layers reject FM; it is oscillator-only".into());
    }
    Ok(layer)
}

fn description(
    input: SfxInput,
    resolve: &mut dyn FnMut(&str) -> Result<super::audio::Pcm, String>,
) -> Result<SfxDesc, String> {
    let preset = input.preset.map(|preset| {
        match preset {
            SfxPreset::Coin => "coin",
            SfxPreset::Pickup => "pickup",
            SfxPreset::Powerup => "powerup",
            SfxPreset::Jump => "jump",
            SfxPreset::Land => "land",
            SfxPreset::Footstep => "footstep",
            SfxPreset::Hit => "hit",
            SfxPreset::Hurt => "hurt",
            SfxPreset::Explosion => "explosion",
            SfxPreset::Laser => "laser",
            SfxPreset::Whoosh => "whoosh",
            SfxPreset::Door => "door",
            SfxPreset::Death => "death",
            SfxPreset::Click => "click",
            SfxPreset::Select => "select",
            SfxPreset::Back => "back",
            SfxPreset::Error => "error",
            SfxPreset::Levelup => "levelup",
        }
        .to_string()
    });
    let defaults = Variation::default();
    let variation = Variation {
        seed: input.seed.unwrap_or(defaults.seed),
        pitch: input.pitch.unwrap_or(defaults.pitch),
        length: input.length.unwrap_or(defaults.length),
        volume: input.volume.unwrap_or(defaults.volume),
    };
    let mut all_layers = match &preset {
        Some(name) => layers(name).ok_or_else(|| format!("no sound preset '{name}'"))?,
        None => Vec::new(),
    };
    for input in input.layers.unwrap_or_default() {
        all_layers.push(layer(input, resolve)?);
    }
    if all_layers.is_empty() {
        return Err("file.sfx needs a preset or at least one layer".into());
    }
    let len_scale = variation.length.clamp(0.1, 10.0);
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
        variation,
    })
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

/// The sole SFX author entry is a checked TS declaration.
pub(crate) static SFX: KindDef = KindDef {
    name: "sound",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: true,
    plan: plan_sfx,
};
pub(crate) static KINDS: &[&KindDef] = &[&SFX];

fn plan_sfx<'a>(
    _: &Cx<'a>,
    _: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // Capture each recording before cache lookup, and cook those exact bytes.
    let mut inputs = Vec::new();
    let desc = s.input::<SfxInput>().and_then(|input| {
        description(input, &mut |name| {
            if !(name.starts_with("assets/") || name.starts_with("sources/")) {
                return Err(
                    "sample.source must be inside the owning module's assets/ or sources/".into(),
                );
            }
            let path = crate::cook::source_path::checked(&super::assets_root(s), name)
                .map_err(|e| format!("sample source '{name}': {e}"))?;
            let bytes = std::fs::read(path).map_err(|e| format!("{name}: {e}"))?;
            inputs.push(bytes.clone());
            super::audio::decode(name, &bytes)
        })
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

struct SfxJob<'a>(&'a Source, Result<SfxDesc, String>);
impl Job for SfxJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_desc(self.1.as_ref().map_err(|e| CookError(e.clone()))?, p.stamp)
            .map(|(b, n)| (b, vec![n]))
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.0.bytes, false)
    }
}
