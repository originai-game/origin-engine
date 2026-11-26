//! Typed PCM source data enters the ordinary sound import/encoding path.
use super::{SOUND, Source};
use crate::cook::{assets_root, generator};
use og_core::ogt::Document;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pcm {
    rate: u32,
    channels: u32,
    samples: Vec<f32>,
}

fn make(s: &Source) -> Result<Option<Vec<u8>>, String> {
    if s.kind != &SOUND || !s.rel.ends_with(".ogt") || s.error.is_some() {
        return Ok(None);
    }
    let doc = Document::parse(&String::from_utf8_lossy(&s.bytes)).map_err(|e| e.to_string())?;
    if doc.version != 1 || doc.body.entries.len() != 1 {
        return Err("type sound version 1 contains one generate block; use type sfx for oscillator synthesis".into());
    }
    let e = &doc.body.entries[0];
    if e.key != "generate" || !e.values.is_empty() {
        return Err("type sound needs generate { script ... }".into());
    }
    let spec = generator::Spec::standalone(e.block.as_ref().ok_or("generate is a block")?)?;
    let program = generator::Program::prepare(&assets_root(s), &s.rel, &spec)?;
    let pcm: Pcm = serde_json::from_value(program.run()?).map_err(|e| e.to_string())?;
    if !(8000..=192000).contains(&pcm.rate)
        || !(1..=2).contains(&pcm.channels)
        || pcm.samples.is_empty()
        || !pcm.samples.len().is_multiple_of(pcm.channels as usize)
        || pcm.samples.iter().any(|v| !v.is_finite())
    {
        return Err("PCM source needs rate 8000..192000, channels 1 or 2 and at least one complete frame of finite samples".into());
    }
    // This is an in-memory import, like model text becoming GLB. Float WAV
    // retains the input; normal decoding owns clamping and PCM conversion,
    // then the same QOA/Opus/streaming pipeline as an authored sound runs.
    let mut bytes = Vec::new();
    let spec = hound::WavSpec {
        channels: pcm.channels as u16,
        sample_rate: pcm.rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer =
        hound::WavWriter::new(std::io::Cursor::new(&mut bytes), spec).map_err(|e| e.to_string())?;
    for sample in pcm.samples {
        writer.write_sample(sample).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(Some(bytes))
}

pub(crate) fn expand(sources: &mut [Source]) {
    for s in sources {
        match make(s) {
            Ok(Some(bytes)) => s.bytes = bytes,
            Ok(None) => {}
            Err(e) => s.error = Some(e),
        }
    }
}
