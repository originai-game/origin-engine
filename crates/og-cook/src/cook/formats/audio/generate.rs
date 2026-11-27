//! Typed PCM source data enters the ordinary sound import/encoding path.
use crate::cook::{generator, source_schemas};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, og_core::reflect::source::schemars::JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Pcm {
    /// Sample rate in hertz; no resampling before the codec import.
    #[schemars(range(min = 8000, max = 192000), extend("x-og-unit" = "Hz"))]
    pub rate: u32,
    /// One mono channel or interleaved stereo.
    #[schemars(range(min = 1, max = 2))]
    pub channels: u32,
    /// Finite PCM samples, at least one complete interleaved frame.
    #[schemars(length(min = 1))]
    pub samples: Vec<f32>,
}

impl Pcm {
    pub fn schema() -> &'static og_core::reflect::source::InputSchema<Self> {
        static SCHEMA: std::sync::OnceLock<og_core::reflect::source::InputSchema<Pcm>> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| og_core::reflect::source::InputSchema::new().expect("PCM schema"))
    }

    pub(crate) fn shape(&self) -> Result<(u32, u32, u64), String> {
        if !(8000..=192000).contains(&self.rate)
            || !(1..=2).contains(&self.channels)
            || self.samples.is_empty()
            || !self.samples.len().is_multiple_of(self.channels as usize)
            || self.samples.iter().any(|v| !v.is_finite())
        {
            return Err("PCM source needs rate 8000..192000, channels 1 or 2 and at least one complete frame of finite samples".into());
        }
        Ok((
            self.rate,
            self.channels,
            (self.samples.len() / self.channels as usize) as u64,
        ))
    }

    pub(crate) fn into_pcm(self) -> Result<super::Pcm, String> {
        self.shape()?;
        Ok(super::Pcm {
            rate: self.rate,
            channels: self.channels,
            samples: self.samples.into_iter().map(super::quantize).collect(),
            from: "typed float PCM".into(),
        })
    }
}

pub(crate) fn derived(data: &generator::value::Data) -> Result<BTreeMap<String, String>, String> {
    let file: source_schemas::File =
        serde_json::from_value(data.root().into_json()).map_err(|e| e.to_string())?;
    let pcm: Pcm = serde_json::from_value(file.value).map_err(|e| e.to_string())?;
    let (rate, channels, frames) = pcm.shape()?;
    Ok(super::SoundSettings::default()
        .chunking(rate, channels, frames)
        .into_iter()
        .flat_map(|(_, count)| 1..count)
        .map(|n| (format!("chunk{n}"), "sound-chunk".into()))
        .collect())
}
